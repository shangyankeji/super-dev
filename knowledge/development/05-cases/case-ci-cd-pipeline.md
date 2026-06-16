---
id: case-ci-cd-pipeline
title: 案例研究：CI/CD 流水线从 0 到 1 搭建实战
domain: development
category: 05-cases
difficulty: intermediate
tags: [agent, case, checklist, development, pipeline, 元数据]
quality_score: 70
last_updated: 2026-06-15
---
# 案例研究：CI/CD 流水线从 0 到 1 搭建实战

## 元数据

| 字段 | 值 |
|------|------|
| 行业 | 金融科技（支付网关） |
| 系统规模 | 12 个微服务，日交易 500 万笔 |
| 技术栈 | Go + React + PostgreSQL + Kubernetes |
| 团队规模 | 后端 16 人，前端 6 人，QA 4 人，SRE 3 人 |
| 搭建周期 | 12 周（2024-01 至 2024-03） |
| 核心目标 | 从手工部署到全自动化交付，满足金融合规要求 |

---

## 一、背景

### 1.1 现状分析

搭建前的开发与部署流程：

```
开发流程：
1. 开发者在本地开发 → 手动运行 go test
2. 提交 PR → 人工 Code Review（无自动检查）
3. 合并到 main → 通知 SRE "可以发了"
4. SRE 手动 ssh 到服务器执行部署脚本
5. 部署后人工验证功能是否正常

问题统计（过去 3 个月）：
- 平均部署频率：每周 1.5 次
- 单次部署耗时：2-4 小时（含人工操作 + 验证）
- 部署失败率：23%（主要原因：配置遗漏、依赖不一致）
- 线上回滚次数：月均 3 次
- 开发者等待构建：日均 40 分钟（本地构建 + 手动测试）
```

### 1.2 痛点清单

| 类别 | 痛点 | 业务影响 |
|------|------|----------|
| 效率 | 手工部署耗时长 | SRE 50% 时间花在部署上 |
| 质量 | 无自动化测试门禁 | 线上 Bug 率高，每周 2-3 个 Hotfix |
| 安全 | 无安全扫描 | 金融监管合规风险 |
| 一致性 | 环境配置手工管理 | "在我机器上能跑"问题频发 |
| 可追溯 | 无部署审计日志 | 监管审查时无法提供变更记录 |
| 速度 | 部署窗口限制 | 紧急修复也要等部署窗口 |

### 1.3 目标定义

| 指标 | 当前值 | 目标值 |
|------|--------|--------|
| 部署频率 | 1.5 次/周 | 5+ 次/天 |
| 部署耗时 | 2-4 小时 | < 15 分钟 |
| 部署失败率 | 23% | < 2% |
| 代码到生产 | 3-5 天 | < 2 小时 |
| 测试覆盖率 | 35% | > 80% |
| 安全扫描 | 无 | 每次构建 |
| 审计可追溯 | 无 | 100% 变更可追溯 |

---

## 二、挑战

### 2.1 金融合规要求

作为支付网关，CI/CD 流水线必须满足：

1. **PCI DSS 要求**：所有代码变更必须经过审查和审批
2. **变更管理**：每次部署需要关联变更工单和审批记录
3. **环境隔离**：开发/测试/预发/生产环境严格隔离
4. **密钥管理**：密钥和证书不能出现在代码仓库或构建日志中
5. **审计追踪**：保留 1 年的构建和部署日志

### 2.2 技术约束

1. 12 个微服务使用不同的构建方式（8 个 Go，3 个 React，1 个 Python）
2. 服务间存在版本依赖关系（支付核心 → 风控 → 路由）
3. 数据库迁移需要与代码部署协调
4. 前后端需要版本对齐部署

### 2.3 组织约束

1. 团队无 CI/CD 经验，SRE 3 人需要同时承担日常运维
2. 不能影响现有业务，必须平滑过渡
3. 预算有限，优先使用开源方案

---

## 三、方案设计

### 3.1 工具链选型

