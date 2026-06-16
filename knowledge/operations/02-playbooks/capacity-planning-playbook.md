---
id: capacity-planning-playbook
title: 容量规划作战手册 (Capacity Planning Playbook)
domain: operations
category: 02-playbooks
difficulty: intermediate
tags: [capacity, operations, planning, playbook, 业务增长评估模板, 业务概况, 前置条件, 报告信息]
quality_score: 70
last_updated: 2026-06-15
---
# 容量规划作战手册 (Capacity Planning Playbook)

## 概述

容量规划是确保系统在业务增长过程中持续满足性能与可用性目标的系统性工程实践。本手册覆盖从需求分析到自动伸缩的完整流程，帮助团队建立数据驱动的容量决策机制，避免资源浪费与性能劣化。适用于中大型生产系统（日活 > 1 万或月请求量 > 1 亿）。

## 前置条件

### 必须满足

- [ ] 生产环境可观测性已部署（Prometheus / Datadog / CloudWatch）
- [ ] 核心业务指标已定义 SLI/SLO
- [ ] 历史流量数据至少覆盖最近 90 天
- [ ] 基础设施即代码（IaC）已就绪，支持快速扩缩容
- [ ] 成本监控工具已配置（AWS Cost Explorer / GCP Billing / 自建）

### 建议满足

- [ ] 业务方已提供未来 6-12 个月增长预期
- [ ] 已完成至少一次基线压测
- [ ] 告警体系已覆盖核心资源指标（CPU/Memory/Disk/Network）

---

## 阶段一：需求分析与业务建模

### 1.1 业务增长预测

```markdown
## 业务增长评估模板

### 当前基线
- 日活用户数 (DAU): ___
- 峰值 QPS: ___
- 平均请求延迟 (P50/P95/P99): ___/___/___ ms
- 存储增长速率: ___ GB/月
- 数据库连接峰值: ___

### 增长预期
- 未来 3 个月 DAU 预期: ___（增长率 ___%）
- 未来 6 个月 DAU 预期: ___（增长率 ___%）
- 未来 12 个月 DAU 预期: ___（增长率 ___%）
- 计划中的营销活动 / 大促日期: ___
- 新功能上线对流量的预估影响: ___

### 季节性因素
- 日内峰谷比: ___（如 10:1）
- 周内波动模式: ___
- 年内季节性高峰: ___
```

### 1.2 资源消耗画像

针对每个核心服务，建立资源消耗模型：

```yaml
# 服务资源画像示例
service: order-service
resource_profile:
  cpu:
    baseline: 200m        # 空闲时 CPU 使用
    per_request: 5m       # 每请求增量
    peak_multiplier: 3.0  # 峰值倍率
  memory:
    baseline: 256Mi
    per_connection: 2Mi
    cache_overhead: 128Mi
  disk_io:
    read_iops: 500
    write_iops: 200
  network:
    ingress_per_req: 2KB
    egress_per_req: 8KB
  dependencies:
    - name: postgres
      connections_per_pod: 10
      max_pool: 50
    - name: redis
      connections_per_pod: 20
```

### 1.3 容量需求计算

```python
# 容量需求计算公式
def calculate_capacity(current_qps, growth_rate, months, safety_margin=0.3):
    """
    计算目标容量需求

    Args:
        current_qps: 当前峰值 QPS
        growth_rate: 月增长率 (如 0.15 表示 15%)
        months: 规划周期（月）
        safety_margin: 安全余量（默认 30%）

    Returns:
        target_qps: 目标容量
    """
    projected_qps = current_qps * (1 + growth_rate) ** months
    target_qps = projected_qps * (1 + safety_margin)
    return target_qps

# 示例：当前 1000 QPS，月增长 10%，规划 6 个月
# 目标 = 1000 * 1.1^6 * 1.3 ≈ 2304 QPS
```

---

## 阶段二：负载测试

### 2.1 k6 压测方案

