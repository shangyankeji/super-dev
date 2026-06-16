---
id: distributed-systems-patterns
title: 分布式系统模式
domain: architecture
category: 01-standards
difficulty: intermediate
tags: [agent, architecture, checklist, distributed, patterns, systems, 实战代码示例, 常见陷阱]
quality_score: 70
last_updated: 2026-06-15
---
# 分布式系统模式

## 概述
分布式系统面临网络不可靠、节点故障、时钟偏差等固有挑战。本指南覆盖CAP定理、一致性模型、分区容忍、Leader选举、共识算法等核心模式,帮助团队理解并正确应对分布式系统的复杂性。

## 核心概念

### 1. CAP定理
分布式系统最多只能同时满足以下三项中的两项:
- **Consistency(一致性)**: 所有节点在同一时刻看到相同数据
- **Availability(可用性)**: 每个请求都能收到响应(不保证最新)
- **Partition Tolerance(分区容忍)**: 网络分区时系统继续运行

实际选择:
- **CP系统**: 牺牲可用性保一致性 — ZooKeeper/etcd/HBase
- **AP系统**: 牺牲一致性保可用性 — Cassandra/DynamoDB/CouchDB
- 网络分区不可避免,所以实际是在C和A之间权衡

### 2. 一致性模型

| 模型 | 强度 | 描述 | 代表 |
|------|------|------|------|
| 强一致性 | 最强 | 读到最新写入 | 单机数据库/Raft |
| 线性一致性 | 强 | 实时顺序保证 | etcd/ZooKeeper |
| 顺序一致性 | 中强 | 全序但非实时 | — |
| 因果一致性 | 中 | 因果关系有序 | MongoDB(多数读写) |
| 最终一致性 | 弱 | 最终收敛到一致 | DNS/DynamoDB/S3 |
| 读己之写 | 弱+ | 能读到自己的写入 | 社交Feed |

### 3. 分布式系统8大谬误
1. 网络是可靠的
2. 延迟为零
3. 带宽无限
4. 网络是安全的
5. 拓扑不变
6. 只有一个管理员
7. 传输成本为零
8. 网络是同构的

## 实战代码示例

### 分布式锁(Redis)

```python
# Redis分布式锁(Redlock简化版)
import redis.asyncio as redis
import uuid
import asyncio

class DistributedLock:
    """Redis分布式锁"""

    def __init__(self, redis_client: redis.Redis, lock_name: str, ttl: int = 30):
        self.redis = redis_client
        self.lock_name = f"lock:{lock_name}"
        self.ttl = ttl
        self.token = str(uuid.uuid4())
        self._renewal_task = None

    async def acquire(self, timeout: float = 10.0) -> bool:
        """获取锁(带超时)"""
        deadline = asyncio.get_event_loop().time() + timeout
        while asyncio.get_event_loop().time() < deadline:
            acquired = await self.redis.set(
                self.lock_name, self.token, nx=True, ex=self.ttl
            )
            if acquired:
                # 启动续期任务
                self._renewal_task = asyncio.create_task(self._auto_renew())
                return True
            await asyncio.sleep(0.1)
        return False

    async def release(self):
        """释放锁(原子操作,确保只释放自己的锁)"""
        if self._renewal_task:
            self._renewal_task.cancel()
        lua_script = """
        if redis.call("get", KEYS[1]) == ARGV[1] then
            return redis.call("del", KEYS[1])
        else
            return 0
        end
        """
        await self.redis.eval(lua_script, 1, self.lock_name, self.token)

    async def _auto_renew(self):
        """自动续期(防止长任务锁过期)"""
        try:
            while True:
                await asyncio.sleep(self.ttl // 3)
                lua_script = """
                if redis.call("get", KEYS[1]) == ARGV[1] then
                    return redis.call("expire", KEYS[1], ARGV[2])
                else
                    return 0
                end
                """
                result = await self.redis.eval(
                    lua_script, 1, self.lock_name, self.token, self.ttl
                )
                if not result:
                    break
        except asyncio.CancelledError:
            pass

    async def __aenter__(self):
        acquired = await self.acquire()
        if not acquired:
            raise TimeoutError(f"Failed to acquire lock: {self.lock_name}")
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.release()

# 使用
async def process_order(order_id: str):
    lock = DistributedLock(redis_client, f"order:{order_id}")
    async with lock:
        order = await get_order(order_id)
        await process(order)
        await save_order(order)
```

### 断路器模式

