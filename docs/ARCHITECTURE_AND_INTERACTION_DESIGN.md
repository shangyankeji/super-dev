# Super Dev — 架构与交互设计文档

> 本文档基于源码事实审计(非想象),定义 Super Dev 的**产品定位、整体架构、工作原理、用户旅程、交互细节**,并给出"真正让用户开发出商业应用"的**差距分析与实施路线**。
>
> 审计日期:2026-06-16。所有"现状"描述均可在源码中验证。

---

## 一、产品定位(一句话 + 展开)

### 一句话

**Super Dev 是给 AI 编码宿主(claude code / codex / 自定义 API)套壳的专业开发流程编排器。它本身不写代码、不含模型,而是用一套「9 阶段流水线 + 实时治理 + 质量门 + 合规审计」的 playbook 指挥宿主按商业级标准交付。**

### 展开(来自 spec SUPER_DEV_HOST_SPEC_V1.md §coach metaphor)

> Super Dev is a coach for the host. It does not write code itself.
> It hands the host a complete, pipeline-shaped playbook... and then steps off the field.
> The host's existing model + tools execute; the coach's standard is what makes the result commercial-grade.

| 维度 | Super Dev(本产品) | 宿主(claude code 等) |
|------|-------------------|----------------------|
| 角色 | 教练 / 治理层 / playbook 制定者 | 执行者 / 真正写代码的 |
| 是否含模型 | ❌ 不含(除非用户配自定义 API) | ✅ 自带模型 + 工具 |
| 是否写代码 | ❌ 不写一行 | ✅ 写全部代码 |
| 产出 | 流程编排、结构化 prompt、治理决策、审计证据 | 真实的代码文件、项目工程 |
| 可替换性 | 宿主可换(claude-code/codex + 自定义 API) | — |

**这是核心差异化**:Super Dev 不和 claude code 竞争(它驱动 claude code),不和 Lovable 竞争(它是流程层不是生成器)。它的价值 = **让任意 AI 编码宿主的产出达到商业级**。

---

## 二、整体架构(基于源码)

```
┌─────────────────────────────────────────────────────────────────┐
│  super-dev binary (纯 Rust,单二进制)                            │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  super-dev-tui  ← 用户唯一入口(ratatui 全屏 TUI)         │  │
│  │  · 首次启动:分组 picker(宿主CLI / 第三方API / 离线)      │  │
│  │  · 第三方 API:对话式向导(12预设 + 实时验证)             │  │
│  │  · 聊天主界面:输入需求 → 触发流水线                       │  │
│  │  · gate 审核交互                                           │  │
│  │  · 斜杠命令:/run /continue /revise /provider /status …    │  │
│  └──────────────────────────┬────────────────────────────────┘  │
│                             │ 触发                                │
│  ┌──────────────────────────▼────────────────────────────────┐  │
│  │  super-dev-agent  ← 流水线编排(playbook 执行器)          │  │
│  │  · 9 阶段状态机 + 2 gate                                   │  │
│  │  · 每阶段:拼 prompt → 调 runtime → 落盘产物 → 验证        │  │
│  │  · prompt 工程层(experts.rs):research/prd/arch/uiux/      │  │
│  │    frontend/backend 六套专家 prompt                         │  │
│  │  · 知识检索(BM25/向量)注入 prompt                         │  │
│  └──────────┬──────────────────────────────┬──────────────────┘  │
│             │ 调用                          │ 治理                │
│  ┌──────────▼────────────┐  ┌──────────────▼──────────────────┐  │
│  │ super-dev-runtime      │  │ super-dev-governance            │  │
│  │ Runtime trait(唯一抽象)│  │ · rules.rs:SD-CODE-001/002/005 │  │
│  │ ┌────────────────────┐ │  │   (emoji/颜色/slop 实时拦截)    │  │
│  │ │ HostDriver(2 CLI) │ │  │ · audit:JSONL 证据链            │  │
│  │ │ OpenAiHttp(新)    │ │  │ · compliance:SOC2/ISO27001/     │  │
│  │ │ AnthropicHttp(新) │ │  │   EU AI Act 映射                │  │
│  │ │ OfflineRuntime    │ │  │ · fail-open:治理永不阻塞宿主     │  │
│  │ └────────────────────┘ │  └─────────────────────────────────┘  │
│  └──────────┬─────────────┘                                       │
│             │ spawn 子进程 / HTTP                                  │
└─────────────┼─────────────────────────────────────────────────────┘
              │
              ▼
   ┌──────────────────────────────┐
   │  宿主(用户已安装并登录)      │
   │  claude code / codex / ...   │
   │  收到 prompt → 写代码 → 返回  │
   └──────────────────────────────┘
```

