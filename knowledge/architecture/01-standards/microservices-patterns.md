---
id: microservices-patterns
title: 微服务架构模式完整指南
domain: architecture
category: 01-standards
difficulty: intermediate
tags: [agent, architecture, checklist, microservices, patterns, 参考资料, 可观测性, 常见反模式]
quality_score: 70
last_updated: 2026-06-15
---
# 微服务架构模式完整指南

## 概述

微服务架构是一种将应用拆分为一组小型、独立部署的服务的架构风格。每个服务围绕特定业务能力构建，拥有自己的数据库，通过 API 通信。本指南覆盖微服务架构的核心模式、通信策略、数据管理、部署和运维。

### 何时使用微服务

✅ **适合微服务**:
- 团队规模 > 20 人，需要独立部署
- 业务域复杂，需要不同技术栈
- 需要独立扩展不同组件
- 发布频率高，需要持续交付

❌ **不适合微服务**:
- 团队 < 10 人（增加运维复杂度）
- 业务域简单（过度设计）
- 无 DevOps 基础设施（无法运维）
- 项目早期（领域边界未明确）

---

## 核心设计模式

### 1. 服务拆分模式

#### 按业务能力拆分 (Decompose by Business Capability)

```
电商系统拆分:
├── 用户服务 (User Service)        → 注册/登录/Profile
├── 商品服务 (Product Service)     → 商品CRUD/搜索/分类
├── 订单服务 (Order Service)       → 下单/订单管理/状态机
├── 支付服务 (Payment Service)     → 支付/退款/对账
├── 库存服务 (Inventory Service)   → 库存管理/预占/释放
├── 通知服务 (Notification Service)→ 邮件/短信/推送
└── 推荐服务 (Recommendation)      → 推荐算法/用户画像
```

#### 按子域拆分 (Decompose by Subdomain - DDD)

```python
# 领域驱动设计的限界上下文
class BoundedContext:
    """
    核心域 (Core Domain):        订单、支付 — 核心竞争力
    支撑域 (Supporting Domain):  库存、物流 — 支撑业务运转
    通用域 (Generic Domain):     用户、通知 — 通用能力
    """
    pass
```

### 2. API Gateway 模式

```yaml
# Kong / Nginx API Gateway 配置示例
services:
  - name: user-service
    url: http://user-service:8080
    routes:
      - paths: ["/api/v1/users"]
        methods: ["GET", "POST", "PUT"]
    plugins:
      - name: rate-limiting
        config:
          minute: 100
      - name: jwt
      - name: cors

  - name: order-service
    url: http://order-service:8080
    routes:
      - paths: ["/api/v1/orders"]
    plugins:
      - name: rate-limiting
        config:
          minute: 50
      - name: jwt
```

#### BFF (Backend for Frontend)

```
Mobile App  ──→ Mobile BFF  ──→ User Service
                              ──→ Order Service (精简字段)

Web App     ──→ Web BFF     ──→ User Service
                              ──→ Order Service (完整字段)
                              ──→ Analytics Service

Admin Panel ──→ Admin BFF   ──→ All Services (管理权限)
```

### 3. 服务通信模式

#### 同步通信: REST / gRPC

```protobuf
// gRPC 服务定义 (推荐服务间通信)
syntax = "proto3";

service OrderService {
  rpc CreateOrder (CreateOrderRequest) returns (OrderResponse);
  rpc GetOrder (GetOrderRequest) returns (OrderResponse);
  rpc ListOrders (ListOrdersRequest) returns (stream OrderResponse);
}

message CreateOrderRequest {
  string user_id = 1;
  repeated OrderItem items = 2;
  string payment_method = 3;
}

message OrderResponse {
  string order_id = 1;
  string status = 2;
  double total_amount = 3;
  google.protobuf.Timestamp created_at = 4;
}
```

#### 异步通信: 事件驱动

```python
# Kafka 事件发布 (Python)
from confluent_kafka import Producer

producer = Producer({'bootstrap.servers': 'kafka:9092'})

def publish_order_created(order):
    event = {
        "event_type": "OrderCreated",
        "timestamp": datetime.utcnow().isoformat(),
        "data": {
            "order_id": order.id,
            "user_id": order.user_id,
            "total": order.total,
            "items": [{"sku": i.sku, "qty": i.qty} for i in order.items]
        }
    }
    producer.produce(
        topic="orders",
        key=order.id,
        value=json.dumps(event).encode("utf-8")
    )
    producer.flush()
```

