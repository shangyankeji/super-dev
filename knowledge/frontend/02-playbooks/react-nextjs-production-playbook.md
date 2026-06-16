---
id: react-nextjs-production-playbook
title: React/Next.js 生产级实战手册
domain: frontend
category: 02-playbooks
difficulty: advanced
tags: [frontend, react, nextjs, server-components, performance, ssr, streaming, suspense, code-splitting, bundle, rsc]
quality_score: 93
maintainer: frontend-team@superdev.com
last_updated: 2026-06-14
---

# React/Next.js 生产级实战手册

> 基于 [Next.js 官方 Production Checklist](https://nextjs.org/docs/app/guides/production-checklist) + [Next.js 14 性能优化](https://dev.to/hijazi313/nextjs-14-performance-optimization-modern-approaches-for-production-applications-3n65)

## Server Components vs Client Components

### 默认用 Server Components
```tsx
// ✅ 默认就是 Server Component（零客户端 JS）
// 数据在服务端获取，HTML 直传浏览器
async function ProductList() {
  const products = await fetch('/api/products').then(r => r.json());
  return products.map(p => <ProductCard key={p.id} {...p} />);
}

// ✅ 只在需要交互时标 "use client"
"use client";
import { useState } from 'react';
function AddToCart({ productId }: { productId: string }) {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(count + 1)}>Add ({count})</button>;
}
```

### 何时用哪种
| 场景 | 选择 | 原因 |
|------|------|------|
| 静态内容/列表展示 | Server Component | 零 JS，SEO 友好 |
| 数据获取 | Server Component | 服务端并行获取 |
| 事件处理/状态 | Client Component | 需要 useState/useEffect |
| 浏览器 API | Client Component | window/localStorage |

## 性能优化模式

### 1. Streaming + Suspense
```tsx
// ✅ 流式渲染——页面立刻显示，数据部分加载后填充
export default function Page() {
  return (
    <>
      <Header />           {/* 立刻渲染 */}
      <Suspense fallback={<Skeleton />}>
        <SlowDashboard />  {/* 数据加载完后填充 */}
      </Suspense>
      <Suspense fallback={<Skeleton />}>
        <Comments />       {/* 独立加载 */}
      </Suspense>
    </>
  );
}
```

### 2. 路由级代码分割
```tsx
// ✅ 按路由自动分割（Next.js App Router 内置）
// 每个页面/布局自动是独立 chunk
// 只有访问该路由时才加载

// 手动动态导入（重型组件）
import dynamic from 'next/dynamic';
const CodeEditor = dynamic(() => import('./CodeEditor'), {
  loading: () => <Skeleton />,
  ssr: false,  // 只在客户端加载
});
```

### 3. 图片优化
```tsx
import Image from 'next/image';

// ✅ 自动：AVIF/WebP 转换 + 响应式 + 懒加载 + 防布局偏移
<Image
  src="/hero.jpg"
  alt="Hero"
  width={1200}
  height={600}
  priority          // 首屏图片优先加载（LCP）
  sizes="(max-width: 768px) 100vw, 50vw"
/>
```

### 4. 字体优化
```tsx
import { Inter } from 'next/font/google';
// ✅ 自动 self-host + font-display: swap + 子集化
const inter = Inter({ subsets: ['latin'], display: 'swap' });
// 用 <html className={inter.className}>
```

## Bundle 大小控制

```tsx
// ❌ 全量导入（bundle 膨胀）
import _ from 'lodash';           // 70KB
import * as Icons from 'lucide';  // 全部图标！

// ✅ 按需导入
import debounce from 'lodash/debounce';  // 1KB
import { Search, Menu } from 'lucide-react';  // 只引两个
```

| 资源 | 预算 |
|------|------|
| First Load JS | < 150KB (gzip) |
| 单页 JS | < 50KB |
| CSS | < 30KB |

## 数据获取最佳实践

```tsx
// ✅ Server Component 内直接 fetch（自动缓存 + 去重）
async function Products() {
  const res = await fetch('/api/products', { next: { revalidate: 60 } });
  // revalidate=60: 60 秒 ISR（增量静态重新生成）
  return <ProductGrid products={await res.json()} />;
}

// ✅ 并行获取（消除瀑布）
async function Dashboard() {
  const [stats, orders, users] = await Promise.all([
    fetch('/api/stats'),
    fetch('/api/orders'),
    fetch('/api/users'),
  ]);
  // 三个请求并行，不是串行
}
```

## 常见陷阱

### 1. Server Component 传函数给 Client Component
```tsx
// ❌ Server Component 不能传函数到 Client Component
<ClientList onClick={handleClick} />  // 报错！

// ✅ 传数据，交互在 Client Component 内部处理
<ClientList items={items} />  // ClientList 内部定义 onClick
```

### 2. 过度使用 "use client"
```tsx
// ❌ 整个页面标 "use client"（放弃 SSR 优势）
"use client";
export default function Page() { /* 全客户端渲染 */ }

// ✅ 只在叶子组件标 "use client"
export default function Page() {  // Server Component
  return (
    <div>
      <StaticContent />      {/* Server */}
      <InteractiveWidget />  {/* Client Component */}
    </div>
  );
}
```
