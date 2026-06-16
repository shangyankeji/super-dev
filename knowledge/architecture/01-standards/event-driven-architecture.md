---
id: event-driven-architecture
title: 事件驱动架构完整指南
domain: architecture
category: 01-standards
difficulty: intermediate
tags: [agent, architecture, checklist, driven, event, 实战代码示例, 常见陷阱, 最佳实践]
quality_score: 70
last_updated: 2026-06-15
---
# 事件驱动架构完整指南

## 概述
事件驱动架构(EDA)是一种以事件的产生、检测、消费为核心的软件架构模式。本指南覆盖事件溯源(Event Sourcing)、CQRS、Saga模式、消息代理选型和幂等性设计,帮助团队构建松耦合、高可扩展的分布式系统。

## 核心概念

### 1. 事件驱动模式分类
- **事件通知(Event Notification)**: 通知某事发生,接收方自行获取详情
- **事件携带状态转移(Event-Carried State Transfer)**: 事件中包含完整状态变更
- **事件溯源(Event Sourcing)**: 将状态变更存储为事件序列,而非当前状态
- **CQRS(Command Query Responsibility Segregation)**: 读写模型分离

### 2. 消息代理对比

| 特性 | Kafka | RabbitMQ | NATS | Pulsar | Redis Streams |
|------|-------|----------|------|--------|--------------|
| 吞吐量 | 极高(百万/秒) | 中(万级/秒) | 高(十万/秒) | 极高 | 中高 |
| 延迟 | 低(ms级) | 极低(亚ms) | 极低 | 低 | 极低 |
| 持久化 | 磁盘(日志) | 内存+磁盘 | 可选 | 分层存储 | AOF/RDB |
| 消息重放 | 支持 | 不支持 | JetStream支持 | 支持 | 有限支持 |
| 消费模式 | Pull | Push/Pull | Push/Pull | Push/Pull | Pull |
| 顺序保证 | 分区内有序 | 队列内有序 | 主题内有序 | 分区内有序 | 流内有序 |
| 适用场景 | 大数据流/日志 | 任务队列/RPC | 微服务/IoT | 多租户/云原生 | 简单事件流 |

### 3. 事件设计原则
- **不可变(Immutable)**: 事件一旦发布不可修改
- **自描述(Self-describing)**: 包含足够上下文理解事件含义
- **有版本(Versioned)**: Schema可演进,向后兼容
- **有因果(Causal)**: 记录因果关系(correlation_id/causation_id)

## 实战代码示例

### 事件溯源(Event Sourcing)

```python
# 事件基类与聚合根
from dataclasses import dataclass, field
from datetime import datetime
from typing import Any
import uuid
import json

@dataclass
class DomainEvent:
    """领域事件基类"""
    event_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    event_type: str = ""
    aggregate_id: str = ""
    aggregate_type: str = ""
    version: int = 0
    timestamp: datetime = field(default_factory=datetime.utcnow)
    data: dict = field(default_factory=dict)
    metadata: dict = field(default_factory=dict)

    def to_dict(self) -> dict:
        return {
            "event_id": self.event_id,
            "event_type": self.event_type,
            "aggregate_id": self.aggregate_id,
            "aggregate_type": self.aggregate_type,
            "version": self.version,
            "timestamp": self.timestamp.isoformat(),
            "data": self.data,
            "metadata": self.metadata,
        }

# 具体事件
@dataclass
class OrderCreated(DomainEvent):
    event_type: str = "OrderCreated"
    aggregate_type: str = "Order"

@dataclass
class OrderItemAdded(DomainEvent):
    event_type: str = "OrderItemAdded"
    aggregate_type: str = "Order"

@dataclass
class OrderConfirmed(DomainEvent):
    event_type: str = "OrderConfirmed"
    aggregate_type: str = "Order"

@dataclass
class OrderCancelled(DomainEvent):
    event_type: str = "OrderCancelled"
    aggregate_type: str = "Order"

# 聚合根
class Order:
    """订单聚合根 — 事件溯源"""

    def __init__(self):
        self.id: str = ""
        self.user_id: str = ""
        self.items: list = []
        self.status: str = ""
        self.total: float = 0.0
        self.version: int = 0
        self._pending_events: list[DomainEvent] = []

    # === 命令处理 ===
    @classmethod
    def create(cls, order_id: str, user_id: str) -> "Order":
        order = cls()
        order._apply(OrderCreated(
            aggregate_id=order_id,
            data={"user_id": user_id},
        ))
        return order

    def add_item(self, product_id: str, quantity: int, price: float):
        if self.status != "created":
            raise ValueError("Cannot add items to a non-created order")
        self._apply(OrderItemAdded(
            aggregate_id=self.id,
            data={"product_id": product_id, "quantity": quantity, "price": price},
        ))

    def confirm(self):
        if self.status != "created":
            raise ValueError("Cannot confirm order in current state")
        if not self.items:
            raise ValueError("Cannot confirm empty order")
        self._apply(OrderConfirmed(
            aggregate_id=self.id,
            data={"total": self.total},
        ))

    def cancel(self, reason: str):
        if self.status in ("cancelled", "shipped"):
            raise ValueError(f"Cannot cancel order in {self.status} state")
        self._apply(OrderCancelled(
            aggregate_id=self.id,
            data={"reason": reason},
        ))

    # === 事件应用 ===
    def _apply(self, event: DomainEvent):
        self.version += 1
        event.version = self.version
        self._on(event)
        self._pending_events.append(event)

    def _on(self, event: DomainEvent):
        handler = getattr(self, f"_on_{event.event_type}", None)
        if handler:
            handler(event)

    def _on_OrderCreated(self, event: DomainEvent):
        self.id = event.aggregate_id
        self.user_id = event.data["user_id"]
        self.status = "created"

    def _on_OrderItemAdded(self, event: DomainEvent):
        self.items.append(event.data)
        self.total += event.data["price"] * event.data["quantity"]

    def _on_OrderConfirmed(self, event: DomainEvent):
        self.status = "confirmed"

    def _on_OrderCancelled(self, event: DomainEvent):
        self.status = "cancelled"

    # === 从事件流重建 ===
    @classmethod
    def from_events(cls, events: list[DomainEvent]) -> "Order":
        order = cls()
        for event in events:
            order.version = event.version
            order._on(event)
        return order

    def get_pending_events(self) -> list[DomainEvent]:
        events = self._pending_events.copy()
        self._pending_events.clear()
        return events
```

