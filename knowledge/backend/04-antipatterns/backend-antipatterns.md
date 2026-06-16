---
id: backend-antipatterns
title: 后端反模式手册
domain: backend
category: 04-antipatterns
difficulty: intermediate
tags: [antipatterns, backend, controller, 同步阻塞, 无超时设置, 无连接池, 无重试策略, 无限分页]
quality_score: 70
last_updated: 2026-06-15
---
# 后端反模式手册

> 覆盖 Python 和 Node.js 后端开发中最常见的 10 类反模式。
> 每项包含：问题描述、问题代码（Python + Node.js）、修复代码、检测方法。

---

## 1. N+1 查询

**问题**：获取列表后，逐条查询关联数据，导致数据库请求数与数据量成线性关系。100 条数据产生 101 次查询。

### 问题代码 - Python

```python
# Django ORM - N+1
def get_orders(request):
    orders = Order.objects.all()[:100]  # 1 次查询
    result = []
    for order in orders:
        # 每次循环产生 1 次查询 → 共 100 次
        customer = order.customer  # SELECT * FROM customer WHERE id = ?
        items = order.items.all()  # SELECT * FROM order_item WHERE order_id = ?
        result.append({
            "id": order.id,
            "customer_name": customer.name,
            "item_count": len(items),
        })
    return JsonResponse({"data": result})  # 总计 201 次查询
```

### 问题代码 - Node.js

```ts
// Prisma - N+1
app.get('/orders', async (req, res) => {
  const orders = await prisma.order.findMany({ take: 100 });

  const result = [];
  for (const order of orders) {
    // 每次循环 2 次查询
    const customer = await prisma.customer.findUnique({ where: { id: order.customerId } });
    const items = await prisma.orderItem.findMany({ where: { orderId: order.id } });
    result.push({
      id: order.id,
      customerName: customer?.name,
      itemCount: items.length,
    });
  }

  res.json({ data: result }); // 201 次查询
});
```

### 修复代码 - Python

```python
# Django - select_related + prefetch_related
def get_orders(request):
    orders = (
        Order.objects
        .select_related("customer")      # JOIN 一次性加载
        .prefetch_related("items")        # 第二次查询批量加载
        .all()[:100]
    )
    # 总计 2 次查询（无论数据量多少）
    result = [
        {
            "id": order.id,
            "customer_name": order.customer.name,
            "item_count": order.items.count(),
        }
        for order in orders
    ]
    return JsonResponse({"data": result})

# SQLAlchemy - joinedload
from sqlalchemy.orm import joinedload

orders = (
    session.query(Order)
    .options(joinedload(Order.customer), joinedload(Order.items))
    .limit(100)
    .all()
)
```

### 修复代码 - Node.js

```ts
// Prisma - include 一次性加载
app.get('/orders', async (req, res) => {
  const orders = await prisma.order.findMany({
    take: 100,
    include: {
      customer: { select: { name: true } },
      items: true,
    },
  });
  // 总计 1 次查询（Prisma 自动 JOIN 或批量查询）
  res.json({
    data: orders.map(o => ({
      id: o.id,
      customerName: o.customer.name,
      itemCount: o.items.length,
    })),
  });
});
```

**检测方法**：Django Debug Toolbar 的 SQL 面板；Prisma 日志 `prisma.$on('query')`；查询数 > 列表条数即为 N+1。

---

## 2. 无连接池

**问题**：每次请求新建数据库连接，高并发时连接数爆炸，耗尽数据库资源或产生大量 TIME_WAIT。

### 问题代码 - Python

```python
import psycopg2

def get_user(user_id: str):
    # 每次调用创建新连接 → 高并发时连接数爆炸
    conn = psycopg2.connect(
        host="localhost", port=5432,
        dbname="mydb", user="admin", password="secret"
    )
    try:
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM users WHERE id = %s", (user_id,))
        return cursor.fetchone()
    finally:
        conn.close()  # 连接直接关闭，无法复用
```

### 问题代码 - Node.js

```ts
import { Client } from 'pg';

async function getUser(userId: string) {
  // 每次请求新建 Client → 无连接复用
  const client = new Client({
    host: 'localhost', port: 5432,
    database: 'mydb', user: 'admin', password: 'secret',
  });
  await client.connect();
  try {
    const result = await client.query('SELECT * FROM users WHERE id = $1', [userId]);
    return result.rows[0];
  } finally {
    await client.end();
  }
}
```

