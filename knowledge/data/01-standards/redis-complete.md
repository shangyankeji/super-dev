---
id: redis-complete
title: Redis 数据领域完整指南
domain: data
category: 01-standards
difficulty: intermediate
tags: [complete, data, redis, 分布式锁, 持久化, 核心数据结构, 概述, 消息队列]
quality_score: 70
last_updated: 2026-06-15
---
# Redis 数据领域完整指南

> 文档版本: v1.0 | 最后更新: 2026-03-28

## 概述

Redis (Remote Dictionary Server) 是一款开源的内存键值数据库，以极低的读写延迟（微秒级）著称。它不仅是缓存层的首选方案，还广泛应用于会话管理、排行榜、实时分析、分布式锁、消息队列等场景。Redis 7.x 支持多线程 I/O、ACL 2.0、Function API 和 Sharded Pub/Sub，在数据领域承担着从缓存加速到流式计算的关键角色。

### 核心特性

- **内存优先**: 所有数据驻留内存，读写延迟 < 1ms
- **丰富数据结构**: String / Hash / List / Set / Sorted Set / Stream / HyperLogLog / Bitmap / Geospatial
- **持久化可选**: RDB 快照 + AOF 日志 + 混合持久化
- **高可用**: 主从复制 / Sentinel 哨兵 / Cluster 集群
- **原子操作**: 单线程命令执行模型，天然避免竞态
- **可扩展**: Lua 脚本、Module API、Function API

### 典型数据领域使用场景

| 场景 | 数据结构 | 说明 |
|------|----------|------|
| ETL 中间缓存 | String / Hash | 加速重复查询，减轻源库压力 |
| 实时指标聚合 | Sorted Set / HyperLogLog | 排行榜、UV 去重统计 |
| 流式数据管道 | Stream | 替代轻量级 Kafka 场景 |
| 数据血缘锁 | String (SET NX EX) | 防止并发血缘刷新冲突 |
| 特征存储 | Hash | ML 在线特征毫秒级读取 |
| 数据目录缓存 | String + JSON | 元数据目录热点加速 |

---

## 核心数据结构

### 1. String（字符串）

最基础的数据类型，可存储字符串、整数、浮点数或二进制数据（最大 512MB）。

```bash
# 基本读写
SET user:1001 "Alice"
GET user:1001

# 带过期时间
SET session:abc123 "token_data" EX 3600       # 秒级过期
SET session:abc123 "token_data" PX 3600000    # 毫秒级过期
SETEX cache:product:99 300 '{"name":"Widget"}'

# 条件写入
SET lock:order:5001 "owner_a" NX EX 30   # 仅当 key 不存在时写入
SET counter:daily 100 XX                  # 仅当 key 存在时写入

# 原子自增/自减
SET counter:pageview 0
INCR counter:pageview           # -> 1
INCRBY counter:pageview 10     # -> 11
DECR counter:pageview           # -> 10
INCRBYFLOAT price:item:1 2.5   # 浮点自增

# 批量操作
MSET k1 "v1" k2 "v2" k3 "v3"
MGET k1 k2 k3

# 位操作（适合标记类场景）
SETBIT user:1001:login 0 1     # 第0天登录
GETBIT user:1001:login 0       # -> 1
BITCOUNT user:1001:login        # 统计登录天数

# 获取子串
GETRANGE user:1001 0 2         # -> "Ali"
STRLEN user:1001               # -> 5
```

**Python redis-py 示例:**

```python
import redis

r = redis.Redis(host="localhost", port=6379, db=0, decode_responses=True)

# 基本操作
r.set("user:1001", "Alice", ex=3600)
value = r.get("user:1001")  # "Alice"

# 原子自增
r.set("counter:pv", 0)
r.incr("counter:pv")        # 1
r.incrby("counter:pv", 10)  # 11

# 批量操作
r.mset({"k1": "v1", "k2": "v2", "k3": "v3"})
values = r.mget("k1", "k2", "k3")  # ["v1", "v2", "v3"]

# 条件写入（分布式锁基础）
acquired = r.set("lock:order:5001", "owner_a", nx=True, ex=30)
```

### 2. Hash（哈希）

适合存储对象，每个 key 下可以有多个 field-value 对。

```bash
# 设置字段
HSET user:1001 name "Alice" age 30 email "alice@example.com" role "analyst"

# 读取
HGET user:1001 name             # -> "Alice"
HMGET user:1001 name age        # -> ["Alice", "30"]
HGETALL user:1001               # 返回所有字段

# 字段自增
HINCRBY user:1001 age 1         # -> 31
HINCRBYFLOAT user:1001 score 0.5

# 字段操作
HDEL user:1001 role
HEXISTS user:1001 name          # -> 1
HLEN user:1001                  # 字段数量
HKEYS user:1001                 # 所有字段名
HVALS user:1001                 # 所有字段值

# 扫描大 Hash（生产环境推荐）
HSCAN user:1001 0 MATCH "na*" COUNT 100
```

**Python redis-py 示例:**

```python
# 对象存储
r.hset("user:1001", mapping={
    "name": "Alice",
    "age": 30,
    "email": "alice@example.com",
})

user = r.hgetall("user:1001")
# {"name": "Alice", "age": "30", "email": "alice@example.com"}

r.hincrby("user:1001", "age", 1)  # 31
exists = r.hexists("user:1001", "name")  # True

# 特征存储示例
r.hset("feature:user:1001", mapping={
    "click_rate": "0.032",
    "avg_session": "245",
    "last_active": "2026-03-28",
})
features = r.hmget("feature:user:1001", "click_rate", "avg_session")
```

### 3. List（列表）

双向链表，支持头/尾插入和弹出，适合队列和栈场景。

