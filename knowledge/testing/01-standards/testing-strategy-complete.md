---
id: testing-strategy-complete
title: 软件测试策略完整指南
domain: testing
category: 01-standards
difficulty: intermediate
tags: [complete, mock, playwright, strategy, testing, 单元测试, 概述, 测试]
quality_score: 70
last_updated: 2026-06-15
---
# 软件测试策略完整指南

## 概述

软件测试是保证代码质量的核心环节。本指南覆盖测试金字塔、测试类型、测试框架选择、TDD/BDD、Mock 策略和测试最佳实践。

---

## 测试金字塔

```
        /  E2E Tests  \          (少量, 慢, 贵)
       / Integration   \         (适量, 中速)
      /   Unit Tests    \        (大量, 快, 便宜)
```

| 层级 | 占比 | 速度 | 成本 | 信心 |
|------|------|------|------|------|
| 单元测试 | 70% | 毫秒 | 低 | 低-中 |
| 集成测试 | 20% | 秒 | 中 | 中-高 |
| E2E测试 | 10% | 分钟 | 高 | 高 |

---

## 单元测试

### Python (pytest)

```python
import pytest
from decimal import Decimal

class PricingEngine:
    def calculate_discount(self, price: Decimal, quantity: int) -> Decimal:
        if quantity >= 100:
            return price * Decimal("0.8")
        elif quantity >= 50:
            return price * Decimal("0.9")
        elif quantity >= 10:
            return price * Decimal("0.95")
        return price

class TestPricingEngine:
    @pytest.fixture
    def engine(self):
        return PricingEngine()

    @pytest.mark.parametrize("price,quantity,expected", [
        (Decimal("100"), 1, Decimal("100")),
        (Decimal("100"), 10, Decimal("95")),
        (Decimal("100"), 50, Decimal("90")),
        (Decimal("100"), 100, Decimal("80")),
        (Decimal("0"), 100, Decimal("0")),
    ])
    def test_calculate_discount(self, engine, price, quantity, expected):
        assert engine.calculate_discount(price, quantity) == expected

    def test_negative_quantity_raises(self, engine):
        with pytest.raises(ValueError):
            engine.calculate_discount(Decimal("100"), -1)
```

### JavaScript (Vitest)

```javascript
import { describe, it, expect, vi, beforeEach } from 'vitest';

class UserService {
  constructor(userRepo, emailService) {
    this.userRepo = userRepo;
    this.emailService = emailService;
  }

  async createUser(name, email) {
    const existing = await this.userRepo.findByEmail(email);
    if (existing) throw new Error('Email already exists');
    const user = await this.userRepo.create({ name, email });
    await this.emailService.sendWelcome(email);
    return user;
  }
}

describe('UserService', () => {
  let service, mockRepo, mockEmail;

  beforeEach(() => {
    mockRepo = {
      findByEmail: vi.fn(),
      create: vi.fn(),
    };
    mockEmail = { sendWelcome: vi.fn() };
    service = new UserService(mockRepo, mockEmail);
  });

  it('creates user and sends welcome email', async () => {
    mockRepo.findByEmail.mockResolvedValue(null);
    mockRepo.create.mockResolvedValue({ id: 1, name: 'Alice', email: 'a@b.com' });

    const user = await service.createUser('Alice', 'a@b.com');

    expect(user.name).toBe('Alice');
    expect(mockRepo.create).toHaveBeenCalledWith({ name: 'Alice', email: 'a@b.com' });
    expect(mockEmail.sendWelcome).toHaveBeenCalledWith('a@b.com');
  });

  it('rejects duplicate email', async () => {
    mockRepo.findByEmail.mockResolvedValue({ id: 1 });

    await expect(service.createUser('Bob', 'a@b.com'))
      .rejects.toThrow('Email already exists');
    expect(mockRepo.create).not.toHaveBeenCalled();
  });
});
```

---

## 集成测试

### API 集成测试 (Python + FastAPI)