```javascript
// k6-capacity-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const latency = new Trend('request_latency');

// 阶梯式负载：逐步增加并发，找到系统拐点
export const options = {
  stages: [
    { duration: '2m', target: 50 },    // 热身
    { duration: '5m', target: 100 },   // 基线负载
    { duration: '5m', target: 200 },   // 1.5x 负载
    { duration: '5m', target: 400 },   // 3x 负载
    { duration: '5m', target: 600 },   // 4.5x 负载（探测极限）
    { duration: '5m', target: 800 },   // 6x 负载（过载测试）
    { duration: '3m', target: 0 },     // 回落恢复
  ],
  thresholds: {
    http_req_duration: ['p(95)<500', 'p(99)<1000'],
    errors: ['rate<0.01'],
  },
};

export default function () {
  // 模拟核心业务场景（按真实流量比例混合）
  const scenarios = [
    { weight: 50, fn: browseProducts },
    { weight: 30, fn: searchProducts },
    { weight: 15, fn: addToCart },
    { weight: 5,  fn: checkout },
  ];

  const rand = Math.random() * 100;
  let cumulative = 0;
  for (const s of scenarios) {
    cumulative += s.weight;
    if (rand < cumulative) {
      s.fn();
      break;
    }
  }

  sleep(1);
}

function browseProducts() {
  const res = http.get(`${__ENV.BASE_URL}/api/products`);
  check(res, { 'browse 200': (r) => r.status === 200 });
  errorRate.add(res.status !== 200);
  latency.add(res.timings.duration);
}

function searchProducts() {
  const res = http.get(`${__ENV.BASE_URL}/api/search?q=test`);
  check(res, { 'search 200': (r) => r.status === 200 });
  errorRate.add(res.status !== 200);
  latency.add(res.timings.duration);
}

function addToCart() {
  const payload = JSON.stringify({ productId: 1, quantity: 1 });
  const params = { headers: { 'Content-Type': 'application/json' } };
  const res = http.post(`${__ENV.BASE_URL}/api/cart`, payload, params);
  check(res, { 'cart 200': (r) => r.status === 200 });
  errorRate.add(res.status !== 200);
  latency.add(res.timings.duration);
}

function checkout() {
  const res = http.post(`${__ENV.BASE_URL}/api/checkout`, '{}', {
    headers: { 'Content-Type': 'application/json' },
  });
  check(res, { 'checkout 200': (r) => r.status === 200 });
  errorRate.add(res.status !== 200);
  latency.add(res.timings.duration);
}
```

### 2.2 Locust 压测方案

```python
# locustfile.py
from locust import HttpUser, task, between, events
import json
import logging

class CapacityTestUser(HttpUser):
    """容量规划压测用户模型"""
    wait_time = between(0.5, 2.0)

    @task(50)
    def browse_products(self):
        with self.client.get("/api/products", catch_response=True) as resp:
            if resp.status_code != 200:
                resp.failure(f"Status {resp.status_code}")

    @task(30)
    def search_products(self):
        with self.client.get("/api/search?q=test", catch_response=True) as resp:
            if resp.status_code != 200:
                resp.failure(f"Status {resp.status_code}")

    @task(15)
    def add_to_cart(self):
        payload = {"productId": 1, "quantity": 1}
        with self.client.post("/api/cart", json=payload, catch_response=True) as resp:
            if resp.status_code != 200:
                resp.failure(f"Status {resp.status_code}")

    @task(5)
    def checkout(self):
        with self.client.post("/api/checkout", json={}, catch_response=True) as resp:
            if resp.status_code != 200:
                resp.failure(f"Status {resp.status_code}")

# 运行命令：
# locust -f locustfile.py --host=http://target:8080 \
#   --users 500 --spawn-rate 10 --run-time 30m \
#   --csv=capacity_report --html=capacity_report.html
```

### 2.3 压测执行规范

| 项目 | 要求 |
|------|------|
| 测试环境 | 必须与生产配置一致（实例规格、副本数、中间件版本） |
| 数据准备 | 数据量级至少为生产的 50%，分布特征一致 |
| 基线测试 | 先在当前配置下跑出基线 P50/P95/P99 |
| 阶梯递增 | 每阶段递增 50-100% 负载，持续至少 5 分钟 |
| 监控覆盖 | 压测期间必须同步采集 CPU/Memory/Disk IO/Network/DB 指标 |
| 超时设定 | 请求超时应与生产一致（不可放大） |
| 重复验证 | 关键结论至少重复验证 2 次 |

---

## 阶段三：瓶颈识别

### 3.1 资源维度分析矩阵