```bash
# 左/右推入
LPUSH queue:tasks "task1" "task2" "task3"
RPUSH queue:tasks "task4"

# 弹出
LPOP queue:tasks            # -> "task3"（LIFO 栈）
RPOP queue:tasks            # -> "task4"（FIFO 队列）

# 阻塞弹出（消费者模式）
BLPOP queue:tasks 30        # 阻塞等待最多30秒
BRPOP queue:tasks 30

# 范围查询
LRANGE queue:tasks 0 -1     # 所有元素
LRANGE queue:tasks 0 9      # 前10个

# 长度和索引
LLEN queue:tasks
LINDEX queue:tasks 0        # 第一个元素

# 修剪（保留最近N条）
LTRIM queue:tasks 0 99      # 只保留前100个
```

**Python redis-py 示例:**

```python
# 简单任务队列
r.lpush("queue:tasks", "task1", "task2", "task3")

# 消费者
task = r.brpop("queue:tasks", timeout=30)
# ("queue:tasks", "task1")

# 保留最近100条日志
r.lpush("log:app", "2026-03-28 10:00 INFO started")
r.ltrim("log:app", 0, 99)
```

### 4. Set（集合）

无序不重复集合，支持集合运算。

```bash
# 添加成员
SADD tags:article:1 "python" "redis" "database"
SADD tags:article:2 "python" "django" "web"

# 查询
SMEMBERS tags:article:1      # 所有成员
SISMEMBER tags:article:1 "redis"  # -> 1
SCARD tags:article:1         # 成员数量

# 集合运算
SINTER tags:article:1 tags:article:2      # 交集 -> {"python"}
SUNION tags:article:1 tags:article:2      # 并集
SDIFF tags:article:1 tags:article:2       # 差集

# 随机成员
SRANDMEMBER tags:article:1 2    # 随机取2个
SPOP tags:article:1             # 随机弹出1个

# 删除
SREM tags:article:1 "database"
```

**Python redis-py 示例:**

```python
# 标签系统
r.sadd("tags:article:1", "python", "redis", "database")
r.sadd("tags:article:2", "python", "django", "web")

common_tags = r.sinter("tags:article:1", "tags:article:2")
# {"python"}

all_tags = r.sunion("tags:article:1", "tags:article:2")
# {"python", "redis", "database", "django", "web"}

is_member = r.sismember("tags:article:1", "redis")  # True
```

### 5. Sorted Set（有序集合 / ZSet）

带分值的有序集合，按 score 排序，适合排行榜和范围查询。

```bash
# 添加（score + member）
ZADD leaderboard 100 "alice" 95 "bob" 88 "charlie" 120 "diana"

# 排名查询（升序，0-based）
ZRANK leaderboard "alice"        # -> 2
ZREVRANK leaderboard "alice"     # -> 1（降序排名）

# 分值查询
ZSCORE leaderboard "alice"       # -> 100

# 范围查询
ZRANGE leaderboard 0 2                    # 升序前3
ZREVRANGE leaderboard 0 2 WITHSCORES     # 降序前3（带分值）
ZRANGEBYSCORE leaderboard 90 110         # 分值 90-110
ZRANGEBYSCORE leaderboard -inf +inf LIMIT 0 10  # 分页

# 分值自增
ZINCRBY leaderboard 5 "alice"    # -> 105

# 集合运算
ZUNIONSTORE combined 2 leaderboard:week1 leaderboard:week2 WEIGHTS 1 2
ZINTERSTORE overlap 2 set1 set2

# 删除
ZREM leaderboard "charlie"
ZREMRANGEBYRANK leaderboard 0 0          # 删除最低分
ZREMRANGEBYSCORE leaderboard 0 60        # 删除低于60分的

# 数量统计
ZCARD leaderboard
ZCOUNT leaderboard 90 110               # 分值在 90-110 之间的数量
```

**Python redis-py 示例:**

```python
# 实时排行榜
r.zadd("leaderboard", {"alice": 100, "bob": 95, "charlie": 88, "diana": 120})

# Top 3（降序）
top3 = r.zrevrange("leaderboard", 0, 2, withscores=True)
# [("diana", 120.0), ("alice", 100.0), ("bob", 95.0)]

# 用户排名
rank = r.zrevrank("leaderboard", "alice")  # 1

# 分值自增
r.zincrby("leaderboard", 5, "alice")  # 105.0

# 分页查询
page = r.zrevrangebyscore("leaderboard", "+inf", "-inf", start=0, num=10, withscores=True)
```

### 6. Stream（流）

Redis 5.0+ 引入的日志型数据结构，支持消费者组，适合事件流和消息队列。

```bash
# 写入消息
XADD events:orders * action "created" order_id "5001" amount "99.50"
XADD events:orders * action "paid" order_id "5001" payment "stripe"

# 读取
XLEN events:orders
XRANGE events:orders - +               # 全部
XRANGE events:orders - + COUNT 10      # 前10条
XREVRANGE events:orders + - COUNT 5    # 最近5条

# 消费者组
XGROUP CREATE events:orders cg_analytics $ MKSTREAM
XREADGROUP GROUP cg_analytics consumer1 COUNT 10 BLOCK 5000 STREAMS events:orders >

# 确认消费
XACK events:orders cg_analytics 1679012345678-0

# 查看待处理消息
XPENDING events:orders cg_analytics - + 10

# 裁剪（保留最近1000条）
XTRIM events:orders MAXLEN ~ 1000
```

**Python redis-py 示例:**

```python
# 生产者
msg_id = r.xadd("events:orders", {
    "action": "created",
    "order_id": "5001",
    "amount": "99.50",
})

# 创建消费者组
try:
    r.xgroup_create("events:orders", "cg_analytics", id="$", mkstream=True)
except redis.exceptions.ResponseError:
    pass  # 组已存在

# 消费者
messages = r.xreadgroup(
    groupname="cg_analytics",
    consumername="consumer1",
    streams={"events:orders": ">"},
    count=10,
    block=5000,
)

for stream, msgs in messages:
    for msg_id, fields in msgs:
        print(f"Processing {msg_id}: {fields}")
        r.xack("events:orders", "cg_analytics", msg_id)
```

