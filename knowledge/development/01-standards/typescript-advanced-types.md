---
id: typescript-advanced-types
title: TypeScript 高级类型系统完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [advanced, development, types, typescript, 与框架集成, 内置工具类型, 基础类型系统回顾, 完整指南]
quality_score: 70
last_updated: 2026-06-15
---
# TypeScript 高级类型系统完整指南

## 概述
TypeScript 的类型系统是图灵完备的,远超简单的类型注解。掌握高级类型能力可以在编译期捕获更多错误、提升代码可维护性、实现精准的 API 契约。本指南面向已有 TypeScript 基础的开发者,系统梳理泛型、工具类型、高级模式、装饰器、模块系统、项目配置、框架集成、常见陷阱与性能优化。

## 基础类型系统回顾

### 原始类型与字面量类型

```typescript
// 原始类型
const name: string = "Alice";
const age: number = 30;
const active: boolean = true;
const id: bigint = 9007199254740991n;
const key: symbol = Symbol("key");

// 字面量类型 — 将值本身作为类型
type Direction = "north" | "south" | "east" | "west";
type HttpStatus = 200 | 301 | 404 | 500;
type Toggle = true | false;

// as const 将值收窄为字面量类型
const config = {
  endpoint: "https://api.example.com",
  retries: 3,
  debug: false,
} as const;
// typeof config.endpoint => "https://api.example.com"（字面量,非 string）
```

### 联合类型与交叉类型

```typescript
// 联合类型: 或
type Result<T> = T | Error;
type StringOrNumber = string | number;

// 交叉类型: 且
type WithTimestamps = {
  createdAt: Date;
  updatedAt: Date;
};
type User = { id: string; name: string } & WithTimestamps;

// 可辨识联合（Discriminated Union）
type Shape =
  | { kind: "circle"; radius: number }
  | { kind: "rectangle"; width: number; height: number }
  | { kind: "triangle"; base: number; height: number };

function area(shape: Shape): number {
  switch (shape.kind) {
    case "circle":
      return Math.PI * shape.radius ** 2;
    case "rectangle":
      return shape.width * shape.height;
    case "triangle":
      return (shape.base * shape.height) / 2;
  }
}
```

### 类型别名与接口

```typescript
// 接口 — 可声明合并,适合对外 API
interface ApiResponse<T> {
  data: T;
  status: number;
  message: string;
}

// 类型别名 — 支持联合/交叉/映射/条件,适合内部建模
type Nullable<T> = T | null;
type ReadonlyDeep<T> = {
  readonly [K in keyof T]: T[K] extends object ? ReadonlyDeep<T[K]> : T[K];
};

// 接口继承
interface BaseEntity {
  id: string;
  createdAt: Date;
}
interface UserEntity extends BaseEntity {
  email: string;
  role: "admin" | "user";
}

// 接口声明合并（Declaration Merging）
interface Window {
  __APP_CONFIG__: Record<string, unknown>;
}
```

## 泛型（Generics）

### 基础泛型

```typescript
// 泛型函数
function identity<T>(value: T): T {
  return value;
}
const str = identity("hello"); // 推导为 string
const num = identity(42);      // 推导为 number

// 泛型接口
interface Repository<T> {
  findById(id: string): Promise<T | null>;
  findAll(filter?: Partial<T>): Promise<T[]>;
  create(data: Omit<T, "id">): Promise<T>;
  update(id: string, data: Partial<T>): Promise<T>;
  delete(id: string): Promise<boolean>;
}

// 泛型类
class TypedEventEmitter<Events extends Record<string, unknown[]>> {
  private listeners = new Map<keyof Events, Set<Function>>();

  on<K extends keyof Events>(event: K, handler: (...args: Events[K]) => void) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(handler);
    return this;
  }

  emit<K extends keyof Events>(event: K, ...args: Events[K]) {
    this.listeners.get(event)?.forEach((handler) => handler(...args));
  }
}

// 使用
type AppEvents = {
  login: [userId: string, timestamp: number];
  logout: [userId: string];
  error: [error: Error, context: string];
};
const emitter = new TypedEventEmitter<AppEvents>();
emitter.on("login", (userId, timestamp) => { /* 完全类型安全 */ });
```

### 泛型约束（Constraints）

```typescript
// extends 约束
function getProperty<T, K extends keyof T>(obj: T, key: K): T[K] {
  return obj[key];
}

// 多重约束
interface Serializable {
  serialize(): string;
}
interface Loggable {
  log(): void;
}
function process<T extends Serializable & Loggable>(item: T): string {
  item.log();
  return item.serialize();
}

// 构造函数约束
type Constructor<T = {}> = new (...args: any[]) => T;

function Timestamped<TBase extends Constructor>(Base: TBase) {
  return class extends Base {
    createdAt = new Date();
    updatedAt = new Date();
  };
}

// 递归约束
type JSONValue =
  | string
  | number
  | boolean
  | null
  | JSONValue[]
  | { [key: string]: JSONValue };

function deepClone<T extends JSONValue>(value: T): T {
  return JSON.parse(JSON.stringify(value));
}
```

### 泛型默认值

