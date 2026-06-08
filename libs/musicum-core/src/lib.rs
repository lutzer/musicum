pub mod config;
pub mod db;
pub mod edit_registry;
pub mod error;
pub mod processor_loader;
pub mod services;
pub mod sidecar;
pub mod audio;

pub mod edit {
    pub use crate::db::entities::edit::{ProcessorEdit, ProcessorEditList, ProcessorEditType};
}

pub use db::entities::edit::ProcessorEdit;
pub use edit_registry::{EditRegistry, EditRegistryEntry, EditType, ParamInfo};
pub use error::ServiceError;
pub use processor_loader::{LoadedProcessor, ProcessorLoadError, ProcessorRegistry};

pub use audio::devices::{list_output_devices};
pub use audio::queue::{PlaybackQueue, PlaybackQueueItem};
pub use audio::engine::{AudioEngine, CpalEngine};
pub use audio::player::AudioPlayer;
