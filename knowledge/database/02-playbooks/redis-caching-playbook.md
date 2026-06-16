---
id: redis-caching-playbook
title: Redis 缓存实战手册
domain: database
category: 02-playbooks
difficulty: advanced
tags: [redis, caching, cache-aside, write-through, write-behind, invalidation, ttl, stampede, production, database, performance]
quality_score: 93
maintainer: platform-team@superdev.com
last_updated: 2026-06-15
---

# Redis 缓存实战手册

> 基于 [AWS Redis Caching Strategies](https://docs.aws.amazon.com/whitepapers/latest/database-caching-strategies-using-redis/caching-patterns.html) + [Azure Cache-Aside Pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/cache-aside) + [Redis Blog](https://redis.io/blog/why-your-caching-strategies-might-be-holding-you-back-and-what-to-consider-next/)

## 四种缓存模式

### 1. Cache-Aside（最常用）
应用先查缓存，miss 时查 DB 并回填。最终一致（TTL 控制）。
```python
def get_user(user_id):
    cached = redis.get(f"user:{user_id}")
    if cached:
        return json.loads(cached)
    user = db.query(User).get(user_id)
    redis.setex(f"user:{user_id}", 300, json.dumps(user.to_dict()))  # TTL 5min
    return user
```

### 2. Write-Through（强一致）
写操作同时更新缓存和 DB（同步双写）。
```python
def update_user(user_id, data):
    user = db.update(User, user_id, data)
    redis.set(f"user:{user_id}", json.dumps(user.to_dict()))  # 同步更新缓存
    return user
```

### 3. Write-Behind（高写吞吐）
先写缓存，异步刷 DB。容忍短暂不一致。
```python
def increment_counter(key):
    redis.incr(key)                    # 先写缓存
    enqueue_job("persist_counter", key) # 异步刷 DB
```

### 4. Refresh-Ahead（热数据预热）
过期前主动刷新。
```python
def get_hot_data(key):
    cached = redis.get(key)
    ttl = redis.ttl(key)
    if cached and ttl < 60:  # 快过期了
        enqueue_job("refresh_cache", key)  # 异步刷新
    return cached
```

## 模式选择决策表
| 场景 | 推荐模式 | 一致性 |
|------|---------|--------|
| 读多写少（用户资料/配置） | Cache-Aside | 最终一致 |
| 写多且需强一致 | Write-Through | 强一致 |
| 超高写吞吐（计数器/排行） | Write-Behind | 最终一致 |
| 热点数据不可过期 | Refresh-Ahead | 最新 |

## 缓存失效策略

```python
# TTL 失效（最简单，推荐基础方案）
redis.setex(key, 300, value)  # 5 分钟后自动过期

# 主动失效（数据变更时立刻删缓存）
def update_product(product_id, data):
    product = db.update(Product, product_id, data)
    redis.delete(f"product:{product_id}")  # 主动删
    return product

# 版本化 key（适合批量失效）
redis.set(f"products:v{version}", data)
# 版本号+1 → 旧 key 自然过期
```

## 防缓存击穿（Stampede）

```python
# ❌ 缓存 miss 时 1000 个请求同时查 DB
def get_product(pid):
    cached = redis.get(f"product:{pid}")
    if not cached:
        return db.query(Product).get(pid)  # 1000 并发 = DB 爆

# ✅ 互斥锁（singleflight）—— 只让一个请求查 DB
import fcntl
def get_product(pid):
    cached = redis.get(f"product:{pid}")
    if cached:
        return json.loads(cached)
    lock = redis.set(f"lock:product:{pid}", "1", nx=True, ex=10)
    if lock:
        product = db.query(Product).get(pid)       # 只有 1 个请求查 DB
        redis.setex(f"product:{pid}", 300, json.dumps(product.to_dict()))
        redis.delete(f"lock:product:{pid}")
        return product
    else:
        time.sleep(0.1)                              # 其他请求等一下重试
        return get_product(pid)
```

## 生产检查清单
- [ ] 每个缓存 key 有 TTL（无永久缓存）
- [ ] 写操作更新/删除缓存（cache-aside + invalidation）
- [ ] 热点 key 有防击穿（互斥锁/singleflight）
- [ ] Redis 有降级策略（Redis 挂了仍能服务，直查 DB）
- [ ] 缓存命中率 > 90%（`INFO stats` 监控）
- [ ] 内存淘汰策略设置（`maxmemory-policy allkeys-lru`）
- [ ] 序列化用 JSON/MessagePack（不用 pickle——跨语言 + 安全）
