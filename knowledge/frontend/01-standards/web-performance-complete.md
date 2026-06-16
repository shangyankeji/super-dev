---
id: web-performance-complete
title: Web 性能优化完整指南
domain: frontend
category: 01-standards
difficulty: intermediate
tags: [complete, core, frontend, performance, vitals, web, 性能监控, 性能预算]
quality_score: 70
last_updated: 2026-06-15
---
# Web 性能优化完整指南

## 概述

Web 性能直接影响用户体验、SEO 排名和业务转化率。Google 研究表明页面加载时间每增加 1 秒，转化率下降 7%。本指南覆盖 Core Web Vitals 优化、资源优化、渲染优化和网络优化的完整策略。

---

## Core Web Vitals

### LCP (Largest Contentful Paint)

目标：< 2.5 秒

LCP 衡量最大内容元素的渲染时间，通常是 Hero 图片、视频封面或大段文字。

**优化策略：**

```html
<!-- 预加载关键资源 -->
<link rel="preload" href="/hero.webp" as="image" fetchpriority="high" />
<link rel="preload" href="/fonts/inter.woff2" as="font" type="font/woff2" crossorigin />

<!-- 关键 CSS 内联 -->
<style>
  /* 首屏关键样式直接内联 */
  .hero { min-height: 60vh; display: flex; align-items: center; }
</style>

<!-- 非关键 CSS 异步加载 -->
<link rel="stylesheet" href="/styles.css" media="print" onload="this.media='all'" />
```

```tsx
// Next.js / React - LCP 图片优化
import Image from "next/image";

function Hero() {
  return (
    <Image
      src="/hero.webp"
      alt="Hero"
      width={1200}
      height={600}
      priority                    // 关键：LCP 图片必须 priority
      sizes="100vw"
      placeholder="blur"
      blurDataURL={blurDataUrl}
    />
  );
}
```

### INP (Interaction to Next Paint)

目标：< 200ms

INP 衡量整个页面生命周期中最慢交互的响应延迟。

```typescript
// 拆分长任务，让出主线程
function processLargeList(items: Item[]) {
  const CHUNK_SIZE = 50;
  let index = 0;

  function processChunk() {
    const end = Math.min(index + CHUNK_SIZE, items.length);
    for (let i = index; i < end; i++) {
      processItem(items[i]);
    }
    index = end;
    if (index < items.length) {
      // 让出主线程给用户交互
      requestIdleCallback(processChunk);
    }
  }
  processChunk();
}

// React 18: useTransition 降低交互阻塞
function SearchResults() {
  const [query, setQuery] = useState("");
  const [isPending, startTransition] = useTransition();

  function handleChange(e: ChangeEvent<HTMLInputElement>) {
    setQuery(e.target.value);               // 高优先级：立即更新输入框
    startTransition(() => {
      setFilteredResults(filterItems(e.target.value)); // 低优先级：延迟更新列表
    });
  }

  return (
    <>
      <input value={query} onChange={handleChange} />
      {isPending ? <Spinner /> : <ResultList items={filteredResults} />}
    </>
  );
}
```

### CLS (Cumulative Layout Shift)

目标：< 0.1

CLS 衡量页面视觉稳定性。

```css
/* 图片/视频预留空间 */
img, video {
  aspect-ratio: 16/9;     /* 现代浏览器 */
  width: 100%;
  height: auto;
}

/* 字体加载防抖动 */
@font-face {
  font-family: "Inter";
  src: url("/fonts/inter.woff2") format("woff2");
  font-display: swap;     /* 或 optional（更激进） */
  size-adjust: 100%;
  ascent-override: 90%;
  descent-override: 22%;
  line-gap-override: 0%;
}

/* 骨架屏占位 */
.skeleton {
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: 4px;
}
```

---

## 资源优化

### 图片优化

```bash
# 格式选择优先级
# 1. AVIF (最小体积，兼容性增长中)
# 2. WebP (广泛支持)
# 3. JPEG/PNG (兜底)
```

```html
<picture>
  <source srcset="/hero.avif" type="image/avif" />
  <source srcset="/hero.webp" type="image/webp" />
  <img src="/hero.jpg" alt="Hero" width="1200" height="600" loading="lazy" decoding="async" />
</picture>
```

### JavaScript 优化

