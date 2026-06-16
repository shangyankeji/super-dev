---
id: backend-engineering-complete
title: backend-engineering-complete
domain: development
category: backend-engineering-complete.md
difficulty: intermediate
tags: [backend, complete, development, engineering, 后端工程完整知识库]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 后端工程完整知识库

### 1. 分层架构
- 路由层只做协议适配与参数接收。
- 服务层编排业务流程与事务边界。
- 仓储层封装数据访问，不泄露底层实现细节。

### 2. 领域建模
- 主实体必须定义唯一标识、生命周期状态、关键约束。
- 跨聚合写操作必须明确一致性策略。
- 领域事件用于解耦非核心同步流程。

### 3. 异常与错误码
- 错误码分层：业务错误、系统错误、依赖错误。
- 错误响应结构统一，便于前端和监控系统消费。
- 不向外暴露内部堆栈与敏感实现细节。

### 4. 依赖调用治理
- 所有外部调用必须配置超时。
- 仅对可恢复错误启用有限重试。
- 依赖异常触发熔断与降级策略。

### 5. 配置与环境
- 配置集中管理，分环境隔离。
- 敏感配置仅通过安全存储注入。
- 变更配置必须可回滚并可审计。

### 6. 可观测性
- 日志统一包含请求ID、用户ID、租户ID。
- 关键链路暴露时延、成功率、错误率指标。
- 分布式调用必须具备追踪ID贯通能力。