### 六大 crate 职责

| crate | 职责 | 关键事实 |
|-------|------|---------|
| `super-dev` | binary 入口(clap CLI + doctor + hook) | 非 hook 命令已 hide,TUI 是唯一入口 |
| `super-dev-tui` | 全屏 TUI + 首启 picker + API 向导 + gate 交互 | ratatui,唯一用户界面 |
| `super-dev-agent` | 9 阶段流水线 + prompt 工程 + 状态机 | experts.rs 六套专家 prompt |
| `super-dev-runtime` | Runtime trait(唯一抽象)+ HTTP runtime | OpenAi/Anthropic/Offline 三实现 |
| `super-dev-host` | 2 个宿主 CLI(claude code/codex)驱动 | `claude --print "<prompt>"` |
| `super-dev-governance` | 治理规则 + 审计 + 合规映射 | fail-open,43 条条款 |

---

## 三、工作原理(端到端,从用户视角)

### 用户旅程:从安装到交付

```
1. 安装
   brew install super-dev / npm i -g super-dev
   用户已装好并登录 claude code(或 codex / 任意宿主)

2. 首次启动 (super-dev)
   ┌ 首启 picker(分组)─────────────────┐
   │ 已登录的 CLI:claude code(ready) │  ← 选这个(最常见)
   │              codex(ready)        │
   │              (claude-code / codex)│
   │ 接入第三方 API:DeepSeek/Ollama…  │  ← 没装 CLI 时走这条
   │ 离线模板                          │  ← demo/CI
   └──────────────────────────────────┘
   选择存入 ~/.super-dev/config.toml

3. 输入需求
   用户在 TUI 聊天框打:"做一个带邮箱登录的 SaaS 落地页"
   → 触发 9 阶段流水线

4. 流水线执行(super-dev 编排,宿主干活)
   ┌─ research ─ 宿主调研 → output/xxx-research.md ─┐
   │  docs     ─ 宿主写 → PRD + 架构 + UIUX 三份文档 │
   ╔═══════════════ GATE 1: docs_confirm ═══════════╗
   ║ 用户审核三份文档,按 c 继续 / 打字提修改意见      ║
   ╚═══════════════════╤════════════════════════════╝
   │  spec     ─ 宿主写执行计划                       │
   │  frontend ─ 宿主【真正创建前端项目代码文件】      │
   │            + verify 跑 npm install/build 验证    │
   ╔═════════════ GATE 2: preview_confirm ═══════════╗
   ║ 用户预览前端,按 c 继续 / 提修改                  ║
   ╚═══════════════════╤════════════════════════════╝
   │  backend  ─ 宿主【真正创建后端代码 + 测试】       │
   │  quality  ─ 跑构建/测试 + 质量评分(必须过门)    │
   │  delivery ─ 打包 proof-pack(合规证据)          │
   └──────────────────────────────────────────────┘

5. 交付
   output/ 下是规划文档 + 宿主生成的代码工程
   release/ 下是 proof-pack.zip(合规证据包)
   用户拿到一个能跑的应用 + 完整交付证据
```

### 每一阶段宿主收到的 playbook(来自 experts.rs 源码)

