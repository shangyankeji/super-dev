---
id: case-sre-practices
title: 案例研究：SRE 实践落地 - 从理念到工程化的完整路径
domain: operations
category: 05-cases
difficulty: intermediate
tags: [budget, case, engineering, operations, practices, sre, 为例, 优化]
quality_score: 70
last_updated: 2026-06-15
---
# 案例研究：SRE 实践落地 - 从理念到工程化的完整路径

## 元数据

| 字段 | 值 |
|------|------|
| 行业 | 电商平台 |
| 系统规模 | 注册用户 5000 万，日活 800 万，日均订单 200 万 |
| 技术栈 | Go + Java + PostgreSQL + Redis + Kafka + Kubernetes |
| 团队规模 | 后端 60 人，SRE 8 人，QA 10 人 |
| 实施周期 | 12 个月（2024-01 至 2024-12） |
| 核心目标 | 建立 SRE 体系，将可用性从 99.5% 提升至 99.95% |

---

## 一、背景与动机

### 1.1 现状痛点

某电商平台在快速增长阶段遭遇严重的稳定性挑战：

- **可用性不达标**：过去 12 个月发生 18 次 P1 级故障，累计不可用时间 43 小时（可用性约 99.5%）
- **MTTR 过长**：平均故障恢复时间 2.4 小时，最长一次持续 8 小时
- **运维模式落后**：大量手动操作，SRE 团队 70% 时间花在重复性工作（Toil）
- **缺乏量化目标**：团队对"系统够不够稳定"没有统一标准
- **On-Call 疲劳**：告警风暴频发，On-Call 人员每周被唤醒 3-5 次

### 1.2 SRE 转型目标

团队决定借鉴 Google SRE 核心理念，系统性地解决稳定性问题。设定 12 个月路线图：

| 阶段 | 时间 | 目标 |
|------|------|------|
| Q1 | 月 1-3 | 建立 SLO 体系 + 可观测性基础 |
| Q2 | 月 4-6 | Error Budget 机制运行 + Toil 治理 |
| Q3 | 月 7-9 | On-Call 优化 + 自动化修复 |
| Q4 | 月 10-12 | Chaos Engineering + 持续改进 |

---

## 二、Google SRE 核心理念落地

### 2.1 核心原则

SRE 团队首先统一了 Google SRE 的 5 条核心原则，并针对自身场景做了落地解读：

| Google SRE 原则 | 本团队落地解读 |
|----------------|--------------|
| **拥抱风险** (Embracing Risk) | 100% 可用性不是目标，用 Error Budget 量化可接受的风险水平 |
| **服务级别目标** (SLOs) | 每个核心服务定义 SLI/SLO/SLA，用数据驱动决策 |
| **消除苦差事** (Eliminating Toil) | Toil 占比不超过 50%，自动化是工程工作的核心产出 |
| **监控与可观测** (Monitoring) | 从"基于告警"转向"基于 SLO"的监控体系 |
| **简单化** (Simplicity) | 拒绝不必要的复杂性，每次架构变更必须论证必要性 |

### 2.2 组织架构调整

- SRE 团队从运维部独立出来，直接向 CTO 汇报
- SRE 与开发团队比例 1:8（8 名 SRE 对应 60 名开发）
- 每个核心服务指定一名 SRE 作为稳定性 Owner
- 建立"SRE 嵌入"机制：SRE 参加业务团队的 Sprint Planning

---

## 三、SLO 设定实战（以电商 API 为例）

### 3.1 SLI 定义

选择能直接反映用户体验的 SLI（Service Level Indicator）：

```yaml
# 订单服务 SLI 定义
order-service:
  availability:
    description: "成功响应的请求占比"
    formula: "count(status < 500) / count(total_requests)"
    measurement: "Prometheus metrics at load balancer"

  latency:
    description: "请求响应时间"
    formula: "histogram_quantile(0.99, request_duration_seconds)"
    measurement: "Application-level metrics"

  correctness:
    description: "订单创建后数据一致性"
    formula: "count(order_verified) / count(order_created)"
    measurement: "异步校验 Job 每 5 分钟运行"
```

