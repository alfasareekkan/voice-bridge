use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use tokio::sync::mpsc::UnboundedSender;

use crate::audio::{
    downmix_to_mono, f32_to_pcm16_le_bytes, find_input_device, resample_linear_f32,
    TARGET_SAMPLE_RATE,
};

/// Captures microphone audio and streams mono PCM16-LE bytes at
/// TARGET_SAMPLE_RATE to `pcm_tx`, ready for base64 encoding into
/// `input_audio_buffer.append` frames (see src-tauri/src/websocket.rs).
///
/// Like `audio::PlaybackStream`, this owns its CPAL `Stream` on a dedicated
/// OS thread (CPAL streams aren't reliably `Send`) rather than inside the
/// async session task.
pub struct MicrophoneCapture {
    stop_tx: std::sync::mpsc::Sender<()>,
    join_handle: Option<std::thread::JoinHandle<()>>,
}

impl MicrophoneCapture {
    pub fn start(device_id: &str, pcm_tx: UnboundedSender<Vec<u8>>) -> Result<Self, String> {
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();
        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();

        let device_id = device_id.to_string();

        let join_handle = std::thread::spawn(move || {
            let result = (|| -> Result<cpal::Stream, String> {
                let device = find_input_device(&device_id)?;
                let supported_config = device
                    .default_input_config()
                    .map_err(|e| format!("failed to get default input config: {e}"))?;
                let sample_format = supported_config.sample_format();
                let stream_config: StreamConfig = supported_config.config();
                let source_rate = stream_config.sample_rate.0;
                let channels = stream_config.channels as usize;

                let stream = match sample_format {
                    SampleFormat::F32 => device.build_input_stream(
                        &stream_config,
                        move |data: &[f32], _| {
                            let mono = downmix_to_mono(data, channels);
                            let resampled = resample_linear_f32(&mono, source_rate, TARGET_SAMPLE_RATE);
                            let bytes = f32_to_pcm16_le_bytes(&resampled);
                            let _ = pcm_tx.send(bytes);
                        },
                        move |err| eprintln!("input stream error: {err}"),
                        None,
                    ),
                    other => return Err(format!("unsupported input sample format: {other:?}")),
                }
                .map_err(|e| format!("failed to build input stream: {e}"))?;

                stream
                    .play()
                    .map_err(|e| format!("failed to start input stream: {e}"))?;
                Ok(stream)
            })();

            match result {
                Ok(stream) => {
                    let _ = ready_tx.send(Ok(()));
                    let _ = stop_rx.recv();
                    drop(stream);
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                }
            }
        });

        ready_rx
            .recv()
            .map_err(|_| "microphone capture thread died before starting".to_string())??;

        Ok(Self {
            stop_tx,
            join_handle: Some(join_handle),
        })
    }

    pub fn stop(mut self) {
        let _ = self.stop_tx.send(());
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}
