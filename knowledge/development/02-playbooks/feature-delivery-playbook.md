---
id: feature-delivery-playbook
title: 功能交付作战手册 (Feature Delivery Playbook)
domain: development
category: 02-playbooks
difficulty: intermediate
tags: [checklist, code, delivery, development, feature, playbook, review, 前置条件]
quality_score: 70
last_updated: 2026-06-15
---
# 功能交付作战手册 (Feature Delivery Playbook)

## 概述

功能交付是从需求澄清到生产上线的完整开发过程。本手册定义了每个阶段的输入输出、执行标准和门禁条件，确保交付的功能满足业务需求、代码质量和运维标准。适用于中大型功能开发（预估工时 > 2 人天）。

## 前置条件

### 必须满足

- [ ] 需求文档或 PRD 已评审通过并签字确认
- [ ] 产品验收标准（AC）已明确定义
- [ ] 开发资源和排期已确认
- [ ] 依赖的上下游服务和接口已明确
- [ ] Git 仓库、CI/CD 流水线、开发/测试环境就绪

### 建议满足

- [ ] UI 设计稿已完成并通过评审
- [ ] 技术方案已完成预研（针对新技术栈或复杂场景）
- [ ] 性能和安全约束已明确

---

## 阶段一：需求澄清

### 1.1 需求分析会议

```markdown
## 需求分析模板

### 业务背景
- 为什么要做这个功能？解决什么业务问题？
- 目标用户是谁？使用场景是什么？
- 期望的业务成果指标？

### 功能范围
- 核心功能列表（Must Have）
- 增强功能列表（Should Have）
- 未来功能列表（Nice to Have）
- 明确不做的事项（Out of Scope）

### 验收标准
- [ ] AC-1: [具体的可验证条件]
- [ ] AC-2: [具体的可验证条件]

### 非功能性需求
- 性能：响应时间 < Xms，并发量 >= Y
- 安全：认证方式、数据加密要求
- 兼容：浏览器/设备/API 版本
- 可用性：SLA 目标
```

### 1.2 需求拆解

```markdown
## 用户故事拆解

### Epic: [功能名称]

#### Story 1: [子功能]
作为 [角色]，我希望 [功能]，以便 [价值]
预估：X 故事点
依赖：无

#### Story 2: [子功能]
作为 [角色]，我希望 [功能]，以便 [价值]
预估：X 故事点
依赖：Story 1

### 依赖关系图
Story 1 --> Story 2 --> Story 4
Story 1 --> Story 3 --> Story 4
```

### 1.3 阶段门禁

- [ ] 所有核心用户故事有明确的验收标准
- [ ] 依赖关系已识别并有应对方案
- [ ] 团队对需求理解一致（无分歧项）

---

## 阶段二：技术方案

### 2.1 方案设计模板

