use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64};

use crate::audio::buffer::{AudioProducer, AudioStore, BufferedSource, SourceHandle};
use crate::audio::output::{AudioOutput, AudioOutputError, CpalOutput};
use crate::audio::source::{AudioSource, SymphoniaSource};

pub trait AudioEngine: Send {
    fn load(&mut self, path: &Path) -> anyhow::Result<()>;
    fn play(&mut self)  -> anyhow::Result<()>;
    fn pause(&mut self) -> anyhow::Result<()>;
    fn seek(&mut self, secs: f64);
    fn position_secs(&self)  -> f64;
    fn duration_secs(&self)  -> f64;
    fn sample_rate(&self)    -> u32;
    fn channels(&self)       -> u8;
    fn is_playing(&self)     -> bool;
    fn is_exhausted(&self)   -> bool;
}

pub struct CpalEngine {
    output:      CpalOutput,
    source_handle: Option<SourceHandle>,
}

impl CpalEngine {
    pub fn new() -> Result<Self, AudioOutputError> {
        Ok(Self { output: CpalOutput::new()?, source_handle: None })
    }
}

impl AudioEngine for CpalEngine {
    fn load(&mut self, path: &Path) -> anyhow::Result<()> {
        if let Some(h) = &self.source_handle {
            h.shutdown();
        }

        let sample_rate = self.output.sample_rate();
        let channels    = self.output.channels();

        let decoder   = SymphoniaSource::new(path, sample_rate, channels)?;
        let src_rate  = decoder.sample_rate();
        let src_ch    = decoder.channels();
        let duration  = decoder.duration_secs();
        let ring_cap  = 2  * src_rate as usize * src_ch as usize;
        let store_cap = 30 * src_rate as usize * src_ch as usize;

        let (ring_tx, ring_rx) = rtrb::RingBuffer::new(ring_cap);
        let store         = Arc::new(Mutex::new(AudioStore::new(store_cap, src_ch)));
        let seek_pending  = Arc::new(AtomicBool::new(false));
        let seek_frame    = Arc::new(AtomicU64::new(0));
        let producer_done = Arc::new(AtomicBool::new(false));
        let shutdown      = Arc::new(AtomicBool::new(false));

        let producer = AudioProducer::new(
            decoder, store, ring_tx, ring_cap,
            seek_pending.clone(), seek_frame.clone(), producer_done.clone(),
            shutdown.clone(),
        );
        let source = BufferedSource::new(
            ring_rx, src_rate, src_ch, duration,
            seek_pending.clone(), seek_frame.clone(), producer_done,
        );

        std::thread::spawn(|| producer.run());
        self.output.set_source(Box::new(source))?;
        self.source_handle = Some(SourceHandle::new(seek_pending, seek_frame, shutdown, src_rate));
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
        let g = self.output.get_source().lock().unwrap();
        g.as_ref().map(|s| s.position_secs()).unwrap_or(0.0)
    }

    fn duration_secs(&self) -> f64 {
        let g = self.output.get_source().lock().unwrap();
        g.as_ref().map(|s| s.duration_secs()).unwrap_or(0.0)
    }

    fn sample_rate(&self) -> u32  { self.output.sample_rate() }
    fn channels(&self)    -> u8   { self.output.channels() }
    fn is_playing(&self)  -> bool { self.output.is_playing() }

    fn is_exhausted(&self) -> bool {
        let g = self.output.get_source().lock().unwrap();
        g.as_ref().map(|s| s.is_exhausted()).unwrap_or(false)
    }
}
