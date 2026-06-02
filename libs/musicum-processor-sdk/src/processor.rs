use abi_stable::StableAbi;
use crate::analyzer::AnalysisContext;

pub enum ProcessorParamaterInfo {
    Float {
        id:      &'static str,
        name:    &'static str,
        default: f32,
        min:     f32,
        max:     f32,
        step:    f32,
        unit:    &'static str,
        editable: bool
    },
    Bool  { id: &'static str, name: &'static str, default: bool, editable: bool },
    Time  { id: &'static str, name: &'static str, default: f64, editable: bool },
    Int   { id: &'static str, name: &'static str, default: i64, min: i64, max: i64, editable: bool },
}

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
    fn prepare(&mut self, 
        context: &ProcessorContext, 
        ctx: &mut AnalysisContext);

    /// gets the processors descriptor
    fn descriptor(&self) ->  &'static ProcessorDescriptor;

    fn get_parameter(&self, id: &str) -> f64;
    fn set_parameter(&mut self, id: &str, value: f64);

    /// tells the processor pipeline that this processor needs to run an analysis
    fn requires_analysis(&self) -> bool;
}

pub trait StreamProcessor: BaseProcessor {
    /// Processes the sample buffer in place and returns a buffer of the same length.
    fn process( &mut self,
        sample_buffer: &mut [f32],
        time: f64,
        context: &ProcessorContext);
}

pub trait StructuralProcessor: BaseProcessor {

    /// Map a time in the *source* domain forward to the *processed* domain.
    /// `duration` is the audio length (seconds) *before* this edit.
    fn map_source_time(&self, 
        source_time: f64,
        duration: f64,
        context: &ProcessorContext) -> f64;
    
    /// Map a time in the *processed* domain back to the *source* domain.
    /// `duration` is the audio length (seconds) *before* this edit.
    fn map_processed_time(
        &self,
        processed_time: f64,
        duration: f64,
        context: &ProcessorContext
    ) -> f64;

    /// Duration of the output audio (seconds) given input `duration`.
    fn output_duration(
        &self,
        duration: f64,
        context: &ProcessorContext
    ) -> f64;
}