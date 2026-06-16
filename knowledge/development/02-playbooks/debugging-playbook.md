---
id: debugging-playbook
title: 系统化调试手册 (Debugging Playbook)
domain: development
category: 02-playbooks
difficulty: intermediate
tags: [agent, checklist, debugging, development, playbook, 前置条件, 回滚方案, 报告]
quality_score: 70
last_updated: 2026-06-15
---
# 系统化调试手册 (Debugging Playbook)

## 概述

调试是通过系统化方法定位和修复软件缺陷的过程。本手册覆盖从日志分析、断点调试、分布式链路追踪到内存泄漏排查的完整方法论，强调科学假设 -> 验证 -> 排除的迭代流程，避免盲目猜测。

## 前置条件

### 必须满足

- [ ] 能复现问题（或有足够的日志/监控数据还原现场）
- [ ] 有可观测性基础设施（日志、指标、追踪至少具其二）
- [ ] 有访问相关系统日志和监控数据的权限
- [ ] 了解系统架构和关键组件的交互关系

### 工具清单

| 场景 | 工具 |
|------|------|
| 日志分析 | ELK Stack, Loki+Grafana, CloudWatch |
| 断点调试 | pdb/ipdb (Python), delve (Go), gdb/lldb (C/C++) |
| 远程调试 | debugpy (Python), dlv attach (Go) |
| 分布式追踪 | Jaeger, Zipkin, SkyWalking, Datadog APM |
| 网络调试 | tcpdump, wireshark, curl -v, httpie |
| 内存分析 | tracemalloc, objgraph, valgrind, MAT |
| 系统级 | strace, ltrace, dmesg, journalctl |

---

## 步骤一：问题定义

### 1.1 问题描述模板

```markdown
## Bug 报告

### 现象
- 什么：[具体的错误表现]
- 何时：[首次出现时间、出现频率]
- 哪里：[哪个环境、哪个接口、哪个页面]
- 谁受影响：[用户群体、影响范围]

### 上下文
- 最近有发布/变更吗？
- 最近有流量/数据量变化吗？
- 最近有基础设施变更吗？
- 是否只在特定条件下出现？

### 已有信息
- 错误日志：[粘贴关键日志]
- 错误码/消息：[具体的错误信息]
- 复现步骤：[1, 2, 3...]
- 已尝试的排查：[做过什么]

### 影响
- 业务影响：[具体影响]
- 紧急程度：P0/P1/P2/P3
```

### 1.2 调试思维模型

```
科学方法应用于调试：

1. 观察现象 → 收集所有相关信息
2. 形成假设 → 基于现象推测可能的原因
3. 设计实验 → 制定验证假设的步骤
4. 执行验证 → 运行实验
5. 分析结果 → 假设成立？排除？修正？
6. 重复 2-5   → 直到定位根因

关键原则：
- 一次只改一个变量
- 记录每个假设和验证结果
- 先排除最可能的原因
- 不要假设，要验证
```

---

## 步骤二：日志分析

### 2.1 结构化日志查询

```bash
# ELK - Kibana Query Language (KQL)
# 按时间范围和错误级别过滤
level:ERROR AND service:"order-service" AND @timestamp >= "2024-01-15T10:00:00"

# 按请求 ID 追踪
request_id:"req_abc123def456"

# 按用户追踪
user_id:12345 AND level:(ERROR OR WARN)

# 按错误类型统计
level:ERROR | stats count() by error_code
```

```bash
# Loki (Grafana) - LogQL
# 按标签过滤
{service="order-service", level="error"}

# 正则匹配
{service="order-service"} |~ "timeout|connection refused"

# 按模式统计
{service="order-service"} | pattern `<_> ERROR <_> <error_type> <_>` | line_format "{{.error_type}}" | label_format error_type
```

