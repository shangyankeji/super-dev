---
id: performance-optimization-playbook
title: 性能优化作战手册 (Performance Optimization Playbook)
domain: development
category: 02-playbooks
difficulty: intermediate
tags: [agent, checklist, development, optimization, performance, playbook, 前置条件, 回滚方案]
quality_score: 70
last_updated: 2026-06-15
---
# 性能优化作战手册 (Performance Optimization Playbook)

## 概述

性能优化是一个科学的迭代过程：度量 -> 分析 -> 定位 -> 优化 -> 验证。本手册涵盖从系统层到应用层的完整优化方法论，包括 CPU、内存、IO、网络、数据库各维度的瓶颈定位和优化策略。核心原则：不度量就不优化，不验证就不上线。

## 前置条件

### 必须满足

- [ ] 已定义明确的性能目标（SLA/SLO）：响应时间、吞吐量、并发量
- [ ] 具备可观测性基础设施：指标采集（Prometheus）、日志（ELK）、追踪（Jaeger/Zipkin）
- [ ] 有可重复的性能测试环境（与生产配置一致或可对比换算）
- [ ] 有性能基线数据作为对比基准
- [ ] 已排除功能性 Bug（性能问题与功能缺陷要分开处理）

### 建议满足

- [ ] 有 APM 工具（如 Datadog、New Relic、SkyWalking）
- [ ] 有持续的性能回归测试机制
- [ ] 性能预算已定义（页面大小、加载时间、API 响应时间）

---

## 步骤一：建立性能基线

### 1.1 指标采集

```bash
# API 性能基线 - 使用 wrk 进行基准测试
wrk -t4 -c100 -d60s --latency http://localhost:8080/api/v1/target-endpoint \
  > baseline/api_performance.txt

# 输出示例：
# Latency Distribution
#   50%   12.3ms
#   75%   18.5ms
#   90%   25.1ms
#   99%   48.7ms

# 更精细的压测 - 使用 k6
cat > load_test.js << 'EOF'
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const latencyTrend = new Trend('latency');

export const options = {
  stages: [
    { duration: '1m', target: 50 },   // 预热
    { duration: '3m', target: 100 },   // 正常负载
    { duration: '2m', target: 200 },   // 峰值负载
    { duration: '1m', target: 0 },     // 降载
  ],
  thresholds: {
    http_req_duration: ['p(95)<200', 'p(99)<500'],
    errors: ['rate<0.01'],
  },
};

export default function () {
  const res = http.get('http://localhost:8080/api/v1/target');
  check(res, { 'status is 200': (r) => r.status === 200 });
  errorRate.add(res.status !== 200);
  latencyTrend.add(res.timings.duration);
  sleep(1);
}
EOF
k6 run load_test.js --out json=baseline/k6_results.json
```

### 1.2 系统资源基线

```bash
# CPU 和内存基线
vmstat 1 60 > baseline/vmstat.txt

# IO 基线
iostat -x 1 60 > baseline/iostat.txt

# 网络基线
sar -n DEV 1 60 > baseline/network.txt

# 进程级别资源监控
pidstat -p $(pgrep -f "your-app") -r -u -d 1 60 > baseline/process_stats.txt
```

### 1.3 数据库基线

```sql
-- PostgreSQL 慢查询统计
SELECT query,
       calls,
       mean_exec_time,
       total_exec_time,
       rows
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 20;

-- 索引使用率
SELECT schemaname, tablename, indexname,
       idx_scan, idx_tup_read, idx_tup_fetch
FROM pg_stat_user_indexes
ORDER BY idx_scan ASC
LIMIT 20;

-- 表膨胀检查
SELECT schemaname, tablename,
       pg_size_pretty(pg_total_relation_size(schemaname || '.' || tablename)) as total_size,
       n_dead_tup,
       n_live_tup,
       ROUND(n_dead_tup::numeric / NULLIF(n_live_tup, 0) * 100, 2) as dead_ratio
FROM pg_stat_user_tables
ORDER BY n_dead_tup DESC
LIMIT 20;
```

