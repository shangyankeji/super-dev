# Changelog

本文件记录 Super Dev 的所有重要变更。格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)。

## [Unreleased]

### 新增 — 3 个主流 backend(13 个 worker 总覆盖)

继续补全主流 AI 编码 CLI 矩阵,这一轮加 3 个:

| Backend ID | TUI 命令 | 调用形式 | 说明 |
|---|---|---|---|
| **`trae`** | `/trae` | `trae-cli run "<p>"` | ByteDance Trae Agent(注意:**`trae-cli`** 不是 IDE 的 `trae`) |
| **`plandex`** | `/plandex` | `plandex tell --skip-menu --stop "<p>"` | 大上下文(2M tokens)开源 agent,147k stars |
| **`cody`** | `/cody` | `cody chat --message "<p>"` | Sourcegraph 企业级,带代码索引上下文 |

实现:都是 `SimpleHostDriver` 工厂函数,Plandex 因为是 agentic 模式(`tell` 默认会编辑文件)也用了 `STDOUT_ONLY_SUFFIX`(同 Droid)。`BACKEND_IDS.len() == 13` 锁定,`driver_for` 13 路全连。

**研究后跳过的主流 CLI**:
- **Open Interpreter**:`interpreter` 二进制只有交互模式,没有一发即走的 CLI 形式(Python API 才有 `interpreter.chat(...)`),不适合 subprocess 驱动。
- **Cline / Roo Code**:都是 VS Code 扩展,没有独立 CLI。
- **Warp 2.0**:是终端而不是 agent。
- **Codeium / Windsurf**:只是 VS Code 自动补全,没有非交互 CLI。
- **Devin**(Cognition):没有公开 CLI 接口。

### Backend 出厂质量 — 5 个核心 host 完美适配

针对 **claude-code / gemini / codex / droid / opencode** 这 5 个旗舰 backend,逐个端到端真测 + 修发现的所有问题。出厂结论:

| Backend | 状态 | 实测条件 |
|---|---|---|
| `claude-code` | ✓ 完美 | 完整 9 阶段流水线 + proof-pack 落地 + 95/100 质量门 |
| `gemini` | ✓ 完美 | 真实 Linear/Stripe 案例分析,docs_confirm 225 行 LLM 实输出 |
| `droid` | ✓ 完美 | `say hello` 单行干净返回,9 阶段流水线持续验证中 |
| `codex` | ✓ 驱动完美 | 网络可达即开箱可用(需要能访问 `chatgpt.com/backend-api`) |
| `opencode` | ✓ 驱动完美 | 接通 + TUI header 已剥离,LLM 调用速度取决于用户配的 model 提供商 |

### Backend 驱动修复

- **Codex 之前 `codex exec "<p>"` 在非 git 目录会 hang**:加 `--sandbox workspace-write`(headless 必备,否则进入交互 approval 等输入)、`--skip-git-repo-check`(Super Dev workspace 通常不是 git repo)、`--color never`(避免 ANSI 噪声)。
- **Claude Code 驱动加 `--output-format text`**:之前裸 `--print` 在某些版本会返回 JSON 信封。**故意不加 `--bare`**:bare 模式会跳过 OAuth + keychain,要求 `ANTHROPIC_API_KEY` —— 而 Super Dev 全部价值就在于驱动用户**已经登录的订阅**,所以 `--bare` 会反向破坏目标用户。
- **Droid 驱动加 `-o text`**:虽然是文档默认值,显式声明避免未来 Droid 改默认值时回归。
- **OpenCode 输出 sanitize**:`opencode run` 在 stdout 顶部输出 `> build · <model>` TUI 风格头部,Super Dev 在 `SimpleHostDriver::complete()` 里加 backend-specific 后处理(`strip_opencode_header`)剥掉这个头,让喂给下游 phase 的内容只有真实 LLM 文本。
- **CodexDriver / SimpleHostDriver 文档全部更新**:每个 flag 都有"为什么"注释 + 实测验证版本号 + 已知环境依赖。

### 新增 — 8 个新 host backend (10 个 worker 全适配)

之前只接了 Claude Code 和 Codex。这一版把所有主流 AI 编码 CLI 一次性收齐:

| Backend ID | 触发命令 | 调用形式 |
|---|---|---|
| `claude-code` | `/claude` | `claude --print "<prompt>"` |
| `codex` | `/codex` | `codex exec --skip-git-repo-check "<prompt>"` |
| **`gemini`** | `/gemini` | `gemini -p "<prompt>"` (Google Gemini CLI) |
| **`droid`** | `/droid` | `droid exec --auto medium "<prompt>"` (Factory.ai) |
| **`opencode`** | `/opencode` | `opencode run "<prompt>"` (开源,Go) |
| **`cursor-agent`** | `/cursor` | `cursor-agent -p --output-format text "<prompt>"` |
| **`qwen`** | `/qwen` | `qwen -p "<prompt>"` (阿里 Qwen Code,Gemini CLI fork) |
| **`continue`** | `/continue-cli` | `cn -p "<prompt>"` (Continue.dev) |
| **`copilot`** | `/copilot` | `copilot -p --allow-all-tools "<prompt>"` (新版 GitHub Copilot CLI) |
| **`aider`** | `/aider` | `aider --yes --no-stream --message "<prompt>"` |

