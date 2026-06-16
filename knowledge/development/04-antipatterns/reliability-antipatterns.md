---
id: reliability-antipatterns
title: 稳定性反模式指南
domain: development
category: 04-antipatterns
difficulty: intermediate
tags: [antipatterns, breaker, circuit, development, failure, fatigue, incident, point]
quality_score: 70
last_updated: 2026-06-15
---
# 稳定性反模式指南

> 适用范围：分布式系统 / 微服务 / 高可用架构
> 约束级别：SHALL（必须在架构评审和 SRE 审查阶段拦截）

---

## 1. 无超时与熔断（Missing Timeout and Circuit Breaker）

### 描述
调用外部依赖（HTTP API、数据库、Redis、消息队列、第三方服务）时不设超时，也没有熔断降级策略。一个下游服务的响应变慢或不可用时，上游服务的线程/连接被占满，导致级联故障（雪崩效应）。

### 错误示例
```python
# HTTP 无超时
def get_recommendations(user_id):
    # 推荐服务变慢（30 秒响应），调用方所有线程被阻塞
    response = requests.get(f"{RECOMMEND_SERVICE}/users/{user_id}/recs")
    return response.json()

# 数据库无超时
def complex_report():
    # 复杂查询执行 10 分钟，连接池耗尽
    return db.execute("SELECT ... FROM huge_table JOIN ... WHERE ...")

# 无熔断 -- 明知服务已宕机仍继续请求
def get_user_avatar(user_id):
    try:
        return requests.get(f"{AVATAR_SERVICE}/avatars/{user_id}").content
    except:
        return DEFAULT_AVATAR
    # 每个请求都尝试，即使服务已经连续超时 1000 次
```

### 正确示例
```python
import httpx
from circuitbreaker import circuit, CircuitBreakerError

# 1. HTTP 超时
client = httpx.AsyncClient(
    timeout=httpx.Timeout(connect=2.0, read=5.0, write=5.0, pool=10.0),
)

# 2. 熔断器
@circuit(failure_threshold=5, recovery_timeout=30)
async def get_recommendations(user_id: str) -> list[Product]:
    response = await client.get(f"{RECOMMEND_SERVICE}/users/{user_id}/recs")
    response.raise_for_status()
    return [Product(**p) for p in response.json()]

# 3. 降级策略
async def get_recommendations_with_fallback(user_id: str) -> list[Product]:
    try:
        return await get_recommendations(user_id)
    except CircuitBreakerError:
        logger.warning("Recommendation circuit open, returning cached/popular items")
        return await cache.get_popular_products()
    except httpx.TimeoutException:
        logger.warning("Recommendation service timeout")
        return await cache.get_popular_products()

# 4. 数据库超时
engine = create_engine(
    DATABASE_URL,
    pool_size=10,
    max_overflow=5,
    pool_timeout=10,
    connect_args={"connect_timeout": 5, "options": "-c statement_timeout=30000"},
)

# 5. 重试 + 退避
from tenacity import retry, stop_after_attempt, wait_exponential

@retry(
    stop=stop_after_attempt(3),
    wait=wait_exponential(multiplier=1, min=1, max=10),
    retry=retry_if_exception_type(httpx.TransportError),
)
async def idempotent_external_call(data: dict) -> dict:
    response = await client.post(f"{EXTERNAL_API}/process", json=data)
    response.raise_for_status()
    return response.json()
```

### 检测方法
- HTTP 调用无 `timeout` 参数。
- 数据库连接无 `connect_timeout` / `statement_timeout`。
- 无 `circuitbreaker` / `pybreaker` 等熔断库。
- 外部调用失败时直接报错，无降级策略。
- 重试逻辑无退避策略（立即重试导致放大效应）。

### 修复步骤
1. 所有 HTTP 调用设置 connect + read + write timeout。
2. 数据库设置 connection timeout + statement timeout。
3. 关键外部依赖添加熔断器（阈值 5 次失败，恢复周期 30 秒）。
4. 每个外部依赖定义降级策略（缓存 / 默认值 / 部分降级）。
5. 幂等操作添加重试 + 指数退避。
6. 建立超时和熔断的监控告警。

