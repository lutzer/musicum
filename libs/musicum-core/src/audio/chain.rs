use std::sync::{Arc, Mutex};

use uuid::Uuid;

use musicum_processor_sdk::processor::{ProcessorContext, StreamProcessor, StructuralProcessor};

use crate::audio::node::StreamProcessorNode;
use crate::audio::source::AudioSource;
use crate::audio::timeline::Timeline;
use crate::edit::{ProcessorEdit, ProcessorEditType};
use crate::processor_loader::ProcessorRegistry;

pub type ProcessorHandle = Arc<Mutex<Box<dyn StreamProcessor>>>;
pub type StructuralHandle = Arc<Mutex<Box<dyn StructuralProcessor>>>;


// TODO: change entries and structural entries to hashmap, rename entries to stream_entries

pub struct ProcessorChain {
    entries: Vec<(Uuid, ProcessorHandle)>,
    structural_entries: Vec<(Uuid, StructuralHandle)>,
    structure_dirty: bool
}

impl ProcessorChain {
    pub fn empty() -> Self {
        Self { entries: vec![], structural_entries: vec![], structure_dirty: false }
    }

    pub fn from_edits(edits: &[ProcessorEdit], registry: &ProcessorRegistry) -> Self {
        let mut entries = Vec::new();
        let mut structural_entries = Vec::new();
        for edit in edits {
            if !edit.enabled { continue; }
            match edit.kind {
                ProcessorEditType::StreamProcessor => {
                    let Some(loaded) = registry.create(&edit.processor_id) else { continue };
                    let Some(mut proc) = loaded.into_stream_processor() else { continue };
                    for (id, &value) in &edit.params { proc.set_parameter(id, value); }
                    entries.push((edit.uuid, Arc::new(Mutex::new(proc)) as ProcessorHandle));
                }
                ProcessorEditType::StructuralProcessor => {
                    let Some(loaded) = registry.create(&edit.processor_id) else { continue };
                    let Some(mut proc) = loaded.into_structural_processor() else { continue };
                    for (id, &value) in &edit.params { proc.set_parameter(id, value); }
                    structural_entries.push((edit.uuid, Arc::new(Mutex::new(proc)) as StructuralHandle));
                }
                ProcessorEditType::Analyzer => {}
            }
        }
        Self { entries, structural_entries, structure_dirty : true }
    }

    pub fn build_timeline(
        &self,
        source_frames: u64,
        sample_rate: u32,
        ctx: &ProcessorContext,
    ) -> Timeline {
        let mut timeline = Timeline::identity(source_frames, sample_rate);
        for (_, handle) in &self.structural_entries {
            let segs = handle.lock().unwrap().segments(timeline.output_duration(), ctx);
            timeline.apply_edit(&segs);
        }
        timeline
    }

    pub fn build_source(&self, root: Box<dyn AudioSource>) -> Box<dyn AudioSource> {
        self.entries.iter().fold(root, |upstream, (_, handle)| {
            Box::new(StreamProcessorNode::new(upstream, Arc::clone(handle)))
                as Box<dyn AudioSource>
        })
    }

    pub fn get_handle(&self, uuid: &Uuid) -> Option<&ProcessorHandle> {
        self.entries.iter().find(|(id, _)| id == uuid).map(|(_, h)| h)
    }

    pub fn handles(&self) -> impl Iterator<Item = (&Uuid, &ProcessorHandle)> {
        self.entries.iter().map(|(id, h)| (id, h))
    }

    pub fn get_structural_handle(&self, uuid: &Uuid) -> Option<&StructuralHandle> {
        self.structural_entries.iter().find(|(id, _)| id == uuid).map(|(_, h)| h)
    }

    pub fn structural_handles(&self) -> impl Iterator<Item = (&Uuid, &StructuralHandle)> {
        self.structural_entries.iter().map(|(id, h)| (id, h))
    }

    pub fn has_structural(&self) -> bool { !self.structural_entries.is_empty() }

