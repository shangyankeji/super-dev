---
id: architecture-glossary
title: 架构术语表
domain: architecture
category: 06-glossary
difficulty: intermediate
tags: [architecture, devops, glossary, 事件与数据模式, 分布式系统理论, 可观测性与, 可靠性工程, 微服务与服务治理]
quality_score: 70
last_updated: 2026-06-15
---
# 架构术语表

> 收录软件架构领域核心术语，涵盖分布式系统、微服务、设计原则、架构模式等方向。
> 适用于团队 Onboarding、架构评审对齐、文档编写时的术语参考。

---

## 分布式系统理论

| 术语 | 英文全称 | 定义 |
|------|----------|------|
| CAP 定理 | CAP Theorem | 分布式系统最多同时满足一致性（Consistency）、可用性（Availability）、分区容错性（Partition Tolerance）中的两项。由 Eric Brewer 于 2000 年提出。 |
| BASE | Basically Available, Soft state, Eventually consistent | 与 ACID 对应的分布式系统设计理念：基本可用、软状态、最终一致性。适用于对可用性要求高于强一致性的场景。 |
| ACID | Atomicity, Consistency, Isolation, Durability | 数据库事务的四大特性：原子性、一致性、隔离性、持久性。传统关系型数据库的核心保证。 |
| 最终一致性 | Eventual Consistency | 分布式系统中，数据在没有新写入的情况下，所有副本最终会达到一致状态。BASE 的核心概念之一。 |
| 拜占庭容错 | Byzantine Fault Tolerance (BFT) | 系统能够在部分节点发送错误或恶意信息的情况下仍然达成共识。常见于区块链和高安全性系统。 |
| 向量时钟 | Vector Clock | 用于检测分布式系统中事件因果关系和并发冲突的逻辑时钟算法。每个节点维护一个计数器向量。 |
| Quorum | Quorum | 分布式一致性协议中，执行读写操作所需的最少节点数。常见公式：W + R > N（写节点 + 读节点 > 总节点）。 |

## 领域驱动设计（DDD）

| 术语 | 英文全称 | 定义 |
|------|----------|------|
| DDD | Domain-Driven Design | 以业务领域为核心驱动软件设计的方法论，强调通用语言、限界上下文和领域模型。由 Eric Evans 于 2003 年提出。 |
| 限界上下文 | Bounded Context | DDD 中定义领域模型边界的核心概念。同一术语在不同上下文中可能有不同含义，上下文之间通过集成映射通信。 |
| 聚合根 | Aggregate Root | 聚合中唯一允许外部直接引用的实体，作为聚合的访问入口，保证聚合内的不变量约束。 |
| 领域事件 | Domain Event | 表示领域中已发生的重要事实。不可变，包含事件发生时间和相关数据。是事件驱动架构的基础。 |
| 通用语言 | Ubiquitous Language | 开发团队与业务专家共同使用的统一术语体系，贯穿代码、文档和沟通。 |
| 上下文映射 | Context Map | 描述多个限界上下文之间关系和集成模式的全局视图。常见关系：共享内核、防腐层、开放主机服务等。 |

## 事件与数据模式

| 术语 | 英文全称 | 定义 |
|------|----------|------|
| CQRS | Command Query Responsibility Segregation | 将读操作（Query）和写操作（Command）的模型分离。读模型针对查询优化，写模型针对业务规则优化。常与 Event Sourcing 搭配使用。 |
| Event Sourcing | Event Sourcing | 以事件序列而非当前状态作为数据存储的核心模式。系统状态通过重放事件序列恢复。提供完整审计轨迹。 |
| Saga 模式 | Saga Pattern | 通过一系列本地事务和补偿操作实现跨服务的分布式事务。分为编排式（Choreography）和协调式（Orchestration）两种。 |
| CDC | Change Data Capture | 捕获数据库变更并将其传播到下游系统的技术。常用工具：Debezium、Canal。用于数据同步和事件驱动集成。 |
| Outbox 模式 | Transactional Outbox | 将领域事件与业务数据在同一事务中写入 Outbox 表，再由异步进程发布到消息队列，解决本地事务与消息发送的一致性问题。 |

