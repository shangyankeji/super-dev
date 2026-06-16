---
id: frontend-antipatterns
title: 前端反模式手册
domain: frontend
category: 04-antipatterns
difficulty: intermediate
tags: [antipatterns, boundaries, component, error, frontend, prop, useeffect, 内联样式滥用]
quality_score: 70
last_updated: 2026-06-15
---
# 前端反模式手册

> 覆盖 React/Vue 应用中最常见的 10 类反模式。
> 每项包含：问题描述、问题代码、修复代码、检测方法。

---

## 1. Prop Drilling（属性透传地狱）

**问题**：数据通过多层中间组件逐级传递，中间组件本身不使用该数据，导致组件耦合度高、维护困难。

### 问题代码

```tsx
// 数据从 App → Layout → Sidebar → UserMenu → Avatar 逐层传递
function App() {
  const [user, setUser] = useState<User>(currentUser);
  return <Layout user={user} onLogout={() => setUser(null)} />;
}

function Layout({ user, onLogout }: { user: User; onLogout: () => void }) {
  // Layout 本身不使用 user，只是中转
  return (
    <div>
      <Sidebar user={user} onLogout={onLogout} />
      <Main />
    </div>
  );
}

function Sidebar({ user, onLogout }: { user: User; onLogout: () => void }) {
  // Sidebar 也不使用，继续传递
  return <UserMenu user={user} onLogout={onLogout} />;
}

function UserMenu({ user, onLogout }: { user: User; onLogout: () => void }) {
  return <Avatar name={user.name} onLogout={onLogout} />;
}
```

### 修复代码

```tsx
// 方案 1：Context（适合低频更新的全局状态）
const AuthContext = createContext<AuthState | null>(null);

function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
}

function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User>(currentUser);
  const logout = useCallback(() => setUser(null), []);
  return (
    <AuthContext.Provider value={{ user, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

// 消费端直接使用，无需中间层传递
function Avatar() {
  const { user, logout } = useAuth();
  return <button onClick={logout}>{user.name}</button>;
}

// 方案 2：组合模式（Composition）
function App() {
  const [user, setUser] = useState<User>(currentUser);
  return (
    <Layout
      sidebar={<UserMenu user={user} onLogout={() => setUser(null)} />}
    >
      <Main />
    </Layout>
  );
}

function Layout({ sidebar, children }: { sidebar: ReactNode; children: ReactNode }) {
  return (
    <div>
      <aside>{sidebar}</aside>
      <main>{children}</main>
    </div>
  );
}
```

**检测方法**：ESLint 规则 `react/jsx-max-depth`；同一 prop 在 3 层以上传递即需重构。

---

## 2. 巨型组件（God Component）

**问题**：单个组件文件超过 300 行，包含多种职责：数据获取、业务逻辑、UI 渲染混杂在一起。

### 问题代码

```tsx
function DashboardPage() {
  const [users, setUsers] = useState([]);
  const [orders, setOrders] = useState([]);
  const [stats, setStats] = useState(null);
  const [filter, setFilter] = useState('all');
  const [sortBy, setSortBy] = useState('date');
  const [page, setPage] = useState(1);
  const [isExporting, setIsExporting] = useState(false);

  useEffect(() => { fetchUsers().then(setUsers); }, []);
  useEffect(() => { fetchOrders(filter, sortBy, page).then(setOrders); }, [filter, sortBy, page]);
  useEffect(() => { fetchStats().then(setStats); }, []);

  const handleExport = async () => {
    setIsExporting(true);
    // 50 行导出逻辑 ...
    setIsExporting(false);
  };

  const filteredUsers = useMemo(() => {
    // 30 行过滤逻辑 ...
  }, [users, filter]);

  // 200+ 行 JSX 渲染：统计卡片、用户表格、订单列表、筛选器、分页、导出按钮 ...
  return <div>{/* ... */}</div>;
}
```

### 修复代码