```typescript
// 带默认类型参数的泛型
interface PaginatedResponse<T, M = { total: number; page: number }> {
  items: T[];
  meta: M;
}

// 使用默认值
type UserList = PaginatedResponse<User>;
// 覆盖默认值
type CursorUserList = PaginatedResponse<User, { cursor: string; hasMore: boolean }>;

// 带默认值的泛型工厂函数
function createStore<
  State extends Record<string, unknown> = Record<string, unknown>,
  Actions extends Record<string, Function> = Record<string, Function>
>(config: { state: State; actions: Actions }) {
  return config;
}
```

### 条件类型（Conditional Types）

```typescript
// 基本条件类型
type IsString<T> = T extends string ? true : false;
type A = IsString<"hello">;  // true
type B = IsString<42>;        // false

// 分布式条件类型 — 联合类型自动分配
type ToArray<T> = T extends unknown ? T[] : never;
type C = ToArray<string | number>; // string[] | number[]

// 阻止分布式行为
type ToArrayNonDist<T> = [T] extends [unknown] ? T[] : never;
type D = ToArrayNonDist<string | number>; // (string | number)[]

// 条件类型嵌套
type TypeName<T> =
  T extends string ? "string" :
  T extends number ? "number" :
  T extends boolean ? "boolean" :
  T extends undefined ? "undefined" :
  T extends Function ? "function" :
  "object";

// infer 关键字 — 在条件类型中提取子类型
type UnpackPromise<T> = T extends Promise<infer U> ? U : T;
type E = UnpackPromise<Promise<string>>; // string

type FunctionParams<T> = T extends (...args: infer P) => any ? P : never;
type F = FunctionParams<(a: string, b: number) => void>; // [a: string, b: number]

type FirstElement<T> = T extends [infer First, ...unknown[]] ? First : never;
type G = FirstElement<[string, number, boolean]>; // string

// 复杂 infer 示例: 提取嵌套路径
type ExtractRouteParams<T extends string> =
  T extends `${string}:${infer Param}/${infer Rest}`
    ? { [K in Param | keyof ExtractRouteParams<Rest>]: string }
    : T extends `${string}:${infer Param}`
      ? { [K in Param]: string }
      : {};

type Params = ExtractRouteParams<"/users/:userId/posts/:postId">;
// { userId: string; postId: string }
```

### 映射类型（Mapped Types）

```typescript
// 基本映射
type Optional<T> = {
  [K in keyof T]?: T[K];
};

// 添加/移除修饰符
type Mutable<T> = {
  -readonly [K in keyof T]: T[K];
};
type Required<T> = {
  [K in keyof T]-?: T[K];
};

// 键重映射（as 子句, TypeScript 4.1+）
type Getters<T> = {
  [K in keyof T as `get${Capitalize<string & K>}`]: () => T[K];
};
type Setters<T> = {
  [K in keyof T as `set${Capitalize<string & K>}`]: (value: T[K]) => void;
};

interface Person {
  name: string;
  age: number;
}
type PersonGetters = Getters<Person>;
// { getName: () => string; getAge: () => number }

// 过滤键
type FilterByType<T, U> = {
  [K in keyof T as T[K] extends U ? K : never]: T[K];
};
type StringProps = FilterByType<{ id: number; name: string; email: string }, string>;
// { name: string; email: string }

// 映射联合类型到对象
type EventHandlers<T extends string> = {
  [K in T as `on${Capitalize<K>}`]: (event: K) => void;
};
type DomHandlers = EventHandlers<"click" | "focus" | "blur">;
// { onClick: (event: "click") => void; onFocus: ...; onBlur: ... }
```

### 模板字面量类型（Template Literal Types）

```typescript
// 基础模板字面量
type Greeting = `Hello, ${string}!`;
const g: Greeting = "Hello, TypeScript!"; // OK

// 联合类型组合
type Color = "red" | "green" | "blue";
type Size = "small" | "medium" | "large";
type ColorSize = `${Color}-${Size}`;
// "red-small" | "red-medium" | "red-large" | "green-small" | ...

// 内置字符串操作类型
type Upper = Uppercase<"hello">;       // "HELLO"
type Lower = Lowercase<"HELLO">;       // "hello"
type Cap = Capitalize<"hello">;        // "Hello"
type Uncap = Uncapitalize<"Hello">;    // "hello"

// CSS 单位类型
type CSSUnit = "px" | "em" | "rem" | "vh" | "vw" | "%";
type CSSValue = `${number}${CSSUnit}`;
const width: CSSValue = "100px";    // OK
const height: CSSValue = "50vh";    // OK

// 深度路径类型
type PathOf<T, Prefix extends string = ""> = T extends object
  ? {
      [K in keyof T & string]: T[K] extends object
        ? `${Prefix}${K}` | PathOf<T[K], `${Prefix}${K}.`>
        : `${Prefix}${K}`;
    }[keyof T & string]
  : never;

interface Config {
  db: { host: string; port: number };
  cache: { ttl: number };
}
type ConfigPath = PathOf<Config>;
// "db" | "db.host" | "db.port" | "cache" | "cache.ttl"
```

## 内置工具类型

### 对象操作类

