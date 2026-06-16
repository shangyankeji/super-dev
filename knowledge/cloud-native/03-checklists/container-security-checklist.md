---
title: 容器安全检查清单
version: 1.0.0
last_updated: 2025-03-20
owner: security-team
tags: [container, security, docker, checklist]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：容器安全全面检查清单
# 作用：确保容器镜像和运行时满足安全标准
# 创建时间：2025-03-20
# 最后修改：2025-03-20

## 概述

本检查清单覆盖容器安全的全生命周期，包括镜像构建、运行时配置、网络安全和持续监控。

**检查项分类：**
- [P0] 必须完成（阻塞发布）
- [P1] 强烈建议（应在发布前完成）
- [P2] 推荐完成（可后续迭代）

---

## 1. 镜像安全

### 1.1 基础镜像 [P0]

- [ ] **官方镜像**：使用官方维护的基础镜像
- [ ] **版本固定**：使用确定版本标签（如 `python:3.11-slim`），禁止 `latest`
- [ ] **Digest 验证**：使用镜像 digest 进行完整性验证
- [ ] **最小化镜像**：优先选择 alpine/distroless 等小镜像
- [ ] **镜像来源可信**：镜像来自可信仓库

```dockerfile
# [DONE] 正确示例
FROM python:3.11-slim-bookworm@sha256:abc123...

# [FAIL] 错误示例
FROM python:latest
FROM ubuntu  # 无版本标签
```

### 1.2 漏洞扫描 [P0]

- [ ] **扫描工具配置**：配置漏洞扫描工具（Trivy/Clair/Grype）
- [ ] **无严重漏洞**：无 Critical 级别漏洞
- [ ] **无高危漏洞**：无 High 级别漏洞（或已评估风险）
- [ ] **定期扫描**：配置定期扫描计划
- [ ] **基线镜像扫描**：基础镜像更新时重新扫描

```bash
# Trivy 扫描示例
trivy image --severity HIGH,CRITICAL --exit-code 1 myimage:v1.0.0

# 输出示例
# Total: 0 (CRITICAL: 0, HIGH: 0)
```

### 1.3 镜像签名 [P1]

- [ ] **签名工具**：配置镜像签名工具（cosign/Notary）
- [ ] **镜像已签名**：所有生产镜像已签名
- [ ] **签名验证**：部署时验证签名
- [ ] **密钥管理**：签名密钥安全存储
- [ ] **签名策略**：定义签名策略

```bash
# cosign 签名
cosign sign --key cosign.key myimage:v1.0.0

# cosign 验证
cosign verify --key cosign.pub myimage:v1.0.0
```

### 1.4 敏感信息 [P0]

- [ ] **无硬编码密钥**：镜像中无硬编码密码/密钥
- [ ] **无敏感文件**：.env、credentials 等文件已排除
- [ ] **.dockerignore 配置**：正确配置 .dockerignore
- [ ] **构建参数安全**：ARG 参数不包含敏感信息
- [ ] **历史记录清理**：镜像层中无敏感信息

```dockerfile
# .dockerignore 示例
.env
.env.*
credentials.json
*.key
*.pem
.git
```

---

## 2. 构建安全

### 2.1 Dockerfile 安全 [P0]

- [ ] **非 root 用户**：创建并切换到非 root 用户
- [ ] **最小权限**：仅安装必要软件包
- [ ] **层优化**：减少镜像层数
- [ ] **缓存清理**：安装后清理包管理器缓存
- [ ] **无敏感信息**：Dockerfile 中无密码/密钥

```dockerfile
# [DONE] 安全 Dockerfile
FROM python:3.11-slim AS builder
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

FROM python:3.11-slim
RUN groupadd -r appgroup && useradd -r -g appgroup appuser
WORKDIR /app
COPY --from=builder /usr/local/lib/python3.11/site-packages /usr/local/lib/python3.11/site-packages
COPY --chown=appuser:appgroup . .
USER appuser
CMD ["python", "app.py"]
```

### 2.2 多阶段构建 [P1]

- [ ] **使用多阶段构建**：分离构建和运行环境
- [ ] **构建工具隔离**：构建工具不进入最终镜像
- [ ] **最小化最终镜像**：最终镜像仅包含运行时必需文件

