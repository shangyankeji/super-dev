---
id: cicd-pipeline-complete
title: CI/CD流水线完整指南
domain: devops
category: 01-standards
difficulty: intermediate
tags: [cicd, complete, devops, pipeline, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# CI/CD流水线完整指南

## 概述
CI/CD(持续集成/持续部署)自动化软件交付流程。本指南覆盖CI/CD原则、工具选择、流水线设计和最佳实践。

## 核心概念

### 1. CI/CD流程

```
代码提交 → 构建 → 测试 → 部署到测试环境 → 部署到生产环境
```

### 2. GitLab CI配置

```yaml
# .gitlab-ci.yml
stages:
  - build
  - test
  - deploy

variables:
  DOCKER_IMAGE: myapp:${CI_COMMIT_SHA}

build:
  stage: build
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker login -u $CI_REGISTRY_USER -p $CI_REGISTRY_PASSWORD $CI_REGISTRY
    - docker build -t $DOCKER_IMAGE .
    - docker push $DOCKER_IMAGE

test:
  stage: test
  image: python:3.11
  script:
    - pip install -r requirements.txt
    - pytest tests/ --cov=app --cov-report=xml
  coverage: '/TOTAL.+?(\d+%)/'

deploy_staging:
  stage: deploy
  environment:
    name: staging
    url: https://staging.example.com
  script:
    - kubectl set image deployment/myapp myapp=$DOCKER_IMAGE
  only:
    - develop

deploy_production:
  stage: deploy
  environment:
    name: production
    url: https://example.com
  script:
    - kubectl set image deployment/myapp myapp=$DOCKER_IMAGE
  only:
    - main
  when: manual
```

### 3. Jenkins Pipeline

```groovy
// Jenkinsfile
pipeline {
    agent any
    
    environment {
        DOCKER_IMAGE = 'myapp'
    }
    
    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }
        
        stage('Build') {
            steps {
                sh 'docker build -t ${DOCKER_IMAGE}:${BUILD_NUMBER} .'
            }
        }
        
        stage('Test') {
            steps {
                sh 'pytest tests/ --junitxml=test-results.xml'
            }
            post {
                always {
                    junit 'test-results.xml'
                }
            }
        }
        
        stage('Deploy') {
            when {
                branch 'main'
            }
            steps {
                sh 'kubectl set image deployment/myapp myapp=${DOCKER_IMAGE}:${BUILD_NUMBER}'
            }
        }
    }
    
    post {
        success {
            slackSend(color: 'good', message: 'Build succeeded!')
        }
        failure {
            slackSend(color: 'danger', message: 'Build failed!')
        }
    }
}
```

### 4. 蓝绿部署

```yaml
deploy_blue_green:
  stage: deploy
  script:
    # 部署到绿色环境
    - kubectl apply -f k8s/green.yaml
    
    # 等待就绪
    - kubectl rollout status deployment/myapp-green
    
    # 健康检查
    - |
      for i in {1..30}; do
        if curl -f http://green.internal/health; then
          break
        fi
        sleep 10
      done
    
    # 切换流量
    - kubectl patch service myapp -p '{"spec":{"selector":{"version":"green"}}}'
    
    # 删除蓝色环境
    - kubectl delete deployment myapp-blue
```

### 5. 金丝雀发布

```yaml
deploy_canary:
  stage: deploy
  script:
    # 部署金丝雀版本(10%流量)
    - kubectl apply -f k8s/canary.yaml
    
    # 监控指标
    - ./scripts/monitor-canary.sh
    
    # 如果指标正常,逐步增加流量
    - kubectl patch service myapp -p '{"spec":{"selector":{"canary":"true"}}}'
```

### 6. 回滚策略

```bash
# Kubernetes回滚
kubectl rollout undo deployment/myapp

# 查看历史
kubectl rollout history deployment/myapp

# 回滚到特定版本
kubectl rollout undo deployment/myapp --to-revision=2
```

## 最佳实践

### ✅ DO

1. **自动化测试**
```yaml
test:
  script:
    - pytest tests/unit/
    - pytest tests/integration/
  coverage: '/TOTAL.+?(\d+%)/'
```

2. **使用环境变量**
```yaml
variables:
  DATABASE_URL: $CI_DATABASE_URL
```

3. **添加审批门禁**
```yaml
deploy_production:
  when: manual
  only:
    - main
```

### ❌ DON'T

1. **不要跳过测试**
```yaml
# ❌ 差
deploy:
  script:
    - kubectl apply -f k8s/
```

2. **不要硬编码凭据**
```yaml
# ❌ 差
script:
  - docker login -u user -p password

# ✅ 好
script:
  - docker login -u $CI_USER -p $CI_PASSWORD
```

## 学习路径

### 初级 (1-2周)
1. CI/CD基础
2. GitLab CI配置
3. 基本流水线

### 中级 (2-3周)
1. 多环境部署
2. 蓝绿部署
3. 回滚策略

### 高级 (2-4周)
1. 金丝雀发布
2. GitOps
3. 安全集成

---

**知识ID**: `cicd-pipeline-complete`  
**领域**: devops  
**类型**: standards  
**难度**: intermediate  
**质量分**: 93  
**维护者**: devops-team@superdev.com  
**最后更新**: 2026-03-28
