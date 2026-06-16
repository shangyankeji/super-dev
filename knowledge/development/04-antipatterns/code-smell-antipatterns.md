---
id: code-smell-antipatterns
title: 代码坏味道反模式指南
domain: development
category: 04-antipatterns
difficulty: intermediate
tags: [antipatterns, code, copy-paste, development, method, naming, nesting, numbers]
quality_score: 70
last_updated: 2026-06-15
---
# 代码坏味道反模式指南

> 适用范围：Python / JavaScript / TypeScript / Go / Java / Rust
> 约束级别：SHALL（必须在 Code Review 阶段拦截）

---

## 1. God Object（上帝对象）

### 描述
一个类承担了过多的职责，集中了大量属性和方法，导致修改任何功能都需要修改该类。违反单一职责原则（SRP），是系统耦合度攀升的最常见根因。

### 错误示例
```python
class OrderService:
    """一个类管理了订单、库存、支付、通知、日志、缓存。"""

    def __init__(self):
        self.db = Database()
        self.cache = Redis()
        self.mailer = EmailClient()
        self.sms = SmsGateway()
        self.logger = Logger()
        self.inventory = {}

    def create_order(self, user_id, items):
        # 校验库存
        for item in items:
            stock = self.db.query(f"SELECT stock FROM products WHERE id={item['id']}")
            if stock < item['qty']:
                self.logger.error(f"库存不足: {item['id']}")
                return None
            self.inventory[item['id']] = stock - item['qty']

        # 创建订单
        order_id = self.db.insert("orders", {"user_id": user_id, "items": items})

        # 扣减库存
        for item in items:
            self.db.update("products", {"stock": self.inventory[item['id']]})

        # 发送通知
        user = self.db.query(f"SELECT * FROM users WHERE id={user_id}")
        self.mailer.send(user['email'], f"订单 {order_id} 已创建")
        self.sms.send(user['phone'], f"订单 {order_id} 已创建")

        # 清缓存
        self.cache.delete(f"user_orders:{user_id}")
        self.cache.delete(f"product_stock:{items[0]['id']}")

        return order_id
```

### 正确示例
```python
class OrderService:
    """只负责订单编排，将具体职责委托给专门的服务。"""

    def __init__(
        self,
        inventory_service: InventoryService,
        payment_service: PaymentService,
        notification_service: NotificationService,
        order_repository: OrderRepository,
    ):
        self._inventory = inventory_service
        self._payment = payment_service
        self._notification = notification_service
        self._repo = order_repository

    def create_order(self, user_id: str, items: list[OrderItem]) -> Order:
        self._inventory.reserve(items)
        try:
            order = self._repo.create(user_id=user_id, items=items)
            self._notification.send_order_created(order)
            return order
        except Exception:
            self._inventory.release(items)
            raise
```

### 检测方法
- 类的行数超过 300 行、方法数超过 15 个、依赖注入超过 5 个。
- 静态分析：`radon cc` (Python)、`eslint complexity` (JS/TS) 报告圈复杂度 > 20。
- 文件变更频率在 git log 中位居前 3 且关联 PR 跨多个业务域。

### 修复步骤
1. 列出类的所有公开方法，按业务域分组。
2. 为每个业务域创建独立的服务类。
3. 将原类重构为编排者（Orchestrator），仅调用各服务的接口。
4. 为每个新服务编写独立的单元测试。
5. 运行全量回归测试确认行为不变。

### Agent Checklist
- [ ] 单个类方法数 <= 15
- [ ] 单个类行数 <= 300
- [ ] 构造函数依赖注入 <= 5 个
- [ ] 圈复杂度 <= 15
- [ ] 变更该文件不影响无关业务域

---

## 2. 过长方法（Long Method）

### 描述
单个方法超过 50 行，包含多层逻辑（校验、业务处理、持久化、通知），难以测试、难以复用、难以理解。

