---
id: frontend-performance-playbook
title: 前端性能优化 Playbook
domain: frontend
category: 02-playbooks
difficulty: intermediate
tags: [frontend, performance, playbook, vitals, 代码分割, 关键渲染路径优化, 性能审计, 懒加载]
quality_score: 70
last_updated: 2026-06-15
---
# 前端性能优化 Playbook

> 适用场景：Web 应用性能审计、优化执行、持续监控全流程。
> 优先级排序：关键渲染路径 > 资源体积 > 运行时性能 > 缓存与分发。

---

## 1. 性能审计

### 1.1 Lighthouse 审计

```bash
# CLI 审计（Chrome 需已安装）
npx lighthouse https://example.com \
  --output=json,html \
  --output-path=./reports/lighthouse \
  --preset=desktop \
  --chrome-flags="--headless --no-sandbox"

# 移动端审计
npx lighthouse https://example.com \
  --preset=perf \
  --emulated-form-factor=mobile \
  --throttling.cpuSlowdownMultiplier=4
```

**关键指标阈值**：

| 指标 | 良好 | 需改善 | 差 |
|------|------|--------|----|
| LCP (Largest Contentful Paint) | < 2.5s | 2.5-4.0s | > 4.0s |
| FID (First Input Delay) | < 100ms | 100-300ms | > 300ms |
| CLS (Cumulative Layout Shift) | < 0.1 | 0.1-0.25 | > 0.25 |
| INP (Interaction to Next Paint) | < 200ms | 200-500ms | > 500ms |
| TTFB (Time to First Byte) | < 800ms | 800-1800ms | > 1800ms |

### 1.2 WebPageTest 审计

```bash
# 使用 WebPageTest API
curl -X POST "https://www.webpagetest.org/runtest.php" \
  -d "url=https://example.com" \
  -d "f=json" \
  -d "runs=3" \
  -d "fvonly=1" \
  -d "location=ec2-ap-northeast-1.3GFast" \
  -d "k=YOUR_API_KEY"
```

**重点关注**：
- Waterfall 瀑布图：识别阻塞资源
- filmstrip 胶片视图：确认视觉加载过程
- 请求数与传输体积
- TTFB 与服务端响应时间

### 1.3 Chrome DevTools 性能分析

```
Performance 面板：
1. 录制页面加载 → 分析 Main 线程长任务（> 50ms）
2. 检查 Layout Shift 区域
3. 识别 Scripting / Rendering / Painting 占比

Network 面板：
1. 启用 Slow 3G 节流 → 观察资源加载顺序
2. 按 Size 排序 → 定位大文件
3. 检查 Cache-Control 头
```

---

## 2. 关键渲染路径优化

### 2.1 消除渲染阻塞资源

```html
<!-- 错误：阻塞渲染的 CSS -->
<link rel="stylesheet" href="/styles/non-critical.css">

<!-- 正确：关键 CSS 内联 + 非关键 CSS 异步加载 -->
<style>
  /* 首屏关键 CSS，通过 critical 工具提取 */
  .hero { display: flex; min-height: 100vh; }
  .nav  { position: sticky; top: 0; }
</style>
<link rel="preload" href="/styles/non-critical.css" as="style"
      onload="this.onload=null;this.rel='stylesheet'">
<noscript>
  <link rel="stylesheet" href="/styles/non-critical.css">
</noscript>
```

```bash
# 使用 critical 提取关键 CSS
npx critical index.html \
  --base ./ \
  --inline \
  --minify \
  --width 1300 \
  --height 900 \
  > index-optimized.html
```

### 2.2 脚本加载策略

```html
<!-- 关键脚本：同步加载（仅限必要的少量代码） -->
<script src="/js/critical.js"></script>

<!-- 非关键脚本：延迟加载 -->
<script defer src="/js/app.js"></script>

<!-- 完全独立的脚本：异步加载 -->
<script async src="/js/analytics.js"></script>

<!-- 模块脚本：默认 defer 行为 -->
<script type="module" src="/js/app.mjs"></script>
```

