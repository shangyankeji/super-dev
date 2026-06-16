---
id: case-db-hotspot-optimization
title: 案例研究：数据库热点治理——核心查询延迟降低 95%
domain: development
category: 05-cases
difficulty: intermediate
tags: [agent, case, checklist, development, hotspot, optimization, 元数据]
quality_score: 70
last_updated: 2026-06-15
---
# 案例研究：数据库热点治理——核心查询延迟降低 95%

## 元数据

| 字段 | 值 |
|------|------|
| 行业 | 金融科技（消费信贷平台） |
| 系统规模 | 日放款 20 万笔，峰值 QPS 6,000 |
| 技术栈 | Java Spring Boot + MySQL 8.0 + Redis |
| 数据规模 | 核心表 12 亿行，日增 500 万行 |
| 团队规模 | 后端 16 人，DBA 2 人 |
| 治理周期 | 4 周（2024-05 至 2024-06） |
| 核心目标 | 放款查询 P99 从 3.2s 降到 150ms |

---

## 一、背景

### 1.1 业务场景

某消费信贷平台的核心业务流程：

```
用户申请 → 风控审核 → 额度计算 → 放款 → 还款
```

其中**放款查询**是高频操作：
- 用户查看放款进度（APP 首页轮询，每 5 秒一次）
- 风控系统查询历史放款记录（批量查询）
- 运营后台查询放款报表（复杂聚合）
- 催收系统查询逾期放款（定时批量）

### 1.2 问题表现

2024 年 5 月，随着业务量增长，放款相关查询出现严重性能劣化：

| 指标 | 正常值 | 当前值 | 影响 |
|------|--------|--------|------|
| 放款查询 P50 | 30ms | 450ms | 用户感知明显卡顿 |
| 放款查询 P99 | 150ms | 3,200ms | APP 超时，客诉激增 |
| MySQL CPU | < 60% | 88-95% | 数据库濒临崩溃 |
| 慢查询/小时 | < 10 | 850 | 影响所有查询性能 |
| InnoDB 行锁等待 | < 100/s | 2,800/s | 写操作严重阻塞 |

### 1.3 核心表结构

```sql
-- 放款记录表（12 亿行，580GB）
CREATE TABLE loan_records (
    id              BIGINT PRIMARY KEY AUTO_INCREMENT,
    loan_no         VARCHAR(32) UNIQUE,
    user_id         BIGINT NOT NULL,
    product_id      INT NOT NULL,
    amount          DECIMAL(15,2) NOT NULL,
    status          TINYINT NOT NULL,  -- 0:处理中 1:已放款 2:已结清 3:逾期 4:已关闭
    apply_time      DATETIME NOT NULL,
    loan_time       DATETIME,
    due_date        DATE,
    channel_id      INT NOT NULL,
    merchant_id     INT NOT NULL,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_user_id (user_id),
    INDEX idx_status (status),
    INDEX idx_loan_time (loan_time),
    INDEX idx_channel (channel_id),
    INDEX idx_merchant (merchant_id)
) ENGINE=InnoDB;
```

---

## 二、分析过程

### 2.1 慢查询采集与分类

开启 MySQL 慢查询日志（阈值 200ms），采集 24 小时数据后分类：

| 类别 | 查询模式 | 频率 | 平均延迟 | 占比 |
|------|----------|------|----------|------|
| A | 用户查询自己的放款列表 | 3,200 QPS | 450ms | 45% |
| B | 按状态统计放款笔数/金额 | 200 QPS | 2,800ms | 25% |
| C | 按渠道+时间范围查询 | 150 QPS | 1,500ms | 15% |
| D | 按商户+日期查询逾期 | 80 QPS | 3,200ms | 10% |
| E | 其他 | 50 QPS | 800ms | 5% |

### 2.2 逐类分析

#### 类别 A：用户查询放款列表

```sql
-- 高频查询（3,200 QPS）
SELECT * FROM loan_records
WHERE user_id = 12345
ORDER BY created_at DESC
LIMIT 20;
```

EXPLAIN 分析：

```
+----+-------------+--------------+------+---------------+-------------+---------+-------+------+-----------+
| id | select_type | table        | type | possible_keys | key         | key_len | ref   | rows | Extra     |
+----+-------------+--------------+------+---------------+-------------+---------+-------+------+-----------+
|  1 | SIMPLE      | loan_records | ref  | idx_user_id   | idx_user_id | 8       | const | 4521 | Using ... |
|    |             |              |      |               |             |         |       |      | filesort  |
+----+-------------+--------------+------+---------------+-------------+---------+-------+------+-----------+
```

