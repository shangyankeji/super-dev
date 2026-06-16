---
id: python-design-patterns
title: Python设计模式完整知识体系
domain: development
category: 01-standards
difficulty: intermediate
tags: [agent, checklist, design, development, patterns, python, python特有模式, 创建型模式]
quality_score: 70
last_updated: 2026-06-15
---
# Python设计模式完整知识体系

## 概述

设计模式是解决反复出现的软件设计问题的可复用方案。Python因其动态类型、一等函数、鸭子类型和丰富的内置协议，使得许多传统GoF模式在Python中有更简洁、更惯用的实现方式。

### Python设计模式的特殊性

- **一等函数替代策略/命令类层次**: Python函数是一等公民，许多需要单方法接口的模式可直接用函数或callable对象实现
- **鸭子类型替代接口**: 无需显式接口声明，依赖协议（Protocol）和结构化子类型
- **装饰器语法糖**: Python内置的`@decorator`语法让装饰器模式成为语言特性
- **元类和描述符**: 提供Java/C++中不存在的元编程能力
- **上下文管理器协议**: `with`语句为资源管理提供标准化模式
- **dataclass和NamedTuple**: 极大简化值对象和数据传输对象的创建

### Python惯用法 vs 传统GoF模式

| GoF模式 | Java实现 | Python惯用实现 |
|---------|----------|---------------|
| 策略模式 | 接口 + 多个实现类 | 函数/callable传参 |
| 单例模式 | 双重检查锁 | 模块级变量/`__new__` |
| 迭代器模式 | Iterator接口 | `__iter__`/`__next__`/生成器 |
| 模板方法 | 抽象类 + 子类覆写 | 函数参数/hooks |
| 观察者模式 | Listener接口 | 信号/回调函数列表 |
| 装饰器模式 | 接口包装类 | `@decorator`语法 |
| 命令模式 | Command接口 + 实现 | callable对象/闭包 |

---

## 创建型模式

### 1. 工厂方法模式 (Factory Method)

**问题描述**: 需要根据不同条件创建不同类型的对象，但调用方不应该依赖具体类。

**Python实现**:

```python
from typing import Protocol, runtime_checkable
from dataclasses import dataclass

@runtime_checkable
class Notification(Protocol):
    def send(self, message: str) -> bool: ...

@dataclass
class EmailNotification:
    recipient: str

    def send(self, message: str) -> bool:
        print(f"Email to {self.recipient}: {message}")
        return True

@dataclass
class SMSNotification:
    phone: str

    def send(self, message: str) -> bool:
        print(f"SMS to {self.phone}: {message}")
        return True

@dataclass
class SlackNotification:
    channel: str

    def send(self, message: str) -> bool:
        print(f"Slack #{self.channel}: {message}")
        return True

# Python惯用：用字典注册表替代工厂子类层次
_notification_registry: dict[str, type] = {
    "email": EmailNotification,
    "sms": SMSNotification,
    "slack": SlackNotification,
}

def create_notification(channel: str, **kwargs) -> Notification:
    """工厂函数：根据渠道类型创建通知对象"""
    cls = _notification_registry.get(channel)
    if cls is None:
        raise ValueError(f"Unknown notification channel: {channel}")
    return cls(**kwargs)

def register_notification(channel: str, cls: type) -> None:
    """运行时注册新的通知类型，支持插件式扩展"""
    _notification_registry[channel] = cls

# 使用
notif = create_notification("email", recipient="user@example.com")
notif.send("Your order has shipped")
```

**使用场景**: 插件系统、多渠道通知、数据库驱动选择、序列化格式选择。

**注意事项**:
- 优先使用字典注册表而非类继承层次
- 工厂函数比工厂类更Pythonic
- 用`Protocol`定义产品接口而非抽象基类（除非需要默认实现）

### 2. 抽象工厂模式 (Abstract Factory)

**问题描述**: 需要创建一组相关或相互依赖的对象族，且不指定它们的具体类。

**Python实现**:

```python
from __future__ import annotations
from typing import Protocol
from dataclasses import dataclass, field

class Button(Protocol):
    def render(self) -> str: ...

class TextInput(Protocol):
    def render(self) -> str: ...

class Theme(Protocol):
    def create_button(self, label: str) -> Button: ...
    def create_text_input(self, placeholder: str) -> TextInput: ...

@dataclass
class MaterialButton:
    label: str
    def render(self) -> str:
        return f'<button class="md-btn">{self.label}</button>'

@dataclass
class MaterialTextInput:
    placeholder: str
    def render(self) -> str:
        return f'<input class="md-input" placeholder="{self.placeholder}"/>'

@dataclass
class AntDesignButton:
    label: str
    def render(self) -> str:
        return f'<button class="ant-btn">{self.label}</button>'

@dataclass
class AntDesignTextInput:
    placeholder: str
    def render(self) -> str:
        return f'<input class="ant-input" placeholder="{self.placeholder}"/>'

class MaterialTheme:
    def create_button(self, label: str) -> Button:
        return MaterialButton(label)
    def create_text_input(self, placeholder: str) -> TextInput:
        return MaterialTextInput(placeholder)

class AntDesignTheme:
    def create_button(self, label: str) -> Button:
        return AntDesignButton(label)
    def create_text_input(self, placeholder: str) -> TextInput:
        return AntDesignTextInput(placeholder)

def build_form(theme: Theme) -> list[str]:
    """使用抽象工厂构建一致的UI表单"""
    btn = theme.create_button("Submit")
    inp = theme.create_text_input("Enter your name")
    return [inp.render(), btn.render()]
```

**使用场景**: UI主题系统、跨数据库ORM适配、多云基础设施抽象。

**注意事项**:
- Python中抽象工厂的使用频率低于Java，因为鸭子类型天然支持多态
- 考虑是否真的需要对象族一致性，否则简单工厂函数即可

### 3. 建造者模式 (Builder)

**问题描述**: 构造复杂对象需要多个步骤，且不同表示可能需要相同的构造流程。

**Python实现**:

```python
from __future__ import annotations
from dataclasses import dataclass, field
from typing import Self

@dataclass
class HTTPRequest:
    method: str = "GET"
    url: str = ""
    headers: dict[str, str] = field(default_factory=dict)
    query_params: dict[str, str] = field(default_factory=dict)
    body: str | bytes | None = None
    timeout: float = 30.0
    retries: int = 0

    class Builder:
        """链式建造者，利用Python的Self类型实现流畅接口"""
        def __init__(self) -> None:
            self._method = "GET"
            self._url = ""
            self._headers: dict[str, str] = {}
            self._query_params: dict[str, str] = {}
            self._body: str | bytes | None = None
            self._timeout: float = 30.0
            self._retries: int = 0

        def method(self, method: str) -> Self:
            self._method = method
            return self

        def url(self, url: str) -> Self:
            self._url = url
            return self

        def header(self, key: str, value: str) -> Self:
            self._headers[key] = value
            return self

        def query(self, key: str, value: str) -> Self:
            self._query_params[key] = value
            return self

        def body(self, body: str | bytes) -> Self:
            self._body = body
            return self

        def timeout(self, seconds: float) -> Self:
            self._timeout = seconds
            return self

        def retries(self, count: int) -> Self:
            self._retries = count
            return self

        def build(self) -> HTTPRequest:
            if not self._url:
                raise ValueError("URL is required")
            return HTTPRequest(
                method=self._method,
                url=self._url,
                headers=self._headers,
                query_params=self._query_params,
                body=self._body,
                timeout=self._timeout,
                retries=self._retries,
            )

# 使用
request = (
    HTTPRequest.Builder()
    .method("POST")
    .url("https://api.example.com/orders")
    .header("Content-Type", "application/json")
    .header("Authorization", "Bearer token123")
    .body('{"item": "widget", "qty": 3}')
    .timeout(10.0)
    .retries(3)
    .build()
)
```

**使用场景**: 复杂配置对象、SQL/查询构建器、HTTP请求构建、测试数据工厂。

**注意事项**:
- Python更常使用`dataclass` + 关键字参数替代简单的Builder
- 仅当构造过程有验证逻辑或步骤顺序约束时才值得用Builder
- `Self`类型（Python 3.11+）使链式调用具有正确的类型推导

### 4. 单例模式 (Singleton)

**问题描述**: 确保一个类只有一个实例，并提供全局访问点。

**Python实现**:

```python
# 方式一：模块级变量（最Pythonic，推荐）
# config.py
class _AppConfig:
    def __init__(self) -> None:
        self._settings: dict[str, object] = {}
        self._loaded = False

    def load(self, path: str) -> None:
        # 从文件加载配置
        self._loaded = True

    def get(self, key: str, default: object = None) -> object:
        return self._settings.get(key, default)

# 模块级单例：Python模块天然只初始化一次
app_config = _AppConfig()


# 方式二：__new__控制实例化（需要类级别单例时）
class DatabasePool:
    _instance: "DatabasePool | None" = None

    def __new__(cls) -> "DatabasePool":
        if cls._instance is None:
            cls._instance = super().__new__(cls)
            cls._instance._initialized = False
        return cls._instance

    def __init__(self) -> None:
        if self._initialized:
            return
        self._initialized = True
        self._connections: list = []
        self._max_size = 10

    def get_connection(self):
        # 从池中获取连接
        pass


# 方式三：元类（适用于框架级控制）
class SingletonMeta(type):
    _instances: dict[type, object] = {}

    def __call__(cls, *args, **kwargs):
        if cls not in cls._instances:
            cls._instances[cls] = super().__call__(*args, **kwargs)
        return cls._instances[cls]

class CacheManager(metaclass=SingletonMeta):
    def __init__(self) -> None:
        self._store: dict[str, object] = {}

    def get(self, key: str) -> object | None:
        return self._store.get(key)

    def set(self, key: str, value: object) -> None:
        self._store[key] = value
```

**使用场景**: 配置管理、连接池、缓存管理器、日志管理。

**注意事项**:
- **优先使用模块级变量**，这是最Pythonic的单例实现
- 单例模式使测试困难（全局状态），考虑依赖注入替代
- 线程安全场景需加锁或使用`threading.Lock`
- 避免过度使用——大多数"需要单例"的场景实际上可以通过依赖注入解决

### 5. 原型模式 (Prototype)

**问题描述**: 通过复制现有对象来创建新对象，避免重复的初始化开销。

**Python实现**:

```python
import copy
from dataclasses import dataclass, field

@dataclass
class GameCharacter:
    name: str
    health: int
    attack: int
    defense: int
    skills: list[str] = field(default_factory=list)
    inventory: dict[str, int] = field(default_factory=dict)

    def clone(self, **overrides) -> "GameCharacter":
        """深拷贝并允许覆盖特定字段"""
        cloned = copy.deepcopy(self)
        for key, value in overrides.items():
            if not hasattr(cloned, key):
                raise AttributeError(f"GameCharacter has no attribute '{key}'")
            setattr(cloned, key, value)
        return cloned

# 预设原型注册表
_character_prototypes: dict[str, GameCharacter] = {
    "warrior": GameCharacter(
        name="Warrior",
        health=150,
        attack=30,
        defense=25,
        skills=["slash", "shield_bash"],
        inventory={"health_potion": 3},
    ),
    "mage": GameCharacter(
        name="Mage",
        health=80,
        attack=50,
        defense=10,
        skills=["fireball", "ice_shield"],
        inventory={"mana_potion": 5},
    ),
}

def create_character(archetype: str, name: str) -> GameCharacter:
    proto = _character_prototypes.get(archetype)
    if proto is None:
        raise ValueError(f"Unknown archetype: {archetype}")
    return proto.clone(name=name)

# 使用
hero = create_character("warrior", name="Arthas")
```

**使用场景**: 配置模板、游戏对象预设、测试数据快速构造、文档模板。

**注意事项**:
- Python的`copy.deepcopy`是原型模式的天然支持
- 注意深拷贝与浅拷贝的区别，可变嵌套对象必须深拷贝
- `dataclass`的`replace`（Python 3.13+）提供了更安全的浅拷贝

---

## 结构型模式

### 1. 适配器模式 (Adapter)

**问题描述**: 将一个接口转换为客户端期望的另一种接口，使不兼容的类可以协作。

**Python实现**:

```python
from typing import Protocol
import json
import xml.etree.ElementTree as ET

class DataParser(Protocol):
    def parse(self, raw: str) -> dict: ...

class JSONParser:
    """原生支持的JSON解析器"""
    def parse(self, raw: str) -> dict:
        return json.loads(raw)

class LegacyXMLService:
    """遗留系统，接口不符合DataParser协议"""
    def read_xml(self, xml_string: str) -> ET.Element:
        return ET.fromstring(xml_string)

    def extract_data(self, element: ET.Element) -> dict[str, str]:
        return {child.tag: (child.text or "") for child in element}

class XMLParserAdapter:
    """适配器：将LegacyXMLService适配为DataParser接口"""
    def __init__(self, legacy_service: LegacyXMLService | None = None) -> None:
        self._service = legacy_service or LegacyXMLService()

    def parse(self, raw: str) -> dict:
        element = self._service.read_xml(raw)
        return self._service.extract_data(element)

def process_data(parser: DataParser, raw: str) -> dict:
    """客户端代码只依赖DataParser协议"""
    return parser.parse(raw)

# 使用
json_result = process_data(JSONParser(), '{"name": "Alice"}')
xml_result = process_data(XMLParserAdapter(), "<root><name>Alice</name></root>")
```

**使用场景**: 第三方库集成、遗留系统封装、API版本适配、多数据源统一接口。

**注意事项**:
- Python的鸭子类型使适配更轻量——只需实现相同的方法签名
- 优先使用组合（持有被适配对象引用）而非继承

### 2. 装饰器模式 (Decorator)

**问题描述**: 动态地给对象添加额外职责，比子类化更灵活。

**Python实现**:

```python
import functools
import time
import logging
from typing import Callable, ParamSpec, TypeVar

P = ParamSpec("P")
R = TypeVar("R")

logger = logging.getLogger(__name__)

def retry(max_attempts: int = 3, delay: float = 1.0):
    """重试装饰器：失败时自动重试"""
    def decorator(func: Callable[P, R]) -> Callable[P, R]:
        @functools.wraps(func)
        def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
            last_exception: Exception | None = None
            for attempt in range(1, max_attempts + 1):
                try:
                    return func(*args, **kwargs)
                except Exception as exc:
                    last_exception = exc
                    logger.warning(
                        "Attempt %d/%d failed for %s: %s",
                        attempt, max_attempts, func.__name__, exc,
                    )
                    if attempt < max_attempts:
                        time.sleep(delay * attempt)  # 指数退避
            raise last_exception  # type: ignore[misc]
        return wrapper
    return decorator

def timing(func: Callable[P, R]) -> Callable[P, R]:
    """计时装饰器：记录函数执行时间"""
    @functools.wraps(func)
    def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
        start = time.perf_counter()
        try:
            result = func(*args, **kwargs)
            return result
        finally:
            elapsed = time.perf_counter() - start
            logger.info("%s took %.3fs", func.__name__, elapsed)
    return wrapper

def cache_result(ttl_seconds: float = 60.0):
    """带TTL的缓存装饰器"""
    def decorator(func: Callable[P, R]) -> Callable[P, R]:
        _cache: dict[tuple, tuple[float, R]] = {}

        @functools.wraps(func)
        def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
            key = (args, tuple(sorted(kwargs.items())))
            now = time.time()
            if key in _cache:
                cached_time, cached_value = _cache[key]
                if now - cached_time < ttl_seconds:
                    return cached_value
            result = func(*args, **kwargs)
            _cache[key] = (now, result)
            return result
        return wrapper
    return decorator

# 装饰器堆叠：从下往上应用
@timing
@retry(max_attempts=3, delay=0.5)
def fetch_user_data(user_id: int) -> dict:
    """模拟获取用户数据的远程调用"""
    # 实际实现...
    return {"id": user_id, "name": "Alice"}
```

**使用场景**: 日志、缓存、认证检查、重试、限流、参数校验、事务管理。

**注意事项**:
- **必须使用`@functools.wraps`**保留原函数元信息（`__name__`, `__doc__`, `__module__`）
- 装饰器堆叠顺序很重要，从下到上执行（最内层先执行）
- 用`ParamSpec`和`TypeVar`保持正确的类型签名
- 对类方法使用装饰器时注意`self`参数的处理

### 3. 代理模式 (Proxy)

**问题描述**: 控制对另一个对象的访问，提供额外的间接层。

**Python实现**:

```python
from typing import Protocol
import time

class ImageLoader(Protocol):
    def load(self) -> bytes: ...
    @property
    def filename(self) -> str: ...

class RealImageLoader:
    """真实的图片加载器，可能涉及昂贵的IO操作"""
    def __init__(self, filename: str) -> None:
        self._filename = filename
        self._data: bytes | None = None

    @property
    def filename(self) -> str:
        return self._filename

    def load(self) -> bytes:
        if self._data is None:
            print(f"Loading image from disk: {self._filename}")
            time.sleep(0.5)  # 模拟IO延迟
            self._data = b"<image-binary-data>"
        return self._data

class LazyImageProxy:
    """延迟加载代理：直到真正需要数据时才加载"""
    def __init__(self, filename: str) -> None:
        self._filename = filename
        self._real_loader: RealImageLoader | None = None

    @property
    def filename(self) -> str:
        return self._filename

    def _ensure_loaded(self) -> RealImageLoader:
        if self._real_loader is None:
            self._real_loader = RealImageLoader(self._filename)
        return self._real_loader

    def load(self) -> bytes:
        return self._ensure_loaded().load()

class AccessControlProxy:
    """访问控制代理：检查权限后转发请求"""
    def __init__(self, loader: ImageLoader, allowed_roles: set[str]) -> None:
        self._loader = loader
        self._allowed_roles = allowed_roles

    @property
    def filename(self) -> str:
        return self._loader.filename

    def load(self, *, role: str = "viewer") -> bytes:
        if role not in self._allowed_roles:
            raise PermissionError(
                f"Role '{role}' not allowed to load {self.filename}"
            )
        return self._loader.load()
```

**使用场景**: 延迟加载、访问控制、缓存代理、远程代理（RPC）、日志代理。

**注意事项**:
- Python中`__getattr__`可以实现透明代理，自动转发所有未定义的属性访问
- 与装饰器模式的区别：代理控制访问，装饰器增强功能

### 4. 外观模式 (Facade)

**问题描述**: 为复杂子系统提供简化的统一接口。

**Python实现**:

```python
from dataclasses import dataclass

# 复杂的子系统组件
class InventoryService:
    def check_stock(self, product_id: str) -> int:
        return 42  # 模拟库存查询

    def reserve(self, product_id: str, quantity: int) -> str:
        return f"RESERVE-{product_id}-{quantity}"

class PaymentService:
    def charge(self, amount: float, payment_method: str) -> str:
        return f"TXN-{amount}"

    def refund(self, transaction_id: str) -> bool:
        return True

class ShippingService:
    def calculate_cost(self, weight: float, destination: str) -> float:
        return weight * 2.5

    def create_shipment(self, order_id: str, destination: str) -> str:
        return f"SHIP-{order_id}"

class NotificationService:
    def send_email(self, to: str, subject: str, body: str) -> None:
        pass

    def send_sms(self, phone: str, message: str) -> None:
        pass

@dataclass
class OrderResult:
    order_id: str
    transaction_id: str
    shipment_id: str
    total_cost: float

class OrderFacade:
    """订单外观：将复杂的多步骤订单流程封装为简单接口"""
    def __init__(
        self,
        inventory: InventoryService | None = None,
        payment: PaymentService | None = None,
        shipping: ShippingService | None = None,
        notification: NotificationService | None = None,
    ) -> None:
        self._inventory = inventory or InventoryService()
        self._payment = payment or PaymentService()
        self._shipping = shipping or ShippingService()
        self._notification = notification or NotificationService()

    def place_order(
        self,
        product_id: str,
        quantity: int,
        payment_method: str,
        destination: str,
        customer_email: str,
    ) -> OrderResult:
        """一步完成下单：检查库存->预留->计费->发货->通知"""
        stock = self._inventory.check_stock(product_id)
        if stock < quantity:
            raise ValueError(f"Insufficient stock: {stock} < {quantity}")

        reservation = self._inventory.reserve(product_id, quantity)
        shipping_cost = self._shipping.calculate_cost(quantity * 0.5, destination)
        total = quantity * 19.99 + shipping_cost
        txn_id = self._payment.charge(total, payment_method)
        ship_id = self._shipping.create_shipment(reservation, destination)
        self._notification.send_email(
            customer_email, "Order Confirmed", f"Shipment: {ship_id}"
        )
        return OrderResult(
            order_id=reservation,
            transaction_id=txn_id,
            shipment_id=ship_id,
            total_cost=total,
        )
```

**使用场景**: 复杂API简化、子系统封装、微服务聚合层、SDK封装。

**注意事项**:
- 外观不应成为"上帝对象"——只封装常见工作流，不代替子系统
- 允许客户端在需要时绕过外观直接使用子系统

### 5. 组合模式 (Composite)

**问题描述**: 将对象组合成树形结构以表示"部分-整体"层次，使客户端对单个对象和组合对象的使用具有一致性。

**Python实现**:

```python
from __future__ import annotations
from dataclasses import dataclass, field

@dataclass
class FileSystemItem:
    name: str

    def size(self) -> int:
        raise NotImplementedError

    def display(self, indent: int = 0) -> str:
        raise NotImplementedError

@dataclass
class File(FileSystemItem):
    _size: int = 0

    def size(self) -> int:
        return self._size

    def display(self, indent: int = 0) -> str:
        prefix = "  " * indent
        return f"{prefix}{self.name} ({self._size}B)"

@dataclass
class Directory(FileSystemItem):
    children: list[FileSystemItem] = field(default_factory=list)

    def add(self, item: FileSystemItem) -> None:
        self.children.append(item)

    def remove(self, name: str) -> None:
        self.children = [c for c in self.children if c.name != name]

    def size(self) -> int:
        return sum(child.size() for child in self.children)

    def display(self, indent: int = 0) -> str:
        prefix = "  " * indent
        lines = [f"{prefix}{self.name}/ ({self.size()}B)"]
        for child in self.children:
            lines.append(child.display(indent + 1))
        return "\n".join(lines)

# 使用
root = Directory("project")
src = Directory("src")
src.add(File("main.py", 1200))
src.add(File("utils.py", 800))
root.add(src)
root.add(File("README.md", 500))
print(root.display())
# project/ (2500B)
#   src/ (2000B)
#     main.py (1200B)
#     utils.py (800B)
#   README.md (500B)
```

**使用场景**: 文件系统、UI组件树、组织架构、菜单系统、权限树。

**注意事项**:
- 注意避免循环引用（目录不应包含自己的祖先）
- 递归操作注意深度限制（`sys.setrecursionlimit`）

### 6. 享元模式 (Flyweight)

**问题描述**: 通过共享细粒度对象来减少内存使用。

**Python实现**:

```python
import weakref
from dataclasses import dataclass

@dataclass(frozen=True)
class TextStyle:
    """不可变的文本样式——内在状态，可共享"""
    font_family: str
    font_size: int
    bold: bool
    italic: bool
    color: str

class TextStyleFactory:
    """享元工厂：相同样式只创建一个实例"""
    _cache: dict[tuple, TextStyle] = {}

    @classmethod
    def get_style(
        cls,
        font_family: str = "Arial",
        font_size: int = 12,
        bold: bool = False,
        italic: bool = False,
        color: str = "#000000",
    ) -> TextStyle:
        key = (font_family, font_size, bold, italic, color)
        if key not in cls._cache:
            cls._cache[key] = TextStyle(
                font_family=font_family,
                font_size=font_size,
                bold=bold,
                italic=italic,
                color=color,
            )
        return cls._cache[key]

    @classmethod
    def cache_size(cls) -> int:
        return len(cls._cache)

@dataclass
class TextCharacter:
    """单个字符——外在状态（位置）+ 共享的内在状态（样式）"""
    char: str
    x: int
    y: int
    style: TextStyle

# 使用：100万个字符只需少量样式对象
body_style = TextStyleFactory.get_style("Arial", 12, False, False, "#333")
bold_style = TextStyleFactory.get_style("Arial", 12, True, False, "#333")
characters = [TextCharacter("H", 0, 0, bold_style)]
characters.extend(TextCharacter(c, i * 8, 0, body_style) for i, c in enumerate("ello World", 1))
# 尽管有11个字符对象，只创建了2个样式对象
```

**使用场景**: 文本渲染、地图瓦片、粒子系统、字符串驻留。

**注意事项**:
- Python的`str`和小整数已使用享元（字符串驻留、整数缓存）
- 用`frozen=True`的`dataclass`或`__slots__`进一步优化
- `weakref.WeakValueDictionary`可让不再引用的享元被GC回收

---

## 行为型模式

### 1. 策略模式 (Strategy)

**问题描述**: 定义一族算法，将每个算法封装起来，使它们可以互相替换。

**Python实现**:

```python
from typing import Callable
from dataclasses import dataclass

# Python惯用：策略就是函数
PricingStrategy = Callable[[float, int], float]

def regular_pricing(unit_price: float, quantity: int) -> float:
    """标准定价"""
    return unit_price * quantity

def bulk_pricing(unit_price: float, quantity: int) -> float:
    """批量折扣：10件以上打8折"""
    total = unit_price * quantity
    if quantity > 10:
        total *= 0.8
    return total

def vip_pricing(unit_price: float, quantity: int) -> float:
    """VIP定价：全场7折"""
    return unit_price * quantity * 0.7

def seasonal_pricing(discount: float = 0.85) -> PricingStrategy:
    """季节性定价：闭包捕获折扣率"""
    def strategy(unit_price: float, quantity: int) -> float:
        return unit_price * quantity * discount
    return strategy

@dataclass
class Order:
    product: str
    unit_price: float
    quantity: int
    pricing: PricingStrategy = regular_pricing  # 默认策略

    def total(self) -> float:
        return self.pricing(self.unit_price, self.quantity)

# 使用：策略作为参数传入
order_a = Order("Widget", 10.0, 5, pricing=regular_pricing)
order_b = Order("Widget", 10.0, 20, pricing=bulk_pricing)
order_c = Order("Widget", 10.0, 5, pricing=seasonal_pricing(0.9))
print(order_a.total())  # 50.0
print(order_b.total())  # 160.0
print(order_c.total())  # 45.0
```

**使用场景**: 定价策略、排序算法选择、验证规则、序列化格式、压缩算法。

**注意事项**:
- Python中策略模式的最佳实现就是传递函数，无需类层次
- 需要携带状态的策略可用闭包或callable类
- 避免为了"模式"而创建不必要的Strategy接口和实现类

### 2. 观察者模式 (Observer)

**问题描述**: 定义对象间的一对多依赖关系，当一个对象状态改变时，所有依赖者都得到通知。

**Python实现**:

```python
from __future__ import annotations
from typing import Callable, Any
from dataclasses import dataclass, field
from collections import defaultdict
import weakref

# 事件类型标识
type EventHandler = Callable[[str, dict[str, Any]], None]

class EventBus:
    """轻量级事件总线：基于回调函数的观察者模式"""
    def __init__(self) -> None:
        self._handlers: dict[str, list[EventHandler]] = defaultdict(list)

    def subscribe(self, event: str, handler: EventHandler) -> Callable[[], None]:
        """订阅事件，返回取消订阅的函数"""
        self._handlers[event].append(handler)
        def unsubscribe() -> None:
            self._handlers[event].remove(handler)
        return unsubscribe

    def publish(self, event: str, data: dict[str, Any] | None = None) -> None:
        """发布事件，通知所有订阅者"""
        for handler in self._handlers.get(event, []):
            handler(event, data or {})

    def clear(self, event: str | None = None) -> None:
        if event:
            self._handlers.pop(event, None)
        else:
            self._handlers.clear()

# 使用
bus = EventBus()

def on_user_created(event: str, data: dict[str, Any]) -> None:
    print(f"Send welcome email to {data['email']}")

def on_user_created_audit(event: str, data: dict[str, Any]) -> None:
    print(f"Audit log: user {data['user_id']} created")

unsub1 = bus.subscribe("user.created", on_user_created)
unsub2 = bus.subscribe("user.created", on_user_created_audit)

bus.publish("user.created", {"user_id": 42, "email": "new@example.com"})
# Send welcome email to new@example.com
# Audit log: user 42 created

unsub1()  # 取消welcome邮件的订阅
```

**使用场景**: UI事件处理、领域事件、消息队列抽象、插件系统、数据绑定。

**注意事项**:
- 注意内存泄漏：长生命周期的事件总线可能持有已不需要的handler引用
- 考虑使用`weakref`避免循环引用
- 异步场景使用`asyncio.Event`或异步事件总线

### 3. 命令模式 (Command)

**问题描述**: 将请求封装为对象，支持撤销、队列和日志记录。

**Python实现**:

```python
from __future__ import annotations
from typing import Protocol
from dataclasses import dataclass, field

class Command(Protocol):
    def execute(self) -> None: ...
    def undo(self) -> None: ...
    @property
    def description(self) -> str: ...

@dataclass
class TextDocument:
    content: str = ""

    def insert(self, position: int, text: str) -> None:
        self.content = self.content[:position] + text + self.content[position:]

    def delete(self, position: int, length: int) -> str:
        deleted = self.content[position:position + length]
        self.content = self.content[:position] + self.content[position + length:]
        return deleted

@dataclass
class InsertCommand:
    doc: TextDocument
    position: int
    text: str

    @property
    def description(self) -> str:
        return f"Insert '{self.text}' at {self.position}"

    def execute(self) -> None:
        self.doc.insert(self.position, self.text)

    def undo(self) -> None:
        self.doc.delete(self.position, len(self.text))

@dataclass
class DeleteCommand:
    doc: TextDocument
    position: int
    length: int
    _deleted_text: str = field(default="", init=False)

    @property
    def description(self) -> str:
        return f"Delete {self.length} chars at {self.position}"

    def execute(self) -> None:
        self._deleted_text = self.doc.delete(self.position, self.length)

    def undo(self) -> None:
        self.doc.insert(self.position, self._deleted_text)

class CommandHistory:
    """命令历史管理器：支持撤销/重做"""
    def __init__(self) -> None:
        self._undo_stack: list[Command] = []
        self._redo_stack: list[Command] = []

    def execute(self, command: Command) -> None:
        command.execute()
        self._undo_stack.append(command)
        self._redo_stack.clear()  # 新命令清除重做栈

    def undo(self) -> None:
        if not self._undo_stack:
            return
        cmd = self._undo_stack.pop()
        cmd.undo()
        self._redo_stack.append(cmd)

    def redo(self) -> None:
        if not self._redo_stack:
            return
        cmd = self._redo_stack.pop()
        cmd.execute()
        self._undo_stack.append(cmd)

# 使用
doc = TextDocument()
history = CommandHistory()
history.execute(InsertCommand(doc, 0, "Hello World"))
history.execute(InsertCommand(doc, 5, ","))
print(doc.content)  # "Hello, World"
history.undo()
print(doc.content)  # "Hello World"
history.redo()
print(doc.content)  # "Hello, World"
```

**使用场景**: 文本编辑器撤销/重做、事务回滚、宏录制、任务队列、CLI命令执行。

**注意事项**:
- 简单场景可以用callable替代Command类
- 命令应捕获执行时的状态以支持可靠的undo
- 注意命令与Memento模式结合以保存完整快照

### 4. 状态机模式 (State Machine)

**问题描述**: 允许对象在其内部状态改变时改变其行为。

**Python实现**:

```python
from __future__ import annotations
from enum import Enum, auto
from dataclasses import dataclass, field
from typing import Callable

class OrderStatus(Enum):
    PENDING = auto()
    PAID = auto()
    SHIPPED = auto()
    DELIVERED = auto()
    CANCELLED = auto()

# 状态转换定义：(当前状态, 事件) -> 下一个状态
type TransitionKey = tuple[OrderStatus, str]

_TRANSITIONS: dict[TransitionKey, OrderStatus] = {
    (OrderStatus.PENDING, "pay"): OrderStatus.PAID,
    (OrderStatus.PENDING, "cancel"): OrderStatus.CANCELLED,
    (OrderStatus.PAID, "ship"): OrderStatus.SHIPPED,
    (OrderStatus.PAID, "cancel"): OrderStatus.CANCELLED,
    (OrderStatus.SHIPPED, "deliver"): OrderStatus.DELIVERED,
}

type HookFn = Callable[["OrderStateMachine", str], None]

@dataclass
class OrderStateMachine:
    order_id: str
    status: OrderStatus = OrderStatus.PENDING
    _on_transition: list[HookFn] = field(default_factory=list)

    def add_hook(self, hook: HookFn) -> None:
        self._on_transition.append(hook)

    def trigger(self, event: str) -> None:
        key = (self.status, event)
        next_status = _TRANSITIONS.get(key)
        if next_status is None:
            raise ValueError(
                f"Invalid transition: {self.status.name} + '{event}'"
            )
        old = self.status
        self.status = next_status
        for hook in self._on_transition:
            hook(self, f"{old.name} -> {next_status.name}")

    @property
    def is_terminal(self) -> bool:
        return self.status in {OrderStatus.DELIVERED, OrderStatus.CANCELLED}

# 使用
order = OrderStateMachine("ORD-001")
order.add_hook(lambda sm, t: print(f"[{sm.order_id}] {t}"))
order.trigger("pay")       # [ORD-001] PENDING -> PAID
order.trigger("ship")      # [ORD-001] PAID -> SHIPPED
order.trigger("deliver")   # [ORD-001] SHIPPED -> DELIVERED
```

**使用场景**: 订单流程、工作流引擎、游戏AI、协议解析、审批流程。

**注意事项**:
- 简单状态机用字典映射比类层次更清晰
- 复杂场景考虑`transitions`库或`statemachine`库
- 状态转换必须是显式的——不允许隐式跳跃

### 5. 模板方法模式 (Template Method)

**问题描述**: 定义算法骨架，将某些步骤延迟到子类实现。

**Python实现**:

```python
from abc import ABC, abstractmethod
from dataclasses import dataclass
import time

@dataclass
class ETLResult:
    records_extracted: int
    records_transformed: int
    records_loaded: int
    elapsed_seconds: float

class ETLPipeline(ABC):
    """ETL管道模板：固定的提取->转换->加载流程"""
    def run(self) -> ETLResult:
        start = time.time()
        raw = self.extract()
        transformed = self.transform(raw)
        count = self.load(transformed)
        elapsed = time.time() - start
        self.on_complete(count, elapsed)  # 钩子方法
        return ETLResult(
            records_extracted=len(raw),
            records_transformed=len(transformed),
            records_loaded=count,
            elapsed_seconds=elapsed,
        )

    @abstractmethod
    def extract(self) -> list[dict]: ...

    @abstractmethod
    def transform(self, raw: list[dict]) -> list[dict]: ...

    @abstractmethod
    def load(self, data: list[dict]) -> int: ...

    def on_complete(self, count: int, elapsed: float) -> None:
        """钩子方法：子类可选覆写"""
        pass

class CSVToPostgresETL(ETLPipeline):
    def __init__(self, csv_path: str, table: str) -> None:
        self.csv_path = csv_path
        self.table = table

    def extract(self) -> list[dict]:
        print(f"Reading CSV from {self.csv_path}")
        return [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]

    def transform(self, raw: list[dict]) -> list[dict]:
        return [
            {**row, "name": row["name"].strip().title()}
            for row in raw
        ]

    def load(self, data: list[dict]) -> int:
        print(f"Inserting {len(data)} rows into {self.table}")
        return len(data)

    def on_complete(self, count: int, elapsed: float) -> None:
        print(f"ETL complete: {count} records in {elapsed:.2f}s")
```

**使用场景**: ETL管道、测试框架（setUp/tearDown）、构建系统、报告生成器。

**注意事项**:
- Python也可以用函数参数替代子类化实现模板方法
- 钩子方法提供默认空实现，子类按需覆写
- 避免模板方法嵌套太深——超过5个步骤考虑拆分

### 6. 迭代器模式 (Iterator)

**问题描述**: 提供一种顺序访问聚合对象元素的方法，而不暴露其底层表示。

**Python实现**:

```python
from typing import Iterator, Generator
from dataclasses import dataclass

@dataclass
class Pagination:
    """分页迭代器：逐页获取远程数据"""
    base_url: str
    page_size: int = 20

    def fetch_page(self, page: int) -> list[dict]:
        """模拟远程API分页请求"""
        if page > 5:  # 模拟5页数据
            return []
        return [
            {"id": (page - 1) * self.page_size + i, "data": f"item-{i}"}
            for i in range(self.page_size)
        ]

    def __iter__(self) -> Generator[dict, None, None]:
        """生成器实现：逐条yield，惰性加载每页"""
        page = 1
        while True:
            items = self.fetch_page(page)
            if not items:
                break
            yield from items
            page += 1

def chunked(iterable, size: int) -> Generator[list, None, None]:
    """通用分块迭代器"""
    chunk: list = []
    for item in iterable:
        chunk.append(item)
        if len(chunk) == size:
            yield chunk
            chunk = []
    if chunk:
        yield chunk

# 使用
paginator = Pagination("https://api.example.com/items", page_size=20)
for item in paginator:
    if item["id"] > 10:
        break
    print(item)

# 分块处理
for batch in chunked(range(25), 10):
    print(f"Processing batch of {len(batch)} items")
```

**使用场景**: 分页API遍历、大文件逐行读取、数据库游标、树遍历、流式处理。

**注意事项**:
- Python的生成器（`yield`）是迭代器的语法糖，优先使用
- `yield from`委托子迭代器，避免手动循环
- 迭代器是一次性的——需要多次遍历要重新创建或使用`itertools.tee`

### 7. 责任链模式 (Chain of Responsibility)

**问题描述**: 将请求沿着处理者链传递，直到有一个处理者处理它。

**Python实现**:

```python
from __future__ import annotations
from typing import Callable, Any
from dataclasses import dataclass

# Python惯用：中间件/管道式责任链
type Middleware = Callable[[dict[str, Any], Callable], dict[str, Any]]

def auth_middleware(request: dict[str, Any], next_handler: Callable) -> dict[str, Any]:
    token = request.get("headers", {}).get("Authorization")
    if not token:
        return {"status": 401, "body": "Unauthorized"}
    if not token.startswith("Bearer "):
        return {"status": 401, "body": "Invalid token format"}
    request["user"] = {"id": 1, "role": "admin"}  # 模拟解析token
    return next_handler(request)

def rate_limit_middleware(request: dict[str, Any], next_handler: Callable) -> dict[str, Any]:
    client_ip = request.get("client_ip", "unknown")
    # 模拟限流检查
    if client_ip == "banned":
        return {"status": 429, "body": "Rate limit exceeded"}
    return next_handler(request)

def logging_middleware(request: dict[str, Any], next_handler: Callable) -> dict[str, Any]:
    print(f"[LOG] {request.get('method', 'GET')} {request.get('path', '/')}")
    response = next_handler(request)
    print(f"[LOG] Response: {response.get('status', 200)}")
    return response

class MiddlewarePipeline:
    """中间件管道：按顺序组装处理链"""
    def __init__(self) -> None:
        self._middlewares: list[Middleware] = []

    def use(self, middleware: Middleware) -> "MiddlewarePipeline":
        self._middlewares.append(middleware)
        return self

    def handle(self, request: dict[str, Any], final: Callable) -> dict[str, Any]:
        """构建责任链并执行"""
        handler = final
        for mw in reversed(self._middlewares):
            prev_handler = handler
            handler = lambda req, _mw=mw, _next=prev_handler: _mw(req, _next)
        return handler(request)

# 使用
pipeline = MiddlewarePipeline()
pipeline.use(logging_middleware).use(rate_limit_middleware).use(auth_middleware)

def handle_request(request: dict[str, Any]) -> dict[str, Any]:
    return {"status": 200, "body": f"Hello, user {request['user']['id']}"}

response = pipeline.handle(
    {"method": "GET", "path": "/api/data", "headers": {"Authorization": "Bearer abc123"}},
    handle_request,
)
```

**使用场景**: HTTP中间件、日志处理链、审批流程、输入验证管道、异常处理层。

**注意事项**:
- Python中间件模式（WSGI/ASGI）就是责任链的典型应用
- 注意闭包变量捕获问题（使用默认参数绑定）
- 链中的每个处理者应该只关注自己的职责

---

## Python特有模式

### 1. Mixin模式

**问题描述**: 通过多重继承为类添加可复用的功能片段，而不建立is-a关系。

**Python实现**:

```python
import json
import time
from typing import Any

class SerializableMixin:
    """JSON序列化能力"""
    def to_json(self) -> str:
        data = {}
        for key, value in self.__dict__.items():
            if not key.startswith("_"):
                data[key] = value
        return json.dumps(data, default=str, ensure_ascii=False)

    @classmethod
    def from_json(cls, json_str: str) -> "SerializableMixin":
        data = json.loads(json_str)
        instance = cls.__new__(cls)
        instance.__dict__.update(data)
        return instance

class TimestampMixin:
    """自动时间戳追踪"""
    def __init_subclass__(cls, **kwargs: Any) -> None:
        super().__init_subclass__(**kwargs)
        original_init = cls.__init__

        def new_init(self: Any, *args: Any, **kw: Any) -> None:
            original_init(self, *args, **kw)
            self.created_at = time.time()
            self.updated_at = time.time()

        cls.__init__ = new_init  # type: ignore[attr-defined]

    def touch(self) -> None:
        self.updated_at = time.time()

class ValidatableMixin:
    """声明式字段验证"""
    _validators: dict[str, list] = {}

    def validate(self) -> list[str]:
        errors: list[str] = []
        for field_name, validators in self._validators.items():
            value = getattr(self, field_name, None)
            for validator_fn, msg in validators:
                if not validator_fn(value):
                    errors.append(f"{field_name}: {msg}")
        return errors

class User(SerializableMixin, TimestampMixin, ValidatableMixin):
    _validators = {
        "name": [
            (lambda v: v and len(v) >= 2, "Name must be at least 2 characters"),
        ],
        "email": [
            (lambda v: v and "@" in v, "Invalid email format"),
        ],
    }

    def __init__(self, name: str, email: str) -> None:
        self.name = name
        self.email = email

# 使用
user = User("Alice", "alice@example.com")
print(user.to_json())       # 序列化
print(user.created_at)      # 时间戳
print(user.validate())      # 验证 -> []
```

**使用场景**: ORM模型增强、API序列化、审计日志、权限控制、缓存能力注入。

