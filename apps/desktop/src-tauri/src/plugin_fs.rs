use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PluginSource {
    Bundled,
    User,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestFile {
    id: String,
    name: String,
    version: String,
    #[serde(default)]
    description: Option<String>,
    entry: String,
    #[serde(rename = "apiVersion")]
    api_version: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    #[serde(rename = "entryPath")]
    pub entry_path: String,
    pub source: PluginSource,
    #[serde(rename = "apiVersion")]
    pub api_version: u32,
}

/// Scan a directory one level deep for `<bundle>/plugin.json`.
/// Malformed manifests are skipped with a warning; the returned vec preserves scan order.
pub fn scan_dir(dir: &Path, source: PluginSource) -> Vec<PluginManifest> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let bundle_dir = entry.path();
        if !bundle_dir.is_dir() {
            continue;
        }
        let manifest_path = bundle_dir.join("plugin.json");
        let raw = match std::fs::read_to_string(&manifest_path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let parsed: ManifestFile = match serde_json::from_str(&raw) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(path = %manifest_path.display(), error = %e, "invalid plugin manifest");
                continue;
            }
        };
        let entry_path = bundle_dir.join(&parsed.entry);
        if !entry_path.exists() {
            tracing::warn!(path = %entry_path.display(), "plugin entry file missing");
            continue;
        }
        out.push(PluginManifest {
            id: parsed.id,
            name: parsed.name,
            version: parsed.version,
            description: parsed.description,
            entry_path: entry_path.to_string_lossy().into_owned(),
            source,
            api_version: parsed.api_version,
        });
    }
    out
}

pub fn scan_all(bundled: &Path, user: &Path) -> Vec<PluginManifest> {
    let mut out = scan_dir(bundled, PluginSource::Bundled);
    out.extend(scan_dir(user, PluginSource::User));
    out
}

/// Ensure the user plugin dir exists so users can drop files in without a manual mkdir.
pub fn ensure_user_dir(dir: &Path) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    Ok(dir.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_bundle(root: &Path, id: &str, manifest: &str, entry: &str) {
        let dir = root.join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("plugin.json"), manifest).unwrap();
        std::fs::write(dir.join("index.js"), entry).unwrap();
    }

    #[test]
    fn scans_valid_bundles() {
        let tmp = TempDir::new().unwrap();
        write_bundle(
            tmp.path(),
            "alpha",
            r#"{"id":"alpha","name":"Alpha","version":"0.1.0","entry":"index.js","apiVersion":1}"#,
            "// alpha",
        );
        let out = scan_dir(tmp.path(), PluginSource::User);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "alpha");
        assert_eq!(out[0].source, PluginSource::User);
        assert!(out[0].entry_path.ends_with("index.js"));
    }

    #[test]
    fn skips_malformed_manifests() {
        let tmp = TempDir::new().unwrap();
        write_bundle(tmp.path(), "bad", "not json", "// x");
        write_bundle(
            tmp.path(),
            "good",
            r#"{"id":"good","name":"Good","version":"0.1.0","entry":"index.js","apiVersion":1}"#,
            "// y",
        );
        let out = scan_dir(tmp.path(), PluginSource::User);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "good");
    }

    #[test]
    fn skips_missing_entry_file() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("noentry");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("plugin.json"),
            r#"{"id":"noentry","name":"NE","version":"0.1.0","entry":"missing.js","apiVersion":1}"#,
        )
        .unwrap();
        let out = scan_dir(tmp.path(), PluginSource::User);
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn returns_empty_for_nonexistent_dir() {
        let out = scan_dir(Path::new("/nonexistent/path/xyz"), PluginSource::User);
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn scan_all_tags_sources() {
        let bundled = TempDir::new().unwrap();
        let user = TempDir::new().unwrap();
        write_bundle(
            bundled.path(),
            "b",
            r#"{"id":"b","name":"B","version":"0.1.0","entry":"index.js","apiVersion":1}"#,
            "",
        );
        write_bundle(
            user.path(),
            "u",
            r#"{"id":"u","name":"U","version":"0.1.0","entry":"index.js","apiVersion":1}"#,
            "",
        );
        let out = scan_all(bundled.path(), user.path());
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].source, PluginSource::Bundled);
        assert_eq!(out[1].source, PluginSource::User);
    }
}