### 事件存储

```python
# 事件存储(PostgreSQL)
from sqlalchemy import text

class EventStore:
    """事件存储"""

    def __init__(self, db):
        self.db = db

    async def append(self, aggregate_id: str, events: list[DomainEvent], expected_version: int):
        """追加事件(乐观并发控制)"""
        async with self.db.begin() as conn:
            # 检查版本冲突
            result = await conn.execute(text("""
                SELECT MAX(version) FROM events
                WHERE aggregate_id = :aggregate_id
            """), {"aggregate_id": aggregate_id})
            current_version = result.scalar() or 0

            if current_version != expected_version:
                raise ConcurrencyError(
                    f"Expected version {expected_version}, but found {current_version}"
                )

            # 追加事件
            for event in events:
                await conn.execute(text("""
                    INSERT INTO events (
                        event_id, event_type, aggregate_id, aggregate_type,
                        version, timestamp, data, metadata
                    ) VALUES (
                        :event_id, :event_type, :aggregate_id, :aggregate_type,
                        :version, :timestamp, :data, :metadata
                    )
                """), {
                    "event_id": event.event_id,
                    "event_type": event.event_type,
                    "aggregate_id": event.aggregate_id,
                    "aggregate_type": event.aggregate_type,
                    "version": event.version,
                    "timestamp": event.timestamp,
                    "data": json.dumps(event.data),
                    "metadata": json.dumps(event.metadata),
                })

    async def load(self, aggregate_id: str) -> list[DomainEvent]:
        """加载聚合的事件流"""
        async with self.db.connect() as conn:
            result = await conn.execute(text("""
                SELECT * FROM events
                WHERE aggregate_id = :aggregate_id
                ORDER BY version
            """), {"aggregate_id": aggregate_id})

            events = []
            for row in result:
                events.append(DomainEvent(
                    event_id=row.event_id,
                    event_type=row.event_type,
                    aggregate_id=row.aggregate_id,
                    aggregate_type=row.aggregate_type,
                    version=row.version,
                    timestamp=row.timestamp,
                    data=json.loads(row.data),
                    metadata=json.loads(row.metadata),
                ))
            return events
```

### CQRS实现

