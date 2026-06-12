use std::sync::{Arc, Mutex};

use musicum_processor_sdk::{
    analyzer::AnalysisContext,
    processor::{ProcessorContext, StreamProcessor},
};
use crate::audio::source::AudioSource;

pub trait AudioNode: AudioSource {}

pub struct StreamProcessorNode {
    upstream:  Box<dyn AudioSource>,
    processor: Arc<Mutex<Box<dyn StreamProcessor>>>,
    context:   ProcessorContext,
}

impl StreamProcessorNode {
    pub fn new(
        upstream:  Box<dyn AudioSource>,
        processor: Arc<Mutex<Box<dyn StreamProcessor>>>,
    ) -> Self {
        let context = ProcessorContext {
            playing:         true,
            sample_rate:     upstream.sample_rate(),
            number_channels: upstream.channels() as u32,
        };
        let node = Self { upstream, processor, context };
        node.processor
            .lock()
            .unwrap()
            .init(&node.context, &mut AnalysisContext::default());
        node
    }
}

impl AudioSource for StreamProcessorNode {
    fn fill_buffer(&mut self, buffer: &mut [f32]) -> usize {
        let n = self.upstream.fill_buffer(buffer);
        let time = self.upstream.position_secs();
        self.processor
            .lock()
            .unwrap()
            .process(&mut buffer[..n], time, &self.context);
        n
    }
    fn sample_rate(&self)   -> u32  { self.upstream.sample_rate() }
    fn channels(&self)      -> u8   { self.upstream.channels() }
    fn is_exhausted(&self)  -> bool { self.upstream.is_exhausted() }
    fn duration_secs(&self) -> f64  { self.upstream.duration_secs() }
    fn position_secs(&self) -> f64  { self.upstream.position_secs() }
}

impl AudioNode for StreamProcessorNode {}
