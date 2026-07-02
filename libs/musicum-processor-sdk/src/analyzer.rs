use std::any::Any;
use crate::processor::ProcessorContext;

pub struct AnalysisRequest {
    pub analyzer_id: &'static str,
    /// Fingerprint of the processor's own state. Chain-upstream state is
    /// added by the ChainManager — the processor must not include it.
    pub hash:    u64,
    /// Forwarded verbatim to AudioAnalyser::init.
    pub params:      Vec<(String, f64)>,
}

#[typetag::serde(tag = "type")]
pub trait AnalysisResult: Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

pub trait AudioAnalyser: Send + Sync {
    fn init(&mut self, request: &AnalysisRequest);

    fn analyze(
        &mut self,
        samples:   &[f32],
        time:      f64,
        exhausted: bool,
        context:   &ProcessorContext,
    ) -> Option<(String, Box<dyn AnalysisResult>)>;
}
