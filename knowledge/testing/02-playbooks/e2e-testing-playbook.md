---
title: E2E 测试作战手册
version: 1.0.0
last_updated: 2026-03-28
owner: qa-team
tags: [e2e-testing, Playwright, Cypress, test-strategy, CI-integration, parallel-execution, stability]
status: production
domain: testing
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：E2E 测试全流程作战手册
# 作用：指导团队建立和维护高质量的 E2E 测试体系
# 创建时间：2026-03-28
# 最后修改：2026-03-28

## 目标

建立 E2E 测试标准化实践，确保：
- 核心业务流程 100% 覆盖自动化 E2E 测试
- 测试稳定性 > 98%（Flaky Rate < 2%）
- CI 中 E2E 测试总耗时 < 15 分钟（并行执行）
- 测试数据自给自足，不依赖共享环境数据
- 测试报告清晰可读，失败原因可追溯（含截图/视频/Trace）

## 适用场景

- Web 应用端到端回归测试
- 跨服务业务流程验证
- 发布前门禁测试
- 新功能验收测试
- 多浏览器/多设备兼容性测试

## 前置条件

### 环境要求

| 项目 | 要求 |
|------|------|
| Node.js | 18+ |
| 浏览器 | Chromium / Firefox / WebKit（Playwright 自动管理）|
| 测试环境 | 独立 Staging 环境，不与手工测试共享 |
| CI 平台 | GitHub Actions / GitLab CI / Jenkins |
| Docker | 用于 CI 中运行浏览器（可选） |

### 工具链安装

```bash
# Playwright（推荐）
npm init playwright@latest
# 安装浏览器
npx playwright install --with-deps

# Cypress（备选）
npm install cypress --save-dev
npx cypress open  # 首次打开安装浏览器

# 项目结构（Playwright）
mkdir -p tests/e2e/{pages,fixtures,helpers}
# tests/e2e/
# ├── pages/           # Page Object Models
# ├── fixtures/        # 测试夹具（数据/配置）
# ├── helpers/         # 工具函数
# ├── specs/           # 测试用例
# │   ├── auth/
# │   ├── order/
# │   └── product/
# └── playwright.config.ts
```

---

## 一、测试策略

### 1.1 测试金字塔与 E2E 定位

```yaml
测试金字塔:
  单元测试（70%）: 快速/廉价/大量
  集成测试（20%）: API 级别/服务间调用
  E2E 测试（10%）: 核心业务流程/用户视角

E2E 测试原则:
  测什么:
    - 核心业务的关键路径（Happy Path）
    - 涉及多个服务协作的流程
    - 用户最常用的操作流程
    - 涉及金钱/权限的关键操作

  不测什么:
    - 单个组件的边界条件（单元测试覆盖）
    - 纯 API 逻辑（集成测试覆盖）
    - 样式/布局细节（视觉回归测试覆盖）
    - 第三方服务内部逻辑
```

### 1.2 用例优先级矩阵

```yaml
P0 - 必须覆盖（阻断发布）:
  - 用户注册与登录
  - 核心业务流程（搜索→浏览→下单→支付）
  - 权限控制（普通用户不能访问管理后台）
  - 数据安全（敏感信息不在页面明文展示）

P1 - 应该覆盖（影响质量评分）:
  - 搜索与筛选
  - 个人信息管理
  - 订单查询与取消
  - 消息通知

P2 - 可以覆盖（提升信心）:
  - 页面导航与面包屑
  - 表单校验提示
  - 空状态展示
  - 分页与排序
```

### 1.3 浏览器与设备覆盖

```yaml
# Playwright 多浏览器配置
覆盖矩阵:
  桌面端:
    - Chromium（占比 ~65%，必测）
    - Firefox（占比 ~5%，P1 流程覆盖）
    - WebKit/Safari（占比 ~20%，P0 流程覆盖）

  移动端:
    - Mobile Chrome（Android，P0 流程）
    - Mobile Safari（iOS，P0 流程）

  分辨率:
    - 1920×1080（桌面标准）
    - 1366×768（笔记本常见）
    - 375×812（iPhone）
    - 360×800（Android）
```

---

## 二、Playwright 实战

### 2.1 配置文件

