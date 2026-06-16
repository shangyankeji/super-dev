---
id: release-management-playbook
title: 发布管理作战手册 (Release Management Playbook)
domain: cicd
category: 02-playbooks
difficulty: intermediate
tags: [branching, cicd, management, playbook, release, strategy, versioning, 前置条件]
quality_score: 70
last_updated: 2026-06-15
---
# 发布管理作战手册 (Release Management Playbook)

## 概述

发布管理是将已验证的软件变更安全、可靠地交付到生产环境的系统化流程。本手册覆盖从版本策略到灰度发布、回滚操作、Changelog 生成和通知流程的完整链路。适用于需要定期发布的中大型团队（发布频率 >= 每周一次）。

## 前置条件

### 必须满足

- [ ] CI/CD 流水线已就绪（构建/测试/部署自动化）
- [ ] 制品仓库已配置（Docker Registry / Nexus / Artifactory）
- [ ] 环境隔离已就绪（dev / staging / production）
- [ ] Git 分支策略已确定并全员知晓
- [ ] 回滚机制已验证

### 建议满足

- [ ] 功能开关（Feature Flag）基础设施已就绪
- [ ] 灰度发布能力已验证
- [ ] 监控和告警覆盖核心业务指标
- [ ] Changelog 自动生成工具已集成

---

## 阶段一：版本策略 (Versioning)

### 1.1 语义化版本 (SemVer)

```
MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]

示例：
  2.1.5          - 正式版本
  2.2.0-beta.1   - 预发布版本
  2.2.0-rc.1     - 候选版本
  2.2.0+build.42 - 带构建元数据

递增规则：
  MAJOR: 不兼容的 API 变更（破坏性变更）
  MINOR: 向后兼容的功能新增
  PATCH: 向后兼容的缺陷修复
```

### 1.2 版本号管理规范

```yaml
version_policy:
  source_of_truth: "pyproject.toml / package.json 中的 version 字段"

  rules:
    - 版本号只在发布流程中变更（不允许随意修改）
    - 预发布版本：X.Y.Z-beta.N / X.Y.Z-rc.N
    - 正式版本 Tag 格式：v{MAJOR}.{MINOR}.{PATCH}
    - Tag 必须是签名的（git tag -s）
    - 一旦发布的版本号不可复用

  automation:
    # 版本号自动提取
    - "CI 从 Tag 提取版本号，注入构建"
    - "构建产物以版本号命名: app-v2.1.5.tar.gz"
    - "Docker 镜像以版本号标记: app:2.1.5"

  pre_release_flow:
    - "开发完成 -> X.Y.Z-beta.1 -> 内部测试"
    - "测试通过 -> X.Y.Z-rc.1 -> 预发布环境验证"
    - "验证通过 -> X.Y.Z -> 正式发布"
```

### 1.3 多组件版本协调

```markdown
## 版本矩阵示例

| 组件 | 版本 | 兼容范围 | 备注 |
|------|------|---------|------|
| Frontend | 2.3.0 | API >= 2.1.0 | 新增 Dashboard 页面 |
| Backend API | 2.1.5 | DB Schema >= 45 | 修复订单查询 |
| Database Schema | 45 | - | 新增 orders 索引 |
| Mobile App | 1.8.0 | API >= 2.0.0 | 需强制更新 |

## 版本兼容性规则
- API 变更必须向后兼容至少 2 个 MINOR 版本
- 破坏性变更需要提前 1 个版本标记 Deprecated
- 数据库 Schema 变更必须可回滚（至少保留一个版本）
```

---

## 阶段二：分支策略 (Branching Strategy)

### 2.1 Git Flow

