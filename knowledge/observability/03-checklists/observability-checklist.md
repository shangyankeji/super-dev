---
id: observability-checklist
title: 可观测性上线清单
domain: observability
category: 03-checklists
difficulty: beginner
tags: [observability, checklist, logging, metrics, alerting, health-check, dashboard, distributed-tracing]
quality_score: 87
maintainer: platform-team@superdev.com
last_updated: 2024-06-14
---

# 可观测性上线清单

## 日志
- [ ] 所有日志是 JSON 结构化格式
- [ ] 每条日志有 requestId（贯穿请求链路）
- [ ] 日志级别正确（INFO/WARN/ERROR）
- [ ] 敏感信息（密码/token/PII）已脱敏
- [ ] 日志写 stdout（不写文件）
- [ ] 日志采集器（Fluent Bit/Promtail）已配置

## 指标
- [ ] 每个端点有 RED 指标（Rate/Errors/Duration）
- [ ] 数据库连接池使用率有指标
- [ ] 队列深度有指标（如使用消息队列）
- [ ] 业务指标定义（注册量/订单量/收入）
- [ ] 指标暴露在 `/metrics`（Prometheus 格式）

## 告警
- [ ] 5xx 错误率 > 1% → PagerDuty P1
- [ ] p95 延迟 > 500ms → PagerDuty P2
- [ ] 数据库连接池 > 80% → Slack
- [ ] 磁盘空间 < 10% → PagerDuty P1
- [ ] 告警有 runbook（怎么处理）
- [ ] 没有 alert fatigue（误报率 < 5%）

## 健康检查
- [ ] `/api/health` 返回服务状态 + 依赖检查
- [ ] 健康检查包含数据库连接测试
- [ ] 健康检查包含关键依赖测试（Redis/外部API）
- [ ] 容器编排用健康检查做重启决策

## 仪表盘
- [ ] 服务总览仪表盘（QPS/延迟/错误率/饱和度）
- [ ] 业务指标仪表盘（日活/订单/收入）
- [ ] 基础设施仪表盘（CPU/内存/磁盘/网络）
- [ ] 仪表盘有阈值标注（绿色/黄色/红色区间）

## 分布式追踪（多服务时）
- [ ] 请求头传播 traceparent
- [ ] 数据库查询有 span
- [ ] 外部 API 调用有 span
- [ ] span 有关键业务标签（userId/orderId）