```python
# 断路器(Circuit Breaker)
from enum import Enum
from dataclasses import dataclass
import time
import asyncio

class CircuitState(str, Enum):
    CLOSED = "closed"       # 正常:请求通过
    OPEN = "open"           # 断开:请求直接失败
    HALF_OPEN = "half_open" # 半开:允许少量探测

@dataclass
class CircuitBreakerConfig:
    failure_threshold: int = 5       # 失败次数阈值
    recovery_timeout: float = 30.0   # 恢复超时(秒)
    half_open_max_calls: int = 3     # 半开状态最大探测数
    success_threshold: int = 2       # 半开状态成功阈值

class CircuitBreaker:
    def __init__(self, name: str, config: CircuitBreakerConfig = None):
        self.name = name
        self.config = config or CircuitBreakerConfig()
        self.state = CircuitState.CLOSED
        self.failure_count = 0
        self.success_count = 0
        self.last_failure_time = 0.0
        self.half_open_calls = 0

    async def call(self, func, *args, **kwargs):
        """通过断路器执行函数"""
        if self.state == CircuitState.OPEN:
            if time.time() - self.last_failure_time > self.config.recovery_timeout:
                self.state = CircuitState.HALF_OPEN
                self.half_open_calls = 0
                self.success_count = 0
            else:
                raise CircuitOpenError(f"Circuit {self.name} is open")

        if self.state == CircuitState.HALF_OPEN:
            if self.half_open_calls >= self.config.half_open_max_calls:
                raise CircuitOpenError(f"Circuit {self.name} half-open limit reached")
            self.half_open_calls += 1

        try:
            result = await func(*args, **kwargs)
            self._on_success()
            return result
        except Exception as e:
            self._on_failure()
            raise

    def _on_success(self):
        if self.state == CircuitState.HALF_OPEN:
            self.success_count += 1
            if self.success_count >= self.config.success_threshold:
                self.state = CircuitState.CLOSED
                self.failure_count = 0
        else:
            self.failure_count = 0

    def _on_failure(self):
        self.failure_count += 1
        self.last_failure_time = time.time()
        if self.failure_count >= self.config.failure_threshold:
            self.state = CircuitState.OPEN

# 使用
user_service_breaker = CircuitBreaker("user-service", CircuitBreakerConfig(
    failure_threshold=5,
    recovery_timeout=30,
))

async def get_user_safe(user_id: int):
    try:
        return await user_service_breaker.call(user_client.get_user, user_id)
    except CircuitOpenError:
        # 降级:返回缓存数据
        return await cache.get(f"user:{user_id}")
```

### 一致性哈希

```python
# 一致性哈希(用于分布式缓存/分片)
import hashlib
from bisect import bisect_right
from typing import TypeVar

T = TypeVar("T")

class ConsistentHash:
    """一致性哈希环"""

    def __init__(self, nodes: list[str] = None, virtual_nodes: int = 150):
        self.virtual_nodes = virtual_nodes
        self.ring: list[int] = []
        self.node_map: dict[int, str] = {}

        for node in (nodes or []):
            self.add_node(node)

    def _hash(self, key: str) -> int:
        return int(hashlib.md5(key.encode()).hexdigest(), 16)

    def add_node(self, node: str):
        """添加节点(含虚拟节点)"""
        for i in range(self.virtual_nodes):
            virtual_key = f"{node}:vn{i}"
            hash_val = self._hash(virtual_key)
            self.ring.append(hash_val)
            self.node_map[hash_val] = node
        self.ring.sort()

    def remove_node(self, node: str):
        """移除节点"""
        for i in range(self.virtual_nodes):
            virtual_key = f"{node}:vn{i}"
            hash_val = self._hash(virtual_key)
            self.ring.remove(hash_val)
            del self.node_map[hash_val]

    def get_node(self, key: str) -> str:
        """获取key对应的节点"""
        if not self.ring:
            raise ValueError("No nodes in hash ring")
        hash_val = self._hash(key)
        idx = bisect_right(self.ring, hash_val) % len(self.ring)
        return self.node_map[self.ring[idx]]

    def get_nodes(self, key: str, count: int = 3) -> list[str]:
        """获取key对应的多个节点(副本)"""
        if not self.ring:
            return []
        nodes = []
        hash_val = self._hash(key)
        idx = bisect_right(self.ring, hash_val)

        for i in range(len(self.ring)):
            node = self.node_map[self.ring[(idx + i) % len(self.ring)]]
            if node not in nodes:
                nodes.append(node)
            if len(nodes) >= count:
                break
        return nodes

# 使用
ring = ConsistentHash(["cache-1", "cache-2", "cache-3"])
target = ring.get_node("user:12345")  # -> "cache-2"

# 添加节点时只有少量key需要重新分配
ring.add_node("cache-4")
```

