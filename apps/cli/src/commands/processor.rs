use std::sync::Arc;

use musicum_core::{config::Config, EditRegistry, EditType, ParamInfo, ProcessorRegistry};
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
    let registry = EditRegistry::new(Arc::new(proc_reg));
    let mut entries: Vec<ProcessorListEntry> = registry
        .list_entries()
        .into_iter()
        .map(|e| {
            let kind = match e.edit_type {
                EditType::Structural => "structural",
                EditType::Stream     => "stream",
                EditType::Analyzer   => "analyzer",
            }
            .to_string();
            let parameters = e
                .parameters
                .iter()
                .map(|p| match p {
                    ParamInfo::Float { id, default, .. } => format!("{id}={default} (float)"),
                    ParamInfo::Bool  { id, default, .. } => format!("{id}={} (bool)", *default as u8),
                    ParamInfo::Time  { id, default, .. } => format!("{id}={default} (time)"),
                    ParamInfo::Int   { id, default, .. } => format!("{id}={default} (int)"),
                })
                .collect();
            ProcessorListEntry { id: e.id, kind, name: e.name.to_string(), parameters }
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
