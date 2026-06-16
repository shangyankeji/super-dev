---
title: Kubernetes 生产就绪检查清单
version: 1.0.0
last_updated: 2025-03-20
owner: platform-team
tags: [kubernetes, production, readiness, checklist]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：Kubernetes 生产环境部署前检查清单
# 作用：确保应用具备生产环境运行条件
# 创建时间：2025-03-20
# 最后修改：2025-03-20

## 概述

本检查清单用于验证 Kubernetes 应用是否满足生产环境运行标准。在部署到生产环境前，必须完成所有必选项检查。

**检查项分类：**
- [P0] 必须完成（阻塞发布）
- [P1] 强烈建议（应在发布前完成）
- [P2] 推荐完成（可后续迭代）

---

## 1. 工作负载配置

### 1.1 镜像管理 [P0]

- [ ] **镜像版本固定**：使用确定版本标签（如 `v1.2.3`），禁止 `latest`
- [ ] **镜像签名验证**：镜像已签名，部署时验证签名
- [ ] **漏洞扫描通过**：无高危或严重漏洞
- [ ] **镜像大小合理**：镜像大小 < 500MB
- [ ] **基础镜像安全**：使用官方维护的基础镜像

```yaml
# 检查示例
spec:
  containers:
  - name: app
    image: registry.example.com/app:v1.2.3@sha256:abc123...  # [DONE] 固定版本和 digest
    # image: app:latest  # [FAIL] 禁止使用 latest
```

### 1.2 资源配置 [P0]

- [ ] **资源请求已设置**：`requests.cpu` 和 `requests.memory` 已配置
- [ ] **资源限制已设置**：`limits.cpu` 和 `limits.memory` 已配置
- [ ] **资源配额合理**：资源配置经过性能测试验证
- [ ] **资源比例适当**：limits/requests 比例合理（建议 CPU <= 4x，内存 <= 2x）

```yaml
# 检查示例
resources:
  requests:
    cpu: "500m"
    memory: "1Gi"
  limits:
    cpu: "2000m"      # 4x requests
    memory: "2Gi"     # 2x requests
```

### 1.3 健康检查 [P0]

- [ ] **存活探针配置**：`livenessProbe` 已配置且路径正确
- [ ] **就绪探针配置**：`readinessProbe` 已配置且路径正确
- [ ] **启动探针配置**：`startupProbe` 已配置（启动慢的应用）
- [ ] **探针参数合理**：超时、间隔、失败阈值配置合理
- [ ] **探针路径有效**：健康检查端点返回正确响应

```yaml
# 检查示例
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
```

### 1.4 优雅终止 [P0]

- [ ] **终止宽限期设置**：`terminationGracePeriodSeconds` 已配置
- [ ] **PreStop 钩子**：必要时配置 `preStop` 钩子
- [ ] **信号处理**：应用正确处理 SIGTERM 信号
- [ ] **连接排空**：停止前等待现有请求完成

```yaml
# 检查示例
lifecycle:
  preStop:
    exec:
      command: ["/bin/sh", "-c", "sleep 15"]
terminationGracePeriodSeconds: 60
```

---

## 2. 高可用配置

### 2.1 副本管理 [P0]

- [ ] **多副本部署**：生产环境至少 3 个副本
- [ ] **Pod 反亲和性**：配置 Pod 反亲和性跨节点分布
- [ ] **跨可用区部署**：配置 `topologySpreadConstraints` 跨可用区
- [ ] **PDB 配置**：`PodDisruptionBudget` 已配置

```yaml
# 检查示例
spec:
  replicas: 3
  topologySpreadConstraints:
  - maxSkew: 1
    topologyKey: topology.kubernetes.io/zone
    whenUnsatisfiable: ScheduleAnyway
    labelSelector:
      matchLabels:
        app: myapp
  affinity:
    podAntiAffinity:
      preferredDuringSchedulingIgnoredDuringExecution:
      - weight: 100
        podAffinityTerm:
          labelSelector:
            matchLabels:
              app: myapp
          topologyKey: kubernetes.io/hostname
```

### 2.2 更新策略 [P0]

- [ ] **滚动更新配置**：`RollingUpdate` 策略已配置
- [ ] **最大不可用**：`maxUnavailable` 设置为 0 或合理值
- [ ] **最大激增**：`maxSurge` 设置合理（建议 1 或 25%）
- [ ] **回滚方案**：定义回滚触发条件和操作步骤

```yaml
# 检查示例
strategy:
  type: RollingUpdate
  rollingUpdate:
    maxSurge: 1
    maxUnavailable: 0
```

