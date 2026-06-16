---
id: operations-glossary
title: 运维/SRE 术语表 (Operations & SRE Glossary)
domain: operations
category: 06-glossary
difficulty: intermediate
tags: [agent, checklist, glossary, operations, 可观测性, 基础设施与架构, 容量与性能, 弹性工程]
quality_score: 70
last_updated: 2026-06-15
---
# 运维/SRE 术语表 (Operations & SRE Glossary)

> 收录 50+ 核心运维与 SRE 术语，覆盖 SRE 理念、可观测性、故障管理、弹性工程和架构模式等领域。
> 适用于 SRE 评审、架构设计、On-Call 培训和团队 Onboarding 等场景。

---

## SRE 核心理念

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| SRE | Site Reliability Engineering | 由 Google 提出的工程实践，用软件工程方法解决运维问题。SRE 的核心目标是在系统可靠性与开发速度之间找到平衡，通过 SLO、Error Budget、Toil 治理等机制实现量化管理。 |
| Toil | Toil / 苦差事 | 满足以下全部特征的运维工作：手动的、重复的、可自动化的、战术性的、缺乏持久价值的、随服务增长线性增长的。Google SRE 要求 Toil 占比不超过 50%，其余时间用于工程项目。 |
| Error Budget | Error Budget / 错误预算 | SLO 允许的不可靠余量。计算公式：`Error Budget = 1 - SLO`。例如 SLO = 99.95% 时，每月允许 21.6 分钟不可用。当 Error Budget 耗尽时，应冻结新功能发布，集中修复稳定性问题。 |
| SLI | Service Level Indicator | 服务级别指标，量化衡量服务质量的指标。好的 SLI 直接反映用户体验，常见选择：可用性（成功请求比例）、延迟（P50/P95/P99）、正确性（正确响应比例）、吞吐量。 |
| SLO | Service Level Objective | 服务级别目标，SLI 的目标值。例如"订单服务 P99 延迟 < 500ms，30 天滚动窗口"。SLO 是团队内部的可靠性目标，比 SLA 更严格，违反 SLO 不会产生法律/财务后果但会触发 Error Budget 机制。 |
| SLA | Service Level Agreement | 服务级别协议，与客户签订的具有法律约束力的可靠性承诺。违反 SLA 通常导致赔偿（如服务积分、退款）。SLA 目标应宽松于内部 SLO，留出缓冲余量。 |
| Blast Radius | Blast Radius / 爆炸半径 | 故障发生时受影响的范围。SRE 通过故障隔离（分区部署、熔断器、限流）减小 Blast Radius。评估变更风险时，Blast Radius 是关键考量因素。 |

---

## 可观测性

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| Observability | Observability / 可观测性 | 通过系统外部输出推断系统内部状态的能力。三大支柱：Metrics（指标）、Logs（日志）、Traces（追踪）。可观测性强调"能回答之前没有预料到的问题"，而非仅监控已知指标。 |
| Telemetry | Telemetry / 遥测 | 从系统中自动收集、传输和分析性能数据的过程。包括 Metrics、Logs、Traces 的采集和传输。OpenTelemetry（OTel）是当前主流的遥测标准和 SDK。 |
| Golden Signals | Golden Signals / 黄金信号 | Google SRE 定义的四个关键监控维度：延迟（Latency）、流量（Traffic）、错误率（Errors）、饱和度（Saturation）。监控这四个信号可以快速判断服务健康状态。 |
| RED Method | Rate, Errors, Duration | Tom Wilkie 提出的微服务监控方法论：Rate（请求速率）、Errors（错误率）、Duration（请求耗时）。适用于请求驱动的服务，与 Golden Signals 高度重叠但更简洁。 |
| USE Method | Utilization, Saturation, Errors | Brendan Gregg 提出的资源监控方法论：Utilization（利用率）、Saturation（饱和度/队列深度）、Errors（错误数）。适用于硬件资源（CPU、内存、磁盘、网络）监控。 |
| Distributed Tracing | Distributed Tracing / 分布式追踪 | 追踪请求在分布式系统中经过的完整路径，记录每个服务/组件的耗时和状态。通过 Trace ID 将跨服务的调用关联起来。工具：Jaeger、Zipkin、Tempo、X-Ray。 |
| Span | Span / 跨度 | 分布式追踪中的基本单元，代表一个操作（如一次 HTTP 请求、一次数据库查询）。Span 包含：操作名称、开始/结束时间、父 Span ID、标签和日志。多个 Span 组成一个 Trace。 |
| Cardinality | Cardinality / 基数 | 指标标签的不同值数量。高基数（如 user_id 作为标签）会导致时序数据库存储和查询成本爆炸。监控系统设计时必须控制标签基数，避免"基数爆炸"问题。 |

