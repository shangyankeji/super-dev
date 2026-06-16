---
id: docker-complete
title: Docker 完整指南
domain: devops
category: 01-standards
difficulty: intermediate
tags: [complete, compose, devops, docker, dockerfile, 实战, 最佳实践, 核心概念]
quality_score: 70
last_updated: 2026-06-15
---
# Docker 完整指南

> 文档版本: v1.0 | 最后更新: 2026-03-28 | 适用范围: Docker 24.x / 25.x + Compose V2

---

## 目录

1. [概述](#概述)
2. [核心概念](#核心概念)
3. [Dockerfile 最佳实践](#dockerfile-最佳实践)
4. [Docker Compose 实战](#docker-compose-实战)
5. [网络模式详解](#网络模式详解)
6. [存储卷管理](#存储卷管理)
7. [安全加固](#安全加固)
8. [性能优化](#性能优化)
9. [生产部署](#生产部署)
10. [监控与日志](#监控与日志)
11. [常见陷阱与反模式](#常见陷阱与反模式)
12. [故障排查 Playbook](#故障排查-playbook)
13. [Agent Checklist](#agent-checklist)

---

## 概述

Docker 是业界标准的容器化平台，将应用程序及其全部依赖打包为轻量级、可移植的容器镜像。与虚拟机不同，容器共享宿主机内核，启动时间以毫秒计，资源开销极低。

**核心价值**：
- **环境一致性**：开发、测试、生产使用同一镜像，消除 "在我机器上能跑" 问题
- **资源效率**：容器比 VM 轻量 10-100 倍，单台主机可运行数百个容器
- **快速交付**：镜像构建秒级完成，CI/CD 流水线提速显著
- **微服务基石**：每个服务独立容器化，独立部署、独立扩缩

**适用场景**：
- Web 应用 / API 服务的标准化部署
- 微服务架构下的服务编排
- CI/CD 流水线中的构建与测试隔离
- 本地开发环境的快速搭建

**不适用场景**：
- 需要不同内核版本的工作负载（应使用 VM）
- GUI 密集型桌面应用
- 对实时性有极高要求的嵌入式系统

---

## 核心概念

### 镜像 (Image)

镜像是只读的分层文件系统模板，包含运行应用所需的一切：代码、运行时、库、环境变量、配置文件。

```bash
# 拉取官方镜像
docker pull python:3.12-slim

# 查看本地镜像
docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"

# 查看镜像层结构
docker history python:3.12-slim

# 导出/导入镜像（离线环境迁移）
docker save -o python-slim.tar python:3.12-slim
docker load -i python-slim.tar

# 删除悬空镜像（无标签的中间层）
docker image prune -f

# 删除所有未使用的镜像
docker image prune -a -f
```

**镜像命名规范**：
```
<registry>/<namespace>/<repository>:<tag>
例如: harbor.company.com/backend/user-service:v2.1.5-amd64
```

标签规则：
- 永远不要在生产环境使用 `latest` 标签
- 使用语义化版本: `v1.2.3`
- 附加构建元信息: `v1.2.3-abc1234`（git short hash）
- 多架构时附加平台: `v1.2.3-arm64`

### 容器 (Container)

容器是镜像的运行实例，拥有可写层、网络配置、挂载卷等运行时状态。

```bash
# 运行容器（前台）
docker run --rm -it python:3.12-slim python

# 运行容器（后台）
docker run -d --name my-api \
  -p 8080:8000 \
  --restart unless-stopped \
  --memory 512m \
  --cpus 1.0 \
  my-api:v1.0.0

# 查看运行中的容器
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

# 进入运行中的容器
docker exec -it my-api /bin/sh

# 查看容器日志（最近 100 行 + 实时跟踪）
docker logs --tail 100 -f my-api

# 查看容器资源使用
docker stats --no-stream

# 优雅停止容器（先 SIGTERM，10 秒后 SIGKILL）
docker stop --time 10 my-api

# 复制文件进出容器
docker cp ./config.yaml my-api:/app/config.yaml
docker cp my-api:/app/logs/error.log ./error.log
```

### 网络 (Network)

Docker 提供多种网络驱动，容器间通过网络进行通信。

```bash
# 创建自定义桥接网络
docker network create --driver bridge app-net

# 查看网络详情
docker network inspect app-net

# 将容器连接到网络
docker network connect app-net my-api

# 容器间通过容器名通信（自定义桥接网络自动 DNS）
docker run -d --name db --network app-net postgres:16
docker run -d --name api --network app-net my-api:v1.0.0
# api 容器中可直接使用 "db" 作为主机名连接数据库
```

### 存储 (Volume)

Docker 卷是持久化数据的推荐方式，生命周期独立于容器。

```bash
# 创建命名卷
docker volume create pg-data

# 挂载命名卷
docker run -d --name db \
  -v pg-data:/var/lib/postgresql/data \
  postgres:16

# 挂载主机目录（bind mount，开发用）
docker run -d --name api \
  -v $(pwd)/src:/app/src:ro \
  my-api:v1.0.0

# 查看卷信息
docker volume inspect pg-data

# 清理未使用的卷
docker volume prune -f
```

---

## Dockerfile 最佳实践

### 多阶段构建

多阶段构建是减小最终镜像体积的最有效手段，将构建依赖与运行时依赖分离。

```dockerfile
# ============ 阶段 1: 构建 ============
FROM node:20-alpine AS builder

WORKDIR /build

# 先复制依赖清单，利用缓存
COPY package.json package-lock.json ./
RUN npm ci --prefer-offline

# 再复制源码
COPY . .
RUN npm run build

# ============ 阶段 2: 运行 ============
FROM node:20-alpine AS runner

# 安全：创建非 root 用户
RUN addgroup -g 1001 appgroup && \
    adduser -u 1001 -G appgroup -D appuser

WORKDIR /app

# 只复制运行时需要的产物
COPY --from=builder --chown=appuser:appgroup /build/dist ./dist
COPY --from=builder --chown=appuser:appgroup /build/node_modules ./node_modules
COPY --from=builder --chown=appuser:appgroup /build/package.json ./

# 健康检查
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
  CMD wget -qO- http://localhost:3000/health || exit 1

USER appuser
EXPOSE 3000

CMD ["node", "dist/server.js"]
```

**Python 多阶段构建**：

```dockerfile
# ============ 阶段 1: 构建 wheel ============
FROM python:3.12-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential gcc libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY requirements.txt .
RUN pip wheel --no-cache-dir --wheel-dir /build/wheels -r requirements.txt

# ============ 阶段 2: 运行 ============
FROM python:3.12-slim AS runner

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 curl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -g 1001 appgroup && \
    useradd -u 1001 -g appgroup -m appuser

WORKDIR /app

COPY --from=builder /build/wheels /tmp/wheels
RUN pip install --no-cache-dir --no-index --find-links=/tmp/wheels /tmp/wheels/* \
    && rm -rf /tmp/wheels

COPY --chown=appuser:appgroup . .

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
  CMD curl -f http://localhost:8000/health || exit 1

USER appuser
EXPOSE 8000

CMD ["gunicorn", "app:create_app()", "-b", "0.0.0.0:8000", "-w", "4", "-k", "uvicorn.workers.UvicornWorker"]
```

### 层优化

Docker 每条指令创建一层，层越少、变更越靠后，构建缓存命中率越高。

**原则**：
1. 把变动频率低的指令放前面（系统包安装、依赖安装）
2. 把变动频率高的指令放后面（源码复制）
3. 合并多条 RUN 减少层数
4. 在同一层中清理临时文件

```dockerfile
# 反模式：每条 RUN 单独一层，且未清理缓存
RUN apt-get update
RUN apt-get install -y curl
RUN apt-get install -y git
RUN rm -rf /var/lib/apt/lists/*

# 正确做法：合并为一层并在同一层清理
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      curl \
      git \
    && rm -rf /var/lib/apt/lists/*
```

### .dockerignore

始终配置 `.dockerignore` 文件减小构建上下文，加快构建速度并防止敏感文件泄露。

```gitignore
# 版本控制
.git
.gitignore

# 依赖目录
node_modules
__pycache__
*.pyc
.venv
venv

# 构建产物
dist
build
*.egg-info

# IDE
.vscode
.idea
*.swp

# 环境与密钥
.env
.env.*
*.pem
*.key
credentials.json

# Docker
Dockerfile*
docker-compose*.yml
.dockerignore

# 文档与测试（生产镜像不需要）
docs/
tests/
*.md
LICENSE
```

### 安全原则

```dockerfile
# 1. 使用确定性基础镜像标签（带 digest 更佳）
FROM python:3.12.3-slim@sha256:abc123...

# 2. 非 root 用户运行（永远不要以 root 运行应用）
RUN groupadd -r appuser && useradd -r -g appuser appuser
USER appuser

# 3. 只读文件系统（需要写入时单独挂载 tmpfs）
# docker run --read-only --tmpfs /tmp my-app:v1

# 4. 不在镜像中存储密钥
# 反模式：
#   COPY .env /app/.env
#   ENV DB_PASSWORD=secret123
# 正确做法：运行时通过 secret/env 注入

# 5. 最小化安装包，不安装推荐包
RUN apt-get install -y --no-install-recommends <package>

# 6. 扫描镜像漏洞（构建后执行）
# docker scout cves my-app:v1
# trivy image my-app:v1
```

---

## Docker Compose 实战

### 完整应用栈示例

```yaml
# docker-compose.yml
# Compose V2 格式（不需要 version 字段）

services:
  # ---------- 前端 ----------
  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
      args:
        VITE_API_URL: http://localhost:3001
    ports:
      - "3000:80"
    depends_on:
      api:
        condition: service_healthy
    networks:
      - frontend-net
    restart: unless-stopped

  # ---------- API 服务 ----------
  api:
    build:
      context: ./backend
      dockerfile: Dockerfile
      target: runner
    ports:
      - "3001:3001"
    environment:
      NODE_ENV: production
      DATABASE_URL: postgresql://app:${DB_PASSWORD}@postgres:5432/myapp
      REDIS_URL: redis://redis:6379/0
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3001/health"]
      interval: 15s
      timeout: 5s
      retries: 5
      start_period: 30s
    networks:
      - frontend-net
      - backend-net
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: "1.0"
        reservations:
          memory: 256M
          cpus: "0.5"

  # ---------- 数据库 ----------
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: myapp
      POSTGRES_USER: app
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - pg-data:/var/lib/postgresql/data
      - ./scripts/init-db.sql:/docker-entrypoint-initdb.d/init.sql:ro
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app -d myapp"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - backend-net
    restart: unless-stopped

  # ---------- 缓存 ----------
  redis:
    image: redis:7-alpine
    command: >
      redis-server
      --maxmemory 128mb
      --maxmemory-policy allkeys-lru
      --appendonly yes
    volumes:
      - redis-data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - backend-net
    restart: unless-stopped

  # ---------- 反向代理 ----------
  nginx:
    image: nginx:1.27-alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/certs:/etc/nginx/certs:ro
    depends_on:
      frontend:
        condition: service_started
      api:
        condition: service_healthy
    networks:
      - frontend-net
    restart: unless-stopped

volumes:
  pg-data:
    driver: local
  redis-data:
    driver: local

networks:
  frontend-net:
    driver: bridge
  backend-net:
    driver: bridge
    internal: true  # 后端网络不暴露到宿主机
```

### 开发环境覆盖

```yaml
# docker-compose.override.yml（开发环境自动加载）
services:
  api:
    build:
      target: builder  # 使用包含 devDependencies 的阶段
    environment:
      NODE_ENV: development
      LOG_LEVEL: debug
    volumes:
      - ./backend/src:/app/src:delegated  # 热重载
      - /app/node_modules                  # 防止覆盖容器内的 node_modules
    command: npm run dev

  frontend:
    volumes:
      - ./frontend/src:/app/src:delegated
    command: npm run dev
    ports:
      - "3000:5173"  # Vite 开发服务器端口

  postgres:
    ports:
      - "5432:5432"  # 开发时暴露数据库端口方便调试

  redis:
    ports:
      - "6379:6379"
```

### 常用 Compose 命令

```bash
# 启动所有服务（后台）
docker compose up -d

# 只启动指定服务及其依赖
docker compose up -d api postgres redis

# 重新构建并启动
docker compose up -d --build

# 查看服务状态
docker compose ps

# 查看所有服务日志
docker compose logs -f --tail 50

# 查看单个服务日志
docker compose logs -f api

# 停止并删除容器（保留卷）
docker compose down

# 停止并删除容器 + 卷 + 网络（危险，数据库数据会丢失）
docker compose down -v

# 水平扩缩（无状态服务）
docker compose up -d --scale api=3

# 执行一次性命令
docker compose run --rm api npm run db:migrate
docker compose exec postgres psql -U app -d myapp
```

---

## 网络模式详解

### Bridge（桥接，默认）

每个容器分配独立 IP，通过 docker0 虚拟网桥通信。自定义桥接网络支持容器名 DNS 解析。

```bash
# 创建自定义桥接网络，指定子网
docker network create \
  --driver bridge \
  --subnet 172.20.0.0/16 \
  --gateway 172.20.0.1 \
  app-net

# 容器指定固定 IP（少数场景需要）
docker run -d --name api \
  --network app-net \
  --ip 172.20.0.10 \
  my-api:v1
```

### Host（主机）

容器直接使用宿主机网络栈，无网络隔离，性能最佳。适合高吞吐网络密集型应用。

```bash
# 容器直接使用宿主机网络（Linux only）
docker run -d --network host my-api:v1
# 注意：macOS/Windows 上 host 模式行为不同，不推荐使用
```

### None（无网络）

容器无网络接口，适合批处理任务、密码学计算等不需要网络的场景。

```bash
docker run --rm --network none my-batch-job:v1
```

### Overlay（跨主机）

用于 Docker Swarm 或 Kubernetes 等集群环境下跨主机容器通信。

```bash
# 初始化 Swarm（单节点测试）
docker swarm init

# 创建 overlay 网络
docker network create --driver overlay --attachable app-overlay
```

### 网络隔离策略

```yaml
# 推荐的网络分层策略
networks:
  # DMZ 层：前端 + 反向代理
  dmz:
    driver: bridge
  # 应用层：API 服务
  app:
    driver: bridge
  # 数据层：数据库 + 缓存（internal 禁止外部访问）
  data:
    driver: bridge
    internal: true
```

---

## 存储卷管理

### 卷类型对比

| 类型 | 用途 | 性能 | 持久性 | 生产推荐 |
|------|------|------|--------|----------|
| Named Volume | 数据库、持久化数据 | 高 | 持久 | 是 |
| Bind Mount | 开发热重载 | 中 | 持久 | 否 |
| tmpfs | 临时文件、缓存 | 极高 | 非持久 | 视场景 |

### 数据库卷管理

```bash
# 创建带标签的卷
docker volume create --label project=myapp --label env=prod pg-data

# 备份数据库卷
docker run --rm \
  -v pg-data:/source:ro \
  -v $(pwd)/backup:/backup \
  alpine tar czf /backup/pg-data-$(date +%Y%m%d).tar.gz -C /source .

# 恢复数据库卷
docker run --rm \
  -v pg-data:/target \
  -v $(pwd)/backup:/backup:ro \
  alpine sh -c "cd /target && tar xzf /backup/pg-data-20260328.tar.gz"

# 使用 pg_dump 逻辑备份（推荐）
docker compose exec postgres pg_dump -U app -d myapp -Fc > backup.dump

# 恢复逻辑备份
docker compose exec -T postgres pg_restore -U app -d myapp < backup.dump
```

### tmpfs 挂载

```bash
# 临时文件使用 tmpfs（不写入磁盘，容器停止即丢失）
docker run -d --name api \
  --tmpfs /tmp:rw,size=100m,mode=1777 \
  --tmpfs /app/cache:rw,size=50m \
  --read-only \
  my-api:v1
```

### 卷驱动插件

```bash
# 使用 NFS 卷（多主机共享）
docker volume create \
  --driver local \
  --opt type=nfs \
  --opt o=addr=nfs-server.example.com,rw,nfsvers=4 \
  --opt device=:/exports/app-data \
  shared-data
```

---

## 安全加固

### 非 root 用户运行

```dockerfile
# Node.js 应用
FROM node:20-alpine

# 创建专用用户和组
RUN addgroup -g 1001 -S appgroup && \
    adduser -u 1001 -S appuser -G appgroup

# 创建必要目录并设置权限
RUN mkdir -p /app/logs /app/tmp && \
    chown -R appuser:appgroup /app

WORKDIR /app

COPY --chown=appuser:appgroup package*.json ./
RUN npm ci --omit=dev

COPY --chown=appuser:appgroup . .

# 切换到非 root 用户
USER appuser

CMD ["node", "src/server.js"]
```

### 镜像扫描

```bash
# Docker Scout（Docker Desktop 内置）
docker scout cves my-api:v1
docker scout recommendations my-api:v1

# Trivy（CI/CD 推荐）
trivy image --severity HIGH,CRITICAL my-api:v1

# Trivy 集成到 CI（失败阈值）
trivy image --exit-code 1 --severity CRITICAL my-api:v1

# Grype
grype my-api:v1

# CI 中的扫描示例（GitHub Actions）
# - name: Scan image
#   uses: aquasecurity/trivy-action@master
#   with:
#     image-ref: my-api:v1
#     severity: CRITICAL,HIGH
#     exit-code: 1
```

### Secrets 管理

```yaml
# docker-compose.yml 中使用 secrets
services:
  api:
    image: my-api:v1
    secrets:
      - db_password
      - jwt_secret
    environment:
      # 通过文件路径引用 secret
      DB_PASSWORD_FILE: /run/secrets/db_password
      JWT_SECRET_FILE: /run/secrets/jwt_secret

secrets:
  db_password:
    file: ./secrets/db_password.txt    # 开发环境
  jwt_secret:
    file: ./secrets/jwt_secret.txt
```

```python
# 应用内读取 secret 文件的通用模式
import os

def get_secret(name: str) -> str:
    """优先从 Docker secret 文件读取，回退到环境变量。"""
    file_path = os.environ.get(f"{name}_FILE")
    if file_path and os.path.exists(file_path):
        with open(file_path, "r") as f:
            return f.read().strip()
    value = os.environ.get(name)
    if value:
        return value
    raise RuntimeError(f"Secret {name} not found in file or env")
```

### 运行时安全

```bash
# 1. 只读文件系统
docker run --read-only --tmpfs /tmp my-api:v1

# 2. 限制 capabilities（移除所有，仅添加必要的）
docker run --cap-drop ALL --cap-add NET_BIND_SERVICE my-api:v1

# 3. 禁止权限提升
docker run --security-opt no-new-privileges my-api:v1

# 4. 限制系统调用（seccomp）
docker run --security-opt seccomp=./seccomp-profile.json my-api:v1

# 5. 禁止容器内挂载 Docker socket
# 绝对不要: -v /var/run/docker.sock:/var/run/docker.sock

# 6. 综合安全运行示例
docker run -d --name api \
  --read-only \
  --tmpfs /tmp:rw,noexec,nosuid,size=100m \
  --cap-drop ALL \
  --cap-add NET_BIND_SERVICE \
  --security-opt no-new-privileges:true \
  --memory 512m \
  --cpus 1.0 \
  --pids-limit 100 \
  --user 1001:1001 \
  my-api:v1
```

### Docker Content Trust

```bash
# 启用镜像签名验证
export DOCKER_CONTENT_TRUST=1

# 签名并推送镜像
docker push harbor.company.com/backend/api:v1.0.0

# 启用后，未签名镜像无法拉取或运行
```

---

## 性能优化

### 构建性能

```bash
# 1. 使用 BuildKit（Docker 23+ 默认启用）
export DOCKER_BUILDKIT=1

# 2. 并行多阶段构建（BuildKit 自动并行无依赖的阶段）
docker build --progress=plain -t my-app:v1 .

# 3. 缓存挂载（避免重复下载依赖）
# Dockerfile 中使用 --mount=type=cache
```

```dockerfile
# 利用 BuildKit 缓存挂载加速依赖安装
FROM python:3.12-slim

WORKDIR /app

COPY requirements.txt .

# pip 缓存挂载 — 不同构建间共享下载缓存
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install -r requirements.txt

COPY . .

CMD ["python", "-m", "uvicorn", "app:app", "--host", "0.0.0.0"]
```

```dockerfile
# Go 应用的缓存挂载示例
FROM golang:1.22 AS builder

WORKDIR /build

COPY go.mod go.sum ./
RUN --mount=type=cache,target=/go/pkg/mod \
    go mod download

COPY . .
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    CGO_ENABLED=0 go build -o /app/server ./cmd/server

FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/server /server
CMD ["/server"]
```

### 运行时性能

```bash
# 资源限制配置
docker run -d --name api \
  --memory 1g \           # 内存硬限制
  --memory-swap 1g \      # 禁止 swap（等于 --memory）
  --cpus 2.0 \            # CPU 限制（2 核）
  --cpu-shares 1024 \     # CPU 权重（相对优先级）
  --pids-limit 200 \      # 进程数限制
  --ulimit nofile=65535:65535 \  # 文件描述符限制
  my-api:v1
```

### 镜像体积优化

```bash
# 体积对比：选择合适的基础镜像
# python:3.12        -> ~1.0 GB
# python:3.12-slim   -> ~150 MB
# python:3.12-alpine -> ~50 MB（注意 musl 兼容性）

# 分析镜像体积
docker images my-api --format "{{.Size}}"

# 使用 dive 分析每一层
dive my-api:v1
```

**Alpine vs Slim 选择指南**：
- **选 slim**：需要 glibc、有 C 扩展依赖（pandas, numpy, psycopg2）、时区数据
- **选 alpine**：纯静态二进制（Go）、极简 Node.js 应用、对镜像体积有极端要求
- **选 distroless**：生产环境最终镜像、不需要 shell 的场景、安全优先

---

## 生产部署

### 健康检查

```dockerfile
# HTTP 健康检查
HEALTHCHECK --interval=30s --timeout=5s --start-period=60s --retries=3 \
  CMD curl -f http://localhost:8000/health || exit 1

# TCP 健康检查（无 curl 的镜像）
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
  CMD wget -qO- http://localhost:8000/health || exit 1

# 数据库健康检查
HEALTHCHECK --interval=10s --timeout=5s --retries=5 \
  CMD pg_isready -U app -d myapp || exit 1
```

### 优雅停机

```javascript
// Node.js 优雅停机示例
const server = app.listen(3000);

const shutdown = async (signal) => {
  console.log(`Received ${signal}, shutting down gracefully...`);

  // 停止接收新请求
  server.close(() => {
    console.log('HTTP server closed');
  });

  // 关闭数据库连接
  await db.disconnect();

  // 关闭 Redis 连接
  await redis.quit();

  process.exit(0);
};

process.on('SIGTERM', () => shutdown('SIGTERM'));
process.on('SIGINT', () => shutdown('SIGINT'));
```

```python
# Python (FastAPI + Uvicorn) 优雅停机
import signal
import asyncio
from contextlib import asynccontextmanager

@asynccontextmanager
async def lifespan(app):
    # 启动时：初始化连接池
    await db_pool.connect()
    await redis_pool.connect()
    yield
    # 关闭时：释放资源
    await db_pool.disconnect()
    await redis_pool.disconnect()

app = FastAPI(lifespan=lifespan)
```

### 零停机部署

```yaml
# docker-compose 滚动更新（蓝绿部署简易版）
services:
  api:
    image: my-api:${VERSION:-latest}
    deploy:
      replicas: 2
      update_config:
        parallelism: 1        # 每次更新 1 个副本
        delay: 30s             # 更新间隔
        failure_action: rollback
        order: start-first     # 先启动新副本再停旧副本
      rollback_config:
        parallelism: 0
        order: stop-first
```

```bash
# 手动蓝绿部署脚本
#!/bin/bash
set -euo pipefail

NEW_VERSION=$1
OLD_CONTAINER="api-blue"
NEW_CONTAINER="api-green"

# 启动新版本
docker run -d --name $NEW_CONTAINER \
  --network app-net \
  my-api:$NEW_VERSION

# 等待健康检查通过
echo "Waiting for health check..."
for i in $(seq 1 30); do
  if docker exec $NEW_CONTAINER curl -sf http://localhost:8000/health > /dev/null 2>&1; then
    echo "New container is healthy"
    break
  fi
  if [ $i -eq 30 ]; then
    echo "Health check failed, rolling back"
    docker rm -f $NEW_CONTAINER
    exit 1
  fi
  sleep 2
done

# 切换流量（更新 nginx upstream）
docker exec nginx nginx -s reload

# 优雅停止旧容器
docker stop --time 30 $OLD_CONTAINER
docker rm $OLD_CONTAINER
```

### 多架构构建

```bash
# 创建多架构构建器
docker buildx create --name multiarch --use

# 构建并推送多架构镜像
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t harbor.company.com/backend/api:v1.0.0 \
  --push .

# 查看多架构信息
docker manifest inspect harbor.company.com/backend/api:v1.0.0
```

---

## 监控与日志

### 日志驱动配置

```json
// /etc/docker/daemon.json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "50m",
    "max-file": "5",
    "compress": "true"
  }
}
```

```yaml
# docker-compose.yml 中配置日志
services:
  api:
    image: my-api:v1
    logging:
      driver: json-file
      options:
        max-size: "50m"
        max-file: "5"
        tag: "{{.Name}}/{{.ID}}"
```

### 结构化日志

```python
# Python 结构化日志输出到 stdout（Docker 日志采集推荐方式）
import json
import logging
import sys

class JSONFormatter(logging.Formatter):
    def format(self, record):
        log_entry = {
            "timestamp": self.formatTime(record),
            "level": record.levelname,
            "message": record.getMessage(),
            "module": record.module,
            "function": record.funcName,
        }
        if record.exc_info:
            log_entry["exception"] = self.formatException(record.exc_info)
        return json.dumps(log_entry, ensure_ascii=False)

handler = logging.StreamHandler(sys.stdout)
handler.setFormatter(JSONFormatter())
logger = logging.getLogger("app")
logger.addHandler(handler)
logger.setLevel(logging.INFO)
```

### Prometheus 指标采集

```yaml
# docker-compose.yml 监控栈
services:
  prometheus:
    image: prom/prometheus:v2.51.0
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=30d'
    networks:
      - monitoring

  grafana:
    image: grafana/grafana:10.4.0
    ports:
      - "3100:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: ${GRAFANA_PASSWORD}
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/dashboards:/etc/grafana/provisioning/dashboards:ro
    networks:
      - monitoring

  cadvisor:
    image: gcr.io/cadvisor/cadvisor:v0.49.1
    ports:
      - "8080:8080"
    volumes:
      - /:/rootfs:ro
      - /var/run:/var/run:ro
      - /sys:/sys:ro
      - /var/lib/docker/:/var/lib/docker:ro
    networks:
      - monitoring

volumes:
  prometheus-data:
  grafana-data:

networks:
  monitoring:
    driver: bridge
```

```yaml
# monitoring/prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']

  - job_name: 'api'
    static_configs:
      - targets: ['api:8000']
    metrics_path: /metrics
```

### Docker 事件监控

```bash
# 实时监控 Docker 事件
docker events --filter type=container --format '{{.Time}} {{.Action}} {{.Actor.Attributes.name}}'

# 监控 OOM Kill 事件
docker events --filter event=oom

# 查看容器资源使用（单次快照）
docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}\t{{.BlockIO}}"
```

---

## 常见陷阱与反模式

### 1. 使用 latest 标签

```bash
# 反模式：不可复现的部署
docker pull myapp:latest
# latest 随时可能变化，今天和明天拉到的可能是不同版本

# 正确做法：固定版本标签
docker pull myapp:v2.1.5
# 或更严格：使用 digest
docker pull myapp@sha256:abc123...
```

### 2. 在镜像中嵌入 secrets

```dockerfile
# 反模式：密钥硬编码或复制进镜像
ENV DATABASE_URL=postgresql://user:password@host/db
COPY .env /app/.env
# 任何人 docker history 都能看到

# 正确做法：运行时注入
# docker run -e DATABASE_URL_FILE=/run/secrets/db_url ...
# 或使用 Docker secrets / 外部密钥管理
```

### 3. 以 root 运行容器

```dockerfile
# 反模式：默认 root 运行（不写 USER 指令）
FROM node:20
COPY . /app
CMD ["node", "server.js"]
# 容器内进程以 root 运行，逃逸后直接获得宿主 root 权限

# 正确做法：显式非 root 用户
FROM node:20
RUN addgroup --system app && adduser --system --ingroup app app
USER app
COPY --chown=app:app . /app
CMD ["node", "server.js"]
```

### 4. 构建上下文过大

```bash
# 反模式：无 .dockerignore，发送整个项目目录（含 node_modules、.git 等）
# 构建时 "Sending build context to Docker daemon" 显示 500MB+

# 正确做法：配置 .dockerignore 精准排除
# 构建上下文应控制在 10MB 以内
```

### 5. 单层安装依赖不清理

```dockerfile
# 反模式：临时文件残留在镜像层中
RUN apt-get update
RUN apt-get install -y build-essential
RUN make install
RUN apt-get remove -y build-essential  # 虽然删了，但前面的层仍然占空间

# 正确做法：同一 RUN 指令中清理
RUN apt-get update && \
    apt-get install -y --no-install-recommends build-essential && \
    make install && \
    apt-get purge -y build-essential && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*
```

### 6. 容器内数据不持久化

```bash
# 反模式：数据库数据存在容器可写层
docker run -d postgres:16
# 容器删除后数据全部丢失

# 正确做法：使用命名卷
docker run -d -v pg-data:/var/lib/postgresql/data postgres:16
```

### 7. 忽略健康检查

```yaml
# 反模式：Compose 中不配置 healthcheck 和 depends_on condition
services:
  api:
    depends_on:
      - postgres  # 仅等待容器启动，不等数据库就绪
# API 启动时数据库可能尚未初始化完成，导致连接失败

# 正确做法：见上方 Compose 示例中的 healthcheck + condition
```

### 8. 直接挂载 Docker Socket

```bash
# 反模式：给应用容器挂载 Docker socket
docker run -v /var/run/docker.sock:/var/run/docker.sock myapp
# 等同于给容器 root 权限控制整个 Docker daemon

# 正确做法：如果必须管理容器，使用受限的 Docker API 代理
# 例如 Tecnativa/docker-socket-proxy
```

---

## 故障排查 Playbook

### 容器无法启动

```bash
# 1. 查看容器退出码和状态
docker ps -a --filter name=my-api
# Exit code 0: 正常退出
# Exit code 1: 应用错误
# Exit code 137: OOM Killed (128 + 9 = SIGKILL)
# Exit code 139: Segfault
# Exit code 143: SIGTERM

# 2. 查看容器日志
docker logs my-api 2>&1 | tail -50

# 3. 检查容器详细信息
docker inspect my-api --format '{{.State.ExitCode}} {{.State.Error}}'

# 4. 以交互模式调试（覆盖入口点）
docker run --rm -it --entrypoint /bin/sh my-api:v1
```

### OOM Killed

```bash
# 1. 确认是否 OOM
docker inspect my-api --format '{{.State.OOMKilled}}'

# 2. 查看内存限制
docker stats --no-stream my-api

# 3. 查看内核 OOM 日志
dmesg | grep -i "oom\|killed"

# 4. 解决方案
# a. 增加内存限制
docker update --memory 1g --memory-swap 1g my-api
# b. 优化应用内存使用（检查内存泄漏）
# c. 设置合理的 JVM/Node.js 堆大小
#    Node.js: --max-old-space-size=768
#    JVM: -Xmx768m
```

### 网络问题

```bash
# 1. 检查容器网络配置
docker inspect my-api --format '{{json .NetworkSettings.Networks}}' | jq .

# 2. 容器间连通性测试
docker exec my-api ping -c 3 postgres
docker exec my-api nslookup postgres

# 3. 检查端口映射
docker port my-api
netstat -tlnp | grep docker  # 或 ss -tlnp

# 4. 抓包分析
docker exec my-api tcpdump -i eth0 -nn port 5432 -c 20

# 5. 检查 DNS 解析
docker exec my-api cat /etc/resolv.conf

# 6. 检查防火墙/iptables 规则
iptables -L -n -t nat | grep DOCKER
```

### 磁盘空间不足

```bash
# 1. 查看 Docker 磁盘使用
docker system df
docker system df -v

# 2. 分步清理（从安全到激进）
# a. 清理已停止的容器
docker container prune -f
# b. 清理悬空镜像
docker image prune -f
# c. 清理未使用的网络
docker network prune -f
# d. 清理未使用的卷（危险！确认无重要数据）
docker volume prune -f

# 3. 一键清理（不清理卷）
docker system prune -f

# 4. 一键清理（含卷，危险）
docker system prune -a --volumes -f

# 5. 查看 Docker 数据目录大小
du -sh /var/lib/docker/
du -sh /var/lib/docker/overlay2/
```

### 构建缓存失效

```bash
# 1. 查看构建缓存
docker builder prune --dry-run

# 2. 检查 .dockerignore 是否正确
# 确保频繁变动的文件被排除

# 3. 重新组织 Dockerfile 层顺序
# 把 COPY package.json 和 RUN npm ci 放在 COPY . . 之前

# 4. 使用外部缓存源（CI 环境）
docker buildx build \
  --cache-from type=registry,ref=harbor.company.com/cache/my-api \
  --cache-to type=registry,ref=harbor.company.com/cache/my-api,mode=max \
  -t my-api:v1 .
```

### 容器进程僵死

```bash
# 1. 检查进程状态
docker top my-api

# 2. 检查是否有僵尸进程
docker exec my-api ps aux | grep Z

# 3. 使用 tini 作为 init 进程（推荐）
# Dockerfile 中:
# RUN apk add --no-cache tini
# ENTRYPOINT ["/sbin/tini", "--"]
# CMD ["node", "server.js"]

# 4. 或使用 Docker 内置 init
docker run --init my-api:v1
```

---

## Agent Checklist

以下是 Agent 在项目中使用 Docker 时必须检查的要点。每次涉及 Docker 相关的文件变更时，逐项验证。

### Dockerfile 检查

- [ ] 基础镜像使用固定版本标签（禁止 `latest`）
- [ ] 使用多阶段构建分离构建依赖与运行时
- [ ] 最终阶段使用非 root 用户运行（`USER` 指令存在且在末尾）
- [ ] `HEALTHCHECK` 指令已配置
- [ ] `.dockerignore` 存在且覆盖 `.git`、`node_modules`、`__pycache__`、`.env`、`*.key`
- [ ] 依赖安装指令在源码复制之前（缓存优化）
- [ ] 同一 `RUN` 层中清理 apt 缓存和临时文件
- [ ] 未在 Dockerfile 中硬编码任何密钥、密码、Token
- [ ] 使用 `--no-install-recommends` 减小体积
- [ ] `COPY` 指令使用 `--chown` 设置正确所有者

### Docker Compose 检查

- [ ] 所有有状态服务（数据库、缓存）使用命名卷
- [ ] 关键服务配置 `healthcheck` 和 `depends_on.condition`
- [ ] 网络按层划分（前端/后端/数据层），数据层使用 `internal: true`
- [ ] 资源限制已配置（`deploy.resources.limits`）
- [ ] 密钥通过 `secrets` 或环境变量文件注入，不硬编码在 compose 文件
- [ ] 日志驱动配置了 `max-size` 和 `max-file` 防止磁盘爆满
- [ ] `restart: unless-stopped` 或 `restart: always` 已设置

### 安全检查

- [ ] 镜像构建后执行漏洞扫描（trivy/scout）
- [ ] 未挂载 Docker socket 到应用容器
- [ ] 生产容器使用 `--cap-drop ALL`，仅 `--cap-add` 必要权限
- [ ] 生产容器设置 `--security-opt no-new-privileges`
- [ ] 未在镜像层中包含 `.env`、`*.pem`、`*.key` 等敏感文件

### 生产部署检查

- [ ] CI/CD 流水线集成镜像扫描步骤
- [ ] 健康检查端点返回依赖状态（数据库、缓存连接）
- [ ] 应用实现优雅停机（处理 SIGTERM）
- [ ] 日志输出到 stdout/stderr（不写文件）
- [ ] 内存限制与应用堆大小一致（避免 OOM）
- [ ] 数据卷有定期备份策略
- [ ] 使用 `--init` 或 tini 防止僵尸进程

---

> 文档版本: v1.0 | 最后更新: 2026-03-28
