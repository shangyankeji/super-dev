---
title: 案例：Kubernetes 安全事件
version: 1.0.0
last_updated: 2025-03-20
owner: security-team
tags: [kubernetes, security, incident, case-study]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：Kubernetes 安全事件响应实战案例
# 作用：分享 K8s 安全事件的处理经验
# 创建时间：2025-03-20
# 最后修改：2025-03-20

## 背景

某金融科技公司的 Kubernetes 生产集群遭遇安全事件：
- 异常流量从集群内发起对外扫描
- 多个 Pod 被植入挖矿程序
- 敏感数据疑似泄露

## 事件时间线

```
09:15 - 监控告警：异常出站流量
09:20 - 安全团队介入，开始调查
09:30 - 确认安全事件，启动应急响应
10:00 - 隔离受影响 Pod，阻断攻击路径
11:00 - 清除恶意程序，恢复服务
14:00 - 完成事件分析，制定加固措施
16:00 - 实施安全加固
次日 - 发布安全事件报告
```

## 攻击路径分析

### 1. 初始入侵

**漏洞**：未认证的 Kubernetes Dashboard

```yaml
# [WARN] 暴露的 Dashboard（错误配置示例）
apiVersion: v1
kind: Service
metadata:
  name: kubernetes-dashboard
spec:
  type: NodePort  # 对外暴露
  ports:
  - port: 443
    targetPort: 8443
    nodePort: 30000  # 直接暴露
```

**攻击步骤**：
1. 扫描发现 NodePort 30000 开放
2. 访问 Dashboard 无需认证
3. 通过 Dashboard 创建恶意 Pod
4. 获取集群管理员权限

### 2. 横向移动

**利用**：过度授权的 ServiceAccount

```yaml
# [WARN] 过度授权（错误配置示例）
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: dashboard-admin
subjects:
- kind: ServiceAccount
  name: dashboard
  namespace: kubernetes-dashboard
roleRef:
  kind: ClusterRole
  name: cluster-admin  # 完全控制
```

### 3. 权限维持

攻击者创建了伪装的系统更新任务。

## 应急响应

### 第一阶段：遏制和隔离

#### 1.1 网络隔离

```yaml
# 立即实施网络隔离
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: emergency-isolation
  namespace: affected-namespace
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
  # 阻止所有流量
```

```bash
# 阻断可疑出站流量
kubectl apply -f emergency-network-policy.yaml

# 切断外部访问
kubectl patch svc kubernetes-dashboard -p '{"spec":{"type":"ClusterIP"}}'
```

#### 1.2 隔离受影响 Pod

```bash
# 标记受影响 Pod
kubectl label pods -n affected-namespace --all security.incident=true

# 隔离 Pod（添加 NetworkPolicy）
kubectl apply -f isolation-policy.yaml

# 保留证据（导出 Pod 信息）
kubectl get pods -n affected-namespace -o yaml > /forensics/pods-backup.yaml
kubectl logs -n affected-namespace <pod-name> > /forensics/pod-logs.txt
```

#### 1.3 撤销凭证

```bash
# 删除可疑 ServiceAccount token
kubectl delete secret -n kubernetes-dashboard dashboard-token

# 轮换关键 Secret
# 根据应用需求重新创建凭证
```

### 第二阶段：清除和恢复

#### 2.1 删除恶意资源

```bash
# 识别异常资源（检查可疑镜像）
kubectl get pods -A -o json | jq '.items[] | select(.spec.containers[].image | contains("suspicious")) | .metadata.name + " " + .metadata.namespace'

# 删除受影响的 Pod
kubectl delete pod <affected-pod> -n <namespace>

# 删除可疑的 CronJob
kubectl delete cronjob <suspicious-job> -n kube-system

# 清理过度授权的 RBAC
kubectl delete clusterrolebinding dashboard-admin
```

#### 2.2 修复漏洞

```yaml
# [DONE] 修复 Dashboard 配置
apiVersion: v1
kind: Service
metadata:
  name: kubernetes-dashboard
spec:
  type: ClusterIP  # 仅集群内部访问
  ports:
  - port: 443
    targetPort: 8443

---
# 最小权限 RBAC
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: dashboard-view
  namespace: kubernetes-dashboard
rules:
- apiGroups: [""]
  resources: ["pods", "services"]
  verbs: ["get", "list"]
- apiGroups: ["apps"]
  resources: ["deployments"]
  verbs: ["get", "list"]
```

#### 2.3 恢复服务

```bash
# 验证清理完成
kubectl get pods -A | grep -v Running
kubectl get cronjobs -A

# 逐步恢复网络
kubectl apply -f production-network-policy.yaml

# 验证服务健康
kubectl rollout status deployment/api-service -n production
```