```python
import pytest
from httpx import AsyncClient
from app.main import app
from app.database import get_test_db

@pytest.fixture
async def client():
    async with AsyncClient(app=app, base_url="http://test") as ac:
        yield ac

@pytest.mark.asyncio
async def test_create_and_get_user(client):
    # 创建用户
    response = await client.post("/api/users", json={
        "name": "Alice",
        "email": "alice@test.com"
    })
    assert response.status_code == 201
    user_id = response.json()["id"]

    # 获取用户
    response = await client.get(f"/api/users/{user_id}")
    assert response.status_code == 200
    assert response.json()["name"] == "Alice"

@pytest.mark.asyncio
async def test_duplicate_email_returns_409(client):
    await client.post("/api/users", json={"name": "A", "email": "dup@test.com"})
    response = await client.post("/api/users", json={"name": "B", "email": "dup@test.com"})
    assert response.status_code == 409
```

### 数据库集成测试

```python
import pytest
from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession

@pytest.fixture(scope="session")
async def engine():
    engine = create_async_engine("sqlite+aiosqlite:///:memory:")
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
    yield engine
    await engine.dispose()

@pytest.fixture
async def db_session(engine):
    async with AsyncSession(engine) as session:
        async with session.begin():
            yield session
        await session.rollback()  # 每个测试后回滚

@pytest.mark.asyncio
async def test_order_creation(db_session):
    repo = OrderRepository(db_session)
    order = await repo.create(user_id=1, items=[
        {"sku": "ITEM-1", "qty": 2, "price": 29.99}
    ])
    assert order.id is not None
    assert order.total == 59.98
```

---

## E2E 测试 (Playwright)

```python
import pytest
from playwright.sync_api import Page, expect

def test_user_login_flow(page: Page):
    page.goto("http://localhost:3000/login")

    # 填写登录表单
    page.fill("[data-testid=email]", "user@test.com")
    page.fill("[data-testid=password]", "password123")
    page.click("[data-testid=submit]")

    # 验证跳转到仪表盘
    expect(page).to_have_url("http://localhost:3000/dashboard")
    expect(page.locator("[data-testid=welcome]")).to_contain_text("Welcome")

def test_create_order(page: Page):
    # 登录
    login(page, "user@test.com", "password123")

    # 添加商品
    page.goto("http://localhost:3000/products")
    page.click("[data-testid=add-to-cart-ITEM1]")
    page.click("[data-testid=add-to-cart-ITEM2]")

    # 结算
    page.click("[data-testid=cart-icon]")
    expect(page.locator("[data-testid=cart-count]")).to_have_text("2")
    page.click("[data-testid=checkout]")

    # 验证订单创建
    expect(page.locator("[data-testid=order-confirmation]")).to_be_visible()
```

---

## Mock 策略

### 何时 Mock

✅ **应该 Mock**:
- 外部 API 调用（支付网关、第三方服务）
- 发送邮件/短信
- 文件系统 I/O（在单元测试中）
- 当前时间（`datetime.now()`）
- 随机数

❌ **不应该 Mock**:
- 数据库（集成测试中应使用真实数据库）
- 被测对象的内部方法
- 简单的值对象和数据类

### Python Mock 示例

```python
from unittest.mock import AsyncMock, patch, MagicMock
from datetime import datetime

@pytest.mark.asyncio
async def test_send_reminder():
    email_service = AsyncMock()
    email_service.send.return_value = True

    with patch("app.services.datetime") as mock_dt:
        mock_dt.now.return_value = datetime(2026, 3, 28, 10, 0)

        reminder_service = ReminderService(email_service)
        await reminder_service.send_daily_reminders()

        email_service.send.assert_called()
        assert email_service.send.call_count == 3
```

---

## TDD 工作流

```
1. RED:   写一个失败的测试
2. GREEN: 写最少的代码让测试通过
3. REFACTOR: 重构代码，保持测试通过

重复以上循环
```

