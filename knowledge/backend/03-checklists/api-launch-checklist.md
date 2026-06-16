---
id: api-launch-checklist
title: API 上线检查清单
domain: backend
category: 03-checklists
difficulty: intermediate
tags: [agent, alerting, api, authorization, backend, checklist, handling, launch]
quality_score: 70
last_updated: 2026-06-15
---
# API 上线检查清单

## 概述

本清单用于 API 服务从开发完成到正式上线前的全面审查。覆盖认证授权、限流、日志、监控、文档、版本管理和错误处理七大维度。每个维度包含必须（MUST）和建议（SHOULD）两级要求，MUST 级别项未通过则阻止上线。

适用场景：RESTful API、GraphQL API、gRPC 服务的生产环境发布。

---

## 1. 认证与授权（Authentication & Authorization）

### MUST

- [ ] 所有非公开端点要求有效的认证凭证（JWT / OAuth2 / API Key）
- [ ] Token 验证包含签名校验、过期时间校验、发行者校验
- [ ] 敏感操作端点实施细粒度权限控制（RBAC / ABAC）
- [ ] 密码 / Token / Secret 不出现在 URL 参数中（仅通过 Header 或 Body 传递）
- [ ] API Key 支持独立吊销（revoke），不影响其他 Key
- [ ] 认证失败返回 401，授权失败返回 403，不泄漏内部错误细节
- [ ] CORS 配置白名单明确，不使用 `Access-Control-Allow-Origin: *`（公开 API 除外）

### SHOULD

- [ ] 支持 Token 自动刷新机制（Refresh Token）
- [ ] 实施 JWT 黑名单机制处理提前登出场景
- [ ] 敏感端点增加二次验证（MFA / 密码确认）
- [ ] API Key 支持 IP 白名单绑定
- [ ] 定期轮换签名密钥（Key Rotation），支持多密钥同时有效

## 2. 限流与防护（Rate Limiting & Protection）

### MUST

- [ ] 全局限流策略已启用（如 1000 req/min per IP）
- [ ] 关键端点（登录、注册、密码重置）有独立的更严格限流
- [ ] 限流响应返回 429 状态码，携带 `Retry-After` Header
- [ ] 限流信息通过 Header 告知客户端：`X-RateLimit-Limit`、`X-RateLimit-Remaining`、`X-RateLimit-Reset`
- [ ] 请求体大小限制已设置（如 10MB），防止内存耗尽攻击
- [ ] SQL 注入防护：所有数据库查询使用参数化查询
- [ ] XSS 防护：响应 Header 包含 `Content-Type`，JSON 响应不含 HTML

### SHOULD

- [ ] 分层限流：全局 + 用户级 + 端点级
- [ ] 使用滑动窗口算法（而非固定窗口）减少边界突发
- [ ] 实施慢速攻击防护（Slowloris）：设置请求超时
- [ ] 敏感端点加入 CAPTCHA 或人机验证
- [ ] 启用 WAF（Web Application Firewall）规则
- [ ] 实施请求签名防止重放攻击（Replay Attack）

## 3. 日志（Logging）

### MUST

- [ ] 所有请求记录：时间戳、方法、路径、状态码、响应时间、客户端 IP
- [ ] 认证失败、授权失败、限流触发记录为 WARN 级别
- [ ] 服务器错误（5xx）记录为 ERROR 级别，包含完整堆栈
- [ ] 日志不包含敏感信息（密码、Token、信用卡号、身份证号）
- [ ] 日志格式统一为结构化 JSON，便于后续检索
- [ ] 每条日志包含唯一请求 ID（Request-ID），贯穿全链路
- [ ] 日志输出到标准输出（stdout），由基础设施负责收集

### SHOULD

- [ ] 实施日志分级：DEBUG / INFO / WARN / ERROR
- [ ] 慢请求（>2s）自动记录为 WARN 并包含耗时分解
- [ ] 日志中包含用户 ID（脱敏）便于行为追踪
- [ ] 审计日志独立存储，保留期 ≥ 1 年
- [ ] 日志量监控：异常增长触发告警
- [ ] 分布式追踪 ID（Trace-ID）贯穿微服务调用链

## 4. 监控与告警（Monitoring & Alerting）

### MUST

- [ ] 健康检查端点 `/health` 或 `/healthz` 已实现（返回 200 + 依赖状态）
- [ ] 就绪检查端点 `/ready` 区分于存活检查（数据库/缓存连通时才返回 200）
- [ ] 核心指标暴露：请求量、错误率、响应时间 P50/P95/P99
- [ ] 5xx 错误率 > 1% 触发告警
- [ ] 响应时间 P99 > 5s 触发告警
- [ ] 服务不可用（健康检查失败）触发即时告警（<1 分钟）
- [ ] 数据库连接池使用率 > 80% 触发告警

