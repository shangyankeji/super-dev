---
title: 容器安全标准
version: 1.0.0
last_updated: 2025-03-20
owner: security-team
tags: [container, security, docker, hardening]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：容器安全完整标准
# 作用：为容器镜像、运行时、编排提供安全规范
# 创建时间：2025-03-20
# 最后修改：2025-03-20

## 目标

建立容器全生命周期安全标准，确保：
- 镜像构建安全
- 运行时安全
- 网络安全隔离
- 安全审计和合规

## 适用场景

- 容器镜像构建和分发
- 容器运行时安全配置
- Kubernetes 安全加固
- DevSecOps 流程集成

## 核心标准

### 1. 镜像安全标准

#### 安全 Dockerfile 模板

```dockerfile
# 使用官方基础镜像，指定版本
FROM python:3.11-slim-bookworm@sha256:abc123... AS builder

# 设置构建参数
ARG VERSION=1.0.0
ARG BUILD_DATE

# 创建非 root 用户
RUN groupadd -r appgroup && \
    useradd -r -g appgroup -d /app -s /sbin/nologin appuser

# 设置工作目录
WORKDIR /app

# 安装依赖（使用虚拟环境）
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# 复制依赖文件
COPY requirements.txt .

# 安装依赖，清理缓存
RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir -r requirements.txt && \
    rm -rf /root/.cache

# 生产阶段
FROM python:3.11-slim-bookworm@sha256:abc123...

# 安全标签
LABEL org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.created="${BUILD_DATE}" \
      org.opencontainers.image.vendor="Company Name" \
      org.opencontainers.image.title="Application Name"

# 创建非 root 用户
RUN groupadd -r appgroup && \
    useradd -r -g appgroup -d /app -s /sbin/nologin appuser && \
    mkdir -p /app /tmp/app && \
    chown -R appuser:appgroup /app /tmp/app

# 复制虚拟环境
COPY --from=builder /opt/venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# 复制应用代码
COPY --chown=appuser:appgroup . /app/

# 设置安全环境变量
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    HOME=/app

# 切换到非 root 用户
USER appuser

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD python -c "import urllib.request; urllib.request.urlopen('http://localhost:8080/health')" || exit 1

# 暴露端口
EXPOSE 8080

# 启动命令
CMD ["python", "-m", "app.main"]
```

#### 镜像签名和验证

```bash
# 使用 cosign 签名镜像
cosign sign --key cosign.key registry.example.com/app:v1.0.0

# 验证镜像签名
cosign verify --key cosign.pub registry.example.com/app:v1.0.0

# 使用 Kyverno 策略强制验证
apiVersion: kyverno.io/v1
kind: ClusterPolicy
metadata:
  name: verify-image-signatures
spec:
  validationFailureAction: enforce
  background: false
  rules:
  - name: verify-signature
    match:
      resources:
        kinds:
        - Pod
    verify-images:
    - imageReferences:
      - "registry.example.com/*"
      attestors:
      - entries:
        - keys:
            publicKeys: |-
              -----BEGIN PUBLIC KEY-----
              MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...
              -----END PUBLIC KEY-----
```

### 2. 运行时安全标准

#### Pod 安全配置

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: secure-pod
  namespace: production
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
    image: registry.example.com/app:v1.0.0
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop:
        - ALL
        add: []
    volumeMounts:
    - name: tmp
      mountPath: /tmp
    - name: cache
      mountPath: /var/cache
  volumes:
  - name: tmp
    emptyDir: {}
  - name: cache
    emptyDir:
      sizeLimit: "100Mi"
```

#### Pod Security Standards 配置

```yaml
# 命名空间级别强制执行
apiVersion: v1
kind: Namespace
metadata:
  name: production
  labels:
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/enforce-version: latest
    pod-security.kubernetes.io/audit: restricted
    pod-security.kubernetes.io/warn: restricted
```

#### AppArmor/SELinux 配置

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: apparmor-pod
  annotations:
    container.apparmor.security.beta.kubernetes.io/app: localhost/apparmor-profile
spec:
  containers:
  - name: app
    image: registry.example.com/app:v1.0.0
```

```bash
# AppArmor 配置文件示例
#include <tunables/global>
profile apparmor-profile flags=(attach_disconnected,mediate_deleted) {
  #include <abstractions/base>

  # 允许网络
  network inet tcp,
  network inet udp,

  # 允许文件访问
  /app/** r,
  /tmp/** rw,
  /var/cache/** rw,

  # 禁止的访问
  deny /etc/shadow r,
  deny /etc/passwd w,
  deny /proc/** w,
  deny /sys/** w,
}
```

### 3. 网络安全标准

#### 网络策略（默认拒绝）

