---
id: release-readiness-gate
title: release-readiness-gate
domain: cicd
category: release-readiness-gate.md
difficulty: intermediate
tags: [cicd, gate, readiness, release, 发布就绪门禁清单]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 发布就绪门禁清单

### 目标
- 用统一发布门禁阻断高风险变更，保证发布可控。

### 必过门禁
- 代码质量：lint、类型检查、静态分析通过。
- 测试质量：核心回归通过、阻断缺陷清零。
- 安全质量：高危漏洞清零，密钥泄露检查通过。
- 构建质量：制品可追溯、版本标签一致。
- 运维质量：发布与回滚脚本可执行。

### 发布前检查
- 是否完成变更影响评估与回滚预案。
- 是否完成灰度策略与阈值配置。
- 是否完成关键指标基线确认。

### 发布后检查
- 错误率、时延、成功率是否在阈值内。
- 告警是否异常增加。
- 关键业务链路是否连续成功。

### 常见失败模式
- 只检查构建成功，不检查运行质量。
- 门禁定义存在但未强制执行。
