//! Coach mode — writes a self-contained instruction file per phase
//! that the *host* (Claude Code / Codex / Antigravity / …) executes.
//!
//! Super Dev does not need an API key when running inside a host that
//! already has model access. Instead of calling the model itself, the
//! coach writes a deterministic prompt file the host can read and
//! follow. After the host produces the required artifact, the user
//! runs `super-dev continue` and Super Dev verifies + advances.
//!
//! This module is the *single source of truth* for what each phase
//! tells the host to do. Each `coach_<phase>` returns a complete
//! markdown document with:
//!
//! 1. The spec preamble (non-negotiable rules from
//!    `SUPER_DEV_HOST_SPEC_V1`).
//! 2. The expert role definition.
//! 3. The task description.
//! 4. The required output path + section structure.
//! 5. The input context (requirement, knowledge digest, prior artifacts).
//! 6. The next step (`super-dev continue`).

use std::fs;
use std::io;
use std::path::PathBuf;

use super_dev_spec::Phase;

use crate::runner::RunOptions;

/// Subdirectory under `.super-dev/` where coach prompts live.
pub const COACH_DIR: &str = ".super-dev/coach";

/// Write the coach prompt for `phase` to `.super-dev/coach/<NN>-<phase>.md`.
///
/// Returns the absolute path of the written file. The phase number
/// prefix matches `PHASE_CHAIN` ordering so the directory listing
/// reads top-to-bottom in pipeline order.
pub fn write_coach_prompt(opts: &RunOptions, phase: Phase) -> io::Result<PathBuf> {
    let dir = opts.project_root.join(COACH_DIR);
    fs::create_dir_all(&dir)?;
    let body = render_coach_prompt(opts, phase);
    let path = dir.join(coach_filename(phase));
    fs::write(&path, body)?;
    // Mirror to CURRENT.md so the host's CLAUDE.md can point at one
    // stable path without needing to know the phase number.
    let current = dir.join("CURRENT.md");
    let header = format!(
        "<!-- Symlink-equivalent: the active phase is `{}`. See {}. -->\n",
        phase.id(),
        coach_filename(phase),
    );
    let mut current_body = header;
    current_body.push_str(&render_coach_prompt(opts, phase));
    fs::write(&current, current_body)?;
    Ok(path)
}

fn coach_filename(phase: Phase) -> String {
    let n = match phase {
        Phase::Research => 1,
        Phase::Docs => 2,
        Phase::DocsConfirm => 3,
        Phase::Spec => 4,
        Phase::Frontend => 5,
        Phase::PreviewConfirm => 6,
        Phase::Backend => 7,
        Phase::Quality => 8,
        Phase::Delivery => 9,
    };
    format!("{n:02}-{}.md", phase.id())
}

/// Pure renderer — exposed for tests.
#[must_use]
pub fn render_coach_prompt(opts: &RunOptions, phase: Phase) -> String {
    let slug = opts.effective_slug();
    let req = &opts.requirement;
    let preamble = spec_preamble();
    let body = match phase {
        Phase::Research => render_research(&slug, req, opts),
        Phase::Docs => render_docs(&slug, req),
        Phase::DocsConfirm | Phase::PreviewConfirm => render_gate(phase, &slug),
        Phase::Spec => render_spec(&slug, req),
        Phase::Frontend => render_frontend(&slug, req),
        Phase::Backend => render_backend(&slug, req),
        Phase::Quality => render_quality(&slug),
        Phase::Delivery => render_delivery(&slug),
    };
    format!(
        "# Super Dev coach — phase `{}`\n\n\
         > Read this file and produce the required output. Then run \
         `super-dev continue` to advance.\n\n\
         {preamble}\n\n\
         {body}\n",
        phase.id()
    )
}

