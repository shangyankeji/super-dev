---
id: database-migration-playbook
title: 数据库迁移作战手册 (Database Migration Playbook)
domain: development
category: 02-playbooks
difficulty: intermediate
tags: [agent, checklist, database, development, migration, playbook, 前置条件, 回滚方案]
quality_score: 70
last_updated: 2026-06-15
---
# 数据库迁移作战手册 (Database Migration Playbook)

## 概述

数据库迁移是对数据库 Schema 或数据进行变更的结构化操作流程。由于数据库变更通常不可逆且直接影响线上业务，本手册定义了严格的前向/后向兼容策略、安全执行步骤、蓝绿部署方案和回滚机制，确保每次迁移可控、可回滚、零停机。

## 前置条件

### 必须满足

- [ ] 数据库有最新的完整备份（已验证可恢复）
- [ ] 已在与生产环境一致的测试环境验证迁移脚本
- [ ] 已评估迁移对表锁、写入性能的影响
- [ ] 已评估大表变更的执行时间（数据量 > 100 万行需特别关注）
- [ ] 已准备回滚脚本并验证
- [ ] 已获得 DBA 或技术负责人审批

### 建议满足

- [ ] 有数据库变更审计日志
- [ ] 有自动化迁移工具（Alembic、Flyway、Django Migrations、Prisma Migrate）
- [ ] 有数据库只读副本可用于验证
- [ ] 有慢查询监控

---

## 步骤一：迁移设计

### 1.1 前向兼容原则

前向兼容：新 Schema 能被旧代码正常使用。这是零停机部署的核心要求。

```
安全的变更（前向兼容）：
✓ 新增列（带默认值或允许 NULL）
✓ 新增表
✓ 新增索引（CONCURRENTLY）
✓ 放宽约束（如 NOT NULL -> NULL）

危险的变更（需要特殊处理）：
⚠ 重命名列/表
⚠ 修改列类型
⚠ 添加 NOT NULL 约束
⚠ 删除列/表

禁止直接执行的变更：
✗ 删除正在使用的列
✗ 重命名正在使用的列
✗ 修改正在使用列的类型（不兼容）
```

### 1.2 安全重命名列的多步迁移

```sql
-- 场景：将 orders.user_name 重命名为 orders.customer_name

-- 迁移 1：添加新列
ALTER TABLE orders ADD COLUMN customer_name VARCHAR(100);

-- 迁移 2：双写（应用层同时写两列）
-- 代码变更：INSERT/UPDATE 同时写 user_name 和 customer_name

-- 迁移 3：数据回填
UPDATE orders SET customer_name = user_name WHERE customer_name IS NULL;

-- 迁移 4：切换读取（应用层从 customer_name 读取）
-- 代码变更：SELECT 改为读 customer_name

-- 迁移 5：停止写旧列
-- 代码变更：INSERT/UPDATE 只写 customer_name

-- 迁移 6：删除旧列（确认无代码引用后）
ALTER TABLE orders DROP COLUMN user_name;
```

### 1.3 安全添加 NOT NULL 约束

```sql
-- 场景：为 orders.status 添加 NOT NULL 约束

-- 迁移 1：设置默认值并回填
ALTER TABLE orders ALTER COLUMN status SET DEFAULT 'draft';
UPDATE orders SET status = 'draft' WHERE status IS NULL;

-- 迁移 2：添加 CHECK 约束（不阻塞写入）
ALTER TABLE orders ADD CONSTRAINT orders_status_not_null
  CHECK (status IS NOT NULL) NOT VALID;

-- 迁移 3：验证约束（扫描全表但不阻塞写入）
ALTER TABLE orders VALIDATE CONSTRAINT orders_status_not_null;

-- 迁移 4：转换为正式 NOT NULL（可选，PostgreSQL 12+）
ALTER TABLE orders ALTER COLUMN status SET NOT NULL;
ALTER TABLE orders DROP CONSTRAINT orders_status_not_null;
```

### 1.4 大表安全加索引

