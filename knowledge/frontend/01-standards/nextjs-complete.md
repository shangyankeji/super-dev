---
id: nextjs-complete
title: Next.js 完整指南
domain: frontend
category: 01-standards
difficulty: intermediate
tags: [complete, components, frontend, middleware, nextjs, react, router, server]
quality_score: 70
last_updated: 2026-06-15
---
# Next.js 完整指南

## 概述

Next.js 是基于 React 的全栈框架，由 Vercel 开发维护。提供文件系统路由、服务端渲染 (SSR)、静态站点生成 (SSG)、增量静态再生成 (ISR)、API Routes、中间件等能力。Next.js 14+ 引入 App Router 和 React Server Components (RSC)，从根本上改变了数据获取和组件渲染模型。

### 核心特性

- **App Router**: 基于文件系统的嵌套路由，支持 Layout / Loading / Error 等约定文件
- **React Server Components**: 默认服务端组件，减少客户端 JS 体积
- **多种渲染策略**: SSR / SSG / ISR / CSR 按页面粒度灵活选择
- **内置优化**: Image / Font / Script / Metadata 自动优化
- **Middleware**: 在 Edge Runtime 运行的请求拦截层
- **Route Handlers**: 替代 API Routes 的服务端端点
- **Streaming & Suspense**: 流式渲染与渐进式页面加载

---

## App Router 架构

### 目录结构约定

```
app/
├── layout.tsx          # 根布局（必须）
├── page.tsx            # 首页
├── loading.tsx         # 全局 Loading UI
├── error.tsx           # 全局 Error UI
├── not-found.tsx       # 404 页面
├── globals.css
├── dashboard/
│   ├── layout.tsx      # 嵌套布局
│   ├── page.tsx        # /dashboard
│   ├── loading.tsx     # Dashboard Loading
│   └── [id]/
│       └── page.tsx    # /dashboard/:id
├── api/
│   └── users/
│       └── route.ts    # API Route Handler
└── (marketing)/        # Route Group（不影响 URL）
    ├── about/
    │   └── page.tsx
    └── blog/
        └── page.tsx
```

### 路由约定文件

| 文件 | 作用 | 渲染时机 |
|------|------|----------|
| `layout.tsx` | 共享布局，嵌套不重新渲染 | 导航时保持 |
| `page.tsx` | 页面 UI，使路由可访问 | 每次导航 |
| `loading.tsx` | Suspense Loading UI | 页面加载时 |
| `error.tsx` | Error Boundary UI | 出错时 |
| `not-found.tsx` | 404 UI | 未找到时 |
| `route.ts` | API 端点 | 请求时 |
| `template.tsx` | 类似 layout 但每次重新渲染 | 每次导航 |

---

## React Server Components (RSC)

### 服务端组件 vs 客户端组件

```tsx
// 服务端组件（默认）- 不需要标记
// 可以直接访问数据库、文件系统、环境变量
async function ProductList() {
  const products = await db.product.findMany();
  return (
    <ul>
      {products.map(p => (
        <li key={p.id}>{p.name} - ¥{p.price}</li>
      ))}
    </ul>
  );
}
```

```tsx
// 客户端组件 - 需要 "use client" 标记
"use client";

import { useState } from "react";

export function Counter() {
  const [count, setCount] = useState(0);
  return (
    <button onClick={() => setCount(c => c + 1)}>
      Count: {count}
    </button>
  );
}
```

### 何时用服务端 vs 客户端

| 场景 | 服务端组件 | 客户端组件 |
|------|-----------|-----------|
| 数据获取 | 直接 async/await | useEffect / SWR / React Query |
| 敏感逻辑（API Key 等） | 安全 | 不安全 |
| 事件处理（onClick 等） | 不支持 | 支持 |
| State / Effects | 不支持 | 支持 |
| 浏览器 API | 不可用 | 可用 |
| 体积影响 | 零 JS 发送到客户端 | 打包到 bundle |

### 组合模式