fn spec_preamble() -> String {
    "\
## Spec preamble (non-negotiable)\n\n\
You are operating inside a Super Dev pipeline run governed by \
`SUPER_DEV_HOST_SPEC_V1`. Every artifact you produce MUST follow these \
rules:\n\n\
- Use a declared icon library (Lucide / Heroicons / Tabler). NEVER \
  emoji as functional icons.\n\
- Use design tokens (CSS vars / theme keys). NEVER hardcoded hex / \
  rgb / hsl colors in production UI.\n\
- Frontend `fetch` URLs MUST match an API path declared in the \
  architecture document.\n\
- Wait for explicit user approval at `docs_confirm` and \
  `preview_confirm` gates.\n\
- Output goes into structured markdown sections — do not invent new \
  top-level sections, fill the ones requested below.\
"
    .to_string()
}

fn render_research(slug: &str, req: &str, opts: &RunOptions) -> String {
    let knowledge = crate::phases::knowledge_digest(opts);
    format!(
        "## Role\n\nSenior product researcher.\n\n\
         ## Task\n\nProduce a research brief that grounds PM / architect / UI work. \
         Cite up to 5 similar products with one-line takeaways. Surface 5 risks \
         the architecture must mitigate. List 5 UI / UX patterns that are \
         non-negotiable in this domain.\n\n\
         ## Required output\n\n\
         - **Path:** `output/{slug}-research.md`\n\
         - **Sections (in order):**\n\
           - `# Research — {slug}`\n\
           - `## Requirement` (echo verbatim)\n\
           - `## Similar products`\n\
           - `## Domain risks`\n\
           - `## UI / UX must-haves`\n\
           - `## Open questions`\n\n\
         ## Input\n\n\
         ### Requirement\n\n{req}\n\n\
         ### Local knowledge files available\n\n{knowledge}\n",
    )
}

fn render_docs(slug: &str, req: &str) -> String {
    format!(
        "## Role\n\nThree experts in sequence: senior product manager, senior \
         architect, senior UI/UX lead.\n\n\
         ## Task\n\nProduce the three core documents. Each expert builds on the \
         previous artifact — read the research brief and the prior doc before \
         writing the next.\n\n\
         ## Required outputs\n\n\
         ### 1. PRD — `output/{slug}-prd.md`\n\
         - `# PRD — {slug}`\n\
         - `## Goal` (2-4 sentences)\n\
         - `## Scope` (`in` and `out` bullets)\n\
         - `## User stories` (3-7 stories in `As a … I want … so that …` form)\n\
         - `## Acceptance criteria` (testable checkbox list, 5-10 items)\n\
         - `## Risks & open questions`\n\n\
         ### 2. Architecture — `output/{slug}-architecture.md`\n\
         - `# Architecture — {slug}`\n\
         - `## System overview` (1-3 paragraphs)\n\
         - `## API surface` (markdown table `| Method | Path | Purpose |` with ≥5 rows; every path starts with `/`)\n\
         - `## Data model`\n\
         - `## Tech-stack rationale` (one sentence per choice)\n\
         - `## Open trade-offs`\n\n\
         ### 3. UI/UX — `output/{slug}-uiux.md`\n\
         - `# UI/UX — {slug}`\n\
         - `## Design tokens` (a single ```css fenced block with `:root` rules)\n\
         - `## Icon library` (one of Lucide / Heroicons / Tabler — declare it)\n\
         - `## Page hierarchy` (nested list)\n\
         - `## Component skeleton`\n\
         - `## Accessibility notes`\n\n\
         ## Input\n\n\
         ### Requirement\n\n{req}\n\n\
         ### Research brief\n\n\
         Read `output/{slug}-research.md` for context.\n\n\
         ## After you finish\n\n\
         Run `super-dev continue` and approve the `docs_confirm` gate (`super-dev continue`) only after the user has reviewed.\n",
    )
}

