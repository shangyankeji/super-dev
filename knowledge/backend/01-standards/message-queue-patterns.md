---
id: message-queue-patterns
title: 消息队列模式完整指南
domain: backend
category: 01-standards
difficulty: intermediate
tags: [backend, kafka, message, patterns, queue, rabbitmq, redis, streams]
quality_score: 70
last_updated: 2026-06-15
---
# 消息队列模式完整指南

## 概述

消息队列是分布式系统的核心基础设施，实现服务解耦、异步处理、流量削峰和事件驱动架构。本指南覆盖 Kafka、RabbitMQ 和 Redis Streams 的选型、核心模式和最佳实践。

---

## 方案选型对比

| 特性 | Kafka | RabbitMQ | Redis Streams |
|------|-------|----------|---------------|
| 定位 | 分布式事件流平台 | 消息代理 | 轻量级消息流 |
| 吞吐量 | 百万级/秒 | 万级/秒 | 十万级/秒 |
| 消息持久化 | 磁盘，可配置保留期 | 内存+磁盘 | 内存+RDB/AOF |
| 消费模型 | Consumer Group + Offset | Queue/Exchange 路由 | Consumer Group |
| 消息顺序 | Partition 内有序 | Queue 内有序 | Stream 内有序 |
| 延迟 | 毫秒级 | 微秒级 | 微秒级 |
| 适用场景 | 事件溯源/日志/大数据 | 任务队列/RPC/路由 | 轻量事件/缓存层流 |

---

## Kafka

### 核心概念

```
Producer -> Topic (Partition 0, 1, 2...) -> Consumer Group
                                             ├── Consumer A (Partition 0, 1)
                                             └── Consumer B (Partition 2)
```

### 生产者

```python
from confluent_kafka import Producer

producer = Producer({
    "bootstrap.servers": "kafka:9092",
    "acks": "all",                    # 等待所有副本确认
    "retries": 3,
    "enable.idempotence": True,       # 幂等生产者
    "max.in.flight.requests.per.connection": 5,
})

def send_event(topic: str, key: str, value: dict):
    import json
    producer.produce(
        topic=topic,
        key=key.encode("utf-8"),
        value=json.dumps(value).encode("utf-8"),
        callback=delivery_report,
    )
    producer.flush()

def delivery_report(err, msg):
    if err:
        logger.error(f"Message delivery failed: {err}")
    else:
        logger.info(f"Message delivered to {msg.topic()} [{msg.partition()}]")
```

### 消费者

```python
from confluent_kafka import Consumer, KafkaError

consumer = Consumer({
    "bootstrap.servers": "kafka:9092",
    "group.id": "order-service",
    "auto.offset.reset": "earliest",
    "enable.auto.commit": False,     # 手动提交 offset
    "max.poll.interval.ms": 300000,
})

consumer.subscribe(["order-events"])

try:
    while True:
        msg = consumer.poll(timeout=1.0)
        if msg is None:
            continue
        if msg.error():
            if msg.error().code() == KafkaError._PARTITION_EOF:
                continue
            raise KafkaException(msg.error())

        try:
            event = json.loads(msg.value().decode("utf-8"))
            process_order_event(event)
            consumer.commit(msg)     # 处理成功后手动提交
        except ProcessingError as e:
            logger.error(f"Failed to process: {e}")
            send_to_dlq(msg)         # 发送到死信队列
            consumer.commit(msg)
finally:
    consumer.close()
```

### Kafka 最佳实践

- **分区数**: 通常等于消费者数量的倍数
- **Key 设计**: 使用业务 ID 保证同一实体的消息有序
- **保留策略**: 根据业务需求设置（7 天/30 天/永久）
- **压缩**: 使用 lz4 或 snappy 减少网络和存储开销

---

## RabbitMQ

### Exchange 类型

```
Direct:   routing_key 精确匹配
Fanout:   广播到所有绑定队列
Topic:    routing_key 模式匹配 (*.error, order.#)
Headers:  基于 header 属性匹配
```

### 生产者

```python
import pika
import json

connection = pika.BlockingConnection(pika.ConnectionParameters(
    host="rabbitmq",
    credentials=pika.PlainCredentials("guest", "guest"),
    heartbeat=600,
))
channel = connection.channel()

# 声明持久化队列和交换机
channel.exchange_declare(exchange="orders", exchange_type="topic", durable=True)
channel.queue_declare(queue="order-processing", durable=True)
channel.queue_bind(queue="order-processing", exchange="orders", routing_key="order.created")

def publish_order_event(order_id: str, event_type: str, data: dict):
    channel.basic_publish(
        exchange="orders",
        routing_key=f"order.{event_type}",
        body=json.dumps({"order_id": order_id, **data}),
        properties=pika.BasicProperties(
            delivery_mode=2,        # 持久化消息
            content_type="application/json",
            message_id=str(uuid.uuid4()),
            timestamp=int(time.time()),
        ),
    )
```

### 消费者

```python
def callback(ch, method, properties, body):
    try:
        event = json.loads(body)
        process_order(event)
        ch.basic_ack(delivery_tag=method.delivery_tag)
    except Exception as e:
        logger.error(f"Processing failed: {e}")
        # 拒绝并重新入队（或发送到 DLQ）
        ch.basic_nack(delivery_tag=method.delivery_tag, requeue=False)

channel.basic_qos(prefetch_count=10)  # 流控
channel.basic_consume(queue="order-processing", on_message_callback=callback)
channel.start_consuming()
```

### 死信队列 (DLQ)

