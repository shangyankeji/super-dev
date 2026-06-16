---
id: fastapi-complete
title: FastAPI 完整指南
domain: backend
category: 01-standards
difficulty: intermediate
tags: [backend, complete, fastapi, 中间件, 依赖注入系统, 后台任务, 安全与认证, 快速开始]
quality_score: 70
last_updated: 2026-06-15
---
# FastAPI 完整指南

## 概述

FastAPI 是一个现代、高性能的 Python Web 框架,用于构建 API。基于 Starlette 和 Pydantic,利用 Python 3.8+ 的类型提示自动生成 OpenAPI 文档和 JSON Schema。FastAPI 的性能与 NodeJS 和 Go 相当,是最快的 Python 框架之一。

### 核心特性

- **高性能**: 与 NodeJS 和 Go 并驾齐驱(感谢 Starlette 和 Pydantic)
- **快速开发**: 开发速度提升 200%-300%(根据开发者反馈)
- **减少 Bug**: 减少约 40% 的人为错误(基于类型系统)
- **直观**: 完善的编辑器支持,自动补全随处可见
- **简单**: 易于使用和学习,文档完善
- **健壮**: 生产就绪,自动生成交互式文档
- **基于标准**: OpenAPI (Swagger), JSON Schema, OAuth2

### 为什么选择 FastAPI?

✅ **自动文档**: 自动生成 Swagger UI 和 ReDoc  
✅ **类型安全**: 基于 Python 类型提示,编辑器友好  
✅ **异步优先**: 原生支持 async/await  
✅ **依赖注入**: 强大且简洁的依赖注入系统  
✅ **数据验证**: 自动请求验证和序列化  
✅ **测试友好**: 基于 pytest,测试简单  

---

## 快速开始

### 安装

```bash
pip install fastapi uvicorn[standard]
```

### Hello World

```python
from fastapi import FastAPI

app = FastAPI()

@app.get("/")
async def root():
    return {"message": "Hello World"}

@app.get("/items/{item_id}")
async def read_item(item_id: int):
    return {"item_id": item_id}
```

### 运行服务器

```bash
# 开发模式
uvicorn main:app --reload

# 生产模式
uvicorn main:app --host 0.0.0.0 --port 8000 --workers 4
```

### 访问自动文档

- Swagger UI: `http://localhost:8000/docs`
- ReDoc: `http://localhost:8000/redoc`
- OpenAPI JSON: `http://localhost:8000/openapi.json`

---

## 核心概念

### 1. 路径操作 (Path Operations)

FastAPI 使用装饰器定义路由:

```python
from fastapi import FastAPI

app = FastAPI()

# GET 请求
@app.get("/")
async def read_root():
    return {"Hello": "World"}

# POST 请求
@app.post("/items/")
async def create_item(item: dict):
    return {"item": item}

# PUT 请求
@app.put("/items/{item_id}")
async def update_item(item_id: int, item: dict):
    return {"item_id": item_id, "item": item}

# DELETE 请求
@app.delete("/items/{item_id}")
async def delete_item(item_id: int):
    return {"deleted": item_id}

# 其他: @app.options(), @app.head(), @app.patch(), @app.trace()
```

### 2. 路径参数 (Path Parameters)

```python
from fastapi import FastAPI

app = FastAPI()

@app.get("/items/{item_id}")
async def read_item(item_id: int):
    """路径参数自动转换为 int 类型"""
    return {"item_id": item_id}

@app.get("/users/{user_id}/items/{item_id}")
async def read_user_item(user_id: int, item_id: str):
    """多个路径参数"""
    return {"user_id": user_id, "item_id": item_id}

# 枚举路径参数
from enum import Enum

class ModelName(str, Enum):
    alexnet = "alexnet"
    resnet = "resnet"
    lenet = "lenet"

@app.get("/models/{model_name}")
async def get_model(model_name: ModelName):
    if model_name == ModelName.alexnet:
        return {"model": model_name, "message": "Deep Learning FTW!"}
    return {"model": model_name}
```

### 3. 查询参数 (Query Parameters)

