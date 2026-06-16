---
id: database-antipatterns
title: 数据库反模式指南
domain: development
category: 04-antipatterns
difficulty: intermediate
tags: [antipatterns, database, delete, development, index, pagination, problem, query]
quality_score: 70
last_updated: 2026-06-15
---
# 数据库反模式指南

> 适用范围：PostgreSQL / MySQL / MongoDB / Redis
> 约束级别：SHALL（必须在 Code Review 和 SQL Review 阶段拦截）

---

## 1. N+1 查询（N+1 Query Problem）

### 描述
先查询主表获得 N 条记录，然后对每条记录单独查询关联表，导致总共执行 N+1 次数据库查询。在列表页场景下，N 可能是数百甚至数千，直接导致接口响应时间线性增长。

### 错误示例
```python
# Django ORM -- 典型 N+1
def get_orders_with_user(request):
    orders = Order.objects.all()[:100]  # 1 次查询
    result = []
    for order in orders:
        # 每次循环触发 1 次查询，共 100 次
        result.append({
            "order_id": order.id,
            "user_name": order.user.name,       # SELECT * FROM users WHERE id = ?
            "user_email": order.user.email,
        })
    return JsonResponse(result, safe=False)
```

```javascript
// Sequelize -- 典型 N+1
async function getPostsWithComments() {
  const posts = await Post.findAll({ limit: 50 }); // 1 次查询
  for (const post of posts) {
    // 每次循环触发 1 次查询，共 50 次
    post.comments = await Comment.findAll({
      where: { postId: post.id },
    });
  }
  return posts;
}
```

### 正确示例
```python
# Django -- 使用 select_related / prefetch_related
def get_orders_with_user(request):
    orders = Order.objects.select_related("user").all()[:100]  # 1 次 JOIN 查询
    result = [
        {
            "order_id": order.id,
            "user_name": order.user.name,
            "user_email": order.user.email,
        }
        for order in orders
    ]
    return JsonResponse(result, safe=False)
```

```python
# SQLAlchemy -- 使用 joinedload
def get_orders_with_user(session: Session) -> list[Order]:
    return (
        session.query(Order)
        .options(joinedload(Order.user))
        .limit(100)
        .all()
    )
```

```javascript
// Sequelize -- 使用 eager loading
async function getPostsWithComments() {
  return Post.findAll({
    limit: 50,
    include: [{ model: Comment, as: "comments" }],
  });
}
```

### 检测方法
- Django Debug Toolbar 的 SQL 面板：单个请求查询数 > 10 即需警觉。
- `nplusone` 库（Django/SQLAlchemy）：自动检测 N+1 并抛出异常。
- 数据库慢查询日志：相同模板的 SQL 在短时间内执行多次。
- APM 工具（Datadog / New Relic）：查看单个请求的 DB 调用次数。

### 修复步骤
1. 开启 ORM 的 SQL 日志，统计单次请求的查询数量。
2. 对外键关联使用 `select_related`（一对一/多对一）或 `prefetch_related`（一对多/多对多）。
3. 对于非 ORM 场景，使用 `WHERE id IN (...)` 批量查询替代循环单查。
4. 添加集成测试断言查询次数（`assertNumQueries` in Django）。

### Agent Checklist
- [ ] 列表接口查询次数 <= 5
- [ ] 所有外键访问使用 `select_related` 或 `prefetch_related`
- [ ] 循环中无数据库查询
- [ ] 集成测试包含查询次数断言

---

## 2. 缺少索引（Missing Index）

### 描述
WHERE、JOIN、ORDER BY 使用的列未建立索引，导致全表扫描。在数据量从千级增长到百万级时，查询时间从毫秒级劣化到秒级。

### 错误示例
```sql
-- 表结构：无索引
CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER,
    status VARCHAR(20),
    created_at TIMESTAMP,
    total_amount DECIMAL(10, 2)
);

-- 以下查询全部触发全表扫描
SELECT * FROM orders WHERE user_id = 12345;
SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at DESC;
SELECT * FROM orders WHERE created_at BETWEEN '2024-01-01' AND '2024-01-31';
```

