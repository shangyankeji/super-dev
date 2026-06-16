---
id: api-design-playbook
title: REST API 设计作战手册 (API Design Playbook)
domain: development
category: 02-playbooks
difficulty: intermediate
tags: [agent, api, checklist, design, development, playbook, 前置条件, 回滚方案]
quality_score: 70
last_updated: 2026-06-15
---
# REST API 设计作战手册 (API Design Playbook)

## 概述

本手册定义了 REST API 设计的完整最佳实践，涵盖 URL 设计、版本控制、请求/响应格式、错误处理、分页过滤、认证授权、限流保护和文档自动化。目标是构建一致、可预测、易于使用且可长期演进的 API。

## 前置条件

### 必须满足

- [ ] 已明确 API 的目标用户（内部/合作方/公开）
- [ ] 已定义核心资源模型和关系
- [ ] 已确定认证授权方案
- [ ] 已确定 API 版本策略
- [ ] 有 API 设计评审流程

### 建议满足

- [ ] 有 API 设计规范文档（本手册可作为基础）
- [ ] 有 OpenAPI/Swagger 工具链
- [ ] 有 API 网关（用于限流、认证、监控）

---

## 步骤一：URL 设计

### 1.1 资源命名规则

```
规则 1: 使用名词复数形式表示资源集合
  /api/v1/users          ✓
  /api/v1/getUsers       ✗ (动词)
  /api/v1/user           ✗ (单数)

规则 2: 使用路径表达资源层级关系
  /api/v1/users/123/orders          ✓ (用户的订单)
  /api/v1/users/123/orders/456      ✓ (具体订单)
  /api/v1/getUserOrders?userId=123  ✗ (RPC 风格)

规则 3: 层级不超过 3 层，过深用查询参数
  /api/v1/users/123/orders/456/items        ✓ (3 层)
  /api/v1/users/123/orders/456/items/789/reviews  ✗ (太深)
  /api/v1/order-items/789/reviews           ✓ (扁平化)

规则 4: 使用 kebab-case
  /api/v1/order-items    ✓
  /api/v1/orderItems     ✗ (camelCase)
  /api/v1/order_items    ✗ (snake_case)

规则 5: 非 CRUD 操作使用动词子资源
  POST /api/v1/orders/123/cancel     ✓ (取消订单)
  POST /api/v1/orders/123/refund     ✓ (退款)
  POST /api/v1/users/123/activate    ✓ (激活用户)
```

### 1.2 HTTP 方法语义

```
GET    /resources          获取资源列表（安全、幂等）
GET    /resources/:id      获取单个资源（安全、幂等）
POST   /resources          创建资源（非幂等）
PUT    /resources/:id      全量更新资源（幂等）
PATCH  /resources/:id      部分更新资源（幂等）
DELETE /resources/:id      删除资源（幂等）

HEAD   /resources/:id      获取资源元信息（安全、幂等）
OPTIONS /resources         获取支持的方法（安全、幂等）
```

### 1.3 HTTP 状态码规范

```
2xx 成功
  200 OK              GET/PUT/PATCH/DELETE 成功
  201 Created         POST 创建成功（响应中包含 Location 头）
  202 Accepted        异步操作已接受
  204 No Content      DELETE 成功（无响应体）

3xx 重定向
  301 Moved Permanently    永久重定向（API 版本废弃）
  304 Not Modified         条件请求命中缓存

4xx 客户端错误
  400 Bad Request          请求参数非法
  401 Unauthorized         未认证
  403 Forbidden            已认证但无权限
  404 Not Found            资源不存在
  405 Method Not Allowed   HTTP 方法不支持
  409 Conflict             资源冲突（重复创建、乐观锁冲突）
  415 Unsupported Media Type  Content-Type 不支持
  422 Unprocessable Entity    参数格式正确但语义错误
  429 Too Many Requests    限流

5xx 服务端错误
  500 Internal Server Error  未处理的服务端异常
  502 Bad Gateway            上游服务异常
  503 Service Unavailable    服务暂时不可用
  504 Gateway Timeout        上游服务超时
```

---

