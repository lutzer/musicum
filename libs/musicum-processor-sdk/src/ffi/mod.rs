#![allow(non_local_definitions)]

pub mod analysis;
pub use analysis::{
    AnalysisContextFFI, AnalysisRequestFFI, AnalysisResultFFI,
    AnalyzerDescriptorFFI,
};

#[cfg(test)]
mod tests;

use abi_stable::{
    sabi_trait,
    std_types::{ROption, RSlice, RSliceMut, RStr, RString, RVec},
    StableAbi,
};

use crate::analyzer::{AnalysisContext, AnalysisRequest, AudioAnalyser};
use crate::parameters::ProcessorParamaterInfo;
use crate::processor::{BaseProcessor, ProcessorContext, ProcessorDescriptor, ProcessorType, Segment};

// ── Descriptor FFI types ──────────────────────────────────────────────────────

#[repr(u8)]
#[derive(StableAbi, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessorTypeFFI {
    Structural,
    Stream,
    Analyzer,
}

impl From<&ProcessorType> for ProcessorTypeFFI {
    fn from(t: &ProcessorType) -> Self {
        match t {
            ProcessorType::StructuralProcessor => ProcessorTypeFFI::Structural,
            ProcessorType::StreamProcessor     => ProcessorTypeFFI::Stream,
            ProcessorType::Analyzer            => ProcessorTypeFFI::Analyzer,
        }
    }
}

#[repr(C)]
#[derive(StableAbi, Clone)]
pub struct ProcessorDescriptorFFI {
    pub id:             RStr<'static>,
    pub name:           RStr<'static>,
    pub processor_type: ProcessorTypeFFI,
    pub params:         RVec<ProcessorParamFFI>,
}

#[repr(u8)]
#[derive(StableAbi, Clone, Copy)]
pub enum ProcessorParamFFI {
    Float {
        id:       RStr<'static>,
        name:     RStr<'static>,
        default:  f32,
        min:      f32,
        max:      f32,
        step:     f32,
        unit:     RStr<'static>,
        editable: bool,
    },
    Bool {
        id:       RStr<'static>,
        name:     RStr<'static>,
        default:  bool,
        editable: bool,
    },
    Time {
        id:       RStr<'static>,
        name:     RStr<'static>,
        default:  f64,
        editable: bool,
    },
    Int {
        id:       RStr<'static>,
        name:     RStr<'static>,
        default:  i32,
        min:      i32,
        max:      i32,
        editable: bool,
    },
    Canvas {
        id:           RStr<'static>,
        name:         RStr<'static>,
        aspect_ratio: f32,
    },
}

impl From<&'static ProcessorDescriptor> for ProcessorDescriptorFFI {
    fn from(d: &'static ProcessorDescriptor) -> Self {
        Self {
            id:             RStr::from(d.id),
            name:           RStr::from(d.name),
            processor_type: ProcessorTypeFFI::from(&d.processor_type),
            params:         d.parameters.iter().map(ProcessorParamFFI::from).collect(),
        }
    }
}

impl From<&'static ProcessorParamaterInfo> for ProcessorParamFFI {
    fn from(p: &'static ProcessorParamaterInfo) -> Self {
        match p {
            ProcessorParamaterInfo::Float { id, name, default, min, max, step, unit, editable } =>
                Self::Float { id: RStr::from(*id), name: RStr::from(*name), default: *default, min: *min, max: *max, step: *step, unit: RStr::from(*unit), editable: *editable },
            ProcessorParamaterInfo::Bool { id, name, default, editable } =>
                Self::Bool { id: RStr::from(*id), name: RStr::from(*name), default: *default, editable: *editable },
            ProcessorParamaterInfo::Time { id, name, default, editable } =>
                Self::Time { id: RStr::from(*id), name: RStr::from(*name), default: *default, editable: *editable },
            ProcessorParamaterInfo::Int { id, name, default, min, max, editable } =>
                Self::Int { id: RStr::from(*id), name: RStr::from(*name), default: *default, min: *min, max: *max, editable: *editable },
            ProcessorParamaterInfo::Canvas { id, name, aspect_ratio } =>
                Self::Canvas { id: RStr::from(*id), name: RStr::from(*name), aspect_ratio: *aspect_ratio },
        }
    }
}

// ── ABI-safe processor trait + generic adapter ────────────────────────────────

#[sabi_trait]
pub trait AbiProcessor: Send + Sync {
    fn init(
        &mut self,
        uuid: RString,
        ctx: ProcessorContext,
        analysis: AnalysisContextFFI,
    ) -> AnalysisContextFFI;

    fn get_parameter(&self, id: RStr<'_>) -> f64;
    fn set_parameter(&mut self, id: RStr<'_>, value: f64);
    fn requires_analysis(&self) -> bool;
    fn get_analysis_hash(&self) -> RString;

    fn process(&mut self, samples: RSliceMut<'_, f32>, time: f64, ctx: ProcessorContext);
    fn segments(&self, duration: f64, ctx: ProcessorContext) -> RVec<Segment>;
}

pub struct FfiAdapter<T: BaseProcessor>(pub T);

impl<T: BaseProcessor> AbiProcessor for FfiAdapter<T> {
    fn init(
        &mut self,
        uuid: RString,
        ctx: ProcessorContext,
        analysis: AnalysisContextFFI,
    ) -> AnalysisContextFFI {
        let mut native = AnalysisContext::default();
        analysis.drain_into(&mut native);
        self.0.init(uuid.into(), &ctx, &mut native);
        AnalysisContextFFI::from_context(&mut native)
    }
    fn get_parameter(&self, id: RStr<'_>) -> f64 {
        self.0.get_parameter(id.as_str())
    }
    fn set_parameter(&mut self, id: RStr<'_>, value: f64) {
        self.0.set_parameter(id.as_str(), value);
    }
    fn requires_analysis(&self) -> bool { self.0.requires_analysis() }
    fn get_analysis_hash(&self) -> RString { RString::from(self.0.get_analysis_hash()) }
    fn process(
        &mut self,
        mut samples: RSliceMut<'_, f32>,
        time: f64,
        ctx: ProcessorContext,
    ) {
        self.0.process(samples.as_mut_slice(), time, &ctx);
    }
    fn segments(
        &self,
        duration: f64,
        ctx: ProcessorContext,
    ) -> RVec<Segment> {
        self.0.segments(duration, &ctx).into()
    }
}

// ── ABI-safe analyzer trait + generic adapter ─────────────────────────────────

#[sabi_trait]
pub trait AbiAnalyzer: Send + Sync {
    fn init(&mut self, request: AnalysisRequestFFI);
    fn analyze(
        &mut self,
        samples:   RSlice<'_, f32>,
        time:      f64,
        exhausted: bool,
        ctx:       ProcessorContext,
    ) -> ROption<AnalysisResultFFI>;
}

pub struct FfiAnalyzerAdapter<T: AudioAnalyser>(pub T);

impl<T: AudioAnalyser + Send + Sync> AbiAnalyzer for FfiAnalyzerAdapter<T> {
    fn init(&mut self, request: AnalysisRequestFFI) {
        let native = AnalysisRequest::from(&request);
        self.0.init(&native);
    }
    fn analyze(
        &mut self,
        samples:   RSlice<'_, f32>,
        time:      f64,
        exhausted: bool,
        ctx:       ProcessorContext,
    ) -> ROption<AnalysisResultFFI> {
        match self.0.analyze(samples.as_slice(), time, exhausted, &ctx) {
            Some((hash, boxed)) => ROption::RSome(AnalysisResultFFI::from_boxed(hash, &boxed)),
            None => ROption::RNone,
        }
    }
}
