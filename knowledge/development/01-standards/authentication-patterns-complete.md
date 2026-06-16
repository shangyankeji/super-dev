---
id: authentication-patterns-complete
title: 认证模式完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [agent, authentication, checklist, complete, development, patterns, 实战代码示例, 常见陷阱]
quality_score: 70
last_updated: 2026-06-15
---
# 认证模式完整指南

## 概述
认证(Authentication)是确认用户身份的过程,是安全体系的第一道防线。本指南覆盖Session、JWT、OAuth2、OIDC、Passkey、MFA六种核心认证模式,提供选型矩阵、实战代码和安全配置。

## 核心概念

### 1. 认证 vs 授权
- **认证(AuthN)**: 你是谁? — 验证身份
- **授权(AuthZ)**: 你能做什么? — 验证权限
- 本指南聚焦认证,授权参见RBAC/ABAC相关文档

### 2. 认证模式对比

| 模式 | 状态 | 存储 | 适用场景 | 复杂度 |
|------|------|------|----------|--------|
| Session | 有状态 | 服务端(Redis/DB) | 传统Web应用 | 低 |
| JWT | 无状态 | 客户端(Cookie/Header) | API/微服务/SPA | 中 |
| OAuth2 | 委托认证 | 授权服务器 | 第三方登录/API授权 | 高 |
| OIDC | OAuth2+身份层 | 授权服务器 | SSO/企业登录 | 高 |
| Passkey | 无密码 | 设备+服务端 | 现代Web应用 | 中 |
| API Key | 无状态 | 服务端 | 服务间调用/简单API | 低 |

### 3. 安全层级
- **Level 1**: 密码 — 最基础,需配合密码策略
- **Level 2**: 密码 + MFA — 增加第二因素
- **Level 3**: Passkey/FIDO2 — 无密码,抗钓鱼
- **Level 4**: 硬件安全密钥 — 最高安全级别

## 实战代码示例

### Session认证

```python
# FastAPI + Redis Session
from fastapi import FastAPI, Request, Response, Depends, HTTPException
from redis.asyncio import Redis
import uuid
import json

app = FastAPI()
redis = Redis(host="localhost", port=6379, decode_responses=True)

SESSION_TTL = 3600 * 24  # 24小时
SESSION_COOKIE = "session_id"

async def create_session(user_id: int, user_data: dict) -> str:
    """创建会话"""
    session_id = str(uuid.uuid4())
    session_data = json.dumps({
        "user_id": user_id,
        **user_data,
    })
    await redis.setex(f"session:{session_id}", SESSION_TTL, session_data)
    return session_id

async def get_current_user(request: Request) -> dict:
    """从Session获取当前用户"""
    session_id = request.cookies.get(SESSION_COOKIE)
    if not session_id:
        raise HTTPException(status_code=401, detail="Not authenticated")

    session_data = await redis.get(f"session:{session_id}")
    if not session_data:
        raise HTTPException(status_code=401, detail="Session expired")

    # 续期
    await redis.expire(f"session:{session_id}", SESSION_TTL)
    return json.loads(session_data)

@app.post("/login")
async def login(response: Response, email: str, password: str):
    user = await verify_credentials(email, password)
    if not user:
        raise HTTPException(status_code=401, detail="Invalid credentials")

    session_id = await create_session(user.id, {"email": user.email, "role": user.role})

    response.set_cookie(
        key=SESSION_COOKIE,
        value=session_id,
        httponly=True,      # JS无法访问
        secure=True,        # 仅HTTPS
        samesite="lax",     # CSRF防护
        max_age=SESSION_TTL,
    )
    return {"message": "Login successful"}

@app.post("/logout")
async def logout(request: Request, response: Response):
    session_id = request.cookies.get(SESSION_COOKIE)
    if session_id:
        await redis.delete(f"session:{session_id}")
    response.delete_cookie(SESSION_COOKIE)
    return {"message": "Logged out"}

@app.get("/me")
async def me(user: dict = Depends(get_current_user)):
    return user
```

### JWT认证

