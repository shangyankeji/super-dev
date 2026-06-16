---
id: case-database-migration
title: 数据库迁移案例：MySQL 到 PostgreSQL 的实战迁移
domain: data
category: 05-cases
difficulty: intermediate
tags: [10-11, agent, case, checklist, data, database, migration, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# 数据库迁移案例：MySQL 到 PostgreSQL 的实战迁移

## 概述

本案例记录一个中型 SaaS 平台从 MySQL 5.7 迁移到 PostgreSQL 15 的完整过程。系统包含 120 张表，总数据量 800GB，日均写入 500 万行。迁移目标：零数据丢失、停机时间 < 30 分钟。团队规模 4 人（2 后端 + 1 DBA + 1 QA），总周期 12 周。

### 迁移动机

1. MySQL 5.7 即将 EOL，升级到 MySQL 8.0 的兼容性问题多
2. 业务需要 JSON 字段高级查询（PostgreSQL JSONB 支持更好）
3. 需要 CTE（递归查询）处理组织树结构
4. 需要物化视图支持复杂报表场景
5. 许可成本考虑（团队更熟悉 PostgreSQL 生态）

---

## 第一阶段：评估与规划（第 1-2 周）

### 兼容性评估

| MySQL 特性 | PostgreSQL 等价 | 迁移难度 |
|-----------|----------------|---------|
| AUTO_INCREMENT | SERIAL / IDENTITY | 低 |
| TINYINT(1) 布尔 | BOOLEAN | 低 |
| ENUM 类型 | CREATE TYPE ... AS ENUM | 中 |
| ON UPDATE CURRENT_TIMESTAMP | 触发器实现 | 中 |
| GROUP_CONCAT | STRING_AGG | 低 |
| IFNULL | COALESCE | 低 |
| LIMIT offset, count | LIMIT count OFFSET offset | 低 |
| 反引号标识符 | 双引号标识符 | 低 |
| utf8mb4 | UTF8（默认即 UTF-8） | 低 |
| 存储过程 | 需要重写（PL/pgSQL 语法不同） | 高 |
| 全文搜索 MATCH AGAINST | tsvector + tsquery | 高 |

### 风险评估

| 风险项 | 概率 | 影响 | 缓解措施 |
|--------|------|------|---------|
| 数据类型不兼容 | 中 | 高 | 提前做类型映射表，逐表验证 |
| SQL 语法差异 | 高 | 中 | 应用层全面回归测试 |
| 性能差异 | 中 | 高 | 关键查询在 PG 上重新 benchmark |
| 迁移期间数据不一致 | 低 | 极高 | 双写 + 校验脚本 |
| 回滚复杂 | 低 | 高 | 保留 MySQL 实例 2 周不下线 |

### 迁移方案选定

经过评估，选择 **双写双读 + 灰度切换** 方案：

```
阶段 1：MySQL 为主，PG 为从（双写 MySQL + PG，读 MySQL）
阶段 2：PG 为主，MySQL 为从（双写 PG + MySQL，读 PG）
阶段 3：仅 PG（停止 MySQL 写入，保留观察期）
```

---

## 第二阶段：Schema 迁移（第 3-4 周）

### DDL 转换规则

```sql
-- MySQL 原始表
CREATE TABLE orders (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT UNSIGNED NOT NULL,
    status ENUM('pending', 'paid', 'shipped', 'completed', 'cancelled'),
    total_amount DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    metadata JSON,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_user_status (user_id, status),
    INDEX idx_created (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- PostgreSQL 转换后
CREATE TYPE order_status AS ENUM ('pending', 'paid', 'shipped', 'completed', 'cancelled');

CREATE TABLE orders (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id BIGINT NOT NULL,
    status order_status,
    total_amount NUMERIC(10,2) NOT NULL DEFAULT 0.00,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_orders_user_status ON orders(user_id, status);
CREATE INDEX idx_orders_created ON orders(created_at);
CREATE INDEX idx_orders_metadata ON orders USING GIN (metadata);

-- updated_at 自动更新触发器
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_orders_updated_at
    BEFORE UPDATE ON orders
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
```

### 关键类型映射

| MySQL 类型 | PostgreSQL 类型 | 注意事项 |
|-----------|----------------|---------|
| BIGINT UNSIGNED | BIGINT | PG 无 unsigned，值域需确认 |
| TINYINT(1) | BOOLEAN | 需转换 0/1 为 false/true |
| DATETIME | TIMESTAMPTZ | 统一使用带时区类型 |
| DOUBLE | DOUBLE PRECISION | 精度一致 |
| DECIMAL(M,D) | NUMERIC(M,D) | 语法兼容 |
| TEXT / LONGTEXT | TEXT | PG 无长度限制 |
| BLOB | BYTEA | 二进制存储 |
| JSON | JSONB | 推荐用 JSONB（支持索引） |
| ENUM | CREATE TYPE AS ENUM | 需先创建类型 |

### Schema 迁移工具

使用 `pgloader` 自动转换 + 手动修正：

```bash
pgloader mysql://user:pass@mysql-host/mydb postgresql://user:pass@pg-host/mydb

# pgloader 自动处理：类型映射、索引转换、数据迁移
# 手动修正：ENUM 类型、触发器、存储过程、全文搜索索引
```

---

## 第三阶段：应用层适配（第 5-7 周）

### ORM 层修改

```
1. 数据库驱动切换
   - mysql2 → pg (Node.js)
   - 或 Prisma / TypeORM 切换 provider

2. 原生 SQL 修改（共 47 处）
   - 反引号 → 双引号（标识符）
   - IFNULL → COALESCE
   - GROUP_CONCAT → STRING_AGG
   - LIMIT x, y → LIMIT y OFFSET x
   - NOW() → NOW()（兼容，无需改）
   - DATE_FORMAT → TO_CHAR
   - STR_TO_DATE → TO_TIMESTAMP

3. 布尔值处理
   - MySQL 的 0/1 → PostgreSQL 的 true/false
   - ORM 层自动处理，原生 SQL 需修改

4. 自增 ID 获取
   - MySQL: LAST_INSERT_ID()
   - PostgreSQL: RETURNING id（INSERT ... RETURNING id）
```

### 双写层实现

```
架构：
  Application → WriteProxy → MySQL (主)
                           → PostgreSQL (从)

WriteProxy 逻辑：
  1. 先写 MySQL（主库）
  2. 异步写 PostgreSQL（失败记入重试队列）
  3. 写入结果以 MySQL 为准
  4. 定期校验两库数据一致性
```

### 全面回归测试

- 单元测试：将数据库 mock 切换为 PostgreSQL 行为验证
- 集成测试：全部对 PostgreSQL 运行，覆盖 120 个 API 端点
- 数据校验脚本：逐表对比行数、校验和、采样数据

---

## 第四阶段：数据全量迁移（第 8-9 周）

### 迁移步骤

```
1. 停止非关键写入（维护窗口外的批处理任务）
2. 使用 pgloader 全量迁移 800GB 数据
   - 耗时约 6 小时（使用 8 并行线程）
   - 迁移期间 MySQL 正常服务

3. 启用增量同步
   - 使用 Debezium 监听 MySQL binlog
   - 实时同步增量变更到 PostgreSQL
   - 延迟 < 5 秒

4. 数据校验
   - 全表行数对比（120 表全部匹配）
   - 关键表随机采样 10000 行对比（无差异）
   - 聚合数据校验（SUM/COUNT/MAX/MIN）
```

### 迁移遇到的问题

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| 字符编码错误 | MySQL 中存在非法 UTF-8 字符 | 迁移前清洗数据 |
| 自增序列不连续 | pgloader 不自动设置序列起始值 | 手动 `SELECT setval()` |
| ENUM 值大小写 | MySQL ENUM 不区分大小写 | 应用层统一为小写 |
| 时区问题 | MySQL DATETIME 无时区 | 确认服务器时区一致后迁移为 TIMESTAMPTZ |
| 外键约束失败 | 迁移顺序不当 | 先禁用外键，全部迁移后再启用并验证 |

---

## 第五阶段：灰度切换（第 10-11 周）

### 切换步骤

```
第 10 周：10% 读流量切到 PostgreSQL
  - 监控响应时间、错误率
  - 发现 3 个查询性能劣化，优化索引后解决

第 10.5 周：50% 读流量切到 PostgreSQL
  - 稳定运行 3 天
  - 无异常

第 11 周：100% 读流量切到 PostgreSQL
  - 写仍走 MySQL，双写到 PostgreSQL
  - 稳定运行 3 天

第 11.5 周：写流量切到 PostgreSQL
  - 停止 MySQL 写入
  - PostgreSQL 成为唯一主库
  - MySQL 保留只读，作为回滚保险
```

### 性能对比

| 查询类型 | MySQL 延迟 | PostgreSQL 延迟 | 变化 |
|---------|-----------|----------------|------|
| 简单主键查询 | 0.8ms | 0.6ms | -25% |
| 复杂 JOIN 查询 | 45ms | 32ms | -29% |
| JSON 字段查询 | 120ms | 18ms | -85% |
| 聚合报表查询 | 3200ms | 980ms | -69% |
| 全文搜索 | 85ms | 42ms | -51% |

---

## 第六阶段：收尾与清理（第 12 周）

### 操作清单

- [ ] 确认 MySQL 停止接收写入已超过 7 天
- [ ] 最终数据一致性校验通过
- [ ] 移除应用中的双写逻辑和 MySQL 连接配置
- [ ] 移除 Debezium 增量同步
- [ ] MySQL 实例做最终全量备份后下线
- [ ] 更新运维文档（监控、备份、恢复流程）
- [ ] 更新开发者文档（连接信息、SQL 方言差异）
- [ ] 回顾会议总结经验教训

---

## 迁移总结

### 最终成果

| 指标 | 迁移前（MySQL） | 迁移后（PostgreSQL） |
|------|-----------------|---------------------|
| 平均查询延迟 | 35ms | 22ms（-37%） |
| JSON 查询延迟 | 120ms | 18ms（-85%） |
| 报表生成时间 | 3.2s | 0.98s（-69%） |
| 停机时间 | - | 22 分钟（写切换窗口） |
| 数据丢失 | - | 0 行 |

### 经验教训

1. **双写方案虽复杂但安全** - 相比大爆炸切换，灰度切换让我们有足够信心
2. **数据校验是核心** - 投入在校验脚本上的时间回报最高
3. **SQL 兼容性低估** - 47 处原生 SQL 修改超出预期，应更早开始审查
4. **性能不是问题** - PostgreSQL 在所有场景下性能持平或更好
5. **保留旧库是保险** - MySQL 保留 2 周给了团队安全感

---

## Agent Checklist

以下为 AI Agent 在执行数据库迁移任务时必须遵循的硬约束：

- [ ] 迁移前完成源库全量备份
- [ ] 建立完整的类型映射表并逐表验证
- [ ] 所有原生 SQL 语句审查并标记需修改项
- [ ] 迁移后运行全表行数对比校验
- [ ] 迁移后运行关键表采样数据校验
- [ ] 迁移后运行聚合数据校验（SUM/COUNT）
- [ ] 关键查询在目标库的 EXPLAIN ANALYZE 结果已确认
- [ ] 灰度切换方案已定义（不允许一次性全切）
- [ ] 回滚方案已验证可执行
- [ ] 生成迁移报告包含：数据校验结果、性能对比、问题记录
