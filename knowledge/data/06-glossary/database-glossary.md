---
id: database-glossary
title: 数据库术语表 (Database Glossary)
domain: data
category: 06-glossary
difficulty: intermediate
tags: [agent, checklist, data, database, glossary, 事务与一致性模型, 分布式架构, 数据类型与特性]
quality_score: 70
last_updated: 2026-06-15
---
# 数据库术语表 (Database Glossary)

> 收录 50+ 核心数据库术语，覆盖事务模型、存储引擎、索引结构、分布式架构、查询优化、运维管理等领域。
> 适用于架构评审、数据库选型、团队培训、Code Review 等场景。

---

## 事务与一致性模型

### ACID (Atomicity, Consistency, Isolation, Durability)

关系型数据库事务的四大特性：

- **原子性 (Atomicity)**：事务中的操作全部成功或全部回滚，不存在部分完成
- **一致性 (Consistency)**：事务前后数据满足所有约束（外键、唯一性、CHECK 约束等）
- **隔离性 (Isolation)**：并发事务互不干扰，效果等同于串行执行
- **持久性 (Durability)**：已提交事务的结果永久保存，即使系统崩溃也不丢失

PostgreSQL 和 MySQL InnoDB 完整支持 ACID。NoSQL 数据库通常只提供部分 ACID 保证。

### BASE (Basically Available, Soft state, Eventually consistent)

分布式系统中 ACID 的替代模型，是 CAP 定理下的务实选择：

- **基本可用 (Basically Available)**：系统保证可用性，但可能返回降级的响应
- **软状态 (Soft state)**：允许系统中间状态暂时不一致
- **最终一致 (Eventually consistent)**：经过一段时间后，所有副本达到一致

常见于 NoSQL 数据库（Cassandra、DynamoDB）和微服务架构。适合对一致性要求不高但对可用性要求极高的场景。

### CAP 定理 (Consistency, Availability, Partition tolerance)

分布式系统中三者最多只能同时满足两个：

- **一致性 (C)**：所有节点在同一时刻看到相同的数据
- **可用性 (A)**：每个请求都能收到非错误的响应
- **分区容错 (P)**：网络分区发生时系统仍能正常运行

由于网络分区不可避免（P 必须满足），实际选择在 CP 和 AP 之间：
- **CP 系统**：ZooKeeper、etcd、HBase -- 网络分区时拒绝服务以保证一致性
- **AP 系统**：Cassandra、DynamoDB、CouchDB -- 网络分区时继续服务但可能返回过时数据

### MVCC (Multi-Version Concurrency Control)

多版本并发控制。通过为每行数据维护多个版本（带时间戳或事务 ID），使读操作不阻塞写操作、写操作不阻塞读操作。是数据库实现高并发的核心机制。

- **PostgreSQL**：使用 xmin/xmax 标记行的创建和删除事务 ID，旧版本保留在原表中，由 VACUUM 清理
- **MySQL InnoDB**：使用 undo log 维护旧版本，通过 ReadView 决定事务可见的数据版本

### WAL (Write-Ahead Logging)

预写日志。核心思想：在修改数据页之前，先将变更记录写入顺序日志文件。

- **崩溃恢复**：通过重放 WAL 日志将数据库恢复到一致状态
- **流复制**：PostgreSQL 的 WAL 发送到副本节点实现数据同步
- **PITR**：通过 WAL 归档实现时间点恢复

MySQL 的等价物是 redo log（InnoDB）和 binlog（Server 层）。

### 事务隔离级别 (Transaction Isolation Level)

控制并发事务之间可见性的四个级别，从低到高：

1. **Read Uncommitted（读未提交）**：可以读到其他事务未提交的数据（脏读），几乎不使用
2. **Read Committed（读已提交）**：只能读到已提交的数据，PostgreSQL 默认级别
3. **Repeatable Read（可重复读）**：同一事务内多次读取结果一致，MySQL InnoDB 默认级别
4. **Serializable（串行化）**：完全串行执行，一致性最好但并发性能最差

级别越高一致性越好，但并发性能越差。需要根据业务场景选择合适的级别。

### 两阶段提交 (Two-Phase Commit / 2PC)

分布式事务协议：

