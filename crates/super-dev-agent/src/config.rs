//! Project-level configuration overrides via `.superdevrc`.
//!
//! Users can create a `.superdevrc` file (TOML) in the project root to
//! customize Super Dev behavior per-project without modifying the global
//! config.
//!
//! ```toml
//! # .superdevrc
//! [quality]
//! threshold = 85              # override quality gate pass threshold
//! skip_checks = ["dark_mode"] # skip specific quality checks
//!
//! [pipeline]
//! skip_phases = ["research"]  # skip research if you already did it
//! max_review_rounds = 2       # limit review→fix cycles
//!
//! [experts]
//! custom_knowledge = "team-standards/" # extra knowledge directory
//! ```

use std::path::Path;

use serde::Deserialize;

/// Project-level overrides from `.superdevrc`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectConfig {
    /// Quality gate overrides.
    #[serde(default)]
    pub quality: QualityConfig,
    /// Pipeline behavior overrides.
    #[serde(default)]
    pub pipeline: PipelineConfig,
    /// Expert knowledge overrides.
    #[serde(default)]
    pub experts: ExpertsConfig,
}

/// Quality gate customization.
#[derive(Debug, Clone, Deserialize)]
pub struct QualityConfig {
    /// Minimum score to pass (default 90).
    #[serde(default = "default_threshold")]
    pub threshold: u32,
    /// Check names to skip (e.g. `dark_mode`).
    #[serde(default)]
    pub skip_checks: Vec<String>,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            threshold: default_threshold(),
            skip_checks: Vec::new(),
        }
    }
}

fn default_threshold() -> u32 {
    90
}

/// Pipeline behavior customization.
#[derive(Debug, Clone, Deserialize)]
pub struct PipelineConfig {
    /// Phases to skip (e.g. `research` if you already did it).
    #[serde(default)]
    pub skip_phases: Vec<String>,
    /// Max review→fix rounds per document (default 3).
    #[serde(default = "default_review_rounds")]
    pub max_review_rounds: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            skip_phases: Vec::new(),
            max_review_rounds: default_review_rounds(),
        }
    }
}

fn default_review_rounds() -> usize {
    3
}

/// Expert knowledge customization.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExpertsConfig {
    /// Additional knowledge directory (relative to project root).
    /// Files in this dir are injected alongside built-in expert knowledge.
    #[serde(default)]
    pub custom_knowledge: Option<String>,
}

/// Read `.superdevrc` from the project root. Returns `Default` if missing
/// or malformed (fail-soft, same as UserConfig).
#[must_use]
pub fn load_project_config(project_root: &Path) -> ProjectConfig {
    let path = project_root.join(".superdevrc");
    let Ok(body) = std::fs::read_to_string(&path) else {
        return ProjectConfig::default();
    };
    toml::from_str(&body).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_config_has_sane_values() {
        let cfg = ProjectConfig::default();
        assert_eq!(cfg.quality.threshold, 90);
        assert_eq!(cfg.pipeline.max_review_rounds, 3);
        assert!(cfg.pipeline.skip_phases.is_empty());
    }

    #[test]
    fn load_from_missing_file_returns_default() {
        let tmp = TempDir::new().unwrap();
        let cfg = load_project_config(tmp.path());
        assert_eq!(cfg.quality.threshold, 90);
    }

    #[test]
    fn load_parses_toml() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".superdevrc"),
            "[quality]\nthreshold = 80\nskip_checks = [\"dark_mode\"]\n\n[pipeline]\nmax_review_rounds = 2\n",
        )
        .unwrap();
        let cfg = load_project_config(tmp.path());
        assert_eq!(cfg.quality.threshold, 80);
        assert_eq!(cfg.quality.skip_checks, vec!["dark_mode"]);
        assert_eq!(cfg.pipeline.max_review_rounds, 2);
    }
}