### 2.3 自动伸缩 [P1]

- [ ] **HPA 配置**：`HorizontalPodAutoscaler` 已配置
- [ ] **伸缩指标**：CPU/内存或自定义指标配置正确
- [ ] **伸缩范围**：`minReplicas` 和 `maxReplicas` 设置合理
- [ ] **伸缩行为**：配置 `behavior` 防止频繁伸缩

```yaml
# 检查示例
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
spec:
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

---

## 3. 安全配置

### 3.1 Pod 安全 [P0]

- [ ] **非 root 运行**：`runAsNonRoot: true` 已配置
- [ ] **用户 ID 固定**：`runAsUser` 和 `runAsGroup` 已配置
- [ ] **特权提升禁止**：`allowPrivilegeEscalation: false` 已配置
- [ ] **只读根文件系统**：`readOnlyRootFilesystem: true` 已配置
- [ ] **Capabilities 限制**：`capabilities.drop: [ALL]` 已配置
- [ ] **Seccomp 配置**：`seccompProfile.type: RuntimeDefault` 已配置

```yaml
# 检查示例
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  runAsGroup: 1000
  fsGroup: 1000
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  seccompProfile:
    type: RuntimeDefault
  capabilities:
    drop:
    - ALL
```

### 3.2 网络安全 [P0]

- [ ] **网络策略配置**：`NetworkPolicy` 已配置
- [ ] **默认拒绝入站**：入站流量白名单化
- [ ] **默认拒绝出站**：出站流量白名单化
- [ ] **DNS 访问允许**：允许访问 kube-dns
- [ ] **服务间通信限制**：仅允许必要的服务通信

```yaml
# 检查示例
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
spec:
  podSelector:
    matchLabels:
      app: myapp
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          role: frontend
    ports:
    - port: 8080
```

### 3.3 密钥管理 [P0]

- [ ] **Secret 不明文**：敏感信息使用 Secret 而非 ConfigMap
- [ ] **Secret 加密**：etcd 加密已启用
- [ ] **外部密钥管理**：敏感密钥使用外部密钥管理系统
- [ ] **密钥轮换**：密钥轮换策略已定义
- [ ] **无硬编码密钥**：代码中无硬编码密钥

```yaml
# 检查示例
env:
- name: DATABASE_PASSWORD
  valueFrom:
    secretKeyRef:
      name: db-credentials
      key: password
```

### 3.4 RBAC 配置 [P0]

- [ ] **服务账户创建**：专用 ServiceAccount 已创建
- [ ] **权限最小化**：RBAC 权限遵循最小权限原则
- [ ] **禁用默认账户**：不使用 default ServiceAccount
- [ ] **命名空间隔离**：使用 Role 而非 ClusterRole（除非必要）

---

## 4. 配置管理

### 4.1 配置分离 [P0]

- [ ] **配置外部化**：配置通过 ConfigMap/Secret 管理
- [ ] **环境差异配置**：不同环境配置分离
- [ ] **配置版本化**：配置文件纳入版本控制
- [ ] **配置热更新**：支持配置热更新（如需要）

```yaml
# 检查示例
envFrom:
- configMapRef:
    name: app-config
- secretRef:
    name: app-secrets
```

### 4.2 环境变量 [P0]

- [ ] **敏感信息保护**：敏感信息使用 Secret
- [ ] **变量命名规范**：环境变量命名一致
- [ ] **默认值合理**：配置合理的默认值
- [ ] **必需变量验证**：应用启动时验证必需变量

---

## 5. 可观测性

### 5.1 日志配置 [P0]

- [ ] **日志输出标准**：日志输出到 stdout/stderr
- [ ] **日志格式统一**：使用结构化日志（JSON）
- [ ] **日志级别合理**：生产环境使用 INFO 或以上级别
- [ ] **敏感信息过滤**：日志中无敏感信息
- [ ] **日志采集配置**：日志采集器已配置

### 5.2 指标配置 [P0]

- [ ] **Prometheus 指标**：应用暴露 Prometheus 指标
- [ ] **指标路径配置**：`/metrics` 端点可访问
- [ ] **核心指标暴露**：CPU、内存、请求数、错误数等
- [ ] **ServiceMonitor 配置**：Prometheus 采集配置完成
- [ ] **自定义指标**：业务关键指标已暴露

```yaml
# 检查示例
annotations:
  prometheus.io/scrape: "true"
  prometheus.io/port: "8080"
  prometheus.io/path: "/metrics"
