---
title: 系统迁移作战手册
version: 1.0.0
last_updated: 2026-03-28
owner: architecture-team
tags: [migration, blue-green, canary, strangler-fig, database-migration, zero-downtime]
status: production
domain: architecture
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：系统迁移全流程作战手册
# 作用：指导团队完成系统迁移的评估、规划、执行、验证与切换
# 创建时间：2026-03-28
# 最后修改：2026-03-28

## 目标

建立系统迁移标准化流程，确保：
- 迁移过程零数据丢失（RPO = 0）
- 业务中断时间 < 5 分钟（零停机迁移场景 < 0）
- 每个迁移步骤可回滚
- 迁移后系统功能、性能、数据完整性全量验证通过
- 迁移过程全程可观测、可审计

## 适用场景

- 单体到微服务拆分（Strangler Fig 模式）
- 数据库迁移（MySQL → PostgreSQL / 单机 → 分布式）
- 云迁移（IDC → 云 / 云A → 云B）
- 框架升级（Spring Boot 2 → 3 / Django 3 → 5）
- 基础设施升级（Kubernetes 版本升级 / 操作系统升级）
- 第三方服务替换（支付渠道切换 / 短信服务商更换）

## 前置条件

### 必要条件

- [ ] 迁移目标与成功标准已明确定义
- [ ] 迁移范围已确定（系统/数据/接口/配置）
- [ ] 当前系统已有完整监控（基线数据可对比）
- [ ] 最新全量备份已完成并验证可恢复
- [ ] 回滚方案已制定并演练
- [ ] 迁移时间窗口已与业务方确认
- [ ] 团队已进行迁移方案培训

### 风险评估矩阵

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| 数据不一致 | 中 | 高 | 双写 + 数据校验脚本 |
| 性能退化 | 中 | 中 | 灰度发布 + 实时监控 |
| 依赖方不兼容 | 低 | 高 | 提前联调 + 适配层 |
| 迁移时间超预期 | 中 | 中 | 分批迁移 + 回滚预案 |
| 数据迁移失败 | 低 | 高 | 增量同步 + 断点续传 |

---

## 一、评估阶段

### 1.1 现状梳理

```yaml
系统画像:
  服务清单:
    - 列出所有服务/模块及其职责
    - 标注每个服务的技术栈版本
    - 记录服务间依赖关系图

  数据资产:
    - 数据库列表（类型/版本/大小/表数量）
    - 数据增长趋势（日/月增量）
    - 数据保留策略
    - 敏感数据分类（PII/金融/医疗）

  接口清单:
    - 内部 API（服务间调用）
    - 外部 API（第三方集成）
    - 消息队列 Topic
    - 定时任务

  基础设施:
    - 服务器清单（规格/数量/利用率）
    - 网络拓扑
    - 存储配置
    - 证书/密钥清单
```

### 1.2 迁移复杂度评估

```yaml
评分模型（每项 1-5 分，总分 = 加权和）:
  数据量: weight=3
    1: < 10 GB
    3: 10-500 GB
    5: > 500 GB

  服务依赖数: weight=3
    1: 0-2 个依赖
    3: 3-5 个依赖
    5: > 5 个依赖

  停机容忍度: weight=5
    1: 可接受 4+ 小时停机
    3: 可接受 30 分钟停机
    5: 零停机要求

  数据一致性要求: weight=4
    1: 最终一致即可
    3: 短暂不一致可接受（< 5分钟）
    5: 严格强一致

  团队经验: weight=2
    1: 团队有丰富迁移经验
    3: 部分成员有经验
    5: 团队无迁移经验

总分解读:
  < 30: 低复杂度 → 简单停机迁移即可
  30-50: 中复杂度 → 蓝绿部署/灰度发布
  > 50: 高复杂度 → Strangler Fig + 灰度 + 双写
```

### 1.3 迁移策略选择