```python
# Kafka 事件消费
from confluent_kafka import Consumer

consumer = Consumer({
    'bootstrap.servers': 'kafka:9092',
    'group.id': 'inventory-service',
    'auto.offset.reset': 'earliest'
})
consumer.subscribe(['orders'])

while True:
    msg = consumer.poll(1.0)
    if msg is None:
        continue
    event = json.loads(msg.value().decode('utf-8'))
    if event['event_type'] == 'OrderCreated':
        # 库存服务: 扣减库存
        reserve_inventory(event['data']['items'])
```

### 4. Saga 模式 (分布式事务)

#### 编排式 Saga (Choreography)

```
OrderCreated ──→ InventoryService (预占库存)
                      │
              InventoryReserved ──→ PaymentService (扣款)
                                        │
                                PaymentCompleted ──→ OrderService (确认订单)

# 补偿流程 (任一步骤失败):
PaymentFailed ──→ InventoryService (释放库存)
                       │
               InventoryReleased ──→ OrderService (取消订单)
```

#### 协调式 Saga (Orchestration)

```python
class OrderSagaOrchestrator:
    def __init__(self):
        self.steps = [
            SagaStep("reserve_inventory", self.reserve, self.release_inventory),
            SagaStep("process_payment", self.charge, self.refund),
            SagaStep("confirm_order", self.confirm, self.cancel_order),
        ]

    async def execute(self, order):
        completed = []
        try:
            for step in self.steps:
                await step.execute(order)
                completed.append(step)
        except Exception as e:
            # 逆序执行补偿
            for step in reversed(completed):
                await step.compensate(order)
            raise SagaFailure(f"Saga failed at {step.name}: {e}")

    async def reserve(self, order):
        return await inventory_client.reserve(order.items)

    async def release_inventory(self, order):
        return await inventory_client.release(order.items)

    async def charge(self, order):
        return await payment_client.charge(order.user_id, order.total)

    async def refund(self, order):
        return await payment_client.refund(order.payment_id)
```

### 5. CQRS (命令查询职责分离)

```python
# 命令端 (Write Model)
class OrderCommandService:
    def create_order(self, cmd: CreateOrderCommand) -> str:
        order = Order.create(cmd.user_id, cmd.items)
        self.event_store.append(OrderCreatedEvent(order))
        return order.id

    def cancel_order(self, cmd: CancelOrderCommand):
        order = self.event_store.load(cmd.order_id)
        order.cancel(cmd.reason)
        self.event_store.append(OrderCancelledEvent(order))

# 查询端 (Read Model) - 独立数据库，针对查询优化
class OrderQueryService:
    def get_order(self, order_id: str) -> OrderView:
        return self.read_db.find_one({"order_id": order_id})

    def search_orders(self, filters: dict) -> List[OrderView]:
        return self.elasticsearch.search(filters)
```

### 6. 服务发现模式

```yaml
# Kubernetes 原生服务发现
apiVersion: v1
kind: Service
metadata:
  name: user-service
spec:
  selector:
    app: user-service
  ports:
    - port: 80
      targetPort: 8080
  type: ClusterIP

# 其他服务通过 DNS 访问:
# http://user-service.default.svc.cluster.local/api/users
```

```python
# Consul 服务发现 (非K8s环境)
import consul

c = consul.Consul()

# 注册服务
c.agent.service.register(
    name="order-service",
    service_id="order-service-1",
    address="10.0.0.5",
    port=8080,
    check=consul.Check.http("http://10.0.0.5:8080/health", interval="10s")
)

# 发现服务
_, services = c.health.service("user-service", passing=True)
for svc in services:
    host = svc['Service']['Address']
    port = svc['Service']['Port']
```

### 7. 断路器模式 (Circuit Breaker)

```python
import circuitbreaker
from circuitbreaker import circuit

@circuit(failure_threshold=5, recovery_timeout=30, expected_exception=Exception)
def call_payment_service(order_id, amount):
    response = requests.post(
        "http://payment-service/api/charge",
        json={"order_id": order_id, "amount": amount},
        timeout=5
    )
    response.raise_for_status()
    return response.json()

# 使用
try:
    result = call_payment_service("ORD-123", 99.99)
except circuitbreaker.CircuitBreakerError:
    # 断路器打开: 降级处理
    return {"status": "pending", "message": "支付服务暂时不可用，订单已记录"}
```

### 8. Strangler Fig 模式 (渐进式迁移)

