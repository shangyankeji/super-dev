---
id: incident-command-system
title: incident-command-system
domain: operations
category: incident-command-system.md
difficulty: intermediate
tags: [command, incident, operations, system]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 事故指挥体系（ICS）手册

### 目标
- 在生产事故中实现统一指挥、快速止损、清晰协同。

### 角色定义
- 指挥官：统一决策与优先级管理。
- 技术负责人：定位根因与恢复方案执行。
- 沟通负责人：对内对外同步状态与影响。
- 记录员：维护时间线与关键决策记录。

### 响应流程
- 事故分级：按用户影响与业务损失快速定级。
- 首轮止损：优先保护关键交易链路与数据一致性。
- 分工执行：并行推进定位、恢复、沟通。
- 持续更新：固定时间窗发布状态更新。
- 关闭与复盘：确认恢复、收敛风险、输出复盘。

### 执行门禁
- P1及以上事故必须启用指挥角色。
- 关键决策必须有时间戳与责任人。
- 事故结束后24小时内提交初版复盘。

### 常见失败模式
- 没有单一指挥导致多头决策冲突。
- 恢复后不复盘，问题重复发生。
