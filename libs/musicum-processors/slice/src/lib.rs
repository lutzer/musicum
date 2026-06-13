use musicum_processor_sdk::{parameters::{BoolParam, IntParam, ProcessorParamaterInfo}, processor::{
    BaseProcessor, ProcessorDescriptor, ProcessorMeta, ProcessorType, Segment,
}};

static SLICE_PARAMS: [ProcessorParamaterInfo; 3] = [
    ProcessorParamaterInfo::Int {
        id: "slices", name: "Slices",
        default: 1, min: 1, max: 64, editable: true,
    },
    ProcessorParamaterInfo::Int {
        id: "select_slice", name: "Select Slice",
        default: 0, min: 0, max: 63, editable: true,
    },
    ProcessorParamaterInfo::Bool { 
        id: "export_multiple", 
        name: "Export all", default: true, 
        editable: true 
    },
];

static DESCRIPTOR: ProcessorDescriptor = ProcessorDescriptor {
    id: "slice",
    name: "Slice",
    processor_type: ProcessorType::StructuralProcessor,
    parameters: &SLICE_PARAMS,
};

pub struct SliceProcessor {
    slices:       IntParam,
    select_slice: IntParam,
    export_multiple: BoolParam,
}

impl Default for SliceProcessor {
    fn default() -> Self {
        Self {
            slices:       SLICE_PARAMS[0].get_param().unwrap_or_default(),
            select_slice: SLICE_PARAMS[1].get_param().unwrap_or_default(),
            export_multiple: SLICE_PARAMS[2].get_param().unwrap_or_default(),
        }
    }
}

impl BaseProcessor for SliceProcessor {
    fn get_parameter(&self, id: &str) -> f64 {
        match id {
            "slices"       => self.slices.get() as f64,
            "select_slice" => self.select_slice.get() as f64,
            "export_multiple" => self.export_multiple.get() as f64,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, id: &str, value: f64) {
        match id {
            "slices"       => self.slices.set(value as i32),
            "select_slice" => self.select_slice.set(value as i32),
            "export_multiple" => self.export_multiple.set(value >= 1.0),
            _ => {}
        }
    }

    fn segments(
        &self,
        duration: f64,
        _context: &musicum_processor_sdk::processor::ProcessorContext,
    ) -> Vec<Segment> {
        let slices = self.slices.get();
        let select = self.select_slice.get();
        if select >= slices {
            return vec![];
        }
        let slice_dur = duration / slices as f64;
        let src_start = select as f64 * slice_dur;
        let src_end   = (select as f64 + 1.0) * slice_dur;
        vec![Segment { src_start, src_end, rate: 1.0 }]
    }
}

impl ProcessorMeta for SliceProcessor {
    fn descriptor() -> &'static ProcessorDescriptor { &DESCRIPTOR }
}

musicum_processor_sdk::export_processor!(SliceProcessor);

#[cfg(test)]
mod tests {
    use super::*;
    use musicum_processor_sdk::processor::ProcessorContext;

    fn ctx() -> ProcessorContext {
        ProcessorContext { playing: true, sample_rate: 44100, number_channels: 2 }
    }

    #[test]
    fn slices_4_select_0_returns_first_quarter() {
        let mut p = SliceProcessor::default();
        p.set_parameter("slices", 4.0);
        p.set_parameter("select_slice", 0.0);
        let segs = p.segments(1.0, &ctx());
        assert_eq!(segs.len(), 1);
        assert!((segs[0].src_start - 0.0).abs() < 1e-9);
        assert!((segs[0].src_end - 0.25).abs() < 1e-9);
    }

    #[test]
    fn slices_4_select_2_returns_third_quarter() {
        let mut p = SliceProcessor::default();
        p.set_parameter("slices", 4.0);
        p.set_parameter("select_slice", 2.0);
        let segs = p.segments(1.0, &ctx());
        assert_eq!(segs.len(), 1);
        assert!((segs[0].src_start - 0.5).abs() < 1e-9);
        assert!((segs[0].src_end - 0.75).abs() < 1e-9);
    }

    #[test]
    fn select_gte_slices_returns_empty() {
        let mut p = SliceProcessor::default();
        p.set_parameter("slices", 4.0);
        p.set_parameter("select_slice", 4.0);
        assert!(p.segments(1.0, &ctx()).is_empty());
    }

    #[test]
    fn slices_1_select_0_returns_full_duration() {
        let mut p = SliceProcessor::default();
        p.set_parameter("slices", 1.0);
        p.set_parameter("select_slice", 0.0);
        let segs = p.segments(1.0, &ctx());
        assert_eq!(segs.len(), 1);
        assert!((segs[0].src_start - 0.0).abs() < 1e-9);
        assert!((segs[0].src_end - 1.0).abs() < 1e-9);
    }
}
