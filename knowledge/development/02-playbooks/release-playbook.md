---
id: release-playbook
title: 发布作战手册 (Release Playbook)
domain: development
category: 02-playbooks
difficulty: intermediate
tags: [2024-01-15, agent, checklist, development, playbook, release, 前置条件, 发布清单]
quality_score: 70
last_updated: 2026-06-15
---
# 发布作战手册 (Release Playbook)

## 概述

发布是将经过验证的代码安全地部署到生产环境的过程。本手册覆盖从发布准备、质量门禁、灰度放量、监控观测到回滚恢复的完整流程。适用于常规发布、紧急发布和大版本发布三种场景。

## 前置条件

### 必须满足

- [ ] 所有功能开发已完成并通过 Code Review
- [ ] CI 流水线全部通过（Lint、测试、安全扫描）
- [ ] 测试报告已生成，测试通过率 >= 99%
- [ ] 已确认无阻断级别的已知 Bug
- [ ] 回滚方案已准备并经过验证
- [ ] 发布窗口已预约（避免高峰时段）
- [ ] On-call 值班人员已确认

### 建议满足

- [ ] 性能测试已通过
- [ ] 安全扫描无高危漏洞
- [ ] 数据库迁移脚本已在预发布环境验证
- [ ] 变更通知已发送给相关方

---

## 步骤一：发布准备

### 1.1 版本号管理

```bash
# 语义版本号规则
# MAJOR.MINOR.PATCH
# MAJOR: 不兼容的 API 变更
# MINOR: 向后兼容的功能新增
# PATCH: 向后兼容的 Bug 修复

# 查看当前版本
git describe --tags --abbrev=0

# 创建发布分支
git checkout develop
git pull origin develop
git checkout -b release/v1.2.0

# 更新版本号
# Python
sed -i 's/version = ".*"/version = "1.2.0"/' pyproject.toml
# Node.js
npm version 1.2.0 --no-git-tag-version
# 手动更新其他版本引用
```

### 1.2 Changelog 生成

```bash
# 基于 Conventional Commits 自动生成
# 安装: pip install git-changelog
git-changelog --output CHANGELOG.md --style conventional

# 或手动整理
cat >> CHANGELOG.md << 'EOF'

## [1.2.0] - 2024-01-15

### Added
- 新增订单导出功能 (#123)
- 新增批量操作接口 (#145)

### Changed
- 优化列表查询性能，P99 降低 40% (#156)
- 升级 Redis 客户端到 v5.0 (#160)

### Fixed
- 修复并发场景下的库存扣减问题 (#167)
- 修复分页查询的 off-by-one 错误 (#170)

### Security
- 升级依赖修复 CVE-2024-XXXX (#175)
EOF
```

### 1.3 发布清单

```markdown
## 发布清单 v1.2.0

### 代码
- [ ] release 分支已从 develop 创建
- [ ] 版本号已更新
- [ ] CHANGELOG 已更新
- [ ] 无未合并的 hotfix

### 质量
- [ ] 单元测试通过率 100%
- [ ] 集成测试通过率 >= 99%
- [ ] 代码覆盖率 >= 80%
- [ ] 静态分析无新增告警
- [ ] 安全扫描无高危/严重漏洞

### 数据库
- [ ] 迁移脚本已编写并测试
- [ ] 迁移支持回滚
- [ ] 预发布环境已验证迁移
- [ ] 大表变更有预估执行时间

### 基础设施
- [ ] Docker 镜像构建成功
- [ ] 配置变更已同步到各环境
- [ ] 新增的环境变量/Secret 已配置
- [ ] 资源配额已确认（CPU/内存/存储）

### 监控
- [ ] 核心指标告警已配置
- [ ] 新功能的监控指标已添加
- [ ] Dashboard 已更新

### 通知
- [ ] 发布计划已通知团队
- [ ] 影响范围已通知下游
- [ ] On-call 人员已确认
```

---

## 步骤二：质量门禁

### 2.1 自动化门禁

```bash
#!/bin/bash
# release_gate_check.sh - 发布前自动检查

set -e
PASS=0
FAIL=0

check() {
    local name=$1
    local cmd=$2
    echo -n "[$name] "
    if eval "$cmd" > /dev/null 2>&1; then
        echo "PASS"
        ((PASS++))
    else
        echo "FAIL"
        ((FAIL++))
    fi
}

echo "=== 发布门禁检查 ==="

check "Lint" "ruff check src/"
check "Type Check" "mypy src/"
check "Unit Tests" "pytest tests/unit/ -q"
check "Integration Tests" "pytest tests/integration/ -q"
check "Security Scan" "trivy image app:latest --severity HIGH,CRITICAL --exit-code 1"
check "License Check" "pip-licenses --fail-on 'GPL;AGPL'"
check "Docker Build" "docker build -t app:release-candidate ."

echo ""
echo "=== 结果: $PASS 通过, $FAIL 失败 ==="

if [ $FAIL -gt 0 ]; then
    echo "门禁未通过，不允许发布"
    exit 1
fi
echo "门禁通过，可以发布"
```