### 正确示例
```sql
-- 为高频查询模式建立索引
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_status_created ON orders(status, created_at DESC);
CREATE INDEX idx_orders_created_at ON orders(created_at);

-- 使用覆盖索引避免回表
CREATE INDEX idx_orders_user_summary ON orders(user_id)
    INCLUDE (status, total_amount, created_at);

-- 使用部分索引减少索引体积
CREATE INDEX idx_orders_pending ON orders(created_at DESC)
    WHERE status = 'pending';
```

### 检测方法
- `EXPLAIN ANALYZE` 输出中出现 `Seq Scan` 且 `rows` > 1000。
- PostgreSQL：`pg_stat_user_tables` 的 `seq_scan` 计数持续增长。
- MySQL：`SHOW INDEX FROM table_name` 检查是否覆盖高频查询列。
- 慢查询日志：执行时间 > 100ms 的 SQL 逐条分析执行计划。

### 修复步骤
1. 收集慢查询日志，列出 Top 20 慢 SQL。
2. 对每条慢 SQL 执行 `EXPLAIN ANALYZE`，识别全表扫描。
3. 根据查询模式创建合适的索引（单列 / 复合 / 部分 / 覆盖）。
4. 创建索引后重新执行 `EXPLAIN ANALYZE` 确认已使用索引。
5. 监控索引使用率，删除从未使用的冗余索引。

### Agent Checklist
- [ ] 所有 WHERE 条件列有索引（或复合索引的前缀）
- [ ] JOIN 的外键列有索引
- [ ] ORDER BY 列包含在索引中
- [ ] 无未使用的冗余索引
- [ ] 大表（> 100 万行）的高频查询使用覆盖索引

---

## 3. SELECT *（过度获取）

### 描述
查询时使用 `SELECT *` 获取所有列，即使只需要其中 2-3 列。导致网络传输量增大、内存占用增加、无法使用覆盖索引，且表结构变更时可能引入意外的列。

### 错误示例
```python
# 只需要用户名和邮箱，却获取了所有列（包括大文本、二进制字段）
def get_user_list():
    cursor.execute("SELECT * FROM users")  # 包含 avatar_blob、bio_text 等大字段
    return cursor.fetchall()

# ORM 中同样的问题
users = User.objects.all()  # 加载了所有字段
names = [u.name for u in users]  # 只用了 name
```

### 正确示例
```python
# 明确指定所需列
def get_user_list():
    cursor.execute("SELECT id, name, email FROM users")
    return cursor.fetchall()

# ORM 中使用 values / only
users = User.objects.values("id", "name", "email")

# SQLAlchemy 使用 load_only
users = session.query(User).options(load_only(User.id, User.name, User.email)).all()

# 对于大字段，使用 defer 延迟加载
users = User.objects.defer("avatar_blob", "bio_text").all()
```

### 检测方法
- SQL Review 中搜索 `SELECT *` 或 `SELECT table.*`。
- ORM 查询日志中查找未使用 `only()` / `values()` / `load_only()` 的查询。
- 使用 `sqlfluff` lint 工具自动检测 `SELECT *`。

### 修复步骤
1. 审查所有 `SELECT *` 查询，确定实际需要的列。
2. 替换为明确的列名列表。
3. 对包含 BLOB / TEXT 大字段的表，设置 ORM 默认 defer。
4. 在 CI 中加入 `sqlfluff` 检查，禁止 `SELECT *` 进入主分支。

### Agent Checklist
- [ ] 无 `SELECT *` 查询
- [ ] ORM 查询使用 `only()` / `values()` / `load_only()`
- [ ] 大字段使用 `defer()` 延迟加载
- [ ] CI 包含 SQL lint 规则

---

## 4. 过度范式化（Over-Normalization）

### 描述
将数据拆分到过多的表中以追求完美的范式化，导致简单的读取操作需要 JOIN 5-10 张表，查询复杂且性能低下。在读多写少的场景下，适度反范式化是合理的。

