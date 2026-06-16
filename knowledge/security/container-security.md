---
id: container-security
title: 容器安全完整指南
domain: security
category: container-security.md
difficulty: intermediate
tags: [container, layer, security, 主机安全, 容器安全分层, 概述, 编排安全, 网络安全]
quality_score: 70
last_updated: 2026-06-15
---
# 容器安全完整指南

## 概述
容器安全涵盖镜像构建、运行时保护、网络安全和合规审计,确保容器化应用在整个生命周期内的安全性。

## 容器安全分层

```
Layer 1: 镜像安全(Image Security)
Layer 2: 运行时安全(Runtime Security)
Layer 3: 网络安全(Network Security)
Layer 4: 主机安全(Host Security)
Layer 5: 编排安全(Orchestration Security)
```

## Layer 1: 镜像安全

### 1.1 基础镜像选择
```dockerfile
# 优先选择最小化镜像
# Alpine Linux (5MB)
FROM alpine:3.19

# Distroless (无 Shell)
FROM gcr.io/distroless/static-debian12

# 官方镜像 + 特定版本
FROM node:20.11-alpine3.19

# 禁止使用 latest 标签
# FROM node:latest  # 不安全
```

### 1.2 最小化镜像
```dockerfile
# 多阶段构建
# Build stage
FROM golang:1.22-alpine AS builder
WORKDIR /app
COPY . .
RUN go build -o myapp

# Runtime stage
FROM alpine:3.19
RUN apk --no-cache add ca-certificates
COPY --from=builder /app/myapp /usr/local/bin/
USER nobody
ENTRYPOINT ["myapp"]
```

### 1.3 不以 Root 运行
```dockerfile
# 创建非 Root 用户
RUN addgroup -g 1000 -S appgroup && \
    adduser -u 1000 -S appuser -G appgroup

USER appuser

# 或使用 Dockerfile 语法
USER 1000:1000
```

### 1.4 镜像扫描
```bash
# Trivy 扫描
trivy image --severity HIGH,CRITICAL myapp:latest

# Grype 扫描
grype myapp:latest --fail-on high

# Snyk 扫描
snyk container test myapp:latest --severity-threshold=high
```

### 1.5 镜像签名
```bash
# Cosign 签名
cosign sign --key cosign.key myregistry/myapp:latest

# 验证签名
cosign verify --key cosign.pub myregistry/myapp:latest

# Docker Content Trust
export DOCKER_CONTENT_TRUST=1
docker push myregistry/myapp:latest
```

## Layer 2: 运行时安全

### 2.1 容器隔离
```yaml
# Kubernetes Security Context
apiVersion: v1
kind: Pod
metadata:
  name: secure-pod
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    runAsGroup: 1000
    fsGroup: 1000
    seccompProfile:
      type: RuntimeDefault
  containers:
  - name: app
    image: myapp:latest
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop:
        - ALL
        add:
        - NET_BIND_SERVICE
```

### 2.2 资源限制
```yaml
resources:
  limits:
    cpu: "500m"
    memory: "512Mi"
  requests:
    cpu: "250m"
    memory: "256Mi"
```

### 2.3 只读文件系统
```yaml
securityContext:
  readOnlyRootFilesystem: true
volumeMounts:
- name: tmp
  mountPath: /tmp
- name: cache
  mountPath: /var/cache
volumes:
- name: tmp
  emptyDir: {}
- name: cache
  emptyDir: {}
```

### 2.4 能力剪裁
```yaml
# 删除所有能力
securityContext:
  capabilities:
    drop:
    - ALL

# 仅添加必要能力
securityContext:
  capabilities:
    drop:
    - ALL
    add:
    - NET_BIND_SERVICE  # 绑定端口 < 1024
    - CHOWN             # 修改文件所有者
```

### 2.5 Seccomp 配置
```json
{
  "defaultAction": "SCMP_ACT_ERRNO",
  "architectures": ["SCMP_ARCH_X86_64"],
  "syscalls": [
    {
      "names": ["read", "write", "exit", "sigreturn"],
      "action": "SCMP_ACT_ALLOW"
    },
    {
      "names": ["execve", "fork", "clone"],
      "action": "SCMP_ACT_ALLOW"
    }
  ]
}
```

