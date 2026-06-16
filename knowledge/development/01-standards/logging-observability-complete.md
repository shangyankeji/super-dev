---
id: logging-observability-complete
title: 日志与可观测性指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [agent, checklist, complete, development, logging, observability, 实战代码示例, 常见陷阱]
quality_score: 70
last_updated: 2026-06-15
---
# 日志与可观测性指南

## 概述
可观测性(Observability)是理解分布式系统内部状态的能力,由日志(Logs)、指标(Metrics)、追踪(Traces)三大支柱组成。本指南覆盖结构化日志、ELK/Loki、OpenTelemetry、分布式追踪的完整实践。

## 核心概念

### 1. 可观测性三支柱
- **日志(Logs)**: 离散事件记录,描述"发生了什么"
- **指标(Metrics)**: 聚合数值,描述"系统整体状态"
- **追踪(Traces)**: 请求链路,描述"请求经过了哪里"

### 2. 日志级别规范
| 级别 | 用途 | 示例 |
|------|------|------|
| DEBUG | 开发调试,生产关闭 | 变量值、函数入参 |
| INFO | 正常业务事件 | 用户登录、订单创建 |
| WARNING | 异常但可恢复 | 重试成功、配额接近上限 |
| ERROR | 失败需要关注 | API调用失败、数据库连接断开 |
| CRITICAL | 系统级故障 | 数据库不可用、OOM |

### 3. 工具栈对比

| 方案 | 日志 | 指标 | 追踪 | 成本 | 适用规模 |
|------|------|------|------|------|----------|
| ELK | Elasticsearch+Logstash+Kibana | Elastic Metrics | Elastic APM | 高(ES资源) | 中大型 |
| PLG | Loki+Promtail+Grafana | Prometheus | Tempo | 低 | 中小型 |
| OpenTelemetry + 后端 | 灵活对接 | 灵活对接 | 灵活对接 | 看后端 | 任意 |
| Datadog/New Relic | 全集成 | 全集成 | 全集成 | 按量付费 | 企业级 |

### 4. OpenTelemetry
- CNCF标准,厂商中立的遥测数据收集框架
- 支持Logs/Metrics/Traces三种信号
- 自动和手动埋点
- 导出到任意后端(Jaeger/Zipkin/Datadog/Grafana)

## 实战代码示例

### 结构化日志(Python)

```python
# 使用structlog实现结构化日志
import structlog
import logging
import sys

def configure_logging(environment: str = "production"):
    """配置结构化日志"""
    # 处理器链
    shared_processors = [
        structlog.contextvars.merge_contextvars,
        structlog.stdlib.add_log_level,
        structlog.stdlib.add_logger_name,
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.StackInfoRenderer(),
        structlog.processors.UnicodeDecoder(),
    ]

    if environment == "development":
        # 开发环境: 彩色终端输出
        renderer = structlog.dev.ConsoleRenderer()
    else:
        # 生产环境: JSON格式
        renderer = structlog.processors.JSONRenderer()

    structlog.configure(
        processors=[
            *shared_processors,
            structlog.stdlib.ProcessorFormatter.wrap_for_formatter,
        ],
        logger_factory=structlog.stdlib.LoggerFactory(),
        wrapper_class=structlog.stdlib.BoundLogger,
        cache_logger_on_first_use=True,
    )

    formatter = structlog.stdlib.ProcessorFormatter(
        processors=[
            structlog.stdlib.ProcessorFormatter.remove_processors_meta,
            renderer,
        ],
    )

    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(formatter)
    root_logger = logging.getLogger()
    root_logger.addHandler(handler)
    root_logger.setLevel(logging.INFO)

# 使用
logger = structlog.get_logger()

async def create_order(user_id: int, items: list):
    log = logger.bind(user_id=user_id, action="create_order")
    log.info("order_creation_started", item_count=len(items))

    try:
        order = await order_repo.create(user_id, items)
        log.info("order_created", order_id=order.id, total=order.total)
        return order
    except InsufficientStockError as e:
        log.warning("order_creation_failed", reason="insufficient_stock", product_id=e.product_id)
        raise
    except Exception as e:
        log.error("order_creation_error", error=str(e), error_type=type(e).__name__)
        raise
```

