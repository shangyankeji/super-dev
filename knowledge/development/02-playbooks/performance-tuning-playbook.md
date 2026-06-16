---
id: performance-tuning-playbook
title: 性能调优作战手册 (Performance Tuning Playbook)
domain: development
category: 02-playbooks
difficulty: intermediate
tags: [agent, checklist, development, performance, playbook, tuning, 前置条件, 回滚方案]
quality_score: 70
last_updated: 2026-06-15
---
# 性能调优作战手册 (Performance Tuning Playbook)

## 概述

性能调优是针对已识别的具体性能瓶颈，通过系统化的 分析 -> 定位 -> 优化 -> 验证 工作流进行精确优化的过程。与性能优化手册侧重整体方法论不同，本手册聚焦于具体工具的使用方法、火焰图分析、profiling 技术和基准测试实践。

## 前置条件

### 必须满足

- [ ] 已有明确的性能问题描述（哪个接口、什么场景、多慢）
- [ ] 已有可复现的性能问题环境（生产镜像或压测环境）
- [ ] 已有性能基线数据（正常时的指标）
- [ ] profiling 工具已安装并可用
- [ ] 有足够的磁盘空间存储 profiling 数据（建议 > 10GB）

### 工具清单

| 维度 | 工具 | 语言/平台 |
|------|------|----------|
| CPU Profiling | py-spy, cProfile, perf, async-profiler | Python, 系统级, Java |
| Memory Profiling | tracemalloc, memory_profiler, objgraph, MAT | Python, Java |
| 火焰图 | FlameGraph, speedscope, py-spy | 通用 |
| 基准测试 | pytest-benchmark, wrk, k6, ab | Python, HTTP |
| 数据库 | EXPLAIN ANALYZE, pg_stat_statements, slow_query_log | PostgreSQL, MySQL |
| 网络 | tcpdump, wireshark, mtr | 系统级 |
| 综合 APM | Datadog, New Relic, SkyWalking, Jaeger | 通用 |

---

## 步骤一：分析 — 收集性能数据

### 1.1 CPU Profiling

#### Python - py-spy（推荐，低开销，可附加到运行中的进程）

```bash
# 安装
pip install py-spy

# 实时 top 视图 - 查看当前 CPU 热点
py-spy top --pid $(pgrep -f "uvicorn")

# 生成火焰图 SVG
py-spy record -o flame_cpu.svg --pid $(pgrep -f "uvicorn") --duration 30

# 生成 speedscope 格式（交互式分析）
py-spy record -o profile.speedscope --format speedscope --pid $(pgrep -f "uvicorn") --duration 30
# 然后在浏览器打开 https://www.speedscope.app/ 导入

# 采样子进程（适用于 multiprocessing）
py-spy record -o flame.svg --pid $(pgrep -f "uvicorn") --subprocesses --duration 60

# 只采集 GIL 持有者（找到真正占 CPU 的代码）
py-spy record -o flame_gil.svg --pid $(pgrep -f "uvicorn") --gil --duration 30
```

#### Python - cProfile（内置，适用于脚本级分析）

```python
import cProfile
import pstats
from io import StringIO

def profile_function(func, *args, **kwargs):
    """Profile 一个函数并输出统计"""
    profiler = cProfile.Profile()
    profiler.enable()
    result = func(*args, **kwargs)
    profiler.disable()

    # 输出统计
    stream = StringIO()
    stats = pstats.Stats(profiler, stream=stream)
    stats.sort_stats('cumulative')
    stats.print_stats(30)  # Top 30
    print(stream.getvalue())

    # 保存为可分析文件
    stats.dump_stats('profile_output.prof')

    return result

# 使用 snakeviz 可视化
# pip install snakeviz
# snakeviz profile_output.prof
```

#### Python - line_profiler（逐行分析）

```python
# pip install line_profiler

# 方式 1: 装饰器
from line_profiler import profile

@profile
def slow_function(data):
    result = []
    for item in data:               # 看每行耗时
        processed = transform(item)  # 哪行最慢一目了然
        result.append(processed)
    return result

# 运行: kernprof -l -v script.py
```

#### 系统级 - Linux perf