### 7. HyperLogLog

概率数据结构，用极小内存（12KB）估算集合基数（误差 < 0.81%），适合 UV 统计。

```bash
# 添加元素
PFADD uv:2026-03-28 "user1" "user2" "user3" "user1"

# 估算基数
PFCOUNT uv:2026-03-28         # -> 3

# 合并多天
PFMERGE uv:week uv:2026-03-28 uv:2026-03-27 uv:2026-03-26
PFCOUNT uv:week
```

**Python redis-py 示例:**

```python
# 日活统计
r.pfadd("uv:2026-03-28", "user1", "user2", "user3", "user1")
daily_uv = r.pfcount("uv:2026-03-28")  # 3

# 周活统计
keys = [f"uv:2026-03-{d}" for d in range(22, 29)]
r.pfmerge("uv:week", *keys)
weekly_uv = r.pfcount("uv:week")
```

### 8. Bitmap（位图）

基于 String 的位操作，极致节省空间，适合布尔状态标记。

```bash
# 用户签到（day_offset 从年初算）
SETBIT sign:user:1001:2026 86 1    # 第87天签到
GETBIT sign:user:1001:2026 86      # -> 1

# 统计签到天数
BITCOUNT sign:user:1001:2026

# 位运算（统计连续签到）
BITOP AND result sign:user:1001:2026 sign:user:1002:2026
BITPOS sign:user:1001:2026 0       # 第一个未签到的天
```

**Python redis-py 示例:**

```python
# 用户签到系统
day_of_year = 86  # 3月28日大约是第87天
r.setbit("sign:user:1001:2026", day_of_year, 1)

# 统计本年签到天数
total_sign = r.bitcount("sign:user:1001:2026")

# 检查某天是否签到
signed = r.getbit("sign:user:1001:2026", day_of_year)  # 1
```

---

## 持久化

### RDB（快照）

将内存数据以二进制快照形式写入磁盘。

```bash
# redis.conf 配置
save 900 1        # 900秒内至少1次写入则触发
save 300 10       # 300秒内至少10次写入
save 60 10000     # 60秒内至少10000次写入

dbfilename dump.rdb
dir /var/lib/redis/

# 手动触发
BGSAVE              # 后台异步保存（推荐）
SAVE                # 同步保存（阻塞，生产禁用）
LASTSAVE            # 最近一次成功保存时间
```

**优点**: 文件紧凑，恢复速度快，适合灾备。
**缺点**: 可能丢失最近一次快照后的数据。

### AOF（追加日志）

将每条写命令追加到日志文件。

```bash
# redis.conf 配置
appendonly yes
appendfilename "appendonly.aof"
appenddirname "appendonlydir"

# 刷盘策略
appendfsync always     # 每条命令刷盘（最安全，最慢）
appendfsync everysec   # 每秒刷盘（推荐，最多丢1秒）
appendfsync no         # OS 决定（最快，最不安全）

# AOF 重写（压缩日志）
BGREWRITEAOF

# 自动重写阈值
auto-aof-rewrite-percentage 100
auto-aof-rewrite-min-size 64mb
```

**优点**: 数据丢失少（everysec 最多丢1秒），日志可读。
**缺点**: 文件比 RDB 大，恢复速度较慢。

### 混合持久化（Redis 4.0+，推荐）

RDB + AOF 结合：重写时先写 RDB 格式，之后增量命令用 AOF 格式追加。

```bash
# redis.conf
aof-use-rdb-preamble yes    # 默认开启
appendonly yes
```

**优点**: 兼顾快速恢复（RDB 部分）和低数据丢失（AOF 增量）。

**Python 验证持久化状态:**

```python
info = r.info("persistence")
print(f"RDB 上次保存: {info['rdb_last_save_time']}")
print(f"RDB 状态: {info['rdb_last_bgsave_status']}")
print(f"AOF 开启: {info['aof_enabled']}")
print(f"AOF 重写进行中: {info['aof_rewrite_in_progress']}")
```

---

## 高可用

### 主从复制

```bash
# 从节点配置
# redis.conf
replicaof 192.168.1.100 6379
masterauth "your_master_password"

# 动态设置
REPLICAOF 192.168.1.100 6379
REPLICAOF NO ONE               # 提升为主节点

# 查看复制状态
INFO replication
```

**关键参数:**

```bash
repl-backlog-size 256mb          # 复制积压缓冲区
repl-diskless-sync yes           # 无盘复制（网络快于磁盘时开启）
min-replicas-to-write 1          # 至少1个从节点确认写入
min-replicas-max-lag 10          # 从节点最大延迟10秒
```

### Sentinel（哨兵）

自动故障转移，适合中小规模部署。

```bash
# sentinel.conf
sentinel monitor mymaster 192.168.1.100 6379 2      # 2个哨兵同意才切换
sentinel down-after-milliseconds mymaster 5000       # 5秒无响应判定下线
sentinel failover-timeout mymaster 60000             # 故障转移超时60秒
sentinel parallel-syncs mymaster 1                   # 切换后同时同步的从节点数

sentinel auth-pass mymaster "your_password"
```

**Python redis-py 连接 Sentinel:**

```python
from redis.sentinel import Sentinel

sentinel = Sentinel(
    [("sentinel1", 26379), ("sentinel2", 26379), ("sentinel3", 26379)],
    socket_timeout=0.5,
)

# 获取主节点连接
master = sentinel.master_for("mymaster", socket_timeout=0.5, password="your_password")
master.set("key", "value")

# 获取从节点连接（读操作）
slave = sentinel.slave_for("mymaster", socket_timeout=0.5, password="your_password")
value = slave.get("key")
```

### Cluster（集群）

数据分片（16384 个 slot），适合大规模高可用部署。

