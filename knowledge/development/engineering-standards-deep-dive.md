---
id: engineering-standards-deep-dive
title: engineering-standards-deep-dive
domain: development
category: engineering-standards-deep-dive.md
difficulty: intermediate
tags: [deep, development, dive, engineering, standards, 开发环节深度知识库]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 开发环节深度知识库

### 目标
- 让实现质量稳定、可回归、可协作，避免“个人风格驱动”。

### 工程规范
- 命名统一：领域语义优先，禁止无业务含义缩写。
- 模块边界：路由、服务、仓储、模型职责清晰。
- 错误模型：统一错误码、错误分级、用户可见提示规范。
- 配置管理：环境变量最小集、默认值安全、敏感信息隔离。

### 编码实践
- 关键分支必须有单测覆盖。
- 高风险逻辑必须提供幂等或去重机制。
- 外部调用必须具备超时、重试、熔断与降级。
- 写接口先定义契约，再实现与联调。

### 评审基线
- 是否存在跨层依赖污染。
- 是否存在高复杂函数且缺乏测试保护。
- 是否有日志但无请求上下文标识。
- 是否有潜在破坏性变更但无回滚说明。

### 常见失败模式
- 把业务规则散落在控制器与工具函数中。
- 只修当前 bug，不补充防回归测试。
