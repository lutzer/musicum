use crate::plugin_fs::{ensure_user_dir, scan_all, PluginManifest};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn list_plugins(app: AppHandle) -> Vec<PluginManifest> {
    let bundled = match app.path().resource_dir() {
        Ok(p) => p.join("plugins"),
        Err(e) => {
            tracing::warn!(error = %e, "could not resolve resource dir");
            return Vec::new();
        }
    };
    let user_root = match app.path().app_data_dir() {
        Ok(p) => p.join("plugins"),
        Err(e) => {
            tracing::warn!(error = %e, "could not resolve app data dir");
            return scan_all(&bundled, std::path::Path::new(""));
        }
    };
    if let Err(e) = ensure_user_dir(&user_root) {
        tracing::warn!(error = %e, dir = %user_root.display(), "could not create user plugin dir");
    }
    scan_all(&bundled, &user_root)
}
