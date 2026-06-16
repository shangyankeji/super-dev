---
id: api-design-playbook
title: REST API 设计 Playbook
domain: backend
category: 02-playbooks
difficulty: intermediate
tags: [api, backend, design, http, playbook, 分页, 方法语义, 版本控制]
quality_score: 70
last_updated: 2026-06-15
---
# REST API 设计 Playbook

> 适用场景：面向客户端/第三方的 HTTP API 设计与实现。
> 约束：遵循 RESTful 语义、HTTP 标准、安全最佳实践。

---

## 1. URL 设计

### 1.1 资源命名规范

```
# 使用复数名词表示集合
GET    /api/v1/users
GET    /api/v1/users/123
POST   /api/v1/users
PUT    /api/v1/users/123
DELETE /api/v1/users/123

# 使用 kebab-case
GET /api/v1/user-profiles         # 正确
GET /api/v1/userProfiles          # 错误：camelCase
GET /api/v1/user_profiles         # 不推荐：snake_case

# 资源关系通过层级表达
GET /api/v1/users/123/orders              # 用户的订单列表
GET /api/v1/users/123/orders/456          # 特定订单
GET /api/v1/users/123/orders/456/items    # 订单项

# 层级不超过 3 层；更深关系使用查询参数
GET /api/v1/order-items?orderId=456&userId=123   # 替代深层嵌套
```

### 1.2 非 CRUD 操作

```
# 使用动词子资源（仅限不适合 CRUD 的操作）
POST /api/v1/users/123/activate
POST /api/v1/users/123/deactivate
POST /api/v1/orders/456/cancel
POST /api/v1/reports/export

# 批量操作
POST /api/v1/users/batch-delete
  Body: { "ids": ["123", "456", "789"] }

# 搜索（当查询参数过于复杂时）
POST /api/v1/users/search
  Body: { "filters": [...], "sort": [...] }
```

### 1.3 查询参数

```
# 过滤
GET /api/v1/users?status=active&role=admin

# 排序
GET /api/v1/users?sort=created_at:desc,name:asc

# 字段选择（降低传输体积）
GET /api/v1/users?fields=id,name,email

# 关联展开
GET /api/v1/orders?expand=customer,items

# 分页
GET /api/v1/users?page=2&per_page=20
GET /api/v1/users?cursor=eyJpZCI6MTAwfQ&limit=20
```

---

## 2. HTTP 方法语义

| 方法 | 语义 | 幂等 | 安全 | 请求体 | 典型用途 |
|------|------|------|------|--------|----------|
| GET | 读取资源 | 是 | 是 | 无 | 查询、列表 |
| POST | 创建资源 / 触发操作 | 否 | 否 | 有 | 创建、搜索、RPC |
| PUT | 完整替换资源 | 是 | 否 | 有 | 全量更新 |
| PATCH | 部分更新资源 | 否* | 否 | 有 | 局部更新 |
| DELETE | 删除资源 | 是 | 否 | 可选 | 删除 |
| HEAD | 获取元信息（无 body） | 是 | 是 | 无 | 存在性检查 |
| OPTIONS | 获取支持的方法 | 是 | 是 | 无 | CORS 预检 |

> *PATCH 可设计为幂等，但 RFC 不要求。

### PUT vs PATCH

```python
# PUT: 完整替换（缺失字段设为默认值）
# PUT /api/v1/users/123
{
  "name": "Alice",
  "email": "alice@example.com",
  "role": "admin"
  # bio 字段缺失 → 会被设为 null 或默认值
}

# PATCH: 部分更新（只修改传入的字段）
# PATCH /api/v1/users/123
{
  "role": "admin"
  # 只更新 role，其他字段不变
}
```

---

## 3. 状态码使用

### 3.1 成功响应

| 状态码 | 含义 | 使用场景 |
|--------|------|----------|
| 200 OK | 请求成功 | GET、PUT、PATCH、DELETE 成功 |
| 201 Created | 资源已创建 | POST 创建成功，Location 头指向新资源 |
| 202 Accepted | 已接受，异步处理中 | 异步任务（导出、批量操作） |
| 204 No Content | 成功但无响应体 | DELETE 成功、PUT 无需返回 |

