import Link from 'next/link';
import type { LucideIcon } from 'lucide-react';
import {
  ArrowRight,
  BookOpen,
  Boxes,
  Command,
  FolderTree,
  LifeBuoy,
  Package,
  RefreshCw,
  Search,
  Sparkles,
  Terminal,
  Workflow,
  Zap,
} from 'lucide-react';
import { Badge } from '@/components/ui/Badge';
import { CodeBlock } from '@/components/ui/CodeBlock';
import { CopyCommand } from '@/components/ui/CopyCommand';
import { SLASH_HOSTS, TEXT_TRIGGER_HOSTS } from '@/lib/constants';
import { localizedPath, type SiteLocale } from '@/lib/site-locale';

const SLASH_HOST_NAMES = SLASH_HOSTS.map((host) => host.name);
const TEXT_TRIGGER_HOST_NAMES = TEXT_TRIGGER_HOSTS.map((host) => host.name);

type SectionLink = { id: string; label: string; icon: LucideIcon };
type SurfaceGroup = { title: string; body: string; points: string[] };
type TriggerCard = { title: string; command: string; hosts: readonly string[]; note: string; variant: 'certified' | 'compatible' };
type PipelineStep = { step: string; title: string; body: string };
type Content = {
  heroKicker: string;
  heroTitle: string;
  heroBody: string;
  heroStats: { label: string; value: string }[];
  sections: SectionLink[];
  governanceTitle: string;
  governanceBody: string;
  governanceCards: { title: string; body: string }[];
  installTitle: string;
  installBody: string;
  installBullets: string[];
  installCode: string;
  firstMinutesTitle: string;
  firstMinutesBody: string;
  firstMinutesCards: { title: string; body: string; bullets: string[] }[];
  surfacesTitle: string;
  surfacesBody: string;
  surfaces: SurfaceGroup[];
  triggerTitle: string;
  triggerBody: string;
  triggerCards: TriggerCard[];
  matrixTitle: string;
  matrixBody: string;
  matrixGroups: { category: string; items: { host: string; protocol: string; grade: string; trigger: string }[] }[];
  pipelineTitle: string;
  pipelineBody: string;
  pipelineSteps: PipelineStep[];
  controlTitle: string;
  controlBody: string;
  controlCards: { title: string; body: string; code: string }[];
  operationsTitle: string;
  operationsBody: string;
  operationsCards: { title: string; body: string; bullets: string[] }[];
  commandsTitle: string;
  commandsBody: string;
  commands: { title: string; code: string; filename: string }[];
  troubleshootingTitle: string;
  troubleshootingBody: string;
  troubleshootingSteps: string[];
  smokeTitle: string;
  smokeCode: string;
  highlightsTitle: string;
  highlightsBody: string;
  highlightsCards: { title: string; body: string }[];
};

function gradeVariant(grade: string) {
  if (grade === 'Certified') return 'certified';
  return 'compatible';
}

function gradeLabel(grade: string, locale: SiteLocale) {
  if (locale === 'en') {
    return grade === 'Certified' ? 'Recommended' : 'Compatible';
  }
  return grade === 'Certified' ? '推荐' : '兼容';
}