```tsx
// 拆分为自定义 Hook + 子组件
function useDashboardData(filter: string, sortBy: string, page: number) {
  const { data: users } = useQuery(['users'], fetchUsers);
  const { data: orders } = useQuery(['orders', filter, sortBy, page],
    () => fetchOrders(filter, sortBy, page));
  const { data: stats } = useQuery(['stats'], fetchStats);
  return { users, orders, stats };
}

function useExport() {
  const [isExporting, setIsExporting] = useState(false);
  const handleExport = useCallback(async (data: Order[]) => {
    setIsExporting(true);
    try { await exportOrders(data); }
    finally { setIsExporting(false); }
  }, []);
  return { isExporting, handleExport };
}

function DashboardPage() {
  const [filter, setFilter] = useState('all');
  const [sortBy, setSortBy] = useState('date');
  const [page, setPage] = useState(1);
  const { users, orders, stats } = useDashboardData(filter, sortBy, page);
  const { isExporting, handleExport } = useExport();

  return (
    <div>
      <StatsCards stats={stats} />
      <FilterBar filter={filter} onFilterChange={setFilter} />
      <UserTable users={users} />
      <OrderList orders={orders} sortBy={sortBy} onSortChange={setSortBy} />
      <Pagination page={page} onPageChange={setPage} />
      <ExportButton loading={isExporting} onClick={() => handleExport(orders)} />
    </div>
  );
}
```

**检测方法**：设置 ESLint `max-lines-per-function` 为 200；超过即触发拆分。

---

## 3. useEffect 滥用

**问题**：将 useEffect 当作"当 X 改变时做 Y"的通用监听器，导致不必要的渲染循环、竞态条件、难以追踪的 bug。

### 问题代码

```tsx
// 反模式 1：用 useEffect 派生状态
function ProductPage({ productId }: { productId: string }) {
  const [product, setProduct] = useState(null);
  const [price, setPrice] = useState(0);

  useEffect(() => {
    fetchProduct(productId).then(setProduct);
  }, [productId]);

  // 多余的 useEffect：可直接计算
  useEffect(() => {
    if (product) {
      setPrice(product.basePrice * (1 - product.discount));
    }
  }, [product]);

  return <div>{price}</div>;
}

// 反模式 2：用 useEffect 响应事件
function SearchForm() {
  const [query, setQuery] = useState('');
  const [submitted, setSubmitted] = useState(false);

  useEffect(() => {
    if (submitted) {
      doSearch(query);
      setSubmitted(false);
    }
  }, [submitted, query]);

  return <button onClick={() => setSubmitted(true)}>搜索</button>;
}
```

### 修复代码

```tsx
// 修复 1：直接计算派生值，无需 useEffect
function ProductPage({ productId }: { productId: string }) {
  const { data: product } = useQuery(['product', productId],
    () => fetchProduct(productId));

  // 直接计算，无需额外 state 和 effect
  const price = product ? product.basePrice * (1 - product.discount) : 0;

  return <div>{price}</div>;
}

// 修复 2：在事件处理函数中直接执行
function SearchForm() {
  const [query, setQuery] = useState('');

  const handleSubmit = () => {
    doSearch(query);  // 直接在事件中调用，无需 effect
  };

  return <button onClick={handleSubmit}>搜索</button>;
}
```

**检测方法**：ESLint 插件 `eslint-plugin-react-hooks`；审查所有 useEffect，如果 effect 内只是 setState 且依赖另一个 state，通常可改为 useMemo 或直接计算。

---

## 4. 过度重渲染

**问题**：父组件状态变化导致所有子组件无差别重渲染，在大型列表或复杂组件树中造成明显卡顿。

### 问题代码

