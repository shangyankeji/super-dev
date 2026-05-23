/**
 * 开发：Excellent（11964948@qq.com）
 * 功能：全局静态数据常量
 * 作用：宿主列表、站点统计与演示数据
 * 创建时间：2026-03-08
 * 最后修改：2026-03-08
 */

export type HostStatus = 'certified' | 'compatible' | 'experimental';
export type HostTriggerMode = 'slash' | 'text';
export type HostCategory = 'cli' | 'ide' | 'assistant';

export interface Host {
  name: string;
  category: HostCategory;
  integration: HostStatus;
  runtime: HostStatus;
  abbr: string;
  trigger: HostTriggerMode;
  protocol: string;
  firstPrompt: string;
  bestFor: string;
  triggerLabel?: string;
}

export const HOSTS: Host[] = [
  { name: 'Claude Code', category: 'cli', abbr: 'C', integration: 'certified', runtime: 'compatible', trigger: 'slash', protocol: 'CLAUDE.md + settings + project/user skills + subagents', firstPrompt: '/super-dev 你的需求', bestFor: '最快进入 research 和三文档的旗舰 CLI 宿主' },
  { name: 'CodeBuddy CLI', category: 'cli', abbr: 'C', integration: 'compatible', runtime: 'experimental', trigger: 'slash', protocol: 'CODEBUDDY.md + rules + commands + skills + agents', firstPrompt: '/super-dev 你的需求', bestFor: '已经在 CodeBuddy CLI 里做长流程任务协作' },
  { name: 'Copilot CLI', category: 'cli', abbr: 'C', integration: 'experimental', runtime: 'experimental', trigger: 'text', protocol: 'copilot-instructions + AGENTS.md + skills + agents', firstPrompt: 'super-dev: 你的需求', bestFor: '规则和 instructions 驱动的文本入口' },
  { name: 'Codex CLI', category: 'cli', abbr: 'C', integration: 'certified', runtime: 'compatible', trigger: 'text', triggerLabel: 'CLI: $super-dev', protocol: 'AGENTS.md + official skills + CLI $skill entry', firstPrompt: '$super-dev', bestFor: 'Codex CLI 主战场，显式 skill 入口最稳' },
  { name: 'Cursor CLI', category: 'cli', abbr: 'C', integration: 'compatible', runtime: 'experimental', trigger: 'text', protocol: 'AGENTS.md + .cursor/rules + native resume', firstPrompt: 'super-dev: 你的需求', bestFor: 'Cursor CLI 当前目录直接续流程，不靠项目 slash' },
  { name: 'Droid CLI', category: 'cli', abbr: 'D', integration: 'certified', runtime: 'compatible', trigger: 'slash', protocol: 'AGENTS.md + .factory/rules + .factory/skills (+ commands compatibility)', firstPrompt: '/super-dev 你的需求', bestFor: 'Factory 会话里最快进入标准流或 SEEAI' },
  { name: 'Gemini CLI', category: 'cli', abbr: 'G', integration: 'compatible', runtime: 'experimental', trigger: 'slash', protocol: 'GEMINI.md + settings + TOML commands', firstPrompt: '/super-dev 你的需求', bestFor: 'GEMINI.md 和 custom commands 驱动的 slash 宿主' },
  { name: 'Kimi Code', category: 'cli', abbr: 'K', integration: 'compatible', runtime: 'compatible', trigger: 'text', triggerLabel: 'super-dev: · /skill:super-dev · /flow:super-dev', protocol: 'AGENTS.md + explicit /skill:/flow entries + native resume (+ skills enhancement)', firstPrompt: 'super-dev: 你的需求', bestFor: '中文长流程项目，显式 /skill:/flow 和原生继续链' },
  { name: 'Kiro CLI', category: 'cli', abbr: 'K', integration: 'compatible', runtime: 'experimental', trigger: 'slash', triggerLabel: '/super-dev · super-dev:', protocol: 'AGENTS.md + steering + skills + native resume', firstPrompt: '/super-dev 你的需求', bestFor: 'steering + skills 一起约束的 CLI 宿主' },
  { name: 'OpenCode', category: 'cli', abbr: 'O', integration: 'experimental', runtime: 'experimental', trigger: 'slash', protocol: 'AGENTS.md + commands + skills (+ agents enhancement)', firstPrompt: '/super-dev 你的需求', bestFor: 'commands/skills 驱动的原生 slash 宿主' },
  { name: 'Qoder CLI', category: 'cli', abbr: 'Q', integration: 'compatible', runtime: 'experimental', trigger: 'slash', protocol: 'AGENTS.md + rules + commands + skills (+ agents enhancement)', firstPrompt: '/super-dev 你的需求', bestFor: 'rules + commands + skills 同时工作的 Qoder CLI' },
  { name: 'Qwen Code', category: 'cli', abbr: 'Q', integration: 'compatible', runtime: 'experimental', trigger: 'slash', protocol: 'QWEN.md + commands + skills + checkpoint/restore', firstPrompt: '/super-dev 你的需求', bestFor: 'QWEN.md + commands 模型，适合先研究再进实现' },
  { name: 'Antigravity', category: 'ide', abbr: 'A', integration: 'experimental', runtime: 'compatible', trigger: 'slash', protocol: 'recommended GEMINI.md + custom commands (+ workflows enhancement)', firstPrompt: '/super-dev 你的需求', bestFor: '偏实验型 IDE，适合验证 Gemini-style project context' },
  { name: 'CodeBuddy', category: 'ide', abbr: 'C', integration: 'compatible', runtime: 'compatible', trigger: 'slash', protocol: 'CODEBUDDY.md + rules + skills + workspace continuity', firstPrompt: '/super-dev 你的需求', bestFor: '工作区连续性强，适合团队项目迭代' },
  { name: 'CodeBuddyCN', category: 'ide', abbr: 'C', integration: 'compatible', runtime: 'compatible', trigger: 'slash', protocol: 'CODEBUDDY.md + rules + skills + workspace continuity', firstPrompt: '/super-dev 你的需求', bestFor: '中文工作区协作与连续返工' },
  { name: 'Cursor', category: 'ide', abbr: 'C', integration: 'experimental', runtime: 'experimental', trigger: 'slash', protocol: 'Agent Chat + AGENTS.md + rules (+ beta commands)', firstPrompt: '/super-dev 你的需求', bestFor: 'Agent Chat 里直接做 UI 和实现返工' },
  { name: 'Kiro', category: 'ide', abbr: 'K', integration: 'experimental', runtime: 'experimental', trigger: 'slash', protocol: 'AGENTS.md + steering + skills + agent continuity', firstPrompt: '/super-dev 你的需求', bestFor: 'Kiro IDE 里保持 steering 和 agent continuity' },
  { name: 'Qoder', category: 'ide', abbr: 'Q', integration: 'experimental', runtime: 'experimental', trigger: 'slash', protocol: 'AGENTS.md + rules + commands + skills (+ agents enhancement)', firstPrompt: '/super-dev 你的需求', bestFor: 'IDE 里 rules/commands/skills 一起拉齐 Qoder 工作区' },
  { name: 'Trae', category: 'ide', abbr: 'T', integration: 'compatible', runtime: 'experimental', trigger: 'text', protocol: 'recommended project context + compatibility rules + optional skills', firstPrompt: 'super-dev: 你的需求', bestFor: '先用项目上下文稳定接管旧项目工作区' },
  { name: 'TraeCN', category: 'ide', abbr: 'T', integration: 'compatible', runtime: 'compatible', trigger: 'text', protocol: 'recommended CN workspace skills + /plan /spec model', firstPrompt: 'super-dev: 你的需求', bestFor: '中文工作区 + /plan /spec 组合的文本入口' },
  { name: 'Windsurf', category: 'ide', abbr: 'W', integration: 'experimental', runtime: 'experimental', trigger: 'slash', protocol: 'AGENTS.md + workflows + skills', firstPrompt: '/super-dev 你的需求', bestFor: 'workflow 型 IDE 宿主，适合长链任务编排' },
  { name: 'Claude', category: 'assistant', abbr: 'C', integration: 'compatible', runtime: 'compatible', trigger: 'text', triggerLabel: 'super-dev:', protocol: 'Projects + project instructions + project knowledge + desktop extensions/MCP', firstPrompt: 'super-dev: 你的需求', bestFor: 'Project / Knowledge / MCP 驱动的桌面助手' },
  { name: 'Codex', category: 'assistant', abbr: 'C', integration: 'certified', runtime: 'compatible', trigger: 'text', triggerLabel: 'App/Desktop: / → super-dev', protocol: 'AGENTS.md + skills + enabled App/Desktop skill entry', firstPrompt: '从 / 列表选择 super-dev', bestFor: 'Codex App/Desktop 里最完整的旗舰宿主体验' },
  { name: 'Trae SOLO', category: 'assistant', abbr: 'T', integration: 'compatible', runtime: 'compatible', trigger: 'slash', protocol: 'recommended workspace rules + optional skills model', firstPrompt: '/super-dev 你的需求', bestFor: 'SOLO 工作区里快速接入并继续当前流程' },
  { name: 'Trae SOLOCN', category: 'assistant', abbr: 'T', integration: 'compatible', runtime: 'compatible', trigger: 'text', protocol: 'recommended CN workspace MTC / Code + skills model', firstPrompt: 'super-dev: 你的需求', bestFor: '中文 SOLO 工作区，适合比赛和短平快交付' },
  { name: 'WorkBuddy', category: 'assistant', abbr: 'W', integration: 'compatible', runtime: 'compatible', trigger: 'text', protocol: 'recommended task workbench + skills + MCP model', firstPrompt: 'super-dev: 你的需求', bestFor: '任务工作台驱动的桌面助手连续工作流' },
];

