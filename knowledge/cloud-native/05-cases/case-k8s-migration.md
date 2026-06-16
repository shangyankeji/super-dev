---
title: 案例：Kubernetes 迁移
version: 1.0.0
last_updated: 2025-03-20
owner: platform-team
tags: [kubernetes, migration, case-study]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：Kubernetes 迁移实战案例
# 作用：分享传统应用到 Kubernetes 的迁移经验
# 创建时间：2025-03-20
# 最后修改：2025-03-20

## 背景

某电商平台原有系统运行在传统虚拟机上，面临以下问题：
- 部署周期长（2-3 天）
- 资源利用率低（平均 20%）
- 扩容响应慢（需要 30 分钟）
- 运维成本高（每季度 50 万）

**目标**：迁移到 Kubernetes，实现：
- 部署周期缩短到分钟级
- 资源利用率提升到 60% 以上
- 秒级弹性伸缩
- 运维成本降低 50%

## 迁移策略

采用 **绞杀者模式（Strangler Fig Pattern）**，逐步迁移：

1. **阶段 1**：搭建 K8s 集群，迁移无状态服务
2. **阶段 2**：迁移数据库和有状态服务
3. **阶段 3**：迁移遗留系统，下线虚拟机

## 实施步骤

### 第一阶段：基础设施搭建

#### 1.1 集群规划

```yaml
# 集群架构
Production Cluster:
  - Control Plane: 3 节点（高可用）
  - System Node Pool: 3 节点（系统组件）
  - General Node Pool: 10 节点（无状态应用）
  - Memory Node Pool: 5 节点（缓存/数据库）

Staging Cluster:
  - Control Plane: 1 节点
  - General Node Pool: 3 节点
```

#### 1.2 基础组件部署

```yaml
# 核心组件清单
components:
  networking:
    - CNI: Calico
    - Ingress: Nginx Ingress Controller
    - DNS: CoreDNS
    - Load Balancer: MetalLB

  storage:
    - CSI: AWS EBS CSI Driver
    - StorageClass: gp3（默认）、io2（高性能）

  security:
    - RBAC: 原生 RBAC
    - Network Policy: Calico
    - Secret: External Secrets Operator
    - Policy: OPA Gatekeeper

  observability:
    - Monitoring: Prometheus + Grafana
    - Logging: Loki + Promtail
    - Tracing: Jaeger
    - Alerting: Alertmanager

  gitops:
    - ArgoCD
    - Image Updater
```

### 第二阶段：应用迁移

#### 2.1 应用评估和分类

```yaml
# 应用分类
tier_1_critical:  # 核心业务，优先迁移
  - api-gateway
  - order-service
  - payment-service

tier_2_important:  # 重要服务
  - user-service
  - product-service
  - inventory-service

tier_3_supporting:  # 支撑服务
  - recommendation-service
  - analytics-service
```

#### 2.2 容器化改造

**改造前（虚拟机部署）：**

```bash
# 传统部署脚本
#!/bin/bash
apt-get update
apt-get install -y python3 python3-pip nginx
pip3 install -r requirements.txt
python3 manage.py migrate
service nginx start
gunicorn app:app
```

**改造后（容器化）：**

```dockerfile
# Dockerfile
FROM python:3.11-slim AS builder
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir --target=/app/deps -r requirements.txt

FROM python:3.11-slim
RUN groupadd -r appgroup && useradd -r -g appgroup appuser
WORKDIR /app
COPY --from=builder /app/deps /usr/local/lib/python3.11/site-packages
COPY --chown=appuser:appgroup . .
USER appuser
EXPOSE 8080
CMD ["gunicorn", "--bind", "0.0.0.0:8080", "app:app"]
```

```yaml
# Kubernetes 部署
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-gateway
  namespace: production
spec:
  replicas: 3
  selector:
    matchLabels:
      app: api-gateway
  template:
    metadata:
      labels:
        app: api-gateway
        version: v1.0.0
    spec:
      serviceAccountName: api-gateway-sa
      containers:
      - name: api-gateway
        image: registry.example.com/api-gateway:v1.0.0
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
        resources:
          requests:
            cpu: "500m"
            memory: "1Gi"
          limits:
            cpu: "2000m"
            memory: "2Gi"
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchLabels:
                  app: api-gateway
              topologyKey: kubernetes.io/hostname
```

#### 2.3 数据库迁移策略

**双写模式（平滑迁移）：**

```python
# 应用层双写
class DatabaseManager:
    def __init__(self):
        self.old_db = OldDatabase()
        self.new_db = NewDatabase()

    def write(self, data):
        # 写入新数据库
        try:
            self.new_db.write(data)
        except Exception as e:
            logger.error(f"New DB write failed: {e}")

        # 同时写入旧数据库（兼容期）
        self.old_db.write(data)

    def read(self, key):
        # 优先读新数据库
        result = self.new_db.read(key)
        if result:
            return result
        # 回退到旧数据库
        return self.old_db.read(key)
```

**迁移步骤：**

