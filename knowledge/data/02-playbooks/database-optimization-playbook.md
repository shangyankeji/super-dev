---
id: database-optimization-playbook
title: 数据库优化 Playbook
domain: data
category: 02-playbooks
difficulty: intermediate
tags: [agent, checklist, data, database, optimization, playbook, 优化决策流程图, 优化效果基准参考]
quality_score: 70
last_updated: 2026-06-15
---
# 数据库优化 Playbook

## 概述

本 Playbook 提供数据库性能优化的完整操作流程，按照递进式策略组织：慢查询分析 → 索引优化 → 查询重写 → 缓存策略 → 分区 → 读写分离。每一步都先以最小成本的方案解决问题，只在前一步不足时才进入下一步。适用于 PostgreSQL 和 MySQL，特定语法会标注数据库类型。

原则：**先量化后优化，先单点后架构**。

---

## 第一步：慢查询分析

### 目标

找出系统中最消耗资源的 SQL 语句，建立优化优先级。

### 操作步骤

1. **启用慢查询日志**

```
# PostgreSQL - 修改 postgresql.conf
log_min_duration_statement = 200   # 记录超过 200ms 的查询
shared_preload_libraries = 'pg_stat_statements'

# MySQL - 修改 my.cnf
slow_query_log = 1
long_query_time = 0.2
log_queries_not_using_indexes = 1
```

2. **收集慢查询数据（至少 7 天）**

```
# PostgreSQL - 查看 Top 20 慢查询
SELECT query, calls, mean_exec_time, total_exec_time
FROM pg_stat_statements
ORDER BY total_exec_time DESC
LIMIT 20;

# MySQL - 使用 pt-query-digest 分析
pt-query-digest /var/log/mysql/slow.log > slow_report.txt
```

3. **分类慢查询**

| 类型 | 特征 | 优先级 |
|------|------|--------|
| 高频慢查询 | 调用量 > 1000/天，耗时 > 500ms | P0 |
| 低频超慢查询 | 调用量 < 100/天，耗时 > 5s | P1 |
| 全表扫描查询 | EXPLAIN 显示 Seq Scan / Full Table Scan | P1 |
| 锁等待查询 | 等锁时间 > 执行时间 | P0 |

4. **建立基线**

- 记录每条慢查询的当前耗时（P50 / P95 / P99）
- 记录每条慢查询的调用频率
- 优化后对比基线评估效果

---

## 第二步：索引优化

### 目标

通过添加或调整索引消除不必要的全表扫描。

### 操作步骤

1. **分析查询执行计划**

```sql
-- PostgreSQL
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT) SELECT ...;

-- MySQL
EXPLAIN FORMAT=JSON SELECT ...;
```

2. **识别需要索引的场景**

- [ ] WHERE 子句中频繁出现的列
- [ ] JOIN 条件中的关联列
- [ ] ORDER BY / GROUP BY 中的列
- [ ] 区分度高的列（如用户 ID）优先于区分度低的列（如性别）

3. **索引类型选择**

| 场景 | 推荐索引类型 |
|------|-------------|
| 等值查询 | B-Tree（默认） |
| 范围查询 | B-Tree |
| 全文搜索 | GIN (PostgreSQL) / FULLTEXT (MySQL) |
| JSON 字段查询 | GIN (PostgreSQL) |
| 地理位置查询 | GiST (PostgreSQL) / SPATIAL (MySQL) |
| 多列组合查询 | 复合索引（注意列顺序：最左匹配） |

4. **复合索引设计原则**

```
规则：等值列在前，范围列在后，排序列最后

示例：WHERE status = 'active' AND created_at > '2024-01-01' ORDER BY id
索引：CREATE INDEX idx_orders_status_created_id ON orders(status, created_at, id);
```

5. **索引健康检查**

```sql
-- PostgreSQL - 查找未使用的索引
SELECT indexrelname, idx_scan
FROM pg_stat_user_indexes
WHERE idx_scan = 0 AND indexrelname NOT LIKE '%_pkey'
ORDER BY pg_relation_size(indexrelid) DESC;

-- 查找重复索引
SELECT * FROM pg_indexes WHERE tablename = 'orders';
```

6. **索引维护**

- [ ] 删除未使用的索引（减少写入开销）
- [ ] 删除重复索引（被更宽的复合索引覆盖的索引）
- [ ] 定期 REINDEX（PostgreSQL）/ OPTIMIZE TABLE（MySQL）
- [ ] 大表添加索引使用 CONCURRENTLY（PostgreSQL）避免锁表