### 请求上下文追踪

```python
# FastAPI中间件 — 请求ID和上下文注入
import uuid
import structlog
from starlette.middleware.base import BaseHTTPMiddleware
from contextvars import ContextVar

request_id_var: ContextVar[str] = ContextVar("request_id", default="")

class RequestContextMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request, call_next):
        request_id = request.headers.get("X-Request-ID", str(uuid.uuid4()))
        request_id_var.set(request_id)

        # 绑定结构化日志上下文
        structlog.contextvars.clear_contextvars()
        structlog.contextvars.bind_contextvars(
            request_id=request_id,
            method=request.method,
            path=request.url.path,
            client_ip=request.client.host,
        )

        logger = structlog.get_logger()
        logger.info("request_started")

        import time
        start = time.perf_counter()

        try:
            response = await call_next(request)
            duration = time.perf_counter() - start

            logger.info(
                "request_completed",
                status_code=response.status_code,
                duration_ms=round(duration * 1000, 2),
            )

            response.headers["X-Request-ID"] = request_id
            return response
        except Exception as e:
            duration = time.perf_counter() - start
            logger.error(
                "request_failed",
                error=str(e),
                error_type=type(e).__name__,
                duration_ms=round(duration * 1000, 2),
            )
            raise

app.add_middleware(RequestContextMiddleware)
```

### OpenTelemetry集成

```python
# OpenTelemetry完整配置
from opentelemetry import trace, metrics
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanExporter
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
from opentelemetry.sdk.resources import Resource, SERVICE_NAME
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.exporter.otlp.proto.grpc.metric_exporter import OTLPMetricExporter
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor
from opentelemetry.instrumentation.httpx import HTTPXClientInstrumentor
from opentelemetry.instrumentation.sqlalchemy import SQLAlchemyInstrumentor
from opentelemetry.instrumentation.redis import RedisInstrumentor

def setup_telemetry(service_name: str, otlp_endpoint: str = "http://otel-collector:4317"):
    """初始化OpenTelemetry"""
    resource = Resource.create({
        SERVICE_NAME: service_name,
        "deployment.environment": os.getenv("ENV", "production"),
        "service.version": os.getenv("APP_VERSION", "unknown"),
    })

    # Traces
    tracer_provider = TracerProvider(resource=resource)
    tracer_provider.add_span_processor(
        BatchSpanExporter(OTLPSpanExporter(endpoint=otlp_endpoint))
    )
    trace.set_tracer_provider(tracer_provider)

    # Metrics
    metric_reader = PeriodicExportingMetricReader(
        OTLPMetricExporter(endpoint=otlp_endpoint),
        export_interval_millis=60000,
    )
    meter_provider = MeterProvider(resource=resource, metric_readers=[metric_reader])
    metrics.set_meter_provider(meter_provider)

    # 自动埋点
    FastAPIInstrumentor.instrument()
    HTTPXClientInstrumentor().instrument()
    SQLAlchemyInstrumentor().instrument(engine=engine)
    RedisInstrumentor().instrument()

# 手动埋点
tracer = trace.get_tracer(__name__)
meter = metrics.get_meter(__name__)

# 自定义指标
order_counter = meter.create_counter(
    "orders.created",
    unit="1",
    description="Number of orders created",
)
order_duration = meter.create_histogram(
    "orders.creation_duration",
    unit="ms",
    description="Order creation duration",
)

async def create_order(user_id: int, items: list):
    with tracer.start_as_current_span("create_order") as span:
        span.set_attribute("user.id", user_id)
        span.set_attribute("order.item_count", len(items))

        start = time.perf_counter()
        try:
            # 子Span: 库存检查
            with tracer.start_as_current_span("check_inventory"):
                await check_inventory(items)

            # 子Span: 支付处理
            with tracer.start_as_current_span("process_payment"):
                payment = await process_payment(user_id, calculate_total(items))
                span.set_attribute("payment.id", payment.id)

            # 子Span: 创建订单记录
            with tracer.start_as_current_span("save_order"):
                order = await save_order(user_id, items, payment)

            span.set_attribute("order.id", order.id)
            span.set_status(trace.StatusCode.OK)
            order_counter.add(1, {"status": "success"})
            return order

        except Exception as e:
            span.set_status(trace.StatusCode.ERROR, str(e))
            span.record_exception(e)
            order_counter.add(1, {"status": "error"})
            raise
        finally:
            duration = (time.perf_counter() - start) * 1000
            order_duration.record(duration)
```

