pub mod commands;
pub mod plugin_fs;

use tauri::Manager;

use commands::library::AppState;
use musicum_core::config::{self, Config};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            config::init(None);
            let catalog_dir = Config::get().library.catalog_dir.clone();

            let db_result = tauri::async_runtime::block_on(async {
                musicum_core::db::connect(&catalog_dir)
                    .await
                    .map_err(|e| format!(
                        "failed to open library at {}: {e}",
                        catalog_dir.display(),
                    ))
            });

            if let Err(ref e) = db_result {
                tracing::error!("DB init failed: {e}");
            }

            app.manage(AppState { db: db_result });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info::get_app_info,
            commands::plugins::list_plugins,
            commands::library::list_files,
            commands::library::list_clips,
            commands::library::get_library_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
