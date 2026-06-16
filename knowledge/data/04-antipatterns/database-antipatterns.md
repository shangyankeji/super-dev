---
id: database-antipatterns
title: 数据库反模式完全指南
domain: data
category: 04-antipatterns
difficulty: intermediate
tags: [antipatterns, data, database, delete, index, pagination, problem, query]
quality_score: 70
last_updated: 2026-06-15
---
# 数据库反模式完全指南

> 适用范围：PostgreSQL / MySQL / MongoDB / Redis
> 约束级别：SHALL（必须在 Code Review 和 SQL Review 阶段拦截）

---

## 1. 无索引查询（Missing Index）

### 描述
在高频查询的 WHERE / JOIN / ORDER BY 列上缺少索引，导致全表扫描。当表行数超过万级时，响应时间从毫秒级退化到秒级。

### 错误示例
```sql
-- 用户表 100 万行，email 列无索引
SELECT * FROM users WHERE email = 'alice@example.com';
-- Seq Scan on users  (cost=0.00..25432.00 rows=1 width=128)
-- Execution Time: 850ms

-- 订单表按状态过滤，status 无索引
SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at DESC;
-- Seq Scan on orders  (cost=0.00..48210.00 rows=15000 width=256)
```

```python
# SQLAlchemy -- 定义模型时忘记添加索引
class User(Base):
    __tablename__ = "users"
    id = Column(Integer, primary_key=True)
    email = Column(String(255))          # 缺少 index=True
    phone = Column(String(20))           # 缺少 index=True
    created_at = Column(DateTime)        # 缺少 index=True
```

### 正确示例
```sql
-- 为高频查询列创建索引
CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_orders_status_created ON orders (status, created_at DESC);

-- 使用 EXPLAIN ANALYZE 验证
EXPLAIN ANALYZE SELECT * FROM users WHERE email = 'alice@example.com';
-- Index Scan using idx_users_email on users  (cost=0.42..8.44 rows=1 width=128)
-- Execution Time: 0.05ms
```

```python
# SQLAlchemy -- 正确声明索引
class User(Base):
    __tablename__ = "users"
    id = Column(Integer, primary_key=True)
    email = Column(String(255), index=True, unique=True)
    phone = Column(String(20), index=True)
    created_at = Column(DateTime, index=True)

    __table_args__ = (
        Index("idx_users_email_phone", "email", "phone"),  # 复合索引
    )
```

### 检测方法
```sql
-- PostgreSQL: 查找缺少索引的表
SELECT schemaname, relname, seq_scan, seq_tup_read,
       idx_scan, idx_tup_fetch
FROM pg_stat_user_tables
WHERE seq_scan > 1000 AND idx_scan < 10
ORDER BY seq_tup_read DESC;

-- PostgreSQL: 查找未使用的索引（反向清理）
SELECT indexrelname, idx_scan
FROM pg_stat_user_indexes
WHERE idx_scan = 0 AND indexrelname NOT LIKE 'pg_%'
ORDER BY pg_relation_size(indexrelid) DESC;
```

---

## 2. SELECT *（全列查询）

### 描述
使用 `SELECT *` 返回所有列，浪费带宽、内存和 I/O。尤其在包含 TEXT / BLOB / JSONB 大字段的表中，性能损失显著。还会导致索引覆盖扫描失效。

### 错误示例
```sql
-- 产品表含 description (TEXT) 和 images (JSONB)，只需要名称和价格
SELECT * FROM products WHERE category_id = 5;
-- 返回 20 列 x 5000 行，含两个大字段，传输 15MB

-- 子查询中使用 SELECT *
SELECT * FROM (SELECT * FROM orders WHERE user_id = 100) sub
WHERE sub.status = 'completed';
```

```python
# Django -- 无意中加载所有字段
def product_list(request):
    products = Product.objects.filter(category_id=5)  # SELECT *
    return JsonResponse([
        {"name": p.name, "price": p.price}  # 只用了 2 个字段
        for p in products
    ], safe=False)
```