```python
# JWT认证实现
from datetime import datetime, timedelta
from jose import jwt, JWTError
from passlib.context import CryptContext
from pydantic import BaseModel

SECRET_KEY = "your-secret-key-from-env"  # 必须从环境变量读取
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE = timedelta(minutes=15)
REFRESH_TOKEN_EXPIRE = timedelta(days=7)

pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")

class TokenPair(BaseModel):
    access_token: str
    refresh_token: str
    token_type: str = "bearer"

def create_access_token(user_id: int, roles: list[str]) -> str:
    payload = {
        "sub": str(user_id),
        "roles": roles,
        "type": "access",
        "exp": datetime.utcnow() + ACCESS_TOKEN_EXPIRE,
        "iat": datetime.utcnow(),
        "jti": str(uuid.uuid4()),  # JWT ID,用于黑名单
    }
    return jwt.encode(payload, SECRET_KEY, algorithm=ALGORITHM)

def create_refresh_token(user_id: int) -> str:
    payload = {
        "sub": str(user_id),
        "type": "refresh",
        "exp": datetime.utcnow() + REFRESH_TOKEN_EXPIRE,
        "iat": datetime.utcnow(),
        "jti": str(uuid.uuid4()),
    }
    return jwt.encode(payload, SECRET_KEY, algorithm=ALGORITHM)

async def verify_token(token: str, expected_type: str = "access") -> dict:
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        if payload.get("type") != expected_type:
            raise HTTPException(status_code=401, detail="Invalid token type")
        # 检查黑名单(登出后的token)
        if await is_token_blacklisted(payload["jti"]):
            raise HTTPException(status_code=401, detail="Token revoked")
        return payload
    except JWTError:
        raise HTTPException(status_code=401, detail="Invalid token")

# Token刷新端点
@app.post("/auth/refresh", response_model=TokenPair)
async def refresh_tokens(refresh_token: str):
    payload = await verify_token(refresh_token, expected_type="refresh")
    user_id = int(payload["sub"])

    # 吊销旧refresh token
    await blacklist_token(payload["jti"], REFRESH_TOKEN_EXPIRE)

    user = await get_user(user_id)
    return TokenPair(
        access_token=create_access_token(user.id, user.roles),
        refresh_token=create_refresh_token(user.id),
    )

# Token黑名单(Redis)
async def blacklist_token(jti: str, ttl: timedelta):
    await redis.setex(f"blacklist:{jti}", int(ttl.total_seconds()), "1")

async def is_token_blacklisted(jti: str) -> bool:
    return await redis.exists(f"blacklist:{jti}")
```

### OAuth2授权码流程

```python
# OAuth2 GitHub登录
from authlib.integrations.starlette_client import OAuth

oauth = OAuth()
oauth.register(
    name='github',
    client_id='your-client-id',
    client_secret='your-client-secret',
    access_token_url='https://github.com/login/oauth/access_token',
    authorize_url='https://github.com/login/oauth/authorize',
    api_base_url='https://api.github.com/',
    client_kwargs={'scope': 'user:email'},
)

@app.get("/auth/github")
async def github_login(request: Request):
    """重定向到GitHub授权页面"""
    redirect_uri = request.url_for('github_callback')
    return await oauth.github.authorize_redirect(request, redirect_uri)

@app.get("/auth/github/callback")
async def github_callback(request: Request):
    """GitHub回调处理"""
    token = await oauth.github.authorize_access_token(request)
    user_info = await oauth.github.get('user', token=token)
    user_data = user_info.json()

    # 查找或创建本地用户
    user = await find_or_create_user(
        provider="github",
        provider_id=str(user_data["id"]),
        email=user_data.get("email"),
        name=user_data["name"],
        avatar=user_data["avatar_url"],
    )

    # 创建本地Session/JWT
    session_id = await create_session(user.id, {"email": user.email})
    response = RedirectResponse(url="/dashboard")
    response.set_cookie(SESSION_COOKIE, session_id, httponly=True, secure=True)
    return response
```

### Passkey/WebAuthn

