---
id: frontend-launch-checklist
title: 前端上线检查清单 (Frontend Launch Checklist)
domain: frontend
category: 03-checklists
difficulty: intermediate
tags: [accessibility, checklist, engine, frontend, launch, optimization, performance, search]
quality_score: 70
last_updated: 2026-06-15
---
# 前端上线检查清单 (Frontend Launch Checklist)

## 概述

本检查清单覆盖前端应用从开发完成到生产上线的全部验证项，确保性能、SEO、安全、无障碍、兼容性和监控六大维度全部就绪。每次上线前必须逐项确认，未通过的关键项（标记 [CRITICAL]）必须修复后才能继续。

## 使用说明

- **[CRITICAL]** 标记项为强制必过门禁，任一未通过则阻断上线
- **[HIGH]** 标记项为高度建议项，需要负责人签字确认风险后方可跳过
- **[MEDIUM]** 标记项为一般建议项，可带风险上线但需在 7 天内补齐
- 每项需由**执行人签字**和**审核人确认**，记录**确认时间**

---

## 一、性能 (Performance)

### 1.1 Lighthouse 评分

- [ ] **[CRITICAL]** Lighthouse Performance 得分 ≥ 90（移动端）
- [ ] **[CRITICAL]** Lighthouse Performance 得分 ≥ 90（桌面端）
- [ ] **[HIGH]** Lighthouse Accessibility 得分 ≥ 90
- [ ] **[HIGH]** Lighthouse Best Practices 得分 ≥ 90
- [ ] **[HIGH]** Lighthouse SEO 得分 ≥ 90
- [ ] **[MEDIUM]** 已使用 Lighthouse CI 在 Pipeline 中自动化检测

### 1.2 Core Web Vitals

- [ ] **[CRITICAL]** LCP (Largest Contentful Paint) < 2.5s
- [ ] **[CRITICAL]** INP (Interaction to Next Paint) < 200ms
- [ ] **[CRITICAL]** CLS (Cumulative Layout Shift) < 0.1
- [ ] **[HIGH]** FCP (First Contentful Paint) < 1.8s
- [ ] **[HIGH]** TTFB (Time to First Byte) < 800ms
- [ ] **[MEDIUM]** 已在 RUM (Real User Monitoring) 中持续监控 Core Web Vitals

### 1.3 Bundle Size

- [ ] **[CRITICAL]** 主包 (main bundle) gzip 后 < 200KB
- [ ] **[HIGH]** 首屏 JS 总量 gzip 后 < 300KB
- [ ] **[HIGH]** 已配置代码分割 (Code Splitting)，路由级别懒加载
- [ ] **[HIGH]** 已使用 Tree Shaking 移除未使用的代码
- [ ] **[MEDIUM]** 已使用 Bundle Analyzer 检查大型依赖（如 lodash 全量导入、moment.js）
- [ ] **[MEDIUM]** 第三方脚本（Analytics、广告、客服）使用 `async` 或 `defer` 加载
- [ ] **[MEDIUM]** 已评估并移除不必要的 polyfill

### 1.4 图片优化

- [ ] **[CRITICAL]** 所有图片使用现代格式（WebP / AVIF），提供 fallback
- [ ] **[HIGH]** 图片根据显示尺寸提供多种分辨率（`srcset` + `sizes`）
- [ ] **[HIGH]** 首屏以下图片使用懒加载 (`loading="lazy"`)
- [ ] **[HIGH]** 首屏图片设置了 `fetchpriority="high"`
- [ ] **[HIGH]** 所有图片设置了明确的 `width` 和 `height` 属性（防止 CLS）
- [ ] **[MEDIUM]** SVG 图标使用 SVG Sprite 或内联方式，而非单独 HTTP 请求
- [ ] **[MEDIUM]** 大图片已通过 CDN 进行动态裁剪和压缩

### 1.5 缓存策略

- [ ] **[CRITICAL]** 静态资源使用内容哈希文件名（`app.a1b2c3.js`）
- [ ] **[CRITICAL]** 静态资源设置长期缓存头 (`Cache-Control: max-age=31536000, immutable`)
- [ ] **[HIGH]** HTML 文件设置短缓存或不缓存 (`Cache-Control: no-cache`)
- [ ] **[HIGH]** Service Worker 缓存策略已正确配置（如有使用 PWA）
- [ ] **[HIGH]** CDN 缓存已配置并验证命中率
- [ ] **[MEDIUM]** API 响应根据业务场景设置合理的缓存策略
- [ ] **[MEDIUM]** 已配置 CDN 缓存清除（purge）机制，确保紧急更新可快速生效

### 1.6 字体优化

