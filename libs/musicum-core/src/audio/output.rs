use std::sync::{Arc, Mutex};

use cpal::{
    BuildStreamError, PauseStreamError, PlayStreamError, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::audio::source::AudioSource;

#[derive(Debug, thiserror::Error)]
pub enum AudioOutputError {
    #[error("no default output device found")]
    DeviceNotFound,
    #[error("stream build failed: {0}")]
    Build(#[from] BuildStreamError),
    #[error("stream play failed: {0}")]
    Play(#[from] PlayStreamError),
    #[error("stream pause failed: {0}")]
    Pause(#[from] PauseStreamError),
    #[error("source lock poisoned")]
    LockPoisoned,
}

pub trait AudioOutput {
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u8;
    fn is_playing(&self) -> bool;
    fn play(&mut self)  -> Result<(), AudioOutputError>;
    fn pause(&mut self) -> Result<(), AudioOutputError>;
    fn set_source(&mut self, source: Box<dyn AudioSource>) -> Result<(), AudioOutputError>;
}

// Wraps a cpal output stream. The stream callback calls fill_buffer() on the
// currently-loaded AudioSource on every hardware buffer request.
// Source is swapped via set_source() on load; play/pause toggle the stream.
pub struct CpalOutput {
    stream:      cpal::Stream,
    sample_rate: u32,
    channels:    u8,
    playing:     bool,
    source:      Arc<Mutex<Option<Box<dyn AudioSource>>>>,
}

impl CpalOutput {
    pub fn new() -> Result<Self, AudioOutputError> {
        let source: Arc<Mutex<Option<Box<dyn AudioSource>>>> = Arc::new(Mutex::new(None));
        let cb_source = Arc::clone(&source);

        let host     = cpal::default_host();
        let device   = host.default_output_device().ok_or(AudioOutputError::DeviceNotFound)?;
        let supported = device.default_output_config().map_err(|_| AudioOutputError::DeviceNotFound)?;

        let sample_rate = supported.sample_rate();
        let channels    = supported.channels() as u8;
        let config: StreamConfig = supported.into();

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                data.fill(0.0);
                if let Ok(mut guard) = cb_source.lock() {
                    if let Some(src) = guard.as_mut() {
                        src.fill_buffer(data);
                    }
                }
            },
            |err| eprintln!("cpal stream error: {err}"),
            None,
        )?;

        Ok(Self { stream, sample_rate, channels, playing: false, source })
    }
}

impl AudioOutput for CpalOutput {
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn channels(&self) -> u8 { self.channels }
    fn is_playing(&self) -> bool { self.playing }

    fn play(&mut self) -> Result<(), AudioOutputError> {
        self.stream.play().map_err(AudioOutputError::Play)?;
        self.playing = true;
        Ok(())
    }

    fn pause(&mut self) -> Result<(), AudioOutputError> {
        self.stream.pause().map_err(AudioOutputError::Pause)?;
        self.playing = false;
        Ok(())
    }

    fn set_source(&mut self, source: Box<dyn AudioSource>) -> Result<(), AudioOutputError> {
        *self.source.lock().map_err(|_| AudioOutputError::LockPoisoned)? = Some(source);
        Ok(())
    }
}
