---
title: Kubernetes 反模式库
version: 1.0.0
last_updated: 2025-03-20
owner: platform-team
tags: [kubernetes, antipatterns, best-practices]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：Kubernetes 常见反模式识别与修正
# 作用：帮助团队避免常见错误，建立正确实践
# 创建时间：2025-03-20
# 最后修改：2025-03-20

## 反模式分类

- **P0** - 严重问题，必须立即修复
- **P1** - 重要问题，应尽快修复
- **P2** - 建议改进，可计划修复

---

## 1. 资源配置反模式

### 1.1 无资源限制 [P0]

**反模式描述**：Pod 未配置资源请求和限制，导致资源争抢和节点不稳定。

```yaml
# [FAIL] 反模式
apiVersion: v1
kind: Pod
metadata:
  name: app-pod
spec:
  containers:
  - name: app
    image: myapp:v1
    # 缺少 resources 配置
```

**问题影响**：
- 节点资源耗尽导致 OOMKilled
- 资源争抢影响其他工作负载
- 无法进行容量规划
- 成本无法控制

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: v1
kind: Pod
metadata:
  name: app-pod
spec:
  containers:
  - name: app
    image: myapp:v1
    resources:
      requests:
        cpu: "500m"
        memory: "1Gi"
      limits:
        cpu: "2000m"
        memory: "4Gi"
```

### 1.2 资源配置失衡 [P1]

**反模式描述**：limits 远大于 requests，导致过度承诺。

```yaml
# [FAIL] 反模式
resources:
  requests:
    cpu: "10m"
    memory: "16Mi"
  limits:
    cpu: "16000m"    # 1600x requests
    memory: "64Gi"   # 4096x requests
```

**问题影响**：
- 节点过度承诺
- 资源竞争时性能下降
- 难以预测应用行为

**正确实践**：

```yaml
# [DONE] 正确做法
resources:
  requests:
    cpu: "500m"
    memory: "1Gi"
  limits:
    cpu: "2000m"     # 4x requests
    memory: "2Gi"    # 2x requests
```

---

## 2. 可靠性反模式

### 2.1 单副本部署 [P0]

**反模式描述**：生产环境使用单副本，无高可用保障。

```yaml
# [FAIL] 反模式
apiVersion: apps/v1
kind: Deployment
metadata:
  name: critical-app
spec:
  replicas: 1  # 单点故障
```

**问题影响**：
- 节点故障时服务中断
- 更新期间服务不可用
- 无法满足 SLA 要求

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: apps/v1
kind: Deployment
metadata:
  name: critical-app
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
```

### 2.2 缺少健康检查 [P0]

**反模式描述**：未配置 liveness/readiness 探针。

```yaml
# [FAIL] 反模式
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: app
    image: myapp:v1
    # 缺少探针配置
```

**问题影响**：
- 死锁进程无法自动重启
- 流量发送到未就绪的 Pod
- 更新期间服务中断

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: app
    image: myapp:v1
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
```

### 2.3 缺少 Pod 反亲和性 [P1]

**反模式描述**：多个副本可能调度到同一节点。

```yaml
# [FAIL] 反模式
apiVersion: apps/v1
kind: Deployment
spec:
  replicas: 3
  # 缺少反亲和性配置
```

**问题影响**：
- 节点故障影响所有副本
- 节点维护导致服务中断
- 无法实现真正的高可用

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: apps/v1
kind: Deployment
spec:
  replicas: 3
  template:
    spec:
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchLabels:
                  app: myapp
              topologyKey: kubernetes.io/hostname
      topologySpreadConstraints:
      - maxSkew: 1
        topologyKey: topology.kubernetes.io/zone
        whenUnsatisfiable: ScheduleAnyway
```

---

## 3. 安全反模式

### 3.1 以 root 运行 [P0]

**反模式描述**：容器以 root 用户运行。

```yaml
# [FAIL] 反模式
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: app
    image: myapp:v1
    # 默认以 root 运行
```

**问题影响**：
- 容器逃逸风险高
- 权限过大
- 违反最小权限原则

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: v1
kind: Pod
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    runAsGroup: 1000
    fsGroup: 1000
  containers:
  - name: app
    image: myapp:v1
    securityContext:
      allowPrivilegeEscalation: false
```

### 3.2 特权容器 [P0]

**反模式描述**：使用特权模式运行容器。

```yaml
# [FAIL] 反模式
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: app
    image: myapp:v1
    securityContext:
      privileged: true  # 危险
```

**问题影响**：
- 完全访问主机资源
- 可修改内核参数
- 严重安全风险

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: app
    image: myapp:v1
    securityContext:
      privileged: false
      capabilities:
        drop:
        - ALL
        # 仅添加必需的能力
        add:
        - NET_BIND_SERVICE  # 如需要绑定特权端口
```

### 3.3 无网络策略 [P0]

**反模式描述**：未配置 NetworkPolicy，Pod 可自由通信。

```yaml
# [FAIL] 反模式
# 无 NetworkPolicy 配置
# 所有 Pod 可相互访问
```

**问题影响**：
- 横向移动攻击风险
- 数据泄露风险
- 不符合零信任原则

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
---
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-specific
spec:
  podSelector:
    matchLabels:
      app: myapp
  policyTypes:
  - Ingress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          role: frontend
```

---

## 4. 配置管理反模式

### 4.1 使用 latest 标签 [P0]

**反模式描述**：镜像使用 latest 标签。

```yaml
# [FAIL] 反模式
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: app
    image: myapp:latest  # 不可预测