- [ ] **[HIGH]** 自定义字体使用 `font-display: swap` 或 `optional` 防止 FOIT
- [ ] **[HIGH]** 字体文件使用 WOFF2 格式
- [ ] **[HIGH]** 字体文件已子集化（subsetted），仅包含所需字符集
- [ ] **[MEDIUM]** 关键字体使用 `<link rel="preload">` 预加载

---

## 二、SEO (Search Engine Optimization)

### 2.1 Meta 标签

- [ ] **[CRITICAL]** 每个页面都有唯一的 `<title>`（60 字符以内）
- [ ] **[CRITICAL]** 每个页面都有唯一的 `<meta name="description">`（160 字符以内）
- [ ] **[HIGH]** 页面语言已设置 (`<html lang="zh-CN">`)
- [ ] **[HIGH]** 视口已正确配置 (`<meta name="viewport" content="width=device-width, initial-scale=1">`)
- [ ] **[MEDIUM]** Canonical URL 已设置（`<link rel="canonical">`）
- [ ] **[MEDIUM]** 多语言页面已配置 `hreflang` 标签

### 2.2 Open Graph & Social

- [ ] **[HIGH]** Open Graph 标签已配置（`og:title`、`og:description`、`og:image`、`og:url`）
- [ ] **[HIGH]** `og:image` 尺寸至少 1200×630 px
- [ ] **[HIGH]** Twitter Card 标签已配置（`twitter:card`、`twitter:title`、`twitter:image`）
- [ ] **[MEDIUM]** 已使用社交媒体分享调试工具验证预览效果
- [ ] **[MEDIUM]** 配置了 `og:type`（article / website / product）

### 2.3 Sitemap & Robots

- [ ] **[CRITICAL]** `robots.txt` 已配置且允许搜索引擎爬取
- [ ] **[CRITICAL]** 生产环境确认没有 `noindex` / `nofollow` 遗留标签
- [ ] **[HIGH]** `sitemap.xml` 已生成并提交至 Google Search Console
- [ ] **[HIGH]** Sitemap 包含所有公开页面的 URL 和 `lastmod` 日期
- [ ] **[MEDIUM]** 404 页面已自定义，提供有用的导航链接
- [ ] **[MEDIUM]** 已配置结构化数据（JSON-LD）提升搜索结果展示效果

### 2.4 URL 与路由

- [ ] **[HIGH]** URL 使用语义化路径（`/products/shoes` 而非 `/p?id=123`）
- [ ] **[HIGH]** SPA 已配置服务端渲染（SSR）或预渲染（Prerendering）
- [ ] **[HIGH]** 旧 URL 已配置 301 重定向到新 URL
- [ ] **[MEDIUM]** URL 使用小写字母和连字符（`-`）而非下划线（`_`）

---

## 三、安全 (Security)

### 3.1 CSP (Content Security Policy)

- [ ] **[CRITICAL]** 已配置 Content-Security-Policy 响应头
- [ ] **[CRITICAL]** CSP 禁止 `unsafe-inline` 和 `unsafe-eval`（或使用 nonce/hash 替代）
- [ ] **[HIGH]** CSP 限定了 `script-src`、`style-src`、`img-src`、`connect-src` 的域名白名单
- [ ] **[HIGH]** CSP 配置了 `report-uri` / `report-to` 用于收集违规报告
- [ ] **[MEDIUM]** 已在 Report-Only 模式下测试 CSP 规则，确认无误后启用强制模式

### 3.2 XSS 防护

- [ ] **[CRITICAL]** 所有用户输入在渲染时已正确转义
- [ ] **[CRITICAL]** 禁止使用 `dangerouslySetInnerHTML`（React）/ `v-html`（Vue）渲染用户内容
- [ ] **[HIGH]** 已配置 `X-Content-Type-Options: nosniff`
- [ ] **[HIGH]** 已配置 `X-Frame-Options: DENY` 或 CSP `frame-ancestors 'none'`
- [ ] **[HIGH]** Cookie 设置了 `HttpOnly`、`Secure`、`SameSite=Strict/Lax`

### 3.3 CORS (Cross-Origin Resource Sharing)

- [ ] **[CRITICAL]** CORS 白名单仅包含必要的域名，禁止 `Access-Control-Allow-Origin: *`（需认证的 API）
- [ ] **[HIGH]** `Access-Control-Allow-Methods` 仅包含必要的 HTTP 方法
- [ ] **[HIGH]** `Access-Control-Allow-Headers` 仅包含必要的请求头
- [ ] **[MEDIUM]** 预检请求（OPTIONS）已正确缓存 (`Access-Control-Max-Age`)

