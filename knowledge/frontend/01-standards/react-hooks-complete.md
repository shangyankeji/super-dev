---
id: react-hooks-complete
title: React Hooks 深度指南
domain: frontend
category: 01-standards
difficulty: intermediate
tags: [complete, frontend, hooks, react, typescript, 常见陷阱, 性能优化, 最佳实践]
quality_score: 70
last_updated: 2026-06-15
---
# React Hooks 深度指南

## 概述

React Hooks 是 React 16.8 引入的特性,允许在函数组件中使用状态和其他 React 特性,无需编写类组件。Hooks 彻底改变了 React 开发模式,使代码更简洁、复用性更强、逻辑组织更清晰。

### 核心优势

- **更简洁的代码**: 函数组件替代类组件,减少样板代码
- **逻辑复用**: 自定义 Hook 实现逻辑复用,避免 render props 和高阶组件的复杂性
- **更好的代码组织**: 相关逻辑聚合在一起,而非分散在生命周期方法中
- **更容易测试**: 纯函数更容易单元测试
- **更小的包体积**: 减少类组件的额外开销

---

## 核心 Hooks

### 1. useState - 状态管理

`useState` 是最基础的 Hook,用于在函数组件中添加状态。

```javascript
import React, { useState } from 'react';

function Counter() {
  const [count, setCount] = useState(0);
  
  return (
    <div>
      <p>Count: {count}</p>
      <button onClick={() => setCount(count + 1)}>Increment</button>
      <button onClick={() => setCount(prev => prev - 1)}>Decrement</button>
    </div>
  );
}
```

#### 高级用法

```javascript
// 惰性初始化 - 只在首次渲染时执行
const [state, setState] = useState(() => {
  const initialState = expensiveComputation();
  return initialState;
});

// 更新对象状态
const [user, setUser] = useState({ name: '', email: '' });

// 方式 1: 手动合并
setUser({ ...user, name: 'Alice' });

// 方式 2: 函数式更新
setUser(prev => ({ ...prev, name: 'Alice' }));

// 更新数组状态
const [items, setItems] = useState([]);

// 添加项
setItems([...items, newItem]);
setItems(prev => [...prev, newItem]);

// 删除项
setItems(items.filter(item => item.id !== id));

// 更新项
setItems(items.map(item => 
  item.id === id ? { ...item, updated: true } : item
));
```

### 2. useEffect - 副作用处理

`useEffect` 用于处理副作用,如数据获取、订阅、DOM 操作等。

```javascript
import React, { useState, useEffect } from 'react';

function UserProfile({ userId }) {
  const [user, setUser] = useState(null);
  
  // 组件挂载时执行
  useEffect(() => {
    fetchUser(userId).then(data => setUser(data));
  }, [userId]); // 依赖数组
  
  if (!user) return <div>Loading...</div>;
  return <div>{user.name}</div>;
}
```

#### 依赖数组规则

```javascript
// 1. 每次渲染后都执行
useEffect(() => {
  console.log('每次渲染都执行');
});

// 2. 只在挂载时执行一次
useEffect(() => {
  console.log('组件挂载');
}, []);

// 3. 依赖特定值
useEffect(() => {
  console.log('userId 变化时执行');
}, [userId]);

// 4. 多个依赖
useEffect(() => {
  console.log('userId 或 userName 变化时执行');
}, [userId, userName]);
```

#### 清理副作用

```javascript
useEffect(() => {
  const subscription = props.source.subscribe();
  
  // 返回清理函数
  return () => {
    subscription.unsubscribe();
  };
}, [props.source]);

// 真实案例:事件监听
useEffect(() => {
  const handleResize = () => {
    console.log('Window resized');
  };
  
  window.addEventListener('resize', handleResize);
  
  return () => {
    window.removeEventListener('resize', handleResize);
  };
}, []);

// 真实案例:定时器
useEffect(() => {
  const timer = setInterval(() => {
    console.log('Tick');
  }, 1000);
  
  return () => clearInterval(timer);
}, []);
```

#### useEffect 的执行时机

```javascript
useEffect(() => {
  // 在渲染之后执行(异步)
  console.log('DOM 已更新');
});

useLayoutEffect(() => {
  // 在 DOM 变更后同步执行,阻塞绘制
  // 用于读取 DOM 布局并同步重新渲染
  console.log('DOM 变更后,绘制前');
});
```