```bash
# 创建集群（至少6个节点：3主3从）
redis-cli --cluster create \
    192.168.1.101:6379 192.168.1.102:6379 192.168.1.103:6379 \
    192.168.1.104:6379 192.168.1.105:6379 192.168.1.106:6379 \
    --cluster-replicas 1

# 查看集群信息
redis-cli -c -h 192.168.1.101 -p 6379
CLUSTER INFO
CLUSTER NODES
CLUSTER SLOTS

# 添加节点
redis-cli --cluster add-node 192.168.1.107:6379 192.168.1.101:6379

# 重新分片
redis-cli --cluster reshard 192.168.1.101:6379

# 检查集群健康
redis-cli --cluster check 192.168.1.101:6379
```

**Python redis-py 集群连接:**

```python
from redis.cluster import RedisCluster

rc = RedisCluster(
    host="192.168.1.101",
    port=6379,
    password="your_password",
    decode_responses=True,
)

rc.set("key", "value")
value = rc.get("key")

# 注意：集群模式下多 key 操作需要 Hash Tag
rc.mset({"{user:1001}.name": "Alice", "{user:1001}.age": "30"})
```

---

## 缓存策略

### Cache-Aside（旁路缓存，最常用）

```python
def get_user(user_id: int) -> dict:
    cache_key = f"user:{user_id}"

    # 1. 先查缓存
    cached = r.get(cache_key)
    if cached:
        return json.loads(cached)

    # 2. 缓存未命中，查数据库
    user = db.query("SELECT * FROM users WHERE id = %s", (user_id,))
    if user is None:
        # 防止缓存穿透：缓存空值（短TTL）
        r.set(cache_key, json.dumps(None), ex=60)
        return None

    # 3. 写入缓存
    r.set(cache_key, json.dumps(user), ex=3600)
    return user

def update_user(user_id: int, data: dict):
    # 1. 先更新数据库
    db.execute("UPDATE users SET ... WHERE id = %s", (user_id,))
    # 2. 再删除缓存（而非更新缓存，避免并发不一致）
    r.delete(f"user:{user_id}")
```

### Write-Through（写穿透）

```python
def write_through_set(key: str, value: str, ttl: int = 3600):
    """写入时同时更新缓存和数据库"""
    # 1. 更新数据库
    db.execute("INSERT INTO kv_store (key, value) VALUES (%s, %s) "
               "ON CONFLICT (key) DO UPDATE SET value = %s", (key, value, value))
    # 2. 更新缓存
    r.set(key, value, ex=ttl)
```

### Write-Behind（写回，异步写入数据库）

```python
def write_behind_set(key: str, value: str):
    """先写缓存，异步写数据库"""
    r.set(key, value, ex=3600)
    # 将写操作放入队列，异步批量写入数据库
    r.lpush("queue:db_writes", json.dumps({"key": key, "value": value}))

# 异步消费者（独立进程）
def db_write_consumer():
    while True:
        item = r.brpop("queue:db_writes", timeout=5)
        if item:
            data = json.loads(item[1])
            db.execute("UPSERT ...", (data["key"], data["value"]))
```

### TTL 策略

```python
# 分级 TTL 策略
TTL_CONFIG = {
    "hot_data": 300,          # 热点数据：5分钟
    "warm_data": 3600,        # 温数据：1小时
    "cold_data": 86400,       # 冷数据：1天
    "static_data": 604800,    # 静态数据：7天
}

# 带随机偏移的 TTL（防止缓存雪崩）
import random

def set_with_jitter(key: str, value: str, base_ttl: int):
    jitter = random.randint(0, int(base_ttl * 0.1))  # 10% 随机偏移
    r.set(key, value, ex=base_ttl + jitter)
```

### 缓存穿透

查询不存在的数据，请求直接打到数据库。

```python
# 方案1：缓存空值
def get_with_null_cache(key: str):
    cached = r.get(key)
    if cached == "NULL":
        return None
    if cached:
        return json.loads(cached)

    result = db.query(key)
    if result is None:
        r.set(key, "NULL", ex=60)  # 短 TTL 缓存空值
        return None

    r.set(key, json.dumps(result), ex=3600)
    return result

# 方案2：布隆过滤器（Redis Stack / RedisBloom）
# BF.RESERVE valid_ids 0.001 1000000
# BF.ADD valid_ids "id_1001"
# BF.EXISTS valid_ids "id_9999"  -> 0 (不存在，直接拒绝)
```

### 缓存雪崩

大量 key 同时过期，请求同时打到数据库。

```python
# 方案1：TTL 随机化（见上方 set_with_jitter）

# 方案2：互斥锁 + 异步刷新
def get_with_mutex(key: str):
    cached = r.get(key)
    if cached:
        return json.loads(cached)

    lock_key = f"lock:{key}"
    if r.set(lock_key, "1", nx=True, ex=10):
        try:
            result = db.query(key)
            r.set(key, json.dumps(result), ex=3600)
            return result
        finally:
            r.delete(lock_key)
    else:
        # 其他线程正在刷新，等待后重试
        import time
        time.sleep(0.1)
        return get_with_mutex(key)
```

### 缓存击穿

单个热点 key 过期，大量并发请求同时打到数据库。

```python
# 方案1：互斥锁（同上 get_with_mutex）

# 方案2：逻辑过期（永不真正过期）
import time

def set_with_logical_expiry(key: str, value: str, ttl: int):
    data = {"value": value, "expire_at": time.time() + ttl}
    r.set(key, json.dumps(data))  # 不设 Redis TTL

def get_with_logical_expiry(key: str):
    cached = r.get(key)
    if not cached:
        return None

    data = json.loads(cached)
    if time.time() < data["expire_at"]:
        return data["value"]

    # 已逻辑过期，异步刷新
    lock_key = f"lock:refresh:{key}"
    if r.set(lock_key, "1", nx=True, ex=10):
        # 触发异步刷新（线程池/消息队列）
        import threading
        threading.Thread(target=refresh_cache, args=(key,)).start()

    # 返回过期数据（而非 None）
    return data["value"]
```

---

## 分布式锁

