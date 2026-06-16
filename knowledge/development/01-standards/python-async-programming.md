---
id: python-async-programming
title: Python 异步编程完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [async, development, programming, python, 实战案例, 常见陷阱与反模式, 异步编程模式, 性能优化技巧]
quality_score: 70
last_updated: 2026-06-15
---
# Python 异步编程完整指南

## 概述

Python 异步编程是一种并发编程范式,通过 `asyncio` 库和 `async/await` 语法实现高效的 I/O 密集型任务处理。与多线程/多进程相比,异步编程使用单线程事件循环,避免了线程切换开销和 GIL(全局解释器锁)限制,适合处理大量并发网络请求、文件 I/O、数据库操作等场景。

### 核心优势

- **高并发性能**: 单线程可处理数千个并发连接
- **低资源消耗**: 相比多线程,内存和 CPU 开销更小
- **简洁的代码**: `async/await` 语法让异步代码像同步代码一样易读
- **适合 I/O 密集型**: 网络请求、文件操作、数据库查询的理想选择

### 适用场景

✅ **适合异步编程**:
- Web 服务器和 API 服务(FastAPI, Sanic, Tornado)
- 网络爬虫和 HTTP 客户端
- WebSocket 实时通信
- 数据库异步查询(SQLAlchemy async, Tortoise ORM)
- 消息队列消费者
- 微服务间通信

❌ **不适合异步编程**:
- CPU 密集型计算(图像处理、机器学习训练)
- 需要真正的并行计算(应使用多进程)
- 与阻塞式库交互(如 requests, psycopg2)

---

## 核心概念

### 1. 事件循环 (Event Loop)

事件循环是异步编程的核心,负责调度和执行协程。它维护一个任务队列,不断从队列中取出任务执行,当任务遇到 I/O 操作时挂起,让出控制权,等待 I/O 完成后恢复执行。

```python
import asyncio

# 获取当前事件循环
loop = asyncio.get_event_loop()

# Python 3.7+ 推荐方式
async def main():
    print("Hello, Async!")

asyncio.run(main())  # 自动创建和关闭事件循环
```

### 2. 协程 (Coroutine)

协程是使用 `async def` 定义的函数,调用时不会立即执行,而是返回一个协程对象。协程需要被事件循环调度才能执行。

```python
async def fetch_data(url):
    """定义协程"""
    print(f"Fetching {url}")
    await asyncio.sleep(1)  # 模拟 I/O 操作
    return f"Data from {url}"

# 调用协程(不执行)
coro = fetch_data("https://example.com")
print(coro)  # <coroutine object fetch_data at 0x...>

# 执行协程
result = asyncio.run(fetch_data("https://example.com"))
print(result)  # Data from https://example.com
```

### 3. await 关键字

`await` 用于挂起当前协程,等待另一个协程完成。它只能在 `async def` 函数内部使用。

```python
async def step1():
    await asyncio.sleep(1)
    return "Step 1 done"

async def step2():
    await asyncio.sleep(1)
    return "Step 2 done"

async def main():
    result1 = await step1()  # 等待 step1 完成
    result2 = await step2()  # 等待 step2 完成
    print(result1, result2)

asyncio.run(main())
```

### 4. Task (任务)

Task 是协程的包装器,用于在事件循环中调度协程。使用 `asyncio.create_task()` 可以并发执行多个协程。

```python
async def download_file(url):
    await asyncio.sleep(2)
    return f"Downloaded {url}"

async def main():
    # 创建 3 个并发任务
    task1 = asyncio.create_task(download_file("file1.txt"))
    task2 = asyncio.create_task(download_file("file2.txt"))
    task3 = asyncio.create_task(download_file("file3.txt"))
    
    # 等待所有任务完成
    results = await asyncio.gather(task1, task2, task3)
    print(results)

asyncio.run(main())
```

### 5. Future (未来对象)

Future 是一个低级别的可等待对象,代表一个异步操作的最终结果。Task 是 Future 的子类。

```python
async def set_future_value(future, value):
    await asyncio.sleep(1)
    future.set_result(value)

async def main():
    future = asyncio.Future()
    
    # 在另一个任务中设置 Future 的值
    asyncio.create_task(set_future_value(future, "Hello"))
    
    # 等待 Future
    result = await future
    print(result)  # Hello

asyncio.run(main())
```

---

## 异步编程模式

### 模式 1: 并发执行多个任务

使用 `asyncio.gather()` 并发执行多个协程:

```python
import asyncio
import aiohttp

async def fetch_url(session, url):
    async with session.get(url) as response:
        return await response.text()

async def fetch_all(urls):
    async with aiohttp.ClientSession() as session:
        tasks = [fetch_url(session, url) for url in urls]
        results = await asyncio.gather(*tasks)
        return results

urls = [
    "https://example.com",
    "https://httpbin.org",
    "https://jsonplaceholder.typicode.com"
]

results = asyncio.run(fetch_all(urls))
print(f"Fetched {len(results)} pages")
```

### 模式 2: 超时控制

使用 `asyncio.wait_for()` 设置超时:

```python
async def slow_operation():
    await asyncio.sleep(10)
    return "Done"

async def main():
    try:
        result = await asyncio.wait_for(slow_operation(), timeout=3.0)
        print(result)
    except asyncio.TimeoutError:
        print("Operation timed out!")

asyncio.run(main())
```

### 模式 3: 取消任务

```python
async def long_running_task():
    try:
        print("Task started")
        await asyncio.sleep(10)
        print("Task completed")
    except asyncio.CancelledError:
        print("Task was cancelled")
        raise

async def main():
    task = asyncio.create_task(long_running_task())
    
    await asyncio.sleep(2)
    task.cancel()  # 取消任务
    
    try:
        await task
    except asyncio.CancelledError:
        print("Main: Task cancelled")

asyncio.run(main())
```

### 模式 4: 异步上下文管理器

```python
from contextlib import asynccontextmanager

@asynccontextmanager
async def async_database_connection():
    print("Connecting to database...")
    await asyncio.sleep(1)
    conn = {"connected": True}
    try:
        yield conn
    finally:
        print("Closing connection...")
        await asyncio.sleep(0.5)

async def main():
    async with async_database_connection() as conn:
        print(f"Using connection: {conn}")

asyncio.run(main())
```

### 模式 5: 异步迭代器

```python
class AsyncRange:
    def __init__(self, count):
        self.count = count
        self.current = 0
    
    def __aiter__(self):
        return self
    
    async def __anext__(self):
        if self.current < self.count:
            await asyncio.sleep(0.1)
            value = self.current
            self.current += 1
            return value
        else:
            raise StopAsyncIteration

async def main():
    async for number in AsyncRange(5):
        print(number)

asyncio.run(main())
```

### 模式 6: 异步生成器

```python
async def async_fibonacci(n):
    a, b = 0, 1
    for _ in range(n):
        await asyncio.sleep(0.1)
        yield a
        a, b = b, a + b

async def main():
    async for fib in async_fibonacci(10):
        print(fib)

asyncio.run(main())
```

---

## 实战案例

### 案例 1: 异步 HTTP 爬虫

```python
import asyncio
import aiohttp
from typing import List, Dict
import time

class AsyncWebCrawler:
    def __init__(self, max_concurrent: int = 10):
        self.max_concurrent = max_concurrent
        self.semaphore = asyncio.Semaphore(max_concurrent)
    
    async def fetch_page(self, session: aiohttp.ClientSession, url: str) -> Dict:
        async with self.semaphore:
            try:
                async with session.get(url, timeout=aiohttp.ClientTimeout(total=10)) as response:
                    html = await response.text()
                    return {
                        "url": url,
                        "status": response.status,
                        "length": len(html),
                        "success": True
                    }
            except Exception as e:
                return {
                    "url": url,
                    "error": str(e),
                    "success": False
                }
    
    async def crawl(self, urls: List[str]) -> List[Dict]:
        async with aiohttp.ClientSession() as session:
            tasks = [self.fetch_page(session, url) for url in urls]
            results = await asyncio.gather(*tasks, return_exceptions=True)
            return results

async def main():
    urls = [f"https://httpbin.org/delay/{i}" for i in range(1, 20)]
    
    crawler = AsyncWebCrawler(max_concurrent=5)
    
    start = time.time()
    results = await crawler.crawl(urls)
    end = time.time()
    
    success_count = sum(1 for r in results if isinstance(r, dict) and r.get("success"))
    print(f"Crawled {success_count}/{len(urls)} pages in {end-start:.2f}s")

asyncio.run(main())
```

### 案例 2: 异步数据库操作 (使用 SQLAlchemy 2.0)

