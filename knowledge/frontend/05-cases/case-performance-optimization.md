---
id: case-performance-optimization
title: 前端性能优化案例：Lighthouse 从 40 到 95 的实战过程
domain: frontend
category: 05-cases
difficulty: intermediate
tags: [agent, case, checklist, frontend, optimization, performance, 业务影响, 初始状态诊断]
quality_score: 70
last_updated: 2026-06-15
---
# 前端性能优化案例：Lighthouse 从 40 到 95 的实战过程

## 概述

本案例记录一个中型 B2C 电商平台前端性能优化的完整过程。项目初始 Lighthouse Performance 得分仅 40 分，经过 6 个迭代周期的系统化优化，最终稳定在 95 分以上。团队规模 5 人，优化周期 8 周。

技术栈：React 18 + Next.js 14 + TypeScript + Tailwind CSS + PostgreSQL API

---

## 初始状态诊断

### Lighthouse 基线报告（优化前）

| 指标 | 初始值 | 目标值 |
|------|--------|--------|
| Performance Score | 40 | ≥ 90 |
| FCP (First Contentful Paint) | 4.2s | < 1.8s |
| LCP (Largest Contentful Paint) | 8.1s | < 2.5s |
| TBT (Total Blocking Time) | 1800ms | < 200ms |
| CLS (Cumulative Layout Shift) | 0.35 | < 0.1 |
| Speed Index | 6.5s | < 3.4s |

### 根因分析

通过 Chrome DevTools Performance 面板和 webpack-bundle-analyzer 定位到以下核心问题：

1. **Bundle 过大**：主包 2.1MB（gzip 后 680KB），包含全量 lodash、moment.js、antd
2. **图片未优化**：首页 Hero 图 3.2MB PNG，商品列表图无懒加载
3. **阻塞渲染的资源**：12 个同步加载的 CSS 文件，6 个 render-blocking JS
4. **无代码分割**：所有页面打进同一个 bundle
5. **布局抖动**：图片无预设尺寸，字体加载引起 FOIT
6. **第三方脚本**：Google Analytics + Hotjar + Intercom 同步加载

---

## 第一轮：Bundle 瘦身（Score 40 → 55）

### 措施

```
1. lodash → lodash-es + babel-plugin-lodash（按需引入）
   - 减少 71KB (gzip)
2. moment.js → dayjs（API 兼容，体积 2KB vs 67KB）
   - 减少 65KB (gzip)
3. antd 全量引入 → 按需引入 + tree-shaking
   - 减少 120KB (gzip)
4. 移除未使用的依赖（5个）
   - 减少 45KB (gzip)
```

### 效果

- Bundle 大小：680KB → 379KB（-44%）
- TBT：1800ms → 1200ms
- Score：40 → 55

### 关键教训

使用 `npx depcheck` 发现 5 个 package.json 中声明但从未 import 的依赖。`webpack-bundle-analyzer` 可视化后，团队对 "哪些库占了多少体积" 有了直观认知。

---

## 第二轮：代码分割与懒加载（Score 55 → 68）

### 措施

```
1. 路由级代码分割
   - 使用 Next.js dynamic import
   - 每个页面独立 chunk

2. 组件级懒加载
   - 富文本编辑器（420KB）仅在用户点击时加载
   - 图表组件（recharts 180KB）仅在可视区域加载
   - 模态框内容延迟加载

3. 第三方脚本异步化
   - Google Analytics → async + defer
   - Hotjar → 页面 idle 后加载（requestIdleCallback）
   - Intercom → 用户滚动到页面底部时加载
```

### 效果

- 首页 JS 大小：379KB → 142KB
- FCP：3.8s → 2.4s
- Score：55 → 68

---

## 第三轮：图片优化（Score 68 → 78）

### 措施

```
1. 格式转换
   - PNG/JPG → WebP（回退 JPG）
   - 使用 next/image 自动格式协商

2. 尺寸适配
   - srcset 提供 640/1024/1920 三档
   - 移动端不加载桌面尺寸图片

3. 懒加载
   - 首屏以下图片全部 loading="lazy"
   - 首屏图片使用 priority 属性预加载

4. 占位策略
   - LQIP（低质量图片占位）使用 plaiceholder 生成
   - 图片容器预设宽高比，消除 CLS

5. CDN 配置
   - 图片通过 Cloudflare Images 分发
   - 启用自动压缩和格式转换
```

### 效果

- 首页图片总大小：4.8MB → 320KB
- LCP：6.2s → 3.1s
- CLS：0.35 → 0.08
- Score：68 → 78

---

## 第四轮：渲染优化（Score 78 → 85）

### 措施