### 向量时钟(因果一致性)

```python
# 向量时钟(Vector Clock)— 检测因果关系和冲突
from copy import deepcopy

class VectorClock:
    """向量时钟"""

    def __init__(self, node_id: str):
        self.node_id = node_id
        self.clock: dict[str, int] = {}

    def increment(self):
        """本地事件,递增自己的计数器"""
        self.clock[self.node_id] = self.clock.get(self.node_id, 0) + 1
        return deepcopy(self.clock)

    def update(self, other_clock: dict[str, int]):
        """收到消息,合并时钟"""
        for node, count in other_clock.items():
            self.clock[node] = max(self.clock.get(node, 0), count)
        self.increment()

    @staticmethod
    def compare(clock_a: dict, clock_b: dict) -> str:
        """
        比较两个向量时钟:
        - "before": a happened-before b
        - "after": b happened-before a
        - "concurrent": 并发(可能冲突)
        - "equal": 相同
        """
        all_nodes = set(clock_a.keys()) | set(clock_b.keys())
        a_less = False
        b_less = False

        for node in all_nodes:
            va = clock_a.get(node, 0)
            vb = clock_b.get(node, 0)
            if va < vb:
                a_less = True
            elif va > vb:
                b_less = True

        if a_less and not b_less:
            return "before"
        elif b_less and not a_less:
            return "after"
        elif not a_less and not b_less:
            return "equal"
        else:
            return "concurrent"  # 冲突!

# 使用
vc1 = VectorClock("node-1")
vc2 = VectorClock("node-2")

vc1.increment()  # {node-1: 1}
vc1.increment()  # {node-1: 2}

# node-1 发送消息给 node-2
vc2.update(vc1.clock)  # {node-1: 2, node-2: 1}

# node-1 和 node-2 各自独立更新
vc1.increment()  # {node-1: 3}
vc2.increment()  # {node-1: 2, node-2: 2}

# 检测:这两个更新是并发的
result = VectorClock.compare(vc1.clock, vc2.clock)  # "concurrent"
```

### 重试与退避策略

```python
# 完整的重试策略
import random
import asyncio
from functools import wraps

class RetryConfig:
    def __init__(
        self,
        max_retries: int = 3,
        base_delay: float = 1.0,
        max_delay: float = 60.0,
        exponential_base: float = 2.0,
        jitter: bool = True,
        retryable_exceptions: tuple = (Exception,),
    ):
        self.max_retries = max_retries
        self.base_delay = base_delay
        self.max_delay = max_delay
        self.exponential_base = exponential_base
        self.jitter = jitter
        self.retryable_exceptions = retryable_exceptions

def retry(config: RetryConfig = None):
    """重试装饰器(指数退避+抖动)"""
    if config is None:
        config = RetryConfig()

    def decorator(func):
        @wraps(func)
        async def wrapper(*args, **kwargs):
            last_exception = None
            for attempt in range(config.max_retries + 1):
                try:
                    return await func(*args, **kwargs)
                except config.retryable_exceptions as e:
                    last_exception = e
                    if attempt == config.max_retries:
                        break

                    # 计算延迟
                    delay = min(
                        config.base_delay * (config.exponential_base ** attempt),
                        config.max_delay,
                    )
                    if config.jitter:
                        delay = delay * (0.5 + random.random())

                    logger.warning(
                        "Retry attempt",
                        attempt=attempt + 1,
                        max_retries=config.max_retries,
                        delay=delay,
                        error=str(e),
                    )
                    await asyncio.sleep(delay)

            raise last_exception
        return wrapper
    return decorator

# 使用
@retry(RetryConfig(
    max_retries=3,
    base_delay=1.0,
    retryable_exceptions=(TimeoutError, ConnectionError),
))
async def call_external_api(url: str):
    async with httpx.AsyncClient() as client:
        response = await client.get(url, timeout=5.0)
        response.raise_for_status()
        return response.json()
```

### 分布式ID生成(Snowflake)