```typescript
// playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e/specs',
  outputDir: './test-results',

  // 全局超时
  timeout: 30_000,
  expect: { timeout: 5_000 },

  // 失败重试（CI 中开启）
  retries: process.env.CI ? 2 : 0,

  // 并行执行
  fullyParallel: true,
  workers: process.env.CI ? 4 : undefined,

  // 报告
  reporter: [
    ['html', { open: 'never' }],
    ['junit', { outputFile: 'test-results/junit.xml' }],
    process.env.CI ? ['github'] : ['list'],
  ],

  // 全局配置
  use: {
    baseURL: process.env.BASE_URL || 'http://localhost:3000',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    actionTimeout: 10_000,
    navigationTimeout: 15_000,
  },

  // 多浏览器/设备
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
    {
      name: 'mobile-chrome',
      use: { ...devices['Pixel 7'] },
    },
    {
      name: 'mobile-safari',
      use: { ...devices['iPhone 14'] },
    },
  ],

  // 自动启动开发服务器（本地开发时）
  webServer: process.env.CI ? undefined : {
    command: 'npm run dev',
    url: 'http://localhost:3000',
    reuseExistingServer: true,
    timeout: 60_000,
  },
});
```

### 2.2 Page Object Model

```typescript
// tests/e2e/pages/LoginPage.ts
import { Page, Locator, expect } from '@playwright/test';

export class LoginPage {
  readonly page: Page;
  readonly emailInput: Locator;
  readonly passwordInput: Locator;
  readonly submitButton: Locator;
  readonly errorMessage: Locator;

  constructor(page: Page) {
    this.page = page;
    this.emailInput = page.getByLabel('邮箱');
    this.passwordInput = page.getByLabel('密码');
    this.submitButton = page.getByRole('button', { name: '登录' });
    this.errorMessage = page.getByTestId('login-error');
  }

  async goto() {
    await this.page.goto('/login');
  }

  async login(email: string, password: string) {
    await this.emailInput.fill(email);
    await this.passwordInput.fill(password);
    await this.submitButton.click();
  }

  async expectLoginSuccess() {
    await expect(this.page).toHaveURL(/\/dashboard/);
  }

  async expectLoginError(message: string) {
    await expect(this.errorMessage).toContainText(message);
  }
}

// tests/e2e/pages/OrderPage.ts
import { Page, Locator, expect } from '@playwright/test';

export class OrderPage {
  readonly page: Page;
  readonly addToCartButton: Locator;
  readonly cartBadge: Locator;
  readonly checkoutButton: Locator;
  readonly orderConfirmation: Locator;

  constructor(page: Page) {
    this.page = page;
    this.addToCartButton = page.getByRole('button', { name: '加入购物车' });
    this.cartBadge = page.getByTestId('cart-badge');
    this.checkoutButton = page.getByRole('button', { name: '去结算' });
    this.orderConfirmation = page.getByTestId('order-confirmation');
  }

  async addProductToCart(productId: string) {
    await this.page.goto(`/products/${productId}`);
    await this.addToCartButton.click();
    await expect(this.cartBadge).toBeVisible();
  }

  async checkout() {
    await this.page.goto('/cart');
    await this.checkoutButton.click();
  }

  async expectOrderCreated() {
    await expect(this.orderConfirmation).toBeVisible({ timeout: 10_000 });
    const orderId = await this.orderConfirmation.getByTestId('order-id').textContent();
    return orderId;
  }
}
```

### 2.3 测试用例编写

