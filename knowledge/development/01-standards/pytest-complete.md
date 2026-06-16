---
id: pytest-complete
title: Pytest完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, pytest, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Pytest完整指南

## 概述
Pytest是Python最流行的测试框架,简洁强大,支持参数化、fixtures、插件等。本指南覆盖测试编写、fixtures、参数化和最佳实践。

## 核心概念

### 1. 基础测试

**简单测试**:
```python
# test_calculator.py
def add(a, b):
    return a + b

def test_add():
    assert add(2, 3) == 5
    assert add(-1, 1) == 0
    assert add(0, 0) == 0

def test_add_type_error():
    import pytest
    with pytest.raises(TypeError):
        add("2", 3)
```

**运行测试**:
```bash
pytest                    # 运行所有测试
pytest test_calculator.py # 运行单个文件
pytest -v                 # 详细输出
pytest -x                 # 第一次失败后停止
pytest -k "add"           # 运行匹配的测试
pytest --cov=myapp        # 代码覆盖率
```

### 2. Fixtures

**基础fixture**:
```python
import pytest
from myapp.database import Database

@pytest.fixture
def db():
    """数据库fixture"""
    database = Database(':memory:')
    database.create_tables()
    yield database
    database.close()

def test_insert_user(db):
    db.insert_user('Alice', 'alice@example.com')
    user = db.get_user('Alice')
    assert user.email == 'alice@example.com'
```

**fixture作用域**:
```python
@pytest.fixture(scope='function')  # 默认,每个测试函数
def func_fixture():
    return {}

@pytest.fixture(scope='class')  # 每个测试类
class TestClass:
    def test_1(self, class_fixture):
        pass

@pytest.fixture(scope='module')  # 每个模块
@pytest.fixture(scope='session')  # 整个会话
```

**fixture依赖**:
```python
@pytest.fixture
def config():
    return {'debug': True}

@pytest.fixture
def client(config):
    from myapp import create_app
    app = create_app(config)
    return app.test_client()

def test_home(client):
    response = client.get('/')
    assert response.status_code == 200
```

**autouse fixture**:
```python
@pytest.fixture(autouse=True)
def setup_teardown():
    # 每个测试前自动执行
    print("Setup")
    yield
    print("Teardown")
```

### 3. 参数化

**参数化测试**:
```python
@pytest.mark.parametrize("input,expected", [
    (2, 4),
    (3, 9),
    (4, 16),
    (5, 25),
])
def test_square(input, expected):
    assert input ** 2 == expected

@pytest.mark.parametrize("a,b,expected", [
    (1, 2, 3),
    (5, 5, 10),
    (-1, 1, 0),
])
def test_add(a, b, expected):
    assert add(a, b) == expected
```

**多参数化**:
```python
@pytest.mark.parametrize("x", [1, 2, 3])
@pytest.mark.parametrize("y", [10, 20])
def test_multiply(x, y):
    assert x * y == x * y  # 生成6个测试
```

### 4. 标记(Marks)

**跳过测试**:
```python
import pytest

@pytest.mark.skip(reason="Not implemented yet")
def test_future_feature():
    pass

@pytest.mark.skipif(sys.version_info < (3, 10), reason="Requires Python 3.10+")
def test_new_syntax():
    pass
```

**自定义标记**:
```python
@pytest.mark.slow
def test_slow_operation():
    time.sleep(10)

# pytest.ini
[pytest]
markers = slow: marks tests as slow
```

**运行标记测试**:
```bash
pytest -m slow        # 只运行slow测试
pytest -m "not slow"  # 运行非slow测试
```

### 5. 测试异常

```python
import pytest

def test_zero_division():
    with pytest.raises(ZeroDivisionError):
        1 / 0

def test_exception_message():
    with pytest.raises(ValueError, match="invalid value"):
        raise ValueError("invalid value")

def test_exception_attrs():
    with pytest.raises(Exception) as exc_info:
        raise ValueError("error message")
    
    assert str(exc_info.value) == "error message"
```

### 6. Mock和Patch

**使用unittest.mock**:
```python
from unittest.mock import Mock, patch
import pytest

def test_api_call():
    with patch('requests.get') as mock_get:
        mock_get.return_value.json.return_value = {'data': 'test'}
        
        result = fetch_data('http://api.example.com')
        
        assert result == {'data': 'test'}
        mock_get.assert_called_once_with('http://api.example.com')

@pytest.fixture
def mock_db():
    with patch('myapp.database.Database') as mock:
        yield mock
```

### 7. 测试组织

**测试类**:
```python
class TestUser:
    @pytest.fixture(autouse=True)
    def setup(self):
        self.user = User('Alice', 'alice@example.com')
    
    def test_name(self):
        assert self.user.name == 'Alice'
    
    def test_email(self):
        assert self.user.email == 'alice@example.com'
```

**测试目录结构**:
```
tests/
├── conftest.py      # 共享fixtures
├── unit/
│   ├── test_user.py
│   └── test_product.py
├── integration/
│   └── test_api.py
└── fixtures/
    └── sample_data.json
```

## 最佳实践

### ✅ DO

1. **使用描述性测试名称**
```python
# ✅ 好
def test_user_cannot_register_with_duplicate_email():
    pass

# ❌ 差
def test_register():
    pass
```

2. **一个测试一个断言**
```python
# ✅ 好
def test_user_creation():
    user = User('Alice')
    assert user.name == 'Alice'

def test_user_default_active():
    user = User('Alice')
    assert user.is_active == True

# ❌ 差
def test_user():
    user = User('Alice')
    assert user.name == 'Alice'
    assert user.is_active == True
```

3. **使用fixtures共享设置**
```python
# ✅ 好
@pytest.fixture
def sample_user():
    return User('Alice', 'alice@example.com')

def test_user_name(sample_user):
    assert sample_user.name == 'Alice'
```

### ❌ DON'T

1. **不要使用全局状态**
```python
# ❌ 差
global_var = 0

def test_1():
    global global_var
    global_var += 1

def test_2():
    assert global_var == 0  # 依赖执行顺序
```

2. **不要测试实现细节**
```python
# ❌ 差
def test_private_method():
    obj = MyClass()
    assert obj._internal_method() == 42

# ✅ 好
def test_public_behavior():
    obj = MyClass()
    assert obj.calculate() == 42
```

## 学习路径

### 初级 (1周)
1. 基础测试编写
2. assert断言
3. pytest运行

### 中级 (1-2周)
1. fixtures
2. 参数化
3. 标记

### 高级 (2-3周)
1. Mock和patch
2. 插件开发
3. 性能测试

---

**知识ID**: `pytest-complete`  
**领域**: development  
**类型**: standards  
**难度**: intermediate  
**质量分**: 94  
**维护者**: dev-team@superdev.com  
**最后更新**: 2026-03-28
