---
id: database-migration-strategies
title: 数据库迁移策略
domain: development
category: 01-standards
difficulty: intermediate
tags: [agent, checklist, database, development, migration, strategies, 实战代码示例, 常见陷阱]
quality_score: 70
last_updated: 2026-06-15
---
# 数据库迁移策略

## 概述
数据库迁移是生产系统演进中最高风险的操作之一。本指南覆盖零停机迁移、蓝绿部署、双写策略、CDC(变更数据捕获)和回滚方案,帮助团队安全地执行Schema变更和数据迁移。

## 核心概念

### 1. 迁移类型
- **Schema迁移**: 添加/修改/删除表、列、索引
- **数据迁移**: 数据格式转换、数据清洗、数据搬迁
- **引擎迁移**: MySQL→PostgreSQL、单库→分库
- **云迁移**: 自建→RDS/Aurora/Cloud SQL

### 2. 迁移风险等级

| 操作 | 风险 | 是否锁表 | 零停机可能 |
|------|------|----------|-----------|
| 添加列(有默认值) | 低 | PostgreSQL不锁/MySQL可能锁 | 是 |
| 添加列(无默认值,可空) | 低 | 不锁 | 是 |
| 删除列 | 中 | 不锁 | 需分步 |
| 重命名列 | 高 | 不锁但应用需改 | 需分步 |
| 添加索引 | 中 | 可能锁(需CONCURRENTLY) | 是 |
| 修改列类型 | 高 | 通常锁表 | 需分步 |
| 删除表 | 高 | 不锁 | 需分步 |
| 大表数据迁移 | 高 | 取决于方式 | 需特殊处理 |

### 3. 零停机迁移原则
- **前向兼容**: 新代码必须兼容旧Schema
- **后向兼容**: 旧代码必须兼容新Schema
- **分步执行**: 大变更拆分为多个小步骤
- **可逆操作**: 每步都有回滚方案

## 实战代码示例

### Alembic迁移(Python/SQLAlchemy)

```python
# alembic/versions/001_add_user_profile.py
"""add user profile columns

Revision ID: abc123
Revises: None
Create Date: 2025-01-15 10:00:00
"""
from alembic import op
import sqlalchemy as sa

revision = 'abc123'
down_revision = None

def upgrade():
    # 步骤1: 添加可空列(安全,不锁表)
    op.add_column('users', sa.Column('avatar_url', sa.String(500), nullable=True))
    op.add_column('users', sa.Column('bio', sa.Text(), nullable=True))

    # 步骤2: 并发创建索引(不锁表)
    op.create_index(
        'ix_users_email',
        'users',
        ['email'],
        unique=True,
        postgresql_concurrently=True,
    )

def downgrade():
    op.drop_index('ix_users_email', table_name='users')
    op.drop_column('users', 'bio')
    op.drop_column('users', 'avatar_url')
```

### 零停机列重命名(3步法)

```python
# 步骤1: 添加新列(部署迁移,不改代码)
# migration_001_add_full_name.py
def upgrade():
    op.add_column('users', sa.Column('full_name', sa.String(200), nullable=True))
    # 回填数据
    op.execute("UPDATE users SET full_name = name WHERE full_name IS NULL")

def downgrade():
    op.drop_column('users', 'full_name')

# 步骤2: 双写(部署新代码,同时写两列)
# models.py — 应用层双写
class User(Base):
    __tablename__ = 'users'
    name = Column(String(200))       # 旧列,保持兼容
    full_name = Column(String(200))  # 新列

    def set_name(self, value: str):
        """双写: 同时更新两列"""
        self.name = value
        self.full_name = value

# 步骤3: 切换读取到新列,确认无问题后删除旧列
# migration_003_drop_name.py
def upgrade():
    # 确保所有数据已同步
    op.execute("""
        UPDATE users SET full_name = name
        WHERE full_name IS NULL OR full_name != name
    """)
    op.alter_column('users', 'full_name', nullable=False)
    op.drop_column('users', 'name')

def downgrade():
    op.add_column('users', sa.Column('name', sa.String(200)))
    op.execute("UPDATE users SET name = full_name")
```

### 大表安全添加索引

```sql
-- PostgreSQL: CONCURRENTLY创建索引(不锁表)
CREATE INDEX CONCURRENTLY ix_orders_user_id ON orders(user_id);

-- 注意: CONCURRENTLY不能在事务中使用
-- Alembic需要特殊处理:
```

```python
# alembic迁移中使用CONCURRENTLY
from alembic import op

def upgrade():
    # 必须在事务外执行
    op.execute("COMMIT")  # 结束当前事务
    op.create_index(
        'ix_orders_user_id',
        'orders',
        ['user_id'],
        postgresql_concurrently=True,
    )
```

### 双写迁移模式