### 正确示例
```sql
-- 只查询需要的列
SELECT id, name, price, stock FROM products WHERE category_id = 5;

-- 覆盖索引生效
CREATE INDEX idx_products_category_covering
    ON products (category_id) INCLUDE (id, name, price, stock);
-- Index Only Scan，无需回表
```

```python
# Django -- 使用 values / only / defer
def product_list(request):
    products = Product.objects.filter(category_id=5).values("id", "name", "price")
    # 或者 .only("id", "name", "price")
    # 或者 .defer("description", "images")
    return JsonResponse(list(products), safe=False)
```

---

## 3. N+1 查询（N+1 Query Problem）

### 描述
先查主表获得 N 条记录，再对每条记录单独查关联表，共执行 N+1 次查询。列表页场景下 N 可达数百，接口响应时间线性增长。

### 错误示例
```python
# Django ORM -- 典型 N+1
def get_orders(request):
    orders = Order.objects.all()[:100]  # 1 次查询
    result = []
    for order in orders:
        result.append({
            "order_id": order.id,
            "user_name": order.user.name,       # 每次 1 次查询
            "items_count": order.items.count(),  # 每次 1 次查询
        })
    return result  # 共 201 次查询
```

```javascript
// Prisma -- 典型 N+1
async function getArticles() {
  const articles = await prisma.article.findMany({ take: 50 });
  for (const article of articles) {
    article.author = await prisma.user.findUnique({
      where: { id: article.authorId },
    });
    article.tags = await prisma.tag.findMany({
      where: { articleId: article.id },
    });
  }
  return articles; // 共 101 次查询
}
```

### 正确示例
```python
# Django -- select_related (FK/OneToOne) + prefetch_related (M2M/reverse FK)
def get_orders(request):
    orders = (
        Order.objects
        .select_related("user")
        .prefetch_related("items")
        .all()[:100]
    )  # 仅 2-3 次查询
    result = [
        {
            "order_id": o.id,
            "user_name": o.user.name,
            "items_count": len(o.items.all()),
        }
        for o in orders
    ]
    return result
```

```javascript
// Prisma -- 使用 include 一次性加载
async function getArticles() {
  const articles = await prisma.article.findMany({
    take: 50,
    include: {
      author: true,
      tags: true,
    },
  });
  return articles; // 仅 1 次查询 (JOIN)
}
```

---

## 4. 过度范式化（Over-Normalization）

### 描述
将数据拆分到过多的表中，导致简单的业务查询需要 5-10 个 JOIN，SQL 复杂度和执行时间剧增。在读多写少的场景中，适度冗余反而能大幅提升性能。

### 错误示例
```sql
-- 过度范式化：用户地址拆成 5 张表
SELECT u.name, s.name AS street, c.name AS city,
       p.name AS province, co.name AS country
FROM users u
JOIN addresses a ON u.id = a.user_id
JOIN streets s ON a.street_id = s.id
JOIN cities c ON s.city_id = c.id
JOIN provinces p ON c.province_id = p.id
JOIN countries co ON p.country_id = co.id
WHERE u.id = 100;
-- 5 次 JOIN，执行计划复杂，维护成本高
```

### 正确示例
```sql
-- 适度反范式化：将地址信息合并到一张表
CREATE TABLE user_addresses (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    street VARCHAR(255) NOT NULL,
    city VARCHAR(100) NOT NULL,
    province VARCHAR(100) NOT NULL,
    country VARCHAR(100) NOT NULL DEFAULT 'China',
    postal_code VARCHAR(20),
    is_default BOOLEAN DEFAULT FALSE,
    CONSTRAINT idx_user_default UNIQUE (user_id, is_default) WHERE is_default = TRUE
);

-- 单次查询即可获取完整地址
SELECT u.name, a.street, a.city, a.province, a.country
FROM users u
JOIN user_addresses a ON u.id = a.user_id AND a.is_default = TRUE
WHERE u.id = 100;
```

### 反范式化适用场景判定
| 场景 | 是否适合反范式化 | 原因 |
|------|:---:|------|
| 读写比 > 10:1 的查询 | YES | 减少 JOIN，提升读取性能 |
| 报表/统计场景 | YES | 预计算聚合值避免实时大量 JOIN |
| 频繁变更的主数据 | NO | 冗余数据需要同步更新，容易不一致 |
| 事务一致性要求极高 | NO | 范式化能保证数据完整性 |