### SET NX EX（单节点锁）

```python
import uuid
import time

def acquire_lock(lock_name: str, timeout: int = 10) -> str | None:
    """获取分布式锁，返回 token 用于安全释放"""
    token = str(uuid.uuid4())
    acquired = r.set(f"lock:{lock_name}", token, nx=True, ex=timeout)
    return token if acquired else None

# Lua 脚本安全释放（原子操作，防止误删他人的锁）
RELEASE_LOCK_SCRIPT = """
if redis.call('get', KEYS[1]) == ARGV[1] then
    return redis.call('del', KEYS[1])
else
    return 0
end
"""

def release_lock(lock_name: str, token: str) -> bool:
    result = r.eval(RELEASE_LOCK_SCRIPT, 1, f"lock:{lock_name}", token)
    return result == 1

# 使用示例
token = acquire_lock("order:5001", timeout=30)
if token:
    try:
        # 业务逻辑
        process_order(5001)
    finally:
        release_lock("order:5001", token)
```

### Redlock（多节点锁，Martin Kleppmann 有争议但广泛使用）

```python
# pip install redis
# 需要至少5个独立 Redis 实例（非集群节点）

import time
import uuid

REDIS_INSTANCES = [
    redis.Redis(host=f"redis{i}", port=6379) for i in range(1, 6)
]
QUORUM = len(REDIS_INSTANCES) // 2 + 1  # 3

def redlock_acquire(resource: str, ttl_ms: int = 10000) -> str | None:
    token = str(uuid.uuid4())
    start_time = time.monotonic()
    acquired_count = 0

    for instance in REDIS_INSTANCES:
        try:
            if instance.set(f"lock:{resource}", token, nx=True, px=ttl_ms):
                acquired_count += 1
        except redis.exceptions.ConnectionError:
            continue

    elapsed_ms = (time.monotonic() - start_time) * 1000
    # 检查：多数节点获取成功 且 耗时未超过 TTL
    if acquired_count >= QUORUM and elapsed_ms < ttl_ms:
        return token

    # 获取失败，释放所有已获取的锁
    for instance in REDIS_INSTANCES:
        try:
            instance.eval(RELEASE_LOCK_SCRIPT, 1, f"lock:{resource}", token)
        except redis.exceptions.ConnectionError:
            continue
    return None

def redlock_release(resource: str, token: str):
    for instance in REDIS_INSTANCES:
        try:
            instance.eval(RELEASE_LOCK_SCRIPT, 1, f"lock:{resource}", token)
        except redis.exceptions.ConnectionError:
            continue
```

> **注意**: 生产环境建议使用 `python-redis-lock` 或 `pottery` 库的 Redlock 实现，而非自行编写。

---

## 消息队列

### Stream（推荐，Redis 5.0+）

见上方 Stream 数据结构部分。Stream 提供：
- 消费者组（Consumer Group）
- 消息确认（ACK）
- 待处理消息列表（PEL）
- 消息回溯与重放

### Pub/Sub（发布/订阅）

适合实时通知，不保证消息持久化。

```bash
# 订阅（消费者终端）
SUBSCRIBE channel:notifications
PSUBSCRIBE channel:*           # 模式订阅

# 发布（生产者终端）
PUBLISH channel:notifications "order:5001 created"

# Redis 7.0+ Sharded Pub/Sub（集群环境推荐）
SSUBSCRIBE channel:notifications
SPUBLISH channel:notifications "order:5001 created"
```

**Python redis-py 示例:**

```python
# 发布者
r.publish("channel:notifications", "order:5001 created")

# 订阅者（独立线程/进程）
pubsub = r.pubsub()
pubsub.subscribe("channel:notifications")

for message in pubsub.listen():
    if message["type"] == "message":
        print(f"Received: {message['data']}")
```

**Stream vs Pub/Sub 对比:**

| 特性 | Stream | Pub/Sub |
|------|--------|---------|
| 消息持久化 | 是 | 否 |
| 消费者组 | 是 | 否 |
| 消息确认 | 是 | 否 |
| 历史回溯 | 是 | 否 |
| 延迟 | 较低 | 极低 |
| 适用场景 | 可靠消息队列 | 实时通知广播 |

---

## Lua 脚本

Redis 保证 Lua 脚本的原子执行，适合需要多步原子操作的场景。

```bash
# 内联执行
EVAL "return redis.call('get', KEYS[1])" 1 mykey

# 条件自增（库存扣减示例）
EVAL "
local stock = tonumber(redis.call('get', KEYS[1]))
if stock and stock >= tonumber(ARGV[1]) then
    redis.call('decrby', KEYS[1], ARGV[1])
    return 1
end
return 0
" 1 stock:product:1001 2
```

**Python redis-py 示例:**

```python
# 注册 Lua 脚本（推荐，避免重复传输脚本体）
deduct_stock_script = r.register_script("""
local stock = tonumber(redis.call('get', KEYS[1]))
if stock and stock >= tonumber(ARGV[1]) then
    redis.call('decrby', KEYS[1], ARGV[1])
    return 1
end
return 0
""")

# 使用
r.set("stock:product:1001", 100)
result = deduct_stock_script(keys=["stock:product:1001"], args=[2])
# result == 1 表示扣减成功

# 限流器（滑动窗口）
rate_limit_script = r.register_script("""
local key = KEYS[1]
local limit = tonumber(ARGV[1])
local window = tonumber(ARGV[2])
local now = tonumber(ARGV[3])

redis.call('zremrangebyscore', key, 0, now - window)
local count = redis.call('zcard', key)

if count < limit then
    redis.call('zadd', key, now, now .. ':' .. math.random(100000))
    redis.call('expire', key, window)
    return 1
end
return 0
""")

import time
allowed = rate_limit_script(
    keys=["ratelimit:api:user:1001"],
    args=[100, 60, int(time.time())]  # 100次/60秒
)
```

---

## Pipeline 与事务

