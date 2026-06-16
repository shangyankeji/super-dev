---
id: ui-design-system-complete
title: UI 设计系统完整指南
domain: design
category: 01-standards
difficulty: intermediate
tags: [accessibility, complete, design, system, tokens, 动效系统, 响应式设计, 无障碍]
quality_score: 70
last_updated: 2026-06-15
---
# UI 设计系统完整指南

## 概述

设计系统 (Design System) 是一套可复用的组件、模式和标准的集合，用于保证产品的视觉一致性和开发效率。本指南覆盖设计系统的核心元素、构建方法和实施策略。

---

## 设计令牌 (Design Tokens)

设计令牌是设计系统的基础原子，定义颜色、字体、间距等基本视觉属性。

### 颜色系统

```css
:root {
  /* 主色 */
  --color-primary-50: #eff6ff;
  --color-primary-100: #dbeafe;
  --color-primary-500: #3b82f6;
  --color-primary-600: #2563eb;
  --color-primary-700: #1d4ed8;
  --color-primary-900: #1e3a5f;

  /* 语义色 */
  --color-success: #22c55e;
  --color-warning: #f59e0b;
  --color-error: #ef4444;
  --color-info: #3b82f6;

  /* 中性色 */
  --color-gray-50: #f9fafb;
  --color-gray-100: #f3f4f6;
  --color-gray-200: #e5e7eb;
  --color-gray-500: #6b7280;
  --color-gray-700: #374151;
  --color-gray-900: #111827;

  /* 背景 */
  --bg-primary: #ffffff;
  --bg-secondary: #f9fafb;
  --bg-tertiary: #f3f4f6;

  /* 文字 */
  --text-primary: #111827;
  --text-secondary: #4b5563;
  --text-tertiary: #9ca3af;
  --text-inverse: #ffffff;
}

/* 暗色模式 */
[data-theme="dark"] {
  --bg-primary: #111827;
  --bg-secondary: #1f2937;
  --bg-tertiary: #374151;
  --text-primary: #f9fafb;
  --text-secondary: #d1d5db;
  --text-tertiary: #6b7280;
}
```

### 排版系统

```css
:root {
  /* 字体族 */
  --font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --font-mono: 'JetBrains Mono', 'Fira Code', monospace;

  /* 字号 (使用 rem，基于 16px) */
  --text-xs: 0.75rem;     /* 12px */
  --text-sm: 0.875rem;    /* 14px */
  --text-base: 1rem;      /* 16px */
  --text-lg: 1.125rem;    /* 18px */
  --text-xl: 1.25rem;     /* 20px */
  --text-2xl: 1.5rem;     /* 24px */
  --text-3xl: 1.875rem;   /* 30px */
  --text-4xl: 2.25rem;    /* 36px */

  /* 行高 */
  --leading-tight: 1.25;
  --leading-normal: 1.5;
  --leading-relaxed: 1.75;

  /* 字重 */
  --font-normal: 400;
  --font-medium: 500;
  --font-semibold: 600;
  --font-bold: 700;
}
```

### 间距系统

```css
:root {
  /* 4px 基准网格 */
  --space-0: 0;
  --space-1: 0.25rem;  /* 4px */
  --space-2: 0.5rem;   /* 8px */
  --space-3: 0.75rem;  /* 12px */
  --space-4: 1rem;     /* 16px */
  --space-5: 1.25rem;  /* 20px */
  --space-6: 1.5rem;   /* 24px */
  --space-8: 2rem;     /* 32px */
  --space-10: 2.5rem;  /* 40px */
  --space-12: 3rem;    /* 48px */
  --space-16: 4rem;    /* 64px */
  --space-20: 5rem;    /* 80px */

  /* 圆角 */
  --radius-sm: 0.25rem;
  --radius-md: 0.375rem;
  --radius-lg: 0.5rem;
  --radius-xl: 0.75rem;
  --radius-2xl: 1rem;
  --radius-full: 9999px;

  /* 阴影 */
  --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1);
  --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1);
  --shadow-xl: 0 20px 25px -5px rgb(0 0 0 / 0.1);
}
```

