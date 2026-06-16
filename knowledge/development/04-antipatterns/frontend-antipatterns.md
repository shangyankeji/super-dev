---
id: frontend-antipatterns
title: 前端反模式指南
domain: development
category: 04-antipatterns
difficulty: intermediate
tags: [antipatterns, bloat, budget, chaos, component, development, error, frontend]
quality_score: 70
last_updated: 2026-06-15
---
# 前端反模式指南

> 适用范围：React / Vue / Angular / Next.js / Nuxt
> 约束级别：SHALL（必须在 Code Review 阶段拦截）

---

## 1. 组件臃肿（Fat Component）

### 描述
单个组件承载了数据获取、业务逻辑、状态管理、UI 渲染、事件处理等全部职责，导致组件行数膨胀到数百行，难以测试、难以复用。

### 错误示例
```tsx
// OrderPage.tsx -- 500 行的巨型组件
function OrderPage() {
  const [orders, setOrders] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [page, setPage] = useState(1);
  const [filters, setFilters] = useState({});
  const [selectedOrder, setSelectedOrder] = useState(null);
  const [showModal, setShowModal] = useState(false);
  const [sortField, setSortField] = useState("created_at");
  const [sortOrder, setSortOrder] = useState("desc");

  useEffect(() => {
    setLoading(true);
    fetch(`/api/orders?page=${page}&sort=${sortField}&order=${sortOrder}`)
      .then((res) => res.json())
      .then((data) => { setOrders(data.items); setLoading(false); })
      .catch((err) => { setError(err.message); setLoading(false); });
  }, [page, sortField, sortOrder, filters]);

  const handleCancelOrder = async (orderId) => {
    // 30 行的取消逻辑
  };

  const handleExport = async () => {
    // 40 行的导出逻辑
  };

  const calculateTotal = (items) => {
    // 20 行的计算逻辑
  };

  return (
    <div>
      {/* 200 行的 JSX 渲染 */}
    </div>
  );
}
```

### 正确示例
```tsx
// OrderPage.tsx -- 只负责页面组装
function OrderPage() {
  return (
    <PageLayout title="Orders">
      <OrderFilters />
      <OrderList />
      <OrderDetailModal />
    </PageLayout>
  );
}

// hooks/useOrders.ts -- 数据获取逻辑
function useOrders(filters: OrderFilters) {
  return useQuery({
    queryKey: ["orders", filters],
    queryFn: () => orderApi.list(filters),
  });
}

// components/OrderList.tsx -- 只负责列表渲染
function OrderList() {
  const { filters } = useOrderFilters();
  const { data, isLoading, error } = useOrders(filters);

  if (isLoading) return <OrderListSkeleton />;
  if (error) return <ErrorState message={error.message} onRetry={refetch} />;
  if (!data?.items.length) return <EmptyState message="No orders found" />;

  return (
    <div>
      {data.items.map((order) => (
        <OrderCard key={order.id} order={order} />
      ))}
      <Pagination total={data.total} page={data.page} />
    </div>
  );
}
```

### 检测方法
- 组件文件行数 > 200 行。
- 单个组件的 `useState` 调用 > 5 个。
- 组件内包含 `fetch` / `axios` 直接调用。
- 组件内包含复杂的计算逻辑（应提取为 hook 或 util）。

### 修复步骤
1. 将数据获取逻辑提取为自定义 Hook（`useOrders`、`useUser`）。
2. 将业务逻辑提取为独立函数或 Hook。
3. 将 UI 拆分为展示组件（Presentational）和容器组件（Container）。
4. 每个组件只做一件事，文件 <= 150 行。

### Agent Checklist
- [ ] 组件文件行数 <= 200
- [ ] `useState` 调用 <= 5 个（否则用 `useReducer` 或提取 Hook）
- [ ] 组件内无 `fetch` / `axios` 直接调用
- [ ] 有独立的自定义 Hook 管理数据获取

---

## 2. 依赖堆叠（Dependency Bloat）

### 描述
不加节制地引入第三方依赖，每个小功能都用一个库（日期格式化用 moment.js、数组操作用 lodash 全量导入、图标用完整 icon 库），导致 bundle 体积膨胀，首屏加载慢。

