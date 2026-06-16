---
id: full-stack-glossary
title: 全栈开发术语表
domain: development
category: 06-glossary
difficulty: intermediate
tags: [development, devops, full, glossary, stack, 前端术语, 后端术语, 安全术语]
quality_score: 70
last_updated: 2026-06-15
---
# 全栈开发术语表

## 概述

本术语表覆盖全栈开发中常用的技术术语、缩写和概念，按主题分类组织。

---

## 通用术语

| 术语 | 英文 | 定义 |
|------|------|------|
| **API** | Application Programming Interface | 应用程序编程接口，定义软件组件之间的交互协议 |
| **REST** | Representational State Transfer | 一种基于 HTTP 的 API 设计风格，使用资源和动词 |
| **GraphQL** | - | Facebook 开发的 API 查询语言，客户端可以精确指定需要的数据 |
| **gRPC** | gRPC Remote Procedure Call | Google 开发的高性能 RPC 框架，使用 Protocol Buffers |
| **SDK** | Software Development Kit | 软件开发工具包 |
| **CLI** | Command Line Interface | 命令行界面 |
| **IDE** | Integrated Development Environment | 集成开发环境 (VS Code, JetBrains) |
| **CI/CD** | Continuous Integration/Delivery | 持续集成/持续交付 |
| **MVP** | Minimum Viable Product | 最小可行产品 |
| **SaaS** | Software as a Service | 软件即服务 |
| **IaC** | Infrastructure as Code | 基础设施即代码 (Terraform, Pulumi) |
| **SLA** | Service Level Agreement | 服务级别协议 |
| **SLO** | Service Level Objective | 服务级别目标 |
| **SLI** | Service Level Indicator | 服务级别指标 |
| **MTTR** | Mean Time To Recovery | 平均恢复时间 |
| **MTTF** | Mean Time To Failure | 平均故障间隔时间 |

## 前端术语

| 术语 | 定义 |
|------|------|
| **SPA** | Single Page Application，单页应用 |
| **SSR** | Server-Side Rendering，服务端渲染 |
| **SSG** | Static Site Generation，静态站点生成 |
| **CSR** | Client-Side Rendering，客户端渲染 |
| **ISR** | Incremental Static Regeneration，增量静态再生 (Next.js) |
| **Virtual DOM** | 虚拟 DOM，React/Vue 用于高效更新真实 DOM 的技术 |
| **Hydration** | 水合，SSR 后在客户端激活交互的过程 |
| **Tree Shaking** | 树摇优化，移除未使用代码以减小打包体积 |
| **Code Splitting** | 代码分割，按需加载 JavaScript 模块 |
| **Hot Module Replacement (HMR)** | 热模块替换，开发时无需刷新页面即可更新模块 |
| **Web Vitals** | Google 定义的网页性能指标 (LCP, FID, CLS) |
| **LCP** | Largest Contentful Paint，最大内容绘制（< 2.5s） |
| **FID** | First Input Delay，首次输入延迟（< 100ms） |
| **CLS** | Cumulative Layout Shift，累积布局偏移（< 0.1） |
| **CORS** | Cross-Origin Resource Sharing，跨域资源共享 |
| **CSP** | Content Security Policy，内容安全策略 |
| **PWA** | Progressive Web App，渐进式 Web 应用 |
| **A11y** | Accessibility 的缩写，无障碍访问 |
| **WCAG** | Web Content Accessibility Guidelines，网页内容无障碍指南 |

## 后端术语

| 术语 | 定义 |
|------|------|
| **ORM** | Object-Relational Mapping，对象关系映射 (SQLAlchemy, Prisma) |
| **Middleware** | 中间件，请求/响应处理链中的可插拔组件 |
| **Rate Limiting** | 速率限制，防止 API 被过度调用 |
| **Circuit Breaker** | 断路器，防止级联故障的设计模式 |
| **Load Balancer** | 负载均衡器，将请求分发到多个服务实例 |
| **Connection Pool** | 连接池，复用数据库/HTTP 连接以提升性能 |
| **Message Queue** | 消息队列，异步通信组件 (Kafka, RabbitMQ, Redis) |
| **Worker** | 工作进程，处理后台任务的独立进程 |
| **Webhook** | 事件通知机制，服务主动推送事件到指定 URL |
| **Idempotency** | 幂等性，多次执行同一操作结果相同 |
| **CQRS** | Command Query Responsibility Segregation，命令查询职责分离 |
| **Event Sourcing** | 事件溯源，使用事件流记录状态变更 |
| **Saga** | 分布式事务管理模式，通过补偿操作保证一致性 |
| **DDD** | Domain-Driven Design，领域驱动设计 |

