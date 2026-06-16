---
id: postgresql-complete
title: PostgreSQL 数据工程完整指南
domain: data
category: 01-standards
difficulty: intermediate
tags: [complete, data, postgresql, 备份与恢复, 安全, 数据建模, 查询优化, 核心概念]
quality_score: 70
last_updated: 2026-06-15
---
# PostgreSQL 数据工程完整指南

> 文档版本: v1.0 | 最后更新: 2026-03-28 | 适用范围: PostgreSQL 14-17

## 1. 概述

PostgreSQL 是世界上最先进的开源关系型数据库管理系统，具备 ACID 完全合规、MVCC 并发控制、可扩展类型系统和丰富的索引能力。在数据工程场景中，PostgreSQL 承担 OLTP 核心存储、数据仓库辅助层、实时分析和 JSON 文档存储等多重角色。

### 1.1 适用场景

| 场景 | 推荐度 | 说明 |
|------|--------|------|
| OLTP 业务系统 | ★★★★★ | 事务完整性、行级锁、外键约束 |
| JSONB 文档存储 | ★★★★☆ | 替代部分 MongoDB 场景 |
| 地理信息系统 | ★★★★★ | PostGIS 扩展 |
| 全文搜索 | ★★★★☆ | tsvector/tsquery，中小规模可替代 ES |
| 大规模 OLAP | ★★★☆☆ | 需配合 Citus / TimescaleDB 或分区表 |
| 时序数据 | ★★★★☆ | TimescaleDB 扩展 |

### 1.2 版本选择

```sql
-- 查看当前版本
SELECT version();

-- 推荐版本:
-- 生产环境: PostgreSQL 16 (当前稳定长期支持)
-- 新项目: PostgreSQL 17 (最新特性)
-- 遗留系统: PostgreSQL 14+ (最低安全维护线)
```

---

## 2. 核心概念

### 2.1 MVCC (多版本并发控制)

PostgreSQL 通过 MVCC 实现读写不阻塞。每行数据包含 `xmin`（创建事务ID）和 `xmax`（删除事务ID）两个隐藏列，事务根据快照可见性判断数据是否可见。

```sql
-- 查看行的 MVCC 元信息
SELECT xmin, xmax, ctid, * FROM orders LIMIT 5;

-- ctid 是行的物理位置 (页号, 行号)
-- xmin 是插入该行的事务ID
-- xmax 是删除/更新该行的事务ID (0 表示未被删除)

-- 模拟 MVCC 行为
BEGIN;
  UPDATE accounts SET balance = balance - 100 WHERE id = 1;
  -- 此时旧版本仍对其他事务可见
  -- 新版本仅对当前事务可见
COMMIT;
-- 提交后，旧版本标记为可回收，新版本对所有后续事务可见
```

**MVCC 的代价 -- 膨胀 (Bloat):**

```sql
-- 死元组积累导致表膨胀，需要 VACUUM 回收空间
-- 查看表的死元组数量
SELECT relname, n_dead_tup, n_live_tup,
       round(n_dead_tup::numeric / GREATEST(n_live_tup, 1) * 100, 2) AS dead_ratio_pct
FROM pg_stat_user_tables
WHERE n_dead_tup > 1000
ORDER BY n_dead_tup DESC;

-- 手动触发 VACUUM
VACUUM VERBOSE orders;

-- VACUUM FULL 会重写整张表（锁表，慎用）
VACUUM FULL orders;

-- 推荐: 调优 autovacuum 参数而非手动执行
ALTER TABLE orders SET (
    autovacuum_vacuum_threshold = 50,
    autovacuum_vacuum_scale_factor = 0.05,   -- 5% 死元组即触发
    autovacuum_analyze_threshold = 50,
    autovacuum_analyze_scale_factor = 0.02
);
```

### 2.2 WAL (预写日志)

WAL 是 PostgreSQL 持久性的基石。所有数据修改先写入 WAL，再异步刷入数据文件，确保崩溃后可完整恢复。

```sql
-- 查看当前 WAL 位置
SELECT pg_current_wal_lsn(), pg_current_wal_insert_lsn();

-- 查看 WAL 文件大小设置
SHOW wal_segment_size;  -- 默认 16MB

-- 查看 WAL 级别
SHOW wal_level;  -- minimal / replica / logical

-- 生产环境必须设为 replica 或 logical
-- postgresql.conf
-- wal_level = replica          # 支持物理复制
-- wal_level = logical          # 支持逻辑复制 + CDC

-- 查看 WAL 写入统计
SELECT * FROM pg_stat_wal;

-- 查看 WAL 归档状态
SELECT * FROM pg_stat_archiver;
```

### 2.3 事务隔离级别

PostgreSQL 支持四种 SQL 标准隔离级别，实际实现三种（READ UNCOMMITTED 等同 READ COMMITTED）。

```sql
-- 查看当前隔离级别
SHOW transaction_isolation;  -- 默认 read committed

-- 设置会话级隔离
SET SESSION CHARACTERISTICS AS TRANSACTION ISOLATION LEVEL REPEATABLE READ;

-- 设置单事务隔离
BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE;
  -- 最严格：可序列化，防止所有异常
  -- 但性能开销最大，可能触发序列化失败需重试
  SELECT * FROM inventory WHERE product_id = 42 FOR UPDATE;
  UPDATE inventory SET quantity = quantity - 1 WHERE product_id = 42;
COMMIT;
```

**各隔离级别对比:**

| 隔离级别 | 脏读 | 不可重复读 | 幻读 | 序列化异常 | 适用场景 |
|----------|------|-----------|------|-----------|---------|
| READ COMMITTED | 否 | 可能 | 可能 | 可能 | 默认，大多数 OLTP |
| REPEATABLE READ | 否 | 否 | 否* | 可能 | 报表、一致性快照 |
| SERIALIZABLE | 否 | 否 | 否 | 否 | 金融、库存扣减 |

> *PostgreSQL 的 REPEATABLE READ 通过快照隔离同时防止了幻读。

```sql
-- 序列化失败重试模式（应用层必须实现）
-- Python 伪代码:
-- for attempt in range(MAX_RETRIES):
--     try:
--         with conn.cursor() as cur:
--             cur.execute("BEGIN ISOLATION LEVEL SERIALIZABLE")
--             cur.execute("SELECT ... FOR UPDATE")
--             cur.execute("UPDATE ...")
--             cur.execute("COMMIT")
--         break
--     except psycopg2.errors.SerializationFailure:
--         conn.rollback()
--         continue
```

### 2.4 锁机制