### 3.2 客户端错误

| 状态码 | 含义 | 使用场景 |
|--------|------|----------|
| 400 Bad Request | 请求格式错误 | 参数校验失败、JSON 格式错误 |
| 401 Unauthorized | 未认证 | Token 缺失或过期 |
| 403 Forbidden | 无权限 | 认证通过但权限不足 |
| 404 Not Found | 资源不存在 | ID 不存在、路由不匹配 |
| 409 Conflict | 资源冲突 | 唯一约束冲突、并发修改冲突 |
| 422 Unprocessable Entity | 语义错误 | 格式正确但业务规则不满足 |
| 429 Too Many Requests | 限流 | 超过速率限制 |

### 3.3 服务端错误

| 状态码 | 含义 | 使用场景 |
|--------|------|----------|
| 500 Internal Server Error | 服务内部错误 | 未预期异常 |
| 502 Bad Gateway | 上游服务错误 | 代理/网关收到无效响应 |
| 503 Service Unavailable | 服务不可用 | 维护中、过载 |
| 504 Gateway Timeout | 上游超时 | 代理等待上游超时 |

---

## 4. 错误响应格式

### 4.1 标准错误格式

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "请求参数校验失败",
    "details": [
      {
        "field": "email",
        "message": "邮箱格式不正确",
        "value": "not-an-email"
      },
      {
        "field": "age",
        "message": "必须大于 0",
        "value": -1
      }
    ],
    "request_id": "req_abc123def456",
    "doc_url": "https://api.example.com/docs/errors#VALIDATION_ERROR"
  }
}
```

### 4.2 Python 实现

```python
from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse
from pydantic import BaseModel
import uuid

class ErrorDetail(BaseModel):
    field: str | None = None
    message: str
    value: str | None = None

class ErrorResponse(BaseModel):
    code: str
    message: str
    details: list[ErrorDetail] = []
    request_id: str

app = FastAPI()

@app.middleware("http")
async def add_request_id(request: Request, call_next):
    request_id = request.headers.get("X-Request-ID", str(uuid.uuid4()))
    request.state.request_id = request_id
    response = await call_next(request)
    response.headers["X-Request-ID"] = request_id
    return response

@app.exception_handler(HTTPException)
async def http_exception_handler(request: Request, exc: HTTPException):
    return JSONResponse(
        status_code=exc.status_code,
        content={
            "error": {
                "code": exc.detail.get("code", "UNKNOWN"),
                "message": exc.detail.get("message", str(exc.detail)),
                "details": exc.detail.get("details", []),
                "request_id": getattr(request.state, "request_id", "unknown"),
            }
        },
    )
```

### 4.3 Node.js 实现

```ts
// middleware/errorHandler.ts
import { Request, Response, NextFunction } from 'express';

interface AppError extends Error {
  statusCode: number;
  code: string;
  details?: Array<{ field?: string; message: string; value?: string }>;
}

export function errorHandler(err: AppError, req: Request, res: Response, _next: NextFunction) {
  const statusCode = err.statusCode || 500;
  const requestId = req.headers['x-request-id'] || crypto.randomUUID();

  // 不向客户端暴露内部错误详情
  const message = statusCode >= 500 ? '服务内部错误' : err.message;

  res.status(statusCode).json({
    error: {
      code: err.code || 'INTERNAL_ERROR',
      message,
      details: statusCode < 500 ? (err.details || []) : [],
      request_id: requestId,
    },
  });
}
```

---

## 5. 分页

### 5.1 Offset 分页

```python
# Python (FastAPI)
from fastapi import Query

@app.get("/api/v1/users")
async def list_users(
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
):
    offset = (page - 1) * per_page
    users = await db.users.find().skip(offset).limit(per_page).to_list()
    total = await db.users.count_documents({})

    return {
        "data": users,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": total,
            "total_pages": math.ceil(total / per_page),
        },
    }
