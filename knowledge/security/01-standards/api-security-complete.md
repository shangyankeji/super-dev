---
id: api-security-complete
title: API安全完整指南
domain: security
category: 01-standards
difficulty: intermediate
tags: [agent, api, checklist, complete, security, 实战代码示例, 常见陷阱, 最佳实践]
quality_score: 70
last_updated: 2026-06-15
---
# API安全完整指南

## 概述
API是现代应用的核心攻击面。本指南覆盖API认证、授权、限流、CORS、输入验证、HTTPS、API Gateway安全等完整防护体系,帮助团队构建安全的API服务。

## 核心概念

### 1. API安全层次模型
- **传输层**: HTTPS/TLS — 加密通信
- **认证层**: 验证调用者身份(API Key/OAuth2/JWT)
- **授权层**: 验证调用者权限(RBAC/ABAC/Scope)
- **输入层**: 验证请求数据(Schema验证/注入防护)
- **限流层**: 防止滥用(速率限制/配额)
- **审计层**: 记录所有操作(日志/审计追踪)

### 2. OWASP API Security Top 10 (2023)
1. **BOLA**: 对象级授权缺失(越权访问他人资源)
2. **Broken Authentication**: 认证机制缺陷
3. **Broken Object Property Level Authorization**: 属性级越权
4. **Unrestricted Resource Consumption**: 无限制资源消耗
5. **Broken Function Level Authorization**: 功能级越权
6. **Unrestricted Access to Sensitive Business Flows**: 业务流程滥用
7. **Server Side Request Forgery**: SSRF
8. **Security Misconfiguration**: 安全配置错误
9. **Improper Inventory Management**: API资产管理不当
10. **Unsafe Consumption of APIs**: 不安全的API调用

### 3. 认证方案安全等级
| 方案 | 安全性 | 适用场景 | 注意事项 |
|------|--------|----------|----------|
| API Key | 低 | 服务间调用/简单集成 | 不要在URL中传递 |
| Bearer Token(JWT) | 中 | SPA/移动端 | 短有效期+刷新 |
| OAuth2 | 高 | 第三方授权 | 使用PKCE |
| mTLS | 最高 | 服务间/零信任 | 证书管理复杂 |

## 实战代码示例

### 认证中间件

```python
# FastAPI多层认证
from fastapi import FastAPI, Depends, HTTPException, Security
from fastapi.security import HTTPBearer, APIKeyHeader, OAuth2PasswordBearer
from jose import jwt, JWTError

app = FastAPI()

# API Key认证
api_key_header = APIKeyHeader(name="X-API-Key")

async def verify_api_key(api_key: str = Security(api_key_header)) -> str:
    hashed = hashlib.sha256(api_key.encode()).hexdigest()
    client = await get_client_by_key_hash(hashed)
    if not client or not client.is_active:
        raise HTTPException(status_code=401, detail="Invalid API key")
    return client.id

# JWT认证
bearer_scheme = HTTPBearer()

async def verify_jwt(credentials = Security(bearer_scheme)) -> dict:
    token = credentials.credentials
    try:
        payload = jwt.decode(
            token,
            PUBLIC_KEY,
            algorithms=["RS256"],
            audience="https://api.example.com",
            issuer="https://auth.example.com",
        )
        return payload
    except JWTError as e:
        raise HTTPException(status_code=401, detail="Invalid token")

# 组合认证(API Key或JWT)
async def authenticate(
    request: Request,
    api_key: str = Security(api_key_header, auto_error=False),
    jwt_token = Security(bearer_scheme, auto_error=False),
) -> AuthContext:
    if api_key:
        client_id = await verify_api_key(api_key)
        return AuthContext(type="api_key", client_id=client_id)
    if jwt_token:
        payload = await verify_jwt(jwt_token)
        return AuthContext(type="jwt", user_id=payload["sub"], scopes=payload.get("scope", []))
    raise HTTPException(status_code=401, detail="Authentication required")
```

### 授权控制(RBAC/ABAC)