## 微服务与服务治理

| 术语 | 英文全称 | 定义 |
|------|----------|------|
| Service Mesh | Service Mesh | 处理服务间通信的基础设施层。以 Sidecar 代理的形式透明地提供负载均衡、服务发现、流量管理、安全和可观测性。代表实现：Istio、Linkerd。 |
| API Gateway | API Gateway | 作为所有客户端请求的统一入口，处理路由、认证、限流、协议转换、请求聚合等横切关注点。代表实现：Kong、APISIX、AWS API Gateway。 |
| Circuit Breaker | Circuit Breaker | 断路器模式。当依赖服务故障率超过阈值时自动切断请求，防止级联故障。包含关闭、打开、半开三种状态。 |
| Bulkhead | Bulkhead Pattern | 舱壁模式。将系统资源隔离为独立的隔离区，某一部分故障不会耗尽整体资源。常通过线程池或信号量实现。 |
| Sidecar | Sidecar Pattern | 将辅助功能（日志、监控、安全、网络）部署为独立进程/容器，与主应用容器共享生命周期但独立运行。Service Mesh 的核心实现模式。 |
| 服务发现 | Service Discovery | 服务实例自动注册和查找的机制。分为客户端发现（如 Eureka）和服务端发现（如 Kubernetes Service）两种模式。 |
| 背压 | Backpressure | 当下游消费速度跟不上上游生产速度时，向上游发出减速信号的流控机制。防止系统过载和 OOM。 |

## 架构演进模式

| 术语 | 英文全称 | 定义 |
|------|----------|------|
| Strangler Fig | Strangler Fig Pattern | 绞杀者模式。通过逐步用新系统替换旧系统的功能模块来完成系统迁移，避免一次性大规模重写的风险。名称来自热带绞杀榕。 |
| BFF | Backend for Frontend | 为特定前端（Web/Mobile/IoT）定制的后端服务层。根据不同前端的数据需求进行聚合和裁剪，避免通用 API 的过度获取。 |
| Anti-Corruption Layer | Anti-Corruption Layer (ACL) | 防腐层。在新系统与遗留系统之间建立转换层，隔离遗留系统的数据模型和接口对新系统的污染。DDD 上下文映射中的关键模式。 |
| Feature Toggle | Feature Toggle / Feature Flag | 功能开关。通过配置而非代码部署来控制功能的启用/禁用。支持灰度发布、A/B 测试和快速回滚。 |

## 设计原则

| 术语 | 英文全称 | 定义 |
|------|----------|------|
| SOLID | Single responsibility, Open-closed, Liskov substitution, Interface segregation, Dependency inversion | 面向对象设计五大原则：单一职责、开闭原则、里氏替换、接口隔离、依赖倒置。由 Robert C. Martin 整理推广。 |
| 12-Factor App | Twelve-Factor App | 构建云原生 SaaS 应用的十二条方法论：代码库、依赖、配置、后端服务、构建/发布/运行、进程、端口绑定、并发、易处置、开发/生产等价、日志、管理进程。 |
| DRY | Don't Repeat Yourself | 避免重复原则。系统中的每一项知识都应该有且仅有一个明确的、权威的表示。 |
| KISS | Keep It Simple, Stupid | 保持简单原则。系统设计应尽量简单，避免不必要的复杂性。 |
| YAGNI | You Aren't Gonna Need It | 不要提前实现当前不需要的功能。源自极限编程（XP），避免过度设计。 |

## 架构风格

