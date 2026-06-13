use std::sync::{Arc, Mutex};

use musicum_processor_sdk::BaseProcessor;
use musicum_processor_sdk::analyzer::AnalysisContext;
use musicum_processor_sdk::ffi::ProcessorTypeFFI;
use musicum_processor_sdk::processor::ProcessorContext;
use uuid::Uuid;

use crate::audio::node::StreamProcessorNode;
use crate::audio::source::AudioSource;
use crate::audio::timeline::Timeline;
use crate::edit::{ProcessorEdit};
use crate::processor_loader::ProcessorRegistry;

pub type ProcessorHandle = Arc<Mutex<Box<dyn BaseProcessor>>>;

struct ChainEntry {
    uuid:           Uuid,
    processor_type: ProcessorTypeFFI,
    handle:         ProcessorHandle,
}

pub struct ProcessorChain {
    handles:         Vec<ChainEntry>,
    analysis:        AnalysisContext,
    structure_dirty: bool,
}

impl ProcessorChain {
    pub fn empty() -> Self {
        Self {
            handles:         Vec::new(),
            analysis:        AnalysisContext::default(),
            structure_dirty: false,
        }
    }

    pub fn from_edits(edits: &[ProcessorEdit], registry: &ProcessorRegistry) -> Self {
        let mut handles = Vec::new();
        for edit in edits {
            if !edit.enabled { continue; }
            let Some(mut proc) = registry.create(&edit.processor_id) else { continue };
            for (id, &value) in &edit.params { proc.set_parameter(id, value); }
            let Some(processor_type) = registry.processor_type(&edit.processor_id) else { continue };
            let handle: ProcessorHandle = Arc::new(Mutex::new(Box::new(proc)));
            handles.push(ChainEntry { uuid: edit.uuid, processor_type, handle });
        }
        Self {
            handles,
            analysis:        AnalysisContext::default(),
            structure_dirty: true,
        }
    }

    /// Initializes every processor with the given context. Must be called
    /// once before `build_timeline` / `build_source`. The chain's
    /// `AnalysisContext` receives all requests posted from `init()` and is
    /// available afterwards via `analysis()`.
    pub fn init_all(&mut self, ctx: &ProcessorContext) {
        for entry in &self.handles {
            entry.handle.lock().unwrap().init(
                entry.uuid.to_string(), ctx, &mut self.analysis,
            );
        }
    }

    pub fn analysis(&self) -> &AnalysisContext { &self.analysis }

    pub fn build_timeline(
        &self,
        source_frames: u64,
        sample_rate: u32,
        ctx: &ProcessorContext,
    ) -> Timeline {
        let mut timeline = Timeline::identity(source_frames, sample_rate);
        for entry in self.handles.iter().filter(|e| is_structural(e.processor_type)) {
            let segs = entry.handle.lock().unwrap()
                .segments(timeline.output_duration(), ctx);
            timeline.apply_edit(&segs);
        }
        timeline
    }

    pub fn build_source(&self, root: Box<dyn AudioSource>) -> Box<dyn AudioSource> {
        // StructuralAndStream entries are run in both passes, against the
        // same Arc<Mutex>. Both calls are sequential during chain construction,
        // so no deadlock risk.
        self.handles.iter()
            .filter(|e| is_stream(e.processor_type))
            .fold(root, |upstream, entry| {
                Box::new(StreamProcessorNode::new(upstream, Arc::clone(&entry.handle)))
                    as Box<dyn AudioSource>
            })
    }

    pub fn get_handle(&self, uuid: &Uuid) -> Option<&ProcessorHandle> {
        self.handles.iter().find(|e| &e.uuid == uuid).map(|e| &e.handle)
    }

    pub fn handles(&self) -> impl Iterator<Item = (&Uuid, &ProcessorHandle)> {
        self.handles.iter().map(|e| (&e.uuid, &e.handle))
    }

    /// Routes a parameter change to whichever handle owns `uuid`.
    pub fn set_parameter(&mut self, uuid: &Uuid, param_id: &str, value: f64) {
        if let Some(h) = self.get_handle(uuid) {
            h.lock().unwrap().set_parameter(param_id, value);
            self.set_structure_dirty(true);
        }
    }