### 第三阶段：事后分析

#### 3.1 日志分析

```bash
# 审计日志分析
kubectl logs -n kube-system kube-apiserver-master1 --since=24h | grep -E "(create|delete|update)" > audit-events.log

# 查找可疑 API 调用
grep "system:anonymous" audit-events.log

# Pod 创建记录
kubectl get events --all-namespaces --sort-by='.lastTimestamp' | grep Created
```

#### 3.2 影响评估

```yaml
# 影响范围报告
impact_assessment:
  affected_namespaces:
    - production-api
    - production-worker
  affected_pods: 15
  data_exposure:
    - database_credentials
    - api_keys
  malicious_activity:
    - cryptomining
    - network_scanning
  duration: 4 hours
```

## 根因分析

### 1. 安全配置缺陷

| 问题 | 风险等级 | 影响 |
|------|---------|------|
| Dashboard 暴露 | 高 | 初始入侵点 |
| 无认证访问 | 高 | 未经授权访问 |
| 过度授权 | 高 | 权限提升 |
| 无网络策略 | 中 | 横向移动 |
| 无运行时监控 | 中 | 延迟发现 |

### 2. 监控盲区

- 无异常行为检测
- 无出站流量监控
- 无镜像扫描
- 无审计日志分析

## 安全加固措施

### 1. 访问控制加固

```yaml
# 启用 RBAC
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: dashboard-restricted
rules:
- apiGroups: [""]
  resources: ["pods", "services", "configmaps"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["apps"]
  resources: ["deployments", "replicasets"]
  verbs: ["get", "list", "watch"]
# 不包含 create/delete/update
```

### 2. 网络隔离

```yaml
# 默认拒绝策略
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: kubernetes-dashboard
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
```

### 3. 运行时安全

使用 Falco 进行运行时安全监控，配置规则检测异常行为。

### 4. 审计增强

```yaml
# 审计策略
apiVersion: audit.k8s.io/v1
kind: Policy
rules:
# 记录所有 Secret 操作
- level: RequestResponse
  resources:
  - group: ""
    resources: ["secrets"]

# 记录所有 Pod 创建
- level: RequestResponse
  resources:
  - group: ""
    resources: ["pods"]
  verbs: ["create", "delete"]

# 记录所有 RBAC 变更
- level: RequestResponse
  resources:
  - group: "rbac.authorization.k8s.io"
    resources: ["roles", "rolebindings", "clusterroles", "clusterrolebindings"]

# 记录匿名访问
- level: Metadata
  users: ["system:anonymous"]
```

### 5. 镜像安全

```yaml
# Kyverno 镜像策略
apiVersion: kyverno.io/v1
kind: ClusterPolicy
metadata:
  name: restrict-image-registries
spec:
  validationFailureAction: enforce
  rules:
  - name: validate-registry
    match:
      resources:
        kinds:
        - Pod
    validate:
      message: "Images must be from approved registries"
      pattern:
        spec:
          containers:
          - image: "registry.company.com/* | gcr.io/*"
```

## 经验教训

### 关键发现

1. **入口暴露**：Dashboard 不应暴露到公网
2. **权限过大**：最小权限原则未遵循
3. **监控缺失**：无运行时安全监控
4. **响应延迟**：从告警到响应用时过长

### 改进措施

1. **立即**：关闭外部访问入口
2. **短期**：部署网络策略和运行时安全
3. **中期**：建立安全监控体系
4. **长期**：培养安全意识

## 检查清单

### 事件响应检查清单

- [ ] 确认事件范围
- [ ] 隔离受影响资源
- [ ] 保留证据
- [ ] 阻断攻击路径
- [ ] 清除恶意资源
- [ ] 修复漏洞
- [ ] 恢复服务
- [ ] 完成分析报告
- [ ] 实施加固措施
- [ ] 更新应急预案

### 安全加固检查清单

- [ ] RBAC 最小权限
- [ ] NetworkPolicy 配置
- [ ] Pod Security Standards
- [ ] 镜像签名验证
- [ ] 运行时安全监控
- [ ] 审计日志启用
- [ ] Secret 加密
- [ ] 定期安全审计

## 参考资料

- [Kubernetes 安全最佳实践](https://kubernetes.io/docs/concepts/security/)
- [CIS Kubernetes Benchmark](https://www.cisecurity.org/benchmark/kubernetes)
- [NSA Kubernetes 加固指南](https://media.defense.gov/2022/Aug/29/2003055140/1-1021055140/CTR-KUBERNETES-HARDENING-GUIDANCE.PDF)
- [Falco 文档](https://falco.org/docs/)