### 3. useContext - 上下文消费

避免 prop drilling,跨组件共享数据。

```javascript
import React, { createContext, useContext, useState } from 'react';

// 创建 Context
const ThemeContext = createContext('light');
const UserContext = createContext(null);

function App() {
  const [theme, setTheme] = useState('dark');
  const [user, setUser] = useState({ name: 'Alice' });
  
  return (
    <ThemeContext.Provider value={theme}>
      <UserContext.Provider value={{ user, setUser }}>
        <Dashboard />
      </UserContext.Provider>
    </ThemeContext.Provider>
  );
}

function Dashboard() {
  return <Profile />;
}

function Profile() {
  // 消费 Context
  const theme = useContext(ThemeContext);
  const { user } = useContext(UserContext);
  
  return (
    <div className={`profile ${theme}`}>
      <h1>{user.name}</h1>
    </div>
  );
}
```

### 4. useReducer - 复杂状态逻辑

适合管理复杂的状态逻辑,类似 Redux 的 reducer。

```javascript
import React, { useReducer } from 'react';

// Reducer 函数
const initialState = { count: 0 };

function reducer(state, action) {
  switch (action.type) {
    case 'increment':
      return { count: state.count + 1 };
    case 'decrement':
      return { count: state.count - 1 };
    case 'reset':
      return { count: action.payload };
    default:
      throw new Error('Unknown action');
  }
}

function Counter() {
  const [state, dispatch] = useReducer(reducer, initialState);
  
  return (
    <div>
      <p>Count: {state.count}</p>
      <button onClick={() => dispatch({ type: 'increment' })}>+</button>
      <button onClick={() => dispatch({ type: 'decrement' })}>-</button>
      <button onClick={() => dispatch({ type: 'reset', payload: 0 })}>
        Reset
      </button>
    </div>
  );
}
```

#### 惰性初始化

```javascript
function init(initialCount) {
  return { count: initialCount };
}

function Counter({ initialCount }) {
  const [state, dispatch] = useReducer(reducer, initialCount, init);
  // ...
}
```

#### 复杂表单状态

```javascript
const formReducer = (state, action) => {
  switch (action.type) {
    case 'FIELD_CHANGE':
      return {
        ...state,
        values: { ...state.values, [action.field]: action.value },
        touched: { ...state.touched, [action.field]: true }
      };
    case 'SET_ERRORS':
      return { ...state, errors: action.errors };
    case 'RESET':
      return action.initialState;
    default:
      return state;
  }
};

function Form() {
  const [state, dispatch] = useReducer(formReducer, {
    values: { username: '', email: '' },
    errors: {},
    touched: {}
  });
  
  const handleChange = (field) => (e) => {
    dispatch({ type: 'FIELD_CHANGE', field, value: e.target.value });
  };
  
  return (
    <form>
      <input
        value={state.values.username}
        onChange={handleChange('username')}
      />
      {state.errors.username && <span>{state.errors.username}</span>}
    </form>
  );
}
```

### 5. useCallback - 函数缓存

缓存函数引用,避免不必要的重新渲染。

```javascript
import React, { useState, useCallback, memo } from 'react';

// 子组件使用 memo 优化
const Button = memo(({ onClick, children }) => {
  console.log('Button rendered');
  return <button onClick={onClick}>{children}</button>;
});

function Parent() {
  const [count, setCount] = useState(0);
  const [text, setText] = useState('');
  
  // ❌ 不好:每次渲染都创建新函数
  // const handleClick = () => {
  //   console.log('Clicked');
  // };
  
  // ✅ 好:缓存函数引用
  const handleClick = useCallback(() => {
    console.log('Clicked');
  }, []);
  
  // 依赖状态
  const handleIncrement = useCallback(() => {
    setCount(prev => prev + 1);
  }, []);
  
  return (
    <div>
      <input value={text} onChange={e => setText(e.target.value)} />
      <Button onClick={handleClick}>Click Me</Button>
      <Button onClick={handleIncrement}>Count: {count}</Button>
    </div>
  );
}
```

### 6. useMemo - 值缓存

缓存计算结果,避免重复计算。