### 修复代码 - Python

```python
# psycopg2 连接池
from psycopg2 import pool

# 应用启动时创建连接池（全局单例）
db_pool = pool.ThreadedConnectionPool(
    minconn=5,
    maxconn=20,
    host="localhost", port=5432,
    dbname="mydb", user="admin", password="secret",
)

def get_user(user_id: str):
    conn = db_pool.getconn()
    try:
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM users WHERE id = %s", (user_id,))
        return cursor.fetchone()
    finally:
        db_pool.putconn(conn)  # 归还连接池，而非关闭

# 异步方案：asyncpg（自带连接池）
import asyncpg

pool = await asyncpg.create_pool(
    "postgresql://admin:secret@localhost:5432/mydb",
    min_size=5, max_size=20,
    command_timeout=10,
)

async def get_user(user_id: str):
    async with pool.acquire() as conn:
        return await conn.fetchrow("SELECT * FROM users WHERE id = $1", user_id)
```

### 修复代码 - Node.js

```ts
import { Pool } from 'pg';

// 全局连接池
const pool = new Pool({
  host: 'localhost', port: 5432,
  database: 'mydb', user: 'admin', password: 'secret',
  max: 20,               // 最大连接数
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 5000,
});

async function getUser(userId: string) {
  const client = await pool.connect();
  try {
    const result = await client.query('SELECT * FROM users WHERE id = $1', [userId]);
    return result.rows[0];
  } finally {
    client.release(); // 归还连接池
  }
}
```

**检测方法**：监控数据库活跃连接数（`pg_stat_activity`）；连接数随 QPS 线性增长即无连接池。

---

## 3. 同步阻塞 I/O

**问题**：在异步/事件驱动框架中执行同步阻塞操作（文件读写、CPU 密集计算），阻塞事件循环，导致整个服务无法处理其他请求。

### 问题代码 - Python

```python
# FastAPI 中使用同步阻塞调用
import requests
import time

@app.get("/api/data")
async def get_data():
    # requests 是同步库，会阻塞事件循环
    response = requests.get("https://slow-api.example.com/data", timeout=30)

    # 同步文件读写
    with open("large_file.csv", "r") as f:
        data = f.read()  # 阻塞

    # CPU 密集计算直接在事件循环中执行
    result = heavy_computation(data)

    return {"data": result}
```

### 问题代码 - Node.js

```ts
import fs from 'fs';
import crypto from 'crypto';

app.get('/api/data', (req, res) => {
  // 同步文件读取 → 阻塞事件循环
  const data = fs.readFileSync('large_file.csv', 'utf-8');

  // CPU 密集操作直接在主线程
  const hash = crypto.pbkdf2Sync(data, 'salt', 100000, 64, 'sha512');

  res.json({ hash: hash.toString('hex') });
});
```

### 修复代码 - Python

```python
import httpx
import aiofiles
from concurrent.futures import ProcessPoolExecutor
import asyncio

executor = ProcessPoolExecutor(max_workers=4)

@app.get("/api/data")
async def get_data():
    # 使用异步 HTTP 客户端
    async with httpx.AsyncClient() as client:
        response = await client.get("https://slow-api.example.com/data", timeout=30)

    # 异步文件读写
    async with aiofiles.open("large_file.csv", "r") as f:
        data = await f.read()

    # CPU 密集任务放到进程池
    loop = asyncio.get_event_loop()
    result = await loop.run_in_executor(executor, heavy_computation, data)

    return {"data": result}
```

### 修复代码 - Node.js

```ts
import fs from 'fs/promises';
import { Worker } from 'worker_threads';

app.get('/api/data', async (req, res) => {
  // 异步文件读取
  const data = await fs.readFile('large_file.csv', 'utf-8');

  // CPU 密集操作放到 Worker Thread
  const hash = await runInWorker(data);

  res.json({ hash });
});

function runInWorker(data: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const worker = new Worker('./hash-worker.js', { workerData: data });
    worker.on('message', resolve);
    worker.on('error', reject);
  });
}

// hash-worker.js
import { workerData, parentPort } from 'worker_threads';
import crypto from 'crypto';

const hash = crypto.pbkdf2Sync(workerData, 'salt', 100000, 64, 'sha512');
parentPort.postMessage(hash.toString('hex'));
```