### 错误示例
```json
{
  "dependencies": {
    "moment": "^2.29.4",           // 300KB，只用了 format
    "lodash": "^4.17.21",          // 70KB，只用了 debounce
    "antd": "^5.0.0",              // 全量导入
    "@fortawesome/fontawesome-free": "^6.0.0",  // 全量图标
    "jquery": "^3.7.0",            // React 项目不需要
    "axios": "^1.6.0",             // fetch 就够了
    "classnames": "^2.3.0",        // 一行代码能替代
    "uuid": "^9.0.0"               // crypto.randomUUID() 已原生支持
  }
}
```

```javascript
// 全量导入 -- 打包全部模块
import _ from "lodash";
import moment from "moment";
import { Button, Table, Form, Input, Modal, Select, DatePicker } from "antd";

const formatted = moment().format("YYYY-MM-DD");
const debounced = _.debounce(handler, 300);
```

### 正确示例
```json
{
  "dependencies": {
    "date-fns": "^3.0.0"          // 按需导入，tree-shakeable
  }
}
```

```javascript
// 按需导入
import { format } from "date-fns";
import debounce from "lodash/debounce";  // 只导入需要的函数

const formatted = format(new Date(), "yyyy-MM-dd");
const debounced = debounce(handler, 300);

// 原生替代
const id = crypto.randomUUID();                    // 替代 uuid
const classes = [base, active && "active"].filter(Boolean).join(" "); // 替代 classnames

// 动态导入大型组件
const HeavyChart = lazy(() => import("./HeavyChart"));
```

### 检测方法
- `npm run build` 后 bundle 体积 > 500KB（gzipped）。
- `npx bundlephobia` 或 `source-map-explorer` 分析依赖体积。
- `package.json` 中 dependencies 数量 > 30。
- 存在已有原生替代的库（moment -> date-fns/dayjs、lodash -> 原生 ES6）。

### 修复步骤
1. 使用 `source-map-explorer` 或 `webpack-bundle-analyzer` 分析 bundle 组成。
2. 删除未使用的依赖（`npx depcheck`）。
3. 将全量导入改为按需导入（`lodash/debounce` 而非 `lodash`）。
4. 用原生 API 替代小型工具库。
5. 大型组件使用 `React.lazy` 动态加载。
6. 设置 bundle 体积预算，CI 中检查。

### Agent Checklist
- [ ] Bundle 体积 < 500KB gzipped
- [ ] 无全量导入的大型库（lodash / moment / antd）
- [ ] 无原生可替代的小工具库
- [ ] 大型组件使用动态加载
- [ ] CI 包含 bundle 体积检查

---

## 3. 状态管理混乱（State Management Chaos）

### 描述
全局状态和局部状态边界不清，所有数据都放在全局 Store 中（Redux / Vuex），或者 prop drilling 层层传递，或者状态同时存在于多个源（URL、Store、组件本地）导致不一致。

### 错误示例
```tsx
// 所有东西都放全局 Store
const store = createStore({
  user: null,
  orders: [],
  products: [],
  cart: [],
  theme: "light",
  language: "zh",
  modalVisible: false,       // UI 状态不应全局化
  formData: {},              // 表单状态不应全局化
  tooltipPosition: { x: 0 }, // 临时 UI 状态不应全局化
  searchQuery: "",           // 应在 URL 中
  currentPage: 1,            // 应在 URL 中
});

// Prop drilling -- 5 层传递
function App() {
  const [user, setUser] = useState(null);
  return <Layout user={user} setUser={setUser}>
    <Sidebar user={user}>
      <Navigation user={user}>
        <UserMenu user={user} onLogout={() => setUser(null)} />
      </Navigation>
    </Sidebar>
  </Layout>;
}
```

