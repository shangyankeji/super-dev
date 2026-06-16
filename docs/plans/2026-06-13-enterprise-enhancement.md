# Super Dev 企业级全方位增强 — 设计与路线

> 日期: 2026-06-13
> 状态: 阶段 1 已交付 / 阶段 2-4 路线已定
> 约束: 纯 Rust · 三运行时(Anthropic/OpenAI/Antigravity)· fail-open 治理 · 离线单文件

## 动机

4.5 的 Super Dev 在每一层都停留在 vibecoding 玩具深度:

| 维度 | vibecoding 玩具(4.5) | 企业级落地(目标) |
|---|---|---|
| 字段级 | 验收标准是 `Given TODO when TODO` 占位符 | 结构化字段 + 类型 + 约束 + 校验规则 |
| 契约级 | API 表是 `| TODO | /api/... | TODO |` 模板 | OpenAPI 3.1、字段级 validation、破坏性变更检测 |
| 证据级 | quality gate 只查"文件存在 + 正则" | 依赖审计、SAST、覆盖率门、签名、可追溯链 |
| 工程化 | 9 阶段线性跑完出 zip | 多环境、CI/CD 生成、IaC、回滚、迁移 |
| 知识库 | 一个文件夹 + 关键词子串排序 | BM25 倒排索引 + 可选向量语义检索 |
| 编排 | 线性单 prompt 单阶段 | 子任务并行、增量交付、回滚、技术债台账 |

## 六支柱闭环

```
①契约层(Contract)   → openapi.yaml 单一真相源,替换 Markdown API 表      [阶段1 ✓]
②知识库(Knowledge)  → BM25 默认 + 可选 OpenAI 嵌入,替换"一个文件夹"     [阶段1 ✓]
③治理引擎(Governance)→ swc AST 真代码分析,替换正则                        [阶段2]
④证据链(Evidence)   → 内容哈希 + 测试向量,替换"文件存在+计数"            [阶段2]
⑤工程化(Delivery)   → CI/Docker/migrations/IaC 真生成,替换散文           [阶段3 ✓]
⑥编排(Orchestration)→ SubTask 事件 + 回滚 + 技术债台账                  [阶段4 ✓]
```

## 关键决策记录

### D1: MSRV 提升 1.75 → 1.87
解锁 `swc_ecma_parser`(TSX AST)、`oas3`/OpenAPI 3.1、`hnsw_rs`(向量索引)。纯 Rust /
三运行时 / fail-open 三条架构硬规则不变,只是编译器版本。本机 1.95.0 完全支持。

### D2: 嵌入模型 = 混合(BM25 默认 + 可选 OpenAI 嵌入)
不烘焙模型进二进制(违反离线 + 25-100MB 太重);不引入 ort/ONNX C++ FFI(违反纯 Rust)。
默认 BM25 词法检索(零依赖、离线);用户可选配 `OPENAI_EMBED_KEY` 走已有 OpenAI 订阅的
text-embedding-3-small 语义检索。无 key 自动降级 BM25。

---

## 阶段 1(已交付):契约层 + 知识库 RAG

### 新增 crate: `super-dev-knowledge`(替换"一个文件夹")

| 模块 | 职责 |
|---|---|
| `tokenizer.rs` | 中英文混合分词:ASCII ≥2 char + CJK 字符 bigram。**修复 4.5 的核心 bug——纯 CJK 需求(`做一个登录系统`)在旧 ASCII-only 关键词提取器下产生 0 个 token,直接降级为字典序兜底。** |
| `chunker.rs` | Markdown 感知分块:识别 YAML front-matter / blockquote 摘要 / 纯 H1 三种格式,按 `## H2` 切分,每块带 `{path, title, section, tags, domain}` 元数据。 |
| `index.rs` | BM25 倒排索引(k1=1.2, b=0.75),序列化到 `.super-dev/kb-index/bm25.bin`,首次构建后复用。 |
| `vector.rs` | 可选 OpenAI 嵌入层。无 `OPENAI_EMBED_KEY` → 纯空实现,所有方法 no-op。有 key → cosine 检索。fail-open:网络失败返回空,BM25 兜底。 |
| `retrieve.rs` | 统一入口 `retrieve_for_phase()`,阶段感知过滤(每个 phase 映射到相关 knowledge 子目录)。 |