### SHOULD

- [ ] Grafana / Datadog 仪表板已创建，包含核心业务指标
- [ ] 告警分级：P1（即时通知 + 电话）/ P2（消息通知）/ P3（日报汇总）
- [ ] SLO 定义明确（如可用性 99.9%，P99 < 500ms）
- [ ] 错误预算（Error Budget）跟踪
- [ ] 上游依赖（第三方 API）可用性独立监控
- [ ] 定期进行告警有效性审查（消除噪音告警）

## 5. API 文档（Documentation）

### MUST

- [ ] OpenAPI / Swagger 规范文件存在且与实际 API 一致
- [ ] 每个端点包含：路径、方法、参数说明、请求体示例、响应体示例
- [ ] 错误响应有统一格式文档（错误码 + 错误消息 + 字段级错误）
- [ ] 认证方式在文档中明确说明（包含示例 Header）
- [ ] 分页、排序、过滤的通用参数有全局说明

### SHOULD

- [ ] 提供 Postman Collection 或等效可导入文件
- [ ] 文档中包含使用场景示例（Getting Started）
- [ ] Changelog 记录每个版本的 Breaking Changes
- [ ] 提供 SDK / 代码示例（至少覆盖 3 种主流语言）
- [ ] 文档自动从代码注解生成（减少手动维护偏差）
- [ ] 文档包含速率限制说明和最佳实践建议

## 6. 版本管理（Versioning）

### MUST

- [ ] API 版本号明确标识（URL path `/v1/` 或 Header `Accept-Version`）
- [ ] Breaking Change 必须升级主版本号
- [ ] 旧版本有明确的废弃时间表（至少提前 6 个月通知）
- [ ] 多版本同时运行时，路由正确分发

### SHOULD

- [ ] 非 Breaking Change 通过添加可选字段实现，不升级版本
- [ ] 废弃的端点返回 `Sunset` Header 和 `Deprecation` Header
- [ ] 版本迁移指南文档完善
- [ ] 旧版本 API 的使用量监控（推动客户端迁移）
- [ ] 版本策略写入 API 治理规范

## 7. 错误处理（Error Handling）

### MUST

- [ ] 错误响应格式统一：`{ "error": { "code": "...", "message": "...", "details": [...] } }`
- [ ] HTTP 状态码使用正确：400（参数错误）、401（未认证）、403（无权限）、404（不存在）、409（冲突）、422（验证失败）、429（限流）、500（服务器错误）
- [ ] 500 错误不暴露内部实现细节（堆栈、SQL、文件路径）
- [ ] 输入验证失败返回字段级错误（指明哪个字段、什么规则、期望什么）
- [ ] 全局异常处理器兜底，不出现未捕获异常导致进程退出
- [ ] 数据库 / 外部服务不可用时返回 503 + Retry-After

### SHOULD

- [ ] 错误码体系文档化（业务错误码 + HTTP 状态码映射）
- [ ] 可重试错误与不可重试错误在响应中区分
- [ ] 错误消息支持 i18n（国际化）
- [ ] 客户端错误（4xx）和服务端错误（5xx）的日志级别不同
- [ ] 实施熔断器模式（Circuit Breaker）处理级联故障
- [ ] 优雅降级：部分服务不可用时核心功能仍可运行

---

## 上线前最终确认

| 序号 | 检查项 | 状态 |
|------|--------|------|
| 1 | 所有 MUST 级别检查项通过 | [ ] |
| 2 | 压力测试已完成，满足预期 QPS | [ ] |
| 3 | 灰度发布方案已就绪 | [ ] |
| 4 | 回滚方案已验证 | [ ] |
| 5 | On-call 值班人员已通知 | [ ] |
| 6 | 数据库迁移脚本已在 staging 验证 | [ ] |
| 7 | 客户端团队已确认兼容性 | [ ] |

---

## Agent Checklist

以下为 AI Agent 在 API 上线审查时必须逐项验证的硬约束：

- [ ] 对每个端点发送无认证请求，确认非公开端点返回 401
- [ ] 对每个端点发送畸形请求体，确认返回 400/422 而非 500
- [ ] 高频请求单端点，确认限流返回 429 + Retry-After
- [ ] 检查 `/health` 和 `/ready` 端点存在且返回正确状态
- [ ] 检查 OpenAPI spec 文件存在且可被 swagger-ui 解析
- [ ] 检查日志输出为结构化 JSON 且不包含密码/Token
- [ ] 检查 CORS 配置不包含通配符 `*`（公开 API 除外）
- [ ] 检查 500 错误响应不包含堆栈信息或内部路径
- [ ] 确认 API 版本号在 URL 或 Header 中明确标识
- [ ] 生成上线审查报告并附在发布 ticket 中