```typescript
// Partial<T> — 所有属性可选
interface UserUpdate {
  name: string;
  email: string;
  avatar: string;
}
function updateUser(id: string, updates: Partial<UserUpdate>) {
  // updates 的每个字段都是可选的
}

// Required<T> — 所有属性必填
interface Config {
  host?: string;
  port?: number;
  debug?: boolean;
}
function initServer(config: Required<Config>) {
  // config.host, config.port, config.debug 都必填
}

// Readonly<T> — 所有属性只读
function freeze<T extends object>(obj: T): Readonly<T> {
  return Object.freeze(obj);
}

// Record<K, V> — 构造键值对类型
type UserRoles = Record<string, "admin" | "editor" | "viewer">;
const roles: UserRoles = {
  alice: "admin",
  bob: "editor",
};

// 常见 Record 模式: 枚举键映射
type Status = "active" | "inactive" | "pending";
const statusLabels: Record<Status, string> = {
  active: "活跃",
  inactive: "未激活",
  pending: "待审核",
};
```

### 属性筛选类

```typescript
// Pick<T, K> — 选取部分属性
interface User {
  id: string;
  name: string;
  email: string;
  password: string;
  createdAt: Date;
}
type PublicUser = Pick<User, "id" | "name" | "email">;

// Omit<T, K> — 排除部分属性
type CreateUserDTO = Omit<User, "id" | "createdAt">;

// 组合使用
type UserProfile = Pick<User, "id" | "name"> & {
  avatar: string;
  bio?: string;
};
```

### 联合操作类

```typescript
// Exclude<T, U> — 从联合中排除
type AllEvents = "click" | "focus" | "blur" | "scroll" | "resize";
type UIEvents = Exclude<AllEvents, "scroll" | "resize">;
// "click" | "focus" | "blur"

// Extract<T, U> — 从联合中提取
type NumericTypes = Extract<string | number | boolean | bigint, number | bigint>;
// number | bigint

// NonNullable<T> — 排除 null 和 undefined
type MaybeString = string | null | undefined;
type DefiniteString = NonNullable<MaybeString>; // string
```

### 函数操作类

```typescript
// ReturnType<T> — 提取返回值类型
function createUser(name: string, email: string) {
  return { id: crypto.randomUUID(), name, email, createdAt: new Date() };
}
type CreatedUser = ReturnType<typeof createUser>;
// { id: string; name: string; email: string; createdAt: Date }

// Parameters<T> — 提取参数类型
type CreateUserParams = Parameters<typeof createUser>;
// [name: string, email: string]

// ConstructorParameters<T> — 提取构造函数参数
class HttpClient {
  constructor(baseURL: string, timeout: number, headers?: Record<string, string>) {}
}
type HttpClientArgs = ConstructorParameters<typeof HttpClient>;
// [baseURL: string, timeout: number, headers?: Record<string, string>]

// InstanceType<T> — 提取实例类型
type HttpClientInstance = InstanceType<typeof HttpClient>;

// Awaited<T> — 解包 Promise 类型（TypeScript 4.5+）
type ResolvedData = Awaited<Promise<Promise<string>>>; // string

async function fetchUsers(): Promise<User[]> {
  return [];
}
type FetchResult = Awaited<ReturnType<typeof fetchUsers>>; // User[]

// ThisParameterType / OmitThisParameter
function greet(this: { name: string }, greeting: string) {
  return `${greeting}, ${this.name}`;
}
type GreetThis = ThisParameterType<typeof greet>; // { name: string }
type GreetFn = OmitThisParameter<typeof greet>;   // (greeting: string) => string
```

## 高级模式

### 类型守卫（Type Guards）

```typescript
// typeof 守卫
function formatValue(value: string | number): string {
  if (typeof value === "string") {
    return value.toUpperCase();
  }
  return value.toFixed(2);
}

// instanceof 守卫
class ApiError extends Error {
  constructor(public statusCode: number, message: string) {
    super(message);
  }
}
function handleError(error: Error) {
  if (error instanceof ApiError) {
    console.log(`API Error ${error.statusCode}: ${error.message}`);
  }
}

// 自定义类型守卫（type predicate）
interface Cat { meow(): void; }
interface Dog { bark(): void; }
type Pet = Cat | Dog;

function isCat(pet: Pet): pet is Cat {
  return "meow" in pet;
}

function handlePet(pet: Pet) {
  if (isCat(pet)) {
    pet.meow(); // 类型收窄为 Cat
  } else {
    pet.bark(); // 类型收窄为 Dog
  }
}

// 断言函数（Assertion Functions, TypeScript 3.7+）
function assertDefined<T>(value: T | null | undefined, msg?: string): asserts value is T {
  if (value === null || value === undefined) {
    throw new Error(msg ?? "Value is not defined");
  }
}

function processUser(user: User | null) {
  assertDefined(user, "User must exist");
  console.log(user.name); // 此处 user 确定为 User
}

// 复合守卫
function isNonEmptyArray<T>(arr: T[]): arr is [T, ...T[]] {
  return arr.length > 0;
}
```

### 类型断言（Type Assertions）

```typescript
// as 断言 — 告诉编译器"我比你更清楚"
const input = document.getElementById("email") as HTMLInputElement;
input.value = "test@example.com";

// 双重断言（慎用,绕过类型检查）
const value = "hello" as unknown as number; // 强制转换

// const 断言 — 收窄为最精确的字面量类型
const routes = ["home", "about", "contact"] as const;
type Route = (typeof routes)[number]; // "home" | "about" | "contact"

// satisfies 操作符（TypeScript 4.9+）— 校验而不拓宽
type Colors = Record<string, string | string[]>;
const palette = {
  red: "#ff0000",
  green: "#00ff00",
  blue: ["#0000ff", "#0000cc"],
} satisfies Colors;
// palette.red 保持 string 类型（非 string | string[]）
// palette.blue 保持 string[] 类型,可以调用 .map()
```