```bash
# 命令行快速分析（无 ELK 时）
# 查看最近的错误日志
kubectl logs -n production deployment/order-service --since=30m | grep -i error | tail -50

# 多 Pod 聚合查看
stern order-service -n production --since 30m -o raw | grep -E "(ERROR|FATAL|Traceback)"

# 按错误类型统计频率
kubectl logs -n production deployment/order-service --since=1h | \
  grep ERROR | \
  sed 's/.*ERROR *//' | \
  sed 's/\[.*$//' | \
  sort | uniq -c | sort -rn | head -20

# 提取特定时间段
kubectl logs -n production deployment/order-service --since=1h | \
  awk '/2024-01-15T10:00/,/2024-01-15T10:30/'
```

### 2.2 日志关联分析

```python
# 将分散的日志按 request_id 关联成完整链路

import json
from collections import defaultdict

def correlate_logs(log_lines: list[str], request_id: str) -> list[dict]:
    """按 request_id 关联日志，还原请求链路"""
    events = []
    for line in log_lines:
        try:
            log = json.loads(line)
            if log.get('request_id') == request_id:
                events.append(log)
        except json.JSONDecodeError:
            continue
    return sorted(events, key=lambda x: x.get('timestamp', ''))

# 输出示例：
# 10:00:00.001 [INFO]  Received POST /api/v1/orders
# 10:00:00.005 [INFO]  Validating request payload
# 10:00:00.010 [INFO]  Querying user from database
# 10:00:00.510 [WARN]  Database query slow: 500ms
# 10:00:00.515 [INFO]  Checking inventory
# 10:00:01.520 [ERROR] Inventory service timeout after 1000ms
# 10:00:01.521 [ERROR] Order creation failed: upstream timeout
```

### 2.3 日志模式识别

```bash
# 找到错误突增的时间点
kubectl logs -n production deployment/order-service --since=6h | \
  grep ERROR | \
  awk -F'T' '{print $1"T"substr($2,1,5)}' | \
  sort | uniq -c | sort -k2

# 输出示例：
#    5 2024-01-15T10:00
#    3 2024-01-15T10:05
#    2 2024-01-15T10:10
#  158 2024-01-15T10:15  ← 错误突增！
#  203 2024-01-15T10:20
#   12 2024-01-15T10:25

# 检查这个时间点前后发生了什么
# 1. 是否有部署？
kubectl rollout history deployment/order-service -n production

# 2. 是否有配置变更？
git log --since="2024-01-15 10:00" --until="2024-01-15 10:20"
```

---

## 步骤三：断点调试

### 3.1 Python 调试

```python
# pdb - 内置调试器
def process_order(order_data):
    user = get_user(order_data['user_id'])
    import pdb; pdb.set_trace()  # 在此暂停
    # 或使用 breakpoint()（Python 3.7+）
    inventory = check_inventory(order_data['product_id'])
    return create_order(user, inventory)

# pdb 常用命令：
# n (next)     - 执行下一行
# s (step)     - 进入函数
# c (continue) - 继续执行
# p var        - 打印变量
# pp var       - 格式化打印
# l (list)     - 显示代码
# w (where)    - 显示调用栈
# u (up)       - 向上一层调用栈
# d (down)     - 向下一层调用栈
# b 42         - 在第 42 行设置断点
# condition 1 x > 10  - 条件断点

# ipdb - 增强版（支持 Tab 补全和语法高亮）
# pip install ipdb
import ipdb; ipdb.set_trace()
```

```python
# 远程调试 - debugpy（VS Code 远程调试）
# pip install debugpy

import debugpy
debugpy.listen(("0.0.0.0", 5678))
print("Waiting for debugger to attach...")
debugpy.wait_for_client()
# 在 VS Code 中配置 launch.json 连接到 5678 端口

# 条件性调试 - 仅在特定条件下触发
def process_order(order_data):
    result = calculate_total(order_data)
    if result < 0:  # 只有异常值才调试
        import pdb; pdb.set_trace()
    return result
```

### 3.2 Post-mortem 调试

```python
# 程序崩溃后分析
import traceback
import sys

def main():
    try:
        risky_operation()
    except Exception:
        # 打印完整异常信息
        traceback.print_exc()
        # 进入 post-mortem 调试
        import pdb; pdb.post_mortem()

# 或在命令行运行
# python -m pdb script.py
# 崩溃时自动进入 pdb
```

