use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use mail_sieve_store::{Connection, Transaction, settings};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::vault::Vault;

const STATES_KEY: &str = "plugins.enabled";
const MANIFEST: &str = "plugin.json";
const MAX_ID_LEN: usize = 64;

const MAX_SETTINGS_BYTES: usize = 64 * 1024;
const MAX_SECRET_NAME_LEN: usize = 64;

pub type States = BTreeMap<String, bool>;
pub type Secrets = BTreeMap<String, String>;
pub type SecretChanges = BTreeMap<String, Option<String>>;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PluginConfig {
    pub settings: Value,
    pub secrets: Vec<String>,
}

pub struct StoredConfig {
    pub settings: Value,
    pub sealed_secrets: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Manifest {
    name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    author: String,
    #[serde(default = "default_entry")]
    entry: String,
}

fn default_entry() -> String {
    "index.js".into()
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CommunityPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub entry_url: Option<String>,
    pub error: Option<String>,
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ID_LEN
        && id.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub fn states(conn: &Connection) -> mail_sieve_store::Result<States> {
    Ok(settings::get(conn, STATES_KEY)?
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default())
}

pub fn set_state(tx: &Transaction, id: &str, enabled: bool) -> mail_sieve_store::Result<States> {
    let mut current = states(tx)?;
    current.insert(id.to_string(), enabled);
    let encoded = serde_json::to_string(&current).unwrap_or_else(|_| "{}".into());
    settings::set(tx, STATES_KEY, &encoded)?;
    Ok(current)
}

fn settings_key(id: &str) -> String {
    format!("plugins.config.{id}")
}

fn secrets_key(id: &str) -> String {
    format!("plugins.secrets.{id}")
}

pub fn load_config(conn: &Connection, id: &str) -> mail_sieve_store::Result<StoredConfig> {
    let settings = settings::get(conn, &settings_key(id))?
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or(Value::Null);
    let sealed_secrets = settings::get(conn, &secrets_key(id))?;
    Ok(StoredConfig {
        settings,
        sealed_secrets,
    })
}

pub fn save_config(
    tx: &Transaction,
    id: &str,
    settings_value: &Value,
    sealed_secrets: &str,
) -> mail_sieve_store::Result<()> {
    settings::set(tx, &settings_key(id), &settings_value.to_string())?;
    settings::set(tx, &secrets_key(id), sealed_secrets)
}

pub fn open_secrets(vault: &Vault, sealed: Option<&str>) -> Result<Secrets> {
    let Some(encoded) = sealed.filter(|encoded| !encoded.is_empty()) else {
        return Ok(Secrets::new());
    };
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| Error::Internal("stored plugin secrets are corrupt".into()))?;
    vault.open_value(&bytes)
}

pub fn seal_secrets(vault: &Vault, secrets: &Secrets) -> Result<String> {
    Ok(STANDARD.encode(vault.seal_value(secrets)?))
}

pub fn validate_settings(value: &Value) -> Result<()> {
    if value.to_string().len() > MAX_SETTINGS_BYTES {
        return Err(Error::BadRequest(format!(
            "plugin settings must be under {} KB",
            MAX_SETTINGS_BYTES / 1024
        )));
    }
    Ok(())
}

pub fn apply_secret_changes(mut secrets: Secrets, changes: SecretChanges) -> Result<Secrets> {
    for (name, value) in changes {
        let valid = !name.is_empty()
            && name.len() <= MAX_SECRET_NAME_LEN
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !valid {
            return Err(Error::BadRequest(format!(
                "`{name}` is not a valid secret name"
            )));
        }
        match value.map(|value| value.trim().to_string()) {
            Some(value) if !value.is_empty() => {
                secrets.insert(name, value);
            }
            _ => {
                secrets.remove(&name);
            }
        }
    }
    Ok(secrets)
}

pub fn discover(dir: &Path) -> Vec<CommunityPlugin> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<CommunityPlugin> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let id = entry.file_name().to_str()?.to_string();
            valid_id(&id).then(|| load(&entry.path(), id))
        })
        .collect();
    found.sort_by(|a, b| a.id.cmp(&b.id));
    found
}

fn load(dir: &Path, id: String) -> CommunityPlugin {
    let broken = |error: String| CommunityPlugin {
        id: id.clone(),
        name: id.clone(),
        version: String::new(),
        description: String::new(),
        author: String::new(),
        entry_url: None,
        error: Some(error),
    };
    let raw = match std::fs::read_to_string(dir.join(MANIFEST)) {
        Ok(raw) => raw,
        Err(error) => return broken(format!("cannot read {MANIFEST}: {error}")),
    };
    let manifest: Manifest = match serde_json::from_str(&raw) {
        Ok(manifest) => manifest,
        Err(error) => return broken(format!("invalid {MANIFEST}: {error}")),
    };
    if resolve_file(dir, &manifest.entry).is_none() {
        return broken(format!("entry `{}` not found", manifest.entry));
    }
    CommunityPlugin {
        entry_url: Some(format!("/api/plugins/{id}/files/{}", manifest.entry)),
        id,
        name: manifest.name,
        version: manifest.version,
        description: manifest.description,
        author: manifest.author,
        error: None,
    }
}

