---
id: python-complete
title: Python完整知识体系
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, hints, python, type, 基础语法, 异步编程, 核心特性]
quality_score: 70
last_updated: 2026-06-15
---
# Python完整知识体系

## 概述
Python是一门高级、解释型、通用编程语言,强调代码可读性和简洁性。广泛应用于Web开发、数据科学、AI/ML、自动化脚本、DevOps等领域。

## 核心特性

### 1. 语言特性
- **动态类型**: 运行时确定类型,但支持类型提示(Python 3.5+)
- **自动内存管理**: 引用计数 + 垃圾回收
- **多范式**: 支持面向对象、函数式、过程式编程
- **跨平台**: Windows/Linux/macOS一致体验
- **丰富的标准库**: "自带电池"(batteries included)

### 2. Python版本
- **Python 2.x**: 2020年1月1日停止维护,禁止使用
- **Python 3.x**: 当前主流,推荐3.10+(模式匹配、性能优化)
- **Python 3.12+**: 最新稳定版,GIL优化、性能提升

## 基础语法

### 变量与数据类型

```python
# 基本类型
name: str = "Alice"
age: int = 30
height: float = 1.75
is_active: bool = True

# 容器类型
numbers: list[int] = [1, 2, 3, 4, 5]
coordinates: tuple[float, float] = (10.0, 20.0)
person: dict[str, any] = {"name": "Bob", "age": 25}
unique_ids: set[int] = {1, 2, 3, 4}

# None类型
result: None = None
```

### 控制流

```python
# 条件判断
score = 85
if score >= 90:
    grade = "A"
elif score >= 80:
    grade = "B"
else:
    grade = "C"

# 循环
for i in range(10):
    print(i)

# 列表推导式
squares = [x**2 for x in range(10)]

# 字典推导式
word_lengths = {word: len(word) for word in ["hello", "world"]}

# 循环控制
for i in range(10):
    if i == 3:
        continue  # 跳过3
    if i == 7:
        break     # 到7停止
    print(i)
```

### 函数

```python
# 基础函数
def greet(name: str) -> str:
    """问候函数"""
    return f"Hello, {name}!"

# 默认参数
def power(base: int, exp: int = 2) -> int:
    return base ** exp

# 可变参数
def sum_all(*args: int) -> int:
    return sum(args)

# 关键字参数
def create_user(**kwargs: any) -> dict:
    return kwargs

# 类型提示
from typing import List, Dict, Optional, Union

def process_data(
    items: List[int],
    config: Optional[Dict[str, any]] = None
) -> Union[int, str]:
    if config is None:
        return "no config"
    return sum(items)
```

### 类与对象

```python
from dataclasses import dataclass
from typing import ClassVar

# 传统类
class Person:
    species: ClassVar[str] = "Homo sapiens"  # 类变量
    
    def __init__(self, name: str, age: int):
        self.name = name  # 实例变量
        self.age = age
    
    def greet(self) -> str:
        return f"Hi, I'm {self.name}"
    
    @property
    def is_adult(self) -> bool:
        return self.age >= 18
    
    @staticmethod
    def get_species() -> str:
        return Person.species
    
    @classmethod
    def from_dict(cls, data: dict) -> 'Person':
        return cls(data['name'], data['age'])

# Dataclass (推荐)
@dataclass
class User:
    id: int
    username: str
    email: str
    is_active: bool = True
    
    def __post_init__(self):
        self.email = self.email.lower()

# 继承
class Employee(Person):
    def __init__(self, name: str, age: int, employee_id: int):
        super().__init__(name, age)
        self.employee_id = employee_id
```

## 类型系统 (Type Hints)

### 基础类型