```sql
-- 查看当前锁等待
SELECT blocked_locks.pid     AS blocked_pid,
       blocked_activity.usename  AS blocked_user,
       blocking_locks.pid    AS blocking_pid,
       blocking_activity.usename AS blocking_user,
       blocked_activity.query    AS blocked_statement,
       blocking_activity.query   AS current_statement_in_blocking_process
FROM pg_catalog.pg_locks blocked_locks
JOIN pg_catalog.pg_stat_activity blocked_activity
  ON blocked_activity.pid = blocked_locks.pid
JOIN pg_catalog.pg_locks blocking_locks
  ON blocking_locks.locktype = blocked_locks.locktype
  AND blocking_locks.database IS NOT DISTINCT FROM blocked_locks.database
  AND blocking_locks.relation IS NOT DISTINCT FROM blocked_locks.relation
  AND blocking_locks.page IS NOT DISTINCT FROM blocked_locks.page
  AND blocking_locks.tuple IS NOT DISTINCT FROM blocked_locks.tuple
  AND blocking_locks.virtualxid IS NOT DISTINCT FROM blocked_locks.virtualxid
  AND blocking_locks.transactionid IS NOT DISTINCT FROM blocked_locks.transactionid
  AND blocking_locks.classid IS NOT DISTINCT FROM blocked_locks.classid
  AND blocking_locks.objid IS NOT DISTINCT FROM blocked_locks.objid
  AND blocking_locks.objsubid IS NOT DISTINCT FROM blocked_locks.objsubid
  AND blocking_locks.pid != blocked_locks.pid
JOIN pg_catalog.pg_stat_activity blocking_activity
  ON blocking_activity.pid = blocking_locks.pid
WHERE NOT blocked_locks.granted;

-- 设置锁超时（避免无限等待）
SET lock_timeout = '5s';

-- 使用 advisory lock 实现应用级互斥
SELECT pg_advisory_lock(hashtext('order_processing'));
-- ... 执行业务逻辑 ...
SELECT pg_advisory_unlock(hashtext('order_processing'));

-- 非阻塞方式
SELECT pg_try_advisory_lock(hashtext('order_processing'));
```

---

## 3. 数据建模

### 3.1 范式化设计 (3NF)

```sql
-- 规范的三范式设计
CREATE TABLE customers (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    email       VARCHAR(255) UNIQUE NOT NULL,
    name        VARCHAR(100) NOT NULL,
    phone       VARCHAR(20),
    created_at  TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE addresses (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    customer_id BIGINT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    type        VARCHAR(20) NOT NULL CHECK (type IN ('billing', 'shipping')),
    line1       VARCHAR(255) NOT NULL,
    line2       VARCHAR(255),
    city        VARCHAR(100) NOT NULL,
    state       VARCHAR(50),
    postal_code VARCHAR(20) NOT NULL,
    country     CHAR(2) NOT NULL,  -- ISO 3166-1 alpha-2
    UNIQUE (customer_id, type)
);

CREATE TABLE orders (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    customer_id BIGINT NOT NULL REFERENCES customers(id),
    address_id  BIGINT NOT NULL REFERENCES addresses(id),
    status      VARCHAR(20) NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending','confirmed','shipped','delivered','cancelled')),
    total       NUMERIC(12, 2) NOT NULL CHECK (total >= 0),
    ordered_at  TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE order_items (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    order_id    BIGINT NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id  BIGINT NOT NULL,
    quantity    INTEGER NOT NULL CHECK (quantity > 0),
    unit_price  NUMERIC(10, 2) NOT NULL CHECK (unit_price >= 0),
    subtotal    NUMERIC(12, 2) GENERATED ALWAYS AS (quantity * unit_price) STORED
);
```

### 3.2 反范式化设计

```sql
-- 为读密集场景添加冗余字段
ALTER TABLE orders ADD COLUMN customer_name VARCHAR(100);
ALTER TABLE orders ADD COLUMN item_count INTEGER DEFAULT 0;

-- 使用触发器维护冗余数据一致性
CREATE OR REPLACE FUNCTION update_order_item_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE orders SET item_count = item_count + 1 WHERE id = NEW.order_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE orders SET item_count = item_count - 1 WHERE id = OLD.order_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_order_item_count
AFTER INSERT OR DELETE ON order_items
FOR EACH ROW EXECUTE FUNCTION update_order_item_count();

-- 物化视图用于重聚合场景
CREATE MATERIALIZED VIEW mv_daily_sales AS
SELECT
    date_trunc('day', o.ordered_at)::DATE AS sale_date,
    count(DISTINCT o.id) AS order_count,
    sum(o.total) AS revenue,
    count(DISTINCT o.customer_id) AS unique_customers
FROM orders o
WHERE o.status != 'cancelled'
GROUP BY 1;

CREATE UNIQUE INDEX idx_mv_daily_sales_date ON mv_daily_sales(sale_date);

-- 定时刷新（配合 pg_cron 或外部调度）
REFRESH MATERIALIZED VIEW CONCURRENTLY mv_daily_sales;
```

### 3.3 JSONB 混合建模

```sql
-- 结构化 + 半结构化混合
CREATE TABLE products (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    sku         VARCHAR(50) UNIQUE NOT NULL,
    name        VARCHAR(255) NOT NULL,
    category    VARCHAR(100) NOT NULL,
    price       NUMERIC(10, 2) NOT NULL,
    -- 半结构化属性 (颜色、尺寸、规格等因品类而异)
    attributes  JSONB NOT NULL DEFAULT '{}',
    tags        TEXT[] DEFAULT '{}',
    created_at  TIMESTAMPTZ DEFAULT now()
);

-- JSONB 查询示例
-- 查找红色产品
SELECT * FROM products
WHERE attributes @> '{"color": "red"}';

-- 查找包含特定嵌套属性的产品
SELECT * FROM products
WHERE attributes -> 'specs' ->> 'weight_kg' IS NOT NULL
  AND (attributes -> 'specs' ->> 'weight_kg')::numeric < 5.0;

-- JSONB 聚合
SELECT
    attributes ->> 'brand' AS brand,
    count(*) AS product_count,
    avg(price) AS avg_price
FROM products
WHERE attributes ? 'brand'
GROUP BY attributes ->> 'brand'
ORDER BY product_count DESC;

-- 更新 JSONB 字段
UPDATE products
SET attributes = jsonb_set(
    attributes,
    '{specs, weight_kg}',
    '2.5'::jsonb
)
WHERE sku = 'PROD-001';

-- 删除 JSONB 中的键
UPDATE products
SET attributes = attributes - 'deprecated_field'
WHERE attributes ? 'deprecated_field';

-- 生成列基于 JSONB（PostgreSQL 12+）
ALTER TABLE products ADD COLUMN brand VARCHAR(100)
    GENERATED ALWAYS AS (attributes ->> 'brand') STORED;
```

---

## 4. 索引策略

### 4.1 B-tree 索引（默认）