```javascript
import React, { useState, useMemo } from 'react';

function ProductList({ products, filter, sortBy }) {
  // ❌ 不好:每次渲染都重新计算
  // const filteredProducts = products
  //   .filter(p => p.name.includes(filter))
  //   .sort((a, b) => a[sortBy] - b[sortBy]);
  
  // ✅ 好:缓存计算结果
  const filteredProducts = useMemo(() => {
    console.log('重新计算过滤和排序');
    return products
      .filter(p => p.name.includes(filter))
      .sort((a, b) => a[sortBy] - b[sortBy]);
  }, [products, filter, sortBy]);
  
  return (
    <ul>
      {filteredProducts.map(product => (
        <li key={product.id}>{product.name}</li>
      ))}
    </ul>
  );
}

// 复杂计算示例
function Fibonacci({ n }) {
  const fib = useMemo(() => {
    const compute = (num) => {
      if (num <= 1) return num;
      return compute(num - 1) + compute(num - 2);
    };
    return compute(n);
  }, [n]);
  
  return <div>Fib({n}) = {fib}</div>;
}
```

### 7. useRef - 引用持久化

获取 DOM 元素或存储可变值,不触发重新渲染。

```javascript
import React, { useRef, useEffect } from 'react';

function TextInput() {
  const inputRef = useRef(null);
  
  useEffect(() => {
    // 自动聚焦
    inputRef.current.focus();
  }, []);
  
  return <input ref={inputRef} />;
}

// 存储可变值
function Timer() {
  const [seconds, setSeconds] = useState(0);
  const intervalRef = useRef(null);
  
  const startTimer = () => {
    if (intervalRef.current) return;
    intervalRef.current = setInterval(() => {
      setSeconds(prev => prev + 1);
    }, 1000);
  };
  
  const stopTimer = () => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  };
  
  return (
    <div>
      <p>Seconds: {seconds}</p>
      <button onClick={startTimer}>Start</button>
      <button onClick={stopTimer}>Stop</button>
    </div>
  );
}

// 跟踪上一次的值
function usePrevious(value) {
  const ref = useRef();
  
  useEffect(() => {
    ref.current = value;
  }, [value]);
  
  return ref.current;
}

function Counter() {
  const [count, setCount] = useState(0);
  const prevCount = usePrevious(count);
  
  return (
    <div>
      <p>Current: {count}, Previous: {prevCount}</p>
      <button onClick={() => setCount(count + 1)}>Increment</button>
    </div>
  );
}
```

### 8. useImperativeHandle - 自定义 ref 暴露

自定义暴露给父组件的实例值。

```javascript
import React, { useRef, useImperativeHandle, forwardRef } from 'react';

const FancyInput = forwardRef((props, ref) => {
  const inputRef = useRef();
  
  useImperativeHandle(ref, () => ({
    focus: () => {
      inputRef.current.focus();
    },
    clear: () => {
      inputRef.current.value = '';
    }
  }));
  
  return <input ref={inputRef} {...props} />;
});

function Parent() {
  const inputRef = useRef();
  
  return (
    <div>
      <FancyInput ref={inputRef} />
      <button onClick={() => inputRef.current.focus()}>Focus</button>
      <button onClick={() => inputRef.current.clear()}>Clear</button>
    </div>
  );
}
```

---

## 自定义 Hooks

自定义 Hook 是复用逻辑的最佳方式。

### 1. useLocalStorage - 持久化状态

```javascript
import { useState, useEffect } from 'react';

function useLocalStorage(key, initialValue) {
  const [storedValue, setStoredValue] = useState(() => {
    try {
      const item = window.localStorage.getItem(key);
      return item ? JSON.parse(item) : initialValue;
    } catch (error) {
      console.error(error);
      return initialValue;
    }
  });
  
  const setValue = (value) => {
    try {
      const valueToStore = value instanceof Function ? value(storedValue) : value;
      setStoredValue(valueToStore);
      window.localStorage.setItem(key, JSON.stringify(valueToStore));
    } catch (error) {
      console.error(error);
    }
  };
  
  return [storedValue, setValue];
}

// 使用
function App() {
  const [name, setName] = useLocalStorage('name', '');
  
  return (
    <input
      value={name}
      onChange={e => setName(e.target.value)}
      placeholder="输入名字(自动保存)"
    />
  );
}
```

### 2. useFetch - 数据获取

