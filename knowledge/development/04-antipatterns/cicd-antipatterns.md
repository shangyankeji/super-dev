---
id: cicd-antipatterns
title: CI/CD 反模式指南
domain: development
category: 04-antipatterns
difficulty: intermediate
tags: [agent, antipatterns, artifacts, canary, checklist, cicd, development, drift]
quality_score: 70
last_updated: 2026-06-15
---
# CI/CD 反模式指南

> 适用范围：GitHub Actions / GitLab CI / Jenkins / Azure Pipelines
> 约束级别：SHALL（必须在 Pipeline 配置审查阶段拦截）

---

## 1. 无质量门禁直接发布（Skipping Quality Gates）

### 描述
CI 流水线只做构建（build），不做测试、Lint、安全扫描就直接进入部署阶段。或者质量检查存在但设置为 `allow_failure: true`，实际不阻断。等同于不设防，任何有缺陷的代码都能进入生产环境。

### 错误示例
```yaml
# GitHub Actions -- 构建成功就部署
name: Deploy
on:
  push:
    branches: [main]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm install
      - run: npm run build
      - run: ./deploy.sh production  # 直接部署，无测试

# GitLab CI -- 安全扫描不阻断
security_scan:
  script: trivy image myapp:latest
  allow_failure: true  # 扫出漏洞也不阻断

test:
  script: pytest
  allow_failure: true  # 测试失败也不阻断
```

### 正确示例
```yaml
name: CI/CD Pipeline
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm run lint
      - run: npm run type-check

  test:
    runs-on: ubuntu-latest
    needs: [lint]
    steps:
      - uses: actions/checkout@v4
      - run: npm test -- --coverage
      - name: Check coverage threshold
        run: |
          COVERAGE=$(jq '.total.lines.pct' coverage/coverage-summary.json)
          if (( $(echo "$COVERAGE < 80" | bc -l) )); then
            echo "Coverage $COVERAGE% is below 80% threshold"
            exit 1
          fi

  security:
    runs-on: ubuntu-latest
    needs: [lint]
    steps:
      - uses: actions/checkout@v4
      - name: Dependency audit
        run: npm audit --audit-level=high
      - name: SAST scan
        run: semgrep --config auto --error  # 发现问题则失败
      - name: Secret scan
        run: gitleaks detect --source . --verbose

  deploy:
    runs-on: ubuntu-latest
    needs: [test, security]  # 必须全部通过才能部署
    if: github.ref == 'refs/heads/main'
    steps:
      - run: ./deploy.sh production
```

### 检测方法
- 部署 Job 无 `needs` 依赖测试和安全 Job。
- 存在 `allow_failure: true` 的关键检查步骤。
- CI 配置中无 `test` / `lint` / `security` 阶段。
- `git log` 中存在直接 push 到 main 分支（无 PR、无 CI）。

### 修复步骤
1. CI 流水线添加 lint -> test -> security -> deploy 四个阶段。
2. deploy 阶段必须依赖前三个阶段全部通过。
3. 删除所有 `allow_failure: true`，改为硬性阻断。
4. 设置分支保护规则，禁止直接 push 到 main。
5. 设置测试覆盖率阈值（>= 80%）和安全扫描零高危。

### Agent Checklist
- [ ] 部署依赖 lint + test + security 全部通过
- [ ] 无 `allow_failure: true` 的关键检查
- [ ] main 分支有分支保护规则
- [ ] 测试覆盖率 >= 80%
- [ ] 安全扫描零高危漏洞

---

## 2. 无灰度发布与回滚机制（Missing Canary and Rollback）

### 描述
发布直接全量切换到新版本，没有灰度（Canary）或蓝绿部署策略。如果新版本存在 Bug，100% 的用户立即受影响。且没有自动化的回滚机制，需要人工操作恢复。

### 错误示例
```yaml
# 直接全量发布
deploy:
  script:
    - kubectl set image deployment/myapp myapp=myapp:$TAG
    # 没有灰度
    # 没有健康检查
    # 没有回滚条件

# 手动回滚
# "发现问题了！快回滚！"
# "上一个版本号是什么来着？"
# kubectl set image deployment/myapp myapp=myapp:???
```