```tsx
function ParentList() {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [items, setItems] = useState<Item[]>(initialItems);

  // 每次 selectedId 变化，所有 ListItem 都重渲染
  return (
    <ul>
      {items.map(item => (
        <ListItem
          key={item.id}
          item={item}
          isSelected={item.id === selectedId}
          onSelect={() => setSelectedId(item.id)}  // 每次创建新函数
        />
      ))}
    </ul>
  );
}

function ListItem({ item, isSelected, onSelect }: ListItemProps) {
  console.log('render', item.id); // 每次都执行
  return (
    <li onClick={onSelect}
        style={{ background: isSelected ? '#eef' : 'white' }}>
      {item.name}
    </li>
  );
}
```

### 修复代码

```tsx
function ParentList() {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [items, setItems] = useState<Item[]>(initialItems);

  const handleSelect = useCallback((id: string) => {
    setSelectedId(id);
  }, []);

  return (
    <ul>
      {items.map(item => (
        <MemoizedListItem
          key={item.id}
          item={item}
          isSelected={item.id === selectedId}
          onSelect={handleSelect}
        />
      ))}
    </ul>
  );
}

// memo 阻止不必要的重渲染
const MemoizedListItem = memo(function ListItem({
  item, isSelected, onSelect,
}: ListItemProps) {
  return (
    <li onClick={() => onSelect(item.id)}
        style={{ background: isSelected ? '#eef' : 'white' }}>
      {item.name}
    </li>
  );
});
```

**检测方法**：React DevTools Profiler 的 "Why did this render?" 功能；`<React.StrictMode>` 下 double-render 检查。

---

## 5. 内联样式滥用

**问题**：大量使用内联 style 对象，导致无法复用、无法响应媒体查询、每次渲染创建新对象引发重渲染。

### 问题代码

```tsx
function Card({ highlighted }: { highlighted: boolean }) {
  return (
    <div style={{
      padding: '16px',
      borderRadius: '8px',
      boxShadow: '0 2px 8px rgba(0,0,0,0.1)',
      backgroundColor: highlighted ? '#fff3cd' : '#ffffff',
      transition: 'all 0.3s ease',
      // 无法定义 hover 状态
      // 无法使用媒体查询
      // 每次渲染创建新对象 → 子组件 memo 失效
    }}>
      <h3 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '8px' }}>
        Title
      </h3>
    </div>
  );
}
```

### 修复代码

```tsx
// 方案 1：CSS Modules
import styles from './Card.module.css';
import clsx from 'clsx';

function Card({ highlighted }: { highlighted: boolean }) {
  return (
    <div className={clsx(styles.card, highlighted && styles.highlighted)}>
      <h3 className={styles.title}>Title</h3>
    </div>
  );
}

/* Card.module.css */
/*
.card {
  padding: 16px;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  background-color: #ffffff;
  transition: all 0.3s ease;
}
.card:hover { box-shadow: 0 4px 16px rgba(0,0,0,0.15); }
.highlighted { background-color: #fff3cd; }
.title { font-size: 18px; font-weight: 600; margin-bottom: 8px; }

@media (max-width: 768px) {
  .card { padding: 12px; }
  .title { font-size: 16px; }
}
*/

// 方案 2：Tailwind CSS
function Card({ highlighted }: { highlighted: boolean }) {
  return (
    <div className={clsx(
      'p-4 rounded-lg shadow-md transition-all hover:shadow-lg',
      highlighted ? 'bg-yellow-50' : 'bg-white'
    )}>
      <h3 className="text-lg font-semibold mb-2">Title</h3>
    </div>
  );
}
```

**检测方法**：ESLint 规则 `react/no-inline-styles`（自定义）；代码审查时关注 `style={{` 模式。

---

## 6. 无错误边界（Missing Error Boundaries）

**问题**：任何子组件的渲染错误会导致整个应用白屏崩溃，用户无法操作也无法获知原因。

### 问题代码