```

**问题影响**：
- 版本不可追溯
- 回滚困难
- 环境不一致

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: app
    image: myapp:v1.2.3@sha256:abc123...
```

### 4.2 配置硬编码 [P1]

**反模式描述**：配置信息硬编码在镜像或 Pod 中。

```yaml
# [FAIL] 反模式
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: app
    image: myapp:v1
    env:
    - name: DATABASE_URL
      value: "postgres://user:pass@host/db"  # 硬编码
```

**问题影响**：
- 环境差异难管理
- 敏感信息泄露
- 配置更新需重建镜像

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: app
    image: myapp:v1
    env:
    - name: DATABASE_URL
      valueFrom:
        secretKeyRef:
          name: db-credentials
          key: url
```

### 4.3 Secret 明文存储 [P0]

**反模式描述**：敏感信息存储在 ConfigMap 或明文环境变量。

```yaml
# [FAIL] 反模式
apiVersion: v1
kind: ConfigMap
data:
  password: "plaintext-password"  # 明文
```

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
stringData:
  password: "secure_password"  # etcd 已加密
```

---

## 5. 存储反模式

### 5.1 hostPath 挂载 [P0]

**反模式描述**：直接挂载主机目录。

```yaml
# [FAIL] 反模式
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: app
    volumeMounts:
    - name: host
      mountPath: /data
  volumes:
  - name: host
    hostPath:
      path: /var/data  # 安全风险
```

**问题影响**：
- 安全风险
- 节点依赖性
- 数据持久性问题

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: app
    volumeMounts:
    - name: data
      mountPath: /data
  volumes:
  - name: data
    persistentVolumeClaim:
      claimName: app-pvc
```

### 5.2 单副本 StatefulSet [P1]

**反模式描述**：有状态应用使用单副本。

```yaml
# [FAIL] 反模式
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: database
spec:
  replicas: 1  # 单点故障
```

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: database
spec:
  replicas: 3
  # 配置高可用
```

---

## 6. 网络反模式

### 6.1 hostNetwork [P0]

**反模式描述**：使用主机网络命名空间。

```yaml
# [FAIL] 反模式
apiVersion: v1
kind: Pod
spec:
  hostNetwork: true  # 安全风险
  containers:
  - name: app
    image: myapp:v1
```

**问题影响**：
- 端口冲突
- 网络隔离失效
- 安全风险

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: v1
kind: Pod
spec:
  hostNetwork: false
  containers:
  - name: app
    image: myapp:v1
    ports:
    - containerPort: 8080
```

### 6.2 NodePort 滥用 [P1]

**反模式描述**：所有服务使用 NodePort 暴露。

```yaml
# [FAIL] 反模式
apiVersion: v1
kind: Service
spec:
  type: NodePort  # 不必要的暴露
  ports:
  - port: 80
    nodePort: 30080
```

**正确实践**：

```yaml
# [DONE] 正确做法
# 内部服务使用 ClusterIP
apiVersion: v1
kind: Service
spec:
  type: ClusterIP
  ports:
  - port: 80
---
# 外部服务通过 Ingress
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: app-ingress
spec:
  rules:
  - host: app.example.com
    http:
      paths:
      - path: /
        backend:
          service:
            name: app-service
            port:
              number: 80
```

---

## 7. 更新策略反模式

### 7.1 Recreate 策略 [P1]

**反模式描述**：使用 Recreate 更新策略导致服务中断。

```yaml
# [FAIL] 反模式
apiVersion: apps/v1
kind: Deployment
spec:
  strategy:
    type: Recreate  # 服务中断
```

**问题影响**：
- 更新期间服务不可用
- 无法零停机部署

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: apps/v1
kind: Deployment
spec:
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
```

### 7.2 过激进的更新参数 [P1]

**反模式描述**：maxUnavailable 过大导致服务不稳定。

```yaml
# [FAIL] 反模式
apiVersion: apps/v1
kind: Deployment
spec:
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 50%
      maxUnavailable: 50%  # 风险过高
```

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: apps/v1
kind: Deployment
spec:
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0  # 保证可用性
```

---

## 8. 可观测性反模式

### 8.1 日志写入文件 [P1]

**反模式描述**：应用日志写入文件而非 stdout。

```yaml
# [FAIL] 反模式
# 应用写入 /var/log/app.log
# 日志采集器无法收集
```

**正确实践**：

```yaml
# [DONE] 正确做法
# 应用日志输出到 stdout/stderr
# 日志采集器自动收集
```

### 8.2 缺少监控标签 [P1]

**反模式描述**：缺少必要的标签导致监控困难。

```yaml
# [FAIL] 反模式
apiVersion: v1
kind: Pod
metadata:
  name: app
  # 缺少标签
```

**正确实践**：

```yaml
# [DONE] 正确做法
apiVersion: v1
kind: Pod
metadata:
  name: app
  labels:
    app: myapp
    version: v1.2.3
    environment: production
    team: backend
```

---

## 参考资料

- [Kubernetes 反模式](https://kubernetes.io/docs/concepts/configuration/overview/)
- [Kubernetes 最佳实践](https://kubernetes.io/docs/concepts/configuration/organize-cluster-selection-kubeconfig/)
- [Kubernetes 安全最佳实践](https://kubernetes.io/docs/concepts/security/)
- [K8s Pattern](https://k8spatterns.io/)