实现:`crates/super-dev-host/src/simple.rs` 加 `SimpleHostDriver` 通用结构,8 个工厂函数 (`droid()` / `opencode()` / `gemini()` / `cursor_agent()` / `qwen()` / `continue_cli()` / `copilot()` / `aider()`) 一次性生成所有新 backend。每个 backend 都支持 `SUPER_DEV_<NAME>_BIN` 环境变量覆盖二进制路径。`probe_all()` 现在并发探测全部 10 个 host(`tokio::join!` 10 路并行)。Picker / `BackendArg` / slash-command 路由全跟上。

### 修复

- **`super-dev continue` 不再悄悄掉回 offline 模板。** 原行为:`super-dev run "..." --backend claude-code` 跑通 docs_confirm 之后,`super-dev continue` 默认回退到 offline,导致 spec/frontend/backend 阶段不再走真 worker。修复:`WorkflowState` 增加 `backend` 字段,`continue` / `revise` 自动复用 `run` 时声明的 worker,新增 `super-dev continue --backend <id>` 显式覆盖。
- **`codex` 后端在非 git 目录跑会 fail。** Codex CLI 要求工作目录是 git repo 或显式 `--skip-git-repo-check`。Super Dev workspace 经常只有 `output/` + `.super-dev/`,所以 `CodexDriver::base_args()` 默认带上 `--skip-git-repo-check`。
- **`runtime:` 报告标签误导。** 之前 offline 模式打印 `runtime: Anthropic (Claude Agent SDK)`,看上去是真在调 Claude SDK。修成 `runtime: Offline deterministic templates (no AI; demos / CI)` 或 `runtime: Host CLI worker — Claude Code (claude-code)`,名实相符。
- **`transition note` 用 `runtime:` 标签同上,改成 `worker:`** + 把 offline 写成 `offline-templates` 而不是 RuntimeKind 字符串。
- **`super-dev verify` 多出一栏 worker。** `workflow-state: phase=... active_gate=... worker=claude-code ...`,审计链一眼看清哪个 worker 在跑。

### 新增

- **TUI gate 卡片**:停在 `docs_confirm` / `preview_confirm` 时,SuperDev 推一张完整卡片(待审稿工件清单 + `/continue` / `/revise` / `/diff` 三个动作 + 简短指引)。`Gate` 角色的消息额外用黄色 ╔══ ╗ 框包出来。
- **回车 `c` 快捷键**:在 gate 状态下,单字符 `c` / `C` 等价 `/continue`,和 gate 卡片承诺的快捷键名实相符。
- **`/model <id>`**:TUI 内切换 worker model 并落到 config.toml。
- **`/version` overlay**:binary / spec / worker / model / workspace / config 一览。
- **`/changelog` overlay**:`include_str!` 编译期内嵌本文件。
- **`/help` 分组**:从 17 条平铺改成 Worker / Pipeline & gates / Inspect / Editing & exit 四组,picker 模式保留 Navigation 单组。
- **聊天滚动指示**:历史超出可见区域时,标题动态变成 ` Conversation · ↑ N more above `。
- **Pre-flight 计划消息**:用户提交需求那一刻,SuperDev 推一条 9 阶段计划卡片(包含两道 gate 提示),消除"按了回车之后是不是没反应"的疑虑。
- **gate-aware 输入框标题** + **stage-aware did-you-mean**:`/quitz` → `/quit`,`/rev` → `/revise`,未启动 / 跑流水线中 / 已完成 三种状态的 `/continue` 各自给精确引导。
- **`/verify` overlay 含质量门得分**:`88/100 (PASSED)` 直接显示,不必再开 JSON。

### 改进

- `super-dev continue --backend claude-code` 显式覆盖 worker(高于持久化字段)。
- Spec / Verify / Doctor / Diff / History overlay 已完整可用(M14b 留的 stub 已落地)。
- `SUPER_DEV_CODEX_BIN` / `SUPER_DEV_CODEX_EXEC_SUBCMD` 环境变量仍可覆盖默认值。
- `read_workflow_state` 向下兼容老的 state 文件(没有 `backend` 字段时默认为空)。

### 测试 +30 (4.4.0 基线 225 → 现在 255+)

- `app.rs` +13:gate 卡片、`c` 快捷键、`/model` 持久化、`/version` overlay、`/changelog` overlay、did-you-mean、preflight、`/verify` 质量门、JSON 标量解析器。
- `ui.rs` +5:gate-aware 标题、运行中标题、滚动指示、`/help` 分组渲染、对话滚动溢出。
- `state.rs` +2:backend 字段 round-trip、legacy state 向下兼容。
- 其它小修。

