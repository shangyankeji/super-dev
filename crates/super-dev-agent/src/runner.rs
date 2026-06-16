//! Agent runner — drives the 9-phase pipeline.
//!
//! V1 deterministic pipeline:
//!
//! - `run_initial_block`:    research → docs → pause at `docs_confirm`
//! - `continue_after_docs`:  spec → frontend → pause at `preview_confirm`
//! - `continue_after_preview`: backend → quality → delivery → done
//!
//! Later milestones swap the deterministic phase bodies for LLM-driven
//! ones without changing this orchestration.

use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use super_dev_runtime::Runtime;
use super_dev_spec::{Phase, SPEC_VERSION};

use crate::coach::write_coach_prompt;
use crate::events::{null_sink, EngineEvent, EventSink};
use crate::experts::{
    architecture_prompt, backend_prompt, delivery_prompt, excerpt, frontend_prompt, prd_prompt,
    research_prompt, uiux_prompt, Prompt,
};
use crate::gates::Gate;
use crate::phases::{
    run_backend, run_delivery, run_docs, run_frontend, run_quality, run_research, run_spec,
    DocsContent, PhaseOutput,
};
use crate::state::{write_workflow_state, WorkflowState};

/// Whether the markdown section starting at byte `heading_pos` (the heading
/// line for `heading`) has any non-empty body content before the next `##`
/// heading. Catches "present but empty" sections a bare `## goal` would
/// otherwise let through review.
fn section_has_body(lower: &str, heading_pos: usize, heading: &str) -> bool {
    // Body = lines after the heading line, up to the NEXT H2 heading
    // (a line starting with `## ` but NOT `### ` — sub-headings are body).
    // The previous `find("\n##")` treated any `##` substring (including
    // `### In scope`) as a section boundary, so multi-level sections read
    // as empty. Now we split on lines and only stop at a true H2 peer.
    let after_heading = heading_pos + heading.len();
    let rest = &lower[after_heading.min(lower.len())..];
    for line in rest.lines() {
        let trimmed = line.trim_start();
        // A peer H2 heading ends this section: starts with "## " but not "###".
        if trimmed.starts_with("## ") && !trimmed.starts_with("###") {
            break;
        }
        let l = line.trim();
        if !l.is_empty() && !l.starts_with('#') {
            return true;
        }
    }
    false
}

/// Per-phase generation token budget. Long-form artifact phases (docs /
/// architecture / PRD) get a larger budget so a big document isn't
/// truncated; shorter phases (research/spec/quality) get less. Override
/// via `SUPER_DEV_MAX_TOKENS` (>0 = fixed cap for all phases).
fn max_tokens_for_phase(phase: Phase) -> u32 {
    if let Ok(v) = std::env::var("SUPER_DEV_MAX_TOKENS") {
        if let Ok(n) = v.parse::<u32>() {
            if n > 0 {
                return n;
            }
        }
    }
    match phase {
        // Docs produces PRD + architecture + UIUX — the longest artifacts.
        // Backend also emits substantial code + integration notes.
        Phase::Docs | Phase::Backend => 8192,
        // Frontend/quality/delivery/spec/research — moderate.
        _ => 4096,
    }
}

/// User-facing run configuration.
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// Workspace root the agent operates inside.
    pub project_root: PathBuf,
    /// Free-form user requirement (e.g. "做一个登录系统").
    pub requirement: String,
    /// Slug used in artifact filenames. Defaults to the workspace dir
    /// name when callers leave it empty.
    pub slug: String,
    /// Model identifier passed to the runtime (provider-specific).
    pub model: String,
    /// Backend id that's driving this run (e.g. `claude-code`, `codex`).
    /// Empty when running offline templates. Persisted into the workflow
    /// state so subsequent `continue` / `revise` calls can resume against
    /// the same worker without a flag.
    pub backend: String,
    /// Active design system name (e.g. `modern-minimal`). When set, the
    /// coach prompt injects the matching `knowledge/design-systems/<name>.md`
    /// content so the worker binds tokens deterministically.
    pub design_system: String,
    /// Active seed template name (e.g. `dashboard`). When set, the coach
    /// prompt references `knowledge/seed-templates/<name>.md` for the
    /// page structure and quality gates.
    pub seed_template: String,
}

impl RunOptions {
    /// Resolve the effective slug — derives from workspace dir name
    /// when empty.
    #[must_use]
    pub fn effective_slug(&self) -> String {
        if !self.slug.is_empty() {
            return self.slug.clone();
        }
        self.project_root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project")
            .to_string()
    }
}

/// Outcome of a single block of execution.
#[derive(Debug, Clone)]
pub struct RunReport {
    /// Final phase after this block.
    pub final_phase: Phase,
    /// Gate the pipeline paused at, if any. `None` means delivery
    /// completed.
    pub paused_at: Option<Gate>,
    /// Phases that executed during this block, with their artifact lists.
    pub completed: Vec<PhaseOutput>,
}

/// The agent runner. Owns the runtime; phase methods live in [`crate::phases`].
pub struct AgentRunner<R: Runtime> {
    runtime: R,
    options: RunOptions,
    events: Arc<dyn EventSink>,
}

impl<R: Runtime> AgentRunner<R> {
    /// Build a new runner. Events are dropped until [`with_event_sink`]
    /// attaches a real sink.
    ///
    /// [`with_event_sink`]: AgentRunner::with_event_sink
    pub fn new(runtime: R, options: RunOptions) -> Self {
        Self {
            runtime,
            options,
            events: null_sink(),
        }
    }

    /// Attach an event sink so a UI (TUI) can observe pipeline progress.
    #[must_use]
    pub fn with_event_sink(mut self, sink: Arc<dyn EventSink>) -> Self {
        self.events = sink;
        self
    }

    /// Runtime kind (for human-facing announcements).
    pub fn runtime_kind(&self) -> super_dev_runtime::RuntimeKind {
        self.runtime.kind()
    }

    /// Emit `event` to the attached sink (no-op for the null sink).
    fn emit(&self, event: EngineEvent) {
        self.events.emit(event);
    }

    /// Record a phase execution with timing.
    fn record_phase(
        &self,
        phase: Phase,
        output: std::io::Result<PhaseOutput>,
    ) -> std::io::Result<PhaseOutput> {
        let output = output?;
        for artifact in &output.artifacts {
            self.emit(EngineEvent::ArtifactWritten {
                phase,
                path: artifact.clone(),
            });
        }
        self.emit(EngineEvent::PhaseCompleted { phase });
        if let Some(gate) = output.gate {
            self.emit(EngineEvent::GateOpened { gate });
        }
        Ok(output)
    }

