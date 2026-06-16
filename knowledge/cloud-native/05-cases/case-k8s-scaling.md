---
title: 案例：Kubernetes 扩展
version: 1.0.0
last_updated: 2025-03-20
owner: platform-team
tags: [kubernetes, scaling, hpa, vpa, case-study]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：Kubernetes 自动扩展实战案例
# 作用：分享 K8s 弹性伸缩的实践经验
# 创建时间：2025-03-20
# 最后修改：2025-03-20

## 背景

某社交应用在推广活动期间，用户量从 10 万激增到 500 万，面临以下挑战：
- 流量突增 50 倍
- 响应时间从 200ms 恶化到 5 秒
- 大量请求超时失败
- 数据库连接池耗尽

**目标**：通过 Kubernetes 自动扩展，实现：
- 秒级弹性伸缩
- 自动应对流量峰值
- 成本优化（非高峰期减少资源）

## 扩展策略

采用 **多层扩展** 策略：
1. **Pod 层**：HPA/VPA 自动伸缩
2. **节点层**：Cluster Autoscaler
3. **应用层**：异步处理 + 限流

## 实施步骤

### 第一阶段：Pod 自动伸缩

#### 1.1 HPA（Horizontal Pod Autoscaler）

**CPU 基础扩缩**：

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-service-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-service
  minReplicas: 3
  maxReplicas: 100
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
      - type: Pods
        value: 2
        periodSeconds: 60
      selectPolicy: Min
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
      - type: Pods
        value: 4
        periodSeconds: 15
      selectPolicy: Max
```

**自定义指标扩缩**：

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-service-custom-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-service
  minReplicas: 3
  maxReplicas: 100
  metrics:
  # 请求数扩缩
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: 1000
  # 响应时间扩缩
  - type: Pods
    pods:
      metric:
        name: http_request_duration_p99
      target:
        type: AverageValue
        averageValue: 500m
```

**Prometheus Adapter 配置**：

```yaml
apiVersion: apiregistration.k8s.io/v1
kind: APIService
metadata:
  name: v1beta1.custom.metrics.k8s.io
spec:
  service:
    name: prometheus-adapter
    namespace: monitoring
  group: custom.metrics.k8s.io
  version: v1beta1
  insecureSkipTLSVerify: true
  groupPriorityMinimum: 100
  versionPriority: 100

---
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-adapter-config
  namespace: monitoring
data:
  config.yaml: |
    rules:
    - seriesQuery: 'http_requests_total{namespace!="",pod!=""}'
      resources:
        overrides:
          namespace:
            resource: namespace
          pod:
            resource: pod
      name:
        matches: "^(.*)_total"
        as: "${1}_per_second"
      metricsQuery: 'sum(rate(<<.Series>>{<<.LabelMatchers>>}[2m])) by (<<.GroupBy>>)'
```

#### 1.2 VPA（Vertical Pod Autoscaler）

**资源推荐模式**：

```yaml
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: api-service-vpa
  namespace: production
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-service
  updatePolicy:
    updateMode: "Auto"  # Off/Initial/Recreate/Auto
  resourcePolicy:
    containerPolicies:
    - containerName: api
      minAllowed:
        cpu: 100m
        memory: 256Mi
      maxAllowed:
        cpu: 4000m
        memory: 8Gi
      controlledResources: ["cpu", "memory"]
      controlledValues: RequestsAndLimits
```

**VPA 推荐检查脚本**：

```bash
#!/bin/bash
# 获取 VPA 推荐值
kubectl get vpa api-service-vpa -n production -o jsonpath='{.status.recommendation.containerRecommendations}'

# 输出示例
# [
#   {
#     "containerName": "api",
#     "lowerBound": {"cpu": "500m", "memory": "1Gi"},
#     "target": {"cpu": "1", "memory": "2Gi"},
#     "upperBound": {"cpu": "2", "memory": "4Gi"}
#   }
# ]
```

### 第二阶段：节点自动伸缩

#### 2.1 Cluster Autoscaler