```sql
-- 最常用，适合等值和范围查询
CREATE INDEX idx_orders_customer ON orders(customer_id);
CREATE INDEX idx_orders_status_date ON orders(status, ordered_at DESC);

-- 验证索引是否被使用
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT * FROM orders WHERE customer_id = 42 AND status = 'shipped';
```

### 4.2 GIN 索引（通用倒排）

```sql
-- JSONB 查询加速
CREATE INDEX idx_products_attrs ON products USING GIN (attributes);
CREATE INDEX idx_products_attrs_path ON products USING GIN (attributes jsonb_path_ops);
-- jsonb_path_ops 更小更快，但仅支持 @> 操作符

-- 数组查询加速
CREATE INDEX idx_products_tags ON products USING GIN (tags);

-- 全文搜索
ALTER TABLE products ADD COLUMN search_vector tsvector
    GENERATED ALWAYS AS (
        setweight(to_tsvector('simple', coalesce(name, '')), 'A') ||
        setweight(to_tsvector('simple', coalesce(category, '')), 'B')
    ) STORED;
CREATE INDEX idx_products_search ON products USING GIN (search_vector);

-- 全文搜索查询
SELECT name, ts_rank(search_vector, query) AS rank
FROM products, to_tsquery('simple', '无线 & 耳机') AS query
WHERE search_vector @@ query
ORDER BY rank DESC;
```

### 4.3 GiST 索引（通用搜索树）

```sql
-- 范围类型查询
CREATE TABLE reservations (
    id       BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    room_id  INTEGER NOT NULL,
    period   TSTZRANGE NOT NULL,
    EXCLUDE USING GIST (room_id WITH =, period WITH &&)
    -- 排他约束: 同一房间不允许时间重叠
);

-- 地理空间查询 (需要 PostGIS)
-- CREATE INDEX idx_locations_geom ON locations USING GIST (geom);
-- SELECT * FROM locations WHERE ST_DWithin(geom, ST_MakePoint(116.4, 39.9)::geography, 1000);

-- 最近邻搜索
CREATE TABLE points (id serial, coords point);
CREATE INDEX idx_points_coords ON points USING GIST (coords);
SELECT * FROM points ORDER BY coords <-> point '(3,4)' LIMIT 10;
```

### 4.4 BRIN 索引（块范围索引）

```sql
-- 适合物理有序的大表 (时序数据、日志表)
-- 体积极小 (B-tree 的 1/100)，扫描时跳过不相关的块
CREATE TABLE event_logs (
    id         BIGINT GENERATED ALWAYS AS IDENTITY,
    event_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    event_type VARCHAR(50),
    payload    JSONB
);

CREATE INDEX idx_logs_time_brin ON event_logs USING BRIN (event_time)
    WITH (pages_per_range = 32);

-- 查询只扫描相关块
EXPLAIN ANALYZE
SELECT * FROM event_logs
WHERE event_time BETWEEN '2026-03-01' AND '2026-03-28';
```

### 4.5 部分索引

```sql
-- 只索引活跃数据，减小索引体积
CREATE INDEX idx_orders_active ON orders(customer_id, ordered_at)
WHERE status NOT IN ('cancelled', 'delivered');

-- 唯一性约束仅对活跃记录生效
CREATE UNIQUE INDEX idx_users_active_email ON users(email)
WHERE deleted_at IS NULL;
```

### 4.6 覆盖索引 (INCLUDE)

```sql
-- 索引包含额外列，避免回表 (Index-Only Scan)
CREATE INDEX idx_orders_covering ON orders(customer_id, ordered_at DESC)
INCLUDE (status, total);

-- 此查询可完全从索引中获取数据
EXPLAIN ANALYZE
SELECT customer_id, ordered_at, status, total
FROM orders
WHERE customer_id = 42
ORDER BY ordered_at DESC
LIMIT 20;
```

### 4.7 索引维护

```sql
-- 查看索引大小和使用情况
SELECT
    schemaname, tablename, indexname,
    pg_size_pretty(pg_relation_size(indexrelid)) AS index_size,
    idx_scan AS times_used,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
ORDER BY pg_relation_size(indexrelid) DESC;

-- 找出未使用的索引 (运行一段时间后检查)
SELECT indexrelid::regclass AS index_name,
       relid::regclass AS table_name,
       pg_size_pretty(pg_relation_size(indexrelid)) AS size
FROM pg_stat_user_indexes
WHERE idx_scan = 0
  AND indexrelid NOT IN (
      SELECT conindid FROM pg_constraint WHERE contype IN ('p', 'u')
  )
ORDER BY pg_relation_size(indexrelid) DESC;

-- 在线重建索引 (不锁表)
REINDEX INDEX CONCURRENTLY idx_orders_customer;
```

---

## 5. 查询优化

### 5.1 EXPLAIN ANALYZE 深度解读

```sql
-- 基本用法
EXPLAIN (ANALYZE, BUFFERS, COSTS, TIMING, FORMAT TEXT)
SELECT o.id, o.total, c.name
FROM orders o
JOIN customers c ON c.id = o.customer_id
WHERE o.ordered_at >= '2026-01-01'
  AND o.status = 'shipped'
ORDER BY o.ordered_at DESC
LIMIT 50;

-- 关键指标解读:
-- actual time: 首行返回时间..最后行返回时间 (毫秒)
-- rows: 实际返回行数 (对比 Plan rows 判断统计信息准确性)
-- Buffers: shared hit (缓存命中) / shared read (磁盘读)
-- 理想情况: hit >> read

-- 强制更新统计信息
ANALYZE orders;
ANALYZE customers;

-- 查看表级统计
SELECT attname, n_distinct, most_common_vals, histogram_bounds
FROM pg_stats
WHERE tablename = 'orders' AND attname = 'status';
```

### 5.2 CTE (公用表表达式)

```sql
-- 可读性优先的复杂查询拆解
WITH monthly_revenue AS (
    SELECT
        date_trunc('month', ordered_at)::DATE AS month,
        sum(total) AS revenue,
        count(*) AS order_count
    FROM orders
    WHERE status != 'cancelled'
      AND ordered_at >= '2025-01-01'
    GROUP BY 1
),
growth AS (
    SELECT
        month,
        revenue,
        order_count,
        lag(revenue) OVER (ORDER BY month) AS prev_revenue,
        round(
            (revenue - lag(revenue) OVER (ORDER BY month))
            / NULLIF(lag(revenue) OVER (ORDER BY month), 0) * 100, 2
        ) AS growth_pct
    FROM monthly_revenue
)
SELECT * FROM growth ORDER BY month;

-- 递归 CTE: 树形结构遍历
WITH RECURSIVE category_tree AS (
    -- 锚点: 顶级分类
    SELECT id, name, parent_id, 1 AS depth, name::TEXT AS path
    FROM categories WHERE parent_id IS NULL

    UNION ALL

    -- 递归: 子分类
    SELECT c.id, c.name, c.parent_id, ct.depth + 1,
           ct.path || ' > ' || c.name
    FROM categories c
    JOIN category_tree ct ON ct.id = c.parent_id
    WHERE ct.depth < 10  -- 防止无限递归
)
SELECT * FROM category_tree ORDER BY path;

-- PostgreSQL 12+ CTE 默认非物化 (可被内联优化)
-- 强制物化: WITH cte AS MATERIALIZED (...)
-- 强制内联: WITH cte AS NOT MATERIALIZED (...)
```

