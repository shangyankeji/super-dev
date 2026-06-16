---
id: accessibility-complete
title: Web 无障碍完整指南
domain: frontend
category: 01-standards
difficulty: intermediate
tags: [accessibility, aria, complete, frontend, html, 屏幕阅读器优化, 属性, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Web 无障碍完整指南

## 概述

Web 无障碍 (Accessibility, a11y) 确保所有用户（包括视觉、听觉、运动和认知障碍用户）都能有效使用 Web 应用。WCAG 2.1 是国际公认标准，分为 A（基础）、AA（推荐）、AAA（增强）三个等级。商业项目至少需达到 WCAG 2.1 AA 标准。

### 四大原则 (POUR)

1. **可感知 (Perceivable)**: 信息必须以用户可感知的方式呈现
2. **可操作 (Operable)**: UI 组件和导航必须可操作
3. **可理解 (Understandable)**: 信息和操作必须可理解
4. **健壮性 (Robust)**: 内容必须足够健壮，能被各种技术（包括辅助技术）解析

---

## 语义化 HTML

### 正确使用 HTML 元素

```html
<!-- 正确：语义化结构 -->
<header>
  <nav aria-label="主导航">
    <ul>
      <li><a href="/">首页</a></li>
      <li><a href="/products" aria-current="page">产品</a></li>
    </ul>
  </nav>
</header>

<main>
  <h1>产品列表</h1>
  <section aria-labelledby="featured-heading">
    <h2 id="featured-heading">精选产品</h2>
    <article>
      <h3>产品名称</h3>
      <p>产品描述...</p>
    </article>
  </section>
</main>

<aside aria-label="侧边栏">
  <h2>相关推荐</h2>
</aside>

<footer>
  <p>&copy; 2026 Company</p>
</footer>
```

```html
<!-- 错误：div 堆砌，无语义 -->
<div class="header">
  <div class="nav">
    <div class="link" onclick="goto('/')">首页</div>
  </div>
</div>
<div class="content">
  <div class="title">产品列表</div>
</div>
```

### 标题层级

```html
<!-- 正确：标题层级有序 -->
<h1>页面主标题</h1>
  <h2>第一节</h2>
    <h3>子节</h3>
  <h2>第二节</h2>

<!-- 错误：跳过层级 -->
<h1>标题</h1>
  <h3>直接到 h3，跳过了 h2</h3>
```

---

## ARIA 属性

### 常用 ARIA 角色和属性

```html
<!-- 自定义按钮 -->
<div role="button" tabindex="0"
     aria-pressed="false"
     onkeydown="handleKeyDown(event)"
     onclick="toggle()">
  切换主题
</div>

<!-- 更好的做法：直接使用 button -->
<button aria-pressed="false" onclick="toggle()">切换主题</button>
```

### ARIA Live Regions

```html
<!-- 动态内容通知屏幕阅读器 -->
<div aria-live="polite" aria-atomic="true">
  <!-- 内容变化时自动播报 -->
</div>

<!-- 紧急通知 -->
<div role="alert" aria-live="assertive">
  表单提交失败：邮箱格式不正确
</div>

<!-- 状态消息 -->
<div role="status" aria-live="polite">
  已加载 50 条结果
</div>
```

### 表单无障碍

```html
<form>
  <div>
    <label for="email">邮箱地址 <span aria-hidden="true">*</span></label>
    <input
      id="email"
      type="email"
      required
      aria-required="true"
      aria-describedby="email-hint email-error"
      aria-invalid="true"
    />
    <p id="email-hint" class="hint">请输入您的工作邮箱</p>
    <p id="email-error" class="error" role="alert">
      请输入有效的邮箱地址
    </p>
  </div>

  <fieldset>
    <legend>通知偏好</legend>
    <label>
      <input type="radio" name="notify" value="email" /> 邮件通知
    </label>
    <label>
      <input type="radio" name="notify" value="sms" /> 短信通知
    </label>
  </fieldset>

  <button type="submit">提交</button>
</form>
```

### 对话框

```tsx
// React 无障碍对话框
function Dialog({ isOpen, onClose, title, children }) {
  const titleId = useId();

  return isOpen ? (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      onKeyDown={(e) => e.key === "Escape" && onClose()}
    >
      <h2 id={titleId}>{title}</h2>
      <div>{children}</div>
      <button onClick={onClose}>关闭</button>
    </div>
  ) : null;
}
```

---

## 键盘导航

### Tab 顺序管理

```tsx
// 焦点陷阱 - 模态框内循环 Tab
function useFocusTrap(ref: RefObject<HTMLElement>) {
  useEffect(() => {
    const element = ref.current;
    if (!element) return;

    const focusableElements = element.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    const firstElement = focusableElements[0] as HTMLElement;
    const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement;

    function handleKeyDown(e: KeyboardEvent) {
      if (e.key !== "Tab") return;
      if (e.shiftKey) {
        if (document.activeElement === firstElement) {
          e.preventDefault();
          lastElement.focus();
        }
      } else {
        if (document.activeElement === lastElement) {
          e.preventDefault();
          firstElement.focus();
        }
      }
    }

    element.addEventListener("keydown", handleKeyDown);
    firstElement?.focus();
    return () => element.removeEventListener("keydown", handleKeyDown);
  }, [ref]);
}
```

### 跳过导航链接

```html
<body>
  <a href="#main-content" class="skip-link">跳到主要内容</a>
  <header><!-- 导航 --></header>
  <main id="main-content" tabindex="-1">
    <!-- 主要内容 -->
  </main>
</body>

<style>
.skip-link {
  position: absolute;
  top: -40px;
  left: 0;
  padding: 8px 16px;
  background: #000;
  color: #fff;
  z-index: 100;
  transition: top 0.2s;
}
.skip-link:focus {
  top: 0;
}
</style>
```

### 自定义组件键盘交互

```tsx
// 下拉菜单键盘导航
function DropdownMenu({ items }: { items: MenuItem[] }) {
  const [isOpen, setIsOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(-1);

  function handleKeyDown(e: KeyboardEvent) {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setActiveIndex(i => Math.min(i + 1, items.length - 1));
        break;
      case "ArrowUp":
        e.preventDefault();
        setActiveIndex(i => Math.max(i - 1, 0));
        break;
      case "Enter":
      case " ":
        e.preventDefault();
        if (activeIndex >= 0) items[activeIndex].action();
        setIsOpen(false);
        break;
      case "Escape":
        setIsOpen(false);
        break;
      case "Home":
        e.preventDefault();
        setActiveIndex(0);
        break;
      case "End":
        e.preventDefault();
        setActiveIndex(items.length - 1);
        break;
    }
  }

  return (
    <div onKeyDown={handleKeyDown}>
      <button
        aria-haspopup="true"
        aria-expanded={isOpen}
        onClick={() => setIsOpen(!isOpen)}
      >
        菜单
      </button>
      {isOpen && (
        <ul role="menu">
          {items.map((item, index) => (
            <li
              key={item.id}
              role="menuitem"
              tabIndex={index === activeIndex ? 0 : -1}
              aria-selected={index === activeIndex}
            >
              {item.label}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
```

---

## 屏幕阅读器优化

### 隐藏装饰性内容

```html
<!-- 装饰性图片 -->
<img src="/decorative.svg" alt="" aria-hidden="true" />

<!-- 仅屏幕阅读器可见 -->
<span class="sr-only">当前购物车有 3 件商品</span>

<style>
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}
</style>
```

### 图标按钮

```html
<!-- 仅图标按钮必须有标签 -->
<button aria-label="关闭对话框">
  <svg aria-hidden="true"><!-- X icon --></svg>
</button>

<!-- 图标 + 文字 -->
<button>
  <svg aria-hidden="true"><!-- icon --></svg>
  <span>保存</span>
</button>
```

### 表格无障碍

```html
<table>
  <caption>2026 年第一季度销售数据</caption>
  <thead>
    <tr>
      <th scope="col">月份</th>
      <th scope="col">销售额</th>
      <th scope="col">增长率</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <th scope="row">1 月</th>
      <td>¥120,000</td>
      <td>+15%</td>
    </tr>
  </tbody>
</table>
```

---

## 颜色与对比度

### WCAG 对比度要求

| 文本类型 | AA 标准 | AAA 标准 |
|----------|---------|----------|
| 正文 (< 18px) | 4.5:1 | 7:1 |
| 大文本 (>= 18px 或 14px bold) | 3:1 | 4.5:1 |
| UI 组件边界 | 3:1 | - |

```css
/* 不仅依赖颜色传达信息 */
/* 错误：仅用红色表示错误 */
.error { color: red; }

/* 正确：颜色 + 图标 + 文字 */
.error {
  color: #DC2626;
  border-left: 3px solid #DC2626;
  padding-left: 12px;
}
.error::before {
  content: "⚠ ";
}
```

---

## 响应式无障碍

```css
/* 缩放到 200% 仍可用 */
html {
  font-size: 100%;    /* 尊重用户字体设置 */
}

body {
  font-size: 1rem;    /* 使用 rem，不用 px */
  line-height: 1.5;   /* 最低 1.5 */
}

/* 减少动画（尊重用户偏好） */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}

/* 高对比度模式 */
@media (prefers-contrast: high) {
  :root {
    --border-color: #000;
    --text-color: #000;
    --bg-color: #fff;
  }
}

/* 触摸目标最小 44x44px */
button, a, input[type="checkbox"], input[type="radio"] {
  min-width: 44px;
  min-height: 44px;
}
```

---

## 测试工具

### 自动化工具

| 工具 | 类型 | 覆盖率 | 用途 |
|------|------|--------|------|
| axe-core | 代码库 | ~57% | CI 集成 |
| Lighthouse | 浏览器 | ~30% | 快速审计 |
| pa11y | CLI | ~40% | CI 流水线 |
| jest-axe | 测试 | ~57% | 单元测试 |
| Playwright a11y | E2E | ~57% | E2E 测试 |

### jest-axe 测试示例

```typescript
import { render } from "@testing-library/react";
import { axe, toHaveNoViolations } from "jest-axe";

expect.extend(toHaveNoViolations);

test("表单组件无 a11y 违规", async () => {
  const { container } = render(<LoginForm />);
  const results = await axe(container);
  expect(results).toHaveNoViolations();
});
```

### Playwright 无障碍测试

```typescript
import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("首页无障碍扫描", async ({ page }) => {
  await page.goto("/");
  const results = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa"])
    .analyze();
  expect(results.violations).toEqual([]);
});
```

### 手动测试清单

1. **键盘测试**: 只用键盘（Tab/Shift+Tab/Enter/Escape/方向键）完成所有操作
2. **屏幕阅读器测试**: 用 VoiceOver (macOS) / NVDA (Windows) 浏览全页面
3. **缩放测试**: 浏览器缩放至 200%，所有内容可见可用
4. **色觉模拟**: 使用 Chrome DevTools 色觉缺陷模拟

---

## 常见反模式

| 反模式 | 影响 | 正确做法 |
|--------|------|----------|
| div 替代 button | 键盘不可用 | 使用原生 button |
| 缺少 alt 属性 | 屏幕阅读器无法理解 | 所有图片提供 alt |
| 仅依赖颜色 | 色觉障碍用户无法区分 | 颜色+图标+文字 |
| 自动播放视频/音频 | 影响屏幕阅读器用户 | 提供播放控制 |
| 时间限制操作 | 运动障碍用户无法完成 | 提供延长选项 |
| outline: none | 键盘焦点不可见 | 自定义 :focus-visible 样式 |

---

## Agent Checklist

在 AI 编码流水线中实现 Web 无障碍时，必须逐项检查：

- [ ] 使用语义化 HTML 元素（header/nav/main/section/article/footer）
- [ ] 标题层级有序（h1-h6 不跳级）
- [ ] 所有交互元素键盘可达（Tab/Enter/Escape）
- [ ] 所有图片提供 alt 属性（装饰性图片 alt=""）
- [ ] 表单控件关联 label（for/id 或嵌套）
- [ ] 错误提示使用 aria-invalid + aria-describedby
- [ ] 对比度达到 WCAG 2.1 AA（正文 4.5:1，大文本 3:1）
- [ ] 不仅依赖颜色传达信息（配合图标/文字/边框）
- [ ] 模态框有焦点陷阱和 Escape 关闭
- [ ] 提供跳过导航链接
- [ ] 动态内容使用 aria-live 通知
- [ ] 尊重 prefers-reduced-motion 和 prefers-contrast
- [ ] 触摸目标 >= 44x44px
- [ ] CI 集成 axe-core 自动化检测
- [ ] 关键流程通过屏幕阅读器手动测试
