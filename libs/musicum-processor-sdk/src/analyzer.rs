use std::collections::HashMap;
use std::any::Any;
use crate::processor::{ProcessorContext};

#[allow(dead_code)]
#[derive(Default)]
pub struct AnalysisContext {
    pub requests: Vec<AnalysisRequest>,
    pub results: HashMap<&'static str, Box<dyn AnalysisResult>>,
}

pub struct AnalysisRequest {
    pub id: &'static str,
    pub processor_uuid: &'static str,
    pub params: Box<Vec<(String, f64)>>,
}

#[typetag::serde(tag = "type")]
pub trait AnalysisResult: Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

pub trait AudioAnalyser {
    fn init(&mut self, request: &AnalysisRequest);

    fn analyze(
        &mut self,
        samples: &[f32],
        time: f64,
        exhausted: bool,
        context: &ProcessorContext,
        analysis_context: &mut AnalysisContext,
    );

    fn id(&self) -> &'static str;
}