### 5.3 窗口函数

```sql
-- 排名与分析
SELECT
    customer_id,
    ordered_at,
    total,
    -- 同组内排名
    row_number() OVER w AS rn,
    rank()       OVER w AS rnk,
    dense_rank() OVER w AS dense_rnk,
    -- 累计和
    sum(total) OVER (PARTITION BY customer_id ORDER BY ordered_at) AS running_total,
    -- 移动平均
    avg(total) OVER (
        PARTITION BY customer_id
        ORDER BY ordered_at
        ROWS BETWEEN 2 PRECEDING AND CURRENT ROW
    ) AS moving_avg_3,
    -- 首尾值
    first_value(total) OVER w AS first_order_total,
    -- 与前一笔差值
    total - lag(total, 1, 0) OVER w AS diff_from_prev
FROM orders
WHERE status != 'cancelled'
WINDOW w AS (PARTITION BY customer_id ORDER BY ordered_at)
ORDER BY customer_id, ordered_at;

-- Top-N per group (每个客户最近3笔订单)
SELECT * FROM (
    SELECT *, row_number() OVER (
        PARTITION BY customer_id ORDER BY ordered_at DESC
    ) AS rn
    FROM orders
    WHERE status != 'cancelled'
) sub
WHERE rn <= 3;
```

### 5.4 分区表

```sql
-- 按时间范围分区 (最常见)
CREATE TABLE events (
    id          BIGINT GENERATED ALWAYS AS IDENTITY,
    event_time  TIMESTAMPTZ NOT NULL,
    event_type  VARCHAR(50) NOT NULL,
    payload     JSONB,
    PRIMARY KEY (id, event_time)
) PARTITION BY RANGE (event_time);

-- 创建月分区
CREATE TABLE events_2026_01 PARTITION OF events
    FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');
CREATE TABLE events_2026_02 PARTITION OF events
    FOR VALUES FROM ('2026-02-01') TO ('2026-03-01');
CREATE TABLE events_2026_03 PARTITION OF events
    FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');

-- 默认分区 (兜底)
CREATE TABLE events_default PARTITION OF events DEFAULT;

-- 自动创建未来分区的函数
CREATE OR REPLACE FUNCTION create_monthly_partition(
    parent_table TEXT,
    target_month DATE
) RETURNS VOID AS $$
DECLARE
    partition_name TEXT;
    start_date DATE;
    end_date DATE;
BEGIN
    start_date := date_trunc('month', target_month);
    end_date := start_date + INTERVAL '1 month';
    partition_name := parent_table || '_' || to_char(start_date, 'YYYY_MM');

    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF %I FOR VALUES FROM (%L) TO (%L)',
        partition_name, parent_table, start_date, end_date
    );
END;
$$ LANGUAGE plpgsql;

-- 按列表分区 (多租户场景)
CREATE TABLE tenant_data (
    id        BIGINT GENERATED ALWAYS AS IDENTITY,
    tenant_id INTEGER NOT NULL,
    data      JSONB,
    PRIMARY KEY (id, tenant_id)
) PARTITION BY LIST (tenant_id);

CREATE TABLE tenant_data_1 PARTITION OF tenant_data FOR VALUES IN (1);
CREATE TABLE tenant_data_2 PARTITION OF tenant_data FOR VALUES IN (2);

-- 分区裁剪验证
SET enable_partition_pruning = on;  -- 默认开启
EXPLAIN ANALYZE
SELECT * FROM events WHERE event_time BETWEEN '2026-03-01' AND '2026-03-28';
-- 应该只扫描 events_2026_03 分区
```

---

## 6. 高可用

### 6.1 流复制 (Streaming Replication)

```sql
-- 主库配置 (postgresql.conf)
-- wal_level = replica
-- max_wal_senders = 10
-- wal_keep_size = 1GB
-- synchronous_standby_names = 'standby1'  -- 同步复制

-- 主库创建复制用户
CREATE ROLE replicator WITH REPLICATION LOGIN PASSWORD 'strong_password_here';

-- pg_hba.conf 添加
-- host replication replicator standby_ip/32 scram-sha-256
```

```bash
# 从库初始化
pg_basebackup -h primary_host -U replicator -D /var/lib/postgresql/data \
    --wal-method=stream --checkpoint=fast --progress --verbose

# 从库配置 (postgresql.conf)
# primary_conninfo = 'host=primary_host port=5432 user=replicator password=...'
# hot_standby = on
```

```sql
-- 监控复制状态（在主库执行）
SELECT
    client_addr,
    state,
    sent_lsn,
    write_lsn,
    flush_lsn,
    replay_lsn,
    pg_wal_lsn_diff(sent_lsn, replay_lsn) AS replication_lag_bytes,
    sync_state
FROM pg_stat_replication;

-- 监控复制延迟（在从库执行）
SELECT
    now() - pg_last_xact_replay_timestamp() AS replication_delay,
    pg_is_in_recovery() AS is_standby,
    pg_last_wal_receive_lsn(),
    pg_last_wal_replay_lsn();
```

### 6.2 Patroni 高可用集群

```yaml
# patroni.yml 核心配置
scope: pg-cluster
name: node1

restapi:
  listen: 0.0.0.0:8008
  connect_address: node1:8008

etcd3:
  hosts: etcd1:2379,etcd2:2379,etcd3:2379

bootstrap:
  dcs:
    ttl: 30
    loop_wait: 10
    retry_timeout: 10
    maximum_lag_on_failover: 1048576  # 1MB
    postgresql:
      use_pg_rewind: true
      parameters:
        max_connections: 200
        shared_buffers: 4GB
        wal_level: replica
        max_wal_senders: 10
        hot_standby: on

postgresql:
  listen: 0.0.0.0:5432
  connect_address: node1:5432
  data_dir: /var/lib/postgresql/data
  authentication:
    superuser:
      username: postgres
      password: "${POSTGRES_PASSWORD}"
    replication:
      username: replicator
      password: "${REPLICATION_PASSWORD}"
```

```bash
# Patroni 集群状态
patronictl -c /etc/patroni.yml list

# 手动切换主库 (计划内维护)
patronictl -c /etc/patroni.yml switchover --master node1 --candidate node2

# 故障转移 (紧急)
patronictl -c /etc/patroni.yml failover
```

### 6.3 逻辑复制