### 2.2 手动检查项

```markdown
### 发布前人工检查

- [ ] 在预发布环境完整走通核心用户旅程
- [ ] 确认配置文件差异（staging vs production）
- [ ] 确认第三方服务状态正常
- [ ] 确认没有正在进行的大批量数据处理
- [ ] 确认发布窗口没有冲突的其他变更
```

---

## 步骤三：执行发布

### 3.1 数据库迁移

```bash
# 先在生产环境执行数据库迁移
# 原则：迁移必须向后兼容（旧代码能跑新 schema）

# 备份当前数据库
pg_dump -h db-host -U user -d production > backup_pre_release_v120.sql

# 执行迁移
python manage.py migrate --database production

# 验证迁移
python manage.py showmigrations --database production | grep -E "^\[X\]"

# 如果迁移失败，回滚
python manage.py migrate <app> <previous_migration> --database production
```

### 3.2 灰度发布

```bash
# Kubernetes 灰度部署

# 阶段 1: 金丝雀 (5%)
kubectl apply -f - << 'EOF'
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app-canary
  namespace: production
spec:
  replicas: 1  # 生产环境有 19 个副本，1 个金丝雀 = ~5%
  selector:
    matchLabels:
      app: myapp
      track: canary
  template:
    metadata:
      labels:
        app: myapp
        track: canary
    spec:
      containers:
      - name: app
        image: app:v1.2.0
        resources:
          requests:
            cpu: "500m"
            memory: "512Mi"
          limits:
            cpu: "1000m"
            memory: "1Gi"
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
EOF

echo "金丝雀已部署，观察 30 分钟..."
```

```bash
# 阶段 2: 验证金丝雀
# 持续监控金丝雀指标
for i in $(seq 1 30); do
    echo "=== 检查 $i/30 (每分钟一次) ==="

    # 错误率
    ERROR_RATE=$(curl -s "http://prometheus:9090/api/v1/query?query=rate(http_errors_total{track='canary'}[5m])" | jq -r '.data.result[0].value[1] // "0"')
    echo "金丝雀错误率: $ERROR_RATE"

    # P99 延迟
    P99=$(curl -s "http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99,rate(http_duration_seconds_bucket{track='canary'}[5m]))" | jq -r '.data.result[0].value[1] // "0"')
    echo "金丝雀 P99: ${P99}s"

    sleep 60
done
```

```bash
# 阶段 3: 全量发布
kubectl set image deployment/app app=app:v1.2.0 -n production
kubectl rollout status deployment/app -n production

# 删除金丝雀
kubectl delete deployment app-canary -n production
```

### 3.3 发布类型矩阵

| 类型 | 灰度策略 | 观察时间 | 回滚授权 |
|------|----------|---------|---------|
| 常规发布 | 5% -> 25% -> 50% -> 100% | 每阶段 30 分钟 | 发布负责人 |
| 大版本发布 | 1% -> 5% -> 25% -> 50% -> 100% | 每阶段 2 小时 | 技术总监 |
| 紧急发布 | 直接 100%（已验证的 hotfix） | 15 分钟 | On-call 负责人 |
| 配置变更 | 特性开关渐进 | 即时 | 发布负责人 |

---

## 步骤四：发布后观测

### 4.1 核心指标监控

```bash
#!/bin/bash
# post_release_monitor.sh

echo "=== 发布后监控 (每 5 分钟检查一次，持续 2 小时) ==="

BASELINE_ERROR_RATE=0.001
BASELINE_P99=0.2

for i in $(seq 1 24); do
    echo ""
    echo "--- 检查 $i/24 ---"
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

    # 错误率
    ERROR_RATE=$(curl -s "http://prometheus:9090/api/v1/query?query=rate(http_errors_total[5m])" | jq -r '.data.result[0].value[1] // "0"')

    # P99
    P99=$(curl -s "http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99,rate(http_duration_seconds_bucket[5m]))" | jq -r '.data.result[0].value[1] // "0"')

    # QPS
    QPS=$(curl -s "http://prometheus:9090/api/v1/query?query=rate(http_requests_total[5m])" | jq -r '.data.result[0].value[1] // "0"')

    echo "[$TIMESTAMP] 错误率=$ERROR_RATE P99=${P99}s QPS=$QPS"

    # 异常检测
    if (( $(echo "$ERROR_RATE > $BASELINE_ERROR_RATE * 2" | bc -l) )); then
        echo "!!! 警告: 错误率超过基线 2 倍 !!!"
    fi

    sleep 300
done

echo "=== 观察完成 ==="
```

