---
id: backend-antipatterns
title: 后端反模式指南
domain: development
category: 04-antipatterns
difficulty: intermediate
tags: [antipatterns, backend, breaker, circuit, controller, development, error, handling]
quality_score: 70
last_updated: 2026-06-15
---
# 后端反模式指南

> 适用范围：Python / Node.js / Go / Java 后端服务
> 约束级别：SHALL（必须在 Code Review 阶段拦截）

---

## 1. 控制器臃肿（Fat Controller）

### 描述
将业务逻辑、数据访问、外部调用、错误处理全部堆积在 Controller / Handler 层，导致 Controller 成为上帝类。违反关注点分离原则，无法对业务逻辑进行独立单元测试。

### 错误示例
```python
@app.post("/orders")
def create_order(request):
    data = request.json
    # 输入校验
    if not data.get("user_id"):
        return {"error": "user_id required"}, 400
    if not data.get("items"):
        return {"error": "items required"}, 400

    # 查用户
    user = db.execute("SELECT * FROM users WHERE id = %s", (data["user_id"],)).fetchone()
    if not user:
        return {"error": "user not found"}, 404

    # 校验库存
    for item in data["items"]:
        stock = db.execute(
            "SELECT stock FROM products WHERE id = %s", (item["product_id"],)
        ).fetchone()
        if not stock or stock["stock"] < item["qty"]:
            return {"error": f"insufficient stock for {item['product_id']}"}, 422

    # 计算金额
    total = 0
    for item in data["items"]:
        product = db.execute(
            "SELECT price FROM products WHERE id = %s", (item["product_id"],)
        ).fetchone()
        total += product["price"] * item["qty"]

    # 应用优惠券
    if data.get("coupon_code"):
        coupon = db.execute(
            "SELECT * FROM coupons WHERE code = %s AND expires_at > NOW()",
            (data["coupon_code"],)
        ).fetchone()
        if coupon:
            total = total * (1 - coupon["discount_rate"])

    # 创建订单
    order_id = db.execute(
        "INSERT INTO orders (user_id, total, status) VALUES (%s, %s, 'pending') RETURNING id",
        (data["user_id"], total)
    ).fetchone()["id"]

    # 扣减库存 ...
    # 发通知 ...
    # 清缓存 ...
    return {"order_id": order_id, "total": total}, 201
```

### 正确示例
```python
# api/orders.py -- Controller 只负责 HTTP 协议处理
@app.post("/orders", response_model=OrderResponse, status_code=201)
def create_order(
    data: CreateOrderRequest,
    user: User = Depends(get_current_user),
    order_service: OrderService = Depends(get_order_service),
):
    try:
        order = order_service.create(user_id=user.id, items=data.items, coupon_code=data.coupon_code)
        return OrderResponse.from_entity(order)
    except InsufficientStockError as e:
        raise HTTPException(status_code=422, detail=str(e))
    except CouponExpiredError as e:
        raise HTTPException(status_code=400, detail=str(e))

# services/order_service.py -- 业务逻辑层
class OrderService:
    def __init__(
        self,
        order_repo: OrderRepository,
        inventory_service: InventoryService,
        pricing_service: PricingService,
        notification_service: NotificationService,
    ):
        self._repo = order_repo
        self._inventory = inventory_service
        self._pricing = pricing_service
        self._notification = notification_service

    def create(self, user_id: int, items: list[OrderItem], coupon_code: str | None) -> Order:
        self._inventory.check_and_reserve(items)
        total = self._pricing.calculate(items, coupon_code)
        order = self._repo.create(user_id=user_id, items=items, total=total)
        self._notification.send_order_created(order)
        return order
```

### 检测方法
- Controller 文件行数 > 100 行。
- Controller 方法中包含数据库查询（`db.execute` / ORM 查询）。
- Controller 方法中包含外部 HTTP 调用。
- Controller 方法的单元测试需要 mock > 3 个依赖。

### 修复步骤
1. 将业务逻辑提取到 Service 层。
2. 将数据访问提取到 Repository 层。
3. Controller 仅负责：解析请求、调用 Service、返回响应、映射异常到 HTTP 状态码。
4. Service 层通过依赖注入获取 Repository 和其他 Service。

### Agent Checklist
- [ ] Controller 方法行数 <= 20
- [ ] Controller 中无数据库操作
- [ ] Controller 中无外部 HTTP 调用
- [ ] 业务逻辑在 Service 层可独立测试

