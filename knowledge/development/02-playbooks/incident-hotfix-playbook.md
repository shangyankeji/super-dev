---
id: incident-hotfix-playbook
title: 线上故障热修复手册 (Incident Hotfix Playbook)
domain: development
category: 02-playbooks
difficulty: intermediate
tags: [development, hotfix, incident, playbook, 修复方案, 前置条件, 基本信息, 时间线]
quality_score: 70
last_updated: 2026-06-15
---
# 线上故障热修复手册 (Incident Hotfix Playbook)

## 概述

线上故障热修复是在生产环境出现服务异常时，以最快速度恢复服务、最小化业务损失的紧急响应流程。本手册覆盖从故障发现、定级响应、紧急修复到复盘防复发的完整链路，确保每次热修复都可控、可追溯、可回滚。

## 前置条件

### 平时准备（故障发生前必须就绪）

- [ ] 核心服务有健康检查端点和自动告警
- [ ] 具备一键回滚能力（版本回退或流量切换）
- [ ] On-call 轮值表和升级路径已定义并通知到人
- [ ] 关键服务的 Runbook 已就绪（至少涵盖 Top 10 故障场景）
- [ ] 日志、指标、链路追踪三大可观测性支柱已部署
- [ ] 数据库有定期备份和恢复测试记录

### 工具准备

- [ ] 告警通知渠道（钉钉/企微/Slack/PagerDuty）
- [ ] 远程访问生产环境的安全通道（VPN/堡垒机）
- [ ] 故障响应协作频道（专用 War Room）

---

## 步骤一：故障发现与定级

### 1.1 发现渠道

| 渠道 | 响应时间要求 | 负责人 |
|------|-------------|--------|
| 监控告警自动触发 | 1 分钟内确认 | On-call 值班 |
| 用户报障（客服/工单） | 5 分钟内确认 | 客服 -> On-call |
| 内部发现（开发/测试） | 3 分钟内确认 | 发现人 -> On-call |

### 1.2 故障定级

```
P0 - 核心业务完全不可用（支付、登录、核心交易链路中断）
     响应要求：5 分钟内启动 War Room，15 分钟内出止损方案
     通知范围：CTO + 技术总监 + 全部相关团队

P1 - 核心业务严重降级（部分功能不可用、性能严重下降）
     响应要求：15 分钟内启动响应，30 分钟内出止损方案
     通知范围：技术总监 + 服务负责人

P2 - 非核心功能异常（边缘功能故障、非关键页面报错）
     响应要求：30 分钟内响应，2 小时内修复
     通知范围：服务负责人

P3 - 轻微异常（日志报错但不影响用户、数据轻微偏差）
     响应要求：下一个工作日内处理
     通知范围：开发团队
```

### 1.3 启动 War Room

```bash
# 创建故障记录
cat > incident-$(date +%Y%m%d%H%M).md << 'EOF'
# 故障记录

**时间**: YYYY-MM-DD HH:MM
**定级**: P0/P1/P2/P3
**现象**: [简述故障表现]
**影响范围**: [受影响的用户群体和业务功能]
**当前状态**: 响应中 / 止损中 / 修复中 / 观察中 / 已关闭

## 时间线
| 时间 | 事件 | 操作人 |
|------|------|--------|

## 根因
[待分析]

## 修复方案
[待确定]
EOF
```

---

## 步骤二：止损（最高优先级）

### 2.1 止损决策树

```
故障发生
├── 最近有发布？
│   ├── 是 → 立即回滚到上一个稳定版本
│   └── 否 → 继续排查
├── 流量突增导致？
│   ├── 是 → 开启限流/降级/熔断
│   └── 否 → 继续排查
├── 第三方依赖故障？
│   ├── 是 → 切换备用方案/开启缓存兜底
│   └── 否 → 继续排查
├── 数据异常？
│   ├── 是 → 暂停写入，保护现有数据
│   └── 否 → 继续排查
└── 未知原因？
    └── 先降级非核心功能，保护核心链路
```

### 2.2 常用止损命令