## 步骤二：版本控制

### 2.1 版本策略对比

| 策略 | 示例 | 优点 | 缺点 |
|------|------|------|------|
| URL 路径 | `/api/v1/users` | 直观、缓存友好 | URL 变长 |
| 请求头 | `Accept: application/vnd.api.v1+json` | URL 干净 | 不直观、调试困难 |
| 查询参数 | `/api/users?version=1` | 简单 | 缓存不友好 |

**推荐：URL 路径版本**，理由：最直观、CDN/缓存友好、易于文档化。

### 2.2 版本演进策略

```python
# FastAPI 多版本路由示例
from fastapi import FastAPI, APIRouter

app = FastAPI()

# V1 路由
v1_router = APIRouter(prefix="/api/v1")

@v1_router.get("/users/{user_id}")
async def get_user_v1(user_id: int):
    """V1: 返回基础用户信息"""
    return {"id": user_id, "name": "...", "email": "..."}

# V2 路由 - 向后兼容的增强
v2_router = APIRouter(prefix="/api/v2")

@v2_router.get("/users/{user_id}")
async def get_user_v2(user_id: int):
    """V2: 增加 profile 嵌套对象"""
    return {
        "id": user_id,
        "name": "...",
        "email": "...",
        "profile": {"avatar": "...", "bio": "..."}
    }

app.include_router(v1_router)
app.include_router(v2_router)
```

### 2.3 版本废弃流程

```
1. 宣布废弃：在响应头中添加 Deprecation 和 Sunset 头
   Deprecation: true
   Sunset: Sat, 01 Jun 2025 00:00:00 GMT
   Link: <https://api.example.com/api/v2/users>; rel="successor-version"

2. 发送通知：提前 6 个月通知 API 消费方

3. 监控使用量：追踪旧版本 API 的调用量

4. 关闭旧版本：到期后返回 410 Gone
```

---

## 步骤三：请求与响应格式

### 3.1 请求格式

```python
from pydantic import BaseModel, Field, validator
from typing import Optional
from enum import Enum

class OrderStatus(str, Enum):
    DRAFT = "draft"
    CONFIRMED = "confirmed"
    SHIPPED = "shipped"

class CreateOrderRequest(BaseModel):
    """创建订单请求"""
    product_id: int = Field(..., gt=0, description="商品 ID")
    quantity: int = Field(..., ge=1, le=999, description="数量")
    shipping_address: str = Field(
        ..., min_length=5, max_length=500, description="收货地址"
    )
    note: Optional[str] = Field(
        None, max_length=1000, description="订单备注"
    )

    @validator("shipping_address")
    def validate_address(cls, v):
        if not v.strip():
            raise ValueError("地址不能为空白")
        return v.strip()

class UpdateOrderRequest(BaseModel):
    """部分更新订单请求（PATCH）"""
    status: Optional[OrderStatus] = None
    shipping_address: Optional[str] = Field(None, min_length=5, max_length=500)
    note: Optional[str] = Field(None, max_length=1000)
```

### 3.2 响应格式

```python
from datetime import datetime
from typing import TypeVar, Generic, List, Optional
from pydantic import BaseModel

T = TypeVar("T")

# 单个资源响应
class OrderResponse(BaseModel):
    id: int
    product_id: int
    quantity: int
    status: str
    total_amount: float
    shipping_address: str
    note: Optional[str]
    created_at: datetime
    updated_at: datetime

    class Config:
        json_encoders = {
            datetime: lambda v: v.isoformat()
        }

# 列表响应（带分页）
class PaginatedResponse(BaseModel, Generic[T]):
    data: List[T]
    pagination: dict  # {"page": 1, "per_page": 20, "total": 150, "total_pages": 8}
    links: dict       # {"self": "...", "first": "...", "prev": "...", "next": "...", "last": "..."}

# HATEOAS 链接（可选）
class OrderWithLinks(OrderResponse):
    links: dict = {}

    @classmethod
    def from_order(cls, order, base_url: str):
        return cls(
            **order.dict(),
            links={
                "self": f"{base_url}/orders/{order.id}",
                "cancel": f"{base_url}/orders/{order.id}/cancel",
                "user": f"{base_url}/users/{order.user_id}",
            }
        )
```

