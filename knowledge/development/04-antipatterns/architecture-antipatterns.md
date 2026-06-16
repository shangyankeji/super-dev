---
id: architecture-antipatterns
title: 架构反模式指南
domain: development
category: 04-antipatterns
difficulty: intermediate
tags: [agent, antipatterns, architecture, ball, bounded, checklist, context, dependency]
quality_score: 70
last_updated: 2026-06-15
---
# 架构反模式指南

> 适用范围：微服务 / 单体 / 模块化单体 / 事件驱动架构
> 约束级别：SHALL（必须在架构评审阶段拦截）

---

## 1. 分布式单体（Distributed Monolith）

### 描述
名义上拆分为多个微服务，但服务间通过同步 RPC 紧密耦合，共享数据库，必须同时部署。具备了分布式系统的所有复杂性（网络延迟、部分失败、数据一致性），却没有获得微服务的任何好处（独立部署、独立扩展、技术多样性）。

### 错误示例
```python
# order-service 同步调用 user-service、inventory-service、payment-service
# 任何一个服务宕机，整个下单流程失败

class OrderService:
    def create_order(self, user_id: str, items: list) -> Order:
        # 同步 RPC 链：order -> user -> inventory -> payment -> notification
        user = requests.get(f"{USER_SERVICE}/users/{user_id}").json()       # 阻塞
        for item in items:
            stock = requests.get(                                            # 阻塞
                f"{INVENTORY_SERVICE}/stock/{item['product_id']}"
            ).json()
            if stock['quantity'] < item['qty']:
                raise InsufficientStockError(item['product_id'])

        order = self._save_order(user_id, items)

        payment = requests.post(f"{PAYMENT_SERVICE}/charge", json={          # 阻塞
            "order_id": order.id, "amount": order.total
        }).json()

        requests.post(f"{NOTIFICATION_SERVICE}/email", json={                # 阻塞
            "to": user['email'], "template": "order_created"
        })

        return order
```

```yaml
# 所有服务共享同一个数据库 -- 本质还是单体
# docker-compose.yml
services:
  user-service:
    environment:
      DATABASE_URL: postgres://shared-db:5432/main_db
  order-service:
    environment:
      DATABASE_URL: postgres://shared-db:5432/main_db  # 同一个库！
  payment-service:
    environment:
      DATABASE_URL: postgres://shared-db:5432/main_db  # 同一个库！
```

### 正确示例
```python
# 使用事件驱动解耦关键路径
class OrderService:
    def __init__(self, event_bus: EventBus, order_repo: OrderRepository):
        self._event_bus = event_bus
        self._repo = order_repo

    async def create_order(self, user_id: str, items: list) -> Order:
        # 只做最少的同步校验
        order = self._repo.create(user_id=user_id, items=items, status="pending")

        # 通过事件异步编排后续流程
        await self._event_bus.publish(OrderCreatedEvent(
            order_id=order.id,
            user_id=user_id,
            items=items,
            total=order.total,
        ))
        return order

# inventory-service 订阅事件，独立处理
class InventoryEventHandler:
    async def handle_order_created(self, event: OrderCreatedEvent):
        try:
            self._reserve_stock(event.items)
            await self._event_bus.publish(StockReservedEvent(order_id=event.order_id))
        except InsufficientStockError:
            await self._event_bus.publish(StockReservationFailedEvent(
                order_id=event.order_id, reason="insufficient_stock"
            ))

# 每个服务有自己的数据库
# docker-compose.yml
services:
  user-service:
    environment:
      DATABASE_URL: postgres://user-db:5432/users
  order-service:
    environment:
      DATABASE_URL: postgres://order-db:5432/orders
  payment-service:
    environment:
      DATABASE_URL: postgres://payment-db:5432/payments
```

### 检测方法
- 部署时需要多个服务同时发版才能工作。
- 服务间调用链深度 > 3（A -> B -> C -> D）。
- 多个服务连接同一个数据库实例。
- 单个服务宕机导致全链路不可用。
- 服务间 RPC 调用无超时、无熔断、无降级。

