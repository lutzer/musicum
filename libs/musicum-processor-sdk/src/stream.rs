use crate::processor::BaseProcessor;


pub trait ProcessorAudioStream {
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u16;
    fn total_samples(&self) -> u32;
    fn next_chunk(&mut self) -> Option<Vec<f32>>;
    fn reset(&mut self);
    fn seek(&mut self, sample_position: u32);
    fn map(&self, processor: &dyn BaseProcessor) -> Box<dyn ProcessorAudioStream>;
}

pub fn apply_processors_on_stream<'a>(stream: &'a mut dyn ProcessorAudioStream, processors: &[&'a dyn BaseProcessor]) -> Box<&'a mut dyn ProcessedAudioStream> {
    // apply structural edits and also process audio 
    return Box::new(stream);
}