### 类型推导与 infer

```typescript
// 提取数组元素类型
type ElementOf<T> = T extends readonly (infer E)[] ? E : never;
type Nums = ElementOf<number[]>; // number

// 提取函数最后一个参数
type LastParam<T extends (...args: any[]) => any> =
  T extends (...args: [...infer _, infer Last]) => any ? Last : never;
type Last = LastParam<(a: string, b: number, c: boolean) => void>; // boolean

// 提取 Promise 链
type DeepAwaited<T> = T extends Promise<infer U> ? DeepAwaited<U> : T;

// 提取对象值类型
type ValueOf<T> = T[keyof T];
type UserValues = ValueOf<User>; // string | Date

// 提取元组第一个和剩余
type Head<T extends readonly unknown[]> = T extends [infer H, ...unknown[]] ? H : never;
type Tail<T extends readonly unknown[]> = T extends [unknown, ...infer R] ? R : never;
```

### 递归类型

```typescript
// 深度只读
type DeepReadonly<T> = T extends Function
  ? T
  : T extends object
    ? { readonly [K in keyof T]: DeepReadonly<T[K]> }
    : T;

// 深度部分可选
type DeepPartial<T> = T extends Function
  ? T
  : T extends object
    ? { [K in keyof T]?: DeepPartial<T[K]> }
    : T;

// 递归展平数组
type Flatten<T extends readonly unknown[]> = T extends [infer First, ...infer Rest]
  ? First extends readonly unknown[]
    ? [...Flatten<First>, ...Flatten<Rest>]
    : [First, ...Flatten<Rest>]
  : [];
type Flat = Flatten<[1, [2, [3, 4]], 5]>; // [1, 2, 3, 4, 5]

// JSON 安全类型
type JSONSafe<T> = T extends Date
  ? string
  : T extends Function
    ? never
    : T extends object
      ? { [K in keyof T as JSONSafe<T[K]> extends never ? never : K]: JSONSafe<T[K]> }
      : T;
```

### 变体类型（Variance）

```typescript
// 协变（Covariance）— 子类型可替代父类型,用于返回值
type Producer<out T> = () => T;      // TypeScript 4.7+ 显式标注

// 逆变（Contravariance）— 父类型可替代子类型,用于参数
type Consumer<in T> = (value: T) => void;

// 不变（Invariance）— 既不协变也不逆变
type Processor<in out T> = (value: T) => T;

// 实际影响
interface Animal { name: string; }
interface Dog extends Animal { breed: string; }

type AnimalProducer = Producer<Animal>;
type DogProducer = Producer<Dog>;

// DogProducer 可赋值给 AnimalProducer（协变）
const dogFactory: DogProducer = () => ({ name: "Rex", breed: "Labrador" });
const animalFactory: AnimalProducer = dogFactory; // OK

type AnimalConsumer = Consumer<Animal>;
type DogConsumer = Consumer<Dog>;

// AnimalConsumer 可赋值给 DogConsumer（逆变）
const feedAnimal: AnimalConsumer = (animal) => console.log(animal.name);
const feedDog: DogConsumer = feedAnimal; // OK
```

## 装饰器与元数据

### TypeScript 5.x 标准装饰器

```typescript
// 类装饰器
function sealed(constructor: Function) {
  Object.seal(constructor);
  Object.seal(constructor.prototype);
}

// 方法装饰器（标准装饰器 API）
function log<This, Args extends any[], Return>(
  target: (this: This, ...args: Args) => Return,
  context: ClassMethodDecoratorContext<This, (this: This, ...args: Args) => Return>
) {
  const methodName = String(context.name);
  return function (this: This, ...args: Args): Return {
    console.log(`Calling ${methodName} with`, args);
    const result = target.call(this, ...args);
    console.log(`${methodName} returned`, result);
    return result;
  };
}

class Calculator {
  @log
  add(a: number, b: number): number {
    return a + b;
  }
}

// 属性装饰器 — 注入默认值
function defaultValue<T>(value: T) {
  return function <C>(
    _target: undefined,
    context: ClassFieldDecoratorContext<C, T>
  ) {
    return function (this: C, initialValue: T): T {
      return initialValue ?? value;
    };
  };
}

class Settings {
  @defaultValue(3000)
  accessor port: number = 0;

  @defaultValue("localhost")
  accessor host: string = "";
}

// 自动绑定装饰器
function bound<This, Args extends any[], Return>(
  target: (this: This, ...args: Args) => Return,
  context: ClassMethodDecoratorContext<This, (this: This, ...args: Args) => Return>
) {
  const methodName = context.name;
  context.addInitializer(function (this: This) {
    (this as any)[methodName] = (this as any)[methodName].bind(this);
  });
}
```

### 装饰器与依赖注入