    /// Record phase timing to `.super-dev/phase-timing.jsonl`.
    fn record_phase_timing(&self, phase: Phase, started: std::time::Instant) {
        let elapsed_ms = started.elapsed().as_millis();
        let dir = self.options.project_root.join(".super-dev");
        let _ = std::fs::create_dir_all(&dir);
        let entry = serde_json::json!({
            "timestamp": Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "phase": phase.id(),
            "elapsed_ms": elapsed_ms,
            "slug": self.options.effective_slug(),
        });
        let path = dir.join("phase-timing.jsonl");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            use std::io::Write;
            let _ = writeln!(f, "{entry}");
        }
        self.emit(EngineEvent::Note(format!(
            "⏱ {} completed in {:.1}s",
            phase.id(),
            elapsed_ms as f64 / 1000.0
        )));
    }

    /// Write a run summary entry to `.super-dev/runs.jsonl` for historical tracking.
    fn record_run_history(&self, final_phase: Phase, passed: bool, artifact_count: usize) {
        let dir = self.options.project_root.join(".super-dev");
        let _ = std::fs::create_dir_all(&dir);
        let entry = serde_json::json!({
            "timestamp": Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "slug": self.options.effective_slug(),
            "requirement": self.options.requirement,
            "backend": self.options.backend,
            "design_system": self.options.design_system,
            "final_phase": final_phase.id(),
            "quality_passed": passed,
            "artifact_count": artifact_count,
        });
        let path = dir.join("runs.jsonl");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            use std::io::Write;
            let _ = writeln!(f, "{entry}");
        }
    }

    /// Run the workspace's build / install command after a
    /// code-producing phase. Emits `VerifyStarted` + one of
    /// `VerifySkipped` / `VerifyPassed` / `VerifyFailed`, and appends a
    /// row to `.super-dev/audit/verify.jsonl`. Always best-effort —
    /// `Err` paths from the subprocess become structured `VerifyFailed`
    /// events, not Rust errors.
    async fn maybe_verify(&self, phase: Phase) -> VerifyOutcome {
        // Only the code-producing phases need verify; docs / spec / etc.
        // don't have build output to test.
        if !matches!(phase, Phase::Frontend | Phase::Backend | Phase::Quality) {
            return VerifyOutcome {
                passed: true,
                skipped: true,
                failure_detail: String::new(),
            };
        }
        let workspace = &self.options.project_root;
        let kind = crate::verify::detect_project(workspace);
        let command = kind
            .verify_command(workspace)
            .map_or_else(String::new, |(p, args)| format!("{p} {}", args.join(" ")));
        self.emit(EngineEvent::VerifyStarted {
            phase,
            command: command.clone(),
        });
        let outcomes = crate::verify::run_verify(workspace).await;
        if outcomes.is_empty() {
            self.emit(EngineEvent::VerifySkipped {
                phase,
                reason: "no recognised project manifest".to_string(),
            });
            return VerifyOutcome {
                passed: true,
                skipped: true,
                failure_detail: String::new(),
            };
        }

        // Record each step's outcome to the audit log.
        let mut total_ms: u64 = 0;
        let mut any_failed = false;
        for o in &outcomes {
            total_ms = total_ms.saturating_add(o.duration_ms);
            if !o.passed && !o.skipped {
                any_failed = true;
            }
            let _ = crate::verify::record_verify_outcome(workspace, phase.id(), o);
        }

        // Emit an aggregate event. If any non-skipped step failed, the
        // whole verify is a failure (the quality gate consumes this).
        if any_failed {
            let failed_step = outcomes
                .iter()
                .find(|o| !o.passed && !o.skipped)
                .map(|o| format!("{}: {}", o.step, summarize_stderr(&o.stderr)))
                .unwrap_or_default();
            self.emit(EngineEvent::VerifyFailed {
                phase,
                exit_code: outcomes
                    .iter()
                    .find(|o| !o.passed && !o.skipped)
                    .map(|o| o.exit_code)
                    .unwrap_or(-1),
                stderr: failed_step.clone(),
            });
            VerifyOutcome {
                passed: false,
                skipped: false,
                failure_detail: failed_step,
            }
        } else {
            self.emit(EngineEvent::VerifyPassed {
                phase,
                duration_ms: total_ms,
            });
            VerifyOutcome {
                passed: true,
                skipped: false,
                failure_detail: String::new(),
            }
        }
    }

    /// Verify, and if it FAILS, hand the build error back to the worker for
    /// one fix attempt, then re-verify. Mirrors the docs review→fix loop but
    /// for code: a frontend/backend build that doesn't compile gets one
    /// automatic repair pass before the pipeline gives up. Skipped (no
    /// manifest) and passing verifies return immediately.
    async fn maybe_verify_and_fix(&self, phase: Phase) {
        let first = self.maybe_verify(phase).await;
        if first.passed || first.skipped {
            return;
        }
        if self.options.backend.is_empty() {
            return; // no runtime → nothing can fix it
        }
        self.emit(EngineEvent::Note(format!(
            "🔧 {} 构建失败,把错误喂回 worker 修复一次…\n  {}",
            phase.id(),
            first.failure_detail
        )));
        let fix_prompt = Prompt {
            system: format!(
                "The {} code you just wrote failed to build/test. The error                  output is below. Fix the code so the build passes — edit the                  relevant files, do NOT rewrite from scratch. Output a short                  summary of what you changed.",
                phase.id()
            ),
            user: format!(
                "## Build/test error\n\n{}\n\n## Original requirement\n\n{}\n\nFix the failing code now.",
                first.failure_detail, self.options.requirement
            ),
        };
        let _ = self.try_generate(phase, fix_prompt).await;
        // Re-verify after the fix attempt.
        self.maybe_verify(phase).await;
    }

    /// Initialise the workspace for a new run.
    pub fn start(&self) -> std::io::Result<WorkflowState> {
        let state = WorkflowState {
            phase: Phase::Research.id().to_string(),
            active_gate: String::new(),
            slug: self.options.effective_slug(),
            requirement: self.options.requirement.clone(),
            last_transition_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            note: format!(
                "Started run with worker {}: {}",
                if self.options.backend.is_empty() {
                    "offline-templates"
                } else {
                    self.options.backend.as_str()
                },
                self.options.requirement
            ),
            backend: self.options.backend.clone(),
            spec_version: SPEC_VERSION.to_string(),
        };
        write_workflow_state(&self.options.project_root, &state)?;
        // Drop a coach prompt so the host knows what to do on first turn.
        let _ = write_coach_prompt(&self.options, Phase::Research);
        // SD-META-001: ensure the workspace declares its spec conformance.
        // Best-effort — a user-customised super-dev.yaml is left untouched.
        let _ = crate::manifest::SpecManifest::new(self.options.effective_slug())
            .write_to(&self.options.project_root, false);
        Ok(state)
    }

    /// research → docs → pause at `docs_confirm`.
    ///
    /// When `use_runtime` is true, the runner asks the configured
    /// runtime to draft each artifact (research → PRD → architecture →
    /// UIUX) and writes the LLM output verbatim. Any provider error or
    /// empty response falls back to the deterministic template, so the
    /// pipeline never breaks because of an LLM blip.
    pub async fn run_initial_block(&self, use_runtime: bool) -> std::io::Result<RunReport> {
        let mut completed = Vec::new();
        let project_cfg = crate::config::load_project_config(&self.options.project_root);
        let max_reviews = project_cfg.pipeline.max_review_rounds;
        let skip = &project_cfg.pipeline.skip_phases;
        self.emit(EngineEvent::PipelineStarted {
            slug: self.options.effective_slug(),
            requirement: self.options.requirement.clone(),
        });

        // Pre-embed the requirement once so every phase's expert-knowledge
        // section can use true BM25+vector RRF fusion. No-op (returns None)
        // when the vector layer is off or no API key is set — fail-open to BM25.
        let qvec = self.preembed_requirement().await;

        // 1. research
        let research_text = if skip.iter().any(|s| s == "research") {
            self.emit(EngineEvent::Note(
                "⏭ Skipping research (configured in .superdevrc)".to_string(),
            ));
            None
        } else {
            let phase_start = std::time::Instant::now();
            self.emit(EngineEvent::PhaseStarted {
                phase: Phase::Research,
            });
            let (top_files, total) = crate::phases::knowledge_top_files(&self.options);
            if !top_files.is_empty() {
                let preview = top_files
                    .iter()
                    .take(3)
                    .map(|p| format!("`{p}`"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let more = if top_files.len() > 3 {
                    format!(" (+ {} more)", top_files.len() - 3)
                } else {
                    String::new()
                };
                self.emit(EngineEvent::Note(format!(
                    "📚 knowledge: 选了 {} 个文档中的 {} 篇喂给 worker —— {preview}{more}",
                    total,
                    top_files.len(),
                )));
            }
            let text = if use_runtime {
                let research_digest = crate::phases::phase_knowledge_digest_with_vector(
                    &self.options,
                    Phase::Research,
                    qvec.as_deref(),
                );
                let rp = self.with_expert_knowledge(
                    research_prompt(
                        &self.options.effective_slug(),
                        &self.options.requirement,
                        &research_digest,
                    ),
                    &["product-manager"],
                );
                self.generate_with_review(Phase::Research, rp, Self::review_research, max_reviews)
                    .await
            } else {
                None
            };
            completed.push(self.record_phase(
                Phase::Research,
                run_research(&self.options, text.as_deref()),
            )?);
            self.record_phase_timing(Phase::Research, phase_start);
            text
        };
        self.transition(Phase::Docs, "")?;

        // 2. docs
        let phase_start = std::time::Instant::now();
        self.emit(EngineEvent::PhaseStarted { phase: Phase::Docs });
        let docs_content = if use_runtime {
            self.generate_docs_content(research_text.as_deref()).await
        } else {
            DocsContent::default()
        };

        let docs = self.record_phase(Phase::Docs, run_docs(&self.options, &docs_content))?;
        self.record_phase_timing(Phase::Docs, phase_start);
        let gate = docs.gate;
        completed.push(docs);
        self.transition(Phase::DocsConfirm, gate.map_or("", Gate::id_str))?;

        self.emit(EngineEvent::BlockCompleted {
            final_phase: Phase::DocsConfirm,
            paused_at: gate,
        });
        Ok(RunReport {
            final_phase: Phase::DocsConfirm,
            paused_at: gate,
            completed,
        })
    }

    /// Pre-embed the requirement string once so every phase's
    /// expert-knowledge section can use true BM25+vector RRF fusion.
    ///
    /// Also builds the cached vector store if the hybrid engine is on —
    /// this is where the previously-stubbed batch embedding actually
    /// happens (one network round-trip per corpus chunk, cached on disk).
    /// Returns `None` (and leaves retrieval on BM25) when the vector layer
    /// is off, no API key is set, or embedding fails. Fail-open, never
    /// blocks the pipeline.
    async fn preembed_requirement(&self) -> Option<Vec<f32>> {
        let project_cfg = crate::config::load_project_config(&self.options.project_root);
        if project_cfg.knowledge.engine != "hybrid" || !project_cfg.knowledge.enabled {
            return None;
        }
        if !super_dev_knowledge::vector::is_enabled() {
            return None;
        }
        // Build (or incrementally update) the vector store for the corpus.
        // This is the no-longer-stubbed batch embedding: chunks are embedded
        // in batches of 100 and cached at .super-dev/kb-index/vectors.bin.
        let knowledge_dir = self.options.project_root.join("knowledge");
        if knowledge_dir.is_dir() {
            let index = super_dev_knowledge::load_or_build_index(
                &self.options.project_root,
                &knowledge_dir,
            );
            let _ = super_dev_knowledge::build_vector_store_if_enabled(
                &self.options.project_root,
                &index,
            )
            .await;
        }
        // Embed the requirement query itself.
        super_dev_knowledge::vector::embed_query(&self.options.requirement).await
    }

    /// Run one prompt against the configured runtime. Returns `None` on
    /// any failure (empty body, provider error) so the caller can fall
    /// back to the deterministic template.
    ///
    /// `phase` tags `HostOutput` events the UI uses to render the host's
    /// response as it streams past.
    /// Generate content with review→fix loop.
    ///
    /// 1. Generate initial draft via worker
    /// 2. Run review checks (closure returns list of defects)
    /// 3. If defects found and attempts < max, send fix prompt
    /// 4. Repeat until clean or max attempts reached
    ///
    /// `reviewer` takes the generated text and returns a list of
    /// defect descriptions. Empty list = pass.
    /// Generate with review loop. Key principle: NEVER lose the first
    /// successful generation. If fix attempts timeout or fail, we keep
    /// what we have — an imperfect doc is better than a template.
    async fn generate_with_review(
        &self,
        phase: Phase,
        prompt: Prompt,
        reviewer: impl Fn(&str) -> Vec<String>,
        max_attempts: usize,
    ) -> Option<String> {
        let mut text = self.try_generate(phase, prompt).await?;

        // `max_attempts` is the number of review→fix rounds to ALLOW, so the
        // loop runs `max_attempts` times (previously `1..max_attempts` ran one
        // fewer, silently under-delivering review rounds).
        for attempt in 1..=max_attempts {
            let defects = reviewer(&text);
            if defects.is_empty() {
                self.emit(EngineEvent::Note(format!(
                    "✓ {} review passed.",
                    phase.id()
                )));
                break;
            }
            // Only auto-fix STRUCTURAL defects (missing sections) in a single
            // pass. A doc with many defects is usually a generation that
            // diverged from the requirement — a fix pass is unlikely to land
            // cleanly, so keep what we have rather than risk a worse rewrite.
            // Threshold raised to 10 (was 6): real host output (claude code /
            // codex) commonly has 7-9 missing sections on the first pass
            // because the host doesn't know Super Dev's exact section list.
            // Capping at 10 lets the review→fix loop help real runs instead
            // of giving up immediately, while still bailing on truly divergent
            // output (> 10 missing sections = the host went off-script).
            if defects.len() > 10 {
                self.emit(EngineEvent::Note(format!(
                    "⚠ {} review: {} issues — too many to auto-fix, keeping current version.",
                    phase.id(),
                    defects.len()
                )));
                break;
            }
            let defect_list = defects.join("\n- ");
            self.emit(EngineEvent::Note(format!(
                "⚠ Review round {attempt}: {} defect(s). Fixing...\n- {defect_list}",
                defects.len()
            )));
            let fix_prompt = Prompt {
                system: format!(
                    "The document below has quality defects. Fix ONLY the listed \
                     issues. Output the COMPLETE corrected document.\n\nDefects:\n- {defect_list}"
                ),
                user: text.clone(),
            };
            match self.try_generate(phase, fix_prompt).await {
                Some(fixed) if !fixed.trim().is_empty() => text = fixed,
                _ => {
                    self.emit(EngineEvent::Note(
                        "Fix attempt failed — keeping previous version.".to_string(),
                    ));
                    break;
                }
            }
        }

        Some(text)
    }

    /// Review a research document for structural completeness.
    fn review_research(text: &str) -> Vec<String> {
        let lower = text.to_ascii_lowercase();
        let mut defects = Vec::new();
        // Each required section must be present AND have non-empty body
        // (section_has_body), consistent with review_prd/architecture/uiux.
        // `## Discovery` may be spelled "target audience" instead — keep the
        // alternate-string allowance for presence, but body-check when the
        // heading form is found.
        let has_discovery_alt = lower.contains("target audience");
        match lower.find("## discovery") {
            None if !has_discovery_alt => {
                defects.push("Missing ## Discovery section (audience/tone/direction)".into());
            }
            Some(pos) if !section_has_body(&lower, pos, "## discovery") => {
                defects.push("Section '## discovery' is present but empty".into());
            }
            None => {}    // alternate "target audience" present — acceptable
            Some(_) => {} // present with body — OK
        }
        for (heading, label) in [
            ("## similar products", "## Similar products"),
            ("## domain risks", "## Domain risks"),
        ] {
            match lower.find(heading) {
                None => defects.push(format!("Missing {label} section")),
                Some(pos) => {
                    if !section_has_body(&lower, pos, heading) {
                        defects.push(format!("Section '{heading}' is present but empty"));
                    }
                }
            }
        }
        // Depth signal: the Similar products section should name ≥1 actual
        // comparable (a list item or a capitalized product name), not just be
        // a heading. Catches a research doc that lists the section but didn't
        // actually survey competitors.
        if let Some(pos) = lower.find("## similar products") {
            let after = &lower[pos..];
            let next_h2 = after.find("\n## ").unwrap_or(after.len());
            let body = &after[..next_h2];
            let names_items = body
                .lines()
                .filter(|l| {
                    let lt = l.trim();
                    lt.starts_with("- ") || lt.starts_with("* ") || lt.starts_with("| ")
                })
                .count();
            if names_items == 0 && section_has_body(&lower, pos, "## similar products") {
                defects.push(
                    "## similar products section has no list items (name the comparables)".into(),
                );
            }
        }

        // Design recommendation has two accepted heading forms.
        let design_heading = if lower.contains("## design system recommendation") {
            "## design system recommendation"
        } else if lower.contains("## design recommendation") {
            "## design recommendation"
        } else {
            defects.push("Missing ## Design system recommendation section".into());
            ""
        };
        if !design_heading.is_empty() {
            if let Some(pos) = lower.find(design_heading) {
                if !section_has_body(&lower, pos, design_heading) {
                    defects.push(format!("Section '{design_heading}' is present but empty"));
                }
            }
        }
        defects
    }

    /// Review PRD — commercial grade checks.
    fn review_prd(text: &str) -> Vec<String> {
        let lower = text.to_ascii_lowercase();
        let mut defects = Vec::new();
        // Required sections with order checking
        let required = ["## goal", "## scope", "## acceptance criteria"];
        let mut last_pos = 0;
        for section in &required {
            if let Some(pos) = lower.find(section) {
                if pos < last_pos {
                    defects.push(format!(
                        "Section '{section}' is out of order (should come after previous sections)"
                    ));
                }
                if !section_has_body(&lower, pos, section) {
                    defects.push(format!("Section '{section}' is present but empty"));
                }
                last_pos = pos;
            } else {
                defects.push(format!("Missing {section}"));
            }
        }
        // Content depth checks
        if !lower.contains("target user")
            && !lower.contains("persona")
            && !lower.contains("## user")
        {
            defects.push("Missing target users / personas".into());
        }
        // Functional requirements: present AND substantive (≥2 list/table
        // rows in the section, not just the heading).
        let func_heading = if lower.contains("## functional") {
            lower.find("## functional")
        } else if lower.contains("## feature") {
            lower.find("## feature")
        } else {
            None
        };
        match func_heading {
            None => defects.push("Missing functional requirements".into()),
            Some(pos) => {
                // Count list items / table rows in the functional section.
                let after = &lower[pos..];
                let next_h2 = after.find("\n## ").unwrap_or(after.len());
                let body = &after[..next_h2];
                let item_count = body
                    .lines()
                    .filter(|l| {
                        let lt = l.trim();
                        lt.starts_with("- ") || lt.starts_with("* ") || lt.starts_with("| ")
                    })
                    .count();
                if item_count < 2 {
                    defects.push(format!(
                        "Functional requirements section present but only has {item_count} item(s) (need ≥2 features)"
                    ));
                }
            }
        }
        if !lower.contains("non-functional") && !lower.contains("performance") {
            defects.push("Missing non-functional requirements".into());
        }
        let ac_count = text.matches("- [ ]").count();
        if ac_count < 2 {
            defects.push(format!("Only {ac_count} acceptance criteria"));
        }
        // Cross-section depth signal: if the functional-requirements section
        // lists N feature items, the acceptance-criteria count should be at
        // least ~N/2 (each major feature usually has ≥1 AC). A doc with many
        // features but 1-2 ACs is under-specified — flag the gap.
        if ac_count >= 2 && (lower.contains("## functional") || lower.contains("## feature")) {
            let func_start = lower
                .find("## functional")
                .or_else(|| lower.find("## feature"))
                .unwrap_or(0);
            let after = &lower[func_start..];
            let next_h2 = after.find("\n## ").unwrap_or(after.len());
            let func_body = &after[..next_h2];
            let feature_items = func_body
                .lines()
                .filter(|l| {
                    let lt = l.trim();
                    lt.starts_with("- ") || lt.starts_with("* ") || lt.starts_with("| ")
                })
                .count();
            if feature_items >= 4 && ac_count < feature_items / 2 {
                defects.push(format!(
                    "Acceptance criteria ({ac_count}) look thin relative to {feature_items}                      functional items — each major feature should have ≥1 AC"
                ));
            }
        }
        if !lower.contains("metric") && !lower.contains("kpi") {
            defects.push("Missing success metrics".into());
        }
        defects
    }

    /// Review architecture — commercial grade checks.
    fn review_architecture(text: &str) -> Vec<String> {
        let lower = text.to_ascii_lowercase();
        let mut defects = Vec::new();
        // ## API surface must be present AND have body content (not just the
        // heading). Matches review_prd's section_has_body contract so an
        // architecture doc with a bare `## API` heading is flagged.
        match lower.find("## api") {
            None => defects.push("Missing API surface section".into()),
            Some(pos) => {
                if !section_has_body(&lower, pos, "## api") {
                    defects.push("Section '## api' is present but empty".into());
                }
            }
        }
        let api_rows = text
            .lines()
            .filter(|l| {
                let t = l.trim();
                t.starts_with('|') && t.contains('/') && !t.contains("---")
            })
            .count();
        if api_rows < 2 {
            defects.push(format!(
                "API table has {api_rows} rows (need at least a few endpoints)"
            ));
        }
        // Data model: present AND substantive (a field-type table, not just
        // the heading). Require ≥2 table rows mentioning a type near the
        // data-model section, matching the API-table depth signal.
        let has_dm_heading = lower.contains("data model") || lower.contains("schema");
        if has_dm_heading {
            // Count table rows that look like entity field definitions
            // (contain a `|` and a common type keyword).
            let dm_rows = text
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    t.starts_with('|')
                        && (t.contains("text")
                            || t.contains("int")
                            || t.contains("bool")
                            || t.contains("date")
                            || t.contains("uuid")
                            || t.contains("string"))
                })
                .count();
            if dm_rows < 2 {
                defects.push(format!(
                    "Data model section present but has only {dm_rows} typed field rows (need a field-type table)"
                ));
            }
        } else {
            defects.push("Missing data model with field types".into());
        }
        if !lower.contains("auth") {
            defects.push("Missing authentication/authorization section".into());
        }
        if !lower.contains("tech") && !lower.contains("stack") {
            defects.push("Missing tech-stack rationale".into());
        }
        if !lower.contains("error") {
            defects.push("Missing API error convention".into());
        }
        if !lower.contains("structure") && !lower.contains("directory") && !lower.contains("layout")
        {
            defects.push("Missing project structure".into());
        }
        defects
    }

    /// Review a UIUX doc for structural completeness.
    fn review_uiux(text: &str) -> Vec<String> {
        let lower = text.to_ascii_lowercase();
        let mut defects = Vec::new();
        let token_count = text.matches("--").count();
        if token_count < 5 {
            defects.push(format!(
                "Only {token_count} CSS tokens (need at least basic semantic tokens)"
            ));
        }
        // Consistency with section_has_body: require SOME prose beyond the
        // tokens block. UIUX docs aren't always H2-structured, so rather than
        // mandating an H2, we require that non-token, non-heading lines exist
        // (catches a doc that's only `# UIUX` + a few `--*` tokens).
        let has_prose = lower.lines().any(|l| {
            let l = l.trim();
            !l.is_empty()
                && !l.starts_with('#')
                && !l.starts_with("--")   // CSS token line
                && !l.starts_with('|')    // table row
                && !l.contains(": #") // `--x: #hex` token assignment
        });
        if !has_prose {
            defects.push("No prose content beyond tokens (doc looks empty)".into());
        }
        if !lower.contains("prefers-color-scheme") && !lower.contains("dark mode") {
            defects.push("Missing dark mode (@media prefers-color-scheme) block".into());
        }
        if !lower.contains("font-family") && !lower.contains("--font") && !lower.contains("inter") {
            defects.push("Missing typography system (font stack + size scale)".into());
        }
        if !lower.contains("icon") {
            defects.push("Missing icon library declaration".into());
        }
        if !lower.contains("hover") && !lower.contains("states") {
            defects.push("Missing component states (hover/focus/disabled)".into());
        }
        defects
    }

    /// Try to generate content via the worker. Retries on transient errors
    /// (timeout / 429 / 5xx / connection blips) with exponential backoff
    /// (2s, 4s, 8s, …) so a rate-limited or briefly-unreachable host recovers
    /// instead of failing the whole phase. Permanent errors (401, config) are
    /// NOT retried. The base delay is overridable via `SUPER_DEV_RETRY_BASE_MS`
    /// (default 2000) so tests can shrink it.
    async fn try_generate(&self, phase: Phase, prompt: Prompt) -> Option<String> {
        let max_retries = 3;
        let base_ms = retry_base_ms();
        for attempt in 0..max_retries {
            if attempt == 0 {
                self.emit(EngineEvent::Note(format!(
                    "🔄 调用 worker({} 阶段)— 可能需要 30s-5min,请稍候...",
                    phase.id()
                )));
            }
            let req = prompt
                .clone()
                .into_request(&self.options.model, max_tokens_for_phase(phase));
            match self.runtime.complete(req).await {
                Ok(resp) if !resp.text.trim().is_empty() => {
                    for line in resp.text.lines().filter(|l| !l.trim().is_empty()).take(40) {
                        self.emit(EngineEvent::HostOutput {
                            phase,
                            line: line.to_string(),
                        });
                    }
                    return Some(resp.text);
                }
                Ok(_) => {
                    tracing::warn!(runtime = %self.runtime.kind().id(), "empty body");
                    self.emit(EngineEvent::Note(format!(
                        "⚠ Worker 返回空内容({} 阶段)— offline 模板替代。",
                        phase.id()
                    )));
                    return None;
                }
                Err(err) => {
                    // Retry on timeout AND on transient host errors (the
                    // provider returning 429/5xx, or a connection blip). We
                    // can't fully distinguish transient from permanent for
                    // HostProcess, so match on the error string for the
                    // well-known transient signals — matches the retry
                    // policy in vector.rs::http_embed for consistency.
                    let is_timeout = matches!(err, super_dev_runtime::RuntimeError::Timeout(_, _));
                    let err_str = err.to_string().to_ascii_lowercase();
                    let is_transient = is_timeout
                        || err_str.contains("429")
                        || err_str.contains("too many requests")
                        || err_str.contains("502")
                        || err_str.contains("503")
                        || err_str.contains("504")
                        || err_str.contains("service unavailable")
                        || err_str.contains("bad gateway")
                        || err_str.contains("connection reset")
                        || err_str.contains("connection refused")
                        || err_str.contains("timed out");
                    if is_transient && attempt + 1 < max_retries {
                        // Exponential backoff: base * 2^attempt (2s → 4s → 8s).
                        // Sleeping here lets a rate-limited provider recover
                        // before the next attempt — without it the retry hits
                        // the same 429 immediately and wastes the attempt.
                        let delay_ms = base_ms.saturating_mul(2u64.saturating_pow(attempt as u32));
                        self.emit(EngineEvent::Note(format!(
                            "⚠ Worker 调用瞬时失败({} 阶段: {err}), {delay_ms}ms 后重试 {}/{}...",
                            phase.id(),
                            attempt + 2,
                            max_retries
                        )));
                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                        continue;
                    }
                    tracing::warn!(
                        runtime = %self.runtime.kind().id(),
                        error = %err,
                        "runtime call failed"
                    );
                    self.emit(EngineEvent::Note(format!(
                        "⚠ Worker 调用失败({} 阶段): {err}",
                        phase.id()
                    )));
                    return None;
                }
            }
        }
        None
    }

    /// Generate PRD, architecture, UIUX content sequentially so each
    /// expert sees the prior artifact as an excerpt.
    /// Read expert methodology from knowledge/experts/<role>/ and return
    /// a condensed string suitable for injecting into a prompt's system field.
    fn load_expert_knowledge(&self, expert_dirs: &[&str]) -> String {
        let base = self.options.project_root.join("knowledge/experts");
        let mut out = String::new();
        let mut read_dir = |dir_path: &std::path::Path, label: &str| {
            if !dir_path.is_dir() {
                return;
            }
            if let Ok(rd) = std::fs::read_dir(dir_path) {
                for entry in rd.flatten() {
                    let p = entry.path();
                    if p.extension().and_then(|s| s.to_str()) != Some("md") {
                        continue;
                    }
                    if let Ok(content) = std::fs::read_to_string(&p) {
                        let trimmed: String = content.chars().take(1500).collect();
                        out.push_str(&format!(
                            "\n---\n{label} ({}):\n{trimmed}\n",
                            p.file_name().unwrap_or_default().to_string_lossy(),
                        ));
                    }
                }
            }
        };
        for dir in expert_dirs {
            read_dir(&base.join(dir), "Expert reference");
        }
        let project_cfg = crate::config::load_project_config(&self.options.project_root);
        if let Some(custom) = &project_cfg.experts.custom_knowledge {
            read_dir(&self.options.project_root.join(custom), "Custom knowledge");
        }
        out
    }

    /// Enhance a prompt by appending expert methodology to the system field.
    fn with_expert_knowledge(&self, mut prompt: Prompt, expert_dirs: &[&str]) -> Prompt {
        let knowledge = self.load_expert_knowledge(expert_dirs);
        if !knowledge.is_empty() {
            prompt.system.push_str(&knowledge);
        }
        prompt
    }

    async fn generate_docs_content(&self, research: Option<&str>) -> DocsContent {
        let slug = self.options.effective_slug();
        let req = &self.options.requirement;
        let research_excerpt = excerpt(research.unwrap_or(""), 1500);
        let project_cfg = crate::config::load_project_config(&self.options.project_root);
        let max_reviews = project_cfg.pipeline.max_review_rounds;

        self.emit(EngineEvent::Note(
            "📋 Docs phase: generating PRD → Architecture → UI/UX (3 documents).              This may take 5-15 minutes with a worker backend."
                .to_string(),
        ));

        // PRD: inject PM methodology → generate → review → fix
        self.emit(EngineEvent::Note("📋 Generating PRD...".to_string()));
        let prd_p = self.with_expert_knowledge(
            prd_prompt(&slug, req, &research_excerpt),
            &["product-manager"],
        );
        let prd = self
            .generate_with_review(Phase::Docs, prd_p, Self::review_prd, max_reviews)
            .await;
        let prd_excerpt = excerpt(prd.as_deref().unwrap_or(""), 1500);

        // Architecture: inject architect methodology → generate → review → fix
        self.emit(EngineEvent::Note(
            "🏗 Generating Architecture...".to_string(),
        ));
        let arch_p = self.with_expert_knowledge(
            architecture_prompt(&slug, req, &prd_excerpt),
            &["architect"],
        );
        let architecture = self
            .generate_with_review(Phase::Docs, arch_p, Self::review_architecture, max_reviews)
            .await;

        // UIUX: inject designer methodology → generate → review → fix
        self.emit(EngineEvent::Note(
            "🎨 Generating UI/UX design system...".to_string(),
        ));
        let uiux_p =
            self.with_expert_knowledge(uiux_prompt(&slug, req, &prd_excerpt), &["uiux-designer"]);
        let uiux = self
            .generate_with_review(Phase::Docs, uiux_p, Self::review_uiux, max_reviews)
            .await;

        DocsContent {
            prd,
            architecture,
            uiux,
        }
    }

    /// spec → frontend → pause at `preview_confirm`.
    ///
    /// When a worker backend is configured, the spec and frontend phases
    /// are also driven through the worker (not just templates). The worker
    /// creates real project scaffold, components, and pages based on the
    /// approved PRD + Architecture + UIUX documents.
    pub async fn continue_after_docs_confirm(&self) -> std::io::Result<RunReport> {
        let use_runtime = !self.options.backend.is_empty();
        self.transition(Phase::Spec, "")?;
        let mut completed = Vec::new();

        // Spec phase
        let phase_start = std::time::Instant::now();
        self.emit(EngineEvent::PhaseStarted { phase: Phase::Spec });
        if use_runtime {
            self.emit(EngineEvent::Note(
                "📋 Worker generating execution plan + task breakdown...".to_string(),
            ));
            let slug = self.options.effective_slug();
            // Read approved docs for context
            let prd = std::fs::read_to_string(
                self.options
                    .project_root
                    .join(format!("output/{slug}-prd.md")),
            )
            .unwrap_or_default();
            let arch = std::fs::read_to_string(
                self.options
                    .project_root
                    .join(format!("output/{slug}-architecture.md")),
            )
            .unwrap_or_default();
            let context = format!(
                "PRD excerpt:\n{}\n\nArchitecture excerpt:\n{}",
                excerpt(&prd, 2000),
                excerpt(&arch, 2000)
            );
            let spec_text = self
                .try_generate(
                    Phase::Spec,
                    Prompt {
                        system: format!(
                            "Role: senior engineering manager.\n\
                             Write an execution plan with sprint breakdown, coding standards, \
                             and definition of done. Based on these approved documents:\n\n{context}"
                        ),
                        user: format!("Write the execution plan for: {}", self.options.requirement),
                    },
                )
                .await;
            if let Some(text) = spec_text {
                let plan_path = self
                    .options
                    .project_root
                    .join(format!("output/{slug}-execution-plan.md"));
                if let Some(parent) = plan_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&plan_path, &text);
            }
        }
        completed.push(self.record_phase(Phase::Spec, run_spec(&self.options))?);
        self.record_phase_timing(Phase::Spec, phase_start);
        self.transition(Phase::Frontend, "")?;

        // Frontend phase
        let phase_start = std::time::Instant::now();
        self.emit(EngineEvent::PhaseStarted {
            phase: Phase::Frontend,
        });
        if use_runtime {
            self.emit(EngineEvent::Note(
                "🖥 Worker implementing frontend (components, pages, API client)...".to_string(),
            ));
            let slug = self.options.effective_slug();
            let uiux = std::fs::read_to_string(
                self.options
                    .project_root
                    .join(format!("output/{slug}-uiux.md")),
            )
            .unwrap_or_default();
            let arch = std::fs::read_to_string(
                self.options
                    .project_root
                    .join(format!("output/{slug}-architecture.md")),
            )
            .unwrap_or_default();
            let prd = std::fs::read_to_string(
                self.options
                    .project_root
                    .join(format!("output/{slug}-prd.md")),
            )
            .unwrap_or_default();
            self.emit(EngineEvent::SubTaskStarted {
                phase: Phase::Frontend,
                task_id: "frontend.implement".into(),
                label: "worker generating components/styling/state".into(),
            });
            let fe_p = self.with_expert_knowledge(
                frontend_prompt(
                    &slug,
                    &self.options.requirement,
                    &excerpt(&uiux, 3000),
                    &excerpt(&arch, 2000),
                    &excerpt(&prd, 1500),
                ),
                &["frontend-lead", "uiux-designer"],
            );
            let fe_ok = self.try_generate(Phase::Frontend, fe_p).await.is_some();
            self.emit(EngineEvent::SubTaskCompleted {
                phase: Phase::Frontend,
                task_id: "frontend.implement".into(),
                ok: fe_ok,
            });
        }
        let fe = self.record_phase(Phase::Frontend, run_frontend(&self.options))?;
        self.record_phase_timing(Phase::Frontend, phase_start);
        let gate = fe.gate;
        completed.push(fe);
        self.maybe_verify_and_fix(Phase::Frontend).await;
        self.transition(Phase::PreviewConfirm, gate.map_or("", Gate::id_str))?;

        self.emit(EngineEvent::BlockCompleted {
            final_phase: Phase::PreviewConfirm,
            paused_at: gate,
        });
        Ok(RunReport {
            final_phase: Phase::PreviewConfirm,
            paused_at: gate,
            completed,
        })
    }

    /// backend → quality → delivery → done. Call after the user has
    /// approved `preview_confirm`.
    pub async fn continue_after_preview_confirm(&self) -> std::io::Result<RunReport> {
        let use_runtime = !self.options.backend.is_empty();
        self.transition(Phase::Backend, "")?;
        let mut completed = Vec::new();

        let phase_start = std::time::Instant::now();
        self.emit(EngineEvent::PhaseStarted {
            phase: Phase::Backend,
        });
        if use_runtime {
            self.emit(EngineEvent::Note(
                "⚙ Worker implementing backend (routes, database, auth, tests)...".to_string(),
            ));
            let slug = self.options.effective_slug();
            let arch = std::fs::read_to_string(
                self.options
                    .project_root
                    .join(format!("output/{slug}-architecture.md")),
            )
            .unwrap_or_default();
            let prd = std::fs::read_to_string(
                self.options
                    .project_root
                    .join(format!("output/{slug}-prd.md")),
            )
            .unwrap_or_default();
            self.emit(EngineEvent::SubTaskStarted {
                phase: Phase::Backend,
                task_id: "backend.implement".into(),
                label: "worker generating routes/database/auth/tests".into(),
            });
            let be_p = self.with_expert_knowledge(
                backend_prompt(
                    &slug,
                    &self.options.requirement,
                    &excerpt(&arch, 3000),
                    &excerpt(&prd, 1500),
                ),
                &["backend-lead", "architect"],
            );
            let be_ok = self.try_generate(Phase::Backend, be_p).await.is_some();
            self.emit(EngineEvent::SubTaskCompleted {
                phase: Phase::Backend,
                task_id: "backend.implement".into(),
                ok: be_ok,
            });
        }
        completed.push(self.record_phase(Phase::Backend, run_backend(&self.options))?);
        self.record_phase_timing(Phase::Backend, phase_start);
        self.maybe_verify_and_fix(Phase::Backend).await;

        let phase_start = std::time::Instant::now();
        self.transition(Phase::Quality, "")?;
        self.emit(EngineEvent::PhaseStarted {
            phase: Phase::Quality,
        });
        let quality_result = run_quality(&self.options);
        // Did the quality phase PRODUCE a gate file? If it did and we can't
        // read it back, that's a disk/permission failure, not "offline mode" —
        // we must NOT assume pass (that would mask a write failure as success).
        let produced_gate_file = quality_result.as_ref().is_ok_and(|o| {
            o.artifacts
                .iter()
                .any(|p| p.to_string_lossy().ends_with("-quality-gate.json"))
        });
        completed.push(self.record_phase(Phase::Quality, quality_result)?);
        self.record_phase_timing(Phase::Quality, phase_start);
        self.maybe_verify(Phase::Quality).await;

        let qg_path = self.options.project_root.join("output").join(format!(
            "{}-quality-gate.json",
            self.options.effective_slug()
        ));
        let quality_passed = if let Ok(qg) = std::fs::read_to_string(&qg_path) {
            let score = crate::phases::extract_quality_score(&qg);
            self.emit(EngineEvent::Note(format!(
                "质量门结果: {}/100 · {}",
                score.0,
                if score.1 { "PASSED ✓" } else { "BLOCKED ✗" }
            )));
            score.1
        } else if produced_gate_file {
            // The quality phase wrote the file but we can't read it back —
            // treat as a real failure rather than silently assuming pass.
            self.emit(EngineEvent::Note(format!(
                "⚠ 质量门文件写出后无法读回 ({}) — 判定未通过以防掩盖写盘失败。",
                qg_path.display()
            )));
            false
        } else {
            true // no gate file produced = offline/empty run → assume pass
        };

        if !quality_passed && use_runtime {
            self.emit(EngineEvent::Note(
                "⚠ 质量门未通过 — 阻断 delivery（SD-EVID-003）。请修复后重跑:\n  \
                 /redo 重跑整个流水线\n  \
                 或修复后 /continue 继续"
                    .to_string(),
            ));
            // SD-EVID-003: a worker-backed run that emits passed:false MUST
            // refuse to advance to delivery. Pause at quality so the next
            // `continue` re-checks the gate. Offline/template runs skip this
            // block — their quality gate is advisory, not a delivery gate.
            self.record_run_history(Phase::Quality, false, 0);
            self.emit(EngineEvent::BlockCompleted {
                final_phase: Phase::Quality,
                paused_at: None,
            });
            return Ok(RunReport {
                final_phase: Phase::Quality,
                paused_at: None,
                completed,
            });
        }

        let phase_start = std::time::Instant::now();
        self.transition(Phase::Delivery, "")?;
        self.emit(EngineEvent::PhaseStarted {
            phase: Phase::Delivery,
        });
        if use_runtime {
            self.emit(EngineEvent::Note(
                "📦 Worker producing deployment recipe (build verify + deploy commands)…"
                    .to_string(),
            ));
            let slug = self.options.effective_slug();
            let arch = std::fs::read_to_string(
                self.options
                    .project_root
                    .join(format!("output/{slug}-architecture.md")),
            )
            .unwrap_or_default();
            self.emit(EngineEvent::SubTaskStarted {
                phase: Phase::Delivery,
                task_id: "delivery.recipe".into(),
                label: "worker verifying production build + deploy instructions".into(),
            });
            let del_p = self.with_expert_knowledge(
                delivery_prompt(&slug, &self.options.requirement, &excerpt(&arch, 2000)),
                &["devops"],
            );
            let del_ok = self.try_generate(Phase::Delivery, del_p).await.is_some();
            self.emit(EngineEvent::SubTaskCompleted {
                phase: Phase::Delivery,
                task_id: "delivery.recipe".into(),
                ok: del_ok,
            });
        }
        completed.push(self.record_phase(Phase::Delivery, run_delivery(&self.options))?);
        self.record_phase_timing(Phase::Delivery, phase_start);

        // mark pipeline as done — keep phase=delivery, clear gate
        let done = WorkflowState {
            phase: Phase::Delivery.id().to_string(),
            active_gate: String::new(),
            slug: self.options.effective_slug(),
            requirement: self.options.requirement.clone(),
            last_transition_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            note: "Pipeline complete.".to_string(),
            backend: self.options.backend.clone(),
            spec_version: SPEC_VERSION.to_string(),
        };
        write_workflow_state(&self.options.project_root, &done)?;

        let artifact_count = completed.iter().map(|p| p.artifacts.len()).sum();
        self.record_run_history(Phase::Delivery, quality_passed, artifact_count);

        self.emit(EngineEvent::BlockCompleted {
            final_phase: Phase::Delivery,
            paused_at: None,
        });
        Ok(RunReport {
            final_phase: Phase::Delivery,
            paused_at: None,
            completed,
        })
    }

    /// Dispatch: read workflow-state, decide which block to run next.
    ///
    /// Guards against state-machine incoherence: the passed `approved_gate`
    /// MUST match the gate the persisted `workflow-state.json` says is open.
    /// A caller that passes `PreviewConfirm` while the state is still at
    /// `docs_confirm` would otherwise skip spec/frontend regeneration and
    /// jump straight to backend, producing artifacts out of order. On
    /// mismatch we return a descriptive error rather than silently advancing.
    pub async fn continue_from_gate(&self, approved_gate: Gate) -> std::io::Result<RunReport> {
        if let Some(state) = crate::state::read_workflow_state(&self.options.project_root) {
            let persisted = state.active_gate.as_str();
            let expected = approved_gate.id_str();
            if !persisted.is_empty() && persisted != expected {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "gate mismatch: you approved `{expected}` but the workflow state is at \
                         `{persisted}`. Run `super-dev continue` to advance from the actual gate, \
                         or `super-dev rollback latest` to undo."
                    ),
                ));
            }
        }
        match approved_gate {
            Gate::DocsConfirm => self.continue_after_docs_confirm().await,
            Gate::PreviewConfirm => self.continue_after_preview_confirm().await,
        }
    }

    /// Re-run the block that PRODUCED a gate, so a revision request
    /// regenerates the right artifacts and pauses at the same gate again.
    ///
    /// This is the inverse of [`continue_from_gate`], which advances past
    /// a gate. Revising at `docs_confirm` regenerates the three core docs
    /// (research → docs); revising at `preview_confirm` regenerates the
    /// spec → frontend (NOT the already-approved docs). The caller is
    /// expected to fold the user's revision feedback into
    /// `options.requirement` before calling so the worker incorporates it.
    ///
    /// `use_runtime` is honoured on BOTH branches: it forces the worker on
    /// (`true`) or off (`false`) regardless of whether `options.backend`
    /// is set. Callers that want "follow the configured backend" should
    /// pass `!options.backend.is_empty()`.
    ///
    /// [`continue_from_gate`]: AgentRunner::continue_from_gate
    pub async fn revise_at_gate(
        &self,
        gate: Gate,
        use_runtime: bool,
    ) -> std::io::Result<RunReport> {
        match gate {
            Gate::DocsConfirm => self.run_initial_block(use_runtime).await,
            // continue_after_docs_confirm re-derives use_runtime from
            // options.backend; that derivation agrees with the caller's
            // value when the caller passed `!options.backend.is_empty()`,
            // so behaviour is preserved.
            Gate::PreviewConfirm => self.continue_after_docs_confirm().await,
        }
    }

    fn transition(&self, next: Phase, active_gate: &str) -> std::io::Result<()> {
        let state = WorkflowState {
            phase: next.id().to_string(),
            active_gate: active_gate.to_string(),
            slug: self.options.effective_slug(),
            requirement: self.options.requirement.clone(),
            last_transition_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            note: format!(
                "Advanced to {} (worker: {})",
                next.id(),
                if self.options.backend.is_empty() {
                    "offline-templates"
                } else {
                    self.options.backend.as_str()
                }
            ),
            backend: self.options.backend.clone(),
            spec_version: SPEC_VERSION.to_string(),
        };
        write_workflow_state(&self.options.project_root, &state)?;
        // Always refresh the coach prompt so `.super-dev/coach/CURRENT.md`
        // matches the active phase the host should be executing.
        let _ = write_coach_prompt(&self.options, next);
        Ok(())
    }
}