export const SLASH_HOSTS = HOSTS.filter((host) => host.trigger === 'slash');
export const TEXT_TRIGGER_HOSTS = HOSTS.filter((host) => host.trigger === 'text');
export const CLI_HOSTS = HOSTS.filter((host) => host.category === 'cli');
export const IDE_HOSTS = HOSTS.filter((host) => host.category === 'ide');
export const DESKTOP_ASSISTANT_HOSTS = HOSTS.filter((host) => host.category === 'assistant');
export const HOST_MATRIX_GROUPS = [
  { key: 'cli', title: 'CLI', label: 'CLI 12', hosts: CLI_HOSTS },
  { key: 'ide', title: 'IDE', label: 'IDE 9', hosts: IDE_HOSTS },
  { key: 'assistant', title: 'Desktop assistants', label: 'Desktop assistants 5', hosts: DESKTOP_ASSISTANT_HOSTS },
] as const;

export const STATS = {
  zh: [
    { value: '统一矩阵', label: '宿主接入' },
    { value: '5 分钟', label: '安装到宿主首句' },
    { value: '9', label: '治理阶段' },
    { value: '2', label: '强制确认门' },
  ],
  en: [
    { value: 'Unified', label: 'Host matrix' },
    { value: '5 min', label: 'Install to first prompt' },
    { value: '9', label: 'Governed stages' },
    { value: '2', label: 'Required approval gates' },
  ],
} as const;

