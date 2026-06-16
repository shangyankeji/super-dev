---
title: Kubernetes 完整标准
version: 1.0.0
last_updated: 2025-03-20
owner: platform-team
tags: [kubernetes, container-orchestration, cloud-native]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：Kubernetes 完整开发与运维标准
# 作用：为 K8s 集群管理、应用部署、安全治理提供统一规范
# 创建时间：2025-03-20
# 最后修改：2025-03-20

## 目标

建立生产级 Kubernetes 集群的标准化管理规范，确保：
- 集群配置一致性和可重复性
- 应用部署的最佳实践
- 资源利用率和成本优化
- 安全性和合规性

## 适用场景

- 生产环境 K8s 集群规划与建设
- 应用容器化迁移
- 多集群、多环境治理
- 平台工程团队标准化

## 核心标准

### 1. 集群架构标准

#### 控制平面配置

```yaml
# 控制平面节点最低要求
apiVersion: kubeadm.k8s.io/v1beta3
kind: ClusterConfiguration
kubernetesVersion: "1.28.x"
controlPlaneEndpoint: "k8s-api.internal:6443"
etcd:
  local:
    dataDir: /var/lib/etcd
    extraArgs:
      heartbeat-interval: "500"
      election-timeout: "2500"
      snapshot-count: "10000"
apiServer:
  extraArgs:
    enable-admission-plugins: "NodeRestriction,PodSecurityPolicy,LimitRanger,ServiceAccount"
    audit-log-path: /var/log/kubernetes/audit.log
    audit-log-maxage: "30"
    audit-log-maxbackup: "10"
    audit-log-maxsize: "100"
controllerManager:
  extraArgs:
    node-cidr-mask-size: "24"
    cluster-signing-duration: "8760h"
scheduler:
  extraArgs:
    bind-address: "0.0.0.0"
```

#### 节点池规划

```yaml
# 系统节点池
apiVersion: v1
kind: Node
metadata:
  labels:
    node.kubernetes.io/instance-type: system
    node.kubernetes.io/pool: system
spec:
  taints:
  - key: node.kubernetes.io/dedicated
    value: system
    effect: NoSchedule
  capacity:
    cpu: "4"
    memory: "16Gi"

---
# 工作节点池（通用）
apiVersion: v1
kind: Node
metadata:
  labels:
    node.kubernetes.io/instance-type: general-purpose
    node.kubernetes.io/pool: general
spec:
  capacity:
    cpu: "8"
    memory: "32Gi"

---
# 计算密集型节点池
apiVersion: v1
kind: Node
metadata:
  labels:
    node.kubernetes.io/instance-type: compute-optimized
    node.kubernetes.io/pool: compute
spec:
  taints:
  - key: workload-type
    value: compute-intensive
    effect: NoSchedule
  capacity:
    cpu: "32"
    memory: "64Gi"

---
# 内存密集型节点池
apiVersion: v1
kind: Node
metadata:
  labels:
    node.kubernetes.io/instance-type: memory-optimized
    node.kubernetes.io/pool: memory
spec:
  taints:
  - key: workload-type
    value: memory-intensive
    effect: NoSchedule
  capacity:
    cpu: "16"
    memory: "256Gi"
```

### 2. 命名空间标准

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: production-app
  labels:
    environment: production
    team: backend
    cost-center: "12345"
  annotations:
    scheduler.alpha.kubernetes.io/defaultTolerations: '[]'
    scheduler.alpha.kubernetes.io/tolerationsWhitelist: '[]'
    resource-quota.enforcement: "strict"
spec:
  finalizers:
  - kubernetes
```

#### 命名空间配额

```yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: production-quota
  namespace: production-app
spec:
  hard:
    requests.cpu: "100"
    requests.memory: "200Gi"
    limits.cpu: "200"
    limits.memory: "400Gi"
    persistentvolumeclaims: "50"
    pods: "200"
    services: "50"
    secrets: "100"
    configmaps: "100"
  scopeSelector:
    matchExpressions:
    - operator: In
      scopeName: PriorityClass
      values:
      - high-priority
      - medium-priority