**检测方法**：Python 使用 `asyncio.get_event_loop().slow_callback_duration`；Node.js 使用 `--prof` 或 `clinic doctor`。

---

## 4. 无超时设置

**问题**：HTTP 请求、数据库查询、外部服务调用没有超时，下游故障时请求永久挂起，耗尽线程/连接池。

### 问题代码 - Python

```python
import requests

def call_payment_service(order_id: str):
    # 无超时 → 如果支付服务挂了，请求永久等待
    response = requests.post(
        "https://payment.example.com/charge",
        json={"order_id": order_id},
    )
    return response.json()

# 数据库查询无超时
def get_report():
    # 复杂查询可能执行数分钟，期间连接被占用
    return db.execute("SELECT ... FROM huge_table WHERE ...")
```

### 问题代码 - Node.js

```ts
// 无超时的 fetch
async function callPaymentService(orderId: string) {
  const res = await fetch('https://payment.example.com/charge', {
    method: 'POST',
    body: JSON.stringify({ orderId }),
    // 无 signal、无 timeout → 永久等待
  });
  return res.json();
}
```

### 修复代码 - Python

```python
import httpx

# HTTP 请求超时
async def call_payment_service(order_id: str):
    async with httpx.AsyncClient(
        timeout=httpx.Timeout(
            connect=5.0,    # 连接超时 5s
            read=10.0,      # 读取超时 10s
            write=5.0,      # 写入超时 5s
            pool=5.0,       # 连接池等待超时 5s
        )
    ) as client:
        try:
            response = await client.post(
                "https://payment.example.com/charge",
                json={"order_id": order_id},
            )
            return response.json()
        except httpx.TimeoutException:
            raise ServiceUnavailable("支付服务超时")

# 数据库查询超时
import asyncio

async def get_report():
    try:
        return await asyncio.wait_for(
            db.execute("SELECT ... FROM huge_table WHERE ..."),
            timeout=30.0,  # 30 秒超时
        )
    except asyncio.TimeoutError:
        raise ServiceUnavailable("报表查询超时")

# PostgreSQL 语句级超时
await conn.execute("SET statement_timeout = '30s'")
```

### 修复代码 - Node.js

```ts
// fetch 超时
async function callPaymentService(orderId: string) {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), 10_000); // 10s 超时

  try {
    const res = await fetch('https://payment.example.com/charge', {
      method: 'POST',
      body: JSON.stringify({ orderId }),
      headers: { 'Content-Type': 'application/json' },
      signal: controller.signal,
    });
    return await res.json();
  } catch (err) {
    if (err instanceof DOMException && err.name === 'AbortError') {
      throw new ServiceUnavailable('支付服务超时');
    }
    throw err;
  } finally {
    clearTimeout(timeoutId);
  }
}

// Prisma 查询超时
const result = await prisma.$queryRaw`
  SET statement_timeout = '30s';
  SELECT ... FROM huge_table WHERE ...;
`;
```

**检测方法**：搜索代码中所有 HTTP 客户端调用和数据库查询，确认 timeout 参数存在。

---

## 5. 无重试策略

**问题**：网络抖动或临时故障时请求直接失败，不做重试。或者无脑重试无退避，加剧下游压力（重试风暴）。

### 问题代码 - Python

```python
# 无重试 → 偶发网络错误直接失败
def send_notification(user_id: str, message: str):
    response = requests.post(
        "https://notification.example.com/send",
        json={"user_id": user_id, "message": message},
        timeout=5,
    )
    response.raise_for_status()

# 无脑重试 → 重试风暴
def send_with_bad_retry(user_id: str, message: str):
    for _ in range(10):
        try:
            response = requests.post(url, json=data, timeout=5)
            response.raise_for_status()
            return response.json()
        except Exception:
            pass  # 立即重试，无间隔，无退避
```

### 修复代码 - Python

