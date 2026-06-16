---
id: adr-template-and-examples
title: adr-template-and-examples
domain: architecture
category: adr-template-and-examples.md
difficulty: intermediate
tags: [adr, and, architecture, examples, template, 模板与示例规范]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## ADR 模板与示例规范

### 目标
- 对关键架构决策形成可追溯记录，降低后续误解与返工。

### ADR 标准结构
- 背景：当前问题、约束、目标。
- 备选方案：至少 2 个可行方案及优劣对比。
- 决策结果：选择方案与核心理由。
- 影响评估：性能、成本、复杂度、组织影响。
- 风险与回滚：失败情境与回退路径。
- 验证计划：上线前后验证指标。

### 命名规范
- `ADR-YYYYMMDD-主题.md`
- 示例：`ADR-20260306-api-versioning-strategy.md`

### 必填检查项
- 是否定义受影响模块清单。
- 是否明确兼容性与迁移策略。
- 是否给出最终废弃策略与时间点。
- 是否定义发布后观测指标。

### 常见失败模式
- 只写结论不写权衡过程。
- 没有更新历史 ADR 状态，造成“多版本真相”。