```python
from typing import (
    List, Dict, Set, Tuple,
    Optional, Union, Any,
    Callable, TypeVar, Generic
)

# 基础
x: int = 10
y: float = 3.14
name: str = "Alice"

# 容器
numbers: List[int] = [1, 2, 3]
mapping: Dict[str, int] = {"a": 1, "b": 2}
unique: Set[int] = {1, 2, 3}
coord: Tuple[float, float, float] = (1.0, 2.0, 3.0)

# Optional
maybe_int: Optional[int] = None  # 等价于 Union[int, None]

# Union
value: Union[int, str] = 42
value = "forty-two"

# Any
unknown: Any = some_function()
```

### 高级类型

```python
from typing import Literal, TypedDict, Protocol

# Literal类型
Status = Literal["active", "inactive", "pending"]
user_status: Status = "active"

# TypedDict
class UserDict(TypedDict):
    id: int
    name: str
    email: str
    age: NotRequired[int]  # Python 3.11+

user: UserDict = {
    "id": 1,
    "name": "Alice",
    "email": "alice@example.com"
}

# Protocol (结构化子类型)
class Drawable(Protocol):
    def draw(self) -> None: ...

class Circle:
    def draw(self) -> None:
        print("Drawing circle")

def render(obj: Drawable) -> None:
    obj.draw()

render(Circle())  # 无需显式继承
```

### 泛型

```python
T = TypeVar('T')

def first(items: List[T]) -> Optional[T]:
    return items[0] if items else None

class Stack(Generic[T]):
    def __init__(self):
        self._items: List[T] = []
    
    def push(self, item: T) -> None:
        self._items.append(item)
    
    def pop(self) -> Optional[T]:
        return self._items.pop() if self._items else None

# 使用
int_stack: Stack[int] = Stack()
int_stack.push(42)
```

## 异步编程 (Async/Await)

### 基础概念

```python
import asyncio

# 协程定义
async def fetch_data(url: str) -> dict:
    """异步获取数据"""
    await asyncio.sleep(1)  # 模拟I/O
    return {"url": url, "data": "result"}

# 运行协程
async def main():
    result = await fetch_data("https://api.example.com")
    print(result)

# 事件循环
asyncio.run(main())
```

### 并发执行

```python
import asyncio

async def task(name: str, delay: int) -> str:
    await asyncio.sleep(delay)
    return f"Task {name} completed"

async def main():
    # 并发执行多个任务
    results = await asyncio.gather(
        task("A", 1),
        task("B", 2),
        task("C", 1)
    )
    print(results)  # ['Task A completed', 'Task B completed', 'Task C completed']

asyncio.run(main())
```

### 异步上下文管理器

```python
from contextlib import asynccontextmanager

@asynccontextmanager
async def async_resource():
    # 进入
    print("Acquiring resource")
    resource = {"id": 123}
    yield resource
    # 退出
    print("Releasing resource")

async def main():
    async with async_resource() as res:
        print(f"Using {res}")

asyncio.run(main())
```

### 异步生成器

```python
async def async_range(count: int):
    for i in range(count):
        await asyncio.sleep(0.1)
        yield i

async def main():
    async for num in async_range(5):
        print(num)

asyncio.run(main())
```

## 错误处理

### 异常层次

```python
# 基本结构
try:
    result = risky_operation()
except ValueError as e:
    print(f"值错误: {e}")
except TypeError as e:
    print(f"类型错误: {e}")
except Exception as e:
    print(f"未知错误: {e}")
    raise  # 重新抛出
finally:
    cleanup()

# 自定义异常
class InsufficientBalanceError(Exception):
    def __init__(self, balance: float, required: float):
        self.balance = balance
        self.required = required
        super().__init__(
            f"Insufficient balance: {balance} < {required}"
        )

def withdraw(balance: float, amount: float) -> float:
    if balance < amount:
        raise InsufficientBalanceError(balance, amount)
    return balance - amount
```

### 上下文管理器

```python
from contextlib import contextmanager

@contextmanager
def managed_file(path: str, mode: str):
    f = open(path, mode)
    try:
        yield f
    finally:
        f.close()

# 使用
with managed_file("test.txt", "w") as f:
    f.write("Hello")
```