```python
# CQRS — 命令端
class OrderCommandHandler:
    """订单命令处理器(写端)"""

    def __init__(self, event_store: EventStore, event_bus: EventBus):
        self.event_store = event_store
        self.event_bus = event_bus

    async def handle_create_order(self, cmd: CreateOrderCommand):
        order = Order.create(cmd.order_id, cmd.user_id)
        for item in cmd.items:
            order.add_item(item.product_id, item.quantity, item.price)

        events = order.get_pending_events()
        await self.event_store.append(order.id, events, expected_version=0)
        await self.event_bus.publish(events)
        return order.id

    async def handle_confirm_order(self, cmd: ConfirmOrderCommand):
        events = await self.event_store.load(cmd.order_id)
        order = Order.from_events(events)
        order.confirm()

        new_events = order.get_pending_events()
        await self.event_store.append(order.id, new_events, expected_version=order.version - len(new_events))
        await self.event_bus.publish(new_events)

# CQRS — 查询端(投影/读模型)
class OrderProjection:
    """订单查询投影(读端)"""

    def __init__(self, read_db):
        self.read_db = read_db

    async def handle_event(self, event: DomainEvent):
        """处理事件,更新读模型"""
        handler = getattr(self, f"_on_{event.event_type}", None)
        if handler:
            await handler(event)

    async def _on_OrderCreated(self, event: DomainEvent):
        await self.read_db.execute(text("""
            INSERT INTO order_summary (id, user_id, status, total, item_count, created_at)
            VALUES (:id, :user_id, 'created', 0, 0, :created_at)
        """), {
            "id": event.aggregate_id,
            "user_id": event.data["user_id"],
            "created_at": event.timestamp,
        })

    async def _on_OrderItemAdded(self, event: DomainEvent):
        await self.read_db.execute(text("""
            UPDATE order_summary
            SET total = total + :amount,
                item_count = item_count + 1,
                updated_at = :ts
            WHERE id = :id
        """), {
            "id": event.aggregate_id,
            "amount": event.data["price"] * event.data["quantity"],
            "ts": event.timestamp,
        })

    async def _on_OrderConfirmed(self, event: DomainEvent):
        await self.read_db.execute(text("""
            UPDATE order_summary SET status = 'confirmed', updated_at = :ts WHERE id = :id
        """), {"id": event.aggregate_id, "ts": event.timestamp})

# 查询服务
class OrderQueryService:
    """订单查询服务(读端)"""

    def __init__(self, read_db):
        self.read_db = read_db

    async def get_order_summary(self, order_id: str) -> dict:
        result = await self.read_db.execute(text(
            "SELECT * FROM order_summary WHERE id = :id"
        ), {"id": order_id})
        return dict(result.fetchone()._mapping)

    async def list_user_orders(self, user_id: str, page: int = 1, size: int = 20):
        result = await self.read_db.execute(text("""
            SELECT * FROM order_summary
            WHERE user_id = :user_id
            ORDER BY created_at DESC
            LIMIT :size OFFSET :offset
        """), {"user_id": user_id, "size": size, "offset": (page - 1) * size})
        return [dict(row._mapping) for row in result]
```

### Saga模式(编排式)

```python
# Saga编排器 — 管理跨服务事务
from enum import Enum
from dataclasses import dataclass

class SagaStatus(str, Enum):
    STARTED = "started"
    COMPLETED = "completed"
    COMPENSATING = "compensating"
    FAILED = "failed"

@dataclass
class SagaStep:
    name: str
    action: str       # 正向操作
    compensation: str  # 补偿操作
    status: str = "pending"

class OrderSaga:
    """订单创建Saga"""

    def __init__(self, saga_id: str, order_data: dict):
        self.saga_id = saga_id
        self.order_data = order_data
        self.status = SagaStatus.STARTED
        self.completed_steps: list[str] = []
        self.steps = [
            SagaStep("create_order", "order.create", "order.cancel"),
            SagaStep("reserve_inventory", "inventory.reserve", "inventory.release"),
            SagaStep("process_payment", "payment.charge", "payment.refund"),
            SagaStep("confirm_order", "order.confirm", "order.cancel"),
        ]

class SagaOrchestrator:
    """Saga编排器"""

    def __init__(self, event_bus, saga_store):
        self.event_bus = event_bus
        self.saga_store = saga_store

    async def start_saga(self, saga: OrderSaga):
        """开始执行Saga"""
        await self.saga_store.save(saga)
        await self._execute_next_step(saga)

    async def _execute_next_step(self, saga: OrderSaga):
        """执行下一步"""
        for step in saga.steps:
            if step.status == "pending":
                step.status = "executing"
                await self.saga_store.save(saga)
                await self.event_bus.publish_command(step.action, {
                    "saga_id": saga.saga_id,
                    "step": step.name,
                    **saga.order_data,
                })
                return

        # 所有步骤完成
        saga.status = SagaStatus.COMPLETED
        await self.saga_store.save(saga)

    async def handle_step_success(self, saga_id: str, step_name: str):
        """步骤成功"""
        saga = await self.saga_store.load(saga_id)
        for step in saga.steps:
            if step.name == step_name:
                step.status = "completed"
                saga.completed_steps.append(step_name)
                break
        await self._execute_next_step(saga)

    async def handle_step_failure(self, saga_id: str, step_name: str, error: str):
        """步骤失败,开始补偿"""
        saga = await self.saga_store.load(saga_id)
        saga.status = SagaStatus.COMPENSATING

        # 逆序补偿已完成的步骤
        for step_name in reversed(saga.completed_steps):
            step = next(s for s in saga.steps if s.name == step_name)
            await self.event_bus.publish_command(step.compensation, {
                "saga_id": saga.saga_id,
                **saga.order_data,
            })

        saga.status = SagaStatus.FAILED
        await self.saga_store.save(saga)
```