```python
# 从旧表迁移到新表的双写模式
class OrderRepository:
    """订单仓库 — 双写迁移阶段"""

    def __init__(self, old_db, new_db, migration_phase: str):
        self.old_db = old_db
        self.new_db = new_db
        self.phase = migration_phase  # "shadow" | "dual" | "cutover" | "cleanup"

    async def create_order(self, order: Order) -> Order:
        if self.phase == "shadow":
            # 阶段1: 写旧库为主,异步写新库(不影响主流程)
            result = await self.old_db.insert(order)
            try:
                await self.new_db.insert(self._transform(order))
            except Exception as e:
                logger.warning(f"Shadow write failed: {e}")
            return result

        elif self.phase == "dual":
            # 阶段2: 双写,两个都必须成功
            result = await self.old_db.insert(order)
            await self.new_db.insert(self._transform(order))
            return result

        elif self.phase == "cutover":
            # 阶段3: 新库为主,旧库同步写
            result = await self.new_db.insert(self._transform(order))
            try:
                await self.old_db.insert(order)
            except Exception as e:
                logger.warning(f"Legacy write failed: {e}")
            return self._reverse_transform(result)

        elif self.phase == "cleanup":
            # 阶段4: 只写新库
            return await self.new_db.insert(self._transform(order))

    async def get_order(self, order_id: int) -> Order:
        if self.phase in ("shadow", "dual"):
            # 读旧库
            result = await self.old_db.get(order_id)
            # 可选: 对比新库数据一致性
            if self.phase == "dual":
                new_result = await self.new_db.get(order_id)
                if result != self._reverse_transform(new_result):
                    logger.error(f"Data inconsistency for order {order_id}")
            return result
        else:
            # 读新库
            return self._reverse_transform(await self.new_db.get(order_id))
```

### CDC变更数据捕获(Debezium)

```yaml
# Debezium连接器配置(Kafka Connect)
# 捕获PostgreSQL变更并发送到Kafka
{
  "name": "pg-source-connector",
  "config": {
    "connector.class": "io.debezium.connector.postgresql.PostgresConnector",
    "database.hostname": "postgres",
    "database.port": "5432",
    "database.user": "replicator",
    "database.password": "${secrets:pg-password}",
    "database.dbname": "myapp",
    "database.server.name": "myapp-db",
    "table.include.list": "public.users,public.orders",
    "plugin.name": "pgoutput",
    "slot.name": "debezium_slot",
    "publication.name": "debezium_pub",
    "topic.prefix": "cdc",
    "transforms": "route",
    "transforms.route.type": "org.apache.kafka.connect.transforms.RegexRouter",
    "transforms.route.regex": "cdc\\.public\\.(.*)",
    "transforms.route.replacement": "cdc.$1"
  }
}
```

```python
# CDC消费者 — 将变更同步到新系统
from aiokafka import AIOKafkaConsumer
import json

async def sync_from_cdc():
    consumer = AIOKafkaConsumer(
        'cdc.users', 'cdc.orders',
        bootstrap_servers='kafka:9092',
        group_id='migration-sync',
        auto_offset_reset='earliest',
    )
    await consumer.start()

    async for msg in consumer:
        event = json.loads(msg.value)
        operation = event["op"]  # c=create, u=update, d=delete
        after = event.get("after")
        before = event.get("before")

        if operation == "c":
            await new_db.insert(transform(after))
        elif operation == "u":
            await new_db.update(transform(after))
        elif operation == "d":
            await new_db.delete(before["id"])

        await consumer.commit()
```

### 回滚策略

```python
# 迁移回滚脚本模板
class MigrationRollback:
    """迁移回滚管理器"""

    def __init__(self, db_url: str):
        self.engine = create_async_engine(db_url)

    async def check_rollback_safety(self, revision: str) -> dict:
        """检查回滚是否安全"""
        checks = {
            "data_loss": False,
            "schema_compatible": True,
            "active_connections": 0,
            "estimated_duration": "0s",
        }

        # 检查是否有数据丢失风险
        migration = get_migration(revision)
        if migration.drops_column or migration.drops_table:
            checks["data_loss"] = True

        # 检查当前活跃连接
        async with self.engine.connect() as conn:
            result = await conn.execute(text(
                "SELECT count(*) FROM pg_stat_activity WHERE state = 'active'"
            ))
            checks["active_connections"] = result.scalar()

        return checks

    async def rollback(self, target_revision: str, dry_run: bool = True):
        """执行回滚"""
        safety = await self.check_rollback_safety(target_revision)

        if safety["data_loss"]:
            logger.warning("Rollback will cause data loss!")
            if not dry_run:
                # 先备份
                await self.backup_affected_tables(target_revision)

        if dry_run:
            logger.info(f"DRY RUN: Would rollback to {target_revision}")
            logger.info(f"Safety checks: {safety}")
            return

        # 执行Alembic回滚
        alembic_cfg = Config("alembic.ini")
        command.downgrade(alembic_cfg, target_revision)
        logger.info(f"Rolled back to {target_revision}")
```

### 分批数据迁移

