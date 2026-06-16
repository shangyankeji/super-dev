# Super Dev — 企业级架构梳理

> 版本 4.6.0 · 9 个 Rust crate · 600+ 测试 · 0 clippy 警告 · 纯 Rust 零外部进程依赖

## 一句话定位

Super Dev 是一份 AI 编码交付规范的执行体——它把 25 条商业级交付标准注入到你
已登录的 AI 编码 CLI（Claude Code / Codex），并在事后审计它是否
达标。**它自己不调任何大模型 API**，吃的是你现有的 CLI 订阅。

---

## 架构全景（9 个 crate，数据自上而下流动）

```
┌─────────────────────────────────────────────────────────────┐
│  super-dev（二进制）                                          │
│  clap CLI · hook · install · doctor · init · report         │
└────────────┬───────────────────────────────┬────────────────┘
             │                                │
     ┌───────▼────────┐           ┌──────────▼──────────┐
     │ super-dev-tui  │           │ super-dev-agent     │
     │ ratatui 实时UI │           │ 9 阶段流水线 runner │
     └────────────────┘           │ gates · state ·     │
                                  │ scaffolding ·       │
                                  │ lessons · tech_debt │
                                  └──┬──────┬───────┬───┘
                   ┌──────────────────┘      │       └────────────────┐
           ┌───────▼────────┐      ┌────────▼─────────┐    ┌──────────▼────────┐
           │ super-dev-     │      │ super-dev-       │    │ super-dev-        │
           │ governance     │      │ knowledge        │    │ contract          │
           │ 规则+审计+合规 │      │ BM25+向量 RAG    │    │ OpenAPI 3.1 层    │
           └────────────────┘      └──────────────────┘    └───────────────────┘
                   │
           ┌───────▼────────┐      ┌──────────────────┐
           │ super-dev-     │      │ super-dev-       │
           │ spec           │      │ host             │
           │ 25 条 clause   │      │ 子进程驱动 claude-code/codex  │
           │ (真相源)       │      └──────────────────┘
           └────────────────┘              │
                                   ┌──────▼──────┐
                                   │super-dev-   │
                                   │runtime      │
                                   │Runtime trait│
                                   └─────────────┘
```

## 六大支柱

| 支柱 | Crate | 职责 | 当前状态 |
|---|---|---|---|
| **规范** | super-dev-spec | 25 条 clause × 4 层（CODE/FLOW/ART/EVID）+ 9 阶段 | ✅ |
| **治理** | super-dev-governance | emoji/颜色/slop 检查 + API 审计 + 合规映射 + 实时 hook | ✅ |
| **知识** | super-dev-knowledge | BM25 倒排索引 + 可选向量(hybrid) RRF 融合 + 370 文件语料 | ✅ |
| **契约** | super-dev-contract | 类型化 OpenAPI 3.1 + 前端一致性 + PRD 覆盖率校验 | ✅ |
| **证据** | super-dev-agent | verify 真测试序列 + quality gate 22+ 检查 + SHA-256 哈希 | ✅ |
| **编排** | super-dev-agent | 9 阶段流水线 + 2 道人工 gate + 经验闭环回流 | ✅ |

## 9 阶段流水线

```
research → docs → [docs_confirm gate] → spec → frontend → [preview_confirm gate]
    → backend → quality → delivery
```

每阶段：读知识库(BM25/向量) → 组 prompt → 调 Runtime → 写工件 → maybe_verify
两道 gate：暂停等用户 `super-dev continue`
质量门：22+ 检查，build/test 失败 = critical，阻断 delivery（SD-EVID-003）

## 知识库 RAG 架构

```
用户需求
  ↓ pre-embed query (async, fail-open to BM25)
  ↓
retrieve_with_vector(project_root, knowledge_dir, cfg, query, phase, qvec)
  ├─ BM25: 倒排索引 over knowledge/ + .super-dev/learned/ + ~/.super-dev/learned/
  ├─ Vector: 向量存储 .super-dev/kb-index/vectors.bin (content-hash 增量缓存)
  ├─ RRF fusion: 1/(60+rank) 合并两路排名
  └─ quality_score 弱加权 → 返回 top-K chunks
```

- 默认 BM25（离线零依赖）
- 配置 `engine = "hybrid"` + `OPENAI_API_KEY` → 真 HTTP embedding (reqwest pooled)
- 每轮失败经验 → sediment → 索引 → 下轮 coach prompt 注入（闭环）

## 治理双轨制

| 轨道 | 机制 | 适用宿主 |
|---|---|---|
| **实时守门** | `super-dev install` → Claude Code PreToolUse hook | Claude Code only |
| **硬阻断** | quality gate passed:false → 拒绝推进 delivery | 所有宿主 |

诚实承诺：Codex/Gemini 无法实时拦截，靠硬阻断兜底。

## verify 真测试序列

```
Node:  install → lint → typecheck → test → build
Rust:  fmt --check → clippy → test → build --release
Python: install → ruff → mypy → pytest
Go:    vet → test → build
Deno:  lint → test → check
```
缺失 binary → skipped（非 fail），build/test 失败 → critical

## 配置体系

```toml
# .superdevrc（super-dev init 自动生成）
[quality]
threshold = 90
skip_checks = []

[pipeline]
skip_phases = []
max_review_rounds = 3

[knowledge]
enabled = true
engine = "bm25"     # or "hybrid"
top_k = 6
```

## 运行时健壮性

- 子进程超时 → 显式 kill（防孤儿）
- stdout 256 KiB 截断（防 OOM）
- reqwest 连接池（OnceLock<reqwest::Client>）
- RuntimeError::Timeout 结构化变体（非字符串匹配）
- 所有失败 fail-open 到离线模板（永不阻塞宿主）

## 合规映射

每轮自动生成 `output/<slug>-compliance-mapping.json`：
- 25 条 clause → SOC 2 / ISO 27001:2022 / EU AI Act 映射
- 关键工件 SHA-256 内容哈希（防篡改）
- `super-dev report` 输出项目健康度摘要

## 测试覆盖

- **573 单元 + 集成测试**，0 失败
- **0 clippy 警告**（`-D warnings`）
- **4 个 spec vector**（SD-CODE-001/002/003/004 conformance）
- **11 个 e2e 集成测试**（hook / install / report / doctor / pipeline）
- 端到端验证：run → continue → delivery，8 工件 + proof-pack zip
