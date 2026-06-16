---
id: database-antipatterns
title: 数据库设计反模式（避坑指南）
domain: database
category: 04-antipatterns
difficulty: intermediate
tags: [database, sql, postgresql, antipattern, n-plus-1, eav, premature-optimization, no-index, float-money, enum-type]
quality_score: 88
maintainer: platform-team@superdev.com
last_updated: 2024-06-14
---

# 数据库设计反模式（避坑指南）

## 1. 用 FLOAT 存金额
```sql
-- ❌ 浮点丢精度
price FLOAT  -- 0.1 + 0.2 = 0.30000000000000004

-- ✅ 定点数或整数分
price NUMERIC(19,2)    -- 精确
price_cents INTEGER    -- 整数，绝对精确
```

## 2. EAV（实体-属性-值）反模式
```sql
-- ❌ 万能表（查询噩梦，无类型安全）
CREATE TABLE attributes (
    entity_id UUID,
    attribute_name TEXT,
    attribute_value TEXT
);

-- ✅ 显式列 + JSONB 补充
CREATE TABLE products (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    price_cents INTEGER NOT NULL,
    extra_attrs JSONB DEFAULT '{}'  -- 灵活字段用 JSONB + GIN
);
```

## 3. N+1 查询
```python
# ❌ 循环内查子资源（100 用户 = 101 次查询）
for user in users:
    user.orders = db.query("SELECT * FROM orders WHERE user_id = ?", user.id)

# ✅ 批量查 + 内存关联（100 用户 = 2 次查询）
orders = db.query("SELECT * FROM orders WHERE user_id IN ?", [u.id for u in users])
# 或 ORM eager loading
users = db.query(User).options(selectinload(User.orders)).all()
```

## 4. 无索引的外键
```sql
-- ❌ FK 约束创建后，默认无索引！JOIN/WHERE 会全表扫描
ALTER TABLE orders ADD CONSTRAINT fk_orders_users FOREIGN KEY (user_id) REFERENCES users(id);
-- 忘了建索引！

-- ✅ 每个 FK 列必须有索引
CREATE INDEX idx_orders_user_id ON orders(user_id);
```

## 5. ENUM 类型（不灵活）
```sql
-- ❌ ENUM 加值需要 ALTER TYPE（锁表）
CREATE TYPE order_status AS ENUM ('pending', 'paid');

-- ✅ TEXT + CHECK（加值只需改约束）
status TEXT NOT NULL DEFAULT 'pending'
    CHECK (status IN ('pending', 'paid', 'shipped', 'cancelled'))
```

## 6. 不带 LIMIT 的查询
```sql
-- ❌ 全表扫描（10 万行 OOM）
SELECT * FROM events ORDER BY created_at DESC;

-- ✅ 强制分页
SELECT * FROM events ORDER BY created_at DESC LIMIT 100;
```

## 7. 过度规范化（过度拆表）
```sql
-- ❌ 把 email 拆到单独表（每次查用户都要 JOIN）
CREATE TABLE user_emails (user_id UUID, email TEXT);

-- ✅ 高频字段内联，只有多值/大字段才拆表
CREATE TABLE users (id UUID, email TEXT, name TEXT, ...);
```

## 8. 同步锁表 Migration
```sql
-- ❌ 生产环境直接加索引（锁写操作）
CREATE INDEX idx_big_table_col ON big_table(col);

-- ✅ 并发建索引（不锁写）
CREATE INDEX CONCURRENTLY idx_big_table_col ON big_table(col);
```

## 9. 不用事务
```python
# ❌ 转账操作不在事务里（可能转出成功转入失败）
db.execute("UPDATE accounts SET balance -= 100 WHERE id = ?", from_id)
db.execute("UPDATE accounts SET balance += 100 WHERE id = ?", to_id)  # 如果这步崩了？

# ✅ 事务包裹
with db.transaction():
    db.execute("UPDATE accounts SET balance -= 100 WHERE id = ?", from_id)
    db.execute("UPDATE accounts SET balance += 100 WHERE id = ?", to_id)
```
