use std::sync::Arc;

use musicum_processor_sdk::ffi::{ProcessorDescriptorFFI, ProcessorTypeFFI};

use crate::processor_loader::ProcessorRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditType {
    Structural,
    Stream,
    StructuralAndStream,
}

impl From<ProcessorTypeFFI> for EditType {
    fn from(t: ProcessorTypeFFI) -> Self {
        match t {
            ProcessorTypeFFI::Structural => EditType::Structural,
            ProcessorTypeFFI::Stream     => EditType::Stream,
            ProcessorTypeFFI::StructuralAndStream   => EditType::StructuralAndStream,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EditRegistryEntry {
    pub id:         String,
    pub name:       String,
    pub edit_type:  EditType,
    pub parameters: Vec<(String, f64)>,
}

impl From<&ProcessorDescriptorFFI> for EditRegistryEntry {
    fn from(d: &ProcessorDescriptorFFI) -> Self {
        Self {
            id:         d.id.to_string(),
            name:       d.name.to_string(),
            edit_type:  EditType::from(d.processor_type),
            parameters: vec![],
        }
    }
}

pub struct EditRegistry {
    _inner:  Arc<ProcessorRegistry>,
    entries: Vec<EditRegistryEntry>,
}

impl EditRegistry {
    pub fn new(registry: Arc<ProcessorRegistry>) -> Self {
        let entries = registry
            .descriptors()
            .map(EditRegistryEntry::from)
            .collect();
        Self { _inner: registry, entries }
    }

    pub fn list_entries(&self) -> Vec<EditRegistryEntry> {
        self.entries.clone()
    }

    pub fn get_entry(&self, id: &str) -> Option<&EditRegistryEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn registry(&self) -> &Arc<ProcessorRegistry> {
        &self._inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processor_loader::ProcessorRegistry;

    #[test]
    fn empty_registry_has_no_entries() {
        let reg = Arc::new(ProcessorRegistry::new());
        let edit_reg = EditRegistry::new(reg);
        assert_eq!(edit_reg.list_entries().len(), 0);
        assert!(edit_reg.get_entry("anything").is_none());
    }
}
