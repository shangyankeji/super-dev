---
id: python-testing-strategies
title: Python测试策略完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [agent, checklist, development, python, strategies, testing, 实战代码示例, 常见陷阱]
quality_score: 70
last_updated: 2026-06-15
---
# Python测试策略完整指南

## 概述
Python测试是保障代码质量的核心环节。本指南覆盖pytest/unittest/mock/fixture/参数化/覆盖率/CI集成等完整测试体系,帮助团队建立从单元测试到集成测试的全链路质量保障。

## 核心概念

### 1. 测试金字塔
- **单元测试(70%)**: 测试最小功能单元,执行快、隔离性强
- **集成测试(20%)**: 测试模块间交互,验证组合行为
- **端到端测试(10%)**: 模拟真实用户行为,覆盖完整流程

### 2. 测试框架选型

| 框架 | 优势 | 劣势 | 适用场景 |
|------|------|------|----------|
| pytest | 简洁、插件丰富、fixture强大 | 学习曲线稍高 | 推荐首选 |
| unittest | 标准库、兼容性好 | 样板代码多 | 遗留项目 |
| doctest | 文档即测试 | 功能有限 | 简单函数验证 |
| hypothesis | 属性测试、自动生成用例 | 理解成本高 | 边界探索 |

### 3. 测试原则
- **FIRST**: Fast/Independent/Repeatable/Self-validating/Timely
- **AAA模式**: Arrange(准备) → Act(执行) → Assert(断言)
- **单一职责**: 每个测试只验证一个行为
- **测试隔离**: 测试间不应有顺序依赖

## 实战代码示例

### pytest基础用法

```python
# tests/test_user_service.py
import pytest
from datetime import datetime
from app.services.user import UserService, UserNotFoundError

class TestUserService:
    """用户服务测试"""

    def test_create_user_success(self):
        """测试成功创建用户"""
        service = UserService()
        user = service.create_user(
            name="Alice",
            email="alice@example.com"
        )
        assert user.name == "Alice"
        assert user.email == "alice@example.com"
        assert user.created_at is not None

    def test_create_user_duplicate_email_raises(self):
        """测试重复邮箱抛出异常"""
        service = UserService()
        service.create_user(name="Alice", email="alice@example.com")
        with pytest.raises(ValueError, match="Email already exists"):
            service.create_user(name="Bob", email="alice@example.com")

    def test_get_user_not_found(self):
        """测试用户不存在"""
        service = UserService()
        with pytest.raises(UserNotFoundError):
            service.get_user(user_id=999)
```

### Fixture系统

```python
# tests/conftest.py
import pytest
from unittest.mock import AsyncMock
from app.db import Database
from app.services.user import UserService
from app.models import User

@pytest.fixture
def db():
    """提供测试数据库连接"""
    database = Database(url="sqlite:///:memory:")
    database.create_tables()
    yield database
    database.drop_tables()

@pytest.fixture
def user_service(db):
    """提供注入了测试DB的用户服务"""
    return UserService(db=db)

@pytest.fixture
def sample_user(user_service):
    """提供预置测试用户"""
    return user_service.create_user(
        name="TestUser",
        email="test@example.com"
    )

@pytest.fixture(scope="session")
def app_config():
    """会话级别配置,所有测试共享"""
    return {
        "db_url": "sqlite:///:memory:",
        "secret_key": "test-secret",
        "debug": True,
    }

@pytest.fixture(autouse=True)
def reset_caches():
    """每个测试自动清除缓存"""
    yield
    import app.cache
    app.cache.clear_all()

# 异步fixture
@pytest.fixture
async def async_client():
    """异步HTTP客户端"""
    from httpx import AsyncClient
    from app.main import app
    async with AsyncClient(app=app, base_url="http://test") as client:
        yield client
```

### 参数化测试

