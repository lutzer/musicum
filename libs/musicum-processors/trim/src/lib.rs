use musicum_processor_sdk::{parameters::{ProcessorParamaterInfo, TimeParam}, processor::{
    BaseProcessor, ProcessorDescriptor, ProcessorType, Segment, StructuralProcessor
}};

static TRIM_PARAMS: [ProcessorParamaterInfo; 2] = [
    ProcessorParamaterInfo::Time { id: "start", name: "Start", default: 0.0, editable: true },
    ProcessorParamaterInfo::Time { id: "end",   name: "End",   default: 0.0, editable: true },
];

static DESCRIPTOR: ProcessorDescriptor = ProcessorDescriptor {
    id: "trim_processor",
    name: "Trim",
    processor_type: ProcessorType::StructuralProcessor,
    parameters: &TRIM_PARAMS,
};

pub struct TrimProcessor {
    start: TimeParam,
    end:   TimeParam,
}

impl Default for TrimProcessor {
    fn default() -> Self {
        Self { 
            start: TRIM_PARAMS[0].get_param().unwrap_or_default(), 
            end: TRIM_PARAMS[1].get_param().unwrap_or_default() }
    }
}

impl BaseProcessor for TrimProcessor {
    fn init(
        &mut self,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
        _ctx: &mut musicum_processor_sdk::analyzer::AnalysisContext,
    ) {
    }

    fn descriptor(&self) -> &'static ProcessorDescriptor {
        &DESCRIPTOR
    }

    fn get_parameter(&self, id: &str) -> f64 {
        if id == "start" { self.start.get() }
        else if id == "end" { self.end.get() }
        else { 0.0 }
    }

    fn set_parameter(&mut self, id: &str, value: f64) {
        if id == "start" { self.start.set(value); }
        else if id == "end" { self.end.set(value); }
    }

    fn requires_analysis(&self) -> bool {
        false
    }
}

impl StructuralProcessor for TrimProcessor {
    fn segments(
        &self,
        duration: f64,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
    ) -> Vec<Segment> {
        let start = self.start.get().max(0.0);
        let end = (duration - self.end.get().max(0.0)).min(duration);
        if end <= start {
            return vec![];
        }
        vec![Segment { src_start: start, src_end: end, rate: 1.0 }]
    }
}

musicum_processor_sdk::export_processor!(TrimProcessor, Structural);