```typescript
// tests/e2e/specs/auth/login.spec.ts
import { test, expect } from '@playwright/test';
import { LoginPage } from '../../pages/LoginPage';
import { createTestUser, deleteTestUser } from '../../helpers/user';

test.describe('用户登录', () => {
  let testUser: { email: string; password: string };

  test.beforeAll(async () => {
    testUser = await createTestUser();
  });

  test.afterAll(async () => {
    await deleteTestUser(testUser.email);
  });

  test('正确凭据登录成功', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await loginPage.goto();
    await loginPage.login(testUser.email, testUser.password);
    await loginPage.expectLoginSuccess();
  });

  test('错误密码登录失败', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await loginPage.goto();
    await loginPage.login(testUser.email, 'wrong-password');
    await loginPage.expectLoginError('邮箱或密码错误');
  });

  test('空表单提交显示校验提示', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await loginPage.goto();
    await loginPage.submitButton.click();
    await expect(page.getByText('请输入邮箱')).toBeVisible();
    await expect(page.getByText('请输入密码')).toBeVisible();
  });
});

// tests/e2e/specs/order/purchase-flow.spec.ts
import { test, expect } from '@playwright/test';
import { LoginPage } from '../../pages/LoginPage';
import { OrderPage } from '../../pages/OrderPage';
import { createTestUser, deleteTestUser } from '../../helpers/user';
import { createTestProduct, deleteTestProduct } from '../../helpers/product';

test.describe('完整购买流程', () => {
  let testUser: { email: string; password: string };
  let productId: string;

  test.beforeAll(async () => {
    testUser = await createTestUser({ balance: 10000 });
    productId = await createTestProduct({ name: 'Test Product', price: 99 });
  });

  test.afterAll(async () => {
    await deleteTestUser(testUser.email);
    await deleteTestProduct(productId);
  });

  test('搜索商品 → 加入购物车 → 下单 → 支付', async ({ page }) => {
    // 登录
    const loginPage = new LoginPage(page);
    await loginPage.goto();
    await loginPage.login(testUser.email, testUser.password);
    await loginPage.expectLoginSuccess();

    // 搜索商品
    await page.getByPlaceholder('搜索商品').fill('Test Product');
    await page.getByPlaceholder('搜索商品').press('Enter');
    await expect(page.getByText('Test Product')).toBeVisible();
    await page.getByText('Test Product').click();

    // 加入购物车
    const orderPage = new OrderPage(page);
    await orderPage.addToCartButton.click();
    await expect(orderPage.cartBadge).toHaveText('1');

    // 结算
    await orderPage.checkout();

    // 选择收货地址
    await page.getByTestId('address-item').first().click();

    // 提交订单
    await page.getByRole('button', { name: '提交订单' }).click();

    // 支付
    await page.getByRole('button', { name: '确认支付' }).click();

    // 验证
    await expect(page.getByText('支付成功')).toBeVisible({ timeout: 15_000 });
  });
});
```

### 2.4 API 拦截与 Mock

```typescript
// 拦截 API 请求用于验证或 Mock
import { test, expect } from '@playwright/test';

test('商品列表加载正确', async ({ page }) => {
  // 拦截 API 并 Mock 响应
  await page.route('**/api/v1/products*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        code: 0,
        data: {
          items: [
            { id: '1', name: '测试商品A', price: 99.00 },
            { id: '2', name: '测试商品B', price: 199.00 },
          ],
          total: 2,
        },
      }),
    });
  });

  await page.goto('/products');
  await expect(page.getByText('测试商品A')).toBeVisible();
  await expect(page.getByText('测试商品B')).toBeVisible();
});

test('API 超时显示友好提示', async ({ page }) => {
  await page.route('**/api/v1/products*', async (route) => {
    // 模拟超时
    await new Promise((r) => setTimeout(r, 30_000));
    await route.abort('timedout');
  });

  await page.goto('/products');
  await expect(page.getByText('加载失败')).toBeVisible({ timeout: 15_000 });
});

test('验证请求参数正确', async ({ page }) => {
  const requestPromise = page.waitForRequest('**/api/v1/orders');

  await page.goto('/checkout');
  await page.getByRole('button', { name: '提交订单' }).click();

  const request = await requestPromise;
  const body = request.postDataJSON();
  expect(body.items).toHaveLength(1);
  expect(body.items[0].product_id).toBeDefined();
  expect(body.shipping_address_id).toBeDefined();
});
```

---

## 三、Cypress 实战

### 3.1 配置文件

```typescript
// cypress.config.ts
import { defineConfig } from 'cypress';

export default defineConfig({
  e2e: {
    baseUrl: 'http://localhost:3000',
    specPattern: 'cypress/e2e/**/*.cy.{js,ts}',
    supportFile: 'cypress/support/e2e.ts',
    viewportWidth: 1280,
    viewportHeight: 720,
    defaultCommandTimeout: 10_000,
    requestTimeout: 15_000,
    responseTimeout: 15_000,
    video: true,
    screenshotOnRunFailure: true,
    retries: {
      runMode: 2,    // CI
      openMode: 0,   // 本地
    },
    experimentalMemoryManagement: true,
  },
});
```

### 3.2 Cypress 自定义命令