```python
import pytest

# 基础参数化
@pytest.mark.parametrize("input_val,expected", [
    ("hello", 5),
    ("", 0),
    ("hello world", 11),
    ("   ", 3),
])
def test_string_length(input_val, expected):
    assert len(input_val) == expected

# 多参数组合
@pytest.mark.parametrize("a,b,expected", [
    (1, 2, 3),
    (-1, 1, 0),
    (0, 0, 0),
    (100, -50, 50),
])
def test_add(a, b, expected):
    assert add(a, b) == expected

# ID标注提升可读性
@pytest.mark.parametrize("email,is_valid", [
    pytest.param("user@example.com", True, id="normal-email"),
    pytest.param("user@sub.domain.com", True, id="subdomain"),
    pytest.param("invalid", False, id="no-at-sign"),
    pytest.param("@example.com", False, id="no-local-part"),
    pytest.param("user@", False, id="no-domain"),
    pytest.param("", False, id="empty-string"),
], ids=str)
def test_email_validation(email, is_valid):
    assert validate_email(email) == is_valid

# 参数化fixture
@pytest.fixture(params=["sqlite", "postgresql", "mysql"])
def database_engine(request):
    engine = create_engine(request.param)
    yield engine
    engine.dispose()
```

### Mock与Patch

```python
from unittest.mock import Mock, patch, MagicMock, AsyncMock, call
import pytest

class TestPaymentService:

    def test_process_payment_calls_gateway(self):
        """验证支付流程调用了网关"""
        gateway = Mock()
        gateway.charge.return_value = {"status": "success", "tx_id": "abc123"}

        service = PaymentService(gateway=gateway)
        result = service.process_payment(amount=100, currency="USD")

        gateway.charge.assert_called_once_with(
            amount=100, currency="USD"
        )
        assert result["tx_id"] == "abc123"

    def test_payment_retry_on_timeout(self):
        """验证超时后重试逻辑"""
        gateway = Mock()
        gateway.charge.side_effect = [
            TimeoutError("Gateway timeout"),
            {"status": "success", "tx_id": "retry123"},
        ]

        service = PaymentService(gateway=gateway)
        result = service.process_payment(amount=50, currency="EUR")

        assert gateway.charge.call_count == 2
        assert result["tx_id"] == "retry123"

    @patch("app.services.payment.send_notification")
    def test_sends_receipt_after_payment(self, mock_notify):
        """验证支付后发送通知"""
        gateway = Mock()
        gateway.charge.return_value = {"status": "success"}

        service = PaymentService(gateway=gateway)
        service.process_payment(amount=200, currency="CNY")

        mock_notify.assert_called_once()
        args = mock_notify.call_args
        assert args[1]["amount"] == 200

    @patch("app.services.payment.datetime")
    def test_payment_timestamp(self, mock_dt):
        """Mock时间确保一致性"""
        from datetime import datetime
        mock_dt.now.return_value = datetime(2025, 1, 15, 10, 30, 0)
        mock_dt.side_effect = lambda *a, **kw: datetime(*a, **kw)

        gateway = Mock()
        gateway.charge.return_value = {"status": "success"}
        service = PaymentService(gateway=gateway)
        result = service.process_payment(amount=100, currency="USD")

        assert result["timestamp"] == "2025-01-15T10:30:00"

# 异步Mock
class TestAsyncService:

    @pytest.mark.asyncio
    async def test_async_fetch(self):
        """测试异步HTTP调用"""
        mock_client = AsyncMock()
        mock_client.get.return_value = Mock(
            status_code=200,
            json=Mock(return_value={"data": "test"})
        )

        service = DataFetcher(client=mock_client)
        result = await service.fetch("/api/data")

        assert result == {"data": "test"}
        mock_client.get.assert_awaited_once_with("/api/data")
```

### Hypothesis属性测试

```python
from hypothesis import given, strategies as st, assume, settings

@given(st.lists(st.integers()))
def test_sort_preserves_length(xs):
    """排序不改变列表长度"""
    assert len(sorted(xs)) == len(xs)

@given(st.lists(st.integers(), min_size=1))
def test_sort_first_element_is_min(xs):
    """排序后首元素为最小值"""
    assert sorted(xs)[0] == min(xs)

@given(st.text(min_size=1, max_size=100))
def test_encode_decode_roundtrip(s):
    """编解码往返一致性"""
    encoded = encode(s)
    decoded = decode(encoded)
    assert decoded == s

@given(st.dictionaries(
    keys=st.text(min_size=1, max_size=20),
    values=st.integers(min_value=0, max_value=10000),
    min_size=1,
))
@settings(max_examples=200)
def test_cart_total_non_negative(items):
    """购物车总价始终非负"""
    cart = ShoppingCart(items)
    assert cart.total() >= 0
```

