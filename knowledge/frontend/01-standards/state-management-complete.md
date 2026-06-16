---
id: state-management-complete
title: 状态管理完整对比指南
domain: frontend
category: 01-standards
difficulty: intermediate
tags: [complete, frontend, jotai, management, mobx, recoil, redux, state]
quality_score: 70
last_updated: 2026-06-15
---
# 状态管理完整对比指南

## 概述

前端状态管理是复杂应用的核心架构决策。不同方案在理念、API 复杂度、性能特征和生态成熟度上差异显著。本指南覆盖 React、Vue 和跨框架的主流状态管理方案，提供选型依据和最佳实践。

---

## 方案概览对比

| 方案 | 框架 | 理念 | 包体 | 学习曲线 | 适用规模 |
|------|------|------|------|----------|----------|
| Redux Toolkit | React | 单一 Store + Reducer | ~11KB | 中 | 中大型 |
| Zustand | React | 极简 Store | ~1.5KB | 低 | 任意 |
| Jotai | React | 原子化 | ~3KB | 低 | 任意 |
| Recoil | React | 原子化 + 图依赖 | ~20KB | 中 | 中大型 |
| MobX | React | 响应式 | ~16KB | 中 | 任意 |
| Pinia | Vue | Vue 官方推荐 | ~2KB | 低 | 任意 |
| Signals | 多框架 | 细粒度响应式 | ~1KB | 低 | 任意 |

---

## Redux Toolkit (RTK)

### 核心概念

Redux Toolkit 是 Redux 官方推荐的标准化工具集，大幅简化了样板代码。

```typescript
// store/counterSlice.ts
import { createSlice, PayloadAction } from "@reduxjs/toolkit";

interface CounterState {
  value: number;
  status: "idle" | "loading" | "failed";
}

const initialState: CounterState = { value: 0, status: "idle" };

export const counterSlice = createSlice({
  name: "counter",
  initialState,
  reducers: {
    increment: (state) => { state.value += 1; },
    decrement: (state) => { state.value -= 1; },
    incrementByAmount: (state, action: PayloadAction<number>) => {
      state.value += action.payload;
    },
  },
});

export const { increment, decrement, incrementByAmount } = counterSlice.actions;
export default counterSlice.reducer;
```

### RTK Query（异步数据管理）

```typescript
import { createApi, fetchBaseQuery } from "@reduxjs/toolkit/query/react";

export const api = createApi({
  baseQuery: fetchBaseQuery({ baseUrl: "/api" }),
  tagTypes: ["User"],
  endpoints: (builder) => ({
    getUsers: builder.query<User[], void>({
      query: () => "/users",
      providesTags: ["User"],
    }),
    createUser: builder.mutation<User, Partial<User>>({
      query: (body) => ({ url: "/users", method: "POST", body }),
      invalidatesTags: ["User"],
    }),
  }),
});

export const { useGetUsersQuery, useCreateUserMutation } = api;
```

### 适用场景
- 大型团队需要严格的状态变更追踪
- 复杂的跨组件状态共享与调试需求
- 需要 Redux DevTools 时间旅行调试
- 已有 Redux 技术栈的存量项目

---

## Zustand

### 核心概念

Zustand 以极简 API 著称，基于 Hook，不需要 Provider。

```typescript
import { create } from "zustand";
import { devtools, persist } from "zustand/middleware";

interface BearStore {
  bears: number;
  increase: () => void;
  reset: () => void;
}

export const useBearStore = create<BearStore>()(
  devtools(
    persist(
      (set) => ({
        bears: 0,
        increase: () => set((state) => ({ bears: state.bears + 1 })),
        reset: () => set({ bears: 0 }),
      }),
      { name: "bear-storage" }
    )
  )
);

// 组件中使用
function BearCounter() {
  const bears = useBearStore((state) => state.bears);
  const increase = useBearStore((state) => state.increase);
  return <button onClick={increase}>Bears: {bears}</button>;
}
```

### 异步操作

