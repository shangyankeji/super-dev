---
id: maintenance-policy
title: 知识库维护政策
domain: 00-governance
category: 01-standards
difficulty: intermediate
tags: [00-governance, maintenance, policy, lifecycle, quality]
quality_score: 88
last_updated: 2026-06-16
---

# 知识库维护政策

## 所有权与责任矩阵

| 角色 | 职责 | 频率 |
|---|---|---|
| **知识库所有者** (platform-team) | 审批新增/删除域，维护 00-governance/ 规范 | 季度 |
| **域维护者** (per-domain) | 审批域内文件变更，保证质量分 ≥ 70 | 双周 |
| **贡献者** (any agent/user) | 提交新文件或修订，附完整 front-matter | 随时 |

## 文件生命周期

### 新增
1. 文件必须包含完整 front-matter（`id`, `title`, `domain`, `tags`, `quality_score`, `difficulty`）。
2. 文件字数 ≥ 200 词（CJK 按 1.5× 折算）；低于阈值的标记 `status: draft`。
3. 新增域需经知识库所有者审批，且初始至少 3 个文件。
4. `quality_score` 初值 = 70，贡献者可自行调整。

### 修订
1. 修订后必须更新 `last_updated` 日期。
2. 内容变更超过 30% 时 `version` 字段 +1。
3. 修订不得删除已有 H2 章节（只能标记 deprecated）。

### 归档与删除
1. 连续 2 个季度无修订且无检索命中的文件 → 标记 `status: archived`。
2. 归档文件移入 `<domain>/99-archived/` 子目录，不参与默认检索。
3. 仅知识库所有者可执行物理删除，需在变更记录中写明原因。

## 质量门禁

- 每个文件的 `quality_score` 必须 ≥ 50；低于 50 的文件不进入 BM25 索引。
- 每季度全库扫描：`quality_score < 60` 的文件通知域维护者复审。
- front-matter 完整性检查：`tags` 非空、`domain` 与路径一致、`quality_score` 为整数。

## 检索权重规则

- `quality_score` 作为 BM25 分数的弱加权：`score × (1 + quality/200)`。
- `difficulty = advanced` 的文件在同分时优先（企业级场景更需深度知识）。
- `.super-dev/learned/` 下的经验文件 `quality_score` 固定为 80（项目专属高价值）。

## 失效治理

- 每周检测失效链接并替换或移除。
- 每月清理重复条目与过期策略。
- 每季度做知识结构重组，保持检索可读性。
- 索引缓存（`bm25.bin` / `vectors.bin`）通过 content-hash 自动失效，无需手动清除。

## front-matter 兼容性

- schema 变更必须向后兼容（新字段 `#[serde(default)]`）。
- 旧格式文件（无 `quality_score`）通过 `cargo run --example backfill-frontmatter -- --fix` 补齐。
- 三种 front-matter 历史格式（legacy/numbered/complete）均被 chunker 兼容解析。

## 升级流程

1. 在 `docs/plans/` 创建升级方案文档。
2. 知识库所有者审批。
3. 执行变更（新增/修订/归档）。
4. 运行 `cargo test -p super-dev-knowledge` 验证索引构建。
5. 更新 `docs/ARCHITECTURE.md` 如有结构性变更。

## 变更记录

每次更新追加"变更摘要"，说明原因、影响范围、回归检查结果。变更记录写入
`00-governance/changelog.md`（按需创建）。