```tsx
// 没有错误边界，任何子组件异常 → 整个页面白屏
function App() {
  return (
    <div>
      <Header />
      <Dashboard />  {/* 如果 Dashboard 内部抛错，整个 App 崩溃 */}
      <Footer />
    </div>
  );
}

function Dashboard() {
  const data = useData();
  // 如果 data.items 为 undefined，渲染时抛错
  return (
    <ul>
      {data.items.map(item => <li key={item.id}>{item.name}</li>)}
    </ul>
  );
}
```

### 修复代码

```tsx
// 通用错误边界组件
class ErrorBoundary extends Component<
  { children: ReactNode; fallback?: ReactNode; onError?: (error: Error) => void },
  { hasError: boolean; error: Error | null }
> {
  state = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // 上报错误监控
    reportError(error, errorInfo);
    this.props.onError?.(error);
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback || (
        <div role="alert" className="error-fallback">
          <h2>页面出现异常</h2>
          <p>{this.state.error?.message}</p>
          <button onClick={() => this.setState({ hasError: false, error: null })}>
            重试
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

// 分区域包裹，互不影响
function App() {
  return (
    <div>
      <Header />
      <ErrorBoundary fallback={<DashboardError />}>
        <Dashboard />
      </ErrorBoundary>
      <ErrorBoundary fallback={<div>侧边栏加载失败</div>}>
        <Sidebar />
      </ErrorBoundary>
      <Footer />
    </div>
  );
}

// 同时做防御性编程
function Dashboard() {
  const data = useData();
  return (
    <ul>
      {(data?.items ?? []).map(item => <li key={item.id}>{item.name}</li>)}
    </ul>
  );
}
```

**检测方法**：搜索项目中 `ErrorBoundary` 使用次数；关键路由至少一个。

---

## 7. 直接操作 DOM

**问题**：在 React/Vue 中直接使用 `document.querySelector`、`document.getElementById` 等操作 DOM，绕过虚拟 DOM 机制，导致状态不同步、内存泄漏。

### 问题代码

```tsx
function Modal({ isOpen }: { isOpen: boolean }) {
  useEffect(() => {
    if (isOpen) {
      // 直接操作 DOM，绕过 React
      document.body.style.overflow = 'hidden';
      document.getElementById('modal-overlay')!.classList.add('visible');
      document.querySelector('.modal-content')!.focus();
    } else {
      document.body.style.overflow = '';
      document.getElementById('modal-overlay')!.classList.remove('visible');
    }
  }, [isOpen]);

  return (
    <div id="modal-overlay">
      <div className="modal-content" tabIndex={-1}>Content</div>
    </div>
  );
}
```

### 修复代码

```tsx
function Modal({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  const contentRef = useRef<HTMLDivElement>(null);

  // body overflow 通过 effect 管理，清理时恢复
  useEffect(() => {
    if (isOpen) {
      const original = document.body.style.overflow;
      document.body.style.overflow = 'hidden';
      return () => { document.body.style.overflow = original; };
    }
  }, [isOpen]);

  // 聚焦通过 ref
  useEffect(() => {
    if (isOpen && contentRef.current) {
      contentRef.current.focus();
    }
  }, [isOpen]);

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-center justify-center"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
    >
      <div
        ref={contentRef}
        tabIndex={-1}
        className="bg-white rounded-lg p-6"
        onClick={e => e.stopPropagation()}
      >
        Content
      </div>
    </div>
  );
}
```

**检测方法**：在代码库中搜索 `document.getElementById`、`document.querySelector`、`document.getElementsBy`；除极少数场景（如 portal、第三方库集成），均应替换为 ref。

---

## 8. 无代码分割（Monolithic Bundle）

**问题**：整个应用打包为单个 JS 文件，首次加载传输数百 KB 甚至数 MB 的 JavaScript，导致首屏时间极长。

### 问题代码

