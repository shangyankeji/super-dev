# Claude Code 源码学习 —— 对 Super Dev 的补齐清单

> 基于 `/Users/weiyou/Documents/kaifa/claude-code-main/src`(Claude Code 源码快照,TypeScript/React-Ink)的深度分析。
> 目的:找出值得 Super Dev(Rust TUI 编排层,驱动 claude code/codex)借鉴补齐的机制。
> 所有断言附 Claude Code 源码路径;Super Dev 现状基于本项目源码核对。

---

## 一、Claude Code 的五大核心机制(学习所得)

### 1. 分层权限模型 + bypass-immune 安全护栏
- **allow/deny/ask 规则引擎**,规则来自 5 个 source(user/project/local/policy/flag),磁盘持久化在 settings.json 的 `permissions.allow/deny/ask`,字符串语法 `Bash(npm:*)` / `Edit(path)`。
- **5 种权限模式**:default / plan / acceptEdits / bypassPermissions / dontAsk + `auto`(AI 分类器)。
- **bypass-immune**:即使在 `bypassPermissions` 模式,敏感路径(`.git/`/`.claude/`/`.vscode/`/shell 配置)的写操作**仍强制弹窗** —— 这是安全护栏核心。
- **auto 模式**:不弹窗,跑独立 AI 分类器判断 shouldBlock,有熔断(连续 deny 达上限回退人工)。
- 证据:`types/permissions.ts:16-441`、`utils/permissions/permissions.ts:473-1319`、`utils/sandbox/sandbox-adapter.ts`。

### 2. TodoWrite + 子代理编排
- **TodoWrite**:给 LLM 自用的 checklist(prompt 驱动自律),亮点是 **verification nudge** —— 一次性关掉 ≥3 个 todo 且无 verify 步骤时,注入提示逼它 spawn 验证子代理。
- **子代理 spawn**:Agent 工具 spawn 子代理,子代理有**独立上下文 + sidechain transcript + 独立 token 预算**,输出压缩成 result text 回主线程。
- **Task 落盘**:后台 Task 输出写磁盘而非内存,支撑长时间运行 + TaskCreate/List/Get/Stop 工具让 LLM 自管。
- 证据:`tools/TodoWriteTool/`、`tools/AgentTool/runAgent.ts`、`Task.ts`、`tasks/`。

### 3. Skills 系统
- Skill = 带 YAML frontmatter 的 `SKILL.md`,既是用户 `/name` 命令,也是 **LLM 可自动调用的工具**(带 `whenToUse` 描述)。
- **contextModifier**:skill 执行时临时扩权/切模型/切 effort,而非全局改。
- **inline vs fork**:inline 注入会话,fork 成子代理跑。
- 证据:`tools/SkillTool/SkillTool.ts:331-841`、`skills/loadSkillsDir.ts:299-401`。

### 4. System Prompt 构造
- **静态/动态边界**(`SYSTEM_PROMPT_DYNAMIC_BOUNDARY`):固定文案在前(跨用户 prompt cache 命中),会话相关在后(每会话变)。section memoize 到 /clear。
- **git status / CLAUDE.md 走 system-reminder 注入,非塞进 system prompt**,避免每轮膨胀。
- 按 enabledTools 动态裁剪工具说明。
- 证据:`constants/prompts.ts:444-577`、`systemPromptSections.ts:20-58`、`context.ts:36-189`。

### 5. MCP(Model Context Protocol)集成
- 多 transport(stdio/SSE/HTTP/WebSocket/SDK)统一抽象,远程 tool 用单一模板 + `mcpInfo` 区分。
- `mcp__server__tool` 命名 + server 级/通配权限规则。
- annotation(readOnly/destructive/openWorld)映射到权限决策。
- 证据:`services/mcp/client.ts:1743-1813`、`services/mcp/config.ts`。

---

## 二、Super Dev 现状对照(基于本项目源码)

| Claude Code 机制 | Super Dev 现状 | 差距 |
|------------------|---------------|------|
| 分层权限模型 | 有 governance(check_emoji/color/slop)但只检查代码风格,无 allow/deny/ask 工具级权限 | 🔴 缺工具级权限 |
| bypass-immune 敏感路径护栏 | 无(governance 是 fail-open,但覆盖面是代码模式不是路径) | 🔴 缺 |
| TodoWrite verification nudge | 9 阶段流水线有结构化流程,但无"LLM 自用 checklist + verification nudge" | 🟡 部分不同范式 |
| 子代理独立上下文 + sidechain | 每阶段独立调 claude(prompt in/out),但无 sidechain transcript 落盘 | 🟡 部分 |
| Task 输出落盘 | 有(audit JSONL + artifacts),但不是"LLM 可自管的后台 Task" | 🟡 部分 |
| Skill contextModifier | 斜杠命令是纯文本展开,不能临时扩权/切模型 | 🟡 缺 |
| System prompt 静态/动态边界 + cache | 有 knowledge 注入,但无 SYSTEM_PROMPT_DYNAMIC_BOUNDARY 切分 | 🟡 可优化 |
| git status 走 system-reminder | 无(不注入 git 上下文) | 🟡 缺 |
| MCP 集成 | 无 | 🔴 缺 |

