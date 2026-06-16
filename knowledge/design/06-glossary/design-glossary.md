---
id: design-glossary
title: 设计术语表 (Design Glossary)
domain: design
category: 06-glossary
difficulty: intermediate
tags: [accessibility, agent, checklist, design, glossary, typography, 响应式与自适应设计, 工具与流程]
quality_score: 70
last_updated: 2026-06-15
---
# 设计术语表 (Design Glossary)

> 收录 40+ 核心设计术语，覆盖设计系统、响应式设计、无障碍、排版、工具链等领域。
> 适用于设计评审、前端开发、设计系统建设、团队协作等场景。

---

## 设计系统与方法论

### Design Token (设计令牌)

设计系统中最小粒度的视觉变量，以键值对形式定义颜色、字号、间距、阴影、圆角等。跨平台通用（Web / iOS / Android），是设计师与开发者之间的单一事实来源。

- 格式标准：W3C Design Tokens Format
- 工具：Style Dictionary（Amazon）、Tokens Studio（Figma 插件）
- 层级：Global Token → Alias Token → Component Token
- 示例：`color-primary-600: #1A73E8`、`spacing-md: 16px`、`radius-lg: 12px`

好的 Token 体系使得品牌换肤、暗色模式、多主题切换只需修改一层变量。

### Atomic Design (原子设计)

Brad Frost 提出的设计方法论，将 UI 从小到大分为五个层级：

1. **Atoms（原子）**：最基础的 UI 元素，如按钮、输入框、标签、图标
2. **Molecules（分子）**：由原子组合而成的简单组件，如搜索栏 = 输入框 + 按钮
3. **Organisms（有机体）**：由分子组成的复杂区域，如导航栏、商品卡片列表
4. **Templates（模板）**：页面的骨架结构，定义内容区域的布局
5. **Pages（页面）**：填充真实数据的最终页面

帮助团队系统化地构建和组织组件，从底层保证一致性。

### Design System (设计系统)

一套可复用的设计原则、组件、模式和工具的集合，确保产品视觉和交互的一致性。完整的设计系统包含：

- **Design Token（变量层）**：颜色、字号、间距等基础变量
- **Component Library（组件层）**：可复用的 UI 组件
- **Pattern Library（模式层）**：常见交互模式（如表单验证、列表加载、空状态）
- **设计规范文档**：使用指南、最佳实践、Do & Don't

代表性设计系统：Material Design（Google）、Ant Design（蚂蚁）、Arco Design（字节）、Semi Design（字节）、Carbon（IBM）。

### Component Library (组件库)

预构建的 UI 组件集合，包含样式、交互行为和 API 接口。

- 设计侧在 Figma 中维护组件库（含 Variants 变体）
- 开发侧在代码中维护对应实现（React / Vue / Angular）
- 两侧应保持同步（Design-Code Parity）
- 每个组件应有完整的 Props/Variants 和使用文档

好的组件库减少重复工作、保证一致性、降低维护成本。

### Design Handoff (设计交付)

设计师将设计稿交付给开发团队的流程和产物：

- **标注**：间距、颜色（Token 名）、字号、行高
- **切图**：图标（SVG）、图片（PNG/WebP）
- **交互说明**：状态转换、动画参数、手势操作
- **响应式行为**：不同断点下的布局变化
- **边界情况**：超长文本、极端数据、权限差异

工具：Figma Dev Mode、Zeplin。好的 Handoff 减少沟通成本和还原偏差。

---

## 响应式与自适应设计

### Responsive Design (响应式设计)

使用流式布局（Fluid Layout）、弹性媒体和媒体查询（Media Query），让同一套代码在不同屏幕尺寸上自动调整布局和样式。

核心理念：内容驱动断点，而非设备驱动断点。优先 Mobile First 策略（从小屏开始设计，逐步增强）。

技术基础：CSS Flexbox、CSS Grid、Media Query、相对单位（rem / vw / %）。

### Adaptive Design (自适应设计)

为不同屏幕尺寸设计多套固定布局，通过服务端或客户端检测设备后返回对应版本。

与 Responsive 的区别：
- **Responsive**：一套代码流式适配所有尺寸
- **Adaptive**：多套代码分别适配不同尺寸

Adaptive 在性能和体验上可能更优（每套代码更精简），但维护成本更高。实践中两者常结合使用。

### Breakpoint (断点)

媒体查询中触发布局变化的屏幕宽度阈值。常见断点配置：

- **Mobile**：< 768px（单列布局）
- **Tablet**：768px - 1024px（双列布局）
- **Desktop**：1024px - 1440px（三列/侧边栏布局）
- **Wide Desktop**：> 1440px（最大宽度限制，内容居中）

原则：应根据内容需要设置断点，而非固定的设备宽度。当布局在某个宽度"看起来不对"时，就是需要断点的地方。

