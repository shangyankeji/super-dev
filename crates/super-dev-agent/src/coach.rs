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
    let design_inject = load_design_system_inject(opts);
    let expert_knowledge = crate::phases::phase_knowledge_digest(opts, phase);
    let body = match phase {
        Phase::Research => render_research(&slug, req, opts),
        Phase::Docs => render_docs(&slug, req, &design_inject),
        Phase::DocsConfirm | Phase::PreviewConfirm => render_gate(phase, &slug),
        Phase::Spec => render_spec(&slug, req),
        Phase::Frontend => render_frontend(&slug, req, &design_inject),
        Phase::Backend => render_backend(&slug, req),
        Phase::Quality => render_quality(&slug),
        Phase::Delivery => render_delivery(&slug),
    };
    format!(
        "# Super Dev coach — phase `{}`\n\n\
         > Read this file and produce the required output. Then run \
         `super-dev continue` to advance.\n\n\
         {preamble}\n\n\
         {body}\n\
         {expert_knowledge}\n",
        phase.id()
    )
}

fn spec_preamble() -> String {
    "\
## Spec preamble (non-negotiable)\n\n\
You are operating inside a Super Dev pipeline run governed by \
`SUPER_DEV_HOST_SPEC_V1`. Every artifact you produce MUST follow these \
rules:\n\n\
### Technical rules\n\
- Use a declared icon library (Lucide / Heroicons / Tabler). NEVER \
  emoji as functional icons.\n\
- Use design tokens (CSS vars / theme keys). NEVER hardcoded hex / \
  rgb / hsl colors in production UI.\n\
- Frontend `fetch` URLs MUST match an API path declared in the \
  architecture document.\n\
- Wait for explicit user approval at `docs_confirm` and \
  `preview_confirm` gates.\n\
- Output goes into structured markdown sections — do not invent new \
  top-level sections, fill the ones requested below.\n\n\
### Visual quality rules\n\
- Typography drives hierarchy: set a type scale before touching layout.\n\
- Generous whitespace: spacing tokens, not cramped divs.\n\
- Real-looking placeholder content, not \"Lorem ipsum\" or \"Example\".\n\
- Every interactive element needs hover + focus + disabled states.\n\
- No purple/pink gradient hero sections unless the product domain demands it.\n\
- No default-system-font-only designs; always declare a font stack.\n\
- No \"Welcome to [App]\" giant centered headings with no actual content.\n\
- No AI-chatbot shell layouts unless the product IS a chatbot.\n\
- Dark mode support via `prefers-color-scheme` when the UIUX doc defines dark tokens.\
"
    .to_string()
}

fn render_research(slug: &str, req: &str, opts: &RunOptions) -> String {
    let knowledge = crate::phases::knowledge_digest(opts);
    format!(
        "## Role\n\nSenior product researcher + design strategist.\n\n\
         ## Task\n\nProduce a research brief that grounds PM / architect / UI work. \
         This is the FOUNDATION — every later phase reads this document.\n\n\
         ## Required output\n\n\
         - **Sections (in order):**\n\
           - `# Research — {slug}`\n\
           - `## Requirement` (echo verbatim)\n\
           - `## Discovery` — answer these design-grounding questions:\n\
             - **Target audience**: who uses this product? (developers / consumers / enterprise / internal team)\n\
             - **Visual tone**: which fits best? (professional / playful / technical / editorial / bold)\n\
             - **Design direction**: pick ONE from: Modern Minimal / Editorial Clean / Tech Utility / Soft Warm / Bold Geometric\n\
             - **Brand constraints**: any existing colors / fonts / logos to respect? If none, state \"greenfield — design from scratch\"\n\
             - **Platform**: web / mobile / desktop / CLI companion\n\
             - **Complexity**: simple (1-3 screens) / medium (4-8 screens) / complex (9+ screens)\n\
           - `## Similar products` — cite 5 real products with one-line design takeaways (how they look, not just what they do)\n\
           - `## Domain risks` — 5 risks the architecture must mitigate\n\
           - `## UI / UX must-haves` — 5 non-negotiable UI patterns in this domain, with concrete implementation notes\n\
           - `## Design system recommendation` — based on the discovery answers, recommend:\n\
             - Color palette direction (warm / cool / neutral / high-contrast)\n\
             - Typography approach (serif accent / geometric sans / monospace accent / humanist)\n\
             - Spacing philosophy (airy / compact / mixed)\n\
             - One \"signature detail\" that makes this product visually distinct from competitors\n\
           - `## Open questions`\n\n\
         ## Input\n\n\
         ### Requirement\n\n{req}\n\n\
         ### Local knowledge files available\n\n{knowledge}\n\n\
         ## CRITICAL output instruction\n\n\
         **Print the FULL research brief as your text reply.** \
         Do NOT use Edit / Write tools to create the file — Super Dev \
         captures your stdout and writes the file itself.\n",
    )
}

