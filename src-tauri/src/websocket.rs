//! OpenAI Realtime *Translations* API WebSocket client.
//!
//! Targets the dedicated GA translation endpoint (`/v1/realtime/translations`
//! with the `gpt-realtime-translate` model), not the general-purpose
//! conversational Realtime endpoint. This endpoint acts as a direct speech
//! interpreter natively, so unlike the old beta approach there is no
//! system-prompt hack telling a conversational model to behave like a
//! translator.
//!
//! NOTE: the exact model id, headers, event names, and session-config shape
//! documented here reflect OpenAI's docs as of this writing and were not all
//! confirmed against a live connection — verify against
//! https://developers.openai.com/api/docs/guides/realtime-translation and
//! the "Translation client/server events" reference pages if anything here
//! turns out to be wrong. Every literal event/field name is kept as a named
//! constant below so a doc-drift fix only needs to change this file.

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub const REALTIME_URL_BASE: &str = "wss://api.openai.com/v1/realtime/translations";
/// Verify against https://developers.openai.com/api/docs/guides/realtime-translation
/// at implementation/deploy time — realtime model ids are revised periodically.
pub const REALTIME_MODEL: &str = "gpt-realtime-translate";

/// Bound on how long we wait for the server's `session.closed` ack after we
/// send `session.close`, before force-closing the raw socket anyway.
const GRACEFUL_CLOSE_TIMEOUT: Duration = Duration::from_secs(3);

pub mod client_events {
    pub const SESSION_UPDATE: &str = "session.update";
    pub const INPUT_AUDIO_BUFFER_APPEND: &str = "session.input_audio_buffer.append";
    pub const SESSION_CLOSE: &str = "session.close";
}

pub mod server_events {
    pub const OUTPUT_AUDIO_DELTA: &str = "session.output_audio.delta";
    pub const OUTPUT_TRANSCRIPT_DELTA: &str = "session.output_transcript.delta";
    pub const INPUT_TRANSCRIPT_DELTA: &str = "session.input_transcript.delta";
    pub const SESSION_CLOSED: &str = "session.closed";
    pub const ERROR: &str = "error";
}