```yaml
蓝绿部署（Blue-Green）:
  适用: 整体切换/新旧环境可完全并行
  优势: 切换快速（秒级）/回滚简单
  劣势: 需要双倍资源/数据库迁移复杂
  典型场景: 应用版本升级/云环境切换

金丝雀发布（Canary Release）:
  适用: 渐进式迁移/风险控制优先
  优势: 风险可控/逐步放量
  劣势: 需要流量分配能力/双版本共存期较长
  典型场景: 核心服务升级/新架构验证

Strangler Fig（绞杀者模式）:
  适用: 单体到微服务/长期渐进式迁移
  优势: 低风险/按功能模块迁移
  劣势: 过渡期维护成本高/需要请求路由层
  典型场景: 遗留系统现代化

大爆炸迁移（Big Bang）:
  适用: 系统简单/可接受停机/无增量方案
  优势: 一次到位/无需维护双系统
  劣势: 风险集中/回滚困难
  典型场景: 小型内部系统/非核心服务
```

---

## 二、规划阶段

### 2.1 迁移计划制定

```yaml
里程碑规划:
  M1 - 环境准备（第 1-2 周）:
    - 目标环境搭建
    - 网络打通与安全组配置
    - 监控与告警部署
    - 自动化脚本编写

  M2 - 数据迁移（第 3-4 周）:
    - 全量数据同步
    - 增量同步机制建立
    - 数据校验工具开发
    - 数据校验通过

  M3 - 应用迁移（第 5-6 周）:
    - 应用部署到新环境
    - 配置调整与适配
    - 功能回归测试
    - 性能基准测试

  M4 - 灰度切换（第 7 周）:
    - 1% 流量切换 + 24h 观察
    - 10% 流量切换 + 24h 观察
    - 50% 流量切换 + 48h 观察
    - 100% 流量切换

  M5 - 善后清理（第 8 周）:
    - 旧环境保留 7 天（回滚窗口）
    - 旧环境下线
    - 文档更新
    - 复盘会议
```

### 2.2 数据迁移方案

```yaml
全量迁移:
  # PostgreSQL → PostgreSQL（跨版本/跨实例）
  方案A - pg_dump/pg_restore:
    导出: pg_dump -Fc -j 8 -h old-host -d production > full-backup.dump
    导入: pg_restore -j 8 -h new-host -d production full-backup.dump
    适用: 数据量 < 100GB
    耗时: ~1GB/分钟（取决于网络和磁盘）

  方案B - 逻辑复制:
    # 老库配置
    ALTER SYSTEM SET wal_level = 'logical';
    # 创建 Publication
    CREATE PUBLICATION migration_pub FOR ALL TABLES;
    # 新库订阅
    CREATE SUBSCRIPTION migration_sub
      CONNECTION 'host=old-host dbname=production'
      PUBLICATION migration_pub;
    适用: 零停机迁移，数据量不限
    注意: DDL 不会自动同步，需手动在新库执行

  方案C - 异构迁移（MySQL → PostgreSQL）:
    工具: pgloader
    命令: pgloader mysql://user:pass@old-host/db postgresql://user:pass@new-host/db
    注意: 数据类型映射需提前验证

增量同步:
  方案A - CDC（Change Data Capture）:
    工具: Debezium
    流程: 源DB → Debezium → Kafka → 目标DB
    配置示例:
      connector.class: io.debezium.connector.postgresql.PostgresConnector
      database.hostname: old-host
      database.port: 5432
      database.dbname: production
      slot.name: debezium_migration
      plugin.name: pgoutput

  方案B - 双写:
    流程: 应用同时写老库和新库
    实现: 在 Repository 层添加双写逻辑
    风险: 事务一致性需额外处理
    适用: CDC 不可用时的降级方案
```

### 2.3 数据校验方案

