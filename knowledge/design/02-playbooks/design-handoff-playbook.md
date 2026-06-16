---
id: design-handoff-playbook
title: Design Handoff Playbook
domain: design
category: 02-playbooks
difficulty: intermediate
tags: [agent, checklist, design, handoff, playbook, token, 导出, 常见问题与解决方案]
quality_score: 70
last_updated: 2026-06-15
---
# Design Handoff Playbook

## 概述

设计交付（Design Handoff）是设计稿从设计师到开发工程师的完整传递流程。一个高质量的交付流程能显著减少返工、降低沟通成本，并确保最终产品与设计意图一致。本 Playbook 覆盖从 Figma 标注到最终走查验收的完整链路，适用于 Web、移动端和跨平台项目。

---

## 阶段一：设计稿准备与标注

### 1.1 图层与命名规范

- 所有图层使用语义化命名：`header/nav/logo`、`card/product/title`
- 禁止出现 `Frame 123`、`Group 45` 等默认命名
- 组件实例与 Master Component 保持同步，不允许 detach 后修改
- 隐藏图层必须清理，交付稿中不包含废弃设计

### 1.2 Figma 标注要求

| 标注项 | 要求 | 工具 |
|--------|------|------|
| 间距与尺寸 | Auto Layout 约束，非手动标注 | Figma Auto Layout |
| 颜色 | 引用 Design Token，非硬编码 HEX | Figma Variables |
| 字体 | 引用 Text Style，标注 font-family/size/weight/line-height | Figma Text Styles |
| 圆角 | 统一使用变量：`radius-sm/md/lg/xl` | Figma Variables |
| 阴影 | 引用 Effect Style，标注 x/y/blur/spread/color | Figma Effect Styles |
| 图标 | SVG 导出，统一命名：`icon-{category}-{name}` | Figma Export |

### 1.3 状态覆盖

每个交互组件必须包含以下状态的设计稿：

- Default / Hover / Active / Focus / Disabled
- Loading / Empty / Error / Success
- 移动端额外状态：Pressed / Swiped

### 1.4 响应式断点

```
Mobile:    320px - 767px
Tablet:    768px - 1023px
Desktop:   1024px - 1439px
Wide:      1440px+
```

每个关键页面至少提供 Mobile + Desktop 两套设计稿。

---

## 阶段二：Design Token 导出

### 2.1 Token 结构

```json
{
  "color": {
    "primary": { "50": "#E3F2FD", "500": "#2196F3", "900": "#0D47A1" },
    "neutral": { "0": "#FFFFFF", "100": "#F5F5F5", "900": "#212121" },
    "semantic": { "success": "#4CAF50", "warning": "#FF9800", "error": "#F44336" }
  },
  "spacing": {
    "xs": "4px", "sm": "8px", "md": "16px", "lg": "24px", "xl": "32px", "2xl": "48px"
  },
  "typography": {
    "heading-1": { "fontSize": "32px", "fontWeight": 700, "lineHeight": 1.25 },
    "body-md": { "fontSize": "16px", "fontWeight": 400, "lineHeight": 1.5 }
  },
  "radius": { "sm": "4px", "md": "8px", "lg": "12px", "xl": "16px", "full": "9999px" },
  "shadow": {
    "sm": "0 1px 2px rgba(0,0,0,0.05)",
    "md": "0 4px 6px rgba(0,0,0,0.07)",
    "lg": "0 10px 15px rgba(0,0,0,0.1)"
  }
}
```

### 2.2 导出工具链

- **Figma Tokens Studio** -> JSON -> Style Dictionary -> CSS Variables / Tailwind Config
- 确保 Token 命名在设计稿与代码中保持一致
- Token 变更需走 PR 流程，设计师与开发共同 Review

---

## 阶段三：组件映射表

### 3.1 映射文档格式

| Figma 组件 | 前端组件 | Props | 备注 |
|------------|---------|-------|------|
| Button/Primary | `<Button variant="primary">` | size, disabled, loading | 包含 icon 插槽 |
| Card/Product | `<ProductCard>` | title, image, price, rating | 响应式宽度 |
| Input/Text | `<TextField>` | label, error, helper, disabled | 支持前后缀 |
| Modal/Confirm | `<ConfirmDialog>` | title, message, onConfirm, onCancel | 带遮罩层 |
| Toast/Success | `<Toast variant="success">` | message, duration, action | 自动消失 |

### 3.2 组件分级

- **Tier 1 - 原子组件**：Button、Input、Icon、Badge、Avatar
- **Tier 2 - 分子组件**：Card、ListItem、FormField、MenuItem
- **Tier 3 - 有机体组件**：Header、Sidebar、DataTable、Form
- **Tier 4 - 页面模板**：DashboardLayout、AuthLayout、SettingsLayout

优先交付 Tier 1-2，确保基础组件 100% 覆盖后再构建上层。