---

## 5. 不使用事务（Missing Transaction）

### 描述
涉及多表写操作时不使用事务，一旦中途失败会导致数据不一致。典型场景：转账、下单扣库存、注册同时创建关联数据。

### 错误示例
```python
# 转账 -- 无事务保护
def transfer(from_id, to_id, amount):
    from_account = Account.objects.get(id=from_id)
    to_account = Account.objects.get(id=to_id)

    from_account.balance -= amount
    from_account.save()          # 扣款成功

    # 如果这里抛异常，钱已扣但未到账！
    to_account.balance += amount
    to_account.save()

    TransferLog.objects.create(
        from_account=from_account,
        to_account=to_account,
        amount=amount,
    )
```

```javascript
// 下单 -- 无事务保护
async function createOrder(userId, items) {
  const order = await Order.create({ userId, status: "pending" });
  for (const item of items) {
    await OrderItem.create({ orderId: order.id, ...item });
    // 如果这里失败，订单已创建但商品不全
    await Product.decrement("stock", {
      by: item.quantity,
      where: { id: item.productId },
    });
  }
}
```

### 正确示例
```python
# Django -- 使用 atomic 事务
from django.db import transaction

def transfer(from_id, to_id, amount):
    with transaction.atomic():
        from_account = Account.objects.select_for_update().get(id=from_id)
        to_account = Account.objects.select_for_update().get(id=to_id)

        if from_account.balance < amount:
            raise InsufficientFundsError()

        from_account.balance -= amount
        from_account.save()

        to_account.balance += amount
        to_account.save()

        TransferLog.objects.create(
            from_account=from_account,
            to_account=to_account,
            amount=amount,
        )
    # 离开 with 块时自动 COMMIT，异常时自动 ROLLBACK
```

```javascript
// Prisma -- 使用 $transaction
async function createOrder(userId, items) {
  return prisma.$transaction(async (tx) => {
    const order = await tx.order.create({
      data: { userId, status: "pending" },
    });
    for (const item of items) {
      await tx.orderItem.create({
        data: { orderId: order.id, ...item },
      });
      const product = await tx.product.update({
        where: { id: item.productId },
        data: { stock: { decrement: item.quantity } },
      });
      if (product.stock < 0) {
        throw new Error(`Insufficient stock: ${item.productId}`);
      }
    }
    return order;
  });
}
```

---

## 6. 硬删除（Hard Delete）

### 描述
直接 DELETE 数据，无法审计、无法恢复、可能违反合规要求。在有外键关联的场景下还会触发级联删除，造成大量数据丢失。

### 错误示例
```sql
-- 直接删除用户及关联数据
DELETE FROM users WHERE id = 100;
-- 如果有 ON DELETE CASCADE，订单/评论/收藏全部消失

-- 批量清理过期数据
DELETE FROM sessions WHERE expired_at < NOW();
-- 百万行删除可能锁表数分钟
```

```python
# Django -- 硬删除
def delete_user(request, user_id):
    user = User.objects.get(id=user_id)
    user.delete()  # 永久删除，无法恢复
    return JsonResponse({"status": "deleted"})
```

### 正确示例
```sql
-- 软删除方案
ALTER TABLE users ADD COLUMN deleted_at TIMESTAMP NULL DEFAULT NULL;
CREATE INDEX idx_users_active ON users (id) WHERE deleted_at IS NULL;

-- "删除" 操作
UPDATE users SET deleted_at = NOW() WHERE id = 100;

-- 查询时自动过滤已删除记录
SELECT * FROM users WHERE deleted_at IS NULL AND email = 'alice@example.com';
```