```python
import tenacity
import random

@tenacity.retry(
    stop=tenacity.stop_after_attempt(3),                  # 最多 3 次
    wait=tenacity.wait_exponential(multiplier=1, max=10)  # 指数退避：1s, 2s, 4s...
         + tenacity.wait_random(0, 1),                    # 加抖动
    retry=tenacity.retry_if_exception_type(
        (httpx.TimeoutException, httpx.HTTPStatusError)
    ),
    before_sleep=tenacity.before_sleep_log(logger, logging.WARNING),
)
async def send_notification(user_id: str, message: str):
    async with httpx.AsyncClient(timeout=5) as client:
        response = await client.post(
            "https://notification.example.com/send",
            json={"user_id": user_id, "message": message},
        )
        # 仅对 5xx 重试，4xx 不重试
        if response.status_code >= 500:
            response.raise_for_status()
        return response.json()
```

### 修复代码 - Node.js

```ts
// 指数退避重试
async function withRetry<T>(
  fn: () => Promise<T>,
  options: { maxAttempts?: number; baseDelay?: number; maxDelay?: number } = {},
): Promise<T> {
  const { maxAttempts = 3, baseDelay = 1000, maxDelay = 10000 } = options;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (err) {
      if (attempt === maxAttempts) throw err;

      // 仅对可重试错误重试
      if (err instanceof HttpError && err.status < 500) throw err;

      const delay = Math.min(
        baseDelay * Math.pow(2, attempt - 1) + Math.random() * 1000,
        maxDelay,
      );
      console.warn(`Attempt ${attempt} failed, retrying in ${delay}ms...`);
      await new Promise(r => setTimeout(r, delay));
    }
  }
  throw new Error('Unreachable');
}

// 使用
const result = await withRetry(() => callPaymentService(orderId), { maxAttempts: 3 });
```

**检测方法**：搜索所有外部 HTTP 调用，确认有重试 + 指数退避 + 抖动。

---

## 6. 非幂等操作

**问题**：POST/PUT 操作不幂等，网络重试或用户重复提交导致重复创建订单、重复扣款等严重业务问题。

### 问题代码 - Python

```python
@app.post("/api/v1/orders")
async def create_order(order: OrderCreate):
    # 无幂等性保护 → 网络重试时重复创建订单
    new_order = Order(
        customer_id=order.customer_id,
        items=order.items,
        total=calculate_total(order.items),
    )
    db.add(new_order)
    db.commit()
    return {"id": new_order.id}
```

### 修复代码 - Python

```python
@app.post("/api/v1/orders")
async def create_order(
    order: OrderCreate,
    idempotency_key: str = Header(..., alias="Idempotency-Key"),
):
    # 检查幂等键是否已处理
    cache_key = f"idempotency:{idempotency_key}"
    cached = await redis.get(cache_key)
    if cached:
        return json.loads(cached)  # 返回缓存结果

    # 使用数据库唯一约束作为最终防线
    try:
        new_order = Order(
            idempotency_key=idempotency_key,
            customer_id=order.customer_id,
            items=order.items,
            total=calculate_total(order.items),
        )
        db.add(new_order)
        db.commit()
    except IntegrityError:
        # 幂等键冲突 → 返回已有订单
        existing = db.query(Order).filter_by(idempotency_key=idempotency_key).first()
        return {"id": existing.id}

    result = {"id": new_order.id}
    # 缓存结果 24 小时
    await redis.setex(cache_key, 86400, json.dumps(result))
    return result
```

### 修复代码 - Node.js

```ts
app.post('/api/v1/orders', async (req, res) => {
  const idempotencyKey = req.headers['idempotency-key'] as string;
  if (!idempotencyKey) {
    return res.status(400).json({ error: { code: 'MISSING_IDEMPOTENCY_KEY' } });
  }

  // 检查 Redis 缓存
  const cached = await redis.get(`idempotency:${idempotencyKey}`);
  if (cached) {
    return res.status(200).json(JSON.parse(cached));
  }

  // 使用数据库事务 + 唯一约束
  try {
    const order = await prisma.order.create({
      data: {
        idempotencyKey,
        customerId: req.body.customerId,
        items: { create: req.body.items },
        total: calculateTotal(req.body.items),
      },
    });

    const result = { id: order.id };
    await redis.setex(`idempotency:${idempotencyKey}`, 86400, JSON.stringify(result));
    res.status(201).json(result);
  } catch (err) {
    if (err.code === 'P2002') { // Prisma unique constraint violation
      const existing = await prisma.order.findUnique({ where: { idempotencyKey } });
      res.status(200).json({ id: existing!.id });
    } else {
      throw err;
    }
  }
});
```

