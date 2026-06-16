---
id: component-quality-checklist
title: 组件质量检查清单
domain: frontend
category: 03-checklists
difficulty: intermediate
tags: [a11y, checklist, component, frontend, props, quality, 响应式设计, 性能优化]
quality_score: 70
last_updated: 2026-06-15
---
# 组件质量检查清单

## 概述

本清单用于在组件开发完成后、合入主干前进行系统化质量审查。涵盖 Props 验证、样式隔离、无障碍、响应式、性能和测试覆盖六个维度，确保每个组件达到商业级交付标准。

适用场景：React / Vue / Svelte 等主流框架的 UI 组件库开发及业务组件交付。

---

## 1. Props 验证

- [ ] 所有 Props 均已声明类型（TypeScript interface 或 PropTypes）
- [ ] 必填 Props 与可选 Props 区分明确，可选 Props 设置合理默认值
- [ ] 枚举类型 Props 使用联合类型（union type）而非 `string`
- [ ] 回调函数 Props 命名以 `on` 开头（如 `onClick`、`onChange`）
- [ ] 子组件 Props 不透传 `any` 类型
- [ ] 复杂对象 Props 有完整的嵌套类型定义
- [ ] Props 变更有向后兼容策略（deprecated 标注 + 迁移文档）
- [ ] 使用 `children` 或 `render props` 时类型约束明确
- [ ] Props 数量不超过 10 个；超过时考虑拆分组件或使用组合模式
- [ ] 布尔型 Props 默认值为 `false`，命名为肯定形式（如 `disabled` 而非 `notEnabled`）

## 2. 样式隔离

- [ ] 组件样式不泄漏到外部（使用 CSS Modules / Scoped CSS / CSS-in-JS）
- [ ] 不使用全局选择器（如 `div`、`p`、`h1`）
- [ ] 不使用 `!important`（特殊情况需注释说明原因）
- [ ] 类名命名有组件前缀或使用自动哈希（避免命名冲突）
- [ ] 主题变量通过 CSS 自定义属性（CSS Variables）或 Design Token 注入
- [ ] 样式不依赖 DOM 层级结构（避免 `.parent > .child > .target` 深层嵌套）
- [ ] 暗色模式 / 多主题切换已验证
- [ ] z-index 使用项目统一的层级管理常量，不随意指定数值
- [ ] 过渡动画使用 `transform` / `opacity`，避免触发布局重排
- [ ] 组件支持通过 `className` 或 `style` Props 进行外部样式覆盖

## 3. 无障碍（Accessibility / a11y）

- [ ] 交互元素有明确的语义化标签（`<button>`、`<a>`、`<input>` 等）
- [ ] 所有图片有 `alt` 属性；装饰性图片使用 `alt=""`
- [ ] 表单控件关联 `<label>`（通过 `htmlFor` 或嵌套）
- [ ] 自定义控件设置正确的 ARIA role（`role="dialog"`、`role="tablist"` 等）
- [ ] 焦点管理正确：模态框打开时焦点移入，关闭时焦点回到触发元素
- [ ] 键盘可完整操作：Tab 导航、Enter/Space 激活、Escape 关闭
- [ ] 颜色对比度满足 WCAG 2.1 AA 标准（正文 ≥ 4.5:1，大文本 ≥ 3:1）
- [ ] 动态内容变更使用 `aria-live` 通知屏幕阅读器
- [ ] 禁用状态使用 `aria-disabled` 而非仅视觉变灰
- [ ] 通过 axe-core 或 Lighthouse 无障碍扫描无严重错误

## 4. 响应式设计

- [ ] 断点使用项目统一定义（如 sm/md/lg/xl），不硬编码像素值
- [ ] 移动端优先（mobile-first）编写媒体查询
- [ ] 触摸区域最小 44x44px（符合 Apple HIG / Material Design）
- [ ] 文字不使用固定像素大小，使用 rem/em 相对单位
- [ ] 长文本有截断策略（ellipsis / 展开收起 / tooltip）
- [ ] 表格在小屏幕有降级方案（卡片化 / 横向滚动 / 隐藏次要列）
- [ ] 图片使用 `srcset` 或响应式图片方案，避免大图加载浪费带宽
- [ ] 弹窗 / 下拉菜单在小屏幕适配正确（不溢出视口）
- [ ] 横屏模式下布局不破裂
- [ ] 在 320px ~ 2560px 宽度范围内手动验证过渲染效果

## 5. 性能优化