## [4.4.0] - 2026-05-23

### 主题

**Claude Code 同款 chat-style TUI**。`super-dev` 一行进入对话界面,首次启动选 worker(claude-code / codex / offline)写入 `~/.super-dev/config.toml`,之后直接进对话。所有操作走对话框 + 斜杠命令,不再有"Welcome → 流水线进度条"这种割裂屏幕。

### 破坏性变更

- **`super-dev tui` 子命令删除** —— 直接 `super-dev` 即可。CLI verbs(`run` / `continue` / `revise` / ...)保留给脚本用。
- **TUI 内部状态机重写**:`Welcome` + `Running` → `Picker` + `Chat`。welcome 屏 / 9-phase 进度面板 / 事件日志面板全部下线;chat 模式用滚动消息历史承载所有 pipeline 事件。
- **`super-dev_tui::LaunchOptions` 字段精简**:删 `requirement` / `backend`,现在只剩 `project_root` / `slug` / `model`(用户选择从 config 读)。

### 新增

- **`crates/super-dev-tui/src/config.rs`** —— `~/.super-dev/config.toml` 读写。Fail-soft:文件不存在 / 解析失败 / IO 错误都退化为"无偏好,显示 picker"。
  - 字段:`backend = "claude-code" | "codex" | "offline"`、`model = "..."`。
  - 路径:`$XDG_CONFIG_HOME/super-dev/config.toml` 优先,否则 `$HOME/.super-dev/config.toml`。
- **首次启动 Picker** —— 三选项(claude-code / codex / offline)+ 实时 probe 标签。↑↓ 导航,Enter 写盘 + 进 Chat。不可用宿主拒绝(显示提示)。
- **Chat 主屏**:
  - 顶栏 status:版本 + workspace + ● backend + 当前 phase + ⏸ gate。
  - 滚动消息历史:`you` / `super-dev` / `worker` / `gate` / `system` 5 种角色标签(各自颜色)。
  - 输入框:5 行高度,光标 ▌,/ 前缀自动列出候选命令。
  - 底部 footer hint。
- **斜杠命令路由器**:
  - `/claude` `/codex` `/offline` —— 切 worker(写 config + 系统消息)
  - `/continue` —— 批准 gate
  - `/revise <文字>` —— 提修订
  - `/help` `/?` `/commands` —— 帮助浮层
  - `/clear` —— 清屏 history
  - `/quit` `/q` `/exit` —— 退出
  - `/diff` `/spec` `/verify` `/doctor` `/history` —— 暂时给提示(浮层渲染留 M14b)
- **非斜杠输入路由**:无 run 在跑 → 当新需求;gate 打开 → 当修订;delivery 完成 → 当下一个新需求(自动 reset)。

### 测试 +28(共 ~225 总数,因为删了一批旧 Welcome/Running 测试)

- `config.rs` 7:round-trip / 缺失文件 / 损坏文件 / 创建父目录 / XDG_CONFIG_HOME。
- `app.rs` 21:Picker 导航/拒绝不可用/probe 刷新/转 Chat;Chat 普通文本提交/空回车 noop/斜杠 help quit clear claude continue revise/未知命令/gate 时文本当修订/delivery 后文本当新 run/host 输出/history 上限/F1 / spinner。
- `ui.rs` 8:Picker 三选项 + 选中标记/Chat 问候 + 输入框 + 光标 + slash typeahead/gate 角色/worker 角色/help overlay 模式相关。

### 变更

- 版本 4.3.0 → 4.4.0。
- 命令面 11 → 10(删 `tui` 子命令)。
- README / README_EN / CLAUDE.md / guide.txt 全部同步新心智模型。

## [4.3.0] - 2026-05-23

### 主题

大厂级用户交互打磨 + 流式输出 + 知识库智能注入 + CI 自动发 npm。

### 新增

- **`super-dev examples`** —— 一行打印完整 cheat-sheet:首次用法 / CI 用法 / 迭代 / 切 backend / TUI 键位 / 斜杠命令 / 环境变量。
- **`super-dev guide`** —— 60 秒走读:产品定位 / 9 阶段图 / 用户角色 / 9 命令 / 工件清单 / 治理规则。
- **每个命令富 `long_about` + `EXAMPLES:` 块** —— `super-dev <cmd> --help` 给真实示例,不再干瘪。
- **typo 容错** —— `super-dev rin` → `tip: some similar subcommands exist: 'init', 'run'`(clap 默认开,文案有效)。
- **`EngineEvent::HostOutput`** —— host CLI 的每行输出按行 emit 出来,TUI 实时滚动显示 host 在干啥(buffered-at-end,真 wire 流式留给 M9b)。
- **TUI App `HostOutput` 处理** —— 每行带 `    [phase]` 前缀进事件日志,长行 200 字符截断。
- **知识库智能注入** —— `summarise_knowledge_dir` 升级为 `smart_knowledge_digest`:按需求关键词排名 → 挑 top-6 → 每个真摘 600 字符塞进 prompt;无关键词匹配时 lex 排序兜底。
- **CI release.yml 接 npm publish** —— tag `v*` push 时,build 阶段同时把每平台 binary stage 进 `npm/cli-<plat>/`,publish-npm job 拉回来一键发 6 包(需配 `NPM_TOKEN` secret)。
- **`aarch64-unknown-linux-gnu` 平台** —— 用 `cross` 在 ubuntu-latest 上交叉编译,补全 Linux ARM 支持。

