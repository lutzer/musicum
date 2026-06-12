use abi_stable::StableAbi;
use crate::{analyzer::AnalysisContext, parameters::ProcessorParamaterInfo};


pub enum ProcessorType { StructuralProcessor, StreamProcessor, Analyzer }

pub struct ProcessorDescriptor {
    pub id:      &'static str,
    pub name:       &'static str,
    pub processor_type:  ProcessorType,
    pub parameters: &'static [ProcessorParamaterInfo],
}

#[repr(C)]
#[derive(StableAbi, Clone, Copy)]
pub struct ProcessorContext {
    pub playing: bool,
    pub sample_rate: u32,
    pub number_channels: u32,
}

pub trait BaseProcessor: Send + Sync {
    /// Prepares the processor, reads and writes from AnalysisContext if necesarry
    fn init(&mut self, 
        context: &ProcessorContext, 
        ctx: &mut AnalysisContext);

    /// gets the processors descriptor
    fn descriptor(&self) ->  &'static ProcessorDescriptor;

    fn get_parameter(&self, id: &str) -> f64;
    fn set_parameter(&mut self, id: &str, value: f64);

    /// tells the processor pipeline that this processor needs to run an analysis
    fn requires_analysis(&self) -> bool { false }
    fn get_analysis_hash(&self) -> String { String::new() }
}

pub trait StreamProcessor: BaseProcessor {
    /// Processes the sample buffer in place and returns a buffer of the same length.
    fn process( &mut self,
        samples: &mut [f32],
        time: f64,
        context: &ProcessorContext);
}

/// One contiguous span of this processor's *input* timeline that survives
/// into its output, in chain order. Times are seconds in the input domain.
#[repr(C)]
#[derive(StableAbi, Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub src_start: f64,
    pub src_end:   f64,
    pub rate:      f64, // 1.0 = unchanged; speed plugins return != 1.0
}

pub trait StructuralProcessor: BaseProcessor {
    /// The edit, as ordered output segments over an input of `duration` seconds.
    fn segments(&self, duration: f64, context: &ProcessorContext) -> Vec<Segment>;
}