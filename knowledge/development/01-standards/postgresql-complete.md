---
id: postgresql-complete
title: PostgreSQL完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, postgresql, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# PostgreSQL完整指南

## 概述
PostgreSQL是最先进的开源关系数据库,支持复杂查询、JSON、全文搜索等特性。本指南覆盖SQL语法、性能优化、高级特性和最佳实践。

## 核心概念

### 1. 数据类型

**基本类型**:
```sql
-- 整数
SMALLINT  -- 2字节
INTEGER   -- 4字节
BIGINT    -- 8字节

-- 浮点数
REAL         -- 4字节
DOUBLE PRECISION  -- 8字节
DECIMAL(10, 2)    -- 精确小数

-- 字符串
CHAR(10)      -- 定长
VARCHAR(255)  -- 变长
TEXT          -- 无限制

-- 布尔
BOOLEAN

-- 日期时间
DATE
TIME
TIMESTAMP
TIMESTAMPTZ  -- 带时区

-- JSON
JSON
JSONB  -- 二进制JSON,更高效

-- UUID
UUID

-- 数组
INTEGER[]
TEXT[]

-- 自定义类型
CREATE TYPE user_status AS ENUM ('active', 'inactive', 'banned');
```

### 2. 表设计

**创建表**:
```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(100),
    age INTEGER CHECK (age >= 0 AND age <= 150),
    status user_status DEFAULT 'active',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_metadata ON users USING GIN(metadata);

-- 唯一约束
ALTER TABLE users ADD CONSTRAINT unique_username_email UNIQUE (username, email);

-- 外键
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    content TEXT,
    published BOOLEAN DEFAULT false,
    views INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_posts_user_id ON posts(user_id);
CREATE INDEX idx_posts_published ON posts(published) WHERE published = true;
```

### 3. 查询

**基础查询**:
```sql
-- 选择
SELECT * FROM users;
SELECT username, email FROM users WHERE status = 'active';

-- 排序
SELECT * FROM users ORDER BY created_at DESC;

-- 分页
SELECT * FROM users 
ORDER BY created_at DESC 
LIMIT 10 OFFSET 0;

-- 聚合
SELECT 
    status,
    COUNT(*) as count,
    AVG(age) as avg_age,
    MAX(created_at) as latest
FROM users
GROUP BY status
HAVING COUNT(*) > 10;

-- 连接
SELECT 
    u.username,
    p.title,
    p.views
FROM users u
INNER JOIN posts p ON u.id = p.user_id
WHERE p.published = true;

-- 子查询
SELECT * FROM users
WHERE id IN (
    SELECT user_id FROM posts
    WHERE views > 1000
);

-- CTE
WITH active_users AS (
    SELECT * FROM users WHERE status = 'active'
),
popular_posts AS (
    SELECT * FROM posts WHERE views > 500
)
SELECT 
    au.username,
    COUNT(pp.id) as popular_post_count
FROM active_users au
LEFT JOIN popular_posts pp ON au.id = pp.user_id
GROUP BY au.id;
```

**窗口函数**:
```sql
-- 排名
SELECT 
    username,
    views,
    RANK() OVER (ORDER BY views DESC) as rank,
    DENSE_RANK() OVER (ORDER BY views DESC) as dense_rank,
    ROW_NUMBER() OVER (ORDER BY views DESC) as row_num
FROM posts;

-- 分组聚合
SELECT 
    user_id,
    title,
    views,
    SUM(views) OVER (PARTITION BY user_id) as total_views,
    AVG(views) OVER (PARTITION BY user_id) as avg_views
FROM posts;

-- 移动平均
SELECT 
    created_at::DATE as date,
    views,
    AVG(views) OVER (
        ORDER BY created_at 
        ROWS BETWEEN 6 PRECEDING AND CURRENT ROW
    ) as moving_avg_7d
FROM daily_stats;
```

### 4. JSON操作

**JSONB查询**:
```sql
-- 插入JSON
INSERT INTO users (username, email, metadata)
VALUES ('alice', 'alice@example.com', '{"age": 30, "city": "Beijing", "tags": ["developer", "golang"]}');

-- 提取字段
SELECT 
    username,
    metadata->>'age' as age,
    metadata->>'city' as city
FROM users;

-- JSON路径查询
SELECT * FROM users
WHERE metadata @> '{"city": "Beijing"}';

-- JSON数组查询
SELECT * FROM users
WHERE metadata->'tags' ? 'developer';

-- 更新JSON
UPDATE users
SET metadata = jsonb_set(metadata, '{age}', '31')
WHERE username = 'alice';

-- 追加数组元素
UPDATE users
SET metadata = jsonb_set(
    metadata,
    '{tags}',
    (metadata->'tags') || '"python"'
)
WHERE username = 'alice';

-- 删除字段
UPDATE users
SET metadata = metadata - 'age'
WHERE username = 'alice';
```

### 5. 性能优化

