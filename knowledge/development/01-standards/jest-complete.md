---
id: jest-complete
title: Jest测试框架完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, jest, react测试, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Jest测试框架完整指南

## 概述
Jest是JavaScript测试框架,专注于简单性和性能。本指南覆盖Jest配置、匹配器、模拟和最佳实践。

## 核心概念

### 1. 基础测试

**简单测试**:
```javascript
// math.js
function add(a, b) {
  return a + b;
}

module.exports = { add };

// math.test.js
const { add } = require('./math');

test('adds 1 + 2 to equal 3', () => {
  expect(add(1, 2)).toBe(3);
});

test('adds -1 + 1 to equal 0', () => {
  expect(add(-1, 1)).toBe(0);
});
```

### 2. 匹配器(Matchers)

**常用匹配器**:
```javascript
test('common matchers', () => {
  const value = 2 + 2;
  
  // 相等
  expect(value).toBe(4);
  expect(value).toEqual(4);
  
  // 真值
  expect(value).toBeTruthy();
  expect(0).toBeFalsy();
  expect(null).toBeNull();
  expect(undefined).toBeUndefined();
  
  // 数字
  expect(value).toBeGreaterThan(3);
  expect(value).toBeLessThan(5);
  
  // 字符串
  expect('hello').toMatch(/hello/);
  expect('hello world').toContain('world');
  
  // 数组
  expect([1, 2, 3]).toContain(2);
  expect([1, 2, 3]).toHaveLength(3);
});
```

### 3. 异步测试

```javascript
// async.js
function fetchData() {
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve({ data: 'hello' });
    }, 100);
  });
}

// async.test.js
test('async data', async () => {
  const data = await fetchData();
  expect(data).toEqual({ data: 'hello' });
});

test('async with resolves', () => {
  return expect(fetchData()).resolves.toEqual({ data: 'hello' });
});
```

### 4. Mock和Spy

**Mock函数**:
```javascript
// user.js
const fetchUser = () => {
  return fetch('/api/user');
};

// user.test.js
test('mock fetch', async () => {
  global.fetch = jest.fn(() =>
    Promise.resolve({
      json: () => Promise.resolve({ name: 'Alice' })
    })
  );
  
  const user = await fetchUser();
  expect(user.name).toBe('Alice');
  expect(fetch).toHaveBeenCalledTimes(1);
});
```

**SpyOn对象**:
```javascript
const video = {
  play() {
    return 'playing';
  }
};

test('spy on method', () => {
  const spy = jest.spyOn(video, 'play');
  video.play();
  
  expect(spy).toHaveBeenCalled();
  expect(spy).toHaveReturnedWith('playing');
});
```

### 5. 测试前后钩子

```javascript
let database;

beforeAll(() => {
  // 所有测试前执行一次
  database = connectDatabase();
});

afterAll(() => {
  database.close();
});

beforeEach(() => {
  // 每个测试前执行
  database.clear();
});

afterEach(() => {
  // 每个测试后执行
  database.save();
});

test('test 1', () => {
  database.insert('data');
  expect(database.count()).toBe(1);
});
```

### 6. 参数化测试

```javascript
test.each([
  [1, 1, 2],
  [1, 2, 3],
  [2, 2, 4],
])('add(%i, %i) = %i', (a, b, expected) => {
  expect(add(a, b)).toBe(expected);
});
```

### 7. 快照测试

```javascript
test('snapshot', () => {
  const user = {
    name: 'Alice',
    email: 'alice@example.com',
    age: 30
  };
  
  expect(user).toMatchSnapshot();
});
```

## React测试

```javascript
import { render, screen } from '@testing-library/react';
import UserCard from './UserCard';

test('renders user card', () => {
  render(<UserCard name="Alice" email="alice@example.com" />);
  
  expect(screen.getByText('Alice')).toBeInTheDocument();
  expect(screen.getByText('alice@example.com')).toBeInTheDocument();
});

test('click handler', () => {
  const handleClick = jest.fn();
  render(<UserCard onClick={handleClick} />);
  
  screen.getByText('Click').click();
  expect(handleClick).toHaveBeenCalledTimes(1);
});
```

## 最佳实践

### ✅ DO

1. **描述性测试名**
```javascript
// ✅ 好
test('user cannot register with duplicate email', () => {});

// ❌ 差
test('test1', () => {});
```

2. **一个测试一个断言**
```javascript
// ✅ 好
test('user name is correct', () => {
  const user = new User('Alice');
  expect(user.name).toBe('Alice');
});
```

### ❌ DON'T

1. **不要测试实现细节**
```javascript
// ❌ 差
test('internal state', () => {
  expect(component._state.count).toBe(0);
});

// ✅ 好
test('visible output', () => {
  expect(screen.getByText('1')).toBeInTheDocument();
});
```

## 学习路径

### 初级 (1周)
1. 基础测试
2. 匹配器
3. 异步测试

### 中级 (1-2周)
1. Mock和Spy
2. React测试
3. 快照测试

### 高级 (2-3周)
1. 自定义匹配器
2. 测试复杂应用
3. 性能测试

---

**知识ID**: `jest-complete`  
**领域**: development  
**类型**: standards  
**难度**: intermediate  
**质量分**: 94  
**维护者**: frontend-team@superdev.com  
**最后更新**: 2026-03-28
