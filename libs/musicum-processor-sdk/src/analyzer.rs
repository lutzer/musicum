use std::collections::HashMap;
use std::any::Any;
use crate::processor::{ProcessorContext, ProcessorDescriptor};

#[allow(dead_code)]
#[derive(Default)]
pub struct AnalysisContext {
    parameters: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
    results:    HashMap<&'static str, Box<dyn Any + Send + Sync>>,
}

pub trait AudioAnalyser {
    fn analyze(
        &self,
        samples: &[f32],
        context: &ProcessorContext,
        analysis_context: &mut AnalysisContext,
    );

    fn descriptor(&self) -> &'static ProcessorDescriptor;
}