**索引策略**:
```sql
-- B-tree索引(默认)
CREATE INDEX idx_users_username ON users(username);

-- 哈希索引(等值查询)
CREATE INDEX idx_users_email_hash ON users USING HASH(email);

-- GIN索引(JSONB、数组、全文搜索)
CREATE INDEX idx_users_metadata_gin ON users USING GIN(metadata);

-- 部分索引
CREATE INDEX idx_active_users ON users(email) WHERE status = 'active';

-- 表达式索引
CREATE INDEX idx_users_lower_email ON users(LOWER(email));

-- 复合索引
CREATE INDEX idx_posts_user_published ON posts(user_id, published);

-- 解释查询计划
EXPLAIN ANALYZE SELECT * FROM users WHERE email = 'alice@example.com';
```

**查询优化**:
```sql
-- 使用EXPLAIN分析
EXPLAIN (ANALYZE, BUFFERS) 
SELECT u.username, COUNT(p.id)
FROM users u
LEFT JOIN posts p ON u.id = p.user_id
WHERE u.status = 'active'
GROUP BY u.id;

-- 避免SELECT *
SELECT id, username, email FROM users;

-- 使用LIMIT
SELECT * FROM posts ORDER BY created_at DESC LIMIT 100;

-- 批量插入
INSERT INTO users (username, email, password_hash)
VALUES 
    ('user1', 'user1@example.com', 'hash1'),
    ('user2', 'user2@example.com', 'hash2'),
    ('user3', 'user3@example.com', 'hash3');

-- 使用COPY导入大量数据
COPY users(username, email) FROM '/path/to/users.csv' CSV;

-- 更新统计信息
ANALYZE users;

-- 重建索引
REINDEX INDEX idx_users_email;
```

### 6. 事务

**ACID事务**:
```sql
-- 开始事务
BEGIN;

-- 转账示例
UPDATE accounts SET balance = balance - 100 WHERE id = 1;
UPDATE accounts SET balance = balance + 100 WHERE id = 2;

-- 提交
COMMIT;

-- 回滚
ROLLBACK;

-- 保存点
BEGIN;
UPDATE accounts SET balance = balance - 100 WHERE id = 1;
SAVEPOINT my_savepoint;
UPDATE accounts SET balance = balance + 100 WHERE id = 2;
-- 如果出错回滚到保存点
ROLLBACK TO my_savepoint;
COMMIT;

-- 隔离级别
BEGIN TRANSACTION ISOLATION LEVEL READ COMMITTED;
BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ;
BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE;
```

### 7. 存储过程

**函数**:
```sql
-- 基本函数
CREATE OR REPLACE FUNCTION get_user_count()
RETURNS INTEGER AS $$
BEGIN
    RETURN (SELECT COUNT(*) FROM users);
END;
$$ LANGUAGE plpgsql;

-- 带参数
CREATE OR REPLACE FUNCTION get_posts_by_user(p_user_id INTEGER)
RETURNS TABLE(id INTEGER, title VARCHAR, views INTEGER) AS $$
BEGIN
    RETURN QUERY
    SELECT p.id, p.title, p.views
    FROM posts p
    WHERE p.user_id = p_user_id;
END;
$$ LANGUAGE plpgsql;

-- 触发器函数
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器
CREATE TRIGGER users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();
```

## 最佳实践

### ✅ DO

1. **使用连接池**
```python
# Python psycopg2
import psycopg2
from psycopg2 import pool

connection_pool = psycopg2.pool.ThreadedConnectionPool(
    minconn=5,
    maxconn=20,
    host='localhost',
    database='mydb',
    user='user',
    password='password'
)

conn = connection_pool.getconn()
# 使用连接
connection_pool.putconn(conn)
```

2. **使用prepared statements**
```sql
PREPARE get_user_by_email (TEXT) AS
SELECT * FROM users WHERE email = $1;

EXECUTE get_user_by_email('alice@example.com');
```

3. **定期VACUUM**
```sql
-- 自动vacuum(默认启用)
ALTER TABLE users SET (autovacuum_enabled = true);

-- 手动vacuum
VACUUM ANALYZE users;
```

### ❌ DON'T

1. **不要在WHERE中使用函数**
```sql
-- ❌ 差(无法使用索引)
SELECT * FROM users WHERE LOWER(email) = 'alice@example.com';

-- ✅ 好(使用表达式索引)
CREATE INDEX idx_users_lower_email ON users(LOWER(email));
SELECT * FROM users WHERE LOWER(email) = 'alice@example.com';
```

2. **不要使用OR条件**
```sql
-- ❌ 差
SELECT * FROM users WHERE email = 'a@b.com' OR username = 'alice';

-- ✅ 好
SELECT * FROM users WHERE email = 'a@b.com'
UNION
SELECT * FROM users WHERE username = 'alice';
```

## 学习路径

### 初级 (1-2周)
1. SQL基础语法
2. 表设计和索引
3. 基本查询

### 中级 (2-3周)
1. 高级查询(连接、窗口函数)
2. JSON操作
3. 性能优化

### 高级 (2-4周)
1. 事务和并发控制
2. 存储过程
3. 分区和复制

### 专家级 (持续)
1. 查询优化器内部机制
2. 物理存储结构
3. 高可用架构

---

**知识ID**: `postgresql-complete`  
**领域**: development  
**类型**: standards  
**难度**: intermediate  
**质量分**: 94  
**维护者**: dba-team@superdev.com  
**最后更新**: 2026-03-28