export const TERMINAL_LINES = {
  zh: [
    { type: 'input', text: 'uv tool install super-dev' },
    { type: 'output', text: 'Collecting super-dev' },
    { type: 'output', text: 'Installing collected packages: super-dev' },
    { type: 'success', text: 'Successfully installed super-dev-2.4.1' },
    { type: 'blank', text: '' },
    { type: 'input', text: 'super-dev' },
    { type: 'brand', text: 'Super Dev 安装器 — 终端只负责接入与升级' },
    { type: 'info', text: 'Detected hosts: Claude, Codex, Codex CLI, Cursor CLI' },
    { type: 'info', text: 'Recommended host: Codex' },
    { type: 'info', text: 'Writing project-first protocol surfaces' },
    { type: 'info', text: 'Current project focus: Next.js · preview the real page before backend-heavy work' },
    { type: 'info', text: 'UI visual gate: screenshot-level review blocks flat template-like pages' },
    { type: 'info', text: 'Copy the first prompt → return to the host → start with research' },
    { type: 'blank', text: '' },
    { type: 'output', text: 'Next in host: /super-dev 或 super-dev:' },
    { type: 'output', text: 'First prompt: /super-dev 你的需求' },
    { type: 'blank', text: '' },
    { type: 'input', text: 'super-dev update' },
    { type: 'info', text: 'Upgrading Super Dev and migrating onboarded hosts' },
    { type: 'success', text: 'Upgrade complete. Reopen your host and continue there.' },
  ],
  en: [
    { type: 'input', text: 'uv tool install super-dev' },
    { type: 'output', text: 'Collecting super-dev' },
    { type: 'output', text: 'Installing collected packages: super-dev' },
    { type: 'success', text: 'Successfully installed super-dev-2.4.1' },
    { type: 'blank', text: '' },
    { type: 'input', text: 'super-dev' },
    { type: 'brand', text: 'Super Dev installer — terminal only handles onboarding and upgrade' },
    { type: 'info', text: 'Detected hosts: Claude, Codex, Codex CLI, Cursor CLI' },
    { type: 'info', text: 'Recommended host: Codex' },
    { type: 'info', text: 'Writing project-first protocol surfaces' },
    { type: 'info', text: 'Current project focus: Next.js · preview the real page before backend-heavy work' },
    { type: 'info', text: 'UI visual gate: screenshot-level review blocks flat template-like pages' },
    { type: 'info', text: 'Copy the first prompt → return to the host → start with research' },
    { type: 'blank', text: '' },
    { type: 'output', text: 'Next in host: /super-dev or super-dev:' },
    { type: 'output', text: 'First prompt: /super-dev your goal' },
    { type: 'blank', text: '' },
    { type: 'input', text: 'super-dev update' },
    { type: 'info', text: 'Upgrading Super Dev and migrating onboarded hosts' },
    { type: 'success', text: 'Upgrade complete. Reopen your host and continue there.' },
  ],
} as const;