```bash
# 行数校验
echo "--- 行数对比 ---"
for table in users orders products payments; do
  old_count=$(psql -h old-host -d production -t -c "SELECT count(*) FROM ${table}")
  new_count=$(psql -h new-host -d production -t -c "SELECT count(*) FROM ${table}")
  echo "${table}: old=${old_count} new=${new_count} match=$([ $old_count -eq $new_count ] && echo YES || echo NO)"
done

# 校验和对比（采样）
echo "--- 校验和对比 ---"
for table in users orders products; do
  old_md5=$(psql -h old-host -d production -t -c "SELECT md5(string_agg(t::text, '')) FROM (SELECT * FROM ${table} ORDER BY id LIMIT 10000) t")
  new_md5=$(psql -h new-host -d production -t -c "SELECT md5(string_agg(t::text, '')) FROM (SELECT * FROM ${table} ORDER BY id LIMIT 10000) t")
  echo "${table}: $([ "$old_md5" = "$new_md5" ] && echo MATCH || echo MISMATCH)"
done

# 业务关键指标对比
echo "--- 业务指标对比 ---"
for query in \
  "SELECT sum(total_amount) FROM orders WHERE created_at > now() - interval '24h'" \
  "SELECT count(DISTINCT user_id) FROM orders WHERE created_at > now() - interval '24h'" \
  "SELECT count(*) FROM users WHERE status = 'active'"; do
  old_val=$(psql -h old-host -d production -t -c "$query")
  new_val=$(psql -h new-host -d production -t -c "$query")
  echo "old=${old_val} new=${new_val}"
done
```

---

## 三、执行阶段

### 3.1 蓝绿部署执行

```yaml
前提:
  - Green 环境已完全就绪（应用 + 数据 + 配置）
  - 数据同步延迟 < 1 秒
  - Green 环境通过全量回归测试
  - 监控大盘已同时覆盖 Blue 和 Green

切换步骤:
  1. 确认 Green 环境健康:
    - 所有 Pod/实例 Running
    - 健康检查全部通过
    - 数据同步追平

  2. 停止增量同步（如使用逻辑复制）:
    # 记录 LSN 位点
    psql -h old-host -c "SELECT pg_current_wal_lsn()"
    # 确认新库追平
    psql -h new-host -c "SELECT * FROM pg_stat_subscription"

  3. DNS/负载均衡切换:
    # AWS Route53 加权路由
    aws route53 change-resource-record-sets --hosted-zone-id Z123 \
      --change-batch '{
        "Changes": [{
          "Action": "UPSERT",
          "ResourceRecordSet": {
            "Name": "api.target.com",
            "Type": "CNAME",
            "SetIdentifier": "green",
            "Weight": 100,
            "TTL": 60,
            "ResourceRecords": [{"Value": "green-lb.example.com"}]
          }
        }]
      }'

    # 或 Nginx 上游切换
    # upstream backend { server green-host:8080; }
    # nginx -s reload

  4. 验证切换成功:
    - 实时监控错误率（< 0.1%）
    - 检查请求是否到达 Green 环境
    - 核心业务流程端到端验证

  5. 保留 Blue 环境 7 天作为回滚后备
```

### 3.2 金丝雀发布执行

```yaml
# Kubernetes + Istio 金丝雀示例
流量分配步骤:

阶段 1 - 1% 流量（观察 24h）:
  apiVersion: networking.istio.io/v1beta1
  kind: VirtualService
  metadata:
    name: api-service
  spec:
    hosts:
      - api-service
    http:
      - route:
          - destination:
              host: api-service
              subset: stable
            weight: 99
          - destination:
              host: api-service
              subset: canary
            weight: 1

阶段 2 - 10% 流量（观察 24h）:
  # weight: stable=90, canary=10

阶段 3 - 50% 流量（观察 48h）:
  # weight: stable=50, canary=50

阶段 4 - 100% 流量:
  # weight: stable=0, canary=100
  # 确认稳定后，将 canary 标记为 stable

观察指标（每个阶段）:
  - 错误率对比（canary vs stable）
  - P99 延迟对比
  - 资源利用率
  - 业务指标（转化率/成功率）
  - 用户反馈/客诉

晋级条件:
  - 错误率 canary <= stable × 1.1
  - P99 延迟 canary <= stable × 1.2
  - 无 P0/P1 告警
  - 业务指标无显著下降
```

### 3.3 Strangler Fig 执行

