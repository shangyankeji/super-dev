---
id: knowledge-gates-execution
title: knowledge-gates-execution
domain: development
category: 13-implementation-assets
difficulty: intermediate
tags: [development, execution, gates, knowledge, 知识门禁执行手册]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 知识门禁执行手册

### 目标
- 把开发知识规则接入发布链路，避免知识与交付脱节。

### 本地执行
- 运行开发域审计：`python3 scripts/audit_development_kb.py`
- 运行知识门禁：`python3 scripts/check_knowledge_gates.py --project-dir .`
- 生成报告：`--format json|md|html|junit --out artifacts/...`
- 生成阶段包：`python3 scripts/generate_lifecycle_packet.py --project-dir . --name your-packet`

### CI执行
- Jenkins在测试后执行Knowledge Gates阶段。
- 任一门禁失败必须阻断后续构建与发布阶段。

### 失败处理
- 缺失条目：补齐对应目录与catalog索引。
- 规则缺项：更新UI门禁或场景矩阵文件。
- 全流程缺项：补齐stage-exit-criteria流程阶段定义。
- 模板缺项：补齐15-lifecycle-templates模板目录与阶段模板文件。
- 头信息缺失：补齐文档首行规范。
- 内容陈旧：按stale阈值优先刷新高风险条目。
- 主题重复：合并重复主题，保留单一事实源。

### 通过标准
- 审计达到知识库级。
- 场景矩阵包含八类场景。
- UI门禁包含核心八项指标。
- 全流程阶段必须覆盖需求到复盘闭环。
- 全流程模板必须覆盖需求到复盘交付物。
- 关键门禁无缺失项且综合评分达到阈值。