**注意事项**:
- Mixin类不应独立实例化，且不应有`__init__`（除非通过`__init_subclass__`）
- MRO（方法解析顺序）从左到右——将最具体的Mixin放在最左边
- 避免Mixin之间有方法名冲突

### 2. 描述符模式 (Descriptor)

**问题描述**: 通过定义`__get__`、`__set__`、`__delete__`方法控制属性访问行为。

**Python实现**:

```python
from typing import Any, Callable

class Validated:
    """通用验证描述符"""
    def __init__(self, validator: Callable[[Any], bool], error_msg: str) -> None:
        self.validator = validator
        self.error_msg = error_msg
        self.attr_name = ""

    def __set_name__(self, owner: type, name: str) -> None:
        self.attr_name = name
        self.storage_name = f"_desc_{name}"

    def __get__(self, obj: Any, objtype: type | None = None) -> Any:
        if obj is None:
            return self
        return getattr(obj, self.storage_name, None)

    def __set__(self, obj: Any, value: Any) -> None:
        if not self.validator(value):
            raise ValueError(f"{self.attr_name}: {self.error_msg} (got {value!r})")
        setattr(obj, self.storage_name, value)

class PositiveNumber(Validated):
    def __init__(self, error_msg: str = "must be positive") -> None:
        super().__init__(lambda v: isinstance(v, (int, float)) and v > 0, error_msg)

class NonEmptyString(Validated):
    def __init__(self, max_length: int = 255) -> None:
        super().__init__(
            lambda v: isinstance(v, str) and 0 < len(v) <= max_length,
            f"must be non-empty string (max {max_length} chars)",
        )

class InRange(Validated):
    def __init__(self, min_val: float, max_val: float) -> None:
        super().__init__(
            lambda v: isinstance(v, (int, float)) and min_val <= v <= max_val,
            f"must be between {min_val} and {max_val}",
        )

class Product:
    name = NonEmptyString(max_length=100)
    price = PositiveNumber()
    rating = InRange(0.0, 5.0)

    def __init__(self, name: str, price: float, rating: float) -> None:
        self.name = name
        self.price = price
        self.rating = rating

# 使用
p = Product("Widget", 9.99, 4.5)  # OK
# Product("", 9.99, 4.5)          # ValueError: name: must be non-empty string
# Product("Widget", -1, 4.5)      # ValueError: price: must be positive
```

**使用场景**: ORM字段定义、表单验证、属性缓存（`cached_property`就是描述符）、类型强制。

**注意事项**:
- `__set_name__`（Python 3.6+）自动获取属性名，不再需要手动传入
- 描述符存储值时避免用实例`__dict__`的同名键（会遮蔽描述符）
- `property`本身就是描述符的语法糖

### 3. 元类模式 (Metaclass)

**问题描述**: 控制类的创建过程，实现类级别的约束和自动化。

**Python实现**:

```python
from typing import Any

class PluginMeta(type):
    """插件注册元类：所有子类自动注册到注册表"""
    _registry: dict[str, type] = {}

    def __new__(
        mcs,
        name: str,
        bases: tuple[type, ...],
        namespace: dict[str, Any],
        **kwargs: Any,
    ) -> "PluginMeta":
        cls = super().__new__(mcs, name, bases, namespace)
        # 不注册基类本身
        if bases:
            plugin_name = getattr(cls, "plugin_name", name.lower())
            mcs._registry[plugin_name] = cls
        return cls

    @classmethod
    def get_plugin(mcs, name: str) -> type | None:
        return mcs._registry.get(name)

    @classmethod
    def list_plugins(mcs) -> list[str]:
        return list(mcs._registry.keys())

class Plugin(metaclass=PluginMeta):
    """插件基类"""
    plugin_name: str

    def execute(self, data: dict) -> dict:
        raise NotImplementedError

class JSONExporter(Plugin):
    plugin_name = "json_exporter"
    def execute(self, data: dict) -> dict:
        return {"format": "json", "output": str(data)}

class CSVExporter(Plugin):
    plugin_name = "csv_exporter"
    def execute(self, data: dict) -> dict:
        header = ",".join(data.keys())
        values = ",".join(str(v) for v in data.values())
        return {"format": "csv", "output": f"{header}\n{values}"}

# 使用：自动发现所有已注册的插件
print(PluginMeta.list_plugins())  # ['json_exporter', 'csv_exporter']
plugin_cls = PluginMeta.get_plugin("json_exporter")
if plugin_cls:
    result = plugin_cls().execute({"name": "Alice", "age": 30})
```

**使用场景**: 插件系统、ORM模型注册、API路由自动收集、接口合约强制。

**注意事项**:
- 元类是Python最强大的工具，也最容易滥用
- **优先考虑`__init_subclass__`**（Python 3.6+），它能覆盖大部分元类场景且更简单
- 避免多层元类继承——极难调试
- Django ORM和SQLAlchemy大量使用元类，但业务代码很少需要

### 4. 上下文管理器协议 (Context Manager)

**问题描述**: 确保资源的正确获取和释放，即使发生异常。

**Python实现**:

```python
from __future__ import annotations
import time
from contextlib import contextmanager
from typing import Generator, Any
from dataclasses import dataclass, field

# 方式一：类实现
@dataclass
class DatabaseTransaction:
    """数据库事务上下文管理器"""
    connection: Any
    _savepoint: str | None = field(default=None, init=False)

    def __enter__(self) -> "DatabaseTransaction":
        self._savepoint = f"sp_{id(self)}"
        print(f"BEGIN TRANSACTION (savepoint: {self._savepoint})")
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> bool:
        if exc_type is not None:
            print(f"ROLLBACK to {self._savepoint}: {exc_val}")
            # 返回True表示异常已处理，不再传播
            return False
        print("COMMIT")
        return False

    def execute(self, sql: str) -> list[dict]:
        print(f"  EXEC: {sql}")
        return []

# 方式二：生成器实现（更简洁）
@contextmanager
def timer(label: str) -> Generator[dict[str, float], None, None]:
    """计时上下文管理器"""
    result: dict[str, float] = {}
    start = time.perf_counter()
    try:
        yield result
    finally:
        result["elapsed"] = time.perf_counter() - start
        print(f"[{label}] {result['elapsed']:.4f}s")

@contextmanager
def temporary_env(**variables: str) -> Generator[None, None, None]:
    """临时设置环境变量，退出时恢复"""
    import os
    old_values: dict[str, str | None] = {}
    try:
        for key, value in variables.items():
            old_values[key] = os.environ.get(key)
            os.environ[key] = value
        yield
    finally:
        for key, old_value in old_values.items():
            if old_value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = old_value

# 使用
with timer("data_load") as t:
    time.sleep(0.1)
print(f"Elapsed: {t['elapsed']:.4f}s")

with temporary_env(DATABASE_URL="sqlite:///test.db", DEBUG="1"):
    import os
    print(os.environ["DATABASE_URL"])  # sqlite:///test.db
# 退出后环境变量自动恢复
```

**使用场景**: 文件/数据库/网络连接管理、锁管理、临时状态、事务、测试fixture。

**注意事项**:
- `@contextmanager`适合简单场景，复杂逻辑用类实现
- `__exit__`返回`True`会吞掉异常——除非明确需要，否则返回`False`
- 支持`async with`需要实现`__aenter__`/`__aexit__`

### 5. 协程模式 (Coroutine Patterns)

**问题描述**: 利用async/await实现高效的并发模式。

**Python实现**:

```python
import asyncio
from typing import AsyncIterator

# 异步生产者-消费者模式
async def producer(queue: asyncio.Queue[str | None], items: list[str]) -> None:
    for item in items:
        await asyncio.sleep(0.1)  # 模拟异步IO
        await queue.put(item)
        print(f"Produced: {item}")
    await queue.put(None)  # 哨兵值表示结束

async def consumer(queue: asyncio.Queue[str | None], name: str) -> list[str]:
    results: list[str] = []
    while True:
        item = await queue.get()
        if item is None:
            await queue.put(None)  # 传递结束信号给其他消费者
            break
        print(f"[{name}] Consumed: {item}")
        results.append(item)
        queue.task_done()
    return results

# 异步迭代器模式
async def async_paginate(url: str, page_size: int = 10) -> AsyncIterator[dict]:
    """异步分页迭代器"""
    page = 1
    while True:
        # 模拟异步API调用
        await asyncio.sleep(0.05)
        items = [{"id": (page - 1) * page_size + i} for i in range(page_size)]
        if page > 3:  # 模拟数据结束
            break
        for item in items:
            yield item
        page += 1

# 扇出/扇入模式
async def fetch_all_parallel(urls: list[str]) -> list[dict]:
    """并行获取多个URL，收集结果"""
    async def fetch_one(url: str) -> dict:
        await asyncio.sleep(0.1)  # 模拟网络请求
        return {"url": url, "status": 200}

    tasks = [asyncio.create_task(fetch_one(url)) for url in urls]
    results = await asyncio.gather(*tasks, return_exceptions=True)
    return [r for r in results if isinstance(r, dict)]

# 使用
async def main() -> None:
    # 生产者-消费者
    queue: asyncio.Queue[str | None] = asyncio.Queue(maxsize=5)
    items = [f"task-{i}" for i in range(10)]
    await asyncio.gather(
        producer(queue, items),
        consumer(queue, "worker-1"),
        consumer(queue, "worker-2"),
    )

    # 异步迭代
    async for item in async_paginate("https://api.example.com"):
        if item["id"] > 5:
            break

asyncio.run(main())
```

**使用场景**: Web爬虫、API聚合、实时数据流处理、WebSocket服务。

**注意事项**:
- 不要在async函数中调用阻塞IO——使用`run_in_executor`包装
- `asyncio.gather`的`return_exceptions=True`防止一个失败导致全部取消
- 使用`asyncio.Semaphore`限制并发度

### 6. 依赖注入模式 (Dependency Injection)

**问题描述**: 将依赖的创建与使用分离，提高可测试性和灵活性。

**Python实现**:

```python
from __future__ import annotations
from typing import Protocol, Callable, Any, TypeVar
from dataclasses import dataclass, field

T = TypeVar("T")

class Repository(Protocol):
    def find_by_id(self, entity_id: str) -> dict | None: ...
    def save(self, entity: dict) -> None: ...

class NotificationSender(Protocol):
    def send(self, recipient: str, message: str) -> bool: ...

# 简易DI容器
class Container:
    def __init__(self) -> None:
        self._factories: dict[type, Callable[[], Any]] = {}
        self._singletons: dict[type, Any] = {}

    def register(self, interface: type, factory: Callable[[], Any], singleton: bool = False) -> None:
        if singleton:
            self._factories[interface] = factory
            # 延迟创建
        else:
            self._factories[interface] = factory

    def register_singleton(self, interface: type, factory: Callable[[], Any]) -> None:
        self._factories[interface] = factory

    def resolve(self, interface: type[T]) -> T:
        if interface in self._singletons:
            return self._singletons[interface]
        factory = self._factories.get(interface)
        if factory is None:
            raise KeyError(f"No registration for {interface}")
        instance = factory()
        return instance

# 具体实现
class PostgresRepository:
    def __init__(self, dsn: str = "postgresql://localhost/mydb") -> None:
        self.dsn = dsn
    def find_by_id(self, entity_id: str) -> dict | None:
        return {"id": entity_id, "data": "from postgres"}
    def save(self, entity: dict) -> None:
        print(f"Saved to Postgres: {entity}")

class InMemoryRepository:
    """测试用内存存储"""
    def __init__(self) -> None:
        self._store: dict[str, dict] = {}
    def find_by_id(self, entity_id: str) -> dict | None:
        return self._store.get(entity_id)
    def save(self, entity: dict) -> None:
        self._store[entity["id"]] = entity

class EmailSender:
    def send(self, recipient: str, message: str) -> bool:
        print(f"Email to {recipient}: {message}")
        return True

@dataclass
class UserService:
    """通过构造函数注入依赖"""
    repo: Repository
    notifier: NotificationSender

    def create_user(self, name: str, email: str) -> dict:
        user = {"id": f"user-{hash(name) % 10000}", "name": name, "email": email}
        self.repo.save(user)
        self.notifier.send(email, f"Welcome, {name}!")
        return user

# 生产环境配置
container = Container()
container.register(Repository, lambda: PostgresRepository())
container.register(NotificationSender, lambda: EmailSender())
service = UserService(
    repo=container.resolve(Repository),
    notifier=container.resolve(NotificationSender),
)

# 测试环境：替换为mock实现
test_repo = InMemoryRepository()
test_service = UserService(repo=test_repo, notifier=EmailSender())
```

**使用场景**: 服务层构造、测试替身注入、配置驱动实现选择、插件系统。

**注意事项**:
- Python中构造函数注入（传参）是最简单有效的DI方式
- 避免过度工程化——大多数Python项目不需要完整的DI框架
- 如需框架级DI，考虑`dependency-injector`或`inject`库

---

## 架构模式

### 1. 仓储模式 (Repository)

**问题描述**: 将数据访问逻辑与业务逻辑分离，提供集合式接口操作持久化数据。

**Python实现**:

```python
from __future__ import annotations
from typing import Protocol, TypeVar, Generic
from dataclasses import dataclass, field
from abc import abstractmethod

T = TypeVar("T")

class Repository(Protocol[T]):
    def get(self, entity_id: str) -> T | None: ...
    def list(self, offset: int = 0, limit: int = 100) -> list[T]: ...
    def add(self, entity: T) -> None: ...
    def update(self, entity: T) -> None: ...
    def delete(self, entity_id: str) -> bool: ...

@dataclass
class Order:
    id: str
    customer_id: str
    items: list[dict] = field(default_factory=list)
    total: float = 0.0
    status: str = "pending"

class InMemoryOrderRepository:
    """内存仓储——用于测试和原型"""
    def __init__(self) -> None:
        self._store: dict[str, Order] = {}

    def get(self, entity_id: str) -> Order | None:
        return self._store.get(entity_id)

    def list(self, offset: int = 0, limit: int = 100) -> list[Order]:
        orders = sorted(self._store.values(), key=lambda o: o.id)
        return orders[offset:offset + limit]

    def add(self, entity: Order) -> None:
        if entity.id in self._store:
            raise ValueError(f"Order {entity.id} already exists")
        self._store[entity.id] = entity

    def update(self, entity: Order) -> None:
        if entity.id not in self._store:
            raise ValueError(f"Order {entity.id} not found")
        self._store[entity.id] = entity

    def delete(self, entity_id: str) -> bool:
        return self._store.pop(entity_id, None) is not None

class SQLOrderRepository:
    """SQL仓储——实际生产使用"""
    def __init__(self, session) -> None:
        self._session = session

    def get(self, entity_id: str) -> Order | None:
        row = self._session.execute(
            "SELECT * FROM orders WHERE id = :id", {"id": entity_id}
        ).fetchone()
        return self._row_to_order(row) if row else None

    def list(self, offset: int = 0, limit: int = 100) -> list[Order]:
        rows = self._session.execute(
            "SELECT * FROM orders ORDER BY id LIMIT :limit OFFSET :offset",
            {"limit": limit, "offset": offset},
        ).fetchall()
        return [self._row_to_order(r) for r in rows]

    def add(self, entity: Order) -> None:
        self._session.execute(
            "INSERT INTO orders (id, customer_id, total, status) VALUES (:id, :cid, :total, :status)",
            {"id": entity.id, "cid": entity.customer_id, "total": entity.total, "status": entity.status},
        )

    def update(self, entity: Order) -> None:
        self._session.execute(
            "UPDATE orders SET total = :total, status = :status WHERE id = :id",
            {"id": entity.id, "total": entity.total, "status": entity.status},
        )

    def delete(self, entity_id: str) -> bool:
        result = self._session.execute(
            "DELETE FROM orders WHERE id = :id", {"id": entity_id}
        )
        return result.rowcount > 0

    @staticmethod
    def _row_to_order(row) -> Order:
        return Order(id=row.id, customer_id=row.customer_id, total=row.total, status=row.status)
```

**使用场景**: 领域驱动设计中的聚合持久化、多数据源抽象、测试隔离。

**注意事项**:
- 仓储接口应面向领域语言，而非SQL语义
- 一个聚合根对应一个仓储
- 查询复杂度高时可引入规约模式（Specification）

### 2. 工作单元模式 (Unit of Work)

**问题描述**: 跟踪业务事务中所有受影响的对象，协调变更的持久化。

**Python实现**:

```python
from __future__ import annotations
from contextlib import contextmanager
from typing import Generator

class UnitOfWork:
    """工作单元：收集变更，统一提交或回滚"""
    def __init__(self, session_factory) -> None:
        self._session_factory = session_factory
        self._session = None
        self._new: list = []
        self._dirty: list = []
        self._removed: list = []

    def __enter__(self) -> "UnitOfWork":
        self._session = self._session_factory()
        self._new.clear()
        self._dirty.clear()
        self._removed.clear()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> bool:
        if exc_type:
            self.rollback()
        self._session = None
        return False

    def register_new(self, entity) -> None:
        self._new.append(entity)

    def register_dirty(self, entity) -> None:
        if entity not in self._dirty:
            self._dirty.append(entity)

    def register_removed(self, entity) -> None:
        self._removed.append(entity)

    def commit(self) -> None:
        for entity in self._new:
            self._session.add(entity)
        for entity in self._dirty:
            self._session.merge(entity)
        for entity in self._removed:
            self._session.delete(entity)
        self._session.commit()
        self._new.clear()
        self._dirty.clear()
        self._removed.clear()

    def rollback(self) -> None:
        if self._session:
            self._session.rollback()
        self._new.clear()
        self._dirty.clear()
        self._removed.clear()

# 使用
# with UnitOfWork(session_factory) as uow:
#     order = uow.orders.get("ORD-001")
#     order.status = "shipped"
#     uow.register_dirty(order)
#     uow.commit()
```

**使用场景**: 数据库事务管理、批量操作原子性、跨仓储一致性。

**注意事项**:
- SQLAlchemy的`Session`本身就是工作单元的实现
- 与仓储模式配合使用，仓储在UoW内操作
- 保持事务粒度尽量小

### 3. CQRS模式 (Command Query Responsibility Segregation)

**问题描述**: 将读操作（查询）和写操作（命令）分离为不同的模型。

**Python实现**:

```python
from __future__ import annotations
from dataclasses import dataclass
from typing import Protocol, Any

# 命令侧（写）
@dataclass
class CreateOrderCommand:
    customer_id: str
    items: list[dict]

@dataclass
class CancelOrderCommand:
    order_id: str
    reason: str

class CommandHandler(Protocol):
    def handle(self, command: Any) -> None: ...

class CreateOrderHandler:
    def __init__(self, repo, event_bus) -> None:
        self.repo = repo
        self.event_bus = event_bus

    def handle(self, command: CreateOrderCommand) -> None:
        order_id = f"ORD-{hash(command.customer_id) % 100000}"
        order = {
            "id": order_id,
            "customer_id": command.customer_id,
            "items": command.items,
            "status": "pending",
        }
        self.repo.add(order)
        self.event_bus.publish("order.created", {"order_id": order_id})

# 查询侧（读）
@dataclass
class OrderSummaryQuery:
    customer_id: str
    status: str | None = None

@dataclass
class OrderSummary:
    order_id: str
    total: float
    status: str
    item_count: int

class OrderQueryService:
    """查询服务：专为读优化的独立模型"""
    def __init__(self, read_db) -> None:
        self._db = read_db

    def get_order_summaries(self, query: OrderSummaryQuery) -> list[OrderSummary]:
        # 读模型可以是反范式化的视图、缓存或搜索索引
        results = self._db.execute(
            "SELECT order_id, total, status, item_count FROM order_summaries "
            "WHERE customer_id = :cid",
            {"cid": query.customer_id},
        )
        return [
            OrderSummary(
                order_id=r.order_id,
                total=r.total,
                status=r.status,
                item_count=r.item_count,
            )
            for r in results
        ]

# 命令总线
class CommandBus:
    def __init__(self) -> None:
        self._handlers: dict[type, CommandHandler] = {}

    def register(self, command_type: type, handler: CommandHandler) -> None:
        self._handlers[command_type] = handler

    def dispatch(self, command: Any) -> None:
        handler = self._handlers.get(type(command))
        if handler is None:
            raise ValueError(f"No handler for {type(command).__name__}")
        handler.handle(command)
```

**使用场景**: 高读写比系统、需要独立扩展读写的系统、事件驱动架构。

**注意事项**:
- CQRS增加了系统复杂度——只在读写模型差异大时使用
- 通常与事件溯源配合，读模型通过事件投影生成
- 读写模型最终一致即可，不要追求强一致

### 4. 事件溯源模式 (Event Sourcing)

**问题描述**: 不存储当前状态，而是存储导致当前状态的所有事件序列。

**Python实现**:

