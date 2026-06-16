---
id: typescript-enterprise-playbook
title: TypeScript 企业级实战手册
domain: backend
category: 02-playbooks
difficulty: advanced
tags: [typescript, ts, types, generics, error-handling, result-type, discriminated-union, branded-type, type-guard, enterprise]
quality_score: 93
maintainer: platform-team@superdev.com
last_updated: 2026-06-15
---

# TypeScript 企业级实战手册

> 基于 [Udacity: Handling errors like a pro](https://engineering.udacity.com/handling-errors-like-a-pro-in-typescript-d7a314ad4991) + [5 Commandments of Clean Error Handling](https://medium.com/with-orus/the-5-commandments-of-clean-error-handling-in-typescript-93a9cbdf1af5)

## 类型系统最佳实践

### 禁用 any，用 unknown + 收窄
```typescript
// ❌ any 放弃类型安全
function parse(data: any) {
  return data.user.name;  // 运行时崩溃？
}

// ✅ unknown + 类型守卫
function parse(data: unknown): string {
  if (typeof data === 'object' && data !== null && 'user' in data) {
    const user = (data as any).user;
    if (typeof user?.name === 'string') return user.name;
  }
  throw new Error('Invalid data');
}
```

### 品牌类型（Branded Types）防混淆
```typescript
// ❌ UserID 和 OrderID 都是 string，容易传混
type UserId = string;
type OrderId = string;
function getOrders(userId: UserId) { ... }
getOrders(orderId);  // 编译通过但逻辑错误！

// ✅ 品牌类型编译期区分
type UserId = string & { readonly __brand: 'UserId' };
type OrderId = string & { readonly __brand: 'OrderId' };
function getOrders(userId: UserId) { ... }
getOrders(orderId);  // 编译错误！类型不匹配
```

### 判别联合（Discriminated Unions）
```typescript
// ✅ 状态用判别联合，编译期穷尽检查
type ApiState =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: User[] }
  | { status: 'error'; error: string };

function render(state: ApiState) {
  switch (state.status) {
    case 'idle':    return <Placeholder />;
    case 'loading': return <Spinner />;
    case 'success': return <List items={state.data} />;
    case 'error':   return <Error msg={state.error} />;
    // 漏一个 case 编译就报错（穷尽检查）
  }
}
```

## 错误处理：Result 模式

```typescript
// ❌ throw 不可预测（调用者不知道会抛什么）
async function getUser(id: string): Promise<User> {
  const res = await fetch(`/api/users/${id}`);
  if (!res.ok) throw new Error('Failed');  // 调用者怎么知道？
  return res.json();
}

// ✅ Result 类型让错误显式（编译期知道所有可能结果）
type Result<T, E = string> =
  | { ok: true; value: T }
  | { ok: false; error: E };

async function getUser(id: string): Promise<Result<User, 'not_found' | 'network'>> {
  try {
    const res = await fetch(`/api/users/${id}`);
    if (res.status === 404) return { ok: false, error: 'not_found' };
    if (!res.ok) return { ok: false, error: 'network' };
    return { ok: true, value: await res.json() };
  } catch {
    return { ok: false, error: 'network' };
  }
}

// 调用者必须处理错误（编译期强制）
const result = await getUser('123');
if (!result.ok) {
  // result.error 是 'not_found' | 'network'
  console.log(result.error);
} else {
  // result.value 是 User
  console.log(result.value.name);
}
```

## 泛型约束

```typescript
// ❌ 无约束泛型（太宽松）
function getProperty<T>(obj: T, key: any): any {
  return (obj as any)[key];  // 类型安全丢失
}

// ✅ 泛型约束（编译期保证 key 存在）
function getProperty<T, K extends keyof T>(obj: T, key: K): T[K] {
  return obj[key];  // 类型安全，返回 T[K]
}

const user = { name: 'Alice', age: 30 };
const name = getProperty(user, 'name');  // string
const age = getProperty(user, 'age');    // number
// getProperty(user, 'email');  // 编译错误！
```

## 5 条错误处理戒律

1. **确保 Error 是真正的 Error**（不要 throw 字符串/数字）
2. **保留堆栈跟踪**（`Error.captureStackTrace` / `cause` 链）
3. **用常量错误码**（不用动态字符串做匹配）
4. **提供足够上下文**（错误含 input/userId/requestId）
5. **区分可恢复 vs 不可恢复**（4xx 可重试，5xx 告警）