```
适用场景：发布周期较长（>= 2 周）、需要维护多个版本

main ─────────────────────────────────────────── 生产代码
  │
  ├── release/2.1 ──── 发布分支（冻结功能，只修 Bug）
  │     │
  │     └── hotfix/fix-payment ──── 紧急修复
  │
  └── develop ─────────────────────────────────── 开发集成
        │
        ├── feature/user-dashboard ──── 功能分支
        ├── feature/order-export ──────── 功能分支
        └── feature/notification ──────── 功能分支

分支生命周期：
  feature/* : develop 创建，develop 合并，合并后删除
  release/* : develop 创建，main + develop 合并，合并后删除
  hotfix/*  : main 创建，main + develop 合并，合并后删除
```

### 2.2 Trunk-Based Development

```
适用场景：发布频率高（每日多次）、团队 CI/CD 成熟度高

main ──┬──┬──┬──┬──┬──┬──── 持续集成，随时可发布
       │  │  │  │  │  │
       │  │  │  │  │  └── feat-c (短生命周期 < 2 天)
       │  │  │  │  └───── feat-b
       │  │  │  └──────── feat-a
       │  │  └─────────── bugfix-x
       │  └────────────── feat-d
       └───────────────── release/2.1 (仅用于发布冻结)

原则：
  - 主干是唯一的长期分支
  - 功能分支生命周期 < 2 天
  - 每次提交通过 CI 后即可合并
  - 使用 Feature Flag 控制未完成功能
  - Release 分支只在需要时创建（发布冻结期）
```

### 2.3 分支策略选择指南

| 因素 | Git Flow | Trunk-Based |
|------|----------|-------------|
| 发布频率 | 每 2-4 周 | 每天或每周 |
| 团队规模 | 10+ 开发者 | 5-15 开发者 |
| CI/CD 成熟度 | 中等 | 高 |
| 测试自动化 | 部分自动化 | 高度自动化 |
| 多版本维护 | 需要 | 不需要 |
| 合并冲突频率 | 较高 | 较低 |
| 适合类型 | 移动端/桌面端/嵌入式 | Web 应用/SaaS |

---

## 阶段三：发布流程

### 3.1 构建 (Build)

```yaml
# 构建阶段流水线
build_stage:
  triggers:
    - tag: "v*"                    # Tag 触发正式构建
    - branch: "release/*"          # Release 分支触发预发布构建

  steps:
    - name: "版本号注入"
      run: |
        VERSION=$(git describe --tags --always)
        echo "VERSION=${VERSION}" >> $GITHUB_ENV

    - name: "代码质量检查"
      run: |
        ruff check .
        mypy src/

    - name: "单元测试"
      run: pytest tests/unit/ --tb=short -q

    - name: "构建制品"
      run: |
        docker build \
          --build-arg VERSION=${VERSION} \
          --build-arg BUILD_TIME=$(date -u +%Y-%m-%dT%H:%M:%SZ) \
          --build-arg GIT_COMMIT=$(git rev-parse --short HEAD) \
          -t app:${VERSION} .

    - name: "安全扫描"
      run: trivy image --severity HIGH,CRITICAL app:${VERSION}

    - name: "推送制品"
      run: |
        docker tag app:${VERSION} registry.example.com/app:${VERSION}
        docker push registry.example.com/app:${VERSION}

    - name: "签名制品"
      run: cosign sign registry.example.com/app:${VERSION}
```

### 3.2 测试 (Test)

```yaml
# 测试阶段分层
test_layers:
  unit_tests:
    trigger: "每次提交"
    timeout: "5 分钟"
    coverage_threshold: 80%
    blocking: true

  integration_tests:
    trigger: "PR 合并到 develop/main"
    timeout: "15 分钟"
    scope: "API 契约 + 数据库交互 + 消息队列"
    blocking: true

  e2e_tests:
    trigger: "部署到 staging 后"
    timeout: "30 分钟"
    scope: "核心业务流程（注册->下单->支付->退款）"
    blocking: true

  performance_tests:
    trigger: "Release 分支 / 周度"
    timeout: "60 分钟"
    scope: "基线对比（P95 延迟不超过上版本 110%）"
    blocking: false  # 非阻断，但异常需评审

  security_tests:
    trigger: "Release 分支"
    timeout: "30 分钟"
    scope: "SAST + DAST + 依赖漏洞扫描"
    blocking: true   # 高危漏洞阻断发布
```