- **阶段一 (Prepare)**：协调者询问所有参与者是否可以提交
- **阶段二 (Commit/Abort)**：所有参与者都同意则提交，否则全部回滚

存在阻塞和单点故障问题，实践中常被 Saga 模式替代。

### Saga 模式 (Saga Pattern)

将分布式事务拆分为一系列本地事务，每个本地事务有对应的补偿事务。前序事务成功则继续执行下一个，失败则逆序执行已完成事务的补偿。

实现方式：
- **编排式 (Choreography)**：事件驱动，各服务监听事件自主决策
- **协调式 (Orchestration)**：中央协调者控制整个流程

---

## 索引与存储结构

### B-tree (Balanced Tree)

平衡多路搜索树，是关系型数据库最常用的索引结构。每个节点包含多个键值和子节点指针，所有叶子节点在同一层。支持等值查询、范围查询和排序。PostgreSQL 和 MySQL 的默认索引类型。时间复杂度 O(log N)。

### B+ tree (B Plus Tree)

B-tree 的变体，核心差异：
- 所有数据只存储在叶子节点，内部节点只存储键值（导航用）
- 叶子节点通过双向链表相连，支持高效的范围扫描
- 内部节点可以容纳更多键值，树的高度更低

MySQL InnoDB 的聚簇索引和二级索引均使用 B+ tree。

### LSM-tree (Log-Structured Merge-tree)

日志结构合并树，针对写密集型场景优化的存储结构：

1. 写入先进入内存的 MemTable（有序结构，如红黑树 / 跳表）
2. MemTable 满后刷写为磁盘上有序的 SSTable（Sorted String Table）文件
3. 后台合并（Compaction）多个 SSTable，减少文件数量和重复数据

使用者：RocksDB、LevelDB、Cassandra、HBase、TiKV。写性能极高（顺序写），读性能通过 Bloom Filter 和多级缓存优化。

### Bloom Filter (布隆过滤器)

概率型数据结构，用于快速判断元素是否在集合中：

- **可能误判 (False Positive)**：判断"存在"时可能实际不存在
- **不会漏判 (No False Negative)**：判断"不存在"时一定不存在

空间效率极高（每个元素仅需几个 bit）。数据库中的应用：
- LSM-tree 跳过不含目标键的 SSTable
- JOIN 优化（Bloom Join）
- 缓存穿透防护
- Redis 内置 Bloom Filter 模块

### 哈希索引 (Hash Index)

基于哈希表的索引结构。只支持等值查询（=），不支持范围查询和排序。查找时间 O(1)。PostgreSQL 支持哈希索引，MySQL Memory 引擎使用哈希索引。适用于精确匹配的高频查询场景（如按 session_id 查找）。

### GiST (Generalized Search Tree)

通用搜索树，PostgreSQL 的可扩展索引框架。支持多种数据类型的索引：
- 地理空间数据（PostGIS 的几何/地理类型）
- 全文搜索（tsvector）
- 范围类型（int4range、daterange）
- IP 地址（inet、cidr）
- 最近邻搜索（KNN）

### GIN (Generalized Inverted Index)

通用倒排索引，适合索引包含多个元素的值：
- 数组（`INTEGER[]`、`TEXT[]`）
- JSONB 文档
- 全文搜索的 tsvector

一个索引条目指向包含该元素的所有行。GIN 索引构建较慢但查询快，适合读多写少的场景。

### 聚簇索引 (Clustered Index)

表数据按索引键的顺序物理存储。每张表只能有一个聚簇索引。MySQL InnoDB 的主键即为聚簇索引（如无显式主键则使用第一个 UNIQUE NOT NULL 索引，再无则自动生成 6 字节 row_id）。聚簇索引的范围扫描性能最优，因为相邻数据在物理上也相邻。

### 覆盖索引 (Covering Index)

索引包含查询所需的所有列，查询可以仅通过索引完成（Index Only Scan），无需回表读取数据页。大幅减少 I/O。

PostgreSQL 使用 `INCLUDE` 子句：`CREATE INDEX idx ON t (a) INCLUDE (b, c)`。MySQL 中将查询涉及的所有列都放入联合索引中也可达到覆盖索引效果。

