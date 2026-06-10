use std::path::Path;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use musicum_processor_sdk::processor::ProcessorContext;

use crate::audio::buffer::BufferedSource;
use crate::audio::chain::{ProcessorChain};
use crate::audio::output::{AudioOutput, AudioOutputError, CpalOutput};
use crate::audio::producer::{AudioProducer};
use crate::audio::source::{AudioSource, SharedPipelineState, SourceHandle, SymphoniaSource};
use crate::audio::structural::StructuralSource;
use crate::audio::timeline::Timeline;

pub trait AudioEngine: Send {
    fn load_with_processors(
        &mut self,
        path: &Path,
        chain: ProcessorChain,
    ) -> anyhow::Result<()>;

    fn load(&mut self, path: &Path) -> anyhow::Result<()> {
        self.load_with_processors(path, ProcessorChain::empty())
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
    fn processor_chain(&self) -> &ProcessorChain;

    fn set_parameter(&mut self, edit_uuid: &Uuid, param_id: &str, value: f64);

    // Source↔processed mapping for the editor UI. All delegate to the Timeline.
    fn map_processed_to_source(&self, processed_secs: f64) -> f64;
    fn map_source_to_processed(&self, source_secs: f64) -> Option<f64>;
    fn source_position_secs(&self) -> f64;
    fn source_duration_secs(&self) -> f64;
}

// AudioEngine implementation backed by cpal.
// load() tears down any previous pipeline, constructs a new one, and starts
// the producer thread. All subsequent calls go through source_handle (lock-free).
pub struct CpalEngine {
    output:           CpalOutput,
    source_handle:    Option<SourceHandle>,
    processor_chain:  ProcessorChain,
    timeline:         Option<Arc<RwLock<Timeline>>>,
    source_frames:    u64,
    src_rate:         u32,
    src_ch:           u8,
}

impl CpalEngine {
    pub fn new() -> Result<Self, AudioOutputError> {
        Ok(Self {
            output:           CpalOutput::new()?,
            source_handle:    None,
            processor_chain:  ProcessorChain::empty(),
            timeline:         None,
            source_frames:    0,
            src_rate:         44100,
            src_ch:           2,
        })
    }

    // Applies pending structural parameter changes: rebuild the timeline,
    // keep the playhead's *source* position (snapping forward if its region
    // was cut), flush the processed-frame-keyed AudioStore, and reuse the
    // normal seek path to drain audio produced under the old map.
    fn rebuild_timeline(&mut self) {
        let (Some(timeline), Some(handle)) = (&self.timeline, &self.source_handle) else {
            self.processor_chain.set_structure_dirty(false);
            return;
        };
        let ctx = ProcessorContext {
            playing: false,
            sample_rate: self.src_rate,
            number_channels: self.src_ch as u32,
        };
        let new_tl = self.processor_chain.build_timeline(self.source_frames, self.src_rate, &ctx);
        let new_pos = {
            let old = timeline.read().unwrap();
            let source_pos = old.source_time(handle.position_secs());
            new_tl.processed_time_or_next(source_pos)
        };
        *timeline.write().unwrap() = new_tl;
        handle.seek(new_pos);
        self.processor_chain.set_structure_dirty(false);
    }
}

impl AudioEngine for CpalEngine {
    fn load_with_processors(
        &mut self,
        path: &Path,
        chain: ProcessorChain,
    ) -> anyhow::Result<()> {
        if let Some(h) = &self.source_handle {
            h.shutdown();
        }

        let sample_rate = self.output.sample_rate();
        let channels    = self.output.channels();

        // the decoder reads the audio file and resamples it to the output sample rate and channel configuration
        let decoder  = SymphoniaSource::new(path, sample_rate, channels)?;
        let src_rate = decoder.sample_rate();
        let src_ch   = decoder.channels();
        let source_frames = (decoder.duration_secs() * src_rate as f64).round() as u64;

        let ctx = ProcessorContext { playing: false, sample_rate: src_rate, number_channels: src_ch as u32 };
        let timeline = Arc::new(RwLock::new(chain.build_timeline(source_frames, src_rate, &ctx)));
        let out_duration = timeline.read().unwrap().output_duration();

        // structural source presents the processed timeline to the rest of the pipeline
        let structural = StructuralSource::new(Box::new(decoder), Arc::clone(&timeline));

        let state     = SharedPipelineState::new();
        let ring_cap = 2  * src_rate as usize * src_ch as usize;

        // set up ringbuffer and audiostore
        let (ring_tx, ring_rx) = rtrb::RingBuffer::new(ring_cap);

        // set up producer and bufferedsource as the root source
        let producer = AudioProducer::new(structural, ring_tx, state.clone());
        let buffered: Box<dyn AudioSource> =
            Box::new(BufferedSource::new(ring_rx, src_rate, src_ch, out_duration, state.clone()));

        // build plugin chain on top of the buffered source
        let source = chain.build_source(buffered);

        std::thread::spawn(|| producer.run());

        self.output.set_source(source)?;
        self.source_handle = Some(SourceHandle::new(state, src_rate, src_ch, out_duration));
        self.processor_chain = chain;
        self.timeline = Some(timeline);
        self.source_frames = source_frames;
        self.src_rate = src_rate;
        self.src_ch = src_ch;

        Ok(())
    }

    fn play(&mut self) -> anyhow::Result<()> {
        if self.processor_chain.is_structure_dirty() { self.rebuild_timeline(); }
        self.output.play().map_err(|e| anyhow::anyhow!("{e}"))
    }

    fn pause(&mut self) -> anyhow::Result<()> {
        self.output.pause().map_err(|e| anyhow::anyhow!("{e}"))
    }

    fn seek(&mut self, secs: f64) {
        let max = self.duration_secs();
        if let Some(h) = &self.source_handle { h.seek(secs.clamp(0.0, max)); }
    }

    fn position_secs(&self) -> f64 {
        self.source_handle.as_ref().map_or(0.0, |h| h.position_secs())
    }

    fn seekhead_secs(&self) -> Option<f64> {
        self.source_handle.as_ref().and_then(|h| h.seekhead_secs())
    }

    // Answers from the timeline, not SourceHandle — the value frozen in the
    // handle goes stale across rebuilds.
    fn duration_secs(&self) -> f64 {
        self.timeline.as_ref().map_or(0.0, |t| t.read().unwrap().output_duration())
    }

    fn sample_rate(&self) -> u32  { self.output.sample_rate() }
    fn channels(&self)    -> u8   { self.output.channels() }
    fn is_playing(&self)  -> bool { self.output.is_playing() }

    fn is_exhausted(&self) -> bool {
        self.source_handle.as_ref().map_or(false, |h| h.is_exhausted())
    }

    fn processor_chain(&self) -> &ProcessorChain {
        &self.processor_chain
    }

    fn set_parameter(&mut self, edit_uuid: &Uuid, param_id: &str, value: f64) {
        self.processor_chain.set_parameter(edit_uuid, param_id, value)
    }

    fn map_processed_to_source(&self, processed_secs: f64) -> f64 {
        self.timeline.as_ref().map_or(processed_secs, |t| t.read().unwrap().source_time(processed_secs))
    }

    fn map_source_to_processed(&self, source_secs: f64) -> Option<f64> {
        self.timeline.as_ref().and_then(|t| t.read().unwrap().processed_time(source_secs))
    }

    fn source_position_secs(&self) -> f64 {
        self.map_processed_to_source(self.position_secs())
    }

    fn source_duration_secs(&self) -> f64 {
        if self.src_rate == 0 { return 0.0; }
        self.source_frames as f64 / self.src_rate as f64
    }
}
