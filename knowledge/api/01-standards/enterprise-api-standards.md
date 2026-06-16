---
id: enterprise-api-standards
title: 企业级 API 设计标准（完整版）
domain: api
category: 01-standards
difficulty: advanced
tags: [api, rest, enterprise, openapi, contract, validation, pagination, error, versioning, security]
quality_score: 95
maintainer: platform-team@superdev.com
last_updated: 2026-06-14
---

# 企业级 API 设计标准（完整版）

## 资源建模

### 命名约定
```
✅ 复数名词: /api/users, /api/orders, /api/products
✅ 嵌套关系: /api/users/:userId/orders
❌ 动词路径: /api/getUsers, /api/createOrder
❌ 单数名词: /api/user, /api/order
```

### 路径层级
- 一级：`/api/{resource}` — 集合操作（list / create）
- 二级：`/api/{resource}/:id` — 单体操作（get / update / delete）
- 三级：`/api/{resource}/:id/{sub}` — 子资源（`/api/users/:id/orders`）
- 动作端点：`/api/{resource}/:id/action` — 非 CRUD 操作（`/api/orders/:id/cancel`）

## HTTP 方法语义

| 方法 | 幂等 | 安全 | 语义 | 典型状态码 |
|------|------|------|------|-----------|
| GET | ✅ | ✅ | 读取，不改数据 | 200 / 404 |
| POST | ❌ | ❌ | 创建，非幂等 | 201 / 400 / 409 |
| PUT | ✅ | ❌ | 完整替换 | 200 / 204 / 404 |
| PATCH | ❌ | ❌ | 部分更新 | 200 / 404 |
| DELETE | ✅ | ❌ | 删除 | 204 / 404 |

## 分页（三种策略）

### Offset 分页（简单列表）
```json
GET /api/products?page=2&limit=20

Response:
{
  "data": [...],
  "pagination": {
    "page": 2,
    "limit": 20,
    "total": 150,
    "totalPages": 8
  }
}
```

### Cursor 分页（大数据集 / 实时流）
```json
GET /api/events?cursor=eyJpZCI6MTIzfQ&limit=50

Response:
{
  "data": [...],
  "pagination": {
    "nextCursor": "eyJpZCI6MTczfQ",
    "hasMore": true
  }
}
```

### Keyset 分页（排序稳定）
```json
GET /api/orders?after=2024-01-15T10:30:00Z&limit=20
```

## 错误处理（统一错误信封）

### 标准错误格式
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "The request was invalid",
    "details": [
      { "field": "email", "issue": "must be a valid email address" },
      { "field": "quantity", "issue": "must be greater than 0" }
    ],
    "requestId": "req_abc123",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### 错误码分类
| HTTP | error code | 场景 |
|------|-----------|------|
| 400 | VALIDATION_ERROR | 请求体校验失败 |
| 401 | UNAUTHENTICATED | 缺少/无效 token |
| 403 | FORBIDDEN | 权限不足 |
| 404 | NOT_FOUND | 资源不存在 |
| 409 | CONFLICT | 唯一约束冲突 |
| 422 | UNPROCESSABLE | 业务逻辑校验失败 |
| 429 | RATE_LIMITED | 超过速率限制 |
| 500 | INTERNAL_ERROR | 服务器异常 |

## 输入验证

### 每个端点必须有
```python
# Python/FastAPI 示例
from pydantic import BaseModel, EmailStr, constr

class CreateOrderRequest(BaseModel):
    product_id: str  # required
    quantity: int = Field(ge=1, le=999)  # 1-999
    notes: constr(max_length=500) | None = None  # optional, max 500
```

### 校验层次
1. **类型校验** — 字段类型正确（string/int/bool）
2. **格式校验** — email/url/uuid/date 格式
3. **范围校验** — 数值在合理范围
4. **业务校验** — 库存够不够、权限对不对
5. **关联校验** — 外键引用存在

## API 版本化

### URL 路径版本（推荐）
```
/api/v1/users
/api/v2/users
```

### Header 版本（备选）
```
GET /api/users
Accept-Version: 2.0
```

### 版本弃用策略
- 新版本发布时旧版本至少维护 6 个月
- 响应头标注弃用：`Sunset: Sat, 31 Dec 2024 23:59:59 GMT`
- `Deprecation: true` 头告知客户端迁移

## 认证与授权

### JWT Bearer Token
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

### 端点安全矩阵
| 端点类型 | 认证 | 授权 |
|---------|------|------|
| 公开端点（login/register） | 无 | 无 |
| 用户数据端点 | JWT | 只能访问自己的数据 |
| 管理端点 | JWT + admin role | admin only |
| 内部端点 | API key / mTLS | service-to-service |

## 速率限制

### 响应头
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1700000000
```

### 429 响应
```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Too many requests",
    "retryAfter": 60
  }
}
```

## CORS 配置

```http
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Methods: GET, POST, PATCH, DELETE, OPTIONS
Access-Control-Allow-Headers: Authorization, Content-Type
Access-Control-Max-Age: 86400
```

## OpenAPI 契约要求

每个 API 必须有完整的 OpenAPI 3.1 文档：
- 每个端点有 `operationId`（唯一标识符）
- 请求体有 JSON Schema `$ref`
- 响应有所有可能的状态码 + schema
- 安全方案明确声明
- 示例（example）覆盖成功 + 错误场景