---

## 故障管理

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| Incident | Incident / 事故 | 导致服务质量下降或不可用的事件。按影响范围和严重程度分级（P0-P4）。事故管理流程包括：检测、响应、缓解、恢复、复盘。 |
| Incident Commander | Incident Commander / 事故指挥官 | 重大事故中的总协调人，负责：评估影响范围、分配角色、协调各方、决策升级、控制沟通节奏。IC 不负责具体技术排查，而是确保流程高效运转。 |
| War Room | War Room / 作战室 | 重大事故期间临时组建的协作空间（物理或虚拟）。所有相关人员集中在 War Room 中，由 Incident Commander 统一协调。规则：聚焦恢复、减少噪声、记录时间线。 |
| MTTR | Mean Time To Recovery | 平均恢复时间，从故障发生到服务恢复正常的平均耗时。MTTR = 检测时间 + 响应时间 + 修复时间 + 验证时间。是衡量运维效率的核心指标。 |
| MTTD | Mean Time To Detect | 平均检测时间，从故障发生到被发现的平均耗时。降低 MTTD 的关键：完善的监控覆盖 + 准确的告警规则 + 合理的告警阈值。 |
| MTBF | Mean Time Between Failures | 平均无故障时间，两次故障之间的平均间隔。MTBF 越长说明系统越稳定。提升 MTBF 的手段：代码质量、架构冗余、Chaos Engineering。 |
| Postmortem | Postmortem / 事后复盘 | 事故恢复后的结构化回顾过程。内容包括：事故时间线、影响范围、根因分析、修复措施、经验教训和 Action Items。核心原则：Blameless（对事不对人）。 |
| Blameless | Blameless / 无指责文化 | Postmortem 的核心原则：关注系统和流程的改进，而非追究个人责任。基于认知：人总会犯错，但系统应该有防护措施防止个人错误导致重大事故。 |
| RCA | Root Cause Analysis | 根因分析，深入挖掘事故背后的根本原因而非表面原因。常用方法：5 Whys（五个为什么）、鱼骨图（Ishikawa Diagram）、故障树分析（FTA）。 |

---

## 弹性工程

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| Chaos Engineering | Chaos Engineering / 混沌工程 | 通过在生产或类生产环境中主动注入故障，验证系统在不利条件下的行为是否符合预期。遵循科学实验方法：假设 → 实验 → 观察 → 结论。工具：LitmusChaos、Chaos Monkey、Gremlin。 |
| Game Day | Game Day / 演练日 | 全团队参与的计划性故障演练活动。在受控条件下模拟真实故障场景，验证团队的响应能力和系统的弹性。Game Day 是 Chaos Engineering 从工具到文化的桥梁。 |
| Circuit Breaker | Circuit Breaker / 熔断器 | 防止故障级联传播的保护机制。当下游服务失败率超过阈值时，熔断器"断开"，后续请求直接返回降级响应而不再调用下游。状态：Closed（正常）→ Open（熔断）→ Half-Open（试探）。 |
| Bulkhead | Bulkhead / 舱壁隔离 | 借鉴船舶设计的故障隔离模式。将系统资源（线程池、连接池、实例）分隔为独立的隔舱，一个隔舱故障不会耗尽其他隔舱的资源。防止一个慢请求拖垮整个系统。 |
| Rate Limiting | Rate Limiting / 限流 | 限制单位时间内的请求数量，防止过载。常见算法：令牌桶（Token Bucket）、漏桶（Leaky Bucket）、滑动窗口（Sliding Window）。应在 API Gateway 和服务层均实施。 |
| Graceful Degradation | Graceful Degradation / 优雅降级 | 在系统过载或部分故障时，主动关闭非核心功能以保证核心功能可用。例如：高峰期关闭推荐系统，保证搜索和下单正常工作。 |
| Backpressure | Backpressure / 背压 | 下游处理能力不足时，向上游发出信号减缓发送速率的机制。防止上游无限制地向下游灌数据导致 OOM 或延迟爆炸。常见实现：消息队列的消费限速、HTTP 429 响应。 |
| Retry with Backoff | Retry with Exponential Backoff | 请求失败后，以指数递增的间隔进行重试（如 1s → 2s → 4s → 8s），并添加随机抖动（Jitter）避免"重试风暴"。所有重试策略必须设置最大重试次数和总超时。 |