```bash
# 采集 CPU 调用栈（30秒）
sudo perf record -g -p $(pgrep -f "your-app") -- sleep 30

# 生成报告
sudo perf report --stdio --sort=dso,sym | head -50

# 生成火焰图
sudo perf script > perf.unfold
stackcollapse-perf.pl perf.unfold | flamegraph.pl > perf_flame.svg
```

### 1.2 Memory Profiling

#### Python - tracemalloc（内置，推荐）

```python
import tracemalloc
import linecache

def memory_snapshot(label: str = ""):
    """拍摄内存快照并展示 Top 分配"""
    snapshot = tracemalloc.take_snapshot()
    top_stats = snapshot.statistics('lineno')

    print(f"\n=== Memory Snapshot: {label} ===")
    print(f"Top 15 memory allocations:")
    for index, stat in enumerate(top_stats[:15], 1):
        frame = stat.traceback[0]
        print(f"  #{index}: {frame.filename}:{frame.lineno} - {stat.size / 1024:.1f} KB")
        line = linecache.getline(frame.filename, frame.lineno).strip()
        if line:
            print(f"         {line}")

def compare_snapshots(snapshot1, snapshot2):
    """比较两个快照，找出内存增长点"""
    stats = snapshot2.compare_to(snapshot1, 'lineno')
    print("\n=== Memory Growth ===")
    for stat in stats[:10]:
        print(f"  {stat}")

# 使用示例
tracemalloc.start()
snapshot1 = tracemalloc.take_snapshot()

# ... 运行目标代码 ...

snapshot2 = tracemalloc.take_snapshot()
compare_snapshots(snapshot1, snapshot2)
```

#### Python - objgraph（对象引用分析，排查泄漏）

```python
import objgraph

# 查看对象数量变化（执行两次对比）
objgraph.show_growth(limit=10)

# 找到特定类型的引用链（排查为什么对象没被 GC）
objgraph.show_backrefs(
    objgraph.by_type('MyClass')[:3],
    max_depth=5,
    filename='refs.png'
)

# 查看最多的对象类型
objgraph.show_most_common_types(limit=20)
```

### 1.3 IO 与网络 Profiling

```bash
# 查看进程的系统调用耗时分布
strace -c -p $(pgrep -f "your-app") -e trace=network,file 2>&1 | head -30

# 查看 TCP 连接状态
ss -tnp | grep $(pgrep -f "your-app") | awk '{print $1}' | sort | uniq -c | sort -rn

# 抓包分析慢请求
tcpdump -i eth0 -w capture.pcap port 8080 -c 10000
# 用 wireshark 分析 capture.pcap

# DNS 解析延迟
dig @8.8.8.8 api.external-service.com +stats
```

### 1.4 数据库 Profiling

```sql
-- PostgreSQL: 启用 pg_stat_statements
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- 查看最慢的查询
SELECT
    LEFT(query, 100) AS short_query,
    calls,
    ROUND(mean_exec_time::numeric, 2) AS avg_ms,
    ROUND(total_exec_time::numeric, 2) AS total_ms,
    rows,
    ROUND((shared_blks_hit::numeric / NULLIF(shared_blks_hit + shared_blks_read, 0) * 100), 2) AS cache_hit_pct
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 20;

-- 详细执行计划
EXPLAIN (ANALYZE, BUFFERS, TIMING, VERBOSE, FORMAT JSON)
SELECT o.id, o.status, u.name
FROM orders o
JOIN users u ON u.id = o.user_id
WHERE o.created_at > NOW() - INTERVAL '7 days'
  AND o.status = 'active'
ORDER BY o.created_at DESC
LIMIT 50;

-- MySQL: 启用慢查询日志
SET GLOBAL slow_query_log = 'ON';
SET GLOBAL long_query_time = 0.1;  -- 100ms 以上记为慢查询
SET GLOBAL log_queries_not_using_indexes = 'ON';

-- 分析慢查询日志
-- mysqldumpslow -s t /var/log/mysql/slow.log | head -20
```

---

## 步骤二：定位 — 火焰图分析

### 2.1 火焰图阅读方法

