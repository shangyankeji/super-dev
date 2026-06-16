---
id: vue3-complete
title: Vue 3 完整知识体系
domain: frontend
category: 01-standards
difficulty: intermediate
tags: [complete, frontend, pinia, router, vue3, 响应式系统原理, 性能优化, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Vue 3 完整知识体系

## 概述

Vue 3 是 Vue.js 的最新主版本，于 2022 年正式成为默认版本。它带来了组合式 API（Composition API）、基于 Proxy 的响应式系统、更好的 TypeScript 支持、Teleport/Suspense 等新内置组件，以及显著的性能提升。Vue 3 的设计目标是更小的包体积、更快的渲染速度和更好的可维护性。

### Vue 3 vs Vue 2 vs React

| 特性 | Vue 3 | Vue 2 | React 18 |
|------|-------|-------|----------|
| 响应式机制 | Proxy | Object.defineProperty | 不可变状态 + 调度器 |
| API 风格 | 组合式 API + 选项式 | 选项式 API | Hooks |
| 虚拟DOM | 编译时优化的 VDOM | 传统 VDOM | Fiber 架构 |
| TypeScript | 原生支持 | 需要额外配置 | 原生支持 |
| 包大小(min+gzip) | ~16KB | ~23KB | ~42KB(含ReactDOM) |
| 模板编译 | 编译时优化、静态提升 | 运行时编译 | JSX 转换 |
| 状态管理 | Pinia(官方) | Vuex | Redux/Zustand/Jotai |
| 路由 | Vue Router 4 | Vue Router 3 | React Router 6 |
| SSR/SSG | Nuxt 3 | Nuxt 2 | Next.js |
| 学习曲线 | 低-中 | 低 | 中 |
| 并发模式 | 无 | 无 | Concurrent Features |
| Fragment | 支持 | 不支持(需单根节点) | 支持 |

### 核心升级亮点

- **组合式 API**: 逻辑复用和代码组织的终极方案，替代 mixins
- **`<script setup>`**: 编译时语法糖，减少样板代码
- **Proxy 响应式**: 检测属性新增/删除、数组索引变化，不再需要 `Vue.set`
- **Tree-shaking 友好**: 按需引入，未使用的功能不会打包
- **多根节点组件(Fragment)**: 模板不再限制为单一根元素

---

## 组合式 API

### setup 与 `<script setup>`

`<script setup>` 是组合式 API 在单文件组件(SFC)中的编译时语法糖，是 Vue 3 推荐的写法：

```vue
<script setup lang="ts">
import { ref, reactive, computed, watch, watchEffect, onMounted } from 'vue'

// 所有顶层绑定自动暴露给模板
const title = ref('Vue 3 知识体系')
const count = ref(0)

function increment() {
  count.value++
}

onMounted(() => {
  console.log('组件已挂载')
})
</script>

<template>
  <h1>{{ title }}</h1>
  <p>计数: {{ count }}</p>
  <button @click="increment">+1</button>
</template>
```

对比传统 `setup()` 函数写法：

```vue
<script lang="ts">
import { defineComponent, ref } from 'vue'

export default defineComponent({
  setup() {
    const count = ref(0)
    const increment = () => { count.value++ }

    // 必须显式返回要暴露的内容
    return { count, increment }
  }
})
</script>
```

### ref 与 reactive

```vue
<script setup lang="ts">
import { ref, reactive, toRefs, toRef, isRef, unref } from 'vue'

// ref: 包装基本类型或对象，通过 .value 访问
const count = ref(0)
const message = ref('hello')
const user = ref({ name: 'Alice', age: 30 }) // 对象也可以用 ref

// reactive: 包装对象/数组，直接访问属性
const state = reactive({
  items: [] as string[],
  loading: false,
  error: null as string | null
})

// 修改 ref
count.value++
message.value = 'world'

// 修改 reactive（直接操作属性）
state.loading = true
state.items.push('new item')

// toRefs: 将 reactive 对象的每个属性转为 ref，保持响应式
const { items, loading } = toRefs(state)
// loading.value === state.loading

// toRef: 转换单个属性
const errorRef = toRef(state, 'error')

// 工具函数
console.log(isRef(count))   // true
console.log(unref(count))   // 0 (自动解包)
</script>
```

**ref vs reactive 选择指南**:
- 基本类型（string/number/boolean）只能用 `ref`
- 表单状态、复杂对象优先用 `reactive`
- 需要替换整个对象时用 `ref`（`reactive` 不能替换引用）
- composable 函数返回值推荐用 `ref`（解构不丢失响应式）

### computed

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'

const firstName = ref('张')
const lastName = ref('三')

// 只读计算属性
const fullName = computed(() => `${firstName.value}${lastName.value}`)

// 可写计算属性
const fullNameWritable = computed({
  get: () => `${firstName.value}${lastName.value}`,
  set: (val: string) => {
    firstName.value = val[0] || ''
    lastName.value = val.slice(1) || ''
  }
})

// 带调试的计算属性
const expensiveComputed = computed(() => {
  // 复杂计算
  return someHeavyCalculation()
}, {
  onTrack(e) { console.log('依赖被追踪', e) },
  onTrigger(e) { console.log('依赖变化触发重算', e) }
})
</script>
```

### watch 与 watchEffect

```vue
<script setup lang="ts">
import { ref, reactive, watch, watchEffect, watchPostEffect } from 'vue'

const keyword = ref('')
const page = ref(1)
const state = reactive({ filters: { category: 'all', sort: 'date' } })

// 监听单个 ref
watch(keyword, (newVal, oldVal) => {
  console.log(`关键词从 "${oldVal}" 变为 "${newVal}"`)
  fetchResults(newVal)
})

// 监听多个源
watch([keyword, page], ([newKeyword, newPage], [oldKeyword, oldPage]) => {
  fetchResults(newKeyword, newPage)
})

// 监听 reactive 对象的属性（需要用 getter）
watch(
  () => state.filters.category,
  (newCategory) => { console.log('分类变化:', newCategory) }
)

// 深度监听
watch(
  () => state.filters,
  (newFilters) => { console.log('筛选项变化:', newFilters) },
  { deep: true }
)

// 立即执行
watch(keyword, (val) => { fetchResults(val) }, { immediate: true })

// watchEffect: 自动收集依赖，立即执行
const stop = watchEffect((onCleanup) => {
  const controller = new AbortController()
  fetchData(keyword.value, { signal: controller.signal })

  onCleanup(() => {
    controller.abort() // 清理副作用
  })
})

// 停止监听
stop()

// watchPostEffect: DOM 更新后执行
watchPostEffect(() => {
  // 此时可以安全访问更新后的 DOM
  console.log(document.querySelector('#result')?.textContent)
})
</script>
```

### 生命周期钩子

```vue
<script setup lang="ts">
import {
  onBeforeMount,
  onMounted,
  onBeforeUpdate,
  onUpdated,
  onBeforeUnmount,
  onUnmounted,
  onActivated,
  onDeactivated,
  onErrorCaptured
} from 'vue'

onBeforeMount(() => { console.log('挂载前') })
onMounted(() => { console.log('挂载完成，DOM 可用') })
onBeforeUpdate(() => { console.log('更新前') })
onUpdated(() => { console.log('更新完成') })
onBeforeUnmount(() => { console.log('卸载前，清理定时器/事件') })
onUnmounted(() => { console.log('卸载完成') })

// KeepAlive 组件激活/停用
onActivated(() => { console.log('被 KeepAlive 激活') })
onDeactivated(() => { console.log('被 KeepAlive 停用') })

// 错误捕获
onErrorCaptured((err, instance, info) => {
  console.error('子组件错误:', err, info)
  return false // 阻止错误继续向上传播
})
</script>
```

### 组合式函数(Composables)

```typescript
// composables/useFetch.ts
import { ref, watchEffect, type Ref } from 'vue'

interface UseFetchReturn<T> {
  data: Ref<T | null>
  error: Ref<string | null>
  loading: Ref<boolean>
  refetch: () => Promise<void>
}

export function useFetch<T = any>(url: Ref<string> | string): UseFetchReturn<T> {
  const data = ref<T | null>(null) as Ref<T | null>
  const error = ref<string | null>(null)
  const loading = ref(false)

  async function fetchData() {
    loading.value = true
    error.value = null
    try {
      const response = await fetch(typeof url === 'string' ? url : url.value)
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      data.value = await response.json()
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  watchEffect(() => {
    fetchData()
  })

  return { data, error, loading, refetch: fetchData }
}

// composables/useDebounce.ts
import { ref, watch, type Ref } from 'vue'

export function useDebounce<T>(source: Ref<T>, delay = 300): Ref<T> {
  const debounced = ref(source.value) as Ref<T>
  let timer: ReturnType<typeof setTimeout>

  watch(source, (val) => {
    clearTimeout(timer)
    timer = setTimeout(() => { debounced.value = val }, delay)
  })

  return debounced
}

// composables/useLocalStorage.ts
import { ref, watch, type Ref } from 'vue'

export function useLocalStorage<T>(key: string, defaultValue: T): Ref<T> {
  const stored = localStorage.getItem(key)
  const data = ref<T>(stored ? JSON.parse(stored) : defaultValue) as Ref<T>

  watch(data, (val) => {
    localStorage.setItem(key, JSON.stringify(val))
  }, { deep: true })

  return data
}
```

使用组合式函数：

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useFetch } from '@/composables/useFetch'
import { useDebounce } from '@/composables/useDebounce'

const keyword = ref('')
const debouncedKeyword = useDebounce(keyword, 500)
const apiUrl = computed(() => `/api/search?q=${debouncedKeyword.value}`)
const { data, loading, error } = useFetch(apiUrl)
</script>

<template>
  <input v-model="keyword" placeholder="搜索..." />
  <div v-if="loading">加载中...</div>
  <div v-else-if="error">错误: {{ error }}</div>
  <ul v-else>
    <li v-for="item in data" :key="item.id">{{ item.name }}</li>
  </ul>
</template>
```

---

## 响应式系统原理

### Proxy 代理机制

Vue 3 的响应式系统基于 ES6 Proxy，替代了 Vue 2 的 `Object.defineProperty`：

```typescript
// 简化的响应式实现原理
function reactive<T extends object>(target: T): T {
  const handler: ProxyHandler<T> = {
    get(target, key, receiver) {
      const result = Reflect.get(target, key, receiver)
      track(target, key) // 依赖收集
      // 深层响应式：访问嵌套对象时递归代理
      if (typeof result === 'object' && result !== null) {
        return reactive(result)
      }
      return result
    },
    set(target, key, value, receiver) {
      const oldValue = Reflect.get(target, key, receiver)
      const result = Reflect.set(target, key, value, receiver)
      if (oldValue !== value) {
        trigger(target, key) // 触发更新
      }
      return result
    },
    deleteProperty(target, key) {
      const result = Reflect.deleteProperty(target, key)
      trigger(target, key) // 删除属性也能检测
      return result
    }
  }
  return new Proxy(target, handler)
}
```

### 依赖收集与触发更新

```typescript
// 简化的依赖收集系统
type Dep = Set<ReactiveEffect>
type KeyToDepMap = Map<string | symbol, Dep>
const targetMap = new WeakMap<object, KeyToDepMap>()

let activeEffect: ReactiveEffect | null = null

function track(target: object, key: string | symbol) {
  if (!activeEffect) return
  let depsMap = targetMap.get(target)
  if (!depsMap) {
    targetMap.set(target, (depsMap = new Map()))
  }
  let dep = depsMap.get(key)
  if (!dep) {
    depsMap.set(key, (dep = new Set()))
  }
  dep.add(activeEffect) // 将当前 effect 加入依赖集合
}

function trigger(target: object, key: string | symbol) {
  const depsMap = targetMap.get(target)
  if (!depsMap) return
  const dep = depsMap.get(key)
  if (dep) {
    dep.forEach(effect => {
      // 调度器：异步批量更新
      if (effect.scheduler) {
        effect.scheduler()
      } else {
        effect.run()
      }
    })
  }
}
```

### ref 的实现原理

```typescript
// ref 本质是带 value 属性的对象
class RefImpl<T> {
  private _value: T
  private _rawValue: T
  public dep: Set<ReactiveEffect> = new Set()
  public readonly __v_isRef = true

  constructor(value: T) {
    this._rawValue = value
    this._value = isObject(value) ? reactive(value) : value
  }

  get value() {
    trackRefValue(this) // 收集依赖
    return this._value
  }

  set value(newVal: T) {
    if (hasChanged(newVal, this._rawValue)) {
      this._rawValue = newVal
      this._value = isObject(newVal) ? reactive(newVal) : newVal
      triggerRefValue(this) // 触发更新
    }
  }
}
```

---

## 组件设计

### Props 定义与验证

```vue
<script setup lang="ts">
// 类型声明方式（推荐）
interface Props {
  title: string
  count?: number
  items: string[]
  status: 'active' | 'inactive' | 'pending'
  callback?: (id: number) => void
}

const props = withDefaults(defineProps<Props>(), {
  count: 0,
  status: 'active'
})

// 使用 props
console.log(props.title, props.count)
</script>
```

运行时验证写法（适用于需要自定义验证器的场景）：

```vue
<script setup>
const props = defineProps({
  age: {
    type: Number,
    required: true,
    validator: (value) => value >= 0 && value <= 150
  },
  email: {
    type: String,
    default: '',
    validator: (value) => !value || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)
  }
})
</script>
```

### Emits 事件声明

```vue
<script setup lang="ts">
// 类型声明方式
const emit = defineEmits<{
  (e: 'update', id: number, value: string): void
  (e: 'delete', id: number): void
  (e: 'submit'): void
}>()

// Vue 3.3+ 简写语法
const emit = defineEmits<{
  update: [id: number, value: string]
  delete: [id: number]
  submit: []
}>()

function handleSave(id: number) {
  emit('update', id, 'new value')
}
</script>
```

### Slots 插槽

```vue
<!-- BaseCard.vue -->
<template>
  <div class="card">
    <!-- 默认插槽 -->
    <div class="card-body">
      <slot />
    </div>

    <!-- 具名插槽 -->
    <div class="card-header" v-if="$slots.header">
      <slot name="header" />
    </div>

    <!-- 作用域插槽：向父组件传递数据 -->
    <div class="card-list">
      <slot name="item" v-for="item in items" :key="item.id"
        :item="item" :index="items.indexOf(item)" />
    </div>
  </div>
</template>

<script setup lang="ts">
interface Item { id: number; name: string }
defineProps<{ items: Item[] }>()
</script>
```

使用插槽：

```vue
<template>
  <BaseCard :items="products">
    <template #header>
      <h2>产品列表</h2>
    </template>

    <template #item="{ item, index }">
      <div class="product-row">
        {{ index + 1 }}. {{ item.name }}
      </div>
    </template>

    <!-- 默认插槽 -->
    <p>底部描述信息</p>
  </BaseCard>
</template>
```

### Provide / Inject 依赖注入

```typescript
// types/injection-keys.ts
import type { InjectionKey, Ref } from 'vue'

export interface UserContext {
  user: Ref<{ name: string; role: string } | null>
  login: (name: string) => void
  logout: () => void
}

export const UserKey: InjectionKey<UserContext> = Symbol('user')
```

```vue
<!-- 祖先组件 provide -->
<script setup lang="ts">
import { ref, provide } from 'vue'
import { UserKey, type UserContext } from '@/types/injection-keys'

const user = ref<{ name: string; role: string } | null>(null)

const userContext: UserContext = {
  user,
  login: (name: string) => { user.value = { name, role: 'user' } },
  logout: () => { user.value = null }
}

provide(UserKey, userContext)
</script>
```

```vue
<!-- 后代组件 inject -->
<script setup lang="ts">
import { inject } from 'vue'
import { UserKey } from '@/types/injection-keys'

const userCtx = inject(UserKey)
if (!userCtx) throw new Error('UserContext 未提供')

const { user, login, logout } = userCtx
</script>

<template>
  <div v-if="user">
    欢迎, {{ user.name }}
    <button @click="logout">退出</button>
  </div>
  <button v-else @click="login('张三')">登录</button>
</template>
```

### Teleport

```vue
<template>
  <button @click="showModal = true">打开弹窗</button>

  <!-- 将内容传送到 body 下，脱离组件 DOM 层级 -->
  <Teleport to="body">
    <div v-if="showModal" class="modal-overlay" @click.self="showModal = false">
      <div class="modal-content">
        <h2>弹窗标题</h2>
        <p>弹窗内容，渲染在 body 下但逻辑仍属于当前组件</p>
        <button @click="showModal = false">关闭</button>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue'
const showModal = ref(false)
</script>
```

### Suspense（实验性）

```vue
<template>
  <Suspense>
    <!-- 默认插槽：异步组件 -->
    <template #default>
      <AsyncDashboard />
    </template>

    <!-- 后备插槽：加载状态 -->
    <template #fallback>
      <div class="loading-skeleton">
        <div class="skeleton-header" />
        <div class="skeleton-body" />
      </div>
    </template>
  </Suspense>
</template>

<script setup>
import { defineAsyncComponent } from 'vue'

const AsyncDashboard = defineAsyncComponent(() =>
  import('./components/Dashboard.vue')
)
</script>
```

异步 setup 组件（配合 Suspense 使用）：

```vue
<!-- Dashboard.vue -->
<script setup lang="ts">
// 顶层 await 使组件成为异步组件
const response = await fetch('/api/dashboard')
const dashboardData = await response.json()
</script>

<template>
  <div>{{ dashboardData.title }}</div>
</template>
```

---

## Vue Router 4

### 基础配置

```typescript
// router/index.ts
import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Home',
    component: () => import('@/views/Home.vue')
  },
  {
    path: '/users',
    name: 'Users',
    component: () => import('@/views/Users.vue'),
    meta: { requiresAuth: true, title: '用户管理' }
  },
  {
    // 动态路由参数
    path: '/users/:id',
    name: 'UserDetail',
    component: () => import('@/views/UserDetail.vue'),
    props: true // 将路由参数作为 props 传入组件
  },
  {
    // 嵌套路由
    path: '/settings',
    component: () => import('@/layouts/SettingsLayout.vue'),
    children: [
      { path: '', name: 'SettingsGeneral', component: () => import('@/views/settings/General.vue') },
      { path: 'profile', name: 'SettingsProfile', component: () => import('@/views/settings/Profile.vue') },
      { path: 'security', name: 'SettingsSecurity', component: () => import('@/views/settings/Security.vue') }
    ]
  },
  {
    // 捕获所有未匹配路由
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    component: () => import('@/views/NotFound.vue')
  }
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
  scrollBehavior(to, from, savedPosition) {
    if (savedPosition) return savedPosition
    if (to.hash) return { el: to.hash, behavior: 'smooth' }
    return { top: 0 }
  }
})

export default router
```

### 路由守卫

```typescript
// 全局前置守卫
router.beforeEach(async (to, from) => {
  const authStore = useAuthStore()

  // 设置页面标题
  document.title = (to.meta.title as string) || '默认标题'

  // 认证检查
  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    return { name: 'Login', query: { redirect: to.fullPath } }
  }

  // 权限检查
  if (to.meta.requiredRole) {
    const hasRole = authStore.user?.roles.includes(to.meta.requiredRole as string)
    if (!hasRole) return { name: 'Forbidden' }
  }
})

// 全局后置钩子
router.afterEach((to, from) => {
  // 发送页面访问统计
  analytics.trackPageView(to.fullPath)
})
```

组件内守卫：

```vue
<script setup lang="ts">
import { onBeforeRouteLeave, onBeforeRouteUpdate } from 'vue-router'

const hasUnsavedChanges = ref(false)

// 离开路由前确认
onBeforeRouteLeave((to, from) => {
  if (hasUnsavedChanges.value) {
    const answer = window.confirm('有未保存的更改，确定离开吗？')
    if (!answer) return false
  }
})

// 路由参数变化时（如 /users/1 -> /users/2）
onBeforeRouteUpdate(async (to, from) => {
  const userId = to.params.id as string
  await fetchUser(userId)
})
</script>
```

### 动态路由

```typescript
// 运行时添加路由（权限路由场景）
async function initDynamicRoutes() {
  const authStore = useAuthStore()
  const menus = await fetchUserMenus(authStore.user!.id)

  menus.forEach(menu => {
    router.addRoute({
      path: menu.path,
      name: menu.name,
      component: () => import(`@/views/${menu.component}.vue`),
      meta: { title: menu.title, icon: menu.icon }
    })
  })
}

// 移除路由
const removeRoute = router.addRoute({ path: '/temp', component: TempView })
removeRoute() // 调用返回值移除

// 检查路由是否存在
router.hasRoute('UserDetail')
```

### 路由元信息与类型扩展

```typescript
// types/router.d.ts
import 'vue-router'

declare module 'vue-router' {
  interface RouteMeta {
    requiresAuth?: boolean
    requiredRole?: string
    title?: string
    icon?: string
    keepAlive?: boolean
    transition?: string
  }
}
```

---

## Pinia 状态管理

### Store 定义

```typescript
// stores/user.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

// 组合式 API 风格（推荐）
export const useUserStore = defineStore('user', () => {
  // state
  const user = ref<{ id: number; name: string; role: string } | null>(null)
  const token = ref<string | null>(localStorage.getItem('token'))
  const permissions = ref<string[]>([])

  // getters
  const isAuthenticated = computed(() => !!token.value)
  const isAdmin = computed(() => user.value?.role === 'admin')
  const displayName = computed(() => user.value?.name || '游客')

  // actions
  async function login(username: string, password: string) {
    const res = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password })
    })
    if (!res.ok) throw new Error('登录失败')
    const data = await res.json()
    token.value = data.token
    user.value = data.user
    permissions.value = data.permissions
    localStorage.setItem('token', data.token)
  }

  function logout() {
    token.value = null
    user.value = null
    permissions.value = []
    localStorage.removeItem('token')
  }

  function hasPermission(perm: string): boolean {
    return permissions.value.includes(perm)
  }

  return {
    user, token, permissions,
    isAuthenticated, isAdmin, displayName,
    login, logout, hasPermission
  }
})
```

选项式 API 风格：

```typescript
// stores/counter.ts
export const useCounterStore = defineStore('counter', {
  state: () => ({
    count: 0,
    history: [] as number[]
  }),
  getters: {
    doubleCount: (state) => state.count * 2,
    lastThreeHistory: (state) => state.history.slice(-3)
  },
  actions: {
    increment() {
      this.count++
      this.history.push(this.count)
    },
    async incrementAsync() {
      await new Promise(resolve => setTimeout(resolve, 1000))
      this.increment()
    }
  }
})
```

### Store 间交互

```typescript
// stores/cart.ts
import { defineStore } from 'pinia'
import { useUserStore } from './user'

export const useCartStore = defineStore('cart', () => {
  const items = ref<CartItem[]>([])
  const userStore = useUserStore()

  const total = computed(() =>
    items.value.reduce((sum, item) => sum + item.price * item.quantity, 0)
  )

  // VIP 用户打折
  const finalTotal = computed(() =>
    userStore.isAdmin ? total.value * 0.9 : total.value
  )

  return { items, total, finalTotal }
})
```

### Pinia 插件

```typescript
// plugins/pinia-logger.ts
import type { PiniaPluginContext } from 'pinia'

export function piniaLogger({ store }: PiniaPluginContext) {
  store.$onAction(({ name, args, after, onError }) => {
    const startTime = Date.now()
    console.log(`[Store:${store.$id}] Action "${name}" 开始`, args)

    after((result) => {
      console.log(`[Store:${store.$id}] Action "${name}" 完成 (${Date.now() - startTime}ms)`, result)
    })

    onError((error) => {
      console.error(`[Store:${store.$id}] Action "${name}" 失败`, error)
    })
  })
}

// 持久化插件
export function piniaPersist({ store }: PiniaPluginContext) {
  const savedState = localStorage.getItem(`pinia-${store.$id}`)
  if (savedState) {
    store.$patch(JSON.parse(savedState))
  }

  store.$subscribe((mutation, state) => {
    localStorage.setItem(`pinia-${store.$id}`, JSON.stringify(state))
  })
}

// main.ts 注册
import { createPinia } from 'pinia'
const pinia = createPinia()
pinia.use(piniaLogger)
pinia.use(piniaPersist)
app.use(pinia)
```

### 在组件中使用 Store

```vue
<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useUserStore } from '@/stores/user'

const userStore = useUserStore()

// storeToRefs 保持响应式（只解构 state 和 getters）
const { user, isAuthenticated, displayName } = storeToRefs(userStore)

// actions 直接解构（不需要 storeToRefs）
const { login, logout } = userStore

// 订阅状态变化
userStore.$subscribe((mutation, state) => {
  console.log('状态变化:', mutation.type, mutation.storeId)
})
</script>
```

---

## 性能优化

### 虚拟列表

```vue
<!-- VirtualList.vue -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

interface Props {
  items: any[]
  itemHeight: number
  containerHeight: number
  overscan?: number
}

const props = withDefaults(defineProps<Props>(), { overscan: 5 })

const scrollTop = ref(0)
const containerRef = ref<HTMLDivElement>()

const totalHeight = computed(() => props.items.length * props.itemHeight)

const startIndex = computed(() =>
  Math.max(0, Math.floor(scrollTop.value / props.itemHeight) - props.overscan)
)

const endIndex = computed(() =>
  Math.min(
    props.items.length,
    Math.ceil((scrollTop.value + props.containerHeight) / props.itemHeight) + props.overscan
  )
)

const visibleItems = computed(() =>
  props.items.slice(startIndex.value, endIndex.value).map((item, i) => ({
    ...item,
    _index: startIndex.value + i,
    _style: {
      position: 'absolute' as const,
      top: `${(startIndex.value + i) * props.itemHeight}px`,
      height: `${props.itemHeight}px`,
      width: '100%'
    }
  }))
)

function onScroll(e: Event) {
  scrollTop.value = (e.target as HTMLDivElement).scrollTop
}
</script>

<template>
  <div
    ref="containerRef"
    :style="{ height: containerHeight + 'px', overflow: 'auto', position: 'relative' }"
    @scroll="onScroll"
  >
    <div :style="{ height: totalHeight + 'px', position: 'relative' }">
      <div v-for="item in visibleItems" :key="item._index" :style="item._style">
        <slot :item="item" :index="item._index" />
      </div>
    </div>
  </div>
</template>
```

### KeepAlive 缓存

```vue
<template>
  <router-view v-slot="{ Component, route }">
    <Transition :name="route.meta.transition || 'fade'" mode="out-in">
      <KeepAlive :include="cachedRoutes" :max="10">
        <component :is="Component" :key="route.fullPath" />
      </KeepAlive>
    </Transition>
  </router-view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

const cachedRoutes = computed(() =>
  router.getRoutes()
    .filter(route => route.meta.keepAlive)
    .map(route => route.name as string)
)
</script>
```

### 代码分割与懒加载

```typescript
// 路由级代码分割
const routes = [
  {
    path: '/dashboard',
    component: () => import(/* webpackChunkName: "dashboard" */ '@/views/Dashboard.vue')
  }
]

// 组件级懒加载
import { defineAsyncComponent } from 'vue'

const HeavyChart = defineAsyncComponent({
  loader: () => import('@/components/HeavyChart.vue'),
  loadingComponent: LoadingSpinner,
  errorComponent: ErrorDisplay,
  delay: 200,    // 延迟显示 loading（避免闪烁）
  timeout: 10000 // 超时时间
})

// 条件懒加载
const AdminPanel = defineAsyncComponent(() =>
  userStore.isAdmin
    ? import('@/components/AdminPanel.vue')
    : import('@/components/AccessDenied.vue')
)
```

### v-memo 优化

```vue
<template>
  <!-- v-memo: 仅当依赖值变化时重新渲染 -->
  <div v-for="item in largeList" :key="item.id" v-memo="[item.id, item.selected]">
    <span>{{ item.name }}</span>
    <span :class="{ active: item.selected }">{{ item.status }}</span>
    <!-- 只有 item.id 或 item.selected 变化才重新渲染此节点 -->
  </div>
</template>
```

### 其他优化技巧

```vue
<script setup lang="ts">
import { shallowRef, shallowReactive, triggerRef, markRaw } from 'vue'

// shallowRef: 只追踪 .value 的变化，不深层响应
const hugeList = shallowRef<Item[]>([])
hugeList.value = [...newData] // 触发更新
// hugeList.value[0].name = 'x' // 不会触发更新
triggerRef(hugeList) // 手动触发更新

// shallowReactive: 只追踪顶层属性
const state = shallowReactive({
  nested: { count: 0 } // nested.count 变化不会触发更新
})

// markRaw: 标记对象永不转为响应式（如第三方库实例）
const map = markRaw(new Map())
const chart = markRaw(echarts.init(el))
</script>
```

**Tree-shaking 最佳实践**:
- 使用 `import { ref, computed } from 'vue'` 按需导入，不要 `import Vue from 'vue'`
- Vite 默认支持 Tree-shaking，确保依赖提供 ESM 格式
- 使用 `rollup-plugin-visualizer` 分析打包产物

---

## 测试

### Vitest 单元测试

```typescript
// __tests__/composables/useFetch.spec.ts
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref, nextTick } from 'vue'
import { useFetch } from '@/composables/useFetch'

// mock fetch
global.fetch = vi.fn()

describe('useFetch', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('应当成功获取数据', async () => {
    const mockData = [{ id: 1, name: 'Item 1' }]
    ;(fetch as any).mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockData)
    })

    const { data, loading, error } = useFetch('/api/items')

    expect(loading.value).toBe(true)
    await nextTick()
    await vi.waitFor(() => expect(loading.value).toBe(false))

    expect(data.value).toEqual(mockData)
    expect(error.value).toBeNull()
  })

  it('应当处理请求错误', async () => {
    ;(fetch as any).mockResolvedValueOnce({ ok: false, status: 404 })

    const { data, error } = useFetch('/api/not-found')
    await vi.waitFor(() => expect(error.value).toBeTruthy())

    expect(data.value).toBeNull()
    expect(error.value).toContain('404')
  })

  it('应当在 URL 变化时重新请求', async () => {
    ;(fetch as any).mockResolvedValue({
      ok: true,
      json: () => Promise.resolve([])
    })

    const url = ref('/api/items?page=1')
    useFetch(url)

    await nextTick()
    expect(fetch).toHaveBeenCalledTimes(1)

    url.value = '/api/items?page=2'
    await nextTick()
    expect(fetch).toHaveBeenCalledTimes(2)
  })
})
```

### Vue Test Utils 组件测试

```typescript
// __tests__/components/TodoList.spec.ts
import { describe, it, expect, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createTestingPinia } from '@pinia/testing'
import { nextTick } from 'vue'
import TodoList from '@/components/TodoList.vue'
import { useTodoStore } from '@/stores/todo'

describe('TodoList', () => {
  function createWrapper(options = {}) {
    return mount(TodoList, {
      global: {
        plugins: [
          createTestingPinia({
            createSpy: vi.fn,
            initialState: {
              todo: {
                items: [
                  { id: 1, text: '学习 Vue 3', done: false },
                  { id: 2, text: '写测试', done: true }
                ]
              }
            }
          })
        ]
      },
      ...options
    })
  }

  it('渲染待办列表', () => {
    const wrapper = createWrapper()
    const items = wrapper.findAll('[data-testid="todo-item"]')
    expect(items).toHaveLength(2)
    expect(items[0].text()).toContain('学习 Vue 3')
  })

  it('添加新待办', async () => {
    const wrapper = createWrapper()
    const store = useTodoStore()

    const input = wrapper.find('input[data-testid="new-todo"]')
    await input.setValue('新任务')
    await wrapper.find('form').trigger('submit')

    expect(store.addTodo).toHaveBeenCalledWith('新任务')
  })

  it('切换完成状态', async () => {
    const wrapper = createWrapper()
    const store = useTodoStore()

    const checkbox = wrapper.find('[data-testid="todo-checkbox-1"]')
    await checkbox.trigger('change')

    expect(store.toggleTodo).toHaveBeenCalledWith(1)
  })

  it('显示空状态提示', () => {
    const wrapper = mount(TodoList, {
      global: {
        plugins: [
          createTestingPinia({
            initialState: { todo: { items: [] } }
          })
        ]
      }
    })

    expect(wrapper.find('[data-testid="empty-state"]').exists()).toBe(true)
    expect(wrapper.text()).toContain('暂无待办事项')
  })
})
```

### E2E 测试（Playwright）

```typescript
// e2e/todo.spec.ts
import { test, expect } from '@playwright/test'

test.describe('待办应用', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('完整的待办流程', async ({ page }) => {
    // 添加待办
    await page.fill('[data-testid="new-todo"]', '学习 Playwright')
    await page.press('[data-testid="new-todo"]', 'Enter')
    await expect(page.locator('[data-testid="todo-item"]')).toHaveCount(1)

    // 标记完成
    await page.click('[data-testid="todo-checkbox-1"]')
    await expect(page.locator('[data-testid="todo-item-1"]')).toHaveClass(/completed/)

    // 筛选已完成
    await page.click('[data-testid="filter-completed"]')
    await expect(page.locator('[data-testid="todo-item"]')).toHaveCount(1)

    // 删除
    await page.hover('[data-testid="todo-item-1"]')
    await page.click('[data-testid="delete-1"]')
    await expect(page.locator('[data-testid="empty-state"]')).toBeVisible()
  })
})
```

---

## TypeScript 集成

### 组件类型

```typescript
// types/components.ts
import type { Component, DefineComponent } from 'vue'

// 全局组件类型声明
declare module 'vue' {
  interface GlobalComponents {
    BaseButton: typeof import('@/components/BaseButton.vue')['default']
    BaseInput: typeof import('@/components/BaseInput.vue')['default']
    BaseModal: typeof import('@/components/BaseModal.vue')['default']
  }
}
```

### defineExpose 类型

```vue
<!-- ChildForm.vue -->
<script setup lang="ts">
import { ref } from 'vue'

const formData = ref({ name: '', email: '' })

function validate(): boolean {
  return formData.value.name.length > 0
}

function reset() {
  formData.value = { name: '', email: '' }
}

defineExpose({ validate, reset })
</script>
```

```vue
<!-- ParentView.vue -->
<script setup lang="ts">
import { ref } from 'vue'
import ChildForm from './ChildForm.vue'

const formRef = ref<InstanceType<typeof ChildForm>>()

function handleSubmit() {
  if (formRef.value?.validate()) {
    // 提交表单
  }
}
</script>

<template>
  <ChildForm ref="formRef" />
  <button @click="handleSubmit">提交</button>
</template>
```

### 泛型组件（Vue 3.3+）

```vue
<!-- GenericList.vue -->
<script setup lang="ts" generic="T extends { id: number }">
defineProps<{
  items: T[]
  selected?: T
}>()

defineEmits<{
  select: [item: T]
}>()
</script>

<template>
  <ul>
    <li
      v-for="item in items"
      :key="item.id"
      :class="{ active: selected?.id === item.id }"
      @click="$emit('select', item)"
    >
      <slot :item="item" />
    </li>
  </ul>
</template>
```

使用泛型组件：

```vue
<script setup lang="ts">
interface User { id: number; name: string; email: string }
const users = ref<User[]>([])
const selectedUser = ref<User>()
</script>

<template>
  <!-- TypeScript 会推断 item 为 User 类型 -->
  <GenericList :items="users" :selected="selectedUser" @select="selectedUser = $event">
    <template #default="{ item }">
      {{ item.name }} - {{ item.email }}
    </template>
  </GenericList>
</template>
```

### tsconfig.json 推荐配置

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "esModuleInterop": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "noEmit": true,
    "paths": {
      "@/*": ["./src/*"]
    },
    "types": ["vite/client", "vitest/globals"]
  },
  "include": ["src/**/*.ts", "src/**/*.vue", "src/**/*.d.ts"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

---

## SSR / SSG（Nuxt 3）

### 项目初始化与目录结构

```bash
npx nuxi@latest init my-nuxt-app
cd my-nuxt-app && npm install
```

```
my-nuxt-app/
├── app.vue            # 应用入口
├── nuxt.config.ts     # Nuxt 配置
├── pages/             # 文件系统路由
│   ├── index.vue      # /
│   ├── about.vue      # /about
│   └── users/
│       ├── index.vue  # /users
│       └── [id].vue   # /users/:id
├── components/        # 自动导入的组件
├── composables/       # 自动导入的组合式函数
├── server/            # 服务端 API
│   ├── api/
│   │   └── users.ts   # /api/users
│   └── middleware/
├── layouts/           # 布局组件
├── middleware/        # 路由中间件
├── plugins/           # 插件
└── public/            # 静态资源
```

### 数据获取

```vue
<!-- pages/users/[id].vue -->
<script setup lang="ts">
const route = useRoute()

// useFetch: 自动处理 SSR hydration，避免重复请求
const { data: user, pending, error } = await useFetch(
  `/api/users/${route.params.id}`,
  {
    key: `user-${route.params.id}`,
    transform: (data) => ({
      ...data,
      fullName: `${data.firstName} ${data.lastName}`
    })
  }
)

// useAsyncData: 更灵活的数据获取
const { data: posts } = await useAsyncData(
  `user-posts-${route.params.id}`,
  () => $fetch(`/api/users/${route.params.id}/posts`)
)

// 仅客户端获取（不参与 SSR）
const { data: analytics } = await useFetch('/api/analytics', {
  server: false,
  lazy: true
})
</script>

<template>
  <div v-if="pending">加载中...</div>
  <div v-else-if="error">加载失败: {{ error.message }}</div>
  <div v-else>
    <h1>{{ user?.fullName }}</h1>
    <article v-for="post in posts" :key="post.id">
      {{ post.title }}
    </article>
  </div>
</template>
```

### Server API

```typescript
// server/api/users/[id].get.ts
import { defineEventHandler, getRouterParam, createError } from 'h3'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')

  const user = await prisma.user.findUnique({ where: { id: Number(id) } })
  if (!user) {
    throw createError({ statusCode: 404, statusMessage: '用户不存在' })
  }

  return user
})

// server/api/users/index.post.ts
export default defineEventHandler(async (event) => {
  const body = await readBody(event)

  // 验证
  if (!body.name || !body.email) {
    throw createError({ statusCode: 400, statusMessage: '缺少必填字段' })
  }

  return await prisma.user.create({ data: body })
})
```

### Nuxt 配置

```typescript
// nuxt.config.ts
export default defineNuxtConfig({
  devtools: { enabled: true },

  // SSR / SSG 模式切换
  ssr: true,
  // 预渲染指定路由（SSG）
  routeRules: {
    '/': { prerender: true },
    '/about': { prerender: true },
    '/api/**': { cors: true, headers: { 'cache-control': 's-maxage=600' } },
    '/dashboard/**': { ssr: false } // 仅客户端渲染
  },

  // 模块
  modules: [
    '@pinia/nuxt',
    '@nuxtjs/tailwindcss',
    '@vueuse/nuxt',
    '@nuxt/image'
  ],

  // 运行时配置
  runtimeConfig: {
    dbUrl: process.env.DATABASE_URL,   // 仅服务端可用
    public: {
      apiBase: process.env.API_BASE || '/api'  // 客户端可用
    }
  },

  // Vite 配置
  vite: {
    css: {
      preprocessorOptions: {
        scss: { additionalData: '@use "@/assets/scss/variables" as *;' }
      }
    }
  },

  // Nitro 服务引擎
  nitro: {
    preset: 'node-server', // 或 'vercel', 'cloudflare', 'netlify'
    compressPublicAssets: true
  }
})
```

---

## 常见陷阱与反模式

### 1. 响应式丢失

```typescript
// 错误：解构 reactive 对象会丢失响应式
const state = reactive({ count: 0, name: 'Vue' })
let { count } = state // count 是普通变量，失去响应式

// 正确：使用 toRefs
const { count, name } = toRefs(state)
// 或直接使用 ref
const count = ref(0)
```

### 2. ref 忘记 .value

```typescript
// 错误：在 script 中忘记 .value
const count = ref(0)
console.log(count) // RefImpl 对象，不是 0
if (count) { /* 永远为 true，因为 ref 对象是 truthy */ }

// 正确
console.log(count.value) // 0
if (count.value) { /* 正确判断 */ }

// 注意：模板中自动解包，不需要 .value
// <template>{{ count }}</template>  正确
```

### 3. reactive 整体替换

```typescript
// 错误：替换整个 reactive 对象会断开响应式连接
let state = reactive({ items: [] })
state = reactive({ items: [1, 2, 3] }) // 原有引用丢失

// 正确：修改属性而非替换引用
const state = reactive({ items: [] as number[] })
state.items = [1, 2, 3] // 修改属性，响应式保持

// 或者使用 ref
const state = ref({ items: [] as number[] })
state.value = { items: [1, 2, 3] } // ref 可以替换整个值
```

### 4. watch 的陷阱

```typescript
// 错误：直接监听 reactive 对象的属性值
const state = reactive({ count: 0 })
watch(state.count, (val) => { /* 不生效 */ })

// 正确：使用 getter 函数
watch(() => state.count, (val) => { /* 生效 */ })

// 错误：监听 ref 时加了 .value
const count = ref(0)
watch(count.value, (val) => { /* 不生效，因为传入的是原始值 0 */ })

// 正确：直接传 ref
watch(count, (val) => { /* 正确 */ })
```

### 5. 异步操作中的响应式

```typescript
// 错误：异步回调中 this 指向或作用域问题
async function fetchData() {
  const data = ref<any>(null)
  // 如果组件在请求完成前卸载，这里可能导致内存泄漏
  const res = await fetch('/api/data')
  data.value = await res.json()
  return data
}

// 正确：配合 watchEffect + 清理
const data = ref<any>(null)
watchEffect(async (onCleanup) => {
  const controller = new AbortController()
  onCleanup(() => controller.abort())

  try {
    const res = await fetch('/api/data', { signal: controller.signal })
    data.value = await res.json()
  } catch (e) {
    if ((e as Error).name !== 'AbortError') throw e
  }
})
```

### 6. v-for 与 v-if 优先级

```vue
<!-- 错误：Vue 3 中 v-if 优先级高于 v-for，无法访问 v-for 的变量 -->
<li v-for="item in items" v-if="item.active" :key="item.id">
  {{ item.name }}
</li>

<!-- 正确：使用 computed 过滤或用 template 包裹 -->
<li v-for="item in activeItems" :key="item.id">
  {{ item.name }}
</li>

<!-- 或 -->
<template v-for="item in items" :key="item.id">
  <li v-if="item.active">{{ item.name }}</li>
</template>
```

### 7. 组件注册陷阱

```vue
<script setup>
// 错误：在 <script setup> 中用 app.component 全局注册
// 没有导入就使用组件（会在运行时报错）

// 正确：直接 import 即可，<script setup> 自动注册
import MyComponent from './MyComponent.vue'
// 模板中直接使用 <MyComponent />
</script>
```

### 8. Props 修改

```vue
<script setup lang="ts">
const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

// 错误：直接修改 prop
// props.modelValue = 'new value' // Vue 会警告

// 正确：通过 emit 通知父组件
function updateValue(val: string) {
  emit('update:modelValue', val)
}

// 或使用 computed 代理（适用于 v-model 场景）
const localValue = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})
</script>

<template>
  <input v-model="localValue" />
</template>
```

### 9. 内存泄漏

```vue
<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'

// 错误：不清理副作用
onMounted(() => {
  window.addEventListener('resize', handleResize)
  setInterval(pollData, 5000)
})

// 正确：在 onUnmounted 中清理
let intervalId: ReturnType<typeof setInterval>

onMounted(() => {
  window.addEventListener('resize', handleResize)
  intervalId = setInterval(pollData, 5000)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  clearInterval(intervalId)
})
</script>
```

### 10. 过度使用全局状态

```typescript
// 反模式：所有数据都放 Pinia store
// 只有跨组件共享的状态才需要放 store

// 正确分层：
// - 组件局部状态 → ref/reactive
// - 父子通信 → props/emits
// - 跨层级共享 → provide/inject
// - 全局共享 → Pinia store
```

---

## 项目脚手架与工程化

### Vite 项目创建

```bash
npm create vue@latest my-vue-app
# 选择: TypeScript, Vue Router, Pinia, Vitest, ESLint, Prettier

cd my-vue-app
npm install
npm run dev
```

### 推荐项目结构

```
src/
├── assets/            # 静态资源（图片、字体、全局样式）
├── components/        # 通用组件
│   ├── base/          # 基础 UI 组件（Button, Input, Modal）
│   └── business/      # 业务组件
├── composables/       # 组合式函数
├── layouts/           # 布局组件
├── pages/ (或 views/) # 页面组件
├── router/            # 路由配置
├── stores/            # Pinia stores
├── types/             # TypeScript 类型定义
├── utils/             # 工具函数
├── api/               # API 请求封装
├── plugins/           # Vue 插件
├── directives/        # 自定义指令
├── App.vue
└── main.ts
```

### ESLint + Prettier 配置

```javascript
// eslint.config.js (Flat Config)
import pluginVue from 'eslint-plugin-vue'
import vueTsEslintConfig from '@vue/eslint-config-typescript'
import pluginVitest from '@vitest/eslint-plugin'
import prettierConfig from '@vue/eslint-config-prettier'

export default [
  { name: 'app/files-to-lint', files: ['**/*.{ts,mts,tsx,vue}'] },
  { name: 'app/files-to-ignore', ignores: ['**/dist/**', '**/coverage/**'] },
  ...pluginVue.configs['flat/recommended'],
  ...vueTsEslintConfig(),
  { ...pluginVitest.configs.recommended, files: ['src/**/__tests__/*'] },
  prettierConfig
]
```

---

## 学习路径

### 入门 (1-2 周)
1. 模板语法、指令（v-bind/v-on/v-model/v-for/v-if）
2. 组件基础（props/emits/slots）
3. 响应式基础（ref/reactive/computed）
4. 生命周期钩子
5. 事件处理与表单绑定

### 进阶 (2-4 周)
1. 组合式 API 深入（watch/watchEffect/composables）
2. Vue Router 4（动态路由/守卫/嵌套路由）
3. Pinia 状态管理
4. TypeScript 集成
5. 组件设计模式（provide/inject/Teleport/Suspense）

### 高级 (1-2 月)
1. 响应式系统原理（Proxy/依赖收集/调度器）
2. 虚拟 DOM 与编译优化
3. 性能优化（虚拟列表/KeepAlive/v-memo/shallowRef）
4. SSR/SSG（Nuxt 3）
5. 自定义渲染器与高级插件开发

### 专家 (持续)
1. Vue 编译器源码
2. 自定义 Vite 插件
3. 大规模应用架构（微前端/Monorepo）
4. 设计系统构建

## 参考资源

### 官方文档
- [Vue 3 官方文档](https://cn.vuejs.org/)
- [Vue Router](https://router.vuejs.org/zh/)
- [Pinia](https://pinia.vuejs.org/zh/)
- [Nuxt 3](https://nuxt.com/)
- [Vite](https://cn.vitejs.dev/)

### 推荐资源
- [VueUse](https://vueuse.org/) - 组合式工具集合
- [Vue Macros](https://vue-macros.dev/) - 编译时宏扩展
- [Vitest](https://vitest.dev/) - Vite 原生测试框架

---

## Agent Checklist

使用本知识文件时，Agent 应确认以下要点：

- [ ] 项目使用 Vue 3 + `<script setup>` + TypeScript 的推荐写法
- [ ] 响应式数据选择正确（ref 用于基本类型/需要替换的对象，reactive 用于复杂对象）
- [ ] computed 用于派生状态，避免在模板中写复杂表达式
- [ ] watch/watchEffect 正确清理副作用（onCleanup）
- [ ] composable 函数命名以 `use` 前缀开头，返回 ref 而非 reactive
- [ ] Props 使用 TypeScript 接口定义，设置合理的默认值
- [ ] 组件事件使用 defineEmits 类型声明
- [ ] 路由使用懒加载（`() => import(...)`）
- [ ] 路由守卫中正确处理认证和权限逻辑
- [ ] Pinia store 使用组合式 API 风格，通过 storeToRefs 解构
- [ ] 大列表使用虚拟列表或分页，避免一次渲染数千 DOM 节点
- [ ] 合理使用 KeepAlive 缓存频繁切换的组件
- [ ] 使用 shallowRef/shallowReactive 优化大数据结构
- [ ] 第三方库实例使用 markRaw 避免不必要的响应式代理
- [ ] v-for 始终提供稳定的 key
- [ ] 不直接修改 props，通过 emit 或 computed 代理
- [ ] 组件卸载时清理所有副作用（事件监听/定时器/AbortController）
- [ ] Nuxt 3 项目使用 useFetch/useAsyncData 获取数据，利用 SSR hydration
- [ ] ESLint + Prettier 配置就绪，CI 中强制检查
- [ ] 单元测试覆盖核心 composables 和业务组件

---

**知识ID**: `vue3-complete`
**版本**: v1.0
**领域**: frontend
**类型**: standards
**难度**: intermediate
**质量分**: 95
**维护者**: frontend-team@superdev.com
**最后更新**: 2026-03-28