fn render_gate(phase: Phase, slug: &str) -> String {
    let (artifact_block, headline) = match phase {
        Phase::DocsConfirm => (
            format!(
                "- `output/{slug}-prd.md`\n\
                 - `output/{slug}-architecture.md`\n\
                 - `output/{slug}-uiux.md`"
            ),
            "Wait for the user to review the three core documents.",
        ),
        Phase::PreviewConfirm => (
            format!("- `output/{slug}-frontend-notes.md`\n- Live preview"),
            "Wait for the user to verify the frontend preview against the UI/UX spec.",
        ),
        _ => unreachable!("render_gate only handles DocsConfirm / PreviewConfirm"),
    };
    format!(
        "## Role\n\nGate keeper.\n\n\
         ## Task\n\n{headline}\n\n\
         ### Artifacts under review\n\n{artifact_block}\n\n\
         ## What to do now\n\n\
         - Wait. Do NOT advance.\n\
         - When the user is satisfied, they will run `super-dev continue` themselves.\n\
         - If they request revisions, they will run `super-dev revise \"<text>\"`. \
           In that case, re-execute the previous phase with the requested changes.\n",
    )
}

fn render_spec(slug: &str, req: &str) -> String {
    format!(
        "## Role\n\nDelivery lead.\n\n\
         ## Task\n\nTranslate the approved three core documents into an execution plan + machine-trackable task list.\n\n\
         ## Required outputs\n\n\
         - **Plan:** `output/{slug}-execution-plan.md`\n\
           - `# Execution plan — {slug}`\n\
           - `## Goal recap`\n\
           - `## Sequence` (numbered top-level steps that produce real artifacts)\n\
           - `## Risk register`\n\n\
         - **Tasks:** `.super-dev/changes/<change-id>/tasks.md`\n\
           - Flat bullet list of `- [ ] <work item>` lines, one per shippable unit\n\n\
         ## Input\n\n\
         ### Requirement\n\n{req}\n\n\
         Read `output/{slug}-prd.md`, `output/{slug}-architecture.md`, `output/{slug}-uiux.md` first.\n",
    )
}

fn render_frontend(slug: &str, req: &str) -> String {
    format!(
        "## Role\n\nFrontend lead.\n\n\
         ## Task\n\nImplement the frontend skeleton + first runnable preview, following the architecture API surface and UIUX tokens exactly.\n\n\
         ## Hard rules (will be audited)\n\n\
         - Icons from the declared library only (Lucide / Heroicons / Tabler). Zero emoji as icons.\n\
         - Colors from design tokens only. Zero hardcoded hex / rgb / hsl in production code.\n\
         - Every `fetch` URL MUST also appear in `output/{slug}-architecture.md` API surface table.\n\
         - Run a runtime smoke check and capture a screenshot for the preview gate review.\n\n\
         ## Required output\n\n\
         - **Code:** in your project's frontend directory (Super Dev does not enforce its layout)\n\
         - **Notes:** `output/{slug}-frontend-notes.md` summarising what was built, with a checkbox list of the audit items above.\n\n\
         ## Input\n\n\
         ### Requirement\n\n{req}\n\n\
         Read `output/{slug}-prd.md` + `output/{slug}-architecture.md` + `output/{slug}-uiux.md`.\n\n\
         ## After you finish\n\nRun the runtime preview and ask the user to review. They will approve via `super-dev continue`.\n",
    )
}

fn render_backend(slug: &str, req: &str) -> String {
    format!(
        "## Role\n\nBackend lead.\n\n\
         ## Task\n\nImplement the backend handlers + storage to match the architecture API surface and any URL the frontend has already written.\n\n\
         ## Hard rules\n\n\
         - Every route in `.super-dev/audit/frontend-api-calls.jsonl` MUST have a matching backend handler.\n\
         - Add tests covering acceptance criteria from the PRD.\n\
         - Document required env vars / secrets in `output/{slug}-architecture.md`.\n\n\
         ## Required output\n\n\
         - **Code:** in your project's backend directory\n\
         - **Notes:** `output/{slug}-backend-notes.md` with a checklist showing each audited URL → handler.\n\n\
         ## Input\n\n\
         ### Requirement\n\n{req}\n\n\
         Read `output/{slug}-architecture.md` and `.super-dev/audit/frontend-api-calls.jsonl`.\n",
    )
}