```
火焰图解读规则：

1. X 轴：函数调用栈的聚合宽度，代表 CPU 时间占比（越宽越耗 CPU）
2. Y 轴：调用栈深度，底部是入口函数，顶部是叶子函数
3. 颜色：无特殊含义（仅用于区分不同栈帧）

重点关注：
- "平顶山"（Plateau）：顶部的宽方块 = CPU 热点函数
- "悬崖"（Cliff）：调用栈突然变窄 = 该函数消耗了大量 CPU 而非子调用

操作步骤：
1. 先找最宽的顶部方块（最大的 CPU 消耗者）
2. 向下追溯调用路径（谁调用了它）
3. 判断是否合理（必要的计算 vs 可优化的逻辑）
```

### 2.2 不同类型火焰图

```bash
# CPU 火焰图 - 分析 CPU 密集操作
py-spy record -o cpu_flame.svg --pid $PID --duration 30

# Off-CPU 火焰图 - 分析等待/阻塞（IO、锁、sleep）
# 需要 BPF 工具
sudo offcputime-bpfcc -p $PID -f 30 > offcpu.stacks
flamegraph.pl --color=io < offcpu.stacks > offcpu_flame.svg

# 内存火焰图 - 分析内存分配热点
# 使用 tracemalloc 的结果生成
python3 << 'PYEOF'
import tracemalloc
import json

tracemalloc.start(25)  # 保留 25 层调用栈

# ... 运行目标代码 ...

snapshot = tracemalloc.take_snapshot()
stats = snapshot.statistics('traceback')

# 输出 flamegraph 格式
with open('mem.stacks', 'w') as f:
    for stat in stats[:200]:
        frames = [f"{frame.filename}:{frame.lineno}" for frame in reversed(stat.traceback)]
        f.write(f"{';'.join(frames)} {stat.size}\n")
PYEOF
flamegraph.pl --title="Memory Allocation" --countname="bytes" < mem.stacks > mem_flame.svg

# 差分火焰图 - 对比优化前后
difffolded.pl before.folded after.folded | flamegraph.pl > diff_flame.svg
# 红色 = 回退（变慢）, 蓝色 = 改进（变快）
```

### 2.3 speedscope 交互式分析

```bash
# speedscope 提供三种视图：
# 1. Time Order - 按时间顺序查看（适合分析请求处理流程）
# 2. Left Heavy - 按耗时汇总排序（适合找热点函数）
# 3. Sandwich - 查看函数的调用者和被调用者（适合分析函数上下文）

# 生成 speedscope 兼容格式
py-spy record -o profile.speedscope --format speedscope --pid $PID --duration 30

# 在浏览器中打开 https://www.speedscope.app/ 加载 profile.speedscope
```

---

## 步骤三：优化 — 常见瓶颈修复

### 3.1 CPU 密集型优化

```python
# 问题 1: 循环中的重复计算
# 优化前
def process_items(items):
    for item in items:
        config = load_config()  # 每次循环都重新加载！
        result = expensive_transform(item, config)

# 优化后
def process_items(items):
    config = load_config()  # 提到循环外
    for item in items:
        result = expensive_transform(item, config)

# 问题 2: 低效的数据结构
# 优化前 - O(n) 查找
def find_user(users_list, user_id):
    for user in users_list:
        if user['id'] == user_id:
            return user

# 优化后 - O(1) 查找
users_dict = {u['id']: u for u in users_list}
def find_user(users_dict, user_id):
    return users_dict.get(user_id)

# 问题 3: 字符串拼接
# 优化前 - O(n^2) 复杂度
result = ""
for line in lines:
    result += line + "\n"

# 优化后 - O(n) 复杂度
result = "\n".join(lines)

# 问题 4: 使用 C 扩展替代纯 Python
import orjson  # 比 json 快 3-10x
data = orjson.dumps({"key": "value"})

import numpy as np  # 比纯 Python 循环快 100x+
result = np.sum(large_array)
```

### 3.2 IO 密集型优化

