---
id: postgresql-production-playbook
title: PostgreSQL 生产环境调优手册
domain: database
category: 02-playbooks
difficulty: advanced
tags: [database, postgresql, postgres, production, tuning, index, query, optimization, partitioning, vacuum, connection-pool, configuration]
quality_score: 94
maintainer: platform-team@superdev.com
last_updated: 2026-06-14
---

# PostgreSQL 生产环境调优手册

> 基于 [Instaclustr Top 15 PostgreSQL Best Practices](https://www.instaclustr.com/education/postgresql/top-10-postgresql-best-practices-for-2025/) + [Mydbops Tuning Guide](https://www.mydbops.com/blog/postgresql-parameter-tuning-best-practices)

## 索引策略

### 索引类型选择
| 类型 | 场景 | 示例 |
|------|------|------|
| B-tree（默认） | 等值/范围/排序 | `WHERE id = 123` / `ORDER BY created_at` |
| GIN | JSONB/全文/数组 | `WHERE attrs @> '{"color":"red"}'` |
| GiST | 地理/范围 | `WHERE point <@ box` |
| BRIN | 大表时间序列 | 百万行日志按时间分区 |
| Hash | 纯等值 | `WHERE id = 123`（PG 10+ WAL 支持）|

### 复合索引列顺序
```sql
-- 最左前缀：等值列在前，范围列在后
-- 查询: WHERE user_id = ? AND created_at > ?
CREATE INDEX idx_orders_user_created ON orders(user_id, created_at DESC);
-- ✅ user_id（等值）在前，created_at（范围）在后

-- ❌ 反过来：created_at 在前无法用 user_id 过滤
CREATE INDEX idx_orders_bad ON orders(created_at, user_id);
```

### 部分索引（省空间 + 快）
```sql
-- 只索引活跃数据（90% 订单已完成，不需要索引）
CREATE INDEX idx_active_orders ON orders(user_id) WHERE status = 'pending';
```

## 查询优化

### EXPLAIN ANALYZE 诊断
```sql
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM orders WHERE user_id = 123 ORDER BY created_at DESC LIMIT 20;

-- 看关键指标:
-- Seq Scan → 全表扫描（需要索引）
-- Index Scan → 索引扫描（好）
-- Bitmap Heap Scan → 批量索引（还行）
-- Sort → 排序（可能需要索引优化）
-- actual time → 真实耗时（毫秒）
-- rows → 实际返回行数 vs 预估行数（差异大 = 统计信息过期）
```

### 大表分页优化
```sql
-- ❌ OFFSET 10000 = 扫描 10020 行后丢弃前 10000
SELECT * FROM products ORDER BY id OFFSET 10000 LIMIT 20;

-- ✅ Keyset 分页 = WHERE 直接定位 20 行
SELECT * FROM products WHERE id > 5000 ORDER BY id LIMIT 20;

-- ✅ 游标分页（排序复杂时）
SELECT * FROM products
WHERE (created_at, id) < ('2024-06-14', 5000)
ORDER BY created_at DESC, id DESC LIMIT 20;
```

## 服务器调优

### 关键参数
```ini
# postgresql.conf — 按 RAM 调整
shared_buffers = 4GB          # 总 RAM 的 25%
effective_cache_size = 12GB   # 总 RAM 的 75%
work_mem = 64MB               # 排序/哈希内存（注意：每连接×每操作）
maintenance_work_mem = 1GB    # VACUUM/CREATE INDEX
max_connections = 100         # 配合连接池用

# WAL（写入日志）
wal_buffers = 16MB
checkpoint_completion_target = 0.9
max_wal_size = 4GB

# Autovacuum（自动清理）
autovacuum = on
autovacuum_naptime = 30s
autovacuum_vacuum_scale_factor = 0.1   # 变更 10% 后触发
```

### 连接池
```
# PgBouncer（必须！直接连数据库会耗尽连接）
[databases]
app = host=localhost port=5432 dbname=app

[pgbouncer]
pool_mode = transaction     # 事务级池化
max_client_conn = 500       # 客户端连接
default_pool_size = 25      # 到 PostgreSQL 的连接
```

## 分区（大表 > 1000 万行）

```sql
-- 按范围分区（时间序列）
CREATE TABLE events (
    id BIGSERIAL,
    created_at TIMESTAMPTZ NOT NULL,
    data JSONB
) PARTITION BY RANGE (created_at);

CREATE TABLE events_2024_06 PARTITION OF events
    FOR VALUES FROM ('2024-06-01') TO ('2024-07-01');

-- 按列表分区（租户隔离）
CREATE TABLE orders (
    id UUID DEFAULT gen_random_uuid(),
    tenant_id TEXT NOT NULL,
    ...
) PARTITION BY LIST (tenant_id);

CREATE TABLE orders_tenant_a PARTITION OF orders
    FOR VALUES IN ('tenant_a');
```

## VACUUM 与膨胀

```sql
-- 检查表膨胀
SELECT relname, n_live_tup, n_dead_tup,
       round(n_dead_tup::numeric / NULLIF(n_live_tup, 0) * 100, 2) AS bloat_pct
FROM pg_stat_user_tables
WHERE n_live_tup > 10000
ORDER BY bloat_pct DESC;

-- 膨胀 > 20% 需要处理
VACUUM (ANALYZE, VERBOSE) orders;  -- 常规清理
VACUUM FULL orders;                 -- 锁表重建（生产慎用！）
```