```markdown
## 瓶颈识别检查表

### CPU 瓶颈
- [ ] CPU 使用率是否持续 > 70%？
- [ ] 是否存在 CPU throttling（容器场景检查 nr_throttled）？
- [ ] 热点函数是否可优化（通过 profiling 确认）？
- [ ] 是否存在不必要的序列化/反序列化开销？

### 内存瓶颈
- [ ] RSS 内存是否持续增长（内存泄漏）？
- [ ] GC 停顿时间是否影响延迟？
- [ ] OOM Kill 事件是否出现？
- [ ] 缓存命中率是否达标（> 90%）？

### 磁盘 I/O 瓶颈
- [ ] IOPS 是否接近磁盘上限？
- [ ] I/O await 时间是否过高（> 10ms）？
- [ ] 是否存在写放大（WAL、日志过多）？
- [ ] 磁盘使用率是否 > 80%？

### 网络瓶颈
- [ ] 带宽使用率是否接近网卡上限？
- [ ] TCP 连接数是否接近 ulimit？
- [ ] DNS 解析延迟是否过高？
- [ ] 跨可用区流量是否过大？

### 数据库瓶颈
- [ ] 慢查询比例（> 100ms）？
- [ ] 连接池使用率是否 > 80%？
- [ ] 锁等待是否频繁？
- [ ] 索引是否覆盖高频查询？
- [ ] 主从延迟是否 < 1s？

### 外部依赖瓶颈
- [ ] 第三方 API 延迟 P99 是否 < 500ms？
- [ ] 熔断器触发频率是否正常？
- [ ] 消息队列积压量是否可控？
```

### 3.2 性能拐点定位

系统性能拐点是容量规划的核心数据点。通过阶梯压测数据，识别以下拐点：

| 拐点类型 | 定义 | 识别方法 |
|----------|------|----------|
| 拐点 A（线性区终点） | 延迟开始偏离线性增长 | P95 延迟增速 > 20% |
| 拐点 B（饱和点） | 吞吐量不再随负载增加 | QPS 增长 < 5% 而负载增加 > 20% |
| 拐点 C（崩溃点） | 错误率突增、系统不可用 | 错误率 > 1% 或 P99 > SLO 阈值 |

**安全运行区间 = 拐点 A 的 70-80%**

---

## 阶段四：扩容策略

### 4.1 垂直扩容 (Scale Up)

适用场景：
- 数据库主节点（写入瓶颈）
- 有状态服务（迁移成本高）
- 单线程瓶颈（需要更高主频）

```yaml
# 垂直扩容决策矩阵
vertical_scaling:
  when_to_use:
    - 单实例 CPU/Memory 未达上限
    - 应用无法水平扩展（强状态）
    - 扩容窗口紧急（< 1 小时）
  limits:
    - 单机上限受云厂商实例规格限制
    - 扩容需要停机（部分场景）
    - 成本随规格指数增长
  sizing_guide:
    cpu_upgrade: "每次升级 1 档（如 4c -> 8c），观测 24h"
    memory_upgrade: "按当前使用量 2x 扩容，预留 GC 空间"
    disk_upgrade: "IOPS 不足优先升级磁盘类型（gp3 -> io2）"
```

### 4.2 水平扩容 (Scale Out)

适用场景：
- 无状态 Web/API 服务
- 读密集型数据库（加只读副本）
- 消息消费者（增加消费实例）

```yaml
# 水平扩容配置示例 (Kubernetes)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
spec:
  replicas: 6              # 从 3 扩到 6
  strategy:
    rollingUpdate:
      maxSurge: 2          # 滚动更新批次
      maxUnavailable: 1
  template:
    spec:
      containers:
      - name: api
        resources:
          requests:
            cpu: "500m"
            memory: "512Mi"
          limits:
            cpu: "1000m"
            memory: "1Gi"
      topologySpreadConstraints:   # 跨可用区打散
      - maxSkew: 1
        topologyKey: topology.kubernetes.io/zone
        whenUnsatisfiable: DoNotSchedule
```

### 4.3 扩容决策树

