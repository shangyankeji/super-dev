---
id: risk-based-test-matrix
title: risk-based-test-matrix
domain: testing
category: risk-based-test-matrix.md
difficulty: intermediate
tags: [based, matrix, risk, test, testing, 风险驱动测试矩阵]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 风险驱动测试矩阵

### 目标
- 把测试资源聚焦到高风险高影响场景，提升回归效率与事故预防能力。

### 风险评分模型
- 业务影响：低/中/高
- 变更复杂度：低/中/高
- 历史缺陷密度：低/中/高
- 依赖外部系统数量：低/中/高

### 测试等级映射
- R1（低风险）：单元测试 + 冒烟验证。
- R2（中风险）：单元 + 集成 + 契约测试。
- R3（高风险）：单元 + 集成 + E2E + 性能 + 安全专项。

### 执行清单
- 每次发布前更新模块风险评分。
- 高风险模块回归失败时禁止发布。
- 高风险缺陷必须沉淀为长期回归用例。

### 常见失败模式
- 所有模块“一刀切”同等测试深度。
- 风险评估不更新，矩阵失去参考价值。
