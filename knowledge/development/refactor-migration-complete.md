---
id: refactor-migration-complete
title: refactor-migration-complete
domain: development
category: refactor-migration-complete.md
difficulty: intermediate
tags: [complete, development, migration, refactor, 重构与迁移完整知识库]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 重构与迁移完整知识库

### 1. 重构触发条件
- 缺陷密度持续偏高。
- 修改成本显著高于同类模块。
- 性能与稳定性难以通过局部优化解决。

### 2. 重构路径
- 增量重构优先，避免一次性大爆炸改造。
- 先加测试护栏，再改代码结构。
- 关键路径采用双写或灰度迁移策略。

### 3. 迁移策略
- 数据迁移必须定义校验与回滚脚本。
- 接口迁移必须提供兼容层与下线计划。
- 配置迁移必须支持快速回退。

### 4. 风险控制
- 每次变更控制在可评审范围内。
- 高风险模块重构必须双人评审。
- 发布采用分阶段放量并观察核心指标。

### 5. 验收标准
- 功能回归全部通过。
- 性能与稳定性指标不劣化。
- 技术债指标明显下降并可量化。

### 6. 常见失败模式
- 重构与新功能并行交叉，导致目标失焦。
- 迁移缺少兼容窗口，造成调用方中断。