### 错误示例
```python
def process_payment(order_id, payment_info):
    # 80+ 行方法：校验 -> 查订单 -> 查用户 -> 风控 -> 扣款 -> 更新状态 -> 发通知 -> 写日志
    order = db.get_order(order_id)
    if not order:
        log.error("订单不存在")
        return {"error": "ORDER_NOT_FOUND"}
    if order['status'] != 'pending':
        log.error("订单状态不允许支付")
        return {"error": "INVALID_STATUS"}
    user = db.get_user(order['user_id'])
    if not user:
        return {"error": "USER_NOT_FOUND"}
    if user['risk_level'] > 3:
        log.warning("高风险用户")
        return {"error": "RISK_BLOCKED"}
    # ... 继续 60 行扣款、更新、通知逻辑 ...
```

### 正确示例
```python
def process_payment(order_id: str, payment_info: PaymentInfo) -> PaymentResult:
    order = _validate_order(order_id)
    user = _validate_user(order.user_id)
    _check_risk(user)
    transaction = _execute_charge(order, payment_info)
    _update_order_status(order, transaction)
    _send_payment_notification(order, user)
    return PaymentResult(transaction_id=transaction.id, status="success")


def _validate_order(order_id: str) -> Order:
    order = order_repo.get(order_id)
    if not order:
        raise OrderNotFoundError(order_id)
    if order.status != OrderStatus.PENDING:
        raise InvalidOrderStatusError(order.status)
    return order
```

### 检测方法
- 方法行数 > 50 行（`wc -l` 或 IDE 行数提示）。
- 方法内出现 3 个以上不同层次的操作（I/O、计算、副作用混合）。
- 单个方法的单元测试需要 mock 超过 3 个外部依赖。

### 修复步骤
1. 识别方法中的逻辑段落（通常以空行或注释分隔）。
2. 将每个段落提取为独立的私有方法，命名需体现业务意图。
3. 原方法变为高层编排，只包含方法调用序列。
4. 为每个提取出的方法编写独立单元测试。

### Agent Checklist
- [ ] 单个方法行数 <= 50
- [ ] 方法内 mock 依赖 <= 3 个
- [ ] 方法名准确描述其唯一职责
- [ ] 无注释分隔的逻辑段落（应已提取为独立方法）

---

## 3. 过深嵌套（Deep Nesting）

### 描述
条件判断、循环嵌套超过 3 层，形成箭头型代码（Arrow Code），严重降低可读性和可测试性。

### 错误示例
```python
def get_discount(user, order):
    if user is not None:
        if user.is_active:
            if order is not None:
                if order.total > 100:
                    if user.vip_level >= 2:
                        if order.coupon:
                            if order.coupon.is_valid():
                                return order.total * 0.7
                            else:
                                return order.total * 0.85
                        else:
                            return order.total * 0.85
                    else:
                        return order.total * 0.95
                else:
                    return order.total
            else:
                return 0
        else:
            return 0
    else:
        return 0
```

### 正确示例
```python
def get_discount(user: User | None, order: Order | None) -> Decimal:
    if not user or not user.is_active:
        return Decimal(0)
    if not order:
        return Decimal(0)
    if order.total <= 100:
        return order.total

    base_rate = _vip_discount_rate(user.vip_level)
    coupon_rate = _coupon_discount_rate(order.coupon)
    return order.total * min(base_rate, coupon_rate)


def _vip_discount_rate(vip_level: int) -> Decimal:
    return Decimal("0.85") if vip_level >= 2 else Decimal("0.95")


def _coupon_discount_rate(coupon: Coupon | None) -> Decimal:
    if coupon and coupon.is_valid():
        return Decimal("0.70")
    return Decimal("1.0")
```

### 检测方法
- 缩进深度 > 3 层（使用 `ruff` 的 `C901` 规则或 `pylint` 的 `too-many-nested-blocks`）。
- 代码呈箭头形状（左侧缩进逐步增大后再逐步回收）。
- ESLint 规则 `max-depth` 设为 3。