```python
# Django -- 软删除 Mixin
class SoftDeleteMixin(models.Model):
    deleted_at = models.DateTimeField(null=True, blank=True, db_index=True)

    class Meta:
        abstract = True

    def soft_delete(self):
        self.deleted_at = timezone.now()
        self.save(update_fields=["deleted_at"])

    def restore(self):
        self.deleted_at = None
        self.save(update_fields=["deleted_at"])

class SoftDeleteManager(models.Manager):
    def get_queryset(self):
        return super().get_queryset().filter(deleted_at__isnull=True)

class User(SoftDeleteMixin):
    objects = SoftDeleteManager()        # 默认排除已删除
    all_objects = models.Manager()       # 包含已删除（管理员用）
    email = models.EmailField(unique=True)
    name = models.CharField(max_length=100)
```

### 大批量删除的正确做法
```sql
-- 分批删除，避免长时间锁表
DO $$
DECLARE
    batch_size INT := 5000;
    deleted_count INT;
BEGIN
    LOOP
        DELETE FROM sessions
        WHERE id IN (
            SELECT id FROM sessions
            WHERE expired_at < NOW() - INTERVAL '30 days'
            LIMIT batch_size
        );
        GET DIAGNOSTICS deleted_count = ROW_COUNT;
        EXIT WHEN deleted_count = 0;
        PERFORM pg_sleep(0.1);  -- 释放锁，让其他查询通过
    END LOOP;
END $$;
```

---

## 7. 无分页查询（Missing Pagination）

### 描述
一次性返回全部记录，当数据量增长到万级以上时，接口响应缓慢、内存溢出、前端渲染卡死。

### 错误示例
```python
# 返回所有产品 -- 数据量增长后必崩
def product_list(request):
    products = Product.objects.all()  # 可能返回 10 万条
    return JsonResponse(
        [{"id": p.id, "name": p.name} for p in products],
        safe=False,
    )
```

```sql
-- 无限制查询
SELECT * FROM logs ORDER BY created_at DESC;
-- 日志表 500 万行，直接打满内存
```

### 正确示例
```python
# Offset 分页（适合页数不多的场景）
from django.core.paginator import Paginator

def product_list(request):
    page_num = int(request.GET.get("page", 1))
    page_size = min(int(request.GET.get("size", 20)), 100)  # 上限 100

    products = Product.objects.order_by("-created_at")
    paginator = Paginator(products, page_size)
    page = paginator.get_page(page_num)

    return JsonResponse({
        "data": [{"id": p.id, "name": p.name} for p in page],
        "total": paginator.count,
        "page": page.number,
        "pages": paginator.num_pages,
    })
```

```python
# 游标分页（适合深翻页 / 无限滚动）
def product_list_cursor(request):
    cursor = request.GET.get("cursor")  # 上一页最后一条的 id
    page_size = min(int(request.GET.get("size", 20)), 100)

    qs = Product.objects.order_by("-id")
    if cursor:
        qs = qs.filter(id__lt=cursor)
    products = list(qs[:page_size + 1])

    has_next = len(products) > page_size
    products = products[:page_size]

    return JsonResponse({
        "data": [{"id": p.id, "name": p.name} for p in products],
        "next_cursor": products[-1].id if has_next else None,
    })
```

```sql
-- Keyset 分页（数据库层面，性能稳定）
SELECT id, name, price
FROM products
WHERE id < 10050  -- 上一页最后一条 ID
ORDER BY id DESC
LIMIT 20;
-- 无论翻到第几页，执行时间恒定 ~1ms
```

---

## 8. 不使用连接池（Missing Connection Pool）

### 描述
每次请求创建新的数据库连接，建立 TCP + TLS + 认证的开销约 50-100ms。在并发场景下，连接数暴涨可能打满数据库的 `max_connections`（PostgreSQL 默认 100），导致全站不可用。

### 错误示例
```python
# 每次请求创建新连接
import psycopg2

def get_user(user_id):
    conn = psycopg2.connect(
        host="localhost", dbname="myapp",
        user="app", password="secret"
    )
    cursor = conn.cursor()
    cursor.execute("SELECT * FROM users WHERE id = %s", (user_id,))
    user = cursor.fetchone()
    conn.close()  # 关闭后下次又要重建
    return user
```

