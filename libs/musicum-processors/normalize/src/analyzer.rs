use std::any::Any;

use musicum_processor_sdk::{analyzer::{AnalysisRequest, AnalysisResult, AudioAnalyser}, processor::ProcessorContext};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct NormalizeAnalyzerResult {
    pub peak: f32
}

#[typetag::serde]
impl AnalysisResult for NormalizeAnalyzerResult {
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Default)]
pub struct NormalizeAnalyzer {
    peak: f32,
}

impl AudioAnalyser for NormalizeAnalyzer {
    fn init(&mut self, _request: &AnalysisRequest) {
        self.peak = 0.0;
    }

    fn analyze(
        &mut self,
        samples: &[f32],
        _time: f64,
        exhausted: bool,
        _context: &ProcessorContext
    ) -> Option<Box<dyn AnalysisResult>> {
        for &s in samples {
            let abs = s.abs();
            if abs > self.peak {
                self.peak = abs;
            }
        }

        if exhausted {
            let result = NormalizeAnalyzerResult{ peak: self.peak };
            return Some(Box::from(result))
        }
        None
    }
}