---
id: ui-full-lifecycle-cross-platform-playbook
title: UI 全生命周期跨平台设计手册（商业级）
domain: design
category: ui-full-lifecycle-cross-platform-playbook.md
difficulty: intermediate
tags: [cross, design, full, lifecycle, platform, playbook, token, 官方优先来源]
quality_score: 70
last_updated: 2026-06-15
---
# UI 全生命周期跨平台设计手册（商业级）

> 来源：Material Design 3、Apple HIG、Microsoft Fluent 2、Refactoring UI、TDesign、shadcn/ui、Tailwind CSS v4、微信小程序设计指南、48+ 开源设计系统、Awwwards 趋势研究
> 版本：2026-03-20
> 适用平台：Web / H5 / 微信小程序 / APP (iOS/Android) / 桌面端 (Electron/Tauri)

---

## 目标

- 为 Super Dev 提供可直接复用的 UI 全流程知识资产，覆盖 Web/H5/微信小程序/APP/桌面端
- 将"设计精美"转化为可执行标准：Token 体系、组件规格、平台规范、反模式检测、交付门禁
- 确保宿主生成的 UI 达到大厂商业产品标准，而非 AI 模板化输出

## 官方优先来源

| 来源 | URL | 重点 |
|:---|:---|:---|
| Material Design 3 | m3.material.io | 色彩三级 token、动态色、组件规范 |
| Apple HIG | developer.apple.com/design/human-interface-guidelines | iOS/macOS/visionOS 原生规范 |
| Microsoft Fluent 2 | fluent2.microsoft.design | 三层调色板、交互状态、桌面端 |
| Tailwind CSS v4 | tailwindcss.com | 26 色族 OKLCH 色阶、工具类 |
| shadcn/ui | ui.shadcn.com | 语义化 CSS 变量、8 种主题预设 |
| TDesign | tdesign.tencent.com | 腾讯设计规范、小程序组件 |
| 微信设计指南 | developers.weixin.qq.com/miniprogram/design/ | 小程序导航/反馈/适配 |
| Refactoring UI | refactoringui.com | 200+ 实战设计策略 |
| Design System Checklist | designsystemchecklist.com | 完整设计系统清单 |
| WCAG 2.1 | w3.org/WAI/standards-guidelines/wcag/ | 无障碍标准 |

---

## 一、设计 Token 体系（跨平台统一）

### 1.1 颜色 Token 三级架构（Material Design 3 方法）

**第一层：全局 Token（Global / Primitive）**
```css
/* 原始色值，不带语义 */
--color-blue-50: #EFF6FF;
--color-blue-500: #2563EB;
--color-blue-900: #1E3A8A;
--color-neutral-50: #F9FAFB;
--color-neutral-900: #111827;
```

**第二层：语义 Token（Alias / Semantic）**
```css
/* 功能映射，可按主题切换 */
--color-primary: var(--color-blue-500);
--color-on-primary: #FFFFFF;
--color-primary-container: var(--color-blue-50);
--color-surface: #FFFFFF;
--color-on-surface: var(--color-neutral-900);
--color-outline: var(--color-neutral-300);
```

**第三层：组件 Token（Component）**
```css
/* 组件级绑定，最具体 */
--button-bg-primary: var(--color-primary);
--button-bg-primary-hover: var(--color-blue-600);
--card-bg: var(--color-surface);
--card-border: var(--color-outline);
--input-border: var(--color-neutral-300);
--input-border-focus: var(--color-primary);
```

### 1.2 完整语义色集（每个项目必须定义）