/// Result of a verify pass — lets callers decide whether to auto-fix.
#[derive(Debug, Clone)]
struct VerifyOutcome {
    passed: bool,
    skipped: bool,
    /// The summarized stderr of the first failing step (empty if passed).
    failure_detail: String,
}

/// Distill a build/test stderr into the most useful error excerpt for the
/// user. Build tools emit pages of progress noise before the real error; the
/// last non-empty lines are where the actionable error lives. We take the last
/// 8 non-empty lines, capped at 1200 chars, so the TUI shows what actually
/// failed instead of truncating to just the first (usually noise) line.
fn summarize_stderr(stderr: &str) -> String {
    let meaningful: Vec<&str> = stderr
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .rev()
        .take(8)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let joined = meaningful.join("\n");
    if joined.chars().count() > 1200 {
        let mut idx = 1200;
        while !joined.is_char_boundary(idx) {
            idx -= 1;
        }
        let mut out = joined[..idx].to_string();
        out.push_str("\n…[truncated]");
        out
    } else {
        joined
    }
}

/// Read the exponential-retry base delay (ms) from
/// `SUPER_DEV_RETRY_BASE_MS`, defaulting to 2000 (2s). Used by
/// [`crate::runner::AgentRunner::try_generate`] so transient failures back off
/// (2s → 4s → 8s …) instead of hammering a rate-limited provider. Tests set a
/// tiny value to keep retry loops fast.
fn retry_base_ms() -> u64 {
    std::env::var("SUPER_DEV_RETRY_BASE_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|&v| v > 0)
        .unwrap_or(2000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use super_dev_runtime::{
        CompletionRequest, CompletionResponse, Runtime, RuntimeError, RuntimeKind, Usage,
    };
    use tempfile::TempDir;

    struct FakeRuntime;

    #[async_trait]
    impl Runtime for FakeRuntime {
        fn kind(&self) -> RuntimeKind {
            RuntimeKind::Anthropic
        }
        async fn complete(
            &self,
            _req: CompletionRequest,
        ) -> Result<CompletionResponse, RuntimeError> {
            Ok(CompletionResponse {
                text: "stub".into(),
                id: "stub".into(),
                model: "stub".into(),
                usage: Usage::default(),
            })
        }
    }

    fn opts(root: &std::path::Path) -> RunOptions {
        RunOptions {
            project_root: root.to_path_buf(),
            requirement: "build a login page".into(),
            slug: "demo".into(),
            model: "stub".into(),
            backend: String::new(),
            design_system: String::new(),
            seed_template: String::new(),
        }
    }

    /// A runtime that fails the first N calls with a transient 429, then
    /// succeeds. Used to prove try_generate retries with backoff.
    struct FlakyRuntime {
        fails_left: std::sync::Mutex<usize>,
    }

    #[async_trait]
    impl Runtime for FlakyRuntime {
        fn kind(&self) -> RuntimeKind {
            RuntimeKind::Anthropic
        }
        async fn complete(
            &self,
            _req: CompletionRequest,
        ) -> Result<CompletionResponse, RuntimeError> {
            let mut left = self.fails_left.lock().unwrap();
            if *left > 0 {
                *left -= 1;
                return Err(RuntimeError::HostProcess("429 Too Many Requests".into()));
            }
            Ok(CompletionResponse {
                text: "recovered".into(),
                id: "flaky".into(),
                model: "flaky".into(),
                usage: Usage::default(),
            })
        }
    }

    #[test]
    fn summarize_stderr_takes_last_meaningful_lines() {
        let stderr = "compiling...
downloading deps...

error TS2304: Cannot find name 'Foo'
  at src/App.tsx:12:5
";
        let out = summarize_stderr(stderr);
        // Progress noise ("compiling...", "downloading deps...") at the top
        // is dropped because we take the LAST non-empty lines. The actual
        // error must be present.
        assert!(out.contains("Cannot find name 'Foo'"));
        assert!(out.contains("src/App.tsx:12:5"));
    }

    #[test]
    fn summarize_stderr_truncates_long_output() {
        // Each line is long enough that 8 lines exceed 1200 chars → truncates.
        let line = "x".repeat(300);
        let long = format!("{line}\n{line}\n{line}\n{line}\n{line}\n{line}\n{line}\n{line}\n");
        let out = summarize_stderr(&long);
        assert!(
            out.ends_with("…[truncated]"),
            "expected truncation marker, got tail: {:?}",
            &out[out.len().saturating_sub(40)..]
        );
        assert!(out.chars().count() <= 1220); // 1200 + the marker
    }

    #[test]
    fn summarize_stderr_empty_returns_empty() {
        assert_eq!(summarize_stderr(""), "");
        assert_eq!(summarize_stderr("\n\n  \n"), "");
    }

    #[tokio::test]
    async fn verify_and_fix_skips_when_no_runtime() {
        // Offline mode (backend empty) → maybe_verify_and_fix must not attempt
        // a fix even if verify fails, because there's no worker to fix it.
        let tmp = TempDir::new().unwrap();
        // A package.json so detect_project sees a Node project, but no node_modules
        // so npm install fails → verify fails.
        std::fs::write(
            tmp.path().join("package.json"),
            r#"{"name":"x","scripts":{"build":"exit 1"}}"#,
        )
        .unwrap();
        let runner = AgentRunner::new(FakeRuntime, opts(tmp.path()));
        runner.start().unwrap();
        // Must not panic / hang — returns after the (failed) verify with no fix.
        runner.maybe_verify_and_fix(Phase::Frontend).await;
        // If we reach here without panic, the no-runtime early return worked.
    }

    #[test]
    fn verify_outcome_struct_pass_and_fail() {
        let p = VerifyOutcome {
            passed: true,
            skipped: false,
            failure_detail: String::new(),
        };
        assert!(p.passed);
        let f = VerifyOutcome {
            passed: false,
            skipped: false,
            failure_detail: "error TS2304".into(),
        };
        assert!(!f.passed);
        assert_eq!(f.failure_detail, "error TS2304");
    }

    #[test]
    fn retry_base_ms_honors_env_override() {
        // A tiny base keeps retry loops fast in tests.
        let prev = std::env::var("SUPER_DEV_RETRY_BASE_MS").ok();
        std::env::set_var("SUPER_DEV_RETRY_BASE_MS", "5");
        assert_eq!(retry_base_ms(), 5);
        // Restore / remove.
        if let Some(v) = prev {
            std::env::set_var("SUPER_DEV_RETRY_BASE_MS", v);
        } else {
            std::env::remove_var("SUPER_DEV_RETRY_BASE_MS");
        }
    }

    #[test]
    fn retry_base_ms_defaults_to_2000() {
        // Only assert the default when the env var is unset (other tests may
        // have set it). Remove first.
        let prev = std::env::var("SUPER_DEV_RETRY_BASE_MS").ok();
        std::env::remove_var("SUPER_DEV_RETRY_BASE_MS");
        assert_eq!(retry_base_ms(), 2000);
        if let Some(v) = prev {
            std::env::set_var("SUPER_DEV_RETRY_BASE_MS", v);
        }
    }

    #[tokio::test]
    async fn try_generate_retries_transient_then_succeeds() {
        let tmp = TempDir::new().unwrap();
        // Use a tiny backoff so the test is fast.
        std::env::set_var("SUPER_DEV_RETRY_BASE_MS", "1");
        let runner = AgentRunner::new(
            FlakyRuntime {
                fails_left: std::sync::Mutex::new(2),
            },
            opts(tmp.path()),
        );
        runner.start().unwrap();
        let p = crate::experts::research_prompt("demo", "req", "");
        let out = runner.try_generate(Phase::Research, p).await;
        std::env::remove_var("SUPER_DEV_RETRY_BASE_MS");
        // After 2 transient 429s the 3rd call succeeds → recovered text.
        assert_eq!(out.as_deref(), Some("recovered"));
    }

    #[tokio::test]
    async fn try_generate_gives_up_after_max_retries() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("SUPER_DEV_RETRY_BASE_MS", "1");
        // Always fails (fails_left huge) → exhausts retries → None.
        let runner = AgentRunner::new(
            FlakyRuntime {
                fails_left: std::sync::Mutex::new(99),
            },
            opts(tmp.path()),
        );
        runner.start().unwrap();
        let p = crate::experts::research_prompt("demo", "req", "");
        let out = runner.try_generate(Phase::Research, p).await;
        std::env::remove_var("SUPER_DEV_RETRY_BASE_MS");
        assert!(out.is_none(), "should give up after max_retries");
    }

    #[test]
    fn start_writes_initial_state() {
        let tmp = TempDir::new().unwrap();
        let runner = AgentRunner::new(FakeRuntime, opts(tmp.path()));
        let state = runner.start().unwrap();
        assert_eq!(state.phase, "research");
    }

    #[tokio::test]
    async fn initial_block_pauses_at_docs_confirm() {
        let tmp = TempDir::new().unwrap();
        let runner = AgentRunner::new(FakeRuntime, opts(tmp.path()));
        runner.start().unwrap();
        let r = runner.run_initial_block(false).await.unwrap();
        assert_eq!(r.final_phase, Phase::DocsConfirm);
        assert_eq!(r.paused_at, Some(Gate::DocsConfirm));
        assert!(tmp.path().join("output/demo-prd.md").is_file());
    }

    #[tokio::test]
    async fn after_docs_pauses_at_preview() {
        let tmp = TempDir::new().unwrap();
        let runner = AgentRunner::new(FakeRuntime, opts(tmp.path()));
        runner.start().unwrap();
        runner.run_initial_block(false).await.unwrap();
        let r = runner.continue_after_docs_confirm().await.unwrap();
        assert_eq!(r.final_phase, Phase::PreviewConfirm);
        assert_eq!(r.paused_at, Some(Gate::PreviewConfirm));
        assert!(tmp.path().join("output/demo-execution-plan.md").is_file());
        assert!(tmp.path().join("output/demo-frontend-notes.md").is_file());
    }

    #[tokio::test]
    async fn after_preview_runs_to_delivery() {
        let tmp = TempDir::new().unwrap();
        let runner = AgentRunner::new(FakeRuntime, opts(tmp.path()));
        runner.start().unwrap();
        runner.run_initial_block(false).await.unwrap();
        runner.continue_after_docs_confirm().await.unwrap();
        let r = runner.continue_after_preview_confirm().await.unwrap();
        assert_eq!(r.final_phase, Phase::Delivery);
        assert_eq!(r.paused_at, None);

        assert!(tmp.path().join("output/demo-backend-notes.md").is_file());
        assert!(tmp.path().join("output/demo-quality-gate.json").is_file());
        let release = tmp.path().join("release");
        let entries: Vec<_> = std::fs::read_dir(&release)
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert!(entries
            .iter()
            .any(|e| e.path().extension().and_then(|s| s.to_str()) == Some("zip")));
    }

    #[tokio::test]
    async fn worker_run_blocks_delivery_when_quality_gate_fails() {
        // SD-EVID-003: a worker-backed run whose quality gate emits
        // passed:false MUST refuse to advance to delivery. We simulate this
        // by writing a failing quality-gate.json before the preview→delivery
        // block, with a worker backend configured (use_runtime = true).
        let tmp = TempDir::new().unwrap();
        let mut o = opts(tmp.path());
        o.backend = "claude-code".into();
        let runner = AgentRunner::new(FakeRuntime, o);
        runner.start().unwrap();
        runner.run_initial_block(true).await.unwrap();
        runner.continue_after_docs_confirm().await.unwrap();
        // Overwrite the quality gate with a failing one (passed:false, score
        // below threshold). This runs before continue_after_preview_confirm
        // reaches its quality check.
        std::fs::write(
            tmp.path().join("output/demo-quality-gate.json"),
            r#"{"passed":false,"total_score":40,"weighted_score":40.0,"scenario":"test","critical_failures":["Build & test results"],"recommendations":[],"summary":{"executive_summary":"fail","summary_context":{}},"checks":[]}"#,
        )
        .unwrap();
        let r = runner.continue_after_preview_confirm().await.unwrap();
        // Must pause at quality, NOT advance to delivery.
        assert_eq!(r.final_phase, Phase::Quality);
        assert_eq!(r.paused_at, None);
        // No release zip should exist (delivery never ran).
        assert!(
            !tmp.path().join("release").is_dir()
                || !std::fs::read_dir(tmp.path().join("release"))
                    .unwrap()
                    .filter_map(Result::ok)
                    .any(|e| e.path().extension().and_then(|s| s.to_str()) == Some("zip"))
        );
    }

    #[tokio::test]
    async fn subtask_events_emitted_for_backend_worker() {
        // When a worker backend is configured, the backend phase emits
        // SubTaskStarted + SubTaskCompleted around the worker call.
        use crate::events::{EngineEvent, RecordingSink};
        use std::sync::Arc;
        let tmp = TempDir::new().unwrap();
        let mut o = opts(tmp.path());
        o.backend = "claude-code".into(); // triggers use_runtime path
        let sink = RecordingSink::new();
        let runner = AgentRunner::new(FakeRuntime, o).with_event_sink(Arc::new(sink.clone()));
        runner.start().unwrap();
        runner.run_initial_block(true).await.unwrap();
        runner.continue_after_docs_confirm().await.unwrap();
        runner.continue_after_preview_confirm().await.unwrap();

        let started = sink.count(|e| {
            matches!(
                e,
                EngineEvent::SubTaskStarted { task_id, .. } if task_id == "backend.implement"
            )
        });
        let completed = sink.count(|e| matches!(
            e,
            EngineEvent::SubTaskCompleted { task_id, ok: true, .. } if task_id == "backend.implement"
        ));
        assert_eq!(
            started, 1,
            "expected one SubTaskStarted for backend.implement"
        );
        assert_eq!(completed, 1, "expected one successful SubTaskCompleted");
    }

    #[tokio::test]
    async fn dispatch_routes_to_right_block() {
        let tmp = TempDir::new().unwrap();
        let runner = AgentRunner::new(FakeRuntime, opts(tmp.path()));
        runner.start().unwrap();
        runner.run_initial_block(false).await.unwrap();

        let r = runner.continue_from_gate(Gate::DocsConfirm).await.unwrap();
        assert_eq!(r.final_phase, Phase::PreviewConfirm);
        let r = runner
            .continue_from_gate(Gate::PreviewConfirm)
            .await
            .unwrap();
        assert_eq!(r.final_phase, Phase::Delivery);
    }

    #[tokio::test]
    async fn revise_at_docs_gate_regenerates_docs_and_pauses_again() {
        let tmp = TempDir::new().unwrap();
        let runner = AgentRunner::new(FakeRuntime, opts(tmp.path()));
        runner.start().unwrap();
        runner.run_initial_block(false).await.unwrap();
        // Revising at docs_confirm re-runs the docs block and pauses at
        // the SAME gate — it must NOT advance to preview_confirm.
        let r = runner
            .revise_at_gate(Gate::DocsConfirm, false)
            .await
            .unwrap();
        assert_eq!(r.final_phase, Phase::DocsConfirm);
        assert_eq!(r.paused_at, Some(Gate::DocsConfirm));
        assert!(tmp.path().join("output/demo-prd.md").is_file());
    }

    #[tokio::test]
    async fn revise_at_preview_gate_regenerates_frontend_not_docs() {
        let tmp = TempDir::new().unwrap();
        let runner = AgentRunner::new(FakeRuntime, opts(tmp.path()));
        runner.start().unwrap();
        runner.run_initial_block(false).await.unwrap();
        runner.continue_after_docs_confirm().await.unwrap();
        // Revising at preview_confirm re-runs spec→frontend and pauses at
        // preview_confirm again — the key fix: a UI revision must not throw
        // away the approved docs by regenerating them.
        let r = runner
            .revise_at_gate(Gate::PreviewConfirm, false)
            .await
            .unwrap();
        assert_eq!(r.final_phase, Phase::PreviewConfirm);
        assert_eq!(r.paused_at, Some(Gate::PreviewConfirm));
        assert!(tmp.path().join("output/demo-frontend-notes.md").is_file());
    }

    #[tokio::test]
    async fn initial_block_uses_runtime_when_requested() {
        // FakeRuntime returns "stub" text — verify it lands in the artifacts.
        let tmp = TempDir::new().unwrap();
        let runner = AgentRunner::new(FakeRuntime, opts(tmp.path()));
        runner.start().unwrap();
        runner.run_initial_block(true).await.unwrap();

        let research = std::fs::read_to_string(tmp.path().join("output/demo-research.md")).unwrap();
        let prd = std::fs::read_to_string(tmp.path().join("output/demo-prd.md")).unwrap();
        assert_eq!(research.trim(), "stub");
        assert_eq!(prd.trim(), "stub");
    }

    #[tokio::test]
    async fn event_sink_observes_the_full_pipeline() {
        use crate::events::{EngineEvent, RecordingSink};
        use std::sync::Arc;

        let tmp = TempDir::new().unwrap();
        let sink = RecordingSink::new();
        let runner =
            AgentRunner::new(FakeRuntime, opts(tmp.path())).with_event_sink(Arc::new(sink.clone()));
        runner.start().unwrap();
        runner.run_initial_block(false).await.unwrap();
        runner.continue_after_docs_confirm().await.unwrap();
        runner.continue_after_preview_confirm().await.unwrap();

        let events = sink.events();
        // Exactly one PipelineStarted.
        assert_eq!(
            sink.count(|e| matches!(e, EngineEvent::PipelineStarted { .. })),
            1
        );
        // All seven worker phases emit a PhaseStarted (gates do not).
        assert_eq!(
            sink.count(|e| matches!(e, EngineEvent::PhaseStarted { .. })),
            7
        );
        // Both gates open.
        assert_eq!(
            sink.count(|e| matches!(e, EngineEvent::GateOpened { .. })),
            2
        );
        // Three BlockCompleted (initial, docs→preview, preview→delivery).
        assert_eq!(
            sink.count(|e| matches!(e, EngineEvent::BlockCompleted { .. })),
            3
        );
        // The last event is the delivery BlockCompleted with no gate.
        assert!(matches!(
            events.last(),
            Some(EngineEvent::BlockCompleted {
                final_phase: Phase::Delivery,
                paused_at: None,
            })
        ));
        // First event is always PipelineStarted.
        assert!(matches!(
            events.first(),
            Some(EngineEvent::PipelineStarted { .. })
        ));
    }

    #[tokio::test]
    async fn knowledge_note_emitted_when_knowledge_dir_present() {
        use crate::events::{EngineEvent, RecordingSink};
        use std::sync::Arc;

        let tmp = TempDir::new().unwrap();
        // Seed a tiny knowledge/ so the runner picks something.
        let kd = tmp.path().join("knowledge").join("security");
        std::fs::create_dir_all(&kd).unwrap();
        std::fs::write(
            kd.join("login-playbook.md"),
            "# Login Playbook\nUse OAuth2 + PKCE.\n",
        )
        .unwrap();

        let sink = RecordingSink::new();
        let runner =
            AgentRunner::new(FakeRuntime, opts(tmp.path())).with_event_sink(Arc::new(sink.clone()));
        runner.start().unwrap();
        runner.run_initial_block(false).await.unwrap();

        let knowledge_notes: Vec<EngineEvent> = sink
            .events()
            .into_iter()
            .filter(|e| matches!(e, EngineEvent::Note(s) if s.contains("knowledge")))
            .collect();
        assert_eq!(knowledge_notes.len(), 1);
        if let EngineEvent::Note(text) = &knowledge_notes[0] {
            assert!(text.contains("login-playbook"));
        } else {
            unreachable!("filtered for Note above")
        }
    }

    #[tokio::test]
    async fn no_knowledge_note_when_no_knowledge_dir() {
        use crate::events::{EngineEvent, RecordingSink};
        use std::sync::Arc;

        let tmp = TempDir::new().unwrap();
        // No knowledge/ subdir at all.
        let sink = RecordingSink::new();
        let runner =
            AgentRunner::new(FakeRuntime, opts(tmp.path())).with_event_sink(Arc::new(sink.clone()));
        runner.start().unwrap();
        runner.run_initial_block(false).await.unwrap();

        let knowledge_notes = sink
            .events()
            .iter()
            .filter(|e| matches!(e, EngineEvent::Note(s) if s.contains("knowledge")))
            .count();
        assert_eq!(knowledge_notes, 0);
    }

    #[tokio::test]
    async fn null_sink_is_the_default_and_runs_clean() {
        // A runner with no sink attached must behave identically.
        let tmp = TempDir::new().unwrap();
        let runner = AgentRunner::new(FakeRuntime, opts(tmp.path()));
        runner.start().unwrap();
        let r = runner.run_initial_block(false).await.unwrap();
        assert_eq!(r.final_phase, Phase::DocsConfirm);
    }

    #[tokio::test]
    async fn maybe_verify_skips_when_no_project_manifest() {
        use crate::events::{EngineEvent, RecordingSink};
        use std::sync::Arc;

        let tmp = TempDir::new().unwrap();
        let sink = RecordingSink::new();
        let runner =
            AgentRunner::new(FakeRuntime, opts(tmp.path())).with_event_sink(Arc::new(sink.clone()));
        runner.maybe_verify(Phase::Frontend).await;
        let events = sink.events();
        // Frontend → verify started + skipped (no manifest in tmp dir).
        assert!(events
            .iter()
            .any(|e| matches!(e, EngineEvent::VerifyStarted { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, EngineEvent::VerifySkipped { .. })));
    }

    #[tokio::test]
    async fn maybe_verify_is_noop_for_non_code_phases() {
        use crate::events::RecordingSink;
        use std::sync::Arc;

        let tmp = TempDir::new().unwrap();
        let sink = RecordingSink::new();
        let runner =
            AgentRunner::new(FakeRuntime, opts(tmp.path())).with_event_sink(Arc::new(sink.clone()));
        // research / docs / spec / delivery do NOT trigger verify.
        for phase in [Phase::Research, Phase::Docs, Phase::Spec, Phase::Delivery] {
            runner.maybe_verify(phase).await;
        }
        assert!(sink.events().is_empty());
    }

    #[tokio::test]
    async fn maybe_verify_records_outcome_to_audit_jsonl() {
        use std::sync::Arc;

        let tmp = TempDir::new().unwrap();
        // Drop a valid Rust manifest so verify picks Rust + tries cargo check.
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"verify-test\"\nversion = \"0.0.1\"\nedition = \"2021\"",
        )
        .unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        std::fs::write(tmp.path().join("src/main.rs"), "fn main(){}").unwrap();

        let sink = crate::events::RecordingSink::new();
        let runner =
            AgentRunner::new(FakeRuntime, opts(tmp.path())).with_event_sink(Arc::new(sink.clone()));
        runner.maybe_verify(Phase::Frontend).await;

        let audit = tmp.path().join(".super-dev/audit/verify.jsonl");
        assert!(audit.exists(), "verify.jsonl was not created");
        let body = std::fs::read_to_string(&audit).unwrap();
        assert!(body.contains("\"phase\":\"frontend\""));
        assert!(body.contains("\"project_kind\":\"rust\""));
    }

    #[test]
    fn expert_knowledge_injection_works() {
        let tmp = TempDir::new().unwrap();
        // Create an expert file
        let expert_dir = tmp.path().join("knowledge/experts/product-manager");
        std::fs::create_dir_all(&expert_dir).unwrap();
        std::fs::write(
            expert_dir.join("methodology.md"),
            "# PM Methodology\n\n## RICE Scoring\nReach × Impact × Confidence / Effort\n",
        )
        .unwrap();

        let runner = AgentRunner::new(FakeRuntime, opts(tmp.path()));
        let prompt = Prompt {
            system: "Base system prompt.".to_string(),
            user: "Write a PRD.".to_string(),
        };
        let enhanced = runner.with_expert_knowledge(prompt, &["product-manager"]);
        assert!(
            enhanced.system.contains("RICE Scoring"),
            "Expert knowledge not injected into prompt. System: {}",
            enhanced.system
        );
        assert!(
            enhanced.system.contains("Base system prompt"),
            "Original system prompt lost"
        );
    }

    #[test]
    fn expert_knowledge_noop_when_no_files() {
        let tmp = TempDir::new().unwrap();
        let runner = AgentRunner::new(FakeRuntime, opts(tmp.path()));
        let prompt = Prompt {
            system: "Original.".to_string(),
            user: "User.".to_string(),
        };
        let enhanced = runner.with_expert_knowledge(prompt, &["nonexistent-expert"]);
        assert_eq!(enhanced.system, "Original.");
    }

    #[test]
    fn superdevrc_config_is_loaded() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".superdevrc"),
            "[quality]\nthreshold = 75\n\n[pipeline]\nmax_review_rounds = 1\n",
        )
        .unwrap();
        let cfg = crate::config::load_project_config(tmp.path());
        assert_eq!(cfg.quality.threshold, 75);
        assert_eq!(cfg.pipeline.max_review_rounds, 1);
    }

    // ---- review function tests ----

    #[test]
    fn review_research_detects_missing_sections() {
        let empty = AgentRunner::<FakeRuntime>::review_research("");
        assert!(empty.len() >= 3);
    }

    #[test]
    fn review_research_passes_complete_doc() {
        let doc = "## Discovery\ntarget audience: devs\n\n## Similar products\n- Tool A\n- Tool B\n\n## Domain risks\nHigh complexity\n\n## Design system recommendation\nModern minimal";
        let defects = AgentRunner::<FakeRuntime>::review_research(doc);
        assert!(defects.is_empty(), "unexpected defects: {defects:?}");
    }

    #[test]
    fn review_prd_detects_missing_sections() {
        let defects = AgentRunner::<FakeRuntime>::review_prd("Just a paragraph.");
        assert!(defects.len() >= 3, "expected ≥3 defects, got {defects:?}");
    }

    #[test]
    fn review_prd_flags_present_but_empty_section() {
        // Regression: a doc with a bare `## Goal` (no body) used to pass
        // review because only the heading presence was checked.
        let doc =
            "## Goal\n\n## Scope\nreal scope body\n\n## Acceptance Criteria\n- [ ] a\n- [ ] b";
        let defects = AgentRunner::<FakeRuntime>::review_prd(doc);
        assert!(
            defects
                .iter()
                .any(|d| d.contains("'## goal' is present but empty")),
            "empty ## Goal must be flagged, got {defects:?}"
        );
        // ## Scope has body → must NOT be flagged empty.
        assert!(
            !defects
                .iter()
                .any(|d| d.contains("'## scope' is present but empty")),
            "## Scope has body, must not be flagged: {defects:?}"
        );
    }

    #[test]
    fn review_prd_passes_complete_doc() {
        let doc = "\
## Goal\nBuild a login system\n\n\
## Target Users\nPersona: developer\n\n\
## Scope\n### In scope\n- auth\n### Out of scope\n- billing\n\n\
## Functional Requirements\n- Login\n- Register\n\n\
## Non-Functional Requirements\nPerformance: <200ms\n\n\
## Acceptance Criteria\n- [ ] User can login\n- [ ] User can register\n- [ ] Password reset\n\n\
## Success Metrics\nKPI: DAU > 100";
        let defects = AgentRunner::<FakeRuntime>::review_prd(doc);
        assert!(defects.is_empty(), "unexpected defects: {defects:?}");
    }

    #[test]
    fn review_architecture_detects_missing() {
        let defects = AgentRunner::<FakeRuntime>::review_architecture("Just a paragraph.");
        assert!(defects.len() >= 4, "expected ≥4 defects, got {defects:?}");
    }

    #[test]
    fn review_architecture_passes_complete() {
        let doc = "\
## API Surface\n\
| Method | Path | Auth | Description |\n\
|---|---|---|---|\n\
| POST | /api/login | none | Login |\n\
| GET | /api/users | JWT | List users |\n\
| POST | /api/register | none | Register |\n\n\
## Data Model\nUser entity:\n\
| Field | Type | Notes |\n|---|---|---|\n| id | uuid | PK |\n| email | text | unique |\n\n\
## Auth\nJWT tokens\n\n\
## Tech Stack\nRust + React\n\n\
## Error Convention\n4xx/5xx standard codes\n\n\
## Project Structure\nsrc/ layout";
        let defects = AgentRunner::<FakeRuntime>::review_architecture(doc);
        assert!(defects.is_empty(), "unexpected defects: {defects:?}");
    }

    #[test]
    fn review_uiux_detects_missing() {
        let defects = AgentRunner::<FakeRuntime>::review_uiux("Just text.");
        assert!(defects.len() >= 3, "expected ≥3 defects, got {defects:?}");
    }

    #[test]
    fn review_uiux_passes_complete() {
        let tokens = "--color-primary: #1a1a1a;\n".repeat(10);
        let doc = format!(
            "# UIUX\n{tokens}\n\
             Dark mode: @media (prefers-color-scheme: dark)\n\
             font-family: Inter\n\
             Icon library: Lucide\n\
             Hover states defined"
        );
        let defects = AgentRunner::<FakeRuntime>::review_uiux(&doc);
        assert!(defects.is_empty(), "unexpected defects: {defects:?}");
    }
}
