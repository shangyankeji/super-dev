---
id: react-complete
title: React完整开发指南
domain: frontend
category: 01-standards
difficulty: intermediate
tags: [complete, frontend, hooks, react, 学习路径, 常见陷阱, 性能优化, 最佳实践]
quality_score: 70
last_updated: 2026-06-15
---
# React完整开发指南

## 概述
React是一个用于构建用户界面的JavaScript库,由Facebook开发维护。采用组件化、声明式编程范式,支持服务端渲染(SSR)和移动端开发(React Native)。

## 核心概念

### 1. JSX语法

```jsx
// JSX = JavaScript XML
const element = <h1>Hello, world!</h1>;

// 表达式嵌入
const name = "Alice";
const element = <h1>Hello, {name}</h1>;

// 属性
const element = <img src={user.avatarUrl} alt={user.name} />;

// 子元素
const element = (
  <div>
    <h1>Hello!</h1>
    <h2>Good to see you here.</h2>
  </div>
);
```

### 2. 组件

**函数组件**:
```jsx
function Welcome(props) {
  return <h1>Hello, {props.name}</h1>;
}

// 箭头函数
const Welcome = ({ name }) => <h1>Hello, {name}</h1>;
```

**类组件**:
```jsx
class Welcome extends React.Component {
  render() {
    return <h1>Hello, {this.props.name}</h1>;
  }
}
```

**组合组件**:
```jsx
function App() {
  return (
    <div>
      <Welcome name="Alice" />
      <Welcome name="Bob" />
      <Welcome name="Charlie" />
    </div>
  );
}
```

### 3. Props和State

**Props (只读)**:
```jsx
function Avatar({ user, size }) {
  return (
    <img
      className="avatar"
      src={user.avatarUrl}
      alt={user.name}
      width={size}
      height={size}
    />
  );
}

// 默认Props
Avatar.defaultProps = {
  size: 100
};
```

**State (可变)**:
```jsx
import { useState } from 'react';

function Counter() {
  const [count, setCount] = useState(0);
  
  return (
    <div>
      <p>You clicked {count} times</p>
      <button onClick={() => setCount(count + 1)}>
        Click me
      </button>
    </div>
  );
}
```

### 4. 生命周期

**函数组件Hooks**:
```jsx
import { useState, useEffect } from 'react';

function Example() {
  const [count, setCount] = useState(0);
  
  // componentDidMount + componentDidUpdate
  useEffect(() => {
    document.title = `You clicked ${count} times`;
    
    // componentWillUnmount (cleanup)
    return () => {
      console.log('Cleanup');
    };
  }, [count]); // 依赖数组
  
  return (
    <div>
      <p>You clicked {count} times</p>
      <button onClick={() => setCount(count + 1)}>
        Click me
      </button>
    </div>
  );
}
```

**类组件生命周期**:
```jsx
class Example extends React.Component {
  constructor(props) {
    super(props);
    this.state = { count: 0 };
  }
  
  componentDidMount() {
    document.title = `You clicked ${this.state.count} times`;
  }
  
  componentDidUpdate() {
    document.title = `You clicked ${this.state.count} times`;
  }
  
  componentWillUnmount() {
    console.log('Cleanup');
  }
  
  render() {
    return (
      <div>
        <p>You clicked {this.state.count} times</p>
        <button onClick={() => this.setState({ count: this.state.count + 1 })}>
          Click me
        </button>
      </div>
    );
  }
}
```

## React Hooks

### 1. useState

```jsx
import { useState } from 'react';

function Example() {
  // 数组解构
  const [count, setCount] = useState(0);
  const [user, setUser] = useState({ name: '', age: 0 });
  
  // 更新状态
  const increment = () => setCount(count + 1);
  
  // 函数式更新
  const increment = () => setCount(prev => prev + 1);
  
  // 更新对象
  const updateUser = () => {
    setUser(prev => ({ ...prev, name: 'Alice' }));
  };
}
```

### 2. useEffect

```jsx
import { useEffect } from 'react';

function Example() {
  // 每次渲染后执行
  useEffect(() => {
    console.log('Rendered');
  });
  
  // 只在mount时执行
  useEffect(() => {
    console.log('Mounted');
  }, []);
  
  // 依赖项变化时执行
  useEffect(() => {
    console.log('Count changed');
  }, [count]);
  
  // 清理函数
  useEffect(() => {
    const timer = setInterval(() => {}, 1000);
    
    return () => {
      clearInterval(timer);
    };
  }, []);
}
```

### 3. useContext

```jsx
import { createContext, useContext } from 'react';

const ThemeContext = createContext('light');

function App() {
  return (
    <ThemeContext.Provider value="dark">
      <Toolbar />
    </ThemeContext.Provider>
  );
}

function Toolbar() {
  const theme = useContext(ThemeContext);
  return <div>Current theme: {theme}</div>;
}
```