```python
# 基于角色和权限的授权
from functools import wraps
from enum import Enum

class Permission(str, Enum):
    READ_USERS = "users:read"
    WRITE_USERS = "users:write"
    DELETE_USERS = "users:delete"
    READ_ORDERS = "orders:read"
    MANAGE_ORDERS = "orders:manage"
    ADMIN = "admin:*"

ROLE_PERMISSIONS = {
    "viewer": [Permission.READ_USERS, Permission.READ_ORDERS],
    "editor": [Permission.READ_USERS, Permission.WRITE_USERS, Permission.READ_ORDERS],
    "admin": [Permission.ADMIN],
}

def require_permissions(*permissions: Permission):
    """权限检查装饰器"""
    def decorator(func):
        @wraps(func)
        async def wrapper(*args, auth: AuthContext = Depends(authenticate), **kwargs):
            user_permissions = get_user_permissions(auth)
            if Permission.ADMIN in user_permissions:
                return await func(*args, auth=auth, **kwargs)
            for perm in permissions:
                if perm not in user_permissions:
                    raise HTTPException(
                        status_code=403,
                        detail=f"Missing permission: {perm}",
                    )
            return await func(*args, auth=auth, **kwargs)
        return wrapper
    return decorator

# 对象级授权(防BOLA)
@app.get("/api/orders/{order_id}")
@require_permissions(Permission.READ_ORDERS)
async def get_order(order_id: int, auth: AuthContext = Depends(authenticate)):
    order = await order_repo.get(order_id)
    if not order:
        raise HTTPException(404, "Order not found")
    # 关键: 验证资源归属
    if auth.type != "admin" and order.user_id != auth.user_id:
        raise HTTPException(403, "Access denied")
    return order
```

### 速率限制

```python
# Redis滑动窗口限流
from redis.asyncio import Redis
import time

class RateLimiter:
    def __init__(self, redis: Redis):
        self.redis = redis

    async def is_allowed(
        self,
        key: str,
        max_requests: int,
        window_seconds: int,
    ) -> tuple[bool, dict]:
        """滑动窗口限流"""
        now = time.time()
        window_start = now - window_seconds
        pipe = self.redis.pipeline()

        # 清除窗口外的请求记录
        pipe.zremrangebyscore(key, 0, window_start)
        # 统计当前窗口请求数
        pipe.zcard(key)
        # 添加当前请求
        pipe.zadd(key, {str(now): now})
        # 设置过期时间
        pipe.expire(key, window_seconds)

        results = await pipe.execute()
        current_count = results[1]

        headers = {
            "X-RateLimit-Limit": str(max_requests),
            "X-RateLimit-Remaining": str(max(0, max_requests - current_count - 1)),
            "X-RateLimit-Reset": str(int(now + window_seconds)),
        }

        if current_count >= max_requests:
            return False, headers
        return True, headers

# 限流中间件
rate_limiter = RateLimiter(redis)

RATE_LIMITS = {
    "default": {"max_requests": 100, "window": 60},
    "auth": {"max_requests": 5, "window": 60},       # 登录限制更严格
    "upload": {"max_requests": 10, "window": 3600},    # 上传限制
}

class RateLimitMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request, call_next):
        # 确定限流规则
        path = request.url.path
        if path.startswith("/auth"):
            rule = RATE_LIMITS["auth"]
        elif path.startswith("/upload"):
            rule = RATE_LIMITS["upload"]
        else:
            rule = RATE_LIMITS["default"]

        # 限流Key: IP + 路径前缀
        client_ip = request.client.host
        key = f"ratelimit:{client_ip}:{path.split('/')[1]}"

        allowed, headers = await rate_limiter.is_allowed(
            key, rule["max_requests"], rule["window"]
        )

        if not allowed:
            return JSONResponse(
                status_code=429,
                content={"error": "Rate limit exceeded"},
                headers=headers,
            )

        response = await call_next(request)
        for k, v in headers.items():
            response.headers[k] = v
        return response
```

### 输入验证与注入防护