```python
# 大表数据迁移(分批处理,避免锁表)
import asyncio

async def batch_migrate_orders(batch_size: int = 1000):
    """分批迁移订单数据"""
    last_id = 0
    total_migrated = 0
    errors = []

    while True:
        async with old_db.begin() as conn:
            rows = await conn.execute(text("""
                SELECT * FROM orders
                WHERE id > :last_id
                ORDER BY id
                LIMIT :batch_size
            """), {"last_id": last_id, "batch_size": batch_size})

            batch = rows.fetchall()
            if not batch:
                break

            # 转换并写入新库
            transformed = [transform_order(row) for row in batch]
            try:
                async with new_db.begin() as new_conn:
                    await new_conn.execute(
                        new_orders_table.insert(),
                        transformed,
                    )
                total_migrated += len(batch)
                last_id = batch[-1].id
                logger.info(f"Migrated {total_migrated} orders, last_id={last_id}")
            except Exception as e:
                errors.append({"last_id": last_id, "error": str(e)})
                logger.error(f"Batch failed at id={last_id}: {e}")

            # 控制速率,避免压垮数据库
            await asyncio.sleep(0.1)

    return {"total": total_migrated, "errors": errors}
```

## 最佳实践

### 1. 迁移文件管理
- 每个迁移一个文件,有清晰的描述
- 迁移文件提交到版本控制
- upgrade和downgrade都要实现
- CI中运行迁移测试(创建→回滚→再创建)

### 2. 零停机变更清单
- 添加列: 只添加可空列或有默认值的列
- 删除列: 先停止代码读写该列 → 下次部署再删除
- 重命名列: 添加新列 → 双写 → 切换读取 → 删除旧列
- 添加索引: 使用CONCURRENTLY(PostgreSQL)或pt-online-schema-change(MySQL)
- 修改列类型: 添加新列 → 双写 → 切换 → 删除旧列

### 3. 数据验证
- 迁移前后对比行数
- 抽样验证数据正确性
- 校验聚合值(SUM/COUNT)一致
- 运行完整性检查(外键/唯一约束)

### 4. 回滚准备
- 每次迁移前备份
- 准备回滚脚本并测试过
- 设定回滚时间窗口
- 定义回滚触发条件(错误率/延迟阈值)

### 5. 生产执行规范
- 选择低峰期执行
- 先在staging环境验证
- 通知相关团队
- 实时监控数据库性能指标
- 保持回滚能力至少24小时

## 常见陷阱

### 陷阱1: PostgreSQL添加列加默认值锁表
```sql
-- PostgreSQL 10以下: 会锁表
ALTER TABLE users ADD COLUMN status VARCHAR(20) DEFAULT 'active';

-- 正确(PG 11+不锁表,但旧版本需要分步):
ALTER TABLE users ADD COLUMN status VARCHAR(20);
-- 然后分批更新
UPDATE users SET status = 'active' WHERE id BETWEEN 1 AND 10000;
```

### 陷阱2: 忘记NOT NULL约束迁移
```python
# 错误: 直接添加NOT NULL列(如果表有数据会失败)
op.add_column('users', sa.Column('role', sa.String(50), nullable=False))

# 正确: 分步添加
# 步骤1: 添加可空列
op.add_column('users', sa.Column('role', sa.String(50), nullable=True))
# 步骤2: 回填数据
op.execute("UPDATE users SET role = 'user' WHERE role IS NULL")
# 步骤3: 添加NOT NULL约束
op.alter_column('users', 'role', nullable=False)
```

### 陷阱3: 大表一次性UPDATE
```sql
-- 错误: 一次更新千万行,锁表+日志暴涨
UPDATE orders SET status = 'migrated';

-- 正确: 分批更新
DO $$
DECLARE
    batch_size INT := 10000;
    affected INT;
BEGIN
    LOOP
        UPDATE orders SET status = 'migrated'
        WHERE id IN (
            SELECT id FROM orders
            WHERE status IS NULL
            LIMIT batch_size
            FOR UPDATE SKIP LOCKED
        );
        GET DIAGNOSTICS affected = ROW_COUNT;
        EXIT WHEN affected = 0;
        PERFORM pg_sleep(0.1);
    END LOOP;
END $$;
```

### 陷阱4: 迁移脚本不可重复执行
```python
# 错误: 重复运行会报错
def upgrade():
    op.create_table('users', ...)  # 表已存在会失败

# 正确: 检查是否已存在
def upgrade():
    if not op.get_bind().dialect.has_table(op.get_bind(), 'users'):
        op.create_table('users', ...)
```

## Agent Checklist

### 迁移准备
- [ ] 变更类型和风险等级已评估
- [ ] 零停机方案已设计(如需要)
- [ ] 回滚脚本已准备并测试
- [ ] 备份策略已确认

### 执行规范
- [ ] 迁移已在staging环境验证
- [ ] 选择了低峰期执行窗口
- [ ] 相关团队已通知
- [ ] 监控面板已就绪

### 数据验证
- [ ] 迁移前后数据行数一致
- [ ] 抽样数据验证正确
- [ ] 聚合值校验通过
- [ ] 完整性约束检查通过

### 后续清理
- [ ] 旧Schema/旧表已计划清理
- [ ] 双写代码有计划移除
- [ ] 迁移文档已更新
- [ ] 回滚窗口结束后确认稳定
