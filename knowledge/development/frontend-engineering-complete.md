---
id: frontend-engineering-complete
title: frontend-engineering-complete
domain: development
category: frontend-engineering-complete.md
difficulty: intermediate
tags: [complete, development, engineering, frontend, 前端工程完整知识库]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 前端工程完整知识库

### 1. 分层与模块边界
- 页面层负责编排，组件层负责复用，服务层负责数据访问。
- 禁止在视图层写复杂业务规则。
- 组件必须区分展示组件与业务组件。

### 2. 设计系统与一致性
- 建立颜色、间距、字体、圆角、阴影设计令牌。
- 同语义操作使用同交互模式和文案。
- 状态矩阵必须覆盖加载、空态、错误、权限、成功反馈。

### 3. 状态管理策略
- 本地状态优先，跨页面共享状态再上全局状态。
- 远程数据与UI状态隔离管理。
- 状态变更必须可追踪并可重放。

### 4. 前端性能治理
- 建立首屏、交互、资源体积性能预算。
- 采用路由级代码分割与懒加载。
- 对高频渲染组件做记忆化与虚拟列表优化。

### 5. 可观测性与错误治理
- 错误上报必须包含页面、用户、请求ID、版本号。
- 关键交互路径埋点必须有成功率指标。
- JS异常、接口异常、资源加载异常分级处理。

### 6. 可访问性与国际化
- 支持键盘可操作与语义标签。
- 文案国际化键值统一，禁止硬编码。
- 日期、货币、时区按地区规则展示。