---

## 分布式架构

### Sharding (分片)

将数据水平拆分到多个数据库实例中，每个实例持有数据的一个子集。分片键（Shard Key）决定数据如何分布。

常见策略：
- **范围分片**：按 ID 范围或日期范围分片，易于范围查询但可能热点不均
- **哈希分片**：按哈希值取模分片，数据分布均匀但范围查询需扫描所有分片
- **目录分片**：通过查表路由，灵活但引入单点

挑战：跨片查询、分片再均衡、分布式事务、全局唯一 ID 生成。

### Replication (复制)

将数据从主节点（Primary）同步到一个或多个副本节点（Replica）：

- **同步复制**：主节点等待副本确认后才提交，数据零丢失但延迟高
- **异步复制**：主节点不等待副本确认，延迟低但主节点故障时可能丢最近数据
- **半同步复制**：至少一个副本确认后提交，平衡一致性和性能

### Partitioning (分区)

将单张大表按规则拆分为多个物理分区，逻辑上仍是一张表：

- **范围分区**：按日期、ID 范围分区，最常用于时序数据
- **列表分区**：按枚举值分区（如按地区、状态）
- **哈希分区**：按哈希值分区，数据均匀分布

PostgreSQL 原生支持声明式分区。好处：查询自动裁剪无关分区（Partition Pruning）、分区独立维护（VACUUM、备份、归档）。

### 读写分离 (Read-Write Splitting)

写操作发往主库，读操作发往只读副本。适用于读多写少（读写比 > 5:1）的场景。

实现方式：应用层路由（代码控制）、中间件（ProxySQL、PgBouncer）、云服务内置（AWS RDS Read Replica）。需注意副本延迟导致的读不一致问题（写入后立刻读取可能读到旧数据）。

### 一致性哈希 (Consistent Hashing)

将数据和节点都映射到同一个哈希环上，数据被分配到顺时针方向最近的节点。节点增减时只影响相邻数据段，最小化数据迁移量。引入虚拟节点（Virtual Node）解决数据倾斜问题。

使用者：DynamoDB、Cassandra、Redis Cluster（使用哈希槽变体）。

### Raft (Raft Consensus Algorithm)

分布式一致性算法，通过 Leader 选举和日志复制确保多个节点的数据一致。比 Paxos 更易理解和实现。

核心流程：
1. **Leader 选举**：节点超时未收到心跳则发起选举
2. **日志追加**：Leader 接收写请求，复制日志到 Follower
3. **提交确认**：多数节点确认后 Leader 提交并响应客户端

使用者：etcd、CockroachDB、TiKV、Consul。

---

## 查询优化

### Materialized View (物化视图)

将查询结果预计算并存储为物理表。与普通视图不同，物化视图不在查询时实时计算，需要手动或定时刷新。适合复杂聚合查询、报表场景。

PostgreSQL 支持：
- `CREATE MATERIALIZED VIEW mv AS SELECT ...`
- `REFRESH MATERIALIZED VIEW CONCURRENTLY mv`（无锁刷新，需要唯一索引）

### CTE (Common Table Expression)

公用表表达式，使用 `WITH` 子句定义的命名临时结果集。提升复杂查询的可读性和可维护性。

- **普通 CTE**：PostgreSQL 12+ 中默认内联优化（与子查询等效）
- **递归 CTE**：`WITH RECURSIVE` 可用于树形结构查询（组织架构、分类树、BOM 展开）

### Window Function (窗口函数)

在不改变结果行数的前提下，对每行计算基于一组相关行（窗口）的聚合值：

- `ROW_NUMBER()`：行号（无并列）
- `RANK()` / `DENSE_RANK()`：排名（有/无间隔）
- `LAG()` / `LEAD()`：前后行的值
- `SUM() OVER()` / `AVG() OVER()`：累计/移动聚合
- `FIRST_VALUE()` / `LAST_VALUE()`：窗口内首尾值
- `NTILE(n)`：将行等分为 n 组

SQL 分析利器，避免自连接和相关子查询。

### EXPLAIN (查询执行计划)

数据库展示 SQL 查询的执行策略：选择哪些索引、JOIN 顺序和算法、过滤条件应用位置、预估和实际行数。

