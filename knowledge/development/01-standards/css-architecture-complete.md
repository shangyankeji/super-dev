---
id: css-architecture-complete
title: CSS架构指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [agent, architecture, checklist, complete, css, development, 实战代码示例, 常见陷阱]
quality_score: 70
last_updated: 2026-06-15
---
# CSS架构指南

## 概述
CSS架构决定了样式代码的可维护性、可扩展性和团队协作效率。本指南对比Tailwind CSS、CSS Modules、Styled Components、vanilla-extract四种主流方案,覆盖设计Token系统、响应式策略和组件样式模式。

## 核心概念

### 1. CSS方案分类
- **原子化CSS(Utility-first)**: Tailwind CSS — 预定义工具类,不写自定义CSS
- **CSS Modules**: 文件级作用域,自动生成唯一类名
- **CSS-in-JS(Runtime)**: Styled Components/Emotion — JS中写CSS,运行时注入
- **CSS-in-JS(Zero-runtime)**: vanilla-extract/Panda CSS — 编译时提取,零运行时
- **传统方案**: BEM/SMACSS/OOCSS — 命名约定维护全局CSS

### 2. 方案对比

| 特性 | Tailwind | CSS Modules | Styled Components | vanilla-extract |
|------|----------|-------------|-------------------|-----------------|
| 作用域隔离 | 原子类无冲突 | 自动哈希 | 组件级隔离 | 编译时哈希 |
| 运行时开销 | 无 | 无 | 有(JS→CSS) | 无 |
| TypeScript | 插件支持 | 需声明文件 | 模板字符串 | 原生TS |
| Bundle大小 | 小(PurgeCSS) | 取决于使用 | 包含运行时 | 小 |
| 动态样式 | class拼接 | CSS变量 | props驱动 | CSS变量 |
| 学习曲线 | 中(记忆类名) | 低 | 中 | 中 |
| SSR兼容 | 完美 | 完美 | 需额外配置 | 完美 |
| 调试体验 | 类名可读 | 哈希类名 | 生成类名 | 可配类名 |

### 3. 设计Token系统
- **颜色**: 语义化颜色(primary/error/success)而非具体色值
- **间距**: 4px基准的间距比例尺(0/1/2/3/4/5/6/8/10/12/16)
- **字体**: 字体家族/大小/行高/字重的系统化定义
- **圆角/阴影/动画**: 统一预设值,避免随意数值

## 实战代码示例

### Tailwind CSS

```html
<!-- tailwind.config.ts -->
```

```typescript
// tailwind.config.ts
import type { Config } from 'tailwindcss'

const config: Config = {
  content: ['./src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        brand: {
          50: '#eff6ff',
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
          900: '#1e3a5f',
        },
        surface: {
          primary: 'var(--surface-primary)',
          secondary: 'var(--surface-secondary)',
        },
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      spacing: {
        '4.5': '1.125rem',
        '18': '4.5rem',
      },
      animation: {
        'fade-in': 'fadeIn 0.3s ease-out',
        'slide-up': 'slideUp 0.3s ease-out',
      },
      keyframes: {
        fadeIn: { '0%': { opacity: '0' }, '100%': { opacity: '1' } },
        slideUp: {
          '0%': { transform: 'translateY(10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
      },
    },
  },
  plugins: [
    require('@tailwindcss/forms'),
    require('@tailwindcss/typography'),
  ],
}
export default config
```

```tsx
// 组件示例
function Card({ title, description, status }: CardProps) {
  return (
    <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm
                    transition-shadow hover:shadow-md dark:border-gray-700
                    dark:bg-gray-800">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          {title}
        </h3>
        <span className={cn(
          'inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium',
          status === 'active' && 'bg-green-100 text-green-800',
          status === 'pending' && 'bg-yellow-100 text-yellow-800',
          status === 'error' && 'bg-red-100 text-red-800',
        )}>
          {status}
        </span>
      </div>
      <p className="mt-2 text-sm text-gray-600 dark:text-gray-300">
        {description}
      </p>
    </div>
  )
}

// cn工具函数(clsx + tailwind-merge)
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
```

