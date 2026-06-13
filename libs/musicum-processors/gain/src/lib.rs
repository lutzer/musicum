use musicum_processor_sdk::{parameters::{FloatParam, ProcessorParamaterInfo}, processor::{
    BaseProcessor, ProcessorDescriptor, ProcessorType, StreamProcessor
}};

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
    gain: FloatParam,
}

impl Default for GainPlugin {
    fn default() -> Self {
        Self { gain: GAIN_PARAMS[0].get_param().unwrap_or_default() }
    }
}

impl BaseProcessor for GainPlugin {

    fn descriptor(&self) -> &'static ProcessorDescriptor {
        &DESCRIPTOR
    }

    fn get_parameter(&self, id: &str) -> f64 {
        if id == "gain" { self.gain.get() as f64 } else { 0.0 }
    }

    fn set_parameter(&mut self, id: &str, value: f64) {
        if id == "gain" {
            self.gain.set(value as f32);
        }
    }
}

impl StreamProcessor for GainPlugin {
    fn process(
        &mut self,
        samples: &mut [f32],
        _time: f64,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
    ) {
        for s in samples.iter_mut() {
            *s *= self.gain.get();
        }
    }
}

musicum_processor_sdk::export_processor!(GainPlugin, Stream);
