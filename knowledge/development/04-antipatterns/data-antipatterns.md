---
id: data-antipatterns
title: 数据工程反模式指南
domain: development
category: 04-antipatterns
difficulty: intermediate
tags: [antipatterns, audit, backups, data, development, fields, governance, inconsistency]
quality_score: 70
last_updated: 2026-06-15
---
# 数据工程反模式指南

> 适用范围：数据管道 / ETL / 数据仓库 / 数据湖
> 约束级别：SHALL（必须在数据架构评审阶段拦截）

---

## 1. 无索引治理（Missing Index Governance）

### 描述
数据库索引无系统性管理：该建的索引不建（慢查询长期存在），不该建的索引不删（冗余索引拖慢写入），索引策略不随查询模式变化而更新。数据量从千级增长到亿级时，问题从慢查询升级为服务不可用。

### 错误示例
```sql
-- 表有 2 亿行，以下查询每天执行 10 万次，无索引
SELECT * FROM events
WHERE user_id = 12345
AND event_type = 'purchase'
AND created_at > '2024-01-01';
-- Seq Scan，执行时间 15 秒

-- 同时存在大量冗余索引
CREATE INDEX idx_events_user ON events(user_id);
CREATE INDEX idx_events_user_type ON events(user_id, event_type);
CREATE INDEX idx_events_user_type_date ON events(user_id, event_type, created_at);
CREATE INDEX idx_events_user_date ON events(user_id, created_at);
CREATE INDEX idx_events_type ON events(event_type);
-- 5 个索引中有 3 个是冗余的，写入时多维护 3 个索引
```

### 正确示例
```sql
-- 根据实际查询模式，保留最优索引组合
CREATE INDEX idx_events_user_type_date ON events(user_id, event_type, created_at DESC);
-- 一个复合索引覆盖所有查询模式：
-- WHERE user_id = ?
-- WHERE user_id = ? AND event_type = ?
-- WHERE user_id = ? AND event_type = ? AND created_at > ?

-- 删除冗余索引
DROP INDEX idx_events_user;       -- 被 idx_events_user_type_date 包含
DROP INDEX idx_events_user_type;  -- 被 idx_events_user_type_date 包含
DROP INDEX idx_events_user_date;  -- 查询模式已不使用
DROP INDEX idx_events_type;       -- 低选择性，全表扫描更快
```

```python
# 索引巡检自动化
class IndexGovernance:
    def audit_unused_indexes(self, days: int = 30) -> list[dict]:
        """找出过去 N 天未使用的索引"""
        return self._db.execute("""
            SELECT schemaname, relname, indexrelname, idx_scan, pg_size_pretty(pg_relation_size(indexrelid))
            FROM pg_stat_user_indexes
            WHERE idx_scan = 0
            AND indexrelname NOT LIKE '%_pkey'
            ORDER BY pg_relation_size(indexrelid) DESC
        """).fetchall()

    def audit_missing_indexes(self) -> list[dict]:
        """找出可能缺失索引的表（顺序扫描次数高）"""
        return self._db.execute("""
            SELECT relname, seq_scan, seq_tup_read, idx_scan, idx_tup_fetch,
                   ROUND(seq_scan::numeric / NULLIF(seq_scan + idx_scan, 0) * 100, 2) AS seq_pct
            FROM pg_stat_user_tables
            WHERE seq_scan > 1000
            AND seq_scan > idx_scan
            ORDER BY seq_tup_read DESC
            LIMIT 20
        """).fetchall()

    def audit_duplicate_indexes(self) -> list[dict]:
        """找出重复或包含关系的索引"""
        return self._db.execute("""
            SELECT a.indexrelid::regclass AS index_a,
                   b.indexrelid::regclass AS index_b,
                   pg_size_pretty(pg_relation_size(a.indexrelid)) AS size_a
            FROM pg_index a
            JOIN pg_index b ON a.indrelid = b.indrelid
                AND a.indexrelid != b.indexrelid
                AND a.indkey::text LIKE b.indkey::text || '%'
            WHERE a.indisvalid AND b.indisvalid
        """).fetchall()
```