```yaml
核心原理:
  新系统逐步接管老系统的功能，老系统逐步被"绞杀"直到完全退役。
  通过请求路由层（API Gateway / Proxy）控制流量分配。

执行步骤:

阶段 1 - 部署路由层:
  # Nginx 作为路由层示例
  upstream old_system { server old-host:8080; }
  upstream new_user_service { server new-user:8080; }
  upstream new_product_service { server new-product:8080; }

  server {
    listen 80;

    # 已迁移的模块 → 新系统
    location /api/v1/users {
      proxy_pass http://new_user_service;
    }

    # 未迁移的模块 → 老系统
    location / {
      proxy_pass http://old_system;
    }
  }

阶段 2 - 逐模块迁移:
  迁移顺序（按风险/依赖排序）:
    1. 用户模块（依赖少，独立性高）
    2. 商品模块（被订单依赖，需先迁移）
    3. 搜索模块（可独立运行）
    4. 订单模块（核心，最后迁移）
    5. 支付模块（核心，最后迁移）

  每个模块迁移流程:
    a. 新服务开发并通过测试
    b. 数据迁移/同步
    c. 灰度切换流量（1% → 10% → 50% → 100%）
    d. 老模块代码标记为 deprecated
    e. 数据同步反向验证
    f. 确认稳定后移除老模块路由

阶段 3 - 老系统退役:
  - 所有模块迁移完成
  - 老系统保持只读运行 14 天
  - 确认无遗漏流量
  - 下线老系统
  - 清理老系统基础设施
```

### 3.4 数据库迁移执行

```bash
# 零停机数据库迁移示例（PostgreSQL 版本升级 14 → 16）

# 步骤 1: 新实例搭建
# 使用 RDS/CloudSQL 创建新版本实例
aws rds create-db-instance \
  --db-instance-identifier prod-pg16 \
  --engine postgres \
  --engine-version 16.2 \
  --db-instance-class db.r6g.xlarge \
  --allocated-storage 500

# 步骤 2: 建立逻辑复制
# 老库
psql -h old-host -d production -c "
  ALTER SYSTEM SET wal_level = 'logical';
  SELECT pg_reload_conf();
  CREATE PUBLICATION full_migration FOR ALL TABLES;
"

# 新库（先创建相同的表结构）
pg_dump -h old-host -d production --schema-only | psql -h new-host -d production

psql -h new-host -d production -c "
  CREATE SUBSCRIPTION full_migration
    CONNECTION 'host=old-host port=5432 dbname=production user=repl_user password=xxx'
    PUBLICATION full_migration;
"

# 步骤 3: 监控复制状态
watch -n 5 'psql -h new-host -d production -c "
  SELECT subname, received_lsn, latest_end_lsn, latest_end_time
  FROM pg_stat_subscription;
"'

# 步骤 4: 验证数据一致性
# 执行上文的数据校验脚本

# 步骤 5: 切换（维护窗口内）
# a. 应用停止写入（或切为只读模式）
# b. 等待复制追平（lag = 0）
psql -h new-host -c "SELECT * FROM pg_stat_subscription" # 确认无延迟
# c. 应用连接串切换到新库
# d. 验证应用正常
# e. 删除逻辑复制
psql -h new-host -c "DROP SUBSCRIPTION full_migration"
psql -h old-host -c "DROP PUBLICATION full_migration"
```

---

## 四、验证阶段

### 4.1 功能验证

```yaml
验证矩阵:
  冒烟测试（切换后 5 分钟内）:
    - 核心 API 健康检查通过
    - 登录/注册流程正常
    - 核心查询返回正确数据
    - 写入操作正常（创建订单/更新信息）

  回归测试（切换后 1 小时内）:
    - 自动化测试套件全量执行
    - 覆盖所有核心业务用例
    - 第三方集成接口联调验证
    - 定时任务正常触发

  端到端验证（切换后 4 小时内）:
    - 完整业务流程走通（下单→支付→发货→签收）
    - 边界条件测试（大数据量/并发/异常输入）
    - 多端验证（Web/App/小程序/API）
```

### 4.2 性能验证

```bash
# 基准对比测试
# 使用迁移前相同的压测脚本和参数

# k6 压测
k6 run --env TARGET=https://new-api.target.com load-test.js

# 对比关键指标
echo "=== 性能对比 ==="
echo "指标          | 迁移前 | 迁移后 | 差异"
echo "P50 延迟      | 45ms  | ?ms   | "
echo "P99 延迟      | 230ms | ?ms   | "
echo "吞吐量(RPS)   | 5200  | ?     | "
echo "错误率        | 0.02% | ?     | "
echo "CPU 利用率    | 65%   | ?     | "
echo "内存利用率    | 72%   | ?     | "

# 验收标准
# - P99 延迟不超过迁移前的 120%
# - 吞吐量不低于迁移前的 90%
# - 错误率不超过迁移前的 110%
```

