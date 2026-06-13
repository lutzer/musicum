use musicum_processor_sdk::{ffi::{ProcessorDescriptorFFI, ProcessorParamFFI, ProcessorTypeFFI}};

use crate::{ProcessorRegistry, edit::ProcessorEditType};


#[derive(Debug, Clone)]
pub enum EditParamInfo {
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
        default:  i32,
        min:      i32,
        max:      i32,
        editable: bool,
    },
    Hidden
}

impl From<&ProcessorParamFFI> for EditParamInfo {
    fn from(p: &ProcessorParamFFI) -> Self {
        match *p {
            ProcessorParamFFI::Float { id, name, default, min, max, step, unit, editable } =>
                EditParamInfo::Float {
                    id: id.to_string(), name: name.to_string(),
                    default, min, max, step, unit: unit.to_string(), editable,
                },
            ProcessorParamFFI::Bool { id, name, default, editable } =>
                EditParamInfo::Bool { id: id.to_string(), name: name.to_string(), default, editable },
            ProcessorParamFFI::Time { id, name, default, editable } =>
                EditParamInfo::Time { id: id.to_string(), name: name.to_string(), default, editable },
            ProcessorParamFFI::Int { id, name, default, min, max, editable } =>
                EditParamInfo::Int {
                    id: id.to_string(), name: name.to_string(),
                    default, min, max, editable,
                },
            _ => EditParamInfo::Hidden
        }
    }
}

pub struct EditRegistryEntry {
    descriptor: ProcessorDescriptorFFI
}

impl EditRegistryEntry {
    pub fn id(&self) -> String { self.descriptor.id.to_string() }
    pub fn name(&self) -> String { self.descriptor.name.to_string() }
    pub fn edit_type(&self) -> ProcessorEditType {
        match self.descriptor.processor_type {
            ProcessorTypeFFI::Stream => ProcessorEditType::StreamProcessor,
            ProcessorTypeFFI::Structural => ProcessorEditType::StructuralProcessor,
            ProcessorTypeFFI::StructuralAndStream => ProcessorEditType::StructuralAndStreamProcesssor,
        }
    }
    pub fn parameters(&self) -> Vec<EditParamInfo> { 
        self.descriptor.params.iter().map(|p| {
            return EditParamInfo::from(p);
        }).collect()
    }
}

pub struct EditRegistry {
    entries: Vec<EditRegistryEntry>
}

impl EditRegistry {
    pub fn new(processor_registry: &ProcessorRegistry) -> Self {
        let entries = processor_registry.descriptors().map(|d| {
            EditRegistryEntry {
                descriptor: d.clone()
            }
        }).collect();
        EditRegistry { entries }
    }

    pub fn list_entries(&self) -> &Vec<EditRegistryEntry> {
        &self.entries
    }

    pub fn get_entry(&self, id: &str) -> Option<&EditRegistryEntry> {
        self.entries.iter().find(|e| e.id().as_str() == id)
    }
}