```tsx
// 所有页面在入口文件静态导入
import Dashboard from './pages/Dashboard';
import Settings from './pages/Settings';
import Analytics from './pages/Analytics';  // 引入了 chart.js（500KB）
import AdminPanel from './pages/AdminPanel';
import UserProfile from './pages/UserProfile';
// ... 20 个页面全部静态导入

import { marked } from 'marked';           // 非首屏需要
import hljs from 'highlight.js';           // 非首屏需要
import * as XLSX from 'xlsx';              // 导出功能才用

function App() {
  return (
    <Routes>
      <Route path="/" element={<Dashboard />} />
      <Route path="/settings" element={<Settings />} />
      <Route path="/analytics" element={<Analytics />} />
      {/* ... */}
    </Routes>
  );
}
```

### 修复代码

```tsx
import { lazy, Suspense } from 'react';

// 路由级懒加载
const Dashboard = lazy(() => import('./pages/Dashboard'));
const Settings = lazy(() => import('./pages/Settings'));
const Analytics = lazy(() => import(
  /* webpackChunkName: "analytics" */ './pages/Analytics'
));
const AdminPanel = lazy(() => import('./pages/AdminPanel'));

// 重型库按需导入
async function exportToExcel(data: unknown[]) {
  const XLSX = await import('xlsx');
  const worksheet = XLSX.utils.json_to_sheet(data);
  const workbook = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(workbook, worksheet, 'Sheet1');
  XLSX.writeFile(workbook, 'export.xlsx');
}

function App() {
  return (
    <Suspense fallback={<PageSkeleton />}>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="/analytics" element={<Analytics />} />
        <Route path="/admin" element={<AdminPanel />} />
      </Routes>
    </Suspense>
  );
}
```

**检测方法**：`npx vite-bundle-visualizer`；首屏 JS 传输体积 > 200KB（gzip）即需分割。

---

## 9. CSS-in-JS 过度使用

**问题**：在高频渲染路径中使用运行时 CSS-in-JS（styled-components/emotion），每次渲染动态生成样式字符串并注入 DOM，造成性能开销。

### 问题代码

```tsx
import styled from 'styled-components';

// 列表中每个 item 都有动态样式 → 高频渲染时性能差
const ListItem = styled.li<{ isActive: boolean; color: string }>`
  padding: 8px 16px;
  background: ${props => props.isActive ? props.color : 'transparent'};
  border-left: 3px solid ${props => props.isActive ? props.color : 'transparent'};
  opacity: ${props => props.isActive ? 1 : 0.7};
  transition: all 0.2s;
  &:hover {
    background: ${props => props.color}22;
  }
`;

function BigList({ items }: { items: Item[] }) {
  return (
    <ul>
      {items.map(item => (
        // 1000 个 item，每个都生成不同的样式类
        <ListItem key={item.id} isActive={item.active} color={item.color}>
          {item.name}
        </ListItem>
      ))}
    </ul>
  );
}
```

### 修复代码

```tsx
// 方案 1：CSS Modules + CSS 变量（零运行时开销）
import styles from './BigList.module.css';

function BigList({ items }: { items: Item[] }) {
  return (
    <ul>
      {items.map(item => (
        <li
          key={item.id}
          className={`${styles.item} ${item.active ? styles.active : ''}`}
          style={{ '--item-color': item.color } as CSSProperties}
        >
          {item.name}
        </li>
      ))}
    </ul>
  );
}

/* BigList.module.css */
/*
.item {
  padding: 8px 16px;
  opacity: 0.7;
  transition: all 0.2s;
  border-left: 3px solid transparent;
}
.item:hover { background: color-mix(in srgb, var(--item-color) 13%, transparent); }
.active {
  background: var(--item-color);
  border-left-color: var(--item-color);
  opacity: 1;
}
*/

// 方案 2：如果必须用 CSS-in-JS，选择零运行时方案
// 如 vanilla-extract、Linaria、Panda CSS
import { style } from '@vanilla-extract/css';

export const item = style({
  padding: '8px 16px',
  opacity: 0.7,
  transition: 'all 0.2s',
});
```

**检测方法**：React DevTools Profiler 中观察 styled 组件渲染耗时；列表场景中比较 CSS Modules vs styled-components 的帧率。