### 修复步骤
1. 使用 Guard Clause（卫语句）提前返回，消除外层条件。
2. 将嵌套内部逻辑提取为独立函数。
3. 使用策略模式或查找表替代多层 if-else。
4. 确保每个分支都有对应的测试用例。

### Agent Checklist
- [ ] 嵌套深度 <= 3 层
- [ ] 所有失败路径使用卫语句提前返回
- [ ] 无箭头型代码
- [ ] 每个分支路径有测试覆盖

---

## 4. 魔法数字 / 魔法字符串（Magic Numbers / Strings）

### 描述
代码中直接使用字面量数字或字符串，不解释其业务含义，导致维护时无法理解意图，修改时容易遗漏。

### 错误示例
```python
def calculate_shipping(weight, distance):
    if weight > 30:
        return distance * 0.15 + 25.0
    elif distance > 500:
        return distance * 0.08 + 10.0
    else:
        return 5.0

def check_user_status(user):
    if user['status'] == 3:       # 3 是什么状态？
        send_email(user, 'reactivation')
    if user['role'] == 'adm':     # 为什么是 adm 不是 admin？
        grant_access(user, 7)     # 7 天？7 级？
```

### 正确示例
```python
# constants.py
MAX_STANDARD_WEIGHT_KG = 30
LONG_DISTANCE_THRESHOLD_KM = 500
HEAVY_RATE_PER_KM = Decimal("0.15")
HEAVY_BASE_FEE = Decimal("25.0")
STANDARD_RATE_PER_KM = Decimal("0.08")
STANDARD_BASE_FEE = Decimal("10.0")
DEFAULT_SHIPPING_FEE = Decimal("5.0")

class UserStatus(IntEnum):
    ACTIVE = 1
    SUSPENDED = 2
    DEACTIVATED = 3

class UserRole(str, Enum):
    ADMIN = "admin"
    MEMBER = "member"

REACTIVATION_GRACE_DAYS = 7

def calculate_shipping(weight_kg: Decimal, distance_km: Decimal) -> Decimal:
    if weight_kg > MAX_STANDARD_WEIGHT_KG:
        return distance_km * HEAVY_RATE_PER_KM + HEAVY_BASE_FEE
    if distance_km > LONG_DISTANCE_THRESHOLD_KM:
        return distance_km * STANDARD_RATE_PER_KM + STANDARD_BASE_FEE
    return DEFAULT_SHIPPING_FEE
```

### 检测方法
- `ruff` 规则或 `pylint` 的 `magic-value-comparison`。
- ESLint `no-magic-numbers` 规则。
- Code Review 中搜索未命名的数字字面量（排除 0、1、-1 等常见哨兵值）。

### 修复步骤
1. 搜索代码中所有数字和字符串字面量。
2. 为每个字面量确定其业务含义并命名为常量。
3. 将相关常量集中到 `constants.py` 或专门的枚举类中。
4. 全局替换字面量为常量引用。
5. 运行测试确认行为不变。

### Agent Checklist
- [ ] 无未命名的数字字面量（0、1、-1 除外）
- [ ] 无硬编码的业务状态字符串
- [ ] 所有业务枚举使用 Enum 类定义
- [ ] 常量集中管理且命名体现业务语义

---

## 5. Copy-Paste 编程（Duplicated Code）

### 描述
通过复制粘贴实现功能复用，导致相同逻辑散布在多处。修复 Bug 时只改了一处，其他副本仍然存在问题。是技术债增长最快的来源之一。

### 错误示例
```python
# user_api.py
def get_user(user_id):
    try:
        resp = requests.get(f"{API_BASE}/users/{user_id}", timeout=5)
        resp.raise_for_status()
        data = resp.json()
        if 'error' in data:
            logger.error(f"API error: {data['error']}")
            return None
        return data
    except requests.Timeout:
        logger.error(f"Timeout fetching user {user_id}")
        return None
    except requests.RequestException as e:
        logger.error(f"Request failed: {e}")
        return None

# order_api.py  -- 几乎完全相同的代码
def get_order(order_id):
    try:
        resp = requests.get(f"{API_BASE}/orders/{order_id}", timeout=5)
        resp.raise_for_status()
        data = resp.json()
        if 'error' in data:
            logger.error(f"API error: {data['error']}")
            return None
        return data
    except requests.Timeout:
        logger.error(f"Timeout fetching order {order_id}")
        return None
    except requests.RequestException as e:
        logger.error(f"Request failed: {e}")
        return None
```

