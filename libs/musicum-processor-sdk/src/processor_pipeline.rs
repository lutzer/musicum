use std::collections::HashMap;

use crate::{
    analyzer::{AnalysisContext, AudioAnalyser},
    processor::BaseProcessor,
    stream::{ProcessorAudioStream, apply_processors_on_stream},
};

#[allow(dead_code)]
pub struct ProcessorPipeline<'a> {
    processors:       Vec<&'a dyn BaseProcessor>,
    analyzers:        HashMap<&'a str, &'a dyn AudioAnalyser>,
    analysis_context: AnalysisContext,
    analyzing:        bool,
    ready:            bool,
}

impl<'a> ProcessorPipeline<'a> {
    pub fn new(
        processors: Vec<&'a dyn BaseProcessor>,
        analyzers: HashMap<&'a str, &'a dyn AudioAnalyser>,
    ) -> Self {
        ProcessorPipeline {
            processors,
            analyzers,
            analysis_context: AnalysisContext::default(),
            analyzing: false,
            ready: false,
        }
    }

    pub fn prepare(
        &mut self,
        _audio_stream: &'a mut dyn ProcessorAudioStream,
        _progress_callback: fn(f32),
        _finished_callback: fn(),
    ) {
        todo!("analysis pass not yet implemented")
    }

    pub fn get_stream(
        &self,
        audio_stream: &'a mut dyn ProcessorAudioStream,
    ) -> &'a mut dyn ProcessorAudioStream {
        apply_processors_on_stream(audio_stream, &self.processors)
    }
}