**检测方法**：所有创建/支付类 POST 接口必须要求 `Idempotency-Key` 头。

---

## 7. 无限分页

**问题**：分页接口无 `per_page` 上限或默认值过大，客户端可以请求 `per_page=999999`，一次性拉取全表数据，击穿数据库和内存。

### 问题代码 - Python

```python
@app.get("/api/v1/products")
async def list_products(
    page: int = 1,
    per_page: int = 10,  # 无上限校验
):
    # 攻击者请求 ?per_page=1000000 → OOM
    offset = (page - 1) * per_page
    products = await db.execute(
        f"SELECT * FROM products LIMIT {per_page} OFFSET {offset}"
    )
    return {"data": products}
```

### 修复代码 - Python

```python
from fastapi import Query

MAX_PAGE_SIZE = 100
DEFAULT_PAGE_SIZE = 20

@app.get("/api/v1/products")
async def list_products(
    page: int = Query(1, ge=1, le=10000),              # 页码上限
    per_page: int = Query(DEFAULT_PAGE_SIZE, ge=1, le=MAX_PAGE_SIZE),  # 强制上限
):
    offset = (page - 1) * per_page
    products = await db.execute(
        "SELECT * FROM products ORDER BY id LIMIT $1 OFFSET $2",
        per_page, offset,
    )

    # 深度分页保护：offset 过大时建议使用 cursor
    if offset > 10000:
        return JSONResponse(
            status_code=400,
            content={"error": {"code": "DEEP_PAGINATION", "message": "请使用 cursor 分页"}},
        )

    return {"data": products}
```

### 修复代码 - Node.js

```ts
const MAX_PAGE_SIZE = 100;
const DEFAULT_PAGE_SIZE = 20;

app.get('/api/v1/products', async (req, res) => {
  const page = Math.max(1, Math.min(10000, parseInt(req.query.page as string) || 1));
  const perPage = Math.max(1, Math.min(MAX_PAGE_SIZE, parseInt(req.query.per_page as string) || DEFAULT_PAGE_SIZE));
  const skip = (page - 1) * perPage;

  if (skip > 10000) {
    return res.status(400).json({
      error: { code: 'DEEP_PAGINATION', message: '请使用 cursor 分页' },
    });
  }

  const products = await prisma.product.findMany({
    skip,
    take: perPage,
    orderBy: { id: 'asc' },
  });

  res.json({ data: products });
});
```

**检测方法**：对所有分页接口发送 `per_page=999999`，确认返回 400 或被截断到上限值。

---

## 8. Fat Controller

**问题**：Controller/Handler 中堆满业务逻辑、数据库操作、外部调用、数据转换，违反单一职责原则，无法测试和复用。

### 问题代码 - Python

```python
@app.post("/api/v1/orders")
async def create_order(request: Request):
    body = await request.json()

    # 参数校验（应在 schema 层）
    if not body.get("customer_id"):
        raise HTTPException(400, "customer_id required")
    if not body.get("items") or len(body["items"]) == 0:
        raise HTTPException(400, "items required")

    # 业务逻辑（应在 service 层）
    customer = await db.execute("SELECT * FROM customers WHERE id = $1", body["customer_id"])
    if not customer:
        raise HTTPException(404, "customer not found")

    total = 0
    for item in body["items"]:
        product = await db.execute("SELECT * FROM products WHERE id = $1", item["product_id"])
        if product["stock"] < item["quantity"]:
            raise HTTPException(400, f"库存不足: {product['name']}")
        total += product["price"] * item["quantity"]

    # 数据库操作（应在 repository 层）
    order_id = await db.execute(
        "INSERT INTO orders (customer_id, total, status) VALUES ($1, $2, 'pending') RETURNING id",
        body["customer_id"], total,
    )
    for item in body["items"]:
        await db.execute(
            "INSERT INTO order_items (order_id, product_id, quantity) VALUES ($1, $2, $3)",
            order_id, item["product_id"], item["quantity"],
        )
        await db.execute(
            "UPDATE products SET stock = stock - $1 WHERE id = $2",
            item["quantity"], item["product_id"],
        )

    # 发送通知（应在事件/消息层）
    await send_email(customer["email"], f"订单 {order_id} 已创建")
    await send_sms(customer["phone"], f"订单 {order_id} 已创建")

    return {"id": order_id, "total": total}
```