### 正确示例
```yaml
# 灰度发布 + 自动回滚
deploy_canary:
  script:
    # 1. 灰度 10% 流量
    - kubectl apply -f canary-deployment.yaml  # 1 个 Pod
    - echo "Canary deployed, monitoring for 5 minutes..."

    # 2. 健康检查
    - |
      for i in $(seq 1 30); do
        ERROR_RATE=$(curl -s "$METRICS_URL/error_rate?version=$TAG")
        LATENCY_P99=$(curl -s "$METRICS_URL/latency_p99?version=$TAG")

        if (( $(echo "$ERROR_RATE > 1.0" | bc -l) )); then
          echo "Error rate $ERROR_RATE% exceeds 1% threshold"
          kubectl rollout undo deployment/myapp-canary
          exit 1
        fi

        if (( $(echo "$LATENCY_P99 > 500" | bc -l) )); then
          echo "P99 latency ${LATENCY_P99}ms exceeds 500ms threshold"
          kubectl rollout undo deployment/myapp-canary
          exit 1
        fi

        sleep 10
      done

    # 3. 灰度通过，全量发布
    - kubectl set image deployment/myapp myapp=myapp:$TAG
    - kubectl rollout status deployment/myapp --timeout=300s

  on_failure:
    - kubectl rollout undo deployment/myapp
    - slack-notify "Deployment $TAG rolled back due to failure"
```

```python
# 自动回滚脚本
import subprocess
import requests
import time

def deploy_with_canary(tag: str, metrics_url: str):
    # 部署 Canary
    subprocess.run(["kubectl", "set", "image", "deployment/myapp-canary", f"myapp=myapp:{tag}"])

    # 监控 5 分钟
    for _ in range(30):
        metrics = requests.get(f"{metrics_url}/canary").json()
        if metrics["error_rate"] > 1.0 or metrics["p99_latency_ms"] > 500:
            print(f"Canary unhealthy: {metrics}")
            subprocess.run(["kubectl", "rollout", "undo", "deployment/myapp-canary"])
            raise DeploymentError("Canary check failed, rolled back")
        time.sleep(10)

    # 全量发布
    subprocess.run(["kubectl", "set", "image", "deployment/myapp", f"myapp=myapp:{tag}"])
    subprocess.run(["kubectl", "rollout", "status", "deployment/myapp", "--timeout=300s"])
```

### 检测方法
- 部署脚本中无灰度策略（Canary / 蓝绿 / Rolling）。
- 无部署后的健康检查（HTTP health endpoint / 错误率监控）。
- 无自动回滚条件和脚本。
- 回滚需要人工查找上一个版本号。

### 修复步骤
1. 实现灰度部署：先部署 1 个 Canary Pod（10% 流量）。
2. 定义健康指标和阈值：错误率 < 1%、P99 延迟 < 500ms。
3. 灰度期间持续监控，超过阈值自动回滚。
4. 灰度通过后全量发布，并监控全量阶段。
5. 记录每次部署的版本号和制品 SHA，支持一键回滚。

### Agent Checklist
- [ ] 发布使用灰度策略（不直接全量）
- [ ] 部署后有健康检查（错误率 + 延迟）
- [ ] 超过阈值自动回滚
- [ ] 支持一键回滚到上一个版本
- [ ] 部署事件有通知（Slack / 钉钉）

---

## 3. 制品不可追溯（Untraceable Artifacts）

### 描述
构建产物（Docker 镜像、JAR 包、npm 包）无法追溯到对应的 Git commit、构建环境和依赖版本。出现线上问题时，无法确定当前运行的是哪个版本的代码。

### 错误示例
```dockerfile
# Docker 镜像无版本标签
FROM python:3.11
COPY . /app
# docker build -t myapp .
# docker push myapp:latest  -- 永远是 latest，无法区分版本
```

```yaml
# CI 不记录构建信息
build:
  script:
    - docker build -t myapp .
    - docker push myapp:latest
```

### 正确示例
```dockerfile
# Dockerfile 嵌入构建信息
FROM python:3.11-slim

ARG GIT_COMMIT
ARG BUILD_DATE
ARG VERSION

LABEL org.opencontainers.image.revision=$GIT_COMMIT
LABEL org.opencontainers.image.created=$BUILD_DATE
LABEL org.opencontainers.image.version=$VERSION

COPY . /app
WORKDIR /app
RUN pip install --no-cache-dir -r requirements.txt

# 将版本信息写入文件，API 可返回
RUN echo "{\"version\": \"$VERSION\", \"commit\": \"$GIT_COMMIT\", \"built_at\": \"$BUILD_DATE\"}" > /app/build-info.json
```