## Layer 3: 网络安全

### 3.1 网络策略
```yaml
# 限制入站流量
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: api-network-policy
spec:
  podSelector:
    matchLabels:
      app: api
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: frontend
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: database
    ports:
    - protocol: TCP
      port: 5432
```

### 3.2 Service Mesh 安全
```yaml
# Istio mTLS
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
spec:
  mtls:
    mode: STRICT  # 强制 mTLS
```

### 3.3 Network Namespace
```yaml
# 使用 CNI 插件隔离
apiVersion: k8s.cni.cncf.io/v1
kind: NetworkAttachmentDefinition
metadata:
  name: isolated-network
spec:
  config: '{
    "type": "bridge",
    "bridge": "isolated0",
    "ipam": {
      "type": "host-local",
      "subnet": "10.244.1.0/24"
    }
  }'
```

## Layer 4: 主机安全

### 4.1 操作系统加固
```bash
# CIS Benchmark 合规
# 1. 禁用不必要服务
systemctl disable bluetooth
systemctl disable cups

# 2. 配置防火墙
ufw default deny incoming
ufw allow from 10.0.0.0/8 to any port 22
ufw enable

# 3. 内核参数加固
sysctl -w net.ipv4.ip_forward=0
sysctl -w net.ipv4.conf.all.send_redirects=0
```

### 4.2 容器运行时配置
```toml
# /etc/containerd/config.toml
[plugins."io.containerd.grpc.v1.cri"]
  disable_cgroup = false
  [plugins."io.containerd.grpc.v1.cri".containerd]
    snapshotter = "overlayfs"
    [plugins."io.containerd.grpc.v1.cri".containerd.runtimes]
      [plugins."io.containerd.grpc.v1.cri".containerd.runtimes.runc]
        runtime_type = "io.containerd.runc.v2"
        [plugins."io.containerd.grpc.v1.cri".containerd.runtimes.runc.options]
          SystemdCgroup = true
```

### 4.3 Audit 日志
```yaml
# Kubernetes Audit Policy
apiVersion: audit.k8s.io/v1
kind: Policy
rules:
- level: Metadata
  resources:
  - group: ""
    resources: ["secrets"]
  verbs: ["get", "list", "watch"]

- level: RequestResponse
  resources:
  - group: ""
    resources: ["pods"]
  verbs: ["create", "update", "delete"]
```

## Layer 5: 编排安全

### 5.1 RBAC 配置
```yaml
# 最小权限原则
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: pod-reader
  namespace: default
rules:
- apiGroups: [""]
  resources: ["pods"]
  verbs: ["get", "list", "watch"]

---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: read-pods
  namespace: default
subjects:
- kind: ServiceAccount
  name: myapp
  namespace: default
roleRef:
  kind: Role
  name: pod-reader
  apiGroup: rbac.authorization.k8s.io
```

### 5.2 Pod Security Standards
```yaml
# Restricted 策略
apiVersion: pod-security.admission.config.k8s.io/v1
kind: PodSecurityConfiguration
defaults:
  enforce: "restricted"
  enforce-version: "latest"
  audit: "restricted"
  audit-version: "latest"
  warn: "restricted"
  warn-version: "latest"
```

### 5.3 Admission Controller
```yaml
# OPA Gatekeeper 策略
apiVersion: templates.gatekeeper.sh/v1
kind: ConstraintTemplate
metadata:
  name: k8srequiredlabels
spec:
  crd:
    spec:
      names:
        kind: K8sRequiredLabels
      validation:
        openAPIV3Schema:
          properties:
            labels:
              type: array
              items:
                type: string
  targets:
    - target: admission.k8s.gatekeeper.sh
      rego: |
        package k8srequiredlabels

        violation[{"msg": msg}] {
          provided := {label | input.review.object.metadata.labels[label]}
          required := {label | label := input.parameters.labels[_]}
          missing := required - provided
          count(missing) > 0
          msg := sprintf("Missing required labels: %v", [missing])
        }
```