### 3.3 灰度发布 (Canary Release)

```yaml
# Kubernetes Canary 部署示例（使用 Argo Rollouts）
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: api-server
spec:
  replicas: 10
  strategy:
    canary:
      steps:
      - setWeight: 5              # 5% 流量到新版本
      - pause: { duration: 5m }   # 观察 5 分钟
      - analysis:                  # 自动分析指标
          templates:
          - templateName: success-rate
          args:
          - name: service-name
            value: api-server
      - setWeight: 20             # 20% 流量
      - pause: { duration: 10m }
      - setWeight: 50             # 50% 流量
      - pause: { duration: 15m }
      - setWeight: 100            # 全量发布

      # 自动回滚条件
      analysis:
        successfulRunHistoryLimit: 3
        unsuccessfulRunHistoryLimit: 1

---
# 灰度指标分析模板
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: success-rate
spec:
  args:
  - name: service-name
  metrics:
  - name: success-rate
    interval: 60s
    successCondition: result[0] >= 0.99    # 成功率 >= 99%
    provider:
      prometheus:
        address: http://prometheus:9090
        query: |
          sum(rate(http_requests_total{
            service="{{args.service-name}}",
            status=~"2.."
          }[2m])) /
          sum(rate(http_requests_total{
            service="{{args.service-name}}"
          }[2m]))
  - name: latency-p95
    interval: 60s
    successCondition: result[0] <= 0.5     # P95 延迟 <= 500ms
    provider:
      prometheus:
        address: http://prometheus:9090
        query: |
          histogram_quantile(0.95, sum(rate(
            http_request_duration_seconds_bucket{
              service="{{args.service-name}}"
            }[2m]
          )) by (le))
```

### 3.4 全量发布

```yaml
# 全量发布检查清单
full_rollout_checklist:
  pre_release:
    - 灰度阶段指标全部达标
    - 灰度期间无 P0/P1 告警
    - 核心业务冒烟测试通过
    - 回滚方案已确认
    - On-call 人员已就位

  execution:
    - 通知相关团队发布开始
    - 执行全量部署
    - 监控核心指标 15 分钟
    - 执行冒烟测试
    - 确认发布成功

  post_release:
    - 发布通知（Slack/邮件/钉钉）
    - 更新 Changelog
    - 更新版本号标记
    - 关闭相关 Issue/Story
    - 删除已合并的功能分支
```

---

## 阶段四：回滚操作

### 4.1 回滚决策矩阵

| 问题类型 | 影响范围 | 决策 | 回滚方式 |
|----------|---------|------|---------|
| 核心功能不可用 | 全站 | 立即回滚 | 部署回滚 |
| 错误率 > 5% | 部分用户 | 5 分钟内回滚 | 部署回滚 |
| 延迟 > 2x SLO | 全站 | 10 分钟内回滚 | 部署回滚 |
| 数据不一致 | 部分数据 | 评估后回滚 | 部署 + 数据回滚 |
| 非关键 Bug | 部分功能 | 热修复 | Hotfix 发布 |
| UI 异常 | 视觉 | 评估 | Feature Flag 关闭 |

### 4.2 回滚执行步骤