```

```ts
// Node.js (Express + Prisma)
app.get('/api/v1/users', async (req, res) => {
  const page = Math.max(1, parseInt(req.query.page as string) || 1);
  const perPage = Math.min(100, Math.max(1, parseInt(req.query.per_page as string) || 20));
  const skip = (page - 1) * perPage;

  const [users, total] = await Promise.all([
    prisma.user.findMany({ skip, take: perPage, orderBy: { createdAt: 'desc' } }),
    prisma.user.count(),
  ]);

  res.json({
    data: users,
    pagination: { page, per_page: perPage, total, total_pages: Math.ceil(total / perPage) },
  });
});
```

### 5.2 Cursor 分页（推荐大数据集）

```python
# Python - Cursor 分页
import base64, json

def encode_cursor(data: dict) -> str:
    return base64.urlsafe_b64encode(json.dumps(data).encode()).decode()

def decode_cursor(cursor: str) -> dict:
    return json.loads(base64.urlsafe_b64decode(cursor.encode()).decode())

@app.get("/api/v1/orders")
async def list_orders(
    cursor: str | None = None,
    limit: int = Query(20, ge=1, le=100),
):
    query = {}
    if cursor:
        decoded = decode_cursor(cursor)
        query = {"_id": {"$gt": decoded["id"]}}

    orders = await db.orders.find(query).sort("_id", 1).limit(limit + 1).to_list()

    has_next = len(orders) > limit
    if has_next:
        orders = orders[:limit]

    next_cursor = encode_cursor({"id": str(orders[-1]["_id"])}) if has_next else None

    return {
        "data": orders,
        "pagination": {
            "next_cursor": next_cursor,
            "has_next": has_next,
            "limit": limit,
        },
    }
```

**Offset vs Cursor 对比**：

| 特性 | Offset | Cursor |
|------|--------|--------|
| 跳转到任意页 | 支持 | 不支持 |
| 数据一致性 | 新增/删除时可能跳过或重复 | 稳定 |
| 大数据集性能 | O(n) skip | O(1) 索引查找 |
| 适用场景 | 后台管理、总数较小 | 时间线、Feed、大数据集 |

---

## 6. 过滤与排序

```python
# 通用过滤排序参数解析
from fastapi import Query
from typing import Literal

@app.get("/api/v1/products")
async def list_products(
    # 过滤
    category: str | None = None,
    min_price: float | None = Query(None, ge=0),
    max_price: float | None = Query(None, ge=0),
    status: Literal["active", "draft", "archived"] | None = None,
    search: str | None = None,
    # 排序
    sort: str = Query("created_at:desc"),
    # 分页
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
):
    filters = {}
    if category:
        filters["category"] = category
    if min_price is not None:
        filters["price__gte"] = min_price
    if max_price is not None:
        filters["price__lte"] = max_price
    if status:
        filters["status"] = status
    if search:
        filters["name__icontains"] = search

    # 排序解析
    ALLOWED_SORT_FIELDS = {"created_at", "price", "name", "popularity"}
    sort_field, _, sort_dir = sort.partition(":")
    if sort_field not in ALLOWED_SORT_FIELDS:
        sort_field = "created_at"
    order = f"-{sort_field}" if sort_dir == "desc" else sort_field

    return await paginate(Product.objects.filter(**filters).order_by(order), page, per_page)
```

---

## 7. 版本控制

### 7.1 URL 路径版本（推荐）

```
GET /api/v1/users
GET /api/v2/users
```

```python
# FastAPI 路由分组
from fastapi import APIRouter

v1_router = APIRouter(prefix="/api/v1")
v2_router = APIRouter(prefix="/api/v2")

@v1_router.get("/users")
async def list_users_v1():
    return {"data": users, "format": "v1"}

@v2_router.get("/users")
async def list_users_v2():
    # v2 增加了 metadata 字段
    return {"data": users, "meta": {"total": len(users)}}

app.include_router(v1_router)
app.include_router(v2_router)
```

### 7.2 版本迁移策略

```
1. 新版本发布时，旧版本保持运行至少 6 个月
2. 响应头标注版本废弃信息：
   Deprecation: true
   Sunset: Sat, 01 Mar 2026 00:00:00 GMT
   Link: <https://api.example.com/docs/migration-v1-to-v2>; rel="deprecation"