### 3.2 SLO 设定

基于历史数据和业务需求设定 SLO：

| 服务 | SLI | SLO 目标 | 计算窗口 | 依据 |
|------|-----|---------|---------|------|
| 订单服务 | 可用性 | 99.95% | 30 天滚动 | 业务要求：月度不可用 < 21.6 分钟 |
| 订单服务 | P99 延迟 | < 500ms | 30 天滚动 | 用户体验研究：> 500ms 转化率下降 12% |
| 商品服务 | 可用性 | 99.9% | 30 天滚动 | 非交易链路，容忍度较高 |
| 商品服务 | P99 延迟 | < 200ms | 30 天滚动 | 首页/搜索依赖，对速度敏感 |
| 支付服务 | 可用性 | 99.99% | 30 天滚动 | 资金安全，要求极高 |
| 支付服务 | P99 延迟 | < 1000ms | 30 天滚动 | 支付本身耗时较长，用户有预期 |

### 3.3 SLO 实现

```python
# Prometheus 查询：订单服务 30 天可用性
availability_30d = """
  1 - (
    sum(rate(http_requests_total{service="order", code=~"5.."}[30d]))
    /
    sum(rate(http_requests_total{service="order"}[30d]))
  )
"""

# Grafana Dashboard 展示
# - 当前 SLO 达成率（大数字）
# - Error Budget 剩余百分比（进度条）
# - Error Budget 消耗速率（趋势线）
# - 预计 Error Budget 耗尽时间
```

---

## 四、Error Budget 机制

### 4.1 Error Budget 计算

Error Budget 是 SLO 允许的"不可靠余量"：

```
Error Budget = 1 - SLO 目标

订单服务 Error Budget:
  = 1 - 99.95%
  = 0.05%
  = 30 天 × 24 小时 × 60 分钟 × 0.0005
  = 21.6 分钟 / 月
```

### 4.2 Error Budget 策略

建立明确的 Error Budget 消耗规则：

| Budget 剩余 | 状态 | 策略 |
|------------|------|------|
| > 50% | 绿色 | 正常发布节奏，鼓励新功能开发 |
| 25% - 50% | 黄色 | 减少高风险发布，增加灰度比例 |
| 5% - 25% | 橙色 | 冻结非关键发布，全力修复稳定性 |
| < 5% | 红色 | 全面冻结发布，SRE 主导故障排查 |

### 4.3 实际案例：大促前 Error Budget 决策

2024 年 618 大促前两周，订单服务 Error Budget 剩余 35%（黄色）：

```
时间线：
6月1日  - Error Budget 剩余 35%
6月3日  - 产品团队要求发布新促销规则引擎
6月3日  - SRE 评估：新功能涉及订单核心链路，风险高
6月4日  - 决策：推迟到大促后发布
         - 替代方案：通过 Feature Flag + 配置驱动实现部分促销规则
6月5日  - 集中修复已知稳定性问题，Budget 消耗速率下降
6月18日 - 大促期间零 P1 故障，Budget 剩余 28%
```

这个决策避免了大促期间的潜在故障，是 Error Budget 机制发挥作用的典型场景。

---

## 五、Toil 治理

### 5.1 Toil 定义与识别

Toil 是满足以下特征的工作：手动的、重复的、可自动化的、战术性的、缺乏持久价值的、随服务增长线性增长的。

团队首先做了 Toil 审计，记录 SRE 两周内的所有工作：

| Toil 项 | 频率 | 每次耗时 | 月累计 | 自动化难度 |
|---------|------|---------|--------|-----------|
| 手动扩容/缩容 | 日均 3 次 | 15 分钟 | 22.5h | 低 |
| 证书更新 | 月均 8 次 | 30 分钟 | 4h | 低 |
| 日志清理 | 日均 1 次 | 10 分钟 | 5h | 低 |
| 数据库慢查询处理 | 周均 5 次 | 45 分钟 | 15h | 中 |
| 用户数据修复 | 周均 3 次 | 60 分钟 | 12h | 中 |
| 配置变更 | 日均 2 次 | 20 分钟 | 20h | 中 |
| 故障排查 | 周均 2 次 | 120 分钟 | 16h | 高 |