---

## 第三步：查询重写

### 目标

在不改变业务逻辑的前提下，重写低效 SQL 语句。

### 常见优化模式

1. **避免 SELECT ***

```sql
-- 差
SELECT * FROM orders WHERE user_id = 123;
-- 好（只取需要的列，可能命中覆盖索引）
SELECT id, status, total_amount FROM orders WHERE user_id = 123;
```

2. **子查询改写为 JOIN**

```sql
-- 差（相关子查询，每行执行一次）
SELECT * FROM orders WHERE user_id IN (SELECT id FROM users WHERE status = 'active');
-- 好
SELECT o.* FROM orders o JOIN users u ON o.user_id = u.id WHERE u.status = 'active';
```

3. **分页优化**

```sql
-- 差（OFFSET 越大越慢）
SELECT * FROM orders ORDER BY id LIMIT 20 OFFSET 100000;
-- 好（基于游标分页）
SELECT * FROM orders WHERE id > 100000 ORDER BY id LIMIT 20;
```

4. **批量操作替代循环**

```sql
-- 差（N 次单条插入）
INSERT INTO logs (msg) VALUES ('a');
INSERT INTO logs (msg) VALUES ('b');
-- 好（批量插入）
INSERT INTO logs (msg) VALUES ('a'), ('b'), ...;
```

5. **避免函数作用于索引列**

```sql
-- 差（索引失效）
SELECT * FROM orders WHERE DATE(created_at) = '2024-01-01';
-- 好（索引有效）
SELECT * FROM orders WHERE created_at >= '2024-01-01' AND created_at < '2024-01-02';
```

6. **EXISTS 替代 COUNT**

```sql
-- 差（扫描全部匹配行）
SELECT CASE WHEN COUNT(*) > 0 THEN true ELSE false END FROM orders WHERE user_id = 123;
-- 好（找到第一行即返回）
SELECT EXISTS(SELECT 1 FROM orders WHERE user_id = 123);
```

---

## 第四步：缓存策略

### 目标

通过缓存减少数据库查询次数，降低数据库负载。

### 操作步骤

1. **识别缓存候选**

| 特征 | 缓存价值 |
|------|---------|
| 读多写少 | 高 |
| 计算成本高 | 高 |
| 数据变化慢 | 高 |
| 访问频率高 | 高 |
| 实时性要求低 | 高 |

2. **缓存层次设计**

```
L1: 应用内存缓存（本地 LRU）
   - 适合：配置数据、枚举值、小型查找表
   - TTL: 5-30 分钟
   - 库：caffeine (Java) / lru-cache (Node.js) / cachetools (Python)

L2: 分布式缓存（Redis）
   - 适合：会话数据、热点查询结果、排行榜
   - TTL: 根据业务场景设定
   - 序列化：JSON 或 MessagePack

L3: 数据库查询缓存
   - PostgreSQL: 物化视图（Materialized View）
   - MySQL: 查询缓存（8.0 已移除，不推荐依赖）
```

3. **缓存更新策略**

| 策略 | 适用场景 | 实现方式 |
|------|---------|---------|
| Cache-Aside | 通用场景 | 读时查缓存，miss 则查 DB 并写缓存 |
| Write-Through | 一致性要求高 | 写 DB 同时写缓存 |
| Write-Behind | 写入量大 | 先写缓存，异步批量写 DB |
| Refresh-Ahead | 热点数据 | TTL 到期前异步刷新 |

4. **缓存防护**

- [ ] 缓存穿透：布隆过滤器或缓存空值（TTL 短）
- [ ] 缓存击穿：热点 Key 使用互斥锁（singleflight）
- [ ] 缓存雪崩：TTL 加随机偏移，避免集中过期
- [ ] 大 Key 拆分：单个 Value 不超过 1MB

---

## 第五步：表分区

### 目标

当单表数据量超过千万行时，通过分区提升查询和维护效率。

### 操作步骤

1. **评估是否需要分区**

- [ ] 单表行数 > 1000 万
- [ ] 查询普遍包含时间范围过滤
- [ ] 历史数据需要定期归档
- [ ] 索引维护（VACUUM / OPTIMIZE）耗时过长

2. **分区策略选择**

