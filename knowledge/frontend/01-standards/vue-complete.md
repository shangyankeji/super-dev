---
id: vue-complete
title: Vue.js完整指南
domain: frontend
category: 01-standards
difficulty: intermediate
tags: [complete, frontend, vue, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Vue.js完整指南

## 概述
Vue.js是渐进式JavaScript框架,用于构建用户界面。易于上手,性能优秀,生态系统完善。本指南覆盖Vue 3组合式API、响应式系统、组件通信和最佳实践。

## 核心概念

### 1. 组合式API

**setup函数**:
```vue
<script setup>
import { ref, reactive, computed, watch, onMounted } from 'vue'

// 响应式引用
const count = ref(0)
const message = ref('Hello Vue!')

// 响应式对象
const user = reactive({
  name: 'Alice',
  age: 30
})

// 计算属性
const doubledCount = computed(() => count.value * 2)

// 方法
function increment() {
  count.value++
}

// 监听
watch(count, (newValue, oldValue) => {
  console.log(`Count changed from ${oldValue} to ${newValue}`)
})

// 生命周期
onMounted(() => {
  console.log('Component mounted')
})
</script>

<template>
  <div>
    <p>{{ message }}</p>
    <p>Count: {{ count }}</p>
    <p>Doubled: {{ doubledCount }}</p>
    <button @click="increment">Increment</button>
  </div>
</template>
```

### 2. 响应式系统

**ref vs reactive**:
```vue
<script setup>
import { ref, reactive, toRefs } from 'vue'

// ref: 用于基本类型和对象
const count = ref(0)
const user = ref({ name: 'Alice' })

console.log(count.value)  // 访问需要.value
console.log(user.value.name)

// reactive: 用于对象,无需.value
const state = reactive({
  count: 0,
  user: {
    name: 'Alice'
  }
})

console.log(state.count)  // 直接访问

// toRefs: 将reactive转为ref
const { count, user } = toRefs(state)
</script>
```

**computed和watch**:
```vue
<script setup>
import { ref, reactive, computed, watch, watchEffect } from 'vue'

const firstName = ref('Alice')
const lastName = ref('Smith')

// 计算属性(有缓存)
const fullName = computed(() => `${firstName.value} ${lastName.value}`)

// 可写计算属性
const fullNameWritable = computed({
  get() {
    return `${firstName.value} ${lastName.value}`
  },
  set(newValue) {
    [firstName.value, lastName.value] = newValue.split(' ')
  }
})

// watch: 监听特定源
watch(firstName, (newValue, oldValue) => {
  console.log(`First name changed: ${oldValue} -> ${newValue}`)
})

// 监听多个源
watch([firstName, lastName], ([newFirst, newLast], [oldFirst, oldLast]) => {
  console.log('Name changed')
})

// watchEffect: 自动追踪依赖
watchEffect(() => {
  console.log(`Full name is: ${firstName.value} ${lastName.value}`)
})
</script>
```

### 3. 组件通信

**Props和Emits**:
```vue
<!-- ParentComponent.vue -->
<script setup>
import ChildComponent from './ChildComponent.vue'

const message = 'Hello from parent'
const handleChildEvent = (data) => {
  console.log('Received from child:', data)
}
</script>

<template>
  <ChildComponent 
    :message="message"
    @child-event="handleChildEvent"
  />
</template>

<!-- ChildComponent.vue -->
<script setup>
const props = defineProps({
  message: {
    type: String,
    required: true,
    default: 'Default message'
  }
})

const emit = defineEmits(['child-event'])

const sendToParent = () => {
  emit('child-event', { data: 'Hello from child' })
}
</script>

<template>
  <div>
    <p>{{ message }}</p>
    <button @click="sendToParent">Send to Parent</button>
  </div>
</template>
```

**Provide/Inject**:
```vue
<!-- 祖先组件 -->
<script setup>
import { provide, ref } from 'vue'

const theme = ref('dark')
const updateTheme = (newTheme) => {
  theme.value = newTheme
}

provide('theme', theme)
provide('updateTheme', updateTheme)
</script>

<!-- 后代组件 -->
<script setup>
import { inject } from 'vue'

const theme = inject('theme')
const updateTheme = inject('updateTheme')
</script>

<template>
  <div :class="theme">
    <button @click="updateTheme('light')">Light</button>
  </div>
</template>
```

### 4. 生命周期

```vue
<script setup>
import {
  onBeforeMount,
  onMounted,
  onBeforeUpdate,
  onUpdated,
  onBeforeUnmount,
  onUnmounted
} from 'vue'

onBeforeMount(() => {
  console.log('Before mount')
})

onMounted(() => {
  console.log('Mounted')
})

onBeforeUpdate(() => {
  console.log('Before update')
})

onUpdated(() => {
  console.log('Updated')
})

onBeforeUnmount(() => {
  console.log('Before unmount')
})

onUnmounted(() => {
  console.log('Unmounted')
})
</script>
```

### 5. 路由(Vue Router)

```javascript
// router/index.js
import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import About from '../views/About.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home
  },
  {
    path: '/about',
    name: 'About',
    component: About
  },
  {
    path: '/user/:id',
    name: 'User',
    component: () => import('../views/User.vue'),  // 懒加载
    props: true  // 将路由参数作为props传递
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
```

```vue
<!-- 使用路由 -->
<script setup>
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()

const userId = computed(() => route.params.id)

const navigateToAbout = () => {
  router.push('/about')
}
</script>

<template>
  <div>
    <router-link to="/">Home</router-link>
    <router-link to="/about">About</router-link>
    <router-view />
  </div>
</template>
```

### 6. 状态管理(Pinia)

```javascript
// stores/user.js
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useUserStore = defineStore('user', () => {
  // State
  const user = ref(null)
  const isLoggedIn = computed(() => !!user.value)
  
  // Actions
  function login(userData) {
    user.value = userData
  }
  
  function logout() {
    user.value = null
  }
  
  return { user, isLoggedIn, login, logout }
})

// 使用
import { useUserStore } from '@/stores/user'

const userStore = useUserStore()

// 访问state
console.log(userStore.user)

// 访问getter
console.log(userStore.isLoggedIn)

// 调用action
userStore.login({ name: 'Alice' })
```

### 7. 表单处理

```vue
<script setup>
import { ref, reactive } from 'vue'
import { useForm, useField } from 'vee-validate'
import * as yup from 'yup'

// 定义验证规则
const schema = yup.object({
  email: yup.string().required().email(),
  password: yup.string().required().min(8)
})

// 使用vee-validate
const { handleSubmit } = useForm({
  validationSchema: schema
})

const { value: email, errorMessage: emailError } = useField('email')
const { value: password, errorMessage: passwordError } = useField('password')

const onSubmit = handleSubmit((values) => {
  console.log('Form submitted:', values)
})
</script>

<template>
  <form @submit="onSubmit">
    <div>
      <input v-model="email" type="email" placeholder="Email" />
      <p v-if="emailError" class="error">{{ emailError }}</p>
    </div>
    
    <div>
      <input v-model="password" type="password" placeholder="Password" />
      <p v-if="passwordError" class="error">{{ passwordError }}</p>
    </div>
    
    <button type="submit">Submit</button>
  </form>
</template>
```

## 最佳实践

### ✅ DO

1. **使用组合式API**
```vue
<!-- ✅ 好 -->
<script setup>
import { ref } from 'vue'
const count = ref(0)
</script>

<!-- ❌ 差(选项式API) -->
<script>
export default {
  data() {
    return {
      count: 0
    }
  }
}
</script>
```

2. **组件命名使用PascalCase**
```vue
<!-- ✅ 好 -->
<UserProfile />
<MyComponent />

<!-- ❌ 差 -->
<user-profile />
<my-component />
```

3. **使用scoped样式**
```vue
<template>
  <div class="container">...</div>
</template>

<style scoped>
.container {
  /* 只作用于当前组件 */
}
</style>
```

### ❌ DON'T

1. **不要直接修改props**
```vue
<script setup>
const props = defineProps(['modelValue'])

// ❌ 差
props.modelValue = 'new value'

// ✅ 好
const emit = defineEmits(['update:modelValue'])
emit('update:modelValue', 'new value')
</script>
```

2. **不要在模板中使用复杂表达式**
```vue
<!-- ❌ 差 -->
<p>{{ user.items.filter(i => i.active).map(i => i.name).join(', ') }}</p>

<!-- ✅ 好 -->
<script setup>
const activeItemNames = computed(() => 
  user.items
    .filter(i => i.active)
    .map(i => i.name)
    .join(', ')
)
</script>
<p>{{ activeItemNames }}</p>
```

## 学习路径

### 初级 (1-2周)
1. Vue基础语法
2. 组合式API
3. 响应式系统

### 中级 (2-3周)
1. 组件通信
2. 生命周期
3. 路由

### 高级 (2-4周)
1. 状态管理(Pinia)
2. 组合式函数
3. 性能优化

### 专家级 (持续)
1. 自定义渲染器
2. 插件开发
3. TypeScript集成

---

**知识ID**: `vue-complete`  
**领域**: frontend  
**类型**: standards  
**难度**: intermediate  
**质量分**: 94  
**维护者**: frontend-team@superdev.com  
**最后更新**: 2026-03-28