```python
from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession
from sqlalchemy.orm import sessionmaker, declarative_base
from sqlalchemy import Column, Integer, String, select
import asyncio

Base = declarative_base()

class User(Base):
    __tablename__ = "users"
    id = Column(Integer, primary_key=True)
    name = Column(String)
    email = Column(String)

# SQLite 异步引擎
engine = create_async_engine("sqlite+aiosqlite:///test.db", echo=True)
AsyncSessionLocal = sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)

async def create_tables():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)

async def insert_users():
    async with AsyncSessionLocal() as session:
        users = [
            User(name="Alice", email="alice@example.com"),
            User(name="Bob", email="bob@example.com"),
            User(name="Charlie", email="charlie@example.com"),
        ]
        session.add_all(users)
        await session.commit()

async def query_users():
    async with AsyncSessionLocal() as session:
        result = await session.execute(select(User).where(User.name.like("A%")))
        users = result.scalars().all()
        for user in users:
            print(f"{user.name}: {user.email}")

async def main():
    await create_tables()
    await insert_users()
    await query_users()

asyncio.run(main())
```

### 案例 3: 异步消息队列消费者

```python
import asyncio
import json
from collections import deque

class AsyncMessageQueue:
    def __init__(self):
        self.queue = deque()
        self.condition = asyncio.Condition()
    
    async def put(self, message):
        async with self.condition:
            self.queue.append(message)
            self.condition.notify()
    
    async def get(self):
        async with self.condition:
            while not self.queue:
                await self.condition.wait()
            return self.queue.popleft()

async def producer(queue, producer_id):
    for i in range(10):
        message = {"producer": producer_id, "data": i}
        await queue.put(message)
        print(f"Producer {producer_id} sent: {message}")
        await asyncio.sleep(0.5)

async def consumer(queue, consumer_id):
    while True:
        message = await queue.get()
        print(f"Consumer {consumer_id} received: {message}")
        await asyncio.sleep(1)

async def main():
    queue = AsyncMessageQueue()
    
    # 启动 2 个生产者和 3 个消费者
    producers = [producer(queue, i) for i in range(2)]
    consumers = [consumer(queue, i) for i in range(3)]
    
    await asyncio.gather(*producers)
    await asyncio.gather(*consumers)

asyncio.run(main())
```

### 案例 4: 异步 API 服务器 (FastAPI)

```python
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import asyncio
import aiohttp

app = FastAPI()

class Item(BaseModel):
    name: str
    price: float

# 模拟数据库
fake_db = {}

@app.post("/items/")
async def create_item(item: Item):
    await asyncio.sleep(0.1)  # 模拟数据库延迟
    item_id = len(fake_db) + 1
    fake_db[item_id] = item
    return {"id": item_id, "item": item}

@app.get("/items/{item_id}")
async def read_item(item_id: int):
    await asyncio.sleep(0.05)
    if item_id not in fake_db:
        raise HTTPException(status_code=404, detail="Item not found")
    return fake_db[item_id]

@app.get("/external-api")
async def call_external_api():
    async with aiohttp.ClientSession() as session:
        async with session.get("https://httpbin.org/json") as response:
            data = await response.json()
            return data

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

---

## 性能优化技巧

### 1. 使用连接池

```python
import aiohttp

# 错误: 每次请求创建新连接
async def bad_approach():
    for url in urls:
        async with aiohttp.ClientSession() as session:
            async with session.get(url) as response:
                await response.text()

# 正确: 复用连接池
async def good_approach():
    async with aiohttp.ClientSession() as session:
        tasks = [session.get(url) for url in urls]
        responses = await asyncio.gather(*tasks)
```

### 2. 限制并发数量

```python
async def fetch_with_limit(urls, max_concurrent=10):
    semaphore = asyncio.Semaphore(max_concurrent)
    
    async def fetch(url):
        async with semaphore:
            async with aiohttp.ClientSession() as session:
                async with session.get(url) as response:
                    return await response.text()
    
    return await asyncio.gather(*[fetch(url) for url in urls])
```

### 3. 批处理操作

```python
async def batch_insert(items, batch_size=100):
    for i in range(0, len(items), batch_size):
        batch = items[i:i+batch_size]
        # 批量插入数据库
        await database_insert_batch(batch)
```

### 4. 避免阻塞操作

```python
import asyncio
import time

# 错误: 使用阻塞式 sleep
async def bad():
    time.sleep(1)  # 阻塞整个事件循环!

# 正确: 使用异步 sleep
async def good():
    await asyncio.sleep(1)  # 让出控制权
```

### 5. 使用 `asyncio.to_thread()` 处理 CPU 密集型任务

```python
import asyncio