```python
# Pydantic严格输入验证
from pydantic import BaseModel, validator, Field, EmailStr
from typing import Annotated
import re
import bleach

class CreateUserRequest(BaseModel):
    name: Annotated[str, Field(min_length=1, max_length=100, pattern=r'^[\w\s\-]+$')]
    email: EmailStr
    age: Annotated[int, Field(ge=0, le=150)]
    bio: Annotated[str, Field(max_length=1000)] = ""
    website: Annotated[str, Field(max_length=200)] = ""

    @validator("name")
    def sanitize_name(cls, v):
        # 清除HTML标签
        return bleach.clean(v, tags=[], strip=True)

    @validator("bio")
    def sanitize_bio(cls, v):
        # 只允许安全的HTML标签
        return bleach.clean(v, tags=["b", "i", "p", "br"], strip=True)

    @validator("website")
    def validate_website(cls, v):
        if v and not v.startswith(("https://", "http://")):
            raise ValueError("Website must start with http:// or https://")
        return v

# SQL注入防护 — 始终使用参数化查询
async def search_users(query: str):
    # 错误: f"SELECT * FROM users WHERE name LIKE '%{query}%'"
    # 正确: 参数化
    result = await db.execute(
        text("SELECT * FROM users WHERE name LIKE :query"),
        {"query": f"%{query}%"},
    )
    return result.fetchall()
```

### CORS安全配置

```python
# 严格的CORS配置
from fastapi.middleware.cors import CORSMiddleware

# 生产环境: 明确指定允许的域名
ALLOWED_ORIGINS = [
    "https://app.example.com",
    "https://admin.example.com",
]

app.add_middleware(
    CORSMiddleware,
    allow_origins=ALLOWED_ORIGINS,      # 不要使用["*"]
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "DELETE"],
    allow_headers=["Authorization", "Content-Type", "X-Request-ID"],
    expose_headers=["X-RateLimit-Limit", "X-RateLimit-Remaining"],
    max_age=3600,                       # 预检缓存1小时
)
```

### 安全响应头

```python
# 安全头中间件
class SecurityHeadersMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request, call_next):
        response = await call_next(request)

        response.headers["X-Content-Type-Options"] = "nosniff"
        response.headers["X-Frame-Options"] = "DENY"
        response.headers["X-XSS-Protection"] = "0"  # 现代浏览器建议关闭
        response.headers["Strict-Transport-Security"] = "max-age=31536000; includeSubDomains"
        response.headers["Content-Security-Policy"] = "default-src 'self'"
        response.headers["Referrer-Policy"] = "strict-origin-when-cross-origin"
        response.headers["Permissions-Policy"] = "camera=(), microphone=(), geolocation=()"

        # 移除泄露信息的头
        response.headers.pop("Server", None)
        response.headers.pop("X-Powered-By", None)

        return response
```

### API Gateway安全(nginx示例)

```nginx
# nginx API Gateway安全配置
upstream api_backend {
    server api-1:8080;
    server api-2:8080;
}

# 限流配置
limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=auth_limit:10m rate=3r/m;
limit_conn_zone $binary_remote_addr zone=conn_limit:10m;

server {
    listen 443 ssl http2;
    server_name api.example.com;

    # TLS配置
    ssl_certificate /etc/ssl/certs/api.crt;
    ssl_certificate_key /etc/ssl/private/api.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # 请求体大小限制
    client_max_body_size 10m;
    client_body_timeout 10s;
    client_header_timeout 10s;

    # 安全头
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;
    add_header Strict-Transport-Security "max-age=31536000" always;

    # API路由 — 通用限流
    location /api/ {
        limit_req zone=api_limit burst=20 nodelay;
        limit_conn conn_limit 10;

        proxy_pass http://api_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # 超时
        proxy_connect_timeout 5s;
        proxy_read_timeout 30s;
        proxy_send_timeout 10s;
    }

    # 认证端点 — 更严格限流
    location /auth/ {
        limit_req zone=auth_limit burst=5 nodelay;
        proxy_pass http://api_backend;
    }

    # 隐藏内部端点
    location /internal/ {
        deny all;
        return 404;
    }

    # 健康检查不限流
    location /health {
        proxy_pass http://api_backend;
    }
}
```