```bash
#!/bin/bash
# rollback.sh - 生产环境回滚脚本
set -euo pipefail

NAMESPACE="production"
DEPLOYMENT="api-server"
PREVIOUS_VERSION="${1:-}"

echo "=== 开始回滚 ==="
echo "时间: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "操作人: $(whoami)"

# 步骤 1：确认回滚版本
if [ -z "$PREVIOUS_VERSION" ]; then
  echo "回滚到上一版本..."
  kubectl rollout undo deployment/${DEPLOYMENT} -n ${NAMESPACE}
else
  echo "回滚到指定版本: ${PREVIOUS_VERSION}..."
  kubectl set image deployment/${DEPLOYMENT} \
    ${DEPLOYMENT}=registry.example.com/app:${PREVIOUS_VERSION} \
    -n ${NAMESPACE}
fi

# 步骤 2：等待回滚完成
echo "等待回滚完成..."
kubectl rollout status deployment/${DEPLOYMENT} -n ${NAMESPACE} --timeout=300s

# 步骤 3：验证健康检查
echo "验证健康检查..."
for i in $(seq 1 10); do
  STATUS=$(curl -s -o /dev/null -w "%{http_code}" https://api.example.com/healthz)
  if [ "$STATUS" = "200" ]; then
    echo "健康检查通过 (尝试 $i/10)"
    break
  fi
  echo "等待健康检查... (尝试 $i/10)"
  sleep 5
done

# 步骤 4：冒烟测试
echo "执行冒烟测试..."
./scripts/smoke-test.sh

# 步骤 5：通知
echo "发送回滚通知..."
curl -X POST "$SLACK_WEBHOOK" -H 'Content-type: application/json' \
  -d "{
    \"text\": \":warning: 生产环境已回滚\",
    \"attachments\": [{
      \"color\": \"warning\",
      \"fields\": [
        {\"title\": \"服务\", \"value\": \"${DEPLOYMENT}\", \"short\": true},
        {\"title\": \"时间\", \"value\": \"$(date)\", \"short\": true},
        {\"title\": \"操作人\", \"value\": \"$(whoami)\", \"short\": true}
      ]
    }]
  }"

echo "=== 回滚完成 ==="
```

### 4.3 数据库回滚

```sql
-- 数据库变更必须提供正向和反向迁移
-- migration_045_add_order_index.sql (正向)
CREATE INDEX CONCURRENTLY idx_orders_created_at ON orders(created_at);

-- migration_045_add_order_index_rollback.sql (反向)
DROP INDEX CONCURRENTLY IF EXISTS idx_orders_created_at;

-- 回滚原则：
-- 1. DDL 变更使用 CONCURRENTLY（不锁表）
-- 2. 新增字段允许 NULL（兼容旧代码）
-- 3. 删除字段延迟到下一版本（当前版本只停止使用）
-- 4. 数据迁移脚本必须幂等
```

---

## 阶段五：Changelog 生成

### 5.1 Conventional Commits 规范

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]

类型定义：
  feat:     新功能
  fix:      缺陷修复
  docs:     文档变更
  style:    代码格式（不影响逻辑）
  refactor: 重构（非新增/非修复）
  perf:     性能优化
  test:     测试相关
  chore:    构建/工具变更
  ci:       CI/CD 配置变更

  BREAKING CHANGE: 在 footer 中标记破坏性变更

示例：
  feat(auth): add OAuth2 login with Google
  fix(order): prevent duplicate order submission
  perf(search): add Redis cache for product search
  feat(api)!: change pagination format to cursor-based
```

### 5.2 自动生成 Changelog

```bash
#!/bin/bash
# generate-changelog.sh
# 基于 Conventional Commits 自动生成 Changelog

PREVIOUS_TAG=$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || echo "")
CURRENT_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "HEAD")

echo "# Changelog - ${CURRENT_TAG}"
echo ""
echo "发布日期: $(date +%Y-%m-%d)"
echo ""

# 新功能
FEATURES=$(git log ${PREVIOUS_TAG}..${CURRENT_TAG} --pretty=format:"%s" | grep "^feat" || true)
if [ -n "$FEATURES" ]; then
  echo "## New Features"
  echo "$FEATURES" | sed 's/^feat(\(.*\)): /- **\1**: /' | sed 's/^feat: /- /'
  echo ""
fi

# 修复
FIXES=$(git log ${PREVIOUS_TAG}..${CURRENT_TAG} --pretty=format:"%s" | grep "^fix" || true)
if [ -n "$FIXES" ]; then
  echo "## Bug Fixes"
  echo "$FIXES" | sed 's/^fix(\(.*\)): /- **\1**: /' | sed 's/^fix: /- /'
  echo ""
fi