| 环节 | 工具 | 理由 |
|------|------|------|
| 代码托管 | GitLab Self-hosted | 金融合规要求私有化部署 |
| CI 引擎 | GitLab CI | 与代码托管一体化，减少集成成本 |
| CD 引擎 | ArgoCD | GitOps 模式，声明式部署，审计友好 |
| 镜像仓库 | Harbor | 私有化，漏洞扫描内置 |
| 密钥管理 | HashiCorp Vault | 金融级密钥管理 |
| 制品管理 | Nexus | 统一管理 Go/npm/Python 依赖 |
| 代码质量 | SonarQube | 代码质量 + 安全扫描 |
| 容器扫描 | Trivy | 镜像漏洞扫描，开源免费 |
| 测试框架 | Go test + Jest + Playwright | 分别覆盖后端/前端/E2E |
| 监控 | Prometheus + Grafana | 构建和部署指标可视化 |

### 3.2 流水线架构

```
┌──────────────────────────────────────────────────────────┐
│                    CI Pipeline                            │
│                                                          │
│  PR Created                                              │
│    ├── Stage 1: Lint & Format Check        (~30s)       │
│    │     ├── golangci-lint (Go)                          │
│    │     ├── eslint + prettier (React)                   │
│    │     └── ruff + black (Python)                       │
│    │                                                     │
│    ├── Stage 2: Unit Test                  (~2min)       │
│    │     ├── go test -race -coverprofile                 │
│    │     ├── jest --coverage                             │
│    │     └── pytest --cov                                │
│    │                                                     │
│    ├── Stage 3: Security Scan              (~3min)       │
│    │     ├── SonarQube SAST                              │
│    │     ├── go vuln check                               │
│    │     ├── npm audit                                   │
│    │     └── semgrep (custom rules)                      │
│    │                                                     │
│    ├── Stage 4: Build & Push Image         (~2min)       │
│    │     ├── Docker multi-stage build                    │
│    │     ├── Trivy image scan                            │
│    │     └── Push to Harbor                              │
│    │                                                     │
│    └── Stage 5: Integration Test           (~5min)       │
│          ├── docker-compose up (dependencies)            │
│          ├── API contract test (Pact)                    │
│          └── E2E smoke test (Playwright)                 │
│                                                          │
│  Total CI Time: ~12 minutes                              │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│                    CD Pipeline                            │
│                                                          │
│  Merge to main                                           │
│    ├── CI Pipeline (同上)                                │
│    │                                                     │
│    ├── Deploy to Staging                                 │
│    │     ├── ArgoCD sync (auto)                          │
│    │     ├── DB migration (if needed)                    │
│    │     ├── Smoke test suite                            │
│    │     └── Performance test (k6)                       │
│    │                                                     │
│    ├── Deploy to Pre-production                          │
│    │     ├── Manual approval gate (Tech Lead)            │
│    │     ├── ArgoCD sync                                 │
│    │     ├── Full regression test                        │
│    │     └── Security penetration test                   │
│    │                                                     │
│    └── Deploy to Production                              │
│          ├── Manual approval gate (SRE Lead + PM)        │
│          ├── ArgoCD sync (canary 10% → 50% → 100%)      │
│          ├── Health check + metric validation            │
│          ├── Auto-rollback on error rate > 0.1%          │
│          └── Post-deploy verification                    │
│                                                          │
│  Total CD Time: Staging ~8min, Pre-prod ~20min,         │
│                 Prod ~15min (excluding approval wait)     │
└──────────────────────────────────────────────────────────┘
```

### 3.3 GitOps 目录结构

```
infra-repo/
├── base/                          # 基础 K8s manifests
│   ├── payment-core/
│   │   ├── deployment.yaml
│   │   ├── service.yaml
│   │   ├── hpa.yaml
│   │   └── kustomization.yaml
│   ├── risk-engine/
│   └── ...
├── overlays/
│   ├── staging/                   # Staging 环境差异配置
│   │   ├── kustomization.yaml
│   │   └── patches/
│   ├── pre-prod/
│   └── production/
│       ├── kustomization.yaml
│       ├── patches/
│       │   ├── replicas.yaml      # 生产副本数
│       │   ├── resources.yaml     # 生产资源配额
│       │   └── hpa.yaml           # 生产 HPA 配置
│       └── sealed-secrets/        # 加密的 Secrets
└── argocd/
    ├── applications/              # ArgoCD Application 定义
    └── projects/                  # ArgoCD Project 定义
```