### 检测方法
- `pg_stat_user_indexes` 中 `idx_scan = 0` 的索引（从未使用）。
- `pg_stat_user_tables` 中 `seq_scan >> idx_scan` 的表（缺索引）。
- 慢查询日志中 Top 20 SQL 的执行计划。
- 定期运行索引巡检脚本。

### 修复步骤
1. 收集过去 30 天的慢查询日志和索引使用统计。
2. 为高频慢查询创建合适的索引。
3. 删除未使用和冗余的索引。
4. 建立月度索引巡检制度。
5. 在 CI 中对 Schema 变更触发索引影响评估。

### Agent Checklist
- [ ] 高频查询有索引覆盖
- [ ] 无使用次数为 0 的冗余索引
- [ ] 无包含关系的重复索引
- [ ] 有月度索引巡检机制
- [ ] Schema 变更有索引影响评估

---

## 2. 缓存与数据库一致性冲突（Cache Inconsistency）

### 描述
缓存更新策略与数据库写入策略不一致，导致用户读到过期数据。常见问题：先更新缓存再更新数据库（数据库失败但缓存已更新）、先删缓存再更新数据库（并发读回填旧值）、缓存无过期时间（数据永久不一致）。

### 错误示例
```python
# 先更新缓存再更新数据库 -- 数据库失败时缓存已是新值
def update_user_name(user_id, new_name):
    cache.set(f"user:{user_id}", {"name": new_name})  # 缓存已更新
    db.execute("UPDATE users SET name = %s WHERE id = %s", (new_name, user_id))
    # 如果数据库更新失败，缓存中是新值，数据库中是旧值

# 先删缓存再更新数据库 -- 并发读回填旧值
def update_product_price(product_id, new_price):
    cache.delete(f"product:{product_id}")        # T1: 删缓存
    # T2: 另一个请求读到缓存 miss，从数据库读到旧值，回填缓存
    db.execute("UPDATE products SET price = %s WHERE id = %s", (new_price, product_id))
    # 结果：缓存中是旧价格

# 缓存无 TTL -- 永久不一致
def get_config(key):
    cached = cache.get(f"config:{key}")
    if cached:
        return cached
    value = db.execute("SELECT value FROM config WHERE key = %s", (key,)).fetchone()
    cache.set(f"config:{key}", value)  # 无 TTL，永不过期
    return value
```

### 正确示例
```python
# 方案 1: Cache-Aside + 延迟双删
class UserService:
    def update_name(self, user_id: int, new_name: str) -> None:
        # 1. 先更新数据库
        self._db.execute(
            "UPDATE users SET name = %s WHERE id = %s", (new_name, user_id)
        )
        # 2. 删除缓存
        self._cache.delete(f"user:{user_id}")
        # 3. 延迟双删（防止并发读回填旧值）
        asyncio.get_event_loop().call_later(
            1.0,  # 1 秒后再删一次
            self._cache.delete, f"user:{user_id}"
        )

    def get_user(self, user_id: int) -> User:
        # Cache-Aside 模式
        cached = self._cache.get(f"user:{user_id}")
        if cached:
            return User.model_validate_json(cached)

        user = self._db.get_user(user_id)
        if user:
            self._cache.setex(
                f"user:{user_id}",
                300,  # 5 分钟 TTL
                user.model_dump_json(),
            )
        return user

# 方案 2: Write-Through（强一致性场景）
class InventoryService:
    def update_stock(self, product_id: int, new_stock: int) -> None:
        with self._db.transaction() as tx:
            tx.execute(
                "UPDATE products SET stock = %s WHERE id = %s", (new_stock, product_id)
            )
            # 在同一个事务中更新缓存（Redis Pipeline）
            self._cache.setex(
                f"stock:{product_id}", 60, str(new_stock)
            )

# 方案 3: 事件驱动缓存更新
class CacheInvalidator:
    """订阅数据库变更事件，异步刷新缓存"""
    async def on_user_updated(self, event: UserUpdatedEvent):
        await self._cache.delete(f"user:{event.user_id}")
        # 预热：重新加载热点数据
        if await self._is_hot_key(f"user:{event.user_id}"):
            user = await self._db.get_user(event.user_id)
            await self._cache.setex(f"user:{event.user_id}", 300, user.json())
```

