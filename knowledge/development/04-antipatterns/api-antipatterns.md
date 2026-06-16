---
id: api-antipatterns
title: API 反模式指南
domain: development
category: 04-antipatterns
difficulty: intermediate
tags: [antipatterns, api, development, error, handling, limiting, naming, rate]
quality_score: 70
last_updated: 2026-06-15
---
# API 反模式指南

> 适用范围：RESTful API / GraphQL / gRPC
> 约束级别：SHALL（必须在 API Review 阶段拦截）

---

## 1. 不一致的命名（Inconsistent Naming）

### 描述
同一 API 中混用不同的命名风格（camelCase / snake_case / kebab-case），资源名称单复数不一致，动词名词混用。导致调用方无法预测 API 的 URL 和字段名，集成成本持续上升。

### 错误示例
```
# 路径命名混乱
GET  /api/getUsers              # 动词 + 驼峰
GET  /api/order_list            # 蛇形 + 单数
POST /api/Create-Payment        # 大写 + kebab
GET  /api/v1/product/123        # 单数
GET  /api/v1/categories         # 复数

# 响应字段命名不一致
{
    "userId": 1,                  // camelCase
    "user_name": "Alice",         // snake_case
    "Email": "alice@example.com", // PascalCase
    "phone-number": "138xxxx"     // kebab-case
}
```

### 正确示例
```
# 统一的 RESTful 命名规范
GET    /api/v1/users              # 复数名词
GET    /api/v1/users/123          # 单个资源
POST   /api/v1/users              # 创建
PUT    /api/v1/users/123          # 全量更新
PATCH  /api/v1/users/123          # 部分更新
DELETE /api/v1/users/123          # 删除
GET    /api/v1/users/123/orders   # 嵌套资源

# 统一的响应字段（选择一种风格并贯穿整个 API）
{
    "user_id": 1,
    "user_name": "Alice",
    "email": "alice@example.com",
    "phone_number": "138xxxx"
}
```

```python
# 使用 OpenAPI Schema 强制命名规范
from pydantic import BaseModel, ConfigDict

class UserResponse(BaseModel):
    model_config = ConfigDict(
        alias_generator=to_camel,  # 或统一 snake_case
        populate_by_name=True,
    )
    user_id: int
    user_name: str
    email: str
    phone_number: str | None = None
```

### 检测方法
- OpenAPI / Swagger 文档审查：对所有 path 和 field 检查命名一致性。
- `spectral` lint 工具：自定义规则检查命名风格。
- API 测试中断言响应字段命名符合约定。

### 修复步骤
1. 制定团队 API 命名规范文档（路径用 kebab-case 或 snake_case，字段用 snake_case 或 camelCase）。
2. 使用 `spectral` 或自定义 lint 规则在 CI 中检查 OpenAPI spec。
3. 通过 Pydantic / Serializer 的 alias 机制统一输出格式。
4. 对已发布的 API，通过新版本逐步迁移（不可破坏性变更）。

### Agent Checklist
- [ ] API 路径统一使用复数名词、小写、无动词
- [ ] 响应字段统一使用同一种命名风格
- [ ] CI 包含 OpenAPI lint 检查
- [ ] 新旧接口命名风格一致

---

## 2. 无版本控制（Missing API Versioning）

### 描述
API 没有版本号，任何变更直接影响所有调用方。当需要做不兼容变更时，要么破坏现有客户端，要么在代码中维护大量兼容逻辑。

### 错误示例
```python
# 无版本号 -- 任何变更影响所有客户端
@app.get("/users/{user_id}")
def get_user(user_id: int):
    user = user_repo.get(user_id)
    return {
        "id": user.id,
        "name": user.name,      # 后来要拆分为 first_name + last_name
        "email": user.email,
    }

# "向下兼容"导致代码腐化
@app.get("/users/{user_id}")
def get_user(user_id: int, format: str = "v1"):
    user = user_repo.get(user_id)
    if format == "v1":
        return {"id": user.id, "name": user.name}
    elif format == "v2":
        return {"id": user.id, "first_name": user.first_name, "last_name": user.last_name}
    elif format == "v3":
        # ... 越来越多的分支
```