### 变更

- 版本 4.2.0 → 4.3.0。
- 命令面 9 → 11(新增 examples / guide)。

### 测试 +11 → **203 tests pass**

- `extract_keywords_filters_short_and_stopwords`
- `score_path_counts_keyword_hits`
- `smart_digest_picks_keyword_matches_top`
- `smart_digest_falls_back_to_lex_when_no_keyword_match`
- `smart_digest_handles_missing_dir`
- `host_output_lines_land_in_log`
- `host_output_truncates_very_long_lines`
- `examples_command_prints_cheatsheet`
- `guide_command_prints_walkthrough`
- `run_help_includes_examples`
- `unknown_subcommand_suggests_a_correction`

## [4.2.0] - 2026-05-23

### 主题

外挂式产品形态彻底落地:**删干净 plugin 注入式架构,主推 `npm install -g super-dev` 一行装机**。

### 破坏性变更

- **`super-dev install` / `uninstall` / `hook` 三个命令删除** —— 它们对应"把 SKILL.md/AGENTS.md/hook 配置注入宿主目录"的旧模型,和新定位(外挂项目经理,只调度不嵌入)矛盾。
- **`plugin/` 目录整体删除**(3 家宿主的 SKILL.md / AGENTS.md / plugin.json / hook config × 11 文件 + `crates/super-dev/src/install.rs` 约 400 行实现)。
- **`super-dev verify` 不再输出 `## Installed plugins` 节**。
- **`super-dev doctor` 不再做插件相关检查**(check_embedded_plugins、check_installed_plugins、版本错配检测——这些都依赖 plugin 概念)。

### 新增

- **`npm/` 多平台分发** —— 主包 `super-dev` + 5 个平台子包 `@super-dev/cli-{darwin-arm64,darwin-x64,linux-x64,linux-arm64,win32-x64}`,esbuild / biome / swc 同款模式。
  - **JS shim** `npm/super-dev/bin/cli.js`:`require.resolve` 找到匹配平台的预编译 Rust 二进制,`spawnSync(..., {stdio: 'inherit'})` 透传 stdio,TUI 直接可用。
  - **`stage.sh`** 把 prebuilt binary 摆进对应平台子包。
  - **`smoke.sh`** 本地端到端验证(本机已实测通过)。
  - **`publish.sh`** 一行发布 6 包(5 平台 + 主包)。

### 变更

- 版本 4.1.0 → 4.2.0。
- 命令面从 12 个瘦身到 9 个(删 install/uninstall/hook)。
- `super-dev init` 输出的"next steps"提示从"super-dev install claude-code"改成"super-dev"启动 TUI。
- `super-dev-governance` crate **保留**(pipeline 内部仍用 audit / context / compliance),只是没有 CLI 出口。

### 测试 +0 / 192 总数

(删测试和加测试相抵:删 1 个 `hook_check_emoji_returns_block_decision_for_tsx` e2e,删 doctor 的若干 plugin 检测测试,删 install.rs 的 7 个内部测试;保留所有 pipeline / verify loop / TUI / spec / runtime 相关测试。)

## [4.1.0] - 2026-05-23

### 主题

定位钉死:**Super Dev 是 AI 编码的项目经理(确定性外挂编排 Agent),不是 LLM 客户端**。彻底删掉所有"直调 LLM API"的代码与外宣口径。

### 破坏性变更

- **`super-dev-runtime` 的 `anthropic` / `openai` / `antigravity` 三个 HTTP 客户端模块删除**——`super-dev` 二进制不再含直调 Provider API 的能力。
- **CLI `--api` flag + `--runtime` flag 移除**——run/tui 现在只有两种"大脑":`--backend claude-code|codex`(默认推荐)或离线模板。
- **`super-dev-runtime` 不再依赖 `reqwest` / `eventsource-stream` / `tokio::process` / `anyhow` / `tracing`**——降为纯 trait crate(`Runtime` + `OfflineRuntime`),依赖只剩 `async-trait` / `serde` / `serde_json` / `thiserror`。
- `RuntimeError::Transport` / `RuntimeError::Provider` 两个 HTTP 变体删除。

### 新增

