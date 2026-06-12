use std::any::Any;

use musicum_processor_sdk::{analyzer::{AnalysisContext, AnalysisRequest, AnalysisResult, AudioAnalyser}, processor::ProcessorContext};
use serde::{Serialize, Deserialize};

use crate::ANALYZER_ID;

#[derive(Serialize, Deserialize)]
pub struct NormalizeAnalyzerResult {
    pub peak: f32
}

#[typetag::serde]
impl AnalysisResult for NormalizeAnalyzerResult {
    fn as_any(&self) -> &dyn Any { self }
    fn get_analyzer_id(&self) -> &'static str { ANALYZER_ID }
}

#[derive(Default)]
pub struct NormalizeAnalyzer {
    peak: f32,
    hash: String
}

impl AudioAnalyser for NormalizeAnalyzer {
    fn init(&mut self, request: &AnalysisRequest) {
        self.peak = 0.0;
        self.hash = request.hash.clone();
    }

    fn analyze(
        &mut self,
        samples: &[f32],
        _time: f64,
        exhausted: bool,
        _context: &ProcessorContext,
        analysis_context: &mut AnalysisContext,
    ) {
        for &s in samples {
            let abs = s.abs();
            if abs > self.peak {
                self.peak = abs;
            }
        }

        if exhausted {
            let result = NormalizeAnalyzerResult{ peak: self.peak };
            analysis_context.results.insert(self.hash.clone(), Box::from(result));
        }
    }

    fn id(&self) -> &'static str {
        ANALYZER_ID
    }
}