### 修复步骤
1. 绘制服务依赖图，识别同步调用链。
2. 将非关键路径（通知、日志、统计）改为异步事件。
3. 为每个服务建立独立数据库（Database per Service）。
4. 引入消息队列（Kafka / RabbitMQ / NATS）作为服务间通信骨干。
5. 对仍需同步调用的场景，添加超时、熔断、降级策略。
6. 实现 Saga 模式处理跨服务的分布式事务。

### Agent Checklist
- [ ] 每个服务有独立数据库
- [ ] 服务间同步调用链深度 <= 2
- [ ] 非关键路径使用异步事件
- [ ] 同步 RPC 有超时 + 熔断 + 降级
- [ ] 可独立部署和扩展单个服务

---

## 2. 大泥球（Big Ball of Mud）

### 描述
系统没有明确的架构，代码随意组织，模块间无边界，任何代码可以调用任何代码。随着功能增长，系统变得越来越难以理解、修改和测试。通常是长期缺乏架构治理的结果。

### 错误示例
```
# 项目结构：所有代码扁平堆放在一起
src/
├── utils.py          # 3000 行的"工具"文件
├── helpers.py        # 2000 行的"帮助"文件
├── models.py         # 所有表模型混在一个文件
├── views.py          # 所有 API 混在一个文件
├── services.py       # 所有业务逻辑混在一个文件
├── tasks.py          # 所有异步任务混在一个文件
└── constants.py      # 所有常量混在一个文件
```

```python
# utils.py -- 什么都往里塞
def send_email(to, subject, body): ...
def calculate_tax(amount): ...
def parse_csv(file_path): ...
def resize_image(image, width): ...
def validate_credit_card(number): ...
def generate_report(data): ...
def encrypt_password(password): ...
def format_currency(amount, locale): ...
# ... 300 个互不相关的函数
```

### 正确示例
```
# 按业务域组织的模块化结构
src/
├── users/
│   ├── __init__.py
│   ├── models.py          # User、UserProfile
│   ├── repository.py      # UserRepository
│   ├── service.py         # UserService
│   ├── api.py             # /api/v1/users 路由
│   └── tests/
│       ├── test_service.py
│       └── test_api.py
├── orders/
│   ├── __init__.py
│   ├── models.py          # Order、OrderItem
│   ├── repository.py
│   ├── service.py
│   ├── api.py
│   ├── events.py          # OrderCreated、OrderPaid
│   └── tests/
├── payments/
│   ├── __init__.py
│   ├── models.py
│   ├── gateway.py         # 支付网关抽象
│   ├── service.py
│   └── tests/
├── shared/
│   ├── auth/              # 认证模块
│   ├── email/             # 邮件模块
│   └── storage/           # 文件存储模块
└── infrastructure/
    ├── database.py        # 数据库连接
    ├── cache.py           # 缓存配置
    └── message_bus.py     # 消息队列
```

```python
# 明确的模块边界：模块间通过接口通信
# orders/service.py
class OrderService:
    def __init__(
        self,
        order_repo: OrderRepository,
        user_service: UserServiceInterface,    # 接口，不是具体实现
        payment_service: PaymentServiceInterface,
    ):
        self._repo = order_repo
        self._users = user_service
        self._payments = payment_service
```

### 检测方法
- 单文件行数 > 1000 行。
- `utils.py` / `helpers.py` / `common.py` 文件存在且行数 > 500。
- 模块间存在双向 import。
- 无法用一句话描述一个模块的职责。
- `git log` 显示大部分 PR 都修改了同一组文件。

### 修复步骤
1. 梳理业务域，列出 3-7 个核心域（用户、订单、支付等）。
2. 按业务域创建目录结构，每个域包含 models / repository / service / api。
3. 将现有代码按业务域逐步迁移到对应目录。
4. 定义模块间的公开接口（`__init__.py` 只导出接口类）。
5. 使用 `import-linter` 或 `deptry` 强制模块间的依赖方向。
6. 每次迁移后运行全量测试确认行为不变。