### Grid System (栅格系统)

将页面水平划分为等宽列（Column）和间距（Gutter），用于对齐和组织内容：

- **12 列栅格**：Bootstrap、最通用，可被 2/3/4/6 整除
- **24 列栅格**：Ant Design，更精细的布局控制
- 参数：列数（Columns）、列宽（Column Width）、间距（Gutter）、边距（Margin）

响应式栅格在不同断点调整列数：Mobile 4 列 → Tablet 8 列 → Desktop 12 列。

### Fluid Layout (流式布局)

使用百分比、vw/vh、flex、grid 等相对单位定义宽度，使布局在断点之间也能平滑缩放。与固定布局（Fixed Layout，像素定宽）相对。是 Responsive Design 的基础技术。

### Container Query (容器查询)

CSS 新特性，允许组件根据其父容器的尺寸（而非视窗尺寸）调整样式。使组件真正可复用：同一组件放在侧边栏（300px 宽）和主内容区（800px 宽）可自动适配不同宽度。

```css
@container (min-width: 400px) {
  .card { flex-direction: row; }
}
```

---

## 无障碍设计 (Accessibility)

### A11y (Accessibility 的缩写)

"A" + 中间 11 个字母 + "y" 的数字缩写。指确保产品可被所有人使用，包括视觉、听觉、运动和认知障碍用户。不仅是道德责任，在许多国家也是法律要求（ADA、EU Directive 2016/2102、中国《信息无障碍规范》）。

全球约 15% 的人口有某种形式的残障，无障碍设计同时也改善了所有用户的体验（如高对比度在阳光下更易读）。

### WCAG (Web Content Accessibility Guidelines)

W3C 发布的 Web 内容无障碍指南，当前版本 WCAG 2.2。定义三个合规级别：

- **A（基本）**：最低要求，如图片有替代文字
- **AA（推荐）**：大多数法规要求的级别
- **AAA（最高）**：最严格，适合政府和教育网站

四大原则（POUR）：
1. **可感知 (Perceivable)**：信息可通过多种感官获取
2. **可操作 (Operable)**：界面可通过多种方式操作
3. **可理解 (Understandable)**：内容和操作易于理解
4. **健壮 (Robust)**：与辅助技术兼容

### Color Contrast (颜色对比度)

前景色与背景色之间的亮度比值。WCAG AA 要求：

- **正文文字**：对比度 >= 4.5:1
- **大文字**（>= 18pt 或 14pt 粗体）：对比度 >= 3:1
- **UI 组件和图形**：对比度 >= 3:1

工具：Stark（Figma 插件）、Colour Contrast Analyser、Chrome DevTools、WebAIM Contrast Checker。

### Screen Reader (屏幕阅读器)

将屏幕内容转换为语音或盲文输出的辅助技术：

- **VoiceOver**：macOS / iOS 内置
- **NVDA**：Windows 免费开源
- **JAWS**：Windows 商用
- **TalkBack**：Android 内置

开发者需确保：语义化 HTML、正确的 ARIA 属性、合理的焦点管理、有意义的替代文字。

### ARIA (Accessible Rich Internet Applications)

W3C 定义的一组 HTML 属性，用于增强动态 Web 内容的无障碍性：

- `role`：定义元素的语义角色（如 `role="dialog"`）
- `aria-label`：提供替代文字（如图标按钮）
- `aria-expanded`：标识展开/折叠状态
- `aria-live`：标记动态内容区域，变化时通知屏幕阅读器
- `aria-hidden`：对辅助技术隐藏装饰性元素

核心原则：能用原生 HTML 语义的就不用 ARIA（"No ARIA is better than bad ARIA"）。

### Focus Management (焦点管理)

控制键盘焦点的位置和顺序，确保用户可以通过 Tab 键导航所有可交互元素。

关键场景：
- 弹窗打开时焦点移入弹窗，关闭时焦点回到触发元素
- SPA 路由切换时焦点归位到页面顶部或主内容区
- 焦点不应进入视觉隐藏的元素
- 焦点环（Focus Ring）必须可见（不要 `outline: none` 不加替代）

---

## 排版 (Typography)

### Typography Scale (字体比例尺)

一组基于数学比例关系的字号序列，确保页面文字层级和谐。常用比例：

- **Minor Second (1.067)**：极紧凑，适合空间受限场景
- **Major Third (1.25)**：中等，适合大多数 Web 应用
- **Perfect Fourth (1.333)**：较松散，适合内容型网站
- **Golden Ratio (1.618)**：视觉张力强，适合标题突出的设计

示例（基准 16px，比例 1.25）：12px → 16px → 20px → 25px → 31px → 39px。

工具：Type Scale (typescale.com)。

### Line Height (行高)

