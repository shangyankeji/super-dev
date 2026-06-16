---
id: kpi-and-metric-tree
title: kpi-and-metric-tree
domain: product
category: kpi-and-metric-tree.md
difficulty: intermediate
tags: [and, kpi, metric, product, tree, 与指标树方法库]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## KPI 与指标树方法库

### 目标
- 建立从战略目标到页面行为指标的可追踪链路。

### 指标层级
- 北极星指标：反映核心价值交付。
- 一级指标：新增、活跃、留存、收入、成本。
- 二级指标：转化率、漏斗流失率、成功率、时延、错误率。
- 过程指标：页面点击、操作耗时、失败类型分布。

### 指标树构建步骤
- 明确业务目标与目标时间窗。
- 拆解关键用户行为路径。
- 为每个路径节点定义成功与失败指标。
- 绑定阈值与预警规则。

### 指标验收规则
- 每个新功能至少绑定 1 个业务指标和 2 个质量指标。
- 指标必须有口径说明、数据源、更新频率、负责人。
- 指标上生产前必须完成一次历史基线对齐。

### 常见失败模式
- 指标只统计访问量，不统计任务成功率。
- 指标口径跨团队不一致，导致决策冲突。