```python
from fastapi import FastAPI

app = FastAPI()

fake_items_db = [{"item_name": "Foo"}, {"item_name": "Bar"}, {"item_name": "Baz"}]

@app.get("/items/")
async def read_item(skip: int = 0, limit: int = 10):
    """查询参数带默认值"""
    return fake_items_db[skip : skip + limit]

# 必需查询参数
@app.get("/items/{item_id}")
async def read_item(item_id: str, needy: str):
    """needy 是必需的查询参数"""
    return {"item_id": item_id, "needy": needy}

# 可选查询参数
from typing import Optional

@app.get("/users/{user_id}/items/{item_id}")
async def read_user_item(
    user_id: int, 
    item_id: str, 
    q: Optional[str] = None, 
    short: bool = False
):
    item = {"item_id": item_id, "owner_id": user_id}
    if q:
        item.update({"q": q})
    if not short:
        item.update({"description": "This is an amazing item"})
    return item
```

### 4. 请求体 (Request Body)

使用 Pydantic 模型定义请求体:

```python
from fastapi import FastAPI
from pydantic import BaseModel
from typing import Optional

app = FastAPI()

class Item(BaseModel):
    name: str
    description: Optional[str] = None
    price: float
    tax: Optional[float] = None

@app.post("/items/")
async def create_item(item: Item):
    return item

@app.put("/items/{item_id}")
async def update_item(item_id: int, item: Item, q: Optional[str] = None):
    result = {"item_id": item_id, "item": item}
    if q:
        result.update({"q": q})
    return result
```

### 5. Pydantic 模型详解

```python
from pydantic import BaseModel, Field, validator
from typing import Optional, List
from datetime import datetime

class User(BaseModel):
    id: int
    name: str = Field(..., min_length=1, max_length=50, description="用户名")
    email: str = Field(..., regex=r"^[\w\.-]+@[\w\.-]+\.\w+$")
    age: Optional[int] = Field(None, ge=0, le=120, description="年龄")
    tags: List[str] = Field(default_factory=list)
    created_at: datetime = Field(default_factory=datetime.now)
    
    @validator('name')
    def name_must_not_contain_space(cls, v):
        if ' ' in v:
            raise ValueError('must not contain a space')
        return v.title()

class Item(BaseModel):
    name: str
    description: Optional[str] = None
    price: float = Field(..., gt=0, description="价格必须大于 0")
    tax: Optional[float] = None
    
    class Config:
        schema_extra = {
            "example": {
                "name": "Foo",
                "description": "A very nice Item",
                "price": 35.4,
                "tax": 3.2,
            }
        }

@app.post("/users/")
async def create_user(user: User):
    return user
```

---

## 依赖注入系统

FastAPI 的依赖注入系统是一个强大而简洁的特性。

### 基本用法

```python
from fastapi import FastAPI, Depends

app = FastAPI()

async def common_parameters(q: Optional[str] = None, skip: int = 0, limit: int = 100):
    return {"q": q, "skip": skip, "limit": limit}

@app.get("/items/")
async def read_items(commons: dict = Depends(common_parameters)):
    return commons

@app.get("/users/")
async def read_users(commons: dict = Depends(common_parameters)):
    return commons
```

### 类作为依赖

```python
from fastapi import FastAPI, Depends

app = FastAPI()

class CommonParams:
    def __init__(self, q: Optional[str] = None, skip: int = 0, limit: int = 100):
        self.q = q
        self.skip = skip
        self.limit = limit

@app.get("/items/")
async def read_items(commons: CommonParams = Depends()):
    return {"q": commons.q, "skip": commons.skip, "limit": commons.limit}
```

### 可调用依赖

```python
from fastapi import FastAPI, Depends, HTTPException

app = FastAPI()

class FixedContentQueryChecker:
    def __init__(self, fixed_content: str):
        self.fixed_content = fixed_content
    
    def __call__(self, q: str = ""):
        if q:
            return self.fixed_content in q
        return False

checker = FixedContentQueryChecker("bar")

@app.get("/items/")
async def read_items(is_fixed: bool = Depends(checker)):
    if is_fixed:
        return {"result": "Query contains 'bar'"}
    return {"result": "Query does not contain 'bar'"}
```

### 依赖树

```python
from fastapi import FastAPI, Depends

app = FastAPI()

def query_extractor(q: Optional[str] = None):
    return q

def query_or_cookie_extractor(
    q: str = Depends(query_extractor), 
    last_query: Optional[str] = None
):
    if not q:
        return last_query
    return q

@app.get("/items/")
async def read_query(query_extractor: str = Depends(query_or_cookie_extractor)):
    return {"query": query_extractor}
```

