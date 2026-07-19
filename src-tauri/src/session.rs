use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;

use crate::audio::{pcm16_le_bytes_to_i16, PlaybackStream};
use crate::microphone::MicrophoneCapture;
use crate::settings::get_api_key;
use crate::websocket::{self, RealtimeClient};

const MAX_RECONNECT_ATTEMPTS: u32 = 5;

struct SessionHandle {
    cancel: CancellationToken,
    task: tauri::async_runtime::JoinHandle<()>,
}

#[derive(Default)]
pub struct SessionManager {
    current: Mutex<Option<SessionHandle>>,
}

struct StartArgs {
    input_device_id: String,
    output_device_id: String,
    source_lang: String,
    target_lang: String,
}

static TRANSCRIPT_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_transcript_id() -> String {
    format!("t{}", TRANSCRIPT_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn emit_status(app: &AppHandle, status: &str, message: Option<String>) {
    let _ = app.emit(
        "connection-status",
        serde_json::json!({ "status": status, "message": message }),
    );
}

fn emit_error(app: &AppHandle, message: String, recoverable: bool) {
    let _ = app.emit(
        "session-error",
        serde_json::json!({ "message": message, "recoverable": recoverable }),
    );
}

fn emit_transcript(app: &AppHandle, role: &str, text: String, is_final: bool) {
    let _ = app.emit(
        "transcript-update",
        serde_json::json!({
            "id": next_transcript_id(),
            "role": role,
            "text": text,
            "isFinal": is_final,
            "timestamp": now_millis(),
        }),
    );
}

fn handle_server_event(app: &AppHandle, playback: &PlaybackStream, value: &Value) {
    let Some(event_type) = value.get("type").and_then(|t| t.as_str()) else {
        return;
    };

    match event_type {
        websocket::server_events::OUTPUT_AUDIO_DELTA => {
            if let Some(bytes) = websocket::decode_audio_delta(value) {
                playback.push_samples(&pcm16_le_bytes_to_i16(&bytes));
            }
        }
        websocket::server_events::OUTPUT_TRANSCRIPT_DELTA => {
            if let Some(text) = websocket::extract_transcript_delta(value) {
                emit_transcript(app, "translated", text, false);
            }
        }
        websocket::server_events::INPUT_TRANSCRIPT_DELTA => {
            if let Some(text) = websocket::extract_transcript_delta(value) {
                emit_transcript(app, "source", text, false);
            }
        }
        websocket::server_events::ERROR => {
            let message = value
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error from OpenAI Realtime API")
                .to_string();
            emit_error(app, message, true);
        }
        _ => {}
    }
}

async fn run_session_once(
    app: &AppHandle,
    args: &StartArgs,
    api_key: &str,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let mut client = RealtimeClient::connect(api_key).await?;
    client.send_session_update(&args.target_lang).await?;

    emit_status(app, "connected", None);

    let (pcm_tx, mut pcm_rx) = mpsc::unbounded_channel::<Vec<u8>>();

    let input_device_id = args.input_device_id.clone();
    let mic = tokio::task::spawn_blocking(move || MicrophoneCapture::start(&input_device_id, pcm_tx))
        .await
        .map_err(|e| format!("microphone capture task panicked: {e}"))??;

    let output_device_id = args.output_device_id.clone();
    let playback = tokio::task::spawn_blocking(move || PlaybackStream::start(&output_device_id))
        .await
        .map_err(|e| format!("playback task panicked: {e}"))??;

    let result = loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                break Ok(());
            }
            chunk = pcm_rx.recv() => {
                match chunk {
                    Some(bytes) => {
                        if let Err(e) = client.send_audio_chunk(&bytes).await {
                            break Err(e);
                        }
                    }
                    None => break Err("microphone channel closed unexpectedly".to_string()),
                }
            }
            event = client.next_event() => {
                match event {
                    Some(Ok(value)) => handle_server_event(app, &playback, &value),
                    Some(Err(e)) => break Err(e),
                    None => break Err("OpenAI Realtime connection closed unexpectedly".to_string()),
                }
            }
        }
    };

    mic.stop();
    playback.stop();
    client.close().await;
    result
}

async fn run_session(app: AppHandle, args: StartArgs, cancel: CancellationToken) {
    emit_status(&app, "connecting", None);

    let mut attempt = 0u32;
    loop {
        if cancel.is_cancelled() {
            emit_status(&app, "disconnected", None);
            return;
        }

        let Some(api_key) = get_api_key(&app) else {
            emit_error(&app, "No OpenAI API key configured. Add one in Settings.".to_string(), false);
            emit_status(&app, "error", Some("Missing API key".to_string()));
            return;
        };

        match run_session_once(&app, &args, &api_key, &cancel).await {
            Ok(()) => {
                emit_status(&app, "disconnected", None);
                return;
            }
            Err(e) => {
                if cancel.is_cancelled() {
                    emit_status(&app, "disconnected", None);
                    return;
                }

                attempt += 1;
                if attempt > MAX_RECONNECT_ATTEMPTS {
                    emit_error(&app, format!("Giving up after {attempt} failed attempts: {e}"), false);
                    emit_status(&app, "error", Some(e));
                    return;
                }

                let backoff_secs = (1u64 << attempt).min(30);
                emit_status(&app, "reconnecting", Some(format!("Attempt {attempt}: {e}")));

                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(backoff_secs)) => {}
                    _ = cancel.cancelled() => {
                        emit_status(&app, "disconnected", None);
                        return;
                    }
                }
            }
        }
    }
}

#[tauri::command]
pub async fn start_session(
    app: AppHandle,
    manager: State<'_, SessionManager>,
    input_device_id: String,
    output_device_id: String,
    source_lang: String,
    target_lang: String,
) -> Result<(), String> {
    let mut current = manager.current.lock().await;
    if current.is_some() {
        return Err("a session is already running".to_string());
    }

    let cancel = CancellationToken::new();
    let args = StartArgs {
        input_device_id,
        output_device_id,
        source_lang,
        target_lang,
    };

    let task_app = app.clone();
    let task_cancel = cancel.clone();
    let task = tauri::async_runtime::spawn(async move {
        run_session(task_app, args, task_cancel).await;
    });

    *current = Some(SessionHandle { cancel, task });
    Ok(())
}

#[tauri::command]
pub async fn stop_session(manager: State<'_, SessionManager>) -> Result<(), String> {
    let mut current = manager.current.lock().await;
    if let Some(handle) = current.take() {
        handle.cancel.cancel();
        let _ = handle.task.await;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_session_status(manager: State<'_, SessionManager>) -> Result<bool, String> {
    let current = manager.current.lock().await;
    Ok(current.is_some())
}
