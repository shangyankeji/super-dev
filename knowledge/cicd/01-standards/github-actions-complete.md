---
id: github-actions-complete
title: GitHub Actions 完整指南
domain: cicd
category: 01-standards
difficulty: intermediate
tags: [actions, agent, changes, checklist, cicd, complete, github, 安全最佳实践]
quality_score: 70
last_updated: 2026-06-15
---
# GitHub Actions 完整指南

## 概述

GitHub Actions 是 GitHub 原生的 CI/CD 平台，允许直接在仓库中定义自动化工作流。通过 YAML 文件配置，支持构建、测试、部署、发布等全生命周期自动化。

### 核心优势

- **原生集成**: 与 GitHub 深度集成，无需额外工具
- **丰富市场**: 16000+ 社区 Actions 可复用
- **矩阵构建**: 并行测试多个环境/版本
- **自托管 Runner**: 支持自定义执行环境
- **免费额度**: 公开仓库免费，私有仓库 2000 分钟/月

---

## 核心概念

### Workflow 结构

```yaml
# .github/workflows/ci.yml
name: CI Pipeline          # 工作流名称

on:                         # 触发条件
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * 1'    # 每周一凌晨2点
  workflow_dispatch:         # 手动触发
    inputs:
      environment:
        description: 'Deploy environment'
        required: true
        default: 'staging'
        type: choice
        options: [staging, production]

env:                         # 全局环境变量
  NODE_VERSION: '20'
  PYTHON_VERSION: '3.11'

jobs:                        # 作业定义
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run linter
        run: npm run lint

  test:
    needs: lint              # 依赖 lint 完成
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node: [18, 20, 22]   # 矩阵测试
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node }}
      - run: npm ci
      - run: npm test
```

### 触发事件详解

```yaml
on:
  # Push 事件
  push:
    branches: [main, 'release/**']
    tags: ['v*']
    paths:
      - 'src/**'
      - '!src/**/*.test.ts'  # 排除测试文件

  # PR 事件
  pull_request:
    types: [opened, synchronize, reopened]
    branches: [main]

  # 定时任务
  schedule:
    - cron: '30 5 * * 1-5'   # 工作日 5:30 UTC

  # 其他仓库事件
  issues:
    types: [opened, labeled]
  release:
    types: [published]
  workflow_run:
    workflows: ["Build"]
    types: [completed]
```

---

## 实战模板

### 1. Python CI/CD

```yaml
name: Python CI/CD

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ['3.10', '3.11', '3.12']

    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: test
          POSTGRES_DB: testdb
        ports: ['5432:5432']
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

      redis:
        image: redis:7
        ports: ['6379:6379']

    steps:
      - uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python-version }}
          cache: 'pip'

      - name: Install dependencies
        run: |
          python -m pip install --upgrade pip
          pip install -e ".[dev]"

      - name: Lint
        run: |
          ruff check .
          black --check .
          mypy src/

      - name: Test
        env:
          DATABASE_URL: postgresql://postgres:test@localhost:5432/testdb
          REDIS_URL: redis://localhost:6379
        run: |
          pytest --cov=src --cov-report=xml -v

      - name: Upload coverage
        if: matrix.python-version == '3.11'
        uses: codecov/codecov-action@v4
        with:
          file: coverage.xml

  deploy:
    needs: test
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    environment: production

    steps:
      - uses: actions/checkout@v4

      - name: Deploy
        env:
          DEPLOY_KEY: ${{ secrets.DEPLOY_KEY }}
        run: |
          echo "Deploying to production..."
```

### 2. Node.js + Docker 构建

```yaml
name: Build & Push Docker

on:
  push:
    tags: ['v*']

jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ghcr.io/${{ github.repository }}
          tags: |
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=sha

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

### 3. 矩阵构建 + 跨平台

```yaml
name: Cross-Platform Build

on: [push, pull_request]

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        node: [18, 20]
        include:
          - os: ubuntu-latest
            node: 20
            coverage: true
        exclude:
          - os: windows-latest
            node: 18

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node }}
      - run: npm ci
      - run: npm test
      - name: Coverage
        if: matrix.coverage
        run: npm run test:coverage
```

### 4. 自动化发布

```yaml
name: Release

on:
  push:
    tags: ['v*']

