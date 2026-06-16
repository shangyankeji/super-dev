---
id: rate-limiting-complete
title: 限流完整指南
domain: backend
category: 01-standards
difficulty: intermediate
tags: [backend, complete, http, limiting, rate, 分布式限流, 响应头, 多维度限流]
quality_score: 70
last_updated: 2026-06-15
---
# 限流完整指南

## 概述

限流 (Rate Limiting) 是保护系统免受过载和滥用的核心机制。通过限制单位时间内的请求数量，防止系统被恶意攻击或突发流量击垮。本指南覆盖令牌桶、漏桶、滑动窗口等算法以及分布式限流和降级策略。

---

## 限流算法

### 1. 令牌桶算法 (Token Bucket)

以固定速率向桶中添加令牌，请求需要获取令牌才能通过。桶满时多余令牌丢弃，允许突发流量。

```python
import time
import threading

class TokenBucket:
    def __init__(self, rate: float, capacity: int):
        self.rate = rate            # 每秒生成的令牌数
        self.capacity = capacity    # 桶容量
        self.tokens = capacity      # 当前令牌数
        self.last_refill = time.monotonic()
        self.lock = threading.Lock()

    def allow(self, tokens: int = 1) -> bool:
        with self.lock:
            now = time.monotonic()
            elapsed = now - self.last_refill
            self.tokens = min(
                self.capacity,
                self.tokens + elapsed * self.rate
            )
            self.last_refill = now

            if self.tokens >= tokens:
                self.tokens -= tokens
                return True
            return False

# 使用
limiter = TokenBucket(rate=100, capacity=200)  # 100 req/s，允许突发 200
if limiter.allow():
    process_request()
else:
    return_429()
```

### 2. 漏桶算法 (Leaky Bucket)

请求进入桶中排队，以固定速率处理。桶满时拒绝新请求。输出速率恒定。

```python
import time
import threading
from collections import deque

class LeakyBucket:
    def __init__(self, rate: float, capacity: int):
        self.rate = rate
        self.capacity = capacity
        self.queue: deque = deque()
        self.last_leak = time.monotonic()
        self.lock = threading.Lock()

    def allow(self) -> bool:
        with self.lock:
            self._leak()
            if len(self.queue) < self.capacity:
                self.queue.append(time.monotonic())
                return True
            return False

    def _leak(self):
        now = time.monotonic()
        elapsed = now - self.last_leak
        leaked = int(elapsed * self.rate)
        if leaked > 0:
            for _ in range(min(leaked, len(self.queue))):
                self.queue.popleft()
            self.last_leak = now
```

### 3. 固定窗口计数器 (Fixed Window Counter)

将时间分为固定窗口（如每分钟），统计窗口内的请求数。

```python
class FixedWindowCounter:
    def __init__(self, limit: int, window_seconds: int):
        self.limit = limit
        self.window = window_seconds
        self.counts: dict[str, int] = {}
        self.lock = threading.Lock()

    def allow(self, key: str) -> bool:
        window_key = f"{key}:{int(time.time()) // self.window}"
        with self.lock:
            count = self.counts.get(window_key, 0)
            if count >= self.limit:
                return False
            self.counts[window_key] = count + 1
            # 清理旧窗口
            self._cleanup()
            return True

    def _cleanup(self):
        current_window = int(time.time()) // self.window
        expired = [k for k in self.counts if int(k.split(":")[-1]) < current_window - 1]
        for k in expired:
            del self.counts[k]
```

### 4. 滑动窗口日志 (Sliding Window Log)

记录每个请求的时间戳，统计滑动窗口内的请求数。精度高但内存开销大。

```python
class SlidingWindowLog:
    def __init__(self, limit: int, window_seconds: int):
        self.limit = limit
        self.window = window_seconds
        self.logs: dict[str, list[float]] = {}
        self.lock = threading.Lock()

    def allow(self, key: str) -> bool:
        now = time.monotonic()
        with self.lock:
            if key not in self.logs:
                self.logs[key] = []

            # 清除过期记录
            cutoff = now - self.window
            self.logs[key] = [t for t in self.logs[key] if t > cutoff]

            if len(self.logs[key]) >= self.limit:
                return False
            self.logs[key].append(now)
            return True
```

### 5. 滑动窗口计数器 (Sliding Window Counter)

结合固定窗口和滑动窗口，通过加权计算实现近似滑动窗口。