```python
# 声明 DLQ
channel.queue_declare(queue="order-processing-dlq", durable=True)

# 主队列绑定 DLQ
channel.queue_declare(
    queue="order-processing",
    durable=True,
    arguments={
        "x-dead-letter-exchange": "",
        "x-dead-letter-routing-key": "order-processing-dlq",
        "x-message-ttl": 86400000,     # 消息 TTL: 24h
        "x-max-length": 100000,        # 队列最大长度
    },
)
```

---

## Redis Streams

### 基本操作

```python
import redis

r = redis.Redis(host="localhost", port=6379, decode_responses=True)

# 生产者：添加消息到 Stream
message_id = r.xadd("order-events", {
    "order_id": "ORD-001",
    "event": "created",
    "amount": "99.99",
    "timestamp": str(int(time.time())),
})

# 创建消费者组
r.xgroup_create("order-events", "order-service", id="0", mkstream=True)

# 消费者：读取消息
while True:
    messages = r.xreadgroup(
        groupname="order-service",
        consumername="worker-1",
        streams={"order-events": ">"},
        count=10,
        block=5000,
    )

    for stream, entries in messages:
        for msg_id, fields in entries:
            try:
                process_event(fields)
                r.xack("order-events", "order-service", msg_id)
            except Exception as e:
                logger.error(f"Failed: {e}")
                # 消息留在 PEL 中，稍后重试

# 处理 pending 消息（故障恢复）
pending = r.xpending_range("order-events", "order-service", "-", "+", count=100)
for entry in pending:
    if entry["time_since_delivered"] > 300000:  # 超过 5 分钟未确认
        messages = r.xclaim(
            "order-events", "order-service", "worker-1",
            min_idle_time=300000,
            message_ids=[entry["message_id"]],
        )
        for msg_id, fields in messages:
            reprocess_event(fields)
            r.xack("order-events", "order-service", msg_id)
```

---

## 事件驱动架构模式

### 事件溯源 (Event Sourcing)

```python
# 事件存储
class EventStore:
    def __init__(self, producer: KafkaProducer):
        self.producer = producer

    def append(self, aggregate_id: str, event: DomainEvent):
        self.producer.send(
            topic=f"events-{event.aggregate_type}",
            key=aggregate_id,
            value={
                "event_id": str(uuid.uuid4()),
                "aggregate_id": aggregate_id,
                "event_type": event.__class__.__name__,
                "data": event.to_dict(),
                "version": event.version,
                "timestamp": datetime.utcnow().isoformat(),
            },
        )

# 事件重放
class OrderAggregate:
    def __init__(self):
        self.status = None
        self.items = []
        self.total = 0

    def apply(self, event: DomainEvent):
        if isinstance(event, OrderCreated):
            self.status = "created"
            self.items = event.items
        elif isinstance(event, OrderPaid):
            self.status = "paid"
            self.total = event.amount
```

### 幂等消费者

```python
class IdempotentConsumer:
    def __init__(self, redis_client: redis.Redis):
        self.redis = redis_client
        self.ttl = 86400 * 7  # 7 天去重窗口

    def process(self, message_id: str, handler: Callable):
        key = f"processed:{message_id}"
        if self.redis.exists(key):
            logger.info(f"Duplicate message {message_id}, skipping")
            return

        handler()

        self.redis.set(key, "1", ex=self.ttl)
```

### 重试策略

```python
import time
from functools import wraps

def retry_with_backoff(max_retries=3, base_delay=1.0, max_delay=60.0):
    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            for attempt in range(max_retries + 1):
                try:
                    return func(*args, **kwargs)
                except RetryableError as e:
                    if attempt == max_retries:
                        raise
                    delay = min(base_delay * (2 ** attempt), max_delay)
                    jitter = delay * 0.1 * random.random()
                    time.sleep(delay + jitter)
                    logger.warning(f"Retry {attempt + 1}/{max_retries}: {e}")
        return wrapper
    return decorator
```

---

## 监控指标

| 指标 | 说明 | 告警阈值 |
|------|------|----------|
| Consumer Lag | 消费者落后的消息数 | > 10000 |
| Message Rate | 每秒消息产生/消费量 | 异常波动 |
| Error Rate | 处理失败率 | > 1% |
| DLQ Size | 死信队列堆积数 | > 100 |
| Processing Latency | 消息处理延迟 P99 | > 5s |

---

## 常见反模式

| 反模式 | 问题 | 正确做法 |
|--------|------|----------|
| 消息体过大 | 网络/存储开销 | 消息只传 ID + 元数据，数据走存储 |
| 不做幂等 | 重复处理 | 消息 ID 去重 + 幂等写入 |
| 自动提交 offset | 消息丢失 | 处理成功后手动提交 |
| 忽略 DLQ | 失败消息丢失 | 配置死信队列并监控 |
| 无重试策略 | 临时故障导致消息丢失 | 指数退避重试 |
| 强依赖消息顺序 | 扩展困难 | 仅在必要时保证分区内有序 |

---

## Agent Checklist

- [ ] 根据吞吐量和延迟需求选择合适的消息队列
- [ ] 生产者启用幂等写入和重试
- [ ] 消费者手动提交 offset/ack
- [ ] 实现幂等消费者（消息 ID 去重）
- [ ] 配置死信队列 (DLQ) 并监控大小
- [ ] 实现指数退避重试策略
- [ ] 消息体控制在 1MB 以内
- [ ] Kafka 分区 Key 设计保证业务有序性
- [ ] 消费者 Group 内消费者数 <= 分区数
- [ ] 监控 Consumer Lag / Error Rate / DLQ Size
- [ ] 消息 Schema 有版本管理（Avro/Protobuf + Schema Registry）
- [ ] 生产环境 Kafka 至少 3 副本，acks=all