### 正确示例
```python
# URL 路径版本控制
from fastapi import APIRouter

router_v1 = APIRouter(prefix="/api/v1")
router_v2 = APIRouter(prefix="/api/v2")

@router_v1.get("/users/{user_id}")
def get_user_v1(user_id: int) -> UserResponseV1:
    user = user_repo.get(user_id)
    return UserResponseV1(id=user.id, name=f"{user.first_name} {user.last_name}")

@router_v2.get("/users/{user_id}")
def get_user_v2(user_id: int) -> UserResponseV2:
    user = user_repo.get(user_id)
    return UserResponseV2(
        id=user.id, first_name=user.first_name, last_name=user.last_name
    )

app.include_router(router_v1)
app.include_router(router_v2)
```

```python
# Header 版本控制（适用于微服务间调用）
from fastapi import Header, HTTPException

@app.get("/users/{user_id}")
def get_user(user_id: int, x_api_version: str = Header("2024-01-01")):
    user = user_repo.get(user_id)
    if x_api_version < "2024-06-01":
        return UserResponseV1.from_user(user)
    return UserResponseV2.from_user(user)
```

### 检测方法
- API 路径中不包含版本号（`/v1/`、`/v2/`）。
- 无 `Accept-Version` 或 `X-API-Version` header 支持。
- 同一个 endpoint handler 中包含版本分支逻辑。
- 客户端集成文档未说明版本策略。

### 修复步骤
1. 选择版本控制策略（URL 路径 vs Header vs Query Param）。
2. 为当前 API 标记为 v1，所有路径加上 `/api/v1/` 前缀。
3. 设立弃用策略：旧版本至少维护 N 个月后下线。
4. 在 API 文档和响应 Header 中标注版本信息和弃用时间。

### Agent Checklist
- [ ] 所有 API 路径包含版本号
- [ ] 不兼容变更通过新版本发布
- [ ] 旧版本有明确的弃用时间表
- [ ] API 文档标注当前最新版本

---

## 3. 缺乏错误处理（Poor Error Handling）

### 描述
API 返回的错误信息不统一、不可机器解析、或泄露内部实现细节。调用方无法程序化处理错误，只能靠人工阅读错误消息。

### 错误示例
```python
@app.post("/orders")
def create_order(data: dict):
    try:
        order = order_service.create(data)
        return order
    except Exception as e:
        # 问题 1: 所有错误返回 500
        # 问题 2: 泄露堆栈信息
        # 问题 3: 无统一格式
        return {"error": str(e)}, 500

# 不同接口返回不同的错误格式
# 接口 A: {"error": "Not found"}
# 接口 B: {"message": "用户不存在", "code": -1}
# 接口 C: {"err_msg": "insufficient balance", "err_code": 40001}
```

### 正确示例
```python
from enum import Enum
from pydantic import BaseModel

class ErrorCode(str, Enum):
    VALIDATION_ERROR = "VALIDATION_ERROR"
    NOT_FOUND = "NOT_FOUND"
    CONFLICT = "CONFLICT"
    INSUFFICIENT_BALANCE = "INSUFFICIENT_BALANCE"
    RATE_LIMITED = "RATE_LIMITED"
    INTERNAL_ERROR = "INTERNAL_ERROR"

class ErrorResponse(BaseModel):
    error: ErrorCode
    message: str
    details: list[dict] | None = None
    request_id: str

class AppException(Exception):
    def __init__(self, code: ErrorCode, message: str, status: int = 400, details=None):
        self.code = code
        self.message = message
        self.status = status
        self.details = details

@app.exception_handler(AppException)
async def app_exception_handler(request: Request, exc: AppException):
    return JSONResponse(
        status_code=exc.status,
        content=ErrorResponse(
            error=exc.code,
            message=exc.message,
            details=exc.details,
            request_id=request.state.request_id,
        ).model_dump(),
    )

@app.exception_handler(Exception)
async def generic_exception_handler(request: Request, exc: Exception):
    logger.exception("Unhandled exception", request_id=request.state.request_id)
    return JSONResponse(
        status_code=500,
        content=ErrorResponse(
            error=ErrorCode.INTERNAL_ERROR,
            message="An internal error occurred. Please try again later.",
            request_id=request.state.request_id,
        ).model_dump(),
    )

# 业务代码使用统一异常
@app.post("/orders", response_model=OrderResponse, status_code=201)
def create_order(data: CreateOrderRequest):
    user = user_repo.get(data.user_id)
    if not user:
        raise AppException(
            code=ErrorCode.NOT_FOUND,
            message=f"User {data.user_id} not found",
            status=404,
        )
    if user.balance < data.total:
        raise AppException(
            code=ErrorCode.INSUFFICIENT_BALANCE,
            message="Insufficient balance to place this order",
            status=422,
            details=[{"field": "total", "required": str(data.total), "available": str(user.balance)}],
        )
    return order_service.create(user, data)
```

