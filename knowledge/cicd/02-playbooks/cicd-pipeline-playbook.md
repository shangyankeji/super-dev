---
id: cicd-pipeline-playbook
title: CI/CD Pipeline 设计实战手册
domain: cicd
category: 02-playbooks
difficulty: advanced
tags: [cicd, github-actions, gitlab-ci, monorepo, pipeline, cache, parallel, deployment, blue-green, canary, enterprise]
quality_score: 94
maintainer: devops-team@superdev.com
last_updated: 2026-06-15
---

# CI/CD Pipeline 设计实战手册

> 基于 [TechBuddies 2025 CI/CD Case Study](https://www.techbuddies.io/2025/12/17/case-study-how-we-optimized-ci-cd-pipelines-in-github-actions-and-gitlab-ci/) + [Buildkite Monorepo CI](https://buildkite.com/resources/blog/monorepo-ci-best-practices/) + [GitLab Monorepo Guide](https://about.gitlab.com/blog/building-a-gitlab-ci-cd-pipeline-for-a-monorepo-the-easy-way/)

## Pipeline 分层架构

```
Push → Lint/Fmt → Unit Test → Build → Integration Test → Deploy(staging) → E2E → Deploy(prod)
         ↑----------- 并行 -----------↑                    ↑-- 手动批准 --↑
```

每层只有前一层通过才触发。快速反馈在前（Lint < 30s），慢操作在后（E2E > 5min）。

## Monorepo 路径过滤

### GitHub Actions
```yaml
# .github/workflows/api.yml
on:
  push:
    paths:
      - 'services/api/**'       # 只有 API 目录变更才触发
      - '.github/workflows/api.yml'
      - 'packages/shared/**'    # 共享包变更也触发

jobs:
  api:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Detect changes
        uses: dorny/paths-filter@v3
        id: changes
        with:
          filters: |
            api: ['services/api/**']
            shared: ['packages/shared/**']
      - if: steps.changes.outputs.api == 'true'
        run: cd services/api && npm ci && npm test
```

### GitLab CI
```yaml
api:
  rules:
    - changes:
        - services/api/**       # only:changes 替代
  script:
    - cd services/api
    - npm ci
    - npm test
```

## 缓存加速

```yaml
# GitHub Actions — 缓存依赖
- uses: actions/setup-node@v4
  with:
    node-version: '20'
    cache: 'npm'                # 自动缓存 node_modules
    cache-dependency-path: 'services/api/package-lock.json'

# Docker 层缓存
- uses: docker/build-push-action@v5
  with:
    cache-from: type=gha        # GitHub Actions cache
    cache-to: type=gha,mode=max
```

## 并行 Fan-Out

```yaml
# 拆分测试到多个 runner 并行跑
test-matrix:
  strategy:
    matrix:
      shard: [1, 2, 3, 4]       # 4 个 runner 各跑 1/4 测试
  steps:
    - run: npx jest --shard=${{ matrix.shard }}/4
```

## 部署策略

### 蓝绿部署（零停机）
```yaml
deploy:
  steps:
    - name: Deploy to green
      run: kubectl apply -f k8s/green/
    - name: Health check
      run: ./scripts/health-check.sh green
    - name: Switch traffic
      run: kubectl patch service app -p '{"spec":{"selector":{"version":"green"}}}'
    - name: Cleanup blue
      run: kubectl delete deployment app-blue
      if: success()              # 只在成功后删旧版
```

### 金丝雀部署（渐进放量）
```yaml
canary:
  steps:
    - run: ./deploy --weight 5    # 5% 流量到新版
    - run: sleep 300              # 观察 5 分钟
    - run: ./deploy --weight 25   # 25%
    - run: sleep 300
    - run: ./deploy --weight 100  # 全量
```

## Pipeline 性能优化

| 技术 | 效果 | 示例 |
|------|------|------|
| 路径过滤 | 只测变更部分 | `paths: [services/api/**]` |
| 依赖缓存 | 省去重复安装 | `cache: 'npm'` |
| 并行 Fan-Out | 测试拆多 runner | `matrix: shard: [1,2,3,4]` |
| 增量构建 | 只构建变更的包 | Turborepo / Nx affected |
| 浅克隆 | 少拉 Git 历史 | `fetch-depth: 1` |
| 条件跳过 | 文档变更跳过 E2E | `if: steps.changes.outputs.code` |

## 生产检查清单
- [ ] Pipeline 总时长 < 10 分钟（反馈循环）
- [ ] 路径过滤（monorepo 只触发变更部分）
- [ ] 依赖缓存（npm/cargo/pip）
- [ ] 并行测试（fan-out）
- [ ] 产物缓存（Docker 层）
- [ ] 分支保护（main 不能直接 push）
- [ ] PR 必须过 CI 才能 merge
- [ ] 生产部署需人工批准（environment protection）
- [ ] 蓝绿/金丝雀部署（零停机 + 快速回滚）
- [ ] Pipeline 失败有 Slack/邮件通知