- `EXPLAIN`：显示预估执行计划
- `EXPLAIN ANALYZE`（PostgreSQL）：实际执行并显示真实耗时和行数
- `EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)`：最详细的分析输出

是 SQL 性能调优的核心工具，每条慢查询都应该先 EXPLAIN。

### 查询计划缓存 (Query Plan Cache)

数据库缓存已编译的查询计划，避免重复解析和优化。PostgreSQL 使用 Prepared Statement 触发计划缓存（前 5 次使用自定义计划，之后可能切换为通用计划）。MySQL 的 Query Cache 已在 8.0 中移除（因失效粒度太粗）。

### 统计信息 (Statistics)

数据库收集的表和列的数据分布信息：行数、不同值数量（n_distinct）、最常见值（most_common_vals）、直方图（histogram_bounds）等。供查询优化器选择最优执行计划。

- PostgreSQL 通过 `ANALYZE` 命令或 autovacuum 自动更新
- 统计信息过时可能导致优化器选择次优计划（如预估行数偏差大）
- `ALTER TABLE ... ALTER COLUMN ... SET STATISTICS 1000` 可提高统计精度

---

## 数据类型与特性

### JSONB (JSON Binary)

PostgreSQL 的二进制 JSON 存储类型。与 JSON 类型不同，JSONB 解析后以二进制格式存储：

- 支持 GIN 索引，查询性能高
- 支持运算符：`->>` 取值、`@>` 包含、`?` 存在键、`#>` 路径取值
- 适合半结构化数据，替代 EAV 模式
- 不保留格式（空格、键顺序），去重重复键

### Full-Text Search (全文搜索)

对文本内容进行分词、索引和排名的搜索能力：

- **PostgreSQL 内置**：`tsvector`（文档向量）+ `tsquery`（查询向量）+ GIN 索引
- 支持中文需安装 zhparser 或 pg_jieba 分词扩展
- 支持权重（A/B/C/D）和排名（ts_rank）
- **Elasticsearch**：专用全文搜索引擎，功能更强但运维更复杂

### UUID (Universally Unique Identifier)

128 位全局唯一标识符。格式：`550e8400-e29b-41d4-a716-446655440000`。

- PostgreSQL 原生支持 UUID 类型和 `gen_random_uuid()` 函数
- 适合分布式系统中的主键（无需中央分配）
- 随机 UUID (v4) 会导致 B-tree 索引碎片和页分裂
- UUIDv7（时间排序）解决索引碎片问题，推荐用于主键

### ENUM (枚举类型)

数据库中预定义的一组命名常量：

```sql
CREATE TYPE order_status AS ENUM ('draft', 'pending', 'paid', 'shipped', 'completed');
```

优点：存储空间小（4 字节）、自带约束、可读性好。
缺点：修改值列表需要 ALTER TYPE，不适合频繁变化的枚举。在应用层维护枚举映射通常更灵活。

### 数组类型 (Array Type)

PostgreSQL 支持列存储数组值：`INTEGER[]`、`TEXT[]`。

- 支持运算符：`@>` 包含、`&&` 交集、`||` 连接
- 支持 GIN 索引
- 适合标签、权限列表等简单场景
- 避免滥用：需要 JOIN 或频繁更新单个元素的场景应使用关联表

---

## 运维与管理

### Connection Pool (连接池)

预先创建并维护一组数据库连接，应用需要时从池中获取、用完后归还。避免频繁创建/销毁连接的开销（TCP + TLS + 认证约 50-100ms）。

常用连接池：PgBouncer（PostgreSQL 外部代理）、ProxySQL（MySQL）、HikariCP（Java）、SQLAlchemy Pool（Python）。

### ORM (Object-Relational Mapping)

对象关系映射，将数据库表映射为编程语言的类/对象，SQL 操作映射为方法调用。

代表框架：SQLAlchemy（Python）、Django ORM（Python）、Prisma（TypeScript）、GORM（Go）、Hibernate（Java）。

好处：开发效率高、防 SQL 注入、数据库可移植。
风险：N+1 查询、生成低效 SQL、隐藏性能问题。需要理解 ORM 生成的 SQL 并定期审查。

