use serde::{Deserialize, Serialize};

use crate::db::entities::{clip, file};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileListItem {
    pub file: file::Model,
    pub clips: Vec<clip::Model>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClipListItem {
    pub clip: clip::Model,
    pub file: file::Model,
}