```markdown
## 技术方案

### 架构变更
- 新增/修改的模块和组件
- 数据流变化
- 系统交互变化

### 数据模型
```sql
-- 新增表
CREATE TABLE feature_table (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    data JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_feature_table_user_status ON feature_table(user_id, status);
```

### 接口设计
```yaml
POST /api/v1/features:
  request:
    content_type: application/json
    body:
      name: string (required, 1-100 chars)
      type: enum [typeA, typeB]
  response:
    201:
      body: { id, name, type, created_at }
    400:
      body: { error: { code, message, details[] } }
    409:
      body: { error: { code: "DUPLICATE_NAME", message } }
```

### 兼容性
- API 版本策略
- 数据库迁移的前向/后向兼容
- 旧客户端适配

### 性能考量
- 预估数据量和增长速度
- 热点查询优化策略
- 缓存策略

### 风险与应对
| 风险 | 概率 | 影响 | 应对 |
|------|------|------|------|
| 第三方 API 不稳定 | 中 | 高 | 熔断 + 缓存兜底 |

### 回滚策略
- 代码回滚方式
- 数据回滚方式
- 回滚判定条件
```

### 2.2 方案评审

评审要点：
- 架构合理性：是否过度设计？是否满足扩展需求？
- 数据模型：索引是否合理？查询模式是否覆盖？
- 接口设计：是否符合 RESTful 规范？错误处理是否完整？
- 安全性：认证授权、输入校验、数据加密
- 性能：是否有明显瓶颈？是否考虑了缓存？

### 2.3 阶段门禁

- [ ] 技术方案文档完成并评审通过
- [ ] 接口契约已定义并同步给上下游
- [ ] 数据库变更脚本已准备（如需）
- [ ] 回滚策略已明确

---

## 阶段三：实施开发

### 3.1 分支策略

```bash
# 从 develop 分支创建功能分支
git checkout develop
git pull origin develop
git checkout -b feature/PROJ-123-feature-name

# 开发过程中定期同步
git fetch origin develop
git rebase origin/develop
```

### 3.2 开发规范

```python
# 提交规范
# feat: 新功能
# fix: 修复
# refactor: 重构
# test: 测试
# docs: 文档
# chore: 工具/配置

# 提交粒度：一个逻辑变更一个提交
git commit -m "feat(order): add order creation endpoint

- Add POST /api/v1/orders endpoint
- Add request validation with Pydantic schema
- Add unit tests for order creation logic

Refs: PROJ-123"
```

### 3.3 分批提交策略

```
提交顺序（推荐）：
1. 数据模型和迁移脚本
2. 领域模型和业务逻辑（含单元测试）
3. 接口层（Controller/Router + 集成测试）
4. 前端页面和交互
5. 端到端测试
6. 文档更新
```

### 3.4 Code Review 标准

```markdown
## Code Review Checklist

### 正确性
- [ ] 逻辑是否正确实现了需求？
- [ ] 边界条件是否处理？
- [ ] 错误处理是否完整？

### 安全性
- [ ] 输入是否做了校验和过滤？
- [ ] 敏感数据是否加密？
- [ ] 权限检查是否到位？

### 性能
- [ ] 是否有 N+1 查询？
- [ ] 大数据量场景是否考虑？
- [ ] 是否需要缓存？

### 可维护性
- [ ] 命名是否清晰？
- [ ] 函数是否单一职责？
- [ ] 是否有充分的测试？

### 运维
- [ ] 关键操作是否有日志？
- [ ] 监控指标是否添加？
- [ ] 配置是否外部化？
```

### 3.5 阶段门禁

- [ ] 所有代码已通过 Code Review
- [ ] 单元测试覆盖率 >= 80%
- [ ] Lint 和静态分析无告警
- [ ] 所有 CI 检查通过

---

## 阶段四：联调测试

### 4.1 联调准备

```bash
# 部署到联调环境
kubectl apply -f k8s/staging/ -n staging

# 验证服务启动
kubectl get pods -n staging -l app=<service>
curl -s http://staging-api.example.com/health | jq '.'

# Mock 外部依赖（如需）
docker run -d --name wiremock -p 8089:8080 wiremock/wiremock
```

### 4.2 测试用例

```markdown
## 核心链路测试

### 正常流程
- [ ] 创建功能 -> 验证返回 201 和正确数据
- [ ] 查询列表 -> 验证分页和过滤
- [ ] 更新功能 -> 验证数据变更
- [ ] 删除功能 -> 验证软删除

### 异常流程
- [ ] 重复创建 -> 验证返回 409
- [ ] 无权限访问 -> 验证返回 403
- [ ] 参数非法 -> 验证返回 400 和错误详情
- [ ] 依赖服务超时 -> 验证熔断和降级
- [ ] 并发操作 -> 验证数据一致性

### 兼容性测试
- [ ] 旧版本 API 调用 -> 验证向后兼容
- [ ] 旧客户端访问 -> 验证不报错
```

### 4.3 阶段门禁

- [ ] 核心链路测试全部通过
- [ ] 异常链路测试全部通过
- [ ] 上下游联调确认通过
- [ ] 性能测试达标（如有要求）
- [ ] 安全扫描无高危漏洞

---

## 阶段五：发布观测

### 5.1 发布前检查

```bash
# 发布前自动化检查脚本
#!/bin/bash
set -e

echo "=== 发布前检查 ==="

echo "[1/5] 代码分支检查"
CURRENT=$(git branch --show-current)
echo "当前分支: $CURRENT"

echo "[2/5] 测试执行"
pytest tests/ --tb=short -q

echo "[3/5] 安全扫描"
trivy image <image-name>:<tag> --severity HIGH,CRITICAL

echo "[4/5] 数据库迁移检查"
python manage.py migrate --check 2>/dev/null && echo "无待执行迁移" || echo "有待执行迁移"

echo "[5/5] 回滚脚本验证"
test -f rollback.sh && echo "回滚脚本存在" || echo "警告: 缺少回滚脚本"

echo "=== 检查完成 ==="
```

### 5.2 灰度发布

```yaml
# 灰度策略
stage_1:
  canary_percentage: 5%
  duration: 30m
  success_criteria:
    error_rate: < 0.1%
    p99_latency: < 500ms
stage_2:
  canary_percentage: 25%
  duration: 2h
  success_criteria:
    error_rate: < 0.05%
    p99_latency: < 300ms
stage_3:
  canary_percentage: 100%
  observation_window: 24h
```

### 5.3 发布后观测

```bash
# 关键指标监控
watch -n 10 'echo "错误率:" && curl -s "http://prometheus:9090/api/v1/query?query=rate(http_errors_total{service=\"target\"}[5m])" | jq ".data.result[0].value[1]" && echo "P99延迟:" && curl -s "http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99,rate(http_duration_seconds_bucket{service=\"target\"}[5m]))" | jq ".data.result[0].value[1]"'
```

### 5.4 阶段门禁

- [ ] 灰度期间错误率未超过阈值
- [ ] 核心指标（延迟、吞吐量）与基线一致
- [ ] 无用户投诉
- [ ] 24 小时观察窗口无异常

---

## 回滚方案

### 代码回滚

```bash
# Kubernetes 回滚
kubectl rollout undo deployment/<service> -n production

# 验证
kubectl rollout status deployment/<service> -n production
```

### 数据回滚

```bash
# 如果涉及数据库迁移
python manage.py migrate <app> <previous_migration_number>

# 如果涉及数据修复
psql -h db-host -U user -d dbname < rollback_data.sql
```

### 回滚触发条件

| 指标 | 阈值 | 动作 |
|------|------|------|
| 错误率 | > 0.5% | 自动回滚 |
| P99 延迟 | > 基线 2 倍 | 人工确认后回滚 |
| 核心功能不可用 | 任何 | 立即回滚 |
| 数据异常 | 任何 | 立即回滚 + 数据修复 |

---

## Agent Checklist

AI 编码 Agent 在执行功能交付时必须逐项确认：

- [ ] **需求理解**：已阅读 PRD 和验收标准，理解业务目标
- [ ] **技术方案**：已完成技术方案并获得确认
- [ ] **接口契约**：API 接口定义已确认并同步给相关方
- [ ] **数据模型**：数据库变更有迁移脚本，支持回滚
- [ ] **分支管理**：从正确的基线分支创建功能分支
- [ ] **分批提交**：按逻辑分批提交，每个提交可独立编译运行
- [ ] **测试覆盖**：单元测试覆盖率 >= 80%，核心路径有集成测试
- [ ] **Code Review**：所有代码已通过审查
- [ ] **联调通过**：核心链路和异常链路测试全部通过
- [ ] **安全检查**：无高危漏洞，输入校验完整
- [ ] **发布策略**：灰度发布配置就绪，回滚方案已验证
- [ ] **监控就绪**：关键指标监控和告警已配置
- [ ] **文档更新**：API 文档、变更记录已更新