```
Phase 1: 单体 + API Gateway
┌─────────────────────────────────┐
│  API Gateway                     │
│  /api/users  → Monolith          │
│  /api/orders → Monolith          │
│  /api/products → Monolith        │
└─────────────────────────────────┘

Phase 2: 逐步迁移
┌─────────────────────────────────┐
│  API Gateway                     │
│  /api/users  → User Service ✨   │
│  /api/orders → Monolith          │
│  /api/products → Monolith        │
└─────────────────────────────────┘

Phase 3: 完全微服务
┌─────────────────────────────────┐
│  API Gateway                     │
│  /api/users    → User Service    │
│  /api/orders   → Order Service   │
│  /api/products → Product Service │
└─────────────────────────────────┘
```

---

## 数据管理模式

### Database per Service

```
User Service     → PostgreSQL (用户数据)
Product Service  → MongoDB (商品目录, 灵活Schema)
Order Service    → PostgreSQL (订单事务)
Search Service   → Elasticsearch (全文搜索)
Cache Service    → Redis (热点数据缓存)
Analytics        → ClickHouse (分析查询)
```

### Event Sourcing

```python
class EventStore:
    def append(self, aggregate_id: str, event: DomainEvent):
        self.db.insert({
            "aggregate_id": aggregate_id,
            "event_type": type(event).__name__,
            "data": event.to_dict(),
            "timestamp": datetime.utcnow(),
            "version": self._next_version(aggregate_id)
        })

    def load(self, aggregate_id: str) -> List[DomainEvent]:
        rows = self.db.find({"aggregate_id": aggregate_id}).sort("version")
        return [self._deserialize(row) for row in rows]

    def replay(self, aggregate_id: str) -> Aggregate:
        """重放事件重建聚合状态"""
        events = self.load(aggregate_id)
        aggregate = Aggregate()
        for event in events:
            aggregate.apply(event)
        return aggregate
```

---

## 可观测性

### 分布式追踪 (OpenTelemetry)

```python
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.exporter.jaeger.thrift import JaegerExporter

# 配置追踪
trace.set_tracer_provider(TracerProvider())
jaeger_exporter = JaegerExporter(agent_host_name="jaeger", agent_port=6831)
trace.get_tracer_provider().add_span_processor(
    BatchSpanProcessor(jaeger_exporter)
)

tracer = trace.get_tracer(__name__)

@app.post("/api/orders")
async def create_order(request):
    with tracer.start_as_current_span("create_order") as span:
        span.set_attribute("user_id", request.user_id)

        # 调用库存服务 (自动传播 trace context)
        with tracer.start_as_current_span("reserve_inventory"):
            await inventory_client.reserve(request.items)

        # 调用支付服务
        with tracer.start_as_current_span("process_payment"):
            await payment_client.charge(request.amount)

        return {"order_id": order.id}
```

### 健康检查

```python
@app.get("/health")
async def health_check():
    checks = {
        "database": await check_db(),
        "redis": await check_redis(),
        "kafka": await check_kafka(),
    }
    healthy = all(v["status"] == "up" for v in checks.values())
    return {
        "status": "up" if healthy else "degraded",
        "checks": checks,
        "timestamp": datetime.utcnow().isoformat()
    }
```

---

## 常见反模式

### 1. 分布式单体 (Distributed Monolith)
**症状**: 服务之间强耦合，必须同时部署
**解决**: 确保服务有独立数据库，使用异步事件通信

### 2. 过度拆分 (Nano-services)
**症状**: 每个函数一个服务，运维成本爆炸
**解决**: 按限界上下文拆分，一个服务包含完整的业务能力

### 3. 同步调用链过长
**症状**: A → B → C → D → E，延迟累加，可用性下降
**解决**: 使用异步事件驱动，或合并强耦合服务

### 4. 共享数据库
**症状**: 多个服务访问同一个数据库
**解决**: Database per Service + 事件同步

---

## Agent Checklist

Agent 在设计微服务架构时必须检查:

- [ ] 每个服务是否有独立数据库？
- [ ] 服务间通信是否使用异步事件驱动（优先）或 gRPC？
- [ ] 是否有 API Gateway 统一入口？
- [ ] 分布式事务是否使用 Saga 模式？
- [ ] 是否配置断路器和超时？
- [ ] 是否有分布式追踪（OpenTelemetry/Jaeger）？
- [ ] 是否有健康检查端点？
- [ ] 是否有服务发现机制？
- [ ] 数据一致性是否接受最终一致性？
- [ ] 服务拆分粒度是否合理（不过大不过小）？

---

## 参考资料

- [Microservices Patterns - Chris Richardson](https://microservices.io/patterns/)
- [Building Microservices - Sam Newman](https://samnewman.io/books/building_microservices_2nd_edition/)
- [Domain-Driven Design - Eric Evans](https://www.domainlanguage.com/)

---

**文档版本**: v1.0
**最后更新**: 2026-03-28
**质量评分**: 91/100