def cpu_intensive_task(n):
    """CPU 密集型函数"""
    return sum(i * i for i in range(n))

async def main():
    # 在单独的线程中运行 CPU 密集型任务
    result = await asyncio.to_thread(cpu_intensive_task, 10**6)
    print(result)

asyncio.run(main())
```

---

## 常见陷阱与反模式

### 反模式 1: 忘记 await

```python
# ❌ 错误: 忘记 await
async def bad():
    result = async_function()  # 返回协程对象,不执行
    print(result)  # <coroutine object>

# ✅ 正确
async def good():
    result = await async_function()
    print(result)
```

### 反模式 2: 在同步代码中调用异步函数

```python
# ❌ 错误: 在同步函数中调用异步函数
def bad():
    result = await async_function()  # SyntaxError!

# ✅ 正确: 使用 asyncio.run()
def good():
    result = asyncio.run(async_function())
```

### 反模式 3: 过度使用 asyncio.sleep(0)

```python
# ❌ 不好: 过度让步
async def bad():
    for i in range(10000):
        await asyncio.sleep(0)  # 不必要的让步
        process(i)

# ✅ 更好: 只在必要时让步
async def good():
    for i in range(10000):
        process(i)
        if i % 100 == 0:  # 每 100 次让步一次
            await asyncio.sleep(0)
```

### 反模式 4: 不处理异常

```python
# ❌ 错误: 不处理异常会导致任务静默失败
async def bad():
    tasks = [task1(), task2(), task3()]
    await asyncio.gather(*tasks)  # 如果一个失败,其他继续

# ✅ 正确: 捕获和处理异常
async def good():
    tasks = [task1(), task2(), task3()]
    results = await asyncio.gather(*tasks, return_exceptions=True)
    for result in results:
        if isinstance(result, Exception):
            print(f"Task failed: {result}")
```

### 反模式 5: 不关闭资源

```python
# ❌ 错误: 不关闭会话
async def bad():
    session = aiohttp.ClientSession()
    await session.get("https://example.com")
    # 忘记关闭 session!

# ✅ 正确: 使用上下文管理器
async def good():
    async with aiohttp.ClientSession() as session:
        await session.get("https://example.com")
    # 自动关闭
```

---

## 调试技巧

### 1. 启用异步调试模式

```python
import asyncio

# 启用调试模式
asyncio.run(main(), debug=True)
```

### 2. 检查协程是否被 await

```python
import warnings

# 启用协程未 await 警告
warnings.filterwarnings("error", category=RuntimeWarning)
```

### 3. 使用 `asyncio.all_tasks()` 查看运行中的任务

```python
async def monitor_tasks():
    while True:
        tasks = asyncio.all_tasks()
        print(f"Running tasks: {len(tasks)}")
        for task in tasks:
            print(f"  - {task.get_name()}: {task.get_coro()}")
        await asyncio.sleep(5)
```

### 4. 使用 `asyncio.current_task()` 获取当前任务

```python
async def my_task():
    current = asyncio.current_task()
    print(f"Current task: {current.get_name()}")
```

---

## 测试异步代码

### 使用 pytest + pytest-asyncio

```python
# 安装: pip install pytest pytest-asyncio

import pytest
import asyncio

@pytest.mark.asyncio
async def test_async_function():
    result = await async_function()
    assert result == expected_value

@pytest.mark.asyncio
async def test_concurrent_tasks():
    task1 = asyncio.create_task(slow_operation())
    task2 = asyncio.create_task(another_operation())
    
    results = await asyncio.gather(task1, task2)
    assert len(results) == 2

# 测试异常
@pytest.mark.asyncio
async def test_exception_handling():
    with pytest.raises(ValueError):
        await function_that_raises()
```

### Mock 异步函数

```python
from unittest.mock import AsyncMock, patch

@pytest.mark.asyncio
async def test_with_mock():
    with patch("module.async_function") as mock_func:
        mock_func.return_value = "mocked result"
        
        result = await module.async_function()
        assert result == "mocked result"
        mock_func.assert_called_once()
```

---

## 迁移指南: 从同步到异步

### Step 1: 识别阻塞点

```python
# 同步代码
import requests
import time

def fetch_all(urls):
    results = []
    for url in urls:
        response = requests.get(url)  # 阻塞点!
        results.append(response.text)
    return results
```

### Step 2: 替换为异步库

```python
# 异步代码
import aiohttp
import asyncio