```

### 5.3 追踪配置 [P1]

- [ ] **分布式追踪集成**：集成 Jaeger/Zipkin
- [ ] **Trace ID 传递**：Trace Context 正确传递
- [ ] **关键路径追踪**：关键业务路径有追踪埋点

### 5.4 告警配置 [P0]

- [ ] **告警规则定义**：关键告警规则已配置
- [ ] **告警渠道配置**：告警通知渠道已配置
- [ ] **告警分级**：告警按严重程度分级
- [ ] **值班轮换**：告警响应责任明确

```yaml
# 检查示例
- alert: HighErrorRate
  expr: |
    sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m])) > 0.05
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "High error rate detected"
```

---

## 6. 存储配置

### 6.1 持久化存储 [P0]

- [ ] **PVC 配置**：持久化存储使用 PVC
- [ ] **StorageClass 指定**：明确指定 StorageClass
- [ ] **存储大小合理**：存储容量经过评估
- [ ] **备份策略**：数据备份策略已定义
- [ ] **访问模式正确**：`accessModes` 配置正确

```yaml
# 检查示例
apiVersion: v1
kind: PersistentVolumeClaim
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: ssd-storage
  resources:
    requests:
      storage: 100Gi
```

### 6.2 临时存储 [P1]

- [ ] **emptyDir 使用**：临时数据使用 emptyDir
- [ ] **大小限制**：emptyDir 大小限制已配置
- [ ] **数据不丢失**：关键数据不存储在 emptyDir

---

## 7. 网络配置

### 7.1 服务暴露 [P0]

- [ ] **Service 配置**：Service 已正确配置
- [ ] **端口定义**：端口名称和编号明确
- [ ] **类型选择**：Service 类型选择合理（ClusterIP/NodePort/LoadBalancer）
- [ ] **Ingress 配置**：外部访问通过 Ingress

### 7.2 DNS 配置 [P0]

- [ ] **服务发现**：使用 Service 名称进行服务发现
- [ ] **DNS 策略**：`dnsPolicy` 配置正确
- [ ] **自定义 DNS**：如需要，`dnsConfig` 已配置

---

## 8. 部署流程

### 8.1 GitOps 配置 [P0]

- [ ] **清单版本化**：Kubernetes 清单纳入 Git 管理
- [ ] **环境分支策略**：Git 分支与环境对应
- [ ] **PR 审核**：变更需 PR 审核
- [ ] **自动化部署**：部署流程自动化

### 8.2 回滚准备 [P0]

- [ ] **回滚文档**：回滚操作文档已编写
- [ ] **回滚测试**：回滚流程已测试
- [ ] **版本历史**：保留历史版本镜像
- [ ] **数据回滚**：数据回滚方案已定义

---

## 9. 文档与知识

### 9.1 运维文档 [P0]

- [ ] **架构文档**：系统架构文档已编写
- [ ] **部署文档**：部署步骤文档已编写
- [ ] **配置说明**：配置项说明文档已编写
- [ ] **故障手册**：常见故障处理手册已编写

### 9.2 值班与支持 [P0]

- [ ] **值班名单**：值班人员名单已确定
- [ ] **联系方式**：紧急联系方式已更新
- [ ] **升级流程**：问题升级流程已定义

---

## 检查评分

### 评分标准

- **通过**：所有 [P0] 检查项完成
- **有条件通过**：所有 [P0] 完成，[P1] 完成率 >= 80%
- **不通过**：存在未完成的 [P0] 检查项

### 检查结果

| 类别 | P0 完成 | P1 完成 | P2 完成 | 状态 |
|------|---------|---------|---------|------|
| 工作负载配置 | /5 | /0 | /0 | [ ] |
| 高可用配置 | /5 | /4 | /0 | [ ] |
| 安全配置 | /8 | /0 | /0 | [ ] |
| 配置管理 | /4 | /0 | /0 | [ ] |
| 可观测性 | /5 | /2 | /0 | [ ] |
| 存储配置 | /3 | /3 | /0 | [ ] |
| 网络配置 | /4 | /0 | /0 | [ ] |
| 部署流程 | /3 | /0 | /0 | [ ] |
| 文档与知识 | /4 | /0 | /0 | [ ] |
| **总计** | **/41** | **/9** | **/0** | [ ] |

---

## 参考资料

- [Kubernetes 生产就绪检查清单](https://github.com/elseu/kubernetes-production-checklist)
- [CIS Kubernetes Benchmark](https://www.cisecurity.org/benchmark/kubernetes)
- [Kubernetes 最佳实践](https://kubernetes.io/docs/concepts/configuration/overview/)