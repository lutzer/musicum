use musicum_processor_sdk::{
    analyzer::{AnalysisRequest, AnalysisResult},
    fingerprint::Fingerprint,
    parameters::{BoolParam, FloatParam, ProcessorParamaterInfo},
    processor::{
        BaseProcessor, ProcessorContext, ProcessorDescriptor, ProcessorMeta, ProcessorType,
        Segment,
    },
};

use crate::analyzer::{TrimThresholdAnalyzer, TrimThresholdAnalyzerResult};

pub static ANALYZER_ID: &str = "trim_threshold_analyzer";

static TRIM_THRESHOLD_PARAMS: [ProcessorParamaterInfo; 5] = [
    ProcessorParamaterInfo::Float {
        id: "threshold", name: "Threshold",
        min: -60.0, max: 0.0, default: -40.0, step: 0.5,
        unit: "dBFS", editable: true,
    },
    ProcessorParamaterInfo::Float {
        id: "offset", name: "Offset",
        min: 0.0, max: 10.0, default: 0.1, step: 0.01,
        unit: "s", editable: true,
    },
    ProcessorParamaterInfo::Float {
        id: "detected_start", name: "Detected Start",
        min: 0.0, max: 86400.0, default: 0.0, step: 0.001,
        unit: "s", editable: false,
    },
    ProcessorParamaterInfo::Float {
        id: "detected_end", name: "Detected End",
        min: 0.0, max: 86400.0, default: 0.0, step: 0.001,
        unit: "s", editable: false,
    },
    ProcessorParamaterInfo::Bool {
        id: "peaks_found", name: "Peaks Found",
        default: false, editable: false,
    },
];

static DESCRIPTOR: ProcessorDescriptor = ProcessorDescriptor {
    id: "trim_threshold",
    name: "Trim Threshold",
    processor_type: ProcessorType::StructuralProcessor,
    parameters: &TRIM_THRESHOLD_PARAMS,
};

pub struct TrimThresholdProcessor {
    threshold:      FloatParam,
    offset:         FloatParam,
    detected_start: FloatParam,
    detected_end:   FloatParam,
    peaks_found:    BoolParam,
}

impl Default for TrimThresholdProcessor {
    fn default() -> Self {
        Self {
            threshold:      TRIM_THRESHOLD_PARAMS[0].get_param().unwrap_or_default(),
            offset:         TRIM_THRESHOLD_PARAMS[1].get_param().unwrap_or_default(),
            detected_start: TRIM_THRESHOLD_PARAMS[2].get_param().unwrap_or_default(),
            detected_end:   TRIM_THRESHOLD_PARAMS[3].get_param().unwrap_or_default(),
            peaks_found:    TRIM_THRESHOLD_PARAMS[4].get_param().unwrap_or_default(),
        }
    }
}

impl BaseProcessor for TrimThresholdProcessor {
    fn init(&mut self, _uuid: String, _ctx: &ProcessorContext) {}

    fn request_analysis(&self, _ctx: &ProcessorContext) -> Option<AnalysisRequest> {
        let linear = 10_f32.powf(self.threshold.get() / 20.0);
        Some(AnalysisRequest {
            analyzer_id: ANALYZER_ID,
            slot_key:    Fingerprint::of_f32(linear),
            params:      vec![("threshold_linear".into(), linear as f64)],
        })
    }

    fn apply_analysis(&mut self, result: &dyn AnalysisResult) {
        let Some(r) = result.as_any().downcast_ref::<TrimThresholdAnalyzerResult>() else { return };
        match (r.first_above_secs, r.last_above_secs) {
            (Some(first), Some(last)) => {
                self.detected_start.set(first as f32);
                self.detected_end.set(last as f32);
                self.peaks_found.set(true);
            }
            _ => self.peaks_found.set(false),
        }
    }

    fn get_parameter(&self, id: &str) -> f64 {
        match id {
            "threshold"      => self.threshold.get() as f64,
            "offset"         => self.offset.get() as f64,
            "detected_start" => self.detected_start.get() as f64,
            "detected_end"   => self.detected_end.get() as f64,
            "peaks_found"    => self.peaks_found.get() as f64,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, id: &str, value: f64) {
        match id {
            "threshold"      => self.threshold.set(value as f32),
            "offset"         => self.offset.set(value as f32),
            "detected_start" => self.detected_start.set(value as f32),
            "detected_end"   => self.detected_end.set(value as f32),
            "peaks_found"    => self.peaks_found.set(value >= 0.5),
            _ => {}
        }
    }

    fn segments(
        &self,
        duration: f64,
        _context: &ProcessorContext,
    ) -> Vec<Segment> {
        if !self.peaks_found.get_bool() {
            return vec![Segment { src_start: 0.0, src_end: duration, rate: 1.0 }];
        }
        let offset = self.offset.get();
        let start = (self.detected_start.get() - offset).max(0.0) as f64;
        let end   = ((self.detected_end.get()   + offset) as f64).min(duration);
        if end <= start {
            return vec![];
        }
        vec![Segment { src_start: start, src_end: end, rate: 1.0 }]
    }
}

impl ProcessorMeta for TrimThresholdProcessor {
    fn descriptor() -> &'static ProcessorDescriptor { &DESCRIPTOR }
}

pub mod analyzer;

musicum_processor_sdk::export_processor!(
    TrimThresholdProcessor,
    with: TrimThresholdAnalyzer = ANALYZER_ID
);

#[cfg(test)]
mod tests {
    use super::*;
    use musicum_processor_sdk::processor::{BaseProcessor, ProcessorContext, ProcessorMeta};
    use crate::analyzer::TrimThresholdAnalyzerResult;

