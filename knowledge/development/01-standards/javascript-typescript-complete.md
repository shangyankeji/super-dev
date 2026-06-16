---
id: javascript-typescript-complete
title: JavaScript/TypeScript 完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, javascript, typescript, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# JavaScript/TypeScript 完整指南

## 概述
JavaScript是最流行的编程语言,TypeScript是其类型安全超集。本指南覆盖ES6+特性、TypeScript类型系统、异步编程、模块化和最佳实践。

## 核心概念

### 1. ES6+ 新特性

**箭头函数**:
```javascript
// 传统函数
function add(a, b) {
  return a + b;
}

// 箭头函数
const add = (a, b) => a + b;

// 带函数体
const greet = (name) => {
  const message = `Hello, ${name}!`;
  return message;
};

// 对象方法
const obj = {
  name: 'Alice',
  greet() {
    return `Hi, I'm ${this.name}`;
  }
};
```

**解构赋值**:
```javascript
// 数组解构
const [first, second, ...rest] = [1, 2, 3, 4, 5];
console.log(first);  // 1
console.log(second); // 2
console.log(rest);   // [3, 4, 5]

// 对象解构
const { name, age, ...others } = {
  name: 'Alice',
  age: 30,
  city: 'Beijing',
  country: 'China'
};
console.log(name);   // 'Alice'
console.log(others); // { city: 'Beijing', country: 'China' }

// 函数参数解构
function greet({ name, age }) {
  console.log(`${name} is ${age} years old`);
}
greet({ name: 'Bob', age: 25 });
```

**模板字符串**:
```javascript
const name = 'Alice';
const age = 30;

// 多行字符串
const html = `
  <div class="user">
    <h1>${name}</h1>
    <p>Age: ${age}</p>
  </div>
`;

// 标签模板
function highlight(strings, ...values) {
  return strings.reduce((result, str, i) => 
    `${result}${str}<mark>${values[i] || ''}</mark>`,
    ''
  );
}

const result = highlight`Hello, ${name}! You are ${age} years old.`;
console.log(result); // "Hello, <mark>Alice</mark>! You are <mark>30</mark> years old."
```

**类和继承**:
```javascript
class Animal {
  constructor(name) {
    this.name = name;
  }
  
  speak() {
    console.log(`${this.name} makes a sound`);
  }
}

class Dog extends Animal {
  constructor(name, breed) {
    super(name);
    this.breed = breed;
  }
  
  speak() {
    console.log(`${this.name} barks`);
  }
  
  fetch() {
    console.log(`${this.name} fetches the ball`);
  }
}

const dog = new Dog('Max', 'Labrador');
dog.speak(); // "Max barks"
dog.fetch(); // "Max fetches the ball"
```

### 2. TypeScript 类型系统

**基础类型**:
```typescript
// 基本类型
let name: string = 'Alice';
let age: number = 30;
let isActive: boolean = true;

// 数组
let numbers: number[] = [1, 2, 3];
let names: Array<string> = ['Alice', 'Bob'];

// 元组
let tuple: [string, number] = ['Alice', 30];

// 枚举
enum Color {
  Red,
  Green,
  Blue
}

let color: Color = Color.Red;

// Any和Unknown
let anything: any = 'anything';
let uncertain: unknown = 'maybe string or number';

// Void和Null
let nothing: void = undefined;
let absent: null = null;
```

**接口和类型别名**:
```typescript
// 接口
interface User {
  id: number;
  name: string;
  email: string;
  age?: number;  // 可选属性
  readonly createdAt: Date;  // 只读属性
}

// 类型别名
type ID = number | string;
type Point = { x: number; y: number };
type UserKeys = keyof User;  // 'id' | 'name' | 'email' | 'age' | 'createdAt'

// 使用
const user: User = {
  id: 1,
  name: 'Alice',
  email: 'alice@example.com',
  createdAt: new Date()
};

// 函数类型
type Callback = (data: string) => void;
type AsyncFunction = () => Promise<void>;
```

**泛型**:
```typescript
// 泛型函数
function identity<T>(arg: T): T {
  return arg;
}

const str = identity<string>('hello');
const num = identity(42);  // 类型推断

// 泛型接口
interface Container<T> {
  value: T;
  getValue(): T;
}

// 泛型类
class Box<T> {
  private value: T;
  
  constructor(value: T) {
    this.value = value;
  }
  
  getValue(): T {
    return this.value;
  }
  
  setValue(value: T): void {
    this.value = value;
  }
}

const stringBox = new Box<string>('hello');
console.log(stringBox.getValue());  // 'hello'

// 泛型约束
interface Lengthwise {
  length: number;
}

function logLength<T extends Lengthwise>(arg: T): T {
  console.log(arg.length);
  return arg;
}

logLength('hello');  // 5
logLength([1, 2, 3]);  // 3
// logLength(123);  // Error: number has no 'length' property
```

**高级类型**:
```typescript
// 联合类型
type ID = string | number;

// 交叉类型
type Employee = User & {
  employeeId: number;
  department: string;
};

// 字面量类型
type Direction = 'up' | 'down' | 'left' | 'right';
type Numeric = 1 | 2 | 3 | 4 | 5;

// 类型守卫
function isString(value: any): value is string {
  return typeof value === 'string';
}