```python
# Step 1: RED - 写测试
def test_password_validator():
    validator = PasswordValidator()
    assert validator.validate("Abc123!@") == True
    assert validator.validate("short") == False
    assert validator.validate("nouppercase1!") == False

# Step 2: GREEN - 最小实现
class PasswordValidator:
    def validate(self, password: str) -> bool:
        if len(password) < 8:
            return False
        if not any(c.isupper() for c in password):
            return False
        if not any(c.isdigit() for c in password):
            return False
        if not any(c in "!@#$%^&*" for c in password):
            return False
        return True

# Step 3: REFACTOR
class PasswordValidator:
    RULES = [
        (lambda p: len(p) >= 8, "至少8个字符"),
        (lambda p: any(c.isupper() for c in p), "至少一个大写字母"),
        (lambda p: any(c.isdigit() for c in p), "至少一个数字"),
        (lambda p: any(c in "!@#$%^&*" for c in p), "至少一个特殊字符"),
    ]

    def validate(self, password: str) -> bool:
        return all(rule(password) for rule, _ in self.RULES)

    def get_errors(self, password: str) -> list[str]:
        return [msg for rule, msg in self.RULES if not rule(password)]
```

---

## 测试覆盖率

```bash
# Python
pytest --cov=src --cov-report=term-missing --cov-report=html
# 目标: 语句覆盖率 >= 80%, 分支覆盖率 >= 70%

# JavaScript
vitest --coverage
# 或
jest --coverage --coverageThreshold='{"global":{"branches":70,"lines":80}}'
```

### 覆盖率陷阱

❌ **不要追求 100% 覆盖率** — 投入产出比递减
✅ **关注关键路径** — 业务逻辑、错误处理、边界条件
✅ **分支覆盖率比行覆盖率更重要**

---

## 常见反模式

### 1. 测试实现而非行为
```python
# ❌ 测试实现细节
def test_uses_redis_cache():
    service.get_user(1)
    mock_redis.get.assert_called_with("user:1")  # 耦合实现

# ✅ 测试行为
def test_returns_user_data():
    user = service.get_user(1)
    assert user.name == "Alice"
```

### 2. 过度 Mock
```python
# ❌ Mock 一切
def test_process_order():
    mock_validator = Mock()
    mock_calculator = Mock()
    mock_repo = Mock()
    # 测试只验证了 Mock 的调用，没有测试真正的逻辑

# ✅ 只 Mock 外部依赖
def test_process_order():
    mock_payment = Mock()
    service = OrderService(payment=mock_payment)
    order = service.process(items=[...])
    assert order.total == 99.99
```

### 3. 不稳定的测试 (Flaky Tests)
```python
# ❌ 依赖时间
def test_token_expiry():
    token = create_token(expires_in=1)
    time.sleep(2)
    assert token.is_expired()  # 不稳定！

# ✅ 注入时间
def test_token_expiry():
    clock = FakeClock(datetime(2026, 3, 28, 12, 0))
    token = create_token(expires_at=datetime(2026, 3, 28, 11, 0), clock=clock)
    assert token.is_expired()
```

---

## Agent Checklist

Agent 在编写测试时必须检查:

- [ ] 是否遵循测试金字塔（单元70% > 集成20% > E2E10%）？
- [ ] 单元测试是否只测试行为而非实现？
- [ ] 是否只 Mock 外部依赖（API/邮件/文件系统）？
- [ ] 数据库测试是否使用事务回滚保证隔离？
- [ ] 是否覆盖了边界条件和错误路径？
- [ ] 测试是否可以独立运行（不依赖执行顺序）？
- [ ] 是否有清晰的 Arrange-Act-Assert 结构？
- [ ] CI 中是否配置了覆盖率阈值？
- [ ] 是否有 E2E 测试覆盖核心用户流程？
- [ ] 是否避免了 Flaky Tests（依赖时间/网络/顺序）？

---

## 参考资料

- [Testing Library Guiding Principles](https://testing-library.com/docs/guiding-principles)
- [pytest 官方文档](https://docs.pytest.org/)
- [Playwright 官方文档](https://playwright.dev/)
- [Martin Fowler - Test Pyramid](https://martinfowler.com/bliki/TestPyramid.html)

---

**文档版本**: v1.0
**最后更新**: 2026-03-28
**质量评分**: 89/100