文本行与行之间的垂直间距：

- **正文**：行高 1.5 - 1.75 倍字号
- **标题**：行高 1.1 - 1.3 倍字号
- **紧凑列表**：行高 1.3 - 1.5 倍字号

CSS 中使用无单位值（如 `line-height: 1.5`）以确保与字号成比例。行高过小影响可读性（特别是中文），过大导致段落松散失去凝聚力。

### Font Pairing (字体搭配)

选择两种及以上字体组合使用：

- 通常一种用于标题（Display / Serif），一种用于正文（Sans-serif）
- 原则：对比明显但风格协调
- 同一字体家族的不同粗细也是有效搭配
- 避免使用超过 3 种字体（增加视觉噪音和加载时间）

中文推荐：思源黑体（正文）+ 思源宋体（标题），或系统字体栈。

### Variable Font (可变字体)

单个字体文件包含多个可变轴（粗细 / 宽度 / 光学大小 / 倾斜等），通过 CSS `font-variation-settings` 控制。

相比传统方式加载多个字重文件（Regular + Medium + Bold + ...），可变字体大幅减少网络请求和文件体积。代表：Inter、Source Sans 3、Noto Sans SC。

---

## 工具与流程

### Figma

基于浏览器的协作设计工具，支持实时多人编辑。核心功能：

- **Auto Layout**：类似 CSS Flexbox 的自动布局
- **Components & Variants**：组件与变体管理
- **Design Tokens**：通过插件（Tokens Studio）管理变量
- **Prototyping**：交互原型，支持 Smart Animate
- **Dev Mode**：开发标注和代码生成

是当前主流的 UI 设计工具，替代了 Sketch + InVision 的组合。

### Storybook

开源的 UI 组件开发和文档工具。在隔离环境中开发、测试和展示组件：

- 通过 Stories 展示组件的不同状态和变体
- 支持 React / Vue / Angular / Web Components 等框架
- 插件生态：视觉回归测试（Chromatic）、无障碍检查（a11y addon）、交互测试（play function）
- 自动生成组件文档

是组件驱动开发（Component-Driven Development）的核心工具。

### Design Lint (设计检查)

自动检测设计稿中不符合规范的元素：

- 硬编码颜色值（非 Token 引用）
- 间距不对齐网格
- 未使用设计系统中的组件
- 字号不在 Typography Scale 中
- 圆角值不一致

类似代码中的 ESLint。工具：Figma Design Lint 插件、Stylelint（CSS 层面）。在设计评审前运行，减少人工检查成本。

### Design Sprint (设计冲刺)

Google Ventures 提出的 5 天设计方法论：

- **Day 1**：理解问题（用户/业务/技术约束）
- **Day 2**：发散方案（Sketch 多种解决方案）
- **Day 3**：决策（投票选择最佳方案）
- **Day 4**：制作原型（高保真可点击原型）
- **Day 5**：用户测试（5 位用户验证）

适合在短时间内验证设计方向的可行性，减少大规模投入前的风险。

### User Flow (用户流程图)

用图示描述用户完成特定目标的步骤序列，包括页面/状态节点和操作/条件连线。帮助设计师和开发理解全局交互路径，发现遗漏的分支和异常场景。

工具：FigJam、Miro、draw.io、Whimsical。

### Wireframe (线框图)

低保真度的页面结构草图，关注信息架构和布局，不含颜色、字体、图片细节。用于快速沟通设计方向、验证信息层级，成本低、迭代快。线框图确认后再进入高保真设计阶段。

### Mockup (视觉稿)

高保真度的静态设计稿，包含完整的颜色、字体、图片和细节。是设计评审和开发实现的主要依据。通常在 Figma 中制作，与设计系统组件关联。

### Prototype (交互原型)

可点击的交互演示，模拟用户操作流程。从低保真（可点击线框图）到高保真（接近真实交互的动画和状态）。用途：可用性测试、利益相关者演示、开发沟通。

Figma Prototype 支持：过渡动画、Smart Animate、条件逻辑、变量控制。

---

## Agent Checklist

- [ ] Design Token 已定义覆盖颜色、字号、间距、阴影、圆角
- [ ] 组件库遵循 Atomic Design 分层（原子/分子/有机体）
- [ ] Typography Scale 使用一致的数学比例
- [ ] 响应式设计覆盖至少 3 个断点（Mobile / Tablet / Desktop）
- [ ] 颜色对比度符合 WCAG AA 标准（正文 >= 4.5:1）
- [ ] 焦点管理和键盘导航已在设计中考虑
- [ ] ARIA 属性正确使用，优先原生 HTML 语义
- [ ] Storybook 覆盖所有组件的核心状态和变体
- [ ] 设计交付物（标注/切图/交互说明）完整
- [ ] Design Lint 在评审前运行无违规
