---
id: observability-standards
title: 可观测性标准（日志/指标/追踪）
domain: observability
category: 01-standards
difficulty: intermediate
tags: [observability, logging, metrics, tracing, monitoring, alerting, structured-logging, request-id]
quality_score: 90
maintainer: platform-team@superdev.com
last_updated: 2026-06-14
---

# 可观测性标准（日志 / 指标 / 追踪）

## 三柱可观测性

### 1. 结构化日志
```json
{
  "timestamp": "2024-06-14T10:30:00.123Z",
  "level": "INFO",
  "service": "order-service",
  "requestId": "req_abc123",
  "userId": "usr_xyz",
  "message": "Order created",
  "orderId": "ord_456",
  "amount": 99.50,
  "duration_ms": 45
}
```

**必须包含的字段：**
- `timestamp`（ISO-8601 UTC）
- `level`（DEBUG / INFO / WARN / ERROR）
- `requestId`（贯穿整个请求链路）
- `service`（服务名）
- `message`（人类可读描述）

**日志级别使用：**
- DEBUG：开发调试（默认关闭）
- INFO：正常业务流程（订单创建、用户注册）
- WARN：异常但可恢复（重试成功、降级触发）
- ERROR：需要人工干预（数据库连接失败、外部 API 超时）

### 2. 指标
```
# RED 指标（每个端点必须）
http_requests_total{method="GET", path="/api/orders", status="200"} 1500
http_request_duration_seconds_bucket{path="/api/orders", le="0.1"} 1200
http_requests_errors_total{path="/api/orders", error="500"} 3

# 业务指标
orders_created_total{plan="pro"} 42
payment_amount_total{currency="USD"} 4200.50
```

**关键指标：**
- **RED**：Rate（请求量）、Errors（错误率）、Duration（延迟分布）
- **USE**：Utilization（资源使用）、Saturation（饱和度）、Errors（错误）
- **业务**：注册量、订单量、收入、活跃用户

### 3. 分布式追踪
```
Trace: req_abc123
├── span: POST /api/orders (45ms)
│   ├── span: validate_input (2ms)
│   ├── span: db.insert_order (15ms)
│   ├── span: payment.charge (20ms)
│   └── span: enqueue_notification (1ms)
```

**追踪头传播：**
```http
traceparent: 00-abcdef1234567890-1234567890-01
```

## 告警规则

| 指标 | 阈值 | 动作 |
|------|------|------|
| 5xx 错误率 | > 1% 持续 5min | PagerDuty P1 |
| p95 延迟 | > 500ms 持续 10min | PagerDuty P2 |
| 429 速率限制 | > 100/min 持续 5min | Slack 通知 |
| 数据库连接池 | > 80% 使用 | Slack 通知 |
| 磁盘空间 | < 10% 剩余 | PagerDuty P1 |

## 健康检查端点

```json
GET /api/health

{
  "status": "healthy",
  "service": "order-service",
  "version": "2.1.0",
  "uptime": 3600,
  "checks": {
    "database": "ok",
    "redis": "ok",
    "external_api": "degraded"
  }
}
```