### 3.3 异步代码调试

```python
# asyncio 调试模式
import asyncio

# 启用 asyncio 调试
asyncio.get_event_loop().set_debug(True)
# 或设置环境变量 PYTHONASYNCIODEBUG=1

# 追踪协程执行
import logging
logging.getLogger('asyncio').setLevel(logging.DEBUG)

# 检测未 await 的协程
import warnings
warnings.filterwarnings('error', category=RuntimeWarning, message='.*was never awaited.*')
```

---

## 步骤四：分布式链路追踪

### 4.1 追踪架构

```
用户请求
  │
  ▼
API Gateway (Span 1: gateway)
  │
  ├─► Order Service (Span 2: order-create)
  │     │
  │     ├─► User Service (Span 3: user-validate)
  │     │
  │     ├─► Inventory Service (Span 4: inventory-check)
  │     │     │
  │     │     └─► Redis Cache (Span 5: cache-lookup)
  │     │
  │     └─► Database (Span 6: db-insert)
  │
  └─► 响应

每个 Span 记录：
- trace_id: 全链路唯一 ID
- span_id: 当前操作 ID
- parent_span_id: 父操作 ID
- operation_name: 操作名称
- start_time / duration: 时间信息
- tags: 元数据（http.method, http.status_code, error）
- logs: 事件日志
```

### 4.2 OpenTelemetry 集成

```python
# pip install opentelemetry-api opentelemetry-sdk opentelemetry-instrumentation-fastapi

from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.exporter.jaeger.thrift import JaegerExporter
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor

# 配置追踪
provider = TracerProvider()
jaeger_exporter = JaegerExporter(
    agent_host_name="jaeger",
    agent_port=6831,
)
provider.add_span_processor(BatchSpanProcessor(jaeger_exporter))
trace.set_tracer_provider(provider)

# 自动检测 FastAPI
FastAPIInstrumentor.instrument_app(app)

# 手动创建 Span（业务逻辑追踪）
tracer = trace.get_tracer("order-service")

async def create_order(order_data: dict):
    with tracer.start_as_current_span("create_order") as span:
        span.set_attribute("order.product_id", order_data["product_id"])

        with tracer.start_as_current_span("validate_user"):
            user = await validate_user(order_data["user_id"])

        with tracer.start_as_current_span("check_inventory") as inv_span:
            available = await check_inventory(order_data["product_id"])
            inv_span.set_attribute("inventory.available", available)
            if not available:
                inv_span.set_status(trace.Status(trace.StatusCode.ERROR, "Out of stock"))
                raise BusinessError("INSUFFICIENT_STOCK")

        with tracer.start_as_current_span("persist_order"):
            order = await save_order(order_data)

        span.set_attribute("order.id", order.id)
        return order
```

### 4.3 Jaeger 查询

```bash
# 按服务和操作查找追踪
curl -s "http://jaeger:16686/api/traces?service=order-service&operation=create_order&limit=20&lookback=1h" | jq '.data[0].spans | length'

# 按 Trace ID 获取完整链路
curl -s "http://jaeger:16686/api/traces/<trace-id>" | jq '.data[0].spans[] | {operationName, duration: (.duration/1000|tostring + "ms"), tags: [.tags[] | select(.key | test("error|http.status")) | {(.key): .value}]}'

# 查找慢请求
curl -s "http://jaeger:16686/api/traces?service=order-service&minDuration=1s&limit=10" | jq '.data[].traceID'

# 查找错误请求
curl -s "http://jaeger:16686/api/traces?service=order-service&tags=error%3Dtrue&limit=10"
```

### 4.4 追踪分析方法

```markdown
链路分析步骤：

1. 找到问题 Trace ID（从日志、告警或 Jaeger 搜索）
2. 在 Jaeger UI 查看完整 Trace
3. 找到耗时最长的 Span（瓶颈定位）
4. 检查 Span 的 Tags 和 Logs（错误信息）
5. 分析 Span 的时间关系：
   - 是串行导致的慢？→ 改为并行
   - 是某个 Span 自身慢？→ 深入该服务分析
   - 是重试导致的慢？→ 检查重试策略
   - 有 Span 之间的 Gap？→ 检查排队等待
```

