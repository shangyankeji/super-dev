---
id: python-web-frameworks-comparison
title: Python Web框架对比
domain: development
category: 01-standards
difficulty: intermediate
tags: [agent, checklist, comparison, development, frameworks, python, web, 场景选型指南]
quality_score: 70
last_updated: 2026-06-15
---
# Python Web框架对比

## 概述
Python Web框架生态丰富,从轻量级微框架到全功能框架各有擅长。本指南深入对比FastAPI、Django、Flask、Starlette、Litestar五大主流框架,帮助团队在不同场景下做出最优选择。

## 核心概念

### 1. 框架分类
- **全功能框架(Batteries-included)**: Django — ORM/Admin/Auth/模板全内置
- **微框架(Micro-framework)**: Flask — 核心精简,按需组合
- **异步优先(Async-first)**: FastAPI/Starlette/Litestar — 原生异步,高并发
- **ASGI vs WSGI**: ASGI支持异步/WebSocket,WSGI仅同步HTTP

### 2. 框架对比总览

| 特性 | FastAPI | Django | Flask | Starlette | Litestar |
|------|---------|--------|-------|-----------|----------|
| 类型 | 异步API | 全功能 | 微框架 | 异步底层 | 异步API |
| 协议 | ASGI | WSGI/ASGI | WSGI | ASGI | ASGI |
| 类型提示 | 原生Pydantic | 可选 | 可选 | 有限 | 原生 |
| ORM | 无(推荐SQLAlchemy) | 内置Django ORM | 无 | 无 | 无 |
| Admin | 无(有社区方案) | 内置 | 无 | 无 | 无 |
| API文档 | 自动Swagger/ReDoc | DRF插件 | 需插件 | 无 | 自动 |
| 学习曲线 | 中等 | 较高 | 低 | 低 | 中等 |
| 性能(req/s) | ~15K | ~3K | ~4K | ~18K | ~16K |
| Stars(2025) | 78K+ | 80K+ | 68K+ | 10K+ | 5K+ |

### 3. 技术栈生态

| 框架 | 数据库 | 认证 | 缓存 | 任务队列 |
|------|--------|------|------|----------|
| FastAPI | SQLAlchemy/Tortoise | 自行实现/fastapi-users | Redis/自行 | Celery/ARQ |
| Django | Django ORM | 内置Auth | 内置cache框架 | Celery/Django-Q |
| Flask | SQLAlchemy(Flask-SQLAlchemy) | Flask-Login | Flask-Caching | Celery |
| Starlette | 任意 | 内置基础 | 自行实现 | 自行选择 |
| Litestar | SQLAlchemy/Tortoise | 内置JWT/Session | 内置 | SAQ |

## 实战代码示例

### FastAPI — 现代异步API

```python
from fastapi import FastAPI, Depends, HTTPException, Query
from pydantic import BaseModel, EmailStr
from sqlalchemy.ext.asyncio import AsyncSession
from typing import Annotated

app = FastAPI(title="User API", version="1.0.0")

# 请求/响应模型自动生成文档
class UserCreate(BaseModel):
    name: str
    email: EmailStr
    age: int = Query(ge=0, le=150)

class UserResponse(BaseModel):
    id: int
    name: str
    email: str

    model_config = {"from_attributes": True}

# 依赖注入
async def get_db() -> AsyncSession:
    async with async_session_factory() as session:
        yield session

@app.post("/users", response_model=UserResponse, status_code=201)
async def create_user(
    user: UserCreate,
    db: Annotated[AsyncSession, Depends(get_db)]
):
    """创建新用户,自动生成Swagger文档"""
    db_user = User(**user.model_dump())
    db.add(db_user)
    await db.commit()
    await db.refresh(db_user)
    return db_user

@app.get("/users/{user_id}", response_model=UserResponse)
async def get_user(
    user_id: int,
    db: Annotated[AsyncSession, Depends(get_db)]
):
    user = await db.get(User, user_id)
    if not user:
        raise HTTPException(status_code=404, detail="User not found")
    return user

# WebSocket支持
@app.websocket("/ws")
async def websocket_endpoint(websocket):
    await websocket.accept()
    while True:
        data = await websocket.receive_text()
        await websocket.send_text(f"Echo: {data}")
```

### Django — 全功能Web框架