### 3.4 其他安全

- [ ] **[CRITICAL]** 全站强制 HTTPS，HTTP 请求 301 重定向到 HTTPS
- [ ] **[CRITICAL]** 已配置 `Strict-Transport-Security` (HSTS) 头
- [ ] **[HIGH]** 敏感数据（Token、密码）不存储在 `localStorage`，使用 `httpOnly Cookie`
- [ ] **[HIGH]** 表单提交已实现 CSRF 防护
- [ ] **[HIGH]** 依赖包无已知高危漏洞（`npm audit` / `snyk test` 通过）
- [ ] **[MEDIUM]** Subresource Integrity (SRI) 已配置（CDN 引入的第三方脚本）
- [ ] **[MEDIUM]** 已配置 `Referrer-Policy: strict-origin-when-cross-origin`

---

## 四、无障碍 (Accessibility)

### 4.1 WCAG 合规

- [ ] **[CRITICAL]** WCAG 2.1 AA 级合规（或满足业务要求的合规等级）
- [ ] **[HIGH]** 已使用自动化工具扫描（axe-core、Lighthouse Accessibility）
- [ ] **[HIGH]** 颜色对比度满足 WCAG 要求（正文 ≥ 4.5:1，大文本 ≥ 3:1）
- [ ] **[MEDIUM]** 已进行真人无障碍测试（邀请视障/运动障碍用户参与）

### 4.2 键盘导航

- [ ] **[CRITICAL]** 所有交互元素可通过 Tab 键访问
- [ ] **[CRITICAL]** 焦点指示器（focus indicator）清晰可见且未被 CSS 隐藏
- [ ] **[HIGH]** Tab 顺序符合逻辑阅读顺序
- [ ] **[HIGH]** 模态框/弹窗实现了焦点陷阱（focus trap）
- [ ] **[HIGH]** 提供了跳过导航链接（Skip to main content）
- [ ] **[MEDIUM]** 自定义组件（下拉菜单、Tab 面板等）实现了键盘操作模式

### 4.3 屏幕阅读器

- [ ] **[CRITICAL]** 所有图片有 `alt` 文本（装饰性图片使用 `alt=""`）
- [ ] **[CRITICAL]** 表单输入有关联的 `<label>` 元素
- [ ] **[HIGH]** 使用语义化 HTML 标签（`<nav>`、`<main>`、`<article>`、`<aside>`）
- [ ] **[HIGH]** ARIA 属性使用正确（`aria-label`、`aria-describedby`、`role`）
- [ ] **[HIGH]** 动态内容更新使用 `aria-live` 通知屏幕阅读器
- [ ] **[MEDIUM]** 已使用 NVDA / VoiceOver / TalkBack 实际测试核心流程
- [ ] **[MEDIUM]** 页面标题层级正确（h1 → h2 → h3，不跳级）

### 4.4 其他无障碍

- [ ] **[HIGH]** 不仅依赖颜色传达信息（如错误不仅用红色，还有图标/文字）
- [ ] **[HIGH]** 动画提供了 `prefers-reduced-motion` 适配
- [ ] **[MEDIUM]** 自动播放的媒体提供了暂停/停止控制
- [ ] **[MEDIUM]** 文本可放大至 200% 而不丢失功能

---

## 五、兼容性 (Compatibility)

### 5.1 浏览器兼容

- [ ] **[CRITICAL]** Chrome 最新两个版本测试通过
- [ ] **[CRITICAL]** Safari 最新两个版本测试通过（含 iOS Safari）
- [ ] **[CRITICAL]** Firefox 最新两个版本测试通过
- [ ] **[HIGH]** Edge 最新两个版本测试通过
- [ ] **[HIGH]** `browserslist` 配置与目标用户浏览器分布匹配
- [ ] **[MEDIUM]** 已处理 CSS 前缀（通过 Autoprefixer 自动添加）
- [ ] **[MEDIUM]** 不支持的浏览器显示友好提示而非白屏

### 5.2 响应式适配

- [ ] **[CRITICAL]** 移动端（375px）布局正常且可操作
- [ ] **[CRITICAL]** 平板端（768px）布局正常
- [ ] **[CRITICAL]** 桌面端（1440px）布局正常
- [ ] **[HIGH]** 超宽屏（1920px+）内容不过度拉伸
- [ ] **[HIGH]** 触摸目标尺寸 ≥ 44×44 px（移动端）
- [ ] **[HIGH]** 横竖屏切换不破坏布局
- [ ] **[MEDIUM]** 打印样式已配置（如有打印需求）

### 5.3 国际化（如适用）

