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
    std_types::{RBox, RSlice, RSliceMut, RStr, RString, RVec},
    StableAbi,
};

use crate::{parameters::ProcessorParamaterInfo, processor::{ProcessorContext, ProcessorDescriptor, ProcessorType, Segment}};

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

// ── ABI-safe traits ───────────────────────────────────────────────────────────
//
// Flat traits (no supertrait inheritance) — #[sabi_trait] does not support
// inheriting methods from supertraits. Each trait duplicates the base methods.

#[sabi_trait]
pub trait AbiStreamProcessor: Send + Sync {
    fn init(
        &mut self,
        ctx: ProcessorContext,
        analysis: AnalysisContextFFI,
    ) -> AnalysisContextFFI;
    fn get_parameter(&self, id: RStr<'_>) -> f64;
    fn set_parameter(&mut self, id: RStr<'_>, value: f64);
    fn requires_analysis(&self) -> bool;
    fn get_analysis_hash(&self) -> RString;
    fn process(&mut self, samples: RSliceMut<'_, f32>, time: f64, ctx: ProcessorContext);
}

#[sabi_trait]
pub trait AbiStructuralProcessor: Send + Sync {
    fn init(
        &mut self,
        ctx: ProcessorContext,
        analysis: AnalysisContextFFI,
    ) -> AnalysisContextFFI;
    fn get_parameter(&self, id: RStr<'_>) -> f64;
    fn set_parameter(&mut self, id: RStr<'_>, value: f64);
    fn requires_analysis(&self) -> bool;
    fn get_analysis_hash(&self) -> RString;
    fn segments(&self, duration: f64, ctx: ProcessorContext) -> RVec<Segment>;
}

#[sabi_trait]
pub trait AbiAnalyzer: Send + Sync {
    fn id(&self) -> RString;
    fn init(&mut self, request: AnalysisRequestFFI);
    fn analyze(
        &mut self,
        samples:   RSlice<'_, f32>,
        time:      f64,
        exhausted: bool,
        ctx:       ProcessorContext,
    ) -> abi_stable::std_types::ROption<AnalysisResultFFI>;
}

// ── ProcessorEntry enum ───────────────────────────────────────────────────────

#[repr(u8)]
#[derive(StableAbi)]
pub enum ProcessorEntry {
    Stream(AbiStreamProcessor_TO<'static, RBox<()>>),
    Structural(AbiStructuralProcessor_TO<'static, RBox<()>>),
    Analyzer(AbiAnalyzer_TO<'static, RBox<()>>),
}