```yaml
# CI 记录完整的构建溯源
build:
  script:
    - export GIT_COMMIT=$(git rev-parse HEAD)
    - export GIT_SHORT=$(git rev-parse --short HEAD)
    - export BUILD_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    - export VERSION="1.2.3-${GIT_SHORT}"

    - docker build
        --build-arg GIT_COMMIT=$GIT_COMMIT
        --build-arg BUILD_DATE=$BUILD_DATE
        --build-arg VERSION=$VERSION
        -t myapp:$VERSION
        -t myapp:$GIT_SHORT
        -t myapp:latest
        .

    - docker push myapp:$VERSION
    - docker push myapp:$GIT_SHORT

    # 记录构建信息到部署追踪系统
    - |
      curl -X POST "$DEPLOY_TRACKER_URL/builds" -H "Content-Type: application/json" -d "{
        \"service\": \"myapp\",
        \"version\": \"$VERSION\",
        \"commit\": \"$GIT_COMMIT\",
        \"branch\": \"$CI_COMMIT_BRANCH\",
        \"built_at\": \"$BUILD_DATE\",
        \"pipeline_url\": \"$CI_PIPELINE_URL\"
      }"
```

```python
# 应用暴露版本信息 API
@app.get("/health")
def health():
    build_info = json.loads(open("build-info.json").read())
    return {
        "status": "healthy",
        "version": build_info["version"],
        "commit": build_info["commit"],
        "built_at": build_info["built_at"],
    }
```

### 检测方法
- Docker 镜像只有 `latest` 标签。
- 无 `/health` 或 `/version` API 返回版本信息。
- 线上出问题时无法确定运行的代码版本。
- 构建日志无 Git commit SHA。

### 修复步骤
1. Docker 镜像使用 `版本号-短 commit SHA` 作为标签。
2. 在镜像中嵌入 `build-info.json`，包含 commit、版本、构建时间。
3. 暴露 `/health` API 返回版本信息。
4. CI 流水线记录构建元数据到追踪系统。
5. 部署工具记录 "哪个版本部署到了哪个环境"。

### Agent Checklist
- [ ] Docker 镜像有语义化版本标签
- [ ] 镜像不使用裸 `latest` 标签发布
- [ ] 应用有 `/health` API 返回版本信息
- [ ] 构建元数据包含 Git commit SHA
- [ ] 可追溯任意时刻各环境运行的版本

---

## 4. 环境不一致（Environment Drift）

### 描述
开发、测试、预发、生产环境的配置和基础设施不一致。"在我机器上能跑"但上线就出问题。常见原因：手动配置环境、不同环境使用不同的依赖版本、基础设施配置未版本化。

### 错误示例
```
# 各环境手动安装依赖，版本可能不同
# 开发：Node 18.17.0 + npm 9.6.7
# 测试：Node 18.19.0 + npm 10.2.3
# 生产：Node 20.10.0 + npm 10.1.0  -- 版本都不同

# 手动修改生产环境配置
ssh production-server
vi /etc/nginx/nginx.conf    # 直接改，不入版本控制
systemctl restart nginx
```

### 正确示例
```dockerfile
# 所有环境使用相同的 Docker 镜像
FROM node:18.19.0-slim AS base
WORKDIR /app

FROM base AS deps
COPY package.json package-lock.json ./
RUN npm ci --production  # 确定性安装

FROM base AS build
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN npm run build

FROM base AS runtime
COPY --from=build /app/dist ./dist
COPY --from=deps /app/node_modules ./node_modules
CMD ["node", "dist/main.js"]
```

```yaml
# 基础设施即代码 (IaC) -- Terraform
resource "aws_ecs_service" "myapp" {
  name            = "myapp-${var.environment}"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.myapp.arn
  desired_count   = var.environment == "production" ? 3 : 1

  # 所有环境使用相同的任务定义，只是副本数不同
}

# 差异只在环境变量，通过配置管理
resource "aws_ssm_parameter" "db_url" {
  name  = "/${var.environment}/myapp/database_url"
  type  = "SecureString"
  value = var.database_url
}
```