### 4.3 数据完整性验证

```bash
# 最终一致性校验（切换后执行）

# 1. 全量行数校验
echo "=== 全量行数校验 ==="
psql -h new-host -d production -c "
  SELECT tablename, n_live_tup
  FROM pg_stat_user_tables
  ORDER BY n_live_tup DESC;
"

# 2. 关键业务数据校验
echo "=== 业务数据校验 ==="
psql -h new-host -d production -c "
  -- 用户总数
  SELECT 'users' as entity, count(*) as total FROM users
  UNION ALL
  -- 活跃订单数
  SELECT 'active_orders', count(*) FROM orders WHERE status NOT IN ('CANCELLED', 'REFUNDED')
  UNION ALL
  -- 商品总数
  SELECT 'products', count(*) FROM products WHERE deleted_at IS NULL
  UNION ALL
  -- 今日交易额
  SELECT 'today_revenue', COALESCE(sum(total_amount), 0)::text::bigint FROM orders WHERE created_at > CURRENT_DATE;
"

# 3. 外键完整性校验
echo "=== 外键完整性 ==="
psql -h new-host -d production -c "
  -- 孤儿订单（user_id 不存在）
  SELECT count(*) as orphan_orders FROM orders o
  WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = o.user_id);

  -- 孤儿支付记录
  SELECT count(*) as orphan_payments FROM payments p
  WHERE NOT EXISTS (SELECT 1 FROM orders o WHERE o.id = p.order_id);
"
```

---

## 五、切换与收尾

### 5.1 切换日 Runbook

```yaml
切换日流程（以蓝绿部署为例）:

T-60min:
  - [ ] 团队全员就位（开发/运维/DBA/产品/客服）
  - [ ] 监控大盘打开（新旧环境并排）
  - [ ] 回滚脚本就绪并测试过
  - [ ] 通知业务方即将切换

T-30min:
  - [ ] 最终数据校验通过
  - [ ] Green 环境健康检查全部通过
  - [ ] 数据同步延迟 < 1 秒

T-0（切换）:
  - [ ] 执行流量切换
  - [ ] 确认流量到达 Green 环境
  - [ ] 冒烟测试通过

T+5min:
  - [ ] 核心 API 错误率 < 0.1%
  - [ ] P99 延迟在正常范围
  - [ ] 无 P0/P1 告警

T+30min:
  - [ ] 自动化回归测试通过
  - [ ] 业务核心指标正常
  - [ ] 通知业务方切换完成

T+24h:
  - [ ] 持续监控无异常
  - [ ] 用户反馈/客诉正常

T+7d:
  - [ ] 旧环境下线确认
  - [ ] 清理临时资源（同步任务/中间件/临时配置）
  - [ ] 更新架构文档/运维手册
  - [ ] 迁移复盘会议
```

### 5.2 善后清理

```bash
# 1. 停止数据同步
psql -h new-host -c "DROP SUBSCRIPTION IF EXISTS full_migration"
psql -h old-host -c "DROP PUBLICATION IF EXISTS full_migration"
psql -h old-host -c "SELECT pg_drop_replication_slot('migration_slot')"

# 2. 清理旧环境 DNS
aws route53 change-resource-record-sets --hosted-zone-id Z123 \
  --change-batch '{
    "Changes": [{
      "Action": "DELETE",
      "ResourceRecordSet": {
        "Name": "old-api.target.com",
        "Type": "CNAME",
        "TTL": 300,
        "ResourceRecords": [{"Value": "old-lb.example.com"}]
      }
    }]
  }'

# 3. 旧环境资源回收（确认 7 天无回滚需求后）
# Kubernetes
kubectl delete namespace old-production
# 或 AWS
aws rds delete-db-instance --db-instance-identifier prod-pg14-old --skip-final-snapshot
aws ec2 terminate-instances --instance-ids i-old1 i-old2 i-old3

# 4. 更新配置管理
# - 移除旧环境的监控告警
# - 更新 CI/CD 管道
# - 更新内部 Wiki/文档
```

