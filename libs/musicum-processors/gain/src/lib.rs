use musicum_processor_sdk::processor::{
    BaseProcessor, StreamProcessor, ProcessorDescriptor, ProcessorType, ProcessorParamaterInfo,
};

static GAIN_PARAMS: [ProcessorParamaterInfo; 1] = [ProcessorParamaterInfo::Float {
    id: "gain",
    name: "Gain",
    min: 0.0,
    max: 4.0,
    default: 1.0,
    step: 0.01,
    unit: "x",
    editable: true,
}];

static DESCRIPTOR: ProcessorDescriptor = ProcessorDescriptor {
    id: "gain_plugin",
    name: "Gain",
    processor_type: ProcessorType::StreamProcessor,
    parameters: &GAIN_PARAMS,
};

pub struct GainPlugin {
    gain: f32,
}

impl Default for GainPlugin {
    fn default() -> Self {
        Self { gain: 1.0 }
    }
}

impl BaseProcessor for GainPlugin {
    fn prepare(
        &mut self,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
        _ctx: &mut musicum_processor_sdk::analyzer::AnalysisContext,
    ) {
    }

    fn descriptor(&self) -> &'static ProcessorDescriptor {
        &DESCRIPTOR
    }

    fn get_parameter(&self, id: &str) -> f64 {
        if id == "gain" { self.gain as f64 } else { 0.0 }
    }

    fn set_parameter(&mut self, id: &str, value: f64) {
        if id == "gain" {
            self.gain = value as f32;
        }
    }

    fn requires_analysis(&self) -> bool {
        false
    }
}

impl StreamProcessor for GainPlugin {
    fn process(
        &mut self,
        sample_buffer: &mut [f32],
        _time: f64,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
    ) {
        for s in sample_buffer.iter_mut() {
            *s *= self.gain;
        }
    }
}

musicum_processor_sdk::export_processor!(GainPlugin, Stream);