### Agent Checklist
- [ ] 按业务域组织目录结构
- [ ] 单文件行数 <= 500
- [ ] 无 `utils.py` / `helpers.py` 万能文件
- [ ] 模块间无双向依赖
- [ ] 有 import lint 工具强制依赖方向

---

## 3. 金锤子（Golden Hammer）

### 描述
对所有问题都使用同一个技术方案，不考虑场景适配性。例如所有存储都用 MySQL、所有通信都用 REST、所有前端都用 React。导致在不适合的场景中勉强使用，增加了复杂度和维护成本。

### 错误示例
```python
# 所有数据都塞进 MySQL，包括日志、会话、搜索、缓存

# 用 MySQL 做缓存（应该用 Redis）
def get_cached_result(key):
    row = db.execute("SELECT value FROM cache_table WHERE key = %s AND expires_at > NOW()", (key,))
    return json.loads(row['value']) if row else None

# 用 MySQL 做全文搜索（应该用 Elasticsearch）
def search_products(query):
    return db.execute(
        "SELECT * FROM products WHERE name LIKE %s OR description LIKE %s",
        (f"%{query}%", f"%{query}%")
    )  # 性能极差，不支持分词、相关度排序

# 用 MySQL 做消息队列（应该用 Kafka / RabbitMQ）
def publish_message(topic, payload):
    db.execute(
        "INSERT INTO message_queue (topic, payload, status) VALUES (%s, %s, 'pending')",
        (topic, json.dumps(payload))
    )

def consume_messages(topic):
    while True:
        rows = db.execute(
            "SELECT * FROM message_queue WHERE topic = %s AND status = 'pending' "
            "ORDER BY id LIMIT 10 FOR UPDATE SKIP LOCKED",
            (topic,)
        )
        for row in rows:
            process(row)
            db.execute("UPDATE message_queue SET status = 'done' WHERE id = %s", (row['id'],))
        time.sleep(1)  # 轮询
```

### 正确示例
```python
# 根据场景选择合适的技术

# 结构化业务数据 -> PostgreSQL
class OrderRepository:
    def __init__(self, db: AsyncSession):
        self._db = db

# 缓存 -> Redis
class CacheService:
    def __init__(self, redis: Redis):
        self._redis = redis

    async def get(self, key: str) -> str | None:
        return await self._redis.get(key)

    async def set(self, key: str, value: str, ttl: int = 300) -> None:
        await self._redis.setex(key, ttl, value)

# 全文搜索 -> Elasticsearch
class ProductSearchService:
    def __init__(self, es: AsyncElasticsearch):
        self._es = es

    async def search(self, query: str, page: int = 1, size: int = 20):
        return await self._es.search(
            index="products",
            body={
                "query": {"multi_match": {"query": query, "fields": ["name^3", "description"]}},
                "from": (page - 1) * size,
                "size": size,
            },
        )

# 异步消息 -> Kafka / RabbitMQ
class EventPublisher:
    def __init__(self, producer: AIOKafkaProducer):
        self._producer = producer

    async def publish(self, topic: str, event: BaseEvent) -> None:
        await self._producer.send_and_wait(
            topic, value=event.model_dump_json().encode()
        )
```

### 检测方法
- 同一个数据库承担了缓存、搜索、消息队列、会话管理等多种职责。
- 技术选型文档中只有一种方案，无对比分析。
- `LIKE '%query%'` 出现在搜索场景中。
- 数据库中存在名为 `cache_table`、`message_queue`、`sessions` 的表。

### 修复步骤
1. 列出系统中所有的数据访问模式（CRUD、缓存、搜索、消息、流处理）。
2. 为每种访问模式选择最适合的技术方案。
3. 制定技术选型决策记录（ADR），记录选型理由和替代方案。
4. 通过抽象层（Repository / Client 接口）隔离具体实现。
5. 逐步迁移，先迁移性能瓶颈最严重的场景。

### Agent Checklist
- [ ] 技术选型有 ADR 文档记录
- [ ] 缓存使用 Redis / Memcached
- [ ] 全文搜索使用 Elasticsearch / Meilisearch
- [ ] 消息队列使用 Kafka / RabbitMQ / NATS
- [ ] 不同技术通过抽象接口隔离

