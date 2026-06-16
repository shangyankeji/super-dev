---
id: message-queue-playbook
title: 消息队列生产实战手册（RabbitMQ/Kafka）
domain: cloud-native
category: 02-playbooks
difficulty: advanced
tags: [message-queue, rabbitmq, kafka, dead-letter-queue, dlq, retry, backoff, outbox, idempotency, production, messaging, async]
quality_score: 94
maintainer: platform-team@superdev.com
last_updated: 2026-06-15
---

# 消息队列生产实战手册（RabbitMQ / Kafka）

> 基于 [Confluent DLQ Guide](https://www.confluent.io/learn/kafka-dead-letter-queue/) + [Redpanda Reliable Processing](https://www.redpanda.com/blog/reliable-message-processing-with-dead-letter-queue) + [Transactional Outbox with RabbitMQ](https://dev.to/sagarmaheshwary/transactional-outbox-with-rabbitmq-part-2-handling-retries-dead-letter-queues-and-observability-4h19)

## 三层错误处理

### 1. 即时重试（瞬时故障）
```python
# 网络抖动 / 临时不可达 → 立刻重试 3 次
@retry(max_attempts=3, backoff=1s)
def process_message(msg):
    external_api.call(msg)  # 可能瞬时失败
```

### 2. 延迟重试队列（可恢复故障）
```
# 消费失败 → 进延迟队列，递增等待后重试
main-queue → 失败 → retry-1 (等 10s) → 失败 → retry-2 (等 30s) → 失败 → retry-3 (等 60s) → 失败 → DLQ
```

### 3. Dead Letter Queue（不可恢复故障）
```python
# 超过最大重试 → 进入 DLQ，人工/脚本处理
def handle_message(msg):
    try:
        process(msg)
    except NonRetryableError as e:
        # 格式错误/数据不合法 → 直接进 DLQ
        send_to_dlq(msg, reason=str(e))
    except RetryableError as e:
        # 网络/超时 → 延迟重试
        retry_with_backoff(msg)
```

## RabbitMQ 配置

```python
# 声明 DLX（Dead Letter Exchange）
channel.exchange_declare('orders.dlx', exchange_type='direct')
channel.queue_declare('orders.dlq', arguments={
    'x-dead-letter-exchange': '',       # DLQ 不再转发
})

# 主队列绑定 DLX
channel.queue_declare('orders', arguments={
    'x-dead-letter-exchange': 'orders.dlx',
    'x-delivery-limit': 5,              # 5 次后进 DLQ（quorum queue）
})
```

## Kafka 重试 Topic 模式

```python
# 多级重试 topic，递增延迟
topics = ['orders', 'orders.retry.5s', 'orders.retry.30s', 'orders.retry.120s', 'orders.DLT']

# 消费 main topic → 失败 → 发到 retry.5s
# 消费 retry.5s → 失败 → 发到 retry.30s
# ... 最终 → DLT（Dead Letter Topic）

consumer.subscribe(['orders'])
for msg in consumer:
    try:
        process(msg)
    except Exception:
        produce('orders.retry.5s', msg)
```

## Transactional Outbox（可靠投递）

```python
# 问题：DB 提交了但消息没发出（或反过来）
# 解决：在同一 DB 事务里写 outbox 表

@app.post("/orders")
@db.transaction()
def create_order(data):
    order = db.insert(Order(**data))
    # 同一事务写 outbox（保证原子性）
    db.insert(Outbox(
        aggregate_type='order',
        aggregate_id=order.id,
        event_type='order.created',
        payload=json.dumps(order.to_dict()),
    ))
    return order

# 独立 worker 轮询 outbox → 投递到 MQ → 标记已发
def outbox_relay():
    pending = db.query(Outbox).filter_by(sent=False).limit(100).all()
    for msg in pending:
        mq.publish('orders', msg.payload)
        msg.sent = True
    db.commit()
```

## 幂等消费

```python
# 同一消息可能被投递多次（at-least-once），消费必须幂等
def process_message(msg):
    # 用消息 ID 去重
    if redis.exists(f"processed:{msg.id}"):
        return  # 已处理过，跳过
    do_business_logic(msg)
    redis.setex(f"processed:{msg.id}", 86400, "1")  # 24h 去重窗口
```

## 生产检查清单
- [ ] 消费者是幂等的（at-least-once 语义下安全）
- [ ] 有 DLQ + 监控 DLQ 深度告警
- [ ] 重试有上限（不会无限重试）
- [ ] 重试用递增退避（1s → 10s → 30s）
- [ ] 区分可恢复 vs 不可恢复错误
- [ ] 用 Transactional Outbox 保证 DB + MQ 一致
- [ ] 消息含追溯信息（traceId / orderId / timestamp）
- [ ] DLQ 消息可回放（修复后重新入队）
