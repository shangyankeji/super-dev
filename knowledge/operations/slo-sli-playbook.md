---
id: slo-sli-playbook
title: slo-sli-playbook
domain: operations
category: slo-sli-playbook.md
difficulty: intermediate
tags: [budget, operations, playbook, sli, slo, 实施流程, 核心概念, 目标]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）
# 功能：SLO/SLI 实战手册
# 作用：提供服务水平目标（SLO）和服务水平指标（SLI）的完整实施指南
# 创建时间：2026-03-20
# 最后修改：2026-03-20

## 目标
建立以 SLO 为中心的可靠性工程体系,量化服务质量,平衡功能交付与稳定性,实现数据驱动的故障响应和容量规划。

## 适用范围
- 所有面向用户的生产服务
- 关键内部依赖服务（数据库、消息队列、缓存）
- 第三方服务集成（支付网关、短信服务、云服务）

## 核心概念

### SLI（Service Level Indicator）服务水平指标
**定义**：量化服务行为的指标,反映服务质量

**SLI 类型**：
1. **可用性（Availability）**
   - 定义：服务正常响应的比例
   - 计算：成功请求数 / 总请求数
   - 示例：99.9% 可用性 = 允许 0.1% 失败率

2. **延迟（Latency）**
   - 定义：请求响应时间
   - 计算：P50/P95/P99 分位数
   - 示例：P95 < 200ms, P99 < 500ms

3. **吞吐量（Throughput）**
   - 定义：单位时间处理的请求数
   - 计算：QPS/RPS
   - 示例：1000 QPS

4. **错误率（Error Rate）**
   - 定义：失败请求占比
   - 计算：错误请求数 / 总请求数
   - 示例：< 0.1% 错误率

5. **数据质量（Data Quality）**
   - 定义：数据准确性/完整性
   - 计算：正确数据数 / 总数据数
   - 示例：订单金额准确率 99.99%

### SLO（Service Level Objective）服务水平目标
**定义**：SLI 的目标值,定义服务的最低可接受行为

**SLO 设计原则**：
- 可衡量：基于可靠的 SLI
- 可达成：考虑历史性能和团队能力
- 用户中心：反映用户真实体验
- 渐进式：逐步提升,避免过度承诺

**SLO 示例**：
```yaml
# 电商订单服务 SLO
availability:
  target: 99.9%
  window: 30d
  description: "30 天内订单服务可用性 >= 99.9%"

latency:
  - target: 200ms
    percentile: 95
    window: 5m
    description: "P95 延迟 < 200ms（5 分钟窗口）"
  - target: 500ms
    percentile: 99
    window: 5m
    description: "P99 延迟 < 500ms（5 分钟窗口）"

throughput:
  target: 1000 QPS
  window: 1m
  description: "吞吐量 >= 1000 QPS（1 分钟窗口）"

error_rate:
  target: 0.1%
  window: 5m
  description: "错误率 < 0.1%（5 分钟窗口）"
```

### SLA（Service Level Agreement）服务水平协议
**定义**：与用户签订的、违反后需要赔偿的正式协议

**SLA 与 SLO 的关系**：
- SLA 是外部承诺,SLO 是内部目标
- SLO 应比 SLA 更严格（留有缓冲）
- 建议：SLO = SLA + 安全边际（如 SLA 99.9% -> SLO 99.95%）

**SLA 示例**：
```yaml
# 电商订单服务 SLA
availability:
  target: 99.9%
  window: 30d
  penalty:
    - range: "99.0% - 99.9%"
      compensation: "10% 服务费用减免"
    - range: "95.0% - 99.0%"
      compensation: "25% 服务费用减免"
    - range: "< 95.0%"
      compensation: "100% 服务费用减免"
```

## SLO 实施流程

### 步骤 1：识别关键服务
**评估维度**：
- 用户可见性：是否直接影响用户体验？
- 收入影响：是否影响核心业务收入？
- 依赖关系：是否被多个服务依赖？
- 故障影响：故障后影响范围多大？

**优先级矩阵**：
| 服务 | 用户可见 | 收入影响 | 依赖度 | 优先级 |
|------|----------|----------|--------|--------|
| 订单服务 | 高 | 高 | 高 | P0 |
| 支付服务 | 高 | 高 | 中 | P0 |
| 搜索服务 | 高 | 中 | 中 | P1 |
| 通知服务 | 中 | 低 | 低 | P2 |