**问题**：
1. `idx_user_id` 索引只包含 `user_id`，查到 4521 行后需要回表取全部字段
2. ORDER BY created_at 需要 filesort（索引无法覆盖排序）
3. SELECT * 取了 15 个字段，大部分前端不需要

#### 类别 B：状态统计

```sql
-- 运营报表（200 QPS）
SELECT status, COUNT(*) as cnt, SUM(amount) as total_amount
FROM loan_records
WHERE loan_time BETWEEN '2024-05-01' AND '2024-05-31'
GROUP BY status;
```

**问题**：
1. `idx_loan_time` 索引扫描 500 万行（整月数据）
2. 每行都需要回表取 status 和 amount 字段
3. GROUP BY 使用临时表

#### 类别 C：渠道查询

```sql
-- 渠道管理（150 QPS）
SELECT * FROM loan_records
WHERE channel_id = 5
  AND loan_time BETWEEN '2024-05-01' AND '2024-05-07'
  AND status IN (0, 1)
ORDER BY loan_time DESC
LIMIT 50;
```

**问题**：
1. `idx_channel` 只有 channel_id，无法利用 loan_time 和 status 条件
2. 优化器在 idx_channel 和 idx_loan_time 之间选择不稳定（有时全表扫描）

#### 类别 D：商户逾期查询

```sql
-- 催收系统（80 QPS）
SELECT * FROM loan_records
WHERE merchant_id = 100
  AND status = 3  -- 逾期
  AND due_date < '2024-05-15'
ORDER BY amount DESC
LIMIT 100;
```

**问题**：
1. `idx_merchant` 无法覆盖 status 和 due_date 条件
2. 逾期记录分布不均，大商户下有数百万条记录

### 2.3 热点行锁分析

```sql
-- 查看行锁等待
SELECT * FROM performance_schema.data_lock_waits;
```

发现热点行锁集中在：
1. **状态更新**：放款成功后 UPDATE status = 1，每秒 3000+ 次
2. **同一用户并发**：用户反复刷新导致同一行被并发读取和更新

---

## 三、优化方案

### 3.1 索引优化

```sql
-- 1. 用户查询：覆盖索引（消除回表 + filesort）
CREATE INDEX idx_user_created ON loan_records(user_id, created_at DESC);

-- 2. 渠道查询：组合索引
CREATE INDEX idx_channel_time_status ON loan_records(channel_id, loan_time, status);

-- 3. 商户逾期查询：组合索引
CREATE INDEX idx_merchant_status_due ON loan_records(merchant_id, status, due_date);

-- 4. 删除冗余索引
DROP INDEX idx_status ON loan_records;     -- 区分度太低（5 个状态值），几乎不被优化器选择
DROP INDEX idx_channel ON loan_records;    -- 被 idx_channel_time_status 覆盖
DROP INDEX idx_merchant ON loan_records;   -- 被 idx_merchant_status_due 覆盖
```

#### 索引添加策略（12 亿行大表）

直接 ALTER TABLE 在 12 亿行表上执行预计耗时 6 小时，使用 pt-online-schema-change 在线添加：

```bash
# 在线添加索引，不锁表
pt-online-schema-change \
  --alter "ADD INDEX idx_user_created (user_id, created_at DESC)" \
  --execute \
  --max-load "Threads_running=50" \
  --critical-load "Threads_running=100" \
  --chunk-size 5000 \
  --progress time,30 \
  D=loan_db,t=loan_records
```

### 3.2 查询改写

#### 类别 A 查询改写

```sql
-- 改写前：SELECT * + filesort
SELECT * FROM loan_records
WHERE user_id = 12345 ORDER BY created_at DESC LIMIT 20;

-- 改写后：指定字段 + 利用覆盖索引
SELECT id, loan_no, amount, status, loan_time, created_at
FROM loan_records
WHERE user_id = 12345
ORDER BY created_at DESC
LIMIT 20;
```

改写后 EXPLAIN：
```
type: ref, key: idx_user_created, rows: 20, Extra: Using index condition
```
消除了 filesort，扫描行数从 4521 降到 20。

#### 类别 B 报表查询改写