```python
import asyncio
import aiohttp

# 问题: 串行 IO 请求
# 优化前
async def fetch_all_serial(urls):
    results = []
    async with aiohttp.ClientSession() as session:
        for url in urls:
            async with session.get(url) as resp:
                results.append(await resp.json())
    return results

# 优化后: 并发 IO
async def fetch_all_concurrent(urls, max_concurrency=10):
    semaphore = asyncio.Semaphore(max_concurrency)
    async with aiohttp.ClientSession() as session:
        async def fetch(url):
            async with semaphore:
                async with session.get(url) as resp:
                    return await resp.json()
        return await asyncio.gather(*[fetch(url) for url in urls])

# 问题: 频繁小 IO
# 优化前
for record in records:
    db.insert(record)  # 1000 次网络往返

# 优化后: 批量 IO
db.insert_many(records)  # 1 次网络往返
```

### 3.3 内存优化

```python
# 问题 1: 一次性加载大数据集
# 优化前
data = list(db.query("SELECT * FROM large_table"))  # 全部加载到内存

# 优化后: 流式处理
def stream_query(query, batch_size=1000):
    offset = 0
    while True:
        batch = db.query(f"{query} LIMIT {batch_size} OFFSET {offset}")
        if not batch:
            break
        yield from batch
        offset += batch_size

for record in stream_query("SELECT * FROM large_table"):
    process(record)

# 问题 2: 对象开销大
# 优化前
class Point:
    def __init__(self, x, y, z):
        self.x = x  # 每个实例有 __dict__，占用约 200+ bytes
        self.y = y
        self.z = z

# 优化后: 使用 __slots__
class Point:
    __slots__ = ('x', 'y', 'z')  # 减少到约 64 bytes
    def __init__(self, x, y, z):
        self.x = x
        self.y = y
        self.z = z

# 或使用 NamedTuple（不可变场景）
from typing import NamedTuple
class Point(NamedTuple):
    x: float
    y: float
    z: float
```

### 3.4 数据库查询优化

```sql
-- 问题 1: 全表扫描
-- 优化前
SELECT * FROM orders WHERE DATE(created_at) = '2024-01-15';
-- 优化后（利用索引）
SELECT * FROM orders
WHERE created_at >= '2024-01-15 00:00:00'
  AND created_at < '2024-01-16 00:00:00';

-- 问题 2: N+1 查询
-- 优化前（应用层 N+1）
-- for order in orders: order.items = query("SELECT * FROM items WHERE order_id = ?", order.id)
-- 优化后: JOIN 或 子查询
SELECT o.*, i.product_name, i.quantity
FROM orders o
LEFT JOIN order_items i ON i.order_id = o.id
WHERE o.user_id = 123;

-- 问题 3: 排序时没有索引支持
-- 添加覆盖查询的复合索引
CREATE INDEX CONCURRENTLY idx_orders_user_status_created
ON orders(user_id, status, created_at DESC)
INCLUDE (total_amount);  -- 覆盖索引，避免回表
```

---

## 步骤四：验证 — 基准测试

### 4.1 微基准测试

```python
# pytest-benchmark
import pytest

def test_json_serialization(benchmark):
    data = {"users": [{"id": i, "name": f"user_{i}"} for i in range(100)]}

    import json
    result = benchmark(json.dumps, data)
    assert result

def test_orjson_serialization(benchmark):
    data = {"users": [{"id": i, "name": f"user_{i}"} for i in range(100)]}

    import orjson
    result = benchmark(orjson.dumps, data)
    assert result

# 运行: pytest tests/bench/ --benchmark-sort=mean --benchmark-compare
```

```python
# timeit - 快速对比
import timeit

# 对比两种实现
setup = "data = list(range(10000))"

time_list_comp = timeit.timeit("[x**2 for x in data]", setup=setup, number=1000)
time_map = timeit.timeit("list(map(lambda x: x**2, data))", setup=setup, number=1000)

print(f"List comprehension: {time_list_comp:.3f}s")
print(f"Map: {time_map:.3f}s")
print(f"Speedup: {time_map/time_list_comp:.2f}x")
```

### 4.2 负载测试

