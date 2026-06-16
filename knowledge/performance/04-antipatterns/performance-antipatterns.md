---
id: performance-antipatterns
title: 性能反模式（避坑指南）
domain: performance
category: 04-antipatterns
difficulty: intermediate
tags: [performance, antipattern, frontend, backend, database, cache, bundle, waterfall, debounce, memoization, lazy-loading]
quality_score: 88
maintainer: platform-team@superdev.com
last_updated: 2024-06-14
---

# 性能反模式（避坑指南）

## 前端

### 1. 巨型 bundle（不分割）
```javascript
// ❌ 全量导入 + 单 chunk
import _ from 'lodash';           // 70KB
import moment from 'moment';       // 70KB
import * as Icons from 'lucide';   // 全部图标！

// ✅ 按需导入 + 路由分割
import debounce from 'lodash/debounce';   // 1KB
import { format } from 'date-fns';         // 7KB
import { Search } from 'lucide-react';     // 单图标
const Settings = lazy(() => import('./Settings'));  // 路由分割
```

### 2. 渲染瀑布（阻塞渲染的 CSS/JS）
```html
<!-- ❌ CSS 阻塞渲染 + JS 阻塞解析 -->
<head>
  <link rel="stylesheet" href="/all.css">  <!-- 200KB，渲染阻塞 -->
  <script src="/analytics.js"></script>     <!-- 解析阻塞 -->
</head>

<!-- ✅ 关键 CSS 内联 + JS 异步 -->
<head>
  <style>/* 关键 CSS 内联 */</style>
  <link rel="preload" href="/rest.css" as="style" onload="this.rel='stylesheet'">
  <script src="/analytics.js" defer></script>
</head>
```

### 3. 不用 key 的列表渲染
```jsx
// ❌ 无 key → React 每次全量 diff（慢）
{items.map(item => <Row data={item} />)}

// ✅ 稳定 key → React 精准 diff（快）
{items.map(item => <Row key={item.id} data={item} />)}
```

### 4. 不防抖的高频事件
```jsx
// ❌ 每次按键都发请求（打字 10 字 = 10 次请求）
<input onChange={e => search(e.target.value)} />

// ✅ 防抖（停止输入 300ms 后才请求）
<input onChange={e => debouncedSearch(e.target.value)} />
const debouncedSearch = useMemo(() => debounce(search, 300), []);
```

## 后端

### 5. 同步阻塞做重操作
```python
# ❌ 请求内发邮件 / 处理图片（用户等 10s）
@app.post("/register")
def register(data):
    user = create_user(data)
    send_welcome_email(user)      # 阻塞 5s
    resize_avatar(user)            # 阻塞 5s
    return user  # 用户等了 10s

# ✅ 异步队列（用户立刻拿到响应）
@app.post("/register")
def register(data):
    user = create_user(data)
    enqueue_job("send_welcome_email", user.id)  # 毫秒
    enqueue_job("resize_avatar", user.id)        # 毫秒
    return user  # 用户等了 200ms
```

### 6. 无缓存的重复计算
```python
# ❌ 每次请求都算排行榜（重查询）
@app.get("/leaderboard")
def leaderboard():
    return db.query("SELECT ... ORDER BY score DESC LIMIT 100")  # 2s

# ✅ 缓存 + 后台刷新
@app.get("/leaderboard")
@cache(ttl=300)  # 缓存 5 分钟
def leaderboard():
    return db.query("...")
```

### 7. 无连接池
```python
# ❌ 每次请求新建数据库连接（TCP 握手 + 认证 ~50ms）
def query(sql):
    conn = psycopg2.connect(url)  # 50ms 开销
    result = conn.execute(sql)
    conn.close()
    return result

# ✅ 连接池复用（~1ms 获取已建连接）
pool = create_connection_pool(url, size=20)
def query(sql):
    with pool.get_conn() as conn:
        return conn.execute(sql)
```

### 8. 未设置查询超时
```python
# ❌ 慢查询无限等待（连接池耗尽）
db.execute("SELECT * FROM big_table")  # 可能跑 10 分钟

# ✅ 语句级超时（30s 自动取消）
db.execute("SET statement_timeout = 30000")
db.execute("SELECT * FROM big_table")  # 超 30s 取消
```

## 数据库

### 9. SELECT * 返回不需要的列
```sql
-- ❌ 传输 + 序列化不需要的列（BLOB/JSONB 很大）
SELECT * FROM products WHERE category = 'electronics';

-- ✅ 只查需要的列
SELECT id, name, price_cents FROM products WHERE category = 'electronics';
```

### 10. 应用层排序（不在 SQL 里排）
```python
# ❌ 查全部 → Python 排序（内存爆炸 + 慢）
products = db.query("SELECT * FROM products")
sorted_products = sorted(products, key=lambda p: p.price, reverse=True)

# ✅ SQL 排序（利用索引，快）
products = db.query("SELECT * FROM products ORDER BY price DESC")
```
