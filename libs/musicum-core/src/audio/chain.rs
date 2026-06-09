use std::sync::{Arc, Mutex};

use uuid::Uuid;

use musicum_processor_sdk::processor::StreamProcessor;

use crate::audio::node::StreamProcessorNode;
use crate::audio::source::AudioSource;
use crate::edit::{ProcessorEdit, ProcessorEditType};
use crate::processor_loader::ProcessorRegistry;

pub type ProcessorHandle = Arc<Mutex<Box<dyn StreamProcessor>>>;

pub struct ProcessorChain {
    entries: Vec<(Uuid, ProcessorHandle)>,
}

impl ProcessorChain {
    pub fn empty() -> Self {
        Self { entries: vec![] }
    }

    pub fn from_edits(edits: &[ProcessorEdit], registry: &ProcessorRegistry) -> Self {
        let mut entries = Vec::new();
        for edit in edits {
            if !edit.enabled { continue; }
            if edit.kind != ProcessorEditType::StreamProcessor { continue; }
            let Some(loaded) = registry.create(&edit.processor_id) else { continue };
            let Some(mut proc) = loaded.into_stream_processor() else { continue };
            for (id, &value) in &edit.params {
                proc.set_parameter(id, value);
            }
            let handle: ProcessorHandle = Arc::new(Mutex::new(proc));
            entries.push((edit.uuid, handle));
        }
        Self { entries }
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

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
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
    fn structural_processor_is_skipped() {
        let registry = ProcessorRegistry::new();
        let edits = vec![edit(ProcessorEditType::StructuralProcessor, true, "trim")];
        let chain = ProcessorChain::from_edits(&edits, &registry);
        assert!(chain.is_empty());
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