| 术语 | 英文全称 | 定义 |
|------|----------|------|
| Clean Architecture | Clean Architecture | 由 Robert C. Martin 提出的分层架构。从外到内：框架与驱动 → 接口适配器 → 应用用例 → 领域实体。依赖规则：外层依赖内层，内层不知道外层。 |
| Hexagonal Architecture | Hexagonal Architecture (Ports & Adapters) | 六边形架构。核心业务逻辑通过端口（Port）定义接口，适配器（Adapter）实现与外部系统的交互。使业务逻辑独立于技术实现。由 Alistair Cockburn 提出。 |
| Onion Architecture | Onion Architecture | 洋葱架构。以领域模型为核心，向外依次为领域服务、应用服务、基础设施。与 Clean Architecture 理念相似，由 Jeffrey Palermo 提出。 |
| Microkernel | Microkernel Architecture (Plugin Architecture) | 微内核架构。系统由核心系统和可插拔的插件模块组成。核心提供最小功能集，扩展通过插件实现。适用于需要高扩展性的产品。 |
| EDA | Event-Driven Architecture | 事件驱动架构。系统组件通过事件进行异步通信。分为简单事件处理、复杂事件处理（CEP）和事件流处理三种拓扑。 |
| SOA | Service-Oriented Architecture | 面向服务的架构。通过定义良好的服务接口实现松耦合的系统集成。ESB（企业服务总线）是其典型基础设施。 |
| Serverless | Serverless Architecture | 无服务器架构。开发者无需管理服务器基础设施，按实际调用量付费。包括 FaaS（函数即服务）和 BaaS（后端即服务）。 |
| CNCF 云原生 | Cloud Native | 利用云计算优势构建和运行应用的方法。核心技术：容器、服务网格、微服务、不可变基础设施、声明式 API。 |

## 可靠性工程

| 术语 | 英文全称 | 定义 |
|------|----------|------|
| SLI | Service Level Indicator | 服务质量的量化度量指标，如请求延迟、错误率、吞吐量。是 SLO 的基础数据来源。 |
| SLO | Service Level Objective | 基于 SLI 设定的目标值或范围，如"99.9% 的请求延迟 < 200ms"。是内部质量目标。 |
| SLA | Service Level Agreement | 服务提供商与客户之间的正式协议，包含 SLO 及违约后果（如赔偿条款）。 |
| 混沌工程 | Chaos Engineering | 在生产或类生产环境中主动注入故障，验证系统韧性的实践方法。代表工具：Chaos Monkey、Litmus、ChaosBlade。 |
| 金丝雀发布 | Canary Release | 将新版本先部署给一小部分用户，观察指标正常后再逐步全量。降低发布风险。 |
| 蓝绿部署 | Blue-Green Deployment | 维护两套完整环境（蓝/绿），通过流量切换实现零停机发布和快速回滚。 |
| 滚动更新 | Rolling Update | 逐批替换旧版本实例为新版本，始终保持部分实例在线服务。Kubernetes 默认的部署策略。 |

## 可观测性与 DevOps

| 术语 | 英文全称 | 定义 |
|------|----------|------|
| 可观测性 | Observability | 通过系统外部输出（指标、日志、追踪）推断系统内部状态的能力。三大支柱：Metrics、Logging、Tracing。 |
| OpenTelemetry | OpenTelemetry (OTel) | CNCF 开源的可观测性框架，提供统一的 API 和 SDK 用于采集指标、日志和追踪数据。由 OpenTracing 和 OpenCensus 合并而来。 |
| 不可变基础设施 | Immutable Infrastructure | 服务器部署后不做任何修改，变更通过替换整个实例实现。搭配容器和基础设施即代码使用，消除配置漂移。 |
| IaC | Infrastructure as Code | 基础设施即代码。通过声明式或命令式代码定义和管理基础设施。代表工具：Terraform、Pulumi、CloudFormation。 |
| GitOps | GitOps | 以 Git 仓库作为基础设施和应用部署的唯一事实来源。变更通过 PR 触发，由自动化工具同步到集群。代表工具：ArgoCD、Flux。 |

## 数据架构