```sql
-- PostgreSQL: 使用 CONCURRENTLY 避免锁表
CREATE INDEX CONCURRENTLY idx_orders_user_created
ON orders(user_id, created_at DESC);

-- 注意事项：
-- 1. CONCURRENTLY 不能在事务内执行
-- 2. 如果中途失败，会留下 INVALID 索引，需要手动清理：
--    DROP INDEX CONCURRENTLY idx_orders_user_created;
-- 3. 会增加临时磁盘空间使用

-- MySQL: 使用 ALGORITHM=INPLACE 或 pt-online-schema-change
ALTER TABLE orders ADD INDEX idx_user_created (user_id, created_at),
  ALGORITHM=INPLACE, LOCK=NONE;

-- 或使用 pt-online-schema-change（大表推荐）
pt-online-schema-change \
  --alter "ADD INDEX idx_user_created (user_id, created_at)" \
  --execute \
  --chunk-size=1000 \
  --max-lag=1 \
  D=mydb,t=orders
```

---

## 步骤二：迁移脚本编写

### 2.1 Alembic (Python/SQLAlchemy)

```python
"""add customer_name column to orders

Revision ID: a1b2c3d4e5f6
Revises: 9z8y7x6w5v4u
Create Date: 2024-01-15 10:30:00.000000
"""
from alembic import op
import sqlalchemy as sa

revision = 'a1b2c3d4e5f6'
down_revision = '9z8y7x6w5v4u'

def upgrade():
    op.add_column('orders',
        sa.Column('customer_name', sa.String(100), nullable=True, comment='客户名称')
    )
    # 回填数据
    op.execute("""
        UPDATE orders SET customer_name = user_name
        WHERE customer_name IS NULL
    """)

def downgrade():
    op.drop_column('orders', 'customer_name')
```

### 2.2 Flyway (Java)

```sql
-- V20240115_1030__add_customer_name_to_orders.sql

-- 前向迁移
ALTER TABLE orders ADD COLUMN customer_name VARCHAR(100);

-- 回填
UPDATE orders SET customer_name = user_name WHERE customer_name IS NULL;

-- 添加注释
COMMENT ON COLUMN orders.customer_name IS '客户名称';
```

```sql
-- U20240115_1030__add_customer_name_to_orders.sql (回滚脚本)

ALTER TABLE orders DROP COLUMN IF EXISTS customer_name;
```

### 2.3 Prisma Migrate (Node.js)

```prisma
// schema.prisma - 变更后
model Order {
  id           Int      @id @default(autoincrement())
  userId       Int      @map("user_id")
  customerName String?  @map("customer_name") @db.VarChar(100)
  status       String   @default("draft") @db.VarChar(20)
  createdAt    DateTime @default(now()) @map("created_at")

  @@map("orders")
  @@index([userId, createdAt(sort: Desc)])
}
```

```bash
# 生成迁移
npx prisma migrate dev --name add_customer_name

# 检查生成的 SQL
cat prisma/migrations/20240115103000_add_customer_name/migration.sql

# 生产环境执行
npx prisma migrate deploy
```

### 2.4 迁移脚本审查清单

```markdown
每个迁移脚本必须检查：

- [ ] 有对应的 downgrade/回滚脚本
- [ ] 不包含不可逆操作（或已标注风险并有备份方案）
- [ ] 大表操作有执行时间预估
- [ ] 索引变更使用 CONCURRENTLY 或等价方式
- [ ] 不在事务中执行耗时操作（避免长事务锁表）
- [ ] 有数据回填逻辑时分批执行
- [ ] 不包含硬编码的业务数据
```

---

## 步骤三：迁移执行

### 3.1 预发布环境验证

```bash
# 在预发布环境执行迁移
# 1. 获取当前状态
alembic current

# 2. 检查待执行的迁移
alembic history --verbose

# 3. 模拟执行（只输出 SQL，不实际执行）
alembic upgrade head --sql > migration_preview.sql
cat migration_preview.sql

# 4. 实际执行
alembic upgrade head

# 5. 验证
alembic current
psql -c "\d orders"  # 检查表结构
```

### 3.2 生产环境执行

