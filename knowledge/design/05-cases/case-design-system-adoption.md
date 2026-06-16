---
id: case-design-system-adoption
title: 案例：设计系统从零到全团队采用
domain: design
category: 05-cases
difficulty: intermediate
tags: [adoption, agent, case, checklist, design, system, 关键成功因素, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# 案例：设计系统从零到全团队采用

## 概述

本案例记录一家中型 SaaS 公司（约 200 人研发团队，8 个产品线）从零开始搭建设计系统，到全团队采用的完整过程。项目历时 9 个月，最终实现组件复用率从 15% 提升至 82%，设计到开发交付周期缩短 40%，UI 一致性评分从 45 分提升至 91 分。本案例涵盖前期调研、核心搭建、推广策略、度量指标和踩过的坑。

---

## 背景

### 组织现状

- **团队规模**：6 名设计师、40 名前端工程师、8 个产品线
- **技术栈**：React + TypeScript，部分产品线使用 Vue
- **设计工具**：Figma（刚从 Sketch 迁移完成）
- **现有状态**：无统一组件库，各产品线自建 UI 组件，风格差异大

### 核心痛点

1. **视觉不一致**：同一品牌下 8 个产品看起来像 8 家公司
2. **重复造轮子**：每个产品线都有自己的 Button、Modal、Form 组件
3. **交付效率低**：设计师每次都从零画组件，开发每次都从零写组件
4. **维护成本高**：修改品牌色需要在 8 个代码库中分别修改
5. **设计走查周期长**：每次走查发现大量一致性问题

### 关键决策

立项前团队面临的核心选择：

| 选项 | 优点 | 缺点 | 决策 |
|------|------|------|------|
| 采用开源 UI 库（Ant Design / MUI） | 快速启动 | 品牌感弱，定制成本高 | 否决 |
| 从最大产品线提取组件 | 有现成代码 | 其他产品线不认同 | 否决 |
| 从零搭建品牌设计系统 | 高度定制，全团队认同 | 初期投入大 | 采纳 |

---

## 第一阶段：审计与规划（第 1-2 月）

### 1.1 组件审计

对 8 个产品线进行组件盘点：

```
审计结果摘要：
- Button 变体：23 种（应统一为 5 种）
- 颜色值：147 个（应统一为 36 个）
- 字号：28 种（应统一为 8 种）
- 间距值：41 种（应统一为 10 种）
- 圆角值：12 种（应统一为 5 种）
- 模态框实现：6 种不同的 Modal 组件
```

### 1.2 利益相关方访谈

访谈了 30 人（设计师、前端、PM、技术负责人），关键发现：

- 设计师最关心：减少重复绘制，提升设计一致性
- 前端最关心：API 设计合理，不破坏现有代码
- PM 最关心：不影响当前迭代进度
- 技术负责人最关心：维护成本和升级策略

### 1.3 路线图

```
M1-M2: 审计 + 规划 + 团队组建
M3-M4: Design Token + 基础组件（Tier 1）
M5-M6: 复合组件 + Figma 库 + 代码库发布
M7-M8: 试点产品线接入 + 文档完善
M9:    全团队推广 + 度量体系建立
```

### 1.4 团队组建

成立 3 人核心小组（不是全职，各投入 50% 时间）：

- 1 名设计师：负责 Figma 组件库和设计规范
- 1 名前端工程师：负责 React 组件库开发
- 1 名前端工程师：负责构建工具、文档站和 Token 管道

---

## 第二阶段：基础搭建（第 3-4 月）

### 2.1 Design Token 体系

首先建立 Token 而非直接画组件：

```
Token 层级：
├── Global Tokens（全局原始值）
│   ├── color.blue.500: #2196F3
│   ├── spacing.16: 16px
│   └── font.size.16: 16px
├── Alias Tokens（语义别名）
│   ├── color.primary: {color.blue.500}
│   ├── spacing.component-gap: {spacing.16}
│   └── font.size.body: {font.size.16}
└── Component Tokens（组件级）
    ├── button.primary.bg: {color.primary}
    ├── button.padding-x: {spacing.16}
    └── button.font-size: {font.size.body}
```

工具链选择：

- **设计端**：Figma Tokens Studio（同步 Token 到 JSON）
- **转换层**：Style Dictionary（JSON -> CSS Variables / TS Constants）
- **代码端**：CSS Custom Properties + Tailwind Config
- **版本管理**：Token JSON 文件纳入 Git，变更走 PR

### 2.2 基础组件（Tier 1）

首批 12 个原子组件：

| 组件 | 变体 | 状态 | 无障碍 |
|------|------|------|--------|
| Button | Primary/Secondary/Tertiary/Ghost/Danger | Default/Hover/Active/Focus/Disabled/Loading | aria-label, role |
| Input | Text/Password/Search/Number | Default/Focus/Error/Disabled | aria-invalid, aria-describedby |
| Checkbox | Default/Indeterminate | Unchecked/Checked/Disabled | aria-checked |
| Radio | Default | Unselected/Selected/Disabled | role="radiogroup" |
| Switch | Default | Off/On/Disabled | role="switch" |
| Select | Single/Multi | Open/Closed/Disabled | aria-expanded |
| Badge | Dot/Count/Status | -- | aria-label |
| Avatar | Image/Initial/Icon | Default/Fallback | alt text |
| Icon | 200+ 图标 | -- | aria-hidden |
| Tooltip | Top/Bottom/Left/Right | -- | role="tooltip" |
| Tag | Default/Closable | -- | aria-label |
| Divider | Horizontal/Vertical | -- | role="separator" |

### 2.3 技术架构

```
@company/design-tokens     → Token JSON + CSS Variables
@company/ui-react          → React 组件库
@company/ui-icons          → SVG 图标库
@company/ui-docs           → Storybook 文档站
```

关键技术决策：

- **构建工具**：Rollup（Tree-shaking 友好）
- **样式方案**：CSS Modules + CSS Variables（非 CSS-in-JS，减少运行时开销）
- **测试**：Vitest + Testing Library + Chromatic（视觉回归）
- **发布**：Changesets（自动版本管理 + Changelog）

---

## 第三阶段：扩展与文档（第 5-6 月）

### 3.1 复合组件（Tier 2-3）

在基础组件上构建：

- **Tier 2**：FormField、Card、ListItem、MenuItem、Breadcrumb、Pagination、Tab
- **Tier 3**：Modal、Drawer、DataTable、Form、Dropdown、DatePicker、Upload

### 3.2 Figma 组件库

与代码组件 1:1 对应：

- 每个 Figma 组件使用 Variants 覆盖所有变体和状态
- 组件属性面板（Component Properties）暴露关键配置
- Auto Layout 约束间距，不允许手动拖拽
- 发布为 Figma Team Library，所有设计师可引用

### 3.3 文档站

使用 Storybook 搭建文档站，每个组件包含：

1. **概述**：组件用途、使用场景
2. **Playground**：可交互的参数调节面板
3. **变体展示**：所有变体的并排对比
4. **Props API**：完整的参数文档
5. **设计指南**：Do / Don't 示例
6. **无障碍说明**：ARIA 属性和键盘操作
7. **Figma 链接**：跳转到对应的 Figma 组件

---

## 第四阶段：试点接入（第 7-8 月）

### 4.1 试点选择

选择 2 个产品线作为试点：

- **产品线 A**：最大的产品线，影响力最大
- **产品线 C**：刚启动新版本，历史包袱最少

### 4.2 接入策略

采用渐进式替换而非一次性迁移：

```
阶段 1: 新页面强制使用设计系统组件
阶段 2: 存量页面在功能迭代时逐步替换
阶段 3: 专项清理遗留自建组件
```

### 4.3 接入过程中的问题

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| 组件 API 不满足需求 | 首批设计未覆盖所有场景 | 收集需求 -> 评审 -> 迭代发布 |
| 升级破坏现有样式 | CSS 变量名冲突 | 添加命名空间前缀 |
| 设计师不愿切换 | 习惯了自己的文件 | 手把手培训 + 模板页面 |
| 开发觉得限制太多 | 组件定制能力不足 | 增加 `className` 和 `style` 透传 |
| 产品线特殊需求 | 通用组件不能满足 | 区分通用组件和业务组件边界 |

### 4.4 接入度量

试点 2 个月后的数据：

```
产品线 A：
- 组件复用率：18% -> 65%
- 新页面开发速度：提升 30%
- 设计走查问题数：减少 55%

产品线 C：
- 组件复用率：0% -> 78%（新项目）
- 设计到开发交付时间：缩短 45%
- UI 一致性评分：92 分
```

---

## 第五阶段：全团队推广（第 9 月）

### 5.1 推广策略

不是发邮件通知，而是多管齐下：

1. **内部发布会**：Demo Day 展示设计系统能力和试点成果
2. **培训工作坊**：设计师 2 小时 + 前端 2 小时，分 4 批完成
3. **迁移指南**：编写从自建组件到设计系统的迁移文档
4. **Codemod 工具**：自动化替换部分常见组件（Button、Input）
5. **度量看板**：实时显示各产品线的采用率
6. **冠军制度**：每个产品线指定 1 名设计系统冠军

### 5.2 治理机制

- **贡献流程**：任何人可提交新组件 RFC，核心小组评审
- **发布节奏**：每两周一次 Minor Release，紧急修复随时 Patch
- **破坏性变更**：需提前一个版本标记 Deprecated，下一版本移除
- **设计审计**：每月对各产品线抽查一致性评分

### 5.3 最终成果

| 指标 | 项目前 | 项目后 | 提升 |
|------|--------|--------|------|
| 组件复用率 | 15% | 82% | +67% |
| 新页面开发周期 | 5 天 | 3 天 | -40% |
| 设计走查问题数 | 每页 15 个 | 每页 3 个 | -80% |
| UI 一致性评分 | 45/100 | 91/100 | +46 |
| 设计师绘制时间 | 每页 8 小时 | 每页 3 小时 | -63% |
| 品牌色修改耗时 | 2 周（8 个库） | 30 分钟（1 个 Token） | -99% |

---

## 踩过的坑

### 坑 1：过早追求完美

初期花了太多时间打磨视觉细节，应该先发布最小可用版本让团队用起来。

**教训**：先有用，再好看。V1 覆盖 80% 场景即可发布。

### 坑 2：忽略开发者体验

Figma 组件做得很好，但 React 组件 API 设计不友好（Props 命名不直觉、类型定义不完整）。

**教训**：API 设计与视觉设计同等重要，前端工程师必须深度参与。

### 坑 3：没有度量就没有话语权

前 4 个月没有建立度量指标，向管理层汇报只能说"感觉变好了"。

**教训**：从第一天就建立可量化指标：复用率、一致性评分、开发速度。

### 坑 4：低估了推广难度

以为"做好了自然会有人用"，实际上需要持续推广和培训。

**教训**：推广投入不低于开发投入。冠军制度和度量看板是关键。

### 坑 5：通用组件 vs 业务组件边界不清

试图把所有组件都放进设计系统，包括高度业务化的组件。

**教训**：设计系统只包含通用组件（2+ 个产品线使用），业务组件由各产品线自行维护。

---

## 关键成功因素

1. **管理层支持**：CTO 授权核心小组 50% 时间投入
2. **渐进式策略**：不要求一步到位，允许渐进替换
3. **双端同步**：Figma 组件与代码组件 1:1 对应
4. **度量驱动**：用数据说话，每月发布采用率报告
5. **社区运营**：内部 Slack 频道、月度分享会、贡献者表彰

---

## Agent Checklist

以下为 AI Agent 在辅助设计系统搭建时的检查要点：

- [ ] 先进行组件审计，量化现状（颜色/字号/间距/组件变体数量）
- [ ] 建立 Design Token 体系后再开始画组件
- [ ] Token 使用三层结构：Global -> Alias -> Component
- [ ] 基础组件覆盖完整状态（Default/Hover/Active/Focus/Disabled/Loading/Error）
- [ ] 每个组件包含无障碍支持（ARIA 属性、键盘导航）
- [ ] Figma 组件与代码组件 1:1 对应
- [ ] 文档包含 Playground、Props API、Do/Don't 示例
- [ ] 采用渐进式接入策略（新页面强制 -> 存量逐步替换）
- [ ] 建立度量指标：复用率、一致性评分、开发速度
- [ ] 区分通用组件和业务组件的边界
