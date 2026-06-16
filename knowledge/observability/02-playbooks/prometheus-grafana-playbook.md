---
id: prometheus-grafana-playbook
title: Prometheus + Grafana 监控告警实战手册
domain: observability
category: 02-playbooks
difficulty: advanced
tags: [prometheus, grafana, monitoring, alerting, slo, red, use, golden-signals, alertmanager, dashboard, metrics, enterprise]
quality_score: 95
maintainer: platform-team@superdev.com
last_updated: 2026-06-15
---

# Prometheus + Grafana 监控告警实战手册

> 基于 [Grafana Alerting Best Practices](https://grafana.com/docs/grafana/latest/alerting/guides/best-practices/) + [Grafana Dashboard Best Practices](https://grafana.com/docs/grafana/latest/visualizations/dashboards/build-dashboards/best-practices/)

## 四大黄金信号

### 服务的 RED 指标（每个端点必须）
```
Rate     — 每秒请求数（QPS）
Errors   — 错误率（5xx / 总请求）
Duration — 延迟分布（p50/p95/p99）
```

### 基础设施的 USE 指标
```
Utilization — 使用率（CPU/内存/磁盘）
Saturation  — 饱和度（队列长度/连接池）
Errors      — 错误（丢包/重传/失败）
```

## Prometheus 指标暴露

```python
# Python 应用暴露 /metrics
from prometheus_client import Counter, Histogram, generate_latest

REQUESTS = Counter('http_requests_total', 'Total requests', ['method', 'path', 'status'])
LATENCY = Histogram('http_request_duration_seconds', 'Request latency',
    ['path'],
    buckets=[0.01, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10]  # 自定义桶
)

@app.get("/metrics")
def metrics():
    return generate_latest()

@app.middleware("http")
async def track_metrics(request, call_next):
    start = time.time()
    response = await call_next(request)
    duration = time.time() - start
    REQUESTS.labels(method=request.method, path=request.url.path, status=response.status_code).inc()
    LATENCY.labels(path=request.url.path).observe(duration)
    return response
```

## SLO + 错误预算告警

```yaml
# ❌ 静态阈值（不知道是正常波动还是真的故障）
alert: HighErrorRate
expr: rate(http_requests_total{status=~"5.."}[1m]) > 0.01

# ✅ 基于错误预算的 SLO 告警（Google SRE 方法）
# SLO: 99.9% 的请求在 30 天内成功
# 错误预算: 0.1% × 30天 = 43.2 分钟的允许故障时间

# 快速燃烧（1小时耗 2% 预算 → P1）
alert: SLOBurnRateFast
expr: |
  (
    sum(rate(http_requests_total{status=~"5.."}[5m]))
    / sum(rate(http_requests_total[5m]))
  ) > (14.4 * 0.001)  # 14.4x 标准燃烧率
for: 2m

# 慢速燃烧（6小时耗 5% 预算 → P2）
alert: SLOBurnRateSlow
expr: |
  (
    sum(rate(http_requests_total{status=~"5.."}[1h]))
    / sum(rate(http_requests_total[1h]))
  ) > (3 * 0.001)  # 3x 标准燃烧率
for: 15m
```

## 告警原则（防 Alert Fatigue）

1. **告症状不告原因** — "用户下单失败率高" > "CPU 80%"
2. **每个告警有 runbook** — 告警描述里含处理链接
3. **按优先级分级** — P1（PagerDuty）/ P2（Slack）/ P3（邮件）
4. **用 `for` 持续时间过滤抖动** — 持续 5 分钟才告警
5. **抑制规则** — 父告警触发时抑制子告警（DB 挂 → 抑制所有 API 告警）

```yaml
# Alertmanager 抑制规则
inhibit_rules:
- source_match: { severity: critical }
  target_match: { severity: warning }
  equal: ['service']  # 同服务的 critical 抑制 warning
```

## Grafana 仪表盘设计

### 黄金仪表盘布局
```
┌─────────────────────────────────────────┐
│ Row 1: 概览（错误率 + p95 延迟 + QPS）    │
├─────────────────────────────────────────┤
│ Row 2: 错误细节（按端点/状态码分组）       │
├─────────────────────────────────────────┤
│ Row 3: 延迟分布（p50/p95/p99 曲线）       │
├─────────────────────────────────────────┤
│ Row 4: 资源使用（CPU/内存/连接池）        │
├─────────────────────────────────────────┤
│ Row 5: 业务指标（订单量/收入/活跃用户）    │
└─────────────────────────────────────────┘
```

### 阈值标注
- 绿色：< SLO 目标
- 黄色：接近 SLO 边界
- 红色：超过 SLO（需关注）

## 生产检查清单
- [ ] 每个端点暴露 RED 指标
- [ ] 每个资源暴露 USE 指标
- [ ] SLO 定义 + 错误预算告警
- [ ] 告警有 runbook + 优先级分级
- [ ] 抑制规则（防告警风暴）
- [ ] Grafana 仪表盘（概览 + 细节 + 业务）
- [ ] 告警静默窗口（维护期不告警）
- [ ] 指标保留 15 天（长期用 Thanos/Mimir）
