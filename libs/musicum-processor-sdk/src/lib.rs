pub mod analyzer;
pub mod ffi;
pub mod fingerprint;
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
    AnalysisRequest, AnalysisResult, AudioAnalyser,
};
pub use processor::{
    BaseProcessor, ProcessorContext, ProcessorDescriptor, ProcessorMeta, ProcessorType,
    Segment, StreamProcessor, StructuralProcessor,
};