```python
# Passkey注册和认证(使用py_webauthn库)
from webauthn import (
    generate_registration_options,
    verify_registration_response,
    generate_authentication_options,
    verify_authentication_response,
)
from webauthn.helpers.structs import (
    AuthenticatorSelectionCriteria,
    ResidentKeyRequirement,
    UserVerificationRequirement,
)

RP_ID = "example.com"
RP_NAME = "My App"
ORIGIN = "https://example.com"

@app.post("/auth/passkey/register/begin")
async def passkey_register_begin(user: dict = Depends(get_current_user)):
    """开始Passkey注册"""
    options = generate_registration_options(
        rp_id=RP_ID,
        rp_name=RP_NAME,
        user_id=str(user["id"]).encode(),
        user_name=user["email"],
        user_display_name=user.get("name", user["email"]),
        authenticator_selection=AuthenticatorSelectionCriteria(
            resident_key=ResidentKeyRequirement.REQUIRED,
            user_verification=UserVerificationRequirement.REQUIRED,
        ),
    )
    # 保存challenge用于验证
    await redis.setex(
        f"passkey_challenge:{user['id']}",
        300,
        options.challenge.hex(),
    )
    return options

@app.post("/auth/passkey/register/complete")
async def passkey_register_complete(
    credential: dict,
    user: dict = Depends(get_current_user),
):
    """完成Passkey注册"""
    challenge = bytes.fromhex(
        await redis.get(f"passkey_challenge:{user['id']}")
    )
    verification = verify_registration_response(
        credential=credential,
        expected_challenge=challenge,
        expected_rp_id=RP_ID,
        expected_origin=ORIGIN,
    )
    # 保存公钥凭证
    await save_passkey(
        user_id=user["id"],
        credential_id=verification.credential_id,
        public_key=verification.credential_public_key,
        sign_count=verification.sign_count,
    )
    return {"status": "registered"}
```

### MFA多因素认证

```python
# TOTP(基于时间的一次性密码)
import pyotp
import qrcode
import io
import base64

class MFAService:
    @staticmethod
    def generate_secret() -> str:
        """生成TOTP密钥"""
        return pyotp.random_base32()

    @staticmethod
    def get_qr_code(secret: str, email: str) -> str:
        """生成二维码(用于Google Authenticator等APP扫描)"""
        totp = pyotp.TOTP(secret)
        uri = totp.provisioning_uri(name=email, issuer_name="MyApp")
        img = qrcode.make(uri)
        buffer = io.BytesIO()
        img.save(buffer, format='PNG')
        return base64.b64encode(buffer.getvalue()).decode()

    @staticmethod
    def verify_totp(secret: str, code: str) -> bool:
        """验证TOTP码"""
        totp = pyotp.TOTP(secret)
        return totp.verify(code, valid_window=1)  # 允许前后30秒

@app.post("/auth/mfa/enable")
async def enable_mfa(user: dict = Depends(get_current_user)):
    secret = MFAService.generate_secret()
    # 临时保存,等验证后再持久化
    await redis.setex(f"mfa_setup:{user['id']}", 600, secret)
    qr_code = MFAService.get_qr_code(secret, user["email"])
    return {"qr_code": qr_code, "secret": secret}

@app.post("/auth/mfa/verify-setup")
async def verify_mfa_setup(
    code: str,
    user: dict = Depends(get_current_user),
):
    secret = await redis.get(f"mfa_setup:{user['id']}")
    if not secret:
        raise HTTPException(400, "MFA setup expired")
    if not MFAService.verify_totp(secret, code):
        raise HTTPException(400, "Invalid code")

    # 持久化密钥
    await save_mfa_secret(user["id"], secret)
    # 生成恢复码
    recovery_codes = [str(uuid.uuid4())[:8] for _ in range(10)]
    await save_recovery_codes(user["id"], recovery_codes)
    return {"recovery_codes": recovery_codes}

@app.post("/auth/login")
async def login_with_mfa(email: str, password: str, mfa_code: str = None):
    user = await verify_credentials(email, password)
    if not user:
        raise HTTPException(401, "Invalid credentials")

    if user.mfa_enabled:
        if not mfa_code:
            return {"requires_mfa": True, "mfa_token": create_mfa_token(user.id)}
        if not MFAService.verify_totp(user.mfa_secret, mfa_code):
            raise HTTPException(401, "Invalid MFA code")

    return create_token_pair(user)
```