### CSS Modules

```css
/* Button.module.css */
.button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.5rem 1rem;
  border-radius: 0.375rem;
  font-weight: 500;
  font-size: 0.875rem;
  transition: all 0.15s ease;
  cursor: pointer;
  border: none;
}

.primary {
  composes: button;
  background-color: var(--color-brand-600);
  color: white;
}

.primary:hover {
  background-color: var(--color-brand-700);
}

.secondary {
  composes: button;
  background-color: transparent;
  color: var(--color-gray-700);
  border: 1px solid var(--color-gray-300);
}

.secondary:hover {
  background-color: var(--color-gray-50);
}

.small {
  padding: 0.25rem 0.75rem;
  font-size: 0.75rem;
}

.large {
  padding: 0.75rem 1.5rem;
  font-size: 1rem;
}

.loading {
  opacity: 0.7;
  pointer-events: none;
}
```

```tsx
// Button.tsx
import styles from './Button.module.css'
import cn from 'classnames'

interface ButtonProps {
  variant?: 'primary' | 'secondary'
  size?: 'small' | 'medium' | 'large'
  loading?: boolean
  children: React.ReactNode
  onClick?: () => void
}

export function Button({
  variant = 'primary',
  size = 'medium',
  loading = false,
  children,
  onClick,
}: ButtonProps) {
  return (
    <button
      className={cn(
        styles[variant],
        size !== 'medium' && styles[size],
        loading && styles.loading,
      )}
      onClick={onClick}
      disabled={loading}
    >
      {loading && <Spinner className={styles.spinner} />}
      {children}
    </button>
  )
}
```

### Styled Components

```tsx
import styled, { css, ThemeProvider } from 'styled-components'

// 主题定义
const theme = {
  colors: {
    brand: { 500: '#3b82f6', 600: '#2563eb', 700: '#1d4ed8' },
    text: { primary: '#111827', secondary: '#6b7280' },
    surface: { primary: '#ffffff', secondary: '#f9fafb' },
    border: '#e5e7eb',
  },
  spacing: (n: number) => `${n * 0.25}rem`,
  radius: { sm: '0.25rem', md: '0.375rem', lg: '0.5rem', full: '9999px' },
  shadows: {
    sm: '0 1px 2px rgba(0,0,0,0.05)',
    md: '0 4px 6px -1px rgba(0,0,0,0.1)',
  },
}

type Theme = typeof theme

// 按钮组件
const ButtonBase = styled.button<{ $variant: 'primary' | 'secondary'; $size: string }>`
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: ${({ theme }) => theme.radius.md};
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: all 0.15s ease;

  ${({ $size, theme }) =>
    $size === 'small'
      ? css`
          padding: ${theme.spacing(1)} ${theme.spacing(3)};
          font-size: 0.75rem;
        `
      : css`
          padding: ${theme.spacing(2)} ${theme.spacing(4)};
          font-size: 0.875rem;
        `}

  ${({ $variant, theme }) =>
    $variant === 'primary'
      ? css`
          background: ${theme.colors.brand[600]};
          color: white;
          &:hover { background: ${theme.colors.brand[700]}; }
        `
      : css`
          background: transparent;
          color: ${theme.colors.text.primary};
          border: 1px solid ${theme.colors.border};
          &:hover { background: ${theme.colors.surface.secondary}; }
        `}

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
`

// 卡片组件
const Card = styled.div`
  background: ${({ theme }) => theme.colors.surface.primary};
  border: 1px solid ${({ theme }) => theme.colors.border};
  border-radius: ${({ theme }) => theme.radius.lg};
  padding: ${({ theme }) => theme.spacing(6)};
  box-shadow: ${({ theme }) => theme.shadows.sm};
  transition: box-shadow 0.2s ease;

  &:hover {
    box-shadow: ${({ theme }) => theme.shadows.md};
  }