---

## 组件设计原则

### 1. 组件层次

```
Tokens (设计令牌)
  └── Atoms (原子组件): Button, Input, Badge, Icon, Avatar
       └── Molecules (分子组件): SearchBar, FormField, Card, Dropdown
            └── Organisms (有机体): Header, Sidebar, DataTable, Form
                 └── Templates (模板): DashboardLayout, AuthLayout
                      └── Pages (页面): LoginPage, DashboardPage
```

### 2. Button 组件设计

```tsx
// React 组件示例
interface ButtonProps {
  variant: 'primary' | 'secondary' | 'outline' | 'ghost' | 'destructive';
  size: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  loading?: boolean;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  children: React.ReactNode;
  onClick?: () => void;
}

const Button: React.FC<ButtonProps> = ({
  variant = 'primary',
  size = 'md',
  disabled = false,
  loading = false,
  leftIcon,
  rightIcon,
  children,
  onClick,
}) => {
  const baseStyles = 'inline-flex items-center justify-center font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50';

  const variants = {
    primary: 'bg-primary-600 text-white hover:bg-primary-700 focus-visible:ring-primary-500',
    secondary: 'bg-gray-100 text-gray-900 hover:bg-gray-200 focus-visible:ring-gray-500',
    outline: 'border border-gray-300 bg-transparent hover:bg-gray-50 focus-visible:ring-gray-500',
    ghost: 'bg-transparent hover:bg-gray-100 focus-visible:ring-gray-500',
    destructive: 'bg-red-600 text-white hover:bg-red-700 focus-visible:ring-red-500',
  };

  const sizes = {
    sm: 'h-8 px-3 text-sm rounded-md gap-1.5',
    md: 'h-10 px-4 text-sm rounded-lg gap-2',
    lg: 'h-12 px-6 text-base rounded-lg gap-2.5',
  };

  return (
    <button
      className={`${baseStyles} ${variants[variant]} ${sizes[size]}`}
      disabled={disabled || loading}
      onClick={onClick}
    >
      {loading ? <Spinner size={size} /> : leftIcon}
      {children}
      {rightIcon}
    </button>
  );
};
```

### 3. 表单组件设计

```tsx
interface FormFieldProps {
  label: string;
  error?: string;
  hint?: string;
  required?: boolean;
  children: React.ReactNode;
}

const FormField: React.FC<FormFieldProps> = ({ label, error, hint, required, children }) => (
  <div className="space-y-1.5">
    <label className="text-sm font-medium text-gray-700">
      {label}
      {required && <span className="text-red-500 ml-0.5">*</span>}
    </label>
    {children}
    {error && <p className="text-sm text-red-600">{error}</p>}
    {hint && !error && <p className="text-sm text-gray-500">{hint}</p>}
  </div>
);
```

---

## 响应式设计

### 断点系统

```css
/* Mobile First 断点 */
/* xs: 0px    — 手机竖屏 */
/* sm: 640px  — 手机横屏 */
/* md: 768px  — 平板 */
/* lg: 1024px — 桌面 */
/* xl: 1280px — 大屏 */
/* 2xl: 1536px — 超大屏 */

/* Tailwind CSS 示例 */
.container {
  @apply px-4 sm:px-6 md:px-8 lg:px-12;
  @apply max-w-full sm:max-w-xl md:max-w-3xl lg:max-w-5xl xl:max-w-7xl;
  @apply mx-auto;
}

/* Grid 布局 */
.grid-layout {
  @apply grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4;
  @apply gap-4 sm:gap-6;
}
```

---

## 无障碍 (Accessibility)

### WCAG 2.1 核心要求

