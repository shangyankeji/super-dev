---
id: performance-standards
title: 性能优化标准（前后端）
domain: performance
category: 01-standards
difficulty: advanced
tags: [performance, optimization, database, query, index, cache, frontend, lcp, bundle, cdn, lazy-loading]
quality_score: 92
maintainer: platform-team@superdev.com
last_updated: 2026-06-14
---

# 性能优化标准

## 数据库性能

### 索引策略
```sql
-- B-tree（默认）：等值查询、范围查询、排序
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_created_at ON orders(created_at DESC);

-- 复合索引：多列查询（顺序很重要！最左前缀）
CREATE INDEX idx_orders_user_status ON orders(user_id, status, created_at);

-- GIN：JSONB / 全文搜索 / 数组
CREATE INDEX idx_products_attrs ON products USING GIN(attrs);

-- 部分索引：只索引活跃数据
CREATE INDEX idx_active_orders ON orders(user_id) WHERE status != 'cancelled';
```

### 查询优化
```sql
-- ❌ SELECT * 返回不需要的列
SELECT * FROM users WHERE team_id = 123;

-- ✅ 只选需要的列
SELECT id, name, email FROM users WHERE team_id = 123;

-- ❌ COUNT(*) 做分页 total（大表慢）
SELECT COUNT(*) FROM orders WHERE user_id = 123;

-- ✅ 估算总数（大表）
SELECT reltuples::bigint FROM pg_class WHERE relname = 'orders';

-- ❌ N+1：循环内查子资源
-- ✅ JOIN 或批量 IN
SELECT o.*, u.name FROM orders o JOIN users u ON o.user_id = u.id WHERE o.status = 'pending';
```

### 连接池
```
# PostgreSQL 连接池配置
max_connections: 100
pool_size: 20        # 应用连接池
pool_max_overflow: 10
pool_recycle: 3600   # 1小时回收防泄漏
statement_timeout: 30000  # 30s 超时
```

## 缓存策略

### 多层缓存
```
浏览器缓存 → CDN → 应用缓存(Redis) → 数据库缓存 → 数据库
```

### Redis 缓存模式
```python
# Cache-Aside（最常用）
def get_user(user_id):
    # 1. 查缓存
    cached = redis.get(f"user:{user_id}")
    if cached:
        return json.loads(cached)
    # 2. 查数据库
    user = db.query(User).get(user_id)
    # 3. 写缓存（带 TTL）
    redis.setex(f"user:{user_id}", 300, json.dumps(user.to_dict()))
    return user

# Write-Through（写时更新缓存）
def update_user(user_id, data):
    user = db.update(User, user_id, data)
    redis.setex(f"user:{user_id}", 300, json.dumps(user.to_dict()))
```

### 缓存失效
- TTL 失效（5-60 分钟）
- 主动失效（数据变更时删除缓存键）
- LRU 淘汰（内存不足时）

## 前端性能

### Core Web Vitals 目标
| 指标 | 目标 | 测量 |
|------|------|------|
| LCP（最大内容绘制） | < 2.5s | 首屏最大元素渲染时间 |
| INP（交互延迟） | < 200ms | 用户交互到响应时间 |
| CLS（布局偏移） | < 0.1 | 视觉稳定性 |

### Bundle 优化
```javascript
// ❌ 全量导入
import _ from 'lodash';

// ✅ 按需导入
import debounce from 'lodash/debounce';

// ✅ 动态导入（代码分割）
const Chart = lazy(() => import('./Chart'));

// ✅ 路由级分割
const Dashboard = lazy(() => import('./pages/Dashboard'));
```

### 图片优化
```html
<!-- ❌ 不优化的原图 -->
<img src="/photos/big.jpg">

<!-- ✅ 响应式 + 懒加载 + 现代格式 -->
<img
  src="/photos/medium.avif"
  srcset="/photos/small.avif 480w, /photos/medium.avif 800w, /photos/large.avif 1200w"
  sizes="(max-width: 600px) 480px, 800px"
  loading="lazy"
  decoding="async"
  width="800" height="600"
/>
```

### API 响应优化
```http
# 启用 gzip/brotli 压缩
Content-Encoding: br

# 条件请求（304 Not Modified）
ETag: "abc123"
Cache-Control: public, max-age=300

# 字段过滤
GET /api/users?fields=id,name,email
```

## 性能预算

| 资源 | 预算 |
|------|------|
| JS bundle | < 150KB (gzip) |
| CSS | < 30KB (gzip) |
| 图片 | < 200KB (首屏) |
| 字体 | < 50KB |
| API 响应 | p95 < 200ms |
| 数据库查询 | p95 < 50ms |