const zhContent: Content = {
  heroKicker: 'Documentation Center',
  heroTitle: '先装上，再回宿主开始真正的开发。',
  heroBody:
    '这页不是协议字典，而是把你从安装带到“宿主第一句”和“安装后 5 分钟”的公开主路径。真正重要的是：装完就回宿主，第一轮先 research，再写三文档并等待确认。',
  heroStats: [
    { label: '宿主矩阵', value: '统一' },
    { label: '安装到首句', value: '5 分钟' },
    { label: '核心阶段', value: '9 段' },
    { label: '确认门', value: '2 个' },
  ],
  sections: [
    { id: 'highlights', label: 'v2.4.1 当前修复重点', icon: Zap },
    { id: 'governance', label: '核心路径', icon: BookOpen },
    { id: 'install', label: '安装方式', icon: Package },
    { id: 'first-minutes', label: '安装后 5 分钟', icon: Sparkles },
    { id: 'surfaces', label: '安装器会写什么', icon: Boxes },
    { id: 'triggers', label: '宿主第一句', icon: Command },
    { id: 'hosts', label: '宿主矩阵', icon: Terminal },
    { id: 'pipeline', label: '流水线', icon: Workflow },
    { id: 'control', label: '宿主里怎么继续', icon: RefreshCw },
    { id: 'operations', label: '研究、预览与交付门', icon: FolderTree },
    { id: 'commands', label: '只需要记住的入口', icon: Package },
    { id: 'troubleshooting', label: '排障', icon: LifeBuoy },
  ],
  governanceTitle: '真正的开发，从回到宿主开始。',
  governanceBody:
    '终端只做安装、升级和清理。真正的第一轮发生在宿主里：research、三文档、确认门、Spec、实现、预览验证、质量门和交付都沿着同一条主路径推进。',
  governanceCards: [
    { title: '终端只做接入', body: '安装器负责把当前宿主接好，让用户知道下一句该怎么说。' },
    { title: '宿主里完成第一轮', body: 'research、三文档、确认门和第一轮前端预览都在宿主里推进。' },
    { title: '关键产物都会落盘', body: 'research、PRD、Architecture、UI/UX、Spec、运行验证和交付证据都会写回项目。' },
  ],
  installTitle: '安装方式',
  installBody:
    '终端只负责接入、升级和卸载。官网首页默认展示 uv 安装；源码安装和版本锁定安装都留在安装文档里。这里主要回答三件事：不会替你装哪些宿主本体、安装器会帮你写什么、以及回宿主后第一句输入什么。',
  installBullets: [
    '首页和主文档默认使用 uv 安装 Super Dev。',
    '不会自动安装 Claude、Codex、Cursor、Trae、Gemini CLI 等宿主本体。',
    '终端输入 super-dev 后，安装器会按项目优先写入当前宿主需要的接入文件；用户级增强面只在显式 opt-in 时补。',
    '安装完成后会直接显示推荐宿主、标准流第一句、比赛流第一句和接入后先验；Droid CLI 已纳入统一矩阵。',
  ],
  installCode:
    'uv tool install super-dev\n\n# 打开安装引导\nsuper-dev\n\n# Droid CLI / Claude Code / Codex 等宿主都在安装器中统一接入\n\n# 终端公开维护入口\nsuper-dev update\nsuper-dev uninstall',
  firstMinutesTitle: '安装后 5 分钟',
  firstMinutesBody:
    '这里决定用户会觉得产品丝滑，还是觉得只是又一个安装页。终端在安装后就应该退场，用户马上回宿主里复制首句、看框架焦点、按烟测指南确认宿主真的在按商业项目流程工作。',
  firstMinutesCards: [
    {
      title: '先离开终端',
      body: '安装、升级、卸载都在终端里做；真正开发不要停留在终端维护命令里。',
      bullets: ['完全关闭宿主并重开项目', '在新会话里继续，不要沿用旧聊天上下文', '把终端留给 install / update / uninstall'],
    },
    {
      title: '直接复制宿主第一句',
      body: '不要自己概括，不要先铺背景。先用安装后烟测指南里的标准流第一句或比赛流第一句，让宿主准确进入 Super Dev 流水线。',
      bullets: ['标准项目复制“标准流第一句”', '比赛项目复制“比赛流第一句”', '宿主第一轮必须明确 research 和三文档确认门'],
    },
    {
      title: '先按烟测和首句走',
      body: '不要自己概括流程。先用安装器给出的标准流第一句或比赛流第一句，再按烟测确认宿主真的进入了 research → 三文档 → 等确认的主路径。',
      bullets: ['复制首句，不要自己改写', '先用 smoke guide 看第一轮回复', '确认 research / 三文档 / 等确认这三件事都被宿主明确说出来'],
    },
    {
      title: '把 UI 当交付门，而不是后修饰',
      body: '截图级视觉门会正式卡掉过平、过空、模板味太重的页面，所以应该从第一轮 UI 方案就要求宿主做出更强的视觉方向。',
      bullets: ['先锁 art direction，再做页面', '不要接受占位稿式的平 UI', 'UI review / quality / release 会继续吃这一层信号'],
    },
  ],
  surfacesTitle: '安装器会写什么',
  surfacesBody:
    '你不需要记住每个宿主的文件路径。安装器会把当前宿主需要的项目级接入文件写好，可选的用户级增强面只有在你显式选择时才补。',
  surfaces: [
    {
      title: '当前项目里的接入文件',
      body: '默认先写当前项目真正需要的文件，让宿主在这个项目里知道什么时候进入 Super Dev。',
      points: ['优先当前项目', '先让这个仓库跑通', '不默认污染全局'],
    },
    {
      title: '可选的用户级增强面',
      body: '只有你明确 opt-in 时，安装器才会补常用宿主的用户级增强面，用来减少后续重复接入。',
      points: ['不是默认路径', '面向常用宿主和个人习惯', '先以项目级为主'],
    },
    {
      title: '真正会持续累积的项目产物',
      body: '研究、三文档、Spec、运行验证和交付证据，才是这条主路径真正会继续累积的东西。',
      points: ['knowledge/', 'output/*', '.super-dev/changes/*'],
    },
  ],
  triggerTitle: '宿主第一句',
  triggerBody: '大多数用户只需要记住自己宿主的第一句。Slash 宿主用 /super-dev，Codex CLI 用 $super-dev，文本宿主用 super-dev:；更细差异放到矩阵里。',
  triggerCards: [
    {
      title: 'Slash 宿主',
      command: '/super-dev 你的需求',
      hosts: SLASH_HOST_NAMES,
      note: '当安装器已经把当前宿主需要的项目级接入文件写好时，直接用这句进入 Super Dev。Codex App/Desktop 也会把启用的 super-dev Skill 暴露进 `/` 列表。',
      variant: 'certified',
    },
    {
      title: '文本触发宿主',
      command: 'super-dev: 你的需求',
      hosts: TEXT_TRIGGER_HOST_NAMES,
      note: '文本宿主不靠 slash 列表，直接把这句作为项目入口；安装器会告诉你它当前是不是已经 ready。',
      variant: 'compatible',
    },
  ],
  matrixTitle: '宿主矩阵',
  matrixBody: '看这张表时只关心三件事：你会先输入什么、Super Dev 会先读什么、以及这个宿主现在是不是已经准备好进入标准项目或 SEEAI。',
  matrixGroups: [
    {
      category: 'CLI',
      items: [
        { host: 'Claude Code', protocol: 'CLAUDE.md、settings、skills、subagents', grade: 'Certified', trigger: '/super-dev' },
        { host: 'Droid CLI', protocol: 'AGENTS.md、.factory/rules、.factory/skills', grade: 'Certified', trigger: '/super-dev' },
        { host: 'Codex CLI', protocol: 'AGENTS.md、official skills、CLI $skill', grade: 'Certified', trigger: 'CLI: $super-dev' },
        { host: 'OpenCode', protocol: 'AGENTS.md、commands、skills、agents', grade: 'Experimental', trigger: '/super-dev' },
        { host: 'Gemini CLI', protocol: 'GEMINI.md、settings、custom commands', grade: 'Compatible', trigger: '/super-dev' },
        { host: 'Kiro CLI', protocol: 'AGENTS.md、steering、skills、native resume', grade: 'Compatible', trigger: '/super-dev · super-dev:' },
        { host: 'Cursor CLI', protocol: 'AGENTS.md、.cursor/rules、native resume', grade: 'Compatible', trigger: 'super-dev:' },
        { host: 'Qoder CLI', protocol: 'AGENTS.md、rules、commands、skills、agents', grade: 'Compatible', trigger: '/super-dev' },
        { host: 'Copilot CLI', protocol: 'copilot-instructions、AGENTS.md、skills、agents', grade: 'Experimental', trigger: 'super-dev:' },
        { host: 'CodeBuddy CLI', protocol: 'CODEBUDDY.md、rules、commands、skills、agents', grade: 'Compatible', trigger: '/super-dev' },
        { host: 'Kimi Code', protocol: 'AGENTS.md、/skill:/flow、native resume', grade: 'Compatible', trigger: 'super-dev: · /skill:super-dev' },
        { host: 'Qwen Code', protocol: 'QWEN.md、commands、skills、checkpoint/restore', grade: 'Compatible', trigger: '/super-dev' },
      ],
    },
    {
      category: 'IDE',
      items: [
        { host: 'Antigravity', protocol: 'GEMINI.md、custom commands、workflow 增强', grade: 'Experimental', trigger: '/super-dev' },
        { host: 'Cursor', protocol: 'Agent Chat、AGENTS.md、rules、beta commands', grade: 'Experimental', trigger: '/super-dev' },
        { host: 'Windsurf', protocol: 'AGENTS.md、workflows、skills', grade: 'Experimental', trigger: '/super-dev' },
        { host: 'Kiro', protocol: 'AGENTS.md、steering、skills、agent continuity', grade: 'Experimental', trigger: '/super-dev' },
        { host: 'Qoder', protocol: 'AGENTS.md、rules、commands、skills、agents', grade: 'Experimental', trigger: '/super-dev' },
        { host: 'Trae', protocol: 'project context、compatibility rules、optional skills', grade: 'Compatible', trigger: 'super-dev:' },
        { host: 'TraeCN', protocol: '中文 workspace skills、/plan /spec', grade: 'Compatible', trigger: 'super-dev:' },
        { host: 'CodeBuddy', protocol: 'CODEBUDDY.md、rules、skills、workspace continuity', grade: 'Experimental', trigger: '/super-dev' },
        { host: 'CodeBuddyCN', protocol: 'CODEBUDDY.md、rules、skills、中文 workspace continuity', grade: 'Experimental', trigger: '/super-dev' },
      ],
    },
    {
      category: '桌面助手',
      items: [
        { host: 'Claude', protocol: 'Projects、project instructions、project knowledge、desktop extensions', grade: 'Compatible', trigger: 'super-dev:' },
        { host: 'Codex', protocol: 'AGENTS.md、skills、enabled app skill entry', grade: 'Certified', trigger: 'App/Desktop: / → super-dev' },
        { host: 'WorkBuddy', protocol: 'task workbench、skills、MCP', grade: 'Compatible', trigger: 'super-dev:' },
        { host: 'Trae SOLO', protocol: 'workspace rules、optional skills', grade: 'Compatible', trigger: '/super-dev' },
        { host: 'Trae SOLOCN', protocol: '中文 workspace MTC / Code、skills', grade: 'Compatible', trigger: 'super-dev:' },
      ],
    },
  ],
  pipelineTitle: '流水线',
  pipelineBody: '入口命令很短，真正重要的是后面的阶段纪律。宿主接到需求后会按研究、文档、确认、实现、验证和交付推进。',
  pipelineSteps: [
    { step: '01', title: '同类产品研究', body: '先使用宿主联网能力研究同类产品、交互模式、差异化机会和商业信号。' },
    { step: '02', title: '需求增强', body: '把边界条件、异常路径、验收口径和优先级补足，避免模糊需求直接进编码。' },
    { step: '03', title: '三份核心文档', body: '生成 PRD、Architecture、UI/UX 三文档，形成可审计的执行基线。' },
    { step: '04', title: '用户确认门', body: '三文档完成后强制暂停，用户确认前不得创建 Spec 或继续编码。' },
    { step: '05', title: 'Spec / Tasks', body: '确认通过后生成变更提案与任务清单，收紧范围和顺序。' },
    { step: '06', title: '前端优先', body: '先做前端、先跑起来、先可审查，再进入后端和联调。' },
    { step: '07', title: '后端与联调', body: '完成 API、服务、数据层和真实交互闭环。' },
    { step: '08', title: '质量门禁', body: '执行 UI Review、红队、安全、性能和架构阈值检查。' },
    { step: '09', title: '交付与最终检查', body: '只有当交付包、预览验证和最后一轮检查都完成后，这个项目才算真的结束。' },
  ],
  controlTitle: '流程控制',
  controlBody: '公开主路径里，不再要求用户记住一串终端治理命令。安装完成后，继续、返工、确认、补充，都优先在宿主里用自然语言推进。',
  controlCards: [
    {
      title: '继续改三文档',
      body: '用户补需求、业务范围变化、或者需要重做方案时，不用回终端跳阶段，直接在宿主里继续围绕三文档修改。',
      code: '/super-dev 这里补一下业务范围，先继续改 PRD / Architecture / UIUX，不要开始编码\n\nsuper-dev: 这个方案还不对，继续改三文档并等待我再次确认',
    },
    {
      title: '继续当前阶段',
      body: 'UI 重做、接口调整、交付前复检，都优先在宿主里继续当前流程，不再把普通用户赶回终端。',
      code: '/super-dev 继续当前流程\n/super-dev 现在下一步是什么\n\nsuper-dev: 继续当前流程，不要重新开题',
    },
    {
      title: '确认关键门',
      body: '文档确认、前端预览确认、UI 改版闭环、架构返工闭环、质量整改闭环，都优先在宿主里明确回复。',
      code: '/super-dev 文档确认，可以继续\n/super-dev 前端预览确认，可以继续\n\nsuper-dev: UI 改版已完成，继续当前流程\nsuper-dev: 质量整改已完成，继续当前流程',
    },
  ],
  operationsTitle: '研究、预览与交付门',
  operationsBody: '真正把产品拉出“会写代码但交付不稳”的，是 research、预览验证和交付门，而不是一大串底层命令。',
  operationsCards: [
    {
      title: '知识库优先',
      body: '当 knowledge/ 和 knowledge-bundle 存在时，宿主必须先读本地知识，再做外部研究。',
      bullets: ['research 前置知识读取', 'PRD / Architecture / UI/UX 阶段化映射', 'quality / delivery 继续复用基线'],
    },
    {
      title: '改动前先看影响范围',
      body: '接手旧仓库、修改登录流、重构 API 或动关键状态流前，先跑 repo-map、feature-checklist 和 impact，把范围覆盖率、受影响模块和回归重点确认清楚。',
      bullets: ['super-dev repo-map', 'super-dev feature-checklist', 'super-dev impact "变更描述" --files ...'],
    },
    {
      title: '代码库理解与回归守卫',
      body: '复杂仓库不要靠宿主临场猜结构。先生成依赖图，再把影响分析转换成可执行的回归清单。',
      bullets: ['super-dev dependency-graph', 'super-dev regression-guard "变更描述" --files ...', '先补回归重点再修改关键路径'],
    },
    {
      title: '前端运行验证门',
      body: '必须有 frontend-runtime 报告通过，中后段才允许继续；如果当前项目冻结了 framework playbook，还要把框架专项执行与交付证据一起纳入验证。',
      bullets: ['preview.html', 'frontend-runtime.json', '真实预览证据与交付材料', '运行通过后再进后端'],
    },
    {
      title: '质量与交付门',
      body: 'UI 评审、规范完整度、交付包和最后一轮检查共同定义“是否可交付”；截图级视觉门会正式阻断那些仍然过平、过空、模板味太重的页面。',
      bullets: ['UI Review', 'Screenshot-level visual gate', 'Spec Quality', 'Final delivery checks'],
    },
  ],
  commandsTitle: '只需要记住的入口',
  commandsBody: '公开主路径只需要两个地方：终端里的安装/更新/卸载，和宿主里的第一句 / 继续 / 确认。其余 CLI 仍然存在，但属于维护和高级排障，不是普通用户日常操作面。',
  commands: [
    {
      title: '终端公开入口',
      code: 'uv tool install super-dev\n\nsuper-dev          # 进入宿主安装引导\nsuper-dev update   # 更新到最新版\nsuper-dev uninstall # 清理宿主注入面',
      filename: 'Terminal',
    },
    {
      title: '宿主内正常使用',
      code: '/super-dev 你的需求\n/super-dev 继续当前流程\n/super-dev 现在下一步是什么\n/super-dev 文档确认，可以继续\n\nsuper-dev: 你的需求\nsuper-dev: 继续当前流程\nsuper-dev: 质量整改已完成，继续当前流程',
      filename: 'Host Conversation',
    },
  ],
  troubleshootingTitle: '排障',
  troubleshootingBody: '大多数问题都出在宿主没有重新加载当前项目的接入文件。先确认安装器写出的项目级文件存在，再确认宿主已重开并用了新会话。',
  troubleshootingSteps: [
    '重新运行 super-dev，确认安装器仍然识别到当前宿主。',
    '确认安装引导输出的项目级与用户级接入面真实存在。',
    '完全关闭宿主，重新打开项目，并新建一个会话。',
    '先用 smoke 触发语句。',
    '如果宿主直接开始开发，优先判断当前会话没有重新加载规则。',
  ],
  highlightsTitle: 'v2.4.1 当前修复重点',
  highlightsBody: '2.4.1 先把影响真实交付判断的几条链收正：backend-only 项目的质量与交付口径、release readiness 的原生测试命令识别、轻量 resume 上下文、文档生命周期索引，以及 --with-user-surfaces 的接入一致性；首页、Docs 和安装入口也一起切到新版本。',
  highlightsCards: [
    { title: '项目优先接入默认化', body: 'onboard / setup / install / start 默认只写项目级协议面；用户级 / 系统级 surface 改成显式 opt-in，不再默认污染全局 AGENTS。' },
    { title: '统一宿主矩阵', body: 'CLI、IDE 和桌面助手都已并入统一矩阵；Claude / Claude Code、Codex / Codex CLI、Trae 系列都拆成了独立条目。' },
    { title: '宿主首句与恢复剧本', body: '每个宿主现在都明确给出标准流第一句、比赛流第一句、恢复方式和修复优先级，不再只有泛化触发词。' },
    { title: '轻量恢复上下文', body: 'resume / continue 默认只让宿主先读 SESSION_BRIEF、workflow-state、当前 active change 和阶段必需文档，不再默认吞掉全量历史、proof-pack 或旧截图。' },
    { title: '文档生命周期索引', body: 'super-dev docs lifecycle 会生成 DOCS_INDEX，明确当前事实、active change、spec 与只读 archive，防止多轮迭代后误改旧文档。' },
    { title: '标准流 / SEEAI 双模式准备度', body: '宿主报告会明确区分“标准流可直接开工”和“SEEAI 比赛模式可直接开工”，不再把文件已写入误判成宿主已跑通。' },
    { title: '当前项目焦点', body: '安装器和 smoke guide 会直接告诉用户当前项目更该先看哪类页面、哪类预览和哪类交付证据。' },
    { title: '安装后先验', body: '安装器会直接显示接入后应该先确认什么，避免用户一装完就误以为已经可以稳定开工。' },
    { title: '项目级 SEEAI 补充面', body: '标准项目接入之外，super-dev-seeai 的项目级 skill/plugin 补充面也进入正式闭环，比赛模式不再靠手工补文件。' },
    { title: 'UI 设计反 AI 味升级', body: 'UI 契约新增视觉方向候选、主视觉哲学、反 AI 味护栏和五维设计批评标尺；截图级视觉门会把过平、过空、模板味过强的页面直接卡在质量与交付链外。' },
  ],
  smokeTitle: 'Smoke 验收',
  smokeCode:
    '# slash 宿主\n/super-dev "请先不要开始编码，只回复 SMOKE_OK，并说明你会先做 research、再写三文档并等待确认。"\n\n# 非 slash 宿主\nsuper-dev: 请先不要开始编码，只回复 SMOKE_OK，并说明你会先做 research、再写三文档并等待确认。',
};