---

## 六、回滚

### 6.1 回滚决策标准

```yaml
立即回滚（任一触发）:
  - 核心 API 错误率 > 5% 持续 5 分钟
  - P99 延迟 > 迁移前 3 倍持续 10 分钟
  - 数据不一致被确认（丢数据/脏数据）
  - P0 告警且 15 分钟内无法修复

考虑回滚:
  - 核心 API 错误率 1%-5% 持续 15 分钟
  - P99 延迟 > 迁移前 2 倍持续 30 分钟
  - 非核心功能异常影响用户体验
  - 第三方集成方报告异常

不回滚（现场修复）:
  - 错误率 < 1% 且可快速定位
  - 性能略有下降但在 SLA 内
  - 非核心功能的已知兼容问题
```

### 6.2 回滚步骤

```bash
# 蓝绿部署回滚（秒级）
# 将流量切回 Blue 环境
aws route53 change-resource-record-sets --hosted-zone-id Z123 \
  --change-batch '{
    "Changes": [{
      "Action": "UPSERT",
      "ResourceRecordSet": {
        "Name": "api.target.com",
        "Type": "CNAME",
        "SetIdentifier": "blue",
        "Weight": 100,
        "TTL": 60,
        "ResourceRecords": [{"Value": "blue-lb.example.com"}]
      }
    }]
  }'

# 金丝雀回滚
kubectl apply -f - <<EOF
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: api-service
spec:
  hosts:
    - api-service
  http:
    - route:
        - destination:
            host: api-service
            subset: stable
          weight: 100
        - destination:
            host: api-service
            subset: canary
          weight: 0
EOF

# 数据库回滚（如已切换写入到新库）
# 1. 应用切回老库连接串
# 2. 将新库中的增量数据同步回老库
# 3. 如数据量少，可手动 SQL 补齐
# 4. 如数据量大，需建立反向逻辑复制

# Strangler Fig 回滚（模块级别）
# 将路由规则中该模块指回老系统
sed -i 's|proxy_pass http://new_user_service|proxy_pass http://old_system|' /etc/nginx/conf.d/migration.conf
nginx -s reload
```

### 6.3 回滚后处理

```yaml
回滚后必做:
  1. 确认回滚成功:
     - 流量已回到旧环境
     - 核心指标恢复正常
     - 数据一致性确认

  2. 根因分析:
     - 收集迁移期间的日志与监控数据
     - 定位失败原因
     - 评估修复工作量

  3. 制定修复计划:
     - 修复根因
     - 更新迁移方案
     - 安排第二次迁移窗口

  4. 复盘与改进:
     - 回滚原因归档
     - 更新迁移检查清单
     - 团队知识分享
```

---

## Agent Checklist

供自动化 Agent 在执行系统迁移流程时逐项核查：

- [ ] 迁移目标与成功标准已明确定义
- [ ] 现状梳理完成（服务/数据/接口/基础设施）
- [ ] 迁移复杂度已评估
- [ ] 迁移策略已选择（蓝绿/金丝雀/Strangler Fig/大爆炸）
- [ ] 迁移计划已制定（里程碑/时间线/责任人）
- [ ] 数据迁移方案已确定（全量+增量）
- [ ] 数据校验方案已准备（行数/校验和/业务指标）
- [ ] 目标环境已搭建并通过验证
- [ ] 全量数据迁移已完成
- [ ] 增量同步已建立且延迟可接受
- [ ] 功能回归测试已通过
- [ ] 性能基准测试已通过
- [ ] 回滚方案已演练
- [ ] 切换日 Runbook 已制定
- [ ] 流量切换已按计划执行
- [ ] 切换后冒烟测试通过
- [ ] 切换后数据完整性验证通过
- [ ] 切换后 24h 持续监控无异常
- [ ] 旧环境已在回滚窗口后安全下线
- [ ] 迁移复盘已完成并归档