```
当前瓶颈是什么？
├── CPU → 应用是否支持多副本？
│   ├── 是 → 水平扩容（增加 Pod）
│   └── 否 → 垂直扩容（升级 CPU）
├── Memory → 是否为缓存可外置？
│   ├── 是 → 引入外部缓存（Redis）
│   └── 否 → 垂直扩容（增加内存）
├── Disk I/O → 是否为读瓶颈？
│   ├── 是 → 加只读副本 / 引入缓存层
│   └── 否 → 升级磁盘类型 / 分库分表
├── Network → 是否为跨区流量？
│   ├── 是 → 增加本区副本 / CDN
│   └── 否 → 升级网卡 / 优化 payload
└── DB 连接 → 引入连接池 / PgBouncer / ProxySQL
```

---

## 阶段五：自动伸缩

### 5.1 Kubernetes HPA

```yaml
# HPA 配置（基于 CPU + 自定义指标）
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-server-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-server
  minReplicas: 3
  maxReplicas: 20
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60     # 扩容冷却 1 分钟
      policies:
      - type: Pods
        value: 4                         # 每次最多加 4 个 Pod
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300    # 缩容冷却 5 分钟
      policies:
      - type: Percent
        value: 25                        # 每次最多缩 25%
        periodSeconds: 120
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 65           # CPU 目标 65%
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 75
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second   # 自定义业务指标
      target:
        type: AverageValue
        averageValue: "100"              # 每 Pod 目标 100 QPS
```

### 5.2 KEDA 事件驱动伸缩

```yaml
# KEDA ScaledObject（基于消息队列深度）
apiVersion: keda.sh/v1alpha1
kind: ScaledObject
metadata:
  name: order-processor-scaler
spec:
  scaleTargetRef:
    name: order-processor
  minReplicaCount: 2
  maxReplicaCount: 50
  pollingInterval: 15
  cooldownPeriod: 120
  triggers:
  - type: rabbitmq
    metadata:
      host: "amqp://rabbitmq.default:5672"
      queueName: "orders"
      queueLength: "100"        # 每 100 条消息扩一个 Pod
  - type: prometheus
    metadata:
      serverAddress: "http://prometheus:9090"
      metricName: order_processing_lag_seconds
      threshold: "30"           # 处理延迟 > 30s 时扩容
      query: |
        avg(order_processing_lag_seconds{job="order-processor"})
```

### 5.3 伸缩策略最佳实践

| 策略 | 推荐值 | 原因 |
|------|--------|------|
| 扩容冷却期 | 30-60s | 快速响应流量突增 |
| 缩容冷却期 | 300-600s | 避免流量波动导致频繁缩容 |
| CPU 目标利用率 | 60-70% | 预留 burst 空间 |
| 最小副本数 | >= 2 | 保证高可用 |
| 最大副本数 | 基于预算和集群容量 | 防止资源耗尽 |
| 扩容步长 | 当前副本数的 50-100% | 快速到位 |
| 缩容步长 | 当前副本数的 10-25% | 平稳收缩 |

---

## 阶段六：成本优化

### 6.1 资源利用率审计

```bash
#!/bin/bash
# 资源利用率审计脚本
echo "=== 资源利用率审计 ==="

# 查看所有 namespace 资源使用 vs 请求
kubectl top pods -A --sort-by=cpu | head -20

# 找到过度分配的 Pod（请求远大于实际使用）
echo "--- 过度分配检查 ---"
kubectl get pods -A -o json | jq -r '
  .items[] |
  select(.spec.containers[0].resources.requests.cpu != null) |
  "\(.metadata.namespace)/\(.metadata.name) - CPU Request: \(.spec.containers[0].resources.requests.cpu)"
'

# 查看 PVC 使用情况
echo "--- 存储利用率 ---"
kubectl get pvc -A --sort-by=.spec.resources.requests.storage
```

### 6.2 成本优化清单

| 优化项 | 预期节省 | 风险等级 | 实施难度 |
|--------|----------|----------|----------|
| 使用 Spot/Preemptible 实例（无状态服务） | 60-80% | 中 | 低 |
| 右调资源 request/limit（Right-sizing） | 20-40% | 低 | 低 |
| 预留实例 / Savings Plan（基线负载） | 30-50% | 低 | 低 |
| 非高峰时段缩容（定时 HPA） | 15-25% | 低 | 低 |
| 冷数据归档（S3 Glacier / 低频存储） | 40-60% | 低 | 中 |
| 多可用区副本精简（非关键服务） | 10-20% | 中 | 中 |
| 缓存层优化减少 DB 读负载 | 间接 | 低 | 中 |

