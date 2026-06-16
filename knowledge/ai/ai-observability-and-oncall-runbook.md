---
id: ai-observability-and-oncall-runbook
title: ai-observability-and-oncall-runbook
domain: ai
category: ai-observability-and-oncall-runbook.md
difficulty: intermediate
tags: [ai, ai可观测性与值班runbook, and, observability, oncall, runbook]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## AI可观测性与值班Runbook

### 目标
- 建立AI系统运行态监控、告警、处置、复盘的标准流程。

### 适用范围
- 适用于AI线上服务、Agent编排服务与RAG检索生成服务。

### 观测指标
- 质量指标：任务成功率、拒答率、幻觉率、用户满意度。
- 稳定性指标：错误率、超时率、重试率、可用性。
- 性能指标：P50/P95时延、工具调用耗时、队列长度。
- 经济指标：token消耗、单请求成本、预算消耗速率。

### 告警分级
- P0：大面积不可用、越权风险、成本突增失控。
- P1：核心指标持续劣化、成功率显著下降。
- P2：局部场景异常、可降级运行。

### 执行清单
- 每个核心指标绑定负责人、阈值与处置动作。
- 每类告警都有首响SLA与升级路径。
- 值班交接必须包含未闭环告警与临时策略变更。

### 处置清单
- 先止损：关闭高风险能力或降级到安全兜底。
- 再定位：按模型、检索、工具、策略层分段排查。
- 后恢复：灰度恢复并持续观察关键指标。

### 验收标准
- 重大告警MTTR达到目标。
- 告警误报率与漏报率有持续下降趋势。

### 常见失败模式
- 只监控系统指标，不监控AI质量与成本漂移。
- 告警无分级，值班噪声过多导致真实事故漏检。

### 回滚策略
- 按开关逐层回退：策略层→路由层→模型层→功能层。
- 回滚完成后进行复盘并沉淀预防动作。