### 步骤 2：定义 SLI
**用户旅程分析**：
```
用户 -> 搜索商品 -> 查看详情 -> 加入购物车 -> 下单 -> 支付 -> 履约
       ↓           ↓           ↓             ↓      ↓      ↓
     搜索延迟    页面加载     响应时间      可用性  成功率  交付时长
```

**SLI 选择清单**：
- [ ] 可用性 SLI：请求成功率
- [ ] 延迟 SLI：响应时间分位数
- [ ] 吞吐量 SLI：QPS/RPS
- [ ] 错误率 SLI：失败请求占比
- [ ] 数据质量 SLI：数据准确性（如适用）
- [ ] 自定义 SLI：业务特定指标（如订单履约时长）

### 步骤 3：设定 SLO 目标
**基于历史数据**：
```promql
# 查询过去 30 天可用性
sum(rate(http_requests_total{status!~"5.."}[30d])) /
sum(rate(http_requests_total[30d])) * 100

# 查询过去 30 天 P95 延迟
histogram_quantile(0.95,
  sum(rate(http_request_duration_seconds_bucket[30d])) by (le)
)
```

**目标设定策略**：
- 激进目标：历史最佳值 + 10% 提升
- 保守目标：历史中位数
- 渐进式：从保守目标开始,每季度提升

**示例 SLO 表**：
| 服务 | SLI | 当前值 | SLO 目标 | SLA 承诺 |
|------|-----|--------|----------|----------|
| 订单服务 | 可用性 | 99.85% | 99.9% | 99.5% |
| 订单服务 | P95 延迟 | 180ms | 200ms | 300ms |
| 支付服务 | 成功率 | 99.92% | 99.95% | 99.5% |
| 搜索服务 | P95 延迟 | 250ms | 300ms | 500ms |

### 步骤 4：实现监控
**SLO 监控架构**：
```
Prometheus（指标采集）
  -> Recording Rules（预计算 SLO）
  -> Alertmanager（告警）
  -> Grafana（可视化）
```

**Recording Rules 示例**：
```yaml
# slo-recording-rules.yaml
groups:
  - name: slo_availability
    rules:
      - record: slo:availability:ratio
        expr: |
          sum(rate(http_requests_total{status!~"5.."}[5m])) by (service) /
          sum(rate(http_requests_total[5m])) by (service)

      - record: slo:availability:ratio:30d
        expr: |
          sum(rate(http_requests_total{status!~"5.."}[30d])) by (service) /
          sum(rate(http_requests_total[30d])) by (service)

  - name: slo_latency
    rules:
      - record: slo:latency:p95
        expr: |
          histogram_quantile(0.95,
            sum(rate(http_request_duration_seconds_bucket[5m])) by (le, service)
          )

      - record: slo:latency:p99
        expr: |
          histogram_quantile(0.99,
            sum(rate(http_request_duration_seconds_bucket[5m])) by (le, service)
          )
```

**Grafana Dashboard 示例**：
```yaml
# SLO 概览 Dashboard
panels:
  - title: "可用性 SLO（30 天）"
    type: gauge
    targets:
      - expr: slo:availability:ratio:30d{service="order-service"} * 100
    thresholds:
      - value: 99.9
        color: green
      - value: 99.0
        color: yellow
      - value: 0
        color: red

  - title: "P95 延迟 SLO（5 分钟）"
    type: time-series
    targets:
      - expr: slo:latency:p95{service="order-service"} * 1000
    thresholds:
      - value: 200
        color: green
      - value: 300
        color: yellow
      - value: 0
        color: red
```

### 步骤 5：配置告警
**错误预算告警**：
```yaml
# 错误预算消耗速率告警
- alert: ErrorBudgetBurningFast
  expr: |
    (
      1 - slo:availability:ratio:30d{service="order-service"}
    ) / (1 - 0.999) > 0.1
  for: 5m
  labels:
    severity: P1
  annotations:
    summary: "订单服务错误预算消耗过快"
    description: "错误预算消耗速率超过 10%,当前可用性 {{ $value | humanizePercentage }}"

# 可用性 SLO 违反告警
- alert: AvailabilitySLOBreach
  expr: slo:availability:ratio:30d{service="order-service"} < 0.999
  for: 5m
  labels:
    severity: P0
  annotations:
    summary: "订单服务可用性 SLO 违反"
    description: "30 天可用性 {{ $value | humanizePercentage }} 低于目标 99.9%"
```