### 2.3 资源提示（Resource Hints）

```html
<head>
  <!-- DNS 预解析 -->
  <link rel="dns-prefetch" href="//cdn.example.com">

  <!-- 预连接（DNS + TCP + TLS） -->
  <link rel="preconnect" href="https://api.example.com" crossorigin>

  <!-- 预加载关键资源 -->
  <link rel="preload" href="/fonts/Inter-Regular.woff2" as="font"
        type="font/woff2" crossorigin>
  <link rel="preload" href="/hero.webp" as="image"
        fetchpriority="high">

  <!-- 预取下一页资源 -->
  <link rel="prefetch" href="/next-page.js">
</head>
```

---

## 3. 资源优化

### 3.1 图片优化

```bash
# 批量转换为 WebP
for f in *.{png,jpg}; do
  cwebp -q 80 "$f" -o "${f%.*}.webp"
done

# 生成 AVIF（更高压缩率）
npx avif --input="src/images/*.{png,jpg}" --output="dist/images" --quality=50

# 生成响应式图片
npx sharp-cli resize 400 --input hero.jpg --output hero-400.jpg
npx sharp-cli resize 800 --input hero.jpg --output hero-800.jpg
npx sharp-cli resize 1200 --input hero.jpg --output hero-1200.jpg
```

```html
<!-- 响应式图片 + 现代格式 -->
<picture>
  <source srcset="hero-400.avif 400w, hero-800.avif 800w, hero-1200.avif 1200w"
          type="image/avif" sizes="(max-width: 600px) 400px, (max-width: 1024px) 800px, 1200px">
  <source srcset="hero-400.webp 400w, hero-800.webp 800w, hero-1200.webp 1200w"
          type="image/webp" sizes="(max-width: 600px) 400px, (max-width: 1024px) 800px, 1200px">
  <img src="hero-800.jpg" alt="Hero image"
       loading="lazy" decoding="async"
       width="1200" height="600">
</picture>

<!-- 首屏图片：禁用 lazy，提高优先级 -->
<img src="hero.webp" alt="Hero" fetchpriority="high"
     width="1200" height="600">
```

### 3.2 字体优化

```css
/* 字体显示策略 */
@font-face {
  font-family: 'Inter';
  src: url('/fonts/Inter-Regular.woff2') format('woff2');
  font-weight: 400;
  font-style: normal;
  font-display: swap;          /* 先显示后备字体，加载后替换 */
  unicode-range: U+0000-00FF;  /* 仅加载拉丁字符子集 */
}

/* 中文字体子集化 */
@font-face {
  font-family: 'NotoSansSC';
  src: url('/fonts/NotoSansSC-Regular-subset.woff2') format('woff2');
  font-display: swap;
  unicode-range: U+4E00-9FFF;  /* CJK 统一汉字 */
}
```

```bash
# 字体子集化（仅保留使用到的字符）
pip install fonttools brotli
pyftsubset NotoSansSC-Regular.ttf \
  --text-file=used-chars.txt \
  --output-file=NotoSansSC-subset.woff2 \
  --flavor=woff2
```

### 3.3 CSS 优化

```bash
# 移除未使用的 CSS（PurgeCSS）
npx purgecss \
  --css dist/styles.css \
  --content 'dist/**/*.html' 'dist/**/*.js' \
  --output dist/styles.purged.css

# CSS 压缩（cssnano）
npx postcss dist/styles.css -o dist/styles.min.css --use cssnano
```

### 3.4 JavaScript 优化

```bash
# 分析打包体积
npx webpack-bundle-analyzer dist/stats.json

# Vite 项目分析
npx vite-bundle-visualizer

# Tree-shaking 验证：确认 sideEffects 配置
# package.json
{
  "sideEffects": false
}
# 或指定有副作用的文件
{
  "sideEffects": ["*.css", "./src/polyfills.js"]
}
```

---