---

## 10. 无类型检查（No Type Safety）

**问题**：整个项目使用纯 JavaScript 或大量 `any`/`as any`，编译期无法捕获类型错误，只能在运行时暴露。

### 问题代码

```tsx
// 全 any，IDE 无法提供补全和错误提示
function processOrder(order: any) {
  const total = order.items.reduce((sum: any, item: any) => {
    return sum + item.price * item.qty;
  }, 0);

  // 拼写错误，运行时才发现
  return {
    orderId: order.id,
    toal: total,  // typo: toal → total
    stauts: 'confirmed',  // typo: stauts → status
  };
}

// API 响应无类型
async function fetchUser(id: string) {
  const res = await fetch(`/api/users/${id}`);
  const data = await res.json(); // data: any
  return data;
}

// 事件处理无类型
function handleClick(e: any) {
  console.log(e.terget.value); // typo: terget → target，运行时才报错
}
```

### 修复代码

```tsx
// 定义明确类型
interface OrderItem {
  id: string;
  name: string;
  price: number;
  qty: number;
}

interface Order {
  id: string;
  items: OrderItem[];
  customer: Customer;
}

interface OrderResult {
  orderId: string;
  total: number;
  status: 'confirmed' | 'pending' | 'cancelled';
}

function processOrder(order: Order): OrderResult {
  const total = order.items.reduce(
    (sum, item) => sum + item.price * item.qty,
    0
  );

  return {
    orderId: order.id,
    total,            // 类型检查确保字段名正确
    status: 'confirmed',
  };
}

// API 响应类型化
interface UserResponse {
  id: string;
  name: string;
  email: string;
}

async function fetchUser(id: string): Promise<UserResponse> {
  const res = await fetch(`/api/users/${id}`);
  if (!res.ok) throw new ApiError(res.status, await res.text());
  return res.json() as Promise<UserResponse>;
}

// 运行时校验（zod）
import { z } from 'zod';

const UserSchema = z.object({
  id: z.string(),
  name: z.string(),
  email: z.string().email(),
});

async function fetchUserSafe(id: string) {
  const res = await fetch(`/api/users/${id}`);
  const data = await res.json();
  return UserSchema.parse(data); // 运行时类型校验
}

// 事件类型
function handleClick(e: React.MouseEvent<HTMLButtonElement>) {
  console.log(e.currentTarget.value); // IDE 自动补全，编译期检查
}
```

**检测方法**：

```bash
# 统计 any 使用数量
grep -r ":\s*any" --include="*.ts" --include="*.tsx" src/ | wc -l

# TypeScript 严格模式
# tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "noUncheckedIndexedAccess": true
  }
}

# ESLint 规则
# @typescript-eslint/no-explicit-any: error
# @typescript-eslint/no-unsafe-assignment: warn
```

---

## Agent Checklist

- [ ] 项目中无超过 3 层的 Prop Drilling（必要时使用 Context 或状态管理）
- [ ] 单个组件文件不超过 300 行；超过则拆分为子组件 + 自定义 Hook
- [ ] 所有 useEffect 已审查：无派生状态计算、无事件响应逻辑
- [ ] 列表渲染使用 `React.memo` + `useCallback` 防止不必要重渲染
- [ ] 无大范围内联样式；使用 CSS Modules / Tailwind / 零运行时 CSS-in-JS
- [ ] 关键路由均有 ErrorBoundary 包裹
- [ ] 无直接 DOM 操作（`document.querySelector` 等），统一使用 ref
- [ ] 路由级和重型库已做代码分割（`lazy` + 动态 `import()`）
- [ ] 高频渲染路径无运行时 CSS-in-JS 性能问题
- [ ] TypeScript `strict: true` 已启用；`any` 使用不超过总类型注解的 5%
- [ ] 以上各项均有对应 ESLint 规则或 CI 检查保障