## 最佳实践

### 1. 密码存储
- 使用bcrypt/scrypt/Argon2,永远不用MD5/SHA系列
- 每个密码独立随机盐值
- 成本因子定期调整(bcrypt rounds >= 12)
- 密码策略: 最少8字符,不限制最大长度

### 2. JWT安全配置
- Access Token短有效期(15分钟)
- Refresh Token用于续期(7-30天)
- 使用httpOnly Cookie存储而非localStorage
- 实现Token黑名单(登出/强制下线)
- 生产环境使用RS256而非HS256(方便密钥轮换)

### 3. Session安全
- Session ID使用密码学安全随机数
- httpOnly + Secure + SameSite Cookie属性
- 登录后轮换Session ID(防Session Fixation)
- 设置合理的空闲超时和绝对超时

### 4. OAuth2安全
- 使用PKCE(Proof Key for Code Exchange)
- state参数防CSRF
- 验证redirect_uri白名单
- Access Token不要暴露给前端

### 5. MFA部署
- 提供多种第二因素(TOTP/SMS/邮件/Passkey)
- 生成恢复码并安全存储
- 敏感操作(改密码/改邮箱)要求重新认证
- 管理员账户强制启用MFA

## 常见陷阱

### 陷阱1: JWT存在localStorage
```javascript
// 错误: XSS攻击可以窃取token
localStorage.setItem('token', jwt)

// 正确: 使用httpOnly Cookie
// 服务端设置,JS无法读取
Set-Cookie: token=xxx; HttpOnly; Secure; SameSite=Lax
```

### 陷阱2: 无Token刷新机制
```python
# 错误: Access Token有效期7天(太长,泄露风险大)
# 正确: Access Token 15分钟 + Refresh Token 7天
# Refresh Token可以单独吊销
```

### 陷阱3: 密码重置流程不安全
```python
# 错误: 重置链接永不过期,可重复使用
# 正确: 一次性令牌 + 短有效期
async def create_reset_token(user_id: int) -> str:
    token = secrets.token_urlsafe(32)
    await redis.setex(f"reset:{token}", 3600, str(user_id))  # 1小时过期
    return token

async def verify_reset_token(token: str) -> int:
    user_id = await redis.get(f"reset:{token}")
    if not user_id:
        raise HTTPException(400, "Invalid or expired reset link")
    await redis.delete(f"reset:{token}")  # 一次性使用
    return int(user_id)
```

### 陷阱4: 登录错误信息泄露
```python
# 错误: 暴露用户是否存在
if not user:
    raise HTTPException(401, "User not found")  # 泄露用户存在性
if not verify_password(password, user.hashed_password):
    raise HTTPException(401, "Wrong password")   # 泄露密码错误

# 正确: 统一错误消息
raise HTTPException(401, "Invalid email or password")
```

## Agent Checklist

### 认证方案选择
- [ ] 根据应用类型选择合适的认证模式
- [ ] 评估安全需求级别
- [ ] 确认是否需要MFA
- [ ] SSO/第三方登录需求已评估

### 安全配置
- [ ] 密码使用bcrypt/Argon2存储
- [ ] Token有效期合理(Access短/Refresh长)
- [ ] Cookie设置httpOnly+Secure+SameSite
- [ ] CSRF保护已启用

### Token管理
- [ ] Token刷新机制已实现
- [ ] Token黑名单/吊销机制已实现
- [ ] 登出清理所有会话/Token
- [ ] 密钥轮换方案已设计

### 用户体验
- [ ] 登录错误消息不泄露信息
- [ ] 密码重置流程安全且友好
- [ ] MFA恢复码已提供
- [ ] 记住我功能安全实现