```typescript
// 简易 DI 容器
const METADATA_KEY = Symbol("inject");

function Injectable() {
  return function <T extends Constructor>(target: T) {
    return target;
  };
}

function Inject(token: string) {
  return function (_target: undefined, context: ClassFieldDecoratorContext) {
    // 在运行时解析依赖
  };
}

@Injectable()
class UserService {
  @Inject("UserRepository")
  private repo!: UserRepository;

  async getUser(id: string) {
    return this.repo.findById(id);
  }
}
```

## 模块系统

### 命名空间（Namespace）

```typescript
// 命名空间 — 仅在特殊场景使用（全局脚本、声明文件）
namespace Validation {
  export interface Validator {
    validate(value: string): boolean;
  }

  export class EmailValidator implements Validator {
    validate(value: string): boolean {
      return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
    }
  }

  export class PhoneValidator implements Validator {
    validate(value: string): boolean {
      return /^\+?[\d\s-]{10,}$/.test(value);
    }
  }
}

// 现代项目应优先使用 ES Module,而非命名空间
```

### 声明合并（Declaration Merging）

```typescript
// 接口合并
interface Box {
  height: number;
  width: number;
}
interface Box {
  depth: number;
}
// 合并结果: { height: number; width: number; depth: number }

// 扩展第三方类型
declare module "express" {
  interface Request {
    user?: { id: string; role: string };
    requestId: string;
  }
}

// 扩展全局类型
declare global {
  interface Window {
    analytics: {
      track(event: string, props?: Record<string, unknown>): void;
    };
  }
}

// 枚举合并
enum Color {
  Red = 1,
  Green = 2,
}
enum Color {
  Blue = 3,
}
// Color.Red, Color.Green, Color.Blue 均可用
```

### 模块解析

```typescript
// 模块解析策略
// tsconfig.json 中 "moduleResolution": "bundler" | "node16" | "nodenext"

// 类型导入（Type-Only Imports, TypeScript 3.8+）
import type { User, UserRole } from "./types";
import { createUser, type CreateUserDTO } from "./user-service";

// 导入断言（Import Assertions）
import data from "./config.json" with { type: "json" };

// 动态导入
async function loadModule() {
  const { default: Chart } = await import("chart.js");
  return new Chart(/* ... */);
}

// 条件类型导出
export type { User } from "./user";
export { UserService } from "./user-service";
```

## 项目配置（tsconfig.json 完整指南）

### 编译选项核心配置

```jsonc
{
  "compilerOptions": {
    // ===== 语言与环境 =====
    "target": "ES2022",               // 编译目标
    "lib": ["ES2022", "DOM", "DOM.Iterable"], // 可用库声明
    "module": "ESNext",               // 模块系统
    "moduleResolution": "bundler",    // 模块解析策略(推荐 bundler)

    // ===== Strict 模式 =====
    "strict": true,                   // 启用所有严格检查(推荐)
    // strict 等价于以下全部开启:
    // "strictNullChecks": true,       // null/undefined 严格检查
    // "strictFunctionTypes": true,    // 函数参数逆变检查
    // "strictBindCallApply": true,    // bind/call/apply 严格类型
    // "strictPropertyInitialization": true, // 类属性必须初始化
    // "noImplicitAny": true,          // 禁止隐式 any
    // "noImplicitThis": true,         // 禁止隐式 this
    // "alwaysStrict": true,           // 每个文件添加 "use strict"
    // "useUnknownInCatchVariables": true, // catch(e) 中 e 为 unknown

    // ===== 额外检查 =====
    "noUncheckedIndexedAccess": true, // 索引签名返回 T | undefined
    "noUnusedLocals": true,           // 未使用的局部变量报错
    "noUnusedParameters": true,       // 未使用的参数报错
    "exactOptionalPropertyTypes": true, // 可选属性不允许赋值 undefined
    "noFallthroughCasesInSwitch": true, // switch 必须 break/return

    // ===== 输出 =====
    "outDir": "./dist",
    "declaration": true,               // 生成 .d.ts
    "declarationMap": true,            // 声明文件 source map
    "sourceMap": true,                 // JS source map
    "removeComments": false,           // 保留注释

    // ===== 互操作 =====
    "esModuleInterop": true,           // CommonJS/ESM 互操作
    "allowSyntheticDefaultImports": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,         // 允许导入 JSON
    "isolatedModules": true,           // 确保单文件可编译

    // ===== 路径映射 =====
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"],
      "@components/*": ["src/components/*"],
      "@utils/*": ["src/utils/*"],
      "@types/*": ["src/types/*"]
    }
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "**/*.test.ts"]
}
```

### 项目引用（Project References）

```jsonc
// tsconfig.json (根目录)
{
  "references": [
    { "path": "./packages/shared" },
    { "path": "./packages/client" },
    { "path": "./packages/server" }
  ],
  "files": []  // 根配置不编译文件
}

// packages/shared/tsconfig.json
{
  "compilerOptions": {
    "composite": true,        // 启用项目引用
    "outDir": "./dist",
    "declaration": true,
    "declarationMap": true,
    "rootDir": "./src"
  },
  "include": ["src/**/*"]
}

// packages/client/tsconfig.json
{
  "compilerOptions": {
    "composite": true,
    "outDir": "./dist"
  },
  "references": [
    { "path": "../shared" }   // 依赖 shared
  ]
}
```

### 不同场景的 tsconfig 模板