```typescript
const useStore = create<Store>((set, get) => ({
  users: [],
  loading: false,
  fetchUsers: async () => {
    set({ loading: true });
    const users = await fetch("/api/users").then(r => r.json());
    set({ users, loading: false });
  },
}));
```

### 适用场景
- 中小型项目需要简单全局状态
- 不想引入 Provider 嵌套
- 需要与 React 外部系统集成
- 追求最小包体积

---

## Jotai

### 核心概念

Jotai 采用原子化（atom）理念，每个状态独立声明，组合灵活。

```typescript
import { atom, useAtom, useAtomValue, useSetAtom } from "jotai";

// 基础 atom
const countAtom = atom(0);

// 派生 atom（只读）
const doubleCountAtom = atom((get) => get(countAtom) * 2);

// 派生 atom（可写）
const incrementAtom = atom(null, (get, set) => {
  set(countAtom, get(countAtom) + 1);
});

// 异步 atom
const userAtom = atom(async () => {
  const res = await fetch("/api/user");
  return res.json();
});

// 组件使用
function Counter() {
  const [count, setCount] = useAtom(countAtom);
  const doubleCount = useAtomValue(doubleCountAtom);
  return (
    <div>
      <p>Count: {count}, Double: {doubleCount}</p>
      <button onClick={() => setCount(c => c + 1)}>+1</button>
    </div>
  );
}
```

### 适用场景
- 需要细粒度状态更新，避免不必要重渲染
- 状态之间有复杂派生关系
- 需要与 React Suspense 深度集成
- 偏好函数式编程风格

---

## Recoil

### 核心概念

Recoil 由 Meta 开发，引入 atom 和 selector 概念，支持异步数据流和状态图。

```typescript
import { atom, selector, useRecoilState, useRecoilValue } from "recoil";

const todoListState = atom<Todo[]>({
  key: "todoListState",
  default: [],
});

const filteredTodoListState = selector({
  key: "filteredTodoListState",
  get: ({ get }) => {
    const list = get(todoListState);
    const filter = get(todoListFilterState);
    switch (filter) {
      case "completed": return list.filter(item => item.isComplete);
      case "uncompleted": return list.filter(item => !item.isComplete);
      default: return list;
    }
  },
});
```

### 适用场景
- 复杂的异步数据依赖图
- 需要状态快照和时间旅行
- Meta 技术栈项目
- 注意：Recoil 维护活跃度下降，新项目建议考虑 Jotai

---

## MobX

### 核心概念

MobX 基于透明响应式编程，自动追踪依赖和更新。

```typescript
import { makeAutoObservable, runInAction } from "mobx";
import { observer } from "mobx-react-lite";

class TodoStore {
  todos: Todo[] = [];
  loading = false;

  constructor() {
    makeAutoObservable(this);
  }

  async fetchTodos() {
    this.loading = true;
    const data = await fetch("/api/todos").then(r => r.json());
    runInAction(() => {
      this.todos = data;
      this.loading = false;
    });
  }

  get completedCount() {
    return this.todos.filter(t => t.done).length;
  }

  toggleTodo(id: string) {
    const todo = this.todos.find(t => t.id === id);
    if (todo) todo.done = !todo.done;
  }
}

const todoStore = new TodoStore();

const TodoList = observer(() => (
  <div>
    <p>Completed: {todoStore.completedCount}</p>
    {todoStore.todos.map(todo => (
      <div key={todo.id} onClick={() => todoStore.toggleTodo(todo.id)}>
        {todo.done ? "✓" : "○"} {todo.title}
      </div>
    ))}
  </div>
));
```

### 适用场景
- 面向对象编程偏好的团队
- 需要自动依赖追踪
- 复杂的本地状态模型
- 从 Java/C# 转前端的团队

---

## Pinia (Vue)

### 核心概念

Pinia 是 Vue 官方推荐的状态管理方案，取代 Vuex。