### Pipeline（批量命令，减少网络往返）

```python
# 不使用 Pipeline：N 次命令 = N 次网络往返
# 使用 Pipeline：N 次命令 = 1 次网络往返

pipe = r.pipeline(transaction=False)  # 非事务 Pipeline
for i in range(1000):
    pipe.set(f"key:{i}", f"value:{i}")
results = pipe.execute()  # 批量执行

# 带读取的 Pipeline
pipe = r.pipeline(transaction=False)
for i in range(100):
    pipe.get(f"key:{i}")
values = pipe.execute()  # 返回100个值
```

### 事务（MULTI/EXEC）

```python
# Redis 事务：原子执行一组命令（但不支持回滚）
pipe = r.pipeline(transaction=True)  # 默认就是 True
pipe.multi()
pipe.set("account:A:balance", 800)
pipe.set("account:B:balance", 1200)
pipe.execute()  # 原子执行

# WATCH 乐观锁
def transfer(from_account: str, to_account: str, amount: int) -> bool:
    with r.pipeline() as pipe:
        while True:
            try:
                pipe.watch(from_account, to_account)
                balance_from = int(pipe.get(from_account) or 0)
                balance_to = int(pipe.get(to_account) or 0)

                if balance_from < amount:
                    pipe.unwatch()
                    return False

                pipe.multi()
                pipe.set(from_account, balance_from - amount)
                pipe.set(to_account, balance_to + amount)
                pipe.execute()
                return True
            except redis.WatchError:
                continue  # 被其他客户端修改，重试
```

---

## 内存优化

### 编码优化

Redis 自动选择紧凑编码以节省内存：

| 数据结构 | 紧凑编码 | 条件 | 普通编码 |
|----------|----------|------|----------|
| Hash | listpack (ziplist) | field 数 <= hash-max-listpack-entries (128) 且值 <= hash-max-listpack-value (64B) | hashtable |
| List | listpack | 元素数 <= list-max-listpack-size (128) | quicklist |
| Set | intset | 全部为整数且数量 <= set-max-intset-entries (512) | hashtable |
| Set | listpack | 元素数 <= set-max-listpack-entries (128) | hashtable |
| ZSet | listpack | 元素数 <= zset-max-listpack-entries (128) 且值 <= zset-max-listpack-value (64B) | skiplist + hashtable |

```bash
# 查看 key 的编码方式
OBJECT ENCODING mykey

# 调整阈值（redis.conf）
hash-max-listpack-entries 128
hash-max-listpack-value 64
list-max-listpack-size 128
set-max-intset-entries 512
zset-max-listpack-entries 128
zset-max-listpack-value 64
```

### 内存策略

```bash
# 最大内存限制
maxmemory 4gb

# 淘汰策略
maxmemory-policy allkeys-lru       # 推荐：所有 key 中淘汰最近最少使用
# 可选策略:
# noeviction        - 不淘汰，写入报错（默认）
# allkeys-lru       - 所有 key LRU 淘汰
# allkeys-lfu       - 所有 key LFU 淘汰（Redis 4.0+）
# volatile-lru      - 仅带 TTL 的 key LRU 淘汰
# volatile-lfu      - 仅带 TTL 的 key LFU 淘汰
# volatile-ttl      - 淘汰 TTL 最短的 key
# allkeys-random    - 随机淘汰
# volatile-random   - 随机淘汰（仅带 TTL 的 key）
```

### 内存碎片整理

```bash
# 查看内存碎片率
INFO memory
# mem_fragmentation_ratio > 1.5 表示碎片严重

# 在线碎片整理（Redis 4.0+）
CONFIG SET activedefrag yes
CONFIG SET active-defrag-threshold-lower 10     # 碎片率 > 10% 开始
CONFIG SET active-defrag-threshold-upper 100    # 碎片率 > 100% 全力整理
CONFIG SET active-defrag-cycle-min 1            # CPU 占用最小 1%
CONFIG SET active-defrag-cycle-max 25           # CPU 占用最大 25%
```

**Python 内存分析:**

```python
info = r.info("memory")
print(f"已使用内存: {info['used_memory_human']}")
print(f"峰值内存: {info['used_memory_peak_human']}")
print(f"碎片率: {info['mem_fragmentation_ratio']}")
print(f"淘汰策略: {info['maxmemory_policy']}")

# 大 key 分析
# redis-cli --bigkeys         # 扫描大 key
# redis-cli --memkeys         # 按内存排序
```

---

## 监控

### INFO 命令

```bash
INFO                     # 全部信息
INFO server              # 服务器信息
INFO clients             # 客户端连接
INFO memory              # 内存使用
INFO stats               # 统计信息
INFO replication         # 复制状态
INFO keyspace            # 数据库 key 统计
INFO commandstats        # 命令执行统计
```

**Python 监控脚本:**

```python
def redis_health_check():
    info = r.info()
    metrics = {
        "connected_clients": info["connected_clients"],
        "used_memory_human": info["used_memory_human"],
        "mem_fragmentation_ratio": info["mem_fragmentation_ratio"],
        "instantaneous_ops_per_sec": info["instantaneous_ops_per_sec"],
        "hit_rate": (
            info["keyspace_hits"]
            / max(info["keyspace_hits"] + info["keyspace_misses"], 1)
            * 100
        ),
        "connected_slaves": info.get("connected_slaves", 0),
        "rejected_connections": info["rejected_connections"],
        "evicted_keys": info["evicted_keys"],
        "expired_keys": info["expired_keys"],
    }

    # 告警阈值
    if metrics["mem_fragmentation_ratio"] > 1.5:
        alert("内存碎片率过高", metrics["mem_fragmentation_ratio"])
    if metrics["hit_rate"] < 90:
        alert("缓存命中率低", metrics["hit_rate"])
    if metrics["connected_clients"] > 9000:
        alert("连接数接近上限", metrics["connected_clients"])

    return metrics
```

### SLOWLOG