## 4. 代码分割

### 4.1 路由级分割（React）

```tsx
import { lazy, Suspense } from 'react';
import { Routes, Route } from 'react-router-dom';

// 路由级懒加载
const Dashboard = lazy(() => import('./pages/Dashboard'));
const Settings  = lazy(() => import('./pages/Settings'));
const Reports   = lazy(() => import(
  /* webpackChunkName: "reports" */
  /* webpackPrefetch: true */
  './pages/Reports'
));

function App() {
  return (
    <Suspense fallback={<PageSkeleton />}>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="/reports" element={<Reports />} />
      </Routes>
    </Suspense>
  );
}
```

### 4.2 组件级分割

```tsx
import { lazy, Suspense, useState } from 'react';

// 仅在用户交互后加载重量级组件
const HeavyChart = lazy(() => import('./components/HeavyChart'));
const MarkdownEditor = lazy(() => import('./components/MarkdownEditor'));

function Dashboard() {
  const [showChart, setShowChart] = useState(false);

  return (
    <div>
      <button onClick={() => setShowChart(true)}>显示图表</button>
      {showChart && (
        <Suspense fallback={<ChartSkeleton />}>
          <HeavyChart data={data} />
        </Suspense>
      )}
    </div>
  );
}
```

### 4.3 Vite 分割策略

```ts
// vite.config.ts
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor-react': ['react', 'react-dom', 'react-router-dom'],
          'vendor-charts': ['recharts', 'd3'],
          'vendor-utils': ['lodash-es', 'date-fns'],
        },
      },
    },
    chunkSizeWarningLimit: 500, // KB
  },
});
```

---

## 5. 懒加载

### 5.1 图片懒加载

```tsx
// 原生懒加载
<img src="photo.webp" loading="lazy" alt="Photo" width="400" height="300" />

// Intersection Observer 自定义实现
function useLazyImage(src: string) {
  const [loaded, setLoaded] = useState(false);
  const ref = useRef<HTMLImageElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setLoaded(true);
          observer.disconnect();
        }
      },
      { rootMargin: '200px' } // 提前 200px 开始加载
    );
    if (ref.current) observer.observe(ref.current);
    return () => observer.disconnect();
  }, []);

  return { ref, src: loaded ? src : undefined };
}
```

### 5.2 无限滚动懒加载

```tsx
function useInfiniteScroll(fetchMore: () => Promise<void>) {
  const sentinelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          fetchMore();
        }
      },
      { threshold: 0.1 }
    );
    if (sentinelRef.current) observer.observe(sentinelRef.current);
    return () => observer.disconnect();
  }, [fetchMore]);

  return sentinelRef;
}

// 使用
function ProductList() {
  const sentinelRef = useInfiniteScroll(loadNextPage);
  return (
    <>
      {products.map(p => <ProductCard key={p.id} product={p} />)}
      <div ref={sentinelRef} style={{ height: 1 }} />
    </>
  );
}
```

---

## 6. 缓存策略

### 6.1 HTTP 缓存

```nginx
# Nginx 缓存配置
location /assets/ {
    # 静态资源：强缓存 1 年（文件名含 hash）
    add_header Cache-Control "public, max-age=31536000, immutable";
}

location /index.html {
    # HTML 入口：协商缓存
    add_header Cache-Control "no-cache";
    etag on;
}

location /api/ {
    # API 响应：不缓存或短期缓存
    add_header Cache-Control "private, no-cache, must-revalidate";
}
```

### 6.2 Service Worker 缓存