`

const CardTitle = styled.h3`
  font-size: 1.125rem;
  font-weight: 600;
  color: ${({ theme }) => theme.colors.text.primary};
  margin: 0 0 ${({ theme }) => theme.spacing(2)} 0;
`
```

### vanilla-extract

```typescript
// styles.css.ts
import { style, styleVariants, createTheme, globalStyle } from '@vanilla-extract/css'
import { recipe, type RecipeVariants } from '@vanilla-extract/recipes'

// 定义Token
export const [themeClass, vars] = createTheme({
  color: {
    brand500: '#3b82f6',
    brand600: '#2563eb',
    brand700: '#1d4ed8',
    textPrimary: '#111827',
    textSecondary: '#6b7280',
    surfacePrimary: '#ffffff',
    border: '#e5e7eb',
  },
  space: {
    xs: '0.25rem',
    sm: '0.5rem',
    md: '1rem',
    lg: '1.5rem',
    xl: '2rem',
  },
  radius: {
    sm: '0.25rem',
    md: '0.375rem',
    lg: '0.5rem',
  },
})

// Recipe模式(类似CVA)
export const button = recipe({
  base: {
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: vars.radius.md,
    fontWeight: 500,
    cursor: 'pointer',
    border: 'none',
    transition: 'all 0.15s ease',
  },
  variants: {
    variant: {
      primary: {
        background: vars.color.brand600,
        color: 'white',
        ':hover': { background: vars.color.brand700 },
      },
      secondary: {
        background: 'transparent',
        color: vars.color.textPrimary,
        border: `1px solid ${vars.color.border}`,
        ':hover': { background: '#f9fafb' },
      },
    },
    size: {
      small: { padding: `${vars.space.xs} ${vars.space.sm}`, fontSize: '0.75rem' },
      medium: { padding: `${vars.space.sm} ${vars.space.md}`, fontSize: '0.875rem' },
      large: { padding: `${vars.space.md} ${vars.space.lg}`, fontSize: '1rem' },
    },
  },
  defaultVariants: {
    variant: 'primary',
    size: 'medium',
  },
})

export type ButtonVariants = RecipeVariants<typeof button>

// 卡片样式
export const card = style({
  background: vars.color.surfacePrimary,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.lg,
  padding: vars.space.lg,
  transition: 'box-shadow 0.2s ease',
  ':hover': {
    boxShadow: '0 4px 6px -1px rgba(0,0,0,0.1)',
  },
})
```

### 响应式设计系统

```css
/* design-tokens.css — 全局CSS变量 */
:root {
  /* 颜色 */
  --color-brand-50: #eff6ff;
  --color-brand-500: #3b82f6;
  --color-brand-600: #2563eb;
  --color-gray-50: #f9fafb;
  --color-gray-200: #e5e7eb;
  --color-gray-900: #111827;

  /* 间距比例尺 (4px base) */
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  --space-3: 0.75rem;
  --space-4: 1rem;
  --space-6: 1.5rem;
  --space-8: 2rem;
  --space-12: 3rem;
  --space-16: 4rem;

  /* 字体 */
  --font-sans: 'Inter', system-ui, -apple-system, sans-serif;
  --font-mono: 'JetBrains Mono', monospace;
  --text-xs: 0.75rem;
  --text-sm: 0.875rem;
  --text-base: 1rem;
  --text-lg: 1.125rem;
  --text-xl: 1.25rem;
  --text-2xl: 1.5rem;
  --text-3xl: 1.875rem;

  /* 容器宽度 */
  --container-sm: 640px;
  --container-md: 768px;
  --container-lg: 1024px;
  --container-xl: 1280px;
}

/* 响应式断点(移动优先) */
/* sm: 640px, md: 768px, lg: 1024px, xl: 1280px, 2xl: 1536px */
```