### 修复代码 - Python

```python
# schema 层：参数校验
class OrderItemCreate(BaseModel):
    product_id: str
    quantity: int = Field(ge=1)

class OrderCreate(BaseModel):
    customer_id: str
    items: list[OrderItemCreate] = Field(min_length=1)

# service 层：业务逻辑
class OrderService:
    def __init__(self, order_repo: OrderRepository, product_repo: ProductRepository,
                 customer_repo: CustomerRepository, notifier: Notifier):
        self.order_repo = order_repo
        self.product_repo = product_repo
        self.customer_repo = customer_repo
        self.notifier = notifier

    async def create_order(self, data: OrderCreate) -> Order:
        customer = await self.customer_repo.get_or_raise(data.customer_id)
        await self._validate_stock(data.items)

        total = await self._calculate_total(data.items)
        order = await self.order_repo.create(
            customer_id=data.customer_id, items=data.items, total=total,
        )
        await self._deduct_stock(data.items)

        # 异步发送通知（不阻塞响应）
        asyncio.create_task(self.notifier.order_created(customer, order))
        return order

# controller 层：仅处理 HTTP 关注点
@app.post("/api/v1/orders", status_code=201)
async def create_order(
    data: OrderCreate,
    service: OrderService = Depends(get_order_service),
):
    order = await service.create_order(data)
    return {"id": order.id, "total": order.total}
```

**检测方法**：Controller 函数超过 30 行即需拆分；Controller 中不应出现直接 SQL 查询。

---

## 9. 无输入验证

**问题**：信任客户端输入，不做类型、范围、格式校验，导致 SQL 注入、XSS、数据损坏等安全和数据完整性问题。

### 问题代码 - Python

```python
@app.get("/api/v1/users")
async def search_users(request: Request):
    name = request.query_params.get("name", "")
    # SQL 注入：name = "'; DROP TABLE users; --"
    users = await db.execute(f"SELECT * FROM users WHERE name LIKE '%{name}%'")

    role = request.query_params.get("role")
    # 无枚举校验：role 可以是任意值
    if role:
        users = [u for u in users if u["role"] == role]

    return {"data": users}

@app.post("/api/v1/users")
async def create_user(request: Request):
    body = await request.json()
    # 无任何校验直接入库
    await db.execute(
        "INSERT INTO users (name, email, age) VALUES ($1, $2, $3)",
        body.get("name"), body.get("email"), body.get("age"),
    )
```

### 修复代码 - Python

```python
from pydantic import BaseModel, Field, EmailStr, field_validator
from typing import Literal
import re

class UserCreate(BaseModel):
    name: str = Field(min_length=1, max_length=100)
    email: EmailStr
    age: int = Field(ge=0, le=150)
    role: Literal["user", "admin", "moderator"] = "user"

    @field_validator("name")
    @classmethod
    def validate_name(cls, v: str) -> str:
        if re.search(r"[<>\"';]", v):
            raise ValueError("名称包含非法字符")
        return v.strip()

@app.get("/api/v1/users")
async def search_users(
    name: str = Query("", max_length=100),
    role: Literal["user", "admin", "moderator"] | None = None,
):
    # 参数化查询，防止 SQL 注入
    query = "SELECT * FROM users WHERE name ILIKE $1"
    params = [f"%{name}%"]
    if role:
        query += " AND role = $2"
        params.append(role)
    users = await db.execute(query, *params)
    return {"data": users}

@app.post("/api/v1/users", status_code=201)
async def create_user(data: UserCreate):
    # Pydantic 自动校验类型、范围、格式
    await db.execute(
        "INSERT INTO users (name, email, age, role) VALUES ($1, $2, $3, $4)",
        data.name, data.email, data.age, data.role,
    )
```

### 修复代码 - Node.js