**Toil 总计：约 94.5 小时/月，占 SRE 总工时的 73%**（8 人 × 160h = 1280h，Toil 占比 = 94.5 × 8 人均分后按实际执行人计算约 73%）。

### 5.2 Toil 治理路线

按"高频 + 低难度"优先原则排序：

**第一批（月 1-2）- 快速自动化**：

```yaml
# HPA 自动扩缩容
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: order-service-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: order-service
  minReplicas: 3
  maxReplicas: 50
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
        - type: Percent
          value: 100
          periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
```

```bash
# cert-manager 自动证书管理
kubectl apply -f - <<EOF
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: api-cert
spec:
  secretName: api-tls
  issuerRef:
    name: letsencrypt-prod
    kind: ClusterIssuer
  dnsNames:
    - api.example.com
  renewBefore: 720h  # 证书过期前 30 天自动续签
EOF
```

**第二批（月 3-4）- 半自动化**：

- 慢查询自动检测 + 建议索引 + 人工确认执行
- 配置变更通过 GitOps 流程，PR Review 后自动应用
- 日志清理通过 CronJob 自动执行

**第三批（月 5-6）- 智能自动化**：

- 用户数据修复：建立自助修复工具，减少 SRE 介入
- 故障排查：建立 Runbook 自动化（后续章节详述）

### 5.3 治理成果

| 指标 | 治理前 | 治理后 | 改善 |
|------|--------|--------|------|
| Toil 占比 | 73% | 28% | -45pp |
| 月均 Toil 工时 | 94.5h | 36h | -62% |
| SRE 工程项目占比 | 27% | 72% | +45pp |

---

## 六、On-Call 优化

### 6.1 On-Call 现状问题

- 告警数量：日均 200+ 条告警，其中 80% 无需人工介入
- 告警疲劳：On-Call 人员对告警脱敏，关键告警被忽略
- 升级不清晰：不确定何时升级、找谁升级
- 知识孤岛：故障处理经验在个人脑中，无法传递

### 6.2 告警治理

**告警分级体系**：

| 级别 | 条件 | 响应要求 | 通知方式 |
|------|------|---------|---------|
| P0 / Critical | Error Budget 消耗 > 10x 正常速率 | 5 分钟内响应 | 电话 + 短信 + Slack |
| P1 / High | SLO 达成率下降但 Budget 充足 | 15 分钟内响应 | 短信 + Slack |
| P2 / Medium | 非核心服务异常 | 1 小时内响应 | Slack |
| P3 / Low | 预警信号 | 次工作日处理 | Slack (低优先级频道) |

**告警降噪措施**：

```yaml
# Alertmanager 告警聚合
route:
  group_by: ['service', 'alertname']
  group_wait: 30s       # 等待 30s 聚合同类告警
  group_interval: 5m    # 同组告警间隔 5m 发送
  repeat_interval: 4h   # 未解决告警 4h 重复提醒

inhibit_rules:
  # 集群级故障抑制 Pod 级告警
  - source_match:
      severity: critical
      scope: cluster
    target_match:
      severity: warning
      scope: pod
    equal: ['cluster']
```

**治理后**：日均告警从 200+ 降至 15 条，P0/P1 告警占比 < 20%。

### 6.3 On-Call 轮值制度

```
轮值规则：
- 主 On-Call + 副 On-Call（双人制）
- 每轮 7 天，周一 10:00 交接
- 交接会议必须包含：
  - 本周告警统计与趋势
  - 未关闭的 Issue 和待跟进事项
  - 已知风险（即将到来的大促/变更）
  - Runbook 更新情况
- On-Call 补偿：每次 On-Call 额外 1 天调休
```

### 6.4 Runbook 自动化

将常见故障处理流程编写为可执行的 Runbook：