```tsx
// 服务端组件可以嵌套客户端组件
// app/dashboard/page.tsx (Server)
import { DashboardChart } from "./DashboardChart"; // Client
import { getStats } from "@/lib/data";

export default async function DashboardPage() {
  const stats = await getStats();
  return (
    <div>
      <h1>Dashboard</h1>
      <p>Total: {stats.total}</p>
      {/* 将服务端数据作为 props 传给客户端组件 */}
      <DashboardChart data={stats.chartData} />
    </div>
  );
}
```

---

## 数据获取

### 服务端数据获取

```tsx
// 直接在组件中 fetch（自动去重和缓存）
async function UserProfile({ userId }: { userId: string }) {
  const user = await fetch(`https://api.example.com/users/${userId}`, {
    next: { revalidate: 3600 }, // ISR: 每小时重新验证
  }).then(res => res.json());

  return <div>{user.name}</div>;
}
```

### 缓存策略

```tsx
// 静态数据（构建时获取，等同 SSG）
fetch("https://api.example.com/data", { cache: "force-cache" });

// 动态数据（每次请求重新获取，等同 SSR）
fetch("https://api.example.com/data", { cache: "no-store" });

// ISR: 每 60 秒重新验证
fetch("https://api.example.com/data", { next: { revalidate: 60 } });
```

### generateStaticParams

```tsx
// app/blog/[slug]/page.tsx
export async function generateStaticParams() {
  const posts = await getAllPosts();
  return posts.map(post => ({ slug: post.slug }));
}

export default async function BlogPost({ params }: { params: { slug: string } }) {
  const post = await getPost(params.slug);
  return <article>{post.content}</article>;
}
```

---

## 中间件 (Middleware)

```typescript
// middleware.ts（项目根目录）
import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

export function middleware(request: NextRequest) {
  // 认证检查
  const token = request.cookies.get("auth-token");
  if (!token && request.nextUrl.pathname.startsWith("/dashboard")) {
    return NextResponse.redirect(new URL("/login", request.url));
  }

  // 国际化重定向
  const locale = request.headers.get("accept-language")?.split(",")[0] || "en";
  if (request.nextUrl.pathname === "/") {
    return NextResponse.redirect(new URL(`/${locale}`, request.url));
  }

  // 添加自定义 Header
  const response = NextResponse.next();
  response.headers.set("x-request-id", crypto.randomUUID());
  return response;
}

export const config = {
  matcher: [
    "/((?!api|_next/static|_next/image|favicon.ico).*)",
  ],
};
```

---

## Server Actions

```tsx
// app/actions.ts
"use server";

import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";

export async function createPost(formData: FormData) {
  const title = formData.get("title") as string;
  const content = formData.get("content") as string;

  // 服务端验证
  if (!title || title.length < 3) {
    return { error: "标题至少 3 个字符" };
  }

  await db.post.create({ data: { title, content } });
  revalidatePath("/blog");
  redirect("/blog");
}
```

```tsx
// app/blog/new/page.tsx
import { createPost } from "../actions";

export default function NewPost() {
  return (
    <form action={createPost}>
      <input name="title" placeholder="标题" required />
      <textarea name="content" placeholder="内容" required />
      <button type="submit">发布</button>
    </form>
  );
}
```

---

## Route Handlers (API)

```typescript
// app/api/users/route.ts
import { NextRequest, NextResponse } from "next/server";

export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const page = parseInt(searchParams.get("page") || "1");
  const users = await db.user.findMany({
    skip: (page - 1) * 20,
    take: 20,
  });
  return NextResponse.json({ users, page });
}

export async function POST(request: NextRequest) {
  const body = await request.json();
  const user = await db.user.create({ data: body });
  return NextResponse.json(user, { status: 201 });
}
```

---

## 性能优化

### Image 组件

```tsx
import Image from "next/image";

export function Hero() {
  return (
    <Image
      src="/hero.jpg"
      alt="Hero image"
      width={1200}
      height={600}
      priority          // LCP 图片预加载
      placeholder="blur" // 模糊占位
      blurDataURL="..."
    />
  );
}
```

### Font 优化

```tsx
// app/layout.tsx
import { Inter, Noto_Sans_SC } from "next/font/google";