```python
# Snowflake ID生成器
import time
import threading

class SnowflakeIDGenerator:
    """Twitter Snowflake风格的分布式ID生成器
    64位: 1位符号 + 41位时间戳 + 10位机器ID + 12位序列号
    """

    EPOCH = 1704067200000  # 2024-01-01 00:00:00 UTC

    def __init__(self, worker_id: int, datacenter_id: int = 0):
        assert 0 <= worker_id < 32, "worker_id must be 0-31"
        assert 0 <= datacenter_id < 32, "datacenter_id must be 0-31"

        self.worker_id = worker_id
        self.datacenter_id = datacenter_id
        self.sequence = 0
        self.last_timestamp = -1
        self.lock = threading.Lock()

    def _current_millis(self) -> int:
        return int(time.time() * 1000)

    def generate(self) -> int:
        with self.lock:
            timestamp = self._current_millis()

            if timestamp == self.last_timestamp:
                self.sequence = (self.sequence + 1) & 0xFFF  # 4096
                if self.sequence == 0:
                    # 等待下一毫秒
                    while timestamp <= self.last_timestamp:
                        timestamp = self._current_millis()
            else:
                self.sequence = 0

            self.last_timestamp = timestamp

            return (
                ((timestamp - self.EPOCH) << 22)
                | (self.datacenter_id << 17)
                | (self.worker_id << 12)
                | self.sequence
            )

# 使用
id_gen = SnowflakeIDGenerator(worker_id=1, datacenter_id=0)
order_id = id_gen.generate()  # 18位数字ID,全局唯一,趋势递增
```

## 最佳实践

### 1. 网络不可靠假设
- 所有远程调用都要有超时
- 实现重试(指数退避+抖动)
- 断路器防止级联故障
- 降级策略(缓存/默认值)

### 2. 一致性选择
- 根据业务需求选择一致性级别
- 金融/支付:强一致性
- 社交/推荐:最终一致性
- 库存:因果一致性(避免超卖)
- 不要过度追求强一致性(代价是可用性)

### 3. 幂等性设计
- 所有写操作必须幂等
- 使用唯一请求ID去重
- 数据库使用UPSERT
- 消息消费使用去重表

### 4. 分区策略
- 选择好的分片键(均匀分布、查询友好)
- 避免跨分区事务
- 预留扩容空间(虚拟分片)
- 监控数据倾斜

### 5. 时间处理
- 不依赖壁钟时间排序(时钟偏差)
- 使用逻辑时钟(Lamport/向量时钟)
- NTP同步并监控偏差
- 使用单调时钟(monotonic clock)测量延迟

## 常见陷阱

### 陷阱1: 忽略网络分区
```python
# 错误: 假设分布式锁永远可靠
lock = await redis.lock("order:123")
# 如果Redis分区,锁可能被多个客户端同时持有

# 正确: Fencing Token
# 每次获取锁时附带递增的token
# 下游操作检查token是否最新
```

### 陷阱2: 同步调用链过长
```
# 错误: A→B→C→D→E 同步调用
# 总延迟 = sum(所有延迟), 可用性 = product(所有可用性)
# 5个服务各99.9%可用 → 总可用性 = 99.5%

# 正确: 异步解耦非关键路径
# A→B(同步,关键) → Event → C/D/E(异步,非关键)
```

### 陷阱3: 分布式事务
```python
# 错误: 尝试跨服务分布式事务(2PC)
# 性能差、可用性低、难以恢复

# 正确: 使用Saga或事件驱动最终一致性
# 每个服务本地事务 + 补偿操作
```

### 陷阱4: 不考虑数据倾斜
```python
# 错误: 用用户ID分片,某些大客户数据集中
# 导致某个分片压力远大于其他

# 正确: 监控分片大小和负载
# 使用组合分片键或二级分片
```

## Agent Checklist

### 基础设计
- [ ] CAP权衡已评估(CP/AP)
- [ ] 一致性模型已选择并文档化
- [ ] 幂等性设计覆盖所有写操作
- [ ] 分布式ID方案已确定

### 韧性设计
- [ ] 所有远程调用有超时和重试
- [ ] 断路器已配置
- [ ] 降级策略已实现
- [ ] 消息消费幂等

### 数据一致性
- [ ] 分片策略合理(键均匀分布)
- [ ] 跨服务一致性通过Saga/事件保证
- [ ] 冲突检测和解决策略已定义
- [ ] 时钟偏差已考虑

### 可观测性
- [ ] 分布式追踪贯穿全链路
- [ ] 一致性检查有监控
- [ ] 分片负载有监控
- [ ] 网络延迟/分区有告警