```typescript
// cypress/support/commands.ts
declare global {
  namespace Cypress {
    interface Chainable {
      login(email: string, password: string): Chainable<void>;
      createTestData(type: string, data: Record<string, any>): Chainable<string>;
      cleanupTestData(type: string, id: string): Chainable<void>;
    }
  }
}

Cypress.Commands.add('login', (email: string, password: string) => {
  // 通过 API 登录（比 UI 快）
  cy.request('POST', '/api/v1/auth/login', { email, password }).then((resp) => {
    window.localStorage.setItem('token', resp.body.data.token);
  });
});

Cypress.Commands.add('createTestData', (type: string, data: Record<string, any>) => {
  return cy.request({
    method: 'POST',
    url: `/api/v1/test-helpers/${type}`,
    body: data,
    headers: { 'X-Test-Helper': 'true' },
  }).then((resp) => resp.body.data.id);
});

Cypress.Commands.add('cleanupTestData', (type: string, id: string) => {
  cy.request({
    method: 'DELETE',
    url: `/api/v1/test-helpers/${type}/${id}`,
    headers: { 'X-Test-Helper': 'true' },
    failOnStatusCode: false,
  });
});
```

### 3.3 Cypress 测试用例

```typescript
// cypress/e2e/order/purchase.cy.ts
describe('完整购买流程', () => {
  let productId: string;

  before(() => {
    cy.createTestData('product', { name: 'Cypress Test', price: 99 }).then((id) => {
      productId = id;
    });
  });

  after(() => {
    cy.cleanupTestData('product', productId);
  });

  beforeEach(() => {
    cy.login('test@example.com', 'password123');
  });

  it('完成下单支付流程', () => {
    cy.visit(`/products/${productId}`);
    cy.contains('加入购物车').click();
    cy.get('[data-testid="cart-badge"]').should('contain', '1');

    cy.visit('/cart');
    cy.contains('去结算').click();

    cy.get('[data-testid="address-item"]').first().click();
    cy.contains('提交订单').click();
    cy.contains('确认支付').click();

    cy.contains('支付成功', { timeout: 15_000 }).should('be.visible');
  });
});
```

---

## 四、测试数据管理

### 4.1 数据策略

```yaml
原则:
  - 每个测试自建自清（Self-contained）
  - 不依赖共享测试数据（避免测试间耦合）
  - 测试数据通过 API 创建而非 DB 直插（保证业务逻辑一致性）
  - 并行执行时数据隔离（使用唯一前缀/随机标识）

数据层次:
  全局种子数据:
    - 管理员账号
    - 商品类目
    - 系统配置
    - 通过 DB Seed 脚本管理

  测试级别数据:
    - 测试用户（beforeAll 创建 / afterAll 清理）
    - 测试商品/订单
    - 通过 Test Helper API 管理

  用例级别数据:
    - 特定场景的一次性数据
    - beforeEach 创建 / afterEach 清理
```

### 4.2 Test Helper API

```typescript
// 后端需提供测试数据辅助接口（仅 staging 环境启用）
// helpers/testData.ts

import { request } from '@playwright/test';

const API_BASE = process.env.BASE_URL || 'http://localhost:3001';
const TEST_HELPER_KEY = process.env.TEST_HELPER_KEY || 'test-secret';

export async function createTestUser(options?: {
  email?: string;
  balance?: number;
}) {
  const apiContext = await request.newContext();
  const email = options?.email || `test-${Date.now()}-${Math.random().toString(36).slice(2)}@test.com`;
  const password = 'Test@123456';

  const resp = await apiContext.post(`${API_BASE}/api/v1/test-helpers/users`, {
    headers: { 'X-Test-Helper-Key': TEST_HELPER_KEY },
    data: { email, password, balance: options?.balance || 0 },
  });

  const body = await resp.json();
  return { email, password, id: body.data.id };
}

export async function deleteTestUser(email: string) {
  const apiContext = await request.newContext();
  await apiContext.delete(`${API_BASE}/api/v1/test-helpers/users`, {
    headers: { 'X-Test-Helper-Key': TEST_HELPER_KEY },
    data: { email },
  });
}

export async function createTestProduct(options: {
  name: string;
  price: number;
}) {
  const apiContext = await request.newContext();
  const resp = await apiContext.post(`${API_BASE}/api/v1/test-helpers/products`, {
    headers: { 'X-Test-Helper-Key': TEST_HELPER_KEY },
    data: options,
  });
  const body = await resp.json();
  return body.data.id;
}

export async function deleteTestProduct(id: string) {
  const apiContext = await request.newContext();
  await apiContext.delete(`${API_BASE}/api/v1/test-helpers/products/${id}`, {
    headers: { 'X-Test-Helper-Key': TEST_HELPER_KEY },
  });
}
```