---

## 步骤四：错误处理

### 4.1 统一错误格式

```python
from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
from pydantic import BaseModel
from typing import Optional, List

class ErrorDetail(BaseModel):
    field: Optional[str] = None
    message: str
    code: str

class ErrorResponse(BaseModel):
    error: dict  # {"code": "...", "message": "...", "details": [...], "request_id": "..."}

# 统一错误响应示例
{
    "error": {
        "code": "VALIDATION_ERROR",
        "message": "请求参数校验失败",
        "details": [
            {"field": "quantity", "message": "必须大于 0", "code": "INVALID_VALUE"},
            {"field": "email", "message": "邮箱格式不正确", "code": "INVALID_FORMAT"}
        ],
        "request_id": "req_abc123def456",
        "doc_url": "https://docs.example.com/errors/VALIDATION_ERROR"
    }
}

# 业务错误码定义
ERROR_CODES = {
    # 通用错误 (1xxx)
    "VALIDATION_ERROR": {"status": 400, "message": "请求参数校验失败"},
    "UNAUTHORIZED": {"status": 401, "message": "未认证"},
    "FORBIDDEN": {"status": 403, "message": "无权限"},
    "NOT_FOUND": {"status": 404, "message": "资源不存在"},
    "CONFLICT": {"status": 409, "message": "资源冲突"},
    "RATE_LIMITED": {"status": 429, "message": "请求频率超限"},

    # 订单错误 (2xxx)
    "ORDER_NOT_FOUND": {"status": 404, "message": "订单不存在"},
    "ORDER_ALREADY_CANCELLED": {"status": 409, "message": "订单已取消"},
    "INSUFFICIENT_STOCK": {"status": 422, "message": "库存不足"},

    # 支付错误 (3xxx)
    "PAYMENT_FAILED": {"status": 422, "message": "支付失败"},
    "PAYMENT_TIMEOUT": {"status": 504, "message": "支付超时"},
}
```

### 4.2 全局异常处理

```python
from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
from fastapi.exceptions import RequestValidationError
import uuid
import logging

logger = logging.getLogger(__name__)
app = FastAPI()

class BusinessError(Exception):
    def __init__(self, code: str, message: str = None, details: list = None):
        self.code = code
        self.message = message or ERROR_CODES.get(code, {}).get("message", "未知错误")
        self.status = ERROR_CODES.get(code, {}).get("status", 500)
        self.details = details or []

@app.exception_handler(BusinessError)
async def business_error_handler(request: Request, exc: BusinessError):
    request_id = request.headers.get("X-Request-ID", str(uuid.uuid4()))
    return JSONResponse(
        status_code=exc.status,
        content={
            "error": {
                "code": exc.code,
                "message": exc.message,
                "details": exc.details,
                "request_id": request_id,
            }
        }
    )

@app.exception_handler(RequestValidationError)
async def validation_error_handler(request: Request, exc: RequestValidationError):
    request_id = request.headers.get("X-Request-ID", str(uuid.uuid4()))
    details = [
        {"field": ".".join(str(loc) for loc in err["loc"]), "message": err["msg"], "code": "INVALID_VALUE"}
        for err in exc.errors()
    ]
    return JSONResponse(
        status_code=400,
        content={
            "error": {
                "code": "VALIDATION_ERROR",
                "message": "请求参数校验失败",
                "details": details,
                "request_id": request_id,
            }
        }
    )

@app.exception_handler(Exception)
async def general_error_handler(request: Request, exc: Exception):
    request_id = request.headers.get("X-Request-ID", str(uuid.uuid4()))
    logger.exception("Unhandled exception", extra={"request_id": request_id})
    return JSONResponse(
        status_code=500,
        content={
            "error": {
                "code": "INTERNAL_ERROR",
                "message": "服务内部错误",
                "request_id": request_id,
            }
        }
    )
```

---

## 步骤五：分页与过滤

### 5.1 分页实现

