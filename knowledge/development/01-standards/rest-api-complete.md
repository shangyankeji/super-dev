---
id: rest-api-complete
title: RESTful API设计完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [api, complete, development, rest, 学习路径, 最佳实践, 核心原则, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# RESTful API设计完整指南

## 概述
REST是Web API架构风格,使用HTTP方法操作资源。本指南覆盖REST原则、端点设计、状态码、版本控制和最佳实践。

## 核心原则

### 1. REST约束

- 客户端-服务器分离
- 无状态(Stateless)
- 可缓存(Cacheable)
- 统一接口(Uniform Interface)
- 分层系统(Layered System)

### 2. 资源和端点

**资源命名**:
```
# ✅ 好 - 使用名词复数
GET    /users
GET    /users/{id}
POST   /users
PUT    /users/{id}
DELETE /users/{id}

# 嵌套资源
GET    /users/{id}/posts
POST   /users/{id}/posts

# ❌ 差 - 使用动词
GET    /getUsers
POST   /createUser
```

### 3. HTTP方法

```python
from fastapi import FastAPI, HTTPException

app = FastAPI()

# 获取资源集合
@app.get('/users')
async def list_users(skip: int = 0, limit: int = 10):
    return users[skip:skip+limit]

# 获取单个资源
@app.get('/users/{user_id}')
async def get_user(user_id: int):
    user = db.get(user_id)
    if not user:
        raise HTTPException(status_code=404, detail='User not found')
    return user

# 创建资源
@app.post('/users', status_code=201)
async def create_user(user: UserCreate):
    new_user = db.create(user)
    return new_user

# 完整更新
@app.put('/users/{user_id}')
async def update_user(user_id: int, user: UserUpdate):
    updated = db.update(user_id, user)
    if not updated:
        raise HTTPException(status_code=404)
    return updated

# 部分更新
@app.patch('/users/{user_id}')
async def partial_update_user(user_id: int, user: UserPatch):
    updated = db.partial_update(user_id, user)
    return updated

# 删除资源
@app.delete('/users/{user_id}', status_code=204)
async def delete_user(user_id: int):
    db.delete(user_id)
    return None
```

### 4. 状态码

**成功响应**:
```
200 OK - 成功
201 Created - 资源创建成功
204 No Content - 删除成功,无返回内容
```

**客户端错误**:
```
400 Bad Request - 请求格式错误
401 Unauthorized - 未认证
403 Forbidden - 无权限
404 Not Found - 资源不存在
409 Conflict - 资源冲突
422 Unprocessable Entity - 验证失败
429 Too Many Requests - 限流
```

**服务器错误**:
```
500 Internal Server Error - 服务器错误
503 Service Unavailable - 服务不可用
```

### 5. 过滤、排序、分页

```python
@app.get('/articles')
async def list_articles(
    status: Optional[str] = None,
    author: Optional[int] = None,
    sort: str = 'created_at',
    order: str = 'desc',
    page: int = 1,
    per_page: int = 20
):
    query = db.query(Article)
    
    # 过滤
    if status:
        query = query.filter(Article.status == status)
    if author:
        query = query.filter(Article.author_id == author)
    
    # 排序
    sort_field = getattr(Article, sort)
    if order == 'desc':
        query = query.order_by(sort_field.desc())
    else:
        query = query.order_by(sort_field.asc())
    
    # 分页
    total = query.count()
    items = query.offset((page - 1) * per_page).limit(per_page).all()
    
    return {
        'items': items,
        'total': total,
        'page': page,
        'per_page': per_page,
        'pages': (total + per_page - 1) // per_page
    }
```

### 6. 版本控制

**URL版本控制**:
```python
@app.get('/v1/users')
async def list_users_v1():
    pass

@app.get('/v2/users')
async def list_users_v2():
    pass
```

**Header版本控制**:
```python
@app.get('/users')
async def list_users(version: str = Header(default='1.0')):
    if version == '2.0':
        return v2_response()
    return v1_response()
```

### 7. HATEOAS

```python
@app.get('/users/{user_id}')
async def get_user(user_id: int, request: Request):
    user = db.get(user_id)
    
    # 添加链接
    return {
        'data': user,
        '_links': {
            'self': {'href': str(request.url)},
            'posts': {'href': f'/users/{user_id}/posts'},
            'update': {'href': f'/users/{user_id}', 'method': 'PUT'},
            'delete': {'href': f'/users/{user_id}', 'method': 'DELETE'}
        }
    }
```

### 8. 错误处理

```python
from fastapi import FastAPI
from fastapi.responses import JSONResponse

class APIError(Exception):
    def __init__(self, status_code: int, message: str, details: dict = None):
        self.status_code = status_code
        self.message = message
        self.details = details or {}

@app.exception_handler(APIError)
async def api_error_handler(request: Request, exc: APIError):
    return JSONResponse(
        status_code=exc.status_code,
        content={
            'error': {
                'code': exc.status_code,
                'message': exc.message,
                'details': exc.details
            }
        }
    )

# 使用
@app.get('/users/{user_id}')
async def get_user(user_id: int):
    user = db.get(user_id)
    if not user:
        raise APIError(404, 'User not found', {'user_id': user_id})
    return user
```

## 最佳实践

### ✅ DO

1. **使用复数名词**
```
✅ /users, /posts, /comments
❌ /user, /post, /comment
```

2. **返回适当状态码**
```python
# ✅ 好
@app.post('/users', status_code=201)
async def create_user():
    pass

# ❌ 差 - 总是返回200
@app.post('/users')
async def create_user():
    return {'status': 'created'}
```

3. **支持内容协商**
```python
@app.get('/users')
async def list_users(accept: str = Header(default='application/json')):
    users = get_users()
    
    if accept == 'application/xml':
        return XMLResponse(users)
    
    return users
```

### ❌ DON'T

1. **不要在URL中使用动词**
```
❌ /users/create
❌ /users/delete/{id}
```

2. **不要返回敏感信息**
```python
# ❌ 差
{
    'password': 'hashed_value',
    'api_key': 'secret'
}

# ✅ 好
{
    'email': 'user@example.com'
}
```

## 学习路径

### 初级 (1周)
1. REST原则
2. HTTP方法
3. 资源命名

### 中级 (1-2周)
1. 状态码
2. 过滤分页
3. 版本控制

### 高级 (2-3周)
1. HATEOAS
2. 内容协商
3. API文档

---

**知识ID**: `rest-api-complete`  
**领域**: development  
**类型**: standards  
**难度**: intermediate  
**质量分**: 94  
**维护者**: api-team@superdev.com  
**最后更新**: 2026-03-28