```sql
-- 发布端 (源库)
CREATE PUBLICATION pub_orders FOR TABLE orders, order_items;
-- 或发布所有表
CREATE PUBLICATION pub_all FOR ALL TABLES;

-- 订阅端 (目标库)
CREATE SUBSCRIPTION sub_orders
    CONNECTION 'host=source_host dbname=mydb user=replicator password=...'
    PUBLICATION pub_orders;

-- 监控逻辑复制
SELECT * FROM pg_stat_subscription;
SELECT * FROM pg_replication_slots;
```

---

## 7. 备份与恢复

### 7.1 pg_dump / pg_restore

```bash
# 逻辑备份 - 自定义格式 (推荐，可并行恢复)
pg_dump -h localhost -U postgres -Fc -Z 6 -j 4 \
    --no-owner --no-privileges \
    -f /backup/mydb_$(date +%Y%m%d_%H%M%S).dump mydb

# 逻辑备份 - 纯 SQL (可读性好，不支持并行恢复)
pg_dump -h localhost -U postgres --schema-only -f schema.sql mydb
pg_dump -h localhost -U postgres --data-only -f data.sql mydb

# 恢复到新库
createdb -h localhost -U postgres mydb_restored
pg_restore -h localhost -U postgres -d mydb_restored -j 4 \
    --no-owner --no-privileges \
    /backup/mydb_20260328.dump

# 只恢复特定表
pg_restore -h localhost -U postgres -d mydb_restored \
    -t orders -t order_items \
    /backup/mydb_20260328.dump
```

### 7.2 pg_basebackup (物理备份)

```bash
# 全量物理备份
pg_basebackup -h primary_host -U replicator \
    -D /backup/base_$(date +%Y%m%d) \
    --wal-method=stream \
    --checkpoint=fast \
    --compress=gzip:6 \
    --progress --verbose

# 备份到 tar 格式
pg_basebackup -h primary_host -U replicator \
    -D /backup/ --format=tar --gzip \
    --wal-method=stream
```

### 7.3 PITR (时间点恢复)

```bash
# 前提: 开启 WAL 归档
# postgresql.conf:
# archive_mode = on
# archive_command = 'cp %p /archive/%f'
# 或使用 pgbackrest / barman

# 恢复步骤:
# 1. 停止 PostgreSQL
# 2. 备份当前数据目录
# 3. 从 base backup 还原数据目录
# 4. 创建 recovery 配置

# postgresql.conf (PostgreSQL 12+):
# restore_command = 'cp /archive/%f %p'
# recovery_target_time = '2026-03-28 10:30:00+08'
# recovery_target_action = 'promote'
```

```sql
-- 创建还原点 (在重大变更前)
SELECT pg_create_restore_point('before_migration_v2');

-- 恢复到指定还原点
-- recovery_target_name = 'before_migration_v2'
-- recovery_target_action = 'promote'
```

---

## 8. 安全

### 8.1 角色与权限

```sql
-- 创建应用角色 (最小权限原则)
CREATE ROLE app_readonly WITH LOGIN PASSWORD 'readonly_pass';
CREATE ROLE app_readwrite WITH LOGIN PASSWORD 'readwrite_pass';
CREATE ROLE app_admin WITH LOGIN PASSWORD 'admin_pass';

-- 只读角色
GRANT CONNECT ON DATABASE mydb TO app_readonly;
GRANT USAGE ON SCHEMA public TO app_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO app_readonly;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT ON TABLES TO app_readonly;

-- 读写角色
GRANT CONNECT ON DATABASE mydb TO app_readwrite;
GRANT USAGE ON SCHEMA public TO app_readwrite;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO app_readwrite;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO app_readwrite;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO app_readwrite;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT USAGE, SELECT ON SEQUENCES TO app_readwrite;

-- 禁止直接用 superuser 连接应用
-- pg_hba.conf:
-- local  all  postgres  peer
-- host   all  postgres  127.0.0.1/32  reject
```

### 8.2 行级安全 (RLS)

```sql
-- 启用 RLS
ALTER TABLE tenant_data ENABLE ROW LEVEL SECURITY;

-- 多租户隔离策略
CREATE POLICY tenant_isolation ON tenant_data
    USING (tenant_id = current_setting('app.current_tenant')::INTEGER);

-- 应用层设置租户上下文
SET app.current_tenant = '42';
SELECT * FROM tenant_data;  -- 只能看到 tenant_id=42 的数据

-- 管理员绕过策略
CREATE POLICY admin_all ON tenant_data
    TO app_admin
    USING (true);

-- 写入策略 (WITH CHECK 控制可插入/更新的行)
CREATE POLICY tenant_insert ON tenant_data
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant')::INTEGER);

-- 强制 RLS 对表属主也生效
ALTER TABLE tenant_data FORCE ROW LEVEL SECURITY;
```

### 8.3 SSL/TLS 连接

```bash
# postgresql.conf
# ssl = on
# ssl_cert_file = '/etc/ssl/certs/server.crt'
# ssl_key_file = '/etc/ssl/private/server.key'
# ssl_ca_file = '/etc/ssl/certs/ca.crt'
# ssl_min_protocol_version = 'TLSv1.3'

# pg_hba.conf - 强制 SSL
# hostssl  all  all  0.0.0.0/0  scram-sha-256

# 客户端连接
# psql "host=db.example.com dbname=mydb user=app sslmode=verify-full sslrootcert=ca.crt"
```

```sql
-- 验证 SSL 连接
SELECT ssl, version, cipher FROM pg_stat_ssl
WHERE pid = pg_backend_pid();

-- 查看所有连接的 SSL 状态
SELECT s.pid, s.usename, s.client_addr, l.ssl, l.version, l.cipher
FROM pg_stat_activity s
LEFT JOIN pg_stat_ssl l ON s.pid = l.pid
WHERE s.state = 'active';
```

### 8.4 审计日志

```sql
-- 使用 pgaudit 扩展
CREATE EXTENSION IF NOT EXISTS pgaudit;

-- postgresql.conf
-- pgaudit.log = 'write, ddl'
-- pgaudit.log_catalog = off
-- pgaudit.log_relation = on

-- 对特定角色开启审计
ALTER ROLE app_readwrite SET pgaudit.log = 'write';
```

---

## 9. 监控

### 9.1 pg_stat_statements