---

## 三、可补齐清单(按 Super Dev 价值×可行性排序)

### 🥇 P0 — 安全护栏(最该补,编排层调破坏性操作时的基石)

**借鉴:bypass-immune 敏感路径强制检查**
- Claude Code 即使在"跳过权限"模式,对 `.git/`/`.claude/`/`.vscode/`/shell 配置的写仍强制拦截。
- **Super Dev 现状**:`super-dev-governance` 的 hook 只查 emoji/color/slop,不查"是否在写敏感路径"。
- **补齐**:governance rules 加一条 `check_sensitive_path` —— 宿主写 `.git/`、`.env`、`settings.json`、`.ssh/` 等敏感路径时,即使 governance fail-open 也强制 block 或 warn。
- 落点:`crates/super-dev-governance/src/rules.rs`(新增 check) + `hook.rs`(调用)。
- 价值:Super Dev 作为编排层驱动 claude code 写文件,这是真实的安全风险点。

### 🥈 P1 — System Prompt 优化(降本 + 提质量)

**借鉴:静态/动态边界 + git status 走 system-reminder**
- Claude Code 把固定指令放前(命中 prompt cache),会话相关放后;git status 不塞 system prompt 而是 reminder。
- **Super Dev 现状**:`experts.rs` 的 prompt 把知识库/文档 excerpt 全混在一起,无 cache 友好的切分。
- **补齐**:`research_prompt`/`frontend_prompt` 等,把固定角色说明(SPEC_PREAMBLE + 角色)放最前,会话相关(requirement/uiux excerpt)放后。当前已部分这么做,可显式标注边界。
- 价值:claude code 调用是按 token 计费的,prompt cache 命中能省钱 + 提速。

### 🥉 P2 — TodoWrite verification nudge(几乎零成本,提质量)

**借鉴:一次性关多个 todo 时注入 verification 提示**
- **Super Dev 现状**:9 阶段流水线的 quality 阶段做验证,但中间阶段(frontend/backend)生成完直接进 gate,无"自我验证 nudge"。
- **补齐**:frontend/backend prompt 末尾加一句"生成完自检:运行 build/test,确认无报错再写 notes"(其实已有 step 8 "Run build",可强化)。
- 价值:提升单阶段产出质量,减少 gate 处的反复修改。

### P3 — MCP 集成(扩展性,长期)

**借鉴:多 transport 统一接入 + mcp__server__tool 命名**
- **Super Dev 现状**:只驱动 claude code/codex,无外部工具接入。
- **补齐**:这是较大的架构扩展,作为长期路线。短期可先支持 MCP stdio 一种 transport。
- 价值:让 Super Dev 能编排更多工具,不只 claude code/codex。

### P4 — Skill contextModifier(能力增强,中期)

**借鉴:命令执行时临时扩权/切模型**
- **Super Dev 现状**:斜杠命令(/provider /design 等)是纯操作,不改运行时上下文。
- **补齐**:让特定命令能临时切 provider/model(如 `/deploy` 临时切到一个强模型)。
- 价值:更灵活的编排。

---

## 四、最重要的认知(来自学习)

Claude Code 的设计哲学:**用 prompt 驱动 LLM 自律 + 用权限/沙箱做硬护栏 + 用 system-reminder 而非 prompt 膨胀注入上下文**。

Super Dev 作为编排层,最该学的是:
1. **硬护栏**(bypass-immune 敏感路径检查)—— 编排层调破坏性操作时的安全底线。
2. **prompt cache 友好的切分** —— 编排层调 claude code 是按 token 计费的,省钱提速。
3. **prompt 驱动自律**(verification nudge)—— 几乎零成本提质量。

这三点是 Super Dev 作为"套壳编排层"最能直接受益的,且实现成本可控。

---

## 附录:Claude Code 关键源码路径索引(供后续参考)

| 机制 | 路径 |
|------|------|
| 权限决策主入口 | `utils/permissions/permissions.ts:473-1319` |
| bypass-immune safetyCheck | `permissions.ts` 1f/1g 步骤 |
| TodoWrite + verification nudge | `tools/TodoWriteTool/` |
| 子代理 spawn + sidechain | `tools/AgentTool/runAgent.ts` |
| System prompt 构造 | `constants/prompts.ts:444-577` |
| 动态边界(cache) | `constants/prompts.ts:114-115` |
| git status 注入 | `context.ts:36-189` |
| Skill contextModifier | `tools/SkillTool/SkillTool.ts:331-841` |
| MCP tool 包装 | `services/mcp/client.ts:1743-1813` |
| 沙箱配置编译 | `utils/sandbox/sandbox-adapter.ts:172-381` |