### Agent Checklist
- [ ] 所有 HTTP 调用有 timeout
- [ ] 数据库有 connection + statement timeout
- [ ] 关键外部依赖有熔断器
- [ ] 每个外部调用有降级方案
- [ ] 重试使用指数退避

---

## 2. 无统一事故指挥流程（Missing Incident Response）

### 描述
事故发生时没有标准化的响应流程，导致多人同时操作互相冲突、关键决策无人拍板、恢复时间不可预测、事故信息在不同群组分散传播。

### 错误示例
```
# 典型混乱场景
12:00 告警触发
12:05 小明在微信群说 "谁看一下"
12:10 小红 SSH 到生产机器查日志
12:15 小李也 SSH 到同一台机器，两人操作冲突
12:20 小明在另一个群问 "是不是发布导致的"
12:25 没人知道最近发布了什么
12:30 Leader 开始问 "什么情况"
12:35 三个人同时尝试不同的修复方案
12:45 修复方案 A 和方案 B 互相冲突，导致问题更严重
13:00 终于有人想起来可以回滚
13:15 回滚完成，但已经 75 分钟了
```

### 正确示例
```yaml
# 事故响应流程（Runbook）
incident_response:
  severity_levels:
    P0: "全站不可用 / 数据丢失 / 安全事件"
    P1: "核心功能不可用 / 部分用户受影响"
    P2: "非核心功能异常 / 性能退化"
    P3: "不影响用户的内部系统问题"

  roles:
    incident_commander: "统一决策，协调资源，对外沟通"
    operations_lead: "执行诊断和修复操作"
    communications_lead: "更新状态页，通知利益方"
    scribe: "记录时间线和操作日志"

  workflow:
    1_detect:
      - "告警触发或用户报告"
      - "值班人员 5 分钟内确认"
      - "判定严重性等级"
    2_triage:
      - "Incident Commander 开启事故频道"
      - "创建事故文档，记录时间线"
      - "召集相关人员"
    3_mitigate:
      - "优先恢复服务（回滚 / 降级 / 扩容）"
      - "修复可以在恢复之后"
      - "所有操作在事故频道中发出，禁止私聊操作"
    4_resolve:
      - "确认问题完全恢复"
      - "监控 30 分钟无复发"
      - "降级告警"
    5_postmortem:
      - "48 小时内完成复盘"
      - "产出 Action Items 并分配 Owner"
      - "纳入发布门禁或监控"

  escalation:
    response_time:
      P0: "5 min 确认, 15 min 召集"
      P1: "15 min 确认, 30 min 召集"
      P2: "1 hour 确认"
    auto_escalation:
      - "P0/P1 超过 30 分钟未缓解 -> 通知 VP Engineering"
      - "P0 超过 1 小时未恢复 -> 通知 CTO"
```

```python
# 自动化事故流程
class IncidentManager:
    async def create_incident(self, severity: str, title: str, reporter: str):
        incident = Incident(severity=severity, title=title, reporter=reporter)

        # 创建事故频道
        channel = await slack.create_channel(f"inc-{incident.id}-{slug(title)}")

        # 通知值班人员
        oncall = await pagerduty.get_oncall(service="platform")
        await slack.send(channel, f"@{oncall.name} Incident Commander assigned")

        # 创建事故文档
        doc = await google_docs.create(
            template="incident_template",
            title=f"[{severity}] {title}",
        )

        # 更新状态页
        if severity in ("P0", "P1"):
            await statuspage.create_incident(title=title, status="investigating")

        return incident
```

### 检测方法
- 无事故响应 Runbook 文档。
- 事故恢复时间 (MTTR) > 1 小时。
- 相同故障在 3 个月内重复发生。
- 事故后无复盘文档和 Action Items。
- 告警触发后超过 15 分钟无人响应。

