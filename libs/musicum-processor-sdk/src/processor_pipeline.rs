use std::collections::HashMap;

use crate::{analyzer::{AnalysisContext, AudioAnalyser}, processor::{BaseProcessor, ProcessorContext}, stream::{ProcessorAudioStream, apply_processors_on_stream}};

pub struct ProcessorPipeline<'a> {
    processors: Vec<&'a dyn BaseProcessor>,
    analyzers: HashMap<&'a str, &'a dyn AudioAnalyser<'a>>,
    analysis_context: AnalysisContext,
    analyzing: bool,
    ready: bool,
}

impl<'a> ProcessorPipeline<'a> {
    fn new(
        processors: Vec<&'a dyn BaseProcessor>, 
        analyzers: HashMap<&'a str, &'a dyn AudioAnalyser<'a>>) -> Self 
    {
        ProcessorPipeline { 
            processors, 
            analyzers,
            analysis_context: AnalysisContext::default(), 
            analyzing: false, 
            ready: false 
        }
    }

    /// Prepares a processor pipeline befor playback can start.
    /// Runs analysis processors if necessary.
    fn prepare(&mut self, 
        audio_stream: &'a mut dyn ProcessorAudioStream, 
        progress_callback: fn(f32),
        finished_callback: fn()
    ) {
        for (i, value) in self.processors.iter().enumerate() {
            // 1. check if any analysis needs to be run first
            if let Some(analyzer_id) = value.requires_analyzer() {
                if let Some(analyzer) = self.analyzers.get(analyzer_id) 
                {
                    // remap audio stream and apply processors
                    let processed_stream = apply_processors_on_stream(audio_stream, &self.processors[..i]);
                    
                    // 2. run analysis on remapped buffer with processed audio effects
                    analyzer.analyze(*processed_stream, &mut self.analysis_context);
                    progress_callback(i as f32 / self.processors.len() as f32);
                }
                else 
                {
                    eprintln!("Cant find analyzer: {analyzer_id}");
                }
            }
        }
        finished_callback();
    }

    fn get_stream(
        &self,
        audio_stream: &'a mut dyn ProcessorAudioStream,
    ) -> Box<&'a mut dyn ProcessorAudioStream> {
        apply_processors_on_stream(audio_stream, &self.processors)
    };

}