```javascript
// webpack / vite - 代码分割
// 路由级分割
const Dashboard = lazy(() => import("./pages/Dashboard"));
const Settings = lazy(() => import("./pages/Settings"));

// 条件加载（仅在需要时引入）
async function openEditor() {
  const { Editor } = await import("./components/Editor");
  renderEditor(Editor);
}

// Tree Shaking - 使用命名导入
import { debounce } from "lodash-es";     // 仅打包 debounce
// 不要: import _ from "lodash";           // 打包整个 lodash
```

```javascript
// vite.config.ts - 手动分块
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ["react", "react-dom"],
          ui: ["@radix-ui/react-dialog", "@radix-ui/react-dropdown-menu"],
          charts: ["recharts"],
        },
      },
    },
  },
});
```

### CSS 优化

```css
/* 使用 CSS Layers 管理优先级 */
@layer base, components, utilities;

/* 使用 CSS containment 优化渲染 */
.card {
  contain: layout style paint;  /* 隔离重排范围 */
  content-visibility: auto;     /* 离屏内容延迟渲染 */
  contain-intrinsic-size: 200px;
}

/* 使用 will-change 提示浏览器 */
.animate-slide {
  will-change: transform;       /* 仅在动画前设置 */
  transition: transform 0.3s ease;
}
```

### 字体优化

```css
/* 仅加载需要的字符子集 */
@font-face {
  font-family: "NotoSansSC";
  src: url("/fonts/NotoSansSC-Regular.woff2") format("woff2");
  font-weight: 400;
  font-display: swap;
  unicode-range: U+4E00-9FFF;  /* 仅 CJK 基本区 */
}
```

---

## 渲染优化

### React 渲染优化

```tsx
// 1. memo 防止不必要重渲染
const ExpensiveList = memo(function ExpensiveList({ items }: { items: Item[] }) {
  return items.map(item => <ListItem key={item.id} item={item} />);
});

// 2. useMemo 缓存计算结果
function Dashboard({ data }: { data: DataPoint[] }) {
  const chartData = useMemo(() =>
    data.map(d => ({ x: d.date, y: d.value })).sort((a, b) => a.x - b.x),
    [data]
  );
  return <Chart data={chartData} />;
}

// 3. useCallback 稳定回调引用
function Parent() {
  const handleClick = useCallback((id: string) => {
    setSelected(id);
  }, []);
  return <ChildList onClick={handleClick} />;
}

// 4. 虚拟列表处理大数据集
import { useVirtualizer } from "@tanstack/react-virtual";

function VirtualList({ items }: { items: Item[] }) {
  const parentRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 50,
    overscan: 5,
  });

  return (
    <div ref={parentRef} style={{ height: "400px", overflow: "auto" }}>
      <div style={{ height: `${virtualizer.getTotalSize()}px`, position: "relative" }}>
        {virtualizer.getVirtualItems().map(virtualRow => (
          <div
            key={virtualRow.index}
            style={{
              position: "absolute",
              top: 0,
              transform: `translateY(${virtualRow.start}px)`,
              height: `${virtualRow.size}px`,
              width: "100%",
            }}
          >
            {items[virtualRow.index].name}
          </div>
        ))}
      </div>
    </div>
  );
}
```

### Vue 渲染优化

```vue
<script setup>
import { shallowRef, computed } from "vue";

// shallowRef 避免深层响应式追踪
const items = shallowRef<Item[]>([]);

// computed 自动缓存
const sortedItems = computed(() =>
  [...items.value].sort((a, b) => a.name.localeCompare(b.name))
);
</script>

<template>
  <!-- v-once 静态内容只渲染一次 -->
  <header v-once>
    <h1>{{ title }}</h1>
  </header>

  <!-- v-memo 条件缓存 -->
  <div v-for="item in items" :key="item.id" v-memo="[item.id, item.updated]">
    {{ item.name }}
  </div>
</template>
```

---

## 网络优化

### HTTP 缓存策略

```nginx
# Nginx 缓存配置
# 不可变资源（带 hash 的静态文件）
location /assets/ {
    expires max;
    add_header Cache-Control "public, max-age=31536000, immutable";
}

# HTML 文件（总是重新验证）
location / {
    add_header Cache-Control "no-cache, must-revalidate";
    etag on;
}

# API 响应（短期缓存）
location /api/ {
    add_header Cache-Control "private, max-age=0, must-revalidate";
}
```