**多窗口告警策略**：
```yaml
# 短期窗口（快速告警）
- alert: HighErrorRate_5m
  expr: slo:availability:ratio{service="order-service"} < 0.99
  for: 5m
  labels:
    severity: P1

# 中期窗口（持续问题）
- alert: HighErrorRate_1h
  expr: slo:availability:ratio:1h{service="order-service"} < 0.995
  for: 1h
  labels:
    severity: P1

# 长期窗口（SLO 违反）
- alert: HighErrorRate_30d
  expr: slo:availability:ratio:30d{service="order-service"} < 0.999
  for: 5m
  labels:
    severity: P0
```

## 错误预算（Error Budget）

### 概念
**定义**：SLO 违反前允许的最大故障时间/错误数

**计算公式**：
```
错误预算 = 1 - SLO 目标
每月错误预算分钟数 = (1 - SLO) * 30 天 * 24 小时 * 60 分钟
```

**示例**：
```
SLO = 99.9%
错误预算 = 1 - 99.9% = 0.1%
每月错误预算分钟数 = 0.1% * 30 * 24 * 60 = 43.2 分钟
```

### 错误预算策略
**使用场景**：
1. **功能发布决策**：
   - 预算充足（> 50%）：可以加速功能发布
   - 预算紧张（< 20%）：暂停非紧急发布,优先修复稳定性问题
   - 预算耗尽（< 0%）：冻结所有发布,全力恢复 SLO

2. **故障优先级**：
   - 预算充足：P1 事故
   - 预算紧张：P0 事故（消耗速率 > 10%/天）
   - 预算耗尽：P0 事故 + 复盘改进

3. **容量规划**：
   - 预算消耗快：需要扩容/优化性能
   - 预算消耗慢：可以推迟扩容,节省成本

**错误预算 Dashboard**：
```yaml
panels:
  - title: "错误预算剩余"
    type: gauge
    targets:
      - expr: |
          (1 - (1 - slo:availability:ratio:30d{service="order-service"}) / (1 - 0.999)) * 100
    thresholds:
      - value: 50
        color: green
      - value: 20
        color: yellow
      - value: 0
        color: red

  - title: "错误预算消耗速率"
    type: stat
    targets:
      - expr: |
          (1 - slo:availability:ratio:1d{service="order-service"}) / (1 - 0.999) * 100
    description: "每天消耗错误预算的百分比"
```

## SLO 评审与演进

### 定期评审（每月）
**议程**：
1. SLO 达成情况回顾
   - 哪些 SLO 达成？
   - 哪些 SLO 违反？
   - 违反根因分析

2. 错误预算使用分析
   - 预算消耗速率
   - 主要消耗来源（故障、发布、维护）
   - 预算预测

3. SLO 适应性评估
   - 当前 SLO 是否合理？
   - 是否需要调整目标？
   - 用户反馈如何？

4. 改进计划
   - 稳定性改进措施
   - 监控完善
   - 团队培训

**评审报告模板**：
```markdown
# SLO 月度评审报告 - YYYY-MM

## SLO 达成情况
| 服务 | SLI | 目标 | 实际 | 状态 |
|------|-----|------|------|------|
| 订单服务 | 可用性 | 99.9% | 99.85% | 违反 |
| 订单服务 | P95 延迟 | 200ms | 180ms | 达成 |

## 错误预算分析
- 月初预算：43.2 分钟
- 本月消耗：8.5 分钟（19.7%）
- 剩余预算：34.7 分钟（80.3%）

## 主要事件
1. [事件 ID] 订单服务数据库连接池耗尽（消耗预算 5 分钟）
2. [事件 ID] 支付网关超时（消耗预算 2 分钟）

## 改进措施
1. 数据库连接池优化（优先级：P1，负责人：XXX，截止：YYYY-MM-DD）
2. 支付网关熔断器配置（优先级：P1，负责人：XXX，截止：YYYY-MM-DD）

## 下月计划
- SLO 目标调整：可用性 99.9% -> 99.95%（渐进提升）
```

### SLO 调整策略
**提升 SLO**：
- 条件：连续 3 个月达成当前 SLO
- 步骤：小幅提升（+ 0.05% - 0.1%），观察 1 个月
- 风险：避免过快提升导致频繁违反