---

## 四、实施步骤

### 4.1 Phase 1：基础设施（Week 1-3）

```
Week 1: 工具部署
  - GitLab Self-hosted 部署（HA 模式）
  - Harbor 部署 + HTTPS + LDAP 集成
  - SonarQube 部署 + Go/JS/Python 插件
  - Vault 部署 + 初始化

Week 2: K8s 集群与 ArgoCD
  - K8s 集群搭建（3 Master + 9 Worker）
  - 命名空间规划：staging / pre-prod / production
  - ArgoCD 部署 + RBAC 配置
  - Sealed Secrets 配置（生产密钥加密）

Week 3: 基础流水线模板
  - 编写 .gitlab-ci.yml 基础模板
  - GitLab Runner 部署（K8s executor，动态 Pod 运行 Job）
  - 镜像构建模板（Kaniko，无 Docker daemon）
  - Nexus 代理仓库配置（加速依赖下载）
```

### 4.2 Phase 2：CI 流水线（Week 4-6）

```
Week 4: Lint + 单元测试
  - Go 服务：golangci-lint + go test + go vet
  - React 应用：eslint + jest
  - Python 服务：ruff + pytest
  - 覆盖率门禁：新代码 > 80%，整体 > 60%

Week 5: 安全扫描 + 镜像构建
  - SonarQube 质量门禁接入
  - Trivy 镜像扫描（CRITICAL 级别阻断）
  - semgrep 自定义规则（SQL 注入、硬编码密钥检测）
  - 多阶段 Dockerfile 模板（构建镜像 vs 运行镜像分离）

Week 6: 集成测试
  - docker-compose 本地集成测试环境
  - Pact 契约测试（服务间 API 兼容性验证）
  - Playwright E2E 冒烟测试（核心支付流程）
```

**GitLab CI 核心配置**（Go 服务示例）：

```yaml
# .gitlab-ci.yml
stages:
  - lint
  - test
  - security
  - build
  - integration
  - deploy

variables:
  GOPROXY: "https://nexus.internal/repository/go-proxy/"
  CGO_ENABLED: "0"

lint:
  stage: lint
  image: golangci/golangci-lint:v1.55
  script:
    - golangci-lint run --timeout 5m ./...
  rules:
    - if: '$CI_PIPELINE_SOURCE == "merge_request_event"'

unit-test:
  stage: test
  image: golang:1.22
  script:
    - go test -race -coverprofile=coverage.out ./...
    - go tool cover -func=coverage.out
  coverage: '/total:\s+\(statements\)\s+(\d+\.\d+)%/'
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: coverage.xml

sonar-scan:
  stage: security
  image: sonarsource/sonar-scanner-cli
  script:
    - sonar-scanner
      -Dsonar.projectKey=${CI_PROJECT_NAME}
      -Dsonar.sources=.
      -Dsonar.go.coverage.reportPaths=coverage.out
      -Dsonar.qualitygate.wait=true

trivy-scan:
  stage: security
  image: aquasec/trivy
  script:
    - trivy fs --exit-code 1 --severity CRITICAL .
    - trivy image --exit-code 1 --severity CRITICAL ${IMAGE_NAME}:${CI_COMMIT_SHA}

build:
  stage: build
  image:
    name: gcr.io/kaniko-project/executor:debug
    entrypoint: [""]
  script:
    - /kaniko/executor
      --context ${CI_PROJECT_DIR}
      --dockerfile Dockerfile
      --destination ${HARBOR_REGISTRY}/${CI_PROJECT_NAME}:${CI_COMMIT_SHA}
      --cache=true
      --cache-repo=${HARBOR_REGISTRY}/${CI_PROJECT_NAME}/cache

integration-test:
  stage: integration
  services:
    - postgres:15
    - redis:7
  variables:
    POSTGRES_DB: test
    POSTGRES_USER: test
    POSTGRES_PASSWORD: test
  script:
    - go test -tags=integration ./tests/integration/...
```

### 4.3 Phase 3：CD 流水线（Week 7-9）

