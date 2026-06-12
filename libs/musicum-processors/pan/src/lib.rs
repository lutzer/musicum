use musicum_processor_sdk::{parameters::{FloatParam, ProcessorParamaterInfo}, processor::{
    BaseProcessor, ProcessorDescriptor, ProcessorType, StreamProcessor,
}};

static PAN_PARAMS: [ProcessorParamaterInfo; 2] = [
    ProcessorParamaterInfo::Float {
        id: "pan", name: "Pan", min: -1.0, max: 1.0, default: 0.0,
        step: 0.01, unit: "", editable: true,
    },
    ProcessorParamaterInfo::Float {
        id: "width", name: "Width", min: 0.0, max: 2.0, default: 1.0,
        step: 0.01, unit: "", editable: true,
    },
];

static DESCRIPTOR: ProcessorDescriptor = ProcessorDescriptor {
    id: "pan",
    name: "Pan / Width",
    processor_type: ProcessorType::StreamProcessor,
    parameters: &PAN_PARAMS,
};

pub struct PanProcessor {
    pan: FloatParam,
    width: FloatParam,
}

impl Default for PanProcessor {
    fn default() -> Self {
        Self {
            pan:   PAN_PARAMS[0].get_param().unwrap_or_default(),
            width: PAN_PARAMS[1].get_param().unwrap_or_default(),
        }
    }
}

impl BaseProcessor for PanProcessor {
    fn init(
        &mut self,
        _uuid: String,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
        _ctx: &mut musicum_processor_sdk::analyzer::AnalysisContext,
    ) {}

    fn descriptor(&self) -> &'static ProcessorDescriptor { &DESCRIPTOR }

    fn get_parameter(&self, id: &str) -> f64 {
        match id {
            "pan"   => self.pan.get() as f64,
            "width" => self.width.get() as f64,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, id: &str, value: f64) {
        match id {
            "pan"   => self.pan.set(value as f32),
            "width" => self.width.set(value as f32),
            _ => {}
        }
    }

    fn requires_analysis(&self) -> bool { false }
}

impl StreamProcessor for PanProcessor {
    fn process(
        &mut self,
        samples: &mut [f32],
        _time: f64,
        context: &musicum_processor_sdk::processor::ProcessorContext,
    ) {
        let channels = context.number_channels as usize;

        if channels < 2 {
            let gain = 1.0 - self.pan.get().abs();
            for s in samples.iter_mut() {
                *s *= gain;
            }
            return;
        }

        let left_gain  = (1.0 - self.pan.get()).clamp(0.0, 1.0);
        let right_gain = (1.0 + self.pan.get()).clamp(0.0, 1.0);

        let frames = samples.len() / channels;
        for f in 0..frames {
            let l = samples[f * channels];
            let r = samples[f * channels + 1];

            let mid  = (l + r) * 0.5;
            let side = (l - r) * 0.5 * self.width.get();

            samples[f * channels]     = (mid + side) * left_gain;
            samples[f * channels + 1] = (mid - side) * right_gain;
        }
    }
}

musicum_processor_sdk::export_processor!(PanProcessor, Stream);

#[cfg(test)]
mod tests {
    use super::*;
    use musicum_processor_sdk::processor::ProcessorContext;

    fn ctx(channels: u32) -> ProcessorContext {
        ProcessorContext { playing: true, sample_rate: 44100, number_channels: channels }
    }

    fn process_stereo(p: &mut PanProcessor, l: f32, r: f32) -> (f32, f32) {
        let mut s = vec![l, r];
        p.process(&mut s, 0.0, &ctx(2));
        (s[0], s[1])
    }

    #[test]
    fn centre_pan_unity_width_is_passthrough() {
        let mut p = PanProcessor::default();
        let (l, r) = process_stereo(&mut p, 0.8, -0.4);
        assert!((l - 0.8).abs() < 1e-6);
        assert!((r - (-0.4)).abs() < 1e-6);
    }

    #[test]
    fn full_right_pan_silences_left() {
        let mut p = PanProcessor::default();
        p.set_parameter("pan", 1.0);
        let (l, r) = process_stereo(&mut p, 0.5, 0.5);
        assert!(l.abs() < 1e-6);
        assert!((r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn full_left_pan_silences_right() {
        let mut p = PanProcessor::default();
        p.set_parameter("pan", -1.0);
        let (l, r) = process_stereo(&mut p, 0.5, 0.5);
        assert!((l - 0.5).abs() < 1e-6);
        assert!(r.abs() < 1e-6);
    }

    #[test]
    fn zero_width_produces_mono() {
        let mut p = PanProcessor::default();
        p.set_parameter("width", 0.0);
        let (l, r) = process_stereo(&mut p, 0.6, 0.2);
        let mid = (0.6 + 0.2) * 0.5;
        assert!((l - mid).abs() < 1e-6);
        assert!((r - mid).abs() < 1e-6);
    }

    #[test]
    fn double_width_widens_stereo() {
        let mut p = PanProcessor::default();
        p.set_parameter("width", 2.0);
        let (l, r) = process_stereo(&mut p, 1.0, 0.0);
        assert!((l - 1.5).abs() < 1e-6);
        assert!((r - (-0.5)).abs() < 1e-6);
    }

    #[test]
    fn mono_input_centre_pan_is_passthrough() {
        let mut p = PanProcessor::default();
        let mut s = vec![0.7_f32, 0.3, -0.5];
        let expected = s.clone();
        p.process(&mut s, 0.0, &ctx(1));
        for (a, b) in s.iter().zip(expected.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn mono_input_full_pan_silences() {
        let mut p = PanProcessor::default();
        p.set_parameter("pan", 1.0);
        let mut s = vec![1.0_f32, -1.0];
        p.process(&mut s, 0.0, &ctx(1));
        assert!(s.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn parameters_clamp() {
        let mut p = PanProcessor::default();
        p.set_parameter("pan", 99.0);
        assert_eq!(p.get_parameter("pan"), 1.0);
        p.set_parameter("pan", -99.0);
        assert_eq!(p.get_parameter("pan"), -1.0);
        p.set_parameter("width", -1.0);
        assert_eq!(p.get_parameter("width"), 0.0);
        p.set_parameter("width", 99.0);
        assert_eq!(p.get_parameter("width"), 2.0);
    }

    #[test]
    fn unknown_parameter_returns_zero() {
        let p = PanProcessor::default();
        assert_eq!(p.get_parameter("unknown"), 0.0);
    }
}