```python
# models.py
from django.db import models
from django.contrib.auth.models import AbstractUser

class User(AbstractUser):
    bio = models.TextField(blank=True)
    avatar = models.ImageField(upload_to="avatars/", null=True)

class Article(models.Model):
    title = models.CharField(max_length=200)
    content = models.TextField()
    author = models.ForeignKey(User, on_delete=models.CASCADE, related_name="articles")
    created_at = models.DateTimeField(auto_now_add=True)
    published = models.BooleanField(default=False)

    class Meta:
        ordering = ["-created_at"]
        indexes = [models.Index(fields=["published", "-created_at"])]

# views.py (DRF)
from rest_framework import viewsets, permissions, filters
from rest_framework.decorators import action
from rest_framework.response import Response
from django_filters.rest_framework import DjangoFilterBackend

class ArticleViewSet(viewsets.ModelViewSet):
    queryset = Article.objects.select_related("author")
    serializer_class = ArticleSerializer
    permission_classes = [permissions.IsAuthenticatedOrReadOnly]
    filter_backends = [DjangoFilterBackend, filters.SearchFilter]
    filterset_fields = ["published", "author"]
    search_fields = ["title", "content"]

    def perform_create(self, serializer):
        serializer.save(author=self.request.user)

    @action(detail=True, methods=["post"])
    def publish(self, request, pk=None):
        article = self.get_object()
        article.published = True
        article.save()
        return Response({"status": "published"})

# admin.py — 自动管理后台
from django.contrib import admin

@admin.register(Article)
class ArticleAdmin(admin.ModelAdmin):
    list_display = ["title", "author", "published", "created_at"]
    list_filter = ["published", "created_at"]
    search_fields = ["title", "content"]
    actions = ["make_published"]

    @admin.action(description="Publish selected articles")
    def make_published(self, request, queryset):
        queryset.update(published=True)
```

### Flask — 轻量微框架

```python
from flask import Flask, request, jsonify, abort
from flask_sqlalchemy import SQLAlchemy
from flask_marshmallow import Marshmallow
from functools import wraps

app = Flask(__name__)
app.config["SQLALCHEMY_DATABASE_URI"] = "sqlite:///app.db"
db = SQLAlchemy(app)
ma = Marshmallow(app)

class User(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    name = db.Column(db.String(100), nullable=False)
    email = db.Column(db.String(120), unique=True, nullable=False)

class UserSchema(ma.SQLAlchemyAutoSchema):
    class Meta:
        model = User
        load_instance = True

user_schema = UserSchema()
users_schema = UserSchema(many=True)

def require_auth(f):
    @wraps(f)
    def decorated(*args, **kwargs):
        token = request.headers.get("Authorization")
        if not token or not verify_token(token):
            abort(401)
        return f(*args, **kwargs)
    return decorated

@app.route("/users", methods=["POST"])
@require_auth
def create_user():
    data = request.get_json()
    errors = user_schema.validate(data)
    if errors:
        return jsonify(errors), 400
    user = user_schema.load(data)
    db.session.add(user)
    db.session.commit()
    return user_schema.dump(user), 201

@app.route("/users/<int:user_id>")
def get_user(user_id):
    user = User.query.get_or_404(user_id)
    return user_schema.dump(user)

@app.errorhandler(404)
def not_found(error):
    return jsonify({"error": "Resource not found"}), 404
```

### Starlette — 高性能ASGI基础

```python
from starlette.applications import Starlette
from starlette.routing import Route, Mount, WebSocketRoute
from starlette.requests import Request
from starlette.responses import JSONResponse
from starlette.middleware import Middleware
from starlette.middleware.cors import CORSMiddleware

async def homepage(request: Request) -> JSONResponse:
    return JSONResponse({"message": "Hello, Starlette!"})

async def create_user(request: Request) -> JSONResponse:
    data = await request.json()
    # 手动验证
    if "name" not in data or "email" not in data:
        return JSONResponse({"error": "Missing fields"}, status_code=400)
    user = await save_user(data)
    return JSONResponse(user, status_code=201)

async def ws_endpoint(websocket):
    await websocket.accept()
    async for message in websocket.iter_text():
        await websocket.send_text(f"Echo: {message}")

routes = [
    Route("/", homepage),
    Route("/users", create_user, methods=["POST"]),
    WebSocketRoute("/ws", ws_endpoint),
]

middleware = [
    Middleware(CORSMiddleware, allow_origins=["*"]),
]

app = Starlette(routes=routes, middleware=middleware)
```

### Litestar — 新一代高性能框架

```python
from litestar import Litestar, get, post, Controller
from litestar.dto import DTOConfig
from dataclasses import dataclass
from advanced_alchemy.extensions.litestar import SQLAlchemyPlugin

@dataclass
class UserCreate:
    name: str
    email: str

@dataclass
class UserResponse:
    id: int
    name: str
    email: str

class UserController(Controller):
    path = "/users"

    @post("/", status_code=201)
    async def create_user(self, data: UserCreate) -> UserResponse:
        user = await save_user(data)
        return UserResponse(**user)

    @get("/{user_id:int}")
    async def get_user(self, user_id: int) -> UserResponse:
        user = await find_user(user_id)
        if not user:
            raise NotFoundException("User not found")
        return UserResponse(**user)

app = Litestar(
    route_handlers=[UserController],
    plugins=[SQLAlchemyPlugin(config=db_config)],
)
```

## 场景选型指南

