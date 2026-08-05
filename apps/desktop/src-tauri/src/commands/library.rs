use tauri::State;

use musicum_core::config::Config;
use musicum_core::db::DatabaseConnection;
use musicum_core::db::entities::file_metadata;
use musicum_core::services::{
    clip_service, file_service, ClipListItem, FileListItem,
};

pub struct AppState {
    pub db: Result<DatabaseConnection, String>,
}

#[tauri::command]
pub async fn list_files(
    state: State<'_, AppState>,
) -> Result<Vec<FileListItem>, String> {
    let db = state.db.as_ref().map_err(|e| e.clone())?;
    file_service::list_files_with_clips(db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_clips(
    state: State<'_, AppState>,
) -> Result<Vec<ClipListItem>, String> {
    let db = state.db.as_ref().map_err(|e| e.clone())?;
    clip_service::list_clips_with_files(db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_library_dir() -> String {
    Config::get()
        .library
        .files_dir
        .to_string_lossy()
        .into_owned()
}

#[tauri::command]
pub async fn get_file_with_clips_by_slug(
    state: State<'_, AppState>,
    slug: String,
) -> Result<FileListItem, String> {
    let db = state.db.as_ref().map_err(|e| e.clone())?;
    file_service::get_file_with_clips_by_slug(db, &slug)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_file_metadata(
    state: State<'_, AppState>,
    file_id: String,
) -> Result<Option<file_metadata::Model>, String> {
    let db = state.db.as_ref().map_err(|e| e.clone())?;
    file_service::get_file_metadata(db, &file_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_file_notes(
    state: State<'_, AppState>,
    slug: String,
    notes: String,
) -> Result<(), String> {
    let db = state.db.as_ref().map_err(|e| e.clone())?;
    file_service::set_file_notes(db, &slug, &notes)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_file_tags(
    state: State<'_, AppState>,
    slug: String,
    tags: String,
) -> Result<(), String> {
    let db = state.db.as_ref().map_err(|e| e.clone())?;
    file_service::set_file_tags(db, &slug, &tags)
        .await
        .map_err(|e| e.to_string())
}