    pub fn stream_handles(&self) -> impl Iterator<Item = (&Uuid, &ProcessorHandle)> {
        self.handles.iter()
            .filter(|e| is_stream(e.processor_type))
            .map(|e| (&e.uuid, &e.handle))
    }

    pub fn structural_handles(&self) -> impl Iterator<Item = (&Uuid, &ProcessorHandle)> {
        self.handles.iter()
            .filter(|e| is_structural(e.processor_type))
            .map(|e| (&e.uuid, &e.handle))
    }

    #[cfg(test)]
    pub(crate) fn push_handle(
        &mut self,
        uuid: Uuid,
        processor_type: ProcessorTypeFFI,
        handle: ProcessorHandle,
    ) {
        self.handles.push(ChainEntry { uuid, processor_type, handle });
    }

    pub fn is_structure_dirty(&self) -> bool { self.structure_dirty }
    pub fn set_structure_dirty(&mut self, dirty: bool) { self.structure_dirty = dirty }
}

fn is_structural(t: ProcessorTypeFFI) -> bool {
    matches!(t, ProcessorTypeFFI::Structural | ProcessorTypeFFI::StructuralAndStream)
}

fn is_stream(t: ProcessorTypeFFI) -> bool {
    matches!(t, ProcessorTypeFFI::Stream | ProcessorTypeFFI::StructuralAndStream)
}

#[cfg(test)]
mod tests {
    use crate::edit::ProcessorEditType;

use super::*;
    use std::collections::HashMap;
    use musicum_processor_sdk::BaseProcessor;

    fn edit(kind: ProcessorEditType, enabled: bool, id: &str) -> ProcessorEdit {
        ProcessorEdit {
            uuid:         Uuid::new_v4(),
            processor_id: id.to_string(),
            enabled,
            kind,
            params:       HashMap::new(),
        }
    }

    #[test]
    fn empty_edits_produces_empty_chain() {
        let registry = ProcessorRegistry::new();
        let chain = ProcessorChain::from_edits(&[], &registry);
        assert_eq!(chain.stream_handles().count(), 0);
        assert_eq!(chain.structural_handles().count(), 0);
    }

    #[test]
    fn disabled_stream_processor_is_skipped() {
        let registry = ProcessorRegistry::new();
        let edits = vec![edit(ProcessorEditType::StreamProcessor, false, "gain")];
        let chain = ProcessorChain::from_edits(&edits, &registry);
        assert_eq!(chain.stream_handles().count(), 0);
        assert_eq!(chain.structural_handles().count(), 0);
    }

    #[test]
    fn structural_edit_with_unknown_id_is_skipped() {
        let registry = ProcessorRegistry::new();
        let edits = vec![edit(ProcessorEditType::StructuralProcessor, true, "trim")];
        let chain = ProcessorChain::from_edits(&edits, &registry);
        assert_eq!(chain.structural_handles().count(), 0);
    }

    #[test]
    fn disabled_structural_edit_is_skipped() {
        let registry = ProcessorRegistry::new();
        let edits = vec![edit(ProcessorEditType::StructuralProcessor, false, "trim")];
        let chain = ProcessorChain::from_edits(&edits, &registry);
        assert_eq!(chain.structural_handles().count(), 0);
    }

    #[test]
    fn build_timeline_without_structural_edits_is_identity() {
        let chain = ProcessorChain::empty();
        let ctx = ProcessorContext { playing: false, sample_rate: 100, number_channels: 2 };
        let tl = chain.build_timeline(1000, 100, &ctx);
        assert_eq!(tl.output_frames(), 1000);
    }