### 正确示例
```tsx
// 状态分层管理

// 1. 服务端状态 -> React Query / SWR
function useUser() {
  return useQuery({ queryKey: ["user"], queryFn: userApi.getCurrent });
}

function useOrders(filters: OrderFilters) {
  return useQuery({ queryKey: ["orders", filters], queryFn: () => orderApi.list(filters) });
}

// 2. URL 状态 -> useSearchParams
function useOrderFilters() {
  const [searchParams, setSearchParams] = useSearchParams();
  return {
    page: Number(searchParams.get("page")) || 1,
    status: searchParams.get("status"),
    setPage: (p: number) => setSearchParams({ ...Object.fromEntries(searchParams), page: String(p) }),
  };
}

// 3. 全局客户端状态 -> Zustand（精简）
const useAuthStore = create<AuthState>((set) => ({
  token: null,
  setToken: (token) => set({ token }),
  logout: () => set({ token: null }),
}));

// 4. 局部 UI 状态 -> useState / useReducer
function OrderDetailModal({ orderId }: { orderId: string }) {
  const [isOpen, setIsOpen] = useState(false); // 局部 UI 状态
  const { data: order } = useOrder(orderId);
  // ...
}

// 5. 跨组件共享（无需全局）-> Context
const ThemeContext = createContext<Theme>("light");
```

### 检测方法
- 全局 Store 包含 UI 临时状态（modal / tooltip / form）。
- 同一数据存在于多个源（Store + URL + 组件内）。
- Props 传递层级 > 3。
- 修改一个状态触发大范围不相关的组件重渲染。

### 修复步骤
1. 分类所有状态：服务端缓存 / URL / 全局客户端 / 局部 UI。
2. 服务端数据迁移到 React Query / SWR。
3. 筛选、分页、搜索参数迁移到 URL。
4. 全局 Store 只保留认证、主题等真正全局的状态。
5. UI 临时状态下沉到使用它的组件内。

### Agent Checklist
- [ ] 全局 Store 只包含认证 / 主题等全局状态
- [ ] 服务端数据使用 React Query / SWR
- [ ] 筛选 / 分页参数在 URL 中
- [ ] Props 传递层级 <= 3
- [ ] 表单 / Modal 状态在组件内局部管理

---

## 4. 只做正常态（Missing Error & Loading States）

### 描述
组件只处理数据正常加载成功的场景，不处理加载中、加载失败、空数据、部分失败等状态，导致用户体验差（白屏、卡死、数据消失无提示）。

### 错误示例
```tsx
function UserProfile({ userId }) {
  const [user, setUser] = useState(null);

  useEffect(() => {
    fetch(`/api/users/${userId}`)
      .then((res) => res.json())
      .then(setUser);
    // 无 loading 状态
    // 无 error 处理
    // 无 404 处理
  }, [userId]);

  // user 为 null 时直接报错：Cannot read property 'name' of null
  return (
    <div>
      <h1>{user.name}</h1>
      <p>{user.email}</p>
    </div>
  );
}
```

### 正确示例
```tsx
function UserProfile({ userId }: { userId: string }) {
  const { data: user, isLoading, error, refetch } = useUser(userId);

  if (isLoading) {
    return <ProfileSkeleton />;  // 骨架屏，不是 spinner
  }

  if (error) {
    if (error.status === 404) {
      return <NotFoundState message="User not found" />;
    }
    return (
      <ErrorState
        title="Failed to load profile"
        message={error.message}
        onRetry={refetch}
      />
    );
  }

  if (!user) {
    return <EmptyState message="No user data available" />;
  }

  return (
    <div>
      <h1>{user.name}</h1>
      <p>{user.email}</p>
      {user.orders?.length ? (
        <OrderList orders={user.orders} />
      ) : (
        <EmptyState message="No orders yet" />
      )}
    </div>
  );
}

// 全局错误边界
class ErrorBoundary extends React.Component {
  state = { hasError: false, error: null };

  static getDerivedStateFromError(error) {
    return { hasError: true, error };
  }

  componentDidCatch(error, errorInfo) {
    errorReporter.capture(error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return <FatalErrorPage error={this.state.error} onReset={() => this.setState({ hasError: false })} />;
    }
    return this.props.children;
  }
}
```

### 检测方法
- 组件直接访问 `data.property` 无空值检查。
- `useEffect` 中的 `fetch` 无 `.catch()` 处理。
- 无 `<ErrorBoundary>` 包裹。
- 页面加载时出现白屏或 console 报错但 UI 无提示。