---

## 4. 过度设计（Over-Engineering）

### 描述
在简单的业务场景中引入了不必要的复杂架构（微服务、CQRS、Event Sourcing、DDD 全套战术模式），导致开发效率低、认知负担高、运维成本大。典型表现是项目只有 3 个开发者和 1000 个 DAU，却部署了 20 个微服务。

### 错误示例
```python
# 一个简单的博客系统，被过度设计为完整的 DDD + CQRS + Event Sourcing

# 6 层抽象才能创建一篇文章
class CreatePostCommandHandler:
    def __init__(self, unit_of_work: UnitOfWork):
        self._uow = unit_of_work

    def handle(self, command: CreatePostCommand) -> None:
        with self._uow:
            post = PostAggregate.create(
                title=Title(command.title),          # 值对象
                content=Content(command.content),    # 值对象
                author_id=AuthorId(command.author_id),  # 值对象
            )
            self._uow.posts.add(post)
            self._uow.commit()

class CreatePostCommand:
    title: str
    content: str
    author_id: str

class PostAggregate(AggregateRoot):
    @classmethod
    def create(cls, title: Title, content: Content, author_id: AuthorId):
        post = cls()
        post.apply(PostCreatedEvent(title=title, content=content, author_id=author_id))
        return post

    def _on_post_created(self, event: PostCreatedEvent):
        self._title = event.title
        self._content = event.content

# 读模型还需要单独的投影和查询处理器...
class PostReadModelProjection:
    def on_post_created(self, event: PostCreatedEvent): ...

class GetPostQueryHandler:
    def handle(self, query: GetPostQuery) -> PostReadModel: ...
```

### 正确示例
```python
# 同样的博客系统，适度设计

# models.py
class Post(Base):
    __tablename__ = "posts"
    id = Column(Integer, primary_key=True)
    title = Column(String(200), nullable=False)
    content = Column(Text, nullable=False)
    author_id = Column(Integer, ForeignKey("users.id"), nullable=False)
    created_at = Column(DateTime, default=func.now())
    updated_at = Column(DateTime, onupdate=func.now())

# service.py
class PostService:
    def __init__(self, db: Session):
        self._db = db

    def create_post(self, author_id: int, title: str, content: str) -> Post:
        post = Post(author_id=author_id, title=title, content=content)
        self._db.add(post)
        self._db.commit()
        return post

    def get_post(self, post_id: int) -> Post | None:
        return self._db.query(Post).get(post_id)

# api.py
@app.post("/posts", response_model=PostResponse, status_code=201)
def create_post(data: CreatePostRequest, user: User = Depends(get_current_user)):
    return post_service.create_post(
        author_id=user.id, title=data.title, content=data.content
    )
```

### 检测方法
- 实现一个简单 CRUD 功能需要修改 > 5 个文件。
- 项目中存在大量只有一个实现的接口（`IXxxRepository`、`IXxxService`）。
- 代码行数和业务复杂度不成比例（1000 DAU 的应用有 10 万行代码）。
- 新人 onboarding 需要 > 1 周才能提交第一个 PR。
- 存在 `AbstractFactoryBuilder`、`CommandHandlerDispatcher` 等深层嵌套的设计模式。

### 修复步骤
1. 评估当前系统的实际规模（DAU、数据量、团队大小）。
2. 对比当前架构复杂度与业务复杂度是否匹配。
3. 删除只有一个实现的接口，直接使用具体类。
4. 合并过度拆分的微服务为模块化单体。
5. 保留架构扩展点（接口），但延迟到真正需要时再引入。

### Agent Checklist
- [ ] CRUD 功能修改文件数 <= 3
- [ ] 接口和实现的比例合理（非 1:1 对应）
- [ ] 架构复杂度与业务规模匹配
- [ ] 新人可在 3 天内提交首个 PR
- [ ] 无不必要的设计模式嵌套

---

## 5. 循环依赖（Circular Dependency）