```python
from fastapi import Query
from typing import Optional
from math import ceil

# 偏移量分页（适用于中小数据集）
@app.get("/api/v1/orders")
async def list_orders(
    page: int = Query(1, ge=1, description="页码"),
    per_page: int = Query(20, ge=1, le=100, description="每页数量"),
):
    offset = (page - 1) * per_page
    total = await Order.count()
    orders = await Order.find().skip(offset).limit(per_page).to_list()

    total_pages = ceil(total / per_page)
    base_url = "/api/v1/orders"

    return {
        "data": orders,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": total,
            "total_pages": total_pages,
        },
        "links": {
            "self": f"{base_url}?page={page}&per_page={per_page}",
            "first": f"{base_url}?page=1&per_page={per_page}",
            "last": f"{base_url}?page={total_pages}&per_page={per_page}",
            "prev": f"{base_url}?page={page-1}&per_page={per_page}" if page > 1 else None,
            "next": f"{base_url}?page={page+1}&per_page={per_page}" if page < total_pages else None,
        }
    }

# 游标分页（适用于大数据集，避免深度分页）
@app.get("/api/v1/events")
async def list_events(
    cursor: Optional[str] = Query(None, description="分页游标"),
    limit: int = Query(20, ge=1, le=100, description="每页数量"),
):
    query = {}
    if cursor:
        query["id"] = {"$gt": decode_cursor(cursor)}

    events = await Event.find(query).sort("id", 1).limit(limit + 1).to_list()

    has_next = len(events) > limit
    if has_next:
        events = events[:limit]

    next_cursor = encode_cursor(events[-1]["id"]) if has_next else None

    return {
        "data": events,
        "pagination": {
            "has_next": has_next,
            "next_cursor": next_cursor,
        }
    }
```

### 5.2 过滤与排序

```python
from fastapi import Query
from typing import Optional, List
from enum import Enum

class SortOrder(str, Enum):
    ASC = "asc"
    DESC = "desc"

@app.get("/api/v1/orders")
async def list_orders(
    # 精确过滤
    status: Optional[str] = Query(None, description="订单状态"),
    user_id: Optional[int] = Query(None, description="用户 ID"),

    # 范围过滤
    min_amount: Optional[float] = Query(None, ge=0, description="最小金额"),
    max_amount: Optional[float] = Query(None, ge=0, description="最大金额"),
    created_after: Optional[datetime] = Query(None, description="创建时间起始"),
    created_before: Optional[datetime] = Query(None, description="创建时间结束"),

    # 模糊搜索
    q: Optional[str] = Query(None, min_length=2, max_length=100, description="关键词搜索"),

    # 排序
    sort_by: str = Query("created_at", description="排序字段"),
    sort_order: SortOrder = Query(SortOrder.DESC, description="排序方向"),

    # 字段选择（减少传输量）
    fields: Optional[str] = Query(None, description="返回字段，逗号分隔"),

    # 分页
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
):
    # 构建查询
    filters = {}
    if status:
        filters["status"] = status
    if user_id:
        filters["user_id"] = user_id
    if min_amount is not None:
        filters["amount__gte"] = min_amount
    if max_amount is not None:
        filters["amount__lte"] = max_amount

    # 排序白名单（防止任意字段排序导致的性能问题）
    ALLOWED_SORT_FIELDS = {"created_at", "updated_at", "amount", "status"}
    if sort_by not in ALLOWED_SORT_FIELDS:
        raise BusinessError("VALIDATION_ERROR", f"不支持按 {sort_by} 排序")

    # 执行查询...
    pass
```

---

## 步骤六：认证与授权

### 6.1 认证方案