```sql
-- 启用扩展
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- postgresql.conf
-- shared_preload_libraries = 'pg_stat_statements'
-- pg_stat_statements.max = 10000
-- pg_stat_statements.track = all

-- Top 10 慢查询
SELECT
    queryid,
    calls,
    round(total_exec_time::numeric, 2) AS total_ms,
    round(mean_exec_time::numeric, 2) AS avg_ms,
    round(stddev_exec_time::numeric, 2) AS stddev_ms,
    rows,
    round((shared_blks_hit * 100.0 /
        NULLIF(shared_blks_hit + shared_blks_read, 0))::numeric, 2) AS cache_hit_pct,
    left(query, 120) AS query_preview
FROM pg_stat_statements
ORDER BY total_exec_time DESC
LIMIT 10;

-- Top 10 IO 密集查询
SELECT
    queryid,
    calls,
    shared_blks_read,
    shared_blks_written,
    temp_blks_read + temp_blks_written AS temp_blks_total,
    left(query, 120) AS query_preview
FROM pg_stat_statements
ORDER BY shared_blks_read + shared_blks_written DESC
LIMIT 10;

-- 重置统计 (定期执行以保持时效性)
SELECT pg_stat_statements_reset();
```

### 9.2 pg_stat_activity

```sql
-- 当前活跃连接
SELECT
    pid,
    usename,
    client_addr,
    state,
    wait_event_type,
    wait_event,
    now() - query_start AS query_duration,
    left(query, 100) AS query_preview
FROM pg_stat_activity
WHERE state != 'idle'
  AND pid != pg_backend_pid()
ORDER BY query_start;

-- 查找长事务（超过5分钟）
SELECT
    pid,
    usename,
    now() - xact_start AS xact_duration,
    state,
    left(query, 150) AS query
FROM pg_stat_activity
WHERE xact_start IS NOT NULL
  AND now() - xact_start > INTERVAL '5 minutes'
ORDER BY xact_start;

-- 终止问题查询
SELECT pg_cancel_backend(pid);      -- 温和取消 (仅取消当前查询)
SELECT pg_terminate_backend(pid);   -- 强制终止 (断开连接)

-- 批量终止空闲超时连接
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE state = 'idle'
  AND now() - state_change > INTERVAL '30 minutes'
  AND usename != 'postgres';
```

### 9.3 表和索引健康度

```sql
-- 表级统计全景
SELECT
    schemaname, relname,
    seq_scan, seq_tup_read,
    idx_scan, idx_tup_fetch,
    n_tup_ins, n_tup_upd, n_tup_del,
    n_live_tup, n_dead_tup,
    last_vacuum, last_autovacuum,
    last_analyze, last_autoanalyze
FROM pg_stat_user_tables
ORDER BY n_dead_tup DESC;

-- 缓存命中率 (应 > 99%)
SELECT
    sum(heap_blks_read) AS heap_read,
    sum(heap_blks_hit) AS heap_hit,
    round(sum(heap_blks_hit) * 100.0 /
        NULLIF(sum(heap_blks_hit) + sum(heap_blks_read), 0), 2) AS cache_hit_pct
FROM pg_statio_user_tables;

-- 索引缓存命中率
SELECT
    sum(idx_blks_read) AS idx_read,
    sum(idx_blks_hit) AS idx_hit,
    round(sum(idx_blks_hit) * 100.0 /
        NULLIF(sum(idx_blks_hit) + sum(idx_blks_read), 0), 2) AS idx_cache_hit_pct
FROM pg_statio_user_indexes;

-- 表和索引膨胀估算
SELECT
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname || '.' || tablename)) AS total_size,
    pg_size_pretty(pg_relation_size(schemaname || '.' || tablename)) AS table_size,
    pg_size_pretty(pg_indexes_size(schemaname || '.' || tablename)) AS index_size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname || '.' || tablename) DESC;
```

---

## 10. 连接池 (PgBouncer)

### 10.1 配置

```ini
; pgbouncer.ini
[databases]
mydb = host=127.0.0.1 port=5432 dbname=mydb

[pgbouncer]
listen_addr = 0.0.0.0
listen_port = 6432
auth_type = scram-sha-256
auth_file = /etc/pgbouncer/userlist.txt

; 连接池模式
pool_mode = transaction    ; 推荐: 事务级复用
; pool_mode = session      ; 会话级 (兼容性最好但效率最低)
; pool_mode = statement    ; 语句级 (不支持事务)

; 连接数控制
default_pool_size = 25
max_client_conn = 1000
min_pool_size = 5
reserve_pool_size = 5
reserve_pool_timeout = 3

; 超时设置
server_idle_timeout = 600
client_idle_timeout = 0
query_timeout = 300
client_login_timeout = 60

; 日志
log_connections = 1
log_disconnections = 1
log_pooler_errors = 1
stats_period = 60
```

### 10.2 监控

```sql
-- 连接 PgBouncer 管理端口
-- psql -h 127.0.0.1 -p 6432 -U pgbouncer pgbouncer

-- 查看连接池状态
SHOW POOLS;

-- 查看客户端连接
SHOW CLIENTS;

-- 查看后端服务器连接
SHOW SERVERS;

-- 统计信息
SHOW STATS;

-- 内存使用
SHOW MEM;
```

### 10.3 Transaction 模式注意事项

```sql
-- 以下功能在 transaction 模式下不可用:
-- 1. SET 命令 (会话级变量)
-- 2. PREPARE / DEALLOCATE (预备语句)
-- 3. LISTEN / NOTIFY
-- 4. LOAD
-- 5. 会话级 advisory lock

-- 解决 SET 问题: 使用函数内的局部设置
-- 错误:
SET work_mem = '256MB';
SELECT ... complex query ...;

-- 正确:
SET LOCAL work_mem = '256MB';  -- 仅在当前事务内生效
SELECT ... complex query ...;
COMMIT;

-- 或使用函数:
CREATE FUNCTION heavy_query() RETURNS SETOF record AS $$
BEGIN
    SET LOCAL work_mem = '256MB';
    RETURN QUERY SELECT ...;
END;
$$ LANGUAGE plpgsql;
```

---

## 11. 性能调优

### 11.1 内存参数

```sql
-- 查看当前设置
SELECT name, setting, unit, context, short_desc
FROM pg_settings
WHERE name IN (
    'shared_buffers', 'effective_cache_size', 'work_mem',
    'maintenance_work_mem', 'wal_buffers', 'huge_pages'
);

-- postgresql.conf 推荐设置:

-- shared_buffers: 总内存的 25% (上限不超过 8GB)
-- 16GB 内存 -> shared_buffers = 4GB
-- shared_buffers = '4GB'

-- effective_cache_size: 总内存的 75% (仅影响查询规划器的成本估算)
-- effective_cache_size = '12GB'

-- work_mem: 每个排序/哈希操作的内存上限
-- 注意: 每个连接的每个操作都独立分配
-- 总消耗 ≈ work_mem × max_connections × 操作数
-- work_mem = '64MB'   -- 复杂分析查询
-- work_mem = '16MB'   -- OLTP 为主

-- maintenance_work_mem: VACUUM、CREATE INDEX 等维护操作
-- maintenance_work_mem = '1GB'

-- wal_buffers: WAL 缓冲区，-1 表示自动 (shared_buffers 的 1/32)
-- wal_buffers = '64MB'

-- huge_pages: 减少 TLB miss (Linux 需配置 vm.nr_hugepages)
-- huge_pages = try
```

