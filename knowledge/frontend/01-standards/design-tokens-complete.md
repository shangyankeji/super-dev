---
id: design-tokens-complete
title: Design Token 完整指南
domain: frontend
category: 01-standards
difficulty: intermediate
tags: [complete, design, frontend, tailwind, token, tokens, 分层架构, 命名规范]
quality_score: 70
last_updated: 2026-06-15
---
# Design Token 完整指南

## 概述

Design Token 是设计系统的最小单元，用于存储颜色、字体、间距、圆角、阴影等视觉属性的命名变量。Token 充当设计与开发之间的单一事实来源 (Single Source of Truth)，确保品牌一致性、多主题支持和跨平台适配。

### 为什么需要 Design Token

- **一致性**: 全产品使用统一的视觉语言
- **可维护性**: 修改一个 Token 即可全局生效
- **多主题**: 浅色/深色/品牌主题只需切换 Token 集
- **跨平台**: 同一 Token 输出 CSS / iOS / Android / Flutter 等格式
- **设计-开发协同**: 设计师和开发者使用同一套命名系统

---

## Token 分层架构

### 三层模型

```
┌──────────────────────────────────┐
│  Component Token (组件级)         │   button-primary-bg
│  直接绑定到组件属性                │   card-border-radius
├──────────────────────────────────┤
│  Semantic Token (语义级)          │   color-bg-primary
│  表达意图，不绑定具体组件           │   spacing-lg
├──────────────────────────────────┤
│  Primitive Token (基础级)         │   blue-600
│  原始设计值，直接对应色值/数值       │   16px
└──────────────────────────────────┘
```

### 示例映射

```
Primitive:   blue-600 = #2563EB
Semantic:    color-bg-primary = {blue-600}
Component:   button-primary-bg = {color-bg-primary}
```

---

## Token 命名规范

### 命名结构

```
{category}-{property}-{modifier}-{state}

# 示例
color-bg-primary           # 类别-属性-修饰符
color-text-secondary       # 类别-属性-修饰符
spacing-lg                 # 类别-修饰符
radius-md                  # 类别-修饰符
shadow-card-hover          # 类别-修饰符-状态
font-size-heading-1        # 类别-属性-修饰符
```

### 颜色 Token

```css
:root {
  /* Primitive: 原始色板 */
  --primitive-blue-50: #EFF6FF;
  --primitive-blue-100: #DBEAFE;
  --primitive-blue-500: #3B82F6;
  --primitive-blue-600: #2563EB;
  --primitive-blue-700: #1D4ED8;
  --primitive-blue-900: #1E3A5F;

  --primitive-gray-50: #F9FAFB;
  --primitive-gray-100: #F3F4F6;
  --primitive-gray-200: #E5E7EB;
  --primitive-gray-500: #6B7280;
  --primitive-gray-700: #374151;
  --primitive-gray-900: #111827;

  /* Semantic: 语义色 */
  --color-bg-page: var(--primitive-gray-50);
  --color-bg-surface: #FFFFFF;
  --color-bg-primary: var(--primitive-blue-600);
  --color-bg-primary-hover: var(--primitive-blue-700);
  --color-bg-danger: #DC2626;

  --color-text-primary: var(--primitive-gray-900);
  --color-text-secondary: var(--primitive-gray-500);
  --color-text-on-primary: #FFFFFF;
  --color-text-link: var(--primitive-blue-600);
  --color-text-danger: #DC2626;

  --color-border-default: var(--primitive-gray-200);
  --color-border-focus: var(--primitive-blue-500);
}
```

### 间距 Token

```css
:root {
  /* 基于 4px 网格系统 */
  --spacing-0: 0px;
  --spacing-1: 4px;      /* 0.25rem */
  --spacing-2: 8px;      /* 0.5rem */
  --spacing-3: 12px;     /* 0.75rem */
  --spacing-4: 16px;     /* 1rem */
  --spacing-5: 20px;     /* 1.25rem */
  --spacing-6: 24px;     /* 1.5rem */
  --spacing-8: 32px;     /* 2rem */
  --spacing-10: 40px;    /* 2.5rem */
  --spacing-12: 48px;    /* 3rem */
  --spacing-16: 64px;    /* 4rem */
  --spacing-20: 80px;    /* 5rem */
  --spacing-24: 96px;    /* 6rem */

  /* 语义间距 */
  --spacing-page-x: var(--spacing-6);       /* 页面水平内边距 */
  --spacing-section-y: var(--spacing-16);   /* 区块垂直间距 */
  --spacing-card-padding: var(--spacing-6); /* 卡片内边距 */
  --spacing-input-x: var(--spacing-3);      /* 输入框水平内边距 */
  --spacing-input-y: var(--spacing-2);      /* 输入框垂直内边距 */
}
```

