---
id: owasp-api-security-playbook
title: OWASP API Security Top 10 企业级防护手册
domain: security
category: 02-playbooks
difficulty: advanced
tags: [security, owasp, api, bola, jwt, oauth2, rbac, rate-limiting, input-validation, cors, authorization, authentication, enterprise]
quality_score: 95
maintainer: security-team@superdev.com
last_updated: 2026-06-14
---

# OWASP API Security Top 10 企业级防护手册

> 基于 [OWASP API Security Top 10 (2023)](https://owasp.org/API-Security/) + 2025 企业级实践

## API1:2023 — Broken Object Level Authorization (BOLA)

**风险**：用户 A 能访问用户 B 的数据（最常见、最严重的 API 漏洞）。

```python
# ❌ 只检查是否登录，没检查资源归属
@app.get("/api/orders/{order_id}")
def get_order(order_id, current_user=Depends(get_current_user)):
    return db.query(Order).get(order_id)  # 任何用户都能看任何订单！

# ✅ 检查资源归属
@app.get("/api/orders/{order_id}")
def get_order(order_id, current_user=Depends(get_current_user)):
    order = db.query(Order).get(order_id)
    if order.user_id != current_user.id and not current_user.is_admin:
        raise HTTPException(403, "Forbidden")
    return order
```

## API2:2023 — Broken Authentication

```python
# ❌ 弱密码策略 + 无锁定 + JWT 永不过期
jwt_token = create_jwt(user_id, expires_in=NEVER)

# ✅ 强密码 + 锁定 + 短 TTL JWT + 刷新令牌
@app.post("/api/auth/login")
def login(email, password):
    user = verify_credentials(email, password)
    # 5 次失败锁定 15 分钟
    if failed_attempts(email) >= 5:
        lock_account(email, duration=900)
    return {
        "access_token": create_jwt(user_id, expires_in=900),    # 15min
        "refresh_token": create_refresh_token(user_id, expires_in=604800),  # 7d
    }
```

## API3:2023 — Broken Object Property Level Authorization

```python
# ❌ Mass assignment — 用户可以修改不该改的字段
@app.patch("/api/users/{id}")
def update_user(id, data: dict):
    return db.update(User, id, **data)  # data 含 role="admin"?!

# ✅ 白名单字段
class UpdateUserRequest(BaseModel):
    name: str | None = None
    email: str | None = None
    # 不暴露 role, is_admin, password_hash

@app.patch("/api/users/{id}")
def update_user(id, data: UpdateUserRequest):
    return db.update(User, id, **data.dict(exclude_unset=True))
```

## API4:2023 — Unrestricted Resource Consumption

```python
# ❌ 无限制（OOM/DoS 风险）
@app.post("/api/upload")
def upload(file: UploadFile):
    save(file)  # 10GB 文件？

@app.get("/api/users")
def list_users():
    return db.query(User).all()  # 100 万行？

# ✅ 限制 + 分页 + 速率
@app.post("/api/upload")
async def upload(file: UploadFile = File(max_size=10_000_000)):  # 10MB 上限
    save(file)

@app.get("/api/users")
@rate_limit(100, per_minute=True)
def list_users(page: int = 1, limit: int = Field(default=20, le=100)):
    return db.query(User).offset((page-1)*limit).limit(limit).all()
```

## API5:2023 — Broken Function Level Authorization

```python
# ❌ 只在前端隐藏 admin 按钮，API 无权限检查
@app.delete("/api/users/{id}")
def delete_user(id):
    db.delete(User, id)  # 任何登录用户都能删任何人

# ✅ 后端 RBAC
@app.delete("/api/users/{id}")
@requires_role("admin")  # 装饰器检查
def delete_user(id, current_user):
    db.delete(User, id)
```

## JWT 硬化清单

| 配置项 | 推荐值 | 说明 |
|--------|--------|------|
| 算法 | RS256/ES256 | 不用 HS256（对称密钥泄露风险）|
| Access TTL | 15min | 短 TTL 降低泄露影响 |
| Refresh TTL | 7d | 带轮换（每次刷新换新 token）|
| 密钥轮换 | 90d | 定期换签名密钥 |
| 不存 localStorage | — | 用 HttpOnly Cookie 防 XSS |
| 撤销列表 | Redis | JWT 默认无状态，需黑名单机制 |

## OAuth2 企业级要点

- **不用 implicit flow**（已弃用）— 用 Authorization Code + PKCE
- **redirect_uri 白名单** — 精确匹配，不允许通配
- **state 参数** — 防 CSRF
- **scope 最小化** — 只申请必需权限
- **token 撤销端点** — 支持主动撤销