### 测试覆盖率配置

```ini
# pyproject.toml
[tool.pytest.ini_options]
testpaths = ["tests"]
python_files = ["test_*.py"]
python_classes = ["Test*"]
python_functions = ["test_*"]
addopts = "-v --tb=short --strict-markers"
markers = [
    "slow: 标记为慢速测试",
    "integration: 集成测试",
    "e2e: 端到端测试",
]
asyncio_mode = "auto"

[tool.coverage.run]
source = ["app"]
branch = true
omit = [
    "*/tests/*",
    "*/migrations/*",
    "*/__main__.py",
]

[tool.coverage.report]
fail_under = 80
show_missing = true
exclude_lines = [
    "pragma: no cover",
    "def __repr__",
    "if __name__ == .__main__.",
    "raise NotImplementedError",
    "pass",
    "if TYPE_CHECKING:",
]
```

### CI集成示例

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ["3.10", "3.11", "3.12"]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python-version }}
      - name: Install dependencies
        run: |
          pip install -e ".[dev]"
      - name: Run linting
        run: ruff check .
      - name: Run unit tests
        run: pytest tests/unit/ -v --junitxml=junit.xml
      - name: Run integration tests
        run: pytest tests/integration/ -v --timeout=120
      - name: Coverage report
        run: |
          pytest --cov=app --cov-report=xml --cov-report=term-missing
      - uses: codecov/codecov-action@v4
        with:
          file: coverage.xml
```

### 测试数据工厂

```python
# tests/factories.py
from dataclasses import dataclass, field
from datetime import datetime, timedelta
import uuid

@dataclass
class UserFactory:
    """用户测试数据工厂"""
    name: str = "Test User"
    email: str = field(default_factory=lambda: f"user-{uuid.uuid4().hex[:8]}@test.com")
    age: int = 25
    is_active: bool = True
    created_at: datetime = field(default_factory=datetime.now)

    def build(self, **overrides) -> dict:
        data = {
            "name": self.name,
            "email": self.email,
            "age": self.age,
            "is_active": self.is_active,
            "created_at": self.created_at,
        }
        data.update(overrides)
        return data

    def build_batch(self, count: int, **overrides) -> list[dict]:
        return [
            self.build(
                email=f"user-{i}-{uuid.uuid4().hex[:4]}@test.com",
                **overrides
            )
            for i in range(count)
        ]

# 使用
def test_bulk_import():
    factory = UserFactory()
    users = factory.build_batch(100, is_active=True)
    result = user_service.bulk_import(users)
    assert result.imported == 100
    assert result.errors == 0
```

### 数据库测试模式

```python
import pytest
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from app.models import Base

@pytest.fixture(scope="session")
def engine():
    """创建测试数据库引擎"""
    engine = create_engine("sqlite:///:memory:")
    Base.metadata.create_all(engine)
    yield engine
    Base.metadata.drop_all(engine)

@pytest.fixture
def db_session(engine):
    """每个测试使用独立事务并回滚"""
    connection = engine.connect()
    transaction = connection.begin()
    session = sessionmaker(bind=connection)()

    yield session

    session.close()
    transaction.rollback()
    connection.close()

def test_create_order(db_session):
    """测试创建订单(自动回滚)"""
    order = Order(user_id=1, total=99.99)
    db_session.add(order)
    db_session.flush()

    assert order.id is not None
    assert db_session.query(Order).count() == 1
```

## 最佳实践

### 1. 测试命名规范
- 文件: `test_<module>.py`
- 类: `Test<Feature>`
- 方法: `test_<action>_<scenario>_<expected_result>`
- 示例: `test_login_with_invalid_password_returns_401`

### 2. 测试组织结构
```
tests/
├── conftest.py          # 全局fixture
├── factories.py         # 数据工厂
├── unit/                # 单元测试
│   ├── conftest.py
│   ├── test_models.py
│   └── test_services.py
├── integration/         # 集成测试
│   ├── conftest.py
│   ├── test_api.py
│   └── test_database.py
└── e2e/                 # 端到端测试
    ├── conftest.py
    └── test_workflows.py