```ts
import { z } from 'zod';

const UserCreateSchema = z.object({
  name: z.string().min(1).max(100).regex(/^[^<>"';]*$/),
  email: z.string().email(),
  age: z.number().int().min(0).max(150),
  role: z.enum(['user', 'admin', 'moderator']).default('user'),
});

app.post('/api/v1/users', async (req, res) => {
  const result = UserCreateSchema.safeParse(req.body);
  if (!result.success) {
    return res.status(400).json({
      error: {
        code: 'VALIDATION_ERROR',
        details: result.error.issues.map(i => ({
          field: i.path.join('.'),
          message: i.message,
        })),
      },
    });
  }

  // 参数化查询
  await prisma.user.create({ data: result.data });
  res.status(201).json({ status: 'created' });
});
```

**检测方法**：搜索字符串拼接 SQL（`f"SELECT`、`"SELECT ... " +`）；所有 POST/PUT/PATCH 必须有 schema 校验。

---

## 10. 硬编码配置

**问题**：数据库地址、API 密钥、端口号等写死在代码中，无法在不同环境（开发/测试/生产）间切换，且密钥可能泄漏到版本库。

### 问题代码 - Python

```python
import psycopg2

# 数据库密码直接写在代码中 → 提交到 Git 后泄漏
conn = psycopg2.connect(
    host="192.168.1.100",
    port=5432,
    dbname="prod_db",
    user="admin",
    password="super_secret_password_123",
)

# API Key 硬编码
STRIPE_API_KEY = "sk_live_abc123def456"

# 服务地址硬编码
NOTIFICATION_URL = "https://notification.prod.example.com"
```

### 修复代码 - Python

```python
from pydantic_settings import BaseSettings

class Settings(BaseSettings):
    # 从环境变量读取，支持 .env 文件
    database_url: str
    redis_url: str = "redis://localhost:6379"
    stripe_api_key: str
    notification_url: str
    debug: bool = False
    log_level: str = "INFO"

    # 校验必填项
    model_config = {
        "env_file": ".env",
        "env_file_encoding": "utf-8",
    }

settings = Settings()

# 使用
conn = await asyncpg.connect(settings.database_url)
```

### 修复代码 - Node.js

```ts
// config.ts
import { z } from 'zod';
import dotenv from 'dotenv';

dotenv.config();

const ConfigSchema = z.object({
  DATABASE_URL: z.string().url(),
  REDIS_URL: z.string().url().default('redis://localhost:6379'),
  STRIPE_API_KEY: z.string().startsWith('sk_'),
  NOTIFICATION_URL: z.string().url(),
  NODE_ENV: z.enum(['development', 'test', 'production']).default('development'),
  PORT: z.coerce.number().default(3000),
});

// 应用启动时校验，缺失直接报错退出
export const config = ConfigSchema.parse(process.env);
```

```bash
# .env（不提交到 Git）
DATABASE_URL=postgresql://admin:secret@localhost:5432/mydb
STRIPE_API_KEY=sk_test_xxx
NOTIFICATION_URL=https://notification.dev.example.com

# .env.example（提交到 Git，仅含占位符）
DATABASE_URL=postgresql://user:password@localhost:5432/dbname
STRIPE_API_KEY=sk_test_xxx
NOTIFICATION_URL=https://notification.example.com
```

```gitignore
# .gitignore
.env
.env.local
.env.production
```

**检测方法**：

```bash
# 搜索硬编码密钥
grep -rn "password\s*=" --include="*.py" --include="*.ts" src/
grep -rn "sk_live_\|sk_test_\|api_key\s*=" src/

# 使用 gitleaks 扫描
gitleaks detect --source=. --verbose
```

---

## Agent Checklist

- [ ] 所有列表查询已检查 N+1 问题（使用 JOIN/include/prefetch）
- [ ] 数据库连接使用连接池，连接数有上限
- [ ] 异步框架中无同步阻塞调用（requests → httpx、readFileSync → readFile）
- [ ] 所有 HTTP 调用和数据库查询有超时设置
- [ ] 外部调用有重试策略（指数退避 + 抖动 + 仅对 5xx 重试）
- [ ] 创建/支付类接口支持幂等键（Idempotency-Key）
- [ ] 分页接口有 per_page 上限和深度分页保护
- [ ] Controller 不超过 30 行，业务逻辑在 Service 层
- [ ] 所有输入有 schema 校验，SQL 使用参数化查询
- [ ] 无硬编码配置/密钥，敏感值通过环境变量注入
- [ ] .env 文件已加入 .gitignore，.env.example 仅含占位符
