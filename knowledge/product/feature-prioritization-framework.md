---
id: feature-prioritization-framework
title: feature-prioritization-framework
domain: product
category: feature-prioritization-framework.md
difficulty: intermediate
tags: [feature, framework, prioritization, product]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 功能优先级决策框架（深度版）

### 目标
- 统一需求排期口径，避免“拍脑袋优先级”导致资源浪费。

### 四维评分模型
- 业务价值：是否直接推动收入、留存、效率提升。
- 用户价值：是否解决高频痛点或关键任务阻塞。
- 实现成本：研发、测试、运维、培训与迁移成本。
- 风险敞口：合规、安全、稳定性、依赖不确定性。

### 评分规则
- 每维 1-5 分，最终分数 = (业务价值+用户价值)×权重 - (实现成本+风险敞口)×权重。
- 默认权重：业务价值0.35、用户价值0.25、实现成本0.2、风险0.2。
- 分数相近时，以“减少关键链路风险”优先。

### 决策门禁
- 涉及账号、支付、权限、数据导出功能时，风险评分必须由安全与运维双签。
- 涉及跨系统变更时，必须有回滚方案与兼容策略。
- 未给出可量化验收标准的需求不得进入开发。

### 输出模板
- 需求名称
- 预期业务影响
- 用户受益群体
- 估算工作量
- 依赖项与风险
- 发布策略与回滚方案
