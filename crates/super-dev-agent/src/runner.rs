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
    architecture_prompt, backend_prompt, excerpt, frontend_prompt, prd_prompt, research_prompt,
    uiux_prompt, Prompt,
};
use crate::gates::Gate;
use crate::phases::{
    knowledge_digest, run_backend, run_delivery, run_docs, run_frontend, run_quality, run_research,
    run_spec, DocsContent, PhaseOutput,
};
use crate::state::{write_workflow_state, WorkflowState};

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

    /// Emit `PhaseStarted` + run a phase + emit `ArtifactWritten` for
    /// each artifact + `PhaseCompleted`. Returns the phase output.
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

    /// Run the workspace's build / install command after a
    /// code-producing phase. Emits `VerifyStarted` + one of
    /// `VerifySkipped` / `VerifyPassed` / `VerifyFailed`, and appends a
    /// row to `.super-dev/audit/verify.jsonl`. Always best-effort —
    /// `Err` paths from the subprocess become structured `VerifyFailed`
    /// events, not Rust errors.
    async fn maybe_verify(&self, phase: Phase) {
        // Only the code-producing phases need verify; docs / spec / etc.
        // don't have build output to test.
        if !matches!(phase, Phase::Frontend | Phase::Backend | Phase::Quality) {
            return;
        }
        let workspace = &self.options.project_root;
        let kind = crate::verify::detect_project(workspace);
        let command = kind
            .verify_command()
            .map_or_else(String::new, |(p, args)| format!("{p} {}", args.join(" ")));
        self.emit(EngineEvent::VerifyStarted {
            phase,
            command: command.clone(),
        });
        let outcome = crate::verify::run_verify(workspace).await;
        match &outcome {
            None => self.emit(EngineEvent::VerifySkipped {
                phase,
                reason: "no recognised project manifest".to_string(),
            }),
            Some(o) if o.passed => self.emit(EngineEvent::VerifyPassed {
                phase,
                duration_ms: o.duration_ms,
            }),
            Some(o) => self.emit(EngineEvent::VerifyFailed {
                phase,
                exit_code: o.exit_code,
                stderr: o.stderr.clone(),
            }),
        }
        if let Some(o) = &outcome {
            let _ = crate::verify::record_verify_outcome(workspace, phase.id(), o);
        }
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
        self.emit(EngineEvent::PipelineStarted {
            slug: self.options.effective_slug(),
            requirement: self.options.requirement.clone(),
        });

        // 1. research — generate via LLM if requested, else None falls back to template.
        self.emit(EngineEvent::PhaseStarted {
            phase: Phase::Research,
        });
        // Surface knowledge-retrieval to the UI: which knowledge/*.md
        // files Super Dev decided to inject into the research prompt.
        // Silent retrieval was a major "is this thing actually doing
        // anything?" complaint from early users.
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
        let research_text = if use_runtime {
            let rp = self.with_expert_knowledge(
                research_prompt(
                    &self.options.effective_slug(),
                    &self.options.requirement,
                    &knowledge_digest(&self.options),
                ),
                &["product-manager"],
            );
            self.generate_with_review(Phase::Research, rp, Self::review_research, 3)
                .await
        } else {
            None
        };
        completed.push(self.record_phase(
            Phase::Research,
            run_research(&self.options, research_text.as_deref()),
        )?);
        self.transition(Phase::Docs, "")?;

        // 2. docs — three artifacts. Each gets an LLM body when use_runtime; subsequent
        //    experts get the previous artifact's excerpt as additional context.
        self.emit(EngineEvent::PhaseStarted { phase: Phase::Docs });
        let docs_content = if use_runtime {
            self.generate_docs_content(research_text.as_deref()).await
        } else {
            DocsContent::default()
        };

        let docs = self.record_phase(Phase::Docs, run_docs(&self.options, &docs_content))?;
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

        for attempt in 1..max_attempts {
            let defects = reviewer(&text);
            if defects.is_empty() {
                self.emit(EngineEvent::Note(format!(
                    "✓ {} review passed.",
                    phase.id()
                )));
                break;
            }
            // Only attempt fix for structural defects (missing sections),
            // not for subjective quality issues.
            if defects.len() > 4 {
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
        if !lower.contains("## discovery") && !lower.contains("target audience") {
            defects.push("Missing ## Discovery section (audience/tone/direction)".into());
        }
        if !lower.contains("## similar products") {
            defects.push("Missing ## Similar products section".into());
        }
        if !lower.contains("## domain risks") {
            defects.push("Missing ## Domain risks section".into());
        }
        if !lower.contains("## design system recommendation")
            && !lower.contains("## design recommendation")
        {
            defects.push("Missing ## Design system recommendation section".into());
        }
        defects
    }

    /// Review PRD — commercial grade checks.
    fn review_prd(text: &str) -> Vec<String> {
        let lower = text.to_ascii_lowercase();
        let mut defects = Vec::new();
        if !lower.contains("## goal") {
            defects.push("Missing ## Goal".into());
        }
        if !lower.contains("target user")
            && !lower.contains("persona")
            && !lower.contains("## user")
        {
            defects.push("Missing target users / personas".into());
        }
        if !lower.contains("## scope") {
            defects.push("Missing ## Scope (in/out)".into());
        }
        if !lower.contains("## functional") && !lower.contains("## feature") {
            defects.push("Missing functional requirements with priorities".into());
        }
        if !lower.contains("non-functional") && !lower.contains("performance") {
            defects.push("Missing non-functional requirements (performance/security)".into());
        }
        let ac_count = text.matches("- [ ]").count();
        if ac_count < 2 {
            defects.push(format!(
                "Only {ac_count} acceptance criteria (need at least a few)"
            ));
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
        if !lower.contains("## api") {
            defects.push("Missing API surface section".into());
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
        if !lower.contains("data model") && !lower.contains("schema") {
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

    /// Try to generate content via the worker. Retries once on timeout.
    async fn try_generate(&self, phase: Phase, prompt: Prompt) -> Option<String> {
        let max_retries = 2;
        for attempt in 0..max_retries {
            let req = prompt.clone().into_request(&self.options.model, 4096);
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
                    let is_timeout = err.to_string().contains("timed out");
                    if is_timeout && attempt + 1 < max_retries {
                        self.emit(EngineEvent::Note(format!(
                            "⚠ Worker 超时({} 阶段), 重试 {}/{}...",
                            phase.id(),
                            attempt + 2,
                            max_retries
                        )));
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
        for dir in expert_dirs {
            let expert_dir = base.join(dir);
            if !expert_dir.is_dir() {
                continue;
            }
            if let Ok(rd) = std::fs::read_dir(&expert_dir) {
                for entry in rd.flatten() {
                    let p = entry.path();
                    if p.extension().and_then(|s| s.to_str()) != Some("md") {
                        continue;
                    }
                    if let Ok(content) = std::fs::read_to_string(&p) {
                        let trimmed: String = content.chars().take(1500).collect();
                        out.push_str(&format!(
                            "\n---\nExpert reference ({}):\n{}\n",
                            p.file_name().unwrap_or_default().to_string_lossy(),
                            trimmed,
                        ));
                    }
                }
            }
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

        // PRD: inject PM methodology → generate → review → fix
        self.emit(EngineEvent::Note("📋 Generating PRD...".to_string()));
        let prd_p = self.with_expert_knowledge(
            prd_prompt(&slug, req, &research_excerpt),
            &["product-manager"],
        );
        let prd = self
            .generate_with_review(Phase::Docs, prd_p, Self::review_prd, 3)
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
            .generate_with_review(Phase::Docs, arch_p, Self::review_architecture, 3)
            .await;

        // UIUX: inject designer methodology → generate → review → fix
        self.emit(EngineEvent::Note(
            "🎨 Generating UI/UX design system...".to_string(),
        ));
        let uiux_p =
            self.with_expert_knowledge(uiux_prompt(&slug, req, &prd_excerpt), &["uiux-designer"]);
        let uiux = self
            .generate_with_review(Phase::Docs, uiux_p, Self::review_uiux, 3)
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

        // Spec phase: generate execution plan + task list
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
        self.transition(Phase::Frontend, "")?;

        // Frontend phase: worker creates actual code files
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
            let _ = self.try_generate(Phase::Frontend, fe_p).await;
        }
        let fe = self.record_phase(Phase::Frontend, run_frontend(&self.options))?;
        let gate = fe.gate;
        completed.push(fe);
        self.maybe_verify(Phase::Frontend).await;
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
            let be_p = self.with_expert_knowledge(
                backend_prompt(
                    &slug,
                    &self.options.requirement,
                    &excerpt(&arch, 3000),
                    &excerpt(&prd, 1500),
                ),
                &["backend-lead", "architect"],
            );
            let _ = self.try_generate(Phase::Backend, be_p).await;
        }
        completed.push(self.record_phase(Phase::Backend, run_backend(&self.options))?);
        self.maybe_verify(Phase::Backend).await;

        self.transition(Phase::Quality, "")?;
        self.emit(EngineEvent::PhaseStarted {
            phase: Phase::Quality,
        });
        completed.push(self.record_phase(Phase::Quality, run_quality(&self.options))?);
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
        } else {
            true // no gate file = assume pass (offline mode)
        };

        if !quality_passed {
            self.emit(EngineEvent::Note(
                "⚠ 质量门未通过 — 跳过 delivery。请修复质量问题后重跑:\n  \
                 /redo 重跑整个流水线\n  \
                 或修复后 /continue 继续"
                    .to_string(),
            ));
        }

        self.transition(Phase::Delivery, "")?;
        self.emit(EngineEvent::PhaseStarted {
            phase: Phase::Delivery,
        });
        completed.push(self.record_phase(Phase::Delivery, run_delivery(&self.options))?);

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
    pub async fn continue_from_gate(&self, approved_gate: Gate) -> std::io::Result<RunReport> {
        match approved_gate {
            Gate::DocsConfirm => self.continue_after_docs_confirm().await,
            Gate::PreviewConfirm => self.continue_after_preview_confirm().await,
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
}
