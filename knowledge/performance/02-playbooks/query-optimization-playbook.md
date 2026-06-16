---
id: query-optimization-playbook
title: 数据库查询优化实战手册
domain: performance
category: 02-playbooks
difficulty: advanced
tags: [performance, database, postgresql, query, optimization, index, explain, analyze, n-plus-1, pagination, cache]
quality_score: 92
maintainer: platform-team@superdev.com
last_updated: 2024-06-14
---

# 数据库查询优化实战手册

## 诊断工具：EXPLAIN ANALYZE

```sql
-- 查看真实执行计划（实际跑一遍）
EXPLAIN ANALYZE
SELECT * FROM orders WHERE user_id = 123 ORDER BY created_at DESC LIMIT 20;

-- 关键看：
-- Seq Scan（全表扫描）→ 需要索引
-- Index Scan（索引扫描）→ 好
-- Bitmap Heap Scan（批量索引）→ 还行
-- Sort（内存排序）→ 可能需要索引
-- Nested Loop（JOIN）→ 大表可能慢
```

## 常见优化模式

### 1. 全表扫描 → 加索引
```sql
-- EXPLAIN 显示 Seq Scan
EXPLAIN ANALYZE SELECT * FROM orders WHERE status = 'pending';
-- → 加索引
CREATE INDEX idx_orders_status ON orders(status);
```

### 2. 大 OFFSET 分页 → cursor 分页
```sql
-- ❌ OFFSET 10000 扫 10020 行
SELECT * FROM products ORDER BY id OFFSET 10000 LIMIT 20;

-- ✅ WHERE 直接定位（扫 20 行）
SELECT * FROM products WHERE id > 5000 ORDER BY id LIMIT 20;
```

### 3. N+1 → 批量查询
```python
# ❌ 100 用户 = 101 次 DB 查询
for u in users:
    u.profile = db.query("SELECT * FROM profiles WHERE user_id=?", u.id)

# ✅ 1 次查全部
profiles = db.query("SELECT * FROM profiles WHERE user_id IN %s", user_ids)
```

### 4. COUNT(*) 慢 → 估算
```sql
-- ❌ 精确 COUNT（大表几秒）
SELECT COUNT(*) FROM orders WHERE status = 'pending';

-- ✅ 统计信息估算（毫秒）
SELECT reltuples::bigint FROM pg_class WHERE relname='orders';

-- ✅ 或维护一个计数器表
UPDATE counters SET count = count + 1 WHERE table_name = 'orders' AND status = 'pending';
```

### 5. 慢 JOIN → 物化视图 / 缓存
```sql
-- ❌ 复杂 JOIN 每次都算
SELECT o.*, u.name, p.title FROM orders o
JOIN users u ON o.user_id = u.id
JOIN products p ON o.product_id = p.id
WHERE o.status = 'pending';

-- ✅ 物化视图定期刷新
CREATE MATERIALIZED VIEW order_summary AS
SELECT o.*, u.name, p.title FROM ...;
REFRESH MATERIALIZED VIEW CONCURRENTLY order_summary;
```

## 连接池配置

```python
# SQLAlchemy 示例
engine = create_engine(url,
    pool_size=20,          # 常驻连接数
    max_overflow=10,       # 突发连接
    pool_pre_ping=True,    # 使用前检查连接活性
    pool_recycle=3600,     # 1 小时回收（防 "server closed the connection"）
)
```

## 查询超时

```sql
-- PostgreSQL 语句级超时
SET statement_timeout = '30s';
SELECT ...;  -- 超过 30s 自动取消
```
