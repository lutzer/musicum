use std::collections::HashMap;

use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub enum ProcessorEditType { StructuralProcessor, StreamProcessor, Analyzer }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct ProcessorEdit {
    pub uuid: Uuid,
    pub processor_id: String,
    pub enabled: bool,
    pub kind: ProcessorEditType,
    pub params: HashMap<String, f64>,
}

// For a Vec, just wrap it
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct ProcessorEditList(pub Vec<ProcessorEdit>);