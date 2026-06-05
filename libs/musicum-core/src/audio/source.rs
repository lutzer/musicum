pub trait AudioSource: Send {
    /// Fill `buffer` with interleaved f32 samples; return frames written.
    fn fill_buffer(&mut self, buffer: &mut [f32]) -> usize;
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u16;
    fn is_exhausted(&self) -> bool;
}

pub trait AudioNode: AudioSource {
    fn connect(&mut self, source: Box<dyn AudioSource>);
}

pub struct PluginNode  {
    
}
