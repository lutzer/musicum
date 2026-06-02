use musicum_processor_sdk::processor::{
    BaseProcessor, StructuralProcessor, ProcessorDescriptor, ProcessorType, ProcessorParamaterInfo,
};

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
    start: f64,
    end:   f64,
}

impl Default for TrimProcessor {
    fn default() -> Self {
        Self { start: 0.0, end: 0.0 }
    }
}

impl BaseProcessor for TrimProcessor {
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
        if id == "start" { self.start }
        else if id == "end" { self.end }
        else { 0.0 }
    }

    fn set_parameter(&mut self, id: &str, value: f64) {
        if id == "start" { self.start = value; }
        else if id == "end" { self.end = value; }
    }

    fn requires_analysis(&self) -> bool {
        false
    }
}

impl StructuralProcessor for TrimProcessor {
    fn map_source_time(
        &self,
        source_time: f64,
        duration: f64,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
    ) -> f64 {
        source_time.min(self.start).max(duration - self.end)
    }

    fn map_processed_time(
        &self,
        processed_time: f64,
        _duration: f64,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
    ) -> f64 {
        self.start + processed_time
    }

    fn output_duration(
        &self,
        duration: f64,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
    ) -> f64 {
        duration - self.start - self.end
    }
}

musicum_processor_sdk::export_processor!(TrimProcessor, Structural);