## 数据库术语

| 术语 | 定义 |
|------|------|
| **ACID** | Atomicity/Consistency/Isolation/Durability，事务四大特性 |
| **BASE** | Basically Available/Soft state/Eventually consistent，NoSQL 理论 |
| **CAP** | Consistency/Availability/Partition tolerance，分布式系统三选二定理 |
| **MVCC** | Multi-Version Concurrency Control，多版本并发控制 |
| **WAL** | Write-Ahead Log，预写日志 |
| **B-tree** | 平衡树索引，数据库最常用的索引结构 |
| **Sharding** | 分片，将数据分布到多个数据库实例 |
| **Replication** | 复制，数据从主节点同步到从节点 |
| **Materialized View** | 物化视图，预计算并缓存的查询结果 |
| **PITR** | Point-In-Time Recovery，时间点恢复 |

## DevOps 术语

| 术语 | 定义 |
|------|------|
| **Container** | 容器，轻量级隔离的运行环境 (Docker) |
| **Pod** | Kubernetes 中最小的部署单元，包含一个或多个容器 |
| **Helm** | Kubernetes 包管理器 |
| **Service Mesh** | 服务网格，管理服务间通信的基础设施层 (Istio, Linkerd) |
| **GitOps** | 以 Git 为唯一事实来源管理基础设施和部署 |
| **Blue-Green Deployment** | 蓝绿部署，两个相同环境交替使用 |
| **Canary Deployment** | 金丝雀部署，将流量逐步切换到新版本 |
| **Rolling Update** | 滚动更新，逐步替换旧版本实例 |
| **Observability** | 可观测性，通过指标/日志/追踪理解系统状态 |
| **SRE** | Site Reliability Engineering，站点可靠性工程 |
| **Chaos Engineering** | 混沌工程，通过故障注入验证系统韧性 |
| **Toil** | 工辛 (SRE术语)，重复性、手工、可自动化的运维工作 |

## 安全术语

| 术语 | 定义 |
|------|------|
| **OWASP** | Open Web Application Security Project，Web 应用安全标准 |
| **XSS** | Cross-Site Scripting，跨站脚本攻击 |
| **CSRF** | Cross-Site Request Forgery，跨站请求伪造 |
| **SSRF** | Server-Side Request Forgery，服务端请求伪造 |
| **SQLi** | SQL Injection，SQL 注入 |
| **JWT** | JSON Web Token，用于身份验证的令牌格式 |
| **OAuth2** | 授权框架标准 |
| **OIDC** | OpenID Connect，基于 OAuth2 的身份验证协议 |
| **RBAC** | Role-Based Access Control，基于角色的访问控制 |
| **Zero Trust** | 零信任安全模型，"永远不信任，始终验证" |
| **SAST** | Static Application Security Testing，静态应用安全测试 |
| **DAST** | Dynamic Application Security Testing，动态应用安全测试 |
| **SCA** | Software Composition Analysis，软件组件分析 |
| **CVE** | Common Vulnerabilities and Exposures，通用漏洞和暴露 |

## AI/ML 术语

| 术语 | 定义 |
|------|------|
| **LLM** | Large Language Model，大语言模型 |
| **RAG** | Retrieval-Augmented Generation，检索增强生成 |
| **Fine-tuning** | 微调，在预训练模型上针对特定任务训练 |
| **Embedding** | 嵌入向量，将文本/图片转换为数值向量 |
| **Token** | 令牌，LLM 处理的最小文本单元 |
| **Context Window** | 上下文窗口，LLM 单次可处理的最大 Token 数 |
| **Prompt Engineering** | 提示工程，设计输入以获取最佳 LLM 输出 |
| **Agent** | 智能体，能自主使用工具完成任务的 AI 系统 |
| **Hallucination** | 幻觉，LLM 生成看似合理但实际错误的内容 |
| **MLOps** | Machine Learning Operations，机器学习运维 |
| **Vector Database** | 向量数据库，专门存储和查询向量的数据库 |
| **Inference** | 推理，使用训练好的模型进行预测 |

---

## Agent Checklist

Agent 在技术文档和代码审查中遇到术语时:

- [ ] 是否使用了团队一致认可的术语？
- [ ] 缩写首次出现时是否有全称？
- [ ] 是否避免了同一概念用不同名称？
- [ ] 中英文术语是否统一？

---

**文档版本**: v1.0
**最后更新**: 2026-03-28
**质量评分**: 85/100