```python
class SlidingWindowCounter:
    def __init__(self, limit: int, window_seconds: int):
        self.limit = limit
        self.window = window_seconds

    def allow(self, key: str, redis_client) -> bool:
        now = time.time()
        current_window = int(now) // self.window
        previous_window = current_window - 1
        window_elapsed = (now % self.window) / self.window

        current_count = int(redis_client.get(f"{key}:{current_window}") or 0)
        previous_count = int(redis_client.get(f"{key}:{previous_window}") or 0)

        # 加权计算
        estimated = previous_count * (1 - window_elapsed) + current_count
        if estimated >= self.limit:
            return False

        pipe = redis_client.pipeline()
        pipe.incr(f"{key}:{current_window}")
        pipe.expire(f"{key}:{current_window}", self.window * 2)
        pipe.execute()
        return True
```

---

## 算法对比

| 算法 | 突发允许 | 精度 | 内存 | 适用场景 |
|------|----------|------|------|----------|
| 令牌桶 | 允许 | 高 | 低 | API 网关 |
| 漏桶 | 不允许 | 高 | 中 | 平滑流量 |
| 固定窗口 | 边界翻倍 | 低 | 低 | 简单计数 |
| 滑动日志 | 不允许 | 最高 | 高 | 精确限流 |
| 滑动计数器 | 部分 | 高 | 低 | 通用推荐 |

---

## 分布式限流

### Redis + Lua 原子操作

```python
# 滑动窗口计数器 - Redis Lua 脚本
SLIDING_WINDOW_SCRIPT = """
local key = KEYS[1]
local window = tonumber(ARGV[1])
local limit = tonumber(ARGV[2])
local now = tonumber(ARGV[3])

-- 清除过期成员
redis.call("ZREMRANGEBYSCORE", key, 0, now - window)

-- 当前窗口内的请求数
local count = redis.call("ZCARD", key)

if count < limit then
    -- 添加当前请求
    redis.call("ZADD", key, now, now .. ":" .. math.random(1000000))
    redis.call("EXPIRE", key, window)
    return 1
else
    return 0
end
"""

def rate_limit_distributed(key: str, limit: int, window: int) -> bool:
    result = r.eval(
        SLIDING_WINDOW_SCRIPT,
        1,
        f"ratelimit:{key}",
        window,
        limit,
        int(time.time() * 1000),
    )
    return result == 1
```

### API 网关限流

```yaml
# Kong API Gateway 配置
plugins:
  - name: rate-limiting
    config:
      minute: 60                # 每分钟 60 次
      hour: 1000                # 每小时 1000 次
      policy: redis              # 分布式策略
      redis_host: redis
      redis_port: 6379
      fault_tolerant: true       # Redis 不可用时放行
      hide_client_headers: false
      limit_by: consumer         # 按消费者限流

# Nginx 限流
http {
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_conn_zone $binary_remote_addr zone=conn:10m;

    server {
        location /api/ {
            limit_req zone=api burst=20 nodelay;
            limit_conn conn 100;
            limit_req_status 429;
        }
    }
}
```

---

## 多维度限流

```python
class MultiDimensionRateLimiter:
    """多维度限流器：IP + 用户 + 接口"""

    def __init__(self, redis_client):
        self.redis = redis_client
        self.rules = {
            "ip": {"limit": 100, "window": 60},         # 每 IP 每分钟 100 次
            "user": {"limit": 1000, "window": 3600},     # 每用户每小时 1000 次
            "endpoint": {"limit": 30, "window": 60},     # 每接口每分钟 30 次
            "login": {"limit": 5, "window": 300},        # 登录每 5 分钟 5 次
        }

    def check(self, ip: str, user_id: str | None, endpoint: str) -> tuple[bool, dict]:
        results = {}

        # IP 维度
        results["ip"] = self._check_dimension(
            f"ratelimit:ip:{ip}", self.rules["ip"]
        )

        # 用户维度
        if user_id:
            results["user"] = self._check_dimension(
                f"ratelimit:user:{user_id}", self.rules["user"]
            )

        # 接口维度
        key_endpoint = f"ratelimit:endpoint:{user_id or ip}:{endpoint}"
        results["endpoint"] = self._check_dimension(
            key_endpoint, self.rules["endpoint"]
        )

        # 敏感接口特殊限流
        if endpoint in ("/api/login", "/api/register", "/api/reset-password"):
            results["sensitive"] = self._check_dimension(
                f"ratelimit:login:{ip}", self.rules["login"]
            )

        allowed = all(r["allowed"] for r in results.values())
        return allowed, results

    def _check_dimension(self, key: str, rule: dict) -> dict:
        allowed = rate_limit_distributed(key, rule["limit"], rule["window"])
        remaining = max(0, rule["limit"] - int(self.redis.zcard(key) or 0))
        return {
            "allowed": allowed,
            "limit": rule["limit"],
            "remaining": remaining,
            "window": rule["window"],
        }
```