### 全局依赖

```python
from fastapi import FastAPI, Depends, Header, HTTPException

async def verify_token(x_token: str = Header(...)):
    if x_token != "fake-super-secret-token":
        raise HTTPException(status_code=400, detail="X-Token header invalid")

async def verify_key(x_key: str = Header(...)):
    if x_key != "fake-super-secret-key":
        raise HTTPException(status_code=400, detail="X-Key header invalid")
    return x_key

app = FastAPI(dependencies=[Depends(verify_token), Depends(verify_key)])

@app.get("/items/")
async def read_items():
    return [{"item": "Foo"}, {"item": "Bar"}]
```

---

## 安全与认证

### OAuth2 with JWT

```python
from fastapi import FastAPI, Depends, HTTPException, status
from fastapi.security import OAuth2PasswordBearer, OAuth2PasswordRequestForm
from pydantic import BaseModel
from typing import Optional
from datetime import datetime, timedelta
from jose import JWTError, jwt
from passlib.context import CryptContext

app = FastAPI()

# 配置
SECRET_KEY = "your-secret-key-here"
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 30

pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="token")

# 模型
class Token(BaseModel):
    access_token: str
    token_type: str

class TokenData(BaseModel):
    username: Optional[str] = None

class User(BaseModel):
    username: str
    email: Optional[str] = None
    full_name: Optional[str] = None
    disabled: Optional[bool] = None

class UserInDB(User):
    hashed_password: str

# 模拟数据库
fake_users_db = {
    "johndoe": {
        "username": "johndoe",
        "full_name": "John Doe",
        "email": "johndoe@example.com",
        "hashed_password": "$2b$12$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW",
        "disabled": False,
    }
}

# 密码工具
def verify_password(plain_password, hashed_password):
    return pwd_context.verify(plain_password, hashed_password)

def get_password_hash(password):
    return pwd_context.hash(password)

def get_user(db, username: str):
    if username in db:
        user_dict = db[username]
        return UserInDB(**user_dict)

def authenticate_user(fake_db, username: str, password: str):
    user = get_user(fake_db, username)
    if not user:
        return False
    if not verify_password(password, user.hashed_password):
        return False
    return user

def create_access_token(data: dict, expires_delta: Optional[timedelta] = None):
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.utcnow() + expires_delta
    else:
        expire = datetime.utcnow() + timedelta(minutes=15)
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt

async def get_current_user(token: str = Depends(oauth2_scheme)):
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Could not validate credentials",
        headers={"WWW-Authenticate": "Bearer"},
    )
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        username: str = payload.get("sub")
        if username is None:
            raise credentials_exception
        token_data = TokenData(username=username)
    except JWTError:
        raise credentials_exception
    user = get_user(fake_users_db, username=token_data.username)
    if user is None:
        raise credentials_exception
    return user

async def get_current_active_user(current_user: UserInDB = Depends(get_current_user)):
    if current_user.disabled:
        raise HTTPException(status_code=400, detail="Inactive user")
    return current_user

# 路由
@app.post("/token", response_model=Token)
async def login_for_access_token(form_data: OAuth2PasswordRequestForm = Depends()):
    user = authenticate_user(fake_users_db, form_data.username, form_data.password)
    if not user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Incorrect username or password",
            headers={"WWW-Authenticate": "Bearer"},
        )
    access_token_expires = timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    access_token = create_access_token(
        data={"sub": user.username}, expires_delta=access_token_expires
    )
    return {"access_token": access_token, "token_type": "bearer"}

@app.get("/users/me/", response_model=User)
async def read_users_me(current_user: User = Depends(get_current_active_user)):
    return current_user

@app.get("/users/me/items/")
async def read_own_items(current_user: User = Depends(get_current_active_user)):
    return [{"item_id": "Foo", "owner": current_user.username}]
```

### API Key 认证

```python
from fastapi import FastAPI, Depends, HTTPException, status
from fastapi.security import APIKeyHeader

app = FastAPI()

API_KEY = "your-api-key-here"
api_key_header = APIKeyHeader(name="X-API-Key")

async def get_api_key(api_key: str = Depends(api_key_header)):
    if api_key != API_KEY:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid API Key"
        )
    return api_key

@app.get("/protected/")
async def protected_route(api_key: str = Depends(get_api_key)):
    return {"message": "Access granted"}
```