```python
from fastapi import Depends, HTTPException, Security
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
import jwt

security = HTTPBearer()

async def verify_token(
    credentials: HTTPAuthorizationCredentials = Security(security)
) -> dict:
    """JWT Bearer Token 认证"""
    try:
        payload = jwt.decode(
            credentials.credentials,
            SECRET_KEY,
            algorithms=["HS256"],
            options={"require": ["exp", "sub", "iss"]}
        )
        return payload
    except jwt.ExpiredSignatureError:
        raise HTTPException(status_code=401, detail="Token 已过期")
    except jwt.InvalidTokenError:
        raise HTTPException(status_code=401, detail="无效的 Token")

# API Key 认证（用于服务间调用）
from fastapi.security import APIKeyHeader

api_key_header = APIKeyHeader(name="X-API-Key")

async def verify_api_key(api_key: str = Security(api_key_header)) -> dict:
    """API Key 认证"""
    client = await get_api_client(api_key)
    if not client:
        raise HTTPException(status_code=401, detail="无效的 API Key")
    return client
```

### 6.2 授权中间件

```python
from functools import wraps
from fastapi import Depends

def require_permission(permission: str):
    """基于权限的访问控制"""
    async def check_permission(user: dict = Depends(verify_token)):
        user_permissions = await get_user_permissions(user["sub"])
        if permission not in user_permissions:
            raise BusinessError("FORBIDDEN", f"缺少权限: {permission}")
        return user
    return check_permission

# 使用示例
@app.delete("/api/v1/orders/{order_id}")
async def delete_order(
    order_id: int,
    user: dict = Depends(require_permission("order:delete"))
):
    ...

# 资源级别权限检查
@app.get("/api/v1/orders/{order_id}")
async def get_order(
    order_id: int,
    user: dict = Depends(verify_token)
):
    order = await Order.get(order_id)
    if not order:
        raise BusinessError("ORDER_NOT_FOUND")
    # 检查资源所有权
    if order.user_id != user["sub"] and "admin" not in user.get("roles", []):
        raise BusinessError("FORBIDDEN")
    return order
```

---

## 步骤七：限流保护

### 7.1 限流策略

```python
import time
import redis

redis_client = redis.Redis()

class RateLimiter:
    """滑动窗口限流"""

    def __init__(self, key_prefix: str, max_requests: int, window_seconds: int):
        self.key_prefix = key_prefix
        self.max_requests = max_requests
        self.window_seconds = window_seconds

    def is_allowed(self, identifier: str) -> tuple[bool, dict]:
        key = f"rate:{self.key_prefix}:{identifier}"
        now = time.time()
        window_start = now - self.window_seconds

        pipe = redis_client.pipeline()
        pipe.zremrangebyscore(key, 0, window_start)  # 清理过期记录
        pipe.zcard(key)                                # 当前计数
        pipe.zadd(key, {str(now): now})               # 添加当前请求
        pipe.expire(key, self.window_seconds)          # 设置过期
        results = pipe.execute()

        current_count = results[1]
        remaining = max(0, self.max_requests - current_count - 1)
        reset_at = int(now + self.window_seconds)

        headers = {
            "X-RateLimit-Limit": str(self.max_requests),
            "X-RateLimit-Remaining": str(remaining),
            "X-RateLimit-Reset": str(reset_at),
        }

        if current_count >= self.max_requests:
            headers["Retry-After"] = str(self.window_seconds)
            return False, headers

        return True, headers

# 限流配置
RATE_LIMITS = {
    "default": RateLimiter("default", max_requests=100, window_seconds=60),
    "auth": RateLimiter("auth", max_requests=10, window_seconds=60),
    "search": RateLimiter("search", max_requests=30, window_seconds=60),
    "export": RateLimiter("export", max_requests=5, window_seconds=300),
}
```

### 7.2 限流中间件

```python
from fastapi import Request
from fastapi.responses import JSONResponse

@app.middleware("http")
async def rate_limit_middleware(request: Request, call_next):
    # 确定限流 key
    client_ip = request.client.host
    api_key = request.headers.get("X-API-Key", "")
    identifier = api_key or client_ip

    # 选择限流策略
    path = request.url.path
    if "/auth/" in path:
        limiter = RATE_LIMITS["auth"]
    elif "/search" in path:
        limiter = RATE_LIMITS["search"]
    elif "/export" in path:
        limiter = RATE_LIMITS["export"]
    else:
        limiter = RATE_LIMITS["default"]

    allowed, headers = limiter.is_allowed(identifier)

    if not allowed:
        return JSONResponse(
            status_code=429,
            content={"error": {"code": "RATE_LIMITED", "message": "请求频率超限"}},
            headers=headers,
        )

    response = await call_next(request)
    for k, v in headers.items():
        response.headers[k] = v
    return response
```

