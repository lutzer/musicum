pub mod analyzer;
pub mod ffi;
pub mod processor;
pub mod parameters;

#[doc(hidden)]
pub mod export;
#[doc(hidden)]
pub use abi_stable;
#[doc(hidden)]
pub use bincode;
#[doc(hidden)]
pub use typetag;

pub use analyzer::{
    AnalysisContext, AnalysisRequest, AnalysisResult, AudioAnalyser,
};
pub use processor::{
    BaseProcessor, ProcessorContext, ProcessorDescriptor, ProcessorType,
    Segment, StreamProcessor, StructuralProcessor,
};
pub use ffi::ProcessorEntry;