async def fetch_all(urls):
    async with aiohttp.ClientSession() as session:
        tasks = []
        for url in urls:
            task = session.get(url)  # 非阻塞
            tasks.append(task)
        
        responses = await asyncio.gather(*tasks)
        results = [await r.text() for r in responses]
    return results
```

### Step 3: 使用异步数据库驱动

| 同步库 | 异步替代 |
|--------|---------|
| `psycopg2` | `asyncpg` 或 `psycopg` (3.0+) |
| `pymysql` | `aiomysql` |
| `sqlite3` | `aiosqlite` |
| `redis-py` | `aioredis` (已集成到 redis-py 4.2+) |
| `pymongo` | `motor` |

### Step 4: 更新函数签名

```python
# 同步
def process_data(data):
    result = database.query(data)
    return result

# 异步
async def process_data(data):
    result = await database.query(data)
    return result
```

---

## 性能对比

### 场景: 并发请求 100 个 URL

```python
import asyncio
import aiohttp
import requests
import time

urls = ["https://httpbin.org/delay/1" for _ in range(100)]

# 同步版本
def sync_fetch():
    start = time.time()
    for url in urls:
        requests.get(url)
    end = time.time()
    print(f"Sync: {end-start:.2f}s")

# 异步版本
async def async_fetch():
    start = time.time()
    async with aiohttp.ClientSession() as session:
        tasks = [session.get(url) for url in urls]
        await asyncio.gather(*tasks)
    end = time.time()
    print(f"Async: {end-start:.2f}s")

# 结果:
# Sync: 105.32s  (串行执行)
# Async: 2.15s   (并发执行,提速 50 倍!)
```

---

## 最佳实践总结

1. **✅ 使用 Python 3.7+ 的 `asyncio.run()`**: 自动管理事件循环生命周期
2. **✅ 优先使用高层 API**: `asyncio.gather()`, `asyncio.create_task()`
3. **✅ 限制并发数量**: 使用 `asyncio.Semaphore` 防止资源耗尽
4. **✅ 处理所有异常**: 使用 `try/except` 或 `return_exceptions=True`
5. **✅ 使用异步上下文管理器**: 确保资源正确关闭
6. **✅ 测试异步代码**: 使用 `pytest-asyncio`
7. **✅ 避免 CPU 密集型任务**: 使用 `asyncio.to_thread()` 或多进程
8. **✅ 使用类型提示**: 提高代码可维护性

```python
from typing import List, Coroutine, Any

async def fetch_urls(urls: List[str]) -> List[str]:
    ...
```

---

## 参考资料

### 官方文档
- [Python asyncio 文档](https://docs.python.org/3/library/asyncio.html)
- [PEP 492 -- Coroutines with async and await syntax](https://www.python.org/dev/peps/pep-0492/)

### 推荐库
- **HTTP 客户端**: `aiohttp`, `httpx`
- **数据库**: `asyncpg`, `SQLAlchemy 2.0`, `Tortoise ORM`, `Motor`
- **Web 框架**: `FastAPI`, `Starlette`, `Sanic`, `Tornado`
- **任务队列**: `arq`, `dramatiq` (异步支持)
- **测试**: `pytest-asyncio`, `aresponses`

### 进阶阅读
- [Async IO in Python: A Complete Walkthrough](https://realpython.com/async-io-python/)
- [FastAPI 官方文档 - Async](https://fastapi.tiangolo.com/async/)
- [SQLAlchemy 2.0 Async](https://docs.sqlalchemy.org/en/20/orm/extensions/asyncio.html)

---

## 学习路径

### 初级 (1-2 周)
1. 理解事件循环和协程的基本概念
2. 掌握 `async/await` 语法
3. 使用 `asyncio.gather()` 并发执行任务
4. 编写简单的异步 HTTP 客户端

### 中级 (3-4 周)
1. 理解 Task, Future 的区别
2. 使用 `asyncio.Semaphore` 控制并发
3. 掌握异步上下文管理器和异步迭代器
4. 使用异步数据库驱动
5. 编写 FastAPI 异步 API

### 高级 (1-2 月)
1. 实现自定义事件循环
2. 编写异步中间件和装饰器
3. 优化异步性能(连接池、批处理)
4. 处理复杂的异步错误场景
5. 贡献异步开源项目

---

**文档版本**: v1.0  
**最后更新**: 2026-03-28  
**维护者**: Super Dev 团队  
**质量评分**: 92/100