## 运行时监控

### 6.1 Falco 规则
```yaml
# 检测异常行为
- rule: Unauthorized Container
  desc: 检测未授权容器启动
  condition: >
    container.id != host and
    not container.image startswith "gcr.io/myorg/"
  output: >
    未授权容器启动 (user=%user.name container=%container.id image=%container.image)
  priority: ERROR
  tags: [container]

- rule: Shell Spawned in Container
  desc: 检测容器内 Shell 启动
  condition: >
    container.id != host and
    proc.name in (bash, sh, zsh)
  output: >
    容器内启动 Shell (user=%user.name container=%container.id shell=%proc.name)
  priority: WARNING
  tags: [container, shell]
```

### 6.2 安全事件响应
```yaml
# 自动响应流程
playbook:
  name: container_security_incident
  steps:
    - action: isolate_pod
      condition: severity == "critical"
      params:
        namespace: "{{ event.namespace }}"
        pod: "{{ event.pod }}"

    - action: capture_forensics
      condition: severity >= "high"
      params:
        container_id: "{{ event.container.id }}"
        output: "/forensics/{{ event.timestamp }}"

    - action: notify_team
      condition: always
      params:
        channel: "#security-alerts"
        message: "{{ event.message }}"
```

## CI/CD 安全集成

### 7.1 镜像构建检查
```yaml
# GitLab CI
stages:
  - security

container_scan:
  stage: security
  image: aquasec/trivy:latest
  script:
    - trivy image --exit-code 1 --severity HIGH,CRITICAL $IMAGE_NAME
  only:
    - main

signature_verify:
  stage: security
  image: gcr.io/projectsigstore/cosign:latest
  script:
    - cosign verify --key cosign.pub $IMAGE_NAME
  only:
    - main
```

### 7.2 部署前验证
```yaml
# Kubernetes ValidatingWebhook
apiVersion: admissionregistration.k8s.io/v1
kind: ValidatingWebhookConfiguration
metadata:
  name: container-security-webhook
webhooks:
- name: security.webhook.cluster.local
  rules:
  - apiGroups: [""]
    apiVersions: ["v1"]
    operations: ["CREATE", "UPDATE"]
    resources: ["pods"]
  failurePolicy: Fail
  sideEffects: None
  admissionReviewVersions: ["v1"]
  clientConfig:
    service:
      name: security-webhook
      namespace: security
      path: "/validate"
```

## 合规检查

### 8.1 CIS Benchmark
```bash
# kube-bench 扫描
kube-bench --config-dir /etc/kube-bench/cfg --benchmark cis-1.8

# 输出示例
[INFO] 1 Control Plane Security Configuration
[INFO] 1.1 Control Plane Node Configuration Files
[PASS] 1.1.1 Ensure that the API server pod specification file permissions are set to 644 or more restrictive
[FAIL] 1.1.2 Ensure that the API server pod specification file ownership is set to root:root
```

### 8.2 策略合规
```yaml
# Checkov IaC 扫描
- name: Run Checkov
  uses: bridgecrewio/checkov-action@master
  with:
    directory: ./kubernetes/
    framework: kubernetes
    check: CKV_K8S_*
    soft_fail: false
```

## 容器逃逸防护

### 9.1 常见逃逸向量
```yaml
# 1. 特权容器
# 危险: 特权容器可访问主机设备
securityContext:
  privileged: false  # 禁止

# 2. 挂载主机路径
# 危险: 可修改主机文件
volumes:
- name: host-root
  hostPath:
    path: /  # 禁止挂载根目录

# 3. 挂载 Docker Socket
# 危险: 可控制 Docker 守护进程
volumes:
- name: docker-sock
  hostPath:
    path: /var/run/docker.sock  # 禁止

# 4. HostPID/HostIPC
# 危险: 可看到主机进程
spec:
  hostPID: false  # 禁止
  hostIPC: false  # 禁止
```