```python
# Runbook 示例：Redis 连接池耗尽自动处理
class RedisConnectionPoolExhausted(Runbook):
    trigger = "redis_connection_pool_usage > 90%"
    severity = "P1"

    def diagnose(self):
        """诊断步骤"""
        checks = [
            self.check_redis_connections(),      # 当前连接数
            self.check_slow_commands(),           # 慢命令统计
            self.check_client_list(),             # 客户端连接分布
            self.check_recent_deployments(),      # 最近部署
        ]
        return self.analyze(checks)

    def auto_remediate(self):
        """自动修复"""
        # Step 1: Kill 空闲超过 300s 的连接
        self.redis_cli("CLIENT KILL IDLE 300")

        # Step 2: 如果仍高于 80%，临时扩大连接池
        if self.check_pool_usage() > 80:
            self.scale_pool(factor=1.5)

        # Step 3: 创建 Ticket 跟踪根因
        self.create_ticket(
            title="Redis 连接池耗尽 - 需要根因分析",
            assignee="on-call",
            priority="high",
        )

    def escalate(self):
        """升级路径"""
        return ["on-call-primary", "on-call-secondary", "sre-lead"]
```

---

## 七、自动化修复

### 7.1 自动修复框架

建立分级自动修复能力：

| 级别 | 自动化程度 | 示例 |
|------|-----------|------|
| L0 | 全自动 | Pod 重启、连接池回收、缓存清理 |
| L1 | 半自动（自动诊断 + 人工确认） | 数据库 Failover、流量降级 |
| L2 | 辅助（提供诊断报告 + 建议操作） | 数据不一致修复、容量规划 |
| L3 | 人工（Runbook 指导） | 架构级故障、数据恢复 |

### 7.2 自愈系统实现

```yaml
# Kubernetes 自愈配置
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
        - name: order-service
          livenessProbe:
            httpGet:
              path: /healthz
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 10
            failureThreshold: 3    # 连续 3 次失败则重启
          readinessProbe:
            httpGet:
              path: /readyz
              port: 8080
            periodSeconds: 5
            failureThreshold: 2    # 连续 2 次失败则从 LB 摘除
          resources:
            requests:
              memory: "512Mi"
              cpu: "500m"
            limits:
              memory: "1Gi"
              cpu: "1000m"
      # OOMKilled 后自动重启
      restartPolicy: Always
```

### 7.3 降级策略自动化

```go
// 熔断降级配置
circuitBreaker := gobreaker.NewCircuitBreaker(gobreaker.Settings{
    Name:        "payment-service",
    MaxRequests: 5,                      // 半开状态允许 5 个请求
    Interval:    10 * time.Second,       // 统计窗口
    Timeout:     30 * time.Second,       // 熔断持续时间
    ReadyToTrip: func(counts gobreaker.Counts) bool {
        failureRatio := float64(counts.TotalFailures) / float64(counts.Requests)
        return counts.Requests >= 10 && failureRatio >= 0.5  // 失败率 > 50% 触发熔断
    },
    OnStateChange: func(name string, from, to gobreaker.State) {
        metrics.CircuitBreakerState.WithLabelValues(name).Set(float64(to))
        if to == gobreaker.StateOpen {
            alerting.Notify(alerting.P1, fmt.Sprintf("Circuit breaker %s opened", name))
        }
    },
})
```

---

## 八、Chaos Engineering 实践

### 8.1 Chaos Engineering 成熟度模型

团队采用渐进式引入策略：

| 阶段 | 时间 | 实验范围 | 工具 |
|------|------|---------|------|
| 起步 | 月 7-8 | 非生产环境 + 单一故障 | LitmusChaos |
| 进阶 | 月 9-10 | 生产环境 + 受控故障 | LitmusChaos + 自研 |
| 成熟 | 月 11-12 | 生产环境 + Game Day | 全套工具链 |

### 8.2 实验设计

**实验 1：Redis 主节点故障**

