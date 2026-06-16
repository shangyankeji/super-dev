---
id: docker-k8s-production-playbook
title: Docker/Kubernetes 生产安全手册
domain: devops
category: 02-playbooks
difficulty: advanced
tags: [devops, docker, kubernetes, k8s, security, multi-stage, non-root, secrets, scanning, rbac, network-policy, tls, enterprise]
quality_score: 94
maintainer: devops-team@superdev.com
last_updated: 2026-06-14
---

# Docker/Kubernetes 生产安全手册

> 基于 [Tigera Container Security 2025](https://www.tigera.io/learn/guides/container-security-best-practices/) + [Sysdig 17 Best Practices](https://www.sysdig.com/learn-cloud-native/container-security-best-practices) + [ThinkSys Docker Guide](https://thinksys.com/devops/docker-best-practices/)

## Docker 五大生产原则

### 1. 永远不以 root 运行
```dockerfile
# ❌ 默认 root（容器逃逸 = 主机 root）
FROM node:20
CMD ["node", "server.js"]

# ✅ 创建非 root 用户
FROM node:20-alpine
RUN addgroup -S app && adduser -S app -G app
USER app                    # 关键！
CMD ["node", "server.js"]
```

### 2. 多阶段构建（减镜像 + 减攻击面）
```dockerfile
# ✅ 构建工具不进最终镜像
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build           # 产物在 dist/

FROM node:20-alpine AS runner
WORKDIR /app
RUN addgroup -S app && adduser -S app -G app
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./
USER app
CMD ["node", "dist/server.js"]
# 最终镜像不含 npm/COPY/源码 → 更小更安全
```

### 3. 密钥不入镜像
```dockerfile
# ❌ 密钥硬编码在 Dockerfile（任何人 pull 镜像就泄露）
ENV DATABASE_URL=postgres://user:pass@db:5432/app

# ✅ 运行时注入（K8s Secret / 环境变量）
# K8s 部署：
env:
  - name: DATABASE_URL
    valueFrom:
      secretKeyRef:
        name: app-secrets
        key: database-url
```

### 4. 用最小基础镜像
```dockerfile
# ❌ 完整 OS（含 shell/apt/系统工具 → 攻击面大）
FROM ubuntu:22.04

# ✅ Alpine（5MB）或 Distroless（无 shell）
FROM node:20-alpine           # Alpine
# 或
FROM gcr.io/distroless/nodejs20  # Distroless（连 shell 都没有）
```

### 5. CI 中扫描漏洞
```yaml
# .github/workflows/ci.yml
- name: Scan image for vulnerabilities
  uses: aquasecurity/trivy-action@master
  with:
    image-ref: app:latest
    severity: CRITICAL,HIGH
    exit-code: 1             # 有高危漏洞就 CI 失败
```

## Kubernetes 生产配置

### Pod 安全上下文
```yaml
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      securityContext:
        runAsNonRoot: true          # 禁止 root
        runAsUser: 1000
        fsGroup: 2000
        seccompProfile:
          type: RuntimeDefault       # 默认 Seccomp 配置
      containers:
      - name: app
        image: app:latest
        securityContext:
          allowPrivilegeEscalation: false   # 禁止提权
          readOnlyRootFilesystem: true       # 只读文件系统
          capabilities:
            drop: [ALL]                     # 丢掉所有 Linux capabilities
            # 只 add 必需的，如：add: [NET_BIND_SERVICE]
        resources:
          limits:
            cpu: 500m
            memory: 512Mi
          requests:
            cpu: 100m
            memory: 128Mi
```

### Network Policy（零信任网络）
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: app-network-policy
spec:
  podSelector:
    matchLabels:
      app: api-server
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: frontend      # 只允许前端 Pod 访问
    ports:
    - protocol: TCP
      port: 3000
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: postgres      # 只允许访问数据库
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - namespaceSelector: {}  # 允许 DNS
    ports:
    - protocol: UDP
      port: 53
```

### RBAC（最小权限）
```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: app-role
rules:
- apiGroups: [""]
  resources: ["configmaps"]
  verbs: ["get", "list"]     # 只读，不写
# 不用 verbs: ["*"]
```

## 生产检查清单

- [ ] 容器以非 root 运行（USER 指令）
- [ ] 多阶段构建（最终镜像无构建工具）
- [ ] 基础镜像是最小化（Alpine/Distroless）
- [ ] CI 有漏洞扫描（Trivy/Grype）
- [ ] 密钥用 K8s Secret / Vault（不入镜像）
- [ ] readOnlyRootFilesystem: true
- [ ] capabilities.drop: [ALL]
- [ ] allowPrivilegeEscalation: false
- [ ] Network Policy 限制 Pod 间通信
- [ ] Resource limits 设置（CPU + memory）
- [ ] RBAC 最小权限（不用 cluster-admin）
- [ ] TLS 加密所有通信
- [ ] 镜像用固定 tag（不用 :latest）
