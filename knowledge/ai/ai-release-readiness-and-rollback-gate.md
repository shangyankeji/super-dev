---
id: ai-release-readiness-and-rollback-gate
title: ai-release-readiness-and-rollback-gate
domain: ai
category: ai-release-readiness-and-rollback-gate.md
difficulty: intermediate
tags: [ai, ai发布就绪与回滚门禁, and, gate, readiness, release, rollback]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## AI发布就绪与回滚门禁

### 目标
- 将AI能力发布纳入可量化门禁，确保上线可控与可回退。

### 适用范围
- 适用于模型升级、提示词变更、工具链变更和策略变更场景。

### 发布门禁
- 准确性门禁：核心场景任务成功率达到目标阈值。
- 安全性门禁：高危攻击用例通过率达到要求。
- 性能门禁：P95时延与错误率满足SLO。
- 成本门禁：单请求成本和日预算占比在阈值内。

### 执行清单
- 灰度流量策略、放量节奏、观察窗口已定义。
- 功能开关可快速关闭，老版本可快速回切。
- 发布责任人与应急联系人明确。

### 验收标准
- 四类门禁全部通过且无阻断缺陷。
- 发布后观察期内关键指标稳定。

### 常见失败模式
- 只看离线效果，忽略真实流量下的成本和失败率。
- 回滚脚本未演练，故障发生时恢复缓慢。

### 回滚策略
- 任一P0指标越线立即回滚到上一稳定版本。
- 回滚后冻结变更并触发根因复盘。
