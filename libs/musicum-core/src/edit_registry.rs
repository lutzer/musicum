use std::sync::Arc;

use musicum_processor_sdk::ffi::{ProcessorDescriptorFFI, ProcessorParamFFI, ProcessorTypeFFI};

use crate::processor_loader::ProcessorRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditType {
    Structural,
    Stream,
    Analyzer,
}

impl From<ProcessorTypeFFI> for EditType {
    fn from(t: ProcessorTypeFFI) -> Self {
        match t {
            ProcessorTypeFFI::Structural => EditType::Structural,
            ProcessorTypeFFI::Stream     => EditType::Stream,
            ProcessorTypeFFI::Analyzer   => EditType::Analyzer,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParamInfo {
    Float {
        id:       String,
        name:     String,
        default:  f32,
        min:      f32,
        max:      f32,
        step:     f32,
        unit:     String,
        editable: bool,
    },
    Bool {
        id:       String,
        name:     String,
        default:  bool,
        editable: bool,
    },
    Time {
        id:       String,
        name:     String,
        default:  f64,
        editable: bool,
    },
    Int {
        id:       String,
        name:     String,
        default:  i64,
        min:      i64,
        max:      i64,
        editable: bool,
    },
}

impl From<&ProcessorParamFFI> for ParamInfo {
    fn from(p: &ProcessorParamFFI) -> Self {
        match *p {
            ProcessorParamFFI::Float { id, name, default, min, max, step, unit, editable } =>
                ParamInfo::Float {
                    id: id.to_string(), name: name.to_string(),
                    default, min, max, step, unit: unit.to_string(), editable,
                },
            ProcessorParamFFI::Bool { id, name, default, editable } =>
                ParamInfo::Bool { id: id.to_string(), name: name.to_string(), default, editable },
            ProcessorParamFFI::Time { id, name, default, editable } =>
                ParamInfo::Time { id: id.to_string(), name: name.to_string(), default, editable },
            ProcessorParamFFI::Int { id, name, default, min, max, editable } =>
                ParamInfo::Int {
                    id: id.to_string(), name: name.to_string(),
                    default, min, max, editable,
                },
        }
    }
}

#[derive(Debug, Clone)]
pub struct EditRegistryEntry {
    pub id:         String,
    pub name:       String,
    pub edit_type:  EditType,
    pub parameters: Vec<ParamInfo>,
}

impl From<&ProcessorDescriptorFFI> for EditRegistryEntry {
    fn from(d: &ProcessorDescriptorFFI) -> Self {
        Self {
            id:         d.id.to_string(),
            name:       d.name.to_string(),
            edit_type:  EditType::from(d.processor_type),
            parameters: d.params.iter().map(ParamInfo::from).collect(),
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