```js
// sw.js - Workbox 策略
import { precacheAndRoute } from 'workbox-precaching';
import { registerRoute } from 'workbox-routing';
import { CacheFirst, StaleWhileRevalidate, NetworkFirst } from 'workbox-strategies';
import { ExpirationPlugin } from 'workbox-expiration';

// 预缓存构建产物
precacheAndRoute(self.__WB_MANIFEST);

// 字体/图片：CacheFirst
registerRoute(
  ({ request }) => request.destination === 'font' || request.destination === 'image',
  new CacheFirst({
    cacheName: 'static-assets',
    plugins: [new ExpirationPlugin({ maxEntries: 100, maxAgeSeconds: 30 * 24 * 3600 })],
  })
);

// API 数据：NetworkFirst
registerRoute(
  ({ url }) => url.pathname.startsWith('/api/'),
  new NetworkFirst({
    cacheName: 'api-cache',
    networkTimeoutSeconds: 3,
    plugins: [new ExpirationPlugin({ maxEntries: 50, maxAgeSeconds: 5 * 60 })],
  })
);
```

---

## 7. CDN 配置

### 7.1 多 CDN 回源策略

```ts
// CDN URL 构建
const CDN_HOSTS = [
  'https://cdn1.example.com',
  'https://cdn2.example.com',
];

function getCdnUrl(path: string, index: number = 0): string {
  const host = CDN_HOSTS[index % CDN_HOSTS.length];
  return `${host}${path}`;
}

// 图片 CDN（支持参数化裁剪/格式转换）
function getImageUrl(key: string, width: number, format: 'webp' | 'avif' = 'webp'): string {
  return `https://img-cdn.example.com/${key}?w=${width}&fmt=${format}&q=80`;
}
```

### 7.2 Vite CDN 外部化

```ts
// vite.config.ts
import { defineConfig } from 'vite';

export default defineConfig({
  build: {
    rollupOptions: {
      external: ['react', 'react-dom'],
      output: {
        globals: {
          react: 'React',
          'react-dom': 'ReactDOM',
        },
      },
    },
  },
});
```

---

## 8. 性能监控（Web Vitals）

### 8.1 采集 Web Vitals

```ts
import { onCLS, onFID, onLCP, onINP, onTTFB } from 'web-vitals';

interface PerfMetric {
  name: string;
  value: number;
  rating: 'good' | 'needs-improvement' | 'poor';
  navigationType: string;
}

function sendToAnalytics(metric: PerfMetric) {
  const body = JSON.stringify(metric);

  // 优先 sendBeacon，确保页面卸载时也能发送
  if (navigator.sendBeacon) {
    navigator.sendBeacon('/api/vitals', body);
  } else {
    fetch('/api/vitals', { body, method: 'POST', keepalive: true });
  }
}

onCLS(sendToAnalytics);
onFID(sendToAnalytics);
onLCP(sendToAnalytics);
onINP(sendToAnalytics);
onTTFB(sendToAnalytics);
```

### 8.2 自定义性能标记

```ts
// 标记关键业务节点
performance.mark('data-fetch-start');
const data = await fetchDashboardData();
performance.mark('data-fetch-end');
performance.measure('dashboard-data-fetch', 'data-fetch-start', 'data-fetch-end');

// 获取测量结果
const [measure] = performance.getEntriesByName('dashboard-data-fetch');
console.log(`数据加载耗时: ${measure.duration.toFixed(0)}ms`);

// 长任务监控
const observer = new PerformanceObserver((list) => {
  for (const entry of list.getEntries()) {
    if (entry.duration > 50) {
      console.warn(`长任务检测: ${entry.duration.toFixed(0)}ms`, entry);
    }
  }
});
observer.observe({ type: 'longtask', buffered: true });
```

---

## 9. 性能预算

### 9.1 预算定义

```json
// budget.json
[
  {
    "resourceSizes": [
      { "resourceType": "script", "budget": 300 },
      { "resourceType": "stylesheet", "budget": 50 },
      { "resourceType": "image", "budget": 500 },
      { "resourceType": "font", "budget": 100 },
      { "resourceType": "total", "budget": 1000 }
    ],
    "resourceCounts": [
      { "resourceType": "third-party", "budget": 10 },
      { "resourceType": "script", "budget": 15 }
    ],
    "timings": [
      { "metric": "interactive", "budget": 3000 },
      { "metric": "first-contentful-paint", "budget": 1500 },
      { "metric": "largest-contentful-paint", "budget": 2500 }
    ]
  }
]
```

### 9.2 CI 中强制执行预算

```yaml
# .github/workflows/perf-budget.yml
name: Performance Budget
on: [pull_request]