3. 旧版本返回 Warning 头：
   Warning: 299 - "API v1 将于 2026-03-01 停止服务，请迁移到 v2"
```

---

## 8. 认证

### 8.1 JWT 认证

```python
# Python (FastAPI)
from fastapi import Depends, HTTPException, status
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
import jwt

security = HTTPBearer()

async def get_current_user(
    credentials: HTTPAuthorizationCredentials = Depends(security),
) -> User:
    try:
        payload = jwt.decode(
            credentials.credentials,
            settings.JWT_SECRET,
            algorithms=["HS256"],
        )
        user_id = payload.get("sub")
        if not user_id:
            raise HTTPException(status_code=401, detail="无效 Token")
    except jwt.ExpiredSignatureError:
        raise HTTPException(status_code=401, detail="Token 已过期")
    except jwt.InvalidTokenError:
        raise HTTPException(status_code=401, detail="无效 Token")

    user = await get_user_by_id(user_id)
    if not user:
        raise HTTPException(status_code=401, detail="用户不存在")
    return user

@app.get("/api/v1/me")
async def get_me(user: User = Depends(get_current_user)):
    return {"data": user}
```

### 8.2 API Key 认证

```ts
// Node.js (Express)
import { Request, Response, NextFunction } from 'express';

async function apiKeyAuth(req: Request, res: Response, next: NextFunction) {
  const apiKey = req.headers['x-api-key'] as string;

  if (!apiKey) {
    return res.status(401).json({
      error: { code: 'MISSING_API_KEY', message: 'X-API-Key 头缺失' },
    });
  }

  // 使用 timing-safe 比较，防止时序攻击
  const client = await db.apiKeys.findOne({
    where: { keyHash: hashApiKey(apiKey), isActive: true },
  });

  if (!client) {
    return res.status(401).json({
      error: { code: 'INVALID_API_KEY', message: 'API Key 无效' },
    });
  }

  // 更新最后使用时间
  await db.apiKeys.update({
    where: { id: client.id },
    data: { lastUsedAt: new Date() },
  });

  req.client = client;
  next();
}
```

---

## 9. 限流

```python
# Python - 令牌桶限流（基于 Redis）
import time, redis

redis_client = redis.Redis()

def is_rate_limited(key: str, limit: int, window: int) -> tuple[bool, dict]:
    """
    令牌桶限流
    key: 限流键（如 user_id 或 IP）
    limit: 窗口内最大请求数
    window: 窗口大小（秒）
    """
    now = time.time()
    pipe = redis_client.pipeline()
    pipe.zremrangebyscore(key, 0, now - window)  # 移除过期记录
    pipe.zadd(key, {str(now): now})              # 添加当前请求
    pipe.zcard(key)                              # 当前窗口请求数
    pipe.expire(key, window)                     # 设置过期
    _, _, count, _ = pipe.execute()

    remaining = max(0, limit - count)
    headers = {
        "X-RateLimit-Limit": str(limit),
        "X-RateLimit-Remaining": str(remaining),
        "X-RateLimit-Reset": str(int(now + window)),
    }

    return count > limit, headers
```

```ts
// Node.js - express-rate-limit
import rateLimit from 'express-rate-limit';
import RedisStore from 'rate-limit-redis';

const apiLimiter = rateLimit({
  store: new RedisStore({ sendCommand: (...args) => redisClient.sendCommand(args) }),
  windowMs: 60 * 1000,    // 1 分钟窗口
  max: 100,                // 每窗口 100 次
  standardHeaders: true,   // 返回 RateLimit-* 头
  legacyHeaders: false,
  message: {
    error: {
      code: 'RATE_LIMITED',
      message: '请求频率超限，请稍后重试',
    },
  },
});