# 性能优化
PERFS=$(git log ${PREVIOUS_TAG}..${CURRENT_TAG} --pretty=format:"%s" | grep "^perf" || true)
if [ -n "$PERFS" ]; then
  echo "## Performance"
  echo "$PERFS" | sed 's/^perf(\(.*\)): /- **\1**: /' | sed 's/^perf: /- /'
  echo ""
fi

# 破坏性变更
BREAKING=$(git log ${PREVIOUS_TAG}..${CURRENT_TAG} --pretty=format:"%b" | grep "BREAKING CHANGE" || true)
if [ -n "$BREAKING" ]; then
  echo "## BREAKING CHANGES"
  echo "$BREAKING" | sed 's/^BREAKING CHANGE: /- /'
  echo ""
fi
```

### 5.3 Changelog 模板

```markdown
# Changelog

## [2.2.0] - 2025-03-15

### New Features
- **auth**: 新增 Google OAuth2 登录 (#234)
- **dashboard**: 新增实时数据大盘 (#245)
- **export**: 支持 CSV/Excel 导出 (#251)

### Bug Fixes
- **order**: 修复重复下单问题 (#260)
- **search**: 修复中文搜索分词异常 (#262)

### Performance
- **search**: 商品搜索新增 Redis 缓存，P95 延迟降低 60% (#255)

### BREAKING CHANGES
- **api**: 分页接口从 offset 切换为 cursor，详见迁移指南 (#248)

### Dependencies
- 升级 fastapi 0.109 -> 0.110
- 升级 pydantic 2.5 -> 2.6

### Contributors
@alice, @bob, @charlie
```

---

## 阶段六：通知流程

### 6.1 通知矩阵

| 事件 | 通知渠道 | 接收者 | 时机 |
|------|---------|--------|------|
| 发布开始 | Slack #deploy | 开发 + SRE | 部署前 |
| 灰度开始 | Slack #deploy | 开发 + SRE + QA | 灰度启动时 |
| 全量完成 | Slack #deploy + 邮件 | 全团队 + 业务方 | 全量后 |
| 回滚执行 | Slack #deploy + PagerDuty | 开发 + SRE + 管理层 | 立即 |
| Changelog | 邮件 + 文档站 | 全公司 | 发布后 24 小时内 |
| 破坏性变更 | 邮件 + 文档站 | API 消费方 | 提前 2 周 |

### 6.2 发布通知模板

```markdown
## Release Notification

**版本**: v2.2.0
**发布时间**: 2025-03-15 14:00 UTC
**发布人**: @alice
**发布类型**: 常规发布

### 变更摘要
- 新增 3 个功能
- 修复 2 个 Bug
- 1 个性能优化

### 用户可见变更
- 新增 Google 登录入口
- 数据大盘新增实时刷新
- 商品搜索速度提升 60%

### 注意事项
- 分页 API 将在 v2.4.0 废弃 offset 模式，请迁移至 cursor 模式
- 详细 Changelog: [链接]

### 回滚联系人
- 主要: @bob (SRE On-call)
- 备选: @charlie
```

---

## Agent Checklist

- [ ] 已确认版本策略采用 SemVer 并在项目中统一执行
- [ ] 已选择并实施分支策略（Git Flow 或 Trunk-Based）
- [ ] 构建流水线包含：版本注入、质量检查、测试、安全扫描、制品签名
- [ ] 测试分层已配置：单元测试 -> 集成测试 -> E2E -> 性能 -> 安全
- [ ] 灰度发布能力已验证（5% -> 20% -> 50% -> 100% 阶梯式）
- [ ] 灰度期间自动指标分析已配置（成功率 + 延迟）
- [ ] 回滚脚本已编写并在 staging 环境验证
- [ ] 数据库变更提供正向和反向迁移脚本
- [ ] Commit 规范已采用 Conventional Commits
- [ ] Changelog 自动生成已集成到发布流程
- [ ] 通知流程已配置（发布/灰度/回滚/Changelog）
- [ ] 发布后观察计划已安排
