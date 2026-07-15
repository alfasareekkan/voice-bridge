//! OpenAI Realtime API WebSocket client.
//!
//! NOTE: the exact model id, required beta header, and PCM sample rate
//! documented here reflect OpenAI's Realtime API as of this writing and can
//! drift. Every literal event/field name is kept as a named constant below
//! so a doc-drift fix only needs to change this file.

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub const REALTIME_URL_BASE: &str = "wss://api.openai.com/v1/realtime";
/// Verify against https://platform.openai.com/docs/guides/realtime at
/// implementation/deploy time — realtime model ids are revised periodically.
pub const REALTIME_MODEL: &str = "gpt-4o-realtime-preview-2024-12-17";
pub const OPENAI_BETA_HEADER_VALUE: &str = "realtime=v1";

pub mod client_events {
    pub const SESSION_UPDATE: &str = "session.update";
    pub const INPUT_AUDIO_BUFFER_APPEND: &str = "input_audio_buffer.append";
}

pub mod server_events {
    pub const RESPONSE_AUDIO_DELTA: &str = "response.audio.delta";
    pub const RESPONSE_AUDIO_TRANSCRIPT_DELTA: &str = "response.audio_transcript.delta";
    pub const INPUT_AUDIO_TRANSCRIPTION_COMPLETED: &str =
        "conversation.item.input_audio_transcription.completed";
    pub const ERROR: &str = "error";
}

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct RealtimeClient {
    stream: WsStream,
}

impl RealtimeClient {
    pub async fn connect(api_key: &str) -> Result<Self, String> {
        let url = format!("{REALTIME_URL_BASE}?model={REALTIME_MODEL}");
        let mut request = url
            .into_client_request()
            .map_err(|e| format!("invalid realtime URL: {e}"))?;

        let headers = request.headers_mut();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {api_key}"))
                .map_err(|e| format!("invalid API key header: {e}"))?,
        );
        headers.insert("OpenAI-Beta", HeaderValue::from_static(OPENAI_BETA_HEADER_VALUE));

        let (stream, _response) = connect_async(request)
            .await
            .map_err(|e| format!("failed to connect to OpenAI Realtime API: {e}"))?;

        Ok(Self { stream })
    }

    /// Configures the session to act as a direct speech interpreter rather
    /// than a conversational assistant. This prompt-based approach is a
    /// known correctness risk (the underlying model can be tempted to
    /// *answer* instead of *translate*) — flagged for real-speech testing.
    pub async fn send_session_update(
        &mut self,
        source_lang_name: &str,
        target_lang_name: &str,
    ) -> Result<(), String> {
        let instructions = format!(
            "You are a real-time interpreter. You will receive spoken audio in {source_lang_name}. \
             Immediately respond with only a spoken translation into {target_lang_name}. \
             Do not answer questions, do not add commentary, do not have a conversation — \
             produce solely the direct translation of what was said, preserving tone and intent."
        );

        let payload = json!({
            "type": client_events::SESSION_UPDATE,
            "session": {
                "modalities": ["audio", "text"],
                "instructions": instructions,
                "input_audio_format": "pcm16",
                "output_audio_format": "pcm16",
                "input_audio_transcription": { "model": "whisper-1" },
                "turn_detection": { "type": "server_vad" }
            }
        });
        self.send_json(&payload).await
    }

    pub async fn send_audio_chunk(&mut self, pcm16_le_bytes: &[u8]) -> Result<(), String> {
        let encoded = base64::engine::general_purpose::STANDARD.encode(pcm16_le_bytes);
        let payload = json!({
            "type": client_events::INPUT_AUDIO_BUFFER_APPEND,
            "audio": encoded,
        });
        self.send_json(&payload).await
    }

    async fn send_json(&mut self, value: &Value) -> Result<(), String> {
        let text = serde_json::to_string(value).map_err(|e| e.to_string())?;
        self.stream
            .send(Message::Text(text.into()))
            .await
            .map_err(|e| format!("websocket send failed: {e}"))
    }

    /// Returns the next parsed server event. `None` means the connection closed.
    pub async fn next_event(&mut self) -> Option<Result<Value, String>> {
        loop {
            return match self.stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    let text_str: &str = text.as_ref();
                    Some(
                        serde_json::from_str(text_str)
                            .map_err(|e| format!("failed to parse server event: {e}")),
                    )
                }
                Some(Ok(Message::Close(_))) | None => None,
                Some(Ok(_)) => continue, // ignore ping/pong/binary frames
                Some(Err(e)) => Some(Err(format!("websocket read error: {e}"))),
            };
        }
    }

    pub async fn close(&mut self) {
        let _ = self.stream.close(None).await;
    }
}

pub fn decode_audio_delta(value: &Value) -> Option<Vec<u8>> {
    let b64 = value.get("delta")?.as_str()?;
    base64::engine::general_purpose::STANDARD.decode(b64).ok()
}

pub fn extract_transcript_delta(value: &Value) -> Option<String> {
    value.get("delta").and_then(|d| d.as_str()).map(|s| s.to_string())
}
