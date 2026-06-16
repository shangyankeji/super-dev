---
id: case-deployment-automation
title: 部署自动化案例：从手动部署到 GitOps 的转型
domain: cicd
category: 05-cases
difficulty: intermediate
tags: [agent, automation, case, checklist, cicd, deployment, 关键决策回顾, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# 部署自动化案例：从手动部署到 GitOps 的转型

## 概述

本案例记录一个 50 人研发团队将部署流程从手动 SSH + 脚本模式迁移到 GitOps 模式的
完整过程。转型历时 3 个月，部署频率从每周 1 次提升到每天 5-8 次，部署失败率从
15% 降至 2%，平均部署耗时从 45 分钟降至 8 分钟。

## 转型前状态

### 部署流程（手动模式）

```
1. 开发者在 Slack 通知运维"准备部署"
2. 运维 SSH 登录服务器（3 台应用服务器）
3. 手动执行 git pull
4. 手动执行 npm install && npm run build
5. 手动重启 PM2 进程
6. 逐台检查日志确认无报错
7. 在 Slack 回复"部署完成"
```

### 核心痛点

| 问题 | 影响 |
|------|------|
| 部署耗时 45min+ | 运维被部署占满，无暇做其他工作 |
| 环境差异 | "我本地可以跑"频繁发生 |
| 回滚困难 | 需要手动 git revert + 重新构建 |
| 无法审计 | 不知道谁在什么时候部署了什么版本 |
| 部署失败率 15% | 漏装依赖、配置遗漏、顺序错误 |
| 单点依赖运维 | 运维请假时无人可部署 |

## 转型路线图

### 第一阶段：容器化（第 1-4 周）

**目标**: 消除环境差异，构建标准化部署单元

```dockerfile
# 多阶段构建
FROM node:18-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
RUN npm run build

FROM node:18-alpine
RUN addgroup -S app && adduser -S app -G app
WORKDIR /app
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
USER app
EXPOSE 3000
CMD ["node", "dist/server.js"]
```

**成果**:
- 所有 12 个微服务完成 Docker 化
- 本地开发使用 docker-compose 统一环境
- 构建产物从"代码+依赖"变为"不可变镜像"

### 第二阶段：CI 流水线（第 3-6 周）

**目标**: 自动化构建、测试、镜像推送

```yaml
# .github/workflows/ci.yml
name: CI
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm ci
      - run: npm test
      - run: npm run lint

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build and push
        run: |
          docker build -t registry.example.com/app:${{ github.sha }} .
          docker push registry.example.com/app:${{ github.sha }}

  scan:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Trivy scan
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: registry.example.com/app:${{ github.sha }}
          severity: CRITICAL,HIGH
          exit-code: 1
```

**成果**:
- 每次 PR 自动运行测试 + Lint
- 合并到 main 自动构建镜像并推送
- 镜像漏洞扫描集成到流水线

### 第三阶段：GitOps 部署（第 5-10 周）

**目标**: 声明式部署，Git 仓库作为唯一真实来源

**工具选型**: ArgoCD + Kustomize

```
代码仓库（app-repo）          配置仓库（deploy-repo）
├── src/                      ├── base/
├── Dockerfile                │   ├── deployment.yaml
└── .github/workflows/        │   ├── service.yaml
    └── ci.yml                │   └── kustomization.yaml
                              ├── overlays/
                              │   ├── dev/
                              │   ├── staging/
                              │   └── prod/
                              └── argocd/
                                  └── application.yaml
```

**部署流程（GitOps 模式）**:

```
1. CI 构建新镜像 → 推送到 Registry
2. CI 自动更新 deploy-repo 中的镜像 tag
3. ArgoCD 检测到 deploy-repo 变更
4. ArgoCD 对比集群当前状态与期望状态
5. ArgoCD 自动同步（dev/staging）或等待审批（prod）
6. 健康检查通过后标记部署成功
```

### 第四阶段：渐进式发布（第 9-12 周）

**目标**: 灰度发布，降低部署风险

- **Canary 发布**: 新版本先接收 10% 流量，监控 5 分钟无异常后逐步提升
- **自动回滚**: 错误率 > 1% 或 P99 延迟 > 500ms 自动回滚
- **Feature Flag**: LaunchDarkly 集成，功能开关与部署解耦

```yaml
# Argo Rollouts canary 策略
spec:
  strategy:
    canary:
      steps:
        - setWeight: 10
        - pause: { duration: 5m }
        - setWeight: 30
        - pause: { duration: 5m }
        - setWeight: 60
        - pause: { duration: 5m }
        - setWeight: 100
      analysis:
        templates:
          - templateName: error-rate
        startingStep: 1
```

## 转型效果

| 指标 | 转型前 | 转型后 | 提升 |
|------|--------|--------|------|
| 部署频率 | 1 次/周 | 5-8 次/天 | 35x-56x |
| 部署耗时 | 45 分钟 | 8 分钟 | 5.6x |
| 部署失败率 | 15% | 2% | 7.5x |
| 回滚耗时 | 30 分钟 | 2 分钟 | 15x |
| MTTR | 60 分钟 | 12 分钟 | 5x |
| 运维人力占比 | 40% 时间做部署 | 5% | 8x |

## 踩过的坑

1. **镜像体积过大**: 初始镜像 1.2GB，多阶段构建后降至 180MB，部署速度提升明显
2. **配置管理混乱**: 初期把配置硬编码在 Kubernetes YAML 中，后改用 ConfigMap + Sealed Secrets
3. **ArgoCD 权限过大**: 初期给了 cluster-admin，后收缩到 namespace 级别
4. **缺少 staging 验证**: 直接从 dev 到 prod 出过事故，补充 staging 环境后稳定
5. **团队抵触**: 部分开发者不适应 PR 驱动的部署流程，通过结对演示和文档逐步解决

## 关键决策回顾

| 决策 | 选择 | 理由 |
|------|------|------|
| 编排工具 | Kubernetes | 团队已有容器基础，K8s 生态最完善 |
| GitOps 工具 | ArgoCD | 社区活跃，UI 直观，声明式管理 |
| 配置管理 | Kustomize | 比 Helm 轻量，适合团队规模 |
| 密钥管理 | Sealed Secrets | 可存入 Git，运维成本低 |
| 渐进式发布 | Argo Rollouts | 与 ArgoCD 集成好 |

## Agent Checklist

- [ ] 应用是否已完成容器化（Dockerfile + 多阶段构建）
- [ ] CI 流水线是否覆盖测试/构建/扫描
- [ ] 配置仓库是否与代码仓库分离
- [ ] GitOps 工具是否配置环境差异化（dev/staging/prod）
- [ ] 生产部署是否有审批机制
- [ ] 渐进式发布（Canary/Blue-Green）是否已实施
- [ ] 自动回滚策略是否基于业务指标
- [ ] 密钥管理是否使用 Sealed Secrets 或外部 Vault
- [ ] 部署审计日志是否完整可追溯
- [ ] 团队是否完成 GitOps 工作流培训
