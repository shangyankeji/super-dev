---
id: docker-compose-playbook
title: Docker Compose完整指南
domain: devops
category: 02-playbooks
difficulty: intermediate
tags: [compose, devops, docker, playbook, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Docker Compose完整指南

## 概述
Docker Compose是多容器Docker应用定义和运行工具。本指南覆盖服务编排、网络、卷管理和最佳实践。

## 核心概念

### 1. 基础配置

**docker-compose.yml**:
```yaml
version: '3.8'

services:
  web:
    build: .
    ports:
      - "8000:8000"
    environment:
      - DATABASE_URL=postgresql://user:pass@db:5432/mydb
    depends_on:
      - db
      - redis
    networks:
      - backend

  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: mydb
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - backend

  redis:
    image: redis:7-alpine
    networks:
      - backend

volumes:
  postgres_data:

networks:
  backend:
```

### 2. 常用命令

```bash
# 启动服务
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down

# 重建服务
docker-compose up -d --build

# 进入容器
docker-compose exec web /bin/bash

# 运行命令
docker-compose run web python manage.py migrate

# 扩展服务
docker-compose up -d --scale worker=3
```

### 3. 环境变量

```yaml
services:
  web:
    image: myapp
    environment:
      - NODE_ENV=production
      - DB_HOST=${DB_HOST}
    env_file:
      - .env
```

### 4. 卷管理

```yaml
services:
  web:
    volumes:
      # 命名卷
      - data_volume:/app/data
      # 绑定挂载
      - ./app:/app
      # 匿名卷
      - /app/node_modules

volumes:
  data_volume:
```

### 5. 网络配置

```yaml
services:
  web:
    networks:
      - frontend
      - backend

  db:
    networks:
      - backend

networks:
  frontend:
  backend:
    internal: true  # 内部网络
```

### 6. 健康检查

```yaml
services:
  web:
    image: myapp
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

### 7. 资源限制

```yaml
services:
  web:
    image: myapp
    deploy:
      resources:
        limits:
          cpus: '0.5'
          memory: 512M
        reservations:
          cpus: '0.25'
          memory: 256M
```

## 最佳实践

### ✅ DO

1. **使用健康检查**
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost/health"]
```

2. **明确指定版本**
```yaml
image: postgres:15-alpine  # ✅ 好
image: postgres  # ❌ 差
```

3. **使用命名卷持久化数据**
```yaml
volumes:
  db_data:
```

### ❌ DON'T

1. **不要使用latest标签**
```yaml
image: myapp:latest  # ❌ 差
image: myapp:v1.2.3  # ✅ 好
```

2. **不要在compose中硬编码密码**
```yaml
# ❌ 差
environment:
  - DB_PASSWORD=mypassword

# ✅ 好
env_file:
  - .env
```

## 学习路径

### 初级 (1周)
1. 基础配置
2. 常用命令
3. 服务管理

### 中级 (1-2周)
1. 网络配置
2. 卷管理
3. 环境变量

### 高级 (2-3周)
1. 多环境配置
2. 生产部署
3. 性能调优

---

**知识ID**: `docker-compose-complete`  
**领域**: devops  
**类型**: playbooks  
**难度**: beginner  
**质量分**: 92  
**维护者**: devops-team@superdev.com  
**最后更新**: 2026-03-28