### 4. useReducer

```jsx
import { useReducer } from 'react';

const initialState = { count: 0 };

function reducer(state, action) {
  switch (action.type) {
    case 'increment':
      return { count: state.count + 1 };
    case 'decrement':
      return { count: state.count - 1 };
    default:
      throw new Error();
  }
}

function Counter() {
  const [state, dispatch] = useReducer(reducer, initialState);
  
  return (
    <>
      Count: {state.count}
      <button onClick={() => dispatch({ type: 'decrement' })}>-</button>
      <button onClick={() => dispatch({ type: 'increment' })}>+</button>
    </>
  );
}
```

### 5. useMemo和useCallback

```jsx
import { useMemo, useCallback } from 'react';

function Example({ items, onItemClick }) {
  // 缓存计算结果
  const sortedItems = useMemo(() => {
    console.log('Sorting...');
    return items.sort((a, b) => a.name.localeCompare(b.name));
  }, [items]);
  
  // 缓存回调函数
  const handleClick = useCallback((id) => {
    console.log('Clicked:', id);
    onItemClick(id);
  }, [onItemClick]);
  
  return (
    <ul>
      {sortedItems.map(item => (
        <li key={item.id} onClick={() => handleClick(item.id)}>
          {item.name}
        </li>
      ))}
    </ul>
  );
}
```

### 6. useRef

```jsx
import { useRef, useEffect } from 'react';

function TextInput() {
  const inputRef = useRef(null);
  
  useEffect(() => {
    // 自动聚焦
    inputRef.current.focus();
  }, []);
  
  return <input ref={inputRef} type="text" />;
}

function Counter() {
  const countRef = useRef(0);
  
  const increment = () => {
    countRef.current++;
    console.log(countRef.current);  // 更新但不触发重渲染
  };
  
  return <button onClick={increment}>Increment</button>;
}
```

### 7. 自定义Hook

```jsx
// 自定义Hook: 获取窗口大小
function useWindowSize() {
  const [size, setSize] = useState({
    width: window.innerWidth,
    height: window.innerHeight
  });
  
  useEffect(() => {
    const handleResize = () => {
      setSize({
        width: window.innerWidth,
        height: window.innerHeight
      });
    };
    
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);
  
  return size;
}

// 使用
function App() {
  const { width, height } = useWindowSize();
  
  return (
    <div>
      Window size: {width} x {height}
    </div>
  );
}
```

## 状态管理

### 1. Context API

```jsx
import { createContext, useContext, useReducer } from 'react';

// 创建Context
const AppStateContext = createContext();
const AppDispatchContext = createContext();

// Reducer
function appReducer(state, action) {
  switch (action.type) {
    case 'LOGIN':
      return { ...state, user: action.payload };
    case 'LOGOUT':
      return { ...state, user: null };
    default:
      return state;
  }
}

// Provider
function AppProvider({ children }) {
  const [state, dispatch] = useReducer(appReducer, { user: null });
  
  return (
    <AppStateContext.Provider value={state}>
      <AppDispatchContext.Provider value={dispatch}>
        {children}
      </AppDispatchContext.Provider>
    </AppStateContext.Provider>
  );
}

// 自定义Hook
function useAppState() {
  return useContext(AppStateContext);
}

function useAppDispatch() {
  return useContext(AppDispatchContext);
}

// 使用
function LoginButton() {
  const dispatch = useAppDispatch();
  
  return (
    <button onClick={() => dispatch({ type: 'LOGIN', payload: { name: 'Alice' } })}>
      Login
    </button>
  );
}
```

### 2. Redux Toolkit

```jsx
import { createSlice, configureStore } from '@reduxjs/toolkit';

// Slice
const counterSlice = createSlice({
  name: 'counter',
  initialState: { value: 0 },
  reducers: {
    increment: state => { state.value += 1; },
    decrement: state => { state.value -= 1; },
    incrementByAmount: (state, action) => { state.value += action.payload; }
  }
});

export const { increment, decrement, incrementByAmount } = counterSlice.actions;

// Store
const store = configureStore({
  reducer: {
    counter: counterSlice.reducer
  }
});

// 使用
import { useSelector, useDispatch } from 'react-redux';

function Counter() {
  const count = useSelector(state => state.counter.value);
  const dispatch = useDispatch();
  
  return (
    <div>
      <button onClick={() => dispatch(decrement())}>-</button>
      <span>{count}</span>
      <button onClick={() => dispatch(increment())}>+</button>
    </div>
  );
}
```

### 3. Zustand

