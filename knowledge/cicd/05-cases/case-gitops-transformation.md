---
id: case-gitops-transformation
title: GitOps 转型实战案例
domain: cicd
category: 05-cases
difficulty: intermediate
tags: [agent, case, checklist, cicd, gitops, transformation, 实施步骤, 技术选型]
quality_score: 70
last_updated: 2026-06-15
---
# GitOps 转型实战案例

## 概述

本案例记录一个 50 人研发团队从传统手动部署迁移到 GitOps 全自动化交付的完整过程。历时 3 个月，部署频率从每周 1 次提升到每天 10+ 次，MTTR 从 2 小时降至 8 分钟。

---

## 背景

### 团队现状（转型前）
- **团队规模**: 50 人，6 个微服务
- **部署方式**: SSH 登录服务器手动执行脚本
- **发布频率**: 每周三下午统一发布
- **部署耗时**: 每次 2-3 小时（含协调时间）
- **回滚方式**: 手动替换 JAR 包，重启服务
- **问题**: 周三下午全员待命，频繁出错，回滚慢，无法追溯变更

### 目标
- 每个 PR 合并后自动部署到 staging
- 生产部署通过 Git tag 触发，全自动
- 回滚时间 < 5 分钟
- 完整的变更审计追踪

---

## 技术选型

| 工具 | 用途 | 选型理由 |
|------|------|----------|
| **GitHub Actions** | CI Pipeline | 团队已用 GitHub，无需额外工具 |
| **ArgoCD** | CD (GitOps) | K8s 原生，声明式，自动同步 |
| **Kubernetes** | 运行时 | 已有 K8s 集群 |
| **Helm** | 包管理 | 模板化 K8s 配置 |
| **Kustomize** | 环境差异 | Overlay 方式管理多环境 |

---

## 实施步骤

### Phase 1: CI 标准化（第 1-2 周）

```yaml
# .github/workflows/ci.yml
name: CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm ci
      - run: npm run lint
      - run: npm test -- --coverage
      - run: npm run build

  docker:
    needs: test
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - uses: docker/build-push-action@v5
        with:
          push: true
          tags: ghcr.io/${{ github.repository }}:${{ github.sha }}
```

### Phase 2: GitOps 仓库搭建（第 3-4 周）

```
deploy-manifests/           # 独立仓库
├── base/                   # 基础配置
│   ├── deployment.yaml
│   ├── service.yaml
│   └── kustomization.yaml
├── overlays/
│   ├── staging/
│   │   ├── kustomization.yaml
│   │   └── replicas-patch.yaml
│   └── production/
│       ├── kustomization.yaml
│       ├── replicas-patch.yaml
│       └── resources-patch.yaml
└── argocd/
    ├── staging-app.yaml
    └── production-app.yaml
```

### Phase 3: ArgoCD 部署（第 5-6 周）

```yaml
# argocd/production-app.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: myapp-production
spec:
  project: default
  source:
    repoURL: https://github.com/org/deploy-manifests
    path: overlays/production
    targetRevision: main
  destination:
    server: https://kubernetes.default.svc
    namespace: production
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
```

### Phase 4: 自动化镜像更新（第 7-8 周）

CI 构建完成后，自动更新 GitOps 仓库中的镜像标签：

```yaml
# CI workflow 中的最后一步
- name: Update GitOps repo
  run: |
    git clone https://github.com/org/deploy-manifests
    cd deploy-manifests
    kustomize edit set image myapp=ghcr.io/org/myapp:${{ github.sha }}
    git commit -am "chore: update myapp to ${{ github.sha }}"
    git push
```

### Phase 5: 金丝雀发布（第 9-12 周）

```yaml
# Argo Rollouts 金丝雀策略
apiVersion: argoproj.io/v1alpha1
kind: Rollout
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
      canaryMetadata:
        labels:
          role: canary
```

---

## 结果数据

| 指标 | 转型前 | 转型后 | 改善 |
|------|--------|--------|------|
| 部署频率 | 1次/周 | 10+次/天 | **70x** |
| 部署耗时 | 2-3小时 | 3分钟 | **40x** |
| 回滚时间 | 30-60分钟 | 3分钟 (git revert) | **15x** |
| MTTR | 2小时 | 8分钟 | **15x** |
| 部署失败率 | 15% | 2% | **87%↓** |
| 变更可追溯性 | 无 | 100% Git 审计 | ✅ |

---

## 经验教训

### 做得好的
1. **先 CI 后 CD** — 没有好的 CI，GitOps 毫无意义
2. **独立部署仓库** — 应用代码和部署配置分离，权限清晰
3. **渐进式推进** — staging 先行，production 跟进

### 踩过的坑
1. **Secret 管理** — 初期把 Secret 放在 Git 仓库（错误！），后改用 Sealed Secrets
2. **ArgoCD 权限** — 初期所有人都有 production sync 权限，后收紧为 RBAC
3. **镜像标签** — 初期用 `latest` 标签（错误！），后改用 Git SHA

---

## Agent Checklist

Agent 在设计 CI/CD 流程时必须检查:

- [ ] CI 是否覆盖 lint/test/build/scan 全流程？
- [ ] 镜像标签是否使用 Git SHA（非 latest）？
- [ ] 部署配置是否与应用代码分离？
- [ ] Secret 是否加密存储（Sealed Secrets/Vault）？
- [ ] 是否有金丝雀/蓝绿发布策略？
- [ ] 回滚是否可以通过 git revert 完成？
- [ ] 是否有变更审计追踪？
- [ ] 生产部署是否需要审批？

---

**文档版本**: v1.0
**最后更新**: 2026-03-28
**质量评分**: 88/100