### 幂等消费者

```python
# 幂等事件消费
class IdempotentConsumer:
    """幂等消费者 — 防止重复处理"""

    def __init__(self, redis, handler):
        self.redis = redis
        self.handler = handler
        self.ttl = 7 * 24 * 3600  # 7天去重窗口

    async def process(self, event: DomainEvent) -> bool:
        """处理事件(幂等)"""
        dedup_key = f"processed:{event.event_id}"

        # 原子检查并标记
        was_set = await self.redis.set(dedup_key, "1", nx=True, ex=self.ttl)
        if not was_set:
            # 已处理过,跳过
            logger.info("Duplicate event skipped", event_id=event.event_id)
            return False

        try:
            await self.handler(event)
            return True
        except Exception:
            # 处理失败,移除标记以允许重试
            await self.redis.delete(dedup_key)
            raise
```

## 最佳实践

### 1. 事件设计
- 事件名使用过去式(OrderCreated,不是CreateOrder)
- 包含correlation_id追踪因果链
- 事件Schema有版本号,支持演进
- 避免过大的事件(考虑事件携带vs事件通知)

### 2. 事件存储
- 使用追加写入(append-only)
- 乐观并发控制(version check)
- 快照机制优化长事件流(每100个事件做快照)
- 事件存储需要备份和归档策略

### 3. CQRS部署
- 接受读模型的最终一致性
- 投影失败时可以从事件流重建
- 读模型可以有多个(针对不同查询场景)
- 写模型和读模型可以用不同数据库

### 4. Saga最佳实践
- 每个步骤的补偿操作必须是幂等的
- 记录Saga状态便于故障恢复
- 设置超时机制处理步骤无响应
- 补偿操作也可能失败,需要人工介入机制

### 5. 幂等性
- 消费者必须实现幂等(消息可能重复投递)
- 使用事件ID作为去重键
- 去重窗口至少覆盖消息保留时间
- 数据库操作使用UPSERT而非INSERT

## 常见陷阱

### 陷阱1: 事件中包含命令
```python
# 错误: 事件不应该指示下游做什么
event = {"type": "OrderCreated", "data": {"send_email": True}}

# 正确: 事件只描述发生了什么,消费者自行决策
event = {"type": "OrderCreated", "data": {"order_id": "123", "user_id": "456"}}
# 邮件服务订阅OrderCreated事件,自行决定是否发邮件
```

### 陷阱2: 双重写入问题
```python
# 错误: 先写数据库再发事件,可能不一致
await db.save(order)
await event_bus.publish(OrderCreated(...))  # 如果这里失败?数据库有,事件没有

# 正确: Transactional Outbox模式
async with db.begin() as tx:
    await tx.save(order)
    await tx.save_outbox_event(OrderCreated(...))  # 同一事务
# 后台进程轮询outbox表发送到消息代理
```

### 陷阱3: 事件顺序依赖
```python
# 错误: 假设事件总是按时间顺序到达
# 分布式系统中事件可能乱序

# 正确: 使用版本号检测乱序
async def handle_event(event):
    current = await read_db.get_version(event.aggregate_id)
    if event.version <= current:
        return  # 旧事件,跳过
    if event.version > current + 1:
        # 丢失了中间事件,请求重放
        await request_replay(event.aggregate_id, current + 1)
        return
    await process(event)
```

### 陷阱4: 投影重建成本过高
```python
# 错误: 百万级事件流每次都从头重建投影
# 正确: 使用快照 + 增量重建
async def rebuild_projection(aggregate_id: str):
    snapshot = await snapshot_store.get_latest(aggregate_id)
    if snapshot:
        state = snapshot.state
        events = await event_store.load_after(aggregate_id, snapshot.version)
    else:
        state = initial_state()
        events = await event_store.load(aggregate_id)

    for event in events:
        state = apply_event(state, event)
    return state
```

## Agent Checklist

### 事件设计
- [ ] 事件使用过去式命名
- [ ] 包含event_id/correlation_id/timestamp
- [ ] 事件Schema有版本管理
- [ ] 事件大小合理(不过大)

### 事件存储
- [ ] 追加写入(不可修改)
- [ ] 乐观并发控制
- [ ] 快照机制(长事件流)
- [ ] 备份和归档策略

### CQRS
- [ ] 读写模型分离
- [ ] 投影可从事件流重建
- [ ] 最终一致性可接受
- [ ] 多个读模型按查询场景优化

### 可靠性
- [ ] 消费者幂等处理
- [ ] 死信队列处理失败消息
- [ ] Saga补偿操作已实现
- [ ] 分布式追踪贯穿事件链
