use musicum_processor_sdk::{parameters::{ProcessorParamaterInfo, TimeParam}, processor::{
    BaseProcessor, ProcessorDescriptor, ProcessorType, Segment, StructuralProcessor,
}};

static CROP_PARAMS: [ProcessorParamaterInfo; 2] = [
    ProcessorParamaterInfo::Time { id: "from", name: "From", default: 0.0, editable: true },
    ProcessorParamaterInfo::Time { id: "to",   name: "To",   default: 0.0, editable: true },
];

static DESCRIPTOR: ProcessorDescriptor = ProcessorDescriptor {
    id: "crop",
    name: "Crop",
    processor_type: ProcessorType::StructuralProcessor,
    parameters: &CROP_PARAMS,
};

pub struct CropProcessor {
    from: TimeParam,
    to:   TimeParam,
}

impl Default for CropProcessor {
    fn default() -> Self {
        Self {
            from: CROP_PARAMS[0].get_param().unwrap_or_default(),
            to:   CROP_PARAMS[1].get_param().unwrap_or_default(),
        }
    }
}

impl BaseProcessor for CropProcessor {
    fn prepare(
        &mut self,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
        _ctx: &mut musicum_processor_sdk::analyzer::AnalysisContext,
    ) {}

    fn descriptor(&self) -> &'static ProcessorDescriptor { &DESCRIPTOR }

    fn get_parameter(&self, id: &str) -> f64 {
        match id {
            "from" => self.from.get(),
            "to"   => self.to.get(),
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, id: &str, value: f64) {
        match id {
            "from" => self.from.set(value),
            "to"   => self.to.set(value),
            _ => {}
        }
    }
}

impl StructuralProcessor for CropProcessor {
    fn segments(
        &self,
        duration: f64,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
    ) -> Vec<Segment> {
        let from_t = self.from.get().max(0.0);
        let to_t   = self.to.get().clamp(from_t, duration);
        if to_t <= from_t {
            return vec![];
        }
        vec![Segment { src_start: from_t, src_end: to_t, rate: 1.0 }]
    }
}

musicum_processor_sdk::export_processor!(CropProcessor, Structural);

#[cfg(test)]
mod tests {
    use super::*;
    use musicum_processor_sdk::processor::ProcessorContext;

    fn ctx() -> ProcessorContext {
        ProcessorContext { playing: true, sample_rate: 44100, number_channels: 2 }
    }

    #[test]
    fn from_05_to_15_on_duration_3_returns_one_segment() {
        let mut p = CropProcessor::default();
        p.set_parameter("from", 0.5);
        p.set_parameter("to", 1.5);
        let segs = p.segments(3.0, &ctx());
        assert_eq!(segs.len(), 1);
        assert!((segs[0].src_start - 0.5).abs() < 1e-9);
        assert!((segs[0].src_end - 1.5).abs() < 1e-9);
    }

    #[test]
    fn to_lte_from_returns_empty() {
        let mut p = CropProcessor::default();
        p.set_parameter("from", 1.0);
        p.set_parameter("to", 1.0);
        assert!(p.segments(3.0, &ctx()).is_empty());

        p.set_parameter("from", 1.0);
        p.set_parameter("to", 0.5);
        assert!(p.segments(3.0, &ctx()).is_empty());
    }

    #[test]
    fn to_greater_than_duration_clamps_to_duration() {
        let mut p = CropProcessor::default();
        p.set_parameter("from", 1.0);
        p.set_parameter("to", 10.0);
        let segs = p.segments(3.0, &ctx());
        assert_eq!(segs.len(), 1);
        assert!((segs[0].src_start - 1.0).abs() < 1e-9);
        assert!((segs[0].src_end - 3.0).abs() < 1e-9);
    }

    #[test]
    fn defaults_return_empty() {
        let p = CropProcessor::default();
        assert!(p.segments(3.0, &ctx()).is_empty());
    }
}