const enContent: Content = {
  heroKicker: 'Documentation Center',
  heroTitle: 'Install once, then go back to the host and start the real work.',
  heroBody:
    'This page is not a protocol dictionary. It exists to move you from install to the host first prompt and the first five minutes. What matters is simple: install, go back to the host, start with research, then generate the three core docs and wait for approval.',
  heroStats: [
    { label: 'Host matrix', value: 'Unified' },
    { label: 'Install to first prompt', value: '5 min' },
    { label: 'Core phases', value: '9' },
    { label: 'Approval gates', value: '2' },
  ],
  sections: [
    { id: 'highlights', label: 'v2.4.1 Current Fix Focus', icon: Zap },
    { id: 'governance', label: 'Core path', icon: BookOpen },
    { id: 'install', label: 'Installation', icon: Package },
    { id: 'first-minutes', label: 'First 5 Minutes', icon: Sparkles },
    { id: 'surfaces', label: 'What the installer writes', icon: Boxes },
    { id: 'triggers', label: 'First prompt', icon: Command },
    { id: 'hosts', label: 'Host Matrix', icon: Terminal },
    { id: 'pipeline', label: 'Pipeline', icon: Workflow },
    { id: 'control', label: 'How to continue in-host', icon: RefreshCw },
    { id: 'operations', label: 'Research, preview, and delivery gates', icon: FolderTree },
    { id: 'commands', label: 'The only entrypoints to remember', icon: Package },
    { id: 'troubleshooting', label: 'Troubleshooting', icon: LifeBuoy },
  ],
  governanceTitle: 'The real development loop starts after you return to the host.',
  governanceBody:
    'The terminal only handles install, upgrade, and cleanup. The real first pass happens inside the host: research, the three core docs, approval gates, spec, implementation, preview validation, quality gates, and delivery all move through one governed path.',
  governanceCards: [
    { title: 'The terminal only onboards', body: 'The installer connects the current host and tells the user what to type next.' },
    { title: 'The first pass happens in-host', body: 'Research, the three core docs, approval gates, and the first frontend preview stay in the host conversation.' },
    { title: 'Key artifacts stay on disk', body: 'Research, PRD, Architecture, UI/UX, Spec, runtime reports, and delivery evidence are written back into the repo.' },
  ],
  installTitle: 'Installation',
  installBody:
    'The terminal only handles onboarding, upgrade, and uninstall. The homepage defaults to pip; uv, source installs, and pinned versions stay in the installation docs. This section answers three public questions: which host apps are not installed for you, what the installer writes, and what to type next inside the host.',
  installBullets: [
    'The homepage and primary docs use pip as the default install path; uv remains supported but stays in the dedicated installation guide.',
    'Super Dev does not install Claude, Codex, Cursor, Trae, Gemini CLI, or any other host app for you.',
    'Running super-dev opens the installer and writes project-first host files by default; user/global enhancements are explicit opt-ins.',
    'The installer now exposes the recommended host, first prompts, and post-onboard checks. Droid CLI is part of the unified matrix.',
  ],
  installCode:
    'uv tool install super-dev\n\n# open the installer\nsuper-dev\n\n# Droid CLI / Claude Code / Codex and other hosts are onboarded from the same installer\n\n# public terminal maintenance entrypoints\nsuper-dev update\nsuper-dev uninstall',
  firstMinutesTitle: 'The first 5 minutes after install',
  firstMinutesBody:
    'This is where the product either feels sharp or feels like just another setup page. After install, the terminal should get out of the way. Users should go straight back into the host, copy the first prompt, read the framework coaching focus, and confirm the host really enters the commercial delivery flow.',
  firstMinutesCards: [
    {
      title: 'Leave the terminal',
      body: 'Installation, upgrade, and uninstall happen in the terminal. Real project work should immediately move back into the host.',
      bullets: ['Close the host completely and reopen the project', 'Use a fresh host session instead of the stale conversation', 'Keep the terminal for install / update / uninstall only'],
    },
    {
      title: 'Copy the first prompt exactly',
      body: 'Do not paraphrase. Do not preload your own background. Start with the standard-flow or competition-flow first prompt from the smoke guide so the host enters Super Dev precisely.',
      bullets: ['Use the standard-flow first prompt for normal projects', 'Use the competition-flow first prompt for SEEAI work', 'The first host reply must explicitly promise research and the three-core-doc gate'],
    },
    {
      title: 'Follow the smoke guide and the first prompt first',
      body: 'Do not paraphrase the workflow. Start with the exact standard-flow or competition-flow first prompt from the installer, then use the smoke guide to confirm the host really enters the research → three core docs → approval path.',
      bullets: ['Copy the first prompt instead of rewriting it', 'Use the smoke guide to inspect the first reply', 'Confirm that research, the three core docs, and approval are all explicit'],
    },
    {
      title: 'Treat UI as a delivery gate, not decoration',
      body: 'The screenshot-grade visual gate now blocks flat, empty, template-looking pages. Users should demand stronger visual direction from the first UI pass, not after implementation is “finished.”',
      bullets: ['Lock art direction before page implementation', 'Reject placeholder-grade flat UI', 'Expect UI review / quality / release to keep enforcing this'],
    },
  ],
  surfacesTitle: 'What the installer writes',
  surfacesBody:
    'You should not have to memorize every host file path. The installer writes the project-level files the current host needs, and user-level enhancements stay optional.',
  surfaces: [
    {
      title: 'Project-level host files',
      body: 'The installer writes the files this repository needs so the current host can recognize and enter Super Dev here first.',
      points: ['project-first by default', 'make this repo work first', 'no silent global pollution'],
    },
    {
      title: 'Optional user-level enhancements',
      body: 'Global or user-level host surfaces are opt-ins. They are useful when you use the same host constantly, but they are no longer the default.',
      points: ['not the default path', 'useful for repeat host setups', 'project-first still comes first'],
    },
    {
      title: 'The project artifacts that keep accumulating',
      body: 'Research, the three core docs, Spec, runtime verification, and delivery evidence are what actually carry the workflow forward.',
      points: ['knowledge/', 'output/*', '.super-dev/changes/*'],
    },
  ],
  triggerTitle: 'First prompt',
  triggerBody: 'Most users only need to remember the first prompt for their host. Slash hosts use /super-dev, Codex CLI uses $super-dev, and text-first hosts use super-dev:. The matrix handles the edge cases.',
  triggerCards: [
    {
      title: 'Slash hosts',
      command: '/super-dev your requirement',
      hosts: SLASH_HOST_NAMES,
      note: 'When the installer has already written the right project-level files for the current host, this is the shortest way to enter Super Dev. Codex App/Desktop also exposes the enabled super-dev Skill in the `/` list.',
      variant: 'certified',
    },
    {
      title: 'Text-trigger hosts',
      command: 'super-dev: your requirement',
      hosts: TEXT_TRIGGER_HOST_NAMES,
      note: 'Text-first hosts do not depend on a slash list. They treat this line as the project entrypoint, and the installer tells you whether that host is already ready.',
      variant: 'compatible',
    },
  ],
  matrixTitle: 'Host matrix',
  matrixBody: 'Read this table for three things only: what you will type first, what Super Dev loads first, and whether that host is genuinely ready to enter the standard product path or SEEAI.',
  matrixGroups: [
    {
      category: 'CLI',
      items: [
        { host: 'Claude Code', protocol: 'CLAUDE.md, settings, skills, subagents', grade: 'Certified', trigger: '/super-dev' },
        { host: 'Codex CLI', protocol: 'AGENTS.md, official skills, CLI $skill', grade: 'Certified', trigger: 'CLI: $super-dev' },
        { host: 'CodeBuddy CLI', protocol: 'CODEBUDDY.md, skills, task continuity', grade: 'Compatible', trigger: '/super-dev' },
        { host: 'Copilot CLI', protocol: 'copilot-instructions, AGENTS.md, skills, agents', grade: 'Experimental', trigger: 'super-dev:' },
        { host: 'Cursor CLI', protocol: 'AGENTS.md, .cursor/rules, native resume', grade: 'Compatible', trigger: 'super-dev:' },
        { host: 'Droid CLI', protocol: 'AGENTS.md, .factory/rules, .factory/skills', grade: 'Certified', trigger: '/super-dev' },
        { host: 'Gemini CLI', protocol: 'GEMINI.md, settings, custom commands', grade: 'Compatible', trigger: '/super-dev' },
        { host: 'Kimi Code', protocol: 'AGENTS.md, /skill:/flow, native resume', grade: 'Compatible', trigger: 'super-dev: · /skill:super-dev' },
        { host: 'Kiro CLI', protocol: 'AGENTS.md, steering, skills, native resume', grade: 'Compatible', trigger: '/super-dev · super-dev:' },
        { host: 'OpenCode', protocol: 'AGENTS.md, commands, skills, agents', grade: 'Experimental', trigger: '/super-dev' },
        { host: 'Qoder CLI', protocol: 'AGENTS.md, rules, commands, skills, agents', grade: 'Compatible', trigger: '/super-dev' },
        { host: 'Qwen Code', protocol: 'QWEN.md, commands, skills, checkpoint/restore', grade: 'Compatible', trigger: '/super-dev' },
      ],
    },
    {
      category: 'IDE',
      items: [
        { host: 'Antigravity', protocol: 'GEMINI.md, custom commands, workflow enhancement', grade: 'Experimental', trigger: '/super-dev' },
        { host: 'CodeBuddy', protocol: 'CODEBUDDY.md, rules, skills, workspace continuity', grade: 'Compatible', trigger: '/super-dev' },
        { host: 'CodeBuddyCN', protocol: 'CODEBUDDY.md, rules, skills, CN workspace continuity', grade: 'Compatible', trigger: '/super-dev' },
        { host: 'Cursor', protocol: 'Agent Chat, AGENTS.md, rules, beta commands', grade: 'Experimental', trigger: '/super-dev' },
        { host: 'Kiro', protocol: 'AGENTS.md, steering, skills', grade: 'Experimental', trigger: '/super-dev' },
        { host: 'Qoder', protocol: 'AGENTS.md, rules, commands, skills, agents', grade: 'Experimental', trigger: '/super-dev' },
        { host: 'Trae', protocol: 'project context, compatibility rules, optional skills', grade: 'Compatible', trigger: 'super-dev:' },
        { host: 'TraeCN', protocol: 'CN workspace skills, /plan /spec', grade: 'Compatible', trigger: 'super-dev:' },
        { host: 'Windsurf', protocol: 'AGENTS.md, workflows, skills', grade: 'Experimental', trigger: '/super-dev' },
      ],
    },
    {
      category: 'Desktop assistants',
      items: [
        { host: 'Claude', protocol: 'Projects, instructions, knowledge, desktop extensions', grade: 'Compatible', trigger: 'super-dev:' },
        { host: 'Codex', protocol: 'AGENTS.md, skills, enabled app skill entry', grade: 'Certified', trigger: 'App/Desktop: / → super-dev' },
        { host: 'Trae SOLO', protocol: 'workspace rules, optional skills', grade: 'Compatible', trigger: '/super-dev' },
        { host: 'Trae SOLOCN', protocol: 'CN workspace MTC / Code, skills', grade: 'Compatible', trigger: 'super-dev:' },
        { host: 'WorkBuddy', protocol: 'task workbench, skills, MCP', grade: 'Compatible', trigger: 'super-dev:' },
      ],
    },
  ],
  pipelineTitle: 'Pipeline',
  pipelineBody: 'The trigger is short. The actual value is the enforced flow behind it: research, documentation, approval, implementation, verification, quality, and delivery.',
  pipelineSteps: [
    { step: '01', title: 'Comparable-product research', body: 'Use the host web tools first to study adjacent products, interaction patterns, differentiation, and commercial signals.' },
    { step: '02', title: 'Requirement expansion', body: 'Fill in boundaries, edge cases, acceptance criteria, and priorities before any implementation starts.' },
    { step: '03', title: 'Three core docs', body: 'Generate PRD, Architecture, and UI/UX as the auditable execution baseline.' },
    { step: '04', title: 'User confirmation gate', body: 'Pause after the docs. No Spec and no coding until the user confirms or requests revisions.' },
    { step: '05', title: 'Spec / Tasks', body: 'Create the proposal and task breakdown only after approval.' },
    { step: '06', title: 'Frontend first', body: 'Build the frontend first, make it runnable and reviewable, then move deeper into implementation.' },
    { step: '07', title: 'Backend & integration', body: 'Complete the APIs, service layers, data flows, and real end-to-end interactions.' },
    { step: '08', title: 'Quality gates', body: 'Run UI review, red-team review, security, performance, and architecture threshold checks.' },
    { step: '09', title: 'Delivery & final checks', body: 'Only finish after the delivery package, preview verification, and the final review pass are complete.' },
  ],
  controlTitle: 'Workflow control',
  controlBody: 'The public path no longer expects end users to memorize a long list of terminal workflow commands. After install, continue, revise, confirm, and recover primarily inside the host conversation.',
  controlCards: [
    {
      title: 'Keep revising the three core docs',
      body: 'When scope changes or the user adds requirements, keep the work inside the host and continue revising PRD / Architecture / UI/UX instead of bouncing back to terminal flow-control commands.',
      code: '/super-dev Add the missing scope and keep revising PRD / Architecture / UI/UX. Do not start coding.\n\nsuper-dev: The plan is still wrong. Keep revising the three core docs and wait for confirmation again.',
    },
    {
      title: 'Continue the current stage',
      body: 'When UI, APIs, or release checks need another pass, keep the user in the host conversation and continue the same Super Dev flow.',
      code: '/super-dev Continue the current flow\n/super-dev What is the next step right now?\n\nsuper-dev: Continue the current flow. Do not restart the project from scratch.',
    },
    {
      title: 'Clear the key gates in-host',
      body: 'Docs approval, preview approval, UI revision closure, architecture revision closure, and quality remediation closure should all be expressed inside the host first.',
      code: '/super-dev The core docs are approved. Continue.\n/super-dev The frontend preview is approved. Continue.\n\nsuper-dev: UI revision is finished. Continue the current flow.\nsuper-dev: Quality remediation is finished. Continue the current flow.',
    },
  ],
  operationsTitle: 'Knowledge and gates',
  operationsBody: 'Local knowledge, approval gates, runtime verification, and delivery standards define how the workflow runs.',
  operationsCards: [
    {
      title: 'Knowledge-first execution',
      body: 'When knowledge/ and a knowledge bundle exist, the host reads them before external research.',
      bullets: ['knowledge before research', 'phase-based mapping into PRD / Architecture / UI/UX', 'continued reuse in quality and delivery'],
    },
    {
      title: 'Inspect impact before changing critical flows',
      body: 'Before refactoring a mature repo, changing auth, reshaping APIs, or touching major state flows, run repo-map, feature-checklist, and impact analysis so the host starts with scope truth instead of guesses.',
      bullets: ['super-dev repo-map', 'super-dev feature-checklist', 'super-dev impact "change description" --files ...'],
    },
    {
      title: 'Codebase intelligence and regression guard',
      body: 'Do not let the host guess its way through a large repo. Generate the dependency graph first, then turn impact analysis into an executable regression checklist.',
      bullets: ['super-dev dependency-graph', 'super-dev regression-guard "change description" --files ...', 'lock the regression focus before modifying critical paths'],
    },
    {
      title: 'Frontend runtime gate',
      body: 'A page file existing is not enough. A passing frontend runtime report is required before later stages continue; when a framework playbook exists, its execution and delivery evidence become part of the gate too.',
      bullets: ['preview.html', 'frontend-runtime.json', 'real preview evidence and delivery materials', 'backend starts after runtime verification'],
    },
    {
      title: 'Quality & delivery gates',
      body: 'UI review, spec completeness, delivery packaging, and the final review pass define completion. The screenshot-grade visual gate now blocks flat, overly empty, template-looking pages.',
      bullets: ['UI Review', 'Screenshot-level visual gate', 'Spec Quality', 'Final delivery checks'],
    },
  ],
  commandsTitle: 'The only entrypoints to remember',
  commandsBody: 'The public path only needs two places: install/update/uninstall in the terminal, then first-prompt / continue / approve inside the host. The rest of the CLI remains available for maintenance and advanced troubleshooting.',
  commands: [
    {
      title: 'Public terminal entrypoints',
      code: 'uv tool install super-dev\n\nsuper-dev           # open the host installer\nsuper-dev update    # update to the latest version\nsuper-dev uninstall # remove injected host surfaces',
      filename: 'Terminal',
    },
    {
      title: 'Normal usage inside the host',
      code: '/super-dev your requirement\n/super-dev Continue the current flow\n/super-dev What is the next step right now?\n/super-dev The core docs are approved. Continue.\n\nsuper-dev: your requirement\nsuper-dev: Continue the current flow\nsuper-dev: Quality remediation is finished. Continue the current flow.',
      filename: 'Host Conversation',
    },
  ],
  troubleshootingTitle: 'Troubleshooting',
  troubleshootingBody: 'Most failures happen because the host did not reload the project files written by onboarding. First check that the project files exist, then verify the host was fully reopened and a fresh session was used.',
  troubleshootingSteps: [
    'Run super-dev again and confirm the installer still recognizes the current host.',
    'Verify that the project-level and user-level surfaces reported by onboarding actually exist.',
    'Close the host completely, reopen the project, and start a fresh chat.',
    'Use a smoke prompt before trying the real requirement.',
    'If the host starts coding immediately, assume the current session did not reload the rules.',
  ],
  highlightsTitle: 'v2.4.1 Current Fix Focus',
  highlightsBody: 'Version 2.4.1 first tightens the parts that directly affect release correctness: backend-only delivery closure, native test-command detection, lightweight resume context, document lifecycle indexing, and consistent --with-user-surfaces onboarding. The homepage, docs, and installer entry are aligned to the new version at the same time.',
  highlightsCards: [
    { title: 'Project-First Onboarding', body: 'onboard / setup / install / start now write project-level protocol surfaces by default. User/global surfaces are explicit opt-ins instead of silent defaults.' },
    { title: 'Unified 26-Host Matrix', body: 'CLI 12, IDE 9, and desktop assistants 5 now sit inside the same current host matrix. Claude / Claude Code, Codex / Codex CLI, and the Trae family are all split into separate entries.' },
    { title: 'Host-Specific First Prompts', body: 'Each host now exposes a standard-flow first prompt, competition-flow first prompt, resume guidance, and repair priority instead of one generic trigger script.' },
    { title: 'Lightweight Resume Context', body: 'resume / continue now asks hosts to read SESSION_BRIEF, workflow-state, the active change, and stage-required docs first instead of full history, proof packs, and old screenshots.' },
    { title: 'Document Lifecycle Index', body: 'super-dev docs lifecycle generates DOCS_INDEX so current truth, active changes, specs, and read-only archives stay separated across iterations.' },
    { title: 'Standard vs. SEEAI Readiness', body: 'Host reports now explicitly distinguish "ready for standard flow" from "ready for SEEAI competition flow" rather than treating written files as equivalent to a working host runtime.' },
    { title: 'Current project focus', body: 'The installer and smoke guide now tell users which pages, previews, and delivery signals matter first for the current project.' },
    { title: 'Post-Onboard Self-Check', body: 'The installer now tells users what to verify first so onboarding is not mistaken for real readiness.' },
    { title: 'Project-Level SEEAI Supplements', body: 'The super-dev-seeai project supplements now participate in formal injection closure, so competition mode is no longer a manual afterthought.' },
    { title: 'UI Design Anti-Slop Upgrade', body: 'The UI contract now freezes art-direction candidates, visual philosophy, anti-AI-slop guardrails, and a five-dimension critique rubric; the screenshot-grade visual gate blocks flat, generic, template-looking pages from passing delivery.' },
  ],
  smokeTitle: 'Smoke validation',
  smokeCode:
    '# slash hosts\n/super-dev "Do not start coding. Reply only with SMOKE_OK and explain that you will do research first, then generate the three core docs, then wait for confirmation."\n\n# non-slash hosts\nsuper-dev: Do not start coding. Reply only with SMOKE_OK and explain that you will do research first, then generate the three core docs, then wait for confirmation.',
};