```javascript
// Node.js -- 每次创建新连接
const { Client } = require("pg");

async function getUser(userId) {
  const client = new Client({ connectionString: DATABASE_URL });
  await client.connect();   // 50-100ms 开销
  const res = await client.query("SELECT * FROM users WHERE id = $1", [userId]);
  await client.end();
  return res.rows[0];
}
```

### 正确示例
```python
# psycopg2 连接池
from psycopg2 import pool

# 应用启动时创建连接池（单例）
db_pool = pool.ThreadedConnectionPool(
    minconn=5,
    maxconn=20,
    host="localhost",
    dbname="myapp",
    user="app",
    password="secret",
)

def get_user(user_id):
    conn = db_pool.getconn()
    try:
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM users WHERE id = %s", (user_id,))
        return cursor.fetchone()
    finally:
        db_pool.putconn(conn)  # 归还连接，不关闭
```

```javascript
// Node.js -- 使用连接池
const { Pool } = require("pg");

const pool = new Pool({
  connectionString: DATABASE_URL,
  max: 20,           // 最大连接数
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 5000,
});

async function getUser(userId) {
  const res = await pool.query("SELECT * FROM users WHERE id = $1", [userId]);
  return res.rows[0];  // 自动从池中获取/归还连接
}
```

### 连接池参数调优指南
| 参数 | 推荐值 | 说明 |
|------|--------|------|
| `max_connections` | CPU 核数 x 2 + 磁盘数 | PostgreSQL 官方建议 |
| `pool_size` | 5-20 | 应用侧每个进程的池大小 |
| `max_overflow` | 10 | 突发流量时允许的额外连接 |
| `pool_recycle` | 3600 | 连接最大存活时间（秒），防止连接泄漏 |
| `pool_pre_ping` | True | 使用前检查连接是否存活 |

---

## 9. 未优化的 JOIN（Inefficient JOIN）

### 描述
在大表上执行笛卡尔积 JOIN、缺少 JOIN 条件、JOIN 列无索引、或在 JOIN 结果上再做全表排序，导致查询时间从毫秒级膨胀到分钟级。

### 错误示例
```sql
-- 缺少 JOIN 条件，产生笛卡尔积
SELECT u.name, o.total
FROM users u, orders o;
-- 1万用户 x 10万订单 = 10亿行结果

-- JOIN 列无索引
SELECT u.name, o.total, o.created_at
FROM users u
JOIN orders o ON o.user_id = u.id
WHERE o.created_at > '2024-01-01'
ORDER BY o.total DESC;
-- orders.user_id 无索引 -> Nested Loop + Seq Scan

-- 多表 JOIN 顺序不当
SELECT *
FROM order_items oi          -- 500 万行
JOIN products p ON oi.product_id = p.id        -- 10 万行
JOIN orders o ON oi.order_id = o.id            -- 100 万行
JOIN users u ON o.user_id = u.id               -- 10 万行
WHERE u.country = 'CN'
  AND p.category = 'electronics';
-- 应该先过滤再 JOIN
```

### 正确示例
```sql
-- 确保 JOIN 列有索引
CREATE INDEX idx_orders_user_id ON orders (user_id);
CREATE INDEX idx_orders_created_at ON orders (created_at);
CREATE INDEX idx_order_items_order_id ON order_items (order_id);
CREATE INDEX idx_order_items_product_id ON order_items (product_id);

-- 使用子查询先过滤，减少 JOIN 的数据量
SELECT u.name, o.total, o.created_at
FROM users u
JOIN (
    SELECT user_id, total, created_at
    FROM orders
    WHERE created_at > '2024-01-01'
    ORDER BY total DESC
    LIMIT 100
) o ON o.user_id = u.id;

-- 使用 CTE 拆分复杂查询
WITH target_users AS (
    SELECT id, name FROM users WHERE country = 'CN'
),
target_products AS (
    SELECT id, name FROM products WHERE category = 'electronics'
)
SELECT tu.name AS user_name, tp.name AS product_name,
       oi.quantity, oi.price
FROM order_items oi
JOIN target_products tp ON oi.product_id = tp.id
JOIN orders o ON oi.order_id = o.id
JOIN target_users tu ON o.user_id = tu.id;
```