pub fn resolve_file(plugin_dir: &Path, relative: &str) -> Option<PathBuf> {
    let relative = Path::new(relative);
    let plain = relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)));
    if !plain || relative.as_os_str().is_empty() {
        return None;
    }
    let root = plugin_dir.canonicalize().ok()?;
    let file = root.join(relative).canonicalize().ok()?;
    (file.starts_with(&root) && file.is_file()).then_some(file)
}

pub fn check_id(id: &str) -> Result<()> {
    if valid_id(id) {
        Ok(())
    } else {
        Err(Error::BadRequest(format!(
            "`{id}` is not a valid plugin id"
        )))
    }
}

pub fn plugin_dir(plugins_root: &Path, id: &str) -> Result<PathBuf> {
    check_id(id)?;
    Ok(plugins_root.join(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_changes_set_keep_and_clear() {
        let current = Secrets::from([
            ("api_key".to_string(), "old".to_string()),
            ("token".to_string(), "keep".to_string()),
        ]);
        let changes = SecretChanges::from([
            ("api_key".to_string(), Some(" new ".to_string())),
            ("other".to_string(), Some(String::new())),
        ]);
        let next = apply_secret_changes(current.clone(), changes).unwrap();
        assert_eq!(next.get("api_key").map(String::as_str), Some("new"));
        assert_eq!(next.get("token").map(String::as_str), Some("keep"));
        assert!(!next.contains_key("other"));
        let cleared =
            apply_secret_changes(current, SecretChanges::from([("token".to_string(), None)]))
                .unwrap();
        assert!(!cleared.contains_key("token"));
        assert!(
            apply_secret_changes(
                Secrets::new(),
                SecretChanges::from([("Bad Name".into(), None)])
            )
            .is_err()
        );
    }

    #[test]
    fn secrets_are_sealed_at_rest() {
        let vault = Vault::new(&[3; 32]);
        let secrets = Secrets::from([("api_key".to_string(), "sk-secret".to_string())]);
        let sealed = seal_secrets(&vault, &secrets).unwrap();
        assert!(!sealed.contains("sk-secret"));
        assert_eq!(open_secrets(&vault, Some(&sealed)).unwrap(), secrets);
        assert!(open_secrets(&vault, None).unwrap().is_empty());
    }

    #[test]
    fn ids_are_lowercase_slugs() {
        assert!(valid_id("wingman"));
        assert!(valid_id("my-plugin-2"));
        assert!(!valid_id(""));
        assert!(!valid_id("-lead"));
        assert!(!valid_id("Upper"));
        assert!(!valid_id("../etc"));
        assert!(!valid_id(&"a".repeat(65)));
    }

    #[test]
    fn files_cannot_escape_the_plugin_dir() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("demo");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::write(dir.join("index.js"), "export default {}").unwrap();
        std::fs::write(dir.join("lib/util.js"), "").unwrap();
        std::fs::write(root.path().join("secret.txt"), "").unwrap();
        assert!(resolve_file(&dir, "index.js").is_some());
        assert!(resolve_file(&dir, "lib/util.js").is_some());
        assert!(resolve_file(&dir, "../secret.txt").is_none());
        assert!(resolve_file(&dir, "/etc/passwd").is_none());
        assert!(resolve_file(&dir, "lib").is_none());
        assert!(resolve_file(&dir, "").is_none());
    }

    #[test]
    fn discovers_manifests_and_reports_broken_ones() {
        let root = tempfile::tempdir().unwrap();
        let good = root.path().join("good");
        std::fs::create_dir_all(&good).unwrap();
        std::fs::write(
            good.join("plugin.json"),
            r#"{"name":"Good","version":"1.0.0"}"#,
        )
        .unwrap();
        std::fs::write(good.join("index.js"), "").unwrap();
        let bad = root.path().join("bad");
        std::fs::create_dir_all(&bad).unwrap();
        std::fs::write(bad.join("plugin.json"), "{").unwrap();
        std::fs::create_dir_all(root.path().join("Not Valid")).unwrap();

        let found = discover(root.path());
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].id, "bad");
        assert!(found[0].error.is_some());
        assert_eq!(found[1].name, "Good");
        assert_eq!(
            found[1].entry_url.as_deref(),
            Some("/api/plugins/good/files/index.js")
        );
        assert!(discover(&root.path().join("missing")).is_empty());
    }
}