---

## 步骤二：瓶颈定位

### 2.1 自顶向下分析法

```
系统层面（宏观）
├── CPU 利用率 > 80%？ → CPU 瓶颈分析
├── 内存使用率 > 85%？ → 内存瓶颈分析
├── IO Wait > 20%？ → IO 瓶颈分析
├── 网络带宽饱和？ → 网络瓶颈分析
└── 以上均正常？ → 应用层分析

应用层面（微观）
├── 请求排队时间长？ → 线程池/连接池调优
├── 某个 Span 耗时突出？ → 定位具体方法
├── 数据库查询慢？ → 查询优化
├── 外部调用慢？ → 超时/缓存/异步化
└── GC 频繁？ → 内存模型调优
```

### 2.2 CPU 瓶颈分析

```bash
# Linux perf - 采集 CPU 火焰图数据
perf record -g -p $(pgrep -f "your-app") -- sleep 30
perf script > perf.out
# 使用 FlameGraph 工具生成火焰图
stackcollapse-perf.pl perf.out | flamegraph.pl > cpu_flame.svg

# Python 应用 - 使用 py-spy
py-spy record -o profile.svg --pid $(pgrep -f "your-app") --duration 30

# Python 应用 - 使用 cProfile
python -m cProfile -o profile.prof your_script.py
# 分析结果
python -c "
import pstats
p = pstats.Stats('profile.prof')
p.sort_stats('cumulative').print_stats(30)
"

# Node.js 应用 - 内置 profiler
node --prof app.js
node --prof-process isolate-*.log > processed_profile.txt
```

### 2.3 内存瓶颈分析

```python
# Python 内存分析 - tracemalloc
import tracemalloc

tracemalloc.start()

# ... 运行目标代码 ...

snapshot = tracemalloc.take_snapshot()
top_stats = snapshot.statistics('lineno')
print("Top 20 memory allocations:")
for stat in top_stats[:20]:
    print(stat)

# Python 内存分析 - memory_profiler
# pip install memory_profiler
# 在目标函数上添加 @profile 装饰器，然后运行:
# python -m memory_profiler your_script.py

# 对象引用分析 - objgraph
import objgraph
objgraph.show_most_common_types(limit=20)
objgraph.show_growth(limit=10)  # 调用两次，对比增长
```

```bash
# Java 堆分析
jmap -histo:live $(pgrep -f "your-app") | head -30
jmap -dump:live,format=b,file=heap.hprof $(pgrep -f "your-app")
# 使用 Eclipse MAT 或 VisualVM 分析 heap.hprof

# Go 内存 pprof
curl http://localhost:6060/debug/pprof/heap > heap.prof
go tool pprof -http=:8081 heap.prof
```

### 2.4 IO 瓶颈分析

```bash
# 查看 IO 等待
iostat -x 1 10

# 查看哪个进程占用 IO
iotop -o -b -n 5

# 文件系统级别追踪
strace -e trace=read,write,open -p $(pgrep -f "your-app") -c -S time
```

### 2.5 数据库瓶颈分析

```sql
-- PostgreSQL 查询执行计划
EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)
SELECT * FROM orders
WHERE user_id = 12345
  AND status = 'active'
ORDER BY created_at DESC
LIMIT 20;

-- 查找缺失的索引
SELECT relname AS table,
       seq_scan,
       seq_tup_read,
       idx_scan,
       ROUND(seq_scan::numeric / NULLIF(seq_scan + idx_scan, 0) * 100, 2) AS seq_scan_pct
FROM pg_stat_user_tables
WHERE seq_scan > 1000
ORDER BY seq_tup_read DESC
LIMIT 20;

-- 活跃锁检查
SELECT pid, mode, relation::regclass, granted,
       now() - query_start AS duration, query
FROM pg_locks
JOIN pg_stat_activity USING (pid)
WHERE NOT granted
ORDER BY query_start;
```

---

## 步骤三：优化策略