```yaml
# LitmusChaos 实验定义
apiVersion: litmuschaos.io/v1alpha1
kind: ChaosEngine
metadata:
  name: redis-failover-test
spec:
  appinfo:
    appns: production
    applabel: app=redis-master
  chaosServiceAccount: litmus-admin
  experiments:
    - name: pod-delete
      spec:
        components:
          env:
            - name: TOTAL_CHAOS_DURATION
              value: '60'           # 故障持续 60 秒
            - name: CHAOS_INTERVAL
              value: '10'
            - name: FORCE
              value: 'true'
```

**预期结果**：Redis Sentinel 在 30 秒内完成 Failover，应用自动重连，订单服务 SLO 不受影响。

**实际结果**：

```
第一次实验（月 8）：
- Failover 耗时 45s（超过预期）
- 应用连接池未正确处理 Failover，出现 120s 的错误
- 订单服务可用性降至 99.2%（短期）

修复措施：
- 调整 Sentinel down-after-milliseconds: 10000 → 5000
- 应用添加 Redis 重连逻辑 + 连接池健康检查
- 引入 Redis 客户端 Sentinel 模式

第二次实验（月 9）：
- Failover 耗时 12s
- 应用在 15s 内恢复正常
- 订单服务可用性保持 99.95% 以上
```

### 8.3 Game Day

每季度组织一次 Game Day（全团队参与的故障演练）：

```
Game Day 流程：
1. 准备（前 1 周）
   - 定义故障场景和影响范围
   - 准备回滚方案
   - 通知所有相关团队

2. 执行（当天）
   09:30 - 团队集合，说明规则
   10:00 - 注入故障（不告知具体故障类型）
   10:00~12:00 - On-Call 团队按正常流程响应
   12:00 - 故障恢复确认

3. 复盘（当天下午）
   - 时间线回顾
   - 发现的问题和改进项
   - 更新 Runbook
   - 指派 Action Item
```

**2024-Q4 Game Day 成果**：

- 场景：主数据库所在可用区网络隔离
- 发现 3 个未知的单点故障
- MTTR 从预估的 30 分钟实际达到 18 分钟
- 识别出 2 个 Runbook 中的过期步骤

---

## 九、成果总结

### 9.1 关键指标对比

| 指标 | 实施前 | 实施后 | 改善 |
|------|--------|--------|------|
| 可用性 | 99.5% | 99.96% | +0.46pp |
| P1 故障次数（年） | 18 次 | 3 次 | -83% |
| MTTR | 2.4h | 22min | -85% |
| Toil 占比 | 73% | 28% | -45pp |
| 日均告警数 | 200+ | 15 | -92% |
| On-Call 夜间唤醒 | 3-5 次/周 | 0.3 次/周 | -92% |

### 9.2 关键经验

1. **SLO 先行**：没有 SLO 就没有 Error Budget，没有 Error Budget 就无法平衡速度与稳定性
2. **Toil 可量化才可治理**：先审计后优化，数据驱动优先级
3. **Chaos Engineering 渐进引入**：从非生产环境开始，建立信心后再进入生产
4. **自动化是工程投资**：前期投入大但复利效应显著
5. **文化比工具重要**：SRE 不只是工具链，更是团队认知的转变

### 9.3 后续规划

- AIOps：基于历史告警数据训练异常检测模型
- 全链路压测：大促前自动化全链路压测流程
- SRE 平台化：将 SRE 工具链打包为内部 PaaS

---

## Agent Checklist

- [ ] 覆盖 Google SRE 核心理念介绍
- [ ] Error Budget 机制包含计算方法和实际决策案例
- [ ] Toil 治理包含审计数据和分阶段自动化路线
- [ ] SLO 设定以电商 API 为具体案例，包含 SLI/SLO/SLA
- [ ] On-Call 优化包含告警治理、轮值制度和 Runbook
- [ ] 自动化修复包含分级框架和代码示例
- [ ] Chaos Engineering 包含渐进式实施和 Game Day 流程
- [ ] 所有数据前后对比清晰，改善幅度可量化
- [ ] 代码示例使用真实工具语法（Prometheus/K8s/Go/Python）
- [ ] 文件超过 250 行
