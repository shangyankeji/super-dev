---
id: structured-logging-playbook
title: 结构化日志实战手册
domain: observability
category: 02-playbooks
difficulty: intermediate
tags: [observability, logging, structured-logging, request-id, correlation, json, tracing, elk, loki]
quality_score: 90
maintainer: platform-team@superdev.com
last_updated: 2024-06-14
---

# 结构化日志实战手册

## 为什么用结构化日志

```
# ❌ 非结构化（无法搜索/聚合/告警）
[2024-06-14 10:30:00] INFO User 123 created order 456 for $99.50

# ✅ 结构化 JSON（可搜索/聚合/告警/关联）
{"ts":"2024-06-14T10:30:00Z","level":"INFO","service":"order","requestId":"req_abc","userId":"123","action":"order.created","orderId":"456","amount":99.50,"duration_ms":45}
```

## 必须贯穿的 Request ID

```python
# 中间件：每个请求生成唯一 requestId，注入所有日志
@app.middleware("http")
async def request_id_middleware(request, call_next):
    request_id = request.headers.get("X-Request-ID") or str(uuid.uuid4())
    # 存入 contextvar，所有日志自动带上
    log_context.set({"requestId": request_id})
    response = await call_next(request)
    response.headers["X-Request-ID"] = request_id
    return response

# 使用时自动带 requestId
log.info("Order created", orderId="456", amount=99.50)
# → {"requestId":"req_abc","orderId":"456","amount":99.50,...}
```

## 日志级别决策树

```
用户操作成功 → INFO
预期内的异常（重试/降级） → WARN
需要人工干预的故障 → ERROR
开发调试细节 → DEBUG（生产关闭）
```

## 敏感信息脱敏

```python
# ❌ 日志泄露密码/token
log.info("User login", email=email, password=password)  # 泄露！

# ✅ 脱敏
log.info("User login", email=mask_email(email))  # "j***@example.com"
# 密码/token 永远不记日志
```

## 日志聚合架构

```
应用 → stdout → 容器日志驱动 → Fluent Bit → Loki/Elasticsearch → Grafana/Kibana
```

**关键原则：**
- 应用只写 stdout（不直接写文件/网络）
- 采集器（Fluent Bit）负责转发
- 查询在 Grafana/Kibana 做
- 保留策略：ERROR 90 天 / INFO 30 天 / DEBUG 不保留