### 修复步骤
1. 为每个数据获取组件添加 Loading / Error / Empty 三个状态。
2. 使用 React Query / SWR 统一管理数据获取状态。
3. 使用骨架屏（Skeleton）替代 Spinner，提升感知性能。
4. 添加全局 ErrorBoundary 捕获未处理的渲染错误。
5. 创建可复用的 `<ErrorState>` / `<EmptyState>` / `<Skeleton>` 组件。

### Agent Checklist
- [ ] 每个数据组件有 Loading / Error / Empty 状态
- [ ] 无未处理的 fetch 错误
- [ ] 有全局 ErrorBoundary
- [ ] 使用骨架屏而非 Spinner
- [ ] 空数据有友好提示

---

## 5. 性能无预算（Missing Performance Budget）

### 描述
无首屏加载时间目标、无 bundle 体积限制、无渲染性能基线。每次迭代都可能引入性能退化，直到用户投诉才发现。

### 错误示例
```tsx
// 不必要的重渲染
function ProductList({ products, onSelect }) {
  // 每次父组件渲染都创建新的 handler，导致所有子组件重渲染
  return products.map((p) => (
    <ProductCard
      key={p.id}
      product={p}
      onClick={() => onSelect(p.id)}      // 每次创建新函数
      style={{ margin: "10px" }}            // 每次创建新对象
    />
  ));
}

// 未优化的大列表
function ChatMessages({ messages }) {
  // 10000 条消息全部渲染
  return messages.map((msg) => <Message key={msg.id} data={msg} />);
}

// 阻塞主线程的计算
function Dashboard({ data }) {
  // 每次渲染都执行昂贵的计算
  const stats = data.reduce((acc, item) => {
    // 复杂统计计算...
  }, {});
  return <StatsPanel stats={stats} />;
}
```

### 正确示例
```tsx
// 优化重渲染
const ProductCard = memo(function ProductCard({ product, onSelect }) {
  return <div onClick={() => onSelect(product.id)}>{product.name}</div>;
});

function ProductList({ products, onSelect }) {
  const handleSelect = useCallback((id: string) => onSelect(id), [onSelect]);

  return products.map((p) => (
    <ProductCard key={p.id} product={p} onSelect={handleSelect} />
  ));
}

// 虚拟滚动大列表
import { useVirtualizer } from "@tanstack/react-virtual";

function ChatMessages({ messages }) {
  const parentRef = useRef(null);
  const virtualizer = useVirtualizer({
    count: messages.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 60,
  });

  return (
    <div ref={parentRef} style={{ height: "600px", overflow: "auto" }}>
      <div style={{ height: virtualizer.getTotalSize() }}>
        {virtualizer.getVirtualItems().map((virtualItem) => (
          <Message key={virtualItem.key} data={messages[virtualItem.index]} />
        ))}
      </div>
    </div>
  );
}

// 缓存昂贵计算
function Dashboard({ data }) {
  const stats = useMemo(() => computeStats(data), [data]);
  return <StatsPanel stats={stats} />;
}
```

```javascript
// vite.config.ts -- 性能预算
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ["react", "react-dom"],
          ui: ["@radix-ui/react-dialog", "@radix-ui/react-dropdown"],
        },
      },
    },
    chunkSizeWarningLimit: 250, // KB
  },
});
```

### 检测方法
- Lighthouse Performance 评分 < 90。
- LCP (Largest Contentful Paint) > 2.5 秒。
- React DevTools Profiler 显示不必要的重渲染。
- 列表超过 100 条数据时出现明显卡顿。
- `vite build` 后单个 chunk > 250KB。

### 修复步骤
1. 设定性能预算：LCP < 2.5s、FID < 100ms、CLS < 0.1、bundle < 500KB gzipped。
2. 使用 `React.memo` / `useMemo` / `useCallback` 减少不必要的重渲染。
3. 列表超过 50 条使用虚拟滚动。
4. 重型计算使用 Web Worker。
5. 在 CI 中集成 Lighthouse CI，设置性能阈值。