```bash
# 1. 部署新数据库到 K8s
kubectl apply -f postgres-statefulset.yaml

# 2. 数据同步
pg_dump -h old-db.internal -U user db_name | psql -h new-db.internal -U user db_name

# 3. 开启双写
kubectl set env deployment/api-gateway ENABLE_DUAL_WRITE=true

# 4. 验证数据一致性
python verify_data.py

# 5. 切换读流量
kubectl set env deployment/api-gateway READ_FROM_NEW_DB=true

# 6. 下线旧数据库
kubectl set env deployment/api-gateway ENABLE_DUAL_WRITE=false
```

### 第三阶段：流量切换

#### 3.1 流量分割（蓝绿发布）

```yaml
# Istio VirtualService
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: api-gateway
spec:
  hosts:
  - api.example.com
  http:
  - route:
    - destination:
        host: api-gateway
        subset: vm  # 虚拟机版本
      weight: 100
    - destination:
        host: api-gateway
        subset: k8s  # K8s 版本
      weight: 0
```

#### 3.2 渐进式切换

```bash
# 流量切换计划
# Week 1: 1% -> 观察 1 周
kubectl patch virtualservice api-gateway --type=json \
  -p='[{"op": "replace", "path": "/spec/http/0/route/0/weight", "value": 99},
       {"op": "replace", "path": "/spec/http/0/route/1/weight", "value": 1}]'

# Week 2: 10% -> 观察 3 天
# Week 3: 50% -> 观察 3 天
# Week 4: 100%

# 监控指标
- 错误率 < 0.1%
- P99 延迟 < 200ms
- 业务指标正常
```

## 遇到的问题和解决方案

### 问题 1：配置管理混乱

**现象**：应用配置分散在配置文件、环境变量、配置中心。

**解决方案**：统一使用 ConfigMap + External Secrets。

```yaml
# External Secrets 配置
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: app-secrets
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: vault-backend
  target:
    name: app-secrets
  data:
  - secretKey: database-url
    remoteRef:
      key: secret/data/production/database
      property: url
```

### 问题 2：日志采集困难

**现象**：容器日志分散，查询困难。

**解决方案**：统一日志采集到 Loki。

```yaml
# Promtail 配置
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: promtail
spec:
  template:
    spec:
      containers:
      - name: promtail
        image: grafana/promtail:latest
        args:
        - -config.file=/etc/promtail/config.yaml
        volumeMounts:
        - name: varlog
          mountPath: /var/log
        - name: config
          mountPath: /etc/promtail
      volumes:
      - name: varlog
        hostPath:
          path: /var/log
```

### 问题 3：网络策略导致服务不通

**现象**：迁移后服务间调用失败。

**解决方案**：梳理服务依赖，配置 NetworkPolicy。

```yaml
# NetworkPolicy
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: api-gateway-policy
spec:
  podSelector:
    matchLabels:
      app: api-gateway
  policyTypes:
  - Egress
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: user-service
    ports:
    - port: 8080
  - to:
    - podSelector:
        matchLabels:
          app: order-service
    ports:
    - port: 8080
```

## 迁移成果

### 指标对比

| 指标 | 迁移前 | 迁移后 | 改进 |
|------|--------|--------|------|
| 部署周期 | 2-3 天 | 10 分钟 | 99% |
| 资源利用率 | 20% | 65% | 225% |
| 扩容响应 | 30 分钟 | 30 秒 | 98% |
| 运维成本 | 50 万/季度 | 20 万/季度 | 60% |
| 故障恢复 | 2 小时 | 5 分钟 | 96% |

### 关键收益

1. **部署效率**：GitOps 自动化部署，10 分钟完成
2. **弹性伸缩**：HPA 自动扩缩，应对流量高峰
3. **成本优化**：资源利用率提升 3 倍
4. **可靠性**：故障自动恢复，SLA 提升

## 经验教训

### 成功因素

1. **渐进式迁移**：降低风险，快速验证
2. **充分测试**：每个阶段完整测试
3. **监控先行**：先建立监控，再迁移
4. **团队培训**：提前培训 K8s 技能

### 注意事项

1. **数据迁移**：提前规划数据迁移策略
2. **网络规划**：充分测试网络策略
3. **配置管理**：统一配置管理方案
4. **回滚方案**：每个阶段准备回滚计划

## 复用要点

### 迁移检查清单

- [ ] 应用评估完成
- [ ] 容器化改造完成
- [ ] K8s 清单编写
- [ ] 测试环境验证
- [ ] 监控部署完成
- [ ] 数据迁移计划
- [ ] 流量切换计划
- [ ] 回滚方案准备
- [ ] 团队培训完成
- [ ] 应急预案准备

### 关键配置模板

```yaml
# 标准化 Deployment 模板
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ${APP_NAME}
  namespace: ${NAMESPACE}
spec:
  replicas: ${REPLICAS}
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    spec:
      containers:
      - name: ${APP_NAME}
        image: ${IMAGE_REGISTRY}/${APP_NAME}:${VERSION}
        resources:
          requests:
            cpu: ${CPU_REQUEST}
            memory: ${MEMORY_REQUEST}
          limits:
            cpu: ${CPU_LIMIT}
            memory: ${MEMORY_LIMIT}
```

## 参考资料

- [Kubernetes 迁移指南](https://kubernetes.io/docs/tasks/administer-cluster/migrating-from-docker/)
- [绞杀者模式](https://martinfowler.com/bliki/StranglerFigApplication.html)
- [12-Factor App](https://12factor.net/)