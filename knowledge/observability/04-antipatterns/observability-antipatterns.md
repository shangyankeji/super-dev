---
id: observability-antipatterns
title: 可观测性反模式（避坑指南）
domain: observability
category: 04-antipatterns
difficulty: intermediate
tags: [observability, antipattern, logging, metrics, alerting, tracing, dashboard, structured-logging, cardinality, noise]
quality_score: 87
maintainer: platform-team@superdev.com
last_updated: 2024-06-14
---

# 可观测性反模式（避坑指南）

## 1. 日志里打印敏感数据
```python
# ❌ 泄露用户密码 / API key / PII
logger.info(f"User login: email={email}, password={password}, token={jwt}")

# ✅ 只记非敏感字段，脱敏 PII
logger.info("User login", email=mask_email(email), auth_method="jwt")
```

## 2. 高基数指标标签（cardinality bomb）
```python
# ❌ userId 做标签 → 每个用户一个时间序列，10 万用户 = 内存爆炸
metrics.counter("requests", labels={"userId": user_id})

# ✅ 低基数标签（status/method/path），userId 放日志
metrics.counter("requests", labels={"method": "GET", "status": "200"})
```

## 3. Alert Fatigue（告警风暴）
```yaml
# ❌ 任何 500 都告警 → 每天上百条，团队无视
alert: http_500
condition: rate(http_requests{status="500"}[1m]) > 0

# ✅ 错误率超阈值 + 持续时间才告警
alert: http_500_rate
condition: |
  rate(http_requests{status="500"}[5m]) / rate(http_requests[5m]) > 0.01
for: 5m
```

## 4. 无 Request ID 关联
```python
# ❌ 日志无法关联到同一请求
logger.info("Received request")
# ... 50 行后 ...
logger.info("Database query failed")  # 哪个请求？

# ✅ Request ID 贯穿
logger.info("Received request", requestId=req_id)
logger.info("Database query failed", requestId=req_id)  # 可关联！
```

## 5. 日志当业务数据用
```python
# ❌ 从日志里解析订单（日志可能丢/截断/格式变）
log.info(f"ORDER_CREATED|{order.id}|{order.amount}")

# ✅ 业务事件走数据库/消息队列，日志只做可观测
db.insert(order)
event_bus.publish("order.created", order)
log.info("Order created", orderId=order.id)  # 辅助
```

## 6. 只看平均延迟
```
# ❌ 平均延迟 100ms 看起来很好
avg = 100ms

# ✅ p95/p99 才反映真实体验
p50 = 50ms   # 一半请求很快
p95 = 800ms  # 但 5% 的用户等近 1 秒！
p99 = 3000ms # 1% 的用户等 3 秒
```

## 7. 无 dashboard 直接看原始日志
```
# ❌ 出事故时 grep 日志（慢、无法聚合、无法看趋势）
kubectl logs api-server | grep ERROR | wc -l

# ✅ Grafana 仪表盘（实时趋势、错误率曲线、延迟分布）
```

## 8. 健康检查只返回 200
```json
// ❌ 数据库挂了但健康检查还是 200
GET /health → 200 OK

// ✅ 检查依赖，降级状态
GET /health → 200, {"db":"ok","redis":"degraded","status":"degraded"}
GET /health → 503, {"db":"down","status":"unhealthy"}
```

## 9. 不设日志保留策略
```
# ❌ 日志永久保留 → 磁盘满 → 服务挂
# ✅ 分级保留
ERROR: 90 天
WARN:  30 天
INFO:  14 天
DEBUG: 不保留（生产关闭）
```
