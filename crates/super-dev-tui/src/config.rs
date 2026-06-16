//! User-scope configuration at `~/.super-dev/config.toml`.
//!
//! Stores the user's chosen runtime — a host CLI backend, a custom API
//! provider, or offline templates — plus a few small UI preferences.
//! First-launch picker writes this file; later launches read it and skip
//! the picker.
//!
//! Format (all fields optional, future-additive):
//!
//! ```toml
//! # Path A — drive a logged-in host CLI (no API key needed).
//! backend = "claude-code"
//! model = "claude-sonnet-4-6"
//!
//! # Path B — point Super Dev at a custom OpenAI-compatible or Anthropic
//! # endpoint with your own key. Comment out `backend` above and set a
//! # `default_provider` instead. `${VAR}` references are resolved from the
//! # environment at call time so real keys never have to live in this file.
//! default_provider = "deepseek"
//!
//! [providers.deepseek]
//! kind     = "openai"                      # or "anthropic"
//! base_url = "https://api.deepseek.com/v1"
//! api_key  = "${DEEPSEEK_API_KEY}"
//! model    = "deepseek-chat"
//! ```
//!
//! All read/write is fail-soft: a corrupt or missing file just means
//! "no preference yet — show the picker." Never panics.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "config.toml";
const DIR_NAME: &str = ".super-dev";

/// One named custom-API endpoint — a provider entry in `[providers.<name>]`.
///
/// `kind` selects the wire protocol, not the vendor: `"openai"` covers every
/// OpenAI-compatible server (`OpenAI` itself, `DeepSeek`, `OpenRouter`,
/// `Together`, `Groq`, Ollama `/v1`, `vLLM`, `LM Studio`, 智谱, 阿里百炼, …),
/// while `"anthropic"` speaks Anthropic's native Messages API. `api_key` may
/// be a bare literal or a `"${ENV_VAR}"` reference resolved at call time.
#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Wire protocol: `"openai"` (default) or `"anthropic"`.
    #[serde(default = "default_provider_kind")]
    pub kind: String,
    /// API root URL (no `/chat/completions` or `/v1/messages` suffix).
    /// A trailing slash is tolerated.
    pub base_url: String,
    /// API key — bare literal or `"${ENV_VAR}"` reference.
    #[serde(default)]
    pub api_key: String,
    /// Model identifier sent in the request body.
    pub model: String,
}

/// Default `kind` when omitted in TOML.
fn default_provider_kind() -> String {
    "openai".to_string()
}

impl ProviderConfig {
    /// `true` when `kind` is a recognised wire protocol.
    #[must_use]
    pub fn kind_is_known(&self) -> bool {
        matches!(self.kind.as_str(), "openai" | "anthropic")
    }
}

/// The on-disk shape of the user config.
#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct UserConfig {
    /// Stable backend id (`claude-code` / `codex` / `offline`).
    /// `None` triggers the first-launch picker.
    /// Mutually exclusive with [`UserConfig::default_provider`] — when both
    /// are set, the provider takes precedence (custom API wins over host CLI).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,

    /// Model identifier passed to the worker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Active design system name (e.g. `modern-minimal`, `tech-utility`).
    /// Saved to config so subsequent runs reuse the same visual direction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub design_system: Option<String>,

    /// Active seed template (e.g. `saas-landing`, `dashboard`, `blog-content`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed_template: Option<String>,

    /// Name of the active custom-API provider (a key into
    /// [`UserConfig::providers`]). When set, Super Dev calls that provider's
    /// HTTP endpoint directly instead of driving a host CLI. Empty/`None`
    /// means "no custom provider — use `backend` or offline templates."
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_provider: Option<String>,

    /// Named custom-API endpoints. Keyed by the name used in
    /// `default_provider` / `/provider <name>`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub providers: BTreeMap<String, ProviderConfig>,
}

impl UserConfig {
    /// `true` when the user has already picked a backend OR a custom provider.
    #[must_use]
    pub fn has_backend(&self) -> bool {
        self.backend.is_some() || self.default_provider.is_some()
    }

    /// `claude-code` / `codex` / `offline` (default when unset).
    #[must_use]
    pub fn backend_or_default(&self) -> String {
        self.backend
            .clone()
            .unwrap_or_else(|| "offline".to_string())
    }