### 正确示例
```python
# http_client.py
class ApiClient:
    def __init__(self, base_url: str, timeout: int = 5):
        self._session = requests.Session()
        self._base_url = base_url
        self._timeout = timeout

    def get(self, path: str) -> dict | None:
        try:
            resp = self._session.get(
                f"{self._base_url}{path}", timeout=self._timeout
            )
            resp.raise_for_status()
            data = resp.json()
            if "error" in data:
                logger.error("API error on %s: %s", path, data["error"])
                return None
            return data
        except requests.Timeout:
            logger.error("Timeout on %s", path)
            return None
        except requests.RequestException as e:
            logger.error("Request failed on %s: %s", path, e)
            return None

# user_api.py
def get_user(user_id: str) -> dict | None:
    return api_client.get(f"/users/{user_id}")

# order_api.py
def get_order(order_id: str) -> dict | None:
    return api_client.get(f"/orders/{order_id}")
```

### 检测方法
- `jscpd`（JavaScript/Python 通用重复代码检测器），阈值设为 5%。
- `pylint` 的 `duplicate-code` (R0801) 检查。
- Code Review 中搜索相似的 try-except 块、相似的 CRUD 函数签名。

### 修复步骤
1. 使用 `jscpd` 或 IDE 的 "Find Duplicates" 定位重复块。
2. 分析重复代码的差异点（通常只有 URL、参数名不同）。
3. 将共同逻辑抽取为通用函数或基类方法，差异点作为参数传入。
4. 替换所有副本为对通用函数的调用。
5. 运行全量测试确认行为一致。

### Agent Checklist
- [ ] `jscpd` 重复率 < 5%
- [ ] 无结构相同但参数不同的函数对（应抽取公共方法）
- [ ] CRUD 操作使用统一的 Repository / Client 基类
- [ ] 修复 Bug 时无需修改多处相同代码

---

## 6. 过早优化（Premature Optimization）

### 描述
在没有性能数据支撑的情况下引入复杂的优化手段（自定义缓存、手写数据结构、内联汇编），牺牲了可读性和可维护性，而实际性能瓶颈往往不在此处。

### 错误示例
```python
# "为了性能"手写了 LRU 缓存和位运算优化
class HandRolledCache:
    def __init__(self, capacity):
        self._capacity = capacity
        self._map = {}
        self._order = []  # 手动维护访问顺序

    def get(self, key):
        if key in self._map:
            self._order.remove(key)  # O(n) 操作，实际上更慢
            self._order.append(key)
            return self._map[key]
        return None

    def put(self, key, value):
        if len(self._map) >= self._capacity:
            oldest = self._order.pop(0)  # O(n) 操作
            del self._map[oldest]
        self._map[key] = value
        self._order.append(key)

def compute_tax(amount):
    # "位运算更快" -- 实际上编译器已经做了这个优化
    return (amount * 13) >> 7  # 不等于 * 0.1，结果错误
```

### 正确示例
```python
from functools import lru_cache

@lru_cache(maxsize=1024)
def get_product_details(product_id: str) -> Product:
    """使用标准库 LRU 缓存，经过充分测试，O(1) 操作。"""
    return product_repo.get(product_id)

def compute_tax(amount: Decimal, rate: Decimal = Decimal("0.13")) -> Decimal:
    """清晰的十进制运算，无精度损失。"""
    return (amount * rate).quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)
```

### 检测方法
- Code Review 中发现自定义实现了标准库已有的功能（缓存、排序、连接池等）。
- 优化代码没有附带基准测试结果或性能分析报告。
- 代码注释中出现 "为了性能" 但无对应的 profiling 数据。