### 描述
模块 A 依赖 B，B 又依赖 A（直接或间接），形成环形依赖。导致无法独立测试、无法独立部署、import 顺序敏感、编译错误。在微服务场景下表现为服务间的循环调用。

### 错误示例
```python
# user/service.py
from order.service import OrderService  # user 依赖 order

class UserService:
    def __init__(self):
        self._order_service = OrderService()

    def get_user_with_orders(self, user_id: int):
        user = self._repo.get(user_id)
        user.orders = self._order_service.get_by_user(user_id)
        return user

    def deactivate_user(self, user_id: int):
        self._repo.deactivate(user_id)

# order/service.py
from user.service import UserService  # order 反过来依赖 user -- 循环！

class OrderService:
    def __init__(self):
        self._user_service = UserService()

    def get_by_user(self, user_id: int):
        return self._repo.get_by_user(user_id)

    def cancel_order(self, order_id: int):
        order = self._repo.get(order_id)
        # 取消订单时需要检查用户状态
        user = self._user_service.get_user(order.user_id)
        if user.is_deactivated:
            raise UserDeactivatedError()
        self._repo.cancel(order_id)
```

### 正确示例
```python
# 方案 1: 依赖倒置 -- 引入接口
# shared/interfaces.py
from abc import ABC, abstractmethod

class UserQueryInterface(ABC):
    @abstractmethod
    def get_user(self, user_id: int) -> User: ...

    @abstractmethod
    def is_active(self, user_id: int) -> bool: ...

# user/service.py -- 实现接口
class UserService(UserQueryInterface):
    def get_user(self, user_id: int) -> User:
        return self._repo.get(user_id)

    def is_active(self, user_id: int) -> bool:
        user = self._repo.get(user_id)
        return user is not None and not user.is_deactivated

# order/service.py -- 依赖接口而非具体实现
class OrderService:
    def __init__(self, user_query: UserQueryInterface):  # 注入接口
        self._user_query = user_query

    def cancel_order(self, order_id: int):
        order = self._repo.get(order_id)
        if not self._user_query.is_active(order.user_id):
            raise UserDeactivatedError()
        self._repo.cancel(order_id)
```

```python
# 方案 2: 事件解耦
# user/service.py
class UserService:
    def deactivate_user(self, user_id: int):
        self._repo.deactivate(user_id)
        self._event_bus.publish(UserDeactivatedEvent(user_id=user_id))

# order/event_handler.py -- 订阅事件，无 import 依赖
class OrderEventHandler:
    def on_user_deactivated(self, event: UserDeactivatedEvent):
        self._order_repo.cancel_pending_orders(event.user_id)
```

### 检测方法
- Python: `import-linter` 配置分层规则，CI 中检查。
- `pydeps` / `madge` (JS) 生成依赖图，检查是否有环。
- 运行时出现 `ImportError: cannot import name` 循环导入错误。
- 微服务场景：绘制服务调用图，检查是否有双向调用。

### 修复步骤
1. 使用 `pydeps` / `madge` 生成依赖图，识别所有环。
2. 对于直接循环：引入接口层（依赖倒置原则）。
3. 对于间接循环：考虑事件驱动解耦或合并紧耦合的模块。
4. 在 CI 中配置 `import-linter` 规则，阻止新的循环依赖进入。
5. 定期检查依赖图，确保架构约束持续有效。

### Agent Checklist
- [ ] 无循环 import（`import-linter` CI 检查通过）
- [ ] 模块间依赖方向单一（上层 -> 下层）
- [ ] 紧耦合的模块通过接口解耦
- [ ] 服务间无双向同步调用

---

## 6. 无边界上下文（Missing Bounded Context）

### 描述
不同业务域共用相同的模型定义，导致一个模型承担多个业务域的含义。例如 `User` 模型同时用于认证、订单、支付、社交，修改认证逻辑可能意外影响订单模块。

