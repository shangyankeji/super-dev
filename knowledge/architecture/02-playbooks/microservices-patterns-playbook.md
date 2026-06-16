---
id: microservices-patterns-playbook
title: 微服务架构模式手册
domain: architecture
category: 02-playbooks
difficulty: advanced
tags: [microservices, architecture, saga, cqrs, api-gateway, service-mesh, circuit-breaker, event-driven, distributed-transactions, enterprise]
quality_score: 94
maintainer: architecture-team@superdev.com
last_updated: 2026-06-15
---

# 微服务架构模式手册

> 基于 [microservices.io (Chris Richardson)](https://microservices.io/patterns/microservices.html) + [Temporal Saga Guide](https://temporal.io/blog/mastering-saga-patterns-for-distributed-transactions-in-microservices) + [Octopus Top 10 Patterns](https://octopus.com/devops/microservices/microservice-design-patterns/)

## 核心模式速查

| 模式 | 解决什么 | 何时用 |
|------|---------|--------|
| API Gateway | 客户端不直接访问后端服务 | 多服务聚合 / 统一认证 |
| Saga | 跨服务分布式事务 | 需要跨服务原子操作 |
| CQRS | 读写模型分离 | 读远多于写（报表/搜索） |
| Circuit Breaker | 防止级联故障 | 调用不可靠的外部服务 |
| Service Mesh | 服务间通信管理 | 超过 5 个微服务 |
| Event Sourcing | 状态变更追溯 | 审计 / 时间旅行 / 重建 |

## Saga 模式（分布式事务）

### 编排式（推荐复杂场景）
```python
# 一个协调器编排整个事务流程
class OrderSaga:
    def execute(self, order):
        try:
            payment = payment_service.charge(order)        # 步骤 1
            inventory = inventory_service.reserve(order)   # 步骤 2
            shipping = shipping_service.schedule(order)    # 步骤 3
            order.status = 'confirmed'
        except Step1Failed:
            pass  # 没做什么，无补偿
        except Step2Failed:
            payment_service.refund(payment.id)             # 补偿步骤 1
        except Step3Failed:
            inventory_service.release(inventory.id)        # 补偿步骤 2
            payment_service.refund(payment.id)             # 补偿步骤 1
```

### 协同式（事件驱动，适合简单场景）
```
Order Service → publish OrderCreated → Payment Service listens → charges
Payment Service → publish PaymentSucceeded → Inventory Service listens → reserves
Payment Service → publish PaymentFailed → Order Service listens → cancels
```

## API Gateway 模式

```yaml
# 统一入口：认证、限流、路由、聚合
API Gateway:
  routes:
    /api/users/* → user-service
    /api/orders/* → order-service
    /api/dashboard →  # 聚合多个服务
      - user-service.getUser
      - order-service.getOrders
      - stats-service.getStats
  middleware:
    - rate_limit: 100/min
    - auth: JWT
    - cors: ["https://app.example.com"]
```

## Circuit Breaker（熔断器）

```python
# 防止对故障服务的调用堆积导致级联崩溃
@circuit_breaker(
    failure_threshold=5,      # 5 次失败后熔断
    recovery_timeout=30,      # 30s 后半开尝试
    fallback=lambda: cached_data,  # 降级返回缓存
)
def call_payment_service(order):
    return external_payment_api.charge(order)
```

三种状态：
```
CLOSED → 正常调用
  ↓ 5 次连续失败
OPEN → 直接返回 fallback，不调远端
  ↓ 30 秒后
HALF-OPEN → 放一个请求试探
  ↓ 成功 → CLOSED | 失败 → OPEN
```

## CQRS（命令查询职责分离）

```
写模型（Command）          读模型（Query）
    │                          │
  INSERT/UPDATE              SELECT
    │                          │
    ↓                          ↓
PostgreSQL (规范化)     Elasticsearch/Redis (反规范化)
    │                          ↑
    └── 事件同步 ──────────────┘
```

适用：读/写比 > 10:1（报表/搜索/推荐）

## Service Mesh（Istio/Linkerd）

```yaml
# 服务间通信由 Sidecar 代理处理（应用无感知）
apiVersion: networking.istio.io/v1
spec:
  http:
  - route:
    - destination:
        host: order-service
    retries:
      attempts: 3              # 自动重试
      perTryTimeout: 2s
    timeout: 10s
    faultInjection:            # 混沌测试
      delay:
        percentage: 10
        fixedDelay: 5s
```

## 何时不用微服务

- 团队 < 5 人 → 用模块化单体
- 域边界不清晰 → 先单体，后拆分
- 吞吐量 < 1000 QPS → 单体够用
- 没有运维团队 → 微服务运维成本 > 收益