**对接(零破坏 ABI):** `phase_knowledge_digest()` / `knowledge_top_files()` 签名不变,内部改调新
crate。`.superdevrc` 新增 `[knowledge]` 段(enabled / engine / top_k),旧 keyword 路径作为
`enabled = false` 时的显式回退保留。

### 新增 crate: `super-dev-contract`(OpenAPI 3.1 单一真相源)

| 模块 | 职责 |
|---|---|
| `parse.rs` | 从 architecture Markdown 的 API 表抽取出类型化 `ApiSpec`(`Endpoint { method, path, operation_id, security }`)。升级 4.5 的 `split('|')` 脆弱解析器,支持路径模板匹配(`/api/users/:id`)。 |
| `extract.rs` | 扫描 worker 产出的前端源码的 `fetch`/`axios`/`ky`/`useSWR` 调用,返回类型化 `(method, path)`。升级 audit.rs 的 bare `Vec<String>`。 |
| `validate.rs` | 真契约校验:`validate_frontend_vs_contract` 检查每个前端调用命中契约(方法+路径模板);`validate_prd_vs_contract` 检查 PRD 路由覆盖。替换 4.5 的子串匹配。 |
| `render.rs` | 渲染 `openapi.json` + `openapi.yaml` 到 `.super-dev/contracts/`(spec §3.3 SD-CODE-003 引用但从未生成)。 |

**对接:** `run_quality` 新增 4 个 `QualityCheck`:OpenAPI 契约存在、前端↔契约一致性、PRD 路由↔
契约覆盖、无占位符内容(权重 3.0,critical)。

### "无占位符"硬门(防 vibecoding)
扫描 `output/*.md` 的 `TODO` / `| TODO |` / `Given TODO` / `Lorem ipsum`。4.5 的离线模板
(`render_prd`/`render_architecture`)满是 TODO——这条门会强制走真实 worker backend。
计数 ≥4 → critical failure,阻断 delivery。

### 测试向量(spec §1.4 引用但从未存在)
新增 `tests/spec_vectors/SD-CODE-{001,002,003}.json`——`(file_path, content) → expected_decision`
元组,任何实现都能跑来验证治理层。

---

## 阶段 2(路线):治理引擎 AST 化 + 证据链硬化

- **swc AST 治理**:emoji/color/slop 从正则升级为 swc TSX AST 分析。1.87 MSRV 已解锁。
- **完整 spec_vectors**:补齐 SD-FLOW-* / SD-ART-* / SD-EVID-* 的向量文件 + 向量运行器(读 JSON,跑治理函数,断言 decision)。
- **内容哈希证据**:`ClauseEvidence.evidence` 携带工件内容 SHA-256,而非仅文件路径 + fired_count。

## 阶段 3(路线):工程化骨架生成

- 从 OpenAPI 契约生成 CI/CD yaml(`.github/workflows/`)、Dockerfile、DB migrations、IaC(Terraform/Pulumi 最小集)。
- 扩展 proof-pack 纳入这些 ops 工件。
- 新增 ops 工件存在性质量门(Dockerfile present / migrations non-empty / CI workflow valid)。

## 阶段 4(路线):编排增强

- `SubTask` 模型:一个 phase 产出 N 个子任务,`futures::join_all` 扇出(前端/后端/测试并行,多微服务)。
- `WorkflowState` 扩展:历史快照(`.super-dev/snapshots/<run-id>/`)支持回滚。
- 技术债台账(`.super-dev/tech-debt.jsonl`):占位符位置、严重度、负责人、解决状态。
- `EngineEvent` 新增 `SubTaskStarted/Completed` 变体,TUI 渲染子任务进度。


---

## 阶段 3(已交付):工程化骨架生成

### 新增模块: `super-dev-agent/src/scaffolding.rs`
从 OpenAPI 契约 + 检测的技术栈,生成真实的 ops 工件到项目根目录(不是散文 checklist):