```tsx
// 1. 颜色对比度 >= 4.5:1 (普通文字) / >= 3:1 (大文字)
// 使用工具验证: https://webaim.org/resources/contrastchecker/

// 2. 键盘导航
const Dialog = ({ isOpen, onClose, children }) => {
  const dialogRef = useRef(null);

  useEffect(() => {
    if (isOpen) {
      // 焦点陷阱
      dialogRef.current?.focus();
    }
  }, [isOpen]);

  const handleKeyDown = (e) => {
    if (e.key === 'Escape') onClose();
    // Tab 焦点陷阱逻辑...
  };

  return isOpen ? (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="dialog-title"
      ref={dialogRef}
      onKeyDown={handleKeyDown}
      tabIndex={-1}
    >
      <h2 id="dialog-title">标题</h2>
      {children}
    </div>
  ) : null;
};

// 3. ARIA 属性
<button aria-label="关闭菜单" aria-expanded={isOpen}>
  <MenuIcon />
</button>

// 4. 跳过导航链接
<a href="#main-content" className="sr-only focus:not-sr-only">
  跳到主要内容
</a>
```

---

## 动效系统

```css
:root {
  /* 时长 */
  --duration-fast: 150ms;
  --duration-normal: 250ms;
  --duration-slow: 350ms;

  /* 缓动函数 */
  --ease-in: cubic-bezier(0.4, 0, 1, 1);
  --ease-out: cubic-bezier(0, 0, 0.2, 1);
  --ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
  --ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);
}

/* 进入动画 */
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from { transform: translateY(8px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

.animate-fade-in {
  animation: fadeIn var(--duration-normal) var(--ease-out);
}

.animate-slide-up {
  animation: slideUp var(--duration-normal) var(--ease-out);
}
```

---

## 暗色模式

```tsx
// React 暗色模式实现
function useTheme() {
  const [theme, setTheme] = useState(() => {
    if (typeof window !== 'undefined') {
      return localStorage.getItem('theme') ||
        (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
    }
    return 'light';
  });

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('theme', theme);
  }, [theme]);

  const toggleTheme = () => setTheme(t => t === 'light' ? 'dark' : 'light');

  return { theme, toggleTheme };
}
```

---

## 常见反模式

### 1. 紫色渐变 + emoji图标 (AI模板味)
❌ 过度使用渐变背景、emoji作为图标、默认系统字体
✅ 使用克制的颜色、专业图标库 (Lucide/Heroicons)、定制字体

### 2. 不一致的间距
❌ 随意使用 13px, 17px, 23px 等非标准间距
✅ 使用 4px 基准网格: 4, 8, 12, 16, 24, 32, 48, 64

### 3. 忽视移动端
❌ 只设计桌面版，移动端靠缩放
✅ Mobile First 设计，渐进增强

---

## Agent Checklist

Agent 在实现 UI 时必须检查:

- [ ] 是否定义了设计令牌（颜色/字体/间距/阴影）？
- [ ] 组件是否遵循原子设计方法论？
- [ ] 是否支持暗色模式？
- [ ] 颜色对比度是否满足 WCAG 4.5:1？
- [ ] 是否支持键盘导航？
- [ ] 是否使用了语义化 HTML 和 ARIA？
- [ ] 响应式是否覆盖 mobile/tablet/desktop？
- [ ] 动效是否使用统一的时长和缓动？
- [ ] 是否避免了 AI 模板感（紫色渐变/emoji图标）？
- [ ] 间距是否使用 4px 基准网格？

---

## 参考资料

- [Tailwind CSS](https://tailwindcss.com/)
- [Radix UI](https://www.radix-ui.com/)
- [shadcn/ui](https://ui.shadcn.com/)
- [Material Design 3](https://m3.material.io/)
- [WCAG 2.1](https://www.w3.org/WAI/WCAG21/quickref/)

---

**文档版本**: v1.0
**最后更新**: 2026-03-28
**质量评分**: 88/100