### Prometheus指标暴露

```python
# FastAPI + prometheus-client
from prometheus_client import (
    Counter, Histogram, Gauge, Info,
    generate_latest, CONTENT_TYPE_LATEST,
)
from starlette.responses import Response

# 定义指标
REQUEST_COUNT = Counter(
    'http_requests_total',
    'Total HTTP requests',
    ['method', 'endpoint', 'status_code'],
)
REQUEST_DURATION = Histogram(
    'http_request_duration_seconds',
    'HTTP request duration',
    ['method', 'endpoint'],
    buckets=[0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
)
ACTIVE_REQUESTS = Gauge(
    'http_requests_active',
    'Active HTTP requests',
)
APP_INFO = Info(
    'app',
    'Application info',
)
APP_INFO.info({
    'version': '1.2.0',
    'environment': os.getenv('ENV', 'production'),
})

# 业务指标
DB_POOL_SIZE = Gauge('db_pool_size', 'Database connection pool size')
CACHE_HIT_RATE = Gauge('cache_hit_rate', 'Cache hit rate percentage')

class MetricsMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request, call_next):
        ACTIVE_REQUESTS.inc()
        method = request.method
        path = request.url.path

        with REQUEST_DURATION.labels(method=method, endpoint=path).time():
            try:
                response = await call_next(request)
                REQUEST_COUNT.labels(
                    method=method,
                    endpoint=path,
                    status_code=response.status_code,
                ).inc()
                return response
            except Exception as e:
                REQUEST_COUNT.labels(
                    method=method, endpoint=path, status_code=500
                ).inc()
                raise
            finally:
                ACTIVE_REQUESTS.dec()

@app.get("/metrics")
async def metrics_endpoint():
    return Response(
        content=generate_latest(),
        media_type=CONTENT_TYPE_LATEST,
    )
```

### Grafana Loki日志查询

```promql
# LogQL查询示例

# 查看特定服务的错误日志
{service="order-service"} |= "error"

# JSON解析并过滤
{service="order-service"} | json | level="error" | duration_ms > 1000

# 统计5分钟内的错误率
sum(rate({service="order-service"} |= "error" [5m])) /
sum(rate({service="order-service"} [5m])) * 100

# 查看特定请求ID的完整链路
{service=~".*"} |= "request_id=abc-123"

# P99延迟
quantile_over_time(0.99,
  {service="order-service"} | json | unwrap duration_ms [5m]
)
```

### 告警规则配置

```yaml
# Prometheus告警规则
groups:
  - name: application
    rules:
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{status_code=~"5.."}[5m])) /
          sum(rate(http_requests_total[5m])) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate ({{ $value | humanizePercentage }})"
          description: "Error rate exceeds 5% for 5 minutes"

      - alert: HighLatency
        expr: |
          histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) > 2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High P99 latency ({{ $value }}s)"

      - alert: DatabaseConnectionPoolExhausted
        expr: db_pool_size < 5
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "DB connection pool nearly exhausted"

  - name: infrastructure
    rules:
      - alert: PodCrashLooping
        expr: rate(kube_pod_container_status_restarts_total[15m]) > 0
        for: 15m
        labels:
          severity: warning

      - alert: HighMemoryUsage
        expr: |
          container_memory_usage_bytes / container_spec_memory_limit_bytes > 0.9
        for: 10m
        labels:
          severity: warning
```