```

### 3. 工作负载标准

#### Pod 配置规范

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: application-pod
  namespace: production-app
  labels:
    app: sample-app
    version: v1.2.3
    tier: backend
spec:
  serviceAccountName: app-service-account
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    runAsGroup: 1000
    fsGroup: 1000
    seccompProfile:
      type: RuntimeDefault
  containers:
  - name: app
    image: registry.internal/app:v1.2.3
    ports:
    - containerPort: 8080
      protocol: TCP
    resources:
      requests:
        cpu: "500m"
        memory: "1Gi"
      limits:
        cpu: "2000m"
        memory: "4Gi"
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop:
        - ALL
    env:
    - name: LOG_LEVEL
      value: "info"
    - name: DATABASE_URL
      valueFrom:
        secretKeyRef:
          name: db-credentials
          key: url
    envFrom:
    - configMapRef:
        name: app-config
    livenessProbe:
      httpGet:
        path: /health/live
        port: 8080
      initialDelaySeconds: 30
      periodSeconds: 10
      timeoutSeconds: 5
      failureThreshold: 3
    readinessProbe:
      httpGet:
        path: /health/ready
        port: 8080
      initialDelaySeconds: 10
      periodSeconds: 5
      timeoutSeconds: 3
      failureThreshold: 3
    startupProbe:
      httpGet:
        path: /health/startup
        port: 8080
      initialDelaySeconds: 10
      periodSeconds: 5
      timeoutSeconds: 3
      failureThreshold: 30
    volumeMounts:
    - name: config
      mountPath: /etc/app/config
      readOnly: true
    - name: tmp
      mountPath: /tmp
    - name: cache
      mountPath: /var/cache/app
    lifecycle:
      preStop:
        exec:
          command: ["/bin/sh", "-c", "sleep 15"]
  volumes:
  - name: config
    configMap:
      name: app-config
  - name: tmp
    emptyDir: {}
  - name: cache
    emptyDir:
      sizeLimit: "1Gi"
  terminationGracePeriodSeconds: 60
  topologySpreadConstraints:
  - maxSkew: 1
    topologyKey: topology.kubernetes.io/zone
    whenUnsatisfiable: ScheduleAnyway
    labelSelector:
      matchLabels:
        app: sample-app
  affinity:
    podAntiAffinity:
      preferredDuringSchedulingIgnoredDuringExecution:
      - weight: 100
        podAffinityTerm:
          labelSelector:
            matchLabels:
              app: sample-app
          topologyKey: kubernetes.io/hostname
```

#### Deployment 标准

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: application
  namespace: production-app
  labels:
    app: sample-app
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: sample-app
  template:
    metadata:
      labels:
        app: sample-app
        version: v1.2.3
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      # 继承上述 Pod 配置
```

### 4. 服务与网络标准

```yaml
apiVersion: v1
kind: Service
metadata:
  name: application-service
  namespace: production-app
  labels:
    app: sample-app
spec:
  type: ClusterIP
  selector:
    app: sample-app
  ports:
  - name: http
    port: 80
    targetPort: 8080
    protocol: TCP
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: application-ingress
  namespace: production-app
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - app.example.com
    secretName: app-tls
  rules:
  - host: app.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: application-service
            port:
              number: 80
```

#### 网络策略

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: application-network-policy
  namespace: production-app
spec:
  podSelector:
    matchLabels:
      app: sample-app
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    - podSelector:
        matchLabels:
          app: another-app
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: database
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - namespaceSelector: {}
      podSelector:
        matchLabels:
          k8s-app: kube-dns
    ports:
    - protocol: UDP
      port: 53
```

### 5. 配置管理标准

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
  namespace: production-app
  labels:
    app: sample-app
data:
  LOG_LEVEL: "info"
  MAX_CONNECTIONS: "100"
  CACHE_TTL: "3600"
  config.yaml: |
    server:
      port: 8080
      timeout: 30s
    database:
      pool_size: 20
      timeout: 5s
---
apiVersion: v1
kind: Secret
metadata:
  name: db-credentials
  namespace: production-app
  labels:
    app: sample-app
type: Opaque
stringData:
  url: "postgresql://user:pass@db:5432/appdb"
  username: "app_user"
  password: "secure_password_here"
```

### 6. 持久化存储标准

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: app-data
  namespace: production-app
  labels:
    app: sample-app
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: ssd-storage
  resources:
    requests:
      storage: 100Gi
---
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: ssd-storage
provisioner: kubernetes.io/gce-pd
parameters:
  type: pd-ssd
  replication-type: regional-pd
reclaimPolicy: Retain
allowVolumeExpansion: true
volumeBindingMode: WaitForFirstConsumer
allowedTopologies:
- matchLabelExpressions:
  - key: topology.kubernetes.io/zone
    values:
    - us-central1-a
    - us-central1-b
```

### 7. 可观测性标准

```yaml
# ServiceMonitor for Prometheus
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: application-monitor
  namespace: production-app
  labels:
    app: sample-app
    release: prometheus
spec:
  selector:
    matchLabels:
      app: sample-app
  endpoints:
  - port: http
    path: /metrics
    interval: 30s
    scrapeTimeout: 10s

---
# PrometheusRule for alerting
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: application-alerts
  namespace: production-app
  labels:
    app: sample-app
    release: prometheus
spec:
  groups:
  - name: application.rules
    rules:
    - alert: HighErrorRate
      expr: |
        sum(rate(http_requests_total{status=~"5..", app="sample-app"}[5m]))
        /
        sum(rate(http_requests_total{app="sample-app"}[5m])) > 0.05
      for: 5m
      labels:
        severity: critical
      annotations:
        summary: "High error rate detected"
        description: "Error rate is {{ $value | humanizePercentage }}"
```

## 执行清单

### 集群初始化

- [ ] 配置控制平面高可用（至少 3 个 master 节点）
- [ ] 启用审计日志并配置保留策略
- [ ] 配置 RBAC 和服务账户
- [ ] 安装网络插件（Calico/Cilium）
- [ ] 配置存储类和默认 StorageClass
- [ ] 安装 metrics-server
- [ ] 配置 Pod Security Standards
- [ ] 安装 ingress controller
- [ ] 配置证书管理（cert-manager）

