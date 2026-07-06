use tauri::State;

use musicum_core::db::DatabaseConnection;
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