## 文件操作

### 文本文件

```python
# 读取
with open("file.txt", "r", encoding="utf-8") as f:
    content = f.read()       # 全部读取
    lines = f.readlines()    # 逐行读取列表

# 写入
with open("file.txt", "w", encoding="utf-8") as f:
    f.write("Hello\n")
    f.writelines(["Line 1\n", "Line 2\n"])

# 追加
with open("file.txt", "a", encoding="utf-8") as f:
    f.write("New line\n")

# 逐行处理(内存高效)
with open("large.txt", "r") as f:
    for line in f:
        process(line)
```

### 二进制文件

```python
# 读取
with open("image.png", "rb") as f:
    data = f.read()

# 写入
with open("copy.png", "wb") as f:
    f.write(data)
```

### Pathlib (推荐)

```python
from pathlib import Path

# 路径操作
home = Path.home()
project = Path.cwd()
config = project / "config" / "settings.yaml"

# 检查
if config.exists():
    print(f"Config found: {config}")
    
if config.is_file():
    content = config.read_text(encoding="utf-8")

# 创建
(project / "logs").mkdir(exist_ok=True)
(project / "temp.txt").write_text("temporary", encoding="utf-8")

# 遍历
for py_file in project.rglob("*.py"):
    print(py_file)
```

## 标准库常用模块

### collections

```python
from collections import Counter, defaultdict, deque, namedtuple

# Counter - 计数器
words = ["apple", "banana", "apple", "cherry", "apple"]
word_counts = Counter(words)
print(word_counts["apple"])  # 3
print(word_counts.most_common(2))  # [('apple', 3), ('banana', 1)]

# defaultdict - 默认值字典
grouped = defaultdict(list)
for word in words:
    grouped[len(word)].append(word)

# deque - 双端队列
dq = deque([1, 2, 3])
dq.appendleft(0)
dq.pop()
dq.popleft()

# namedtuple - 命名元组
Point = namedtuple('Point', ['x', 'y'])
p = Point(10, 20)
print(p.x, p.y)  # 10 20
```

### datetime

```python
from datetime import datetime, timedelta, timezone

# 当前时间
now = datetime.now()
utc_now = datetime.now(timezone.utc)

# 解析
dt = datetime.strptime("2024-03-28 14:30:00", "%Y-%m-%d %H:%M:%S")

# 格式化
formatted = now.strftime("%Y年%m月%d日 %H:%M")

# 时间运算
tomorrow = now + timedelta(days=1)
next_week = now + timedelta(weeks=1)

# 时区转换
utc_dt = datetime.now(timezone.utc)
beijing_dt = utc_dt.astimezone(timezone(timedelta(hours=8)))
```

### json

```python
import json

# 序列化
data = {"name": "Alice", "age": 30}
json_str = json.dumps(data, indent=2, ensure_ascii=False)

# 反序列化
loaded = json.loads(json_str)

# 文件操作
with open("data.json", "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2, ensure_ascii=False)

with open("data.json", "r", encoding="utf-8") as f:
    loaded = json.load(f)
```

### re (正则表达式)

```python
import re

# 匹配
pattern = r'\b\d{3}-\d{3}-\d{4}\b'  # 电话号码
text = "Call 123-456-7890"
match = re.search(pattern, text)
if match:
    print(match.group())  # 123-456-7890

# 查找所有
emails = re.findall(r'\b[\w.-]+@[\w.-]+\.\w+\b', text)

# 替换
cleaned = re.sub(r'\s+', ' ', "too   many    spaces")

# 分割
parts = re.split(r'[,\s]+', "apple, banana cherry")
```

### logging

```python
import logging

# 配置
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler("app.log"),
        logging.StreamHandler()
    ]
)

logger = logging.getLogger(__name__)

# 使用
logger.debug("Debug message")
logger.info("Info message")
logger.warning("Warning message")
logger.error("Error message")
logger.critical("Critical message")

# 异常记录
try:
    1 / 0
except Exception as e:
    logger.exception("Division failed")  # 自动记录堆栈
```

