# Super Dev

<div align="center">

<img src="docs/assets/super-dev-logo.png" alt="Super Dev — A coach for AI coding hosts" width="600">

### AI 编码宿主的教练与编排器 · 一份规范、一个二进制、一个 TUI

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.87%2B-orange)](https://www.rust-lang.org/)
[![Spec](https://img.shields.io/badge/spec-SUPER__DEV__HOST__SPEC__V1-blue)](spec/SUPER_DEV_HOST_SPEC_V1.md)
[![Version](https://img.shields.io/badge/version-4.6.0-success)](CHANGELOG.md)

[English](README_EN.md) | 简体中文

</div>

---

## Super Dev 是什么

`Super Dev` 是 **AI 编码宿主的教练与编排器**——它本身不写代码、不是 IDE。它把一份完整的商业项目交付规范（[SUPER_DEV_HOST_SPEC_V1](spec/SUPER_DEV_HOST_SPEC_V1.md)）按 9 阶段流水线落地：先研究什么、产出哪些工件、什么时候停下来等用户确认、什么文件不许写、留下什么审计证据。

**核心模式:驱动你已登录的宿主 CLI。** Super Dev 把用户已经安装并登录的 AI 编码 CLI 当作按需调用的执行后端——**不需要 API key**,吃的是你已有的宿主订阅。终端里敲 `super-dev` 就进入 ratatui 实时进度界面,正常对话即可。

一个二进制、零运行时依赖、单文件分发。

## 两个核心事实

- **我们不调任何大模型 API**——Super Dev 自己是确定性的 Rust 状态机
- **真正写代码的是你电脑上已登录的 AI 编码 CLI**——我们只把它们当工人调度

| 模式 | 命令 | 需要 API key | 说明 |
|---|---|---|---|
| **宿主 CLI**（推荐） | `--backend claude-code` 等 | **否** | 驱动你已登录的宿主 CLI,吃现有订阅 |
| **离线** | （默认,无 flag） | 否 | 确定性模板,无网络（演示 / CI 用） |

> 1.0 早期版本曾内置直调 Anthropic/OpenAI/Antigravity HTTP API,4.0 起**彻底移除**——Super Dev 是"项目经理",不是"AI 客户端"。

## 驱动两个宿主 CLI

Super Dev 深度适配并驱动两个宿主——你装了哪个就用哪个,**不需要 API key**,吃的是你已有的宿主订阅:

| Backend ID | 二进制 | 非交互调用形式 | TUI 命令 |
|---|---|---|---|
| `claude-code` | `claude` | `claude --print --output-format text "<p>"` | `/claude` |
| `codex` | `codex` | `codex exec --sandbox workspace-write "<p>"` | `/codex` |

没有这两个 CLI?在 TUI 里用 `/provider setup` 接入第三方 API(DeepSeek / OpenRouter / Ollama / 智谱 / 百炼等,自带 key),或 `/offline` 跑确定性模板。运行 `super-dev doctor` 查看本机宿主就绪情况。

## 安装

```bash
# ★ 推荐:一行装好(npm 自动挑你平台的预编译二进制,无需 Node 之外的依赖)
npm install -g super-dev
```

或者:

```bash
# 从源码构建(需要 Rust 1.75+)
git clone https://github.com/shangyankeji/super-dev.git
cd super-dev && cargo build --release
cp target/release/super-dev /usr/local/bin/
```

跨平台 npm 分发支持 macOS(Intel + Apple Silicon)、Linux(x86_64 + ARM64)、Windows x86_64——一个 `super-dev` 命令,五个平台的预编译 Rust 二进制各发一个子包(`@super-dev/cli-<platform>-<arch>`),npm 只会安装匹配你机器的那个。

## 从安装到上线(完整教程)

```bash
# 1. 装 Super Dev
npm install -g super-dev

# 2. 装并登录一个宿主(二选一,或用 /provider 接第三方 API)
npm install -g @anthropic-ai/claude-code   # 然后 claude 登录
# 或
npm install -g @openai/codex               # 然后 codex 登录

# 3. 进项目目录,启动 TUI
cd my-project
super-dev

# 4. 首启选择器:选已登录的宿主(claude-code / codex),
#    或"接入第三方 API"(向导引导,实时验证 key),
#    或"离线模板"。

# 5. 在 TUI 里输入需求,例如:
#    "做一个带邮箱登录的 SaaS 落地页"
#    → 流水线跑 research → docs
#    → 停在 GATE 1:审核 PRD/架构/UIUX 三份文档
#    → 输入 c 继续到 spec → frontend
#    → 停在 GATE 2:前端已生成

# 6. 预览前端效果
/preview        # 自动启动 dev server + 打开浏览器

# 7. 满意后继续到 backend → quality → delivery
c               # 在 GATE 2 输入 c 继续
#    → 宿主写后端代码 + 跑生产构建 + 产出部署指令

# 8. 一键部署上线
/deploy         # 执行宿主记录的部署命令(如 npx vercel --prod)
#    → 抓到 live URL,应用上线

# 9. 自检部署就绪状态
super-dev doctor   # 检测交付产物 / 部署命令 / 平台 CLI 是否就绪
```

**整个流程:输入一句话需求 → 审核 → 预览 → 部署上线。** 宿主(claude code/codex)负责写代码,Super Dev 负责流程编排 + 治理(禁 emoji 图标/硬编码颜色/敏感路径写操作)+ 预览 + 部署。

## 用法

```bash
# ★ 最简用法:一行进 TUI,自动探测已登录的 claude-code / codex
super-dev

# ★ 最简形式 —— 直接进 TUI,在欢迎屏输入需求即可
super-dev

# 初始化 workspace(写出 super-dev.yaml spec manifest;首次使用时跑一次)
super-dev init

# === 命令行流水线(脚本 / CI 用)===
super-dev run "做一个登录系统"                            # 离线确定性模板
super-dev run "..." --backend claude-code                # 驱动宿主 CLI,无需 key
super-dev run "..." --backend codex
super-dev continue                                       # 批准当前 gate 并继续
super-dev revise "把首屏换成深色"                         # 留在当前 gate 请求修订

# === 帮助 / 探索 ===
super-dev examples                                       # 完整 cheat-sheet
super-dev guide                                          # 60 秒走读
super-dev install --host claude-code  # wire real-time governance hook
super-dev doctor                                         # 自检

# === 状态与规范 ===
super-dev verify                                         # 验证 workspace 合规
super-dev spec [--clauses]                               # 打印规范本体 / clause 表
super-dev report                                         # 生成 SD-EVID-004 合规映射
```

## 规范覆盖（25 条 clause × 4 层）

详见 [SUPER_DEV_HOST_SPEC_V1.md](spec/SUPER_DEV_HOST_SPEC_V1.md)。

- **Layer 1 — 代码权重**（4 条）：emoji 禁令、颜色 token、API 路径对齐、技术栈预研
- **Layer 2 — 流程契约**（6 条）：phase chain、docs_confirm / preview_confirm 两道 gate、修订语义、会话连续性
- **Layer 3 — 交付产物**（6 条）：PRD / 架构 / UIUX / Spec / 任务 / ADR
- **Layer 4 — 证据链**（5 条）：API 审计、tool-call 审计、quality 报告、SOC2 / ISO27001 / EU AI Act 合规映射、proof-pack

## 项目结构

```
super-dev/
├── Cargo.toml                 # workspace 清单
├── crates/
│   ├── super-dev/             # 主二进制（CLI + TUI 入口）
│   ├── super-dev-spec/        # 规范的 Rust 数据表达
│   ├── super-dev-governance/  # rules / audit / context / compliance 核心
│   ├── super-dev-agent/       # 9 阶段流水线 + gates + state + 事件流 + manifest
│   ├── super-dev-runtime/     # Runtime trait + OfflineRuntime（确定性兜底）
│   ├── super-dev-host/        # HostDriver —— 驱动已登录的 claude-code / codex
│   ├── super-dev-contract/    # OpenAPI 3.1 契约层（派生 / 解析 / 校验）
│   ├── super-dev-knowledge/   # BM25 + 可选向量（混合）知识检索
│   └── super-dev-tui/         # ratatui 终端应用
├── spec/
│   └── SUPER_DEV_HOST_SPEC_V1.md   # 规范本体（normative）
├── knowledge/                 # 治理知识库
├── super-dev-website/         # Next.js 官网（独立工程）
└── docs/assets/               # README 与 spec 用的图片
```

## 开发

```bash
# 全量编译
cargo build --workspace

# 全量测试
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets

# 格式化
cargo fmt --all
```

## License

MIT —— 见 [LICENSE](LICENSE)。