```typescript
// stores/counter.ts
import { defineStore } from "pinia";

export const useCounterStore = defineStore("counter", {
  state: () => ({
    count: 0,
    name: "Counter",
  }),
  getters: {
    doubleCount: (state) => state.count * 2,
  },
  actions: {
    increment() {
      this.count++;
    },
    async fetchCount() {
      const res = await fetch("/api/count");
      this.count = await res.json();
    },
  },
});

// Composition API 风格
export const useCounterStore = defineStore("counter", () => {
  const count = ref(0);
  const doubleCount = computed(() => count.value * 2);
  function increment() { count.value++; }
  return { count, doubleCount, increment };
});
```

```vue
<script setup>
import { useCounterStore } from "@/stores/counter";
const counter = useCounterStore();
</script>

<template>
  <p>{{ counter.count }} / {{ counter.doubleCount }}</p>
  <button @click="counter.increment()">+1</button>
</template>
```

### 适用场景
- 所有 Vue 3 项目的首选状态管理
- 需要 SSR 支持（Nuxt 内置集成）
- 需要 TypeScript 完整类型推断
- 从 Vuex 迁移

---

## Signals

### 核心概念

Signals 是一种细粒度响应式原语，多个框架正在采纳。

```typescript
// Preact Signals 示例
import { signal, computed, effect } from "@preact/signals-react";

const count = signal(0);
const doubled = computed(() => count.value * 2);

// 自动追踪，无需 useEffect
effect(() => {
  console.log(`Count changed: ${count.value}`);
});

function Counter() {
  return (
    <div>
      <p>Count: {count}, Double: {doubled}</p>
      <button onClick={() => count.value++}>+1</button>
    </div>
  );
}
```

### 适用场景
- 追求极致渲染性能
- Preact / Solid / Angular 16+ 项目
- 细粒度 DOM 更新需求
- 实验性或性能敏感项目

---

## 选型决策矩阵

| 决策因素 | 推荐方案 |
|----------|----------|
| Vue 项目 | Pinia |
| React 小项目 | Zustand |
| React 大型项目 + 强规范 | Redux Toolkit |
| 细粒度更新 + React | Jotai |
| 面向对象 + React | MobX |
| 服务端状态为主 | React Query / SWR + 轻量客户端状态 |
| 表单为主 | React Hook Form + Zustand/Jotai |
| 极致性能 | Signals / Jotai |

### 组合策略

现代应用通常组合使用：

- **服务端状态**: React Query / SWR / RTK Query
- **客户端全局状态**: Zustand / Jotai / Pinia
- **表单状态**: React Hook Form / vee-validate
- **URL 状态**: nuqs / next-usequerystate

---

## 常见反模式

| 反模式 | 问题 | 正确做法 |
|--------|------|----------|
| 所有状态放全局 Store | 不必要的复杂度 | 优先 local state，仅共享状态入 store |
| Store 中存放派生值 | 数据不一致 | 使用 getter/selector/computed |
| 在 action 中做 UI 逻辑 | 关注点混乱 | Store 只管数据，UI 逻辑在组件 |
| 深嵌套 state 结构 | 更新困难 | 扁平化 + normalize |
| 不做状态分割 | 全量重渲染 | 按领域拆分 store/atom |
| 忽略序列化约束 | 持久化/调试失败 | Store 只存可序列化数据 |

---

## Agent Checklist

在 AI 编码流水线中进行状态管理选型和实现时，必须逐项检查：

- [ ] 明确区分服务端状态和客户端状态，分别使用合适工具
- [ ] 优先使用组件本地状态，仅在跨组件共享时提升到 store
- [ ] Store 结构扁平化，避免深层嵌套
- [ ] 派生值使用 selector / getter / computed，不手动维护副本
- [ ] 异步操作有 loading / error / success 三态处理
- [ ] TypeScript 类型完整覆盖 state 和 action
- [ ] 状态更新是不可变的（Redux/Zustand）或受控可变的（MobX/Pinia）
- [ ] 大型 store 按业务领域拆分为多个独立 store/slice
- [ ] 开发环境启用 DevTools 支持
- [ ] 需要持久化时使用官方 persist 中间件
- [ ] SSR 场景正确处理 hydration（避免客户端/服务端状态不一致）
- [ ] 不在 store 中存放非序列化对象（函数、DOM 引用、class 实例）
