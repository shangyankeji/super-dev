---
id: kubernetes-complete
title: Kubernetes完整部署指南
domain: devops
category: 01-standards
difficulty: intermediate
tags: [complete, devops, kubernetes, 学习路径, 持久化存储, 故障排查, 最佳实践, 核心概念]
quality_score: 70
last_updated: 2026-06-15
---
# Kubernetes完整部署指南

## 概述
Kubernetes(K8s)是容器编排平台,用于自动化部署、扩展和管理容器化应用。本指南覆盖K8s核心概念、部署策略、运维最佳实践。

## 核心概念

### 1. 架构组件

**Master节点**:
- **API Server**: RESTful接口
- **etcd**: 分布式键值存储
- **Scheduler**: 调度Pod
- **Controller Manager**: 控制器

**Worker节点**:
- **Kubelet**: 节点代理
- **kube-proxy**: 网络代理
- **Container Runtime**: 容器运行时(Docker/containerd)

### 2. Pod

最小部署单元,包含一个或多个容器。

```yaml
# pod.yaml
apiVersion: v1
kind: Pod
metadata:
  name: my-app
  labels:
    app: my-app
spec:
  containers:
  - name: app
    image: my-app:v1.0
    ports:
    - containerPort: 8080
    resources:
      requests:
        memory: "128Mi"
        cpu: "100m"
      limits:
        memory: "256Mi"
        cpu: "200m"
    livenessProbe:
      httpGet:
        path: /health
        port: 8080
      initialDelaySeconds: 30
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /ready
        port: 8080
      initialDelaySeconds: 5
      periodSeconds: 5
```

### 3. Deployment

管理Pod副本和更新策略。

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: my-app
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    metadata:
      labels:
        app: my-app
    spec:
      containers:
      - name: app
        image: my-app:v1.0
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "200m"
```

**部署命令**:
```bash
# 部署
kubectl apply -f deployment.yaml

# 查看状态
kubectl get deployments
kubectl get pods

# 扩容
kubectl scale deployment my-app --replicas=5

# 更新镜像
kubectl set image deployment/my-app app=my-app:v2.0

# 回滚
kubectl rollout undo deployment/my-app
```

### 4. Service

暴露应用为网络服务。

```yaml
# ClusterIP (内部访问)
apiVersion: v1
kind: Service
metadata:
  name: my-app-service
spec:
  type: ClusterIP
  selector:
    app: my-app
  ports:
  - port: 80
    targetPort: 8080
---
# NodePort (外部访问)
apiVersion: v1
kind: Service
metadata:
  name: my-app-nodeport
spec:
  type: NodePort
  selector:
    app: my-app
  ports:
  - port: 80
    targetPort: 8080
    nodePort: 30080
---
# LoadBalancer (云负载均衡)
apiVersion: v1
kind: Service
metadata:
  name: my-app-lb
spec:
  type: LoadBalancer
  selector:
    app: my-app
  ports:
  - port: 80
    targetPort: 8080
```

### 5. ConfigMap和Secret

**ConfigMap (配置)**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
data:
  database_url: "postgres://localhost:5432/mydb"
  cache_ttl: "3600"
```

**Secret (敏感数据)**:
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secret
type: Opaque
data:
  db_password: cGFzc3dvcmQxMjM=  # base64 encoded
```

**使用**:
```yaml
apiVersion: v1
kind: Pod
metadata:
  name: my-app
spec:
  containers:
  - name: app
    image: my-app:v1.0
    env:
    - name: DATABASE_URL
      valueFrom:
        configMapKeyRef:
          name: app-config
          key: database_url
    - name: DB_PASSWORD
      valueFrom:
        secretKeyRef:
          name: app-secret
          key: db_password
```

### 6. Ingress

HTTP路由规则。

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: my-app-ingress
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
  - host: myapp.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: my-app-service
            port:
              number: 80
  tls:
  - hosts:
    - myapp.example.com
    secretName: tls-secret
```

## 持久化存储

### 1. PersistentVolume (PV)

```yaml
apiVersion: v1
kind: PersistentVolume
metadata:
  name: pv-storage
spec:
  capacity:
    storage: 10Gi
  accessModes:
    - ReadWriteOnce
  persistentVolumeReclaimPolicy: Retain
  storageClassName: standard
  hostPath:
    path: /mnt/data
```

### 2. PersistentVolumeClaim (PVC)

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: pvc-storage
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 5Gi
  storageClassName: standard
```

### 3. 使用PVC

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: my-app
spec:
  containers:
  - name: app
    image: my-app:v1.0
    volumeMounts:
    - mountPath: /data
      name: data-volume
  volumes:
  - name: data-volume
    persistentVolumeClaim:
      claimName: pvc-storage
```

## 部署策略

### 1. Rolling Update (滚动更新)

```yaml
spec:
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1        # 最多多1个Pod
      maxUnavailable: 0  # 不允许不可用
```

### 2. Blue-Green Deployment

```yaml
# Blue版本
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app-blue
spec:
  replicas: 3
  selector:
    matchLabels:
      app: my-app
      version: blue

---
# Green版本
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app-green
spec:
  replicas: 3
  selector:
    matchLabels:
      app: my-app
      version: green

---
# Service切换
apiVersion: v1
kind: Service
metadata:
  name: my-app-service
spec:
  selector:
    app: my-app
    version: blue  # 切换为green
```