| 角色 | 浅色模式 | 深色模式 | 用途 |
|:---|:---|:---|:---|
| **primary** | 品牌主色 | 主色浅变体 | 主要操作、强调元素 |
| **on-primary** | #FFFFFF | 主色深变体 | primary 上的文字/图标 |
| **primary-container** | 主色-50 | 主色-900 | 选中态背景、标签底色 |
| **secondary** | 辅助色 | 辅助色浅变体 | 次要操作、筛选器 |
| **tertiary** | 第三色 | 第三色浅变体 | 额外强调层 |
| **surface** | #FFFFFF | #0F172A | 卡片、容器背景 |
| **surface-variant** | #F1F5F9 | #1E293B | 输入框、区域背景 |
| **on-surface** | #111827 | #F8FAFC | 正文文本 |
| **on-surface-variant** | #6B7280 | #94A3B8 | 辅助文本 |
| **outline** | #D1D5DB | #475569 | 边框、分隔线 |
| **outline-variant** | #E5E7EB | #334155 | 弱边框 |
| **error** | #DC2626 | #F87171 | 错误状态 |
| **error-container** | #FEF2F2 | #450A0A | 错误背景 |
| **success** | #059669 | #34D399 | 成功状态 |
| **success-container** | #ECFDF5 | #064E3B | 成功背景 |
| **warning** | #D97706 | #FBBF24 | 警告状态 |
| **warning-container** | #FFFBEB | #78350F | 警告背景 |
| **info** | #2563EB | #60A5FA | 信息提示 |
| **info-container** | #EFF6FF | #1E3A8A | 信息背景 |
| **destructive** | #DC2626 | #F87171 | 删除、危险操作 |
| **muted** | #F1F5F9 | #1E293B | 禁用背景、分区底色 |
| **muted-foreground** | #64748B | #94A3B8 | 禁用/占位文字 |

### 1.3 色阶生成规则（Refactoring UI + Tailwind 方法）

每种品牌色必须生成 11 个色阶（50-950）：

| 色阶 | 亮度定位 | 典型用途 |
|:---|:---|:---|
| **50** | 极浅（97%亮度） | hover 背景、选中态底色 |
| **100** | 很浅（95%） | 浅色背景、标签底色 |
| **200** | 浅色（90%） | 边框、禁用态背景 |
| **300** | 中浅（80%） | 禁用态文字 |
| **400** | 中色（70%） | 占位文字 |
| **500** | 标准色 | 品牌色基准、按钮背景 |
| **600** | 中深（85%原色） | hover 态、链接悬停 |
| **700** | 深色（70%原色） | active/pressed 态 |
| **800** | 很深（55%原色） | 标题文字（深色模式 surface） |
| **900** | 极深（40%原色） | 正文文字（深色模式基底） |
| **950** | 最深（25%原色） | 深色模式最深表面 |

**关键原则（来自 Refactoring UI）**：
- 灰色不要用纯灰，加微暖色调（如 Tailwind 的 slate/zinc/stone）
- 用 HSL/OKLCH 调整而非直接插值 hex
- 饱和度随亮度变化不是线性的 - 中间色阶饱和度最高
- 深色阶段需要比浅色阶段更高的饱和度

### 1.4 字体 Token

| 级别 | 大小 | 字重 | 行高 | 字间距 | 用途 |
|:---|:---|:---|:---|:---|:---|
| **display-lg** | 56px / 3.5rem | 800 | 1.1 | -0.025em | 首屏超大标题 |
| **display** | 48px / 3rem | 800 | 1.1 | -0.02em | 区块大标题 |
| **h1** | 36px / 2.25rem | 700 | 1.2 | -0.015em | 页面标题 |
| **h2** | 28px / 1.75rem | 600 | 1.25 | -0.01em | 章节标题 |
| **h3** | 22px / 1.375rem | 600 | 1.3 | 0 | 卡片/模块标题 |
| **h4** | 18px / 1.125rem | 600 | 1.35 | 0 | 小标题 |
| **body-lg** | 18px / 1.125rem | 400 | 1.6 | 0 | 正文大字 |
| **body** | 16px / 1rem | 400 | 1.5 | 0 | 正文 |
| **body-sm** | 14px / 0.875rem | 400 | 1.5 | 0 | 辅助文本 |
| **caption** | 12px / 0.75rem | 500 | 1.4 | 0.02em | 标签/注释 |
| **overline** | 11px / 0.6875rem | 700 | 1.6 | 0.08em | 分类标签 |