---

## 数据库集成

### SQLAlchemy 异步

```python
from fastapi import FastAPI, Depends, HTTPException
from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession
from sqlalchemy.orm import sessionmaker, declarative_base
from sqlalchemy import Column, Integer, String, select
from typing import List, Optional
from pydantic import BaseModel

# 数据库配置
SQLALCHEMY_DATABASE_URL = "sqlite+aiosqlite:///./test.db"
# SQLALCHEMY_DATABASE_URL = "postgresql+asyncpg://user:password@localhost/dbname"

engine = create_async_engine(SQLALCHEMY_DATABASE_URL, echo=True)
AsyncSessionLocal = sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)
Base = declarative_base()

# 模型
class User(Base):
    __tablename__ = "users"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    email = Column(String, unique=True, index=True)

# Pydantic 模型
class UserCreate(BaseModel):
    name: str
    email: str

class UserResponse(BaseModel):
    id: int
    name: str
    email: str
    
    class Config:
        orm_mode = True

# 依赖
async def get_db():
    async with AsyncSessionLocal() as session:
        try:
            yield session
        finally:
            await session.close()

# 应用
app = FastAPI()

# 创建表
@app.on_event("startup")
async def startup():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)

# 路由
@app.post("/users/", response_model=UserResponse)
async def create_user(user: UserCreate, db: AsyncSession = Depends(get_db)):
    db_user = User(**user.dict())
    db.add(db_user)
    await db.commit()
    await db.refresh(db_user)
    return db_user

@app.get("/users/", response_model=List[UserResponse])
async def read_users(skip: int = 0, limit: int = 100, db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(User).offset(skip).limit(limit))
    users = result.scalars().all()
    return users

@app.get("/users/{user_id}", response_model=UserResponse)
async def read_user(user_id: int, db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(User).where(User.id == user_id))
    user = result.scalar_one_or_none()
    if user is None:
        raise HTTPException(status_code=404, detail="User not found")
    return user
```

---

## 中间件

### 自定义中间件

```python
from fastapi import FastAPI, Request, Response
import time

app = FastAPI()

@app.middleware("http")
async def add_process_time_header(request: Request, call_next):
    start_time = time.time()
    response = await call_next(request)
    process_time = time.time() - start_time
    response.headers["X-Process-Time"] = str(process_time)
    return response

# CORS 中间件
from fastapi.middleware.cors import CORSMiddleware

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# GZip 中间件
from fastapi.middleware.gzip import GZipMiddleware
app.add_middleware(GZipMiddleware, minimum_size=1000)
```

---

## 后台任务

```python
from fastapi import FastAPI, BackgroundTasks
from typing import Optional

app = FastAPI()

def write_log(message: str):
    with open("log.txt", mode="a") as log:
        log.write(message + "\n")

@app.post("/send-notification/{email}")
async def send_notification(email: str, background_tasks: BackgroundTasks, q: Optional[str] = None):
    message = f"notification to {email}\n"
    if q:
        message += f"query param: {q}\n"
    
    background_tasks.add_task(write_log, message)
    
    return {"message": "Notification sent in the background"}
```

---

## WebSocket

```python
from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from typing import List

app = FastAPI()

class ConnectionManager:
    def __init__(self):
        self.active_connections: List[WebSocket] = []

    async def connect(self, websocket: WebSocket):
        await websocket.accept()
        self.active_connections.append(websocket)

    def disconnect(self, websocket: WebSocket):
        self.active_connections.remove(websocket)

    async def send_personal_message(self, message: str, websocket: WebSocket):
        await websocket.send_text(message)

    async def broadcast(self, message: str):
        for connection in self.active_connections:
            await connection.send_text(message)

manager = ConnectionManager()

@app.get("/")
async def get():
    return {"message": "WebSocket server is running"}

@app.websocket("/ws/{client_id}")
async def websocket_endpoint(websocket: WebSocket, client_id: int):
    await manager.connect(websocket)
    try:
        while True:
            data = await websocket.receive_text()
            await manager.send_personal_message(f"You wrote: {data}", websocket)
            await manager.broadcast(f"Client #{client_id} says: {data}")
    except WebSocketDisconnect:
        manager.disconnect(websocket)
        await manager.broadcast(f"Client #{client_id} left the chat")
```