### 3. Canary Deployment (金丝雀)

```yaml
# 稳定版本 (90%流量)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app-stable
spec:
  replicas: 9
  selector:
    matchLabels:
      app: my-app
      track: stable

---
# 金丝雀版本 (10%流量)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app-canary
spec:
  replicas: 1
  selector:
    matchLabels:
      app: my-app
      track: canary
```

## 监控和日志

### 1. Prometheus + Grafana

**部署Prometheus**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: prometheus
spec:
  replicas: 1
  selector:
    matchLabels:
      app: prometheus
  template:
    metadata:
      labels:
        app: prometheus
    spec:
      containers:
      - name: prometheus
        image: prom/prometheus:latest
        ports:
        - containerPort: 9090
        volumeMounts:
        - name: config
          mountPath: /etc/prometheus
      volumes:
      - name: config
        configMap:
          name: prometheus-config
```

**应用暴露指标**:
```yaml
apiVersion: v1
kind: Pod
metadata:
  name: my-app
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "8080"
    prometheus.io/path: "/metrics"
spec:
  containers:
  - name: app
    image: my-app:v1.0
    ports:
    - containerPort: 8080
```

### 2. 日志收集 (ELK/Loki)

```yaml
# Fluent Bit DaemonSet
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: fluent-bit
spec:
  selector:
    matchLabels:
      app: fluent-bit
  template:
    metadata:
      labels:
        app: fluent-bit
    spec:
      containers:
      - name: fluent-bit
        image: fluent/fluent-bit:latest
        volumeMounts:
        - name: varlog
          mountPath: /var/log
        - name: varlibdockercontainers
          mountPath: /var/lib/docker/containers
      volumes:
      - name: varlog
        hostPath:
          path: /var/log
      - name: varlibdockercontainers
        hostPath:
          path: /var/lib/docker/containers
```

## 最佳实践

### ✅ DO

1. **使用资源限制**
```yaml
resources:
  requests:
    memory: "128Mi"
    cpu: "100m"
  limits:
    memory: "256Mi"
    cpu: "200m"
```

2. **健康检查**
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
readinessProbe:
  httpGet:
    path: /ready
    port: 8080
```

3. **使用Namespace隔离**
```bash
kubectl create namespace production
kubectl apply -f deployment.yaml -n production
```

4. **镜像使用特定版本**
```yaml
image: my-app:v1.2.3  # ✅
image: my-app:latest   # ❌
```

### ❌ DON'T

1. **不要使用latest标签**
```yaml
image: my-app:latest  # ❌ 难以追踪版本
```

2. **不要在Pod中直接运行多个进程**
```yaml
# ❌ 一个Pod多个容器,耦合度高
spec:
  containers:
  - name: app
  - name: database
  - name: cache
```

3. **不要忘记资源限制**
```yaml
# ❌ 没有资源限制
spec:
  containers:
  - name: app
    image: my-app:v1.0
    # 缺少resources
```

## 故障排查

### 常用命令

```bash
# 查看Pod状态
kubectl get pods -o wide

# 查看Pod详情
kubectl describe pod my-app-12345

# 查看日志
kubectl logs my-app-12345
kubectl logs -f my-app-12345  # 实时

# 进入容器
kubectl exec -it my-app-12345 -- /bin/bash

# 查看事件
kubectl get events --sort-by='.lastTimestamp'

# 查看资源使用
kubectl top pods
kubectl top nodes
```

### 常见问题

**1. CrashLoopBackOff**:
```bash
# 检查日志
kubectl logs my-app-12345 --previous

# 检查资源限制
kubectl describe pod my-app-12345
```

**2. ImagePullBackOff**:
```bash
# 检查镜像名称和权限
kubectl describe pod my-app-12345

# 创建imagePullSecret
kubectl create secret docker-registry regcred \
  --docker-server=<your-registry-server> \
  --docker-username=<your-name> \
  --docker-password=<your-pword>
```

**3. Pending状态**:
```bash
# 检查资源
kubectl describe pod my-app-12345

# 检查节点资源
kubectl describe nodes
```

## 学习路径

### 初级 (1-2周)
1. K8s架构和核心概念
2. Pod/Deployment/Service
3. kubectl基础命令

### 中级 (2-3周)
1. ConfigMap/Secret
2. Ingress和持久化存储
3. 日志和监控

### 高级 (2-4周)
1. Helm包管理
2. RBAC和NetworkPolicy
3. 多集群管理

### 专家级 (持续)
1. Operator开发
2. 自定义调度器
3. 性能调优

## 参考资料

### 官方文档
- [K8s官方文档](https://kubernetes.io/docs/)
- [K8s API参考](https://kubernetes.io/docs/reference/)

### 教程
- [K8s教程](https://kubernetes.io/docs/tutorials/)
- [K8s最佳实践](https://kubernetes.io/docs/concepts/)

---

**知识ID**: `kubernetes-complete`  
**领域**: devops  
**类型**: standards  
**难度**: intermediate  
**质量分**: 93  
**维护者**: devops-team@superdev.com  
**最后更新**: 2026-03-28