    /// Routes a parameter change to whichever handle owns `uuid` and reports
    /// which kind was touched (None if the uuid is unknown).
    pub fn set_parameter(&mut self, uuid: &Uuid, param_id: &str, value: f64) {
        if let Some(h) = self.get_handle(uuid) {
            h.lock().unwrap().set_parameter(param_id, value);
        }
        if let Some(h) = self.get_structural_handle(uuid) {
            h.lock().unwrap().set_parameter(param_id, value);
            self.set_structure_dirty(true);
        }
    }

    #[cfg(test)]
    pub(crate) fn push_structural(&mut self, uuid: Uuid, handle: StructuralHandle) {
        self.structural_entries.push((uuid, handle));
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn is_structure_dirty(&self) -> bool { self.structure_dirty }
    pub fn set_structure_dirty(&mut self, dirty: bool) { self.structure_dirty = dirty }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
        assert!(chain.is_empty());
    }

    #[test]
    fn disabled_stream_processor_is_skipped() {
        let registry = ProcessorRegistry::new();
        let edits = vec![edit(ProcessorEditType::StreamProcessor, false, "gain")];
        let chain = ProcessorChain::from_edits(&edits, &registry);
        assert!(chain.is_empty());
    }

    #[test]
    fn structural_edit_with_unknown_id_is_skipped() {
        let registry = ProcessorRegistry::new();
        let edits = vec![edit(ProcessorEditType::StructuralProcessor, true, "trim")];
        let chain = ProcessorChain::from_edits(&edits, &registry);
        assert!(!chain.has_structural());
    }

    #[test]
    fn disabled_structural_edit_is_skipped() {
        let registry = ProcessorRegistry::new();
        let edits = vec![edit(ProcessorEditType::StructuralProcessor, false, "trim")];
        let chain = ProcessorChain::from_edits(&edits, &registry);
        assert!(!chain.has_structural());
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
        chain.push_structural(
            Uuid::new_v4(),
            Arc::new(Mutex::new(Box::new(TestTrim { start: 1.0, end: 1.0 }) as _)),
        );
        chain.push_structural(
            Uuid::new_v4(),
            Arc::new(Mutex::new(Box::new(TestTrim { start: 1.0, end: 0.0 }) as _)),
        );
        let ctx = ProcessorContext { playing: false, sample_rate: 100, number_channels: 2 };
        let tl = chain.build_timeline(1000, 100, &ctx); // 10s → [1,9] → [2,9]
        assert_eq!(tl.output_frames(), 700);
        assert!((tl.source_time(0.0) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn set_parameter_routes_to_structural_and_reports_kind() {
        use crate::audio::tests::test_processors::TestTrim;
        let mut chain = ProcessorChain::empty();
        let uuid = Uuid::new_v4();
        chain.push_structural(uuid, Arc::new(Mutex::new(Box::new(TestTrim::default()) as _)));
        chain.set_parameter(&uuid, "start", 2.5);
        let h = chain.get_structural_handle(&uuid).unwrap();
        assert!((h.lock().unwrap().get_parameter("start") - 2.5).abs() < 1e-9);
    }

    #[test]
    fn analyzer_is_skipped() {
        let registry = ProcessorRegistry::new();
        let edits = vec![edit(ProcessorEditType::Analyzer, true, "lufs")];
        let chain = ProcessorChain::from_edits(&edits, &registry);
        assert!(chain.is_empty());
    }

    #[test]
    fn unregistered_processor_id_is_skipped() {
        let registry = ProcessorRegistry::new();
        let edits = vec![edit(ProcessorEditType::StreamProcessor, true, "nonexistent")];
        let chain = ProcessorChain::from_edits(&edits, &registry);
        assert!(chain.is_empty());
    }

    #[test]
    fn get_handle_returns_none_for_unknown_uuid() {
        let chain = ProcessorChain::empty();
        assert!(chain.get_handle(&Uuid::new_v4()).is_none());
    }
}