| 工件 | 内容 |
|---|---|
| `Dockerfile` | 多阶段构建,匹配检测的栈(Node/Rust/Python/Go) |
| `docker-compose.yml` | app + postgres:16 + healthcheck |
| `.github/workflows/ci.yml` | lint → test → build,内嵌 **Super Dev 质量门步骤**(CI 在质量门未过时失败) |
| `migrations/0001_init.sql` | 从 openapi 路径派生表(/api/users → users 表),含 auth 列 |
| `.env.example` | DATABASE_URL/JWT_SECRET/PORT + 栈特定变量 + 端点计数 |

**技术栈检测**:优先 manifest 文件(Cargo.toml→Rust, go.mod→Go, pyproject.toml→Python, package.json→Node),回退到 architecture doc 关键词(axum/fastapi/express 等)。

**接入点**:在 `run_quality` 开头生成(质量门检查之前),这样 "Ops artifacts present" 检查能看到它们。delivery 阶段把 ops 工件纳入 proof-pack。

**端到端验证**(真实 workspace):Node 栈检测正确 → Dockerfile/CI/migrations/.env/compose 全生成 → 质量门 "Ops artifacts present" = passed 100 → proof-pack 含 Dockerfile + ci.yml + .env.example。



---

## 阶段 4(已交付):编排增强

### 4a 技术债台账(`tech_debt.rs` + `.super-dev/tech-debt.jsonl`)
把 `count_placeholder_markers` 的单数字升级为结构化持久台账:
- [`scan_debt`] 扫描 `output/*.md`,返回 `DebtItem { file, line, kind, snippet, first_seen }`。
- 5 种债务类型(`Todo`/`TodoCell`/`Placeholder`/`FillerText`/`UnfilledAcceptance`),按严重度加权(1-5)。
- [`write_ledger`] 追加到 `.super-dev/tech-debt.jsonl`(每行一个 JSON 对象),跨 run 累积。
- [`summarise`] 给出按类型计数 + 严重度总分,供 `super-dev report` 使用。

### 4b WorkflowState 历史/快照回滚(`state.rs` + CLI)
每次 `write_workflow_state` 自动把上一个状态快照到 `.super-dev/history/<纳秒时间戳>.json`,transition 不再是破坏性的:
- `super-dev history` — 列出所有可回滚快照(显示每个会恢复到哪个 phase)。
- `super-dev rollback latest` — 撤销最后一次 transition(`workflow-state.json` 回退一个快照)。
- `super-dev rollback 20260614T12` — 按前缀匹配精确回滚。
- 纳秒时间戳避免同秒连续 transition 的文件名冲突。
- 注意:磁盘工件不回退,只回退流水线状态(下一步 `continue` 从该 phase 恢复)。

### 4c SubTask 事件 + TUI 渲染(`events.rs` + `runner.rs` + `app.rs`)
新增 `EngineEvent::SubTaskStarted`/`SubTaskCompleted` 变体,让 pipeline 内部的子任务对 TUI 可见:
- frontend 和 backend 的 worker 调用现在都发射 SubTask 事件(task_id: `frontend.implement` / `backend.implement`)。
- TUI 在 chat 流里渲染 `▸ ... started` / `✓ ... done`,无需新 UI 元素。
- 诚实说明:当前 backend↔quality 有真实数据依赖(quality 扫 backend 产出的代码),所以阶段 4 没有强行并行化它们——SubTask 事件是为未来多微服务 fan-out(`tokio::join!`)预留的可观测性基础设施。


---

## 阶段 1 交付验证标准

1. ✅ `cargo test --workspace` 全绿(295 测试,含 51 knowledge + 41 contract)。
2. ✅ `cargo clippy --workspace --all-targets -- -D warnings` 零警告。
3. ✅ BM25 检索对"做一个登录系统"(CJK)稳定召回 `security/login`,而非字典序兜底(头条 bug 修复,有专门测试)。
4. ✅ `output/<slug>-architecture.md` 旁产出 `.super-dev/contracts/openapi.{json,yaml}`,质量门对它的校验是真路径模板 + 方法匹配。
5. ✅ 含 `| TODO |` 的离线模板工件被新质量门判为 critical failure,迫使走真实 backend。
6. ✅ 两个新 crate 的 lib.rs 头部、Cargo.toml、workspace 注册全部符合现有约定。