### 错误示例
```sql
-- 过度拆分：一个用户资料需要 JOIN 6 张表
SELECT u.id, un.first_name, un.last_name, ue.email,
       up.phone, ua.street, ua.city, uc.country_name
FROM users u
JOIN user_names un ON u.name_id = un.id
JOIN user_emails ue ON u.email_id = ue.id
JOIN user_phones up ON u.phone_id = up.id
JOIN user_addresses ua ON u.address_id = ua.id
JOIN countries uc ON ua.country_id = uc.id;

-- 甚至连状态都拆成了单独的表
SELECT os.status_name
FROM orders o
JOIN order_statuses os ON o.status_id = os.id
WHERE o.id = 123;
```

### 正确示例
```sql
-- 适度反范式化：将高频读取的字段内联
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    phone VARCHAR(20),
    street VARCHAR(200),
    city VARCHAR(100),
    country_code CHAR(2) NOT NULL  -- ISO 代码，不需要 JOIN 国家表
);

-- 使用枚举而非外键表
CREATE TYPE order_status AS ENUM ('pending', 'paid', 'shipped', 'delivered', 'cancelled');

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    status order_status NOT NULL DEFAULT 'pending'
);

-- 对于需要分析的场景，使用物化视图
CREATE MATERIALIZED VIEW user_order_summary AS
SELECT u.id, u.first_name, u.last_name, COUNT(o.id) AS order_count,
       SUM(o.total) AS total_spent
FROM users u LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.first_name, u.last_name;
```

### 检测方法
- 单个查询 JOIN 超过 3 张表。
- 存在只包含 `id` + `name` 两列的"字典表"且数据量 < 100 条。
- `EXPLAIN ANALYZE` 显示多层 Nested Loop Join 导致性能退化。

### 修复步骤
1. 分析查询日志，找出 JOIN 数量最多的 Top 10 查询。
2. 评估哪些 JOIN 表是"字典表"（数据量小、变更频率低）。
3. 将字典表的值内联为主表的枚举列或 VARCHAR 列。
4. 对高频聚合查询使用物化视图。
5. 使用数据库迁移脚本执行反范式化，确保数据一致性。

### Agent Checklist
- [ ] 单个查询 JOIN <= 3 张表
- [ ] 数据量 < 100 的字典表考虑用枚举替代
- [ ] 读多写少的场景允许适度冗余
- [ ] 聚合查询使用物化视图或缓存

---

## 5. 不用事务（Missing Transactions）

### 描述
涉及多表写入的业务操作未使用事务，导致中途失败时数据处于不一致的中间状态。例如扣款成功但订单创建失败，用户余额减少但没有对应订单。

### 错误示例
```python
def transfer_money(from_id, to_id, amount):
    # 无事务：如果第二步失败，钱已经从 from 账户扣除但未到 to 账户
    db.execute(
        "UPDATE accounts SET balance = balance - %s WHERE id = %s",
        (amount, from_id)
    )
    # 如果这里抛异常，钱就消失了
    db.execute(
        "UPDATE accounts SET balance = balance + %s WHERE id = %s",
        (amount, to_id)
    )
    db.execute(
        "INSERT INTO transfers (from_id, to_id, amount) VALUES (%s, %s, %s)",
        (from_id, to_id, amount)
    )
```

### 正确示例
```python
def transfer_money(from_id: int, to_id: int, amount: Decimal) -> Transfer:
    with db.transaction() as tx:
        # 加行锁防止并发问题
        from_account = tx.execute(
            "SELECT * FROM accounts WHERE id = %s FOR UPDATE", (from_id,)
        ).fetchone()

        if from_account["balance"] < amount:
            raise InsufficientBalanceError(from_id, amount)

        tx.execute(
            "UPDATE accounts SET balance = balance - %s WHERE id = %s",
            (amount, from_id),
        )
        tx.execute(
            "UPDATE accounts SET balance = balance + %s WHERE id = %s",
            (amount, to_id),
        )
        transfer = tx.execute(
            "INSERT INTO transfers (from_id, to_id, amount, status) "
            "VALUES (%s, %s, %s, 'completed') RETURNING *",
            (from_id, to_id, amount),
        ).fetchone()

        return Transfer(**transfer)
    # 事务自动 commit；异常时自动 rollback
```