### 检测方法
- 缓存 `SET` 操作在数据库 `UPDATE` 之前。
- 缓存无 TTL（`SET` 不带过期时间）。
- 存在 "先删缓存再更新数据库" 模式且无延迟双删。
- 数据对账（缓存 vs 数据库）发现不一致。

### 修复步骤
1. 确定一致性需求：最终一致性（Cache-Aside + TTL）vs 强一致性（Write-Through）。
2. 统一缓存更新模式：先更新数据库，再删除缓存。
3. 所有缓存设置 TTL（兜底保护）。
4. 对高并发场景添加延迟双删。
5. 建立缓存与数据库的定期对账机制。

### Agent Checklist
- [ ] 缓存更新在数据库更新之后（不是之前）
- [ ] 所有缓存有 TTL
- [ ] 高并发场景有延迟双删
- [ ] 有缓存数据对账机制
- [ ] 缓存一致性策略有文档

---

## 3. 关键表缺审计字段（Missing Audit Fields）

### 描述
业务表缺少 `created_at`、`updated_at`、`created_by`、`updated_by` 等审计字段，出问题时无法追溯数据变更的时间和操作者。在合规场景下（金融、医疗），缺少审计字段可能违反法规。

### 错误示例
```sql
CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    total DECIMAL(10, 2) NOT NULL,
    status VARCHAR(20) NOT NULL
    -- 无 created_at: 不知道订单何时创建
    -- 无 updated_at: 不知道最后一次修改是什么时候
    -- 无 created_by: 不知道谁创建的（系统还是人工）
    -- 无 version: 不知道修改了几次
);
```

### 正确示例
```sql
CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    total DECIMAL(10, 2) NOT NULL,
    status VARCHAR(20) NOT NULL,
    -- 审计字段
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by INTEGER,      -- 操作者 ID
    updated_by INTEGER,      -- 最后修改者 ID
    version INTEGER NOT NULL DEFAULT 1,  -- 乐观锁版本号
    deleted_at TIMESTAMP WITH TIME ZONE  -- 软删除
);

-- 自动更新 updated_at
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    NEW.version = OLD.version + 1;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_orders_updated_at
    BEFORE UPDATE ON orders
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- 变更审计日志表
CREATE TABLE audit_log (
    id BIGSERIAL PRIMARY KEY,
    table_name VARCHAR(100) NOT NULL,
    record_id INTEGER NOT NULL,
    action VARCHAR(10) NOT NULL,  -- INSERT / UPDATE / DELETE
    old_values JSONB,
    new_values JSONB,
    changed_by INTEGER,
    changed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
```

```python
# ORM 自动填充审计字段
class AuditMixin:
    created_at = Column(DateTime(timezone=True), server_default=func.now(), nullable=False)
    updated_at = Column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now(), nullable=False)
    created_by = Column(Integer, nullable=True)
    updated_by = Column(Integer, nullable=True)
    version = Column(Integer, nullable=False, default=1)

class Order(Base, AuditMixin):
    __tablename__ = "orders"
    id = Column(Integer, primary_key=True)
    user_id = Column(Integer, nullable=False)
    total = Column(Numeric(10, 2), nullable=False)
    status = Column(String(20), nullable=False)
```

### 检测方法
- 表结构无 `created_at` / `updated_at` 列。
- 无变更审计日志表。
- 数据变更后无法追溯操作者和时间。
- Schema 审查工具检查审计字段覆盖率。

### 修复步骤
1. 为所有业务表添加 `created_at`、`updated_at`、`created_by`、`updated_by`、`version` 字段。
2. 创建数据库触发器自动更新 `updated_at` 和 `version`。
3. 创建审计日志表记录敏感表的变更历史。
4. ORM 层使用 Mixin 统一管理审计字段。
5. 定期审查审计字段覆盖率。

### Agent Checklist
- [ ] 所有业务表有 `created_at` + `updated_at`
- [ ] 敏感表有 `created_by` + `updated_by`
- [ ] 有数据库触发器自动更新审计字段
- [ ] 有变更审计日志表
- [ ] ORM 使用 Mixin 统一管理审计字段