```bash
#!/bin/bash
# migrate_production.sh

set -euo pipefail

DB_HOST="production-db.example.com"
DB_NAME="production"
DB_USER="migrate_user"
BACKUP_DIR="/backups/$(date +%Y%m%d_%H%M%S)"

echo "=== 生产环境数据库迁移 ==="
echo "时间: $(date)"
echo "目标: $DB_HOST/$DB_NAME"

# 步骤 1: 备份
echo "[1/6] 创建备份..."
mkdir -p "$BACKUP_DIR"
pg_dump -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" \
  --format=custom \
  --compress=9 \
  > "$BACKUP_DIR/pre_migration.dump"
echo "备份完成: $BACKUP_DIR/pre_migration.dump"
echo "备份大小: $(du -h "$BACKUP_DIR/pre_migration.dump" | cut -f1)"

# 步骤 2: 检查当前状态
echo "[2/6] 检查当前迁移状态..."
alembic current

# 步骤 3: 检查活跃连接
echo "[3/6] 检查活跃连接..."
ACTIVE_CONNS=$(psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -t -c \
  "SELECT count(*) FROM pg_stat_activity WHERE state = 'active' AND pid != pg_backend_pid();")
echo "活跃连接数: $ACTIVE_CONNS"

# 步骤 4: 执行迁移
echo "[4/6] 执行迁移..."
START_TIME=$(date +%s)
alembic upgrade head
END_TIME=$(date +%s)
echo "迁移耗时: $((END_TIME - START_TIME)) 秒"

# 步骤 5: 验证
echo "[5/6] 验证迁移..."
alembic current
psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -c "\d orders" | head -20

# 步骤 6: 健康检查
echo "[6/6] 应用健康检查..."
for i in {1..5}; do
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://api.example.com/health)
    echo "健康检查 $i: $STATUS"
    sleep 5
done

echo "=== 迁移完成 ==="
```

### 3.3 大数据量回填

```python
# 分批回填数据，避免长事务和锁表

import time
from sqlalchemy import text

def backfill_customer_name(engine, batch_size=5000):
    """分批回填 customer_name"""
    total_updated = 0
    start_time = time.time()

    while True:
        with engine.begin() as conn:
            result = conn.execute(text("""
                UPDATE orders
                SET customer_name = user_name
                WHERE id IN (
                    SELECT id FROM orders
                    WHERE customer_name IS NULL
                    LIMIT :batch_size
                )
            """), {"batch_size": batch_size})

            rows_affected = result.rowcount
            total_updated += rows_affected

            if rows_affected == 0:
                break

        # 打印进度
        elapsed = time.time() - start_time
        print(f"已更新 {total_updated} 行, 耗时 {elapsed:.1f}s")

        # 控制速度，避免对线上造成压力
        time.sleep(0.5)

    print(f"回填完成: 共更新 {total_updated} 行, 总耗时 {time.time() - start_time:.1f}s")
```

---

## 步骤四：蓝绿部署中的迁移

### 4.1 蓝绿部署迁移策略

```
蓝绿部署中数据库迁移的核心约束：
蓝（旧版本）和绿（新版本）共享同一个数据库，
因此 Schema 变更必须同时兼容两个版本。

部署顺序：
1. 执行前向兼容的数据库迁移
2. 部署绿环境（新代码）
3. 验证绿环境
4. 切换流量到绿环境
5. 观察稳定后，执行清理迁移（删除旧列等）
```

```
时间线示例（重命名列）：

T1: [蓝: 读写 user_name] [DB: user_name]
    执行迁移: ADD customer_name

T2: [蓝: 读写 user_name] [DB: user_name + customer_name]
    部署绿: 双写 user_name + customer_name，读 customer_name

T3: [绿: 双写, 读 customer_name] [DB: user_name + customer_name]
    回填 customer_name

T4: [绿: 双写, 读 customer_name] [DB: user_name + customer_name, 数据已同步]
    切换流量到绿

T5: [绿: 只写 customer_name] [DB: user_name + customer_name]
    确认稳定后

T6: [绿: 只写 customer_name] [DB: customer_name]
    执行清理迁移: DROP user_name
```

### 4.2 双写中间层

```python
class OrderRepository:
    """支持蓝绿部署的双写 Repository"""

    def __init__(self, migration_phase: str = "dual_write"):
        # 配置迁移阶段：
        # "old_only"    - 只使用旧列
        # "dual_write"  - 双写，读新列
        # "new_only"    - 只使用新列
        self.phase = migration_phase

    def create_order(self, data: dict):
        if self.phase == "old_only":
            return self._insert(user_name=data["name"])
        elif self.phase == "dual_write":
            return self._insert(
                user_name=data["name"],
                customer_name=data["name"]
            )
        else:  # new_only
            return self._insert(customer_name=data["name"])

    def get_order_name(self, order):
        if self.phase == "old_only":
            return order.user_name
        else:
            return order.customer_name or order.user_name
```

---

## 步骤五：验证

### 5.1 迁移后检查