/// Load the active design system markdown + seed template from the
/// knowledge directory. Returns a ready-to-inject block for the coach
/// prompt (may be empty if nothing is configured).
fn load_design_system_inject(opts: &RunOptions) -> String {
    let mut inject = String::new();
    if !opts.design_system.is_empty() {
        let path = opts
            .project_root
            .join("knowledge/design-systems")
            .join(format!("{}.md", opts.design_system));
        if let Ok(content) = fs::read_to_string(&path) {
            inject.push_str("\n\n## Active design system (binding contract)\n\n");
            inject.push_str("The user selected this design system via `/design ");
            inject.push_str(&opts.design_system);
            inject.push_str(
                "`. Use its tokens, fonts, and component patterns as \
                             your BINDING CONTRACT — copy the CSS `:root` verbatim.\n\n",
            );
            inject.push_str(&content);
        }
    }
    if !opts.seed_template.is_empty() {
        let path = opts
            .project_root
            .join("knowledge/seed-templates")
            .join(format!("{}.md", opts.seed_template));
        if let Ok(content) = fs::read_to_string(&path) {
            inject.push_str("\n\n## Active seed template\n\n");
            inject.push_str("The user selected this template via `/template ");
            inject.push_str(&opts.seed_template);
            inject.push_str("`. Follow its page structure and quality gates.\n\n");
            inject.push_str(&content);
        }
    }
    inject
}

