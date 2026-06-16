---
id: database-schema-standards
title: 数据库 Schema 设计标准
domain: database
category: 01-standards
difficulty: intermediate
tags: [database, postgresql, schema, migration, index, foreign-key, normalization, sql, postgres, mysql]
quality_score: 91
maintainer: platform-team@superdev.com
last_updated: 2026-06-14
---

# 数据库 Schema 设计标准

## 命名约定

### 表名
```sql
-- ✅ 复数蛇形命名
CREATE TABLE users (...);
CREATE TABLE order_items (...);
CREATE TABLE product_categories (...);

-- ❌ 单数 / 驼峰 / 前缀
CREATE TABLE user (...);
CREATE TABLE OrderItems (...);
CREATE TABLE tbl_users (...);
```

### 列名
```sql
-- ✅ 蛇形，语义清晰
created_at, updated_at, deleted_at
owner_id, project_id, assignee_id
is_active, is_deleted
email_verified_at

-- ❌ 缩写 / 匈牙利命名
dt_cre, strName, bolActive
```

### 索引名
```sql
-- 格式：idx_{table}_{columns}
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_status_created ON orders(status, created_at);

-- 唯一索引：uq_{table}_{columns}
CREATE UNIQUE INDEX uq_users_email ON users(email);

-- 外键约束：fk_{from_table}_{to_table}
ALTER TABLE orders ADD CONSTRAINT fk_orders_users FOREIGN KEY (user_id) REFERENCES users(id);
```

## 每表必备列

```sql
CREATE TABLE products (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- 业务列...
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

| 列 | 类型 | 说明 |
|---|---|---|
| `id` | UUID | 主键，gen_random_uuid() |
| `created_at` | TIMESTAMPTZ | 创建时间 |
| `updated_at` | TIMESTAMPTZ | 更新时间（触发器自动维护） |
| `deleted_at` | TIMESTAMPTZ | 软删除（可选） |

## 外键设计

```sql
-- ON DELETE 策略
ALTER TABLE order_items ADD CONSTRAINT fk_items_orders
    FOREIGN KEY (order_id) REFERENCES orders(id)
    ON DELETE CASCADE;        -- 订单删除时，明细也删除

ALTER TABLE tasks ADD CONSTRAINT fk_tasks_projects
    FOREIGN KEY (project_id) REFERENCES projects(id)
    ON DELETE SET NULL;       -- 项目删除时，任务保留但 project_id 置空

ALTER TABLE comments ADD CONSTRAINT fk_comments_users
    FOREIGN KEY (author_id) REFERENCES users(id)
    ON DELETE RESTRICT;       -- 不允许删除有评论的用户
```

| 策略 | 场景 |
|------|------|
| CASCADE | 子资源无意义（订单→明细） |
| SET NULL | 子资源独立存在（项目→任务） |
| RESTRICT | 禁止删除有依赖的记录 |

## 索引原则

```sql
-- 每个外键必须有索引
CREATE INDEX idx_orders_user_id ON orders(user_id);

-- 常用查询条件的列建索引
CREATE INDEX idx_orders_status ON orders(status);

-- 复合索引遵循最左前缀
-- 查询 WHERE user_id = ? AND status = ? 用这个索引
CREATE INDEX idx_orders_user_status ON orders(user_id, status);

-- JSONB 用 GIN
CREATE INDEX idx_products_attrs ON products USING GIN(attrs);

-- 软删除用部分索引（不索引已删数据）
CREATE INDEX idx_active_users ON users(email) WHERE deleted_at IS NULL;
```

## Migration 规范

```sql
-- 0001_init.sql — 初始 schema
-- 0002_add_indexes.sql — 索引优化
-- 0003_add_audit_log.sql — 新功能

-- 每个文件包含 UP + DOWN
-- === UP ===
CREATE TABLE products (...);
-- === DOWN ===
DROP TABLE IF EXISTS products;
```

### Migration 原则
1. **向前兼容** — 新代码能读旧 schema，旧代码能读新 schema
2. **分步迁移** — 先加列（有默认值）→ 部署代码 → 回填数据 → 加约束
3. **不锁表** — 大表加列用 `DEFAULT ... NOT NULL`（PG 11+ 不锁表）
4. **回滚方案** — 每个 migration 有对应的 down migration

## 数据类型选择

| 场景 | 类型 | 说明 |
|------|------|------|
| 主键 | UUID | 全局唯一，无顺序泄露 |
| 金额 | NUMERIC(19,2) 或 INTEGER（分） | 浮点数会丢精度 |
| 状态 | TEXT + CHECK | `CHECK (status IN ('active','inactive'))` |
| 时间 | TIMESTAMPTZ | 带时区，统一存 UTC |
| 布尔 | BOOLEAN | 不用 INTEGER 0/1 |
| JSON | JSONB | 可索引，灵活字段 |
| 枚举 | TEXT + CHECK | 比 ENUM 类型灵活（加值不需 migration） |
| 大文本 | TEXT | 无长度限制 |