---

## 阶段七：容量报告

### 7.1 容量报告模板

```markdown
# 容量规划报告

## 报告信息
- 报告日期: YYYY-MM-DD
- 报告人: ___
- 评审周期: Q_ YYYY
- 下次评审: YYYY-MM-DD

## 1. 业务概况
| 指标 | 上期 | 本期 | 变化 | 预测（下期） |
|------|------|------|------|-------------|
| DAU  |      |      |      |             |
| 峰值 QPS |  |      |      |             |
| 数据量(GB) | |     |      |             |
| 月度成本($) | |    |      |             |

## 2. 核心服务容量状态
| 服务 | 当前容量 | 实际负载 | 利用率 | 拐点距离 | 风险等级 |
|------|---------|---------|--------|---------|---------|
| api-server | 2000 QPS | 1200 QPS | 60% | 800 QPS | 低 |
| order-svc  | 500 QPS  | 420 QPS  | 84% | 80 QPS  | 高 |
| user-db    | 200 conn | 180 conn | 90% | 20 conn | 危险 |

## 3. 资源利用率摘要
| 资源类型 | 总配额 | 实际使用 | 利用率 | 趋势 |
|----------|--------|---------|--------|------|
| CPU (cores) | 100 | 62 | 62% | ↑ |
| Memory (GB) | 256 | 180 | 70% | ↑ |
| Storage (TB) | 10 | 7.2 | 72% | ↑ |
| DB Connections | 500 | 380 | 76% | → |

## 4. 压测结果摘要
- 测试日期: YYYY-MM-DD
- 工具: k6 / Locust
- 拐点 A: ___ QPS (P95 < 200ms)
- 拐点 B: ___ QPS (吞吐饱和)
- 拐点 C: ___ QPS (错误率 > 1%)
- 安全运行上限: ___ QPS

## 5. 风险与建议
| 风险项 | 影响 | 建议措施 | 优先级 | 预估成本 |
|--------|------|---------|--------|---------|
| order-svc 接近饱和 | 订单失败 | 水平扩容至 8 Pod | P0 | $200/月 |
| user-db 连接池满 | 全站降级 | 引入 PgBouncer | P0 | $50/月 |
| 存储增长快 | 磁盘满 | 冷数据归档 | P1 | 节省$100/月 |

## 6. 行动计划
| 编号 | 行动项 | 负责人 | 截止日期 | 状态 |
|------|--------|--------|---------|------|
| CP-01 | order-svc 扩容 | SRE-A | YYYY-MM-DD | 待执行 |
| CP-02 | 引入 PgBouncer | DBA-B | YYYY-MM-DD | 待执行 |
| CP-03 | 冷数据归档方案 | SRE-C | YYYY-MM-DD | 规划中 |
```

### 7.2 容量评审节奏

| 评审类型 | 频率 | 参与者 | 输出 |
|----------|------|--------|------|
| 周度快查 | 每周 | SRE On-call | 异常告警 Slack 通知 |
| 月度评审 | 每月 | SRE + 架构师 | 月度容量简报 |
| 季度规划 | 每季度 | SRE + 架构 + 业务 | 完整容量报告 + 预算申请 |
| 大促专项 | 大促前 4 周 | 全团队 | 压测报告 + 扩容方案 |

---

## Agent Checklist

- [ ] 已完成业务增长预测并建立资源消耗画像
- [ ] 已使用 k6 或 Locust 完成基线压测和阶梯压测
- [ ] 已识别系统性能拐点（A/B/C 三个拐点）
- [ ] 已完成瓶颈分析（CPU/Memory/Disk/Network/DB/外部依赖）
- [ ] 已制定扩容策略（垂直 vs 水平 vs 混合）
- [ ] 已配置自动伸缩（HPA/KEDA 或等效方案）
- [ ] 已完成成本优化审计并输出优化建议
- [ ] 已输出完整的容量规划报告
- [ ] 已建立定期容量评审机制（周/月/季度）
- [ ] 已将容量告警集成到监控体系（利用率 > 70% 预警，> 85% 告警）
- [ ] 已将压测脚本纳入版本控制并可重复执行
- [ ] 已与业务方对齐增长预期并更新容量模型