---

## 4. 备份未演练（Untested Backups）

### 描述
数据库有定时备份，但从未验证过备份能否成功恢复。备份文件可能已损坏、不完整、格式不兼容，直到真正需要恢复时才发现无法使用。

### 错误示例
```bash
# 每天凌晨备份（看起来在跑）
0 3 * * * pg_dump mydb > /backups/mydb_$(date +%Y%m%d).sql

# 问题：
# 1. 从未测试过恢复
# 2. 备份文件存在同一台机器（机器坏了备份也没了）
# 3. 无备份大小监控（文件可能是空的）
# 4. 无保留策略（磁盘早晚满）
# 5. 不知道恢复需要多长时间
```

### 正确示例
```python
import subprocess
from datetime import datetime

class BackupManager:
    def create_backup(self) -> BackupResult:
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        local_path = f"/backups/mydb_{timestamp}.sql.gz"

        # 1. 创建备份（压缩）
        result = subprocess.run(
            ["pg_dump", "-Fc", "-Z", "9", "-f", local_path, self._db_name],
            capture_output=True, text=True, timeout=3600,
        )
        if result.returncode != 0:
            raise BackupError(f"pg_dump failed: {result.stderr}")

        # 2. 验证备份文件大小
        file_size = os.path.getsize(local_path)
        if file_size < self._min_expected_size:
            raise BackupError(f"Backup too small: {file_size} bytes")

        # 3. 上传到远程存储（异地备份）
        s3_key = f"backups/mydb/{timestamp}.sql.gz"
        self._s3.upload_file(local_path, self._bucket, s3_key)

        # 4. 记录备份元数据
        return BackupResult(
            timestamp=timestamp,
            size=file_size,
            local_path=local_path,
            remote_path=f"s3://{self._bucket}/{s3_key}",
            checksum=self._compute_checksum(local_path),
        )

    def verify_backup(self, backup: BackupResult) -> bool:
        """在测试环境恢复备份，验证数据完整性"""
        # 1. 下载备份
        local_path = self._s3.download(backup.remote_path)

        # 2. 恢复到测试数据库
        subprocess.run(
            ["pg_restore", "-d", self._test_db, "-c", local_path],
            check=True, timeout=7200,
        )

        # 3. 验证关键表的行数
        for table, expected_min in self._verification_tables.items():
            count = self._test_db.execute(f"SELECT COUNT(*) FROM {table}").scalar()
            if count < expected_min:
                raise VerificationError(f"{table} has {count} rows, expected >= {expected_min}")

        # 4. 验证最新数据的时间戳
        latest = self._test_db.execute(
            "SELECT MAX(updated_at) FROM orders"
        ).scalar()
        if (datetime.now() - latest).hours > 24:
            raise VerificationError("Backup data is more than 24 hours old")

        return True

    def cleanup_old_backups(self, retention_days: int = 30):
        """清理超过保留期的备份"""
        cutoff = datetime.now() - timedelta(days=retention_days)
        old_backups = self._list_backups_before(cutoff)
        for backup in old_backups:
            self._s3.delete(backup.remote_path)
            os.remove(backup.local_path)
```

```yaml
# 备份策略
backup_policy:
  schedule: "0 3 * * *"  # 每天凌晨 3 点
  type: "full"           # 全量备份
  retention: 30          # 保留 30 天
  storage:
    primary: "s3://backup-bucket/prod/"
    secondary: "gs://backup-bucket-dr/prod/"  # 异地容灾
  verification:
    schedule: "0 6 * * 1"  # 每周一早上 6 点验证
    target_db: "backup-test-db"
  alerts:
    backup_failed: critical
    backup_too_small: warning
    verification_failed: critical
    no_backup_24h: critical
```

### 检测方法
- 无备份恢复演练记录。
- 备份文件存储在同一台机器或同一可用区。
- 无备份大小监控告警。
- 不知道恢复一次需要多长时间（RTO 未知）。
- 无备份保留和清理策略。

### 修复步骤
1. 备份上传到远程存储（S3 / GCS），至少保留异地一份。
2. 每周自动在测试环境执行恢复验证。
3. 监控备份文件大小，异常时告警。
4. 记录 RTO（恢复时间目标），确保在可接受范围内。
5. 设置备份保留策略，自动清理过期备份。
6. 每季度手动演练一次完整的灾难恢复流程。