```
1. 关键 CSS 内联
   - 使用 critters 提取首屏关键 CSS 内联到 <head>
   - 非关键 CSS 异步加载

2. 字体优化
   - font-display: swap → font-display: optional
   - 字体文件 subset（仅保留中文常用 6763 字 + ASCII）
   - 使用 woff2 格式，预加载主字体

3. React 渲染优化
   - 商品列表使用 React.memo + useMemo
   - 搜索输入 debounce 300ms
   - 虚拟滚动处理 2000+ 商品列表（react-window）

4. SSR / SSG 策略
   - 首页 + 商品分类页 → SSG（构建时生成）
   - 商品详情页 → ISR（增量静态生成，60s 刷新）
   - 购物车 / 结算页 → CSR（纯客户端）
```

### 效果

- FCP：2.4s → 1.4s
- TBT：800ms → 280ms
- Score：78 → 85

---

## 第五轮：网络与缓存优化（Score 85 → 92）

### 措施

```
1. HTTP/2 Push
   - 关键资源服务端推送（主 CSS + 主 JS）

2. 缓存策略
   - 静态资源：Cache-Control: public, max-age=31536000, immutable
   - HTML：Cache-Control: no-cache（配合 ETag）
   - API 响应：stale-while-revalidate

3. Service Worker
   - 静态资源缓存优先策略
   - API 请求网络优先 + 离线回退
   - 预缓存核心路由的 JS chunk

4. DNS 预解析 + 预连接
   - <link rel="dns-prefetch"> 第三方域名
   - <link rel="preconnect"> CDN 和 API 域名

5. 接口优化
   - 首页数据从 3 个串行请求合并为 1 个聚合接口
   - GraphQL 替代部分 REST 接口（减少 over-fetching）
```

### 效果

- TTFB：420ms → 180ms
- Speed Index：3.8s → 2.1s
- Score：85 → 92

---

## 第六轮：精细化调优（Score 92 → 95+）

### 措施

```
1. Core Web Vitals 微调
   - INP（Interaction to Next Paint）优化：长任务拆分为微任务
   - 使用 scheduler.yield() 让出主线程

2. 预加载策略
   - 鼠标 hover 链接时预加载目标页面
   - prefetch 下一页数据

3. 监控体系建立
   - 接入 Web Vitals 库实时上报
   - Grafana 仪表板监控 P75 指标
   - 性能预算 CI 检查（超标则阻止合并）

4. 持续防劣化
   - Lighthouse CI 集成到 GitHub Actions
   - 每次 PR 自动跑 Lighthouse，低于 90 分阻止合并
   - 每周性能报告自动发送
```

### 最终效果

| 指标 | 初始值 | 最终值 | 改善幅度 |
|------|--------|--------|---------|
| Performance Score | 40 | 95+ | +137% |
| FCP | 4.2s | 1.1s | -74% |
| LCP | 8.1s | 1.8s | -78% |
| TBT | 1800ms | 120ms | -93% |
| CLS | 0.35 | 0.02 | -94% |
| Speed Index | 6.5s | 1.6s | -75% |

---

## 业务影响

- 跳出率从 58% 降至 32%（-45%）
- 平均页面停留时间从 1.2 分钟增至 3.4 分钟
- 移动端转化率提升 23%
- SEO 排名首页关键词增加 40%

---

## 经验总结

1. **先测量后优化** - 不要凭直觉，用数据定位瓶颈
2. **投入产出比递减** - 前三轮（40→78）用了 3 周，后三轮（78→95）也用了 5 周
3. **防劣化比优化更重要** - 没有 CI 卡点，分数会在 2 个月内回退
4. **移动端是真正的战场** - 桌面端 90 分不代表移动端也是 90 分
5. **团队意识比工具重要** - 每个开发者都应关注自己代码的性能影响

---

## Agent Checklist

以下为 AI Agent 在执行性能优化任务时必须遵循的检查项：

- [ ] 优化前运行 `npx lighthouse <url> --output=json` 记录基线数据
- [ ] 使用 `npx webpack-bundle-analyzer` 分析 bundle 组成
- [ ] 检查所有图片是否使用 WebP/AVIF 格式及懒加载
- [ ] 确认路由级代码分割已启用
- [ ] 确认第三方脚本不阻塞首屏渲染
- [ ] 确认 CSS 关键路径已内联
- [ ] 确认字体使用 `font-display: swap` 或 `optional`
- [ ] 确认 Cache-Control 头正确设置
- [ ] 优化后运行 Lighthouse 确认得分提升且无指标劣化
- [ ] 将 Lighthouse CI 配置写入 CI pipeline 防止回退