```bash
# 版本回滚 - Kubernetes
kubectl rollout undo deployment/<service-name> -n production
kubectl rollout status deployment/<service-name> -n production

# 版本回滚 - Docker Compose
docker-compose -f docker-compose.prod.yml pull <service>:<previous-tag>
docker-compose -f docker-compose.prod.yml up -d <service>

# 流量切换 - Nginx
# 切换到备用上游
sed -i 's/upstream_primary/upstream_backup/' /etc/nginx/conf.d/upstream.conf
nginx -t && nginx -s reload

# 开启限流 - Redis 令牌桶
redis-cli SET rate_limit:api:threshold 100  # 每秒100次
redis-cli SET rate_limit:api:enabled true

# 开启降级 - 特性开关
curl -X PUT http://feature-flags/api/flags/non_critical_features \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'

# 数据库紧急操作 - 阻止有害写入
# 只在确认有害 SQL 时使用
mysql -e "SHOW PROCESSLIST;" | grep "harmful_query_pattern"
mysql -e "KILL <process_id>;"
```

### 2.3 止损验证

```bash
# 验证服务恢复
for i in {1..10}; do
  STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://api.example.com/health)
  echo "Health check $i: $STATUS"
  sleep 2
done

# 验证核心链路
curl -s http://api.example.com/api/v1/orders/test | jq '.status'

# 验证错误率下降
curl -s 'http://prometheus:9090/api/v1/query?query=rate(http_errors_total{service="target"}[1m])' | jq '.data.result[0].value[1]'
```

---

## 步骤三：根因分析

### 3.1 日志排查

```bash
# 查看错误日志 - Kubernetes
kubectl logs -n production deployment/<service> --since=30m | grep -i error | tail -50

# 按时间范围查看 - 使用 stern 多 Pod 聚合
stern <service> -n production --since 30m | grep -E "(ERROR|FATAL|panic)"

# Elasticsearch 日志查询
curl -s 'http://elasticsearch:9200/logs-*/_search' -H 'Content-Type: application/json' -d '{
  "query": {
    "bool": {
      "must": [
        {"range": {"@timestamp": {"gte": "now-30m"}}},
        {"match": {"level": "ERROR"}}
      ]
    }
  },
  "sort": [{"@timestamp": "desc"}],
  "size": 50
}'
```

### 3.2 指标分析

```bash
# 查看关键指标变化
# 错误率飙升时间点
curl -s 'http://prometheus:9090/api/v1/query_range?query=rate(http_errors_total[1m])&start=2024-01-01T00:00:00Z&end=2024-01-01T01:00:00Z&step=60s'

# CPU/内存使用率
curl -s 'http://prometheus:9090/api/v1/query?query=container_cpu_usage_seconds_total{pod=~"service.*"}'

# 数据库连接池
curl -s 'http://prometheus:9090/api/v1/query?query=db_pool_active_connections{service="target"}'
```

### 3.3 链路追踪

```bash
# 查找慢请求的完整链路 (Jaeger)
curl -s "http://jaeger:16686/api/traces?service=<service>&operation=<endpoint>&minDuration=5s&limit=10"

# 通过 trace ID 查看完整链路
curl -s "http://jaeger:16686/api/traces/<trace-id>"
```

### 3.4 五个为什么 (5 Whys)

```markdown
**现象**: API 返回 500 错误

**Why 1**: 数据库查询超时
**Why 2**: 慢查询未命中索引
**Why 3**: 新版本代码改变了查询条件，绕过了已有索引
**Why 4**: Code Review 未覆盖查询计划检查
**Why 5**: 没有查询性能的自动化门禁

**根因**: 缺少查询性能自动化检测机制
**防复发**: 在 CI 中加入慢查询检测 + 预发布环境自动执行 EXPLAIN
```

---

## 步骤四：修复与发布

### 4.1 热修复分支策略

```bash
# 从生产分支创建热修复分支
git checkout main
git pull origin main
git checkout -b hotfix/incident-20240101-db-timeout

# 实施最小修复
# 原则：只修复问题本身，不做额外改进

# 提交
git add -A
git commit -m "hotfix: add missing index for order query timeout

Incident: INC-2024-001
Root cause: new query pattern bypassed existing index
Fix: add composite index on (user_id, created_at, status)"

# 推送并创建 PR
git push origin hotfix/incident-20240101-db-timeout
gh pr create --title "hotfix: fix order query timeout" \
  --body "Incident: INC-2024-001" \
  --base main --label "hotfix"
```

### 4.2 热修复审查原则

- 审查范围：仅限于修复相关的变更
- 审查时间：P0/P1 可以先发布后补审，P2/P3 必须先审后发
- 审查人数：至少一人，P0 可由 On-call 负责人自审
- 必须验证：回滚可行性