---

## 测试

```python
from fastapi.testclient import TestClient
from main import app

client = TestClient(app)

def test_read_main():
    response = client.get("/")
    assert response.status_code == 200
    assert response.json() == {"message": "Hello World"}

def test_create_item():
    response = client.post(
        "/items/",
        json={"name": "Item 1", "price": 10.5}
    )
    assert response.status_code == 200
    assert response.json()["name"] == "Item 1"

def test_read_item():
    response = client.get("/items/1")
    assert response.status_code == 200
    assert "item_id" in response.json()

# 异步测试
import pytest
from httpx import AsyncClient

@pytest.mark.asyncio
async def test_async_read_main():
    async with AsyncClient(app=app, base_url="http://test") as ac:
        response = await ac.get("/")
    assert response.status_code == 200
```

---

## 部署

### Docker 部署

```dockerfile
FROM python:3.11-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
```

### Gunicorn + Uvicorn

```bash
pip install gunicorn uvicorn[standard]

# 启动命令
gunicorn main:app --workers 4 --worker-class uvicorn.workers.UvicornWorker --bind 0.0.0.0:8000
```

### Kubernetes 部署

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fastapi-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: fastapi
  template:
    metadata:
      labels:
        app: fastapi
    spec:
      containers:
      - name: fastapi
        image: your-image:latest
        ports:
        - containerPort: 8000
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: fastapi-service
spec:
  selector:
    app: fastapi
  ports:
  - port: 80
    targetPort: 8000
  type: LoadBalancer
```

---

## 性能优化

### 1. 使用异步数据库驱动

```python
# ❌ 不好: 同步驱动
import psycopg2

# ✅ 好: 异步驱动
import asyncpg
from sqlalchemy.ext.asyncio import create_async_engine
```

### 2. 连接池配置

```python
from sqlalchemy.ext.asyncio import create_async_engine

engine = create_async_engine(
    "postgresql+asyncpg://user:pass@localhost/db",
    pool_size=20,
    max_overflow=40,
    pool_pre_ping=True,
    pool_recycle=3600
)
```

### 3. 启用 GZip

```python
from fastapi.middleware.gzip import GZipMiddleware

app.add_middleware(GZipMiddleware, minimum_size=1000)
```

### 4. 使用 Redis 缓存

```python
from fastapi import FastAPI
from fastapi_cache import FastAPICache
from fastapi_cache.backends.redis import RedisBackend
from fastapi_cache.decorator import cache
from redis import asyncio as aioredis

app = FastAPI()

@app.on_event("startup")
async def startup():
    redis = aioredis.from_url("redis://localhost", encoding="utf8", decode_responses=True)
    FastAPICache.init(RedisBackend(redis), prefix="fastapi-cache")

@app.get("/expensive-query")
@cache(expire=60)
async def expensive_query():
    # 执行昂贵查询
    return {"data": "..."}
```

### 5. 批量操作

```python
# ❌ 不好: N+1 查询
for item in items:
    db_item = await db.get(Item, item.id)
    # ...

# ✅ 好: 批量查询
item_ids = [item.id for item in items]
result = await db.execute(select(Item).where(Item.id.in_(item_ids)))
items = result.scalars().all()
```

---

## 最佳实践

1. **✅ 使用类型提示**: FastAPI 的核心优势
2. **✅ 分离关注点**: 路由、模型、业务逻辑分离
3. **✅ 使用依赖注入**: 提高可测试性和可维护性
4. **✅ 使用 Pydantic 验证**: 充分利用数据验证
5. **✅ 编写测试**: 使用 pytest 和 TestClient
6. **✅ 使用环境变量**: 配置管理
7. **✅ 日志记录**: 使用 Python logging
8. **✅ 错误处理**: 自定义异常处理器
9. **✅ API 版本控制**: /v1, /v2 前缀
10. **✅ 文档完善**: 利用自动文档并添加示例

---

## 参考资料

- [FastAPI 官方文档](https://fastapi.tiangolo.com/)
- [FastAPI GitHub](https://github.com/tiangolo/fastapi)
- [Full Stack FastAPI PostgreSQL Template](https://github.com/tiangolo/full-stack-fastapi-postgresql)

---

**文档版本**: v1.0  
**最后更新**: 2026-03-28  
**质量评分**: 90/100