```bash
#!/bin/bash
# verify_migration.sh

echo "=== 迁移后验证 ==="

# 1. 表结构验证
echo "[1] 表结构检查"
psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -c "
  SELECT column_name, data_type, is_nullable, column_default
  FROM information_schema.columns
  WHERE table_name = 'orders'
  ORDER BY ordinal_position;
"

# 2. 索引验证
echo "[2] 索引检查"
psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -c "
  SELECT indexname, indexdef
  FROM pg_indexes
  WHERE tablename = 'orders';
"

# 3. 约束验证
echo "[3] 约束检查"
psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -c "
  SELECT conname, contype, pg_get_constraintdef(oid)
  FROM pg_constraint
  WHERE conrelid = 'orders'::regclass;
"

# 4. 数据一致性验证
echo "[4] 数据一致性"
psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -c "
  SELECT
    COUNT(*) as total,
    COUNT(customer_name) as has_customer_name,
    COUNT(user_name) as has_user_name,
    COUNT(CASE WHEN customer_name != user_name THEN 1 END) as mismatched
  FROM orders;
"

# 5. 查询性能验证
echo "[5] 查询性能"
psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -c "
  EXPLAIN (ANALYZE, BUFFERS)
  SELECT id, customer_name, status
  FROM orders
  WHERE user_id = 12345
  ORDER BY created_at DESC
  LIMIT 20;
"

echo "=== 验证完成 ==="
```

### 5.2 应用层验证

```bash
# API 功能验证
echo "创建订单测试"
curl -s -X POST http://api.example.com/api/v1/orders \
  -H "Content-Type: application/json" \
  -d '{"product_id": 1, "quantity": 1, "customer_name": "测试"}' | jq '.'

echo "查询订单测试"
curl -s http://api.example.com/api/v1/orders?user_id=12345 | jq '.data[0].customer_name'
```

---

## 回滚方案

### Schema 回滚

```bash
# Alembic 回滚
alembic downgrade -1  # 回滚一步
alembic downgrade <revision_id>  # 回滚到指定版本

# Flyway 回滚
flyway undo

# Django 回滚
python manage.py migrate <app> <previous_migration>

# Prisma（无内置回滚，需手动）
psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -f rollback.sql
```

### 数据回滚

```bash
# 从备份恢复整个数据库（最后手段）
pg_restore -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" \
  --clean --if-exists \
  "$BACKUP_DIR/pre_migration.dump"

# 恢复单表
pg_restore -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" \
  --table=orders --data-only \
  "$BACKUP_DIR/pre_migration.dump"
```

### 回滚决策矩阵

| 场景 | 处理方式 | 时间预估 |
|------|---------|---------|
| 迁移脚本执行失败 | 事务自动回滚，修复后重试 | 即时 |
| 迁移成功但应用报错 | 执行 downgrade 脚本 | 分钟级 |
| 数据回填错误 | 从备份恢复受影响的表 | 取决于数据量 |
| 性能严重下降 | 回滚索引变更或 Schema 变更 | 分钟级 |
| 数据损坏 | 停止写入 + 从备份恢复 | 小时级 |

### 不可回滚的操作及应对

```markdown
以下操作一旦执行就无法简单回滚：
1. DROP COLUMN（数据丢失）→ 执行前必须备份该列数据
2. TRUNCATE TABLE → 执行前必须完整备份
3. 数据类型缩小（如 VARCHAR(200) → VARCHAR(100)）→ 先验证数据范围
4. 删除索引后重建（大表耗时长）→ 评估重建时间

应对策略：
- 对不可逆操作，始终在执行前创建完整备份
- 在低峰期执行
- 保留回填脚本以备需要重建数据
```

---

## Agent Checklist

AI 编码 Agent 在执行数据库迁移时必须逐项确认：

- [ ] **备份完成**：生产数据库有最新备份，已验证可恢复
- [ ] **前向兼容**：迁移后旧代码仍能正常运行
- [ ] **回滚脚本就绪**：每个 upgrade 都有对应的 downgrade
- [ ] **大表评估**：数据量 > 100 万行的表已评估锁影响和执行时间
- [ ] **索引安全**：使用 CONCURRENTLY 创建索引
- [ ] **分批回填**：数据回填分批执行，控制速度
- [ ] **预发布验证**：迁移脚本已在预发布环境成功执行
- [ ] **结构验证**：迁移后表结构、索引、约束均符合预期
- [ ] **数据验证**：迁移后数据一致性已确认
- [ ] **性能验证**：核心查询的执行计划无退化
- [ ] **应用验证**：应用层功能正常，无新增错误
- [ ] **清理计划**：蓝绿部署的旧列/旧表有计划的清理时间
- [ ] **文档更新**：ER 图和数据字典已同步更新