### 11.2 WAL 和 Checkpoint 调优

```sql
-- checkpoint_completion_target: 0.9 (将 checkpoint 写入分散在整个间隔内)
-- checkpoint_timeout: 15min  (两次 checkpoint 的最大间隔)
-- max_wal_size: 4GB  (触发 checkpoint 的 WAL 累积量上限)
-- min_wal_size: 1GB

-- 监控 checkpoint 频率
SELECT * FROM pg_stat_bgwriter;
-- 关注: checkpoints_timed (正常) vs checkpoints_req (被迫)
-- 被迫 checkpoint 过多说明 max_wal_size 太小

-- WAL 写入优化
-- wal_compression = on   -- 压缩全页写入 (减少 WAL 体积)
-- full_page_writes = on  -- 必须开启 (数据安全)
```

### 11.3 查询规划器调优

```sql
-- 并行查询
-- max_parallel_workers_per_gather = 4
-- max_parallel_workers = 8
-- parallel_tuple_cost = 0.01
-- parallel_setup_cost = 100

-- 验证并行执行
EXPLAIN ANALYZE
SELECT count(*) FROM orders WHERE total > 100;
-- 应看到 Parallel Seq Scan 或 Parallel Index Scan

-- JIT 编译 (PostgreSQL 11+)
-- jit = on
-- jit_above_cost = 100000
-- jit_inline_above_cost = 500000
-- jit_optimize_above_cost = 500000

-- 对于 OLTP，可考虑关闭 JIT 以降低延迟
-- SET jit = off;

-- random_page_cost: SSD 调低到 1.1 (默认 4.0 是 HDD 值)
-- random_page_cost = 1.1
-- seq_page_cost = 1.0
-- effective_io_concurrency = 200  -- SSD 推荐

-- 统计目标 (对高基数列提高采样精度)
ALTER TABLE orders ALTER COLUMN customer_id SET STATISTICS 1000;  -- 默认100
ANALYZE orders;
```

---

## 12. 常见陷阱

### 12.1 N+1 查询

```sql
-- 陷阱: 应用层循环查询
-- Python 伪代码:
-- orders = db.query("SELECT * FROM orders WHERE status = 'pending'")
-- for order in orders:
--     items = db.query(f"SELECT * FROM order_items WHERE order_id = {order.id}")
--     # 100 个订单 = 101 次查询!

-- 解决: JOIN 或子查询一次获取
SELECT o.*, json_agg(
    json_build_object('product_id', oi.product_id, 'quantity', oi.quantity, 'price', oi.unit_price)
) AS items
FROM orders o
LEFT JOIN order_items oi ON oi.order_id = o.id
WHERE o.status = 'pending'
GROUP BY o.id;

-- 或使用 LATERAL JOIN (更灵活)
SELECT o.*, items.data
FROM orders o
CROSS JOIN LATERAL (
    SELECT json_agg(oi.*) AS data
    FROM order_items oi
    WHERE oi.order_id = o.id
) items
WHERE o.status = 'pending';
```

### 12.2 锁竞争

```sql
-- 陷阱: DDL 操作阻塞所有查询
ALTER TABLE orders ADD COLUMN notes TEXT;
-- 获取 ACCESS EXCLUSIVE 锁，阻塞所有并发操作

-- 解决: 使用低锁级别操作
-- 添加无默认值的可空列 (瞬时，不重写表)
ALTER TABLE orders ADD COLUMN notes TEXT;  -- OK，快速

-- 陷阱: 添加有默认值的列 (PostgreSQL 11 之前会重写表)
-- PostgreSQL 11+ 不重写表，但旧版本注意

-- 陷阱: CREATE INDEX 锁表
CREATE INDEX idx_orders_notes ON orders(notes);  -- 锁表!

-- 解决: CONCURRENTLY (不锁表，但更慢)
CREATE INDEX CONCURRENTLY idx_orders_notes ON orders(notes);

-- 陷阱: 长事务持有锁
BEGIN;
  SELECT * FROM orders WHERE id = 1 FOR UPDATE;
  -- ... 应用层做了很久的处理 ...
  -- 其他事务等待 id=1 的行锁
COMMIT;

-- 解决: 设置语句/锁超时
SET statement_timeout = '30s';
SET lock_timeout = '5s';
SET idle_in_transaction_session_timeout = '60s';
```

### 12.3 膨胀表 (Table Bloat)

```sql
-- 陷阱: 大量 UPDATE/DELETE 后表膨胀，查询变慢
-- 原因: 死元组占据空间，顺序扫描变慢

-- 诊断: 膨胀比估算
SELECT
    schemaname, tablename,
    pg_size_pretty(pg_total_relation_size(schemaname || '.' || tablename)) AS total_size,
    n_live_tup,
    n_dead_tup,
    CASE WHEN n_live_tup > 0
        THEN round(n_dead_tup::numeric / n_live_tup * 100, 2)
        ELSE 0
    END AS bloat_ratio_pct,
    last_autovacuum
FROM pg_stat_user_tables
WHERE n_dead_tup > 10000
ORDER BY n_dead_tup DESC;

-- 使用 pgstattuple 扩展精确测量
CREATE EXTENSION IF NOT EXISTS pgstattuple;
SELECT * FROM pgstattuple('orders');
-- 关注: dead_tuple_percent, free_percent

-- 解决方案 1: 调优 autovacuum (推荐)
ALTER TABLE orders SET (
    autovacuum_vacuum_scale_factor = 0.02,
    autovacuum_vacuum_cost_delay = 2,         -- 加速 vacuum
    autovacuum_vacuum_cost_limit = 1000
);

-- 解决方案 2: 在线重建 (pg_repack，不锁表)
-- pg_repack --table orders --jobs 4 -d mydb

-- 解决方案 3: VACUUM FULL (锁表，紧急用)
VACUUM FULL VERBOSE orders;
```

### 12.4 连接泄漏

```sql
-- 陷阱: 应用崩溃/异常后连接未释放
-- 诊断
SELECT count(*), state, usename
FROM pg_stat_activity
GROUP BY state, usename
ORDER BY count DESC;

-- 大量 idle 连接消耗内存
-- 每个连接约占 5-10MB 内存

-- 解决: 设置 idle 超时
-- idle_in_transaction_session_timeout = '60s'    -- 空闲事务超时
-- idle_session_timeout = '3600s'                 -- PostgreSQL 14+ 空闲会话超时

-- 解决: 使用 PgBouncer 限制实际连接数 (见第10章)
```

### 12.5 不安全的类型转换

