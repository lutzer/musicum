use std::{collections::HashMap};
use std::any::Any;
use crate::processor::{ProcessorContext};

#[allow(dead_code)]
#[derive(Default)]
pub struct AnalysisContext {
    pub requests: Vec<AnalysisRequest>,
    pub results: HashMap<String, Box<dyn AnalysisResult>>,
}

impl AnalysisContext {
    pub fn get_result<T: AnalysisResult + 'static>(&self, processor_uuid: &String) -> Option<&T> {
        self.results.get(processor_uuid)?.as_any().downcast_ref::<T>()
    }
}

pub struct AnalysisRequest {
    pub analyzer_id: &'static str,
    pub hash: String,
    pub params: Vec<(String, f64)>,
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
    ) -> Option<(String, Box<dyn AnalysisResult>)>;
}