### Agent Checklist
- [ ] 有自动化备份（每日或更频繁）
- [ ] 备份存储在异地（不同区域 / 不同云）
- [ ] 有每周自动恢复验证
- [ ] 备份大小有监控告警
- [ ] RTO 已测量且在可接受范围内
- [ ] 有备份保留和清理策略

---

## 5. 数据管道无幂等（Non-Idempotent Data Pipeline）

### 描述
ETL / 数据管道在重试或重复运行时产生重复数据。管道失败后重跑导致数据翻倍，或者部分成功的状态无法安全重试。

### 错误示例
```python
# 非幂等的 ETL -- 重跑产生重复数据
def sync_orders_to_warehouse():
    orders = source_db.execute("SELECT * FROM orders WHERE date = CURRENT_DATE")
    for order in orders:
        warehouse_db.execute(
            "INSERT INTO fact_orders (order_id, amount, date) VALUES (%s, %s, %s)",
            (order["id"], order["amount"], order["date"])
        )
    # 如果中间失败重跑，已插入的数据会重复
```

### 正确示例
```python
# 幂等的 ETL -- 使用 UPSERT
def sync_orders_to_warehouse(date: str):
    orders = source_db.execute(
        "SELECT * FROM orders WHERE date = %s", (date,)
    )

    with warehouse_db.transaction() as tx:
        for order in orders:
            tx.execute("""
                INSERT INTO fact_orders (order_id, amount, date, synced_at)
                VALUES (%s, %s, %s, NOW())
                ON CONFLICT (order_id)
                DO UPDATE SET amount = EXCLUDED.amount, synced_at = NOW()
            """, (order["id"], order["amount"], order["date"]))

        # 记录同步水位线
        tx.execute("""
            INSERT INTO sync_watermarks (pipeline, last_sync_date, synced_at)
            VALUES ('orders', %s, NOW())
            ON CONFLICT (pipeline)
            DO UPDATE SET last_sync_date = EXCLUDED.last_sync_date, synced_at = NOW()
        """, (date,))

# 分区替换模式（大批量场景）
def sync_daily_events(date: str):
    """整个分区替换，天然幂等"""
    # 1. 写入临时表
    temp_table = f"tmp_events_{date.replace('-', '')}"
    warehouse_db.execute(f"CREATE TABLE IF NOT EXISTS {temp_table} (LIKE fact_events INCLUDING ALL)")
    warehouse_db.execute(f"TRUNCATE {temp_table}")

    # 2. 批量导入到临时表
    events = source_db.execute("SELECT * FROM events WHERE date = %s", (date,))
    warehouse_db.bulk_insert(temp_table, events)

    # 3. 原子替换分区
    warehouse_db.execute(f"""
        ALTER TABLE fact_events DETACH PARTITION fact_events_{date.replace('-', '')};
        ALTER TABLE fact_events ATTACH PARTITION {temp_table}
            FOR VALUES FROM ('{date}') TO ('{date}'::date + INTERVAL '1 day');
    """)
```

### 检测方法
- ETL 使用 `INSERT` 而非 `INSERT ... ON CONFLICT` / `MERGE`。
- 无同步水位线记录（不知道同步到哪里了）。
- 重跑管道后数据量翻倍。
- 无去重逻辑或唯一约束。

### 修复步骤
1. 所有 ETL 写入使用 UPSERT（`INSERT ... ON CONFLICT` / `MERGE`）。
2. 目标表建立业务唯一约束（防止重复）。
3. 记录同步水位线，重跑时从水位线处开始。
4. 大批量场景使用分区替换模式。
5. ETL 测试包含重复运行场景验证。

### Agent Checklist
- [ ] ETL 写入使用 UPSERT / MERGE
- [ ] 目标表有业务唯一约束
- [ ] 有同步水位线记录
- [ ] 重复运行不产生重复数据
- [ ] 有重复运行的测试用例

---

## 6. 数据质量无监控（Missing Data Quality Monitoring）

