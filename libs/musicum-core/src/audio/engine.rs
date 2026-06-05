use cpal::{BuildStreamError, PlayStreamError, StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};

use crate::audio::source::AudioSource;

#[derive(Debug)]
pub enum AudioEngineError {
    DeviceNotFound,
    AlreadyRunning,
    IoError(String),
}

pub trait AudioEngine {
    fn start(&mut self) -> Result<(), AudioEngineError>;
    fn stop(&mut self)  -> Result<(), AudioEngineError>;
    fn add_source(&mut self, source: Box<dyn AudioSource>);
    fn sample_rate(&self) -> u32;
}

pub struct CpalEngine {
    _stream:     cpal::Stream,
    sample_rate: u32
}

#[derive(Debug)]
pub enum CpalEngineError {
    Build(BuildStreamError),
    Play(PlayStreamError),
}

impl From<BuildStreamError> for CpalEngineError {
    fn from(e: BuildStreamError) -> Self { Self::Build(e) }
}

impl From<PlayStreamError> for CpalEngineError {
    fn from(e: PlayStreamError) -> Self { Self::Play(e) }
}

impl CpalEngine {
    pub fn new(root: Box<dyn AudioSource>) -> Result<Self, CpalEngineError> {
        // root is moved into the closure — engine never touches it again
        let mut root = root;
       
        let host    = cpal::default_host();
        let device  = host.default_output_device().unwrap();
        let supported = device.default_input_config().unwrap();

        // Capture format details before `supported` is consumed
        let sample_rate = supported.sample_rate();   // e.g. 48000
        let channels    = supported.channels();        // e.g. 2
        let format      = supported.sample_format();   // SampleFormat::F32

        let config: StreamConfig = supported.into();

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                data.fill(0.0);
                root.fill_buffer(data);  // just this — one pull on the graph root
            },
            |err| eprintln!("{err}"),
            None,
        )?;

        stream.play()?;
        Ok(Self { _stream: stream, sample_rate: config.sample_rate })
    }
}