```python
# Django ORM
from django.db import transaction

@transaction.atomic
def transfer_money(from_id: int, to_id: int, amount: Decimal) -> Transfer:
    from_account = Account.objects.select_for_update().get(id=from_id)
    to_account = Account.objects.select_for_update().get(id=to_id)

    if from_account.balance < amount:
        raise InsufficientBalanceError(from_id, amount)

    from_account.balance -= amount
    from_account.save()
    to_account.balance += amount
    to_account.save()

    return Transfer.objects.create(
        from_account=from_account, to_account=to_account, amount=amount
    )
```

### 检测方法
- 搜索代码中连续多条 `INSERT` / `UPDATE` / `DELETE` 且无事务包裹。
- ORM 中连续多个 `.save()` 调用不在 `transaction.atomic` 内。
- Code Review 中检查涉及资金、库存、状态流转的代码路径。

### 修复步骤
1. 梳理所有涉及多表写入的业务操作。
2. 为每个操作添加事务包裹（`BEGIN ... COMMIT / ROLLBACK`）。
3. 对需要防止并发写入的场景添加行锁（`SELECT ... FOR UPDATE`）。
4. 编写测试模拟中途失败场景，验证事务回滚正确。

### Agent Checklist
- [ ] 多表写入操作包裹在事务中
- [ ] 资金/库存操作使用 `SELECT ... FOR UPDATE` 行锁
- [ ] 事务异常时自动 rollback
- [ ] 有中途失败场景的回滚测试

---

## 6. 硬删除（Hard Delete）

### 描述
直接使用 `DELETE` 物理删除数据，导致无法审计、无法恢复误删数据、外键约束可能级联删除关联数据。在合规场景下（金融、医疗），硬删除可能违反法规要求。

### 错误示例
```python
def delete_user(user_id):
    # 物理删除：数据永久丢失，关联数据可能级联删除
    db.execute("DELETE FROM user_addresses WHERE user_id = %s", (user_id,))
    db.execute("DELETE FROM user_orders WHERE user_id = %s", (user_id,))
    db.execute("DELETE FROM users WHERE id = %s", (user_id,))

# ORM 中同样的问题
user = User.objects.get(id=user_id)
user.delete()  # 级联删除所有关联数据
```

### 正确示例
```python
# 软删除模型
class SoftDeleteMixin:
    deleted_at = Column(DateTime, nullable=True, index=True)
    deleted_by = Column(Integer, nullable=True)

    @hybrid_property
    def is_deleted(self):
        return self.deleted_at is not None

class User(Base, SoftDeleteMixin):
    __tablename__ = "users"
    id = Column(Integer, primary_key=True)
    name = Column(String(100))
    email = Column(String(255))

# 软删除操作
def soft_delete_user(user_id: int, operator_id: int) -> None:
    with db.transaction() as tx:
        tx.execute(
            "UPDATE users SET deleted_at = NOW(), deleted_by = %s WHERE id = %s",
            (operator_id, user_id),
        )
        # 记录审计日志
        tx.execute(
            "INSERT INTO audit_log (entity, entity_id, action, operator_id) "
            "VALUES ('user', %s, 'soft_delete', %s)",
            (user_id, operator_id),
        )

# 查询时自动过滤已删除数据
def get_active_users():
    return db.execute("SELECT * FROM users WHERE deleted_at IS NULL").fetchall()

# Django 软删除 Manager
class ActiveManager(models.Manager):
    def get_queryset(self):
        return super().get_queryset().filter(deleted_at__isnull=True)
```

### 检测方法
- 搜索代码中的 `DELETE FROM` 和 `.delete()` 调用。
- 检查表结构是否包含 `deleted_at` / `is_deleted` 列。
- 数据库审计日志中是否记录了删除操作。

### 修复步骤
1. 为需要软删除的表添加 `deleted_at`、`deleted_by` 列。
2. 创建软删除 Mixin / 基类，统一软删除逻辑。
3. 修改所有查询，添加 `WHERE deleted_at IS NULL` 条件（或使用自定义 Manager）。
4. 将 `DELETE` 语句改为 `UPDATE ... SET deleted_at = NOW()`。
5. 添加定期清理任务，对超过保留期的软删除数据进行物理删除。