### 选FastAPI当
- 构建REST/GraphQL API服务
- 需要自动API文档和类型验证
- 团队熟悉类型提示和Pydantic
- 需要WebSocket和高并发支持
- 微服务架构中的独立服务

### 选Django当
- 需要完整的Web应用(不只是API)
- 需要Admin后台管理界面
- 需要内置用户认证和权限系统
- 团队有Django经验,项目需要快速启动
- 内容管理系统(CMS)类应用

### 选Flask当
- 小型项目或原型验证
- 需要最大灵活性,自由选择组件
- 学习Python Web开发
- 简单的内部工具或脚本HTTP封装

### 选Starlette当
- 需要最高性能的ASGI底层控制
- 构建自定义框架或中间件
- 极简主义偏好,不需要额外抽象

### 选Litestar当
- 需要FastAPI类体验但更严格的类型安全
- 内置DTO/缓存/速率限制等开箱即用
- 需要原生SQLAlchemy集成

## 最佳实践

### 1. 项目结构规范

```
# FastAPI推荐结构
app/
├── main.py              # 应用入口
├── config.py            # 配置管理
├── dependencies.py      # 共享依赖
├── models/              # SQLAlchemy模型
├── schemas/             # Pydantic模型
├── routers/             # 路由模块
│   ├── users.py
│   └── articles.py
├── services/            # 业务逻辑
├── repositories/        # 数据访问
└── middleware/           # 自定义中间件
```

### 2. 性能优化通用原则
- 使用连接池管理数据库连接
- 异步框架配合异步数据库驱动(asyncpg/aiomysql)
- 合理使用缓存(Redis)减少重复查询
- 启用gzip压缩大响应
- 使用CDN处理静态文件

### 3. 安全通用原则
- 所有输入必须验证(Pydantic/marshmallow/Django Forms)
- 使用参数化查询防SQL注入
- 启用CORS白名单而非通配符
- HTTPS强制在生产环境
- 敏感配置用环境变量

### 4. 测试策略
- FastAPI: 使用TestClient(同步)或httpx.AsyncClient
- Django: 使用django.test.TestCase和APIClient
- Flask: 使用app.test_client()
- 所有框架: 集成测试用docker-compose启动依赖

## 常见陷阱

### 陷阱1: Django N+1查询
```python
# 错误: 循环中触发额外查询
articles = Article.objects.all()
for a in articles:
    print(a.author.name)  # 每次循环查一次author

# 正确: 使用select_related/prefetch_related
articles = Article.objects.select_related("author").all()
```

### 陷阱2: FastAPI同步阻塞
```python
# 错误: 在async路由中使用同步IO
@app.get("/data")
async def get_data():
    result = requests.get("https://api.example.com")  # 阻塞!
    return result.json()

# 正确: 使用async客户端
@app.get("/data")
async def get_data():
    async with httpx.AsyncClient() as client:
        result = await client.get("https://api.example.com")
        return result.json()
```

### 陷阱3: Flask全局状态
```python
# 错误: 模块级别可变状态
counter = 0

@app.route("/count")
def count():
    global counter
    counter += 1  # 多worker下不安全!
    return str(counter)

# 正确: 使用Redis等外部状态存储
```

### 陷阱4: 忽略数据库迁移
```python
# Django有内置迁移系统
python manage.py makemigrations
python manage.py migrate

# FastAPI/Flask需要自行集成Alembic
alembic init migrations
alembic revision --autogenerate -m "add user table"
alembic upgrade head
```

### 陷阱5: 生产环境使用开发服务器
```bash
# 错误
uvicorn app:app  # 单worker
flask run        # 开发服务器
python manage.py runserver

# 正确
uvicorn app:app --workers 4 --host 0.0.0.0
gunicorn -w 4 -k uvicorn.workers.UvicornWorker app:app
gunicorn -w 4 myproject.wsgi:application
```

## Agent Checklist

### 框架选型
- [ ] 根据项目类型选择合适框架(API/全栈/微服务)
- [ ] 评估团队技术栈熟悉度
- [ ] 考虑性能需求(同步vs异步)
- [ ] 确认生态满足项目需求(ORM/Auth/Admin)

### 开发规范
- [ ] 项目结构遵循框架最佳实践
- [ ] 输入验证层完整(Pydantic/DRF Serializer)
- [ ] 错误处理统一(异常处理中间件)
- [ ] 日志记录规范(结构化日志)

### 生产就绪
- [ ] 使用生产级ASGI/WSGI服务器
- [ ] 数据库连接池配置合理
- [ ] 健康检查端点已实现
- [ ] CORS/安全头/HTTPS已配置
- [ ] 监控和告警已接入

### 测试覆盖
- [ ] API端点有集成测试
- [ ] 业务逻辑有单元测试
- [ ] 错误场景有覆盖
- [ ] 性能基准测试已建立