```sql
-- 陷阱: 隐式转换导致索引失效
-- 假设 phone 列是 VARCHAR
SELECT * FROM customers WHERE phone = 13800138000;
-- PostgreSQL 把 VARCHAR 转为 NUMERIC 比较，索引失效!

-- 解决: 保持类型一致
SELECT * FROM customers WHERE phone = '13800138000';

-- 陷阱: JSONB 数值比较
SELECT * FROM products WHERE (attributes ->> 'price')::numeric > 100;
-- 每次都做类型转换，无法用索引

-- 解决: 表达式索引
CREATE INDEX idx_products_price ON products(((attributes ->> 'price')::numeric));
```

### 12.6 错误的分页实现

```sql
-- 陷阱: OFFSET 越大越慢
SELECT * FROM orders ORDER BY id LIMIT 20 OFFSET 100000;
-- 数据库必须扫描前 100020 行然后丢弃 100000 行

-- 解决: 游标分页 (Keyset Pagination)
-- 第一页
SELECT * FROM orders ORDER BY id LIMIT 20;

-- 后续页 (假设上一页最后一条 id=100020)
SELECT * FROM orders WHERE id > 100020 ORDER BY id LIMIT 20;
-- 利用索引直接定位，性能恒定

-- 复合排序的游标分页
SELECT * FROM orders
WHERE (ordered_at, id) < ('2026-03-27 10:00:00', 99999)
ORDER BY ordered_at DESC, id DESC
LIMIT 20;
```

---

## 13. 实用运维 SQL

### 13.1 表空间管理

```sql
-- 数据库大小
SELECT pg_size_pretty(pg_database_size(current_database())) AS db_size;

-- 各表大小排名
SELECT
    relname AS table_name,
    pg_size_pretty(pg_total_relation_size(oid)) AS total,
    pg_size_pretty(pg_relation_size(oid)) AS data,
    pg_size_pretty(pg_indexes_size(oid)) AS indexes,
    pg_size_pretty(pg_total_relation_size(oid) - pg_relation_size(oid) - pg_indexes_size(oid)) AS toast
FROM pg_class
WHERE relkind = 'r' AND relnamespace = 'public'::regnamespace
ORDER BY pg_total_relation_size(oid) DESC
LIMIT 20;
```

### 13.2 序列管理

```sql
-- 查看所有序列当前值
SELECT
    sequencename,
    last_value,
    start_value,
    max_value,
    increment_by
FROM pg_sequences
WHERE schemaname = 'public';

-- 重置序列到最大已用值（数据导入后）
SELECT setval('orders_id_seq', (SELECT max(id) FROM orders));
```

### 13.3 批量数据操作

```sql
-- 高效批量插入
-- 方法1: COPY (最快)
COPY orders(customer_id, status, total, ordered_at)
FROM '/tmp/orders.csv' WITH (FORMAT csv, HEADER true);

-- 方法2: 多值 INSERT
INSERT INTO orders(customer_id, status, total)
VALUES
    (1, 'pending', 99.99),
    (2, 'pending', 149.50),
    (3, 'confirmed', 200.00);

-- 方法3: INSERT ... SELECT
INSERT INTO orders_archive
SELECT * FROM orders WHERE ordered_at < '2025-01-01';

-- 高效批量更新 (避免逐行 UPDATE)
UPDATE products p
SET price = t.new_price
FROM (VALUES
    (1, 19.99),
    (2, 29.99),
    (3, 39.99)
) AS t(id, new_price)
WHERE p.id = t.id;

-- 高效批量删除 (大表分批删除，避免长事务)
DO $$
DECLARE
    batch_size INTEGER := 10000;
    deleted INTEGER;
BEGIN
    LOOP
        DELETE FROM event_logs
        WHERE id IN (
            SELECT id FROM event_logs
            WHERE event_time < '2025-01-01'
            LIMIT batch_size
        );
        GET DIAGNOSTICS deleted = ROW_COUNT;
        RAISE NOTICE 'Deleted % rows', deleted;
        EXIT WHEN deleted < batch_size;
        PERFORM pg_sleep(0.1);  -- 释放锁压力
        COMMIT;
    END LOOP;
END $$;
```

---

## Agent Checklist

以下检查项用于 Super Dev 流水线的数据层质量门禁。

### 设计阶段

- [ ] 表设计遵循命名规范：小写 snake_case，表名复数
- [ ] 主键使用 `BIGINT GENERATED ALWAYS AS IDENTITY`（非 SERIAL）
- [ ] 外键约束明确 `ON DELETE` / `ON UPDATE` 行为
- [ ] 金额字段使用 `NUMERIC(p,s)` 而非浮点型
- [ ] 时间字段使用 `TIMESTAMPTZ`（非 TIMESTAMP）
- [ ] JSONB 字段有明确的结构文档和 GIN 索引
- [ ] CHECK 约束覆盖业务不变量
- [ ] 分区策略已评估（单表预期超过 1000 万行时）

### 索引阶段

- [ ] 所有外键列已建索引
- [ ] 高频查询的 WHERE/JOIN/ORDER BY 列有对应索引
- [ ] 部分索引用于过滤活跃子集
- [ ] 覆盖索引用于高频只读查询
- [ ] 全文搜索使用 GIN + tsvector 而非 LIKE '%keyword%'
- [ ] 未使用的索引已清理（pg_stat_user_indexes.idx_scan = 0）

### 查询阶段

- [ ] 核心查询已通过 EXPLAIN ANALYZE 验证
- [ ] 无 Seq Scan on 大表（除非有意为之）
- [ ] 分页使用游标分页而非 OFFSET
- [ ] N+1 查询已用 JOIN/子查询/LATERAL 消除
- [ ] CTE 未导致优化屏障（检查 NOT MATERIALIZED）
- [ ] 统计信息更新频率已调优

### 安全阶段

- [ ] 应用使用最小权限角色连接（非 superuser）
- [ ] 多租户场景启用 RLS
- [ ] SSL/TLS 强制开启
- [ ] pg_hba.conf 仅允许必要 IP 段
- [ ] 密码使用 scram-sha-256 认证
- [ ] pgaudit 审计已配置

### 运维阶段

- [ ] pg_stat_statements 已启用
- [ ] autovacuum 参数已调优（非默认值）
- [ ] 连接池 (PgBouncer) 已部署
- [ ] shared_buffers / work_mem / effective_cache_size 已按内存调优
- [ ] WAL 级别设为 replica 或 logical
- [ ] 自动备份已配置（pg_basebackup + WAL 归档）
- [ ] PITR 恢复已验证
- [ ] 监控告警已覆盖：复制延迟、连接数、缓存命中率、死元组比例
- [ ] 长事务 / 空闲事务超时已设置
- [ ] 定期检查未使用索引和膨胀表

---

> 文档版本: v1.0 | 最后更新: 2026-03-28 | 维护者: Super Dev Knowledge Base
