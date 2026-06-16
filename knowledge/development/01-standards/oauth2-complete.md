---
id: oauth2-complete
title: OAuth2.0完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, oauth2, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# OAuth2.0完整指南

## 概述
OAuth2.0是授权框架,允许第三方应用访问用户资源而无需暴露密码。本指南覆盖授权流程、令牌管理、JWT和最佳实践。

## 核心概念

### 1. 四种授权模式

**授权码(Authorization Code)**:
```python
# 用于服务器端应用
from authlib.integrations.flask_oauth2 import AuthorizationServer

# 授权端点
@app.route('/oauth/authorize', methods=['GET', 'POST'])
def authorize():
    # 验证用户
    user = current_user()
    
    # 生成授权码
    grant = authorization_server.validate_authorization_request()
    return grant.create_authorization_response()

# 令牌端点
@app.route('/oauth/token', methods=['POST'])
def issue_token():
    return authorization_server.create_token_response()
```

**隐式(Implicit)** - 已弃用,不推荐

**密码模式(Resource Owner Password Credentials)** - 已弃用

**客户端凭证(Client Credentials)**:
```python
# 用于机器对机器通信
@app.route('/oauth/token', methods=['POST'])
def client_credentials():
    grant = authorization_server.validate_token_request()
    return grant.create_token_response()
```

### 2. JWT令牌

**生成JWT**:
```python
import jwt
from datetime import datetime, timedelta

def generate_jwt(user_id: int) -> str:
    payload = {
        'sub': user_id,
        'iat': datetime.utcnow(),
        'exp': datetime.utcnow() + timedelta(hours=1),
        'scope': 'read write'
    }
    
    token = jwt.encode(payload, SECRET_KEY, algorithm='HS256')
    return token

def verify_jwt(token: str) -> dict:
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=['HS256'])
        return payload
    except jwt.ExpiredSignatureError:
        raise ValueError('Token expired')
    except jwt.InvalidTokenError:
        raise ValueError('Invalid token')
```

**刷新令牌**:
```python
def refresh_access_token(refresh_token: str) -> str:
    # 验证刷新令牌
    payload = verify_jwt(refresh_token)
    
    # 检查是否在黑名单
    if is_token_revoked(refresh_token):
        raise ValueError('Token revoked')
    
    # 生成新访问令牌
    new_access_token = generate_jwt(payload['sub'])
    return new_access_token
```

### 3. OpenID Connect

**Google OAuth2**:
```python
from authlib.integrations.flask_client import OAuth

oauth = OAuth()

google = oauth.register(
    'google',
    client_id='YOUR_CLIENT_ID',
    client_secret='YOUR_CLIENT_SECRET',
    server_metadata_url='https://accounts.google.com/.well-known/openid-configuration',
    client_kwargs={'scope': 'openid email profile'}
)

@app.route('/login/google')
def login_google():
    redirect_uri = url_for('authorize_google', _external=True)
    return google.authorize_redirect(redirect_uri)

@app.route('/auth/google')
def authorize_google():
    token = google.authorize_access_token()
    user_info = google.parse_id_token(token)
    
    # 创建或获取用户
    user = get_or_create_user(user_info)
    
    # 生成本地JWT
    access_token = generate_jwt(user.id)
    
    return {'access_token': access_token}
```

### 4. FastAPI集成

```python
from fastapi import FastAPI, Depends, HTTPException, status
from fastapi.security import OAuth2PasswordBearer, OAuth2PasswordRequestForm
from passlib.context import CryptContext

app = FastAPI()
oauth2_scheme = OAuth2PasswordBearer(tokenUrl='token')
pwd_context = CryptContext(schemes=['bcrypt'], deprecated='auto')

# 获取当前用户
async def get_current_user(token: str = Depends(oauth2_scheme)):
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail='Could not validate credentials',
        headers={'WWW-Authenticate': 'Bearer'},
    )
    
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=['HS256'])
        user_id: int = payload.get('sub')
        if user_id is None:
            raise credentials_exception
    except jwt.JWTError:
        raise credentials_exception
    
    user = get_user(user_id)
    if user is None:
        raise credentials_exception
    
    return user

# 登录端点
@app.post('/token')
async def login(form_data: OAuth2PasswordRequestForm = Depends()):
    user = authenticate_user(form_data.username, form_data.password)
    if not user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail='Incorrect username or password',
            headers={'WWW-Authenticate': 'Bearer'},
        )
    
    access_token = generate_jwt(user.id)
    return {'access_token': access_token, 'token_type': 'bearer'}

# 受保护端点
@app.get('/users/me')
async def read_users_me(current_user = Depends(get_current_user)):
    return current_user
```

### 5. 权限和作用域

```python
from functools import wraps

def require_scope(required_scope: str):
    def decorator(func):
        @wraps(func)
        async def wrapper(*args, token: str = Depends(oauth2_scheme), **kwargs):
            payload = verify_jwt(token)
            scopes = payload.get('scope', '').split()
            
            if required_scope not in scopes:
                raise HTTPException(
                    status_code=status.HTTP_403_FORBIDDEN,
                    detail='Insufficient permissions'
                )
            
            return await func(*args, **kwargs)
        return wrapper
    return decorator

# 使用装饰器
@app.get('/admin/users')
@require_scope('admin')
async def list_all_users():
    return get_all_users()
```

## 最佳实践

### ✅ DO

1. **使用HTTPS**
```python
# ✅ 好
from fastapi.middleware.httpsredirect import HTTPSRedirectMiddleware
app.add_middleware(HTTPSRedirectMiddleware)
```

2. **短有效期访问令牌**
```python
# ✅ 好
access_token_expires = timedelta(minutes=15)
refresh_token_expires = timedelta(days=7)
```

3. **令牌撤销**
```python
# 使用Redis黑名单
def revoke_token(token: str):
    redis.setex(f'revoked:{token}', 3600, '1')
```

### ❌ DON'T

1. **不要在URL中传递令牌**
```python
# ❌ 差
GET /api/users?token=xxx

# ✅ 好
GET /api/users
Authorization: Bearer xxx
```

2. **不要存储明文密码**
```python
# ❌ 差
password = 'user_password'

# ✅ 好
hashed = pwd_context.hash(password)
```

## 学习路径

### 初级 (1周)
1. OAuth2.0基础概念
2. 授权流程
3. 令牌使用

### 中级 (1-2周)
1. JWT实现
2. OpenID Connect
3. FastAPI集成

### 高级 (2-3周)
1. 权限管理
2. 令牌撤销
3. 安全审计

---

**知识ID**: `oauth2-complete`  
**领域**: development  
**类型**: standards  
**难度**: intermediate  
**质量分**: 93  
**维护者**: security-team@superdev.com  
**最后更新**: 2026-03-28