```yaml
# Cluster Autoscaler 部署
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cluster-autoscaler
  namespace: kube-system
spec:
  selector:
    matchLabels:
      app: cluster-autoscaler
  template:
    metadata:
      labels:
        app: cluster-autoscaler
    spec:
      serviceAccountName: cluster-autoscaler
      containers:
      - name: cluster-autoscaler
        image: k8s.gcr.io/autoscaling/cluster-autoscaler:v1.27.0
        command:
        - ./cluster-autoscaler
        - --cloud-provider=aws
        - --node-group-auto-discovery=asg:tag=k8s.io/cluster-autoscaler/enabled,k8s.io/cluster-autoscaler/production
        - --scale-down-unneeded-time=10m
        - --scale-down-delay-after-add=10m
        - --scale-down-delay-after-failure=3m
        - --scale-down-delay-after-delete=10s
        - --balance-similar-node-groups
        - --expander=least-waste
        - --skip-nodes-with-system-pods=false
        env:
        - name: AWS_REGION
          value: us-east-1
        resources:
          limits:
            cpu: 100m
            memory: 600Mi
          requests:
            cpu: 100m
            memory: 600Mi
```

#### 2.2 节点池配置

```yaml
# AWS EKS 节点组
apiVersion: eksctl.io/v1alpha5
kind: ClusterConfig
metadata:
  name: production
  region: us-east-1
managedNodeGroups:
# 通用节点池
- name: general-purpose
  instanceType: m6i.xlarge
  minSize: 3
  maxSize: 20
  desiredCapacity: 5
  labels:
    node.kubernetes.io/pool: general
  tags:
    k8s.io/cluster-autoscaler/enabled: "true"
    k8s.io/cluster-autoscaler/production: "owned"

# 计算密集型节点池
- name: compute-optimized
  instanceType: c6i.2xlarge
  minSize: 0
  maxSize: 50
  desiredCapacity: 0
  labels:
    node.kubernetes.io/pool: compute
  taints:
  - key: workload-type
    value: compute-intensive
    effect: NoSchedule
  tags:
    k8s.io/cluster-autoscaler/enabled: "true"
    k8s.io/cluster-autoscaler/production: "owned"
```

### 第三阶段：应用层优化

#### 3.1 异步处理

```yaml
# 消息队列消费者 Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: message-consumer
  namespace: production
spec:
  replicas: 10  # 基础副本数
  template:
    spec:
      containers:
      - name: consumer
        image: registry.example.com/message-consumer:v1.0.0
        env:
        - name: KAFKA_BROKERS
          value: "kafka-0.kafka:9092,kafka-1.kafka:9092"
        - name: CONSUMER_GROUP
          value: "production-consumers"
        resources:
          requests:
            cpu: "200m"
            memory: "512Mi"
          limits:
            cpu: "1000m"
            memory: "1Gi"

---
# Kafka 消费者 HPA
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: message-consumer-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: message-consumer
  minReplicas: 10
  maxReplicas: 100
  metrics:
  # 基于 Kafka lag 扩缩
  - type: External
    external:
      metric:
        name: kafka_consumer_lag
        selector:
          matchLabels:
            consumer_group: production-consumers
      target:
        type: AverageValue
        averageValue: 10000
```

#### 3.2 请求限流

```yaml
# Nginx Ingress 限流配置
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: api-ingress
  namespace: production
  annotations:
    nginx.ingress.kubernetes.io/limit-connections: "100"
    nginx.ingress.kubernetes.io/limit-rps: "50"
    nginx.ingress.kubernetes.io/limit-burst: "100"
    nginx.ingress.kubernetes.io/configuration-snippet: |
      limit_req_zone $binary_remote_addr zone=api_limit:10m rate=100r/s;
      limit_req zone=api_limit burst=200 nodelay;
spec:
  rules:
  - host: api.example.com
    http:
      paths:
      - path: /
        backend:
          service:
            name: api-service
            port:
              number: 80
```

#### 3.3 缓存优化

```yaml
# Redis 缓存集群
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: redis-cluster
  namespace: production
spec:
  serviceName: redis-cluster
  replicas: 6
  selector:
    matchLabels:
      app: redis-cluster
  template:
    spec:
      containers:
      - name: redis
        image: redis:7.2-alpine
        command:
        - redis-server
        - /etc/redis/redis.conf
        ports:
        - containerPort: 6379
        resources:
          requests:
            cpu: "500m"
            memory: "2Gi"
          limits:
            cpu: "2000m"
            memory: "8Gi"
        volumeMounts:
        - name: redis-config
          mountPath: /etc/redis
      volumes:
      - name: redis-config
        configMap:
          name: redis-config
```

## 遇到的问题和解决方案

### 问题 1：HPA 震荡

**现象**：副本数频繁增减，导致服务不稳定。

**原因**：
- 扩缩阈值设置不合理
- 评估窗口过短
- 多指标冲突

**解决方案**：