---

## HTTP 响应头

```python
# FastAPI 限流中间件
from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware

class RateLimitMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request: Request, call_next):
        client_ip = request.client.host
        user_id = getattr(request.state, "user_id", None)
        endpoint = request.url.path

        allowed, results = rate_limiter.check(client_ip, user_id, endpoint)

        if not allowed:
            return Response(
                content=json.dumps({"error": "Rate limit exceeded"}),
                status_code=429,
                headers={
                    "Retry-After": "60",
                    "X-RateLimit-Limit": str(results.get("ip", {}).get("limit", 0)),
                    "X-RateLimit-Remaining": "0",
                    "X-RateLimit-Reset": str(int(time.time()) + 60),
                },
                media_type="application/json",
            )

        response = await call_next(request)

        # 添加限流信息头
        ip_info = results.get("ip", {})
        response.headers["X-RateLimit-Limit"] = str(ip_info.get("limit", 0))
        response.headers["X-RateLimit-Remaining"] = str(ip_info.get("remaining", 0))
        return response
```

---

## 降级策略

```python
class GracefulDegradation:
    """优雅降级管理器"""

    def __init__(self):
        self.levels = {
            "normal": {"cache_ttl": 300, "features": "all"},
            "warning": {"cache_ttl": 900, "features": "core_only"},
            "critical": {"cache_ttl": 3600, "features": "readonly"},
            "emergency": {"cache_ttl": 7200, "features": "static"},
        }
        self.current_level = "normal"

    def check_and_degrade(self, metrics: dict):
        error_rate = metrics.get("error_rate", 0)
        latency_p99 = metrics.get("latency_p99", 0)
        cpu_usage = metrics.get("cpu_usage", 0)

        if error_rate > 10 or cpu_usage > 95:
            self.current_level = "emergency"
        elif error_rate > 5 or latency_p99 > 5000:
            self.current_level = "critical"
        elif error_rate > 1 or latency_p99 > 2000:
            self.current_level = "warning"
        else:
            self.current_level = "normal"

    def get_config(self) -> dict:
        return self.levels[self.current_level]

    def is_feature_available(self, feature: str) -> bool:
        available = self.levels[self.current_level]["features"]
        if available == "all":
            return True
        if available == "static":
            return feature in ("health", "status")
        if available == "readonly":
            return feature not in ("write", "upload", "export")
        if available == "core_only":
            return feature in ("auth", "read", "health")
        return False
```

---

## 监控指标

| 指标 | 说明 | 告警阈值 |
|------|------|----------|
| 限流触发次数 | 被拒绝的请求数 | 突增 > 3x |
| 限流命中率 | 触发限流的请求比例 | > 5% |
| 429 响应率 | HTTP 429 响应比例 | > 1% |
| Redis 延迟 | 限流器 Redis 延迟 | > 10ms |

---

## 常见反模式

| 反模式 | 问题 | 正确做法 |
|--------|------|----------|
| 仅客户端限流 | 可被绕过 | 服务端强制限流 |
| 全局统一限额 | 正常用户被误杀 | 按用户/IP/接口分维度 |
| 限流后无 Retry-After | 客户端盲目重试 | 返回 429 + Retry-After |
| Redis 不可用则拒绝所有 | 可用性下降 | fault-tolerant: 降级放行 |
| 不区分接口敏感度 | 登录等接口被暴力攻击 | 敏感接口单独更严限流 |
| 固定窗口边界问题 | 窗口交界处突发翻倍 | 使用滑动窗口计数器 |

---

## Agent Checklist

- [ ] 选择合适的限流算法（推荐滑动窗口计数器或令牌桶）
- [ ] 实现多维度限流（IP / 用户 / 接口 / 敏感操作）
- [ ] 分布式限流使用 Redis Lua 脚本保证原子性
- [ ] 返回标准限流响应头（X-RateLimit-Limit / Remaining / Reset）
- [ ] HTTP 429 响应包含 Retry-After 头
- [ ] 登录/注册/重置密码等接口单独设置更严格限额
- [ ] Redis 不可用时有降级策略（放行或本地限流）
- [ ] 限流规则可动态调整（配置中心/环境变量）
- [ ] 接入监控告警（限流触发率 / 429 响应率）
- [ ] API 网关层和应用层都实施限流
- [ ] 白名单机制（内部服务/健康检查绕过限流）
- [ ] 定期回顾限流阈值，根据实际流量调整