fn render_quality(slug: &str) -> String {
    format!(
        "## Role\n\nQuality lead.\n\n\
         ## Task\n\nSuper Dev runs the quality gate automatically — `super-dev continue` from the backend phase invokes the deterministic scorer. Your job here is to:\n\n\
         - Review `output/{slug}-quality-gate.md` and `output/{slug}-quality-gate.json`.\n\
         - Open any item the gate flagged with `failed` or `warning` and fix it in code.\n\
         - Re-run the affected phase if necessary.\n\n\
         ## Required output\n\n\
         No new artifact — the gate report itself is the artifact. Make sure `quality_gate_passed: true` before advancing to delivery.\n",
    )
}

fn render_delivery(slug: &str) -> String {
    format!(
        "## Role\n\nRelease engineer.\n\n\
         ## Task\n\n`super-dev continue` from the quality phase already:\n\n\
         - Wrote `output/{slug}-compliance-mapping.json` (SOC 2 / ISO 27001 / EU AI Act).\n\
         - Bundled `release/proof-pack-{slug}-<ts>.zip` containing every artifact and audit log.\n\n\
         ## What to do now\n\n\
         - Inspect the proof pack: `unzip -l release/proof-pack-{slug}-*.zip`.\n\
         - Hand the proof pack to the reviewer / compliance officer.\n\
         - Run `super-dev report` later if you need to regenerate the compliance mapping.\n",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    fn opts(root: &Path) -> RunOptions {
        RunOptions {
            project_root: root.to_path_buf(),
            requirement: "build a login system".into(),
            slug: "demo".into(),
            model: "stub".into(),
        }
    }

    #[test]
    fn writes_coach_file_per_phase() {
        let tmp = TempDir::new().unwrap();
        for phase in [
            Phase::Research,
            Phase::Docs,
            Phase::DocsConfirm,
            Phase::Spec,
            Phase::Frontend,
            Phase::PreviewConfirm,
            Phase::Backend,
            Phase::Quality,
            Phase::Delivery,
        ] {
            let path = write_coach_prompt(&opts(tmp.path()), phase).unwrap();
            assert!(path.is_file(), "missing coach file for {phase:?}");
            let body = fs::read_to_string(&path).unwrap();
            assert!(body.contains("SUPER_DEV_HOST_SPEC_V1"));
            assert!(body.contains(phase.id()));
        }
        // CURRENT.md is always the latest
        assert!(tmp.path().join(".super-dev/coach/CURRENT.md").is_file());
    }

    #[test]
    fn research_prompt_carries_requirement_and_paths() {
        let body = render_coach_prompt(&opts(Path::new("/tmp")), Phase::Research);
        assert!(body.contains("output/demo-research.md"));
        assert!(body.contains("build a login system"));
        assert!(body.contains("Similar products"));
    }

    #[test]
    fn docs_prompt_demands_three_artifacts() {
        let body = render_coach_prompt(&opts(Path::new("/tmp")), Phase::Docs);
        assert!(body.contains("output/demo-prd.md"));
        assert!(body.contains("output/demo-architecture.md"));
        assert!(body.contains("output/demo-uiux.md"));
    }

    #[test]
    fn gate_prompts_tell_host_to_wait() {
        let body = render_coach_prompt(&opts(Path::new("/tmp")), Phase::DocsConfirm);
        assert!(body.to_lowercase().contains("wait"));
        assert!(body.contains("Do NOT advance"));
    }

    #[test]
    fn frontend_prompt_locks_hard_rules() {
        let body = render_coach_prompt(&opts(Path::new("/tmp")), Phase::Frontend);
        assert!(body.contains("Lucide"));
        assert!(body.contains("design tokens"));
        assert!(body.contains("frontend-api-calls.jsonl") || body.contains("architecture"));
    }

    #[test]
    fn coach_filename_is_zero_padded_phase_ordered() {
        assert_eq!(coach_filename(Phase::Research), "01-research.md");
        assert_eq!(coach_filename(Phase::Delivery), "09-delivery.md");
    }
}
