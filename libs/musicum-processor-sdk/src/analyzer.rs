use std::{collections::HashMap, hash::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::any::Any;
use crate::processor::{ProcessorContext};

#[allow(dead_code)]
#[derive(Default)]
pub struct AnalysisContext {
    pub requests: Vec<AnalysisRequest>,
    pub results: HashMap<String, Box<dyn AnalysisResult>>,
}

impl AnalysisContext {
    pub fn get_result<T: AnalysisResult + 'static>(&self, hash: &String) -> Option<&T> {
        self.results.get(hash)?.as_any().downcast_ref::<T>()
    }
}

pub struct AnalysisRequest {
    pub analyzer_id: &'static str,
    pub hash: String,
    pub params: Vec<(String, f64)>,
}

impl AnalysisRequest {

     pub fn generate_hash_from(analyzer_id: &'static str, params: &[(String, f64)] ) -> String {
        let mut hasher = DefaultHasher::new();
        analyzer_id.hash(&mut hasher);
        for (key, value) in params {
            key.hash(&mut hasher);
            value.to_bits().hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    }
    
}

#[typetag::serde(tag = "type")]
pub trait AnalysisResult: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn get_analyzer_id(&self) -> &'static str;
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