---

## 2. 外部依赖无超时与熔断（Missing Timeout and Circuit Breaker）

### 描述
调用外部服务（HTTP API、数据库、Redis、消息队列）时不设置超时时间，也没有熔断降级机制。当外部依赖出现故障时，调用方线程阻塞等待，导致连接池耗尽、请求堆积、最终级联雪崩。

### 错误示例
```python
# 无超时 -- 外部服务挂起时永远等待
def get_user_profile(user_id):
    response = requests.get(f"{USER_SERVICE}/users/{user_id}")  # 无 timeout
    return response.json()

# 无熔断 -- 外部服务已宕机仍继续请求
def get_exchange_rate(currency):
    try:
        response = requests.get(f"{RATE_SERVICE}/rate/{currency}", timeout=5)
        return response.json()["rate"]
    except Exception:
        # 每次请求都尝试，即使服务已经连续失败 1000 次
        return None

# 数据库无超时
def long_query():
    # 复杂查询可能执行数分钟，阻塞连接池
    return db.execute("SELECT * FROM huge_table WHERE complex_condition = true")
```

### 正确示例
```python
import httpx
from circuitbreaker import circuit

# HTTP 调用设置超时
http_client = httpx.AsyncClient(
    timeout=httpx.Timeout(
        connect=2.0,     # 连接超时 2 秒
        read=5.0,        # 读取超时 5 秒
        write=5.0,       # 写入超时 5 秒
        pool=10.0,       # 连接池获取超时 10 秒
    ),
    limits=httpx.Limits(
        max_connections=100,
        max_keepalive_connections=20,
    ),
)

# 熔断器保护外部调用
@circuit(
    failure_threshold=5,       # 连续失败 5 次后熔断
    recovery_timeout=30,       # 熔断后 30 秒尝试恢复
    expected_exception=Exception,
)
async def get_exchange_rate(currency: str) -> Decimal:
    response = await http_client.get(f"{RATE_SERVICE}/rate/{currency}")
    response.raise_for_status()
    return Decimal(response.json()["rate"])

# 带降级的调用
async def get_exchange_rate_with_fallback(currency: str) -> Decimal:
    try:
        return await get_exchange_rate(currency)
    except CircuitBreakerError:
        logger.warning("Exchange rate circuit breaker open, using cached rate")
        return await cache.get_cached_rate(currency)

# 数据库超时设置
from sqlalchemy import create_engine

engine = create_engine(
    DATABASE_URL,
    pool_size=10,
    max_overflow=5,
    pool_timeout=10,             # 连接池获取超时
    pool_recycle=300,            # 连接回收时间
    connect_args={
        "connect_timeout": 5,    # 连接超时
        "options": "-c statement_timeout=30000",  # SQL 执行超时 30 秒
    },
)
```

### 检测方法
- `requests.get()` / `requests.post()` 无 `timeout` 参数。
- 数据库连接字符串无 `connect_timeout` / `statement_timeout`。
- 无 `circuitbreaker` / `pybreaker` / `tenacity` 等熔断库使用。
- 外部调用失败时无降级逻辑（直接返回 500）。

### 修复步骤
1. 为所有 HTTP 客户端设置 connect / read / write timeout。
2. 为数据库连接设置 connection timeout 和 statement timeout。
3. 对关键外部依赖添加熔断器。
4. 为每个外部调用定义降级策略（缓存兜底 / 默认值 / 部分功能降级）。
5. 添加监控：超时率、熔断触发次数、降级触发次数。

### Agent Checklist
- [ ] 所有 HTTP 调用有 timeout
- [ ] 数据库连接有 connection timeout + statement timeout
- [ ] 关键外部依赖有熔断器
- [ ] 每个外部调用有降级策略
- [ ] 有超时和熔断的监控告警

---

## 3. 错误处理不统一（Inconsistent Error Handling）

### 描述
各模块各自定义错误格式，有的返回字典、有的抛异常、有的返回 None、有的打日志不返回。调用方无法统一处理，调试时无法快速定位错误来源。

