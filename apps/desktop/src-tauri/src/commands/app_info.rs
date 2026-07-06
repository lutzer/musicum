use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

pub fn build_app_info() -> AppInfo {
    AppInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    build_app_info()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_crate_name_and_version() {
        let info = build_app_info();
        assert_eq!(info.name, "musicum-desktop");
        assert_eq!(info.version, "0.1.0");
    }
}
