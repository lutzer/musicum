use musicum_processor_sdk::{analyzer::{AnalysisRequest}, parameters::{FloatParam, ProcessorParamaterInfo}, processor::{
    BaseProcessor, ProcessorDescriptor, ProcessorMeta, ProcessorType,
}};

use crate::analyzer::NormalizeAnalyzerResult;

pub mod analyzer;
pub use analyzer::NormalizeAnalyzer;

pub static ANALYZER_ID : &'static str = "normalize_analyzer";

static NORMALIZE_PARAMS: [ProcessorParamaterInfo; 2] = [
    ProcessorParamaterInfo::Float {
        id: "target_dbfs", name: "Target dBFS",
        min: -40.0, max: 0.0, default: -3.0, step: 0.5,
        unit: "dBFS", editable: true,
    },
    ProcessorParamaterInfo::Float {
        id: "computed_gain", name: "Computed Gain",
        min: 0.0, max: 100.0, default: 1.0, step: 0.001,
        unit: "x", editable: false,
    },
];

static DESCRIPTOR: ProcessorDescriptor = ProcessorDescriptor {
    id: "normalize",
    name: "Normalize",
    processor_type: ProcessorType::StreamProcessor,
    parameters: &NORMALIZE_PARAMS,
};

pub struct NormalizeProcessor {
    target_dbfs: FloatParam,
    gain:        FloatParam,
    requires_analysis: bool,
    uuid: String
}

impl Default for NormalizeProcessor {
    fn default() -> Self {
        Self {
            target_dbfs: NORMALIZE_PARAMS[0].get_param().unwrap_or_default(),
            gain: NORMALIZE_PARAMS[1].get_param().unwrap_or_default(),
            uuid: String::new(),
            requires_analysis: false
        }
    }
}

impl BaseProcessor for NormalizeProcessor {
    fn init(
        &mut self,
        uuid: String,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
        analysis_context: &mut musicum_processor_sdk::analyzer::AnalysisContext,
    ) {
        self.uuid = uuid;
        let analysis_hash = self.analysis_hash();

        if let Some(result) = analysis_context.get_result::<NormalizeAnalyzerResult>(&analysis_hash) {
            let target_linear = 10_f32.powf(self.target_dbfs.get() / 20.0);
            self.gain.set(target_linear / (result.peak).max(f32::EPSILON));
            self.requires_analysis = false;
        } else {
            analysis_context.requests.push(
                AnalysisRequest{
                    analyzer_id: ANALYZER_ID,
                    hash: analysis_hash,
                    params: vec![]
                }
            );
            self.requires_analysis = true;
        }
    }

    fn get_parameter(&self, id: &str) -> f64 {
        match id {
            "target_dbfs" => self.target_dbfs.get() as f64,
            "computed_gain" => self.gain.get() as f64,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, id: &str, value: f64) {
        match id {
            "target_dbfs" => self.target_dbfs.set(value as f32),
            _ => {}
        }
    }

    fn requires_analysis(&self) -> bool { self.requires_analysis }

    fn analysis_hash(&self) -> String {
        return AnalysisRequest::generate_hash_from(&self.uuid, &vec![])
    }

    fn process(
        &mut self,
        samples: &mut [f32],
        _time: f64,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
    ) {
        let g = self.gain.get();
        for s in samples.iter_mut() {
            *s *= g;
        }
    }
}

impl ProcessorMeta for NormalizeProcessor {
    fn descriptor() -> &'static ProcessorDescriptor { &DESCRIPTOR }
}

musicum_processor_sdk::export_processor!(
    NormalizeProcessor,
    with: NormalizeAnalyzer = ANALYZER_ID
);

#[cfg(test)]
mod tests {
    use super::*;
    use musicum_processor_sdk::processor::ProcessorContext;

    fn ctx() -> ProcessorContext {
        ProcessorContext { playing: true, sample_rate: 44100, number_channels: 2 }
    }

    #[test]
    fn default_state_is_passthrough() {
        let mut p = NormalizeProcessor::default();
        let mut s = vec![0.5_f32, -0.5, 1.0, -1.0];
        let expected = s.clone();
        p.process(&mut s, 0.0, &ctx());
        for (a, b) in s.iter().zip(expected.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn setting_gain_scales_samples_linearly() {
        let mut p = NormalizeProcessor::default();
        p.set_parameter("gain", 2.0);
        let mut s = vec![0.25_f32, -0.5];
        p.process(&mut s, 0.0, &ctx());
        assert!((s[0] - 0.5).abs() < 1e-6);
        assert!((s[1] - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn target_dbfs_clamps() {
        let mut p = NormalizeProcessor::default();
        p.set_parameter("target_dbfs", 99.0);
        assert_eq!(p.get_parameter("target_dbfs"), 0.0);
        p.set_parameter("target_dbfs", -999.0);
        assert_eq!(p.get_parameter("target_dbfs"), -40.0);
    }

    #[test]
    fn gain_clamps() {
        let mut p = NormalizeProcessor::default();
        p.set_parameter("computed_gain", 9999.0);
        assert_eq!(p.get_parameter("computed_gain"), 100.0);
        p.set_parameter("computed_gain", -1.0);
        assert_eq!(p.get_parameter("computed_gain"), 0.0);
    }

    /// Pins current behavior: the SDK does not enforce `editable` at the
    /// value-setter level, so `set_parameter("gain", ...)` writes even though
    /// the descriptor marks `gain` as `editable: false`.
    #[test]
    fn set_parameter_gain_writes_despite_not_editable() {
        let mut p = NormalizeProcessor::default();
        p.set_parameter("computed_gain", 2.5);
        assert!((p.get_parameter("computed_gain") - 2.5).abs() < 1e-6);
    }

    #[test]
    fn unknown_parameter_returns_zero() {
        let p = NormalizeProcessor::default();
        assert_eq!(p.get_parameter("unknown"), 0.0);
    }
}