### 检测方法
- 不同接口的错误响应 JSON 结构不一致。
- 存在 `except Exception: return str(e)` 模式。
- 错误响应中包含堆栈信息、文件路径、SQL 语句。
- 所有错误都返回 HTTP 500。

### 修复步骤
1. 定义统一的 `ErrorResponse` 模型和 `ErrorCode` 枚举。
2. 创建自定义异常基类 `AppException`，包含错误码、消息、HTTP 状态码。
3. 注册全局异常处理器，拦截所有异常并转为统一格式。
4. 在生产环境中隐藏内部错误详情，只返回 request_id 供排查。
5. 更新 API 文档，列出每个接口可能返回的错误码。

### Agent Checklist
- [ ] 所有接口使用统一的错误响应格式
- [ ] 错误码使用枚举，可机器解析
- [ ] 生产环境不泄露堆栈信息
- [ ] 每个错误响应包含 request_id
- [ ] API 文档列出每个接口的错误码

---

## 4. 过大响应（Oversized Response）

### 描述
API 返回了远超客户端需要的数据量，包括不必要的嵌套对象、大文本字段、二进制数据。导致带宽浪费、客户端解析慢、移动端体验差。

### 错误示例
```python
@app.get("/users/{user_id}")
def get_user(user_id: int):
    user = user_repo.get_with_all_relations(user_id)
    return {
        "id": user.id,
        "name": user.name,
        "email": user.email,
        "avatar_base64": user.avatar_base64,     # 500KB 的 base64 图片
        "orders": [                               # 所有历史订单
            {
                "id": o.id,
                "items": [                        # 每个订单的所有商品
                    {
                        "product": {              # 嵌套完整商品信息
                            "id": p.id,
                            "description": p.description,  # 长文本
                            "images": p.images,            # 图片列表
                        }
                    } for p in o.items
                ]
            } for o in user.orders                # 可能有几千个订单
        ],
        "audit_log": user.audit_log,              # 审计日志，不应暴露给客户端
    }
```

### 正确示例
```python
# 基础响应 -- 只包含核心字段
class UserSummary(BaseModel):
    id: int
    name: str
    email: str
    avatar_url: str  # URL 而非 base64

# 详情响应 -- 按需扩展
class UserDetail(UserSummary):
    phone: str | None
    created_at: datetime
    order_count: int  # 聚合数字而非完整列表

@app.get("/users/{user_id}", response_model=UserSummary)
def get_user(user_id: int):
    return user_repo.get_summary(user_id)

@app.get("/users/{user_id}/detail", response_model=UserDetail)
def get_user_detail(user_id: int):
    return user_repo.get_detail(user_id)

# 订单通过独立的分页接口获取
@app.get("/users/{user_id}/orders", response_model=PaginatedResponse[OrderSummary])
def get_user_orders(
    user_id: int,
    page: int = Query(1, ge=1),
    page_size: int = Query(20, ge=1, le=100),
):
    return order_repo.get_by_user(user_id, page=page, page_size=page_size)

# GraphQL 方案 -- 客户端自选字段
type_defs = """
type User {
    id: ID!
    name: String!
    email: String!
    avatarUrl: String
    orders(first: Int = 10, after: String): OrderConnection
}
"""
```

### 检测方法
- 响应 JSON 大小 > 100KB。
- 响应中包含 base64 编码的二进制数据。
- 响应中嵌套层级 > 3 层。
- 响应中包含客户端不需要的内部字段（audit_log、internal_id 等）。

### 修复步骤
1. 分析前端实际使用的字段，删除未使用的字段。
2. 将大对象（图片、文件）改为 URL 引用。
3. 将嵌套列表拆分为独立的分页接口。
4. 使用不同粒度的响应模型（Summary / Detail / Full）。
5. 考虑 GraphQL 或 JSON:API sparse fieldsets 让客户端自选字段。

### Agent Checklist
- [ ] 单个响应 JSON < 100KB
- [ ] 无 base64 编码的二进制数据（使用 URL）
- [ ] 嵌套层级 <= 3
- [ ] 列表数据通过独立分页接口获取
- [ ] 响应模型不包含内部字段

---

## 5. 无限流 / 缺乏限流（Missing Rate Limiting）

### 描述
API 没有请求频率限制，任何客户端可以无限制地发起请求。导致 DDoS 攻击、恶意爬虫、单个客户端耗尽系统资源影响其他用户。

