---
id: caching-strategies-complete
title: 缓存策略完整指南
domain: backend
category: 01-standards
difficulty: intermediate
tags: [backend, caching, complete, http, redis, strategies, 应用层缓存, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# 缓存策略完整指南

## 概述

缓存是提升系统性能和降低数据库负载的核心手段。正确的缓存策略可以将响应时间从秒级降至毫秒级，但错误的缓存策略会导致数据不一致、缓存雪崩等严重问题。本指南覆盖 Redis、Memcached、CDN、HTTP 缓存和应用缓存的完整策略。

---

## 缓存层次架构

```
客户端 -> CDN -> 反向代理缓存 -> 应用层缓存 -> 分布式缓存 -> 数据库
  L1       L2        L3              L4            L5          L6

L1: 浏览器缓存 (HTTP Cache-Control)
L2: CDN 边缘缓存 (CloudFlare/CloudFront)
L3: Nginx/Varnish 反向代理缓存
L4: 进程内缓存 (LRU/本地 Map)
L5: Redis/Memcached 分布式缓存
L6: 数据库查询缓存
```

---

## Redis 缓存

### 基本操作

```python
import redis
import json
from datetime import timedelta

r = redis.Redis(host="localhost", port=6379, decode_responses=True)

# 缓存读取模式 (Cache-Aside)
def get_user(user_id: str) -> dict | None:
    # 1. 先查缓存
    cached = r.get(f"user:{user_id}")
    if cached:
        return json.loads(cached)

    # 2. 缓存未命中，查数据库
    user = db.query("SELECT * FROM users WHERE id = %s", user_id)
    if user is None:
        # 缓存空值，防止缓存穿透
        r.set(f"user:{user_id}", json.dumps(None), ex=60)
        return None

    # 3. 写入缓存
    r.set(f"user:{user_id}", json.dumps(user), ex=3600)
    return user
```

### 缓存更新模式

```python
# 模式1: Cache-Aside (旁路缓存) - 最常用
def update_user(user_id: str, data: dict):
    db.execute("UPDATE users SET ... WHERE id = %s", user_id)
    r.delete(f"user:{user_id}")   # 删除缓存，下次读取时重建

# 模式2: Write-Through (直写缓存)
def update_user_write_through(user_id: str, data: dict):
    db.execute("UPDATE users SET ... WHERE id = %s", user_id)
    user = db.query("SELECT * FROM users WHERE id = %s", user_id)
    r.set(f"user:{user_id}", json.dumps(user), ex=3600)  # 同步更新缓存

# 模式3: Write-Behind (异步写回)
def update_user_write_behind(user_id: str, data: dict):
    r.set(f"user:{user_id}", json.dumps(data), ex=3600)
    queue.send("user_update", {"user_id": user_id, "data": data})
    # 异步消费者批量写入数据库
```

### 缓存穿透防护

```python
import hashlib

# 布隆过滤器防穿透
from pybloom_live import BloomFilter

user_bloom = BloomFilter(capacity=1000000, error_rate=0.01)

# 初始化时加载所有 ID
for user_id in db.query("SELECT id FROM users"):
    user_bloom.add(user_id)

def get_user_safe(user_id: str) -> dict | None:
    # 布隆过滤器快速判断
    if user_id not in user_bloom:
        return None   # 确定不存在

    cached = r.get(f"user:{user_id}")
    if cached == "null":  # 缓存空值
        return None
    if cached:
        return json.loads(cached)

    user = db.query("SELECT * FROM users WHERE id = %s", user_id)
    if user is None:
        r.set(f"user:{user_id}", "null", ex=60)  # 缓存空值 60s
        return None

    r.set(f"user:{user_id}", json.dumps(user), ex=3600)
    return user
```

### 缓存雪崩防护

```python
import random

def set_with_jitter(key: str, value: str, base_ttl: int):
    """添加随机抖动，避免大量 key 同时过期"""
    jitter = random.randint(0, base_ttl // 10)
    r.set(key, value, ex=base_ttl + jitter)

# 互斥锁防止缓存击穿
def get_hot_data(key: str) -> dict:
    cached = r.get(key)
    if cached:
        return json.loads(cached)

    lock_key = f"lock:{key}"
    if r.set(lock_key, "1", nx=True, ex=10):  # 获取锁
        try:
            data = db.query_heavy_data(key)
            r.set(key, json.dumps(data), ex=3600)
            return data
        finally:
            r.delete(lock_key)
    else:
        # 未获取锁，等待后重试
        time.sleep(0.1)
        return get_hot_data(key)
```

### Redis 数据结构应用

```python
# 排行榜 (Sorted Set)
r.zadd("leaderboard", {"user:1": 100, "user:2": 85, "user:3": 92})
top_10 = r.zrevrange("leaderboard", 0, 9, withscores=True)

# 计数器 (String + INCR)
r.incr("page_views:homepage")
r.incrby("api_calls:today", 1)

# 分布式锁
def acquire_lock(resource: str, ttl: int = 10) -> str | None:
    token = str(uuid.uuid4())
    if r.set(f"lock:{resource}", token, nx=True, ex=ttl):
        return token
    return None

def release_lock(resource: str, token: str):
    script = """
    if redis.call("get", KEYS[1]) == ARGV[1] then
        return redis.call("del", KEYS[1])
    end
    return 0
    """
    r.eval(script, 1, f"lock:{resource}", token)

# 会话存储 (Hash)
r.hset("session:abc123", mapping={
    "user_id": "123",
    "role": "admin",
    "login_at": str(int(time.time())),
})
r.expire("session:abc123", 3600)
```

---

## HTTP 缓存

### Cache-Control 策略

```nginx
# 不可变资源（带 hash 的静态文件）
location /assets/ {
    add_header Cache-Control "public, max-age=31536000, immutable";
}

# HTML 入口文件（总是验证）
location / {
    add_header Cache-Control "no-cache";
    etag on;
}

# API 响应（私有短期缓存）
location /api/ {
    add_header Cache-Control "private, max-age=0, must-revalidate";
    add_header Vary "Authorization, Accept-Encoding";
}

# 图片（公共中期缓存）
location /images/ {
    add_header Cache-Control "public, max-age=86400, stale-while-revalidate=3600";
}
```

### ETag 与条件请求

```python
# FastAPI ETag 实现
from fastapi import Request, Response
import hashlib

@app.get("/api/products/{product_id}")
async def get_product(product_id: str, request: Request, response: Response):
    product = await db.get_product(product_id)
    etag = hashlib.md5(json.dumps(product).encode()).hexdigest()

    if request.headers.get("if-none-match") == etag:
        return Response(status_code=304)

    response.headers["ETag"] = etag
    response.headers["Cache-Control"] = "private, max-age=60"
    return product
```

---

## CDN 缓存

### CDN 配置策略

```yaml
# CloudFlare Page Rules 示例
rules:
  - match: "*.example.com/assets/*"
    cache_level: cache_everything
    edge_cache_ttl: 2592000      # 30 天

  - match: "*.example.com/api/*"
    cache_level: bypass           # API 不缓存

  - match: "*.example.com/"
    cache_level: cache_everything
    edge_cache_ttl: 300           # 5 分钟
    browser_cache_ttl: 0
```

### CDN 缓存失效

```python
import requests

def purge_cdn_cache(urls: list[str]):
    """CloudFlare 缓存清除"""
    requests.post(
        f"https://api.cloudflare.com/client/v4/zones/{ZONE_ID}/purge_cache",
        headers={"Authorization": f"Bearer {CF_TOKEN}"},
        json={"files": urls},
    )

# 部署后自动清除
def post_deploy():
    purge_cdn_cache([
        "https://example.com/",
        "https://example.com/manifest.json",
    ])
```

---

## 应用层缓存

### Python LRU 缓存

```python
from functools import lru_cache
from cachetools import TTLCache

# 简单 LRU
@lru_cache(maxsize=1000)
def get_config(key: str) -> str:
    return db.query_config(key)

# TTL 缓存
config_cache = TTLCache(maxsize=500, ttl=300)

def get_setting(key: str) -> str:
    if key in config_cache:
        return config_cache[key]
    value = db.query_setting(key)
    config_cache[key] = value
    return value
```

### Node.js 本地缓存

```typescript
import NodeCache from "node-cache";

const localCache = new NodeCache({
  stdTTL: 300,          // 默认 5 分钟
  checkperiod: 60,      // 每分钟清理过期
  maxKeys: 10000,
});

async function getProduct(id: string): Promise<Product> {
  const cached = localCache.get<Product>(`product:${id}`);
  if (cached) return cached;

  const product = await db.product.findUnique({ where: { id } });
  if (product) {
    localCache.set(`product:${id}`, product, 600);
  }
  return product;
}
```

---

## 缓存失效策略

| 策略 | 说明 | 适用场景 |
|------|------|----------|
| TTL 过期 | 设置固定过期时间 | 通用场景 |
| 主动删除 | 数据变更时删除缓存 | 一致性要求高 |
| 事件驱动失效 | 监听变更事件清除 | 微服务架构 |
| 版本号方案 | Key 包含版本号 | 批量失效 |
| LRU 淘汰 | 空间满时淘汰最久未用 | 内存受限 |

```python
# 事件驱动缓存失效
async def on_user_updated(event: UserUpdatedEvent):
    # 清除相关缓存
    r.delete(f"user:{event.user_id}")
    r.delete(f"user_profile:{event.user_id}")
    # 清除列表缓存（版本号方案）
    r.incr("users_list_version")
```

---

## 监控指标

| 指标 | 说明 | 目标 |
|------|------|------|
| Hit Rate | 缓存命中率 | > 90% |
| Miss Rate | 缓存未命中率 | < 10% |
| Eviction Rate | 淘汰率 | 低且稳定 |
| Memory Usage | 内存使用量 | < 80% maxmemory |
| Latency P99 | 缓存访问延迟 | < 5ms |

---

## 常见反模式

| 反模式 | 问题 | 正确做法 |
|--------|------|----------|
| 缓存所有数据 | 内存浪费 | 只缓存热点和高频数据 |
| TTL 统一设置 | 缓存雪崩 | 添加随机抖动 |
| 不缓存空值 | 缓存穿透 | 空值短 TTL 缓存 |
| 先更新缓存再更新 DB | 数据不一致 | 先更新 DB，再删除缓存 |
| 缓存 Key 无前缀 | 命名冲突 | 统一前缀规范 |
| 不设 maxmemory | OOM | 配置 maxmemory + 淘汰策略 |

---

## Agent Checklist

- [ ] 明确缓存分层策略（客户端/CDN/反向代理/应用/分布式）
- [ ] 使用 Cache-Aside 模式（先查缓存，未命中查 DB，回填缓存）
- [ ] 数据更新时先更新数据库，再删除缓存
- [ ] 缓存 TTL 添加随机抖动防止雪崩
- [ ] 缓存空值防止穿透（短 TTL）
- [ ] 热点 Key 使用互斥锁防止击穿
- [ ] Redis maxmemory 和淘汰策略已配置
- [ ] 缓存 Key 命名规范（前缀:实体:ID）
- [ ] 静态资源带 hash，Cache-Control: immutable
- [ ] HTML 使用 no-cache + ETag
- [ ] 接入缓存命中率监控，目标 > 90%
- [ ] 部署时自动清除 CDN 缓存