function isUser(obj: any): obj is User {
  return typeof obj.id === 'number' && typeof obj.name === 'string';
}

// 使用类型守卫
function process(value: string | number) {
  if (isString(value)) {
    console.log(value.toUpperCase());  // TypeScript知道这里value是string
  } else {
    console.log(value.toFixed(2));  // TypeScript知道这里value是number
  }
}
```

### 3. 异步编程

**Promise**:
```javascript
// 创建Promise
const promise = new Promise((resolve, reject) => {
  setTimeout(() => {
    const success = true;
    if (success) {
      resolve('Operation completed');
    } else {
      reject(new Error('Operation failed'));
    }
  }, 1000);
});

// 使用Promise
promise
  .then(result => console.log(result))
  .catch(error => console.error(error))
  .finally(() => console.log('Cleanup'));

// Promise链
fetch('/api/user/1')
  .then(response => response.json())
  .then(user => fetch(`/api/posts?userId=${user.id}`))
  .then(response => response.json())
  .then(posts => console.log(posts))
  .catch(error => console.error(error));

// Promise.all
const promises = [
  fetch('/api/users'),
  fetch('/api/posts'),
  fetch('/api/comments')
];

Promise.all(promises)
  .then(responses => Promise.all(responses.map(r => r.json())))
  .then(data => console.log(data));

// Promise.race
Promise.race([
  fetch('/api/fast'),
  new Promise((_, reject) => 
    setTimeout(() => reject(new Error('Timeout')), 5000)
  )
])
  .then(response => console.log('First to complete'))
  .catch(error => console.error(error));
```

**Async/Await**:
```javascript
// async函数
async function fetchUser(id) {
  try {
    const response = await fetch(`/api/user/${id}`);
    const user = await response.json();
    return user;
  } catch (error) {
    console.error('Error fetching user:', error);
    throw error;
  }
}

// 并发执行
async function fetchAll() {
  const [users, posts, comments] = await Promise.all([
    fetch('/api/users').then(r => r.json()),
    fetch('/api/posts').then(r => r.json()),
    fetch('/api/comments').then(r => r.json())
  ]);
  
  console.log({ users, posts, comments });
}

// 异步迭代
async function processItems(items) {
  for (const item of items) {
    await processItem(item);  // 顺序执行
  }
}

// 并发处理
async function processItemsParallel(items) {
  await Promise.all(items.map(item => processItem(item)));
}

// 异步生成器
async function* asyncGenerator(count) {
  for (let i = 0; i < count; i++) {
    await sleep(100);
    yield i;
  }
}

// 使用异步生成器
async function useGenerator() {
  for await (const num of asyncGenerator(5)) {
    console.log(num);
  }
}
```

### 4. 模块化

**ES6模块**:
```javascript
// math.js
export const PI = 3.14159;

export function add(a, b) {
  return a + b;
}

export function subtract(a, b) {
  return a - b;
}

export default class Calculator {
  add(a, b) {
    return a + b;
  }
}

// main.js
import Calculator, { PI, add } from './math.js';

console.log(PI);  // 3.14159
console.log(add(1, 2));  // 3

const calc = new Calculator();
console.log(calc.add(5, 3));  // 8

// 动态导入
async function loadModule() {
  const module = await import('./heavy-module.js');
  module.doSomething();
}
```

**CommonJS**:
```javascript
// math.js
const PI = 3.14159;

function add(a, b) {
  return a + b;
}

module.exports = { PI, add };

// main.js
const { PI, add } = require('./math.js');

console.log(PI);  // 3.14159
console.log(add(1, 2));  // 3
```

## 最佳实践

### ✅ DO

1. **使用const和let,避免var**
```javascript
// ✅ 好
const PI = 3.14159;
let count = 0;

// ❌ 差
var x = 10;
```

2. **使用可选链和空值合并**
```typescript
// ✅ 好
const name = user?.profile?.name ?? 'Unknown';

// ❌ 差
const name = user && user.profile && user.profile.name 
  ? user.profile.name 
  : 'Unknown';
```

3. **使用TypeScript严格模式**
```json
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true
  }
}
```

### ❌ DON'T

1. **不要使用any除非必要**
```typescript
// ❌ 差
function process(data: any) {
  return data.value;
}

// ✅ 好
function process<T extends { value: unknown }>(data: T) {
  return data.value;
}
```

2. **不要忽略Promise错误**
```javascript
// ❌ 差
fetch('/api/data');  // 未处理错误

// ✅ 好
fetch('/api/data')
  .then(response => response.json())
  .catch(error => console.error(error));
```

## 学习路径

### 初级 (1-2周)
1. JavaScript基础语法
2. ES6+新特性
3. DOM操作

### 中级 (2-3周)
1. TypeScript类型系统
2. 异步编程
3. 模块化

### 高级 (2-4周)
1. 高级TypeScript特性
2. 设计模式
3. 性能优化

### 专家级 (持续)
1. TypeScript编译器API
2. JavaScript引擎优化
3. 库和框架开发

---

**知识ID**: `javascript-typescript-complete`  
**领域**: development  
**类型**: standards  
**难度**: intermediate  
**质量分**: 93  
**维护者**: dev-team@superdev.com  
**最后更新**: 2026-03-28
