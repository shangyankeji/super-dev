//! User-scope configuration at `~/.super-dev/config.toml`.
//!
//! Stores the user's chosen backend (host CLI) plus a few small UI
//! preferences. First-launch picker writes this file; later launches
//! read it and skip the picker.
//!
//! Format (all fields optional, future-additive):
//!
//! ```toml
//! # Picked at first launch; one of "claude-code" / "codex" / "offline".
//! backend = "claude-code"
//! # Model identifier passed to the worker (driver may ignore).
//! model = "claude-sonnet-4-6"
//! ```
//!
//! All read/write is fail-soft: a corrupt or missing file just means
//! "no preference yet — show the picker." Never panics.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "config.toml";
const DIR_NAME: &str = ".super-dev";

/// The on-disk shape of the user config.
#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct UserConfig {
    /// Stable backend id (`claude-code` / `codex` / `offline`).
    /// `None` triggers the first-launch picker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,

    /// Model identifier passed to the worker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl UserConfig {
    /// `true` when the user has already picked a backend.
    #[must_use]
    pub fn has_backend(&self) -> bool {
        self.backend.is_some()
    }

    /// `claude-code` / `codex` / `offline` (default when unset).
    #[must_use]
    pub fn backend_or_default(&self) -> String {
        self.backend
            .clone()
            .unwrap_or_else(|| "offline".to_string())
    }
}

/// Default location: `$XDG_CONFIG_HOME/super-dev/config.toml` if set,
/// else `$HOME/.super-dev/config.toml`.
#[must_use]
pub fn default_path() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("super-dev").join(FILE_NAME);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(DIR_NAME).join(FILE_NAME);
    }
    // Last-resort fallback so tests / CI never panic when HOME is unset.
    PathBuf::from(DIR_NAME).join(FILE_NAME)
}

/// Read the config from disk. Returns `Default::default()` on any
/// failure (missing file, parse error, IO error). Never panics.
#[must_use]
pub fn load() -> UserConfig {
    load_from(&default_path())
}

/// Read from a specific path. Same fail-soft behaviour.
#[must_use]
pub fn load_from(path: &std::path::Path) -> UserConfig {
    let Ok(body) = fs::read_to_string(path) else {
        return UserConfig::default();
    };
    toml::from_str(&body).unwrap_or_default()
}

/// Write the config to disk at the default location, creating parent
/// directories as needed. Returns an `io::Error` so callers can surface
/// it to the user — but a write failure should never crash the TUI.
pub fn save(config: &UserConfig) -> std::io::Result<PathBuf> {
    save_to(config, &default_path())
}

/// Write to a specific path. Same semantics.
pub fn save_to(config: &UserConfig, path: &std::path::Path) -> std::io::Result<PathBuf> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let body = toml::to_string_pretty(config).map_err(|e| std::io::Error::other(e.to_string()))?;
    fs::write(path, body)?;
    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_config_has_no_backend() {
        let cfg = UserConfig::default();
        assert!(cfg.backend.is_none());
        assert!(!cfg.has_backend());
    }

    #[test]
    fn round_trip_through_disk() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let original = UserConfig {
            backend: Some("claude-code".into()),
            model: Some("claude-sonnet-4-6".into()),
        };
        let written = save_to(&original, &path).unwrap();
        assert_eq!(written, path);
        let loaded = load_from(&path);
        assert_eq!(loaded, original);
    }

    #[test]
    fn load_from_missing_path_returns_default() {
        let tmp = TempDir::new().unwrap();
        let cfg = load_from(&tmp.path().join("nonexistent.toml"));
        assert_eq!(cfg, UserConfig::default());
    }

    #[test]
    fn load_from_corrupt_file_returns_default() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad.toml");
        fs::write(&path, "definitely not toml ===== broken ::: 🚫").unwrap();
        let cfg = load_from(&path);
        // Fail-soft: corrupt config doesn't crash; the picker just shows up again.
        assert!(!cfg.has_backend());
    }

    #[test]
    fn save_creates_missing_parent_directories() {
        let tmp = TempDir::new().unwrap();
        let deep = tmp.path().join("a/b/c/config.toml");
        let cfg = UserConfig {
            backend: Some("codex".into()),
            model: None,
        };
        save_to(&cfg, &deep).unwrap();
        assert!(deep.is_file());
    }

    #[test]
    fn backend_or_default_falls_back_to_offline() {
        let cfg = UserConfig::default();
        assert_eq!(cfg.backend_or_default(), "offline");
        let cfg = UserConfig {
            backend: Some("claude-code".into()),
            model: None,
        };
        assert_eq!(cfg.backend_or_default(), "claude-code");
    }

    #[test]
    fn default_path_honours_xdg_config_home() {
        // SAFETY: we mutate environment then restore — single-threaded test
        // runner (default). The env var is process-wide so we must reset
        // it before returning.
        let prev = std::env::var("XDG_CONFIG_HOME").ok();
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/xdg-test");
        let p = default_path();
        // Restore before any potential panic from assertions.
        if let Some(v) = prev {
            std::env::set_var("XDG_CONFIG_HOME", v);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
        assert!(p.starts_with("/tmp/xdg-test/super-dev"));
        assert!(p.ends_with(FILE_NAME));
    }
}