**中文字体栈优先级**：
```css
font-family: 'Plus Jakarta Sans', 'Noto Sans SC', 'PingFang SC',
             'Hiragino Sans GB', 'Microsoft YaHei', sans-serif;
```

**字体加载策略**：
- 使用 `font-display: swap` 避免 FOIT
- 预加载关键字重（400/600/700）`<link rel="preload">`
- 中文字体使用系统字体栈避免加载延迟
- 英文字体用 Google Fonts 的 `&display=swap` 参数

### 1.5 间距 Token（8px 栅格）

| Token | 值 | 用途 |
|:---|:---|:---|
| **space-0** | 0px | 无间距 |
| **space-px** | 1px | 边框修正 |
| **space-0.5** | 2px | 微调 |
| **space-1** | 4px | 图标与文字间距 |
| **space-1.5** | 6px | 紧凑内间距 |
| **space-2** | 8px | 按钮内间距（紧凑） |
| **space-2.5** | 10px | 标签内间距 |
| **space-3** | 12px | 组件内间距（标准） |
| **space-4** | 16px | 卡片内边距（小） |
| **space-5** | 20px | 表单项间距 |
| **space-6** | 24px | 卡片内边距（标准） |
| **space-8** | 32px | 章节间距 |
| **space-10** | 40px | 区块间距 |
| **space-12** | 48px | 页面间距 |
| **space-16** | 64px | 大区块间距 |
| **space-20** | 80px | 首屏间距 |
| **space-24** | 96px | 超大区块间距 |

**间距原则（来自 Refactoring UI）**：
- 先给大间距再缩减，不是先紧凑再放大
- 用间距和背景色分隔元素，减少边框使用
- 相关元素间距小，不相关元素间距大（接近性原则）
- 不要所有地方都用相同间距 - 间距要传达信息层级

### 1.6 阴影 Token

| 层级 | CSS 值 | 用途 |
|:---|:---|:---|
| **shadow-xs** | `0 1px 2px rgba(0,0,0,0.05)` | 输入框、小元素 |
| **shadow-sm** | `0 1px 3px rgba(0,0,0,0.1), 0 1px 2px rgba(0,0,0,0.06)` | 按钮、标签 |
| **shadow-md** | `0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -2px rgba(0,0,0,0.1)` | 卡片、下拉菜单 |
| **shadow-lg** | `0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -4px rgba(0,0,0,0.1)` | 弹窗、浮层 |
| **shadow-xl** | `0 20px 25px -5px rgba(0,0,0,0.1), 0 8px 10px -6px rgba(0,0,0,0.1)` | 模态框 |
| **shadow-inner** | `inset 0 2px 4px rgba(0,0,0,0.05)` | 凹陷效果 |

**深色模式阴影**：深色模式下阴影效果减弱，用 surface 层级差异（surface-1 到 surface-5）代替阴影传达层级。

### 1.7 圆角 Token

| Token | 值 | 用途 |
|:---|:---|:---|
| **radius-none** | 0px | 无圆角（Brutalist 风格） |
| **radius-sm** | 4px | 小元素（badge/tag） |
| **radius-md** | 8px | 按钮、输入框 |
| **radius-lg** | 12px | 卡片 |
| **radius-xl** | 16px | 弹窗、浮层 |
| **radius-2xl** | 24px | 大卡片、模态框 |
| **radius-full** | 9999px | 圆形头像、pill 标签 |

### 1.8 动效 Token