```javascript
import { useState, useEffect } from 'react';

function useFetch(url) {
  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  
  useEffect(() => {
    const abortController = new AbortController();
    
    async function fetchData() {
      try {
        setLoading(true);
        const response = await fetch(url, { signal: abortController.signal });
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const json = await response.json();
        setData(json);
        setError(null);
      } catch (error) {
        if (error.name !== 'AbortError') {
          setError(error);
        }
      } finally {
        setLoading(false);
      }
    }
    
    fetchData();
    
    return () => abortController.abort();
  }, [url]);
  
  return { data, loading, error };
}

// 使用
function UserProfile({ userId }) {
  const { data: user, loading, error } = useFetch(`/api/users/${userId}`);
  
  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;
  return <div>{user.name}</div>;
}
```

### 3. useDebounce - 防抖

```javascript
import { useState, useEffect } from 'react';

function useDebounce(value, delay) {
  const [debouncedValue, setDebouncedValue] = useState(value);
  
  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);
    
    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);
  
  return debouncedValue;
}

// 使用
function SearchInput() {
  const [searchTerm, setSearchTerm] = useState('');
  const debouncedSearchTerm = useDebounce(searchTerm, 500);
  
  useEffect(() => {
    if (debouncedSearchTerm) {
      // 执行搜索
      searchAPI(debouncedSearchTerm);
    }
  }, [debouncedSearchTerm]);
  
  return (
    <input
      value={searchTerm}
      onChange={e => setSearchTerm(e.target.value)}
      placeholder="Search..."
    />
  );
}
```

### 4. useWindowSize - 窗口尺寸

```javascript
import { useState, useEffect } from 'react';

function useWindowSize() {
  const [windowSize, setWindowSize] = useState({
    width: typeof window !== 'undefined' ? window.innerWidth : 0,
    height: typeof window !== 'undefined' ? window.innerHeight : 0,
  });
  
  useEffect(() => {
    function handleResize() {
      setWindowSize({
        width: window.innerWidth,
        height: window.innerHeight,
      });
    }
    
    window.addEventListener('resize', handleResize);
    handleResize();
    
    return () => window.removeEventListener('resize', handleResize);
  }, []);
  
  return windowSize;
}

// 使用
function ResponsiveComponent() {
  const { width, height } = useWindowSize();
  
  return (
    <div>
      <p>Width: {width}px</p>
      <p>Height: {height}px</p>
      {width < 768 && <p>Mobile view</p>}
    </div>
  );
}
```

### 5. useToggle - 切换状态

```javascript
import { useState, useCallback } from 'react';

function useToggle(initialValue = false) {
  const [value, setValue] = useState(initialValue);
  
  const toggle = useCallback(() => setValue(v => !v), []);
  const setTrue = useCallback(() => setValue(true), []);
  const setFalse = useCallback(() => setValue(false), []);
  
  return { value, toggle, setTrue, setFalse, setValue };
}

// 使用
function Modal() {
  const { value: isOpen, toggle, setFalse: closeModal } = useToggle();
  
  return (
    <div>
      <button onClick={toggle}>Open Modal</button>
      {isOpen && (
        <div className="modal">
          <p>Modal content</p>
          <button onClick={closeModal}>Close</button>
        </div>
      )}
    </div>
  );
}
```

### 6. useForm - 表单管理