```sql
-- 改写方案：预计算 + 物化视图

-- 创建日级汇总表
CREATE TABLE loan_daily_stats (
    stat_date    DATE NOT NULL,
    status       TINYINT NOT NULL,
    channel_id   INT NOT NULL,
    merchant_id  INT NOT NULL,
    loan_count   INT NOT NULL DEFAULT 0,
    total_amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    PRIMARY KEY (stat_date, status, channel_id, merchant_id)
) ENGINE=InnoDB;

-- 每小时定时任务增量更新
INSERT INTO loan_daily_stats (stat_date, status, channel_id, merchant_id, loan_count, total_amount)
SELECT DATE(loan_time), status, channel_id, merchant_id, COUNT(*), SUM(amount)
FROM loan_records
WHERE updated_at >= NOW() - INTERVAL 2 HOUR
GROUP BY DATE(loan_time), status, channel_id, merchant_id
ON DUPLICATE KEY UPDATE
    loan_count = VALUES(loan_count),
    total_amount = VALUES(total_amount);

-- 报表查询改为查汇总表
SELECT status, SUM(loan_count), SUM(total_amount)
FROM loan_daily_stats
WHERE stat_date BETWEEN '2024-05-01' AND '2024-05-31'
GROUP BY status;
-- 扫描 31 天 x 5 状态 x N 渠道 ≈ 几千行（vs 原来 500 万行）
```

### 3.3 缓存层优化

```java
// 用户放款列表缓存
@Cacheable(value = "user:loans", key = "#userId", unless = "#result == null")
public List<LoanBriefDTO> getUserLoans(Long userId, int page) {
    // 缓存未命中时查数据库
    return loanMapper.selectByUserId(userId, page);
}

// 写操作时失效缓存
@CacheEvict(value = "user:loans", key = "#loan.userId")
public void updateLoanStatus(Loan loan) {
    loanMapper.updateStatus(loan);
}

// 缓存 TTL 策略
// 处理中的放款：TTL 30 秒（用户频繁刷新查进度）
// 已结清/已关闭：TTL 30 分钟（状态不再变化）
```

### 3.4 热点行锁治理

```java
// 问题：大量并发 UPDATE 同一行（放款状态更新）
// 方案：消息队列削峰 + 合并更新

// 改造前：同步更新，高并发时行锁等待
public void onLoanSuccess(Long loanId) {
    loanMapper.updateStatus(loanId, LOAN_SUCCESS); // 直接更新
}

// 改造后：异步队列 + 批量更新
public void onLoanSuccess(Long loanId) {
    // 发送到 RocketMQ，按 loanId 取模分区（保证顺序）
    mqProducer.send("loan-status-update", loanId, new StatusUpdate(loanId, LOAN_SUCCESS));
}

// 消费者：批量拉取 + 合并更新
@RocketMQMessageListener(topic = "loan-status-update")
public void onMessage(List<StatusUpdate> batch) {
    // 按 loanId 去重（同一笔放款只取最后一条）
    Map<Long, StatusUpdate> merged = dedup(batch);
    // 批量更新
    loanMapper.batchUpdateStatus(merged);
}
```

### 3.5 读写分离 + 查询路由

```
查询路由策略：
├── 用户查自己的放款（实时性要求高）→ 主库
├── 运营报表查询 → 从库 / 汇总表
├── 渠道管理查询 → 从库
├── 催收系统查询 → 从库
└── 数据导出 → 专用从库（避免影响线上）
```

---

## 四、实施步骤

### 4.1 Week 1：索引优化 + 查询改写

```
Day 1: 在从库验证新索引效果
  - 在从库添加 3 个新索引
  - 用线上慢查询 SQL 验证执行计划
  - 确认优化效果后安排主库变更

Day 2-3: 主库在线添加索引
  - 使用 pt-online-schema-change 添加 3 个索引
  - 分别耗时：2h, 3h, 2.5h
  - 监控主从延迟和主库负载

Day 4: 查询改写上线
  - 类别 A: SELECT * → 指定字段
  - 类别 B: 暂时加 FORCE INDEX 强制使用新索引
  - 灰度 10% → 50% → 100%

Day 5: 删除冗余索引
  - 确认新索引生效后删除 3 个旧索引
  - 释放磁盘空间 ~45GB
```

### 4.2 Week 2：缓存层建设

```
Day 1-2: 用户放款列表缓存
  - Redis 缓存层开发 + 单元测试
  - 缓存失效策略实现

Day 3-4: 汇总表建设
  - loan_daily_stats 表创建
  - 历史数据回填（3 个月）
  - 定时更新任务上线

Day 5: 报表查询切换
  - 运营报表改为查询汇总表
  - 灰度上线
```

### 4.3 Week 3：热点行锁治理 + 读写分离

```
Day 1-3: 状态更新异步化
  - RocketMQ 消费者开发
  - 幂等性保障
  - 灰度上线

Day 4-5: 读写分离
  - ProxySQL 路由规则配置
  - 各类查询路由测试
```

### 4.4 Week 4：压测验证 + 监控