/// Best-effort stable per-machine-user identifier for the required
/// `OpenAI-Safety-Identifier` header (used by OpenAI for abuse monitoring,
/// not validated against anything on our side). Not a cryptographic hash —
/// just needs to be a reasonably stable, non-empty opaque string.
fn safety_identifier() -> String {
    let raw = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "voxbridge-user".to_string());
    let mut hasher = DefaultHasher::new();
    raw.hash(&mut hasher);
    "vb-".to_string() + &format!("{:x}", hasher.finish())
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
        headers.insert(
            "OpenAI-Safety-Identifier",
            HeaderValue::from_str(&safety_identifier())
                .map_err(|e| format!("invalid safety identifier header: {e}"))?,
        );

        let (stream, _response) = connect_async(request)
            .await
            .map_err(|e| format!("failed to connect to OpenAI Realtime Translations API: {e}"))?;

        Ok(Self { stream })
    }

    /// Configures the translation session's target output language. The
    /// dedicated translation model acts as a direct interpreter natively —
    /// no system prompt is needed (unlike the old conversational-model
    /// workaround this replaced). Source language is assumed to be
    /// auto-detected by the model; an explicit `session.audio.input.language`
    /// field was not confirmed in available docs. If translations turn out
    /// wrong for a given source language, that's the first thing to check.
    pub async fn send_session_update(&mut self, target_lang_code: &str) -> Result<(), String> {
        let payload = json!({
            "type": client_events::SESSION_UPDATE,
            "session": {
                "audio": {
                    "output": { "language": target_lang_code }
                }
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
                Some(Ok(Message::Close(frame))) => {
                    let detail = frame
                        .map(|f| format!("code={} reason={}", f.code, f.reason))
                        .unwrap_or_else(|| "no close details provided by server".to_string());
                    Some(Err(format!("server closed the connection: {detail}")))
                }
                None => None,
                Some(Ok(_)) => continue, // ignore ping/pong/binary frames
                Some(Err(e)) => Some(Err(format!("websocket read error: {e}"))),
            };
        }
    }

    /// Gracefully ends the session: sends `session.close` and waits (up to
    /// `GRACEFUL_CLOSE_TIMEOUT`) for the server's `session.closed` ack so
    /// any already-buffered translated audio isn't dropped, per the
    /// translations endpoint's documented close sequence, then closes the
    /// raw socket. Never fails outward — this always runs during cleanup.
    pub async fn close(&mut self) {
        let close_event = json!({ "type": client_events::SESSION_CLOSE });
        if self.send_json(&close_event).await.is_ok() {
            let _ = timeout(GRACEFUL_CLOSE_TIMEOUT, async {
                loop {
                    match self.next_event().await {
                        Some(Ok(value)) => {
                            if value.get("type").and_then(|t| t.as_str()) == Some(server_events::SESSION_CLOSED) {
                                return;
                            }
                        }
                        Some(Err(_)) | None => return,
                    }
                }
            })
            .await;
        }

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

#[cfg(test)]
impl RealtimeClient {
    fn from_test_stream(stream: WsStream) -> Self {
        Self { stream }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;

    /// Spins up a real local websocket server/client pair (no network access
    /// needed) so wire-level payloads and the close handshake can be
    /// exercised end-to-end instead of only unit-testing pure helpers.
    async fn connected_pair() -> (RealtimeClient, WebSocketStream<TcpStream>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_task = tokio::spawn(async move {
            let (tcp, _) = listener.accept().await.unwrap();
            accept_async(tcp).await.unwrap()
        });
        let (client_stream, _) = connect_async(format!("ws://{addr}")).await.unwrap();
        let server_stream = server_task.await.unwrap();
        (RealtimeClient::from_test_stream(client_stream), server_stream)
    }

    #[tokio::test]
    async fn send_session_update_sends_ga_translation_payload() {
        let (mut client, mut server) = connected_pair().await;
        client.send_session_update("ml").await.unwrap();

        let msg = server.next().await.unwrap().unwrap();
        let value: Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
        assert_eq!(value["type"], client_events::SESSION_UPDATE);
        assert_eq!(value["session"]["audio"]["output"]["language"], "ml");
    }

    #[tokio::test]
    async fn send_audio_chunk_uses_ga_scoped_event_type() {
        let (mut client, mut server) = connected_pair().await;
        client.send_audio_chunk(&[1, 2, 3, 4]).await.unwrap();

        let msg = server.next().await.unwrap().unwrap();
        let value: Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
        assert_eq!(value["type"], "session.input_audio_buffer.append");
    }

    #[tokio::test]
    async fn close_waits_for_session_closed_ack_before_returning() {
        let (mut client, mut server) = connected_pair().await;
        let server_task = tokio::spawn(async move {
            let closed = server.next().await.unwrap().unwrap();
            let value: Value = serde_json::from_str(closed.to_text().unwrap()).unwrap();
            assert_eq!(value["type"], client_events::SESSION_CLOSE);

            let ack = json!({ "type": server_events::SESSION_CLOSED }).to_string();
            server.send(Message::Text(ack.into())).await.unwrap();
        });

        let start = std::time::Instant::now();
        client.close().await;
        assert!(start.elapsed() < GRACEFUL_CLOSE_TIMEOUT, "close() should return promptly once the ack arrives");
        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn close_gives_up_after_timeout_if_server_never_acks() {
        let (mut client, _server) = connected_pair().await;

        let start = std::time::Instant::now();
        client.close().await;
        let elapsed = start.elapsed();
        assert!(elapsed >= GRACEFUL_CLOSE_TIMEOUT, "close() should wait out the graceful timeout");
        assert!(elapsed < GRACEFUL_CLOSE_TIMEOUT + Duration::from_secs(2), "close() should not hang well past the timeout");
    }
}