function SectionShell({
  id,
  icon: Icon,
  label,
  title,
  body,
  children,
}: {
  id: string;
  icon: LucideIcon;
  label: string;
  title: string;
  body: string;
  children: React.ReactNode;
}) {
  return (
    <section id={id} className="scroll-mt-24 border-b border-border-default/70 pb-10 last:border-b-0 last:pb-0">
      <div className="mb-6 flex items-start gap-4">
        <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-accent-blue/25 bg-accent-blue/8 text-accent-blue">
          <Icon size={20} />
        </div>
        <div className="max-w-4xl">
          <p className="mb-2 text-xs font-mono uppercase tracking-[0.22em] text-accent-blue">{label}</p>
          <h2 className="mb-3 text-2xl font-bold tracking-tight text-text-primary sm:text-[2rem]">{title}</h2>
          <p className="text-[15px] leading-8 text-text-secondary">{body}</p>
        </div>
      </div>
      {children}
    </section>
  );
}

export function DocsPageContent({ locale = 'zh' }: { locale?: SiteLocale }) {
  const content = locale === 'en' ? enContent : zhContent;
  const homeHref = localizedPath(locale, '/');

  return (
    <main className="min-h-screen bg-bg-primary pt-14" id="main-content">
      <section className="relative overflow-hidden border-b border-border-muted bg-bg-primary">
        <div className="absolute inset-0 opacity-70 [background-image:radial-gradient(circle_at_top_left,rgba(37,99,235,0.22),transparent_32%),radial-gradient(circle_at_80%_12%,rgba(59,130,246,0.12),transparent_24%)]" />
        <div className="absolute inset-x-0 top-0 h-px bg-[linear-gradient(90deg,transparent,rgba(59,130,246,0.65),transparent)]" />
        <div className="relative mx-auto w-full max-w-[1380px] px-4 py-14 sm:px-6 lg:px-8 lg:py-16">
          <div className="grid gap-8 lg:grid-cols-[minmax(0,1fr)_340px]">
            <div className="max-w-[860px]">
              <div className="mb-5 flex flex-wrap items-center gap-2">
                <Badge variant="version">{content.heroKicker}</Badge>
                <Badge variant="certified">v2.4.1</Badge>
                <Badge variant="compatible">{locale === 'en' ? 'Bilingual' : '中英双语'}</Badge>
              </div>
              <h1 className="max-w-[900px] text-4xl font-bold leading-[1.08] tracking-tight text-text-primary sm:text-5xl lg:text-[3.5rem]">
                {content.heroTitle}
              </h1>
              <p className="mt-5 max-w-[760px] text-lg leading-8 text-text-secondary">{content.heroBody}</p>
              <div className="mt-8 flex flex-wrap gap-3">
                <Link
                  href="#first-minutes"
                  className="inline-flex items-center gap-2 rounded-xl bg-accent-blue px-5 py-3 text-sm font-semibold text-white transition-colors hover:bg-accent-blue-hover"
                >
                  {locale === 'en' ? 'Open the first 5 minutes' : '先看安装后 5 分钟'}
                  <ArrowRight size={16} />
                </Link>
                <Link
                  href="#install"
                  className="inline-flex items-center gap-2 rounded-xl border border-border-emphasis px-5 py-3 text-sm font-semibold text-text-primary transition-colors hover:bg-bg-tertiary"
                >
                  {locale === 'en' ? 'Open install guide' : '查看安装方式'}
                </Link>
                <Link
                  href={homeHref}
                  className="inline-flex items-center gap-2 rounded-xl border border-border-default px-5 py-3 text-sm font-semibold text-text-secondary transition-colors hover:bg-bg-tertiary hover:text-text-primary"
                >
                  {locale === 'en' ? 'Back to website' : '返回官网'}
                </Link>
              </div>
            </div>

            <div className="rounded-[24px] border border-border-default bg-[linear-gradient(180deg,rgba(45,51,59,0.92),rgba(22,27,34,0.96))] p-5 glow-blue">
              <div className="mb-4 flex items-center justify-between gap-3">
                <div className="flex items-center gap-2 text-sm font-semibold text-text-primary">
                  <Sparkles size={16} className="text-accent-blue" />
                  {locale === 'en' ? 'Install command' : '安装指令'}
                </div>
                <Badge variant="default">{locale === 'en' ? 'First step' : '第一步'}</Badge>
              </div>
              <p className="mb-5 text-sm leading-7 text-text-secondary">
                {locale === 'en'
                  ? 'Install the CLI first. Then run super-dev in the terminal to open the host installer and write the required protocol surfaces.'
                  : '先安装 CLI。然后在终端输入 super-dev，进入宿主安装引导并写入所需接入面。'}
              </p>
              <div className="rounded-2xl border border-border-default bg-bg-primary/80 p-4">
                <CopyCommand command="uv tool install super-dev" className="w-full" />
                <div className="mt-3">
                  <CodeBlock
                    code={`uv tool install super-dev\n\nsuper-dev\nsuper-dev update\nsuper-dev uninstall`}
                    filename={locale === 'en' ? 'Install' : '安装'}
                    className="bg-bg-primary"
                  />
                </div>
              </div>
            </div>
          </div>

        </div>
      </section>

      <section className="mx-auto w-full max-w-[1380px] px-4 py-10 sm:px-6 lg:px-8 lg:py-14">
        <div className="grid gap-8 xl:grid-cols-[220px_minmax(0,860px)]">
          <aside className="hidden xl:block">
            <div className="sticky top-24 rounded-[24px] border border-border-default bg-bg-secondary/55 p-4 backdrop-blur-sm">
              <div className="mb-3 text-xs font-mono uppercase tracking-[0.18em] text-text-muted">
                {locale === 'en' ? 'Path' : '路径'}
              </div>
              <nav className="space-y-1.5">
                {content.sections.map((section) => {
                  const Icon = section.icon;
                  return (
                    <a
                      key={section.id}
                      href={`#${section.id}`}
                      className="flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm text-text-secondary transition-colors hover:bg-bg-tertiary hover:text-text-primary"
                    >
                      <Icon size={15} className="text-accent-blue" />
                      <span>{section.label}</span>
                    </a>
                  );
                })}
              </nav>
            </div>
          </aside>

          <div className="space-y-8">
            <SectionShell id="highlights" icon={Zap} label={content.sections[0].label} title={content.highlightsTitle} body={content.highlightsBody}>
              <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                {content.highlightsCards.map((card) => (
                  <div key={card.title} className="rounded-2xl border border-accent-blue/25 bg-accent-blue/5 p-5">
                    <h3 className="mb-2 text-base font-semibold text-text-primary">{card.title}</h3>
                    <p className="text-sm leading-7 text-text-secondary">{card.body}</p>
                  </div>
                ))}
              </div>
            </SectionShell>

            <SectionShell id="governance" icon={BookOpen} label={content.sections[1].label} title={content.governanceTitle} body={content.governanceBody}>
              <div className="grid gap-4 lg:grid-cols-2">
                {content.governanceCards.map((item) => (
                  <div key={item.title} className="rounded-2xl border border-border-default bg-bg-elevated/80 p-5">
                    <h3 className="mb-2 text-base font-semibold text-text-primary">{item.title}</h3>
                    <p className="text-sm leading-7 text-text-secondary">{item.body}</p>
                  </div>
                ))}
              </div>
            </SectionShell>

            <SectionShell id="install" icon={Package} label={content.sections[2].label} title={content.installTitle} body={content.installBody}>
              <div className="grid gap-4 lg:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
                <div className="rounded-2xl border border-border-default bg-bg-elevated/80 p-5">
                  <h3 className="mb-4 text-lg font-semibold text-text-primary">{locale === 'en' ? 'Installation notes' : '安装说明'}</h3>
                  <ul className="space-y-3 text-sm leading-7 text-text-secondary">
                    {content.installBullets.map((bullet) => (
                      <li key={bullet} className="flex gap-3">
                        <span className="mt-2 h-1.5 w-1.5 rounded-full bg-accent-blue" />
                        <span>{bullet}</span>
                      </li>
                    ))}
                  </ul>
                </div>
                <CodeBlock code={content.installCode} filename={locale === 'en' ? 'Install Flow' : '安装流程'} />
              </div>
            </SectionShell>

            <SectionShell id="first-minutes" icon={Sparkles} label={content.sections[3].label} title={content.firstMinutesTitle} body={content.firstMinutesBody}>
              <div className="grid gap-4 lg:grid-cols-2">
                {content.firstMinutesCards.map((card) => (
                  <div key={card.title} className="rounded-2xl border border-accent-blue/20 bg-[linear-gradient(180deg,rgba(37,99,235,0.08),rgba(15,23,42,0.02))] p-5">
                    <h3 className="mb-3 text-lg font-semibold text-text-primary">{card.title}</h3>
                    <p className="mb-4 text-sm leading-7 text-text-secondary">{card.body}</p>
                    <ul className="space-y-2 text-sm text-text-secondary">
                      {card.bullets.map((bullet) => (
                        <li key={bullet} className="flex gap-2">
                          <Sparkles size={16} className="mt-1 text-accent-blue" />
                          <span>{bullet}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                ))}
              </div>
            </SectionShell>

            <SectionShell id="surfaces" icon={Boxes} label={content.sections[4].label} title={content.surfacesTitle} body={content.surfacesBody}>
              <div className="grid gap-4 lg:grid-cols-2">
                {content.surfaces.map((surface) => (
                  <div key={surface.title} className="rounded-2xl border border-border-default bg-bg-elevated/80 p-5">
                    <h3 className="mb-3 text-lg font-semibold text-text-primary">{surface.title}</h3>
                    <p className="mb-4 text-sm leading-7 text-text-secondary">{surface.body}</p>
                    <ul className="space-y-2 text-sm text-text-secondary">
                      {surface.points.map((point) => (
                        <li key={point} className="flex gap-2">
                          <span className="mt-2 h-1.5 w-1.5 rounded-full bg-accent-blue" />
                          <span>{point}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                ))}
              </div>
            </SectionShell>

            <SectionShell id="triggers" icon={Command} label={content.sections[5].label} title={content.triggerTitle} body={content.triggerBody}>
              <div className="space-y-4">
                {content.triggerCards.map((card) => (
                  <div key={card.title} className="rounded-2xl border border-border-default bg-bg-elevated/80 p-5">
                    <div className="mb-3 flex items-center justify-between gap-3">
                      <h3 className="text-lg font-semibold text-text-primary">{card.title}</h3>
                      <Badge variant={card.variant}>{card.variant === 'certified' ? 'Slash' : locale === 'en' ? 'Text trigger' : '文本触发'}</Badge>
                    </div>
                    <div className="mb-4 rounded-xl border border-border-default bg-bg-primary px-4 py-3 font-mono text-sm text-accent-blue">{card.command}</div>
                    <p className="mb-4 text-sm leading-7 text-text-secondary">{card.note}</p>
                    <div className="flex flex-wrap gap-2">
                      {card.hosts.map((host) => (
                        <Badge key={host} variant="default" className="font-mono">{host}</Badge>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </SectionShell>

            <SectionShell id="hosts" icon={Terminal} label={content.sections[6].label} title={content.matrixTitle} body={content.matrixBody}>
              <div className="space-y-4">
                {content.matrixGroups.map((group) => (
                  <div key={group.category} className="rounded-2xl border border-border-default bg-bg-elevated/80 p-5">
                    <div className="mb-5 flex items-center justify-between gap-3">
                      <h3 className="text-lg font-semibold text-text-primary">{group.category}</h3>
                      <Badge variant="version">{group.items.length}</Badge>
                    </div>
                    <div className="space-y-3">
                      {group.items.map((item) => (
                        <div key={item.host} className="rounded-xl border border-border-default bg-bg-primary/70 p-4">
                          <div className="mb-2 flex items-center justify-between gap-3">
                            <div className="font-mono text-sm text-text-primary">{item.host}</div>
                            <Badge variant={gradeVariant(item.grade)}>{gradeLabel(item.grade, locale)}</Badge>
                          </div>
                          <div className="mb-2 font-mono text-xs text-accent-blue">{item.trigger}</div>
                          <div className="text-sm leading-6 text-text-secondary">{item.protocol}</div>
                        </div>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </SectionShell>

            <SectionShell id="pipeline" icon={Workflow} label={content.sections[7].label} title={content.pipelineTitle} body={content.pipelineBody}>
              <div className="space-y-4">
                {content.pipelineSteps.map((step, index) => (
                  <div key={step.step} className="grid gap-4 rounded-2xl border border-border-default bg-bg-elevated/80 p-5 lg:grid-cols-[88px_minmax(0,1fr)]">
                    <div className="flex items-center gap-3 lg:block">
                      <div className="inline-flex h-12 w-12 items-center justify-center rounded-2xl border border-accent-blue/35 bg-accent-blue/10 font-mono text-sm text-accent-blue">
                        {step.step}
                      </div>
                      {index < content.pipelineSteps.length - 1 ? <div className="hidden h-10 w-px translate-x-6 bg-border-default lg:block" /> : null}
                    </div>
                    <div>
                      <h3 className="mb-2 text-lg font-semibold text-text-primary">{step.title}</h3>
                      <p className="text-sm leading-7 text-text-secondary">{step.body}</p>
                    </div>
                  </div>
                ))}
              </div>
            </SectionShell>

            <SectionShell id="control" icon={RefreshCw} label={content.sections[8].label} title={content.controlTitle} body={content.controlBody}>
              <div className="grid gap-4 lg:grid-cols-3">
                {content.controlCards.map((card) => (
                  <div key={card.title} className="rounded-2xl border border-border-default bg-bg-elevated/80 p-5">
                    <h3 className="mb-3 text-lg font-semibold text-text-primary">{card.title}</h3>
                    <p className="mb-4 text-sm leading-7 text-text-secondary">{card.body}</p>
                    <CodeBlock code={card.code} filename={locale === 'en' ? 'Control' : '控制'} />
                  </div>
                ))}
              </div>
            </SectionShell>

            <SectionShell id="operations" icon={FolderTree} label={content.sections[9].label} title={content.operationsTitle} body={content.operationsBody}>
              <div className="space-y-4">
                {content.operationsCards.map((card) => (
                  <div key={card.title} className="rounded-2xl border border-border-default bg-bg-elevated/80 p-5">
                    <h3 className="mb-3 text-lg font-semibold text-text-primary">{card.title}</h3>
                    <p className="mb-4 text-sm leading-7 text-text-secondary">{card.body}</p>
                    <ul className="space-y-2 text-sm text-text-secondary">
                      {card.bullets.map((bullet) => (
                        <li key={bullet} className="flex gap-2">
                          <Search size={16} className="mt-1 text-accent-blue" />
                          <span>{bullet}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                ))}
              </div>
            </SectionShell>

            <SectionShell id="commands" icon={Package} label={content.sections[10].label} title={content.commandsTitle} body={content.commandsBody}>
              <div className="space-y-4">
                {content.commands.map((command) => (
                  <div key={command.title} className="rounded-2xl border border-border-default bg-bg-elevated/80 p-5">
                    <h3 className="mb-4 text-lg font-semibold text-text-primary">{command.title}</h3>
                    <CodeBlock code={command.code} filename={command.filename} />
                  </div>
                ))}
              </div>
            </SectionShell>

            <SectionShell id="troubleshooting" icon={LifeBuoy} label={content.sections[11].label} title={content.troubleshootingTitle} body={content.troubleshootingBody}>
              <div className="space-y-4">
                <div className="rounded-2xl border border-border-default bg-bg-elevated/80 p-5">
                  <h3 className="mb-4 text-lg font-semibold text-text-primary">{locale === 'en' ? 'Troubleshooting order' : '排查顺序'}</h3>
                  <ol className="space-y-3 text-sm leading-7 text-text-secondary">
                    {content.troubleshootingSteps.map((step, index) => (
                      <li key={step} className="flex gap-3">
                        <span className="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-full border border-accent-blue/35 bg-accent-blue/10 font-mono text-xs text-accent-blue">
                          {index + 1}
                        </span>
                        <span>{step}</span>
                      </li>
                    ))}
                  </ol>
                  <div className="mt-6 border-t border-border-default pt-6">
                    <h3 className="mb-4 text-lg font-semibold text-text-primary">{content.smokeTitle}</h3>
                    <CodeBlock code={content.smokeCode} filename="Smoke" />
                  </div>
                </div>
              </div>
            </SectionShell>
          </div>
        </div>
      </section>
    </main>
  );
}
