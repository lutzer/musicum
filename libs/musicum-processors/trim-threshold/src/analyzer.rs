use std::any::Any;

use musicum_processor_sdk::analyzer::{AnalysisRequest, AnalysisResult, AudioAnalyser};
use musicum_processor_sdk::processor::ProcessorContext;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TrimThresholdAnalyzerResult {
    pub first_above_secs: Option<f64>,
    pub last_above_secs:  Option<f64>,
}

#[typetag::serde]
impl AnalysisResult for TrimThresholdAnalyzerResult {
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Default)]
pub struct TrimThresholdAnalyzer {
    threshold_linear: f32,
    channels:         u32,
    sample_rate:      u32,
    frames_seen:      u64,
    first_frame:      Option<u64>,
    last_frame:       Option<u64>,
    hash:             String,
}

impl AudioAnalyser for TrimThresholdAnalyzer {
    fn init(&mut self, request: &AnalysisRequest) {
        self.threshold_linear = request.params.iter()
            .find(|(k, _)| k == "threshold_linear")
            .map(|(_, v)| *v as f32)
            .unwrap_or(0.0);
        self.channels    = 0;
        self.sample_rate = 0;
        self.frames_seen = 0;
        self.first_frame = None;
        self.last_frame  = None;
        self.hash        = String::new();
    }

    fn analyze(
        &mut self,
        samples: &[f32],
        _time: f64,
        exhausted: bool,
        context: &ProcessorContext,
    ) -> Option<(String, Box<dyn AnalysisResult>)> {
        if self.channels == 0 {
            self.channels    = context.number_channels;
            self.sample_rate = context.sample_rate;
        }

        if self.channels > 0 {
            for chunk in samples.chunks_exact(self.channels as usize) {
                let frame_peak = chunk.iter().fold(0.0_f32, |m, &s| m.max(s.abs()));
                if frame_peak > self.threshold_linear {
                    if self.first_frame.is_none() {
                        self.first_frame = Some(self.frames_seen);
                    }
                    self.last_frame = Some(self.frames_seen);
                }
                self.frames_seen += 1;
            }
        }

        if exhausted {
            let sr = self.sample_rate.max(1) as f64;
            let to_secs = |f: u64| f as f64 / sr;
            let result = TrimThresholdAnalyzerResult {
                first_above_secs: self.first_frame.map(to_secs),
                last_above_secs:  self.last_frame.map(to_secs),
            };
            return Some((self.hash.clone(), Box::new(result)));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use musicum_processor_sdk::analyzer::{AnalysisRequest, AudioAnalyser};
    use musicum_processor_sdk::processor::ProcessorContext;

    fn ctx(channels: u32, sample_rate: u32) -> ProcessorContext {
        ProcessorContext { playing: false, sample_rate, number_channels: channels }
    }

    fn req(threshold_linear: f64) -> AnalysisRequest {
        AnalysisRequest {
            analyzer_id: "trim_threshold_analyzer",
            hash:    0,
            params:      vec![("threshold_linear".to_string(), threshold_linear)],
        }
    }

    fn drive(analyzer: &mut TrimThresholdAnalyzer, samples: &[f32], channels: u32, sr: u32)
        -> Option<(String, Box<dyn musicum_processor_sdk::analyzer::AnalysisResult>)>
    {
        analyzer.analyze(samples, 0.0, false, &ctx(channels, sr));
        analyzer.analyze(&[], 0.0, true, &ctx(channels, sr))
    }

    fn unwrap_result(r: Option<(String, Box<dyn musicum_processor_sdk::analyzer::AnalysisResult>)>)
        -> TrimThresholdAnalyzerResult
    {
        let (_, boxed) = r.expect("expected a result on exhausted");
        let any = boxed.as_any();
        let downcast = any.downcast_ref::<TrimThresholdAnalyzerResult>()
            .expect("wrong result type");
        TrimThresholdAnalyzerResult {
            first_above_secs: downcast.first_above_secs,
            last_above_secs:  downcast.last_above_secs,
        }
    }

    #[test]
    fn all_zero_stream_has_no_peaks() {
        let mut a = TrimThresholdAnalyzer::default();
        a.init(&req(0.1));
        let samples = vec![0.0_f32; 100 * 2];
        let r = unwrap_result(drive(&mut a, &samples, 2, 100));
        assert_eq!(r.first_above_secs, None);
        assert_eq!(r.last_above_secs,  None);
    }

    #[test]
    fn single_transient_pins_both_ends_to_same_frame() {
        let mut a = TrimThresholdAnalyzer::default();
        a.init(&req(0.5));
        let mut samples = vec![0.0_f32; 100 * 2];
        samples[25 * 2]     = 1.0;
        samples[25 * 2 + 1] = 1.0;
        let r = unwrap_result(drive(&mut a, &samples, 2, 100));
        assert_eq!(r.first_above_secs, Some(0.25));
        assert_eq!(r.last_above_secs,  Some(0.25));
    }

    #[test]
    fn two_transients_pin_first_and_last() {
        let mut a = TrimThresholdAnalyzer::default();
        a.init(&req(0.5));
        let mut samples = vec![0.0_f32; 100 * 2];
        samples[10 * 2]     =  1.0;
        samples[10 * 2 + 1] = -1.0;
        samples[80 * 2]     =  0.9;
        samples[80 * 2 + 1] = -0.9;
        let r = unwrap_result(drive(&mut a, &samples, 2, 100));
        assert_eq!(r.first_above_secs, Some(0.10));
        assert_eq!(r.last_above_secs,  Some(0.80));
    }

    #[test]
    fn multi_channel_one_channel_above_threshold_counts() {
        let mut a = TrimThresholdAnalyzer::default();
        a.init(&req(0.5));
        let mut samples = vec![0.0_f32; 50 * 2];
        samples[30 * 2 + 1] = 0.9;
        let r = unwrap_result(drive(&mut a, &samples, 2, 100));
        assert_eq!(r.first_above_secs, Some(0.30));
    }

    #[test]
    fn samples_at_or_below_threshold_are_ignored() {
        let mut a = TrimThresholdAnalyzer::default();
        a.init(&req(0.5));
        let mut samples = vec![0.0_f32; 10 * 2];
        samples[5 * 2]     = 0.5;
        samples[5 * 2 + 1] = 0.5;
        let r = unwrap_result(drive(&mut a, &samples, 2, 100));
        assert_eq!(r.first_above_secs, None);
    }

    #[test]
    fn non_exhausted_call_returns_none() {
        let mut a = TrimThresholdAnalyzer::default();
        a.init(&req(0.1));
        let samples = vec![1.0_f32; 4];
        let out = a.analyze(&samples, 0.0, false, &ctx(2, 100));
        assert!(out.is_none());
    }
}