### 修复步骤
1. 编写事故响应 Runbook，定义严重性等级和响应流程。
2. 明确事故角色（Incident Commander / Operations Lead / Scribe）。
3. 配置 PagerDuty / OpsGenie 值班轮换和自动升级。
4. 建立事故频道模板和事故文档模板。
5. 制度化事后复盘（48 小时内），Action Items 有 Owner 和 Deadline。

### Agent Checklist
- [ ] 有事故响应 Runbook
- [ ] 事故角色分工明确
- [ ] 有值班轮换和自动升级机制
- [ ] 事后复盘有 Action Items 跟踪
- [ ] MTTR 目标：P0 < 30 min, P1 < 1 hour

---

## 3. 告警疲劳（Alert Fatigue）

### 描述
告警策略设置不当，导致大量无效告警（误报、低优先级、重复告警），值班人员被噪声淹没后开始忽略所有告警，当真正的故障发生时未能及时响应。

### 错误示例
```yaml
# 告警太多、太敏感
alerts:
  - name: "CPU > 50%"
    condition: cpu_usage > 50   # 阈值太低，经常触发
    action: page_oncall          # 所有告警都寻呼值班
    severity: critical           # 所有告警都是 critical

  - name: "Any error in logs"
    condition: error_count > 0   # 1 条错误就告警
    action: page_oncall
    severity: critical

  - name: "Response time > 100ms"
    condition: p99_latency > 100  # 阈值太低
    action: page_oncall
    severity: critical

# 结果：每天 200+ 条告警，值班人员直接静音
```

### 正确示例
```yaml
# 分层告警策略
alerts:
  # P0 -- 立即寻呼（每月 < 5 次）
  critical:
    - name: "Service down"
      condition: health_check_failures >= 3 AND duration > 2m
      action: page_oncall
      runbook: "https://runbook.example.com/service-down"

    - name: "Error rate spike"
      condition: error_rate > 5% AND duration > 3m  # 持续 3 分钟才告警
      action: page_oncall
      runbook: "https://runbook.example.com/error-rate"

  # P1 -- Slack 通知（每天 < 10 次）
  warning:
    - name: "High latency"
      condition: p99_latency > 500ms AND duration > 5m
      action: slack_channel
      auto_resolve: true

    - name: "Disk space low"
      condition: disk_usage > 80%
      action: slack_channel

  # P2 -- 仪表板（随时查看）
  info:
    - name: "Elevated CPU"
      condition: cpu_usage > 70% AND duration > 10m
      action: dashboard_only
```

```python
# 告警去重和聚合
class AlertManager:
    def __init__(self, redis: Redis):
        self._redis = redis

    async def fire(self, alert: Alert):
        # 去重：相同告警 15 分钟内不重复发送
        dedup_key = f"alert:{alert.name}:{alert.source}"
        if await self._redis.get(dedup_key):
            return  # 已存在，跳过

        await self._redis.setex(dedup_key, 900, "1")  # 15 分钟去重窗口

        # 分级通知
        if alert.severity == "critical":
            await self._page_oncall(alert)
        elif alert.severity == "warning":
            await self._notify_slack(alert)
        else:
            await self._log_to_dashboard(alert)

    async def auto_resolve(self, alert_name: str, source: str):
        """条件恢复时自动关闭告警"""
        dedup_key = f"alert:{alert_name}:{source}"
        await self._redis.delete(dedup_key)
        await self._notify_slack(Alert(
            name=alert_name, severity="resolved", message="Auto-resolved"
        ))
```

### 检测方法
- 每日告警数量 > 50。
- 值班人员确认告警的平均时间 > 15 分钟。
- 告警中误报率 > 30%。
- 所有告警使用相同的严重性级别。
- 告警无对应的 Runbook 链接。

### 修复步骤
1. 审计过去 30 天的告警数据，统计各告警的触发次数和有效处理率。
2. 删除误报率 > 50% 的告警或调整阈值。
3. 将告警分为 3 级：Critical（寻呼）/ Warning（Slack）/ Info（仪表板）。
4. 为每条告警添加持续时间条件（避免瞬时抖动触发）。
5. 实现告警去重（相同告警 15 分钟内不重复发送）。
6. 每条 Critical/Warning 告警关联 Runbook。