    /// Resolve the effective custom provider, honouring a project-level
    /// override (`proj`) that may be `Some(name)` / `Some("")` (explicitly
    /// disabled) / `None` (no opinion). Project wins; then global; then `None`.
    ///
    /// An empty project override string means "explicitly use no provider"
    /// (fall through to host CLI / offline), which is how `/provider off`
    /// records its choice.
    #[must_use]
    pub fn effective_provider(&self, proj: Option<&str>) -> Option<&ProviderConfig> {
        match proj {
            Some(name) if !name.is_empty() => self.providers.get(name),
            Some(_) => None, // project explicitly disabled the provider
            None => self
                .default_provider
                .as_deref()
                .and_then(|n| self.providers.get(n)),
        }
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
    // Cross-platform home: HOME on Unix, USERPROFILE on Windows.
    if let Some(home) = home_dir() {
        return home.join(DIR_NAME).join(FILE_NAME);
    }
    // Last-resort fallback so tests / CI never panic when HOME is unset.
    PathBuf::from(DIR_NAME).join(FILE_NAME)
}

/// Cross-platform home directory: `HOME` then `USERPROFILE` (Windows).
fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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

    #[test]
    fn provider_section_round_trips() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut providers = BTreeMap::new();
        providers.insert(
            "deepseek".into(),
            ProviderConfig {
                kind: "openai".into(),
                base_url: "https://api.deepseek.com/v1".into(),
                api_key: "${DEEPSEEK_API_KEY}".into(),
                model: "deepseek-chat".into(),
            },
        );
        let original = UserConfig {
            default_provider: Some("deepseek".into()),
            providers,
            ..Default::default()
        };
        save_to(&original, &path).unwrap();
        let loaded = load_from(&path);
        assert_eq!(loaded, original);
        // The provider is the effective one.
        let eff = loaded.effective_provider(None).expect("provider resolved");
        assert_eq!(eff.base_url, "https://api.deepseek.com/v1");
        assert_eq!(eff.api_key, "${DEEPSEEK_API_KEY}");
    }

    #[test]
    fn effective_provider_honours_project_override() {
        let mut providers = BTreeMap::new();
        providers.insert(
            "global-p".into(),
            ProviderConfig {
                kind: "openai".into(),
                base_url: "https://global".into(),
                api_key: "k".into(),
                model: "m".into(),
            },
        );
        providers.insert(
            "proj-p".into(),
            ProviderConfig {
                kind: "anthropic".into(),
                base_url: "https://proj".into(),
                api_key: "k".into(),
                model: "m".into(),
            },
        );
        let cfg = UserConfig {
            default_provider: Some("global-p".into()),
            providers,
            ..Default::default()
        };
        // No project override → global default.
        assert_eq!(
            cfg.effective_provider(None).unwrap().base_url,
            "https://global"
        );
        // Project override wins.
        assert_eq!(
            cfg.effective_provider(Some("proj-p")).unwrap().base_url,
            "https://proj"
        );
        // Project explicitly disabled ("") → None, falls through to host CLI.
        assert!(cfg.effective_provider(Some("")).is_none());
        // Unknown name → None (fail-open, surfaces as a clear TUI error).
        assert!(cfg.effective_provider(Some("ghost")).is_none());
    }

    #[test]
    fn unknown_provider_kind_still_parses() {
        // A typo'd kind must not break the whole config — only that provider
        // is unusable, surfaced when the TUI tries to build a runtime from it.
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        fs::write(
            &path,
            "default_provider = \"x\"\n[providers.x]\nkind = \"quantum\"\nbase_url = \"u\"\nmodel = \"m\"\n",
        )
        .unwrap();
        let cfg = load_from(&path);
        let p = cfg.effective_provider(None).expect("provider present");
        assert_eq!(p.kind, "quantum");
        assert!(!p.kind_is_known());
    }

    #[test]
    fn provider_kind_defaults_to_openai_when_omitted() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        // kind omitted — should default to "openai".
        fs::write(
            &path,
            "default_provider = \"x\"\n[providers.x]\nbase_url = \"u\"\nmodel = \"m\"\n",
        )
        .unwrap();
        let cfg = load_from(&path);
        let p = cfg.effective_provider(None).unwrap();
        assert_eq!(p.kind, "openai");
        assert!(p.kind_is_known());
    }

    #[test]
    fn has_backend_true_when_only_provider_set() {
        let mut providers = BTreeMap::new();
        providers.insert(
            "p".into(),
            ProviderConfig {
                base_url: "u".into(),
                model: "m".into(),
                ..Default::default()
            },
        );
        let cfg = UserConfig {
            default_provider: Some("p".into()),
            providers,
            ..Default::default()
        };
        assert!(cfg.has_backend());
    }
}