| 类型 | 时长 | 缓动函数 | 用途 |
|:---|:---|:---|:---|
| **instant** | 0ms | - | prefers-reduced-motion 时 |
| **micro** | 100ms | ease-out | 颜色、透明度切换 |
| **short** | 150ms | ease-in-out | hover、focus 状态 |
| **medium** | 250ms | ease-in-out | 展开/收起、Tab 切换 |
| **long** | 400ms | cubic-bezier(0.4, 0, 0.2, 1) | 页面过渡、抽屉 |
| **extra-long** | 600ms | cubic-bezier(0.4, 0, 0.2, 1) | 复杂编排过渡 |

**动效强制规则**：
- 必须支持 `@media (prefers-reduced-motion: reduce)` 降级
- 300ms 以上的动效会让界面感觉迟钝 - 仅用于复杂过渡
- 动效必须有信息传达目的，不做纯装饰

---

## 二、组件样式规格（宿主必须遵守）

### 2.1 按钮系统

| 变体 | 背景 | 文字色 | 圆角 | 高度 | 内边距 | 阴影 | Hover 效果 |
|:---|:---|:---|:---|:---|:---|:---|:---|
| **Primary** | primary-500 | white | 8px | 40px | 16px 24px | shadow-sm | primary-600 + shadow-md |
| **Secondary** | white | neutral-700 | 8px | 40px | 16px 24px | ring-1 neutral-200 | neutral-50 bg |
| **Ghost** | transparent | neutral-600 | 8px | 40px | 16px 24px | none | neutral-100 bg |
| **Destructive** | error-500 | white | 8px | 40px | 16px 24px | shadow-sm | error-600 |
| **Outline** | transparent | primary-500 | 8px | 40px | 16px 24px | ring-1 primary-200 | primary-50 bg |
| **CTA (大)** | primary-500 | white | 12px | 48px | 20px 32px | shadow-md | primary-600 + scale-[1.02] |
| **Icon Only** | transparent | neutral-600 | 8px | 36px | 8px | none | neutral-100 bg |

**按钮尺寸变体**：
| 尺寸 | 高度 | 字号 | 内边距 |
|:---|:---|:---|:---|
| **sm** | 32px | 13px | 12px 16px |
| **md** | 40px | 14px | 16px 24px |
| **lg** | 48px | 16px | 20px 32px |

### 2.2 卡片系统

```css
/* 标准卡片 */
.card {
  background: var(--color-surface);
  border: 1px solid var(--color-outline-variant);
  border-radius: var(--radius-lg); /* 12px */
  padding: var(--space-6); /* 24px */
  transition: box-shadow var(--duration-short) ease,
              transform var(--duration-short) ease;
}
.card:hover {
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}

/* 高亮卡片 */
.card-featured {
  border: 2px solid var(--color-primary);
  box-shadow: 0 0 0 4px var(--color-primary-container);
}

/* 交互卡片 */
.card-interactive {
  cursor: pointer;
  user-select: none;
}
.card-interactive:active {
  transform: scale(0.98);
}
```

### 2.3 输入框系统

```css
.input {
  height: 40px;
  padding: 8px 12px;
  border: 1px solid var(--color-outline);
  border-radius: var(--radius-md); /* 8px */
  font-size: 14px;
  color: var(--color-on-surface);
  background: var(--color-surface);
  transition: border-color var(--duration-micro) ease,
              box-shadow var(--duration-micro) ease;
}
.input::placeholder {
  color: var(--color-muted-foreground);
}
.input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-container);
  outline: none;
}
.input-error {
  border-color: var(--color-error);
  box-shadow: 0 0 0 3px var(--color-error-container);
}
.input:disabled {
  background: var(--color-muted);
  color: var(--color-muted-foreground);
  cursor: not-allowed;
}
```

### 2.4 导航系统

**顶部导航栏**：
```css
.navbar {
  position: sticky;
  top: 0;
  z-index: 50;
  height: 64px;
  border-bottom: 1px solid var(--color-outline-variant);
  background: var(--color-surface) / 95%;
  backdrop-filter: blur(8px);
}
```