```

### 3. Fixture作用域选择
- `function`(默认): 每个测试函数执行一次,最安全
- `class`: 每个测试类执行一次
- `module`: 每个模块执行一次
- `session`: 整个测试会话执行一次,适合昂贵资源

### 4. Mock边界原则
- Mock外部依赖(HTTP/DB/文件系统/第三方API)
- 不Mock被测对象本身的内部方法
- 优先使用依赖注入而非patch
- Mock越少越好,过度Mock会降低测试价值

### 5. 异步测试规范
- 使用`pytest-asyncio`插件
- fixture和测试函数都可以是async
- 设置`asyncio_mode = "auto"`简化标注

## 常见陷阱

### 陷阱1: 测试间状态泄漏
```python
# 错误: 模块级可变状态
cache = {}

def test_a():
    cache["key"] = "value"

def test_b():
    assert "key" not in cache  # 失败!cache未清理

# 正确: 使用fixture隔离
@pytest.fixture(autouse=True)
def clear_cache():
    cache.clear()
    yield
    cache.clear()
```

### 陷阱2: 过度Mock导致测试无效
```python
# 错误: Mock掉了所有东西,测试毫无意义
def test_process():
    with patch("app.service.validate") as mock_v, \
         patch("app.service.transform") as mock_t, \
         patch("app.service.save") as mock_s:
        mock_v.return_value = True
        mock_t.return_value = {"data": "ok"}
        mock_s.return_value = True
        assert process(data) == True  # 只在测试mock逻辑

# 正确: 只Mock外部边界
def test_process(db_session):
    with patch("app.service.external_api.call") as mock_api:
        mock_api.return_value = {"status": "ok"}
        result = process(real_data, db=db_session)
        assert result.saved_to_db == True
```

### 陷阱3: 忽略异常消息验证
```python
# 错误: 只检查异常类型
def test_invalid_input():
    with pytest.raises(ValueError):
        process(None)

# 正确: 同时验证消息内容
def test_invalid_input():
    with pytest.raises(ValueError, match="Input cannot be None"):
        process(None)
```

### 陷阱4: 硬编码时间导致不稳定
```python
# 错误: 依赖当前时间
def test_token_expiry():
    token = create_token(ttl=3600)
    assert token.expires_at > datetime.now()  # 可能有微秒差异

# 正确: 冻结时间
from freezegun import freeze_time

@freeze_time("2025-01-15 10:00:00")
def test_token_expiry():
    token = create_token(ttl=3600)
    assert token.expires_at == datetime(2025, 1, 15, 11, 0, 0)
```

### 陷阱5: 测试文件路径硬编码
```python
# 错误: 依赖绝对路径
def test_read_config():
    config = load_config("/home/user/project/config.yaml")

# 正确: 使用tmp_path fixture
def test_read_config(tmp_path):
    config_file = tmp_path / "config.yaml"
    config_file.write_text("key: value")
    config = load_config(str(config_file))
    assert config["key"] == "value"
```

## Agent Checklist

### 测试策略审查
- [ ] 测试金字塔比例合理(单元70%/集成20%/E2E 10%)
- [ ] 所有公开API有对应测试用例
- [ ] 错误路径和边界条件有测试覆盖
- [ ] 覆盖率达到项目阈值(通常≥80%)
- [ ] 分支覆盖已启用(branch=true)

### 测试质量检查
- [ ] 测试命名清晰描述了被测行为
- [ ] 每个测试只验证一个行为点
- [ ] 测试间无顺序依赖和状态泄漏
- [ ] Mock仅用于外部依赖边界
- [ ] 参数化覆盖了正常/边界/异常场景

### 测试基础设施
- [ ] conftest.py中定义共享fixture
- [ ] 数据库测试使用事务回滚隔离
- [ ] CI中运行完整测试套件
- [ ] 慢速测试有标记可跳过
- [ ] 异步测试使用pytest-asyncio

### 持续改进
- [ ] 新功能必须附带测试
- [ ] Bug修复必须添加回归测试
- [ ] 定期清理过时和冗余测试
- [ ] 监控测试执行时间并优化慢测试
- [ ] 覆盖率报告集成到PR流程
