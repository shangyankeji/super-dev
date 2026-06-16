---
id: knowledge-map
title: knowledge-map
domain: 00-governance
category: knowledge-map.md
difficulty: intermediate
tags: [00-governance, knowledge, map]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 知识库地图（全环节）

### 1. 目标
- 建立可持续演进的项目知识系统，减少“只靠人记忆”带来的交付风险。
- 让需求增强、文档生成、评审与上线决策都能引用统一知识源。

### 2. 生命周期映射
- 战略与机会：`product/`
- 设计与体验：`design/`
- 架构与实现：`architecture/`、`development/`
- 质量与安全：`testing/`、`security/`
- 交付与运维：`cicd/`、`operations/`
- 数据与增长：`data/`
- 事故与复盘：`incident/`
- AI与自动化：`ai/`

### 3. 强关联关系
- `product` 决定 `architecture` 的边界与非功能指标。
- `architecture` 约束 `development` 的实现模式。
- `testing` 与 `security` 共同定义 `cicd` 的门禁。
- `operations` 的监控与事故数据反哺 `product` 与 `architecture`。
- `ai` 的安全与评测要求同时受 `security/testing/operations` 约束。

### 4. 版本与审查
- P0条目：月度审查。
- P1条目：季度审查。
- P2条目：半年度审查。