```
Week 7: Staging 自动部署
  - ArgoCD Application 配置（auto-sync for staging）
  - 数据库迁移集成（golang-migrate，CI Job 执行）
  - Staging 冒烟测试自动触发

Week 8: Pre-production + Production
  - Pre-production 半自动部署（需 Tech Lead 审批）
  - Production 金丝雀部署（Argo Rollouts）
  - 部署后自动化验证（Health check + Metric validation）
  - 自动回滚配置（错误率 > 0.1% 触发）

Week 9: 数据库迁移编排
  - Migration 与代码部署的依赖管理
  - 向前兼容要求（migration 必须支持 N-1 版本代码）
  - 回滚策略（每个 migration 必须有对应 down 脚本）
```

**ArgoCD Rollout 配置**（金丝雀部署）：

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: payment-core
  namespace: production
spec:
  replicas: 10
  strategy:
    canary:
      steps:
        - setWeight: 10
        - pause: { duration: 5m }
        - analysis:
            templates:
              - templateName: success-rate
            args:
              - name: service-name
                value: payment-core
        - setWeight: 50
        - pause: { duration: 10m }
        - analysis:
            templates:
              - templateName: success-rate
        - setWeight: 100
      canaryService: payment-core-canary
      stableService: payment-core-stable
      trafficRouting:
        istio:
          virtualService:
            name: payment-core
---
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: success-rate
spec:
  metrics:
    - name: success-rate
      interval: 60s
      successCondition: result[0] > 0.999
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            sum(rate(http_requests_total{service="{{args.service-name}}",
            code=~"2.."}[2m]))
            /
            sum(rate(http_requests_total{service="{{args.service-name}}"}[2m]))
```

### 4.4 Phase 4：安全与合规（Week 10-11）

```
Week 10: 密钥管理
  - Vault 动态数据库凭据（每次部署自动轮转）
  - K8s Secret 改为 Sealed Secret
  - CI 变量加密存储，构建日志脱敏

Week 11: 审计与合规
  - 部署审计日志 → ELK Stack（保留 1 年）
  - 变更工单集成（Jira + GitLab MR 关联）
  - 合规报告自动生成（每月 PCI DSS 合规摘要）
  - RBAC 精细化（开发者只能部署到 Staging）
```

### 4.5 Phase 5：优化与培训（Week 12）

```
Week 12:
  - 构建缓存优化（Go module cache、Docker layer cache）
  - 并行 Job 优化（独立 Stage 并行执行）
  - 团队培训：CI/CD 使用指南 + 故障排查 + 最佳实践
  - 编写 Runbook：常见 CI 失败处理、回滚操作、紧急发布流程