---

## 五、CI 集成

### 5.1 GitHub Actions 配置

```yaml
# .github/workflows/e2e.yml
name: E2E Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  e2e:
    runs-on: ubuntu-latest
    timeout-minutes: 30

    strategy:
      fail-fast: false
      matrix:
        shard: [1, 2, 3, 4]  # 4 分片并行

    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_DB: test
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
        ports:
          - 5432:5432
        options: --health-cmd pg_isready --health-interval 10s --health-timeout 5s --health-retries 5

      redis:
        image: redis:7
        ports:
          - 6379:6379

    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm

      - name: Install dependencies
        run: npm ci

      - name: Install Playwright browsers
        run: npx playwright install --with-deps chromium

      - name: Build application
        run: npm run build

      - name: Start application
        run: |
          npm run start &
          npx wait-on http://localhost:3000 --timeout 60000

      - name: Run E2E tests (shard ${{ matrix.shard }}/4)
        run: npx playwright test --shard=${{ matrix.shard }}/4
        env:
          BASE_URL: http://localhost:3000
          DATABASE_URL: postgresql://test:test@localhost:5432/test
          TEST_HELPER_KEY: test-secret

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-results-shard-${{ matrix.shard }}
          path: |
            test-results/
            playwright-report/
          retention-days: 7

  merge-reports:
    needs: e2e
    if: always()
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Download all shard results
        uses: actions/download-artifact@v4
        with:
          pattern: e2e-results-shard-*
          merge-multiple: true
          path: all-results/

      - name: Merge reports
        run: npx playwright merge-reports --reporter html ./all-results

      - name: Upload merged report
        uses: actions/upload-artifact@v4
        with:
          name: e2e-report-merged
          path: playwright-report/
          retention-days: 30
```

### 5.2 GitLab CI 配置

```yaml
# .gitlab-ci.yml (E2E 部分)
e2e-tests:
  stage: test
  image: mcr.microsoft.com/playwright:v1.42.0-jammy
  parallel: 4
  services:
    - postgres:16
    - redis:7
  variables:
    DATABASE_URL: postgresql://test:test@postgres:5432/test
    POSTGRES_DB: test
    POSTGRES_USER: test
    POSTGRES_PASSWORD: test
  script:
    - npm ci
    - npm run build
    - npm run start &
    - npx wait-on http://localhost:3000 --timeout 60000
    - npx playwright test --shard=$CI_NODE_INDEX/$CI_NODE_TOTAL
  artifacts:
    when: always
    paths:
      - test-results/
      - playwright-report/
    expire_in: 7 days
  rules:
    - if: $CI_MERGE_REQUEST_ID
    - if: $CI_COMMIT_BRANCH == "main"
```

---

## 六、稳定性治理

### 6.1 Flaky Test 识别与处理

```yaml
识别方法:
  自动检测:
    - CI 中开启 retries=2
    - 通过重试成功判定为 Flaky
    - 每周统计 Flaky Rate 报告

  手动标记:
    # Playwright
    test.fixme('购物车并发操作', async () => { ... });
    # 标记为已知 Flaky，跳过执行但保留在报告中

常见 Flaky 原因与修复:
  时序问题:
    问题: 页面未加载完成就操作
    修复: 使用 expect().toBeVisible() 等待，而非 sleep
    # 错误
    await page.click('#submit');
    # 正确
    await page.getByRole('button', { name: '提交' }).click();  # 自动等待

  动画干扰:
    问题: CSS 动画导致元素位置不稳定
    修复: 等待动画结束或禁用动画
    # playwright.config.ts
    use: {
      launchOptions: {
        args: ['--force-prefers-reduced-motion'],
      },
    }

  数据竞争:
    问题: 并行测试共享数据导致冲突
    修复: 每个测试使用唯一数据（见测试数据管理）

  网络不稳定:
    问题: 外部 API 偶尔超时
    修复: Mock 外部依赖 / 增加超时 / 使用 retry 机制

  时区/时间:
    问题: 日期相关断言在不同时区失败
    修复: 使用 UTC 或固定时区
    # 在 CI 中设置
    env:
      TZ: Asia/Shanghai
```

### 6.2 性能优化