jobs:
  budget-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm ci && npm run build

      - name: Check bundle size
        run: |
          MAX_JS=300  # KB
          MAX_CSS=50  # KB
          JS_SIZE=$(du -sk dist/assets/*.js | awk '{sum+=$1} END {print sum}')
          CSS_SIZE=$(du -sk dist/assets/*.css | awk '{sum+=$1} END {print sum}')
          echo "JS: ${JS_SIZE}KB (budget: ${MAX_JS}KB)"
          echo "CSS: ${CSS_SIZE}KB (budget: ${MAX_CSS}KB)"
          if [ "$JS_SIZE" -gt "$MAX_JS" ]; then
            echo "::error::JS bundle exceeds budget: ${JS_SIZE}KB > ${MAX_JS}KB"
            exit 1
          fi

      - name: Lighthouse CI
        uses: treosh/lighthouse-ci-action@v11
        with:
          budgetPath: ./budget.json
          uploadArtifacts: true
          runs: 3
```

### 9.3 bundlesize 集成

```json
// package.json
{
  "bundlesize": [
    { "path": "dist/assets/index-*.js", "maxSize": "150 kB", "compression": "gzip" },
    { "path": "dist/assets/vendor-*.js", "maxSize": "100 kB", "compression": "gzip" },
    { "path": "dist/assets/index-*.css", "maxSize": "30 kB", "compression": "gzip" }
  ]
}
```

---

## 10. 运行时性能优化

### 10.1 避免强制同步布局

```ts
// 错误：读写交替触发强制回流
elements.forEach(el => {
  const height = el.offsetHeight;       // 读
  el.style.height = height * 2 + 'px';  // 写 → 触发回流
});

// 正确：批量读，批量写
const heights = elements.map(el => el.offsetHeight);  // 批量读
elements.forEach((el, i) => {
  el.style.height = heights[i] * 2 + 'px';            // 批量写
});
```

### 10.2 虚拟滚动

```tsx
import { useVirtualizer } from '@tanstack/react-virtual';

function VirtualList({ items }: { items: Item[] }) {
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 50,
    overscan: 5,
  });

  return (
    <div ref={parentRef} style={{ height: 600, overflow: 'auto' }}>
      <div style={{ height: virtualizer.getTotalSize(), position: 'relative' }}>
        {virtualizer.getVirtualItems().map(virtualRow => (
          <div key={virtualRow.key}
               style={{
                 position: 'absolute',
                 top: 0,
                 transform: `translateY(${virtualRow.start}px)`,
                 height: virtualRow.size,
               }}>
            <ItemRow item={items[virtualRow.index]} />
          </div>
        ))}
      </div>
    </div>
  );
}
```

---

## Agent Checklist

- [ ] 使用 Lighthouse CLI 或 DevTools 完成首次审计，记录基线分数
- [ ] 提取并内联关键 CSS，非关键 CSS 异步加载
- [ ] 所有非关键脚本使用 `defer` 或 `async`
- [ ] 图片已转换为 WebP/AVIF 并提供响应式 srcset
- [ ] 字体使用 `font-display: swap` 并完成子集化
- [ ] 移除未使用的 CSS（PurgeCSS）
- [ ] 路由级和组件级代码分割已实施
- [ ] 首屏外图片和组件已启用懒加载
- [ ] HTTP 缓存头已正确配置（hash 资源强缓存，HTML 协商缓存）
- [ ] Web Vitals 采集已部署，数据可在监控平台查看
- [ ] 性能预算已定义并集成到 CI
- [ ] 长列表使用虚拟滚动
- [ ] 无强制同步布局（Layout Thrashing）
- [ ] bundle 分析已执行，无明显冗余依赖