```jsx
import create from 'zustand';

// 创建store
const useStore = create((set) => ({
  count: 0,
  increment: () => set((state) => ({ count: state.count + 1 })),
  decrement: () => set((state) => ({ count: state.count - 1 })),
  reset: () => set({ count: 0 })
}));

// 使用
function Counter() {
  const { count, increment, decrement, reset } = useStore();
  
  return (
    <div>
      <button onClick={decrement}>-</button>
      <span>{count}</span>
      <button onClick={increment}>+</button>
      <button onClick={reset}>Reset</button>
    </div>
  );
}
```

## 性能优化

### 1. React.memo

```jsx
import { memo } from 'react';

// 只在props变化时重新渲染
const SlowComponent = memo(function SlowComponent({ value }) {
  // 昂贵的计算
  return <div>{value}</div>;
});

// 自定义比较
const SlowComponent = memo(
  function SlowComponent({ user }) {
    return <div>{user.name}</div>;
  },
  (prevProps, nextProps) => {
    return prevProps.user.id === nextProps.user.id;
  }
);
```

### 2. 代码分割

```jsx
import { lazy, Suspense } from 'react';

// 懒加载组件
const OtherComponent = lazy(() => import('./OtherComponent'));

function App() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <OtherComponent />
    </Suspense>
  );
}
```

### 3. 虚拟列表

```jsx
import { FixedSizeList } from 'react-window';

function VirtualList({ items }) {
  const Row = ({ index, style }) => (
    <div style={style}>
      {items[index].name}
    </div>
  );
  
  return (
    <FixedSizeList
      height={600}
      itemCount={items.length}
      itemSize={35}
      width="100%"
    >
      {Row}
    </FixedSizeList>
  );
}
```

## 常见陷阱

### ❌ 陷阱1: 在循环/条件中调用Hooks

```jsx
// ❌ 错误
function Component({ items }) {
  items.forEach(item => {
    useEffect(() => {}, [item]);  // 不要在循环中调用
  });
  
  if (condition) {
    useState(0);  // 不要在条件中调用
  }
}

// ✅ 正确
function Component({ items }) {
  useEffect(() => {
    items.forEach(item => {
      // 在Hook内部循环
    });
  }, [items]);
}
```

### ❌ 陷阱2: 直接修改State

```jsx
// ❌ 错误
const [user, setUser] = useState({ name: 'Alice', age: 30 });
user.name = 'Bob';  // 直接修改,不会触发重渲染

// ✅ 正确
setUser(prev => ({ ...prev, name: 'Bob' }));
```

### ❌ 陷阱3: 忘记useEffect依赖

```jsx
// ❌ 错误: 缺少依赖
useEffect(() => {
  console.log(count);
}, []);  // count变化时不会重新执行

// ✅ 正确
useEffect(() => {
  console.log(count);
}, [count]);
```

## 最佳实践

### ✅ DO

1. **使用函数组件和Hooks**
```jsx
// ✅ 推荐
function Component() {
  const [count, setCount] = useState(0);
  return <div>{count}</div>;
}
```

2. **提取自定义Hook**
```jsx
// ✅ 复用逻辑
function useFetch(url) {
  const [data, setData] = useState(null);
  useEffect(() => {
    fetch(url).then(res => res.json()).then(setData);
  }, [url]);
  return data;
}
```

3. **合理使用key**
```jsx
// ✅ 稳定的key
{items.map(item => <Item key={item.id} {...item} />)}

// ❌ 不要用index作为key
{items.map((item, index) => <Item key={index} {...item} />)}
```

### ❌ DON'T

1. **不要过度使用useEffect**
```jsx
// ❌ 不必要的effect
const [fullName, setFullName] = useState('');
useEffect(() => {
  setFullName(`${firstName} ${lastName}`);
}, [firstName, lastName]);

// ✅ 直接计算
const fullName = `${firstName} ${lastName}`;
```

## 学习路径

### 初级 (1-2周)
1. JSX和组件基础
2. Props和State
3. 事件处理

### 中级 (2-3周)
1. React Hooks
2. 表单处理
3. 路由(React Router)

### 高级 (2-4周)
1. 状态管理(Context/Redux)
2. 性能优化
3. 测试(Testing Library)

### 专家级 (持续)
1. 自定义Hooks库
2. SSR/SSG(Next.js)
3. 设计系统

## 参考资料

### 官方文档
- [React官方文档](https://react.dev/)
- [React Router](https://reactrouter.com/)

### 教程
- [React Tutorial](https://react.dev/learn)
- [React Patterns](https://reactpatterns.com/)

---

**知识ID**: `react-complete`  
**领域**: frontend  
**类型**: standards  
**难度**: intermediate  
**质量分**: 94  
**维护者**: frontend-team@superdev.com  
**最后更新**: 2026-03-28