- **`super-dev` 无参数直接进 TUI** —— 像 `claude` / `codex` 那样,敲一个词就开干。
- **TUI Welcome 屏** —— 自动并发探测 `claude` / `codex`,自动选第一个 ready 的;用户在 TUI 内文本框输入需求,`Tab` 切换工作者,`Enter` 提交进入 Running。
- TUI `LaunchOptions` 替代旧 `(RunOptions, Option<String>)` 参数对。
- `Action::Submit` 事件 —— 从 Welcome 提交后,event loop 自动 build `RunOptions` + spawn pipeline + 切到 Running 模式。
- `App::cycle_backend()` —— `Tab` 在 `[offline, ...ready_backends]` 之间循环。
- `App::enter_running()` —— 显式状态机翻页。

### 变更

- 版本 4.0.0 → 4.1.0。
- 主标语:"a coach for AI coding hosts" → "**AI 编码的项目经理 — drives your logged-in Claude Code / Codex through a 9-phase commercial delivery pipeline. No API key needed.**"
- README / README_EN / spec §9 全部清掉"three SDK runtimes"线,统一"项目经理 + host driver"口径。
- `apply_key(char)` → `apply_key(KeyCode)`,模型层处理 Backspace / Enter / Tab / Esc 等专用键。

## [4.0.0] - 2026-05-22

### 主题

从"装进宿主的插件"演进为"驱动宿主的编排器"：Super Dev 现在是一个 TUI 应用，把用户**已登录的** Claude Code / Codex CLI 当作按需调用的执行后端——零 API key，零额外登录。

### 破坏性变更

- **`super-dev run` 的 `--offline` 标志移除**：执行模式改为三选一——`--backend claude-code|codex`（驱动已登录的宿主 CLI）、`--api`（直调 Provider API，需 key）、或默认离线确定性模板。

### 新增

- **`crates/super-dev-host`** —— 宿主驱动层。`ClaudeCodeDriver` 包 `claude --print`、`CodexDriver` 包 `codex exec`，都实现 `Runtime` trait 让现有 `AgentRunner` 直接驱动。子进程走 `tokio::process` + `.args()`（不走 shell）、`kill_on_drop`、超时保护。`probe_all()` 并发探测宿主可用性。
- **`crates/super-dev-tui`** —— ratatui 终端应用。`super-dev tui "<需求>"` 启动：9 阶段进度面板 + 实时事件日志 + gate 键盘交互（`c` 过 gate / `q` 退出）+ `b` 宿主探测浮层。
- **引擎事件流**（`super-dev-agent::events`）：`EngineEvent` + `EventSink`（`NullSink` / `ChannelSink` / `RecordingSink`）。`AgentRunner` 在 phase 起止、artifact 写入、gate 打开时 emit 事件;TUI 订阅 `ChannelSink` 渲染实时进度。
- **`super-dev init`** —— 写出 `super-dev.yaml` spec manifest（落地 SD-META-001）;`super-dev run` 也会自动补写。
- **`super-dev doctor`** —— 自检命令:binary 完整性、嵌入 plugin/规范、workspace 可写、已装 plugin 版本错配。
- **`super-dev uninstall`** —— 移除宿主插件，保留 `.super-dev/` 用户数据。
- **`super-dev tui`** —— 进入交互式 TUI。

### 变更

- 版本 3.0.0 → 4.0.0。
- spec §7 主机映射表对齐到三家官方 SDK 家族，明确其余宿主 out-of-scope。
- `OfflineRuntime` 从二进制提到 `super-dev-runtime`，CLI 与 TUI 共用;新增 `Box<dyn Runtime>` impl。
- CONTRIBUTING.md 改写为 Rust workspace 贡献指南。

### crate 总览（7 个）

`super-dev` · `super-dev-spec` · `super-dev-governance` · `super-dev-agent` · `super-dev-runtime` · `super-dev-host` · `super-dev-tui`

## [3.0.0] - 2026-05-20

### 主题

彻底重构：Python → Rust，从工具 → 规范产品，从 30 个浅适配宿主 → 3 个深度兼容宿主家族。

### 破坏性变更

- **语言切换**：整个项目从 Python 改写为 Rust workspace。所有 `super_dev/` Python 代码、`pyproject.toml`、`uv.lock`、`requirements.lock`、Python 测试套件全部移除。
- **CLI 全面重构**：50+ 子命令收紧为 4 条（`run` / `spec` / `hook` / `verify`）。Python 时代的 `init / migrate / setup / detect / doctor / quality / review / release / enforce / spec / config / hooks / experts / memory / compact / pipeline / clean / completion / feedback / ...` 等命令全部删除。
- **宿主适配从 30 个收紧到 3 个**：只保留有官方 Agent SDK 的家族——Anthropic（Claude Code / Claude Desktop）、OpenAI（Codex CLI / Codex Desktop）、Google（Antigravity CLI / Antigravity Desktop）。Cursor、Windsurf、Cline、Roo、Continue、Trae、Qoder、CodeBuddy、WorkBuddy、Kiro、Droid、Gemini CLI、Kimi、Qwen 等 27 个浅适配宿主全部移除。
- **MCP server 退场**：纯 Rust 直调 Provider API，不再需要 MCP 中转。
- **SKILL.md / hook 安装器退场**：Python 时代的 `.claude-plugin/`、`plugins/`、宿主 hook 注入器全部移除；规范由 agent 直接驱动，而非由各家宿主的 hook 实现。

