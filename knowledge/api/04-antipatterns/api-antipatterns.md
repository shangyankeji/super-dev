---
id: api-antipatterns
title: API 设计反模式（避坑指南）
domain: api
category: 04-antipatterns
difficulty: intermediate
tags: [api, antipattern, rest, error, validation, security, n-plus-1, pagination]
quality_score: 87
maintainer: platform-team@superdev.com
last_updated: 2026-06-14
---

# API 设计反模式（避坑指南）

## 1. 动词路径
```
❌ POST /api/createUser
❌ GET /api/getOrdersByUser?userId=123
✅ POST /api/users
✅ GET /api/users/123/orders
```

## 2. 返回内部错误细节
```json
// ❌ 泄露数据库结构
{
  "error": "psycopg2.errors.UniqueViolation: duplicate key value violates users_email_key"
}

// ✅ 标准错误信封
{
  "error": {
    "code": "CONFLICT",
    "message": "A user with this email already exists",
    "details": [{"field": "email", "issue": "already registered"}]
  }
}
```

## 3. N+1 查询
```python
# ❌ 每个用户单独查订单（N+1）
for user in users:
    user.orders = db.query(Order).filter_by(user_id=user.id).all()

# ✅ 一次查全部（eager loading）
users = db.query(User).options(joinedload(User.orders)).all()
```

## 4. 不分页的 list 端点
```python
# ❌ 返回全部数据
@app.get("/products")
def list_products():
    return db.query(Product).all()  # 10 万行！

# ✅ 强制分页
@app.get("/products")
def list_products(page: int = 1, limit: int = Field(default=20, le=100)):
    return db.query(Product).offset((page-1)*limit).limit(limit).all()
```

## 5. 缺少幂等性
```python
# ❌ POST 不幂等 → 网络重试导致重复创建
@app.post("/orders")
def create_order(data):
    return db.insert(Order(**data))

# ✅ 幂等键防止重复
@app.post("/orders")
def create_order(data, idempotency_key: str = Header(...)):
    existing = db.query(Order).filter_by(idempotency_key=idempotency_key).first()
    if existing:
        return existing  # 幂等返回
    return db.insert(Order(**data))
```

## 6. 过度暴露字段
```python
# ❌ 返回所有字段包括敏感数据
return user.to_dict()  # 含 password_hash, internal_notes

# ✅ 显式选择返回字段
return {"id": user.id, "name": user.name, "email": user.email}
```

## 7. 版本化缺失
```
# ❌ 破坏性变更直接改 /api/users
# 旧客户端在无警告下崩溃

# ✅ 新版本走 /api/v2/users
# 旧版本至少维护 6 个月 + Sunset 头
```

## 8. 同步阻塞处理
```python
# ❌ 请求内发邮件（阻塞 5 秒）
@app.post("/register")
def register(data):
    user = create_user(data)
    send_welcome_email(user)  # 阻塞！
    return user

# ✅ 异步队列
@app.post("/register")
def register(data):
    user = create_user(data)
    enqueue_job("send_welcome_email", user.id)  # 非阻塞
    return user
```