## 最佳实践

### 1. 选型原则
- **Tailwind CSS**: 快速迭代的产品项目,团队愿意拥抱原子化
- **CSS Modules**: 稳妥选择,任何项目都适合,无运行时
- **Styled Components**: 需要动态主题切换和高度组件化的项目
- **vanilla-extract**: TypeScript重度使用,追求类型安全和零运行时

### 2. 设计Token优先
- 任何方案都应先定义Token系统(颜色/间距/字体/圆角)
- Token通过CSS变量实现,方便主题切换
- 避免在代码中出现魔法数字和硬编码颜色

### 3. 组件样式封装
- 每个组件拥有自己的样式,不依赖全局CSS
- 使用variants模式处理组件变体(size/color/state)
- 暴露className prop允许外部自定义

### 4. 暗色模式实现
- 使用CSS变量切换主题,而非类名覆盖
- Tailwind: `dark:`前缀 + class策略
- CSS Modules: `[data-theme="dark"]`选择器
- CSS-in-JS: ThemeProvider切换

### 5. 性能注意
- 避免CSS-in-JS运行时方案在高频更新场景(动画/滚动)
- Tailwind启用PurgeCSS确保最小产物
- 避免深层CSS嵌套(不超过3层)
- 使用content-visibility优化长列表渲染

## 常见陷阱

### 陷阱1: Tailwind类名过长难维护
```tsx
// 错误: 20+类名在JSX中
<div className="flex items-center justify-between rounded-lg border border-gray-200 bg-white p-4 shadow-sm transition-all hover:shadow-md dark:border-gray-700 dark:bg-gray-800 dark:text-white">

// 正确: 提取为组件或使用cn+变量
const cardStyles = cn(
  'flex items-center justify-between',
  'rounded-lg border p-4 shadow-sm',
  'transition-all hover:shadow-md',
  'border-gray-200 bg-white',
  'dark:border-gray-700 dark:bg-gray-800 dark:text-white'
)
```

### 陷阱2: CSS-in-JS导致SSR水合不匹配
```tsx
// 错误: 服务端和客户端生成的类名不一致
// 正确: 配置ServerStyleSheet(styled-components)
import { ServerStyleSheet } from 'styled-components'

// 在_document.tsx或SSR入口中正确收集样式
```

### 陷阱3: CSS变量未设置回退值
```css
/* 错误 */
color: var(--color-primary);

/* 正确 */
color: var(--color-primary, #3b82f6);
```

### 陷阱4: z-index混乱
```css
/* 错误: 随意设置z-index */
.modal { z-index: 9999; }
.dropdown { z-index: 99999; }

/* 正确: 统一z-index层级系统 */
:root {
  --z-dropdown: 100;
  --z-sticky: 200;
  --z-overlay: 300;
  --z-modal: 400;
  --z-toast: 500;
}
```

## Agent Checklist

### 架构选型
- [ ] 根据项目特点选择CSS方案
- [ ] 团队已达成一致并了解方案优缺点
- [ ] 设计Token系统已定义(颜色/间距/字体)

### 代码规范
- [ ] 组件样式隔离,无全局冲突风险
- [ ] 响应式设计遵循移动优先原则
- [ ] 暗色模式通过CSS变量实现
- [ ] z-index使用分层系统

### 性能优化
- [ ] 未使用的CSS已清除(PurgeCSS/Tree Shaking)
- [ ] 无运行时方案或已评估性能影响
- [ ] 关键CSS内联(首屏)
- [ ] 字体加载策略已优化(font-display: swap)

### 可维护性
- [ ] 样式文件与组件共同定位
- [ ] 复杂样式提取为可复用变体
- [ ] 命名语义化(描述用途而非外观)
- [ ] 设计Token覆盖所有常用值