---

## 阶段四：交付会议

### 4.1 会议准备

- 设计师提前 1 天发送交付页面链接和标注说明
- 开发提前浏览设计稿，准备技术可行性问题
- PM 准备需求优先级和时间排期

### 4.2 会议议程（60 分钟）

1. **全局走查**（15 min）：页面流程、导航逻辑、关键路径
2. **组件细节**（20 min）：交互状态、边界情况、动效说明
3. **技术讨论**（15 min）：实现难点、性能考量、兼容性
4. **确认与排期**（10 min）：确认交付范围、分期计划、验收标准

### 4.3 会议产出

- 交付确认清单（Checklist）签字
- 技术难点 Spike 任务分配
- 各组件开发排期到人到天

---

## 阶段五：开发实现

### 5.1 开发规范

- 组件 Props 命名与 Figma 属性名保持一致
- 使用 Design Token 变量，禁止硬编码颜色/间距/字号
- 响应式实现使用设计稿定义的断点
- 动效参数（duration、easing、delay）从设计标注中获取

### 5.2 实现顺序

```
1. Token 系统搭建（颜色/字体/间距/圆角/阴影）
2. 原子组件库（Button/Input/Icon 等）
3. 分子组件（Card/ListItem 等）
4. 页面布局（Grid/Layout/Navigation）
5. 页面组装与数据对接
6. 动效与微交互
7. 响应式适配与兼容性
```

### 5.3 实现过程中的沟通

- 遇到设计稿不清晰的地方，在 Figma 中添加 Comment 并 @ 设计师
- 技术限制导致无法完全还原时，提出替代方案并记录差异
- 每完成一个组件/页面立即推送预览链接

---

## 阶段六：设计走查（Design QA）

### 6.1 走查准备

- 部署到 Staging 环境或提供预览链接
- 准备设计稿与实现的并排对比环境
- 记录已知差异和技术限制说明

### 6.2 走查维度

| 维度 | 检查内容 | 优先级 |
|------|---------|--------|
| 布局 | 间距、对齐、网格一致性 | P0 |
| 颜色 | Token 使用正确性、对比度 | P0 |
| 字体 | 字号/字重/行高/字间距 | P0 |
| 交互 | 各状态（hover/focus/disabled）还原度 | P0 |
| 响应式 | 各断点布局、元素隐显 | P1 |
| 动效 | 过渡效果、缓动函数、时长 | P1 |
| 内容 | 长文本截断、空状态、错误提示 | P1 |
| 无障碍 | 键盘导航、屏幕阅读器、对比度 | P1 |

### 6.3 问题记录格式

```
[页面] 首页 > Hero 区域
[问题] 标题字号偏大，设计稿为 32px 实际为 36px
[优先级] P0
[截图] {设计稿截图} vs {实现截图}
[修复建议] 修改 heading-1 token 值
```

### 6.4 走查轮次

- **第一轮**：整体布局与视觉还原度（修复 P0）
- **第二轮**：交互状态与响应式（修复 P1）
- **第三轮**：细节打磨与边界情况（收尾）
- 每轮走查后 48 小时内修复，修复后触发下一轮

---

## 阶段七：验收与归档

### 7.1 验收标准

- P0 问题全部修复
- P1 问题修复率 >= 90%
- 主流程 100% 还原
- 响应式三端（Mobile/Tablet/Desktop）达标
- 无障碍基础检查通过（WCAG 2.1 AA）

### 7.2 归档产出

- 组件映射表最终版（含实际 Props 和差异说明）
- Design Token 版本号与代码同步记录
- 走查问题汇总与修复状态
- 设计师签字确认验收通过

---

## 常见问题与解决方案

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| 颜色不一致 | 未使用 Token | 强制使用 CSS Variables |
| 间距偏差 | 手动标注 vs Auto Layout | 统一使用 Auto Layout |
| 字体渲染差异 | 系统字体差异 | 指定 font-display 策略 |
| 动效不流畅 | GPU 合成层问题 | 使用 transform/opacity |
| 响应式断裂 | 断点未对齐 | 使用设计稿定义的断点 |

---

## Agent Checklist

以下为 AI Agent 在执行设计交付流程时的检查要点：

- [ ] 确认设计稿图层命名规范，无默认命名
- [ ] 确认所有交互组件包含完整状态设计
- [ ] 确认 Design Token 已导出并与代码同步
- [ ] 确认组件映射表已建立并经双方确认
- [ ] 确认响应式断点与设计稿一致
- [ ] 确认开发实现使用 Token 变量而非硬编码
- [ ] 确认每个组件/页面完成后立即进行设计走查
- [ ] 确认 P0 问题全部修复后方可进入下一阶段
- [ ] 确认最终验收由设计师签字确认
- [ ] 确认所有走查记录和差异说明已归档