app.use('/api/', apiLimiter);
```

---

## 10. HATEOAS

```json
// GET /api/v1/orders/456
{
  "data": {
    "id": "456",
    "status": "pending",
    "total": 299.00,
    "created_at": "2025-01-15T10:30:00Z"
  },
  "links": {
    "self": { "href": "/api/v1/orders/456", "method": "GET" },
    "cancel": { "href": "/api/v1/orders/456/cancel", "method": "POST" },
    "pay": { "href": "/api/v1/orders/456/pay", "method": "POST" },
    "items": { "href": "/api/v1/orders/456/items", "method": "GET" },
    "customer": { "href": "/api/v1/users/123", "method": "GET" }
  }
}

// 状态变为 shipped 后，cancel 和 pay 链接消失
// GET /api/v1/orders/456
{
  "data": {
    "id": "456",
    "status": "shipped",
    "total": 299.00
  },
  "links": {
    "self": { "href": "/api/v1/orders/456", "method": "GET" },
    "track": { "href": "/api/v1/orders/456/tracking", "method": "GET" },
    "items": { "href": "/api/v1/orders/456/items", "method": "GET" }
  }
}
```

---

## 11. OpenAPI 文档

### 11.1 FastAPI 自动生成

```python
from fastapi import FastAPI

app = FastAPI(
    title="My API",
    description="商城 API 服务",
    version="1.0.0",
    docs_url="/api/docs",        # Swagger UI
    redoc_url="/api/redoc",      # ReDoc
    openapi_url="/api/openapi.json",
)

@app.get(
    "/api/v1/products/{product_id}",
    response_model=ProductResponse,
    summary="获取商品详情",
    description="根据商品 ID 获取完整的商品信息，包括价格、库存、分类等。",
    responses={
        404: {"model": ErrorResponse, "description": "商品不存在"},
        500: {"model": ErrorResponse, "description": "服务内部错误"},
    },
    tags=["商品"],
)
async def get_product(product_id: str):
    ...
```

### 11.2 Express + swagger-jsdoc

```ts
import swaggerJsdoc from 'swagger-jsdoc';
import swaggerUi from 'swagger-ui-express';

const spec = swaggerJsdoc({
  definition: {
    openapi: '3.0.0',
    info: { title: 'My API', version: '1.0.0' },
    servers: [{ url: '/api/v1' }],
  },
  apis: ['./src/routes/*.ts'],
});

app.use('/api/docs', swaggerUi.serve, swaggerUi.setup(spec));

/**
 * @openapi
 * /products/{id}:
 *   get:
 *     summary: 获取商品详情
 *     tags: [商品]
 *     parameters:
 *       - in: path
 *         name: id
 *         required: true
 *         schema: { type: string }
 *     responses:
 *       200:
 *         description: 成功
 *         content:
 *           application/json:
 *             schema: { $ref: '#/components/schemas/Product' }
 *       404:
 *         description: 商品不存在
 */
router.get('/products/:id', getProduct);
```

### 11.3 CI 校验 OpenAPI

```bash
# 校验 OpenAPI 规范
npx @redocly/cli lint openapi.yaml

# 生成客户端 SDK
npx openapi-generator-cli generate \
  -i openapi.yaml \
  -g typescript-axios \
  -o ./sdk/

# 对比版本差异（检测破坏性变更）
npx oasdiff breaking old-openapi.yaml new-openapi.yaml
```

---

## Agent Checklist

- [ ] URL 使用复数名词、kebab-case、层级不超过 3 层
- [ ] HTTP 方法与语义匹配（GET 不修改、PUT 幂等、DELETE 幂等）
- [ ] 所有错误响应使用统一格式（code + message + details + request_id）
- [ ] 状态码使用准确（区分 400/401/403/404/409/422/429）
- [ ] 分页已实现并有 per_page 上限（Offset 或 Cursor）
- [ ] 排序字段有白名单校验
- [ ] API 版本控制已实施（URL 路径方式）
- [ ] 认证方案已实现（JWT 或 API Key）
- [ ] 限流已配置并返回 RateLimit 响应头
- [ ] OpenAPI 文档自动生成并在 CI 中校验
- [ ] 无破坏性变更引入（oasdiff 检查）
- [ ] 所有输入参数有校验（类型、范围、格式）