- [ ] **[HIGH]** 文本使用 i18n 框架管理，无硬编码文案
- [ ] **[HIGH]** 支持 RTL（Right-to-Left）布局（如目标市场包含阿拉伯语/希伯来语）
- [ ] **[MEDIUM]** 日期、数字、货币格式已本地化
- [ ] **[MEDIUM]** 长文本不会撑破布局（德语/俄语等文本通常比英语长 30-40%）

---

## 六、监控 (Monitoring)

### 6.1 错误追踪

- [ ] **[CRITICAL]** 前端错误追踪已接入（Sentry / Datadog / Bugsnag）
- [ ] **[CRITICAL]** 未捕获的异常（unhandledrejection / onerror）已全局捕获并上报
- [ ] **[HIGH]** 错误上报包含用户上下文（浏览器、OS、页面 URL、用户 ID）
- [ ] **[HIGH]** Source Map 已上传到错误追踪平台（生产环境不暴露 Source Map 文件）
- [ ] **[HIGH]** 关键业务流程（登录、支付、下单）设置了错误率告警
- [ ] **[MEDIUM]** 已配置错误采样率（高流量场景避免上报量爆炸）

### 6.2 性能监控

- [ ] **[HIGH]** Real User Monitoring (RUM) 已接入
- [ ] **[HIGH]** Core Web Vitals 持续监控并设置告警阈值
- [ ] **[HIGH]** 关键页面加载时间已建立基线（baseline）
- [ ] **[MEDIUM]** 长任务（Long Tasks > 50ms）监控已启用
- [ ] **[MEDIUM]** 资源加载失败（CSS/JS/图片 404）监控已启用

### 6.3 用户分析

- [ ] **[HIGH]** Analytics 工具已接入（GA4 / Plausible / Mixpanel）
- [ ] **[HIGH]** 关键事件追踪已配置（注册、登录、购买、核心功能使用）
- [ ] **[HIGH]** 隐私合规已确认（GDPR Cookie Consent / CCPA）
- [ ] **[MEDIUM]** UTM 参数追踪已配置
- [ ] **[MEDIUM]** 漏斗分析已建立（从着陆到转化的完整路径）

### 6.4 可用性监控

- [ ] **[CRITICAL]** 外部拨测（Synthetic Monitoring）已配置，覆盖核心页面
- [ ] **[HIGH]** CDN 可用性和命中率监控已配置
- [ ] **[HIGH]** API 健康检查接口存在且被监控
- [ ] **[MEDIUM]** DNS 解析监控已配置
- [ ] **[MEDIUM]** TLS 证书过期监控已配置（提前 30 天告警）

---

## 七、上线前最终确认

### 7.1 环境配置

- [ ] **[CRITICAL]** 生产环境环境变量已正确配置（API URL、Feature Flag 等）
- [ ] **[CRITICAL]** 生产构建已使用 `production` 模式（无 debug 日志、无 source map 暴露）
- [ ] **[CRITICAL]** console.log / debugger 语句已清理
- [ ] **[HIGH]** 环境变量中无硬编码的密钥/Token

### 7.2 回滚准备

- [ ] **[CRITICAL]** 回滚方案已文档化且经过测试
- [ ] **[HIGH]** 上一个稳定版本的制品仍可用
- [ ] **[HIGH]** 数据库变更（如有）支持回滚
- [ ] **[MEDIUM]** 已确认回滚后的用户体验（缓存、Service Worker 清理）

### 7.3 沟通与协调

- [ ] **[HIGH]** 上线时间已通知相关团队（后端、QA、客服、产品）
- [ ] **[HIGH]** 上线后观察计划已制定（谁在哪个时间段观察哪些指标）
- [ ] **[MEDIUM]** 用户公告/更新日志已准备（如有面向用户的重大变更）

---

## Agent Checklist

- [ ] 覆盖全部六大维度：性能、SEO、安全、无障碍、兼容性、监控
- [ ] 性能部分包含 Lighthouse 评分、Core Web Vitals、Bundle Size、图片优化、缓存策略
- [ ] SEO 部分包含 Meta 标签、Open Graph、Sitemap、robots.txt
- [ ] 安全部分包含 CSP、XSS 防护、CORS 配置
- [ ] 无障碍部分包含 WCAG、键盘导航、屏幕阅读器支持
- [ ] 兼容性部分包含浏览器兼容和响应式适配
- [ ] 监控部分包含错误追踪、性能监控、用户分析
- [ ] 使用 [CRITICAL] / [HIGH] / [MEDIUM] 分级标记
- [ ] 检查项具备可操作性，包含具体阈值和工具建议
- [ ] 文件超过 200 行
