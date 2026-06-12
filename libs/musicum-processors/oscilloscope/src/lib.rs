use musicum_processor_sdk::{parameters::{FloatParam, ProcessorParamaterInfo}, processor::{
    BaseProcessor, ProcessorDescriptor, ProcessorType, StreamProcessor,
}};

static OSCILLOSCOPE_PARAMS: [ProcessorParamaterInfo; 3] = [
    ProcessorParamaterInfo::Float {
        id: "chunk_size", name: "Chunk Size",
        min: 32.0, max: 16348.0, default: 1024.0, step: 8.0,
        unit: "samples", editable: true,
    },
    ProcessorParamaterInfo::Float {
        id: "trigger_threshold", name: "Trigger Threshold",
        min: 0.0, max: 1.0, default: 0.0, step: 0.01,
        unit: "", editable: true,
    },
    ProcessorParamaterInfo::Canvas {
        id: "waveform", name: "Waveform", aspect_ratio: 1.0,
    },
];

static DESCRIPTOR: ProcessorDescriptor = ProcessorDescriptor {
    id: "oscilloscope",
    name: "Oscilloscope",
    processor_type: ProcessorType::StreamProcessor,
    parameters: &OSCILLOSCOPE_PARAMS,
};

pub struct OscilloscopeProcessor {
    chunk_size:        FloatParam,
    trigger_threshold: FloatParam,
    buffer:            Vec<f32>,
}

impl Default for OscilloscopeProcessor {
    fn default() -> Self {
        Self {
            chunk_size:        OSCILLOSCOPE_PARAMS[0].get_param().unwrap_or_default(),
            trigger_threshold: OSCILLOSCOPE_PARAMS[1].get_param().unwrap_or_default(),
            buffer:            Vec::new(),
        }
    }
}

impl BaseProcessor for OscilloscopeProcessor {
    fn init(
        &mut self,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
        _analysis: &mut musicum_processor_sdk::analyzer::AnalysisContext,
    ) {}

    fn descriptor(&self) -> &'static ProcessorDescriptor { &DESCRIPTOR }

    fn get_parameter(&self, id: &str) -> f64 {
        match id {
            "chunk_size"        => self.chunk_size.get() as f64,
            "trigger_threshold" => self.trigger_threshold.get() as f64,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, id: &str, value: f64) {
        match id {
            "chunk_size"        => self.chunk_size.set(value as f32),
            "trigger_threshold" => self.trigger_threshold.set(value as f32),
            _ => {}
        }
    }

    fn requires_analysis(&self) -> bool { false }
}

impl StreamProcessor for OscilloscopeProcessor {
    fn process(
        &mut self,
        samples: &mut [f32],
        _time: f64,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
    ) {
        let cap = self.chunk_size.get() as usize;
        self.buffer.extend_from_slice(samples);
        if self.buffer.len() > cap {
            let excess = self.buffer.len() - cap;
            self.buffer.drain(..excess);
        }
    }
}

musicum_processor_sdk::export_processor!(OscilloscopeProcessor, Stream);

#[cfg(test)]
mod tests {
    use super::*;
    use musicum_processor_sdk::processor::ProcessorContext;

    fn ctx(channels: u32) -> ProcessorContext {
        ProcessorContext { playing: true, sample_rate: 44100, number_channels: channels }
    }

    #[test]
    fn process_is_bit_exact_passthrough_stereo() {
        let mut p = OscilloscopeProcessor::default();
        let mut s = vec![0.5_f32, -0.5, 1.0, -1.0];
        let expected = s.clone();
        p.process(&mut s, 0.0, &ctx(2));
        assert_eq!(s, expected);
    }

    #[test]
    fn process_is_bit_exact_passthrough_mono() {
        let mut p = OscilloscopeProcessor::default();
        let mut s = vec![0.1_f32, 0.2, 0.3, 0.4, 0.5];
        let expected = s.clone();
        p.process(&mut s, 0.0, &ctx(1));
        assert_eq!(s, expected);
    }

    #[test]
    fn chunk_size_clamps() {
        let mut p = OscilloscopeProcessor::default();
        p.set_parameter("chunk_size", 1.0);
        assert!((p.get_parameter("chunk_size") - 32.0).abs() < 1e-6);
        p.set_parameter("chunk_size", 999_999.0);
        assert!((p.get_parameter("chunk_size") - 16348.0).abs() < 1e-6);
    }

    #[test]
    fn trigger_threshold_clamps() {
        let mut p = OscilloscopeProcessor::default();
        p.set_parameter("trigger_threshold", 99.0);
        assert!((p.get_parameter("trigger_threshold") - 1.0).abs() < 1e-6);
        p.set_parameter("trigger_threshold", -1.0);
        assert!((p.get_parameter("trigger_threshold") - 0.0).abs() < 1e-6);
    }
}
