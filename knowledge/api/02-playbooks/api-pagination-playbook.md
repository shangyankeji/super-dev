---
id: api-pagination-playbook
title: API 分页实战手册
domain: api
category: 02-playbooks
difficulty: intermediate
tags: [api, pagination, cursor, offset, performance, database, query, limit, offset]
quality_score: 90
maintainer: platform-team@superdev.com
last_updated: 2026-06-14
---

# API 分页实战手册

## 何时用哪种分页

### Offset 分页（默认选择）
适用：中小数据集（< 10 万行），用户需要跳页。

```sql
-- PostgreSQL
SELECT * FROM products ORDER BY created_at DESC LIMIT 20 OFFSET 40;
```

问题：OFFSET 大时性能下降（数据库仍扫描跳过的行）。

### Cursor 分页（大数据集）
适用：时间线 / feed / 日志 / 事件流。

```sql
-- 用 WHERE 而非 OFFSET，利用索引
SELECT * FROM events
WHERE created_at < '2024-01-15T10:30:00Z'
ORDER BY created_at DESC LIMIT 50;
```

cursor 编码：`base64(last_item.created_at + ':' + last_item.id)`。

### Keyset 分页（排序稳定）
适用：按唯一字段排序的大列表。

```sql
SELECT * FROM orders WHERE id > 12345 ORDER BY id LIMIT 20;
```

## 分页响应格式

```json
{
  "data": [...],
  "pagination": {
    "page": 2,
    "limit": 20,
    "total": 1500,
    "totalPages": 75,
    "hasNext": true,
    "hasPrev": true
  }
}
```

## 常见陷阱

### 1. 忘记 total count
```python
# ❌ 只返回数据，客户端无法显示总页数
return {"data": users}

# ✅ 带 total
return {"data": users, "pagination": {"total": total, ...}}
```

### 2. 默认 limit 过大
```python
# ❌ 默认返回全部
@app.get("/users")
def list_users(limit=None):
    return db.query(User).limit(limit).all()

# ✅ 默认 + 上限
@app.get("/users")
def list_users(limit: int = Field(default=20, le=100)):
    return db.query(User).limit(limit).all()
```

### 3. 排序不稳定
```sql
-- ❌ 只按 created_at 排序，相同时间戳顺序不确定
SELECT * FROM products ORDER BY created_at LIMIT 20;

-- ✅ 加唯一字段做 tiebreaker
SELECT * FROM products ORDER BY created_at DESC, id DESC LIMIT 20;
```
