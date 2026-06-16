---
id: github-actions-complete
title: GitHub Actions完整指南
domain: devops
category: 01-standards
difficulty: intermediate
tags: [actions, complete, devops, github, 学习路径, 常用actions, 最佳实践, 核心概念]
quality_score: 70
last_updated: 2026-06-15
---
# GitHub Actions完整指南

## 概述
GitHub Actions是GitHub的CI/CD平台,自动化构建、测试、部署。本指南覆盖工作流语法、常用操作、 secrets管理和最佳实践。

## 核心概念

### 1. 工作流语法

**基础工作流**:
```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Python
      uses: actions/setup-python@v5
      with:
        python-version: '3.11'
    
    - name: Install dependencies
      run: |
        pip install -r requirements.txt
        pip install pytest
    
    - name: Run tests
      run: pytest tests/
```

### 2. 触发条件

```yaml
on:
  # 推送时触发
  push:
    branches: [ main ]
    paths:
      - 'src/**'
      - 'tests/**'
  
  # PR时触发
  pull_request:
    branches: [ main ]
  
  # 定时触发
  schedule:
    - cron: '0 2 * * *'  # 每天凌晨2点
  
  # 手动触发
  workflow_dispatch:
    inputs:
      environment:
        description: 'Deployment environment'
        required: true
        default: 'staging'
  
  # 其他工作流完成时触发
  workflow_run:
    workflows: ["Build"]
    types: [completed]
```

### 3. Jobs和Steps

**Job依赖**:
```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Building..."
  
  test:
    needs: build  # 等待build完成
    runs-on: ubuntu-latest
    steps:
      - run: echo "Testing..."
  
  deploy:
    needs: [build, test]  # 等待多个job
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploying..."
```

**条件执行**:
```yaml
jobs:
  deploy:
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploy to production"
```

### 4. 矩阵构建

```yaml
jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        python-version: ['3.9', '3.10', '3.11']
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Python ${{ matrix.python-version }}
      uses: actions/setup-python@v5
      with:
        python-version: ${{ matrix.python-version }}
    
    - run: python --version
```

### 5. Secrets和环境变量

```yaml
jobs:
  deploy:
    runs-on: ubuntu-latest
    
    steps:
    - name: Deploy to AWS
      env:
        AWS_ACCESS_KEY: ${{ secrets.AWS_ACCESS_KEY }}
        AWS_SECRET_KEY: ${{ secrets.AWS_SECRET_KEY }}
      run: |
        aws s3 sync ./dist s3://my-bucket
```

**设置Secrets**:
1. GitHub仓库 → Settings → Secrets and variables → Actions
2. New repository secret
3. Name: AWS_ACCESS_KEY
4. Value: your-access-key

### 6. 缓存

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Cache pip dependencies
      uses: actions/cache@v3
      with:
        path: ~/.cache/pip
        key: ${{ runner.os }}-pip-${{ hashFiles('requirements.txt') }}
        restore-keys: |
          ${{ runner.os }}-pip-
    
    - run: pip install -r requirements.txt
```

### 7. Artifacts

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Build
      run: |
        npm run build
        tar -czf dist.tar.gz dist/
    
    - name: Upload artifact
      uses: actions/upload-artifact@v4
      with:
        name: dist
        path: dist.tar.gz
  
  deploy:
    needs: build
    runs-on: ubuntu-latest
    steps:
    - name: Download artifact
      uses: actions/download-artifact@v4
      with:
        name: dist
    
    - run: tar -xzf dist.tar.gz
```

### 8. Docker构建和推送

```yaml
jobs:
  docker:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Login to Docker Hub
      uses: docker/login-action@v3
      with:
        username: ${{ secrets.DOCKERHUB_USERNAME }}
        password: ${{ secrets.DOCKERHUB_TOKEN }}
    
    - name: Build and push
      uses: docker/build-push-action@v5
      with:
        context: .
        push: true
        tags: myapp:${{ github.sha }}
```

## 常用Actions

**检查代码**:
```yaml
- uses: actions/checkout@v4
```

**设置Python**:
```yaml
- uses: actions/setup-python@v5
  with:
    python-version: '3.11'
```

**设置Node.js**:
```yaml
- uses: actions/setup-node@v4
  with:
    node-version: '18'
    cache: 'npm'
```

**运行shell脚本**:
```yaml
- name: Run script
  run: bash scripts/deploy.sh
```

## 最佳实践

### ✅ DO

1. **使用具体的版本标签**
```yaml
# ✅ 好
- uses: actions/checkout@v4

# ❌ 差
- uses: actions/checkout@main
```

2. **缓存依赖**
```yaml
- uses: actions/cache@v3
  with:
    path: ~/.cache/pip
    key: ${{ runner.os }}-pip-${{ hashFiles('requirements.txt') }}
```

3. **使用secrets**
```yaml
env:
  API_KEY: ${{ secrets.API_KEY }}
```

### ❌ DON'T

1. **不要在日志中暴露secrets**
```yaml
# ❌ 危险
- run: echo ${{ secrets.API_KEY }}
```

2. **不要在PR上运行部署**
```yaml
# ❌ 差
on: pull_request

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: deploy-to-production
```

## 学习路径

### 初级 (1周)
1. 基础工作流
2. 触发条件
3. 常用actions

### 中级 (1-2周)
1. 矩阵构建
2. Secrets管理
3. Artifacts

### 高级 (2-3周)
1. 自定义actions
2. 复杂工作流
3. 安全最佳实践

---

**知识ID**: `github-actions-complete`  
**领域**: devops  
**类型**: standards  
**难度**: intermediate  
**质量分**: 93  
**维护者**: devops-team@superdev.com  
**最后更新**: 2026-03-28