### 错误示例
```python
# 无任何限流措施
@app.post("/api/send-sms")
def send_sms(phone: str, message: str):
    # 恶意调用者可以无限发送短信，消耗短信费用
    sms_service.send(phone, message)
    return {"status": "sent"}

@app.get("/api/search")
def search(query: str):
    # 无限制搜索请求，可能打垮搜索引擎
    return search_engine.query(query)

# 登录接口无限制 -- 可暴力破解
@app.post("/api/login")
def login(username: str, password: str):
    user = authenticate(username, password)
    if not user:
        return {"error": "Invalid credentials"}, 401
    return {"token": create_token(user)}
```

### 正确示例
```python
from slowapi import Limiter
from slowapi.util import get_remote_address
import redis

limiter = Limiter(key_func=get_remote_address)

# 全局默认限流
@app.middleware("http")
async def rate_limit_middleware(request: Request, call_next):
    # 全局：每分钟 60 次
    pass

# 精细化限流
@app.post("/api/send-sms")
@limiter.limit("5/minute")  # 短信接口：每分钟 5 次
def send_sms(request: Request, data: SmsRequest):
    sms_service.send(data.phone, data.message)
    return {"status": "sent"}

@app.post("/api/login")
@limiter.limit("10/minute")  # 登录接口：每分钟 10 次
def login(request: Request, credentials: LoginRequest):
    user = authenticate(credentials.username, credentials.password)
    if not user:
        # 记录失败次数，连续失败超过阈值锁定账号
        failed_count = login_tracker.increment(credentials.username)
        if failed_count >= MAX_LOGIN_ATTEMPTS:
            login_tracker.lock(credentials.username, duration=LOCKOUT_MINUTES)
        raise AppException(ErrorCode.UNAUTHORIZED, "Invalid credentials", status=401)
    login_tracker.reset(credentials.username)
    return {"token": create_token(user)}

# API Key 维度的限流
class ApiKeyRateLimiter:
    def __init__(self, redis_client: redis.Redis):
        self._redis = redis_client

    def check(self, api_key: str, limit: int, window_seconds: int) -> bool:
        key = f"rate_limit:{api_key}"
        current = self._redis.incr(key)
        if current == 1:
            self._redis.expire(key, window_seconds)
        return current <= limit
```

```
# 响应 Header 包含限流信息
HTTP/1.1 200 OK
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1704067200

# 超限时返回 429
HTTP/1.1 429 Too Many Requests
Retry-After: 30
{
    "error": "RATE_LIMITED",
    "message": "Too many requests. Please retry after 30 seconds.",
    "request_id": "req_abc123"
}
```

### 检测方法
- API 代码中无限流 middleware 或装饰器。
- 响应 Header 中无 `X-RateLimit-*` 字段。
- 负载测试中无 429 响应。
- 短信、邮件、登录等敏感接口无独立的限流策略。

### 修复步骤
1. 引入限流中间件（`slowapi` / Nginx `limit_req` / API Gateway）。
2. 设置全局默认限流（如 60 次/分钟）。
3. 对敏感接口（登录、短信、支付）设置更严格的限流。
4. 在响应 Header 中返回限流信息。
5. 超限时返回 HTTP 429 + `Retry-After` header。
6. 使用 Redis 实现分布式限流（多实例部署场景）。

### Agent Checklist
- [ ] 所有公开 API 有全局限流
- [ ] 敏感接口（登录/短信/支付）有独立限流
- [ ] 响应包含 `X-RateLimit-*` Header
- [ ] 超限返回 HTTP 429 + `Retry-After`
- [ ] 登录接口有连续失败锁定机制

---

## 6. 无认证 / 认证不当（Missing or Broken Authentication）

### 描述
API 没有认证机制，或认证实现存在严重缺陷（Token 不过期、无刷新机制、敏感接口可匿名访问、JWT 密钥硬编码）。

### 错误示例
```python
# 无认证：任何人都能访问
@app.get("/api/users")
def list_users():
    return user_repo.get_all()

@app.delete("/api/users/{user_id}")
def delete_user(user_id: int):
    user_repo.delete(user_id)
    return {"status": "deleted"}

# JWT 实现缺陷
SECRET_KEY = "super-secret-123"  # 硬编码密钥

def create_token(user):
    return jwt.encode(
        {"user_id": user.id},       # 无过期时间
        SECRET_KEY,
        algorithm="HS256"
    )

def verify_token(token):
    try:
        return jwt.decode(token, SECRET_KEY, algorithms=["HS256"])
    except:
        return None  # 吞掉所有异常
```

