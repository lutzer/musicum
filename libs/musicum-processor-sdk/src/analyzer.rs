use std::collections::HashMap;
use crate::processor::{ProcessorContext};
use std::any::{Any};

pub struct AnalysisContext {
    parameters: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
    results: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
}

pub struct AnalyzerInfo {
    pub id: &'static str,
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self { parameters: Default::default(), results: Default::default() }
    }
}

pub trait AudioAnalyser<'a> {
    fn analyze(
        &self,
        chunk_generator: impl Iterator<Item = Vec<f32>>,
        processor_context: &ProcessorContext,
        analysis_context: &mut AnalysisContext
    );

    fn descriptor(&self) ->  &'static AnalyzerInfo;
}