```dockerfile
# 多阶段构建示例
FROM golang:1.21 AS builder
WORKDIR /app
COPY . .
RUN CGO_ENABLED=0 go build -o app

FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/app /
USER nonroot:nonroot
ENTRYPOINT ["/app"]
```

### 2.3 依赖管理 [P0]

- [ ] **依赖锁定**：使用 lock 文件固定依赖版本
- [ ] **依赖扫描**：扫描依赖漏洞
- [ ] **最小依赖**：仅安装必要依赖
- [ ] **可信源**：依赖来自可信源

---

## 3. 运行时安全

### 3.1 用户权限 [P0]

- [ ] **非 root 运行**：容器以非 root 用户运行
- [ ] **用户 ID 固定**：指定确定的 UID/GID
- [ ] **禁止 root 容器**：禁止使用 root 用户
- [ ] **用户命名空间**：启用用户命名空间（如可能）

```yaml
# Kubernetes Pod 安全配置
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  runAsGroup: 1000
  fsGroup: 1000
```

### 3.2 能力限制 [P0]

- [ ] **丢弃所有能力**：`capabilities.drop: [ALL]`
- [ ] **最小能力**：仅添加必需能力
- [ ] **禁止特权**：`privileged: false`
- [ ] **禁止特权提升**：`allowPrivilegeEscalation: false`

```yaml
# 能力限制配置
securityContext:
  allowPrivilegeEscalation: false
  capabilities:
    drop:
    - ALL
    # add: []  # 仅在必要时添加
```

### 3.3 文件系统 [P0]

- [ ] **只读根文件系统**：`readOnlyRootFilesystem: true`
- [ ] **临时目录挂载**：需要写入的目录挂载 emptyDir
- [ ] **禁止挂载主机**：不挂载主机敏感目录
- [ ] **文件系统类型限制**：限制文件系统类型

```yaml
# 只读根文件系统
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
  emptyDir:
    sizeLimit: "100Mi"
```

### 3.4 Seccomp/AppArmor [P1]

- [ ] **Seccomp 配置**：启用 seccomp 配置
- [ ] **AppArmor 配置**：启用 AppArmor 配置
- [ ] **默认配置使用**：使用 RuntimeDefault

```yaml
# Seccomp 配置
securityContext:
  seccompProfile:
    type: RuntimeDefault

# AppArmor 注解
annotations:
  container.apparmor.security.beta.kubernetes.io/app: localhost/apparmor-profile
```

---

## 4. 网络安全

### 4.1 网络隔离 [P0]

- [ ] **网络策略配置**：NetworkPolicy 已配置
- [ ] **默认拒绝入站**：入站流量默认拒绝
- [ ] **默认拒绝出站**：出站流量默认拒绝
- [ ] **最小访问**：仅允许必要通信
- [ ] **命名空间隔离**：跨命名空间访问控制

```yaml
# 默认拒绝策略
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
```

### 4.2 端口管理 [P0]

- [ ] **最小暴露端口**：仅暴露必要端口
- [ ] **端口范围**：不使用特权端口（< 1024）
- [ ] **端口命名**：端口有明确命名

### 4.3 服务访问 [P0]

- [ ] **服务网格**：使用服务网格进行 mTLS
- [ ] **Ingress 安全**：Ingress 配置 TLS
- [ ] **API 网关**：通过 API 网关访问
- [ ] **限流配置**：配置 API 限流

---

## 5. Secret 管理

### 5.1 Secret 存储 [P0]

- [ ] **无明文 Secret**：Secret 不以明文存储
- [ ] **etcd 加密**：etcd 启用加密
- [ ] **外部密钥管理**：敏感 Secret 使用外部 KMS
- [ ] **Secret 轮换**：定期轮换 Secret

```yaml
# External Secrets 配置
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: db-credentials
spec:
  secretStoreRef:
    name: vault-backend
  target:
    name: db-credentials
  data:
  - secretKey: password
    remoteRef:
      key: secret/data/production/database
      property: password
```

### 5.2 Secret 访问 [P0]