```jsonc
// React 项目 (Vite)
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noEmit": true,              // Vite 处理编译
    "isolatedModules": true,
    "skipLibCheck": true
  }
}

// Node.js 后端项目
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "outDir": "./dist",
    "declaration": true,
    "sourceMap": true,
    "esModuleInterop": true
  }
}

// 库项目
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "declaration": true,
    "declarationMap": true,
    "outDir": "./dist",
    "stripInternal": true        // 移除 @internal 标记的导出
  }
}
```

## 与框架集成

### React + TypeScript

```typescript
// 函数组件类型
interface ButtonProps {
  variant: "primary" | "secondary" | "danger";
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
  onClick?: (event: React.MouseEvent<HTMLButtonElement>) => void;
  children: React.ReactNode;
}

const Button: React.FC<ButtonProps> = ({ variant, size = "md", children, ...rest }) => {
  return <button className={`btn-${variant} btn-${size}`} {...rest}>{children}</button>;
};

// 泛型组件
interface ListProps<T> {
  items: T[];
  renderItem: (item: T, index: number) => React.ReactNode;
  keyExtractor: (item: T) => string;
  emptyState?: React.ReactNode;
}

function List<T>({ items, renderItem, keyExtractor, emptyState }: ListProps<T>) {
  if (items.length === 0) return <>{emptyState}</>;
  return (
    <ul>
      {items.map((item, index) => (
        <li key={keyExtractor(item)}>{renderItem(item, index)}</li>
      ))}
    </ul>
  );
}

// Hook 类型
function useLocalStorage<T>(key: string, initialValue: T) {
  const [stored, setStored] = React.useState<T>(() => {
    try {
      const item = window.localStorage.getItem(key);
      return item ? (JSON.parse(item) as T) : initialValue;
    } catch {
      return initialValue;
    }
  });

  const setValue = React.useCallback(
    (value: T | ((prev: T) => T)) => {
      setStored((prev) => {
        const next = value instanceof Function ? value(prev) : value;
        window.localStorage.setItem(key, JSON.stringify(next));
        return next;
      });
    },
    [key]
  );

  return [stored, setValue] as const;
}

// Ref 类型
const inputRef = React.useRef<HTMLInputElement>(null);
const timerRef = React.useRef<ReturnType<typeof setInterval>>();

// Context 类型
interface ThemeContextValue {
  theme: "light" | "dark";
  toggleTheme: () => void;
}
const ThemeContext = React.createContext<ThemeContextValue | null>(null);

function useTheme(): ThemeContextValue {
  const context = React.useContext(ThemeContext);
  if (!context) throw new Error("useTheme must be used within ThemeProvider");
  return context;
}
```

### Vue 3 + TypeScript

```typescript
// Composition API with TypeScript
import { defineComponent, ref, computed, PropType } from "vue";

interface Todo {
  id: number;
  text: string;
  done: boolean;
}

export default defineComponent({
  props: {
    items: {
      type: Array as PropType<Todo[]>,
      required: true,
    },
    filter: {
      type: String as PropType<"all" | "active" | "done">,
      default: "all",
    },
  },
  emits: {
    toggle: (id: number) => typeof id === "number",
    delete: (id: number) => typeof id === "number",
  },
  setup(props, { emit }) {
    const searchQuery = ref("");

    const filteredItems = computed(() => {
      let items = props.items;
      if (props.filter === "active") items = items.filter((t) => !t.done);
      if (props.filter === "done") items = items.filter((t) => t.done);
      if (searchQuery.value) {
        items = items.filter((t) =>
          t.text.toLowerCase().includes(searchQuery.value.toLowerCase())
        );
      }
      return items;
    });

    return { searchQuery, filteredItems };
  },
});

// <script setup> 写法（推荐）
// <script setup lang="ts">
// const props = defineProps<{ title: string; count?: number }>();
// const emit = defineEmits<{ update: [value: string]; close: [] }>();
// </script>
```

### Express + TypeScript

```typescript
import express, { Request, Response, NextFunction, RequestHandler } from "express";

// 类型安全的请求处理
interface CreateUserBody {
  name: string;
  email: string;
  role: "admin" | "user";
}

interface UserParams {
  id: string;
}

interface UserQuery {
  include?: string;
}

// 泛型请求处理器
type TypedRequestHandler<
  Params = {},
  ResBody = unknown,
  ReqBody = unknown,
  Query = {}
> = RequestHandler<Params, ResBody, ReqBody, Query>;

const createUser: TypedRequestHandler<{}, User, CreateUserBody> = async (req, res, next) => {
  try {
    const { name, email, role } = req.body; // 完全类型安全
    const user = await userService.create({ name, email, role });
    res.status(201).json(user);
  } catch (error) {
    next(error);
  }
};

const getUser: TypedRequestHandler<UserParams, User, {}, UserQuery> = async (req, res, next) => {
  const { id } = req.params;       // string 类型
  const { include } = req.query;   // string | undefined
  // ...
};

// 类型安全的中间件
interface AuthenticatedRequest extends Request {
  user: { id: string; role: string };
}

function requireAuth(req: Request, res: Response, next: NextFunction): void {
  const token = req.headers.authorization?.split(" ")[1];
  if (!token) {
    res.status(401).json({ error: "Unauthorized" });
    return;
  }
  (req as AuthenticatedRequest).user = verifyToken(token);
  next();
}

// 错误处理中间件
interface AppError extends Error {
  statusCode: number;
  code: string;
}

const errorHandler = (err: AppError, req: Request, res: Response, _next: NextFunction) => {
  res.status(err.statusCode || 500).json({
    error: err.code || "INTERNAL_ERROR",
    message: err.message,
  });
};
```