### 修复步骤
1. 使用 `cProfile` / `py-spy` / Chrome DevTools 定位实际瓶颈。
2. 删除无 profiling 数据支撑的自定义优化代码。
3. 替换为标准库或成熟第三方库的实现。
4. 对真正的瓶颈进行有数据支撑的优化，并记录基准测试结果。

### Agent Checklist
- [ ] 无自定义实现的标准库功能（缓存、排序、序列化等）
- [ ] 所有性能优化附带 profiling 数据或基准测试
- [ ] 使用 Decimal 处理金融计算，不用浮点位运算
- [ ] 优化代码可读性未明显下降

---

## 7. 无意义命名（Poor Naming）

### 描述
变量、函数、类使用无业务语义的名称（`data`、`info`、`temp`、`x`、`handler`、`process`），导致代码阅读者必须追踪上下文才能理解意图。

### 错误示例
```python
def process(data):
    result = []
    for item in data:
        if item['t'] == 1:
            tmp = item['v'] * 1.1
            if tmp > item['l']:
                result.append({'id': item['id'], 'val': tmp, 'flag': True})
            else:
                result.append({'id': item['id'], 'val': item['v'], 'flag': False})
    return result

class Manager:
    def handle(self, info):
        d = self.get_data(info)
        r = self.do_stuff(d)
        return r
```

### 正确示例
```python
def apply_price_adjustments(products: list[Product]) -> list[PriceResult]:
    results = []
    for product in products:
        if product.category == ProductCategory.TAXABLE:
            adjusted_price = product.base_price * TAX_MULTIPLIER
            exceeds_limit = adjusted_price > product.price_ceiling
            results.append(
                PriceResult(
                    product_id=product.id,
                    final_price=adjusted_price if exceeds_limit else product.base_price,
                    tax_applied=exceeds_limit,
                )
            )
    return results

class PricingService:
    def calculate_order_total(self, order: Order) -> OrderTotal:
        line_items = self._compute_line_totals(order)
        discount = self._apply_promotions(order, line_items)
        return OrderTotal(items=line_items, discount=discount)
```

### 检测方法
- 变量名 <= 2 个字符（循环变量 `i`、`j` 除外）。
- 函数名为泛型词：`process`、`handle`、`do`、`run`、`manage`、`get_data`。
- 类名为泛型词：`Manager`、`Handler`、`Helper`、`Utils`（无业务前缀）。
- `pylint` 的 `invalid-name` 规则，ESLint 的 `id-length` 规则。

### 修复步骤
1. 为每个变量/函数/类确定其业务含义。
2. 使用 "名词 + 动词" 或 "形容词 + 名词" 命名法。
3. 函数名以动词开头，描述其行为（`calculate_tax`，而非 `process`）。
4. 类名使用具体业务名词（`PricingService`，而非 `Manager`）。
5. 在 PR 描述中注明命名变更的理由。

### Agent Checklist
- [ ] 无 <= 2 字符的变量名（循环索引除外）
- [ ] 无泛型函数名（process / handle / do / manage）
- [ ] 无泛型类名（Manager / Handler / Helper / Utils 无业务前缀）
- [ ] 命名能让不熟悉项目的人理解其业务意图

---

## 全局 Agent Checklist

| 检查项 | 阈值 | 工具 |
|--------|------|------|
| 单类行数 | <= 300 | `wc -l` / IDE |
| 单类方法数 | <= 15 | `radon` / `pylint` |
| 单方法行数 | <= 50 | `wc -l` / IDE |
| 圈复杂度 | <= 15 | `radon cc` / `eslint complexity` |
| 嵌套深度 | <= 3 | `pylint` / `eslint max-depth` |
| 代码重复率 | < 5% | `jscpd` |
| 魔法数字 | 0 个 | `ruff` / `eslint no-magic-numbers` |
| 泛型命名 | 0 个 | Code Review / `pylint` |