- [ ] **最小访问**：RBAC 限制 Secret 访问
- [ ] **命名空间隔离**：Secret 不跨命名空间共享
- [ ] **审计日志**：Secret 访问有审计日志

---

## 6. 资源限制

### 6.1 资源配额 [P0]

- [ ] **CPU 限制**：limits.cpu 已配置
- [ ] **内存限制**：limits.memory 已配置
- [ ] **资源请求**：requests 已配置
- [ ] **合理配置**：资源配置经过测试验证

```yaml
# 资源配置
resources:
  requests:
    cpu: "100m"
    memory: "128Mi"
  limits:
    cpu: "500m"
    memory: "512Mi"
```

### 6.2 限制范围 [P1]

- [ ] **LimitRange 配置**：命名空间配置 LimitRange
- [ ] **ResourceQuota 配置**：命名空间配置 ResourceQuota
- [ ] **防止资源耗尽**：防止资源过度使用

---

## 7. 监控与审计

### 7.1 运行时监控 [P0]

- [ ] **容器运行时安全**：部署运行时安全工具（Falco）
- [ ] **异常检测**：检测异常行为
- [ ] **实时告警**：安全事件实时告警
- [ ] **日志采集**：安全日志集中采集

```yaml
# Falco 规则示例
- rule: Container Drift Detected
  desc: Detect if a container has been modified at runtime
  condition: >
    spawned_process and container and
    proc.name in (apt, apt-get, yum, dnf, apk, pip, npm, gem)
  output: >
    Container drift detected (user=%user.name container=%container.id
    process=%proc.name parent=%proc.pname)
  priority: WARNING
```

### 7.2 审计日志 [P0]

- [ ] **Kubernetes 审计**：启用 API Server 审计
- [ ] **日志完整性**：日志不可篡改
- [ ] **日志保留**：日志保留符合合规要求
- [ ] **日志分析**：定期分析审计日志

```yaml
# 审计策略配置
apiVersion: audit.k8s.io/v1
kind: Policy
rules:
- level: RequestResponse
  resources:
  - group: ""
    resources: ["secrets"]
  verbs: ["get", "create", "update", "delete"]
```

### 7.3 合规检查 [P1]

- [ ] **CIS Benchmark**：通过 CIS Benchmark 检查
- [ ] **合规扫描**：定期合规扫描
- [ ] **修复追踪**：合规问题追踪修复

---

## 8. 供应链安全

### 8.1 来源验证 [P0]

- [ ] **镜像来源可信**：镜像来源可信
- [ ] **签名验证**：验证镜像签名
- [ ] **SBOM 生成**：生成软件物料清单
- [ ] **依赖来源**：依赖来源可信

### 8.2 构建安全 [P0]

- [ ] **构建隔离**：构建环境隔离
- [ ] **构建日志**：构建日志保存
- [ ] **构建可追溯**：构建可追溯

---

## 检查评分

### 评分标准

- **通过**：所有 [P0] 检查项完成
- **有条件通过**：所有 [P0] 完成，[P1] 完成率 >= 80%
- **不通过**：存在未完成的 [P0] 检查项

### 检查结果

| 类别 | P0 完成 | P1 完成 | 状态 |
|------|---------|---------|------|
| 镜像安全 | /10 | /5 | [ ] |
| 构建安全 | /5 | /3 | [ ] |
| 运行时安全 | /10 | /4 | [ ] |
| 网络安全 | /8 | /0 | [ ] |
| Secret 管理 | /5 | /0 | [ ] |
| 资源限制 | /4 | /3 | [ ] |
| 监控与审计 | /5 | /3 | [ ] |
| 供应链安全 | /4 | /0 | [ ] |
| **总计** | **/51** | **/18** | [ ] |

---

## 参考资料

- [CIS Docker Benchmark](https://www.cisecurity.org/benchmark/docker)
- [CIS Kubernetes Benchmark](https://www.cisecurity.org/benchmark/kubernetes)
- [OWASP Docker Security](https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html)
- [NIST SP 800-190](https://csrc.nist.gov/publications/detail/sp/800-190/final)
- [Trivy 文档](https://aquasecurity.github.io/trivy/)
- [Falco 文档](https://falco.org/docs/)