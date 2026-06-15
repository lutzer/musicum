use musicum_processor_sdk::{
    analyzer::AnalysisRequest,
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
    threshold:         FloatParam,
    offset:            FloatParam,
    detected_start:    FloatParam,
    detected_end:      FloatParam,
    peaks_found:       BoolParam,
    requires_analysis: bool,
    uuid:              String,
}

impl Default for TrimThresholdProcessor {
    fn default() -> Self {
        Self {
            threshold:         TRIM_THRESHOLD_PARAMS[0].get_param().unwrap_or_default(),
            offset:            TRIM_THRESHOLD_PARAMS[1].get_param().unwrap_or_default(),
            detected_start:    TRIM_THRESHOLD_PARAMS[2].get_param().unwrap_or_default(),
            detected_end:      TRIM_THRESHOLD_PARAMS[3].get_param().unwrap_or_default(),
            peaks_found:       TRIM_THRESHOLD_PARAMS[4].get_param().unwrap_or_default(),
            requires_analysis: false,
            uuid:              String::new(),
        }
    }
}

impl BaseProcessor for TrimThresholdProcessor {
    fn init(
        &mut self,
        uuid: String,
        _context: &ProcessorContext,
        analysis_context: &mut musicum_processor_sdk::analyzer::AnalysisContext,
    ) {
        self.uuid = uuid;
        let linear = 10_f32.powf(self.threshold.get() / 20.0);
        let hash = format!("{}:{:.6}", self.uuid, linear);

        if let Some(result) = analysis_context.get_result::<TrimThresholdAnalyzerResult>(&hash) {
            if let (Some(first), Some(last)) = (result.first_above_secs, result.last_above_secs) {
                self.detected_start.set(first as f32);
                self.detected_end.set(last as f32);
                self.peaks_found.set(true);
            } else {
                self.peaks_found.set(false);
            }
            self.requires_analysis = false;
        } else {
            analysis_context.requests.push(AnalysisRequest {
                analyzer_id: ANALYZER_ID,
                hash,
                params: vec![("threshold_linear".to_string(), linear as f64)],
            });
            self.peaks_found.set(false);
            self.requires_analysis = true;
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

    fn requires_analysis(&self) -> bool { self.requires_analysis }

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
    use musicum_processor_sdk::analyzer::AnalysisContext;
    use musicum_processor_sdk::processor::{BaseProcessor, ProcessorContext, ProcessorMeta};
    use crate::analyzer::TrimThresholdAnalyzerResult;

    fn ctx() -> ProcessorContext {
        ProcessorContext { playing: false, sample_rate: 44100, number_channels: 2 }
    }

    /// Build a processor and feed it a cached analyzer result so segments()
    /// can be exercised without the (currently todo!()) chain analyzer pipeline.
    fn primed(threshold_dbfs: f32, offset_secs: f32, first: Option<f64>, last: Option<f64>)
        -> TrimThresholdProcessor
    {
        let uuid = "test-uuid".to_string();
        let linear = 10_f32.powf(threshold_dbfs / 20.0);
        let hash = format!("{uuid}:{linear:.6}");

        let mut analysis = AnalysisContext::default();
        analysis.results.insert(
            hash.clone(),
            Box::new(TrimThresholdAnalyzerResult {
                first_above_secs: first,
                last_above_secs:  last,
            }),
        );

        let mut p = TrimThresholdProcessor::default();
        p.set_parameter("threshold", threshold_dbfs as f64);
        p.set_parameter("offset",    offset_secs as f64);
        p.init(uuid, &ctx(), &mut analysis);
        p
    }

    #[test]
    fn descriptor_identity() {
        let d = TrimThresholdProcessor::descriptor();
        assert_eq!(d.id, "trim_threshold");
        assert_eq!(d.name, "Trim Threshold");
    }

    #[test]
    fn default_state_has_no_peaks_and_empty_segments() {
        let p = TrimThresholdProcessor::default();
        assert!(p.segments(10.0, &ctx()).is_empty());
    }

    #[test]
    fn init_without_cached_result_requests_analysis() {
        let mut p = TrimThresholdProcessor::default();
        let mut analysis = AnalysisContext::default();
        p.init("uuid".to_string(), &ctx(), &mut analysis);
        assert!(p.requires_analysis());
        assert_eq!(analysis.requests.len(), 1);
        assert_eq!(analysis.requests[0].analyzer_id, "trim_threshold_analyzer");
        assert!(analysis.requests[0].hash.starts_with("uuid:"));
        let has_param = analysis.requests[0].params.iter()
            .any(|(k, _)| k == "threshold_linear");
        assert!(has_param);
    }

    #[test]
    fn init_with_cached_peaks_populates_detected_and_clears_requires_analysis() {
        let p = primed(-40.0, 0.5, Some(2.0), Some(8.0));
        assert!(!p.requires_analysis());
        assert_eq!(p.get_parameter("detected_start"), 2.0);
        assert_eq!(p.get_parameter("detected_end"),   8.0);
        assert_eq!(p.get_parameter("peaks_found"),    1.0);
    }

    #[test]
    fn init_with_cached_no_peaks_sets_peaks_found_false() {
        let p = primed(-40.0, 0.5, None, None);
        assert!(!p.requires_analysis());
        assert_eq!(p.get_parameter("peaks_found"), 0.0);
        assert!(p.segments(10.0, &ctx()).is_empty());
    }

    #[test]
    fn segments_expand_around_detected_range() {
        let p = primed(-40.0, 0.5, Some(2.0), Some(8.0));
        let segs = p.segments(10.0, &ctx());
        assert_eq!(segs.len(), 1);
        assert!((segs[0].src_start - 1.5).abs() < 1e-9);
        assert!((segs[0].src_end   - 8.5).abs() < 1e-9);
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
    fn threshold_change_produces_different_request_hash() {
        let mut p = TrimThresholdProcessor::default();
        p.set_parameter("threshold", -20.0);
        let mut a1 = AnalysisContext::default();
        p.init("uuid".to_string(), &ctx(), &mut a1);
        let hash1 = a1.requests[0].hash.clone();

        p.set_parameter("threshold", -40.0);
        let mut a2 = AnalysisContext::default();
        p.init("uuid".to_string(), &ctx(), &mut a2);
        let hash2 = a2.requests[0].hash.clone();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn dbfs_to_linear_smoke() {
        let mut p = TrimThresholdProcessor::default();
        p.set_parameter("threshold", -20.0);
        let mut a = AnalysisContext::default();
        p.init("uuid".to_string(), &ctx(), &mut a);
        let linear = a.requests[0].params.iter()
            .find(|(k, _)| k == "threshold_linear").unwrap().1;
        assert!((linear - 0.1).abs() < 1e-3);
    }

    #[test]
    fn get_parameter_unknown_returns_zero() {
        let p = TrimThresholdProcessor::default();
        assert_eq!(p.get_parameter("nope"), 0.0);
    }
}
