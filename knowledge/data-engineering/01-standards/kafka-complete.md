---
id: kafka-complete
title: Apache Kafka完整指南
domain: data-engineering
category: 01-standards
difficulty: intermediate
tags: [complete, connect, data-engineering, kafka, schema管理, streams, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Apache Kafka完整指南

## 概述

Apache Kafka是一个分布式事件流平台,用于高吞吐量、低延迟的实时数据管道和流处理。最初由LinkedIn开发,后捐赠给Apache基金会。Kafka以其持久化日志模型、水平扩展能力和容错设计,成为现代数据架构的核心基础设施。

### 消息队列对比

| 特性 | Kafka | RabbitMQ | Redis Streams | Pulsar |
|------|-------|----------|---------------|--------|
| 模型 | 分布式日志 | AMQP消息代理 | 内存流 | 分层存储日志 |
| 吞吐量 | 百万级/秒 | 万级/秒 | 十万级/秒 | 百万级/秒 |
| 延迟 | 毫秒级 | 微秒级 | 亚毫秒级 | 毫秒级 |
| 持久化 | 磁盘顺序写 | 可选持久化 | AOF/RDB | BookKeeper |
| 消息回溯 | 支持(Offset) | 不支持 | 支持(ID) | 支持(MessageID) |
| 消费模式 | 拉取(Pull) | 推送(Push) | 拉取/阻塞读 | 推送+拉取 |
| 协议 | 自有二进制协议 | AMQP/MQTT/STOMP | Redis协议 | 自有二进制协议 |
| 多租户 | 有限(ACL) | VHost隔离 | 无原生支持 | 原生多租户 |
| 存算分离 | KRaft模式部分支持 | 不支持 | 不支持 | 原生支持 |
| 适用场景 | 事件流/日志聚合/CDC | 任务队列/RPC | 轻量实时流 | 大规模多租户流 |

**选型建议**:
- **高吞吐事件流/日志采集/CDC**: 选择Kafka
- **复杂路由/任务队列/低延迟RPC**: 选择RabbitMQ
- **轻量级实时流/已有Redis生态**: 选择Redis Streams
- **多租户/存算分离/跨地域复制**: 选择Pulsar

## 核心概念

### 1. Broker

Broker是Kafka集群中的单个服务器节点,负责消息的接收、存储和分发。

```
Kafka集群拓扑:
┌─────────┐  ┌─────────┐  ┌─────────┐
│ Broker 0│  │ Broker 1│  │ Broker 2│
│ (Leader) │  │(Follower)│  │(Follower)│
│ P0,P3   │  │ P1,P4   │  │ P2,P5   │
└─────────┘  └─────────┘  └─────────┘
      │            │            │
      └────────────┼────────────┘
                   │
          ┌────────┴────────┐
          │  ZooKeeper/KRaft │
          └─────────────────┘
```

**关键配置**:
```properties
# server.properties
broker.id=0
listeners=PLAINTEXT://0.0.0.0:9092
advertised.listeners=PLAINTEXT://kafka-broker-0:9092
log.dirs=/var/kafka-logs
num.partitions=6
default.replication.factor=3
min.insync.replicas=2
log.retention.hours=168
log.segment.bytes=1073741824
```

### 2. Topic与Partition

Topic是消息的逻辑分类,Partition是Topic的物理分片,是Kafka并行处理的基本单元。

```
Topic: order-events (3 Partitions, RF=3)

Partition 0: [msg0, msg3, msg6, msg9,  ...] → Leader: Broker 0
Partition 1: [msg1, msg4, msg7, msg10, ...] → Leader: Broker 1
Partition 2: [msg2, msg5, msg8, msg11, ...] → Leader: Broker 2

每条消息在Partition内有唯一递增的Offset:
Partition 0: offset 0 → offset 1 → offset 2 → ...
```

**Topic管理**:
```bash
# 创建Topic
kafka-topics.sh --bootstrap-server localhost:9092 \
  --create --topic order-events \
  --partitions 6 --replication-factor 3

# 查看Topic列表
kafka-topics.sh --bootstrap-server localhost:9092 --list

# 查看Topic详情
kafka-topics.sh --bootstrap-server localhost:9092 \
  --describe --topic order-events

# 修改Partition数(只能增加不能减少)
kafka-topics.sh --bootstrap-server localhost:9092 \
  --alter --topic order-events --partitions 12

# 删除Topic
kafka-topics.sh --bootstrap-server localhost:9092 \
  --delete --topic order-events
```

### 3. Consumer Group

Consumer Group是一组协同消费同一Topic的消费者。同一组内每个Partition只被一个消费者消费,实现负载均衡。

```
Consumer Group: order-processing-group

Topic: order-events (6 Partitions)

Consumer A ← P0, P1
Consumer B ← P2, P3
Consumer C ← P4, P5

如果Consumer B宕机:
Consumer A ← P0, P1, P2
Consumer C ← P3, P4, P5  (触发Rebalance)
```

### 4. Offset管理

Offset是消息在Partition中的位置标识,Consumer通过Offset追踪消费进度。

```
Partition 0:
  ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐
  │ 0 │ 1 │ 2 │ 3 │ 4 │ 5 │ 6 │ 7 │ 8 │ 9 │
  └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘
                ↑               ↑           ↑
          committed         current       LEO
           offset           position   (Log End Offset)

committed offset: 已提交的消费位移
current position: 当前消费位置
LEO: 日志末端偏移量(下一条写入位置)
HW (High Watermark): 已同步到所有ISR副本的最大Offset
```

### 5. ISR/Leader/Follower

ISR（In-Sync Replicas）是与Leader保持同步的副本集合。

```
Partition 0 (RF=3):
  Leader:   Broker 0  (接收读写)
  Follower: Broker 1  (ISR成员, 同步中)
  Follower: Broker 2  (ISR成员, 同步中)

当Follower落后超过replica.lag.time.max.ms时,被移出ISR:
  ISR: [0, 1, 2] → [0, 1]  (Broker 2被移出)

Leader选举: 只从ISR中选举新Leader
  如果unclean.leader.election.enable=true, 允许从非ISR选举(可能丢数据)
```

## 生产者

### 1. 基础生产者

```java
// Java生产者
Properties props = new Properties();
props.put("bootstrap.servers", "kafka1:9092,kafka2:9092,kafka3:9092");
props.put("key.serializer", "org.apache.kafka.common.serialization.StringSerializer");
props.put("value.serializer", "org.apache.kafka.common.serialization.StringSerializer");

KafkaProducer<String, String> producer = new KafkaProducer<>(props);

// 异步发送
producer.send(new ProducerRecord<>("order-events", "order-123", orderJson),
    (metadata, exception) -> {
        if (exception != null) {
            log.error("发送失败", exception);
        } else {
            log.info("发送成功: topic={}, partition={}, offset={}",
                metadata.topic(), metadata.partition(), metadata.offset());
        }
    });

// 同步发送
RecordMetadata metadata = producer.send(
    new ProducerRecord<>("order-events", "order-123", orderJson)).get();

producer.close();
```

```python
# Python生产者(confluent-kafka)
from confluent_kafka import Producer

conf = {
    'bootstrap.servers': 'kafka1:9092,kafka2:9092,kafka3:9092',
    'client.id': 'order-producer',
    'acks': 'all',
}

producer = Producer(conf)

def delivery_callback(err, msg):
    if err:
        print(f'发送失败: {err}')
    else:
        print(f'发送成功: topic={msg.topic()}, partition={msg.partition()}, offset={msg.offset()}')

producer.produce(
    topic='order-events',
    key='order-123',
    value=order_json.encode('utf-8'),
    callback=delivery_callback
)
producer.flush()
```

### 2. 分区策略

```java
// 默认分区策略:
// 1. 指定partition → 直接使用
// 2. 有key → hash(key) % numPartitions
// 3. 无key → 粘性分区(Sticky Partitioner, Kafka 2.4+)

// 自定义分区器
public class OrderPartitioner implements Partitioner {
    @Override
    public int partition(String topic, Object key, byte[] keyBytes,
                         Object value, byte[] valueBytes, Cluster cluster) {
        List<PartitionInfo> partitions = cluster.partitionsForTopic(topic);
        int numPartitions = partitions.size();

        if (key == null) {
            // 无key使用轮询
            return ThreadLocalRandom.current().nextInt(numPartitions);
        }

        String orderKey = (String) key;
        // VIP订单路由到专用分区
        if (orderKey.startsWith("VIP-")) {
            return 0;
        }
        // 其他订单按key哈希
        return Math.abs(Utils.murmur2(keyBytes)) % numPartitions;
    }

    @Override
    public void close() {}

    @Override
    public void configure(Map<String, ?> configs) {}
}

// 使用自定义分区器
props.put("partitioner.class", "com.example.OrderPartitioner");
```

### 3. 幂等性与Exactly-Once语义

```properties
# 幂等生产者配置(防止重复发送)
enable.idempotence=true
acks=all
retries=2147483647
max.in.flight.requests.per.connection=5
```

```java
// 事务性生产者(跨分区Exactly-Once)
props.put("enable.idempotence", "true");
props.put("transactional.id", "order-tx-producer-1");

KafkaProducer<String, String> producer = new KafkaProducer<>(props);
producer.initTransactions();

try {
    producer.beginTransaction();

    // 发送到多个Topic/Partition(原子操作)
    producer.send(new ProducerRecord<>("order-events", orderKey, orderJson));
    producer.send(new ProducerRecord<>("inventory-events", skuKey, inventoryJson));
    producer.send(new ProducerRecord<>("payment-events", paymentKey, paymentJson));

    // 提交消费位移(消费-转换-生产模式)
    producer.sendOffsetsToTransaction(offsets, consumerGroupMetadata);

    producer.commitTransaction();
} catch (ProducerFencedException | OutOfOrderSequenceException e) {
    producer.close();  // 不可恢复的错误
} catch (KafkaException e) {
    producer.abortTransaction();  // 可恢复的错误,回滚
}
```

### 4. 批处理与压缩

```properties
# 批处理配置
batch.size=65536               # 批次大小(字节), 默认16384
linger.ms=20                   # 等待时间(毫秒), 默认0
buffer.memory=67108864         # 缓冲区总大小(64MB)

# 压缩配置
compression.type=lz4           # 可选: none, gzip, snappy, lz4, zstd
# 压缩效果对比:
# gzip:  压缩率最高, CPU消耗最大
# snappy: 压缩率中等, CPU消耗低
# lz4:   压缩率中等, 速度最快
# zstd:  压缩率高, 速度快(推荐Kafka 2.1+)
```

### 5. acks配置与可靠性

```properties
# acks=0: 不等待确认(最快, 可能丢数据)
# acks=1: 等待Leader确认(默认, Leader宕机可能丢数据)
# acks=all/-1: 等待所有ISR确认(最安全, 配合min.insync.replicas)
acks=all

# 可靠性最佳组合
acks=all
min.insync.replicas=2
replication.factor=3
# 保证: 即使1个Broker宕机也不丢数据
```

## 消费者

### 1. 基础消费者

```java
// Java消费者
Properties props = new Properties();
props.put("bootstrap.servers", "kafka1:9092,kafka2:9092,kafka3:9092");
props.put("group.id", "order-processing-group");
props.put("key.deserializer", "org.apache.kafka.common.serialization.StringDeserializer");
props.put("value.deserializer", "org.apache.kafka.common.serialization.StringDeserializer");
props.put("auto.offset.reset", "earliest");

KafkaConsumer<String, String> consumer = new KafkaConsumer<>(props);
consumer.subscribe(Arrays.asList("order-events"));

try {
    while (true) {
        ConsumerRecords<String, String> records = consumer.poll(Duration.ofMillis(1000));
        for (ConsumerRecord<String, String> record : records) {
            log.info("消费: topic={}, partition={}, offset={}, key={}, value={}",
                record.topic(), record.partition(), record.offset(),
                record.key(), record.value());
            processOrder(record.value());
        }
    }
} finally {
    consumer.close();
}
```

```python
# Python消费者(confluent-kafka)
from confluent_kafka import Consumer

conf = {
    'bootstrap.servers': 'kafka1:9092,kafka2:9092,kafka3:9092',
    'group.id': 'order-processing-group',
    'auto.offset.reset': 'earliest',
    'enable.auto.commit': False,
}

consumer = Consumer(conf)
consumer.subscribe(['order-events'])

try:
    while True:
        msg = consumer.poll(timeout=1.0)
        if msg is None:
            continue
        if msg.error():
            print(f'消费错误: {msg.error()}')
            continue

        process_order(msg.value().decode('utf-8'))
        consumer.commit(asynchronous=False)
finally:
    consumer.close()
```

### 2. 自动提交vs手动提交

```java
// 自动提交(简单但可能重复消费或丢失)
props.put("enable.auto.commit", "true");
props.put("auto.commit.interval.ms", "5000");

// 手动同步提交(逐条)
props.put("enable.auto.commit", "false");

for (ConsumerRecord<String, String> record : records) {
    processOrder(record.value());
    // 处理完一条提交一次(性能差但最安全)
    consumer.commitSync(Collections.singletonMap(
        new TopicPartition(record.topic(), record.partition()),
        new OffsetAndMetadata(record.offset() + 1)
    ));
}

// 手动同步提交(批次)
for (ConsumerRecord<String, String> record : records) {
    processOrder(record.value());
}
consumer.commitSync();  // 处理完一批再提交

// 手动异步提交(高性能)
consumer.commitAsync((offsets, exception) -> {
    if (exception != null) {
        log.error("提交失败: {}", offsets, exception);
    }
});

// 最佳实践: 异步+同步混合
try {
    while (true) {
        ConsumerRecords<String, String> records = consumer.poll(Duration.ofMillis(1000));
        for (ConsumerRecord<String, String> record : records) {
            processOrder(record.value());
        }
        consumer.commitAsync();  // 正常用异步
    }
} catch (Exception e) {
    log.error("消费异常", e);
} finally {
    consumer.commitSync();  // 关闭前用同步确保提交
    consumer.close();
}
```

### 3. Rebalance策略

```java
// Rebalance触发条件:
// 1. 消费者加入/离开Group
// 2. 订阅Topic的Partition数变化
// 3. 消费者心跳超时(session.timeout.ms)
// 4. 消费者处理超时(max.poll.interval.ms)

// 关键配置
props.put("session.timeout.ms", "30000");          // 心跳超时
props.put("heartbeat.interval.ms", "10000");        // 心跳间隔(建议session.timeout的1/3)
props.put("max.poll.interval.ms", "300000");         // 两次poll最大间隔
props.put("max.poll.records", "500");                // 单次poll最大记录数

// 分区分配策略
props.put("partition.assignment.strategy",
    "org.apache.kafka.clients.consumer.CooperativeStickyAssignor");
// 可选策略:
// RangeAssignor: 按范围分配(默认), 可能不均匀
// RoundRobinAssignor: 轮询分配, 较均匀
// StickyAssignor: 粘性分配, 尽量保持原有分配
// CooperativeStickyAssignor: 增量式协同Rebalance(推荐, 避免Stop-the-world)

// Rebalance监听器(用于保存中间状态)
consumer.subscribe(Arrays.asList("order-events"), new ConsumerRebalanceListener() {
    @Override
    public void onPartitionsRevoked(Collection<TopicPartition> partitions) {
        // 分区被回收前: 提交当前offset, 保存处理状态
        consumer.commitSync();
        log.info("分区被回收: {}", partitions);
    }

    @Override
    public void onPartitionsAssigned(Collection<TopicPartition> partitions) {
        // 分区被分配后: 恢复处理状态
        log.info("分区被分配: {}", partitions);
    }
});
```

### 4. 消费策略

```java
// 从指定Offset消费
consumer.assign(Arrays.asList(new TopicPartition("order-events", 0)));
consumer.seek(new TopicPartition("order-events", 0), 1000L);

// 从指定时间戳消费
Map<TopicPartition, Long> timestamps = new HashMap<>();
timestamps.put(new TopicPartition("order-events", 0),
    Instant.parse("2026-03-01T00:00:00Z").toEpochMilli());
Map<TopicPartition, OffsetAndTimestamp> offsets =
    consumer.offsetsForTimes(timestamps);
for (Map.Entry<TopicPartition, OffsetAndTimestamp> entry : offsets.entrySet()) {
    consumer.seek(entry.getKey(), entry.getValue().offset());
}

// 从头消费
consumer.seekToBeginning(consumer.assignment());

// 从末尾消费
consumer.seekToEnd(consumer.assignment());
```

## 集群管理

### 1. 副本与ISR配置

```properties
# Broker端配置
default.replication.factor=3        # 默认副本因子
min.insync.replicas=2               # 最小ISR数量
replica.lag.time.max.ms=30000       # 副本最大落后时间
unclean.leader.election.enable=false # 禁止不干净的选举(防止丢数据)

# Topic级别覆盖
kafka-configs.sh --bootstrap-server localhost:9092 \
  --entity-type topics --entity-name order-events \
  --alter --add-config min.insync.replicas=2
```

### 2. Controller角色

```
Controller职责:
1. 分区Leader选举
2. 副本状态管理
3. Topic创建/删除
4. Broker上下线处理

ZooKeeper模式:
  集群中一个Broker担任Controller
  Controller通过ZooKeeper的临时节点选举
  Controller将元数据写入ZooKeeper

KRaft模式(Kafka 3.3+, 推荐):
  不再依赖ZooKeeper
  使用Raft协议进行Controller选举
  元数据存储在Kafka自身的__cluster_metadata Topic中
```

### 3. ZooKeeper vs KRaft

```properties
# ZooKeeper模式配置(旧版)
zookeeper.connect=zk1:2181,zk2:2181,zk3:2181/kafka
zookeeper.connection.timeout.ms=18000

# KRaft模式配置(Kafka 3.3+, 推荐)
process.roles=broker,controller    # 或只设broker/controller
node.id=1
controller.quorum.voters=1@kafka1:9093,2@kafka2:9093,3@kafka3:9093
controller.listener.names=CONTROLLER
listeners=PLAINTEXT://:9092,CONTROLLER://:9093

# KRaft优势:
# 1. 去除ZooKeeper依赖, 简化运维
# 2. 更快的Controller切换(秒级→毫秒级)
# 3. 支持更多Partition(百万级)
# 4. 元数据同步更高效
```

**KRaft迁移步骤**:
```bash
# 1. 生成集群ID
kafka-storage.sh random-uuid

# 2. 格式化存储
kafka-storage.sh format -t <cluster-id> -c config/kraft/server.properties

# 3. 启动KRaft节点
kafka-server-start.sh config/kraft/server.properties

# 4. 从ZooKeeper迁移(Kafka 3.6+支持在线迁移)
kafka-metadata.sh --snapshot /path/to/snapshot \
  --cluster-id <cluster-id>
```

## Schema管理

### 1. Schema Registry

```
生产者 → Schema Registry → Kafka Broker → Schema Registry → 消费者
         (注册Schema)      (存储消息)      (获取Schema)

Schema Registry存储层: _schemas (Kafka内部Topic)
兼容性检查: 写入时验证新Schema与已有Schema的兼容性
```

### 2. Avro Schema

```json
{
  "type": "record",
  "name": "OrderEvent",
  "namespace": "com.example.events",
  "fields": [
    {"name": "orderId", "type": "string"},
    {"name": "userId", "type": "string"},
    {"name": "amount", "type": "double"},
    {"name": "currency", "type": "string", "default": "CNY"},
    {"name": "status", "type": {
      "type": "enum",
      "name": "OrderStatus",
      "symbols": ["CREATED", "PAID", "SHIPPED", "DELIVERED", "CANCELLED"]
    }},
    {"name": "items", "type": {
      "type": "array",
      "items": {
        "type": "record",
        "name": "OrderItem",
        "fields": [
          {"name": "skuId", "type": "string"},
          {"name": "quantity", "type": "int"},
          {"name": "price", "type": "double"}
        ]
      }
    }},
    {"name": "createdAt", "type": {"type": "long", "logicalType": "timestamp-millis"}},
    {"name": "metadata", "type": ["null", {"type": "map", "values": "string"}], "default": null}
  ]
}
```

### 3. Protobuf Schema

```protobuf
syntax = "proto3";
package com.example.events;

message OrderEvent {
  string order_id = 1;
  string user_id = 2;
  double amount = 3;
  string currency = 4;
  OrderStatus status = 5;
  repeated OrderItem items = 6;
  int64 created_at = 7;
  map<string, string> metadata = 8;

  enum OrderStatus {
    CREATED = 0;
    PAID = 1;
    SHIPPED = 2;
    DELIVERED = 3;
    CANCELLED = 4;
  }

  message OrderItem {
    string sku_id = 1;
    int32 quantity = 2;
    double price = 3;
  }
}
```

### 4. 兼容性策略

```
兼容性级别:
┌──────────────────┬──────────────────────────────────────────┐
│ BACKWARD         │ 新Schema可以读旧数据(默认)              │
│ BACKWARD_TRANSITIVE │ 新Schema可以读所有历史版本数据        │
│ FORWARD          │ 旧Schema可以读新数据                    │
│ FORWARD_TRANSITIVE  │ 所有历史版本可以读新数据              │
│ FULL             │ 双向兼容(最新版本)                     │
│ FULL_TRANSITIVE  │ 双向兼容(所有版本)                     │
│ NONE             │ 不检查兼容性(不推荐生产使用)           │
└──────────────────┴──────────────────────────────────────────┘

安全的Schema演进操作:
✅ 添加带默认值的字段(BACKWARD兼容)
✅ 删除带默认值的字段(FORWARD兼容)
✅ 添加可选字段(FULL兼容)
❌ 删除必需字段(破坏BACKWARD)
❌ 修改字段类型(破坏所有兼容性)
❌ 重命名字段(破坏所有兼容性)
```

```bash
# Schema Registry API
# 注册Schema
curl -X POST http://schema-registry:8081/subjects/order-events-value/versions \
  -H "Content-Type: application/vnd.schemaregistry.v1+json" \
  -d '{"schema": "{\"type\":\"record\",\"name\":\"OrderEvent\",...}"}'

# 查看兼容性
curl http://schema-registry:8081/config/order-events-value

# 设置兼容性级别
curl -X PUT http://schema-registry:8081/config/order-events-value \
  -H "Content-Type: application/vnd.schemaregistry.v1+json" \
  -d '{"compatibility": "FULL_TRANSITIVE"}'

# 兼容性测试
curl -X POST http://schema-registry:8081/compatibility/subjects/order-events-value/versions/latest \
  -H "Content-Type: application/vnd.schemaregistry.v1+json" \
  -d '{"schema": "{...}"}'
```

## Kafka Streams

### 1. KStream与KTable

```java
// KStream: 无界事件流(每条记录是独立事件)
// KTable: 变更日志流(每个key只保留最新值,类似数据库表)

StreamsBuilder builder = new StreamsBuilder();

// KStream: 订单事件流
KStream<String, OrderEvent> orderStream = builder.stream("order-events",
    Consumed.with(Serdes.String(), orderEventSerde));

// KTable: 用户信息表(从compacted topic读取)
KTable<String, UserInfo> userTable = builder.table("user-info",
    Materialized.as("user-info-store"));

// 流处理: 过滤 + 转换
KStream<String, EnrichedOrder> enrichedOrders = orderStream
    .filter((key, order) -> order.getAmount() > 0)
    .mapValues(order -> EnrichedOrder.from(order))
    .selectKey((key, order) -> order.getUserId());

// Stream-Table Join(用户信息关联)
KStream<String, OrderWithUser> ordersWithUser = enrichedOrders.join(
    userTable,
    (order, user) -> new OrderWithUser(order, user)
);

ordersWithUser.to("enriched-order-events",
    Produced.with(Serdes.String(), enrichedOrderSerde));
```

### 2. 窗口操作

```java
// 滚动窗口(Tumbling Window): 固定大小,无重叠
KTable<Windowed<String>, Long> tumblingCounts = orderStream
    .groupByKey()
    .windowedBy(TimeWindows.ofSizeWithNoGrace(Duration.ofMinutes(5)))
    .count(Materialized.as("tumbling-counts"));

// 跳跃窗口(Hopping Window): 固定大小,有重叠
KTable<Windowed<String>, Long> hoppingCounts = orderStream
    .groupByKey()
    .windowedBy(TimeWindows.ofSizeAndGrace(
        Duration.ofMinutes(5),
        Duration.ofMinutes(1))
        .advanceBy(Duration.ofMinutes(1)))
    .count(Materialized.as("hopping-counts"));

// 滑动窗口(Sliding Window): 基于时间差
KTable<Windowed<String>, Long> slidingCounts = orderStream
    .groupByKey()
    .windowedBy(SlidingWindows.ofTimeDifferenceAndGrace(
        Duration.ofMinutes(5),
        Duration.ofMinutes(1)))
    .count(Materialized.as("sliding-counts"));

// 会话窗口(Session Window): 基于活动间隔
KTable<Windowed<String>, Long> sessionCounts = orderStream
    .groupByKey()
    .windowedBy(SessionWindows.ofInactivityGapAndGrace(
        Duration.ofMinutes(30),
        Duration.ofMinutes(5)))
    .count(Materialized.as("session-counts"));
```

### 3. 状态存储

```java
// Kafka Streams使用RocksDB作为本地状态存储
// 状态存储自动备份到changelog topic(容错)

// 自定义状态存储
StoreBuilder<KeyValueStore<String, OrderAggregate>> storeBuilder =
    Stores.keyValueStoreBuilder(
        Stores.persistentKeyValueStore("order-aggregate-store"),
        Serdes.String(),
        orderAggregateSerde
    ).withCachingEnabled()
     .withLoggingEnabled(new HashMap<>());  // 启用changelog

builder.addStateStore(storeBuilder);

// 在Processor中使用状态存储
orderStream.process(() -> new Processor<String, OrderEvent, String, OrderAggregate>() {
    private KeyValueStore<String, OrderAggregate> store;

    @Override
    public void init(ProcessorContext<String, OrderAggregate> context) {
        store = context.getStateStore("order-aggregate-store");
    }

    @Override
    public void process(Record<String, OrderEvent> record) {
        OrderAggregate agg = store.get(record.key());
        if (agg == null) agg = new OrderAggregate();
        agg.add(record.value());
        store.put(record.key(), agg);
        context().forward(record.withValue(agg));
    }
}, "order-aggregate-store");
```

## Kafka Connect

### 1. Source Connector(数据导入)

```json
{
  "name": "mysql-source-connector",
  "config": {
    "connector.class": "io.debezium.connector.mysql.MySqlConnector",
    "tasks.max": "1",
    "database.hostname": "mysql-host",
    "database.port": "3306",
    "database.user": "debezium",
    "database.password": "${env:MYSQL_PASSWORD}",
    "database.server.id": "184054",
    "topic.prefix": "cdc-mysql",
    "database.include.list": "ecommerce",
    "table.include.list": "ecommerce.orders,ecommerce.users",
    "schema.history.internal.kafka.bootstrap.servers": "kafka:9092",
    "schema.history.internal.kafka.topic": "schema-history.ecommerce",
    "include.schema.changes": "true",
    "snapshot.mode": "initial",
    "transforms": "route",
    "transforms.route.type": "org.apache.kafka.connect.transforms.RegexRouter",
    "transforms.route.regex": "cdc-mysql\\.ecommerce\\.(.*)",
    "transforms.route.replacement": "cdc.$1"
  }
}
```

### 2. Sink Connector(数据导出)

```json
{
  "name": "elasticsearch-sink-connector",
  "config": {
    "connector.class": "io.confluent.connect.elasticsearch.ElasticsearchSinkConnector",
    "tasks.max": "3",
    "topics": "order-events,user-events",
    "connection.url": "http://elasticsearch:9200",
    "type.name": "_doc",
    "key.ignore": "false",
    "schema.ignore": "true",
    "behavior.on.null.values": "delete",
    "write.method": "upsert",
    "transforms": "extractKey,timestampRouter",
    "transforms.extractKey.type": "org.apache.kafka.connect.transforms.ExtractField$Key",
    "transforms.extractKey.field": "id",
    "transforms.timestampRouter.type": "org.apache.kafka.connect.transforms.TimestampRouter",
    "transforms.timestampRouter.topic.format": "${topic}-${timestamp}",
    "transforms.timestampRouter.timestamp.format": "yyyyMMdd"
  }
}
```

### 3. Connect管理API

```bash
# 查看已安装的Connector插件
curl http://connect:8083/connector-plugins | jq

# 创建Connector
curl -X POST http://connect:8083/connectors \
  -H "Content-Type: application/json" \
  -d @mysql-source-connector.json

# 查看Connector状态
curl http://connect:8083/connectors/mysql-source-connector/status | jq

# 暂停/恢复
curl -X PUT http://connect:8083/connectors/mysql-source-connector/pause
curl -X PUT http://connect:8083/connectors/mysql-source-connector/resume

# 重启Connector
curl -X POST http://connect:8083/connectors/mysql-source-connector/restart

# 重启单个Task
curl -X POST http://connect:8083/connectors/mysql-source-connector/tasks/0/restart

# 删除Connector
curl -X DELETE http://connect:8083/connectors/mysql-source-connector
```

## 性能优化

### 1. 分区数规划

```
分区数计算公式:
  目标吞吐量 / min(生产者单分区吞吐, 消费者单分区吞吐)

示例:
  目标: 100MB/s
  生产者单分区: 20MB/s
  消费者单分区: 10MB/s
  分区数 = 100 / 10 = 10 (取消费者瓶颈)

分区数建议:
  - 小规模(< 10MB/s): 6-12个分区
  - 中规模(10-100MB/s): 12-64个分区
  - 大规模(> 100MB/s): 64-256个分区
  - 分区数上限考虑: 每个分区占用Broker约1MB内存和一个文件句柄

注意: 分区数只能增加不能减少,规划时留有余量
```

### 2. 生产者性能调优

```properties
# 批处理(核心优化)
batch.size=131072              # 128KB(默认16KB太小)
linger.ms=20                   # 等待20ms凑批(默认0立即发送)

# 压缩
compression.type=lz4           # LZ4压缩(速度快,压缩率适中)

# 缓冲区
buffer.memory=134217728        # 128MB发送缓冲区
max.block.ms=60000             # 缓冲区满时最大阻塞时间

# 网络
send.buffer.bytes=131072       # TCP发送缓冲区
receive.buffer.bytes=65536     # TCP接收缓冲区

# 请求
max.request.size=10485760      # 单个请求最大10MB
request.timeout.ms=30000       # 请求超时30s
delivery.timeout.ms=120000     # 总投递超时120s
```

### 3. 消费者性能调优

```properties
# 拉取配置
fetch.min.bytes=1048576        # 最小拉取1MB(减少请求次数)
fetch.max.bytes=52428800       # 最大拉取50MB
fetch.max.wait.ms=500          # 最大等待500ms
max.partition.fetch.bytes=10485760  # 单分区最大拉取10MB

# 消费批次
max.poll.records=1000          # 单次poll最大记录数

# 并行度: 消费者数 = 分区数(最佳1:1映射)
```

### 4. Broker端性能优化

```properties
# 零拷贝(sendfile系统调用, 默认开启)
# Kafka使用零拷贝技术, 数据从磁盘直接传输到网卡, 不经过用户空间

# 页面缓存(Page Cache)
# Kafka依赖OS页面缓存而非JVM堆
# 建议: 预留25-50%物理内存给页面缓存
# JVM堆设置: 6-8GB即可(不要过大)
export KAFKA_HEAP_OPTS="-Xms6g -Xmx6g"

# 磁盘
log.dirs=/data1/kafka-logs,/data2/kafka-logs  # 多磁盘并行写入
log.flush.interval.messages=10000              # 每10000条刷盘(依赖页面缓存更佳)
log.flush.interval.ms=1000                     # 每秒刷盘

# 网络线程
num.network.threads=8          # 网络IO线程
num.io.threads=16              # 磁盘IO线程
num.replica.fetchers=4         # 副本拉取线程

# 日志段
log.segment.bytes=1073741824   # 1GB段文件
log.index.interval.bytes=4096  # 索引间隔
```

## 监控

### 1. 核心JMX指标

```
Broker指标:
┌──────────────────────────────────────────┬────────────────────────┐
│ 指标                                      │ 说明                   │
├──────────────────────────────────────────┼────────────────────────┤
│ kafka.server:type=BrokerTopicMetrics,    │ 消息入站速率(条/秒)  │
│   name=MessagesInPerSec                  │                        │
│ kafka.server:type=BrokerTopicMetrics,    │ 入站字节速率(B/秒)   │
│   name=BytesInPerSec                     │                        │
│ kafka.server:type=BrokerTopicMetrics,    │ 出站字节速率(B/秒)   │
│   name=BytesOutPerSec                    │                        │
│ kafka.server:type=ReplicaManager,        │ ISR扩缩次数(频繁则不健康) │
│   name=IsrShrinksPerSec                  │                        │
│ kafka.server:type=ReplicaManager,        │ 副本不足的分区数       │
│   name=UnderReplicatedPartitions         │                        │
│ kafka.controller:type=KafkaController,   │ 活跃Controller数(应=1) │
│   name=ActiveControllerCount             │                        │
│ kafka.server:type=ReplicaManager,        │ Leader分区数           │
│   name=LeaderCount                       │                        │
│ kafka.network:type=RequestMetrics,       │ 请求延迟(ms)         │
│   name=TotalTimeMs,request=Produce       │                        │
│ kafka.log:type=LogFlushStats,            │ 日志刷盘速率           │
│   name=LogFlushRateAndTimeMs             │                        │
└──────────────────────────────────────────┴────────────────────────┘

生产者指标:
  record-send-rate: 发送速率(条/秒)
  record-error-rate: 发送错误率
  request-latency-avg: 平均请求延迟
  batch-size-avg: 平均批次大小
  compression-rate-avg: 平均压缩率

消费者指标:
  records-consumed-rate: 消费速率(条/秒)
  records-lag-max: 最大消费延迟(条)
  fetch-latency-avg: 平均拉取延迟
  commit-latency-avg: 平均提交延迟
```

### 2. Lag监控

```bash
# 命令行查看Consumer Lag
kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group order-processing-group

# 输出示例:
# GROUP                  TOPIC           PARTITION  CURRENT-OFFSET  LOG-END-OFFSET  LAG
# order-processing-group order-events    0          1000            1050            50
# order-processing-group order-events    1          2000            2100            100
# order-processing-group order-events    2          3000            3010            10
```

**Burrow监控配置**:
```yaml
# burrow.toml
[general]
access-control-allow-origin = "*"

[zookeeper]
servers = ["zk1:2181", "zk2:2181", "zk3:2181"]

[cluster.production]
class-name = "kafka"
servers = ["kafka1:9092", "kafka2:9092", "kafka3:9092"]
topic-refresh = 60
offset-refresh = 30

[consumer.production]
class-name = "kafka"
cluster = "production"
servers = ["kafka1:9092", "kafka2:9092", "kafka3:9092"]
group-denylist = "^console-consumer-"
offset-refresh = 30

[notifier.slack]
class-name = "http"
url-open = "https://hooks.slack.com/services/xxx"
template-open = "burrow-alert.tmpl"
send-close = true
interval = 60
threshold = 2  # WARNING以上才告警

# Burrow评估状态:
# OK: Lag稳定或下降
# WARNING: Lag持续增长
# ERR: Lag增长且消费停滞
# STOP: 消费完全停止
```

### 3. Prometheus + Grafana监控

```yaml
# docker-compose.yml - JMX Exporter
services:
  kafka:
    environment:
      KAFKA_JMX_OPTS: >-
        -Dcom.sun.management.jmxremote
        -Dcom.sun.management.jmxremote.port=9999
        -Dcom.sun.management.jmxremote.authenticate=false
        -Dcom.sun.management.jmxremote.ssl=false
      EXTRA_ARGS: >-
        -javaagent:/opt/jmx-exporter/jmx_prometheus_javaagent.jar=7071:/opt/jmx-exporter/kafka-broker.yml

# prometheus.yml
scrape_configs:
  - job_name: 'kafka'
    static_configs:
      - targets: ['kafka1:7071', 'kafka2:7071', 'kafka3:7071']
```

## 安全

### 1. SASL认证

```properties
# Broker配置(SASL/SCRAM)
listeners=SASL_SSL://0.0.0.0:9093
advertised.listeners=SASL_SSL://kafka-broker:9093
security.inter.broker.protocol=SASL_SSL
sasl.mechanism.inter.broker.protocol=SCRAM-SHA-512
sasl.enabled.mechanisms=SCRAM-SHA-512

# 创建SCRAM用户
kafka-configs.sh --bootstrap-server localhost:9092 \
  --alter --add-config 'SCRAM-SHA-512=[password=secret123]' \
  --entity-type users --entity-name producer-user

kafka-configs.sh --bootstrap-server localhost:9092 \
  --alter --add-config 'SCRAM-SHA-512=[password=secret456]' \
  --entity-type users --entity-name consumer-user
```

### 2. SSL/TLS加密

```bash
# 生成CA证书
openssl req -new -x509 -keyout ca-key -out ca-cert -days 3650 \
  -subj "/CN=KafkaCA" -nodes

# 为每个Broker生成密钥库
keytool -keystore kafka-broker.keystore.jks -alias broker \
  -genkey -keyalg RSA -validity 3650 \
  -dname "CN=kafka-broker,OU=Kafka,O=Example,L=BJ,ST=BJ,C=CN" \
  -storepass changeit -keypass changeit

# 签名证书
keytool -keystore kafka-broker.keystore.jks -alias broker \
  -certreq -file cert-file -storepass changeit
openssl x509 -req -CA ca-cert -CAkey ca-key -in cert-file \
  -out cert-signed -days 3650 -CAcreateserial

# 导入CA和签名证书
keytool -keystore kafka-broker.keystore.jks -alias CARoot \
  -import -file ca-cert -storepass changeit -noprompt
keytool -keystore kafka-broker.keystore.jks -alias broker \
  -import -file cert-signed -storepass changeit

# 创建信任库
keytool -keystore kafka.truststore.jks -alias CARoot \
  -import -file ca-cert -storepass changeit -noprompt
```

```properties
# Broker SSL配置
ssl.keystore.location=/etc/kafka/ssl/kafka-broker.keystore.jks
ssl.keystore.password=changeit
ssl.key.password=changeit
ssl.truststore.location=/etc/kafka/ssl/kafka.truststore.jks
ssl.truststore.password=changeit
ssl.client.auth=required
ssl.endpoint.identification.algorithm=https
```

### 3. ACL授权

```bash
# 授权生产者写入
kafka-acls.sh --bootstrap-server localhost:9092 \
  --add --allow-principal User:producer-user \
  --operation Write --topic order-events

# 授权消费者读取
kafka-acls.sh --bootstrap-server localhost:9092 \
  --add --allow-principal User:consumer-user \
  --operation Read --topic order-events \
  --group order-processing-group

# 授权Consumer Group
kafka-acls.sh --bootstrap-server localhost:9092 \
  --add --allow-principal User:consumer-user \
  --operation Read --group order-processing-group

# 查看ACL
kafka-acls.sh --bootstrap-server localhost:9092 \
  --list --topic order-events

# 删除ACL
kafka-acls.sh --bootstrap-server localhost:9092 \
  --remove --allow-principal User:producer-user \
  --operation Write --topic order-events

# 通配符授权(前缀匹配)
kafka-acls.sh --bootstrap-server localhost:9092 \
  --add --allow-principal User:analytics-user \
  --operation Read --topic order- --resource-pattern-type prefixed
```

## 运维

### 1. 扩容与缩容

```bash
# 扩容: 添加新Broker后, 重新分配分区
# 1. 生成分配方案
kafka-reassign-partitions.sh --bootstrap-server localhost:9092 \
  --topics-to-move-json-file topics.json \
  --broker-list "0,1,2,3" \
  --generate

# topics.json
# {"topics": [{"topic": "order-events"}], "version": 1}

# 2. 执行迁移
kafka-reassign-partitions.sh --bootstrap-server localhost:9092 \
  --reassignment-json-file reassignment.json \
  --execute \
  --throttle 50000000  # 限速50MB/s避免影响业务

# 3. 验证迁移状态
kafka-reassign-partitions.sh --bootstrap-server localhost:9092 \
  --reassignment-json-file reassignment.json \
  --verify

# 缩容: 先迁移分区到其他Broker, 再下线
# 确保待下线Broker上无Leader分区
```

### 2. 数据保留策略

```properties
# 基于时间保留
log.retention.hours=168              # 保留7天(默认)
log.retention.minutes=10080          # 更精确的分钟级设置
log.retention.ms=604800000           # 最精确的毫秒级设置

# 基于大小保留
log.retention.bytes=107374182400     # 每分区保留100GB
# -1表示无大小限制

# 基于压缩(Compaction)
log.cleanup.policy=compact           # 只保留每个key的最新值
log.cleaner.min.compaction.lag.ms=86400000  # 最小压缩延迟24h
log.cleaner.delete.retention.ms=86400000    # 墓碑消息保留24h

# 混合策略(同时基于时间和压缩)
log.cleanup.policy=compact,delete

# Topic级别覆盖
kafka-configs.sh --bootstrap-server localhost:9092 \
  --entity-type topics --entity-name order-events \
  --alter --add-config retention.ms=2592000000  # 30天
```

### 3. 跨数据中心复制(MirrorMaker 2)

```properties
# mm2.properties (MirrorMaker 2配置)
clusters = source, target

source.bootstrap.servers = dc1-kafka1:9092,dc1-kafka2:9092
target.bootstrap.servers = dc2-kafka1:9092,dc2-kafka2:9092

# 复制配置
source->target.enabled = true
source->target.topics = order-events,user-events,payment-events
source->target.topics.exclude = .*-internal,__.*
source->target.groups = order-processing-group,analytics-group

# 同步配置
replication.factor = 3
offset-syncs.topic.replication.factor = 3
heartbeats.topic.replication.factor = 3
checkpoints.topic.replication.factor = 3

# 性能配置
tasks.max = 4
producer.buffer.memory = 134217728
consumer.fetch.max.bytes = 52428800

# 偏移量同步(故障切换时保持消费位置)
sync.group.offsets.enabled = true
sync.group.offsets.interval.seconds = 10
emit.checkpoints.enabled = true
emit.checkpoints.interval.seconds = 30
```

```bash
# 启动MirrorMaker 2
connect-mirror-maker.sh mm2.properties

# 灾难恢复切换:
# 1. 停止源集群的生产者
# 2. 等待MirrorMaker 2同步完成(检查checkpoint lag)
# 3. 将消费者指向目标集群
# 4. 使用synced offset恢复消费位置
# 5. 启动目标集群的生产者
```

### 4. Topic迁移

```bash
# 分区Leader重新选举(优先副本选举)
kafka-leader-election.sh --bootstrap-server localhost:9092 \
  --election-type preferred \
  --all-topic-partitions

# 增加Topic副本因子
# 1. 生成增加副本的reassignment JSON
cat > increase-rf.json << 'EOF'
{
  "version": 1,
  "partitions": [
    {"topic": "order-events", "partition": 0, "replicas": [0, 1, 2]},
    {"topic": "order-events", "partition": 1, "replicas": [1, 2, 0]},
    {"topic": "order-events", "partition": 2, "replicas": [2, 0, 1]}
  ]
}
EOF

# 2. 执行
kafka-reassign-partitions.sh --bootstrap-server localhost:9092 \
  --reassignment-json-file increase-rf.json \
  --execute --throttle 50000000
```

## 常见陷阱

### 1. 消费者Lag暴涨

```
症状: Consumer Lag持续增长, 消费速度跟不上生产速度
原因:
  - 消费者处理逻辑耗时过长(数据库慢查询/外部调用超时)
  - 消费者数量不足(少于分区数)
  - GC暂停导致消费停滞
  - max.poll.records过大, 处理超过max.poll.interval.ms

排查步骤:
  1. kafka-consumer-groups.sh --describe 查看各分区Lag
  2. 检查消费者日志是否有处理异常/超时
  3. 监控消费者JVM GC情况
  4. 检查下游依赖(DB/缓存/HTTP)的响应时间

解决方案:
  ✅ 增加消费者实例(不超过分区数)
  ✅ 减小max.poll.records, 增大max.poll.interval.ms
  ✅ 异步处理: poll后放入本地队列, 多线程处理
  ✅ 优化下游调用(批量写DB/连接池/缓存)
  ❌ 盲目增加分区数(需要同时增加消费者才有效)
```

### 2. Rebalance风暴

```
症状: Consumer频繁触发Rebalance, 消费几乎停滞
原因:
  - session.timeout.ms过短, 心跳超时触发Rebalance
  - max.poll.interval.ms过短, 处理慢导致被踢出Group
  - Consumer频繁启停(K8s Pod频繁重启)
  - GC暂停超过session.timeout.ms

解决方案:
  ✅ 使用CooperativeStickyAssignor(增量Rebalance)
  ✅ 增大session.timeout.ms(30-60s)
  ✅ 增大max.poll.interval.ms(5-10min)
  ✅ 减小max.poll.records, 确保处理时间可控
  ✅ 设置group.instance.id启用静态成员(避免重启触发Rebalance)

# 静态成员配置(Kafka 2.3+)
group.instance.id=consumer-host-1  # 每个实例唯一
session.timeout.ms=60000           # 可以设更长(静态成员离开不立即Rebalance)
```

### 3. 分区过多

```
症状: Broker内存占用高, Controller切换慢, 端到端延迟增加
原因: 分区数远超实际吞吐需求

影响:
  - 每个分区占用约1MB Broker内存(元数据+索引)
  - Controller故障恢复时间与分区数成正比
  - 文件句柄数增加(每分区2-3个文件)
  - 生产者内存增加(每分区一个RecordBatch缓冲)

建议:
  ✅ 单集群分区总数 < 200,000(KRaft模式可更多)
  ✅ 单Broker分区数 < 4,000
  ✅ 根据实际吞吐需求规划, 预留20-30%余量
  ❌ 不要盲目设置大量分区(分区数只增不减)
```

### 4. 消息丢失

```
场景1: 生产端丢失
  原因: acks=0或acks=1且Leader宕机
  解决: acks=all + min.insync.replicas=2 + retries=MAX

场景2: Broker端丢失
  原因: unclean.leader.election.enable=true, 非ISR副本当选Leader
  解决: unclean.leader.election.enable=false

场景3: 消费端丢失
  原因: 自动提交offset后处理失败
  解决: 手动提交, 先处理后提交(at-least-once)

生产环境防丢失配置组合:
  # 生产者
  acks=all
  retries=2147483647
  enable.idempotence=true
  max.in.flight.requests.per.connection=5

  # Broker
  min.insync.replicas=2
  unclean.leader.election.enable=false
  default.replication.factor=3

  # 消费者
  enable.auto.commit=false
  # 手动提交: 先处理,后提交
```

### 5. 重复消费

```
场景: 消费者处理完消息但提交offset前宕机, 重启后重复消费
原因: at-least-once语义下的正常行为

解决方案:
  1. 幂等消费(推荐)
     - 使用消息中的唯一ID(orderId)做去重
     - 数据库INSERT时使用UPSERT/ON CONFLICT
     - Redis SETNX记录已处理消息ID

  2. Exactly-Once语义
     - Kafka事务(消费-转换-生产场景)
     - 将offset和业务数据写入同一事务(如同一数据库)

  3. 消费去重表
     CREATE TABLE consumed_offsets (
       consumer_group VARCHAR(255),
       topic VARCHAR(255),
       partition_id INT,
       offset_val BIGINT,
       processed_at TIMESTAMP,
       PRIMARY KEY (consumer_group, topic, partition_id)
     );
```

## 学习路线

### 入门级 (1-2周)
1. 理解Kafka核心概念(Topic/Partition/Consumer Group/Offset)
2. 搭建单节点Kafka(Docker)
3. 使用命令行工具收发消息
4. 编写简单的生产者和消费者

### 中级 (2-4周)
1. 集群搭建与副本管理
2. Schema管理与序列化
3. Consumer Group与Rebalance机制
4. Kafka Connect数据集成
5. 基础监控与告警

### 高级 (1-2月)
1. Kafka Streams流处理
2. 事务与Exactly-Once语义
3. 性能调优与容量规划
4. 安全配置(SASL/SSL/ACL)
5. 跨数据中心复制(MirrorMaker 2)

### 专家级 (持续)
1. KRaft架构与迁移
2. 大规模集群运维(百万分区)
3. CDC管道设计(Debezium)
4. 故障演练与灾难恢复
5. 自定义Interceptor/Serializer/Partitioner

## 参考资料

### 官方文档
- [Kafka官方文档](https://kafka.apache.org/documentation/)
- [Confluent文档](https://docs.confluent.io/)
- [KRaft文档](https://kafka.apache.org/documentation/#kraft)

### 工具
- [Kafka UI](https://github.com/provectus/kafka-ui) - Web管理界面
- [Burrow](https://github.com/linkedin/Burrow) - Consumer Lag监控
- [AKHQ](https://github.com/tchiotludo/akhq) - Kafka管理平台
- [Debezium](https://debezium.io/) - CDC连接器

### 书籍
- 《Kafka权威指南》(第2版) - O'Reilly
- 《Kafka Streams实战》 - Manning

---

## Agent Checklist

- [ ] Topic设计: 命名规范(domain.entity.event), 分区数合理, 副本因子>=3
- [ ] 生产者: acks=all, 开启幂等性, 批处理+压缩配置
- [ ] 消费者: 手动提交offset, CooperativeStickyAssignor, 幂等消费
- [ ] Schema: 使用Schema Registry, 设置兼容性策略, Avro/Protobuf序列化
- [ ] 集群: min.insync.replicas=2, unclean.leader.election.enable=false
- [ ] 安全: SASL认证 + SSL/TLS加密 + ACL授权, 最小权限原则
- [ ] 监控: JMX指标接入Prometheus, Consumer Lag告警, Broker健康检查
- [ ] 性能: 分区数与消费者数匹配, 零拷贝+页面缓存, 合理的JVM堆(6-8GB)
- [ ] 保留策略: 按业务需求配置retention, 重要Topic使用compact策略
- [ ] 灾备: MirrorMaker 2跨DC复制, 定期故障演练, 切换SOP文档化
- [ ] 防丢失: acks=all + min.insync.replicas=2 + 手动提交 + 幂等消费
- [ ] 防重复: 业务幂等 + 去重表/Redis去重 + Exactly-Once事务(如需要)
- [ ] KRaft迁移: 评估是否已满足Kafka 3.3+要求, 规划ZK→KRaft迁移路径

---

**知识ID**: `kafka-complete`
**领域**: data-engineering
**类型**: standards
**难度**: intermediate
**质量分**: 95
**维护者**: data-team@superdev.com
**最后更新**: 2026-03-28