**侧边栏**：
- 展开宽度：240-280px
- 折叠宽度：64px
- 分隔：用背景色而非边框
- 活动项：primary-container 背景 + primary 文字

### 2.5 表格系统

| 元素 | 规格 |
|:---|:---|
| 表头 | 背景 muted、字重 600、字号 12px |
| 行高 | 48-56px |
| 行 hover | surface-variant 背景 |
| 选中行 | primary-container 背景 |
| 斑马纹 | 偶数行 muted/50% 背景 |
| 边框 | 仅水平分隔线 outline-variant |

---

## 三、平台特定设计规范

### 3.1 Web 端

**断点系统**：
| 名称 | 宽度范围 | 栅格列数 | 间隔 | 容器最大宽度 |
|:---|:---|:---|:---|:---|
| **xs** | < 640px | 4 | 16px | 100% |
| **sm** | 640-768px | 8 | 16px | 640px |
| **md** | 768-1024px | 8 | 24px | 768px |
| **lg** | 1024-1280px | 12 | 24px | 1024px |
| **xl** | 1280-1536px | 12 | 32px | 1280px |
| **2xl** | > 1536px | 12 | 32px | 1536px |

**Tailwind 响应式写法**：
```html
<div class="px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 lg:gap-6">
```

### 3.2 微信小程序

**官方字号规范**：
| 用途 | 大小 | 字重 | 颜色 |
|:---|:---|:---|:---|
| 大标题 | 22pt | 500 | #000000 |
| 页面标题 | 17pt | 500 | #000000 |
| 正文 | 15pt | 400 | #353535 |
| 辅助文字 | 14pt | 400 | #888888 |
| 备注/标签 | 12pt | 400 | #B2B2B2 |

**导航规范**：
- Tab 栏：2-5 个（推荐 4 个），可置顶或置底
- 官方菜单：固定右上角，不可覆盖，不可隐藏
- 返回按钮：二级页面必须有
- iOS 左滑返回 / Android 物理返回键

**反馈系统**：
| 类型 | 时长 | 用途 |
|:---|:---|:---|
| Icon Toast | 1.5s 自动消失 | 成功提示 |
| Text Toast | 1.5s 自动消失 | 轻量错误 |
| 半屏弹窗 | 用户主动关闭 | 需要确认的操作 |
| 全屏结果页 | 用户主动操作 | 流程终点 |

**触控规范**：最小触控目标 44rpx (约 7mm)

**包体管理**：
- 单包上限 2MB，总包上限 20MB
- 使用分包加载，主包只放核心页面
- 图片使用 CDN，不放本地

### 3.3 APP 端

**iOS（Apple HIG）**：
| 元素 | 规格 |
|:---|:---|
| 导航栏高度 | 44pt（大标题 96pt） |
| Tab 栏高度 | 49pt |
| 最小触控目标 | 44x44pt |
| 安全区域（顶部） | Dynamic Island / 刘海屏 |
| 安全区域（底部） | Home Indicator 34pt |
| 圆角类型 | Continuous corner |
| 系统字体 | SF Pro (英) / PingFang SC (中) |

**Android（Material Design 3）**：
| 元素 | 规格 |
|:---|:---|
| 导航栏高度 | 56dp（大屏 64dp） |
| 底部导航高度 | 80dp |
| 最小触控目标 | 48x48dp |
| FAB 标准尺寸 | 56dp |
| FAB 大尺寸 | 96dp |
| 卡片圆角 | 12dp |
| 按钮圆角 | 20dp（全圆角风格） |
| 系统字体 | Roboto (英) / Noto Sans CJK (中) |

### 3.4 桌面端（Electron / Tauri）

**窗口规范**：
| 平台 | 标题栏高度 | 按钮位置 | 最小窗口 |
|:---|:---|:---|:---|
| macOS | 32px | 左上（红绿灯） | 800x600px |
| Windows | 32px | 右上（最小/最大/关闭） | 800x600px |
| Linux | 32px | 右上（跟随 DE） | 800x600px |

