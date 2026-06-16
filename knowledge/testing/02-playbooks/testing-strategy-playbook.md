---
id: testing-strategy-playbook
title: 测试策略实战手册（金字塔模型 + 现代实践）
domain: testing
category: 02-playbooks
difficulty: advanced
tags: [testing, unit, integration, e2e, test-pyramid, coverage, mocking, tdd, bdd, playwright, jest, enterprise]
quality_score: 93
maintainer: qa-team@superdev.com
last_updated: 2026-06-15
---

# 测试策略实战手册

> 基于 [TestRail Testing Pyramid](https://www.testrail.com/blog/testing-pyramid/) + [CircleCI Strategy](https://circleci.com/blog/testing-pyramid/) + [Bunnyshell E2E 2025](https://www.bunnyshell.com/blog/best-practices-for-end-to-end-testing-in-2025/)

## 测试金字塔

```
          /\
         /E2E\          少量（关键路径，< 5%）
        /------\
       /Integra-\        适量（服务边界，~25%）
      /  tion   \
     /------------\
    /    Unit      \      大量（业务逻辑，~70%）
   /----------------\
```

### Unit Tests（单元测试）
- 测什么：纯函数、业务逻辑、数据转换
- 工具：Jest/Vitest（JS）, pytest（Python）, `#[test]`（Rust）
- 速度：< 100ms/测试
- Mock：外部依赖（DB/API）全部 mock
```typescript
// ✅ 纯函数测试（快、确定、无副作用）
describe('calculateTotal', () => {
  it('sums line items with tax', () => {
    const items = [{ price: 100, qty: 2 }, { price: 50, qty: 1 }];
    expect(calculateTotal(items, 0.1)).toBe(275);
  });
});
```

### Integration Tests（集成测试）
- 测什么：模块间交互、API 端点、DB 查询
- 工具：Supertest（API）, Testcontainers（DB）
- 速度：< 2s/测试
- 用真实 DB（测试容器）而非 mock
```typescript
// ✅ API 端点集成测试（真实 DB container）
describe('POST /api/orders', () => {
  it('creates an order and returns 201', async () => {
    const res = await request(app)
      .post('/api/orders')
      .set('Authorization', `Bearer ${token}`)
      .send({ productId: 'p1', quantity: 2 });
    expect(res.status).toBe(201);
    expect(res.body.id).toBeDefined();
    // 验证 DB 确实写入了
    const order = await db.query(Order).findById(res.body.id);
    expect(order).toBeTruthy();
  });
});
```

### E2E Tests（端到端测试）
- 测什么：完整用户流程（注册→下单→支付）
- 工具：Playwright/Cypress
- 速度：< 30s/测试
- 少而精——只测关键业务路径
```typescript
// ✅ Playwright E2E（真实浏览器）
test('user can place an order', async ({ page }) => {
  await page.goto('/login');
  await page.fill('[name=email]', 'test@example.com');
  await page.fill('[name=password]', 'password');
  await page.click('button[type=submit]');
  await page.click('text=Products');
  await page.click('text=Add to Cart');
  await page.click('text=Checkout');
  await expect(page.locator('text=Order confirmed')).toBeVisible();
});
```

## 测试原则

1. **Arrange-Act-Assert** — 每个测试三段式
2. **一个测试一个断言重点** — 失败时立刻知道哪里错
3. **测试独立** — 不依赖其他测试的执行顺序
4. **确定性** — 同样输入永远同样结果（不依赖时间/随机/网络）
5. **Fast feedback** — Unit 全量 < 10s，CI 全量 < 5min

## Mock 策略

```typescript
// ✅ Mock 边界（外部依赖），不 mock 内部逻辑
// Mock 的：数据库、HTTP API、文件系统、时间
// 不 Mock 的：被测函数本身、纯业务逻辑

// ❌ 过度 mock（测了 mock 不是代码）
it('creates order', async () => {
  db.insert = jest.fn().mockReturnValue({ id: 1 });  // mock 了 DB
  const result = await createOrder({ productId: 'p1' });
  expect(result.id).toBe(1);  // 这只测了 mock 返回什么...
});

// ✅ 合理 mock（只 mock 外部边界）
it('creates order with valid data', async () => {
  // 用 testcontainer 真实 DB
  const result = await createOrder({ productId: 'p1', quantity: 2 });
  expect(result.id).toBeDefined();
  expect(result.status).toBe('pending');
});
```

## 覆盖率目标

| 层级 | 目标 | 说明 |
|------|------|------|
| 行覆盖率 | ≥ 80% | 每行代码至少被测一次 |
| 分支覆盖率 | ≥ 70% | 每个 if/else 分支 |
| 关键路径 | 100% | 支付/认证/权限 |
| 工具类 | ≥ 90% | 纯函数工具 |

**覆盖率不是目的**——80% 覆盖率但测试都是无意义的 `expect(true)` 毫无价值。测**关键行为**比追求数字重要。