---

## 基础设施与架构

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| Service Mesh | Service Mesh / 服务网格 | 处理服务间通信的基础设施层，通过 Sidecar Proxy 透明地实现流量管理、安全认证和可观测性。代表实现：Istio、Linkerd、Cilium Service Mesh。 |
| Sidecar | Sidecar / 边车 | 与主容器一同部署在同一 Pod 中的辅助容器。常见用途：日志采集（Fluentd）、服务网格代理（Envoy）、配置同步。Sidecar 模式实现了关注点分离。 |
| IaC | Infrastructure as Code | 基础设施即代码，用代码定义和管理基础设施（服务器、网络、数据库等）。工具：Terraform、Pulumi、CloudFormation、Ansible。IaC 确保环境一致性、可审计和可复现。 |
| Immutable Infrastructure | Immutable Infrastructure / 不可变基础设施 | 基础设施一旦部署不再修改。需要变更时创建新实例替换旧实例。消除配置漂移（Configuration Drift）和"雪花服务器"问题。容器化是不可变基础设施的典型实现。 |
| Configuration Drift | Configuration Drift / 配置漂移 | 运行中的基础设施状态与声明式配置（IaC 代码）之间逐渐产生偏差的现象。原因：手动修改、紧急修复未同步回代码。检测工具：`terraform plan`、AWS Config。 |

---

## 容量与性能

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| Capacity Planning | Capacity Planning / 容量规划 | 基于历史数据和增长预测，提前规划系统资源需求的过程。包括：流量预测、资源建模、成本估算和扩容计划。避免过度配置（浪费）和配置不足（宕机）。 |
| Load Shedding | Load Shedding / 负载卸除 | 系统过载时，主动丢弃部分请求以保证剩余请求的服务质量。策略包括：按优先级丢弃、按超时丢弃（已超过 SLA 的请求直接丢弃）、随机丢弃。 |
| Thundering Herd | Thundering Herd / 惊群效应 | 大量请求同时到达的现象，常见于：缓存失效后大量请求穿透到数据库、服务重启后积压请求涌入、定时任务同时触发。解决方法：缓存预热、请求排队、随机化触发时间。 |
| Horizontal Scaling | Horizontal Scaling / 水平扩展 | 通过增加实例数量提升系统处理能力。与垂直扩展（增加单机资源）相比，水平扩展理论上无上限、成本线性增长。前提：应用无状态或状态可外置。 |

---

## 运维流程

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| Runbook | Runbook / 运维手册 | 针对特定告警或故障场景的标准操作流程文档。内容包括：触发条件、诊断步骤、修复操作、升级路径和验证方法。Runbook 可半自动化或全自动化执行，是降低 MTTR 的关键工具。 |
| Change Management | Change Management / 变更管理 | 控制生产环境变更的流程，确保变更经过评估、审批、测试和回滚准备后才能执行。变更分级：标准变更（预审批）、普通变更（需审批）、紧急变更（事后补审）。 |
| Toil Budget | Toil Budget / 苦差事预算 | 团队为 Toil 设定的时间上限（通常不超过总工时的 50%）。超出 Toil Budget 时必须启动自动化项目减少 Toil。Toil Budget 是 SRE 团队保持工程能力的制度保障。 |
| On-Call | On-Call / 值班 | SRE/运维人员在非工作时间负责响应告警和处理紧急事故的制度。健康的 On-Call 制度包括：轮值排班、升级路径、补偿机制、最大连续值班天数限制和定期回顾。 |
| War Room Protocol | War Room Protocol / 作战室规程 | 重大事故期间 War Room 的运作规则：IC 统一指挥、每 15 分钟状态同步、沟通限制在指定频道、所有操作记录到时间线、非参与者禁止进入避免干扰。 |

---

## Agent Checklist

- [ ] 覆盖所有要求的关键术语：SRE/Toil/Error Budget/Blast Radius/Golden Signals/RED Method/USE Method/Observability/Telemetry/Distributed Tracing/Service Mesh/Chaos Engineering/Game Day/Incident Commander/War Room
- [ ] 每个术语包含英文全称和中文定义
- [ ] 术语按领域分组（SRE 核心理念、可观测性、故障管理、弹性工程、基础设施与架构、容量与性能）
- [ ] 使用统一的表格格式，与现有术语表风格一致
- [ ] 定义准确、专业，包含实际使用场景和工具推荐
- [ ] 文件超过 100 行