fn render_docs(slug: &str, req: &str, design_inject: &str) -> String {
    format!(
        "## Role\n\nThree domain experts execute in sequence. Each reads the prior \
         expert's output before writing. Think of this as a cross-functional \
         design review where each expert brings their professional standards:\n\n\
         1. **Senior Product Manager** — 10+ years in B2B/B2C SaaS. Writes PRDs that \
            engineers can implement without ambiguity. Every acceptance criterion is testable.\n\
         2. **Senior Software Architect** — systems-level thinker. API surfaces are \
            consistent, data models are normalized, tech choices are justified by constraints.\n\
         3. **Senior UI/UX Designer** — creates design systems, not mockups. Tokens, type \
            scales, component inventories with state matrices, accessibility baked in.\n\n\
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
         ### 3. UI/UX — `output/{slug}-uiux.md`\n\n\
         **IMPORTANT**: Before writing, check if `knowledge/design-systems/` exists in \
         the workspace. If it does, read the design system that matches the direction \
         chosen in the research brief's Discovery section (e.g. if \"Modern Minimal\" → \
         read `knowledge/design-systems/modern-minimal.md`). Use its color palette, \
         typography, spacing, and component patterns as your BINDING CONTRACT — copy \
         the CSS `:root` tokens verbatim, don't reinvent them.\n\n\
         If no matching design system file exists, create tokens from scratch following \
         the direction chosen below.\n\n\
         - `# UI/UX — {slug}`\n\
         - `## Visual direction` — pick ONE direction that fits the product domain. \
           Consider these archetypes:\n\
           - **Editorial Clean** — magazine-like, serif-accent headings, generous whitespace, \
             photography-driven. Best for: content sites, blogs, portfolio.\n\
           - **Modern Minimal** — geometric sans-serif, precise spacing, monochrome with one accent. \
             Best for: SaaS, dev tools, dashboards.\n\
           - **Tech Utility** — monospace accents, dense information, dark-mode-native. \
             Best for: CLI companions, code platforms, data tools.\n\
           - **Soft Warm** — rounded corners, warm palette, friendly illustrations. \
             Best for: consumer apps, education, wellness.\n\
           - **Bold Geometric** — high contrast, oversized type, asymmetric grid. \
             Best for: creative agencies, portfolios, brand launches.\n\
           State which you chose and ONE sentence why it fits this product. \
           Then define ALL tokens deterministically from that choice — the frontend \
           phase COPIES these tokens, it does not reinvent them.\n\
         - `## Color palette` — a `:root` CSS block with semantic tokens. \
           Require at minimum: `--color-bg`, `--color-surface`, `--color-text`, \
           `--color-text-secondary`, `--color-primary`, `--color-primary-hover`, \
           `--color-accent`, `--color-border`, `--color-error`, `--color-success`. \
           Must include dark-mode overrides via `@media (prefers-color-scheme: dark)`. \
           NO purple/pink gradients unless the product domain demands it.\n\
         - `## Typography system` — font stack (2 families max: one for headings, \
           one for body), type scale (7 steps: `--text-xs` through `--text-3xl`), \
           line-height tokens, font-weight tokens. NO system-font-only.\n\
         - `## Spacing scale` — mathematical progression (4px base), at least 8 \
           steps from `--space-1` (4px) to `--space-12` (48px).\n\
         - `## Icon library` — declare exactly ONE: Lucide / Heroicons / Tabler.\n\
         - `## Page hierarchy` — nested list with route paths.\n\
         - `## Component inventory` — list every component the frontend needs, with \
           states: default / hover / active / disabled / loading / error.\n\
         - `## Motion guidelines` — transition durations + easing functions as tokens \
           (`--transition-fast: 150ms ease-out`).\n\
         - `## Anti-patterns` — 5 things this design explicitly avoids \
           (e.g. \"no decorative hero gradients\", \"no AI-chat-shell layout\", \
           \"no emoji as functional icons\").\n\
         - `## Self-critique` — score this design on 5 dimensions (1-10 each): \
           Hierarchy clarity / Visual distinctiveness / Detail polish / \
           Functional completeness / Innovation. If any ≤ 6, revise before submitting.\n\
         - `## Accessibility notes` — contrast ratios, focus rings, ARIA landmarks.\n\n\
         ## Input\n\n\
         ### Requirement\n\n{req}\n\n\
         ### Research brief\n\n\
         Read `output/{slug}-research.md` for context.\n\n\
         ## CRITICAL output instruction\n\n\
         **Print the FULL content of each document as your text reply.** \
         Do NOT use Edit / Write / Create tools to write the files — \
         Super Dev's pipeline captures your stdout and writes the files \
         itself. If you write the files AND return a summary, the \
         pipeline will overwrite your files with the summary, losing \
         the real content.\n\n\
         For each document, output the complete markdown body starting \
         with the `#` heading. Separate the three documents with \
         `---` on its own line.\n\n\
         ## After you finish\n\n\
         Run `super-dev continue` and approve the `docs_confirm` gate only after the user has reviewed.\n\
         {design_inject}",
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
        "## Role\n\nSenior delivery lead / engineering manager — translates approved \
         documents into executable work breakdown. Every task is scoped to < 4 hours \
         and has a clear done-condition.\n\n\
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

fn render_frontend(slug: &str, req: &str, design_inject: &str) -> String {
    format!(
        "## Role\n\nSenior frontend engineer + visual craftsperson.\n\n\
         ## Task\n\nImplement a production-grade frontend that looks \
         professionally designed — not like an AI template.\n\n\
         ## Design quality contract\n\n\
         Before writing ANY component, read `output/{slug}-uiux.md` and \
         lock these into your code:\n\
         1. **Typography first** — set the type scale in `:root` before \
            touching layout. Headings drive visual hierarchy.\n\
         2. **Whitespace is a feature** — generous padding/margin using \
            the spacing scale. No cramped layouts.\n\
         3. **Color from tokens only** — bind every `color`, \
            `background`, `border-color`, `box-shadow` to a CSS var. \
            Zero hardcoded hex/rgb/hsl.\n\
         4. **Real content** — use realistic placeholder text, not \
            \"Lorem ipsum\" or \"Example\". Names, dates, numbers that \
            feel authentic.\n\
         5. **Component states** — every interactive element must have \
            hover, active, focus, disabled styles. No state-less buttons.\n\
         6. **Dark mode** — if the UIUX doc defines dark tokens, wire \
            them via `prefers-color-scheme`.\n\
         7. **Responsive** — at minimum 2 breakpoints (mobile 360px, \
            desktop 1024px). Test both.\n\
         8. **Motion** — use the transition tokens from UIUX. Subtle \
            enter/exit animations on cards and modals.\n\n\
         ## Hard rules (will be audited)\n\n\
         - Icons from the declared library only. Zero emoji as icons.\n\
         - Every `fetch` URL MUST appear in `output/{slug}-architecture.md`.\n\
         - Run `npm run build` / `cargo check` — zero errors before submitting.\n\
         - Take a screenshot of the running app for the preview gate.\n\n\
         ## Anti-patterns to avoid (P0 cardinal sins)\n\n\
         - Purple/pink gradient hero sections\n\
         - Lorem ipsum / filler text\n\
         - \"Welcome to App\" generic headings\n\
         - Invented metrics without a source (\"10x faster\")\n\
         - Emoji used as functional icons\n\
         - Cards with identical placeholder text repeated 3x\n\
         - \"AI chatbot\" shell layout when the product is not a chatbot\n\
         - More than 2 accent-colored elements per viewport\n\
         - 3+ sections with identical layout (alternate the rhythm)\n\n\
         For the full P0/P1/P2 checklist, read \
         `knowledge/design-systems/00-craft-rules.md` if it exists.\n\n\
         Also check `knowledge/seed-templates/` for a matching page type \
         (saas-landing, dashboard, blog-content) — the seed template gives \
         you the section order and component patterns to follow.\n\n\
         ## Required output\n\n\
         - **Code:** in your project's frontend directory\n\
         - **Notes:** `output/{slug}-frontend-notes.md` — audit checklist of \
           the design quality contract items above (checked/unchecked).\n\n\
         ## Input\n\n\
         ### Requirement\n\n{req}\n\n\
         Read these in order (each informs the next):\n\
         1. `output/{slug}-uiux.md` — your PRIMARY visual guide. Every CSS var, \
            font choice, spacing value, and component pattern comes from here.\n\
         2. `output/{slug}-architecture.md` — API surface and tech stack.\n\
         3. `output/{slug}-prd.md` — acceptance criteria and user stories.\n\
         4. If `knowledge/design-systems/*.md` exists and the UIUX doc references \
            a design direction, read the matching file for detailed Do/Don't rules \
            and component patterns.\n\n\
         ## After you finish\n\n\
         Run the app, take a screenshot, ask the user to review. \
         They will approve via `super-dev continue`.\n\
         {design_inject}",
    )
}

fn render_backend(slug: &str, req: &str) -> String {
    format!(
        "## Role\n\nSenior backend engineer — builds secure, tested APIs. Every \
         handler has input validation, error responses, and a matching test. \
         Secrets go in env vars, never in code.\n\n\
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
        "## Role\n\nQuality lead + design critic.\n\n\
         ## Task\n\nTwo-part quality check:\n\n\
         ### Part 1 — Automated gate\n\n\
         Super Dev runs the quality gate automatically. Review \
         `output/{slug}-quality-gate.md` and `output/{slug}-quality-gate.json`. \
         Fix anything flagged `failed` or `warning`.\n\n\
         ### Part 2 — Visual design critique (5 dimensions)\n\n\
         Open the running frontend and score it 1-10 on each dimension:\n\n\
         | Dimension | What to evaluate | Minimum |\n\
         |---|---|---|\n\
         | **Hierarchy** | Can a new user find the primary action in < 2 seconds? | 7 |\n\
         | **Distinctiveness** | Does it look like a custom design, not a template? | 7 |\n\
         | **Detail** | Hover states, focus rings, loading skeletons, empty states — all present? | 7 |\n\
         | **Function** | Every button works, every link navigates, every form validates? | 8 |\n\
         | **Polish** | Spacing consistent, type scale respected, dark mode works? | 7 |\n\n\
         If ANY dimension < minimum, go back to the frontend phase and fix it. \
         Append your scores to `output/{slug}-quality-gate.md`.\n\n\
         ## Required output\n\n\
         No new artifact — the gate report + your 5-dimension scores. Make sure \
         `quality_gate_passed: true` and all 5 dimensions ≥ minimum before delivery.\n",
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
            backend: String::new(),
            design_system: String::new(),
            seed_template: String::new(),
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
    fn research_prompt_carries_requirement_and_discovery() {
        let body = render_coach_prompt(&opts(Path::new("/tmp")), Phase::Research);
        assert!(body.contains("build a login system"));
        assert!(body.contains("Similar products"));
        assert!(body.contains("Discovery"));
        assert!(body.contains("Target audience"));
        assert!(body.contains("Design direction"));
        assert!(body.contains("CRITICAL output instruction"));
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