---

## 步骤五：内存泄漏排查

### 5.1 检测内存泄漏

```bash
# 监控进程内存趋势
while true; do
    RSS=$(ps -o rss= -p $(pgrep -f "your-app"))
    echo "$(date +%H:%M:%S) RSS: ${RSS}KB"
    sleep 10
done > memory_trend.log

# 用 matplotlib 可视化
python3 << 'PYEOF'
import matplotlib.pyplot as plt

times, values = [], []
with open('memory_trend.log') as f:
    for line in f:
        parts = line.strip().split()
        times.append(parts[0])
        values.append(int(parts[2].rstrip('KB')))

plt.figure(figsize=(12, 4))
plt.plot(values)
plt.ylabel('RSS (KB)')
plt.title('Memory Usage Over Time')
plt.savefig('memory_trend.png')
PYEOF
```

### 5.2 定位泄漏点

```python
import tracemalloc
import gc

# 启用追踪
tracemalloc.start(25)

# 第一次快照
gc.collect()
snapshot1 = tracemalloc.take_snapshot()

# ... 运行一段时间或执行 N 次操作 ...

# 第二次快照
gc.collect()
snapshot2 = tracemalloc.take_snapshot()

# 对比找增长
stats = snapshot2.compare_to(snapshot1, 'traceback')
print("\n=== Top 10 Memory Growth ===")
for stat in stats[:10]:
    print(f"\n{stat}")
    for line in stat.traceback.format():
        print(f"  {line}")
```

### 5.3 常见泄漏模式

```python
# 泄漏模式 1: 全局缓存无限增长
# 问题
_cache = {}
def get_data(key):
    if key not in _cache:
        _cache[key] = expensive_fetch(key)  # 永远不清理！
    return _cache[key]

# 修复: 使用 LRU 缓存
from functools import lru_cache

@lru_cache(maxsize=1024)
def get_data(key):
    return expensive_fetch(key)


# 泄漏模式 2: 事件监听器未取消
# 问题
class DataProcessor:
    def __init__(self, event_bus):
        event_bus.subscribe("data_ready", self.on_data)  # 注册了但从不取消

# 修复: 使用弱引用或显式取消
import weakref

class DataProcessor:
    def __init__(self, event_bus):
        self._event_bus = event_bus
        event_bus.subscribe("data_ready", weakref.WeakMethod(self.on_data))

    def __del__(self):
        self._event_bus.unsubscribe("data_ready", self.on_data)


# 泄漏模式 3: 闭包捕获大对象
# 问题
def create_handler(large_data):
    def handler():
        return len(large_data)  # 闭包持有 large_data 的引用
    return handler

# 修复: 只捕获必要的值
def create_handler(large_data):
    data_length = len(large_data)  # 提取需要的值
    def handler():
        return data_length
    return handler


# 泄漏模式 4: 循环引用
# 问题
class Parent:
    def __init__(self):
        self.child = Child(self)

class Child:
    def __init__(self, parent):
        self.parent = parent  # 循环引用

# 修复: 使用弱引用
class Child:
    def __init__(self, parent):
        self.parent = weakref.ref(parent)
```

### 5.4 生产环境内存诊断