**降低 SLO**：
- 条件：连续 3 个月违反当前 SLO
- 步骤：与业务方沟通，重新评估用户容忍度
- 风险：降低用户信任，影响品牌形象

**新增 SLO**：
- 场景：新增关键功能、新服务上线
- 步骤：试运行 1 个月（非正式），收集数据后正式设定

## SLO 工具与平台

### 开源工具
**SLO 计算框架**：
- Pyrra：Kubernetes 原生 SLO 管理 https://pyrra.dev/
- Sloth：SLO 生成器（Prometheus）https://slok.github.io/sloth/
- OpenSLO：SLO 规范标准 https://openslo.com/

**示例：Pyrra SLO 配置**：
```yaml
apiVersion: pyrra.dev/v1alpha1
kind: ServiceLevelObjective
metadata:
  name: order-service-availability
  namespace: monitoring
spec:
  target: 99.9
  window: 30d
  serviceLevelIndicator:
    ratio:
      total:
        metric: http_requests_total{service="order-service"}
      errors:
        metric: http_requests_total{service="order-service",status=~"5.."}
  alerting:
    name: OrderServiceAvailability
    labels:
      severity: critical
```

### 商业平台
- Datadog SLO Monitor
- New Relic Service Level Management
- Google Cloud SLO Monitoring
- AWS CloudWatch ServiceLens

## 常见失败模式

### 1. SLO 设计问题
- **SLO 过于宽松**：99.9% 可用性,实际用户已投诉（实际需要 99.95%）
- **SLO 过于严格**：99.999% 可用性,成本过高且无业务价值
- **缺少关键 SLI**：只监控可用性,忽略延迟和数据质量
- **SLO 与业务脱节**：技术 SLO 达成,但业务 KPI 下降

### 2. 监控实施问题
- **指标采集不准确**：缺少关键路径埋点,SLO 计算失真
- **窗口选择不当**：短期窗口告警噪音大,长期窗口响应慢
- **缺少错误预算**：有 SLO 但无错误预算,无法指导决策
- **Dashboard 不清晰**：SLO 信息淹没在海量指标中

### 3. 组织流程问题
- **SLO 制定后不评审**：SLO 永不调整,与实际需求脱节
- **SLO 违反无行动**：反复违反但无改进措施,SLO 形同虚设
- **缺少跨团队对齐**：服务间 SLO 不匹配,下游 SLO 无法支撑上游
- **SLO 与绩效考核挂钩不当**：导致团队隐瞒问题、调整指标

### 4. 技术债务问题
- **历史系统无监控**：无法计算 SLO,只能靠用户反馈
- **依赖外部服务无 SLO**：第三方服务故障影响自身 SLO,但无 SLA 约束
- **缺少自动化**：SLO 计算、告警、Dashboard 维护成本高

## 验收标准

### 功能验收
- [ ] 关键服务（Top 5）SLO 定义 100%
- [ ] SLO Dashboard 部署 100%
- [ ] SLO 告警规则配置 100%
- [ ] 错误预算计算和可视化
- [ ] SLO 评审流程文档化

### 质量验收
- [ ] SLO 可达成率 >= 90%（3 个月观察期）
- [ ] 误报率 < 5%（错误告警 / 总告警）
- [ ] SLO 违反检测时间 < 5 分钟
- [ ] SLO 数据准确性 >= 99%（与实际业务数据对比）

### 运营验收
- [ ] 每月 SLO 评审会议召开
- [ ] SLO 评审报告产出
- [ ] 团队 SLO 培训覆盖率 100%
- [ ] SLO 文档完整性 >= 90%

## 参考资源

### 经典著作
- Google SRE Book - Chapter 4: Service Level Objectives
- Site Reliability Workbook - Chapter 2: Implementing SLOs
- Building Secure and Reliable Systems - Chapter 8: Reliability

### 最佳实践
- The Art of SLOs（Google）https://sre.google/workbook/implementing-slos/
- SLO Adoption Framework（Datadog）
- Error Budget Best Practices（PagerDuty）

### 工具与框架
- OpenSLO 规范：https://github.com/OpenSLO/OpenSLO
- Pyrra：https://pyrra.dev/
- Sloth：https://github.com/slok/sloth