### 4.2 业务指标确认

```markdown
### 发布后业务检查

- [ ] 核心交易量正常（与上周同期对比偏差 < 10%）
- [ ] 用户登录成功率正常
- [ ] 支付成功率正常
- [ ] 搜索结果正常
- [ ] 无异常客服工单上升
```

### 4.3 发布确认

```bash
# 发布成功后的收尾
# 1. 合并 release 分支
git checkout main
git merge --no-ff release/v1.2.0
git tag -a v1.2.0 -m "Release v1.2.0"
git push origin main --tags

# 2. 同步到 develop
git checkout develop
git merge --no-ff release/v1.2.0
git push origin develop

# 3. 删除 release 分支
git branch -d release/v1.2.0
git push origin --delete release/v1.2.0

# 4. 更新发布记录
gh release create v1.2.0 --title "v1.2.0" --notes-file CHANGELOG.md
```

---

## 步骤五：发布后清理

### 5.1 资源清理

```bash
# 清理旧版本镜像（保留最近 5 个版本）
docker images app --format "{{.Tag}}" | sort -V | head -n -5 | xargs -I {} docker rmi app:{}

# 清理预发布环境
kubectl delete deployment app-canary -n production 2>/dev/null || true
```

### 5.2 文档更新

- [ ] API 文档已发布新版本
- [ ] 用户文档/帮助中心已更新
- [ ] 内部 Wiki 已更新
- [ ] 发布邮件/通知已发送

---

## 回滚方案

### 快速回滚

```bash
# Kubernetes 一键回滚
kubectl rollout undo deployment/app -n production

# 验证回滚
kubectl rollout status deployment/app -n production

# 确认服务恢复
for endpoint in /health /api/v1/status; do
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" "http://api.example.com$endpoint")
    echo "$endpoint: $STATUS"
done
```

### 数据库回滚

```bash
# 回滚迁移
python manage.py migrate <app> <previous_migration> --database production

# 如果需要数据修复
psql -h db-host -U user -d production < backup_pre_release_v120.sql
```

### 回滚决策矩阵

| 信号 | 阈值 | 动作 | 决策者 |
|------|------|------|--------|
| 5xx 错误率 | > 1% | 立即回滚 | 自动 |
| P99 延迟 | > 基线 3 倍 | 立即回滚 | 自动 |
| 核心功能异常 | 任何 | 立即回滚 | On-call |
| 非核心功能异常 | 影响 > 5% 用户 | 评估后回滚 | 发布负责人 |
| 数据异常 | 任何数据不一致 | 立即回滚 + 数据修复 | 技术总监 |

### 回滚后处理

```bash
# 1. 确认回滚成功
kubectl get pods -n production -l app=myapp
curl -s http://api.example.com/health | jq '.'

# 2. 通知相关方
echo "v1.2.0 已回滚至 v1.1.x，原因: [填写原因]"

# 3. 创建故障报告
# 参考 incident-hotfix-playbook.md

# 4. 修复后重新走发布流程
```

---

## Agent Checklist

AI 编码 Agent 在协助发布时必须逐项确认：

- [ ] **版本号正确**：符合语义版本规范，所有引用处已同步
- [ ] **Changelog 完整**：所有变更已记录，格式规范
- [ ] **质量门禁通过**：Lint、测试、安全扫描全部绿色
- [ ] **迁移已验证**：数据库迁移在预发布环境成功执行
- [ ] **回滚已准备**：回滚脚本存在且经过验证
- [ ] **灰度配置就绪**：金丝雀部署配置正确
- [ ] **监控已配置**：核心指标告警和 Dashboard 就绪
- [ ] **发布窗口确认**：避开高峰时段，On-call 已确认
- [ ] **观测进行中**：发布后持续监控至少 2 小时
- [ ] **分支已合并**：release 分支已合并到 main 和 develop
- [ ] **Tag 已创建**：Git tag 和 GitHub Release 已创建
- [ ] **通知已发送**：发布结果已通知所有相关方