### Agent Checklist
- [ ] Critical 告警每月 < 5 次
- [ ] 告警分为 3 个严重性级别
- [ ] 所有告警有持续时间条件（不瞬时触发）
- [ ] 告警有 15 分钟去重窗口
- [ ] 每条告警关联 Runbook

---

## 4. 单点故障（Single Point of Failure）

### 描述
系统中存在单一的不可替代组件，该组件的故障导致整个系统不可用。常见单点：单实例数据库、单机部署、单一网络路径、单一外部依赖无降级。

### 错误示例
```yaml
# 单实例部署 -- 机器挂了全完了
services:
  app:
    image: myapp:latest
    deploy:
      replicas: 1  # 只有 1 个实例

  postgres:
    image: postgres:15
    volumes:
      - /data/postgres:/var/lib/postgresql/data  # 单机本地存储，磁盘坏了数据全没
    # 无从库、无备份
```

```python
# 唯一的外部依赖无降级
def process_payment(order):
    # 只对接了一个支付渠道，它挂了就完全无法收款
    result = stripe_client.charge(order.total)
    return result
```

### 正确示例
```yaml
# 多实例 + 多可用区
services:
  app:
    image: myapp:latest
    deploy:
      replicas: 3
      placement:
        constraints:
          - node.labels.zone != same  # 分布到不同可用区

  postgres:
    # 使用托管数据库服务（自带多可用区和自动故障转移）
    # 或者自建：主 + 同步从 + 异步从
    image: postgres:15
    environment:
      POSTGRES_REPLICATION_MODE: master
    volumes:
      - type: volume
        source: pg-data
        target: /var/lib/postgresql/data
        volume:
          driver: rbd  # 分布式存储

  postgres-replica:
    image: postgres:15
    environment:
      POSTGRES_REPLICATION_MODE: replica
      POSTGRES_MASTER_HOST: postgres
```

```python
# 多渠道降级
class PaymentService:
    def __init__(self, primary: StripeClient, fallback: AlipayClient):
        self._primary = primary
        self._fallback = fallback

    async def charge(self, order: Order) -> PaymentResult:
        try:
            return await self._primary.charge(order.total)
        except PaymentGatewayError:
            logger.warning("Primary payment failed, falling back to Alipay")
            return await self._fallback.charge(order.total)

# 缓存降级
class ProductService:
    async def get_product(self, product_id: str) -> Product:
        # L1: 本地缓存
        cached = self._local_cache.get(product_id)
        if cached:
            return cached

        # L2: Redis 缓存
        try:
            cached = await self._redis.get(f"product:{product_id}")
            if cached:
                product = Product.model_validate_json(cached)
                self._local_cache.set(product_id, product)
                return product
        except RedisError:
            logger.warning("Redis unavailable, falling through to DB")

        # L3: 数据库
        product = await self._db.get_product(product_id)
        return product
```

### 检测方法
- 部署 replicas = 1。
- 数据库无从库或备份。
- 应用部署在单一可用区。
- 关键外部依赖只有一个供应商且无降级。
- 负载均衡器后只有一台服务器。

### 修复步骤
1. 应用至少部署 2 个实例，分布在不同可用区。
2. 数据库配置主从复制 + 自动故障转移。
3. 关键外部依赖配置备用渠道。
4. 实现多级缓存降级策略（本地 -> Redis -> DB）。
5. 定期进行故障注入演练（Chaos Engineering）。

### Agent Checklist
- [ ] 应用实例 >= 2，分布在不同可用区
- [ ] 数据库有主从复制和自动故障转移
- [ ] 关键外部依赖有降级方案
- [ ] 有定期故障演练
- [ ] 无单机本地存储（使用分布式存储）

---

## 5. 复盘无闭环（Postmortem Without Follow-through）

### 描述
事故后虽然做了复盘，但 Action Items 无人跟进、无截止日期、无验收机制，导致同类故障反复发生。复盘变成了走形式。