### Agent Checklist
- [ ] Lighthouse Performance >= 90
- [ ] LCP < 2.5s
- [ ] Bundle 主包 < 250KB gzipped
- [ ] 大列表使用虚拟滚动
- [ ] CI 包含 Lighthouse 性能检查

---

## 6. 无可访问性（Missing Accessibility）

### 描述
完全忽略 Web 可访问性（a11y），自定义控件无键盘操作、无 ARIA 标签、无语义化 HTML、颜色对比度不足。导致残障用户无法使用，且在部分地区可能面临法规合规风险。

### 错误示例
```tsx
// 无语义化：全部用 div
<div onClick={handleClick}>Submit</div>              // 不是 button
<div className="nav">                                // 不是 nav
  <div onClick={() => navigate("/")}>Home</div>     // 不是 a/Link
</div>

// 自定义下拉菜单：无键盘支持、无 ARIA
<div className="dropdown" onClick={toggle}>
  <div className="selected">{value}</div>
  {isOpen && (
    <div className="options">
      {options.map((opt) => (
        <div key={opt} onClick={() => select(opt)}>{opt}</div>
      ))}
    </div>
  )}
</div>

// 图片无 alt
<img src={user.avatar} />

// 表单无 label
<input type="email" placeholder="Enter email" />
```

### 正确示例
```tsx
// 语义化 HTML
<button type="button" onClick={handleClick}>Submit</button>

<nav aria-label="Main navigation">
  <Link to="/">Home</Link>
  <Link to="/orders">Orders</Link>
</nav>

// 可访问的下拉菜单（使用 Radix UI 或 Headless UI）
import * as Select from "@radix-ui/react-select";

<Select.Root value={value} onValueChange={onChange}>
  <Select.Trigger aria-label="Select status">
    <Select.Value placeholder="Select..." />
  </Select.Trigger>
  <Select.Content>
    {options.map((opt) => (
      <Select.Item key={opt.value} value={opt.value}>
        <Select.ItemText>{opt.label}</Select.ItemText>
      </Select.Item>
    ))}
  </Select.Content>
</Select.Root>

// 图片有 alt
<img src={user.avatar} alt={`${user.name}'s avatar`} />

// 表单有 label
<label htmlFor="email">Email address</label>
<input id="email" type="email" aria-describedby="email-hint" />
<span id="email-hint">We will never share your email.</span>
```

### 检测方法
- `eslint-plugin-jsx-a11y` 报告 a11y 违规。
- Lighthouse Accessibility 评分 < 90。
- 键盘 Tab 无法导航到所有交互元素。
- `axe-core` 浏览器扩展报告问题。
- `<img>` 标签无 `alt` 属性。

### 修复步骤
1. 启用 `eslint-plugin-jsx-a11y` 规则。
2. 使用语义化 HTML（`button`、`nav`、`main`、`article`、`section`）。
3. 自定义控件使用 Radix UI / Headless UI 等无障碍组件库。
4. 所有图片添加有意义的 `alt` 文本。
5. 所有表单控件关联 `label`。
6. 验证键盘导航和屏幕阅读器体验。

### Agent Checklist
- [ ] Lighthouse Accessibility >= 90
- [ ] 无 `<div onClick>` 替代 `<button>`
- [ ] 所有 `<img>` 有 `alt` 属性
- [ ] 所有表单控件有 `<label>`
- [ ] eslint-plugin-jsx-a11y 无错误

---

## 全局 Agent Checklist

| 检查项 | 阈值 | 工具 |
|--------|------|------|
| 组件文件行数 | <= 200 | ESLint / Code Review |
| Bundle 体积 | < 500KB gzipped | `source-map-explorer` |
| 全局 Store 状态数 | <= 10 | Code Review |
| Loading/Error/Empty 覆盖 | 100% 数据组件 | Code Review |
| Lighthouse Performance | >= 90 | Lighthouse CI |
| Lighthouse Accessibility | >= 90 | Lighthouse CI |
| a11y 违规 | 0 个 | `eslint-plugin-jsx-a11y` / `axe-core` |