    fn ctx() -> ProcessorContext {
        ProcessorContext { playing: false, sample_rate: 44100, number_channels: 2 }
    }

    /// Build a processor and feed it an analyzer result so segments() can be
    /// exercised without driving the offline analysis pipeline.
    fn primed(threshold_dbfs: f32, offset_secs: f32, first: Option<f64>, last: Option<f64>)
        -> TrimThresholdProcessor
    {
        let mut p = TrimThresholdProcessor::default();
        p.set_parameter("threshold", threshold_dbfs as f64);
        p.set_parameter("offset",    offset_secs as f64);
        p.init("test-uuid".into(), &ctx());
        let result = TrimThresholdAnalyzerResult {
            first_above_secs: first,
            last_above_secs:  last,
        };
        p.apply_analysis(&result);
        p
    }

    #[test]
    fn descriptor_identity() {
        let d = TrimThresholdProcessor::descriptor();
        assert_eq!(d.id, "trim_threshold");
        assert_eq!(d.name, "Trim Threshold");
    }

    #[test]
    fn default_state_has_no_peaks_and_passes_through() {
        // Default `peaks_found = false` means segments() now returns full
        // pass-through rather than an empty Vec, because no peaks have been
        // detected yet — the chain shouldn't drop everything before analysis
        // runs.
        let p = TrimThresholdProcessor::default();
        let segs = p.segments(10.0, &ctx());
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].src_start, 0.0);
        assert_eq!(segs[0].src_end,   10.0);
    }

    #[test]
    fn request_analysis_carries_threshold_linear() {
        let p = TrimThresholdProcessor::default();
        let req: AnalysisRequest = p.request_analysis(&ctx()).expect("request");
        assert_eq!(req.analyzer_id, ANALYZER_ID);
        let has_param = req.params.iter().any(|(k, _)| k == "threshold_linear");
        assert!(has_param);
    }

    #[test]
    fn threshold_change_produces_different_slot_key() {
        let mut p = TrimThresholdProcessor::default();
        p.set_parameter("threshold", -20.0);
        let k1 = p.request_analysis(&ctx()).unwrap().slot_key;
        p.set_parameter("threshold", -40.0);
        let k2 = p.request_analysis(&ctx()).unwrap().slot_key;
        assert_ne!(k1, k2);
    }

    #[test]
    fn apply_analysis_with_peaks_populates_detected_and_segments() {
        let p = primed(-40.0, 0.5, Some(2.0), Some(8.0));
        assert_eq!(p.get_parameter("detected_start"), 2.0);
        assert_eq!(p.get_parameter("detected_end"),   8.0);
        assert_eq!(p.get_parameter("peaks_found"),    1.0);
        let segs = p.segments(10.0, &ctx());
        assert_eq!(segs.len(), 1);
        assert!((segs[0].src_start - 1.5).abs() < 1e-9);
        assert!((segs[0].src_end   - 8.5).abs() < 1e-9);
    }

    #[test]
    fn apply_analysis_with_no_peaks_sets_peaks_found_false() {
        let p = primed(-40.0, 0.5, None, None);
        assert_eq!(p.get_parameter("peaks_found"), 0.0);
        // peaks_found=false → segments() now passes through
        let segs = p.segments(10.0, &ctx());
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn offset_clamps_to_file_bounds() {
        let p = primed(-40.0, 3.0, Some(2.0), Some(8.0));
        let segs = p.segments(10.0, &ctx());
        assert_eq!(segs.len(), 1);
        assert!((segs[0].src_start - 0.0).abs()  < 1e-9);
        assert!((segs[0].src_end   - 10.0).abs() < 1e-9);
    }

    #[test]
    fn single_frame_peak_at_zero_still_produces_a_segment() {
        let p = primed(-40.0, 0.5, Some(0.0), Some(0.0));
        let segs = p.segments(10.0, &ctx());
        assert_eq!(segs.len(), 1);
        assert!((segs[0].src_start - 0.0).abs() < 1e-9);
        assert!((segs[0].src_end   - 0.5).abs() < 1e-9);
    }

    #[test]
    fn dbfs_to_linear_smoke() {
        let mut p = TrimThresholdProcessor::default();
        p.set_parameter("threshold", -20.0);
        let req = p.request_analysis(&ctx()).expect("request");
        let linear = req.params.iter()
            .find(|(k, _)| k == "threshold_linear").unwrap().1;
        assert!((linear - 0.1).abs() < 1e-3);
    }

    #[test]
    fn get_parameter_unknown_returns_zero() {
        let p = TrimThresholdProcessor::default();
        assert_eq!(p.get_parameter("nope"), 0.0);
    }
}