```yaml
# 默认拒绝所有入站流量
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-ingress
  namespace: production
spec:
  podSelector: {}
  policyTypes:
  - Ingress

---
# 默认拒绝所有出站流量
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-egress
  namespace: production
spec:
  podSelector: {}
  policyTypes:
  - Egress

---
# 允许 DNS 解析
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-dns
  namespace: production
spec:
  podSelector: {}
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: kube-system
      podSelector:
        matchLabels:
          k8s-app: kube-dns
    ports:
    - protocol: UDP
      port: 53
    - protocol: TCP
      port: 53
```

#### 应用网络策略

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: app-network-policy
  namespace: production
spec:
  podSelector:
    matchLabels:
      app: sample-app
  policyTypes:
  - Ingress
  - Egress
  ingress:
  # 仅允许来自 Ingress Controller
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
  egress:
  # 允许访问数据库
  - to:
    - podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
  # 允许访问外部 API
  - to:
    - ipBlock:
        cidr: 0.0.0.0/0
        except:
        - 10.0.0.0/8
        - 172.16.0.0/12
        - 192.168.0.0/16
    ports:
    - protocol: TCP
      port: 443
```

### 4. 密钥管理标准

```yaml
# 外部密钥管理（使用 External Secrets Operator）
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: db-credentials
  namespace: production
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: vault-backend
    kind: ClusterSecretStore
  target:
    name: db-credentials
    creationPolicy: Owner
    template:
      type: Opaque
      data:
        url: "{{ .url }}"
        username: "{{ .username }}"
        password: "{{ .password }}"
  data:
  - secretKey: url
    remoteRef:
      key: secret/data/production/database
      property: url
  - secretKey: username
    remoteRef:
      key: secret/data/production/database
      property: username
  - secretKey: password
    remoteRef:
      key: secret/data/production/database
      property: password
```

#### Secret 加密配置

```yaml
# KMS 加密配置
apiVersion: apiserver.config.k8s.io/v1
kind: EncryptionConfiguration
resources:
  - resources:
    - secrets
    providers:
    - kms:
        name: myKMS
        endpoint: unix:///path/to/kms/socket
        cachesize: 1000
        timeout: 3s
    - identity: {}
```

### 5. 漏洞扫描标准

```yaml
# Trivy 扫描配置
apiVersion: aquasecurity.github.io/v1alpha1
kind: ClusterScan
metadata:
  name: cluster-vulnerability-scan
spec:
  schedule: "0 2 * * *"
  scanConfig:
    scanType: "vulnerability"
    scanAllNamespaces: true
    resources:
    - type: Pod
    - type: Deployment
    - type: StatefulSet
    - type: DaemonSet
  reportConfig:
    format: "json"
    store:
      type: "s3"
      bucket: "security-reports"
      prefix: "trivy/"
```

#### CI/CD 漏洞扫描

```yaml
# GitLab CI 配置
container_scanning:
  stage: security
  image: aquasec/trivy:latest
  script:
    - trivy image --exit-code 1 --severity HIGH,CRITICAL $IMAGE_NAME:$IMAGE_TAG
  allow_failure: false

# GitHub Actions 配置
name: Container Security Scan
on: [push, pull_request]
jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Build image
      run: docker build -t app:${{ github.sha }} .
    - name: Run Trivy vulnerability scanner
      uses: aquasecurity/trivy-action@master
      with:
        image-ref: app:${{ github.sha }}
        format: 'sarif'
        output: 'trivy-results.sarif'
        severity: 'CRITICAL,HIGH'
        exit-code: '1'