### 3.1 数据库优化

```sql
-- 添加复合索引（覆盖高频查询）
CREATE INDEX CONCURRENTLY idx_orders_user_status_created
ON orders(user_id, status, created_at DESC);

-- 分区表（大数据量场景）
CREATE TABLE orders_partitioned (
    LIKE orders INCLUDING ALL
) PARTITION BY RANGE (created_at);

CREATE TABLE orders_2024_q1 PARTITION OF orders_partitioned
    FOR VALUES FROM ('2024-01-01') TO ('2024-04-01');

-- 查询优化 - 避免 SELECT *
-- 优化前
SELECT * FROM orders WHERE user_id = 123;
-- 优化后
SELECT id, status, total, created_at FROM orders WHERE user_id = 123;

-- 批量操作优化 - 避免 N+1
-- 优化前（N+1）：循环中逐条查询
-- 优化后：批量查询
SELECT * FROM order_items WHERE order_id IN (1, 2, 3, 4, 5);
```

### 3.2 缓存策略

```python
import redis
import json
import hashlib
from functools import wraps

redis_client = redis.Redis(host='localhost', port=6379, decode_responses=True)

def cache_with_ttl(prefix: str, ttl: int = 300):
    """通用缓存装饰器"""
    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            # 生成缓存 key
            key_data = f"{prefix}:{args}:{sorted(kwargs.items())}"
            cache_key = f"{prefix}:{hashlib.md5(key_data.encode()).hexdigest()}"

            # 尝试读缓存
            cached = redis_client.get(cache_key)
            if cached:
                return json.loads(cached)

            # 执行原函数
            result = func(*args, **kwargs)

            # 写缓存
            redis_client.setex(cache_key, ttl, json.dumps(result, default=str))
            return result
        return wrapper
    return decorator

# 缓存失效策略
class CacheInvalidator:
    @staticmethod
    def invalidate_pattern(pattern: str):
        """按模式批量删除缓存"""
        cursor = 0
        while True:
            cursor, keys = redis_client.scan(cursor, match=pattern, count=100)
            if keys:
                redis_client.delete(*keys)
            if cursor == 0:
                break

    @staticmethod
    def invalidate_on_write(entity: str, entity_id: int):
        """写操作时主动失效相关缓存"""
        patterns = [
            f"{entity}:{entity_id}:*",
            f"{entity}:list:*",
        ]
        for pattern in patterns:
            CacheInvalidator.invalidate_pattern(pattern)
```

### 3.3 异步化

```python
import asyncio
from concurrent.futures import ThreadPoolExecutor

# 将 CPU 密集型任务移到线程池
executor = ThreadPoolExecutor(max_workers=4)

async def process_request(data):
    # IO 密集型 - 直接 await
    user = await fetch_user(data['user_id'])

    # CPU 密集型 - 放到线程池
    loop = asyncio.get_event_loop()
    result = await loop.run_in_executor(
        executor, cpu_intensive_calculation, data
    )

    # 并行执行多个 IO 操作
    notifications, audit_log = await asyncio.gather(
        send_notification(user, result),
        write_audit_log(data, result),
    )

    return result
```

### 3.4 连接池调优

```python
# SQLAlchemy 连接池配置
from sqlalchemy import create_engine

engine = create_engine(
    "postgresql://user:pass@host/db",
    pool_size=20,           # 常驻连接数
    max_overflow=10,        # 额外突发连接数
    pool_timeout=30,        # 获取连接超时（秒）
    pool_recycle=1800,      # 连接回收时间（秒）
    pool_pre_ping=True,     # 使用前检测连接是否有效
)

# Redis 连接池
import redis

pool = redis.ConnectionPool(
    host='localhost',
    port=6379,
    max_connections=50,
    socket_timeout=5,
    socket_connect_timeout=5,
    retry_on_timeout=True,
)
redis_client = redis.Redis(connection_pool=pool)
```

### 3.5 序列化优化