```

---

## 五、结果数据

### 5.1 核心指标对比

| 指标 | 搭建前 | 搭建后 | 改善幅度 |
|------|--------|--------|----------|
| 部署频率 | 1.5 次/周 | 8 次/天 | 37x |
| 部署耗时 | 2-4 小时 | 12 分钟 | 15x |
| 部署失败率 | 23% | 1.5% | -93% |
| 代码到生产 | 3-5 天 | 1.5 小时 | 48x |
| 线上 Bug 率 | 2-3/周 | 0.3/周 | -87% |
| 回滚次数 | 3 次/月 | 0.5 次/月 | -83% |
| 回滚耗时 | 30 分钟 | 2 分钟（自动） | 15x |
| 测试覆盖率 | 35% | 82% | +134% |
| 安全漏洞发现 | 渗透测试时 | 每次 PR | 实时 |

### 5.2 CI 性能指标

| 阶段 | 耗时 | 说明 |
|------|------|------|
| Lint | 28s | golangci-lint 全量扫描 |
| Unit Test | 1m 45s | 并行执行，含覆盖率采集 |
| Security Scan | 2m 30s | SonarQube + Trivy 并行 |
| Build & Push | 1m 50s | Kaniko + Docker layer cache |
| Integration Test | 4m 20s | 含依赖服务启动时间 |
| **Total CI** | **~11 min** | 目标 12 分钟内达成 |

### 5.3 合规审计

| 合规项 | 状态 |
|--------|------|
| 所有变更可追溯 | 通过（GitLab MR + ArgoCD 审计日志） |
| 密钥不在代码中 | 通过（Vault + Sealed Secrets） |
| 生产部署需审批 | 通过（ArgoCD Manual Sync + Jira 工单） |
| 环境隔离 | 通过（K8s Namespace + NetworkPolicy） |
| 安全扫描 | 通过（每次 PR + 每次构建） |
| 日志保留 1 年 | 通过（ELK Stack + S3 归档） |

### 5.4 团队效能

| 指标 | 搭建前 | 搭建后 |
|------|--------|--------|
| SRE 部署工作占比 | 50% | 5% |
| 开发者等待构建 | 40 min/天 | 0（后台运行） |
| 新服务接入时间 | 2 天（手动配置） | 30 分钟（模板化） |
| 夜间紧急发布 | 月均 2 次（人工） | 月均 0.5 次（自助） |

---

## 六、经验教训

### 6.1 做对的事

1. **模板化优先**：为 Go/React/Python 各建立标准 CI 模板，新服务接入只需引用模板 + 填写变量
2. **分阶段交付价值**：Week 4 团队就有了 Lint + 测试门禁，不用等到 Week 12 才能用
3. **GitOps 模式**：所有环境配置都在 Git 中，审计和回滚变得极其简单
4. **金丝雀 + 自动回滚**：生产部署的风险降到最低，团队敢于频繁发布
5. **安全左移**：安全扫描前移到 PR 阶段，漏洞在代码合并前就被发现

### 6.2 做错的事

1. **初期未考虑构建缓存**：前 4 周 CI 时间 20+ 分钟，团队抱怨多，后来加了缓存才降到 11 分钟
2. **集成测试环境不稳定**：docker-compose 方式在 CI Runner 上偶尔启动失败，后改为固定测试环境
3. **数据库迁移工具选型犹豫**：先用 goose 后改 golang-migrate，浪费了 1 周
4. **文档滞后**：工具搭好了但文档没跟上，团队采用率前 4 周只有 40%

### 6.3 关键认知

- CI/CD 不只是技术项目，是团队文化转变。30% 的时间应花在培训和文档上
- 快速反馈是核心价值：CI 超过 15 分钟，开发者会绕过它
- 安全不是附加项，必须内建到流水线中（Security as Code）
- GitOps 是金融合规的天然盟友：声明式 + 版本化 + 可审计
- 从一个服务试点开始，验证后再推广到全部服务

---

## Agent Checklist

在 AI Agent 辅助搭建 CI/CD 流水线时，应逐项确认：

### CI 阶段
- [ ] **代码检查**：是否配置了 Lint + Format + 静态分析
- [ ] **单元测试**：是否运行单元测试并采集覆盖率
- [ ] **覆盖率门禁**：是否设置了最低覆盖率要求
- [ ] **安全扫描**：是否集成了 SAST + 依赖漏洞扫描
- [ ] **镜像构建**：是否使用多阶段构建减小镜像体积
- [ ] **镜像扫描**：构建后的镜像是否通过漏洞扫描
- [ ] **集成测试**：是否有服务间的契约测试和 E2E 测试
- [ ] **构建缓存**：是否配置了依赖缓存和 Docker 层缓存
- [ ] **构建时间**：CI 总时长是否在 15 分钟以内

### CD 阶段
- [ ] **环境隔离**：Staging / Pre-prod / Production 是否严格隔离
- [ ] **审批门禁**：生产部署是否需要人工审批
- [ ] **金丝雀发布**：是否支持灰度发布和自动分析
- [ ] **自动回滚**：是否配置了基于指标的自动回滚
- [ ] **数据库迁移**：Migration 是否集成到部署流程，且支持回滚
- [ ] **密钥管理**：密钥是否通过 Vault/Sealed Secrets 管理
- [ ] **部署审计**：每次部署是否有审计日志和变更记录

### 运维阶段
- [ ] **监控看板**：是否有 CI/CD 指标的 Grafana Dashboard
- [ ] **告警配置**：构建失败/部署失败是否有及时告警
- [ ] **Runbook**：常见 CI 失败和回滚操作是否有文档
- [ ] **模板化**：新服务接入是否有标准化模板
- [ ] **权限管理**：谁能部署到哪个环境是否有 RBAC 控制