### Prisma + TypeScript

```typescript
import { PrismaClient, Prisma, User } from "@prisma/client";

const prisma = new PrismaClient();

// Prisma 自动生成的类型
type UserWithPosts = Prisma.UserGetPayload<{
  include: { posts: true };
}>;

type UserCreateInput = Prisma.UserCreateInput;

// 类型安全的查询构建
async function findUsers(params: {
  where?: Prisma.UserWhereInput;
  orderBy?: Prisma.UserOrderByWithRelationInput;
  take?: number;
  skip?: number;
}) {
  return prisma.user.findMany(params);
}

// 带事务的类型安全操作
async function transferCredits(fromId: string, toId: string, amount: number) {
  return prisma.$transaction(async (tx) => {
    const sender = await tx.user.update({
      where: { id: fromId },
      data: { credits: { decrement: amount } },
    });
    if (sender.credits < 0) {
      throw new Error("Insufficient credits");
    }
    const receiver = await tx.user.update({
      where: { id: toId },
      data: { credits: { increment: amount } },
    });
    return { sender, receiver };
  });
}

// 动态选择字段
function selectUserFields<T extends Prisma.UserSelect>(select: T) {
  return prisma.user.findMany({ select });
}
// 返回类型自动推导为选择的字段
```

## 常见陷阱

### any 滥用

```typescript
// 反模式: any 传染
function parseData(raw: any): any {
  return JSON.parse(raw); // 所有类型信息丢失
}

// 正确: 使用 unknown + 类型守卫
function parseData(raw: string): unknown {
  return JSON.parse(raw);
}

function isUser(data: unknown): data is User {
  return (
    typeof data === "object" &&
    data !== null &&
    "id" in data &&
    "name" in data &&
    typeof (data as User).id === "string"
  );
}

// 需要灵活类型时的替代方案
// 用 unknown 代替 any — 强制使用前检查
// 用 Record<string, unknown> 代替 any — 至少约束为对象
// 用泛型代替 any — 保留类型关联
```

### 类型体操过度

```typescript
// 反模式: 难以理解的深层类型嵌套
type DeepMerge<A, B> = {
  [K in keyof A | keyof B]: K extends keyof B
    ? K extends keyof A
      ? A[K] extends object
        ? B[K] extends object
          ? DeepMerge<A[K], B[K]>
          : B[K]
        : B[K]
      : B[K]
    : K extends keyof A
      ? A[K]
      : never;
};

// 建议: 超过 3 层嵌套条件类型时,拆解或加注释
// 如果团队无法理解类型定义,宁可放宽约束也不要写无人能维护的类型
// 可以使用 // @ts-expect-error + 运行时校验作为折中方案
```

### 枚举 vs 联合类型

```typescript
// 反模式: 过度使用枚举
enum Status {
  Active = "active",
  Inactive = "inactive",
  Pending = "pending",
}
// 枚举会生成运行时代码,tree-shaking 不友好

// 推荐: 字符串联合类型
type Status = "active" | "inactive" | "pending";

// 如果需要枚举行为,使用 as const 对象
const Status = {
  Active: "active",
  Inactive: "inactive",
  Pending: "pending",
} as const;
type Status = (typeof Status)[keyof typeof Status];
// 零运行时开销,完全可 tree-shake

// 枚举的合理使用场景: 位标志
enum Permission {
  Read = 1 << 0,    // 1
  Write = 1 << 1,   // 2
  Execute = 1 << 2, // 4
  Admin = Read | Write | Execute, // 7
}
```

### 类型窄化失败

```typescript
// 陷阱 1: 回调中的类型窄化丢失
function process(value: string | null) {
  if (value !== null) {
    // 此处 value 为 string
    setTimeout(() => {
      // 陷阱: TypeScript 无法保证回调执行时 value 仍非 null
      // 但实际上 const 变量在闭包中类型守卫仍然有效
      console.log(value.length); // OK — value 是 const
    }, 1000);
  }
}

// 陷阱 2: 解构后的类型窄化
type Response = { status: "ok"; data: string } | { status: "error"; message: string };

function handle(res: Response) {
  const { status } = res;
  if (status === "ok") {
    // 陷阱: res 已窄化,但 status 只是 string
    console.log(res.data); // OK — 通过 res 访问
  }
}

// 陷阱 3: 属性赋值后类型窄化丢失
interface Config {
  mode: "dev" | "prod";
  debug?: boolean;
}

function setup(config: Config) {
  if (config.mode === "dev") {
    config.debug = true;
    // 赋值后 config.mode 的窄化可能丢失
  }
}

// 解决: 用局部 const 变量保存窄化结果
function handleSafe(res: Response) {
  if (res.status === "ok") {
    const { data } = res; // 在窄化的分支内解构
    console.log(data);
  }
}
```

### 其他常见陷阱