```yaml
测试执行加速:
  并行执行:
    - Playwright: fullyParallel=true, workers=4
    - CI 分片: --shard=1/4（跨 Job 并行）
    - 预估加速: 4 分片可将 20 分钟缩短到 ~6 分钟

  减少等待:
    - API 登录替代 UI 登录（节省 ~3s/测试）
    - 路由拦截替代真实 API 调用（不稳定场景）
    - 避免 page.waitForTimeout()（硬等待）

  浏览器优化:
    - CI 中只用 Chromium（除非需要跨浏览器）
    - 禁用不必要的浏览器功能
    launchOptions:
      args:
        - '--disable-gpu'
        - '--disable-dev-shm-usage'
        - '--disable-extensions'

  数据优化:
    - 使用 globalSetup 创建共享只读数据
    - 使用 storageState 复用登录状态
```

### 6.3 登录状态复用

```typescript
// tests/e2e/auth.setup.ts
import { test as setup, expect } from '@playwright/test';

const authFile = 'tests/e2e/.auth/user.json';

setup('authenticate', async ({ page }) => {
  await page.goto('/login');
  await page.getByLabel('邮箱').fill('test@example.com');
  await page.getByLabel('密码').fill('password123');
  await page.getByRole('button', { name: '登录' }).click();
  await expect(page).toHaveURL(/\/dashboard/);

  // 保存登录状态
  await page.context().storageState({ path: authFile });
});

// playwright.config.ts 中配置
projects: [
  { name: 'setup', testMatch: /.*\.setup\.ts/ },
  {
    name: 'chromium',
    use: {
      ...devices['Desktop Chrome'],
      storageState: 'tests/e2e/.auth/user.json',
    },
    dependencies: ['setup'],
  },
],
```

---

## 七、验证

### 7.1 测试质量指标

| 指标 | 目标 | 测量方法 |
|------|------|---------|
| P0 用例覆盖率 | 100% | 对照业务用例清单 |
| Flaky Rate | < 2% | CI 统计（重试成功/总运行） |
| 测试总耗时（CI） | < 15 分钟 | CI Pipeline 时间 |
| 测试通过率 | > 98% | CI 统计 |
| 缺陷逃逸率 | < 5% | 线上 Bug 中 E2E 应拦截未拦截的比例 |

### 7.2 测试报告审查

```bash
# 本地查看 Playwright 报告
npx playwright show-report

# 报告应包含：
# - 总体通过率与耗时
# - 失败用例的截图、视频、Trace
# - 每个 Shard 的执行情况
# - Flaky 测试标识

# 查看 Trace（调试利器）
npx playwright show-trace test-results/xxx/trace.zip
# Trace 包含：
# - 每一步操作的截图
# - 网络请求/响应
# - Console 日志
# - DOM 快照
```

---

## 八、回滚

### E2E 测试失败时的处理

```yaml
CI 中 E2E 失败:
  阻断发布:
    - P0 用例失败 → 阻止合并/部署
    - 修复优先级高于新功能开发

  不阻断但需跟进:
    - P1/P2 用例失败 → 允许合并，创建 Bug 跟踪
    - 已知 Flaky 且重试通过 → 不阻断，但需在 Sprint 内修复

回滚测试环境:
  # 当测试环境被污染时
  # 1. 重置数据库
  npm run db:reset
  # 2. 重新 Seed
  npm run db:seed
  # 3. 重启服务
  docker-compose restart

回滚测试框架升级:
  # 当 Playwright/Cypress 升级导致大面积失败
  # 1. 回退版本
  npm install playwright@previous-version
  # 2. 检查 Breaking Changes
  # 3. 逐步适配后再升级
```

---

## Agent Checklist

供自动化 Agent 在执行 E2E 测试流程时逐项核查：

- [ ] 测试工具已安装（Playwright / Cypress + 浏览器）
- [ ] 测试策略已制定（覆盖范围/优先级/浏览器矩阵）
- [ ] Page Object Model 已建立
- [ ] P0 核心业务流程已有对应测试用例
- [ ] 测试数据管理方案已实施（自建自清/Test Helper API）
- [ ] 多浏览器配置已完成
- [ ] CI 集成已配置（分片并行/报告上传）
- [ ] 登录状态复用已实现（storageState）
- [ ] Flaky Test 监控机制已建立
- [ ] 测试执行总耗时 < 15 分钟
- [ ] Flaky Rate < 2%
- [ ] 测试报告含截图/视频/Trace
- [ ] P0 用例失败会阻断发布
- [ ] 测试环境回滚方案就绪