    #[test]
    fn build_timeline_applies_structural_handles_in_order() {
        use crate::audio::tests::test_processors::TestTrim;
        let mut chain = ProcessorChain::empty();
        chain.push_handle(
            Uuid::new_v4(),
            ProcessorTypeFFI::Structural,
            Arc::new(Mutex::new(Box::new(TestTrim { start: 1.0, end: 1.0, record: None }) as Box<dyn BaseProcessor>)),
        );
        chain.push_handle(
            Uuid::new_v4(),
            ProcessorTypeFFI::Structural,
            Arc::new(Mutex::new(Box::new(TestTrim { start: 1.0, end: 0.0, record: None }) as Box<dyn BaseProcessor>)),
        );
        let ctx = ProcessorContext { playing: false, sample_rate: 100, number_channels: 2 };
        let tl = chain.build_timeline(1000, 100, &ctx); // 10s → [1,9] → [2,9]
        assert_eq!(tl.output_frames(), 700);
        assert!((tl.source_time(0.0) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn set_parameter_routes_to_handle() {
        use crate::audio::tests::test_processors::TestTrim;
        let mut chain = ProcessorChain::empty();
        let uuid = Uuid::new_v4();
        chain.push_handle(
            uuid,
            ProcessorTypeFFI::Structural,
            Arc::new(Mutex::new(Box::new(TestTrim::default()) as Box<dyn BaseProcessor>)),
        );
        chain.set_parameter(&uuid, "start", 2.5);
        let h = chain.get_handle(&uuid).unwrap();
        assert!((h.lock().unwrap().get_parameter("start") - 2.5).abs() < 1e-9);
    }


    #[test]
    fn unregistered_processor_id_is_skipped() {
        let registry = ProcessorRegistry::new();
        let edits = vec![edit(ProcessorEditType::StreamProcessor, true, "nonexistent")];
        let chain = ProcessorChain::from_edits(&edits, &registry);
        assert_eq!(chain.stream_handles().count(), 0);
        assert_eq!(chain.structural_handles().count(), 0);
    }

    #[test]
    fn get_handle_returns_none_for_unknown_uuid() {
        let chain = ProcessorChain::empty();
        assert!(chain.get_handle(&Uuid::new_v4()).is_none());
    }

    #[test]
    fn init_all_calls_init_on_stream_and_structural_handles() {
        use crate::audio::tests::test_processors::{InitRecord, TestStream, TestTrim};

        let mut chain = ProcessorChain::empty();

        let stream_rec = Arc::new(Mutex::new(InitRecord::default()));
        let stream_uuid = Uuid::new_v4();
        chain.push_handle(
            stream_uuid,
            ProcessorTypeFFI::Stream,
            Arc::new(Mutex::new(Box::new(TestStream { record: Arc::clone(&stream_rec) }) as Box<dyn BaseProcessor>)),
        );

        let struct_rec = Arc::new(Mutex::new(InitRecord::default()));
        let struct_uuid = Uuid::new_v4();
        chain.push_handle(
            struct_uuid,
            ProcessorTypeFFI::Structural,
            Arc::new(Mutex::new(Box::new(TestTrim {
                start: 0.0, end: 0.0, record: Some(Arc::clone(&struct_rec)),
            }) as Box<dyn BaseProcessor>)),
        );

        let ctx = ProcessorContext { playing: false, sample_rate: 44100, number_channels: 2 };
        chain.init_all(&ctx);

        assert_eq!(stream_rec.lock().unwrap().uuid, stream_uuid.to_string());
        assert_eq!(struct_rec.lock().unwrap().uuid, struct_uuid.to_string());
    }

    #[test]
    fn init_all_shares_analysis_context() {
        use crate::audio::tests::test_processors::{InitRecord, TestStream, TestTrim};

        let mut chain = ProcessorChain::empty();
        chain.push_handle(
            Uuid::new_v4(),
            ProcessorTypeFFI::Stream,
            Arc::new(Mutex::new(Box::new(TestStream {
                record: Arc::new(Mutex::new(InitRecord::default())),
            }) as Box<dyn BaseProcessor>)),
        );
        chain.push_handle(
            Uuid::new_v4(),
            ProcessorTypeFFI::Structural,
            Arc::new(Mutex::new(Box::new(TestTrim {
                start: 0.0, end: 0.0,
                record: Some(Arc::new(Mutex::new(InitRecord::default()))),
            }) as Box<dyn BaseProcessor>)),
        );

        let ctx = ProcessorContext { playing: false, sample_rate: 44100, number_channels: 2 };
        chain.init_all(&ctx);

        assert_eq!(chain.analysis().requests.len(), 2);
    }
}