```python
from __future__ import annotations
from dataclasses import dataclass, field
from datetime import datetime
from typing import Any

@dataclass(frozen=True)
class DomainEvent:
    event_type: str
    aggregate_id: str
    data: dict[str, Any]
    timestamp: datetime = field(default_factory=datetime.utcnow)
    version: int = 0

class EventStore:
    """事件存储：追加写入，按聚合ID查询"""
    def __init__(self) -> None:
        self._events: list[DomainEvent] = []

    def append(self, event: DomainEvent) -> None:
        self._events.append(event)

    def get_events(self, aggregate_id: str) -> list[DomainEvent]:
        return [e for e in self._events if e.aggregate_id == aggregate_id]

class BankAccount:
    """银行账户聚合：通过事件重建状态"""
    def __init__(self, account_id: str) -> None:
        self.account_id = account_id
        self.balance: float = 0.0
        self.is_active: bool = False
        self._pending_events: list[DomainEvent] = []
        self._version: int = 0

    def apply_event(self, event: DomainEvent) -> None:
        handler = getattr(self, f"_on_{event.event_type}", None)
        if handler:
            handler(event.data)
        self._version = event.version

    def _on_account_opened(self, data: dict) -> None:
        self.is_active = True
        self.balance = data.get("initial_deposit", 0.0)

    def _on_money_deposited(self, data: dict) -> None:
        self.balance += data["amount"]

    def _on_money_withdrawn(self, data: dict) -> None:
        self.balance -= data["amount"]

    def _on_account_closed(self, data: dict) -> None:
        self.is_active = False

    # 命令方法：产生事件
    def open(self, initial_deposit: float) -> None:
        if self.is_active:
            raise ValueError("Account already open")
        self._raise_event("account_opened", {"initial_deposit": initial_deposit})

    def deposit(self, amount: float) -> None:
        if not self.is_active:
            raise ValueError("Account is closed")
        if amount <= 0:
            raise ValueError("Amount must be positive")
        self._raise_event("money_deposited", {"amount": amount})

    def withdraw(self, amount: float) -> None:
        if not self.is_active:
            raise ValueError("Account is closed")
        if amount > self.balance:
            raise ValueError("Insufficient funds")
        self._raise_event("money_withdrawn", {"amount": amount})

    def _raise_event(self, event_type: str, data: dict) -> None:
        event = DomainEvent(
            event_type=event_type,
            aggregate_id=self.account_id,
            data=data,
            version=self._version + 1,
        )
        self.apply_event(event)
        self._pending_events.append(event)

    def flush_events(self) -> list[DomainEvent]:
        events = self._pending_events[:]
        self._pending_events.clear()
        return events

    @classmethod
    def from_events(cls, account_id: str, events: list[DomainEvent]) -> BankAccount:
        """从事件流重建聚合状态"""
        account = cls(account_id)
        for event in events:
            account.apply_event(event)
        return account

# 使用
store = EventStore()
account = BankAccount("ACC-001")
account.open(1000.0)
account.deposit(500.0)
account.withdraw(200.0)

# 持久化事件
for event in account.flush_events():
    store.append(event)

# 从事件重建状态
rebuilt = BankAccount.from_events("ACC-001", store.get_events("ACC-001"))
print(rebuilt.balance)  # 1300.0
```

**使用场景**: 金融系统、审计追踪、时间旅行调试、分布式系统状态同步。

**注意事项**:
- 事件一旦持久化就不可变——修正错误通过补偿事件
- 事件流过长时使用快照优化重建性能
- 事件schema的演化需要版本管理

### 5. 领域驱动设计在Python中的实现 (DDD)

**问题描述**: 将复杂业务逻辑组织为领域模型，使代码结构反映业务概念。

**Python实现**:

```python
from __future__ import annotations
from dataclasses import dataclass, field
from datetime import datetime
from typing import NewType
from enum import Enum, auto

# 值对象
OrderId = NewType("OrderId", str)
CustomerId = NewType("CustomerId", str)

@dataclass(frozen=True)
class Money:
    """值对象：不可变，通过值比较"""
    amount: float
    currency: str = "CNY"

    def __add__(self, other: Money) -> Money:
        if self.currency != other.currency:
            raise ValueError(f"Cannot add {self.currency} and {other.currency}")
        return Money(self.amount + other.amount, self.currency)

    def __mul__(self, factor: int | float) -> Money:
        return Money(round(self.amount * factor, 2), self.currency)

class OrderStatus(Enum):
    DRAFT = auto()
    CONFIRMED = auto()
    PAID = auto()
    SHIPPED = auto()
    COMPLETED = auto()
    CANCELLED = auto()

# 实体
@dataclass
class OrderItem:
    product_id: str
    product_name: str
    unit_price: Money
    quantity: int

    @property
    def subtotal(self) -> Money:
        return self.unit_price * self.quantity

# 聚合根
@dataclass
class Order:
    """订单聚合根：封装所有业务不变量"""
    id: OrderId
    customer_id: CustomerId
    items: list[OrderItem] = field(default_factory=list)
    status: OrderStatus = OrderStatus.DRAFT
    created_at: datetime = field(default_factory=datetime.utcnow)
    _events: list[dict] = field(default_factory=list, repr=False)

    @property
    def total(self) -> Money:
        if not self.items:
            return Money(0.0)
        result = Money(0.0)
        for item in self.items:
            result = result + item.subtotal
        return result

    def add_item(self, product_id: str, name: str, price: Money, qty: int) -> None:
        if self.status != OrderStatus.DRAFT:
            raise ValueError("Can only add items to draft orders")
        if qty <= 0:
            raise ValueError("Quantity must be positive")
        self.items.append(OrderItem(product_id, name, price, qty))

    def confirm(self) -> None:
        if self.status != OrderStatus.DRAFT:
            raise ValueError(f"Cannot confirm order in {self.status.name} status")
        if not self.items:
            raise ValueError("Cannot confirm empty order")
        self.status = OrderStatus.CONFIRMED
        self._events.append({"type": "order.confirmed", "order_id": self.id})

    def cancel(self, reason: str) -> None:
        if self.status in {OrderStatus.SHIPPED, OrderStatus.COMPLETED}:
            raise ValueError(f"Cannot cancel order in {self.status.name} status")
        self.status = OrderStatus.CANCELLED
        self._events.append({
            "type": "order.cancelled",
            "order_id": self.id,
            "reason": reason,
        })

    def collect_events(self) -> list[dict]:
        events = self._events[:]
        self._events.clear()
        return events

# 领域服务
class PricingService:
    """领域服务：跨聚合的业务逻辑"""
    def calculate_discount(self, order: Order, customer_tier: str) -> Money:
        base = order.total
        discount_rate = {"bronze": 0.0, "silver": 0.05, "gold": 0.10, "platinum": 0.15}
        rate = discount_rate.get(customer_tier, 0.0)
        return Money(round(base.amount * rate, 2), base.currency)

# 使用
order = Order(id=OrderId("ORD-001"), customer_id=CustomerId("CUST-042"))
order.add_item("PROD-1", "Widget", Money(29.90), 3)
order.add_item("PROD-2", "Gadget", Money(99.00), 1)
print(order.total)  # Money(amount=188.7, currency='CNY')
order.confirm()
events = order.collect_events()  # [{'type': 'order.confirmed', ...}]
```

**使用场景**: 复杂业务系统（电商、金融、物流）、需要长期维护的核心域。

**注意事项**:
- 值对象必须不可变（`frozen=True`）
- 聚合根是事务一致性的边界——一个事务只修改一个聚合
- 跨聚合的业务规则放在领域服务中
- 不要对所有代码都用DDD——只对核心复杂域使用

---

## 反模式

### 1. God Object（上帝对象）

**问题描述**: 一个类承担了过多职责，知道太多、做太多。

**识别信号**:
- 文件超过1000行
- 类有20+个方法
- 构造函数注入10+个依赖
- 修改任何功能都要改这个类

**修复方案**: 按职责拆分为多个协作类，使用外观模式提供统一入口。

```python
# 反模式
class OrderManager:
    def create_order(self): ...
    def process_payment(self): ...
    def send_notification(self): ...
    def generate_invoice(self): ...
    def update_inventory(self): ...
    def calculate_shipping(self): ...
    def apply_discount(self): ...
    def handle_refund(self): ...
    # ... 50+ methods

# 正确做法：拆分为独立服务
class OrderService:
    def __init__(self, payment: PaymentService, inventory: InventoryService):
        self.payment = payment
        self.inventory = inventory

    def create_order(self, items): ...

class PaymentService:
    def process(self, order): ...
    def refund(self, order): ...

class InventoryService:
    def reserve(self, items): ...
    def release(self, items): ...
```

### 2. Spaghetti Code（意大利面代码）

**问题描述**: 缺乏清晰结构，控制流复杂交织，难以追踪执行路径。

**识别信号**:
- 函数超过50行
- 深层嵌套（if-else超过4层）
- 大量全局变量
- goto风格的控制流（异常用于流程控制）

**修复方案**: 提取函数、使用早返回、引入状态模式或策略模式。

```python
# 反模式：深层嵌套
def process_order(order):
    if order:
        if order.items:
            if order.customer:
                if order.customer.is_active:
                    if order.total > 0:
                        # 实际逻辑深埋在这里
                        pass

# 正确做法：卫语句 + 早返回
def process_order(order):
    if not order:
        raise ValueError("Order is required")
    if not order.items:
        raise ValueError("Order must have items")
    if not order.customer or not order.customer.is_active:
        raise ValueError("Active customer required")
    if order.total <= 0:
        raise ValueError("Order total must be positive")
    # 实际逻辑在最外层
    _execute_order(order)
```

### 3. 过度设计 (Over-Engineering)

**问题描述**: 为未来可能永远不会出现的需求增加不必要的抽象层。

**识别信号**:
- 只有一个实现的接口/抽象类
- 为3个字段创建Builder模式
- 未使用的扩展点
- "以后可能需要"的抽象

**修复方案**: 遵循YAGNI（You Ain't Gonna Need It），先实现最简方案，当第三次遇到相似需求时再抽象。

```python
# 过度设计：只有一种通知方式却创建了完整的策略体系
class NotificationStrategyFactory(AbstractNotificationFactory):
    ...  # 100行只为发一封邮件

# 正确做法：直接写
def send_welcome_email(user_email: str, user_name: str) -> None:
    # 直接发邮件，等真正需要SMS时再抽象
    smtp.send(to=user_email, subject="Welcome", body=f"Hi {user_name}")
```

### 4. 过早抽象 (Premature Abstraction)

**问题描述**: 在只有一个用例时就创建通用框架，导致抽象与实际需求不匹配。

**修复方案**: Rule of Three——等看到三个相似场景后再提取抽象。

### 5. Singleton滥用

**问题描述**: 将单例当作全局变量使用，导致隐式依赖、测试困难、并发问题。

**识别信号**:
- 在函数内部直接调用`XxxManager.instance()`而非通过参数接收
- 单例持有可变状态且被多线程访问
- 测试时需要"重置"单例状态

**修复方案**: 用依赖注入替代。将单例降级为"只创建一次"的普通对象，通过参数传递。

```python
# 反模式：到处直接访问单例
def process_payment(amount):
    config = AppConfig.instance()  # 隐式依赖
    db = DatabasePool.instance()   # 隐式依赖
    logger = Logger.instance()     # 隐式依赖
    ...

# 正确做法：显式依赖
def process_payment(amount, config, db, logger):
    ...

# 在组合根（入口点）创建并注入
config = AppConfig()
db = DatabasePool(config.db_url)
logger = Logger(config.log_level)
process_payment(100.0, config, db, logger)
```

