use musicum_processor_sdk::{parameters::{ProcessorParamaterInfo, TimeParam}, processor::{
    BaseProcessor, ProcessorDescriptor, ProcessorType, Segment, StructuralProcessor
}};

static CUT_PARAMS: [ProcessorParamaterInfo; 2] = [
    ProcessorParamaterInfo::Time { id: "start", name: "Start", default: 0.0, editable: true },
    ProcessorParamaterInfo::Time { id: "end",   name: "End",   default: 0.0, editable: true },
];

static DESCRIPTOR: ProcessorDescriptor = ProcessorDescriptor {
    id: "cut_processor",
    name: "Cut",
    processor_type: ProcessorType::StructuralProcessor,
    parameters: &CUT_PARAMS,
};

pub struct CutProcessor {
    start: TimeParam,
    end:   TimeParam,
}

impl Default for CutProcessor {
    fn default() -> Self {
        Self { 
            start: CUT_PARAMS[0].get_param().unwrap_or_default(), 
            end: CUT_PARAMS[1].get_param().unwrap_or_default() }
    }
}

impl BaseProcessor for CutProcessor {
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
}

impl StructuralProcessor for CutProcessor {
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
        vec![
            Segment { src_start: 0.0, src_end: start, rate: 1.0 },
            Segment { src_start: end, src_end: duration, rate: 1.0 }
        ]
    }
}

musicum_processor_sdk::export_processor!(CutProcessor, Structural);