- [ ] 大列表使用虚拟滚动（react-window / vue-virtual-scroller）
- [ ] 避免在 render 中创建新对象 / 新函数（使用 useMemo / useCallback）
- [ ] 图片使用懒加载（`loading="lazy"` 或 Intersection Observer）
- [ ] 组件按需加载（React.lazy / dynamic import）
- [ ] 避免不必要的重渲染（React.memo / shouldComponentUpdate / computed）
- [ ] 事件处理器有防抖 / 节流（搜索输入、滚动监听、窗口 resize）
- [ ] SVG 图标使用 sprite 或内联，不逐个请求
- [ ] 动画帧率保持 60fps，避免主线程阻塞
- [ ] 组件卸载时清理定时器、事件监听器、取消未完成请求
- [ ] Bundle 分析确认组件不引入过大的第三方依赖

## 6. 测试覆盖

- [ ] 单元测试覆盖所有 Props 组合的核心渲染路径
- [ ] 交互行为测试（点击、输入、焦点切换）使用 Testing Library
- [ ] 快照测试（Snapshot）仅用于稳定组件，频繁变更组件不使用
- [ ] 边界条件测试：空数据、超长文本、特殊字符、极大数值
- [ ] 异步行为测试：加载状态、错误状态、超时处理
- [ ] 可访问性测试（jest-axe / @axe-core/react）
- [ ] 视觉回归测试（Chromatic / Percy / Playwright screenshot）
- [ ] 测试覆盖率 ≥ 80%（行覆盖 + 分支覆盖）
- [ ] 测试用例有清晰的 describe/it 描述，不使用 test1/test2 命名
- [ ] CI 中测试通过才允许合并

## 7. 代码规范与文档

- [ ] 组件有 JSDoc / TSDoc 注释说明用途和使用示例
- [ ] Storybook 或同类工具中有完整 Story（包含各状态展示）
- [ ] 导出类型定义供外部消费
- [ ] 组件目录结构统一（index.ts + Component.tsx + Component.test.tsx + Component.module.css）
- [ ] 命名规范：组件 PascalCase，文件名与组件名一致
- [ ] 无 console.log / debugger 残留
- [ ] 无注释掉的代码块
- [ ] ESLint / Stylelint 无警告

---

## 评审流程

| 阶段 | 检查重点 | 工具 |
|------|---------|------|
| 开发自查 | Props 验证 + 样式隔离 + 代码规范 | ESLint, TypeScript |
| 同行评审 | 无障碍 + 响应式 + 性能 | axe-core, Lighthouse |
| QA 验收 | 全维度回归 | Playwright, Chromatic |
| 发布前 | 测试覆盖率 + Bundle 大小 | Jest, webpack-bundle-analyzer |

---

## 组件复杂度评估

在决定组件是否需要拆分时，参考以下指标：

| 指标 | 健康值 | 需要关注 | 必须拆分 |
|------|--------|---------|---------|
| Props 数量 | ≤ 5 | 6-10 | > 10 |
| 组件行数 | ≤ 150 | 150-300 | > 300 |
| 嵌套层级 | ≤ 3 | 4-5 | > 5 |
| useState 数量 | ≤ 3 | 4-5 | > 5 |
| useEffect 数量 | ≤ 2 | 3 | > 3 |
| 条件渲染分支 | ≤ 3 | 4-5 | > 5 |

拆分原则：
- **单一职责**：一个组件只做一件事
- **容器与展示分离**：数据获取逻辑放在容器组件，UI 渲染放在展示组件
- **组合优于继承**：通过 children / render props 组合，而非创建深层继承链

---

## 常见不合格项 Top 5

1. **Props 类型为 `any`** - 丧失类型安全，重构时无法发现调用错误
2. **全局样式污染** - 组件在不同页面表现不一致
3. **缺少键盘操作支持** - 自定义下拉框/弹窗无法用键盘关闭
4. **大列表不使用虚拟滚动** - 数据超过 500 条时页面卡顿
5. **组件卸载不清理副作用** - 导致内存泄漏和控制台报错

---

## Agent Checklist

以下为 AI Agent 在审查组件时必须逐项验证的硬约束：

- [ ] 运行 `tsc --noEmit` 确认无类型错误
- [ ] 运行 `npx eslint --ext .tsx,.ts <component-path>` 确认无 lint 错误
- [ ] 运行 `npx jest --coverage <component-test-path>` 确认覆盖率 ≥ 80%
- [ ] 运行 Lighthouse Accessibility 审计得分 ≥ 90
- [ ] 检查组件在 375px（iPhone SE）和 1920px（桌面）两个宽度下的渲染
- [ ] 确认组件 bundle 大小不超过 50KB（gzip 后）
- [ ] 确认 Storybook Story 存在且可正常渲染
- [ ] 确认组件无 `// TODO`、`// FIXME`、`// HACK` 遗留标记
- [ ] 若组件为新增，确认已在组件索引文件中导出
- [ ] 生成审查报告并附在 PR 评论中
