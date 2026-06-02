use musicum_processor_sdk::processor::{BaseProcessor, StreamProcessor, ProcessorDescriptor, ProcessorType, ProcessorParamaterInfo};

// static DESCRIPTOR: PluginDescriptor = PluginDescriptor {
//     id: "gain",
//     name: "Gain",
//     version: "0.1.0",
//     mode: PluginMode::Realtime,
//     parameters: &GAIN_PARAMS,
// };

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

static DESCRIPTOR : ProcessorDescriptor = ProcessorDescriptor {
    id: "gain_plugin",
    name: "Gain",
    processor_type: ProcessorType::StreamProcessor,
    parameters: &GAIN_PARAMS
};

pub struct GainPlugin {
    gain: f32,
}

impl BaseProcessor for GainPlugin {
    fn prepare(&mut self, 
        _context: &musicum_processor_sdk::processor::ProcessorContext, 
        _ctx: &mut musicum_processor_sdk::analyzer::AnalysisContext) {
        todo!()
    }

    fn descriptor(&self) ->  &'static ProcessorDescriptor {
        return &DESCRIPTOR;
    }

    fn get_parameter(&self, id: &str) -> f32 {
        if id == "gain" { self.gain } else { 0.0 }
    }

    fn set_parameter(&mut self, id: &str, value: f32) {
        if id == "gain" {
            self.gain = value;
        }
    }

    fn requires_analysis(&self) -> bool { false }
}

impl StreamProcessor for GainPlugin {
    fn process( &mut self,
        sample_buffer: &mut [f32],
        _time: f64,
        _context: &musicum_processor_sdk::processor::ProcessorContext) {
        
        for s in sample_buffer.iter_mut() {
            *s *= self.gain as f32;
        }
    }
    // fn descriptor() -> &'static PluginDescriptor {
    //     &DESCRIPTOR
    // }

    // fn new() -> Self {
    //     GainPlugin { gain: GAIN_PARAMS[0].float_param() }
    // }

    // fn set_parameter(&mut self, id: &str, value: f32) {
    //     if id == "gain" {
    //         self.gain.set(value);
    //     }
    // }

    // fn get_parameter(&self, id: &str) -> f32 {
    //     if id == "gain" { self.gain.get() } else { 0.0 }
    // }

    // fn process(
    //     &mut self,
    //     samples: &mut [f32],
    //     _channels: usize,
    //     _sample_rate: f32,
    //     _timestamp_secs: f64,
    // ) {
    //     for s in samples.iter_mut() {
    //         *s *= self.gain.get();
    //     }
    // }
}

// static GAIN_PARAMS: [PluginParameter; 1] = [PluginParameter::Float {
//     id: "gain",
//     name: "Gain",
//     min: 0.0,
//     max: 4.0,
//     default: 1.0,
//     step: 0.01,
//     unit: "x",
//     disabled: false,
//     hidden: false,
// }];

// static DESCRIPTOR: PluginDescriptor = PluginDescriptor {
//     id: "gain",
//     name: "Gain",
//     version: "0.1.0",
//     mode: PluginMode::Realtime,
//     parameters: &GAIN_PARAMS,
// };

// pub struct GainPlugin {
//     gain: FloatParam,
// }

// impl StreamProcessor for GainPlugin {
//     fn descriptor() -> &'static PluginDescriptor {
//         &DESCRIPTOR
//     }

//     fn new() -> Self {
//         GainPlugin { gain: GAIN_PARAMS[0].float_param() }
//     }

//     fn set_parameter(&mut self, id: &str, value: f32) {
//         if id == "gain" {
//             self.gain.set(value);
//         }
//     }

//     fn get_parameter(&self, id: &str) -> f32 {
//         if id == "gain" { self.gain.get() } else { 0.0 }
//     }

//     fn process(
//         &mut self,
//         samples: &mut [f32],
//         _channels: usize,
//         _sample_rate: f32,
//         _timestamp_secs: f64,
//     ) {
//         for s in samples.iter_mut() {
//             *s *= self.gain.get();
//         }
//     }
// }