```javascript
import { useState, useCallback } from 'react';

function useForm(initialValues, validate) {
  const [values, setValues] = useState(initialValues);
  const [errors, setErrors] = useState({});
  const [touched, setTouched] = useState({});
  
  const handleChange = useCallback((e) => {
    const { name, value, type, checked } = e.target;
    setValues(prev => ({
      ...prev,
      [name]: type === 'checkbox' ? checked : value
    }));
  }, []);
  
  const handleBlur = useCallback((e) => {
    const { name } = e.target;
    setTouched(prev => ({ ...prev, [name]: true }));
    
    if (validate) {
      const validationErrors = validate(values);
      setErrors(validationErrors);
    }
  }, [values, validate]);
  
  const handleSubmit = useCallback((onSubmit) => (e) => {
    e.preventDefault();
    
    if (validate) {
      const validationErrors = validate(values);
      setErrors(validationErrors);
      setTouched(
        Object.keys(values).reduce((acc, key) => ({ ...acc, [key]: true }), {})
      );
      
      if (Object.keys(validationErrors).length > 0) {
        return;
      }
    }
    
    onSubmit(values);
  }, [values, validate]);
  
  const reset = useCallback(() => {
    setValues(initialValues);
    setErrors({});
    setTouched({});
  }, [initialValues]);
  
  return {
    values,
    errors,
    touched,
    handleChange,
    handleBlur,
    handleSubmit,
    reset,
    setValues
  };
}

// 使用
function LoginForm() {
  const {
    values,
    errors,
    touched,
    handleChange,
    handleBlur,
    handleSubmit
  } = useForm(
    { email: '', password: '' },
    (values) => {
      const errors = {};
      if (!values.email) errors.email = 'Email is required';
      if (!values.password) errors.password = 'Password is required';
      return errors;
    }
  );
  
  const onSubmit = (data) => {
    console.log('Form submitted:', data);
  };
  
  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <input
        name="email"
        value={values.email}
        onChange={handleChange}
        onBlur={handleBlur}
      />
      {touched.email && errors.email && <span>{errors.email}</span>}
      
      <input
        name="password"
        type="password"
        value={values.password}
        onChange={handleChange}
        onBlur={handleBlur}
      />
      {touched.password && errors.password && <span>{errors.password}</span>}
      
      <button type="submit">Submit</button>
    </form>
  );
}
```

---

## Hooks 规则

### 规则 1: 只在顶层调用 Hooks

```javascript
// ❌ 错误:在循环中调用
function BadComponent({ items }) {
  items.forEach(item => {
    const [state, setState] = useState(item); // 错误!
  });
}

// ✅ 正确:在顶层调用
function GoodComponent({ items }) {
  const [states, setStates] = useState(items);
}

// ❌ 错误:在条件语句中调用
function BadComponent({ condition }) {
  if (condition) {
    const [value, setValue] = useState(0); // 错误!
  }
}

// ✅ 正确:条件逻辑在 Hook 内部
function GoodComponent({ condition }) {
  const [value, setValue] = useState(0);
  
  useEffect(() => {
    if (condition) {
      // 条件逻辑
    }
  }, [condition]);
}
```

### 规则 2: 只在 React 函数中调用

```javascript
// ❌ 错误:在普通函数中调用
function handleClick() {
  const [count, setCount] = useState(0); // 错误!
}

// ✅ 正确:在组件或自定义 Hook 中调用
function MyComponent() {
  const [count, setCount] = useState(0);
  
  function handleClick() {
    setCount(count + 1);
  }
}
```

### ESLint 插件

```bash
npm install eslint-plugin-react-hooks --save-dev
```

```json
{
  "plugins": ["react-hooks"],
  "rules": {
    "react-hooks/rules-of-hooks": "error",
    "react-hooks/exhaustive-deps": "warn"
  }
}
```

---

## 性能优化

### 1. 使用 React.memo 优化子组件

```javascript
const ChildComponent = memo(({ data, onClick }) => {
  console.log('Child rendered');
  return <div onClick={onClick}>{data}</div>;
});
```

### 2. 正确使用依赖数组

```javascript
// ❌ 缺少依赖
useEffect(() => {
  fetchData(userId);
}, []); // 缺少 userId 依赖

// ✅ 完整依赖
useEffect(() => {
  fetchData(userId);
}, [userId]);
```

### 3. 避免内联函数和对象

```javascript
// ❌ 每次渲染都创建新函数/对象
function Parent() {
  const [count, setCount] = useState(0);
  
  return (
    <Child
      onClick={() => console.log('click')}  // 新函数
      style={{ color: 'red' }}               // 新对象
    />
  );
}

// ✅ 使用 useCallback 和 useMemo
function Parent() {
  const [count, setCount] = useState(0);
  
  const handleClick = useCallback(() => {
    console.log('click');
  }, []);
  
  const style = useMemo(() => ({ color: 'red' }), []);
  
  return <Child onClick={handleClick} style={style} />;
}
```

### 4. 虚拟化长列表