## 最佳实践

### 1. 日志规范
- 使用结构化日志(JSON),而非纯文本
- 每条日志包含: timestamp、level、service、request_id、message
- 敏感数据脱敏(密码/Token/身份证号)
- 避免在循环中打日志(性能影响)
- 日志级别在运行时可调(不需重启)

### 2. 指标设计
- 遵循RED方法: Rate(请求率)、Errors(错误率)、Duration(延迟)
- 遵循USE方法: Utilization(利用率)、Saturation(饱和度)、Errors
- 标签基数控制(避免高基数标签如user_id)
- 使用Histogram而非Summary(更灵活)

### 3. 追踪策略
- 采样率根据流量调整(高流量服务降低采样率)
- 关键路径100%采样(支付/登录)
- 传播Context跨服务(W3C Trace Context标准)
- Span名称语义化(http.get /api/users而非handler_123)

### 4. 告警原则
- 告警应可操作(收到告警知道该做什么)
- 避免告警疲劳(控制数量和阈值)
- 分层告警(Warning→仪表盘,Critical→通知)
- 定期review告警规则(删除无人处理的告警)

### 5. 数据保留策略
- 日志: 热数据7天,温数据30天,冷存储90天
- 指标: 原始数据15天,降采样数据1年
- 追踪: 7-30天(按采样率和存储预算)

## 常见陷阱

### 陷阱1: 日志中包含敏感数据
```python
# 错误
logger.info("User login", email=email, password=password)
logger.info("API call", headers=dict(request.headers))  # 可能含Auth头

# 正确
logger.info("User login", email=mask_email(email))
safe_headers = {k: v for k, v in request.headers.items() if k.lower() != "authorization"}
logger.info("API call", headers=safe_headers)
```

### 陷阱2: 指标标签爆炸
```python
# 错误: user_id作为标签,百万用户=百万时间序列
REQUEST_COUNT = Counter('requests', 'Total requests', ['user_id'])

# 正确: 使用低基数标签
REQUEST_COUNT = Counter('requests', 'Total requests', ['method', 'endpoint', 'status'])
# user_id放日志中而非指标标签
```

### 陷阱3: 异步日志丢失上下文
```python
# 错误: 异步任务中丢失request_id
async def background_task(data):
    logger.info("Processing", data=data)  # 缺少request_id!

# 正确: 传递上下文
async def background_task(data, request_id: str):
    structlog.contextvars.bind_contextvars(request_id=request_id)
    logger.info("Processing", data=data)  # 包含request_id
```

### 陷阱4: 只在出错时打日志
```python
# 错误: 只有error日志,无法追踪正常流程
try:
    result = await process(data)
except Exception as e:
    logger.error("Failed", error=str(e))

# 正确: 关键步骤都有日志
logger.info("processing_started", data_id=data.id)
try:
    result = await process(data)
    logger.info("processing_completed", data_id=data.id, result_size=len(result))
except Exception as e:
    logger.error("processing_failed", data_id=data.id, error=str(e))
```

## Agent Checklist

### 日志规范
- [ ] 使用结构化日志(JSON格式)
- [ ] 每条日志包含request_id
- [ ] 敏感数据已脱敏
- [ ] 日志级别使用合理
- [ ] 日志输出到stdout(容器友好)

### 指标覆盖
- [ ] RED指标已覆盖(Rate/Errors/Duration)
- [ ] 业务关键指标已定义
- [ ] 标签基数可控
- [ ] /metrics端点已暴露

### 分布式追踪
- [ ] OpenTelemetry SDK已集成
- [ ] 自动埋点覆盖HTTP/DB/Redis
- [ ] 关键业务有手动Span
- [ ] Trace Context跨服务传播

### 告警和仪表盘
- [ ] 核心告警规则已配置
- [ ] 仪表盘覆盖关键指标
- [ ] 告警通知渠道已设置
- [ ] 数据保留策略已定义
