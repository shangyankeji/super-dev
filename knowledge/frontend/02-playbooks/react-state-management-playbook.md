---
id: react-state-management-playbook
title: React 状态管理实战手册（2025）
domain: frontend
category: 02-playbooks
difficulty: advanced
tags: [react, state-management, zustand, redux, redux-toolkit, jotai, context, state, frontend, enterprise]
quality_score: 93
maintainer: frontend-team@superdev.com
last_updated: 2026-06-15
---

# React 状态管理实战手册（2025）

> 基于 [State Management in 2025](https://dev.to/hijazi313/state-management-in-2025-when-to-use-context-redux-zustand-or-jotai-2d2k) + [Makers Den 2025 Guide](https://makersden.io/blog/react-state-management-in-2025) + [Reddit 2025 共识](https://www.reddit.com/r/react/comments/1neu4wc/)

## 选型决策树

```
状态类型？
├── 服务端数据（API 响应）→ React Query / SWR（不是 Redux！）
├── URL 状态（筛选/分页）→ URL params / searchParams
├── 表单状态 → React Hook Form / Zod
├── 局部 UI 状态 → useState / useReducer
└── 全局客户端状态
    ├── 简单（< 10 个状态）→ Zustand
    ├── 复杂（大量交互/时间旅行）→ Redux Toolkit
    ├── 细粒度派生 → Jotai
    └── 状态机 → XState
```

## Zustand（推荐默认选择）

```typescript
import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';

interface CartStore {
  items: CartItem[];
  addItem: (item: CartItem) => void;
  removeItem: (id: string) => void;
  clear: () => void;
  total: () => number;
}

// ✅ 无 Provider、无 boilerplate、性能好
export const useCart = create<CartStore>()(
  devtools(                              // Redux DevTools 支持
    persist(                              // localStorage 持久化
      (set, get) => ({
        items: [],
        addItem: (item) => set((s) => ({ items: [...s.items, item] })),
        removeItem: (id) => set((s) => ({ items: s.items.filter(i => i.id !== id) })),
        clear: () => set({ items: [] }),
        total: () => get().items.reduce((sum, i) => sum + i.price * i.qty, 0),
      }),
      { name: 'cart-storage' }
    )
  )
);

// 组件中用——只订阅需要的字段（性能优化）
function CartTotal() {
  const total = useCart((s) => s.items.reduce((sum, i) => sum + i.price * i.qty, 0));
  return <span>${total.toFixed(2)}</span>;
}
```

## Redux Toolkit（复杂应用）

```typescript
import { createSlice, configureStore, createAsyncThunk } from '@reduxjs/toolkit';

// Slice（比旧版 Redux 少 80% boilerplate）
const orderSlice = createSlice({
  name: 'orders',
  initialState: { items: [], loading: false, error: null },
  reducers: {
    clearOrders: (state) => { state.items = []; },
  },
  extraReducers: (builder) => {
    builder
      .addCase(fetchOrders.pending, (state) => { state.loading = true; })
      .addCase(fetchOrders.fulfilled, (state, action) => {
        state.loading = false;
        state.items = action.payload;
      })
      .addCase(fetchOrders.rejected, (state, action) => {
        state.loading = false;
        state.error = action.error.message;
      });
  },
});

// Async thunk
export const fetchOrders = createAsyncThunk('orders/fetch', async () => {
  return await api.get('/api/orders');
});

// Store
const store = configureStore({
  reducer: { orders: orderSlice.reducer },
});
```

## Jotai（细粒度原子状态）

```typescript
import { atom, useAtom } from 'jotai';

// ✅ 原子化——每个状态独立，精准更新
const filterAtom = atom('all');
const todosAtom = atom<Todo[]>([]);
const filteredTodosAtom = atom((get) => {
  const filter = get(filterAtom);
  const todos = get(todosAtom);
  if (filter === 'active') return todos.filter(t => !t.done);
  if (filter === 'done') return todos.filter(t => t.done);
  return todos;
});

// 组件只订阅派生的原子
function TodoList() {
  const [todos] = useAtom(filteredTodosAtom);
  return todos.map(t => <TodoItem key={t.id} todo={t} />);
}
```

## 服务端状态：React Query（不要用 Redux 管 API 数据！）

```typescript
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

// ✅ React Query 管 API 数据（缓存/重试/失效/乐观更新）
function useOrders() {
  return useQuery({
    queryKey: ['orders'],
    queryFn: () => api.get('/api/orders'),
    staleTime: 60000,    // 1 分钟内不重新请求
  });
}

function useCreateOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data) => api.post('/api/orders', data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['orders'] }),  // 自动刷新列表
  });
}
```

## 状态反模式

```tsx
// ❌ 把 API 响应放 Redux（React Query 做得更好）
// ❌ 把 URL 筛选放全局状态（URL params 更可分享/可书签）
// ❌ Context 放高频更新数据（所有消费者重渲染）
// ❌ 所有状态都全局化（能局部就局部）

// ✅ 服务端数据 → React Query
// ✅ URL 状态 → useSearchParams
// ✅ 全局 UI 状态 → Zustand
// ✅ 局部状态 → useState
```

## 生产检查清单
- [ ] 服务端数据用 React Query/SWR（不用 Redux）
- [ ] URL 状态在 URL（可分享/可书签）
- [ ] 全局状态用 Zustand（默认）或 RTK（复杂）
- [ ] Context 只用于低频更新（theme/locale）
- [ ] 状态选择器精准（避免全量重渲染）
- [ ] 持久化关键状态（购物车/偏好）
- [ ] DevTools 可调试（Redux DevTools 兼容）