```javascript
import { FixedSizeList } from 'react-window';

function VirtualizedList({ items }) {
  const Row = ({ index, style }) => (
    <div style={style}>{items[index].name}</div>
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

---

## 常见陷阱

### 陷阱 1: 闭包陷阱

```javascript
// ❌ 错误:闭包捕获旧值
function Counter() {
  const [count, setCount] = useState(0);
  
  useEffect(() => {
    const timer = setInterval(() => {
      console.log(count); // 永远是 0
    }, 1000);
    return () => clearInterval(timer);
  }, []);
}

// ✅ 正确:使用函数式更新
function Counter() {
  const [count, setCount] = useState(0);
  
  useEffect(() => {
    const timer = setInterval(() => {
      setCount(prev => {
        console.log(prev); // 正确的值
        return prev;
      });
    }, 1000);
    return () => clearInterval(timer);
  }, []);
}

// ✅ 或使用 ref
function Counter() {
  const [count, setCount] = useState(0);
  const countRef = useRef(count);
  
  countRef.current = count;
  
  useEffect(() => {
    const timer = setInterval(() => {
      console.log(countRef.current); // 正确的值
    }, 1000);
    return () => clearInterval(timer);
  }, []);
}
```

### 陷阱 2: 无限循环

```javascript
// ❌ 错误:导致无限循环
function BadComponent() {
  const [items, setItems] = useState([]);
  
  useEffect(() => {
    setItems([...items, newItem]); // 触发重新渲染
  }, [items]); // 依赖 items,导致循环
}

// ✅ 正确:使用函数式更新
function GoodComponent() {
  const [items, setItems] = useState([]);
  
  useEffect(() => {
    setItems(prev => [...prev, newItem]);
  }, []); // 不依赖 items
}
```

### 陷阱 3: 直接修改状态

```javascript
// ❌ 错误:直接修改状态
const [user, setUser] = useState({ name: 'Alice' });
user.name = 'Bob'; // 不会触发重新渲染

// ✅ 正确:创建新对象
setUser({ ...user, name: 'Bob' });
```

---

## 最佳实践

1. **✅ 使用函数式更新**: `setCount(prev => prev + 1)`
2. **✅ 保持依赖数组完整**: 使用 eslint-plugin-react-hooks
3. **✅ 拆分复杂状态**: 多个 useState 而非一个巨大对象
4. **✅ 自定义 Hook 复用逻辑**: 避免复制粘贴
5. **✅ 命名规范**: use 开头的自定义 Hook
6. **✅ 合理使用 useMemo/useCallback**: 不要过度优化
7. **✅ 错误边界**: 处理渲染错误
8. **✅ 清理副作用**: useEffect 返回清理函数
9. **✅ 测试 Hooks**: 使用 @testing-library/react-hooks
10. **✅ TypeScript 支持**: 为 Hook 添加类型

---

## TypeScript 集成

```typescript
import React, { useState, useEffect, useCallback, useMemo } from 'react';

interface User {
  id: number;
  name: string;
  email: string;
}

// useState with TypeScript
function UserForm() {
  const [user, setUser] = useState<User>({
    id: 0,
    name: '',
    email: ''
  });
  
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setUser(prev => ({ ...prev, [name]: value }));
  };
  
  return (
    <form>
      <input name="name" value={user.name} onChange={handleChange} />
      <input name="email" value={user.email} onChange={handleChange} />
    </form>
  );
}

// Custom Hook with TypeScript
function useFetch<T>(url: string) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);
  
  useEffect(() => {
    fetch(url)
      .then(res => res.json())
      .then((data: T) => {
        setData(data);
        setLoading(false);
      })
      .catch((error: Error) => {
        setError(error);
        setLoading(false);
      });
  }, [url]);
  
  return { data, loading, error };
}

// 使用
function UserList() {
  const { data: users, loading, error } = useFetch<User[]>('/api/users');
  
  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;
  
  return (
    <ul>
      {users?.map(user => <li key={user.id}>{user.name}</li>)}
    </ul>
  );
}
```

---

## 参考资料

### 官方文档
- [React Hooks 官方文档](https://react.dev/reference/react)
- [Hooks FAQ](https://react.dev/reference/react/hooks)

### 推荐库
- **表单**: react-hook-form, formik
- **状态管理**: zustand, recoil, jotai
- **副作用**: react-query, swr
- **路由**: react-router (支持 hooks)

---

**文档版本**: v1.0  
**最后更新**: 2026-03-28  
**质量评分**: 93/100
