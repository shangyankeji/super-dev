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
//!
//! [knowledge]
//! enabled = true                       # use structured BM25 retrieval (default)
//! engine = "bm25"                      # "bm25" (offline) or "hybrid" (+ OpenAI embeddings)
//! top_k = 6                            # chunks injected per phase
//!
//! [model]
//! provider = "deepseek"                # use this named provider (defined in
//!                                       # ~/.super-dev/config.toml) instead of
//!                                       # the global default. Empty = disable
//!                                       # any custom provider for this project.
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
    /// Knowledge-base / RAG retrieval overrides.
    #[serde(default)]
    pub knowledge: KnowledgeConfig,
    /// Custom-model (API provider) overrides.
    #[serde(default)]
    pub model: ModelConfig,
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

/// Custom-model (API provider) overrides.
///
/// A project can pin which named provider (defined in the user's
/// `~/.super-dev/config.toml`) runs this project's pipeline. This lets one
/// machine use different models for different projects — e.g. a cheap model
/// for throwaway experiments, a strong one for the production repo.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ModelConfig {
    /// Name of the provider to use for this project, overriding the global
    /// `default_provider`. An empty string means "explicitly use no custom
    /// provider" (fall back to host CLI / offline). `None` (field absent)
    /// means "no opinion — use the global setting."
    #[serde(default)]
    pub provider: Option<String>,
}

/// Knowledge-base (RAG) retrieval customization.
///
/// Controls the [`super_dev_knowledge`] retrieval engine that replaces the
/// legacy "keyword-sort the folder" approach. Default uses BM25 (offline,
/// zero-dependency); setting `engine = "hybrid"` additionally enables the
/// optional OpenAI embeddings layer when `OPENAI_EMBED_KEY` is present.
#[derive(Debug, Clone, Deserialize)]
pub struct KnowledgeConfig {
    /// Whether structured retrieval is enabled (default `true`). Set
    /// `false` to fall back to the legacy keyword-scoring path.
    #[serde(default = "default_knowledge_enabled")]
    pub enabled: bool,
    /// Retrieval engine: `"bm25"` (default) or `"hybrid"`.
    #[serde(default = "default_knowledge_engine")]
    pub engine: String,
    /// How many knowledge chunks to inject per phase (default 6).
    #[serde(default = "default_knowledge_top_k")]
    pub top_k: usize,
}

impl Default for KnowledgeConfig {
    fn default() -> Self {
        Self {
            enabled: default_knowledge_enabled(),
            engine: default_knowledge_engine(),
            top_k: default_knowledge_top_k(),
        }
    }
}

fn default_knowledge_enabled() -> bool {
    true
}

fn default_knowledge_engine() -> String {
    "bm25".to_string()
}

fn default_knowledge_top_k() -> usize {
    6
}

/// Read `.superdevrc` from the project root. Returns `Default` if missing
/// or malformed (fail-soft, same as UserConfig).
#[must_use]
pub fn load_project_config(project_root: &Path) -> ProjectConfig {
    let path = project_root.join(".superdevrc");
    let Ok(body) = std::fs::read_to_string(&path) else {
        return ProjectConfig::default();
    };
    let mut cfg: ProjectConfig = toml::from_str(&body).unwrap_or_default();
    // Validate the knowledge engine: only "bm25" and "hybrid" are legal.
    // Unknown values (e.g. "quantum") silently fall back to "bm25" so a
    // typo never breaks retrieval.
    if cfg.knowledge.engine != "bm25" && cfg.knowledge.engine != "hybrid" {
        if !cfg.knowledge.engine.is_empty() {
            tracing::warn!(
                "knowledge.engine = {:?} is not bm25 or hybrid — falling back to bm25",
                cfg.knowledge.engine
            );
        }
        cfg.knowledge.engine = "bm25".to_string();
    }
    // Clamp quality threshold and top_k to sensible bounds.
    cfg.quality.threshold = cfg.quality.threshold.min(100);
    cfg.knowledge.top_k = cfg.knowledge.top_k.clamp(1, 50);
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn invalid_engine_falls_back_to_bm25() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".superdevrc"),
            "[knowledge]\nengine = \"quantum\"\ntop_k = 999\n",
        )
        .unwrap();
        let cfg = load_project_config(tmp.path());
        assert_eq!(cfg.knowledge.engine, "bm25", "quantum must fall back");
        assert_eq!(cfg.knowledge.top_k, 50, "top_k must clamp to 50");
    }

    #[test]
    fn valid_hybrid_engine_preserved() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".superdevrc"),
            "[knowledge]\nengine = \"hybrid\"\ntop_k = 8\n",
        )
        .unwrap();
        let cfg = load_project_config(tmp.path());
        assert_eq!(cfg.knowledge.engine, "hybrid");
        assert_eq!(cfg.knowledge.top_k, 8);
    }

    #[test]
    fn threshold_clamped_to_100() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".superdevrc"),
            "[quality]\nthreshold = 999\n",
        )
        .unwrap();
        let cfg = load_project_config(tmp.path());
        assert_eq!(cfg.quality.threshold, 100);
    }

    #[test]
    fn default_config_has_sane_values() {
        let cfg = ProjectConfig::default();
        assert_eq!(cfg.quality.threshold, 90);
        assert_eq!(cfg.pipeline.max_review_rounds, 3);
        assert!(cfg.pipeline.skip_phases.is_empty());
        // Knowledge defaults: BM25 enabled, top_k 6.
        assert!(cfg.knowledge.enabled);
        assert_eq!(cfg.knowledge.engine, "bm25");
        assert_eq!(cfg.knowledge.top_k, 6);
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

    #[test]
    fn knowledge_section_parses() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".superdevrc"),
            "[knowledge]\nengine = \"hybrid\"\ntop_k = 10\n",
        )
        .unwrap();
        let cfg = load_project_config(tmp.path());
        assert_eq!(cfg.knowledge.engine, "hybrid");
        assert_eq!(cfg.knowledge.top_k, 10);
        assert!(cfg.knowledge.enabled); // defaults to true when omitted
    }

    #[test]
    fn knowledge_disabled_parses() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".superdevrc"),
            "[knowledge]\nenabled = false\n",
        )
        .unwrap();
        let cfg = load_project_config(tmp.path());
        assert!(!cfg.knowledge.enabled);
    }

    #[test]
    fn model_section_parses() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".superdevrc"),
            "[model]\nprovider = \"deepseek\"\n",
        )
        .unwrap();
        let cfg = load_project_config(tmp.path());
        assert_eq!(cfg.model.provider.as_deref(), Some("deepseek"));
    }

    #[test]
    fn model_section_disabled_via_empty_string() {
        let tmp = TempDir::new().unwrap();
        // Empty string = "explicitly use no provider for this project".
        std::fs::write(tmp.path().join(".superdevrc"), "[model]\nprovider = \"\"\n").unwrap();
        let cfg = load_project_config(tmp.path());
        assert_eq!(cfg.model.provider.as_deref(), Some(""));
    }

    #[test]
    fn model_section_absent_is_none() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".superdevrc"),
            "[quality]\nthreshold = 80\n",
        )
        .unwrap();
        let cfg = load_project_config(tmp.path());
        assert!(cfg.model.provider.is_none());
    }
}