---

## 步骤八：API 文档

### 8.1 OpenAPI 自动生成

```python
from fastapi import FastAPI

app = FastAPI(
    title="Order Service API",
    description="订单服务 API 文档",
    version="1.2.0",
    docs_url="/api/docs",
    redoc_url="/api/redoc",
    openapi_url="/api/openapi.json",
    servers=[
        {"url": "https://api.example.com", "description": "生产环境"},
        {"url": "https://staging-api.example.com", "description": "预发布环境"},
    ],
)

# 为每个端点添加详细文档
@app.post(
    "/api/v1/orders",
    response_model=OrderResponse,
    status_code=201,
    summary="创建订单",
    description="创建新订单。需要 order:create 权限。",
    responses={
        201: {"description": "订单创建成功"},
        400: {"description": "请求参数校验失败", "model": ErrorResponse},
        401: {"description": "未认证"},
        422: {"description": "库存不足", "model": ErrorResponse},
    },
    tags=["orders"],
)
async def create_order(request: CreateOrderRequest):
    ...
```

### 8.2 API 文档检查清单

```markdown
每个 API 端点必须包含：
- [ ] 简短的摘要（summary）
- [ ] 详细的描述（description），包含使用场景
- [ ] 所有参数的类型、约束和说明
- [ ] 所有可能的响应状态码和示例
- [ ] 认证要求
- [ ] 限流配额
- [ ] 请求/响应示例（至少一个成功和一个失败）
```

---

## 验证

### API 设计评审 Checklist

```markdown
### URL 设计
- [ ] 资源命名使用复数名词
- [ ] 层级不超过 3 层
- [ ] 使用 kebab-case
- [ ] HTTP 方法语义正确

### 请求/响应
- [ ] 所有输入有校验规则
- [ ] 响应格式统一
- [ ] 日期时间使用 ISO 8601
- [ ] 金额使用最小单位（分）或 Decimal

### 错误处理
- [ ] 错误码体系完整
- [ ] 错误响应包含 request_id
- [ ] 4xx 和 5xx 区分清晰
- [ ] 不泄漏内部信息

### 安全
- [ ] 认证机制就位
- [ ] 授权检查到位（资源级别）
- [ ] 限流配置合理
- [ ] CORS 配置正确
- [ ] 敏感数据不出现在 URL 中
```

---

## 回滚方案

### API 版本回滚

```bash
# 如果新版 API 有问题，保持旧版 API 可用
# 通过 API 网关路由切换
curl -X PUT http://api-gateway/routes/v2/orders \
  -d '{"upstream": "order-service-v1", "status": "deprecated"}'

# 如果破坏性变更已影响客户端
# 1. 立即回滚服务到旧版本
kubectl rollout undo deployment/order-service -n production
# 2. 发布兼容性补丁
# 3. 通知受影响的 API 消费方
```

---

## Agent Checklist

AI 编码 Agent 在设计或实现 API 时必须逐项确认：

- [ ] **URL 规范**：资源命名、HTTP 方法、状态码均符合 RESTful 规范
- [ ] **版本策略**：使用 URL 路径版本，有版本演进计划
- [ ] **输入校验**：所有参数有类型、范围、长度约束
- [ ] **响应格式**：统一的成功和错误响应结构
- [ ] **错误处理**：完整的错误码体系，包含 request_id
- [ ] **分页实现**：列表接口有分页，大数据集使用游标分页
- [ ] **过滤排序**：排序字段有白名单限制
- [ ] **认证授权**：所有非公开接口有认证，敏感操作有授权检查
- [ ] **限流保护**：已配置合理的限流策略，响应包含限流头
- [ ] **文档完整**：OpenAPI 文档覆盖所有端点和错误码
- [ ] **幂等性**：PUT/DELETE 幂等，POST 有去重机制（如需）
- [ ] **向后兼容**：新增字段可选，不删除已有字段