### 新增

- **Rust workspace**（5 个 crate）：
  - `super-dev` — 主二进制
  - `super-dev-spec` — 规范的 Rust 数据表达（25 条 clause × 4 层 + 9 阶段 + 2 gate）
  - `super-dev-governance` — 治理核心（`rules` / `audit` / `context` / `compliance`），fail-open
  - `super-dev-agent` — 9 阶段流水线 runner + gate 语义 + workflow state
  - `super-dev-runtime` — Anthropic / OpenAI / Antigravity 三家 HTTP 适配（直调 Provider API）
- **`SUPER_DEV_HOST_SPEC_V1` 规范本体**进入 `spec/` 顶层目录，与代码 1:1 对齐
- **单二进制分发**：`cargo build --release` 出一个静态二进制，零运行时依赖
- **`super-dev hook`** 子命令把所有治理判定收敛到一条入口：宿主配置只写 `super-dev hook check-emoji` 等命令，无需 Python 解释器
- **多 runtime 选择**：`super-dev run "..." --runtime anthropic|openai|antigravity`

### 保留

- `spec/SUPER_DEV_HOST_SPEC_V1.md` — 规范本体
- `knowledge/` — 治理知识库
- `super-dev-website/` — Next.js 官网（独立工程）
- `docs/assets/` — README 图片
- `output/`、`.super-dev/` — 用户项目数据（gitignore）

### 迁移

- 没有 from-2.4 的迁移路径。3.0 是从零重启；旧 Python 用户保留旧版本即可。

## [2.3.4] - 2026-04-10

### 主题

Plan-Execute 编排升级 + Overseer 监督者 + Claude/Codex 混合审查

### 致谢