```python
# 不停机的生产环境内存分析端点
from fastapi import FastAPI
import tracemalloc
import gc

app = FastAPI()

# 在应用启动时开启 tracemalloc（有约 5% 性能开销）
tracemalloc.start(10)
_baseline_snapshot = None

@app.post("/debug/memory/baseline")
async def set_memory_baseline():
    """设置内存基线"""
    global _baseline_snapshot
    gc.collect()
    _baseline_snapshot = tracemalloc.take_snapshot()
    return {"status": "baseline set"}

@app.get("/debug/memory/growth")
async def get_memory_growth(top_n: int = 20):
    """查看内存增长"""
    gc.collect()
    current = tracemalloc.take_snapshot()
    if _baseline_snapshot:
        stats = current.compare_to(_baseline_snapshot, 'lineno')
    else:
        stats = current.statistics('lineno')

    return {
        "top_allocations": [
            {
                "file": str(stat.traceback),
                "size_kb": stat.size / 1024,
                "count": stat.count,
            }
            for stat in stats[:top_n]
        ]
    }

@app.get("/debug/memory/objects")
async def get_object_stats(top_n: int = 20):
    """查看对象统计"""
    gc.collect()
    import objgraph
    return {
        "most_common": objgraph.most_common_types(limit=top_n),
        "growth": objgraph.growth(limit=top_n),
    }
```

---

## 步骤六：网络问题调试

### 6.1 连接问题

```bash
# DNS 解析
dig api.external-service.com +short
nslookup api.external-service.com

# 连通性测试
curl -v -o /dev/null -w "\
  DNS: %{time_namelookup}s\n\
  TCP: %{time_connect}s\n\
  TLS: %{time_appconnect}s\n\
  First byte: %{time_starttransfer}s\n\
  Total: %{time_total}s\n\
  Status: %{http_code}\n" \
  https://api.external-service.com/health

# 路由追踪
mtr -r -c 10 api.external-service.com

# TCP 连接状态分析
ss -tnp | awk '{print $1}' | sort | uniq -c | sort -rn
# 如果 TIME_WAIT 过多，可能需要调优内核参数
# 如果 CLOSE_WAIT 过多，应用未正确关闭连接
```

### 6.2 请求级调试

```bash
# 详细的 HTTP 请求调试
curl -v --trace-time https://api.example.com/api/v1/orders 2>&1 | head -50

# 使用 httpie（更友好的输出）
http --print=HhBb GET https://api.example.com/api/v1/orders Authorization:"Bearer xxx"

# 抓包分析
sudo tcpdump -i any -A -s 0 'port 8080 and host 10.0.0.1' -w debug.pcap -c 1000
# 使用 wireshark 分析 debug.pcap
```

---

## 验证

### 调试完成确认

```markdown
调试完成的标志：

1. [ ] 根因已确认（不是表面症状，而是底层原因）
2. [ ] 修复方案已验证（问题不再复现）
3. [ ] 回归测试通过（修复没有引入新问题）
4. [ ] 相关监控已添加（同类问题能被及时发现）
5. [ ] 知识已沉淀（调试过程和经验已文档化）
```

---

## 回滚方案

### 调试操作回滚

```bash
# 移除调试代码
grep -rn "pdb\|breakpoint()\|debugpy\|import ipdb" src/ | head -20
# 确保无调试代码残留

# 移除调试端点
# 确保 /debug/ 路径不暴露到生产环境

# 关闭 tracemalloc（如果开销显著）
tracemalloc.stop()

# 恢复日志级别
# 调试时可能调高了日志级别，调试完成后恢复
```

### 修复回滚

```bash
# 如果修复引入了新问题
git revert <fix-commit>

# 如果修复涉及配置变更
kubectl rollout undo deployment/<service> -n production
```

---

## Agent Checklist

AI 编码 Agent 在协助调试时必须逐项确认：

- [ ] **问题已定义**：有明确的现象描述、影响范围和复现条件
- [ ] **信息已收集**：日志、监控指标、追踪数据已获取
- [ ] **假设已记录**：每个假设和验证结果有记录
- [ ] **科学排查**：一次只验证一个假设，不盲目猜测
- [ ] **根因已确认**：不只是修复了表面症状
- [ ] **修复最小化**：修复范围尽可能小
- [ ] **回归已验证**：修复没有引入新问题
- [ ] **调试代码已清理**：无 pdb/breakpoint/debugpy 残留
- [ ] **监控已补充**：针对此类问题的监控告警已添加
- [ ] **测试已补充**：覆盖此次 Bug 场景的测试已添加
- [ ] **知识已沉淀**：调试过程和根因分析已文档化
- [ ] **防复发措施**：有明确的措施防止同类问题再次发生