### 错误示例
```python
# 模块 A: 返回 None 表示错误
def get_user(user_id):
    user = db.query(User).get(user_id)
    if not user:
        return None  # 调用方怎么区分"未找到"和"查询失败"？

# 模块 B: 返回错误字典
def create_payment(order_id, amount):
    if amount <= 0:
        return {"success": False, "error": "invalid amount"}
    # ...
    return {"success": True, "payment_id": "pay_123"}

# 模块 C: 吞掉异常
def send_notification(user_id, message):
    try:
        email_service.send(user_id, message)
    except Exception:
        pass  # 静默失败，完全不知道发生了什么

# 模块 D: 打日志但返回默认值
def get_config(key):
    try:
        return config_service.get(key)
    except Exception as e:
        logging.error(f"Config error: {e}")
        return "default_value"  # 调用方以为成功了
```

### 正确示例
```python
# 统一的异常体系
class AppError(Exception):
    """所有业务异常的基类"""
    def __init__(self, message: str, code: str, details: dict | None = None):
        self.message = message
        self.code = code
        self.details = details or {}
        super().__init__(message)

class NotFoundError(AppError):
    def __init__(self, entity: str, entity_id: str | int):
        super().__init__(
            message=f"{entity} with id {entity_id} not found",
            code="NOT_FOUND",
            details={"entity": entity, "id": str(entity_id)},
        )

class ValidationError(AppError):
    def __init__(self, field: str, reason: str):
        super().__init__(
            message=f"Validation failed on {field}: {reason}",
            code="VALIDATION_ERROR",
            details={"field": field, "reason": reason},
        )

class ExternalServiceError(AppError):
    def __init__(self, service: str, original_error: Exception):
        super().__init__(
            message=f"External service {service} failed",
            code="EXTERNAL_SERVICE_ERROR",
            details={"service": service, "original": str(original_error)},
        )

# 模块 A: 抛出明确异常
def get_user(user_id: int) -> User:
    user = db.query(User).get(user_id)
    if not user:
        raise NotFoundError("User", user_id)
    return user

# 模块 B: 抛出明确异常
def create_payment(order_id: str, amount: Decimal) -> Payment:
    if amount <= 0:
        raise ValidationError("amount", "must be positive")
    return payment_gateway.charge(order_id, amount)

# 模块 C: 记录异常并让调用方决定是否忽略
def send_notification(user_id: int, message: str) -> None:
    try:
        email_service.send(user_id, message)
    except Exception as e:
        logger.error("Notification failed", user_id=user_id, error=str(e))
        raise ExternalServiceError("email_service", e)
```

### 检测方法
- 搜索 `except Exception: pass` 或 `except: pass`。
- 函数返回类型是 `dict | None` 且 dict 中包含 `error` / `success` 字段。
- `bandit` B110 规则（try_except_pass）。
- 同一项目中存在 3 种以上不同的错误返回格式。

### 修复步骤
1. 定义统一的异常基类层次结构（AppError -> NotFoundError / ValidationError / ...）。
2. 将 `return None` 改为 `raise NotFoundError(...)`。
3. 将 `return {"error": ...}` 改为 `raise AppError(...)`。
4. 将 `except: pass` 改为 `except: logger.error(...); raise`。
5. 在 Controller 层统一捕获 AppError 并转为 HTTP 响应。

### Agent Checklist
- [ ] 无 `except: pass` 或 `except Exception: pass`
- [ ] 所有业务错误使用统一异常体系
- [ ] 函数不返回 `{"error": ...}` 字典
- [ ] 不用 `return None` 表示错误（使用异常）
- [ ] 异常日志包含上下文信息

---

## 4. 幂等性缺失（Missing Idempotency）

### 描述
写操作（创建订单、扣款、发通知）没有幂等保护，客户端重试或网络重传时导致重复执行。例如用户点击两次"支付"按钮，扣款两次。

### 错误示例
```python
# 非幂等的支付接口 -- 重试会导致重复扣款
@app.post("/payments")
def create_payment(data: PaymentRequest):
    payment = payment_gateway.charge(
        user_id=data.user_id,
        amount=data.amount,
    )
    db.execute(
        "INSERT INTO payments (user_id, amount, status) VALUES (%s, %s, 'success')",
        (data.user_id, data.amount)
    )
    return {"payment_id": payment.id}
    # 客户端超时重试 -> 再次扣款

# 非幂等的通知 -- 重复发送
@app.post("/notifications/send")
def send_notification(data: NotificationRequest):
    email_service.send(data.user_email, data.subject, data.body)
    return {"status": "sent"}
    # 重试 -> 用户收到多封相同邮件
```

