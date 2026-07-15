//! Phase 2 extension point. The First Milestone plays translated audio
//! through regular speakers only (see `audio::PlaybackStream`); routing that
//! audio into a virtual microphone (e.g. VB-CABLE on Windows) so apps like
//! Google Meet can pick it up as mic input is future work.
//!
//! `SessionManager` (session.rs) holds an `Option<Box<dyn VirtualMicOutput>>`
//! defaulted to `None` so a real implementation can be slotted in later
//! without changing the session's control flow — it just becomes another
//! sink alongside `PlaybackStream` in the existing PCM pipeline.

#[derive(Debug, thiserror::Error)]
pub enum VirtualMicError {
    #[error("virtual microphone output is not implemented yet")]
    Unimplemented,
}

pub trait VirtualMicOutput: Send {
    fn write_frames(&mut self, pcm16_mono: &[i16]) -> Result<(), VirtualMicError>;
}

/// Default no-op implementation used until Phase 2 adds a real
/// VB-CABLE-backed CPAL output device here.
pub struct NoopVirtualMic;

impl VirtualMicOutput for NoopVirtualMic {
    fn write_frames(&mut self, _pcm16_mono: &[i16]) -> Result<(), VirtualMicError> {
        Ok(())
    }
}