### 4.3 发布验证

```bash
# 发布热修复
kubectl set image deployment/<service> <container>=<image>:<hotfix-tag> -n production

# 逐步验证
echo "Step 1: 健康检查"
kubectl rollout status deployment/<service> -n production

echo "Step 2: 功能验证"
curl -s http://api.example.com/api/v1/orders?user_id=test | jq '.status'

echo "Step 3: 性能验证"
curl -s 'http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99,rate(http_duration_seconds_bucket{handler="/api/v1/orders"}[5m]))' | jq '.data.result[0].value[1]'

echo "Step 4: 错误率验证"
curl -s 'http://prometheus:9090/api/v1/query?query=rate(http_errors_total{handler="/api/v1/orders"}[5m])' | jq '.data.result[0].value[1]'
```

---

## 步骤五：复盘与防复发

### 5.1 复盘模板

```markdown
# 故障复盘报告

## 基本信息
- 故障编号: INC-YYYY-NNN
- 发生时间: YYYY-MM-DD HH:MM ~ HH:MM
- 影响时长: X 分钟
- 影响范围: [用户数/交易量/业务功能]
- 故障等级: P0/P1/P2/P3

## 时间线
| 时间 | 事件 | 操作人 |
|------|------|--------|
| HH:MM | 告警触发 | 系统 |
| HH:MM | On-call 确认 | 张三 |
| HH:MM | 止损完成 | 张三 |
| HH:MM | 根因确认 | 李四 |
| HH:MM | 修复发布 | 张三 |
| HH:MM | 全面恢复确认 | 张三 |

## 根因分析
[使用 5 Whys 方法]

## 修复方案
[说明修复内容]

## 防复发措施
| 措施 | 负责人 | 完成时间 | 状态 |
|------|--------|---------|------|
| 添加索引性能门禁 | 张三 | 2024-01-15 | 待完成 |
| 补充慢查询监控 | 李四 | 2024-01-10 | 待完成 |

## 经验教训
[团队应学到什么]
```

### 5.2 24 小时内必须完成

- [ ] 热修复代码已合并到 main 和 develop 分支
- [ ] 补充回归测试覆盖此次故障场景
- [ ] 更新 Runbook（如果是新的故障模式）
- [ ] 提交复盘报告
- [ ] 防复发措施录入任务追踪系统

---

## 回滚方案

### 热修复回滚

```bash
# 如果热修复引入了新问题，立即回滚
kubectl rollout undo deployment/<service> -n production

# 验证回滚成功
kubectl rollout status deployment/<service> -n production
curl -s http://api.example.com/health | jq '.status'
```

### 数据回滚

```bash
# 如果热修复涉及数据变更，先备份再操作
pg_dump -h db-host -U user -d dbname -t affected_table > backup_before_hotfix.sql

# 数据修复后如需回滚
psql -h db-host -U user -d dbname < backup_before_hotfix.sql
```

### 回滚触发条件

| 场景 | 阈值 | 处理 |
|------|------|------|
| 热修复后错误率上升 | > 修复前水平 | 立即回滚热修复 |
| 热修复后出现新错误类型 | 任意新 5xx | 立即回滚热修复 |
| 热修复后性能恶化 | P99 > 修复前 2 倍 | 回滚并重新分析 |

---

## Agent Checklist

AI 编码 Agent 在协助故障修复时必须逐项确认：

- [ ] **故障定级完成**：已明确 P0/P1/P2/P3 级别和影响范围
- [ ] **止损优先**：在深入分析前，先确保止损措施到位
- [ ] **最小修复原则**：热修复仅修复故障本身，不做额外改进
- [ ] **回滚可行性**：修复方案有明确的回滚步骤和验证方法
- [ ] **数据安全**：涉及数据变更时先备份
- [ ] **不跳过审查**：P0 可先发后审，但不能完全跳过
- [ ] **测试覆盖**：热修复后 24 小时内补充针对性测试
- [ ] **分支合并**：热修复代码已同步到所有活跃分支
- [ ] **复盘提交**：故障复盘报告在 48 小时内完成
- [ ] **防复发跟踪**：所有防复发措施有负责人和截止时间
- [ ] **Runbook 更新**：新故障模式已补充到 Runbook
- [ ] **告警优化**：如果此次故障告警不及时，已优化告警规则
