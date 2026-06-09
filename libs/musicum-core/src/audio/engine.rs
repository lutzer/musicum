use std::path::Path;
use std::sync::{Arc, Mutex};

use musicum_processor_sdk::processor::StreamProcessor;

use crate::audio::buffer::BufferedSource;
use crate::audio::node::StreamProcessorNode;
use crate::audio::output::{AudioOutput, AudioOutputError, CpalOutput};
use crate::audio::producer::{AudioProducer, AudioStore};
use crate::audio::source::{AudioSource, SharedPipelineState, SourceHandle, SymphoniaSource};

pub trait AudioEngine: Send {
    fn load_with_processors(
        &mut self,
        path: &Path,
        processors: Vec<Box<dyn StreamProcessor>>,
    ) -> anyhow::Result<()>;

    fn load(&mut self, path: &Path) -> anyhow::Result<()> {
        self.load_with_processors(path, vec![])
    }

    fn play(&mut self)  -> anyhow::Result<()>;
    fn pause(&mut self) -> anyhow::Result<()>;
    fn seek(&mut self, secs: f64);
    fn position_secs(&self)  -> f64;
    fn seekhead_secs(&self)  -> Option<f64>;
    fn duration_secs(&self)  -> f64;
    fn sample_rate(&self)    -> u32;
    fn channels(&self)       -> u8;
    fn is_playing(&self)     -> bool;
    fn is_exhausted(&self)   -> bool;
}

// AudioEngine implementation backed by cpal.
// load() tears down any previous pipeline, constructs a new one, and starts
// the producer thread. All subsequent calls go through source_handle (lock-free).
pub struct CpalEngine {
    output:        CpalOutput,
    source_handle: Option<SourceHandle>,
}

impl CpalEngine {
    pub fn new() -> Result<Self, AudioOutputError> {
        Ok(Self { output: CpalOutput::new()?, source_handle: None })
    }
}

impl AudioEngine for CpalEngine {
    fn load_with_processors(
        &mut self,
        path: &Path,
        processors: Vec<Box<dyn StreamProcessor>>,
    ) -> anyhow::Result<()> {
        if let Some(h) = &self.source_handle {
            h.shutdown();
        }

        let sample_rate = self.output.sample_rate();
        let channels    = self.output.channels();

        let decoder  = SymphoniaSource::new(path, sample_rate, channels)?;
        let src_rate = decoder.sample_rate();
        let src_ch   = decoder.channels();
        let duration = decoder.duration_secs();

        let state     = SharedPipelineState::new();
        let ring_cap  = 2  * src_rate as usize * src_ch as usize;
        let store_cap = 30 * src_rate as usize * src_ch as usize;

        // set up ringbuffer and audiostore
        let (ring_tx, ring_rx) = rtrb::RingBuffer::new(ring_cap);
        let store = Arc::new(Mutex::new(AudioStore::new(store_cap, src_ch)));

        // set up producer and bufferedsource as the root source
        let producer = AudioProducer::new(decoder, store, ring_tx, state.clone());
        let buffered: Box<dyn AudioSource> =
            Box::new(BufferedSource::new(ring_rx, src_rate, src_ch, duration, state.clone()));

        // create plugin chain
        let source = processors
            .into_iter()
            .fold(buffered, |upstream, processor| {
                Box::new(StreamProcessorNode::new(upstream, processor)) as Box<dyn AudioSource>
            });

        std::thread::spawn(|| producer.run());
        self.output.set_source(source)?;
        self.source_handle = Some(SourceHandle::new(state, src_rate, src_ch, duration));
        Ok(())
    }

    fn play(&mut self) -> anyhow::Result<()> {
        self.output.play().map_err(|e| anyhow::anyhow!("{e}"))
    }

    fn pause(&mut self) -> anyhow::Result<()> {
        self.output.pause().map_err(|e| anyhow::anyhow!("{e}"))
    }

    fn seek(&mut self, secs: f64) {
        if let Some(h) = &self.source_handle { h.seek(secs); }
    }

    fn position_secs(&self) -> f64 {
        self.source_handle.as_ref().map_or(0.0, |h| h.position_secs())
    }

    fn seekhead_secs(&self) -> Option<f64> {
        self.source_handle.as_ref().and_then(|h| h.seekhead_secs())
    }

    fn duration_secs(&self) -> f64 {
        self.source_handle.as_ref().map_or(0.0, |h| h.duration_secs())
    }

    fn sample_rate(&self) -> u32  { self.output.sample_rate() }
    fn channels(&self)    -> u8   { self.output.channels() }
    fn is_playing(&self)  -> bool { self.output.is_playing() }

    fn is_exhausted(&self) -> bool {
        self.source_handle.as_ref().map_or(false, |h| h.is_exhausted())
    }
}