```bash
# 配置慢查询阈值（微秒）
CONFIG SET slowlog-log-slower-than 10000    # 10ms
CONFIG SET slowlog-max-len 128              # 保留最近128条

# 查看慢查询
SLOWLOG GET 10          # 最近10条
SLOWLOG LEN             # 慢查询数量
SLOWLOG RESET           # 清空
```

### 延迟监控

```bash
# 实时延迟检测
redis-cli --latency                  # 持续检测
redis-cli --latency-history          # 每15秒一组
redis-cli --latency-dist             # 延迟分布图

# 内部延迟监控
CONFIG SET latency-monitor-threshold 100    # 超过100ms记录
LATENCY LATEST                               # 最近事件
LATENCY HISTORY event_name                   # 历史记录
LATENCY RESET                                # 清空
```

### CLIENT LIST

```bash
# 查看所有客户端连接
CLIENT LIST
CLIENT LIST TYPE normal     # 仅普通连接

# 关键字段
# id=5 addr=127.0.0.1:50000 fd=8 name= db=0 cmd=get age=100 idle=0
# age: 连接时长  idle: 空闲时长  cmd: 最近命令

# 关闭空闲连接
CLIENT KILL ID 5
CONFIG SET timeout 300      # 300秒空闲自动断开
```

---

## 安全

### ACL（Redis 6.0+）

```bash
# 查看当前用户
ACL WHOAMI

# 创建用户
ACL SETUSER analyst on >strong_password ~report:* &* +get +mget +hgetall +info -@dangerous

# 语法说明:
# on/off         - 启用/禁用
# >password      - 设置密码
# ~key_pattern   - 允许的 key 模式
# &channel       - 允许的 Pub/Sub channel
# +command       - 允许的命令
# -command       - 禁止的命令
# +@category     - 允许整个命令类别
# -@dangerous    - 禁止危险命令

# 查看所有用户
ACL LIST
ACL GETUSER analyst

# 持久化 ACL 规则
ACL SAVE                    # 保存到 aclfile
ACL LOAD                    # 从 aclfile 加载

# 常用命令类别
ACL CAT                     # 列出所有类别
# @read, @write, @set, @hash, @list, @sortedset, @string
# @admin, @dangerous, @slow, @fast, @keyspace, @pubsub
```

### TLS 加密

```bash
# redis.conf
tls-port 6380
port 0                          # 禁用非 TLS 端口
tls-cert-file /path/to/redis.crt
tls-key-file /path/to/redis.key
tls-ca-cert-file /path/to/ca.crt
tls-auth-clients optional       # 客户端证书验证

# 客户端连接
redis-cli --tls --cert /path/to/client.crt --key /path/to/client.key --cacert /path/to/ca.crt -h host -p 6380
```

**Python TLS 连接:**

```python
import ssl

ssl_context = ssl.create_default_context(cafile="/path/to/ca.crt")
ssl_context.load_cert_chain(
    certfile="/path/to/client.crt",
    keyfile="/path/to/client.key",
)

r = redis.Redis(
    host="redis-host",
    port=6380,
    ssl=True,
    ssl_ca_certs="/path/to/ca.crt",
    ssl_certfile="/path/to/client.crt",
    ssl_keyfile="/path/to/client.key",
    decode_responses=True,
)
```

### 密码认证

```bash
# redis.conf
requirepass "your_strong_password_here"

# 客户端认证
AUTH "your_strong_password_here"
redis-cli -a "your_strong_password_here"     # 不推荐（密码出现在进程列表）
redis-cli --askpass                           # 推荐（交互式输入）
```

### 安全加固清单

```bash
# 1. 绑定地址
bind 127.0.0.1 192.168.1.100    # 不要 bind 0.0.0.0

# 2. 禁用危险命令
rename-command FLUSHALL ""
rename-command FLUSHDB ""
rename-command CONFIG ""         # 或重命名为随机字符串
rename-command DEBUG ""
rename-command KEYS ""           # 用 SCAN 替代

# 3. 禁用 Lua 调试
enable-debug-command no

# 4. 连接限制
maxclients 10000
timeout 300                      # 空闲超时

# 5. 保护模式
protected-mode yes               # 默认开启
```

---

## 常见陷阱

### 大 Key（Big Key）

大 Key 导致：阻塞其他命令、网络拥塞、内存倾斜、主从同步延迟。

```bash
# 检测大 Key
redis-cli --bigkeys
redis-cli --memkeys --memkeys-samples 100

# 查看单个 key 内存
MEMORY USAGE mykey SAMPLES 0
OBJECT ENCODING mykey
DEBUG OBJECT mykey              # 生产慎用
```

**治理方案:**

```python
# 拆分大 Hash（>= 5000 fields 应拆分）
def split_large_hash(key: str, data: dict, bucket_size: int = 500):
    for field, value in data.items():
        bucket = hash(field) % (len(data) // bucket_size + 1)
        r.hset(f"{key}:bucket:{bucket}", field, value)

def get_from_split_hash(key: str, field: str, total_buckets: int):
    bucket = hash(field) % total_buckets
    return r.hget(f"{key}:bucket:{bucket}", field)

# 异步删除大 Key（Redis 4.0+）
r.unlink("large_key")   # 异步删除，不阻塞主线程
# 避免使用 DEL 删除大 Key
```

**大 Key 阈值参考:**

| 数据类型 | 危险阈值 |
|----------|----------|
| String | > 10KB |
| Hash | > 5000 fields 或 value > 10KB |
| List | > 10000 元素 |
| Set | > 10000 成员 |
| ZSet | > 10000 成员 |

### 热 Key（Hot Key）

单个 key 被高并发访问，导致单节点成为瓶颈。