### 字体 Token

```css
:root {
  /* 字体族 */
  --font-family-sans: "Inter", "Noto Sans SC", system-ui, -apple-system, sans-serif;
  --font-family-mono: "JetBrains Mono", "Fira Code", monospace;

  /* 字号 */
  --font-size-xs: 0.75rem;     /* 12px */
  --font-size-sm: 0.875rem;    /* 14px */
  --font-size-base: 1rem;      /* 16px */
  --font-size-lg: 1.125rem;    /* 18px */
  --font-size-xl: 1.25rem;     /* 20px */
  --font-size-2xl: 1.5rem;     /* 24px */
  --font-size-3xl: 1.875rem;   /* 30px */
  --font-size-4xl: 2.25rem;    /* 36px */

  /* 字重 */
  --font-weight-regular: 400;
  --font-weight-medium: 500;
  --font-weight-semibold: 600;
  --font-weight-bold: 700;

  /* 行高 */
  --line-height-tight: 1.25;
  --line-height-normal: 1.5;
  --line-height-relaxed: 1.75;

  /* 语义字体组合 */
  --font-heading-1: var(--font-weight-bold) var(--font-size-4xl)/var(--line-height-tight) var(--font-family-sans);
  --font-heading-2: var(--font-weight-semibold) var(--font-size-3xl)/var(--line-height-tight) var(--font-family-sans);
  --font-body: var(--font-weight-regular) var(--font-size-base)/var(--line-height-normal) var(--font-family-sans);
  --font-caption: var(--font-weight-regular) var(--font-size-sm)/var(--line-height-normal) var(--font-family-sans);
}
```

### 圆角与阴影 Token

```css
:root {
  /* 圆角 */
  --radius-none: 0;
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-xl: 16px;
  --radius-full: 9999px;

  /* 阴影 */
  --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1);
  --shadow-xl: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1);

  /* 动效 */
  --duration-fast: 150ms;
  --duration-normal: 250ms;
  --duration-slow: 400ms;
  --easing-default: cubic-bezier(0.4, 0, 0.2, 1);
  --easing-in: cubic-bezier(0.4, 0, 1, 1);
  --easing-out: cubic-bezier(0, 0, 0.2, 1);
}
```

---

## 多主题支持

### CSS 变量主题切换

```css
/* 浅色主题（默认） */
:root, [data-theme="light"] {
  --color-bg-page: #F9FAFB;
  --color-bg-surface: #FFFFFF;
  --color-text-primary: #111827;
  --color-text-secondary: #6B7280;
  --color-border-default: #E5E7EB;
}

/* 深色主题 */
[data-theme="dark"] {
  --color-bg-page: #0F172A;
  --color-bg-surface: #1E293B;
  --color-text-primary: #F1F5F9;
  --color-text-secondary: #94A3B8;
  --color-border-default: #334155;
}

/* 跟随系统 */
@media (prefers-color-scheme: dark) {
  :root:not([data-theme]) {
    --color-bg-page: #0F172A;
    --color-bg-surface: #1E293B;
    --color-text-primary: #F1F5F9;
    --color-text-secondary: #94A3B8;
    --color-border-default: #334155;
  }
}
```

### 主题切换实现

```tsx
// React 主题切换 Hook
function useTheme() {
  const [theme, setTheme] = useState<"light" | "dark" | "system">(() => {
    return (localStorage.getItem("theme") as "light" | "dark") || "system";
  });

  useEffect(() => {
    const root = document.documentElement;
    if (theme === "system") {
      root.removeAttribute("data-theme");
    } else {
      root.setAttribute("data-theme", theme);
    }
    localStorage.setItem("theme", theme);
  }, [theme]);

  return { theme, setTheme };
}
```

---

## Tailwind CSS 集成

### tailwind.config.ts 映射