---

## 10. 缺少备份策略（Missing Backup）

### 描述
没有定期备份、没有验证备份可恢复、备份与主库在同一台机器上。一旦发生硬件故障、误操作或勒索攻击，数据永久丢失。

### 错误示例
```bash
# "我们有备份" -- 备份在同一台机器
pg_dump mydb > /var/lib/postgresql/backup.sql
# 磁盘故障 -> 主库和备份同时丢失

# 从不测试恢复
# 3 年前设置的 cron 备份任务，从未验证过
# 某天需要恢复时发现：备份文件损坏 / 格式不兼容 / 缺少依赖
```

### 正确示例
```bash
#!/bin/bash
# backup.sh -- 生产级备份脚本

set -euo pipefail

DB_NAME="myapp_prod"
BACKUP_DIR="/mnt/nfs/backups/postgres"
S3_BUCKET="s3://myapp-backups/postgres"
RETENTION_DAYS=30
DATE=$(date +%Y%m%d_%H%M%S)

# 1. 创建压缩备份
pg_dump -Fc -Z 9 "$DB_NAME" > "${BACKUP_DIR}/${DB_NAME}_${DATE}.dump"

# 2. 上传到异地存储
aws s3 cp "${BACKUP_DIR}/${DB_NAME}_${DATE}.dump" \
    "${S3_BUCKET}/${DB_NAME}_${DATE}.dump" \
    --storage-class STANDARD_IA

# 3. 验证备份可恢复
pg_restore --list "${BACKUP_DIR}/${DB_NAME}_${DATE}.dump" > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo "ALERT: Backup verification failed!" | \
        mail -s "Backup Failed: ${DB_NAME}" ops@example.com
    exit 1
fi

# 4. 清理过期本地备份
find "$BACKUP_DIR" -name "*.dump" -mtime +${RETENTION_DAYS} -delete

echo "Backup completed: ${DB_NAME}_${DATE}.dump"
```

```yaml
# PostgreSQL 持续归档 (WAL + PITR)
# postgresql.conf
archive_mode: "on"
archive_command: 'aws s3 cp %p s3://myapp-wal-archive/%f'
wal_level: replica

# 恢复到指定时间点
# recovery.conf
restore_command = 'aws s3 cp s3://myapp-wal-archive/%f %p'
recovery_target_time = '2024-06-15 14:30:00'
recovery_target_action = 'promote'
```

### 备份策略矩阵
| 备份类型 | 频率 | RPO | 存储位置 | 用途 |
|---------|------|-----|---------|------|
| 逻辑备份 (pg_dump) | 每日 | 24h | 异地 S3 | 完整恢复 / 迁移 |
| WAL 归档 (PITR) | 持续 | ~5min | 异地 S3 | 时间点恢复 |
| 物理备份 (pg_basebackup) | 每周 | 7d | 异地存储 | 快速全量恢复 |
| 流复制 (Streaming Replica) | 实时 | ~0s | 异地机房 | 高可用故障转移 |

---

## 11. 明文存储敏感数据（Plaintext Sensitive Data）

### 描述
密码、身份证号、银行卡号等敏感数据以明文存储在数据库中。一旦数据库被拖库或备份泄漏，所有用户信息直接暴露。违反 GDPR / 等保 / PCI-DSS 等合规要求。

### 错误示例
```sql
-- 密码明文存储
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL,  -- 'P@ssw0rd123' 明文！
    id_card VARCHAR(18),             -- '310101199001011234' 明文！
    bank_card VARCHAR(19)            -- '6222021234567890123' 明文！
);

INSERT INTO users (email, password, id_card)
VALUES ('alice@example.com', 'P@ssw0rd123', '310101199001011234');
```

```python
# 应用层 -- 明文比对密码
def login(email, password):
    user = User.objects.get(email=email)
    if user.password == password:  # 直接比对明文
        return create_token(user)
    raise AuthError("Invalid credentials")
```