### 正确示例
```python
import uuid

# 幂等键方案
@app.post("/payments")
def create_payment(
    data: PaymentRequest,
    idempotency_key: str = Header(..., alias="Idempotency-Key"),
):
    # 检查幂等键是否已使用
    existing = payment_repo.get_by_idempotency_key(idempotency_key)
    if existing:
        return PaymentResponse.from_entity(existing)  # 直接返回之前的结果

    # 首次执行
    with db.transaction() as tx:
        payment = payment_gateway.charge(
            user_id=data.user_id,
            amount=data.amount,
            idempotency_key=idempotency_key,
        )
        payment_repo.create(
            user_id=data.user_id,
            amount=data.amount,
            idempotency_key=idempotency_key,
            status="success",
            gateway_id=payment.id,
        )
    return PaymentResponse.from_entity(payment)

# 数据库层幂等约束
# CREATE UNIQUE INDEX idx_payments_idempotency ON payments(idempotency_key);

# 通知去重
class NotificationService:
    def send_once(self, notification_id: str, user_email: str, subject: str, body: str):
        cache_key = f"notification_sent:{notification_id}"
        if self._redis.get(cache_key):
            logger.info("Notification already sent", notification_id=notification_id)
            return
        email_service.send(user_email, subject, body)
        self._redis.setex(cache_key, 86400, "1")  # 24 小时去重窗口
```

### 检测方法
- 写接口无 `Idempotency-Key` header 或请求内的唯一标识。
- 数据库写操作无唯一约束防止重复。
- 外部支付/通知调用无去重逻辑。
- 负载测试中重复提交产生重复数据。

### 修复步骤
1. 为所有写接口添加 `Idempotency-Key` header 支持。
2. 数据库层添加唯一索引（幂等键列）。
3. 外部调用前检查幂等键是否已使用。
4. 使用 Redis 实现短时间窗口的去重。
5. 编写重复提交的测试用例。

### Agent Checklist
- [ ] 写接口支持 `Idempotency-Key`
- [ ] 数据库有唯一索引防止重复
- [ ] 支付/扣款调用有幂等保护
- [ ] 通知发送有去重机制
- [ ] 有重复提交的测试用例

---

## 5. 日志质量差（Poor Logging）

### 描述
日志缺乏结构化、缺少上下文信息（request_id、user_id），或者过度打印敏感信息（密码、Token、身份证号），或者日志级别使用不当。出问题时无法通过日志定位根因。

### 错误示例
```python
# 无结构化，无上下文
def process_order(order_id, user_id):
    print(f"Processing order {order_id}")  # 用 print 代替 logger
    try:
        result = payment_service.charge(order_id)
        print(f"Payment done: {result}")
    except Exception as e:
        print(f"Error: {e}")  # 无堆栈、无上下文

# 泄露敏感信息
def login(username, password):
    logger.info(f"Login attempt: username={username}, password={password}")  # 密码入日志
    user = authenticate(username, password)
    logger.info(f"User token: {user.token}")  # Token 入日志

# 级别不当
def get_user(user_id):
    logger.error(f"Getting user {user_id}")  # 正常操作用 ERROR 级别
    user = db.query(User).get(user_id)
    if not user:
        logger.debug(f"User {user_id} not found")  # 业务错误用 DEBUG 级别
    return user
```

### 正确示例
```python
import structlog

logger = structlog.get_logger()

def process_order(order_id: str, user_id: str) -> Order:
    log = logger.bind(order_id=order_id, user_id=user_id)
    log.info("order_processing_started")

    try:
        result = payment_service.charge(order_id)
        log.info("payment_completed", payment_id=result.id, amount=result.amount)
    except PaymentError as e:
        log.error("payment_failed", error_code=e.code, error_message=e.message)
        raise

    return order

# 敏感信息脱敏
def login(username: str, password: str):
    logger.info("login_attempt", username=username)  # 不记录密码
    user = authenticate(username, password)
    logger.info("login_success", user_id=user.id)    # 不记录 Token

# 正确的日志级别
# DEBUG: 开发调试信息（请求参数、SQL 语句）
# INFO:  正常业务事件（订单创建、支付完成）
# WARNING: 可恢复的异常情况（缓存未命中、降级触发）
# ERROR: 需要关注的错误（支付失败、外部服务超时）
# CRITICAL: 系统级故障（数据库不可用、磁盘满）

# 结构化日志配置
structlog.configure(
    processors=[
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.add_log_level,
        structlog.processors.StackInfoRenderer(),
        structlog.processors.format_exc_info,
        structlog.processors.JSONRenderer(),
    ],
)
```