---

## 实战案例：用设计模式重构支付系统

### 重构前：混乱的支付处理

```python
# payment_processor.py - 重构前（典型的God Object + Spaghetti Code）
class PaymentProcessor:
    def __init__(self):
        self.db = DatabasePool.instance()
        self.config = AppConfig.instance()
        self.logger = Logger.instance()

    def process(self, order_id, payment_type, card_number=None,
                alipay_id=None, wechat_openid=None, amount=None):
        # 800行方法，处理所有支付类型
        order = self.db.query(f"SELECT * FROM orders WHERE id = '{order_id}'")  # SQL注入
        if not order:
            return {"success": False, "error": "Order not found"}

        if payment_type == "credit_card":
            if not card_number:
                return {"success": False, "error": "Card number required"}
            # 100行信用卡处理逻辑...
            result = self._call_stripe(card_number, amount)
            if result["status"] == "success":
                self.db.execute(f"UPDATE orders SET status='paid' WHERE id='{order_id}'")
                self.db.execute(f"INSERT INTO payments ...")
                # 发送邮件通知
                import smtplib
                server = smtplib.SMTP("smtp.example.com")
                server.sendmail("noreply@shop.com", order["email"], "Payment received")
                return {"success": True}
            else:
                return {"success": False, "error": result["error"]}
        elif payment_type == "alipay":
            # 又是100行支付宝逻辑...
            pass
        elif payment_type == "wechat":
            # 又是100行微信逻辑...
            pass
        else:
            return {"success": False, "error": "Unknown payment type"}
```

### 重构后：清晰的模式应用

```python
# domain/models.py - 值对象和实体
from dataclasses import dataclass
from enum import Enum, auto

class PaymentMethod(Enum):
    CREDIT_CARD = auto()
    ALIPAY = auto()
    WECHAT_PAY = auto()

@dataclass(frozen=True)
class Money:
    amount: float
    currency: str = "CNY"

    def __post_init__(self):
        if self.amount < 0:
            raise ValueError("Amount cannot be negative")

@dataclass
class PaymentResult:
    success: bool
    transaction_id: str | None = None
    error: str | None = None


# domain/gateway.py - 策略模式：支付网关
from typing import Protocol

class PaymentGateway(Protocol):
    """支付网关协议——每种支付方式一个实现"""
    def charge(self, amount: Money, credentials: dict) -> PaymentResult: ...
    def refund(self, transaction_id: str, amount: Money) -> PaymentResult: ...

class StripeGateway:
    def __init__(self, api_key: str) -> None:
        self._api_key = api_key

    def charge(self, amount: Money, credentials: dict) -> PaymentResult:
        card = credentials.get("card_number")
        if not card:
            return PaymentResult(success=False, error="Card number required")
        # 调用Stripe API
        return PaymentResult(success=True, transaction_id="stripe_txn_001")

    def refund(self, transaction_id: str, amount: Money) -> PaymentResult:
        return PaymentResult(success=True, transaction_id=f"refund_{transaction_id}")

class AlipayGateway:
    def __init__(self, app_id: str, private_key: str) -> None:
        self._app_id = app_id
        self._private_key = private_key

    def charge(self, amount: Money, credentials: dict) -> PaymentResult:
        return PaymentResult(success=True, transaction_id="alipay_txn_001")

    def refund(self, transaction_id: str, amount: Money) -> PaymentResult:
        return PaymentResult(success=True, transaction_id=f"refund_{transaction_id}")

class WechatPayGateway:
    def __init__(self, mch_id: str, api_key: str) -> None:
        self._mch_id = mch_id
        self._api_key = api_key

    def charge(self, amount: Money, credentials: dict) -> PaymentResult:
        return PaymentResult(success=True, transaction_id="wechat_txn_001")

    def refund(self, transaction_id: str, amount: Money) -> PaymentResult:
        return PaymentResult(success=True, transaction_id=f"refund_{transaction_id}")


# domain/repository.py - 仓储模式
class OrderRepository(Protocol):
    def get(self, order_id: str) -> dict | None: ...
    def update_status(self, order_id: str, status: str) -> None: ...

class PaymentRepository(Protocol):
    def save(self, payment: dict) -> None: ...
    def get_by_order(self, order_id: str) -> dict | None: ...


# domain/notification.py - 观察者模式：支付事件通知
from collections import defaultdict
from typing import Callable, Any

class PaymentEventBus:
    def __init__(self) -> None:
        self._handlers: dict[str, list[Callable]] = defaultdict(list)

    def subscribe(self, event: str, handler: Callable) -> None:
        self._handlers[event].append(handler)

    def publish(self, event: str, data: dict[str, Any]) -> None:
        for handler in self._handlers.get(event, []):
            try:
                handler(data)
            except Exception as e:
                # 通知失败不应影响支付流程
                print(f"Handler error: {e}")

def email_notification_handler(data: dict) -> None:
    print(f"Email: Payment {data['transaction_id']} for order {data['order_id']}")

def inventory_update_handler(data: dict) -> None:
    print(f"Inventory: Release reserved items for order {data['order_id']}")

def audit_log_handler(data: dict) -> None:
    print(f"Audit: Payment event recorded for order {data['order_id']}")


# application/service.py - 外观模式：统一支付服务入口
class PaymentService:
    """支付服务：协调网关、仓储和事件"""
    def __init__(
        self,
        gateways: dict[PaymentMethod, PaymentGateway],
        order_repo: OrderRepository,
        payment_repo: PaymentRepository,
        event_bus: PaymentEventBus,
    ) -> None:
        self._gateways = gateways
        self._orders = order_repo
        self._payments = payment_repo
        self._events = event_bus

    def process_payment(
        self,
        order_id: str,
        method: PaymentMethod,
        credentials: dict,
    ) -> PaymentResult:
        # 1. 查找订单
        order = self._orders.get(order_id)
        if order is None:
            return PaymentResult(success=False, error="Order not found")

        # 2. 选择支付网关（策略模式）
        gateway = self._gateways.get(method)
        if gateway is None:
            return PaymentResult(success=False, error=f"Unsupported: {method.name}")

        # 3. 执行支付
        amount = Money(order["total"])
        result = gateway.charge(amount, credentials)

        if not result.success:
            return result

        # 4. 持久化（仓储模式）
        self._orders.update_status(order_id, "paid")
        self._payments.save({
            "order_id": order_id,
            "transaction_id": result.transaction_id,
            "amount": amount.amount,
            "method": method.name,
        })

        # 5. 发布事件（观察者模式）
        self._events.publish("payment.completed", {
            "order_id": order_id,
            "transaction_id": result.transaction_id,
            "amount": amount.amount,
        })

        return result


# composition_root.py - 组合根：在入口点组装所有依赖
def create_payment_service(config: dict) -> PaymentService:
    """依赖注入：在应用入口组装整个对象图"""
    gateways = {
        PaymentMethod.CREDIT_CARD: StripeGateway(config["stripe_key"]),
        PaymentMethod.ALIPAY: AlipayGateway(config["alipay_app_id"], config["alipay_key"]),
        PaymentMethod.WECHAT_PAY: WechatPayGateway(config["wechat_mch_id"], config["wechat_key"]),
    }

    event_bus = PaymentEventBus()
    event_bus.subscribe("payment.completed", email_notification_handler)
    event_bus.subscribe("payment.completed", inventory_update_handler)
    event_bus.subscribe("payment.completed", audit_log_handler)

    # order_repo和payment_repo从实际数据库创建
    # 此处省略，实际使用SQLOrderRepository等

    return PaymentService(
        gateways=gateways,
        order_repo=order_repo,
        payment_repo=payment_repo,
        event_bus=event_bus,
    )
```

### 重构收益总结

| 维度 | 重构前 | 重构后 |
|------|--------|--------|
| 可测试性 | 无法单测（依赖全局单例） | 每个组件独立可测 |
| 扩展性 | 添加支付方式需修改核心类 | 新增Gateway实现即可 |
| 安全性 | SQL注入、硬编码凭证 | 参数化查询、配置注入 |
| 职责清晰度 | 单文件800行 | 每个模块不超过100行 |
| 通知扩展 | 硬编码邮件发送 | 订阅事件即可添加通知渠道 |
| 应用模式 | 无 | 策略、仓储、观察者、外观、DI |

---

## Agent Checklist

在代码审查和架构设计中，使用以下检查清单评估设计模式的应用质量：

### 创建型模式检查
- [ ] 工厂方法是否使用字典注册表而非冗长的if-elif链
- [ ] 单例实现是否优先使用模块级变量
- [ ] 单例是否可以被依赖注入替代以提高可测试性
- [ ] Builder是否在构造函数参数超过5个或有构造验证时才使用
- [ ] 原型模式是否正确使用了`copy.deepcopy`处理可变嵌套对象

### 结构型模式检查
- [ ] 装饰器是否使用了`@functools.wraps`保留元信息
- [ ] 装饰器堆叠顺序是否正确（最内层先执行）
- [ ] 适配器是否使用组合而非继承
- [ ] 外观是否只封装常见工作流，而非试图代替子系统全部功能
- [ ] 代理与装饰器是否正确区分（控制访问 vs 增强功能）

### 行为型模式检查
- [ ] 策略模式是否优先使用函数/callable而非单方法类
- [ ] 观察者/事件总线是否有防止内存泄漏的措施（取消订阅、weakref）
- [ ] 命令模式的undo是否捕获了执行时的完整状态
- [ ] 状态机的转换是否显式定义且不允许非法跳跃
- [ ] 责任链中的闭包是否正确绑定了变量（使用默认参数）

### Python特有模式检查
- [ ] Mixin类是否遵守了"不独立实例化"原则
- [ ] 描述符是否使用了`__set_name__`自动获取属性名
- [ ] 元类是否可以用`__init_subclass__`替代
- [ ] 上下文管理器的`__exit__`是否正确处理了异常（不意外吞掉）
- [ ] 异步代码是否避免了在async函数中调用阻塞IO

### 架构模式检查
- [ ] 仓储接口是否面向领域语言而非SQL语义
- [ ] 工作单元是否在事务边界内正确管理了提交和回滚
- [ ] CQRS是否只在读写模型确实不同时才使用
- [ ] 事件溯源的事件schema是否有版本管理
- [ ] DDD值对象是否不可变（`frozen=True`）

### 反模式检查
- [ ] 是否存在超过500行的单个类文件（God Object信号）
- [ ] 是否存在超过4层的if-else嵌套（Spaghetti信号）
- [ ] 是否存在只有一个实现的抽象接口（过度设计信号）
- [ ] 是否存在函数内部直接调用`.instance()`的单例访问（隐式依赖信号）
- [ ] 抽象是否在看到至少三个相似场景后才引入（Rule of Three）