### 9.2 安全加固配置
```yaml
apiVersion: v1
kind: Pod
metadata:
  name: hardened-pod
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    seccompProfile:
      type: RuntimeDefault
  hostPID: false
  hostIPC: false
  hostNetwork: false
  containers:
  - name: app
    image: myapp:latest
    securityContext:
      privileged: false
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop:
        - ALL
```

## 镜像仓库安全

### 10.1 私有仓库
```yaml
# Harbor 配置
apiVersion: v1
kind: Secret
metadata:
  name: harbor-registry
type: kubernetes.io/dockerconfigjson
data:
  .dockerconfigjson: <base64-encoded-config>
```

### 10.2 镜像策略
```yaml
# 仅允许来自可信仓库的镜像
apiVersion: constraints.gatekeeper.sh/v1beta1
kind: K8sAllowedRepos
metadata:
  name: repo-isolation
spec:
  match:
    kinds:
      - apiGroups: [""]
        kinds: ["Pod"]
  parameters:
    repos:
      - "gcr.io/myorg/"
      - "harbor.company.com/"
```

### 10.3 镜像扫描策略
```yaml
# Harbor 扫描策略
scan:
  enabled: true
  schedule: "0 0 * * *"  # 每天扫描

policy:
  type: "vulnerability"
  parameters:
    minimum_severity: "high"
    whitelist:
      - CVE-2021-44228
```

## 安全基线检查

### 11.1 自动化检查脚本
```bash
#!/bin/bash
# 容器安全检查脚本

echo "检查容器安全配置..."

# 1. 检查特权容器
kubectl get pods --all-namespaces -o json | jq '.items[] | select(.spec.containers[].securityContext.privileged==true) | .metadata.name' | while read pod; do
  echo "[FAIL] 发现特权容器: $pod"
done

# 2. 检查 Root 用户
kubectl get pods --all-namespaces -o json | jq '.items[] | select(.spec.securityContext.runAsNonRoot!=true) | .metadata.name' | while read pod; do
  echo "[WARN] 未限制 Root 运行: $pod"
done

# 3. 检查资源限制
kubectl get pods --all-namespaces -o json | jq '.items[] | select(.spec.containers[].resources.limits==null) | .metadata.name' | while read pod; do
  echo "[WARN] 未设置资源限制: $pod"
done

# 4. 检查镜像标签
kubectl get pods --all-namespaces -o json | jq '.items[] | select(.spec.containers[].image | endswith(":latest")) | .metadata.name' | while read pod; do
  echo "[WARN] 使用 latest 标签: $pod"
done
```

## 容器安全工具链

| 类别 | 工具 | 用途 |
|------|------|------|
| 镜像扫描 | Trivy, Grype, Clair | 漏洞扫描 |
| 运行时监控 | Falco, Sysdig Inspect | 行为监控 |
| 策略引擎 | OPA Gatekeeper, Kyverno | 准入控制 |
| 合规检查 | kube-bench, Checkov | 基线扫描 |
| 密钥管理 | Vault, External Secrets | 密钥注入 |
| 网络安全 | Calico, Cilium, Istio | 网络隔离 |
| 审计日志 | Audit2RBAC, kube-audit | 审计追踪 |

## 实施检查清单

- [ ] 使用最小化基础镜像
- [ ] 不以 Root 用户运行容器
- [ ] 只读文件系统
- [ ] 删除不必要的 Linux 能力
- [ ] 配置资源限制
- [ ] 镜像签名验证
- [ ] 定期镜像扫描
- [ ] 网络策略隔离
- [ ] RBAC 最小权限
- [ ] Pod Security Standards
- [ ] 运行时监控(Falco)
- [ ] Audit 日志启用
- [ ] 定期合规检查
- [ ] 灾难恢复计划
- [ ] 安全培训和演练

## 参考资料
- [CIS Docker Benchmark](https://www.cisecurity.org/benchmark/docker)
- [CIS Kubernetes Benchmark](https://www.cisecurity.org/benchmark/kubernetes)
- [OWASP Docker Security](https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html)
- [Kubernetes Security Best Practices](https://kubernetes.io/docs/concepts/security/)
- [Falco Documentation](https://falco.org/docs/)