### 错误示例
```markdown
# 事故复盘文档
## 2024-01-15 数据库宕机事故

### 根因
数据库磁盘满导致写入失败。

### 改进措施
- 增加磁盘监控告警
- 优化数据归档策略
- 增加磁盘容量

### 状态：已复盘 ✅

# 三个月后...同样的问题再次发生
# "上次复盘的改进措施做了吗？" "呃..."
```

### 正确示例
```markdown
# 事故复盘文档
## INC-2024-001: 数据库磁盘满导致订单服务不可用

### 时间线
- 14:00 告警触发：PostgreSQL 磁盘使用率 > 95%
- 14:05 值班工程师确认
- 14:10 开始紧急清理临时表
- 14:25 磁盘使用率降至 70%，服务恢复
- 14:30 确认服务完全恢复

### 影响范围
- 持续时间：25 分钟
- 影响用户：约 5,000 用户无法下单
- 数据丢失：无

### 根因分析
1. 审计日志表 `audit_log` 无数据归档策略，累计 500GB。
2. 磁盘告警阈值设为 90%，触发时已无足够缓冲时间。
3. 无定期数据清理计划任务。

### Action Items

| # | 措施 | Owner | Deadline | 验收标准 | 状态 |
|---|------|-------|----------|----------|------|
| 1 | 磁盘告警阈值调整为 80% | SRE-张三 | 2024-01-17 | 告警规则已更新且触发测试通过 | ✅ Done |
| 2 | audit_log 表添加 30 天自动归档 | DBA-李四 | 2024-01-22 | Cron Job 运行正常，历史数据已归档 | ✅ Done |
| 3 | 所有数据库表制定保留策略 | DBA-李四 | 2024-02-01 | 保留策略文档已发布 | 🔄 In Progress |
| 4 | CI 流水线添加磁盘预算检查 | DevOps-王五 | 2024-02-15 | PR 中大表变更触发 DBA 审批 | ⬜ TODO |
```

```python
# 自动化跟踪 Action Items
class PostmortemTracker:
    async def check_overdue_items(self):
        """每日检查超期未完成的 Action Items"""
        overdue = await self._repo.get_overdue_actions()
        for item in overdue:
            days_overdue = (datetime.now() - item.deadline).days
            if days_overdue > 7:
                # 超期 7 天以上，升级通知
                await self._notify_manager(item)
            else:
                await self._notify_owner(item)

    async def block_similar_changes(self, incident_id: str):
        """将复盘改进措施纳入 CI 门禁"""
        actions = await self._repo.get_actions(incident_id)
        for action in actions:
            if action.ci_rule and action.status != "done":
                # 相关代码变更被阻断，直到改进措施完成
                await self._ci.add_blocking_rule(action.ci_rule)
```

### 检测方法
- 复盘文档中 Action Items 无 Owner 或无 Deadline。
- 超过 Deadline 的 Action Items 占比 > 30%。
- 相同根因的事故在 6 个月内重复发生。
- 复盘会议后无跟踪机制。

### 修复步骤
1. 复盘文档模板强制包含：Owner + Deadline + 验收标准。
2. Action Items 录入项目管理工具（Jira / Linear），设置到期提醒。
3. 每周站会检查复盘 Action Items 进度。
4. 将改进措施纳入 CI 门禁或告警规则。
5. 季度审计：统计 Action Items 完成率和重复故障率。

### Agent Checklist
- [ ] 复盘文档包含 Owner + Deadline + 验收标准
- [ ] Action Items 在项目管理工具中跟踪
- [ ] 有每周跟踪机制
- [ ] 关键改进纳入 CI 门禁
- [ ] 相同根因故障不重复发生

---

## 6. 缺乏可观测性（Poor Observability）

### 描述
系统缺少日志、指标、链路追踪三大支柱中的一个或多个，导致出问题时无法快速定位根因。只有日志没有指标（无法看全局趋势），只有指标没有链路追踪（无法定位单个请求的问题）。

### 错误示例
```python
# 无结构化日志 + 无指标 + 无追踪
@app.post("/orders")
def create_order(data):
    print(f"Creating order: {data}")  # 无结构化日志
    order = order_service.create(data)
    print(f"Order created: {order.id}")
    return order
    # 问题来了：
    # - 无法知道 P99 延迟是多少
    # - 无法知道某个慢请求经过了哪些服务
    # - 无法知道错误率的趋势
```