## 最佳实践

### 1. 认证安全
- API Key通过Header传递,不放URL(日志泄露)
- JWT使用RS256(非对称),不用HS256
- Token有效期: Access 15min, Refresh 7天
- 实现Token黑名单(登出/异常)

### 2. 授权安全
- 每个端点都要验证权限(默认拒绝)
- 对象级授权: 检查资源归属(防BOLA)
- 属性级授权: 不同角色返回不同字段
- 使用中间件统一授权,避免遗漏

### 3. 输入安全
- 使用Schema验证(Pydantic/Joi/Zod)
- 限制请求体大小
- 参数化SQL查询(永远不拼接)
- HTML输出编码(防XSS)
- 文件上传: 验证类型/大小/内容

### 4. 传输安全
- 全链路HTTPS(包括内部服务)
- HSTS头强制HTTPS
- TLS 1.2+,禁用旧协议
- 证书自动续期(Let's Encrypt/cert-manager)

### 5. 审计与监控
- 记录所有认证事件(成功/失败)
- 记录敏感操作(删除/修改/导出)
- 监控异常模式(暴力破解/扫描)
- 告警: 高频401/403/429

## 常见陷阱

### 陷阱1: BOLA(对象级越权)
```python
# 错误: 只验证了认证,没验证资源归属
@app.get("/api/users/{user_id}/orders")
async def get_orders(user_id: int, auth = Depends(authenticate)):
    return await order_repo.get_by_user(user_id)  # 任何人都能查看!

# 正确: 验证资源归属
@app.get("/api/users/{user_id}/orders")
async def get_orders(user_id: int, auth = Depends(authenticate)):
    if auth.user_id != user_id and not auth.is_admin:
        raise HTTPException(403, "Access denied")
    return await order_repo.get_by_user(user_id)
```

### 陷阱2: 批量端点遗漏权限检查
```python
# 错误: 批量删除没有逐条检查权限
@app.delete("/api/orders/batch")
async def batch_delete(order_ids: list[int], auth = Depends(authenticate)):
    await order_repo.delete_many(order_ids)  # 可能删除别人的订单!

# 正确: 验证每个资源的归属
@app.delete("/api/orders/batch")
async def batch_delete(order_ids: list[int], auth = Depends(authenticate)):
    orders = await order_repo.get_many(order_ids)
    for order in orders:
        if order.user_id != auth.user_id:
            raise HTTPException(403, f"No access to order {order.id}")
    await order_repo.delete_many(order_ids)
```

### 陷阱3: 错误响应泄露信息
```python
# 错误: 暴露内部实现
except DatabaseError as e:
    raise HTTPException(500, detail=str(e))  # 泄露SQL/表结构

# 正确: 统一错误格式,隐藏内部细节
except DatabaseError as e:
    logger.error("Database error", error=str(e), request_id=request_id)
    raise HTTPException(500, detail="Internal server error")
```

### 陷阱4: CORS配置过宽
```python
# 错误
allow_origins=["*"],
allow_credentials=True,  # 与*冲突且不安全!

# 正确
allow_origins=["https://app.example.com"],
allow_credentials=True,
```

## Agent Checklist

### 认证与授权
- [ ] 所有端点都要求认证(除公开端点)
- [ ] 对象级授权已实现(检查资源归属)
- [ ] 功能级授权已实现(角色/权限检查)
- [ ] 认证失败统一返回401,授权失败返回403

### 输入验证
- [ ] 所有输入使用Schema验证
- [ ] SQL查询使用参数化
- [ ] 请求体大小已限制
- [ ] 文件上传有类型和大小验证

### 传输与配置
- [ ] 全链路HTTPS
- [ ] CORS限制到具体域名
- [ ] 安全响应头已配置
- [ ] 敏感头信息已移除(Server等)

### 限流与监控
- [ ] API限流已配置
- [ ] 认证端点有更严格的限流
- [ ] 认证事件有审计日志
- [ ] 异常访问模式有告警