```yaml
# 增加稳定窗口和行为策略
behavior:
  scaleDown:
    stabilizationWindowSeconds: 600  # 10 分钟稳定窗口
    policies:
    - type: Percent
      value: 10  # 每次最多减少 10%
      periodSeconds: 120
  scaleUp:
    stabilizationWindowSeconds: 60
    policies:
    - type: Percent
      value: 100  # 快速扩容
      periodSeconds: 15
    - type: Pods
      value: 4
      periodSeconds: 15
    selectPolicy: Max
```

### 问题 2：节点扩容延迟

**现象**：流量高峰时节点来不及扩容。

**原因**：
- 节点启动时间过长（5-8 分钟）
- 镜像拉取慢

**解决方案**：

1. **预热节点**：

```yaml
# 保持备用节点
managedNodeGroups:
- name: general-purpose
  minSize: 5  # 保持最小 5 个节点
  desiredCapacity: 8
```

2. **镜像预热**：

```yaml
# DaemonSet 预热镜像
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: image-prefetch
spec:
  template:
    spec:
      containers:
      - name: prefetch
        image: registry.example.com/prefetch:v1.0.0
        command:
        - /bin/sh
        - -c
        - |
          crictl pull registry.example.com/api-service:v1.0.0
          crictl pull registry.example.com/cache-service:v1.0.0
```

### 问题 3：数据库连接池耗尽

**现象**：Pod 扩展后数据库连接数过多。

**解决方案**：

1. **连接池配置**：

```python
# 应用层连接池配置
import sqlalchemy
from sqlalchemy.pool import QueuePool

engine = sqlalchemy.create_engine(
    DATABASE_URL,
    poolclass=QueuePool,
    pool_size=5,  # 每个 Pod 连接数
    max_overflow=10,
    pool_pre_ping=True,
    pool_recycle=3600
)
```

2. **PgBouncer 代理**：

```yaml
# PgBouncer 连接池代理
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pgbouncer
  namespace: production
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: pgbouncer
        image: edoburu/pgbouncer:latest
        env:
        - name: DATABASE_URL
          value: "postgres://user:pass@postgres:5432/db"
        - name: POOL_MODE
          value: "transaction"
        - name: MAX_CLIENT_CONN
          value: "1000"
        - name: DEFAULT_POOL_SIZE
          value: "50"
```

## 扩展成果

### 性能指标

| 指标 | 优化前 | 优化后 | 改进 |
|------|--------|--------|------|
| 扩容响应时间 | 30 分钟 | 30 秒 | 98% |
| 最大 QPS | 1,000 | 50,000 | 5000% |
| P99 延迟 | 5 秒 | 200ms | 96% |
| 故障恢复 | 手动 | 自动 | - |
| 成本效率 | 低 | 高 | 60% |

### 弹性能力

- **日常**：3 个 Pod，2 个节点
- **中等负载**：20 个 Pod，5 个节点
- **高峰负载**：100 个 Pod，20 个节点
- **极端峰值**：200 个 Pod，50 个节点

## 经验教训

### 成功因素

1. **多层扩展**：Pod + 节点 + 应用层协同
2. **监控完善**：提前建立监控告警
3. **渐进优化**：先验证再推广
4. **预案准备**：提前测试极端场景

### 注意事项

1. **扩缩策略**：扩容要快，缩容要慢
2. **资源规划**：合理设置 min/max
3. **依赖容量**：确保依赖服务可扩展
4. **成本控制**：设置上限避免成本失控

## 复用要点

### 扩展检查清单

- [ ] HPA 配置正确
- [ ] VPA 配置正确（如使用）
- [ ] Cluster Autoscaler 部署
- [ ] 节点池配置合理
- [ ] 自定义指标配置
- [ ] 监控告警配置
- [ ] 限流策略配置
- [ ] 缓存层部署
- [ ] 数据库连接池优化
- [ ] 压力测试完成

### 关键配置模板

```yaml
# 标准 HPA 模板
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: ${APP_NAME}-hpa
  namespace: ${NAMESPACE}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: ${APP_NAME}
  minReplicas: ${MIN_REPLICAS}
  maxReplicas: ${MAX_REPLICAS}
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
    scaleUp:
      stabilizationWindowSeconds: 60
```

## 参考资料

- [Kubernetes HPA 文档](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/)
- [Kubernetes VPA 文档](https://github.com/kubernetes/autoscaler/tree/master/vertical-pod-autoscaler)
- [Cluster Autoscaler](https://github.com/kubernetes/autoscaler/tree/master/cluster-autoscaler)
- [Prometheus Adapter](https://github.com/kubernetes-sigs/prometheus-adapter)