```python
# JSON 替代方案对比
import json
import orjson      # 更快的 JSON 库
import msgpack     # 二进制序列化

data = {"users": [{"id": i, "name": f"user_{i}"} for i in range(1000)]}

# orjson - 比标准 json 快 3-10 倍
result = orjson.dumps(data)

# msgpack - 体积更小，速度更快
result = msgpack.packb(data, use_bin_type=True)

# 响应压缩
# 在 Web 框架中启用 gzip/brotli 压缩
# FastAPI 示例
from fastapi.middleware.gzip import GZipMiddleware
app.add_middleware(GZipMiddleware, minimum_size=1000)
```

---

## 步骤四：验证

### 4.1 对比测试

```bash
# 使用相同的负载条件重新测试
k6 run load_test.js --out json=optimized/k6_results.json

# 对比结果
python3 << 'EOF'
import json

def load_k6_summary(path):
    with open(path) as f:
        data = json.load(f)
    return data

baseline = load_k6_summary("baseline/k6_results.json")
optimized = load_k6_summary("optimized/k6_results.json")

print("=== 性能对比 ===")
print(f"P95 延迟: {baseline['p95']}ms -> {optimized['p95']}ms")
print(f"P99 延迟: {baseline['p99']}ms -> {optimized['p99']}ms")
print(f"吞吐量: {baseline['rps']} -> {optimized['rps']} req/s")
print(f"错误率: {baseline['error_rate']}% -> {optimized['error_rate']}%")
EOF
```

### 4.2 回归验证

```bash
# 确保优化没有引入功能回归
pytest tests/ -v --tb=short

# 确保优化没有引入内存泄漏
# 长时间运行测试
k6 run --duration 30m load_test.js

# 期间监控内存趋势
while true; do
  ps -o pid,rss,vsz -p $(pgrep -f "your-app") >> memory_trend.log
  sleep 10
done
```

### 4.3 性能回归门禁

```yaml
# CI 中的性能门禁配置
performance_gate:
  api_p95_latency_ms: 200
  api_p99_latency_ms: 500
  api_error_rate_pct: 0.1
  api_throughput_rps: 500
  regression_tolerance_pct: 10  # 允许 10% 的波动
```

---

## 回滚方案

### 优化回滚

```bash
# 如果优化引入了问题
git revert <optimization-commit>

# 如果添加了新索引导致写入变慢
DROP INDEX CONCURRENTLY idx_orders_user_status_created;

# 如果缓存导致数据不一致
redis-cli FLUSHDB  # 清空缓存（慎用，仅在紧急情况下）
```

### 回滚触发条件

| 指标 | 阈值 | 动作 |
|------|------|------|
| 错误率上升 | > 优化前 + 0.1% | 回滚优化 |
| P99 延迟增加 | > 优化前 1.5 倍 | 回滚优化 |
| 内存持续增长 | 30 分钟内增长 > 20% | 回滚并排查泄漏 |
| CPU 利用率增加 | > 优化前 1.3 倍 | 回滚并分析 |

---

## Agent Checklist

AI 编码 Agent 在执行性能优化时必须逐项确认：

- [ ] **目标明确**：有具体的性能 SLO（如 P99 < 200ms），不是模糊的"变快"
- [ ] **基线已建立**：优化前有可量化的性能数据
- [ ] **瓶颈已定位**：通过 profiling/tracing 确认了具体瓶颈，不是猜测
- [ ] **优化有针对性**：优化措施直接针对定位到的瓶颈
- [ ] **逐项优化**：每次只做一项优化，单独度量效果
- [ ] **无功能回归**：所有功能测试通过
- [ ] **无新瓶颈**：优化不能引入新的性能问题
- [ ] **无内存泄漏**：长时间运行测试确认内存稳定
- [ ] **效果可量化**：有优化前后的对比数据
- [ ] **门禁已设置**：性能回归门禁已加入 CI
- [ ] **文档已更新**：优化决策和效果记录在案
- [ ] **监控已覆盖**：优化相关的指标有持续监控