### 正确示例
```python
# 密码 -- 使用 bcrypt 哈希
import bcrypt

def register(email, password):
    salt = bcrypt.gensalt(rounds=12)
    hashed = bcrypt.hashpw(password.encode(), salt)
    User.objects.create(
        email=email,
        password_hash=hashed.decode(),  # '$2b$12$...' 存储哈希值
    )

def login(email, password):
    user = User.objects.get(email=email)
    if bcrypt.checkpw(password.encode(), user.password_hash.encode()):
        return create_token(user)
    raise AuthError("Invalid credentials")
```

```python
# 敏感字段 -- 使用 AES 加密
from cryptography.fernet import Fernet

# 密钥从 KMS / Vault 获取，不硬编码
ENCRYPTION_KEY = get_key_from_vault("user-data-key")
cipher = Fernet(ENCRYPTION_KEY)

def save_id_card(user_id, id_card):
    encrypted = cipher.encrypt(id_card.encode()).decode()
    UserSensitive.objects.update_or_create(
        user_id=user_id,
        defaults={"id_card_encrypted": encrypted},
    )

def get_id_card(user_id):
    record = UserSensitive.objects.get(user_id=user_id)
    return cipher.decrypt(record.id_card_encrypted.encode()).decode()
```

```sql
-- PostgreSQL -- 使用 pgcrypto 扩展
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- 密码哈希
INSERT INTO users (email, password_hash)
VALUES ('alice@example.com', crypt('P@ssw0rd123', gen_salt('bf', 12)));

-- 验证密码
SELECT id FROM users
WHERE email = 'alice@example.com'
  AND password_hash = crypt('P@ssw0rd123', password_hash);

-- 字段加密
INSERT INTO user_sensitive (user_id, id_card_encrypted)
VALUES (1, pgp_sym_encrypt('310101199001011234', 'encryption-key'));

-- 解密
SELECT pgp_sym_decrypt(id_card_encrypted::bytea, 'encryption-key')
FROM user_sensitive WHERE user_id = 1;
```

---

## 反模式速查矩阵

| # | 反模式 | 风险等级 | 检测时机 | 检测工具 |
|---|--------|:-------:|---------|---------|
| 1 | 无索引查询 | HIGH | CI/CD + Review | pg_stat_user_tables / EXPLAIN |
| 2 | SELECT * | MEDIUM | Lint + Review | sqlfluff / eslint-plugin-sql |
| 3 | N+1 查询 | HIGH | APM + Review | django-debug-toolbar / Prisma logging |
| 4 | 过度范式化 | MEDIUM | 架构评审 | ER 图审查 |
| 5 | 不用事务 | CRITICAL | Review | 静态分析 + 代码规范 |
| 6 | 硬删除 | HIGH | Review | 模型审查 |
| 7 | 无分页 | HIGH | API Review | API 规范检查 |
| 8 | 不用连接池 | HIGH | 架构评审 | 连接数监控 |
| 9 | 未优化 JOIN | MEDIUM | Slow Query Log | EXPLAIN ANALYZE |
| 10 | 缺少备份 | CRITICAL | 运维审计 | 备份监控 |
| 11 | 明文存储 | CRITICAL | 安全审计 | 数据分类扫描 |

---

## Agent Checklist

- [ ] 所有 SQL 查询的 WHERE / JOIN / ORDER BY 列已创建索引
- [ ] 代码中无 `SELECT *`，所有查询只返回必要字段
- [ ] ORM 查询已使用 eager loading 消除 N+1 问题
- [ ] 范式化程度适当，读多写少的场景已考虑反范式化
- [ ] 所有多表写操作包裹在事务中
- [ ] 业务数据使用软删除，硬删除有审批流程
- [ ] 所有列表接口实现分页，单页上限不超过 100
- [ ] 使用数据库连接池，连接参数已调优
- [ ] 复杂 JOIN 已通过 EXPLAIN ANALYZE 验证执行计划
- [ ] 备份策略覆盖逻辑备份 + WAL 归档，备份已验证可恢复
- [ ] 密码使用 bcrypt/argon2 哈希，敏感字段使用 AES 加密
- [ ] 加密密钥从 KMS / Vault 获取，不硬编码在代码或配置中
