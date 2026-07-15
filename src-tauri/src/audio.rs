use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// OpenAI Realtime API expects/produces mono PCM16 at this sample rate.
/// See src-tauri/src/websocket.rs for where this assumption is used.
pub const TARGET_SAMPLE_RATE: u32 = 24000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

pub fn input_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    let host = cpal::default_host();
    let default_name = host.default_input_device().and_then(|d| d.name().ok());
    let devices = host
        .input_devices()
        .map_err(|e| format!("failed to enumerate input devices: {e}"))?;

    Ok(devices
        .filter_map(|d| d.name().ok())
        .map(|name| AudioDeviceInfo {
            is_default: Some(&name) == default_name.as_ref(),
            id: name.clone(),
            name,
        })
        .collect())
}

pub fn output_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    let host = cpal::default_host();
    let default_name = host.default_output_device().and_then(|d| d.name().ok());
    let devices = host
        .output_devices()
        .map_err(|e| format!("failed to enumerate output devices: {e}"))?;

    Ok(devices
        .filter_map(|d| d.name().ok())
        .map(|name| AudioDeviceInfo {
            is_default: Some(&name) == default_name.as_ref(),
            id: name.clone(),
            name,
        })
        .collect())
}

/// Finds a CPAL input device by the name we handed out as its id in
/// `input_devices()`. CPAL has no stable device-id concept across platforms,
/// so the device name is used as the id — acceptable for a single-user MVP
/// where device names are unique in practice.
pub fn find_input_device(id: &str) -> Result<cpal::Device, String> {
    let host = cpal::default_host();
    host.input_devices()
        .map_err(|e| format!("failed to enumerate input devices: {e}"))?
        .find(|d| d.name().map(|n| n == id).unwrap_or(false))
        .ok_or_else(|| format!("input device not found: {id}"))
}

pub fn find_output_device(id: &str) -> Result<cpal::Device, String> {
    let host = cpal::default_host();
    host.output_devices()
        .map_err(|e| format!("failed to enumerate output devices: {e}"))?
        .find(|d| d.name().map(|n| n == id).unwrap_or(false))
        .ok_or_else(|| format!("output device not found: {id}"))
}

#[tauri::command]
pub fn list_input_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    input_devices()
}

#[tauri::command]
pub fn list_output_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    output_devices()
}

/// Averages interleaved multi-channel samples down to mono.
pub fn downmix_to_mono(data: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return data.to_vec();
    }
    data.chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
        .collect()
}

/// Simple linear-interpolation resampler. Not audiophile-grade, but
/// dependency-light and easy to reason about for an MVP; swap for a proper
/// windowed-sinc resampler later if quality/aliasing becomes an issue.
pub fn resample_linear_f32(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || input.is_empty() {
        return input.to_vec();
    }
    let ratio = to_rate as f64 / from_rate as f64;
    let out_len = ((input.len() as f64) * ratio).round() as usize;
    let mut output = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f64 / ratio;
        let idx = src_pos.floor() as usize;
        let frac = (src_pos - idx as f64) as f32;
        let a = input.get(idx).copied().unwrap_or(0.0);
        let b = input.get(idx + 1).copied().unwrap_or(a);
        output.push(a + (b - a) * frac);
    }
    output
}

pub fn resample_linear_i16(input: &[i16], from_rate: u32, to_rate: u32) -> Vec<i16> {
    if from_rate == to_rate || input.is_empty() {
        return input.to_vec();
    }
    let as_f32: Vec<f32> = input.iter().map(|s| *s as f32).collect();
    resample_linear_f32(&as_f32, from_rate, to_rate)
        .into_iter()
        .map(|s| s.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16)
        .collect()
}

pub fn f32_to_pcm16_le_bytes(samples: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for s in samples {
        let clamped = (s * i16::MAX as f32).round().clamp(i16::MIN as f32, i16::MAX as f32);
        bytes.extend_from_slice(&(clamped as i16).to_le_bytes());
    }
    bytes
}

pub fn pcm16_le_bytes_to_i16(bytes: &[u8]) -> Vec<i16> {
    bytes
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect()
}

/// Plays back mono PCM16 audio (as received from OpenAI at
/// `TARGET_SAMPLE_RATE`) through a chosen output device.
///
/// CPAL's `Stream` is not `Send` on every platform backend, so it cannot be
/// held across `.await` points in an async task. Instead we spawn a plain OS
/// thread that owns the stream for its whole lifetime and blocks until told
/// to stop; audio to play is handed over through a shared ring buffer rather
/// than by moving the stream itself.
pub struct PlaybackStream {
    buffer: Arc<Mutex<VecDeque<i16>>>,
    device_sample_rate: u32,
    device_channels: usize,
    stop_tx: std::sync::mpsc::Sender<()>,
    join_handle: Option<std::thread::JoinHandle<()>>,
}

impl PlaybackStream {
    pub fn start(device_id: &str) -> Result<Self, String> {
        let buffer: Arc<Mutex<VecDeque<i16>>> = Arc::new(Mutex::new(VecDeque::new()));
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(u32, usize), String>>();
        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();

        let device_id = device_id.to_string();
        let thread_buffer = buffer.clone();

        let join_handle = std::thread::spawn(move || {
            let result = (|| -> Result<(cpal::Stream, u32, usize), String> {
                let device = find_output_device(&device_id)?;
                let supported_config = device
                    .default_output_config()
                    .map_err(|e| format!("failed to get default output config: {e}"))?;
                let sample_format = supported_config.sample_format();
                let stream_config: StreamConfig = supported_config.config();
                let sample_rate = stream_config.sample_rate.0;
                let channels = stream_config.channels as usize;

                let callback_buffer = thread_buffer.clone();
                let stream = match sample_format {
                    SampleFormat::F32 => device.build_output_stream(
                        &stream_config,
                        move |data: &mut [f32], _| {
                            let mut buf = callback_buffer.lock().unwrap();
                            for sample in data.iter_mut() {
                                let s = buf.pop_front().unwrap_or(0);
                                *sample = s as f32 / i16::MAX as f32;
                            }
                        },
                        move |err| eprintln!("output stream error: {err}"),
                        None,
                    ),
                    other => return Err(format!("unsupported output sample format: {other:?}")),
                }
                .map_err(|e| format!("failed to build output stream: {e}"))?;

                stream
                    .play()
                    .map_err(|e| format!("failed to start output stream: {e}"))?;
                Ok((stream, sample_rate, channels))
            })();

            match result {
                Ok((stream, sample_rate, channels)) => {
                    let _ = ready_tx.send(Ok((sample_rate, channels)));
                    let _ = stop_rx.recv();
                    drop(stream);
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                }
            }
        });

        let (device_sample_rate, device_channels) = ready_rx
            .recv()
            .map_err(|_| "playback thread died before starting".to_string())??;

        Ok(Self {
            buffer,
            device_sample_rate,
            device_channels,
            stop_tx,
            join_handle: Some(join_handle),
        })
    }

    /// Resamples mono PCM16 (at TARGET_SAMPLE_RATE) to the device's native
    /// rate and enqueues it for playback, replicated across all output
    /// channels.
    pub fn push_samples(&self, pcm16_mono: &[i16]) {
        let resampled = resample_linear_i16(pcm16_mono, TARGET_SAMPLE_RATE, self.device_sample_rate);
        let mut buf = self.buffer.lock().unwrap();
        for sample in resampled {
            for _ in 0..self.device_channels {
                buf.push_back(sample);
            }
        }
    }

    pub fn stop(mut self) {
        let _ = self.stop_tx.send(());
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}