```typescript
// 陷阱: 对象字面量的多余属性检查只在直接赋值时触发
interface Options { timeout: number; }
const opts = { timeout: 3000, retries: 5 };
const config: Options = opts; // 不报错! 多余属性检查被绕过

// 陷阱: 可选链与 nullish coalescing 的优先级
const value = obj?.nested?.value ?? "default"; // 正确
const wrong = obj?.nested?.value || "default"; // 空字符串、0、false 都被替换!

// 陷阱: readonly 只是浅层
interface Data {
  readonly items: string[];
}
const data: Data = { items: ["a"] };
// data.items = [];     // 报错: readonly
data.items.push("b");   // 不报错! 数组内容可变
// 解决: 使用 ReadonlyArray 或 readonly string[]
```

## 性能优化

### 类型检查加速

```typescript
// 1. 使用接口代替交叉类型（接口缓存更好）
// 慢
type SlowProps = BaseProps & StyleProps & EventProps;
// 快
interface FastProps extends BaseProps, StyleProps, EventProps {}

// 2. 避免深度条件类型递归
// 设置递归深度限制
type MaxDepth = 10;

// 3. 使用 type 而非 interface 做联合类型
type Result = Success | Failure; // 联合类型只能用 type

// 4. 减少模板字面量类型的组合爆炸
// 反模式: 排列组合导致类型数量爆炸
type Variant = `${Color}-${Size}-${State}-${Theme}`;
// 如果每个有 5 个值,结果有 625 个类型成员!

// 5. 善用 skipLibCheck 加速(不检查 node_modules 中的 .d.ts)
// tsconfig.json: "skipLibCheck": true
```

### 项目引用与增量编译

```bash
# 增量编译
# tsconfig.json 中添加:
# "incremental": true
# "tsBuildInfoFile": "./.tsbuildinfo"

# 项目引用构建
tsc --build                    # 构建所有引用项目
tsc --build --watch            # 监听模式
tsc --build --clean            # 清理构建产物
tsc --build --force            # 强制重建

# 使用 --build 模式时:
# - 只重新编译变更的项目
# - 自动解析依赖顺序
# - 生成 .tsbuildinfo 供增量使用
```

### 大型项目优化策略

```typescript
// 1. 拆分 tsconfig: 应用 / 测试 / 工具
// tsconfig.app.json — 仅包含源码
// tsconfig.test.json — 包含测试文件
// tsconfig.json — 总配置(references)

// 2. 使用 paths 减少相对路径深度
// 深层相对路径影响可读性,不影响编译性能
// 但 paths 可减少因路径错误导致的类型解析失败

// 3. 类型导入隔离
import type { HeavyType } from "./heavy-module";
// type-only import 不会触发模块执行
// 有助于 bundler 做 tree-shaking

// 4. 声明文件预编译
// 对于不常变化的内部库,预编译 .d.ts 避免重复检查
// 在 package.json 中设置 "types" 字段指向预编译的 .d.ts

// 5. 使用 @ts-check 渐进迁移
// 在 JS 文件顶部添加 // @ts-check
// 逐步享受类型检查,无需一次全量迁移
```

## Agent Checklist

以下检查项供 AI Agent 在 TypeScript 项目中执行代码审查与生成时使用:

### 类型安全
- [ ] 项目是否启用 `strict: true`
- [ ] 是否存在 `any` 类型（排查并替换为 `unknown` 或具体类型）
- [ ] 是否启用 `noUncheckedIndexedAccess`
- [ ] catch 块中 error 是否为 `unknown` 类型并在使用前校验
- [ ] 可选属性是否正确处理（不依赖 falsy 判断）

### 泛型与工具类型
- [ ] 泛型是否有适当约束（避免无约束 `<T>`）
- [ ] 是否优先使用内置工具类型而非手写等价物
- [ ] 条件类型嵌套是否超过 3 层（超过需拆解或加注释）
- [ ] 映射类型是否使用了 `as` 子句做键过滤/重映射

### 模式与实践
- [ ] 联合类型是否使用了可辨识字段（discriminated union）
- [ ] 类型守卫函数是否返回 `x is Type` 而非 `boolean`
- [ ] 是否使用 `satisfies` 操作符避免类型拓宽
- [ ] 是否使用 `as const` 代替手动字面量类型枚举
- [ ] 枚举是否可以替换为联合类型或 `as const` 对象

### 项目配置
- [ ] `moduleResolution` 是否与实际运行环境匹配
- [ ] 是否配置了路径映射（`paths`）减少相对路径嵌套
- [ ] 是否启用 `incremental` 编译加速构建
- [ ] 大型 monorepo 是否使用了项目引用（`composite` + `references`）
- [ ] 是否启用 `skipLibCheck` 加速类型检查

### 框架集成
- [ ] React 组件 props 是否定义为独立接口
- [ ] React Hook 返回值是否使用 `as const` 保留元组类型
- [ ] Express 中间件是否正确扩展 Request 类型
- [ ] Prisma 查询是否利用自动生成的类型而非手动定义

### 性能
- [ ] 是否使用 `import type` 隔离类型导入
- [ ] 模板字面量类型是否存在组合爆炸风险
- [ ] 是否优先使用 `interface extends` 代替交叉类型
- [ ] 递归类型是否设置了深度限制
- [ ] `.tsbuildinfo` 文件是否被 `.gitignore` 忽略