| 阶段 | prompt 角色 | 要求宿主做什么 | 产物 |
|------|------------|--------------|------|
| research | 调研员 | 调研需求、技术选型 | research.md |
| docs | PM/架构/UIUX | 写 PRD、架构、UIUX 设计令牌 | prd/architecture/uiux.md |
| spec | 技术负责人 | 写执行计划、任务拆分 | execution-plan.md |
| **frontend** | **资深前端** | **"Create REAL CODE FILES — components, pages, API client. Run build — fix all errors"** | **真实前端代码 + frontend-notes.md** |
| **backend** | **资深后端** | **"Create REAL CODE FILES — routes, models, middleware, tests. Run tests — fix all failures"** | **真实后端代码 + backend-notes.md** |
| quality | QA | 跑构建/测试 + 质量评分 | quality-gate.json/md |
| delivery | 发布 | 打包证据 | proof-pack.zip |

---

## 四、交互设计现状与完善

### 当前已有的交互(审计确认)

| 交互 | 状态 | 位置 |
|------|------|------|
| 首启分组 picker(宿主CLI/第三方API/离线) | ✅ 已实现 | app.rs default_picker_items |
| 第三方 API 对话式向导(12预设+实时验证) | ✅ 已实现 | app.rs ProviderWizard |
| 自定义模型 /provider 命令族 | ✅ 已实现 | app.rs slash_provider |
| 聊天输入需求触发流水线 | ✅ 已实现 | app.rs submit_text |
| GATE 审核(c 继续 / 打字修改) | ✅ 已实现 | app.rs submit_text |
| 斜杠命令(run/continue/revise/status/verify/doctor…) | ✅ 已实现 | app.rs SLASH_VERBS |
| 滚动 overlay(status/spec/verify/config…) | ✅ 已实现 | app.rs open_*_overlay |
| 流水线实时进度(阶段/产物/错误) | ✅ 已实现 | lib.rs event_loop |

### 交互缺口(基于"让用户真正交付商业应用"的目标)

这是这份文档最重要的部分。按"用户交付商业应用"的必需性排序:

#### 🔴 缺口 1:GATE 2(preview_confirm)没有真正给用户预览

**现状**:frontend 阶段 prompt 第 8 步是 "Run build"(编译验证),**但没有"启动 dev server 给用户预览"这一步**。GATE 2 叫"预览确认",但 playbook 没有指挥宿主产出可预览状态。全仓库无 `npm run dev`/`webbrowser`/`open` 代码。

**影响**:不懂代码的用户,前端代码生成完了,不知道怎么看效果。这是"服务非技术用户"的最大断点。

**完善方案**(符合治理层定位 —— 指挥宿主干,不自己越界):
1. frontend prompt 加第 9 步:"启动 dev server,确认无报错,把运行地址写进 frontend-notes.md 的 `## Preview URL` 段"
2. frontend-notes 模板加 "Preview URL" 字段
3. GATE 2 打开时,TUI 从 notes 读出 URL,push 提示:"预览已就绪:http://localhost:5173"
4. `/preview` 命令:读 notes 的 URL,`open` 浏览器(治理层可做的轻量辅助)

#### 🟡 缺口 2:没有"一键部署"指挥

**现状**:delivery 阶段只打 proof-pack(合规 zip),没有指挥宿主部署。

**影响**:用户拿到能跑的代码,但不知道怎么上线。

**方案**:delivery prompt 加"部署到 Vercel/Netlify"的可选步骤(指挥宿主执行,Super Dev 不自己调部署 API)。

#### 🟡 缺口 3:gate 审核时看的是 markdown,不够直观

**现状**:GATE 1 用户看 PRD/架构/UIUX 的 markdown;GATE 2 看 frontend-notes markdown。

**方案**:TUI 的 gate 卡片可以加"在 overlay 里渲染 markdown"(比纯文本更可读),但这是体验优化非必需。

#### 🟢 缺口 4:缺少"从零到上线"的引导提示

**现状**:新用户进来不知道该说什么需求、流水线会怎么走。

**方案**:首启进 chat 后,greeting 里给 1-2 个示例需求 + 流程说明。

---

## 五、真正让用户开发出商业应用 —— 完整能力清单

一个用户(尤其非技术用户)从"想法"到"商业应用上线",Super Dev 必须覆盖的环节:

| 环节 | Super Dev 的职责 | 现状 | 缺口 |
|------|-----------------|------|------|
| 1. 想法 → 需求 | 引导用户描述需求 | ✅ 聊天输入 | 🟢 缺示例引导 |
| 2. 需求 → 规划 | research + docs(宿主写文档) | ✅ 已有 | — |
| 3. 规划审核 | GATE 1 人工把关 | ✅ 已有 | 🟡 markdown 可读性 |
| 4. 规划 → 前端代码 | frontend 阶段(宿主写代码) | ✅ 已有 | — |
| 5. 前端预览 | GATE 2 让用户看效果 | 🔴 **缺失** | **缺口 1** |
| 6. 前端 → 后端代码 | backend 阶段(宿主写代码) | ✅ 已有 | — |
| 7. 质量验证 | quality 阶段(构建/测试/评分) | ✅ 已有 | — |
| 8. 部署上线 | delivery 阶段 | 🟡 只打包证据 | **缺口 2** |
| 9. 全程治理 | governance 实时拦截 + 审计 | ✅ 已有 | — |
| 10. 合规交付 | compliance 映射 | ✅ 已有 | — |

**结论:10 个环节里,8 个已具备,2 个是关键缺口(预览 + 部署)。补齐这两个,产品就形成"想法→上线"的完整闭环。**

---

## 六、实施路线(优先级排序)

### P0 — 补齐预览闭环(缺口 1)

这是让非技术用户能用的**最关键一步**。符合治理层定位(指挥宿主,不越界):

1. `experts.rs frontend_prompt`:加第 9 步"启动 dev server + 写 Preview URL"
2. `phases.rs run_frontend`:notes 模板加 `## Preview URL` 段
3. `app.rs`:GATE 2 打开时读 notes 的 URL,push 预览提示
4. `app.rs`:新增 `/preview` 命令(读 URL + open 浏览器)
5. 测试 + 全量验证

### P1 — 补齐部署闭环(缺口 2)

1. `experts.rs`:新增 delivery 部署 prompt 模板
2. delivery 阶段指挥宿主部署到 Vercel/Netlify(可选)
3. TUI 呈现部署后的 live URL

### P2 — 交互体验打磨

- gate 卡片 markdown 渲染
- 首启 greeting 示例需求
- 流水线进度可视化增强

---

## 七、硬实力护城河(竞品没有的)

基于调研(Lovable/bolt/v0/Cursor/通义灵码/CodeBuddy/Trae),这些**全都没有**:

1. **结构化 9 阶段流水线** —— 它们是"一句话直出",容易产出垃圾;Super Dev 强制调研→文档→审核→代码
2. **两道人工 gate** —— 防止 AI 一路狂奔,关键节点人把关
3. **实时 governance 拦截** —— 宿主写 emoji 图标/硬编码颜色/AI slop 时实时 block
4. **合规审计映射** —— SOC 2 / ISO 27001 / EU AI Act,企业级交付证据
5. **宿主可换** —— claude-code/codex + 自定义 API,流水线不变

**定位声明:Super Dev 不与 claude code 竞争(它驱动 claude code),不与 Lovable 竞争(它是流程层)。它让任意 AI 编码宿主的产出达到商业级标准。**

---

## 附录:源码审计证据索引

| 断言 | 证据位置 |
|------|---------|
| "coach for the host, does not write code" | spec/SUPER_DEV_HOST_SPEC_V1.md §coach metaphor |
| 9 阶段链条 | super-dev-spec/src/lib.rs:177-185 PHASE_CHAIN |
| frontend prompt 要求宿主写真实代码 | experts.rs:291 "Create REAL CODE FILES" |
| backend prompt 同理 | experts.rs:328 "Create REAL CODE FILES" |
| verify 跑构建验证 | verify.rs:86 verify_steps (install/test/build) |
| 43 条 spec 条款 | super-dev-spec/src/lib.rs (SD-CODE/FLOW/META/EVID) |
| 无预览/无 dev server/无浏览器打开 | 全仓库 grep 零结果 |
| governance fail-open | super-dev-governance/src/lib.rs:11 "MUST NEVER be blocked" |
| 2 个宿主后端(claude-code/codex) | super-dev-host/src/lib.rs BACKEND_IDS |
| 合规三框架 | compliance.rs:6-7 SOC2/ISO27001/AI Act |
