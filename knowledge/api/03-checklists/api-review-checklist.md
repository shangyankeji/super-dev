---
id: api-review-checklist
title: API 代码审查清单
domain: api
category: 03-checklists
difficulty: beginner
tags: [api, review, checklist, validation, error, security, pagination, openapi, contract]
quality_score: 88
maintainer: platform-team@superdev.com
last_updated: 2026-06-14
---

# API 代码审查清单

## 契约一致性
- [ ] OpenAPI 契约与实现一致（路径、方法、状态码）
- [ ] 每个 operationId 在代码中有对应 handler
- [ ] 请求/响应 schema 与契约定义匹配
- [ ] 破坏性变更已版本化（/v2/）

## 输入验证
- [ ] 每个字段有类型校验
- [ ] 字符串有长度限制
- [ ] 数值有范围限制
- [ ] email/url/uuid 有格式校验
- [ ] 枚举值在白名单内
- [ ] 文件上传有大小 + 类型限制

## 错误处理
- [ ] 每个端点返回标准错误信封
- [ ] 错误含 code + message + requestId
- [ ] 400（校验）/401（未认证）/403（无权限）/404（不存在）/409（冲突）/500（服务器）全覆盖
- [ ] 500 错误不泄露堆栈/SQL
- [ ] 校验错误列出所有字段问题（不只第一个）

## 安全
- [ ] 公开端点（login/register）有速率限制
- [ ] 写操作需要 CSRF token（如适用）
- [ ] 用户只能访问自己的数据（authorization check）
- [ ] 敏感字段不返回（password_hash / internal_id）
- [ ] SQL 注入防护（参数化查询）
- [ ] CORS 配置正确（不使用 *）

## 性能
- [ ] list 端点有分页（默认 limit + 上限）
- [ ] N+1 查询已消除（eager loading）
- [ ] 响应字段可选择（`?fields=id,name`）
- [ ] 大响应有缓存头（ETag / Cache-Control）
- [ ] 慢查询有索引

## 可观测性
- [ ] 每个请求有 requestId（贯穿日志）
- [ ] 请求耗时记录
- [ ] 错误率告警
- [ ] 结构化日志（JSON 格式）