```
Day 1-2: 全链路压测
  - 模拟 6,000 QPS 峰值场景
  - 模拟 Redis 故障降级场景
  - 验证所有优化效果

Day 3-5: 监控体系完善
  - 慢查询实时告警（阈值 100ms）
  - 索引使用率监控
  - 行锁等待监控
  - Grafana Dashboard 搭建
```

---

## 五、结果数据

### 5.1 查询性能对比

| 查询类别 | 优化前 P99 | 优化后 P99 | 改善幅度 |
|----------|-----------|-----------|----------|
| A: 用户放款列表 | 3,200ms | 25ms（缓存命中）/ 80ms（未命中） | -97% |
| B: 状态统计报表 | 2,800ms | 45ms（查汇总表） | -98% |
| C: 渠道查询 | 1,500ms | 120ms | -92% |
| D: 商户逾期查询 | 3,200ms | 95ms | -97% |
| **综合 P99** | **3,200ms** | **120ms** | **-96%** |

### 5.2 数据库指标

| 指标 | 优化前 | 优化后 | 改善 |
|------|--------|--------|------|
| MySQL CPU | 88-95% | 35-45% | -55% |
| 慢查询/小时 | 850 | 3 | -99.6% |
| 行锁等待/秒 | 2,800 | 120 | -95.7% |
| 磁盘 IOPS | 12,000 | 4,500 | -62.5% |
| 索引空间 | 180GB | 135GB（删冗余后） | -25% |

### 5.3 业务指标

| 指标 | 优化前 | 优化后 |
|------|--------|--------|
| APP 放款查询超时率 | 2.8% | 0.01% |
| 客诉"页面卡顿" | 日均 45 条 | 日均 2 条 |
| 放款成功率 | 98.5%（行锁超时导致失败） | 99.95% |

---

## 六、经验教训

### 6.1 做对的事

1. **先分析再优化**：通过慢查询分类发现了 4 类不同的问题，每类用不同方案解决，比统一加缓存更有效
2. **覆盖索引消除回表**：类别 A 查询从 4521 行扫描降到 20 行，是索引优化的典型收益
3. **汇总表替代实时聚合**：报表查询的数据量从 500 万行降到几千行，本质是空间换时间
4. **异步削峰**：将高并发的状态更新改为消息队列异步处理，消除了行锁争用的根源
5. **先从库验证**：新索引先在从库验证效果，避免在主库做无效变更

### 6.2 做错的事

1. **初期想用 SELECT ... FOR UPDATE 解决并发**：实际上加了更多锁，性能更差。正确方案是异步化
2. **pt-osc 执行期间未控制并发**：第一个索引添加时 pt-osc chunk 过大，导致主库短暂抖动。后调小 chunk-size
3. **汇总表初期未考虑数据修正**：放款状态变更后汇总数据不准确，后增加了修正逻辑

### 6.3 关键认知

- 数据库热点治理必须同时看**索引、查询、缓存、并发控制**四个维度
- 12 亿行大表的任何 Schema 变更都必须用在线 DDL 工具
- `SELECT *` 是性能杀手，尤其是在大表上——只查需要的字段
- 单列索引在复杂查询中几乎无用，组合索引才是正解
- 报表查询和 OLTP 查询必须分离，一个慢查询可以拖垮整个数据库
- 索引不是越多越好，冗余索引浪费空间和写入性能

---

## Agent Checklist

在 AI Agent 辅助执行数据库热点治理时，应逐项确认：

- [ ] **慢查询采集**：是否开启了慢查询日志并按频率/延迟分类
- [ ] **EXPLAIN 分析**：每个慢查询是否执行了 EXPLAIN ANALYZE
- [ ] **索引诊断**：是否检查了现有索引的使用率和覆盖度
- [ ] **组合索引**：多条件查询是否设计了合适的组合索引
- [ ] **覆盖索引**：高频查询是否可以通过覆盖索引消除回表
- [ ] **冗余索引**：是否识别并删除了冗余/无效索引
- [ ] **查询改写**：是否消除了 SELECT * 和不必要的 JOIN
- [ ] **读写分离**：报表/分析查询是否路由到从库
- [ ] **缓存设计**：高频读查询是否有缓存层保护
- [ ] **汇总表**：聚合查询是否通过预计算/物化视图加速
- [ ] **行锁治理**：高并发写操作是否有削峰/异步/合并策略
- [ ] **在线 DDL**：大表的索引/字段变更是否使用了在线 DDL 工具
- [ ] **压测验证**：优化后是否通过压测验证了目标 QPS 和延迟
- [ ] **监控告警**：是否建立了慢查询/锁等待/CPU 的实时监控和告警