permissions:
  contents: write

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Generate changelog
        id: changelog
        run: |
          PREV_TAG=$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || echo "")
          if [ -n "$PREV_TAG" ]; then
            CHANGELOG=$(git log ${PREV_TAG}..HEAD --pretty=format:"- %s (%h)" --no-merges)
          else
            CHANGELOG=$(git log --pretty=format:"- %s (%h)" --no-merges -20)
          fi
          echo "changelog<<EOF" >> $GITHUB_OUTPUT
          echo "$CHANGELOG" >> $GITHUB_OUTPUT
          echo "EOF" >> $GITHUB_OUTPUT

      - name: Create Release
        uses: softprops/action-gh-release@v2
        with:
          body: |
            ## Changes
            ${{ steps.changelog.outputs.changelog }}
          draft: false
          prerelease: ${{ contains(github.ref, '-rc') }}
```

---

## 高级技巧

### Secrets 管理

```yaml
# 在 Settings → Secrets and variables → Actions 中设置
env:
  API_KEY: ${{ secrets.API_KEY }}
  DATABASE_URL: ${{ secrets.DATABASE_URL }}

# 环境级 Secrets (需要审批)
jobs:
  deploy:
    environment:
      name: production
      url: https://myapp.com
    steps:
      - run: echo "Using ${{ secrets.PROD_API_KEY }}"
```

### 缓存优化

```yaml
# pip 缓存
- uses: actions/setup-python@v5
  with:
    python-version: '3.11'
    cache: 'pip'

# npm 缓存
- uses: actions/setup-node@v4
  with:
    node-version: '20'
    cache: 'npm'

# 自定义缓存
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-
```

### 可复用工作流

```yaml
# .github/workflows/reusable-test.yml
name: Reusable Test
on:
  workflow_call:
    inputs:
      python-version:
        required: true
        type: string
    secrets:
      codecov-token:
        required: false

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: ${{ inputs.python-version }}
      - run: pytest

# 调用方
# .github/workflows/ci.yml
jobs:
  test:
    uses: ./.github/workflows/reusable-test.yml
    with:
      python-version: '3.11'
    secrets:
      codecov-token: ${{ secrets.CODECOV_TOKEN }}
```

### 条件执行

```yaml
steps:
  # 只在 main 分支执行
  - if: github.ref == 'refs/heads/main'
    run: echo "On main branch"

  # 只在 PR 中执行
  - if: github.event_name == 'pull_request'
    run: echo "In PR"

  # 前一步成功才执行
  - if: success()
    run: echo "Previous step succeeded"

  # 前一步失败时执行
  - if: failure()
    run: echo "Previous step failed"

  # 总是执行 (清理)
  - if: always()
    run: echo "Always runs"

  # 包含特定标签
  - if: contains(github.event.pull_request.labels.*.name, 'deploy')
    run: echo "Has deploy label"
```

---

## 安全最佳实践

1. **✅ 最小权限**: 使用 `permissions` 限制 GITHUB_TOKEN 权限
2. **✅ 固定版本**: 使用 `actions/checkout@v4` 而非 `@main`
3. **✅ 审查第三方 Actions**: 检查源码，使用 SHA 固定版本
4. **✅ 保护 Secrets**: 不在日志中打印，使用环境级 Secrets
5. **✅ 保护分支**: 要求 PR 审查和状态检查通过
6. **❌ 不要**: 在 PR 中运行不受信任的代码（`pull_request_target`风险）
7. **❌ 不要**: 硬编码密钥在工作流文件中

---

## Agent Checklist

Agent 在配置 CI/CD 时必须检查:

- [ ] 工作流是否覆盖 lint/test/build/deploy 全流程？
- [ ] 是否使用矩阵测试覆盖多版本？
- [ ] Secrets 是否通过 GitHub Secrets 管理（非硬编码）？
- [ ] 是否配置缓存优化构建速度？
- [ ] Docker 构建是否使用多阶段构建和缓存？
- [ ] 生产部署是否需要环境审批？
- [ ] 是否有自动化发布流程（tag触发）？
- [ ] Actions 版本是否固定（SHA或major版本）？
- [ ] 工作流权限是否遵循最小权限原则？
- [ ] 是否有失败通知机制？

---

**文档版本**: v1.0
**最后更新**: 2026-03-28
**质量评分**: 90/100