| 术语 | 英文全称 | 定义 |
|------|----------|------|
| 数据湖 | Data Lake | 以原始格式存储大量结构化和非结构化数据的集中式存储。支持多种分析和处理引擎直接读取。 |
| 数据网格 | Data Mesh | 将数据所有权去中心化到业务领域团队的架构范式。四大原则：领域所有权、数据即产品、自助平台、联邦计算治理。 |
| 读写分离 | Read-Write Splitting | 将数据库读请求路由到只读副本、写请求路由到主库的模式。提升读性能和系统整体吞吐量。 |
| 多级缓存 | Multi-Level Cache | 在客户端、CDN、API 网关、应用进程内、分布式缓存等多个层级设置缓存，逐级降低后端压力。 |

## 安全架构

| 术语 | 英文全称 | 定义 |
|------|----------|------|
| 零信任 | Zero Trust Architecture | 不隐式信任任何用户或设备，每次访问都需要验证身份和授权。核心原则："永不信任，始终验证"。 |
| mTLS | Mutual TLS | 双向 TLS 认证。客户端和服务端互相验证对方证书，确保通信双方身份可信。Service Mesh 中常用于服务间通信加密。 |
| OAuth 2.0 | Open Authorization 2.0 | 行业标准的授权框架。允许第三方应用在用户授权下访问资源，无需暴露用户凭据。定义了授权码、隐式、密码、客户端凭据四种授权流程。 |
| OIDC | OpenID Connect | 基于 OAuth 2.0 的身份认证层。在授权的基础上增加了身份验证能力，返回 ID Token（JWT 格式）标识用户身份。 |
| RBAC | Role-Based Access Control | 基于角色的访问控制。将权限赋予角色，再将角色分配给用户。简化大规模系统的权限管理。 |
| ABAC | Attribute-Based Access Control | 基于属性的访问控制。根据用户属性、资源属性、环境属性和操作属性动态计算访问权限，比 RBAC 更灵活。 |

## 测试与质量

| 术语 | 英文全称 | 定义 |
|------|----------|------|
| 契约测试 | Contract Testing | 验证服务间接口契约的测试方法。消费者定义期望的接口行为，提供者验证是否满足。代表工具：Pact。 |
| 测试金字塔 | Test Pyramid | 单元测试（底层，数量最多）→ 集成测试（中层）→ 端到端测试（顶层，数量最少）的测试分层策略。由 Mike Cohn 提出。 |
| 影子流量 | Traffic Shadowing / Mirroring | 将生产流量复制一份到新版本服务，但不返回响应给用户。用于在真实流量下验证新版本的正确性和性能。 |

## 性能与扩展

| 术语 | 英文全称 | 定义 |
|------|----------|------|
| 限流 | Rate Limiting | 控制单位时间内的请求数量，保护系统不被过量流量冲垮。常用算法：令牌桶、漏桶、固定窗口、滑动窗口。 |
| 熔断降级 | Circuit Breaking & Degradation | 当依赖服务异常时触发熔断停止调用，同时返回降级响应（缓存数据/默认值/简化功能），保障核心链路可用。 |
| 弹性伸缩 | Auto Scaling | 根据实时负载指标（CPU、内存、请求量）自动增加或减少服务实例数量。包含水平伸缩（实例数）和垂直伸缩（实例规格）。 |
| 读扩散/写扩散 | Fan-out on Read / Fan-out on Write | 社交 Feed 等场景中的两种数据分发策略。写扩散在写入时推送给所有关注者；读扩散在读取时实时聚合。 |
| 一致性哈希 | Consistent Hashing | 将数据和节点映射到同一哈希环上的分布式数据分配算法。节点增减时仅影响相邻区间的数据迁移，最小化数据重分布。 |

---

## Agent Checklist

- [ ] 术语定义准确且与业界共识一致
- [ ] 每个术语包含英文全称以便跨语言沟通
- [ ] 术语覆盖架构评审中常见的核心概念
- [ ] 表格格式统一，便于快速查阅
- [ ] 新增术语前已检查是否与现有条目重复
- [ ] 术语表已同步更新至团队知识库