### 正确示例
```python
import structlog
from opentelemetry import trace
from prometheus_client import Counter, Histogram

# 1. 结构化日志
logger = structlog.get_logger()

# 2. 指标
order_created_total = Counter(
    "order_created_total", "Total orders created", ["status"]
)
order_creation_duration = Histogram(
    "order_creation_duration_seconds", "Order creation latency",
    buckets=[0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
)

# 3. 链路追踪
tracer = trace.get_tracer(__name__)

@app.post("/orders")
async def create_order(data: CreateOrderRequest, request: Request):
    with tracer.start_as_current_span("create_order") as span:
        span.set_attribute("user_id", data.user_id)
        span.set_attribute("item_count", len(data.items))

        log = logger.bind(
            request_id=request.state.request_id,
            user_id=data.user_id,
            trace_id=span.get_span_context().trace_id,
        )

        log.info("order_creation_started", item_count=len(data.items))

        with order_creation_duration.time():
            try:
                order = await order_service.create(data)
                order_created_total.labels(status="success").inc()
                log.info("order_created", order_id=order.id, total=str(order.total))
                return order
            except InsufficientStockError as e:
                order_created_total.labels(status="insufficient_stock").inc()
                log.warning("order_creation_failed", reason="insufficient_stock")
                raise
            except Exception as e:
                order_created_total.labels(status="error").inc()
                log.error("order_creation_error", error=str(e))
                raise

# 4. 中间件自动注入 request_id 和追踪上下文
@app.middleware("http")
async def observability_middleware(request: Request, call_next):
    request_id = request.headers.get("X-Request-ID", str(uuid4()))
    request.state.request_id = request_id

    with tracer.start_as_current_span(
        f"{request.method} {request.url.path}",
        attributes={
            "http.method": request.method,
            "http.url": str(request.url),
            "http.request_id": request_id,
        },
    ):
        response = await call_next(request)
        response.headers["X-Request-ID"] = request_id
        return response
```

### 检测方法
- 无 Prometheus / Datadog / New Relic 等指标采集。
- 无 Jaeger / Zipkin / OpenTelemetry 链路追踪。
- 日志使用 `print()` 且无结构化。
- 出问题时需要 SSH 到服务器 `grep` 日志定位原因。
- 无法回答 "当前系统的 P99 延迟是多少" 这种基础问题。

### 修复步骤
1. 引入 OpenTelemetry SDK，统一日志 + 指标 + 追踪。
2. 使用结构化日志（`structlog` / `python-json-logger`），输出 JSON 格式。
3. 定义核心业务指标（RED: Rate / Error / Duration）。
4. 为关键链路添加 Span 追踪。
5. 搭建可观测性平台（Grafana + Prometheus + Jaeger 或 Datadog）。
6. 创建标准仪表板：系统总览 / 服务详情 / 错误率趋势 / 延迟分布。

### Agent Checklist
- [ ] 有结构化日志（JSON 格式）
- [ ] 有指标采集（Prometheus / Datadog）
- [ ] 有链路追踪（OpenTelemetry / Jaeger）
- [ ] 核心 API 有 RED 指标仪表板
- [ ] 日志包含 request_id 和 trace_id

---

## 全局 Agent Checklist

| 检查项 | 阈值 | 工具 |
|--------|------|------|
| HTTP 调用无 timeout | 0 处 | Code Review |
| 熔断器覆盖 | 100% 外部依赖 | 架构审查 |
| 事故 MTTR | P0 < 30min, P1 < 1h | 事故跟踪 |
| 每日告警数 | < 20 条 | 告警平台统计 |
| Critical 告警/月 | < 5 条 | 告警平台统计 |
| 服务实例数 | >= 2 | 部署配置 |
| 复盘 Action 完成率 | > 90% | 项目管理工具 |
| 可观测性三支柱 | 日志 + 指标 + 追踪 | 平台审查 |