### 错误示例
```python
# 一个 User 模型被所有业务域共用
class User(Base):
    __tablename__ = "users"
    id = Column(Integer, primary_key=True)
    # 认证域
    username = Column(String(50))
    password_hash = Column(String(255))
    last_login = Column(DateTime)
    mfa_secret = Column(String(100))
    # 订单域
    shipping_address = Column(String(500))
    default_payment_method = Column(String(50))
    # 社交域
    bio = Column(Text)
    avatar_url = Column(String(500))
    follower_count = Column(Integer)
    # 营销域
    newsletter_opt_in = Column(Boolean)
    referral_code = Column(String(20))
    loyalty_points = Column(Integer)

# 所有服务都直接操作这个巨型 User 表
# auth_service.py
def login(username, password):
    user = session.query(User).filter_by(username=username).first()
    # 加载了 20+ 列，只需要 2 列

# social_service.py
def get_profile(user_id):
    user = session.query(User).get(user_id)
    # 修改这个查询可能影响认证服务的性能
```

### 正确示例
```python
# 按边界上下文拆分模型

# auth/models.py -- 认证上下文
class AuthUser(Base):
    __tablename__ = "auth_users"
    id = Column(Integer, primary_key=True)
    username = Column(String(50), unique=True, nullable=False)
    password_hash = Column(String(255), nullable=False)
    last_login = Column(DateTime)
    mfa_secret = Column(String(100))

# orders/models.py -- 订单上下文
class Customer(Base):
    """订单域中的用户表示 -- 只包含订单相关的属性"""
    __tablename__ = "order_customers"
    id = Column(Integer, primary_key=True)
    user_id = Column(Integer, index=True)  # 关联到 auth_users.id
    shipping_address = Column(String(500))
    default_payment_method = Column(String(50))

# social/models.py -- 社交上下文
class UserProfile(Base):
    """社交域中的用户表示 -- 只包含社交相关的属性"""
    __tablename__ = "social_profiles"
    id = Column(Integer, primary_key=True)
    user_id = Column(Integer, unique=True, index=True)
    display_name = Column(String(100))
    bio = Column(Text)
    avatar_url = Column(String(500))
    follower_count = Column(Integer, default=0)

# 上下文间通过 user_id 关联，不直接 import 对方的模型
# 跨上下文的数据同步通过事件实现
class UserRegisteredEventHandler:
    """当用户注册时，在其他上下文中创建对应的记录"""
    def handle(self, event: UserRegisteredEvent):
        # 在订单域创建 Customer
        customer_repo.create(Customer(user_id=event.user_id))
        # 在社交域创建 Profile
        profile_repo.create(UserProfile(user_id=event.user_id))
```

### 检测方法
- 单个模型类的字段数 > 20。
- 模型类包含属于不同业务域的字段（认证 + 支付 + 社交）。
- 修改一个业务域的逻辑需要修改另一个业务域的代码。
- 多个团队频繁在同一个模型文件上产生合并冲突。

### 修复步骤
1. 进行事件风暴（Event Storming），识别业务域和边界上下文。
2. 为每个边界上下文定义独立的模型，即使它们代表同一个实体。
3. 上下文间通过 ID 关联，不直接 import 对方的模型。
4. 使用领域事件同步跨上下文的数据变更。
5. 每个上下文可以有自己的数据库 Schema 或独立数据库。

### Agent Checklist
- [ ] 每个业务域有独立的模型定义
- [ ] 单个模型字段数 <= 15
- [ ] 模块间不直接 import 对方的模型类
- [ ] 跨域数据同步使用事件机制
- [ ] 不同团队不会在同一个模型文件上冲突

---

## 全局 Agent Checklist

| 检查项 | 阈值 | 工具 |
|--------|------|------|
| 服务间同步调用链深度 | <= 2 | 调用链追踪 / APM |
| 共享数据库 | 0 个 | 架构图审查 |
| 单文件行数 | <= 500 | `wc -l` |
| 循环依赖 | 0 个 | `import-linter` / `pydeps` |
| 单模型字段数 | <= 15 | Code Review |
| CRUD 修改文件数 | <= 3 | git diff 统计 |
| 万能工具文件 | 0 个 | `grep -r "utils.py"` |
