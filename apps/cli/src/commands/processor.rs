use musicum_core::{
    config::Config,
    edit::ProcessorEditType,
    edit_registry::{EditParamInfo, EditRegistry},
    ProcessorRegistry,
};
use serde::Serialize;

use crate::output::{print_json, print_table};

#[derive(Serialize)]
struct ProcessorListEntry {
    id:         String,
    #[serde(rename = "type")]
    kind:       String,
    name:       String,
    parameters: Vec<String>,
}

pub fn run(json: bool) {
    let mut proc_reg = ProcessorRegistry::new();
    proc_reg.load_dir(&Config::get().processors.processor_dir).ok();
    let registry = EditRegistry::new(&proc_reg);
    let mut entries: Vec<ProcessorListEntry> = registry
        .list_entries()
        .iter()
        .map(|e| {
            let kind = match e.edit_type() {
                ProcessorEditType::StructuralProcessor           => "structural",
                ProcessorEditType::StreamProcessor               => "stream",
                ProcessorEditType::StructuralAndStreamProcesssor => "structural+stream",
            }
            .to_string();
            let parameters: Vec<String> = e
                .parameters()
                .iter()
                .filter_map(|p| match p {
                    EditParamInfo::Float { id, default, .. } => Some(format!("{id}={default} (float)")),
                    EditParamInfo::Bool  { id, default, .. } => Some(format!("{id}={} (bool)", *default as u8)),
                    EditParamInfo::Time  { id, default, .. } => Some(format!("{id}={default} (time)")),
                    EditParamInfo::Int   { id, default, .. } => Some(format!("{id}={default} (int)")),
                    EditParamInfo::Hidden                    => None,
                })
                .collect();
            ProcessorListEntry { id: e.id(), kind, name: e.name(), parameters }
        })
        .collect();

    entries.sort_by(|a, b| a.id.cmp(&b.id));

    if json {
        print_json(&entries);
    } else if entries.is_empty() {
        println!("No processors registered.");
    } else {
        print_table(
            "processors",
            &["ID", "TYPE", "NAME", "PARAMETERS"],
            entries
                .iter()
                .map(|e| vec![
                    e.id.clone(),
                    e.kind.clone(),
                    e.name.clone(),
                    e.parameters.join(", "),
                ])
                .collect(),
        );
    }
}