### Migration (数据库迁移)

以版本化脚本管理数据库 Schema 变更（建表、加列、改索引等）。每个迁移文件包含 up（应用变更）和 down（回滚变更）。

工具：Alembic（SQLAlchemy）、Django Migrations、Prisma Migrate、Flyway（Java）、golang-migrate。

最佳实践：
- 每个迁移只做一件事
- 大表 DDL 使用在线迁移工具（gh-ost、pg_repack）
- 在 CI 中验证迁移脚本可正向和反向执行

### PITR (Point-in-Time Recovery)

时间点恢复，利用全量备份 + WAL 日志将数据库恢复到过去任意时间点的状态。

典型场景：误删数据后恢复到删除前 1 秒。PostgreSQL 通过 `restore_command` + `recovery_target_time` 实现。RPO 取决于 WAL 归档频率（通常 < 5 分钟）。

### VACUUM (清理回收)

PostgreSQL 特有的维护操作：

- **VACUUM**：标记已删除行占用的空间可重用（不缩小文件）
- **VACUUM FULL**：物理收缩表文件（需要排它锁，会阻塞读写）
- **autovacuum**：自动触发 VACUUM 和 ANALYZE，生产环境必须开启
- 还负责防止事务 ID 回卷（Transaction ID Wraparound）

### 慢查询日志 (Slow Query Log)

记录执行时间超过阈值的 SQL 语句：

- **PostgreSQL**：`log_min_duration_statement = 200`（记录 > 200ms 的查询）
- **MySQL**：`slow_query_log = 1` + `long_query_time = 0.2`

是性能调优的第一手数据源，应在生产环境持续开启。配合 pgBadger（PostgreSQL）或 pt-query-digest（MySQL）分析。

### pg_stat_statements

PostgreSQL 的查询统计扩展，记录所有 SQL 的：执行次数、总耗时、平均耗时、最大/最小耗时、I/O 统计（shared blocks hit/read）等。

用于识别高频查询和慢查询。`CREATE EXTENSION pg_stat_statements;` 安装后通过同名视图查询。是 PostgreSQL 性能分析的必备扩展。

### 死锁 (Deadlock)

两个或多个事务相互等待对方持有的锁，导致所有相关事务永远无法继续。数据库通过死锁检测器定期检查等待图（Wait-for Graph），发现死锁后回滚代价最小的事务。

预防措施：
- 按固定顺序获取锁
- 缩短事务持续时间
- 使用乐观锁替代悲观锁
- 设置合理的 lock_timeout

### 乐观锁 (Optimistic Locking)

假设并发冲突很少发生，读取时不加锁，提交时检查数据是否被修改：

```sql
UPDATE products SET stock = stock - 1, version = version + 1
WHERE id = 100 AND version = 3;
-- affected_rows = 0 则说明被其他事务修改，需重试
```

适合读多写少、冲突率低的场景。

### 悲观锁 (Pessimistic Locking)

假设并发冲突频繁，读取时即加锁阻止其他事务修改：

```sql
SELECT * FROM products WHERE id = 100 FOR UPDATE;  -- 行级排它锁
SELECT * FROM products WHERE id = 100 FOR SHARE;   -- 行级共享锁
```

锁持续到事务结束。适合写多、冲突率高的场景（如库存扣减、转账）。

---

## Agent Checklist

- [ ] 团队成员理解 ACID 与 BASE 的适用场景差异
- [ ] 索引类型选择正确（B-tree / GIN / GiST / Hash）
- [ ] 分布式架构中 CAP 权衡已明确记录
- [ ] 分片键选择已评估数据均匀性和查询模式
- [ ] MVCC 和事务隔离级别已根据业务场景配置
- [ ] 复杂查询使用 EXPLAIN ANALYZE 验证执行计划
- [ ] CTE 和 Window Function 替代了自连接和子查询
- [ ] JSONB 字段已创建 GIN 索引
- [ ] 连接池参数已根据并发量调优
- [ ] Migration 脚本包含 up 和 down 两个方向
- [ ] PITR 配置已验证可恢复到指定时间点
- [ ] 慢查询日志和 pg_stat_statements 已开启
- [ ] 死锁预防策略已在文档中记录