## 性能优化

### 性能分析

```python
import time
import timeit
from functools import wraps

# 计时装饰器
def timer(func):
    @wraps(func)
    def wrapper(*args, **kwargs):
        start = time.perf_counter()
        result = func(*args, **kwargs)
        end = time.perf_counter()
        print(f"{func.__name__} took {end - start:.4f}s")
        return result
    return wrapper

# timeit
time_taken = timeit.timeit('[x**2 for x in range(1000)]', number=1000)

# cProfile
import cProfile
cProfile.run('expensive_function()')
```

### 优化技巧

```python
# ❌ 慢: 字符串拼接
result = ""
for word in words:
    result += word

# ✅ 快: join
result = "".join(words)

# ❌ 慢: 列表append
result = []
for i in range(10000):
    result = result + [i]

# ✅ 快: append
result = []
for i in range(10000):
    result.append(i)

# ✅ 更快: 列表推导
result = [i for i in range(10000)]

# ❌ 慢: 成员检查(list)
items = [1, 2, 3, 4, 5]
if 3 in items:  # O(n)
    pass

# ✅ 快: 成员检查(set)
items = {1, 2, 3, 4, 5}
if 3 in items:  # O(1)
    pass

# 使用生成器节省内存
def fibonacci(n):
    a, b = 0, 1
    for _ in range(n):
        yield a
        a, b = b, a + b

# 使用内置函数
sum_result = sum(numbers)  # 比循环快
max_result = max(numbers)
min_result = min(numbers)
```

## 测试

### unittest

```python
import unittest

class TestMathOperations(unittest.TestCase):
    def setUp(self):
        self.numbers = [1, 2, 3, 4, 5]
    
    def test_sum(self):
        self.assertEqual(sum(self.numbers), 15)
    
    def test_max(self):
        self.assertEqual(max(self.numbers), 5)
    
    def test_exception(self):
        with self.assertRaises(ZeroDivisionError):
            1 / 0

if __name__ == '__main__':
    unittest.main()
```

### pytest (推荐)

```python
import pytest

# 简单测试
def test_addition():
    assert 1 + 1 == 2

# fixture
@pytest.fixture
def sample_data():
    return [1, 2, 3, 4, 5]

def test_with_fixture(sample_data):
    assert len(sample_data) == 5

# 参数化测试
@pytest.mark.parametrize("input,expected", [
    (1, 1),
    (2, 4),
    (3, 9),
])
def test_square(input, expected):
    assert input ** 2 == expected

# 异常测试
def test_division_by_zero():
    with pytest.raises(ZeroDivisionError):
        1 / 0
```

## 包管理

### requirements.txt

```txt
fastapi==0.110.0
pydantic>=2.0.0,<3.0.0
requests~=2.31.0
```

### pyproject.toml (现代)

```toml
[project]
name = "my-project"
version = "1.0.0"
requires-python = ">=3.10"
dependencies = [
    "fastapi>=0.110.0",
    "pydantic>=2.0.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0.0",
    "black>=23.0.0",
    "ruff>=0.1.0",
]
```

### 虚拟环境

```bash
# venv
python -m venv venv
source venv/bin/activate  # Linux/macOS
venv\Scripts\activate     # Windows

# poetry
poetry install
poetry shell

# uv (最快)
uv venv
source .venv/bin/activate
uv pip install -r requirements.txt
```

## 最佳实践

### ✅ DO

1. **使用类型提示**: 提升代码可读性和IDE支持
```python
def process(items: list[int]) -> dict[str, int]:
    ...
```

2. **遵循PEP 8**: 使用black/ruff自动格式化
```bash
black .
ruff check .
```