```bash
# k6 负载测试 - 阶梯式加压
cat > soak_test.js << 'EOF'
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '5m', target: 100 },   // 预热到 100 并发
    { duration: '30m', target: 100 },   // 持续 100 并发 30 分钟
    { duration: '5m', target: 200 },    // 加压到 200
    { duration: '30m', target: 200 },   // 持续 200 并发 30 分钟
    { duration: '5m', target: 0 },      // 降载
  ],
  thresholds: {
    http_req_duration: ['p(95)<200', 'p(99)<500'],
    http_req_failed: ['rate<0.01'],
  },
};

export default function () {
  const res = http.get(`${__ENV.BASE_URL}/api/v1/orders?page=1&per_page=20`);
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 200ms': (r) => r.timings.duration < 200,
  });
  sleep(Math.random() * 2);  // 模拟用户思考时间
}
EOF
k6 run soak_test.js -e BASE_URL=http://localhost:8080
```

### 4.3 A/B 对比验证

```bash
# 使用 wrk 进行优化前后对比

echo "=== 优化前 ==="
wrk -t4 -c100 -d30s --latency http://localhost:8080/api/v1/orders > before.txt 2>&1

echo "=== 部署优化后的代码 ==="
# ... 部署 ...

echo "=== 优化后 ==="
wrk -t4 -c100 -d30s --latency http://localhost:8080/api/v1/orders > after.txt 2>&1

echo "=== 对比 ==="
diff before.txt after.txt
```

### 4.4 持续性能回归

```yaml
# GitHub Actions 中的性能回归检测
- name: Run performance benchmark
  run: |
    pytest tests/bench/ --benchmark-json=benchmark.json

- name: Compare with baseline
  run: |
    python3 << 'EOF'
    import json
    with open('benchmark.json') as f:
        current = json.load(f)
    with open('benchmark_baseline.json') as f:
        baseline = json.load(f)

    regressions = []
    for bench in current['benchmarks']:
        name = bench['name']
        baseline_bench = next((b for b in baseline['benchmarks'] if b['name'] == name), None)
        if baseline_bench:
            ratio = bench['stats']['mean'] / baseline_bench['stats']['mean']
            if ratio > 1.1:  # 超过 10% 回退
                regressions.append(f"{name}: {ratio:.2f}x slower")

    if regressions:
        print("Performance regressions detected:")
        for r in regressions:
            print(f"  - {r}")
        exit(1)
    print("No regressions detected.")
    EOF
```

---

## 回滚方案

### 优化代码回滚

```bash
# 如果优化引入了 Bug 或性能反而下降
git revert <optimization-commit-hash>
git push origin main
```

### 配置调优回滚

```bash
# 连接池/缓存/限流等配置回滚
kubectl rollout undo deployment/<service> -n production

# 或通过配置中心回滚
curl -X PUT http://config-center/api/config/<service> \
  -d '{"pool_size": 10, "cache_ttl": 300}'  # 旧配置
```

### 回滚触发条件

| 指标 | 触发条件 | 动作 |
|------|---------|------|
| 错误率 | 优化后上升 > 0.1% | 立即回滚 |
| P99 延迟 | 优化后上升 > 20% | 回滚并分析 |
| 内存使用 | 30 分钟持续增长 | 回滚并排查泄漏 |
| CPU 使用 | 优化后上升 > 30% | 回滚并分析 |
| 基准测试 | 回退 > 10% | 阻断合并 |

---

## Agent Checklist

AI 编码 Agent 在执行性能调优时必须逐项确认：

- [ ] **问题可复现**：有具体的复现步骤和量化的性能数据
- [ ] **基线已记录**：优化前有 profiling 数据和基准测试结果
- [ ] **瓶颈已确认**：通过火焰图/profiling 确认了 CPU/内存/IO 的具体热点
- [ ] **优化有针对性**：每次优化只针对一个瓶颈点
- [ ] **微基准测试**：优化的函数/方法有独立的基准测试
- [ ] **负载测试**：在接近生产的负载下验证优化效果
- [ ] **无功能回归**：所有功能测试通过
- [ ] **无内存泄漏**：长时间运行（> 30 分钟）内存稳定
- [ ] **效果可量化**：有优化前后的精确对比数据（延迟、吞吐量、资源使用）
- [ ] **差分火焰图**：生成了优化前后的差分火焰图确认改进
- [ ] **回归门禁**：性能基准测试已加入 CI
- [ ] **文档记录**：优化决策、工具使用、效果数据已记录