### 正确示例
```python
import os
from datetime import datetime, timedelta, timezone
from fastapi import Depends, HTTPException, Security
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer

# 密钥从环境变量读取
JWT_SECRET = os.environ["JWT_SECRET_KEY"]
JWT_ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 15
REFRESH_TOKEN_EXPIRE_DAYS = 7

security = HTTPBearer()

def create_access_token(user_id: int, roles: list[str]) -> str:
    now = datetime.now(timezone.utc)
    payload = {
        "sub": str(user_id),
        "roles": roles,
        "iat": now,
        "exp": now + timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES),
        "jti": str(uuid4()),  # 唯一 ID，支持 Token 吊销
    }
    return jwt.encode(payload, JWT_SECRET, algorithm=JWT_ALGORITHM)

def create_refresh_token(user_id: int) -> str:
    now = datetime.now(timezone.utc)
    payload = {
        "sub": str(user_id),
        "type": "refresh",
        "exp": now + timedelta(days=REFRESH_TOKEN_EXPIRE_DAYS),
        "jti": str(uuid4()),
    }
    return jwt.encode(payload, JWT_SECRET, algorithm=JWT_ALGORITHM)

async def get_current_user(
    credentials: HTTPAuthorizationCredentials = Security(security),
) -> User:
    try:
        payload = jwt.decode(
            credentials.credentials, JWT_SECRET, algorithms=[JWT_ALGORITHM]
        )
    except jwt.ExpiredSignatureError:
        raise HTTPException(status_code=401, detail="Token expired")
    except jwt.InvalidTokenError:
        raise HTTPException(status_code=401, detail="Invalid token")

    # 检查 Token 是否被吊销
    if token_blacklist.is_revoked(payload["jti"]):
        raise HTTPException(status_code=401, detail="Token revoked")

    user = user_repo.get(int(payload["sub"]))
    if not user:
        raise HTTPException(status_code=401, detail="User not found")
    return user

def require_role(*roles: str):
    async def role_checker(user: User = Depends(get_current_user)):
        if not any(r in user.roles for r in roles):
            raise HTTPException(status_code=403, detail="Insufficient permissions")
        return user
    return role_checker

# 接口使用认证和授权
@app.get("/api/users", dependencies=[Depends(require_role("admin"))])
def list_users(current_user: User = Depends(get_current_user)):
    return user_repo.get_all()

@app.delete("/api/users/{user_id}", dependencies=[Depends(require_role("admin"))])
def delete_user(user_id: int, current_user: User = Depends(get_current_user)):
    if user_id == current_user.id:
        raise AppException(ErrorCode.CONFLICT, "Cannot delete yourself")
    user_repo.soft_delete(user_id, operator=current_user.id)
    return {"status": "deleted"}
```

### 检测方法
- 接口 handler 无认证依赖（`Depends(get_current_user)` 或中间件）。
- JWT 密钥硬编码在源码中。
- Token 无 `exp` 字段或过期时间 > 24 小时。
- 无 Token 吊销机制（黑名单）。
- 管理接口和普通接口使用相同的权限级别。

### 修复步骤
1. 建立全局认证中间件，默认所有接口需要认证（白名单制）。
2. JWT 密钥从环境变量读取，不同环境使用不同密钥。
3. Access Token 过期时间设为 15 分钟，Refresh Token 设为 7 天。
4. 实现 Token 吊销机制（Redis 黑名单或数据库记录）。
5. 使用 RBAC 对不同接口设置不同的角色要求。
6. 安全扫描工具（`bandit` / `semgrep`）检查硬编码密钥。

### Agent Checklist
- [ ] 所有非公开接口要求认证
- [ ] JWT 密钥从环境变量读取
- [ ] Access Token 过期时间 <= 30 分钟
- [ ] 有 Refresh Token 机制
- [ ] Token 支持吊销（黑名单）
- [ ] 管理接口有独立的角色校验
- [ ] 无硬编码的密钥或 Token

---

## 全局 Agent Checklist

| 检查项 | 阈值 | 工具 |
|--------|------|------|
| API 命名一致性 | 100% | `spectral` / OpenAPI lint |
| API 版本号 | 必须有 | URL / Header 检查 |
| 错误格式统一性 | 100% | API 测试 / Contract 测试 |
| 单响应大小 | < 100KB | APM / 网络监控 |
| 限流覆盖率 | 100% 公开接口 | 压力测试 / 429 检查 |
| 认证覆盖率 | 100% 非公开接口 | 安全扫描 / Pentest |
| 硬编码密钥 | 0 处 | `bandit` / `semgrep` |
