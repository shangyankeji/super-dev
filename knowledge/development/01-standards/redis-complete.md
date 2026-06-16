---
id: redis-complete
title: Redis完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, python客户端, redis, 学习路径, 最佳实践, 核心数据结构, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Redis完整指南

## 概述
Redis是内存键值数据库,支持多种数据结构(字符串、哈希、列表、集合、有序集合)。用于缓存、会话存储、消息队列等场景。

## 核心数据结构

### 1. 字符串(String)

```bash
# 设置
SET user:1 "Alice"
SET user:2 "Bob" EX 3600  # 3600秒过期

# 获取
GET user:1

# 批量操作
MSET key1 "value1" key2 "value2"
MGET key1 key2

# 自增
SET counter 0
INCR counter
INCRBY counter 10
DECR counter

# 追加
APPEND user:1 " Smith"

# 获取长度
STRLEN user:1
```

### 2. 哈希(Hash)

```bash
# 设置字段
HSET user:1 name "Alice" age 30 email "alice@example.com"

# 获取字段
HGET user:1 name
HGETALL user:1
HMGET user:1 name age

# 删除字段
HDEL user:1 age

# 检查字段存在
HEXISTS user:1 name

# 自增
HINCRBY user:1 age 1
```

### 3. 列表(List)

```bash
# 推入
LPUSH queue "task1" "task2"  # 左边插入
RPUSH queue "task3"          # 右边插入

# 弹出
LPOP queue  # 左边弹出
RPOP queue  # 右边弹出

# 获取范围
LRANGE queue 0 -1  # 获取全部

# 获取长度
LLEN queue

# 阻塞弹出
BLPOP queue 5  # 5秒超时
BRPOP queue 5
```

### 4. 集合(Set)

```bash
# 添加
SADD tags "python" "redis" "docker"

# 获取所有成员
SMEMBERS tags

# 检查存在
SISMEMBER tags "python"

# 删除
SREM tags "docker"

# 集合操作
SADD set1 "a" "b" "c"
SADD set2 "b" "c" "d"

SINTER set1 set2  # 交集: b, c
SUNION set1 set2  # 并集: a, b, c, d
SDIFF set1 set2   # 差集: a
```

### 5. 有序集合(Sorted Set)

```bash
# 添加成员(带分数)
ZADD leaderboard 100 "Alice" 95 "Bob" 87 "Carol"

# 获取排名范围
ZRANGE leaderboard 0 -1 WITHSCORES  # 升序
ZREVRANGE leaderboard 0 -1 WITHSCORES  # 降序

# 获取排名
ZRANK leaderboard "Alice"  # 升序排名
ZREVRANK leaderboard "Alice"  # 降序排名

# 获取分数
ZSCORE leaderboard "Alice"

# 增加分数
ZINCRBY leaderboard 10 "Alice"

# 删除成员
ZREM leaderboard "Carol"
```

## 高级功能

### 1. 发布订阅(Pub/Sub)

```bash
# 订阅频道
SUBSCRIBE channel1 channel2

# 发布消息
PUBLISH channel1 "Hello World"

# 模式订阅
PSUBSCRIBE news:*
```

### 2. 事务

```bash
MULTI
SET key1 "value1"
SET key2 "value2"
INCR counter
EXEC  # 执行
# 或 DISCARD  # 取消
```

### 3. 过期时间

```bash
# 设置过期(秒)
EXPIRE user:1 3600

# 设置过期(毫秒)
PEXPIRE user:1 3600000

# 设置过期时间戳
EXPIREAT user:1 1704067200

# 查看剩余时间
TTL user:1  # 秒
PTTL user:1  # 毫秒

# 取消过期
PERSIST user:1
```

### 4. 持久化

**RDB快照**:
```bash
# redis.conf
save 900 1      # 900秒内至少1个key变化
save 300 10     # 300秒内至少10个key变化
save 60 10000   # 60秒内至少10000个key变化

# 手动触发
SAVE   # 阻塞
BGSAVE  # 后台
```

**AOF日志**:
```bash
# redis.conf
appendonly yes
appendfsync everysec  # 每秒同步
```

## Python客户端

```python
import redis

# 连接
r = redis.Redis(host='localhost', port=6379, db=0)

# 字符串
r.set('user:1', 'Alice', ex=3600)
name = r.get('user:1')

# 哈希
r.hset('user:1', mapping={
    'name': 'Alice',
    'age': 30,
    'email': 'alice@example.com'
})
user = r.hgetall('user:1')

# 列表
r.lpush('queue', 'task1', 'task2')
task = r.rpop('queue')

# 集合
r.sadd('tags', 'python', 'redis')
tags = r.smembers('tags')

# 有序集合
r.zadd('leaderboard', {'Alice': 100, 'Bob': 95})
top = r.zrevrange('leaderboard', 0, 9, withscores=True)

# 发布订阅
pubsub = r.pubsub()
pubsub.subscribe('channel1')

for message in pubsub.listen():
    print(message)
```

## 最佳实践

### ✅ DO

1. **使用连接池**
```python
pool = redis.ConnectionPool(
    host='localhost',
    port=6379,
    max_connections=100
)
r = redis.Redis(connection_pool=pool)
```

2. **使用Pipeline批量操作**
```python
pipe = r.pipeline()
for i in range(1000):
    pipe.set(f'key:{i}', i)
pipe.execute()
```

3. **设置合理的过期时间**
```python
r.set('session:user:123', data, ex=3600)  # 1小时
```

### ❌ DON'T

1. **不要使用KEYS命令**
```python
# ❌ 差(阻塞)
keys = r.keys('*')

# ✅ 好(使用SCAN)
for key in r.scan_iter('*'):
    print(key)
```

2. **不要存储大对象**
```python
# ❌ 差
r.set('big:data', huge_json)  # 几MB

# ✅ 好: 分片存储
for i, chunk in enumerate(chunks):
    r.set(f'big:data:{i}', chunk)
```

## 学习路径

### 初级 (1周)
1. 基本数据结构
2. 常用命令
3. 过期时间

### 中级 (1-2周)
1. 持久化(RDB/AOF)
2. 发布订阅
3. 事务

### 高级 (2-3周)
1. 集群和分片
2. Lua脚本
3. 性能优化

---

**知识ID**: `redis-complete`  
**领域**: development  
**类型**: standards  
**难度**: intermediate  
**质量分**: 93  
**维护者**: dba-team@superdev.com  
**最后更新**: 2026-03-28
