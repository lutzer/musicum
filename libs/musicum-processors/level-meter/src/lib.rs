use musicum_processor_sdk::{parameters::ProcessorParamaterInfo, processor::{
    BaseProcessor, ProcessorDescriptor, ProcessorType, StreamProcessor,
}};

static LEVEL_METER_PARAMS: [ProcessorParamaterInfo; 2] = [
    ProcessorParamaterInfo::Canvas { id: "level_left",  name: "Left",  aspect_ratio: 10.0 },
    ProcessorParamaterInfo::Canvas { id: "level_right", name: "Right", aspect_ratio: 10.0 },
];

static DESCRIPTOR: ProcessorDescriptor = ProcessorDescriptor {
    id: "level-meter",
    name: "Level Meter",
    processor_type: ProcessorType::StreamProcessor,
    parameters: &LEVEL_METER_PARAMS,
};

const HOLD_DURATION: f64 = 1.5;

pub struct LevelMeterProcessor {
    left_peak:       f32,
    right_peak:      f32,
    left_hold:       f32,
    right_hold:      f32,
    left_hold_time:  f64,
    right_hold_time: f64,
}

impl Default for LevelMeterProcessor {
    fn default() -> Self {
        Self {
            left_peak: 0.0, right_peak: 0.0,
            left_hold: 0.0, right_hold: 0.0,
            left_hold_time: 0.0, right_hold_time: 0.0,
        }
    }
}

impl BaseProcessor for LevelMeterProcessor {
    fn prepare(
        &mut self,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
        _ctx: &mut musicum_processor_sdk::analyzer::AnalysisContext,
    ) {}

    fn descriptor(&self) -> &'static ProcessorDescriptor { &DESCRIPTOR }

    fn get_parameter(&self, _id: &str) -> f64 { 0.0 }
    fn set_parameter(&mut self, _id: &str, _value: f64) {}
}

impl StreamProcessor for LevelMeterProcessor {
    fn process(
        &mut self,
        samples: &mut [f32],
        time: f64,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
    ) {
        let mut left_peak  = 0.0_f32;
        let mut right_peak = 0.0_f32;

        for (i, &s) in samples.iter().enumerate() {
            let abs = s.abs();
            if i % 2 == 0 {
                left_peak = left_peak.max(abs);
            } else {
                right_peak = right_peak.max(abs);
            }
        }

        self.left_peak  = left_peak;
        self.right_peak = right_peak;

        if left_peak >= self.left_hold {
            self.left_hold = left_peak;
            self.left_hold_time = time;
        } else if time - self.left_hold_time > HOLD_DURATION {
            self.left_hold = left_peak;
        }

        if right_peak >= self.right_hold {
            self.right_hold = right_peak;
            self.right_hold_time = time;
        } else if time - self.right_hold_time > HOLD_DURATION {
            self.right_hold = right_peak;
        }
    }
}

musicum_processor_sdk::export_processor!(LevelMeterProcessor, Stream);

#[cfg(test)]
mod tests {
    use super::*;
    use musicum_processor_sdk::processor::ProcessorContext;

    fn ctx(channels: u32) -> ProcessorContext {
        ProcessorContext { playing: true, sample_rate: 44100, number_channels: channels }
    }

    #[test]
    fn process_is_bit_exact_passthrough_stereo() {
        let mut p = LevelMeterProcessor::default();
        let mut s = vec![0.5_f32, -0.3, 0.2, 0.8];
        let expected = s.clone();
        p.process(&mut s, 0.0, &ctx(2));
        assert_eq!(s, expected);
    }

    #[test]
    fn process_is_bit_exact_passthrough_mono() {
        let mut p = LevelMeterProcessor::default();
        let mut s = vec![0.1_f32, 0.2, 0.3];
        let expected = s.clone();
        p.process(&mut s, 0.0, &ctx(1));
        assert_eq!(s, expected);
    }

    #[test]
    fn process_does_not_panic_on_empty_buffer() {
        let mut p = LevelMeterProcessor::default();
        let mut s: Vec<f32> = vec![];
        p.process(&mut s, 0.0, &ctx(2));
    }
}
