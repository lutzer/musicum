use std::any::Any;

use musicum_processor_sdk::{analyzer::{AnalysisContext, AnalysisRequest, AnalysisResult, AudioAnalyser}, processor::ProcessorContext};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct NormalizeAnalyzerResult {
    peak: f32
}

#[typetag::serde]
impl AnalysisResult for NormalizeAnalyzerResult {
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Default)]
pub struct NormalizeAnalyzer {
    peak: f32,
    target_dbfs: f32,
    processor_id: &'static str
}

impl AudioAnalyser for NormalizeAnalyzer {
    fn init(&mut self, request: &AnalysisRequest) {
        self.peak = 0.0;
        self.processor_id = request.processor_uuid;

        // sets parameters for analysis
        request.params.iter().for_each(|(key,value)| {
            if key == "target_dbfs" {
                self.target_dbfs = *value as f32;
            }
        })
    }

    fn analyze(
        &mut self,
        samples: &[f32],
        time: f64,
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
            analysis_context.results.insert(self.processor_id, Box::from(result));
        }
    }

    fn id(&self) -> &'static str {
        "normalize_analyzer"
    }
}