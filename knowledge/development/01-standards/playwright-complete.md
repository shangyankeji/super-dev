---
id: playwright-complete
title: Playwright端到端测试完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, playwright, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Playwright端到端测试完整指南

## 概述
Playwright是现代化的Web测试框架,支持Chromium、Firefox、WebKit。本指南覆盖测试编写、页面交互、断言和最佳实践。

## 核心概念

### 1. 基础测试

**配置Playwright**:
```javascript
// playwright.config.js
module.exports = {
  use: {
    baseURL: 'http://localhost:3000',
    headless: true,
    viewport: { width: 1280, height: 720 }
  }
};
```

**简单测试**:
```javascript
// tests/login.spec.js
const { test, expect } = require('@playwright/test');

test('login flow', async ({ page }) => {
  await page.goto('/login');
  
  await page.fill('#email', 'user@example.com');
  await page.fill('#password', 'password');
  await page.click('button[type="submit"]');
  
  await expect(page).toHaveURL('/dashboard');
  await expect(page.locator('h1')).toContainText('Welcome');
});
```

### 2. 页面交互

**点击和输入**:
```javascript
test('form submission', async ({ page }) => {
  await page.goto('/contact');
  
  // 填写表单
  await page.fill('#name', 'Alice');
  await page.fill('#email', 'alice@example.com');
  await page.fill('#message', 'Hello World');
  
  // 点击提交
  await page.click('button[type="submit"]');
  
  // 验证成功消息
  await expect(page.locator('.success')).toBeVisible();
});
```

**下拉菜单和复选框**:
```javascript
test('select and checkbox', async ({ page }) => {
  await page.goto('/settings');
  
  // 选择下拉选项
  await page.selectOption('#country', 'China');
  
  // 勾选复选框
  await page.check('#notifications');
  
  // 验证
  expect(await page.isChecked('#notifications')).toBeTruthy();
});
```

### 3. 等待和断言

**等待元素**:
```javascript
test('wait for element', async ({ page }) => {
  await page.goto('/dynamic');
  
  // 等待元素出现
  await page.waitForSelector('.loaded', { timeout: 5000 });
  
  // 等待文本
  await page.waitForSelector('text=Success');
  
  // 等待URL
  await page.waitForURL('/success');
});
```

**断言**:
```javascript
test('assertions', async ({ page }) => {
  await page.goto('/user/1');
  
  // 元素可见
  await expect(page.locator('.profile')).toBeVisible();
  
  // 包含文本
  await expect(page.locator('.name')).toContainText('Alice');
  
  // 属性
  await expect(page.locator('input')).toHaveAttribute('type', 'email');
  
  // 数量
  await expect(page.locator('.item')).toHaveCount(5);
});
```

### 4. API Mock

**拦截请求**:
```javascript
test('mock API', async ({ page }) => {
  // 拦截API请求
  await page.route('**/api/users', route => {
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([{ id: 1, name: 'Mock User' }])
    });
  });
  
  await page.goto('/users');
  
  await expect(page.locator('.user-name')).toContainText('Mock User');
});
```

### 5. 视觉测试

**截图对比**:
```javascript
test('visual regression', async ({ page }) => {
  await page.goto('/homepage');
  
  // 截图并对比
  await expect(page).toHaveScreenshot('homepage.png');
});

// 允许差异
test('visual with threshold', async ({ page }) => {
  await page.goto('/dashboard');
  
  await expect(page).toHaveScreenshot('dashboard.png', {
    maxDiffPixels: 100
  });
});
```

### 6. 多浏览器测试

```javascript
// projects config
module.exports = {
  projects: [
    { name: 'chromium', use: { browserName: 'chromium' } },
    { name: 'firefox', use: { browserName: 'firefox' } },
    { name: 'webkit', use: { browserName: 'webkit' } }
  ]
};

// 测试自动运行在所有浏览器
test('cross browser', async ({ page, browserName }) => {
  await page.goto('/');
  await expect(page.locator('h1')).toBeVisible();
});
```

## 最佳实践

### ✅ DO

1. **使用Page Object Model**
```javascript
// pages/LoginPage.js
class LoginPage {
  constructor(page) {
    this.page = page;
    this.emailInput = '#email';
    this.passwordInput = '#password';
    this.submitButton = 'button[type="submit"]';
  }
  
  async login(email, password) {
    await this.page.fill(this.emailInput, email);
    await this.page.fill(this.passwordInput, password);
    await this.page.click(this.submitButton);
  }
}

// 使用
test('login with page object', async ({ page }) => {
  const loginPage = new LoginPage(page);
  await loginPage.login('user@example.com', 'password');
});
```

### ❌ DON'T

1. **不要使用固定等待**
```javascript
// ❌ 差
await page.waitForTimeout(1000);

// ✅ 好
await page.waitForSelector('.loaded');
```

## 学习路径

### 初级 (1周)
1. 基础测试
2. 页面交互
3. 等待和断言

### 中级 (2-3周)
1. API Mock
2. 视觉测试
3. Page Object Model

### 高级 (2-3周)
1. 自定义Fixtures
2. 测试报告
3. CI/CD集成

---

**知识ID**: `playwright-complete`  
**领域**: development  
**类型**: standards  
**难度**: intermediate  
**质量分**: 94  
**维护者**: testing-team@superdev.com  
**最后更新**: 2026-03-28