- 感谢 **staruhub** 提交并推动合入 [PR #10](https://github.com/shangyankeji/super-dev/pull/10)，本次版本的核心能力来自这次贡献。

### 新增

- **Plan-Execute 执行引擎**：结构化执行计划、拓扑波次排序、步骤状态机、步骤级验证门、失败预算和持久化计划状态。
- **Overseer 监督者角色**：独立质量观察者，在阶段与步骤检查点持续监控计划偏差、质量下降和未解决审查结果，并可在关键问题下中止流水线。
- **Claude Code + Codex 混合模式**：Claude Code 负责实现、Codex 负责独立审查，审查结果由 Overseer 统一跟踪和校验。
- 新增配置项：`execution_mode`、`overseer_enabled`、`codex_review_enabled`、`codex_review_phases`、`overseer_halt_on_critical`、`plan_failure_budget`。

### 变更

- 官网首页已同步到 `2.3.4`。
- 官网更新历史页已新增 `2.3.4` 条目。
- README、发布说明和版本真源已统一到 `2.3.4`。

## [2.3.3] - 2026-04-07

### 主题

宿主适配质量 + 安装升级体验

### 废弃

- **Claude Code 不再安装 `super-dev-core` 别名**：统一为 `super-dev` 单一入口。升级后自动清理旧版残留（对用户无感）。

### 新增

- **`super-dev update` 升级后自动迁移**：pip/uv 升级完成后自动调用新版 `super-dev migrate`，一步完成升级+迁移。
- **`super-dev` 无参数自动迁移旧版**：检测到项目配置版本低于当前 CLI 版本时自动迁移。
- **`super-dev migrate` 全宿主迁移**：重写为全宿主迁移引擎，自动检测所有已接入宿主并重建配置/Skill/slash/协议到最新版。
- **`--auto` 同族宿主智能去重**：同时检测到 cursor + cursor-cli 等同族宿主时自动选择功能更完整的 CLI 版本。
- **Roo Code**：IntegrationTarget 补齐 `.roo/commands/super-dev.md`。
- **OpenCode**：IntegrationTarget 补齐 `.opencode/commands/super-dev.md`。
- **Kilo Code**：补齐项目级 `.kilocode/skills/super-dev-core/SKILL.md` 和用户级 Skill surface。
- **VS Code Copilot**：补齐 HOST_CERTIFICATIONS 认证条目。

### 修复

- **commands 文件内容错误**：`setup()` 生成 `.roo/commands/`、`.opencode/commands/` 等 command 文件时走 fallback 生成了通用 rules 内容而非 slash command 格式。
- **SkillFrontmatter 默认 name**：从 `"super-dev-core"` 改为 `"super-dev"`。
- **所有 `skill_name="super-dev-core"` 硬编码**：全部改为 target-aware 或统一为 `"super-dev"`。
- **`install.sh` Skill 提示**：去除 `super-dev-core` 名称。
- **版本提示**：统一为 `super-dev update` 而非 `pip install -U`。

### 测试验证

- 全量 2151 测试通过，0 失败。

## [2.3.2] - 2026-04-06

### 新增

- Claude Code 按 `CLAUDE.md + .claude/skills + ~/.claude/skills + optional plugin enhancement` 收口。
- Codex 按 `AGENTS.md + .agents/skills + repo plugin enhancement` 收口，并区分 App/Desktop、CLI、fallback 三入口。
- `session_resume_card`、`doctor`、`detect`、`start`、`continue`、Web API 显示现实场景卡（第二天回来继续开发、只想知道当前唯一下一步、当前确认门内继续修改、本地流程中断后恢复）。
- `.super-dev/SESSION_BRIEF.md` 新增 `## 现实场景怎么做` 段落。
- 新增/强化 `workflow-history`、语义 workflow 事件、`hook-history`、`workflow/framework/hook/operational harness`、`recent operational timeline`，已进入 `proof-pack`、`release readiness`、恢复卡、`SESSION_BRIEF`。
- `framework_playbook` 覆盖 `uni-app`、`Taro`、`React Native`、`Flutter`、`Desktop Web Shell`，进入提示词、UIUX 文档、`ui-contract.json`、frontend/implementation builder、runtime/quality gate/proof-pack/release readiness。

### 变更

- 21 个宿主口径继续收正：`20` 个统一接入宿主 + `1` 个 OpenClaw 手动插件宿主。
- Kiro / Qoder / Cursor / Trae / CodeBuddy 等宿主的官网说明、安装引导、能力审计页与代码模型重新对齐。
- emoji 作为功能图标被系统级禁止，被 runtime、UI review、quality gate、release readiness 一起拦截。

## [2.3.1] - 2026-04-03

### 新增

- Codex 深度适配：`AGENTS.md + Skills + repo plugin 增强` 双层模型。
- Codex 三入口统一：App/Desktop `/super-dev`、CLI `$super-dev`、回退 `super-dev: 你的需求`。
- Claude Code 深度适配：`CLAUDE.md + .claude/skills + ~/.claude/skills + optional plugin enhancement`。

### 变更

- 安装引导不再使用"slash 宿主 / 非 slash 宿主"二分法，改为基于宿主真实入口模型。
- Codex 标记为 skill-first 模式（App/Desktop `/super-dev`、CLI `$super-dev`）。
- Claude Code 标记为 `CLAUDE.md + Skills` 主模型，`commands / agents` 仅作为兼容层保留。
- Onboard 完成页删除过期 `/super-dev init` 提示，改为真实宿主入口指导。
- 版本真源统一为 `2.3.1`，README / README_EN / QUICKSTART / HOST_USAGE_GUIDE / INSTALL_OPTIONS 同步更新。
- 官网首页 Hero、终端演示、更新历史同步到 `2.3.1`。

## [2.3.0] - 2026-03-31

### 新增

#### Enforcement 执行层

- `super-dev enforce install` — 自动为宿主配置 hooks（PreToolUse emoji 检查等）。
- `super-dev enforce validate` — 运行验证脚本，检查 emoji/import/color/route 合规性。
- `super-dev enforce status` — 查看当前执行层配置状态。
- `super-dev detect --auto` 时自动安装 enforcement hooks。

#### Memory 记忆系统

- `super-dev memory list` — 列出所有项目记忆。
- `super-dev memory show <name>` — 查看指定记忆内容。
- `super-dev memory forget <name>` — 删除指定记忆。
- `super-dev memory consolidate` — 触发 Dream 整合。
- 4 种记忆类型：user、feedback、project、reference。
- MEMORY.md 索引，200 行 / 25KB 自动限制。
- Dream 整合器：4 阶段后台记忆合并（去重、聚合、摘要、写回）。

#### 代码生成器

- `super-dev generate scaffold --frontend next` — Next.js App Router 项目脚手架（16 个文件）。
- `super-dev generate components` — UI 组件脚手架（Button/Card/Input/Modal/Nav/Layout）。
- `super-dev generate types` — 从架构文档生成共享 TypeScript 类型。
- `super-dev generate tailwind` — 从 UIUX 设计 tokens 生成 Tailwind 配置。

#### Expert 专家系统

- `super-dev experts list` — 列出所有专家（内置 + 自定义）。
- `super-dev experts show <name>` — 查看专家定义。
- 12 位内置专家 Markdown 定义：PM、ARCHITECT、UI、UX、SECURITY、CODE、DBA、QA、DEVOPS、RCA、PRODUCT、VERIFICATION。
- 用户可通过 `.super-dev/experts/*.md` 自定义专家。
- 新增对抗性验证专家（VERIFICATION.md），在质量门禁中担任"红方"角色。

#### Hook 系统

- `super-dev hooks list` — 列出已配置的 hooks。
- `super-dev hooks test <event>` — 测试 hook 执行。
- 8 种 hook 事件：PrePhase、PostPhase、PreDocument、PostDocument、PreQualityGate、PostQualityGate、OnError、SessionStart。
- 在 `super-dev.yaml` 中通过 YAML 配置，支持 Shell 和 Python 执行器。

#### Context Compact（上下文压缩）

- `super-dev compact list` — 列出各阶段的压缩摘要。
- `super-dev compact show` — 查看指定阶段的压缩内容。
- 9 段结构化摘要模板，自动在阶段切换时保存/恢复上下文。

#### Web API

- 11 个新端点：记忆管理、hooks 管理、专家查询、上下文压缩、会话状态等。

#### 条件规则系统

- 新模块 `super_dev/rules/` — 支持 `.super-dev/rules/*.md` 条件规则。
- 规则可通过 frontmatter `paths` 指定只对特定文件生效，支持排除模式。

#### UX 增强

- 首次使用引导：3 步快速开始面板，最多显示 4 次后自动隐藏。
- Tips 提示系统：根据当前阶段显示上下文相关的操作建议。
- 项目模板：`super-dev init --template ecommerce/saas/dashboard/mobile/api/blog/miniapp`。
- `doctor --fix`：自动修复检测到的安装问题。
- Shell 补全：`super-dev completion bash/zsh/fish`。
- 版本更新检查：PyPI 24h 缓存，有新版时提示升级。
- `super-dev feedback`：快速打开 GitHub Issues 反馈。
- `super-dev migrate`：2.2.0 → 2.3.0 一键迁移。

### 变更

- Skill 模板引擎升级：支持宿主特定 frontmatter 渲染、编码前门禁、常见错误速查、阶段宣告机制。
- Prompt 生成器重构为分层注册制（9 段优先级架构），支持数据驱动规则和行为约束模板。
- Pipeline 引擎集成 Hook 系统、上下文压缩、记忆提取、Session Brief 全链路增强。
- CLAUDE.md 增强：编码约束段、技术栈预研要求、图标与视觉规则、前后端对齐规则、每文件自检要求。
- 4-Agent 并行审查框架（复用 + 质量 + 效率 + 安全）。
- 验证脚本增强：多级输出（Level 1 阻塞 / Level 2 警告 / Level 3 建议），新增 console.log / hardcoded localhost / TODO-FIXME / 大文件 / package.json scripts 检查。
- `--help` 分组显示（核心 / 治理 / 分析）。
- 品牌输出使用纯 ASCII 字符，兼容所有终端。
- 版本号全面统一为 2.3.0。

### 破坏性变更

- 版本号从 2.2.0 升级至 2.3.0。
- `super-dev.yaml` 中 version 字段默认值变更为 2.3.0。
- 配置迁移：运行 `super-dev detect --auto` 以更新宿主集成配置。

### 修复

- `detect --auto` 现在会实际安装文件（之前仅生成报告）。
- `detect` 与 `doctor` 现在使用相同的检测逻辑（不再出现互相矛盾的结果）。
- `super-dev` 无参数时显示欢迎信息而非内部状态。
- `super-dev status` 在初始化后显示"已初始化，等待开始"。
- SKILL.md 中 `config show` 修正为 `config list`。
- 仓库地址修正为 `shangyankeji/super-dev`。

### 测试验证

- 全量测试：1643 passed。
- `ruff check`：通过。
- `python3 -m compileall super_dev`：通过。

## [2.2.0] - 2026-03-29

### 新增

- 重构工作流状态机与恢复链，补齐 `resume / next / SESSION_BRIEF / workflow-state` 语义，支持下班后、宿主关闭后、第二天回来继续当前流程。
- 宿主接入与诊断链路升级，统一 `start / detect / doctor / Web API` 的决策卡与恢复卡。
- UI 系统正式接入主流程：新增 `ui-contract.json`、`design-tokens.css`、`ui-contract-alignment`，从需求到 release 全链路治理。
- Release / proof-pack / quality gate / release readiness 进一步打通。
- UI 组件生态偏向 `shadcn/ui + Radix + Tailwind`，允许基于场景选择更合适方案。
- UI review 新增对主题入口、导航骨架、组件导入路径、反模式命中的结构化检查。
- `quality gate`、`proof-pack`、`release readiness`、`frontend-runtime` 均已纳入 UI 契约执行校验。
- 支持 Windows / 自定义安装路径发现逻辑，支持 `SUPER_DEV_HOST_PATH_<HOST>` 覆盖。

### 变更

- 正式产品口径统一为 `20` 个统一接入宿主，`OpenClaw` 改为手动插件安装路径。
- 宿主安装、检测、恢复、继续、返工、发布动作语义统一。
- 显式指定宿主时，系统会围绕该宿主给出决策，不再被自动检测结果带偏。

### 测试验证

- 全量测试：1281 passed。
- `ruff check`：通过。
- `python3 -m compileall super_dev`：通过。