### 检测方法
- 部署文档包含手动 SSH 操作步骤。
- 各环境的 Dockerfile 或基础设施配置不同。
- `package-lock.json` / `poetry.lock` 未提交到版本控制。
- 线上出现的 Bug 在开发环境无法复现。

### 修复步骤
1. 所有环境使用相同的 Docker 镜像（通过环境变量配置差异）。
2. 基础设施使用 IaC（Terraform / Pulumi / CDK）管理，入版本控制。
3. 依赖锁文件（`package-lock.json`、`poetry.lock`）必须提交。
4. 使用 `npm ci`（而非 `npm install`）确保确定性安装。
5. 禁止手动 SSH 修改生产环境配置。

### Agent Checklist
- [ ] 所有环境使用相同的 Docker 镜像
- [ ] 基础设施配置使用 IaC 并入版本控制
- [ ] 依赖锁文件已提交
- [ ] 使用 `npm ci` / `pip install -r requirements.txt` 确定性安装
- [ ] 无手动 SSH 操作流程

---

## 5. 流水线过慢（Slow Pipeline）

### 描述
CI 流水线执行时间过长（> 30 分钟），开发者不愿等待，开始绕过 CI 直接部署，或者批量合并变更导致问题难以定位。

### 错误示例
```yaml
# 串行执行所有步骤 -- 总耗时 40 分钟
pipeline:
  steps:
    - name: Install
      run: npm install          # 5 min（无缓存）
    - name: Lint
      run: npm run lint         # 3 min
    - name: Type check
      run: npm run type-check   # 3 min
    - name: Unit tests
      run: npm test             # 10 min
    - name: E2E tests
      run: npm run e2e          # 15 min（串行执行所有场景）
    - name: Build
      run: npm run build        # 5 min
    # 总计：41 分钟
```

### 正确示例
```yaml
# 并行执行 + 缓存 + 分层 -- 总耗时 15 分钟
name: CI

on: [push, pull_request]

jobs:
  # 第一层：快速检查（并行，3 分钟）
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 18
          cache: "npm"  # 依赖缓存
      - run: npm ci
      - run: npm run lint

  type-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 18, cache: "npm" }
      - run: npm ci
      - run: npm run type-check

  # 第二层：测试（并行分片，10 分钟）
  unit-test:
    needs: [lint, type-check]
    runs-on: ubuntu-latest
    strategy:
      matrix:
        shard: [1, 2, 3, 4]  # 4 个分片并行
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 18, cache: "npm" }
      - run: npm ci
      - run: npm test -- --shard=${{ matrix.shard }}/4

  e2e-test:
    needs: [lint, type-check]
    runs-on: ubuntu-latest
    strategy:
      matrix:
        browser: [chromium, firefox]  # 并行浏览器
    steps:
      - uses: actions/checkout@v4
      - run: npm ci
      - run: npx playwright test --project=${{ matrix.browser }}

  # 第三层：构建 + 部署
  deploy:
    needs: [unit-test, e2e-test]
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - run: npm run build
      - run: ./deploy.sh
```

### 检测方法
- CI 流水线平均执行时间 > 15 分钟。
- 所有步骤串行执行（无并行 Job）。
- `npm install` 每次重新下载（无缓存）。
- 开发者频繁绕过 CI 直接 merge。

### 修复步骤
1. 将独立步骤并行化（lint / type-check / security 可同时执行）。
2. 启用依赖缓存（`actions/cache` / `cache: npm`）。
3. 测试分片执行（`--shard=1/4`）。
4. 将 E2E 测试按场景分组并行。
5. 目标：CI 总耗时 < 15 分钟。

### Agent Checklist
- [ ] CI 总耗时 < 15 分钟
- [ ] 独立步骤并行执行
- [ ] 依赖安装有缓存
- [ ] 测试有分片策略
- [ ] 无开发者绕过 CI 的情况

---

## 全局 Agent Checklist

| 检查项 | 阈值 | 工具 |
|--------|------|------|
| 质量门禁覆盖 | lint + test + security | CI 配置审查 |
| `allow_failure` 滥用 | 0 处 | CI 配置审查 |
| 灰度发布 | 必须有 | 部署配置审查 |
| 自动回滚 | 必须有 | 部署配置审查 |
| 制品可追溯 | commit SHA + 版本号 | `/health` API |
| 环境一致性 | Docker + IaC | 架构审查 |
| CI 总耗时 | < 15 分钟 | CI 监控 |