3. **写文档字符串**: 解释复杂函数
```python
def calculate_compound_interest(
    principal: float,
    rate: float,
    time: int
) -> float:
    """
    计算复利
    
    Args:
        principal: 本金
        rate: 年利率(小数)
        time: 时间(年)
    
    Returns:
        最终金额
    
    Example:
        >>> calculate_compound_interest(1000, 0.05, 10)
        1628.89
    """
    return principal * (1 + rate) ** time
```

4. **使用上下文管理器**: 确保资源释放
```python
with open("file.txt") as f:
    ...
```

5. **优先使用pathlib**: 而非os.path
```python
from pathlib import Path
config = Path("config") / "settings.yaml"
```

### ❌ DON'T

1. **不要用裸except**
```python
# ❌
try:
    ...
except:
    pass

# ✅
try:
    ...
except ValueError as e:
    logger.error(f"Value error: {e}")
```

2. **不要用`from module import *`**: 污染命名空间

3. **不要在循环中修改列表**
```python
# ❌
for item in items:
    if condition(item):
        items.remove(item)

# ✅
items = [item for item in items if not condition(item)]
```

4. **不要用可变默认参数**
```python
# ❌
def add_item(item, items=[]):
    items.append(item)
    return items

# ✅
def add_item(item, items=None):
    if items is None:
        items = []
    items.append(item)
    return items
```

5. **不要过早优化**: 先测性能再优化

## 常见陷阱

### 1. 可变默认参数
```python
# ❌ Bug
def append_to(element, target=[]):
    target.append(element)
    return target

# 多次调用共享同一个列表
append_to(1)  # [1]
append_to(2)  # [1, 2]  ← Bug!

# ✅ 修复
def append_to(element, target=None):
    if target is None:
        target = []
    target.append(element)
    return target
```

### 2. 延迟绑定闭包
```python
# ❌ Bug
functions = [lambda: i for i in range(3)]
for f in functions:
    print(f())  # 2, 2, 2  ← 都是最后一个值!

# ✅ 修复
functions = [lambda i=i: i for i in range(3)]
for f in functions:
    print(f())  # 0, 1, 2
```

### 3. 整数缓存
```python
# Python缓存小整数(-5到256)
a = 256
b = 256
print(a is b)  # True

a = 257
b = 257
print(a is b)  # False (可能)

# 永远用==比较值,不要用is
```

## 学习路径

### 初级 (0-3个月)
1. 基础语法: 变量、控制流、函数
2. 数据结构: list/dict/set/tuple
3. 文件操作: 读写文件
4. 错误处理: try/except
5. 标准库: os/sys/json/datetime

### 中级 (3-6个月)
1. 面向对象: 类、继承、多态
2. 类型系统: type hints、泛型
3. 异步编程: async/await
4. 装饰器: @decorator
5. 测试: unittest/pytest

### 高级 (6-12个月)
1. 元编程: metaclass、descriptor
2. 性能优化: profiling、Cython
3. 并发: threading、multiprocessing
4. 设计模式: Python实现
5. 内部机制: GIL、垃圾回收

### 专家 (1年+)
1. 解释器: CPython源码
2. 编译器: AST、字节码
3. C扩展: Python C API
4. 语言设计: PEP提案
5. 生态系统: 包开发、分发

## 参考资源

### 官方文档
- [Python官方文档](https://docs.python.org/3/)
- [PEP索引](https://peps.python.org/)
- [Python打包指南](https://packaging.python.org/)

### 推荐书籍
- 《Fluent Python》(第2版)
- 《Effective Python》(第2版)
- 《Python Cookbook》(第3版)

### 在线资源
- [Real Python](https://realpython.com/)
- [Python Morsels](https://pythonmorsels.com/)
- [Talk Python to Me](https://talkpython.fm/)

---

**知识ID**: `python-complete`  
**领域**: development  
**类型**: standards  
**难度**: intermediate  
**质量分**: 90  
**维护者**: dev-team@superdev.com  
**最后更新**: 2026-03-28