### 描述
数据管道只关注是否执行成功，不检查数据本身的质量。数据可能为空、重复、格式异常、超出合理范围，但管道显示 "成功"。下游消费者发现数据问题时，问题已经扩散。

### 错误示例
```python
# 管道 "成功" 但数据有问题
def sync_user_data():
    data = fetch_from_api("/users")
    db.bulk_insert("users", data)
    logger.info(f"Synced {len(data)} users")  # "成功"
    # 问题：data 可能是空列表、字段可能缺失、email 可能无效
```

### 正确示例
```python
from great_expectations import DataContext

class DataQualityChecker:
    def check_user_data(self, data: list[dict]) -> QualityReport:
        checks = [
            # 完整性检查
            Check("row_count", lambda d: len(d) > 0, "Data is not empty"),
            Check("row_count_range", lambda d: 100 < len(d) < 1000000, "Row count in expected range"),

            # 唯一性检查
            Check("unique_ids", lambda d: len(set(r["id"] for r in d)) == len(d), "IDs are unique"),

            # 格式检查
            Check("valid_emails", lambda d: all(
                re.match(r"^[^@]+@[^@]+\.[^@]+$", r.get("email", "")) for r in d
            ), "All emails are valid format"),

            # 范围检查
            Check("age_range", lambda d: all(
                0 < r.get("age", 0) < 150 for r in d
            ), "All ages in valid range"),

            # 时效性检查
            Check("freshness", lambda d: any(
                parse_date(r.get("updated_at", "1970-01-01")) > datetime.now() - timedelta(hours=24)
                for r in d
            ), "Data is fresh (updated within 24h)"),

            # NULL 检查
            Check("no_null_names", lambda d: all(
                r.get("name") is not None and r.get("name").strip() != "" for r in d
            ), "No null or empty names"),
        ]

        results = [check.run(data) for check in checks]
        report = QualityReport(checks=results)

        if not report.all_passed:
            logger.error("Data quality check failed", failures=report.failures)
            alert_service.send(
                severity="critical" if report.critical_failures else "warning",
                message=f"Data quality issues: {report.summary}",
            )

        return report

# 在管道中集成质量检查
def sync_user_data():
    data = fetch_from_api("/users")

    # 入库前检查
    report = quality_checker.check_user_data(data)
    if report.critical_failures:
        raise DataQualityError(f"Critical quality issues: {report.failures}")

    db.bulk_insert("users", data)

    # 入库后检查
    db_count = db.execute("SELECT COUNT(*) FROM users WHERE synced_at = NOW()::date").scalar()
    if abs(db_count - len(data)) > 0:
        raise DataQualityError(f"Row count mismatch: API={len(data)}, DB={db_count}")

    logger.info("User data synced", row_count=len(data), quality_score=report.score)
```

### 检测方法
- 数据管道无质量检查步骤。
- 下游消费者频繁报告数据异常。
- 无数据完整性 / 唯一性 / 时效性监控。
- 管道 "成功" 但数据为空或严重异常。

### 修复步骤
1. 为每个数据管道定义质量检查规则（完整性、唯一性、格式、范围、时效性）。
2. 入库前执行质量检查，关键规则失败时阻断写入。
3. 入库后验证行数一致性。
4. 使用 Great Expectations / dbt test / Soda 等工具自动化。
5. 质量检查结果纳入监控仪表板。

### Agent Checklist
- [ ] 数据管道有入库前质量检查
- [ ] 关键质量规则失败时阻断写入
- [ ] 有行数一致性验证
- [ ] 有数据时效性监控
- [ ] 质量检查结果有仪表板

---

## 全局 Agent Checklist

| 检查项 | 阈值 | 工具 |
|--------|------|------|
| 高频查询有索引 | 100% | `EXPLAIN ANALYZE` |
| 冗余索引 | 0 个 | `pg_stat_user_indexes` |
| 缓存有 TTL | 100% | Code Review |
| 业务表有审计字段 | 100% | Schema 审查 |
| 备份恢复验证 | 每周 1 次 | 备份系统 |
| ETL 幂等 | 100% | 重复运行测试 |
| 数据质量检查 | 每个管道 | 管道配置审查 |