```

### 6. 审计日志标准

```yaml
# Kubernetes 审计策略
apiVersion: audit.k8s.io/v1
kind: Policy
rules:
# 记录所有 Secret 访问
- level: RequestResponse
  resources:
  - group: ""
    resources: ["secrets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]

# 记录所有认证失败
- level: Metadata
  omitStages:
  - RequestReceived
  resources:
  - group: ""
    resources: ["*"]
  verbs: ["*"]
  users: ["system:anonymous"]

# 记录所有 Pod 操作
- level: RequestResponse
  resources:
  - group: ""
    resources: ["pods"]
  - group: "apps"
    resources: ["deployments", "statefulsets", "daemonsets"]

# 记录 RBAC 变更
- level: RequestResponse
  resources:
  - group: "rbac.authorization.k8s.io"
    resources: ["roles", "rolebindings", "clusterroles", "clusterrolebindings"]
```

## 执行清单

### 镜像构建

- [ ] 使用官方基础镜像
- [ ] 固定镜像版本和 digest
- [ ] 创建非 root 用户
- [ ] 设置只读根文件系统
- [ ] 删除不必要的包和缓存
- [ ] 进行漏洞扫描
- [ ] 签名镜像

### 运行时配置

- [ ] 禁止特权容器
- [ ] 禁止特权提升
- [ ] 限制 capabilities
- [ ] 配置 seccomp/AppArmor
- [ ] 设置资源限制
- [ ] 配置网络策略
- [ ] 启用审计日志

### 密钥管理

- [ ] 使用外部密钥管理系统
- [ ] 启用 Secret 加密
- [ ] 定期轮换密钥
- [ ] 审计密钥访问
- [ ] 最小权限 RBAC

### 监控告警

- [ ] 配置安全事件告警
- [ ] 监控异常访问
- [ ] 跟踪漏洞修复
- [ ] 审计日志分析

## 最佳实践

### 1. 镜像构建

```dockerfile
# [DONE] 正确：多阶段构建，最小化攻击面
FROM golang:1.21 AS builder
WORKDIR /app
COPY . .
RUN CGO_ENABLED=0 go build -o app .

FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/app /
USER nonroot:nonroot
ENTRYPOINT ["/app"]
```

```dockerfile
# [FAIL] 错误：包含不必要的工具和包
FROM ubuntu:latest
RUN apt-get update && apt-get install -y curl wget vim
COPY . .
CMD ["./app"]
```

### 2. 网络隔离

```yaml
# [DONE] 正确：最小权限网络策略
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: app-policy
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

### 3. RBAC 配置

```yaml
# [DONE] 正确：最小权限
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: app-role
  namespace: production
rules:
- apiGroups: [""]
  resources: ["configmaps", "secrets"]
  resourceNames: ["app-config", "app-secrets"]
  verbs: ["get"]
- apiGroups: [""]
  resources: ["pods"]
  verbs: ["get", "list"]
```

```yaml
# [FAIL] 错误：过度授权
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: app-role
rules:
- apiGroups: ["*"]
  resources: ["*"]
  verbs: ["*"]
```

## 反模式

### 禁止操作

```yaml
# [FAIL] 特权容器
securityContext:
  privileged: true

# [FAIL] hostNetwork
spec:
  hostNetwork: true

# [FAIL] hostPath 挂载
volumes:
- name: host
  hostPath:
    path: /

# [FAIL] 以 root 运行
securityContext:
  runAsUser: 0

# [FAIL] 最新标签
image: myapp:latest

# [FAIL] 无资源限制
# 缺少 resources 配置

# [FAIL] 无网络策略
# 缺少 NetworkPolicy

# [FAIL] 无健康检查
# 缺少 livenessProbe/readinessProbe
```

## 实战案例

### 案例 1：容器逃逸防护

```yaml
# 防护措施组合
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
  containers:
  - name: app
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop:
        - ALL
    volumeMounts:
    - name: tmp
      mountPath: /tmp
  volumes:
  - name: tmp
    emptyDir: {}
```

### 案例 2：镜像漏洞修复流程

```bash
# 1. 扫描镜像
trivy image registry.example.com/app:v1.0.0

# 2. 分析漏洞
trivy image --severity HIGH,CRITICAL registry.example.com/app:v1.0.0

# 3. 更新基础镜像
# Dockerfile 中更新 FROM 指令

# 4. 重建镜像
docker build -t registry.example.com/app:v1.0.1 .

# 5. 验证修复
trivy image registry.example.com/app:v1.0.1

# 6. 推送并签名
docker push registry.example.com/app:v1.0.1
cosign sign registry.example.com/app:v1.0.1
```

## 检查清单

### 镜像安全检查

- [ ] 基础镜像是官方维护版本
- [ ] 镜像版本已固定（含 digest）
- [ ] 无高危或严重漏洞
- [ ] 镜像已签名
- [ ] 无敏感信息硬编码
- [ ] 镜像大小合理（< 500MB）

### 运行时安全检查

- [ ] 非 root 用户运行
- [ ] 只读根文件系统
- [ ] 已限制 capabilities
- [ ] 已配置 seccomp
- [ ] 已配置网络策略
- [ ] 资源限制已设置
- [ ] 健康检查已配置

### 访问控制检查

- [ ] RBAC 遵循最小权限
- [ ] Secret 已加密
- [ ] 网络策略已生效
- [ ] 审计日志已启用
- [ ] 无特权容器
- [ ] 无 hostNetwork/hostPID

### 合规性检查

- [ ] 符合 CIS Benchmark
- [ ] 符合 PCI DSS（如适用）
- [ ] 符合 SOC 2（如适用）
- [ ] 安全事件响应流程已建立

## 参考资料

- [CIS Docker Benchmark](https://www.cisecurity.org/benchmark/docker)
- [CIS Kubernetes Benchmark](https://www.cisecurity.org/benchmark/kubernetes)
- [Pod Security Standards](https://kubernetes.io/docs/concepts/security/pod-security-standards/)
- [Trivy 文档](https://aquasecurity.github.io/trivy/)
- [OWASP Docker Security](https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html)
- [NIST Container Security Guide](https://csrc.nist.gov/publications/detail/sp/800-190/final)