### 应用部署

- [ ] 定义资源请求和限制
- [ ] 配置健康检查（liveness/readiness/startup）
- [ ] 配置优雅终止（terminationGracePeriodSeconds）
- [ ] 设置 Pod 反亲和性（跨节点/可用区）
- [ ] 配置网络策略
- [ ] 设置 PodDisruptionBudget
- [ ] 配置 HorizontalPodAutoscaler
- [ ] 添加监控和日志采集

### 安全加固

- [ ] 禁用特权容器
- [ ] 强制非 root 用户运行
- [ ] 配置只读根文件系统
- [ ] 限制 capabilities
- [ ] 启用 seccomp 配置
- [ ] 配置网络策略
- [ ] 启用镜像签名验证
- [ ] 配置 secret 加密

## 最佳实践

### 1. 资源管理

```yaml
# 推荐的资源配额层级
# Tier 1: 关键服务
resources:
  requests:
    cpu: "1000m"
    memory: "2Gi"
  limits:
    cpu: "4000m"
    memory: "8Gi"

# Tier 2: 核心服务
resources:
  requests:
    cpu: "500m"
    memory: "1Gi"
  limits:
    cpu: "2000m"
    memory: "4Gi"

# Tier 3: 一般服务
resources:
  requests:
    cpu: "100m"
    memory: "256Mi"
  limits:
    cpu: "500m"
    memory: "1Gi"
```

### 2. 更新策略

```yaml
# 生产环境推荐配置
strategy:
  type: RollingUpdate
  rollingUpdate:
    maxSurge: 1          # 每次最多新增 1 个 Pod
    maxUnavailable: 0    # 不允许不可用

# 配合 PDB
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: application-pdb
  namespace: production-app
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: sample-app
```

### 3. 自动扩缩

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: application-hpa
  namespace: production-app
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: application
  minReplicas: 3
  maxReplicas: 20
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
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
```

### 4. 节点维护

```bash
# 安全驱逐 Pod 进行节点维护
kubectl cordon <node-name>
kubectl drain <node-name> --ignore-daemonsets --delete-emptydir-data --grace-period=60 --timeout=300s

# 维护完成后恢复
kubectl uncordon <node-name>
```

## 反模式

### 禁止操作

- 不设置资源限制导致资源争抢
- 使用 latest 镜像标签
- 单副本部署无反亲和性
- 特权容器运行
- 硬编码配置在镜像中
- 忽略健康检查配置
- 网络策略全开放
- Secret 明文存储在 ConfigMap

### 配置示例

```yaml
# [FAIL] 错误示例
apiVersion: v1
kind: Pod
metadata:
  name: bad-pod
spec:
  containers:
  - name: app
    image: myapp:latest  # 禁止使用 latest
    # 缺少资源限制
    # 缺少健康检查
    securityContext:
      privileged: true  # 禁止特权模式
```

## 实战案例

### 案例 1：生产集群升级

```bash
# 升级前检查
kubeadm upgrade plan

# 升级控制平面
kubeadm upgrade apply v1.28.0

# 逐个升级工作节点
kubectl drain <node> --ignore-daemonsets
kubeadm upgrade node
kubectl uncordon <node>
```

### 案例 2：资源优化

```yaml
# 使用 VPA 推荐资源
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: application-vpa
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: application
  updatePolicy:
    updateMode: "Auto"
  resourcePolicy:
    containerPolicies:
    - containerName: app
      minAllowed:
        cpu: 100m
        memory: 256Mi
      maxAllowed:
        cpu: 2000m
        memory: 4Gi
      controlledResources: ["cpu", "memory"]
```

## 检查清单

### 部署前检查

- [ ] 镜像使用确定版本标签
- [ ] 资源请求和限制已配置
- [ ] 健康检查已配置
- [ ] 网络策略已定义
- [ ] RBAC 权限最小化
- [ ] Secret 已加密存储
- [ ] 日志和监控已配置
- [ ] 配置了 PDB
- [ ] 配置了 HPA/VPA

### 运行时检查

- [ ] Pod 均匀分布在不同节点
- [ ] 资源利用率在合理范围
- [ ] 无 OOMKilled 或 CrashLoopBackOff
- [ ] 告警规则正常触发
- [ ] 日志正常采集
- [ ] 备份策略已执行

### 安全检查

- [ ] 无特权容器
- [ ] 无 hostNetwork/hostPID
- [ ] Secret 已加密
- [ ] NetworkPolicy 已生效
- [ ] RBAC 权限最小化
- [ ] 镜像无已知漏洞
- [ ] 审计日志正常

## 参考资料

- [Kubernetes 官方文档](https://kubernetes.io/docs/)
- [Kubernetes 安全最佳实践](https://kubernetes.io/docs/concepts/security/)
- [Pod Security Standards](https://kubernetes.io/docs/concepts/security/pod-security-standards/)
- [Kubernetes 网络策略](https://kubernetes.io/docs/concepts/services-networking/network-policies/)
- [CNCF 云原生 landscape](https://landscape.cncf.io/)
- [Kubernetes Pattern](https://k8spatterns.io/)