### Agent Checklist
- [ ] 业务表使用软删除（`deleted_at` 列）
- [ ] 无直接 `DELETE FROM` 语句（除定期清理任务）
- [ ] 查询默认过滤已删除数据
- [ ] 删除操作记录审计日志

---

## 7. 无分页（Missing Pagination）

### 描述
查询接口不限制返回数量，一次性返回全部数据。在数据量增长后导致内存溢出、响应超时、网络带宽耗尽。

### 错误示例
```python
# 返回所有订单 -- 数据量增长后直接 OOM
@app.get("/orders")
def list_orders():
    orders = Order.objects.all()  # 可能有几百万条
    return {"orders": [serialize(o) for o in orders]}

# API 无分页参数
@app.get("/users")
def list_users(status: str = None):
    query = "SELECT * FROM users"
    if status:
        query += f" WHERE status = '{status}'"  # 还有 SQL 注入风险
    return db.execute(query).fetchall()
```

### 正确示例
```python
from fastapi import Query

@app.get("/orders")
def list_orders(
    page: int = Query(1, ge=1, description="页码"),
    page_size: int = Query(20, ge=1, le=100, description="每页数量"),
    status: str | None = Query(None, description="订单状态过滤"),
):
    query = Order.objects.all()
    if status:
        query = query.filter(status=status)

    total = query.count()
    offset = (page - 1) * page_size
    orders = query.order_by("-created_at")[offset : offset + page_size]

    return {
        "data": [serialize(o) for o in orders],
        "pagination": {
            "page": page,
            "page_size": page_size,
            "total": total,
            "total_pages": (total + page_size - 1) // page_size,
        },
    }

# 对于大数据量，使用游标分页（keyset pagination）
@app.get("/orders/cursor")
def list_orders_cursor(
    after: str | None = Query(None, description="上一页最后一条的游标"),
    limit: int = Query(20, ge=1, le=100),
):
    query = Order.objects.all().order_by("-created_at")
    if after:
        cursor_date = decode_cursor(after)
        query = query.filter(created_at__lt=cursor_date)
    orders = list(query[:limit + 1])

    has_next = len(orders) > limit
    orders = orders[:limit]
    next_cursor = encode_cursor(orders[-1].created_at) if has_next else None

    return {
        "data": [serialize(o) for o in orders],
        "next_cursor": next_cursor,
        "has_next": has_next,
    }
```

### 检测方法
- API 接口无 `page` / `limit` / `cursor` 参数。
- ORM 查询无 `LIMIT` 子句。
- 响应 JSON 中无分页元数据（`total`、`page`、`next_cursor`）。
- 负载测试：随数据量增长，响应时间线性增加。

### 修复步骤
1. 为所有列表接口添加分页参数（`page` + `page_size` 或 `cursor` + `limit`）。
2. 设置 `page_size` 上限（通常 100），防止客户端请求过大。
3. 返回分页元数据（总数、总页数、下一页游标）。
4. 对超过 10 万条数据的表，使用游标分页替代偏移量分页。
5. 添加集成测试验证分页逻辑正确性。

### Agent Checklist
- [ ] 所有列表接口包含分页参数
- [ ] `page_size` 有上限（<= 100）
- [ ] 响应包含分页元数据
- [ ] 大数据量场景使用游标分页
- [ ] ORM 查询包含 `LIMIT` 子句

---

## 全局 Agent Checklist

| 检查项 | 阈值 | 工具 |
|--------|------|------|
| 列表接口查询次数 | <= 5 | Django Debug Toolbar / APM |
| N+1 查询 | 0 个 | `nplusone` / SQL 日志 |
| 缺失索引 | 0 个 | `EXPLAIN ANALYZE` / `pg_stat` |
| `SELECT *` 使用 | 0 处 | `sqlfluff` / Code Review |
| 单查询 JOIN 数 | <= 3 | `EXPLAIN ANALYZE` |
| 无事务多表写入 | 0 处 | Code Review |
| 硬删除语句 | 0 条 | Code Review / `grep DELETE` |
| 无分页列表接口 | 0 个 | API 文档审查 |