### 检测方法
- 代码中使用 `print()` 作为日志。
- 日志中包含 `password`、`token`、`secret`、`credit_card` 等关键词。
- 正常流程使用 `ERROR` / `CRITICAL` 级别。
- 日志无 `request_id`、`user_id` 等上下文字段。
- `bandit` 的日志相关规则。

### 修复步骤
1. 将 `print()` 替换为 `structlog` / `logging` 模块。
2. 统一日志格式为 JSON 结构化日志。
3. 为每个请求绑定 `request_id`，贯穿整个调用链。
4. 审查日志中的敏感信息，添加脱敏处理。
5. 校准日志级别，确保告警系统不被噪声淹没。

### Agent Checklist
- [ ] 无 `print()` 语句用于日志
- [ ] 使用结构化日志（JSON 格式）
- [ ] 日志包含 request_id 上下文
- [ ] 无密码 / Token / 密钥出现在日志中
- [ ] 日志级别使用正确

---

## 6. 配置硬编码（Hardcoded Configuration）

### 描述
将环境相关的配置（数据库地址、缓存地址、第三方 API URL、功能开关）硬编码在源码中，导致部署不同环境时需要修改代码。

### 错误示例
```python
# 硬编码环境配置
class Config:
    DB_HOST = "192.168.1.100"
    DB_PORT = 5432
    REDIS_HOST = "192.168.1.101"
    API_URL = "https://api.production.example.com"
    FEATURE_NEW_CHECKOUT = True
    MAX_UPLOAD_SIZE = 10 * 1024 * 1024  # 10MB

# 根据环境名称 if-else
import os
env = os.getenv("ENV", "dev")
if env == "production":
    DB_HOST = "prod-db.internal"
    CACHE_TTL = 3600
elif env == "staging":
    DB_HOST = "staging-db.internal"
    CACHE_TTL = 600
else:
    DB_HOST = "localhost"
    CACHE_TTL = 60
```

### 正确示例
```python
from pydantic_settings import BaseSettings
from pydantic import Field

class Settings(BaseSettings):
    """所有配置从环境变量加载，支持 .env 文件覆盖。"""

    # 数据库
    db_host: str = "localhost"
    db_port: int = 5432
    db_name: str = "myapp"
    db_user: str = "app_user"
    db_password: str

    # Redis
    redis_url: str = "redis://localhost:6379/0"
    cache_ttl: int = Field(default=300, description="Cache TTL in seconds")

    # 外部服务
    payment_api_url: str
    notification_api_url: str

    # 功能开关
    feature_new_checkout: bool = False
    max_upload_size_mb: int = 10

    @property
    def database_url(self) -> str:
        return f"postgresql://{self.db_user}:{self.db_password}@{self.db_host}:{self.db_port}/{self.db_name}"

    @property
    def max_upload_size_bytes(self) -> int:
        return self.max_upload_size_mb * 1024 * 1024

    model_config = {"env_file": ".env", "env_prefix": "APP_"}

settings = Settings()
```

### 检测方法
- 源码中包含 IP 地址、域名、端口号字面量。
- 代码中有 `if env == "production"` 分支。
- 修改配置值需要修改源码并重新部署。
- 不同环境的配置差异通过代码分支实现。

### 修复步骤
1. 使用 Pydantic Settings / python-decouple / 12-Factor 方式从环境变量加载配置。
2. 为所有配置项提供合理的开发环境默认值。
3. 使用 `.env` 文件管理本地开发配置。
4. 功能开关使用配置中心或环境变量，不硬编码。
5. 将配置验证放在应用启动阶段，缺少必需配置时 fail-fast。

### Agent Checklist
- [ ] 环境相关配置从环境变量加载
- [ ] 无 IP 地址 / 域名硬编码在源码中
- [ ] 无 `if env == "production"` 代码分支
- [ ] 应用启动时验证必需配置
- [ ] 功能开关通过配置管理

---

## 全局 Agent Checklist

| 检查项 | 阈值 | 工具 |
|--------|------|------|
| Controller 方法行数 | <= 20 | Code Review |
| HTTP 调用无 timeout | 0 处 | `grep timeout` |
| `except: pass` | 0 处 | `bandit` B110 |
| 写接口无幂等保护 | 0 个 | API Review |
| `print()` 日志 | 0 处 | `ruff` T201 |
| 硬编码 IP / 域名 | 0 处 | Code Review |