const inter = Inter({ subsets: ["latin"], variable: "--font-inter" });
const notoSansSC = Noto_Sans_SC({
  subsets: ["latin"],
  weight: ["400", "500", "700"],
  variable: "--font-noto",
});

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html className={`${inter.variable} ${notoSansSC.variable}`}>
      <body>{children}</body>
    </html>
  );
}
```

### Streaming 与 Suspense

```tsx
import { Suspense } from "react";

export default function Dashboard() {
  return (
    <div>
      <h1>Dashboard</h1>
      <Suspense fallback={<ChartSkeleton />}>
        <RevenueChart />
      </Suspense>
      <Suspense fallback={<TableSkeleton />}>
        <RecentOrders />
      </Suspense>
    </div>
  );
}
```

---

## 部署

### Vercel (推荐)

```bash
# 自动检测 Next.js 并优化部署
vercel deploy
```

### Docker 自托管

```dockerfile
FROM node:20-alpine AS base

FROM base AS deps
WORKDIR /app
COPY package.json pnpm-lock.yaml ./
RUN corepack enable pnpm && pnpm install --frozen-lockfile

FROM base AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .
ENV NEXT_TELEMETRY_DISABLED=1
RUN npm run build

FROM base AS runner
WORKDIR /app
ENV NODE_ENV=production
RUN addgroup --system --gid 1001 nodejs && adduser --system --uid 1001 nextjs
COPY --from=builder /app/public ./public
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static
USER nextjs
EXPOSE 3000
ENV PORT=3000
CMD ["node", "server.js"]
```

### next.config.js 生产配置

```javascript
/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "standalone",     // Docker 部署需要
  poweredByHeader: false,   // 移除 X-Powered-By
  compress: true,
  images: {
    remotePatterns: [
      { protocol: "https", hostname: "cdn.example.com" },
    ],
  },
  headers: async () => [
    {
      source: "/(.*)",
      headers: [
        { key: "X-Frame-Options", value: "DENY" },
        { key: "X-Content-Type-Options", value: "nosniff" },
        { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
      ],
    },
  ],
};

module.exports = nextConfig;
```

---

## 常见反模式

| 反模式 | 问题 | 正确做法 |
|--------|------|----------|
| 在服务端组件用 useState | 编译错误 | 拆分为客户端组件 |
| 在客户端组件直接查数据库 | 安全漏洞 | 通过 API/Server Action |
| 所有组件标 "use client" | 失去 RSC 优势 | 仅交互组件标记 |
| 不设 revalidate | 数据永不更新 | 按业务设置过期时间 |
| 在 Middleware 做重计算 | Edge 超时 | Middleware 只做路由/鉴权 |
| 不用 Image 组件 | 无自动优化 | 始终用 next/image |

---

## Agent Checklist

在 AI 编码流水线中使用 Next.js 时，必须逐项检查：

- [ ] App Router 目录结构遵循约定（layout / page / loading / error）
- [ ] 组件默认为服务端组件，仅在需要交互/状态时标记 "use client"
- [ ] 数据获取使用合适的缓存策略（force-cache / no-store / revalidate）
- [ ] LCP 图片使用 `<Image priority />` 并提供 width/height
- [ ] 字体使用 next/font 避免布局偏移
- [ ] Middleware 仅处理路由/鉴权/重定向，不做重计算
- [ ] Server Actions 包含服务端验证，不信任客户端输入
- [ ] 使用 Suspense 包裹异步组件实现流式加载
- [ ] 生产环境配置 output: "standalone"（Docker）或 Vercel 部署
- [ ] 安全 Header 在 next.config.js 或 Middleware 中统一配置
- [ ] generateStaticParams 用于高访问量的动态路由
- [ ] 敏感环境变量不带 NEXT_PUBLIC_ 前缀
- [ ] 错误边界（error.tsx）覆盖所有关键路由段
- [ ] metadata 或 generateMetadata 为每个页面提供 SEO 信息