**桌面端特有交互**：
- 必须支持 Cmd/Ctrl 快捷键（保存/撤销/搜索/关闭）
- Tab 键导航 + 方向键列表导航
- 右键上下文菜单
- 拖拽操作（文件/窗口/面板）
- 窗口大小和位置记忆
- 系统通知（macOS Notification Center / Windows Toast）
- 托盘图标和菜单

---

## 四、商业级 UI 反模式清单（强制检测）

### 4.1 AI 生成的典型问题

| 反模式 | 为什么是问题 | 正确做法 |
|:---|:---|:---|
| 紫/粉渐变主视觉 | AI 工具默认偏好，缺乏品牌感 | 使用产品专属配色方案 |
| Emoji 充当功能图标 | 跨平台不一致、不专业、无法搜索 | Lucide / Heroicons / Tabler SVG |
| 系统字体直出 | Inter/Arial 直出无品牌识别 | 定义品牌字体组合 + 字号层级 |
| 同质化卡片墙 | 所有模块同一层级无重点 | 建立信息层级和视觉重量差异 |
| 空洞 Hero 区 | 只有口号没有截图/数据/证据 | 产品截图 + 价值数据 + CTA |
| 边框泛滥 | 每个元素都有 1px 边框 | 用间距/背景色/阴影分隔 |
| 均匀间距 | 所有间距都是 16px | 按亲疏关系使用梯度间距 |
| 装饰过度 | 渐变/玻璃/光效/粒子堆砌 | 装饰服务于信息传达 |
| 灰色文字彩色背景 | 对比度不足看不清 | 用浅/深同色系文字 |
| 全屏填满 | 内容撑满整个视口 | 适当留白，内容最大宽度限制 |
| 行宽过大 | 文字行宽超过 75 字符 | 控制在 45-75 字符（中文 25-40 字） |

### 4.2 交付前检查清单（强制执行 - 门禁级别）

**视觉完整性**：
- [ ] 品牌字体已加载，非系统字体回退直出
- [ ] 配色使用项目定义的 Token，非硬编码 hex
- [ ] 所有 SVG 图标，零 emoji 图标
- [ ] 按钮/输入框高度统一（40px 基线）
- [ ] 卡片圆角统一（8-12px）
- [ ] 阴影层级正确（卡片 md、弹窗 lg、模态 xl）

**交互完整性**：
- [ ] 所有可点击元素有 cursor-pointer
- [ ] hover 状态使用 150-300ms 平滑过渡
- [ ] focus 状态对键盘导航可见（ring / outline）
- [ ] loading / empty / error / disabled 状态完整
- [ ] 表单有实时验证 + 提交时验证
- [ ] 尊重 prefers-reduced-motion 偏好

**可访问性**：
- [ ] 文字对比度 >= 4.5:1（WCAG AA）
- [ ] 图片有 alt 属性
- [ ] 交互元素有 aria-label
- [ ] 语义化 HTML 标签（main/nav/header/footer/section）
- [ ] Tab 键可以完整导航所有交互元素

**响应式**：
- [ ] 覆盖 375px / 768px / 1024px / 1440px 断点
- [ ] 触控目标 >= 44px（移动端）
- [ ] 图片支持 srcset 多分辨率
- [ ] 长文本有 text-overflow 处理

---

## 五、产品类型设计策略速查

### 5.1 Landing / 营销页

**首屏必须包含**：价值主张 + 产品截图/演示 + 主 CTA + 信任标识
**页面节奏**：Hero > 信任区 > 核心能力 > 使用场景 > 案例/数据 > 定价 > FAQ > CTA + Footer
**关键指标**：5 秒内让用户理解产品是什么、解决什么问题
**禁止**：空洞口号、纯装饰区块、没有 CTA 的区块