```typescript
import type { Config } from "tailwindcss";

const config: Config = {
  theme: {
    extend: {
      colors: {
        bg: {
          page: "var(--color-bg-page)",
          surface: "var(--color-bg-surface)",
          primary: "var(--color-bg-primary)",
        },
        text: {
          primary: "var(--color-text-primary)",
          secondary: "var(--color-text-secondary)",
        },
        border: {
          default: "var(--color-border-default)",
        },
      },
      spacing: {
        "page-x": "var(--spacing-page-x)",
        "section-y": "var(--spacing-section-y)",
      },
      borderRadius: {
        card: "var(--radius-md)",
        button: "var(--radius-md)",
      },
      boxShadow: {
        card: "var(--shadow-md)",
      },
      fontFamily: {
        sans: ["var(--font-family-sans)"],
        mono: ["var(--font-family-mono)"],
      },
    },
  },
};

export default config;
```

---

## 工具链

### Style Dictionary

```json
// tokens/color/base.json
{
  "color": {
    "blue": {
      "600": { "value": "#2563EB", "type": "color" }
    },
    "bg": {
      "primary": {
        "value": "{color.blue.600}",
        "type": "color",
        "description": "Primary background color"
      }
    }
  }
}
```

```javascript
// config.js
module.exports = {
  source: ["tokens/**/*.json"],
  platforms: {
    css: {
      transformGroup: "css",
      buildPath: "output/css/",
      files: [{ destination: "design-tokens.css", format: "css/variables" }],
    },
    ios: {
      transformGroup: "ios-swift",
      buildPath: "output/ios/",
      files: [{ destination: "DesignTokens.swift", format: "ios-swift/class.swift" }],
    },
    android: {
      transformGroup: "android",
      buildPath: "output/android/",
      files: [{ destination: "design_tokens.xml", format: "android/resources" }],
    },
  },
};
```

### Figma Token Studio

```
Figma Token Studio -> JSON export -> Style Dictionary -> CSS / iOS / Android
```

---

## 组件级 Token 使用

```css
/* 按钮组件 Token */
.btn {
  padding: var(--spacing-input-y) var(--spacing-input-x);
  border-radius: var(--radius-md);
  font: var(--font-body);
  transition: all var(--duration-fast) var(--easing-default);
}

.btn-primary {
  background: var(--color-bg-primary);
  color: var(--color-text-on-primary);
}

.btn-primary:hover {
  background: var(--color-bg-primary-hover);
}

/* 卡片组件 Token */
.card {
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  padding: var(--spacing-card-padding);
  box-shadow: var(--shadow-sm);
}
```

---

## 治理与维护

### Token 变更流程

1. 设计师在 Figma 更新 Token
2. 导出 JSON 到仓库
3. CI 自动运行 Style Dictionary 构建
4. 视觉回归测试验证变更
5. PR Review 后合并

### Token 审计

```bash
# 检查未使用的 Token
grep -r "var(--" src/ | grep -oP "var\(--[^)]+\)" | sort -u > used-tokens.txt
grep -oP "^\s*--[^:]+:" styles/tokens.css | sed 's/://;s/^\s*//' | sort -u > defined-tokens.txt
diff defined-tokens.txt used-tokens.txt
```

---

## 常见反模式

| 反模式 | 问题 | 正确做法 |
|--------|------|----------|
| 直接写死色值 | 主题无法切换 | 始终引用 Token |
| Primitive Token 直接用于组件 | 语义丢失 | 用 Semantic Token 间接引用 |
| Token 命名含具体色值 | 深色模式失效 | 按意图命名 (primary, danger) |
| Token 数量爆炸 | 维护困难 | 控制在 100-200 个 |
| 不同组件重复定义相同值 | 不一致风险 | 统一为共享 Token |

---

## Agent Checklist

在 AI 编码流水线中实现 Design Token 时，必须逐项检查：

- [ ] Token 采用三层架构（Primitive -> Semantic -> Component）
- [ ] 命名遵循 {category}-{property}-{modifier}-{state} 结构
- [ ] 颜色按语义命名（bg-primary, text-secondary），不按色值命名
- [ ] 间距基于 4px 网格（4/8/12/16/24/32/48/64）
- [ ] 字体定义族、大小、字重、行高的完整 Token
- [ ] 支持浅色/深色主题切换（CSS 变量 + data-theme）
- [ ] 深色主题由显式 Token 映射生成，不是简单反转
- [ ] Tailwind 配置映射到 Token 变量
- [ ] 所有组件样式通过 Token 引用，不直接写魔法数字
- [ ] Token 数量控制在 100-200 个以内
- [ ] Token 变更有审计流程（设计同步 -> 构建 -> 视觉回归）
- [ ] Token 文件是机器可解析的 JSON/YAML 格式