```python
# 方案1：本地缓存 + Redis（两级缓存）
from functools import lru_cache
import time

LOCAL_CACHE = {}
LOCAL_TTL = 5  # 本地缓存5秒

def get_hot_key(key: str):
    now = time.time()
    if key in LOCAL_CACHE and now < LOCAL_CACHE[key]["expire"]:
        return LOCAL_CACHE[key]["value"]

    value = r.get(key)
    LOCAL_CACHE[key] = {"value": value, "expire": now + LOCAL_TTL}
    return value

# 方案2：读副本分散（key 加后缀）
import random

def set_hot_key(key: str, value: str, replicas: int = 3):
    pipe = r.pipeline(transaction=False)
    for i in range(replicas):
        pipe.set(f"{key}:r{i}", value, ex=3600)
    pipe.execute()

def get_hot_key_replica(key: str, replicas: int = 3):
    replica = random.randint(0, replicas - 1)
    return r.get(f"{key}:r{replica}")
```

### 内存碎片

长时间运行后频繁的创建/删除操作会产生内存碎片。

```python
def check_fragmentation():
    info = r.info("memory")
    ratio = info["mem_fragmentation_ratio"]

    if ratio > 1.5:
        print(f"WARNING: 内存碎片率 {ratio:.2f}，建议启用在线碎片整理")
        r.config_set("activedefrag", "yes")
    elif ratio < 1.0:
        print(f"WARNING: 碎片率 < 1.0 ({ratio:.2f})，可能存在 swap，检查系统内存")
    else:
        print(f"OK: 内存碎片率 {ratio:.2f}")
```

### 其他常见陷阱

```python
# 1. KEYS 命令（生产禁用，用 SCAN 替代）
# ❌ KEYS user:*           # O(N) 全量扫描，阻塞
# ✅ SCAN 0 MATCH user:* COUNT 100

def scan_keys(pattern: str, count: int = 100):
    """安全扫描 key"""
    cursor = 0
    keys = []
    while True:
        cursor, batch = r.scan(cursor=cursor, match=pattern, count=count)
        keys.extend(batch)
        if cursor == 0:
            break
    return keys

# 2. 连接泄漏（务必使用连接池）
# ❌
r = redis.Redis(host="localhost")  # 每次新建连接

# ✅
pool = redis.ConnectionPool(host="localhost", port=6379, max_connections=50)
r = redis.Redis(connection_pool=pool)

# 3. 序列化选择
import json
import pickle

# ❌ pickle（安全风险 + 不可跨语言）
r.set("data", pickle.dumps(obj))

# ✅ JSON（安全 + 可跨语言）
r.set("data", json.dumps(obj))

# ✅ msgpack（更紧凑）
import msgpack
r.set("data", msgpack.packb(obj))

# 4. 过期时间丢失
r.set("key", "value", ex=3600)
r.set("key", "new_value")       # ⚠️ TTL 被清除！
r.set("key", "new_value", keepttl=True)  # ✅ 保留 TTL (Redis 6.0+)
```

---

## Agent Checklist

开发和运维 Redis 相关任务时，按以下清单检查：

### 数据模型设计
- [ ] 选择了最合适的数据结构（不是所有场景都用 String）
- [ ] Hash/List/Set/ZSet 的元素数量在紧凑编码阈值内（或已评估超出的影响）
- [ ] Key 命名规范统一（`业务:对象类型:ID:属性`，使用冒号分隔）
- [ ] 已评估大 Key 风险（单个 value < 10KB，集合类 < 10000 元素）
- [ ] 热 Key 已识别并有应对方案（本地缓存 / 读副本）

### 缓存策略
- [ ] 明确选择了缓存模式（Cache-Aside / Write-Through / Write-Behind）
- [ ] 所有缓存 key 设置了合理的 TTL
- [ ] TTL 添加了随机偏移（防止缓存雪崩）
- [ ] 缓存穿透防护已就位（空值缓存 / 布隆过滤器）
- [ ] 热点 key 击穿防护已就位（互斥锁 / 逻辑过期）

### 高可用与持久化
- [ ] 持久化策略已配置（推荐混合持久化）
- [ ] 主从复制已验证 `INFO replication`
- [ ] Sentinel / Cluster 故障转移已测试
- [ ] 客户端使用 Sentinel/Cluster 感知的连接方式
- [ ] `min-replicas-to-write` 和 `min-replicas-max-lag` 已配置

### 性能与内存
- [ ] 使用 Pipeline 批量执行命令（减少网络往返）
- [ ] 生产环境禁用 KEYS 命令，使用 SCAN 替代
- [ ] `maxmemory` 和 `maxmemory-policy` 已配置
- [ ] 内存碎片率定期检查（`mem_fragmentation_ratio`）
- [ ] 大 Key 删除使用 `UNLINK` 而非 `DEL`
- [ ] 连接使用连接池，max_connections 已合理设置

### 分布式锁
- [ ] 锁有 token（UUID），释放时校验 token（Lua 原子操作）
- [ ] 锁设置了合理的过期时间（防止死锁）
- [ ] 已考虑锁续期场景（长任务需 watchdog 机制）
- [ ] 跨数据中心场景评估了 Redlock 的适用性

### 安全
- [ ] 启用了 ACL（Redis 6.0+），最小权限原则
- [ ] 危险命令已禁用或重命名（FLUSHALL / FLUSHDB / CONFIG / KEYS / DEBUG）
- [ ] 绑定了具体 IP（非 0.0.0.0）
- [ ] TLS 加密已启用（公网 / 跨网段必须）
- [ ] `protected-mode yes` 已确认

### 监控与运维
- [ ] SLOWLOG 已配置（阈值 10ms）
- [ ] 缓存命中率持续监控（`keyspace_hits / (hits + misses)`）
- [ ] 连接数、内存、OPS 有告警
- [ ] 定期执行 `redis-cli --bigkeys` 巡检
- [ ] AOF 重写和 RDB 快照的资源影响已评估

---

**知识ID**: `redis-complete`
**领域**: data
**类型**: standards
**难度**: intermediate-advanced
**质量分**: 95
**维护者**: data-team@superdev.com
**最后更新**: 2026-03-28
