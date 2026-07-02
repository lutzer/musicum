use abi_stable::StableAbi;
use crate::analyzer::{AnalysisRequest, AnalysisResult};
use crate::parameters::ProcessorParamaterInfo;

pub enum ProcessorType { StructuralProcessor, StreamProcessor, StructuralAndStreamProcesssor }

pub struct ProcessorDescriptor {
    pub id:             &'static str,
    pub name:           &'static str,
    pub processor_type: ProcessorType,
    pub parameters:     &'static [ProcessorParamaterInfo],
}

#[repr(C)]
#[derive(StableAbi, Clone, Copy)]
pub struct ProcessorContext {
    pub playing:         bool,
    pub sample_rate:     u32,
    pub number_channels: u32,
}

#[repr(C)]
#[derive(StableAbi, Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub src_start: f64,
    pub src_end:   f64,
    pub rate:      f64,
}

pub trait BaseProcessor: Send + Sync + 'static {
    fn init(&mut self, _uuid: String, _ctx: &ProcessorContext) {}

    fn request_analysis(&self, _ctx: &ProcessorContext) -> Option<AnalysisRequest> { None }

    fn apply_analysis(&mut self, _result: &dyn AnalysisResult) {}

    fn get_parameter(&self, _id: &str) -> f64 { 0.0 }
    fn set_parameter(&mut self, _id: &str, _value: f64) {}

    fn process(&mut self, _samples: &mut [f32], _time: f64, _ctx: &ProcessorContext) {}

    fn segments(&self, duration: f64, _ctx: &ProcessorContext) -> Vec<Segment> {
        vec![Segment { src_start: 0.0, src_end: duration, rate: 1.0 }]
    }
}

pub trait ProcessorMeta {
    fn descriptor() -> &'static ProcessorDescriptor;
}

pub trait StreamProcessor: BaseProcessor {}
pub trait StructuralProcessor: BaseProcessor {}
impl<T: BaseProcessor + ?Sized> StreamProcessor for T {}
impl<T: BaseProcessor + ?Sized> StructuralProcessor for T {}