| 策略 | 适用场景 | 示例 |
|------|---------|------|
| 范围分区（Range） | 时间序列数据 | 按月分区 |
| 列表分区（List） | 有限枚举值 | 按地区/状态分区 |
| 哈希分区（Hash） | 均匀分布需求 | 按用户 ID 哈希分区 |

3. **PostgreSQL 分区示例**

```sql
-- 创建分区表
CREATE TABLE orders (
    id BIGSERIAL,
    created_at TIMESTAMPTZ NOT NULL,
    status TEXT,
    total_amount DECIMAL
) PARTITION BY RANGE (created_at);

-- 创建月度分区
CREATE TABLE orders_2024_01 PARTITION OF orders
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');

-- 自动化分区管理（使用 pg_partman）
SELECT partman.create_parent('public.orders', 'created_at', 'native', 'monthly');
```

4. **分区维护**

- [ ] 自动创建未来分区（提前 3 个月）
- [ ] 过期分区归档后 DETACH（不 DROP，保留可恢复性）
- [ ] 分区键必须包含在所有唯一约束中
- [ ] 验证查询计划确认分区裁剪生效

---

## 第六步：读写分离

### 目标

将读请求分流到只读副本，降低主库负载。

### 操作步骤

1. **架构设计**

```
写请求 → 主库（Primary）
读请求 → 只读副本（Replica） × N

连接管理方案：
- 应用层路由：在代码中区分读写连接
- 中间件路由：ProxySQL (MySQL) / PgBouncer + 自定义路由
- 框架支持：Django ReadReplicaRouter / Spring @Transactional(readOnly=true)
```

2. **一致性处理**

| 场景 | 处理方式 |
|------|---------|
| 写后立即读 | 读主库（write-then-read 模式） |
| 报表查询 | 读副本（延迟可接受） |
| 关键业务查询 | 读主库 |
| 一般列表查询 | 读副本 |

3. **副本延迟监控**

```sql
-- PostgreSQL - 查看副本延迟
SELECT client_addr, state, sent_lsn, write_lsn, flush_lsn, replay_lsn,
       (sent_lsn - replay_lsn) AS replay_lag
FROM pg_stat_replication;

-- MySQL - 查看副本延迟
SHOW SLAVE STATUS\G  -- 查看 Seconds_Behind_Master
```

4. **运维要点**

- [ ] 副本延迟 > 10s 触发告警
- [ ] 副本延迟 > 30s 自动将读请求回退到主库
- [ ] 副本数量 ≥ 2（高可用）
- [ ] 副本与主库不在同一可用区（容灾）
- [ ] 定期验证副本数据一致性

---

## 优化决策流程图

```
性能问题
  │
  ├→ 慢查询分析 → 找到 Top N 慢查询
  │     │
  │     ├→ 缺索引？ → 第二步：索引优化
  │     ├→ SQL 低效？ → 第三步：查询重写
  │     ├→ 查询频率过高？ → 第四步：缓存策略
  │     ├→ 数据量过大？ → 第五步：表分区
  │     └→ 读负载过高？ → 第六步：读写分离
  │
  └→ 仍未解决 → 考虑架构升级（分库分表 / NewSQL / 专用存储）
```

---

## 优化效果基准参考

| 优化手段 | 典型改善幅度 | 实施成本 |
|---------|------------|---------|
| 添加合适索引 | 10x-1000x | 低 |
| 查询重写 | 2x-50x | 低 |
| 应用层缓存 | 5x-100x | 中 |
| 表分区 | 2x-10x | 中 |
| 读写分离 | 2x-5x（读吞吐量） | 高 |

---

## Agent Checklist

以下为 AI Agent 在执行数据库优化时必须遵循的硬约束：

- [ ] 优化前使用 EXPLAIN ANALYZE 记录基线执行计划
- [ ] 不在生产环境直接修改索引，先在 staging 验证
- [ ] 大表添加索引使用 CONCURRENTLY（PostgreSQL）或在低峰期执行
- [ ] 缓存方案必须包含失效策略和防护机制
- [ ] 查询重写后验证结果集与原查询一致
- [ ] 分区方案需验证分区裁剪在查询计划中生效
- [ ] 读写分离需确认写后读场景的一致性处理
- [ ] 每步优化后运行性能测试对比基线
- [ ] 将优化结果记录到性能优化日志
- [ ] 生成优化报告包含：问题、方案、基线对比、风险评估