### 5.2 SaaS / 工作台

**核心界面**：全局导航 + 侧边栏 + 内容区 + 操作面板
**信息密度**：medium-high，关键数据突出，次要信息折叠
**关键要求**：状态透明（loading/empty/error）、操作可撤销、快捷键支持
**禁止**：把工作台做成营销页、大面积空白、只有图表没有操作路径

### 5.3 Dashboard / 数据看板

**核心能力**：一屏读懂风险与优先级
**布局策略**：顶部 KPI 卡片 + 中部图表区 + 底部数据表格
**关键要求**：数据可读（读出结论不是看图表）、操作直达（数据到操作 <= 2 步）
**禁止**：装饰性图表、没有业务含义的可视化

### 5.4 电商

**核心流程**：浏览 > 详情 > 加购 > 结算 > 支付 > 确认
**信任要素**：商品实拍、用户评价、退换政策、安全支付标识
**关键要求**：移动优先、拇指热区、价格和 CTA 始终可见
**禁止**：复杂动画干扰下单、价格分散、缺少信任元素

### 5.5 内容平台

**核心体验**：阅读效率 + 内容发现
**排版要求**：正文行宽 45-75 字符、行高 1.5-1.8、段间距明确
**关键要求**：字体可读性优先、广告不干扰阅读、深色模式支持
**禁止**：视觉噪音打断阅读、正文层级混乱

---

## 六、设计系统成熟度参考

### 6.1 业界大厂设计系统对标

| 公司 | 系统名 | Token 层级 | 组件数 | 平台覆盖 | 开源 |
|:---|:---|:---|:---|:---|:---|
| Google | Material Design 3 | 3 级(global/alias/component) | 50+ | Web/Android/iOS/Flutter | 是 |
| Apple | HIG | 语义化 | 100+ | iOS/macOS/watchOS/visionOS | 否 |
| Microsoft | Fluent 2 | 3 层(neutral/shared/brand) | 80+ | Web/Windows/macOS | 是 |
| IBM | Carbon | 完整色阶 | 60+ | Web/React | 是 |
| GitHub | Primer | 完整色阶 | 30+ | Web/React | 是 |
| Salesforce | Lightning | 40+ token | 80+ | Web/React | 是 |
| Ant Design | 蚂蚁设计 | 完整色阶 | 60+ | Web/React/Vue | 是 |
| TDesign | 腾讯设计 | 完整色阶 | 50+ | Web/小程序/React/Vue | 是 |
| Atlassian | ADS | 30+ token | 40+ | Web/React | 是 |
| shadcn/ui | shadcn | 语义 CSS 变量 | 40+ | Web/React | 是 |

### 6.2 设计系统完整清单（来自 Design System Checklist）

**设计语言层**：品牌愿景 + 设计原则 + 语气 + 术语 + 品牌资产
**基础层**：颜色 + 布局 + 字体 + 高度/阴影 + 动效 + 图标
**组件层**：44 种核心组件（按钮/卡片/表单/导航/反馈/布局）
**维护层**：文档 + 本地库 + 团队流程 + 社区 + 贡献指南

---

## 七、强约束（可作为质量门禁）

1. 必须有完整 Token 体系（颜色/字体/间距/圆角/阴影/动效），禁止大面积硬编码样式
2. 必须覆盖关键状态（loading/empty/error/disabled/success），禁止只交正常态
3. 必须具备跨端策略，至少覆盖 Web + H5（移动端适配）
4. 必须有交付验证证据（截图/运行报告），禁止仅口头说明"已完成"
5. 必须通过无障碍基线（对比度 4.5:1、键盘可达、语义标签）
6. 必须使用项目专属配色方案，禁止 AI 默认的紫/粉渐变
7. 必须使用 SVG 图标库，禁止 Emoji 充当功能图标
8. 组件库必须做品牌化 Token 重写，禁止默认样式直出