### 预加载与预连接

```html
<!-- DNS 预解析 -->
<link rel="dns-prefetch" href="//api.example.com" />

<!-- 预连接（包含 TLS 握手）-->
<link rel="preconnect" href="https://cdn.example.com" crossorigin />

<!-- 预加载关键路由 -->
<link rel="prefetch" href="/dashboard" />

<!-- 模块预加载 -->
<link rel="modulepreload" href="/js/dashboard.js" />
```

### Service Worker 离线策略

```javascript
// sw.js - Stale While Revalidate 策略
self.addEventListener("fetch", (event) => {
  if (event.request.destination === "image") {
    event.respondWith(
      caches.open("images-v1").then(async (cache) => {
        const cached = await cache.match(event.request);
        const fetched = fetch(event.request).then((response) => {
          cache.put(event.request, response.clone());
          return response;
        });
        return cached || fetched;
      })
    );
  }
});
```

---

## 性能监控

### Web Vitals 采集

```typescript
import { onLCP, onINP, onCLS } from "web-vitals";

function sendToAnalytics(metric: Metric) {
  const body = JSON.stringify({
    name: metric.name,
    value: metric.value,
    rating: metric.rating,   // "good" | "needs-improvement" | "poor"
    delta: metric.delta,
    id: metric.id,
    url: location.href,
  });
  navigator.sendBeacon("/api/vitals", body);
}

onLCP(sendToAnalytics);
onINP(sendToAnalytics);
onCLS(sendToAnalytics);
```

### Performance API

```typescript
// 标记和测量自定义时间段
performance.mark("data-fetch-start");
const data = await fetchData();
performance.mark("data-fetch-end");
performance.measure("data-fetch", "data-fetch-start", "data-fetch-end");

const measure = performance.getEntriesByName("data-fetch")[0];
console.log(`数据获取耗时: ${measure.duration.toFixed(0)}ms`);
```

---

## 性能预算

```json
{
  "budgets": [
    {
      "resourceType": "script",
      "budget": 300,
      "unit": "KB",
      "action": "error"
    },
    {
      "resourceType": "stylesheet",
      "budget": 100,
      "unit": "KB",
      "action": "warn"
    },
    {
      "metric": "LCP",
      "budget": 2500,
      "unit": "ms",
      "action": "error"
    },
    {
      "metric": "CLS",
      "budget": 0.1,
      "action": "warn"
    }
  ]
}
```

---

## 常见反模式

| 反模式 | 影响 | 正确做法 |
|--------|------|----------|
| 首屏加载全量 JS | LCP 恶化 | 按路由代码分割 |
| 大图未压缩 | 带宽浪费 | WebP/AVIF + 响应式尺寸 |
| 同步第三方脚本 | 阻塞渲染 | async/defer + 延迟加载 |
| 无限滚动无虚拟化 | 内存泄漏 | 虚拟列表 |
| CSS-in-JS 运行时开销 | INP 恶化 | 编译时 CSS (Tailwind/vanilla-extract) |
| 字体全量加载 | FOIT/FOUT | unicode-range + font-display |
| 忽略 cache header | 重复下载 | 分层缓存策略 |

---

## Agent Checklist

在 AI 编码流水线中优化 Web 性能时，必须逐项检查：

- [ ] LCP < 2.5s：首屏关键图片 preload + priority，关键 CSS 内联
- [ ] INP < 200ms：长任务拆分，useTransition 处理低优先级更新
- [ ] CLS < 0.1：图片/视频预留空间，字体 font-display: swap
- [ ] JS 按路由分割，首屏 JS < 300KB (gzip)
- [ ] 图片使用 WebP/AVIF，设置 width/height，非首屏 loading="lazy"
- [ ] 第三方脚本 async/defer，非关键脚本延迟到交互后加载
- [ ] 静态资源带 hash 并设置 immutable 缓存
- [ ] HTML 设置 no-cache + ETag，确保更新及时
- [ ] 大列表使用虚拟化 (@tanstack/react-virtual 或类似方案)
- [ ] 字体子集化并使用 next/font 或 @font-face unicode-range
- [ ] 部署 gzip/brotli 压缩
- [ ] 接入 Web Vitals 监控并设置性能预算
- [ ] 预连接关键第三方域名 (CDN/API)
- [ ] 开发环境运行 Lighthouse CI 回归检测
