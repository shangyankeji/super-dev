---
id: container-runtime-security
title: 容器运行时安全
domain: security
category: 01-standards
difficulty: intermediate
tags: [agent, checklist, container, runtime, security, 实战代码示例, 常见陷阱, 最佳实践]
quality_score: 70
last_updated: 2026-06-15
---
# 容器运行时安全

## 概述
容器化应用面临独特的安全挑战:镜像漏洞、特权逃逸、网络暴露、供应链风险等。本指南覆盖镜像扫描、非root运行、seccomp、AppArmor、网络策略等完整的容器运行时安全实践。

## 核心概念

### 1. 容器安全层次
| 层次 | 风险 | 防护措施 |
|------|------|----------|
| 镜像构建 | 漏洞基础镜像、恶意依赖 | 镜像扫描、最小基础镜像、多阶段构建 |
| 运行时 | 特权逃逸、资源滥用 | 非root、只读根文件系统、资源限制 |
| 系统调用 | 内核利用 | seccomp、AppArmor/SELinux |
| 网络 | 横向移动、数据泄露 | NetworkPolicy、Service Mesh |
| 编排层 | RBAC滥用、API暴露 | 最小RBAC、API审计、准入控制 |

### 2. 安全原则
- **最小权限**: 容器只拥有运行所需的最低权限
- **不可变基础设施**: 运行时不修改容器内容
- **纵深防御**: 多层安全控制,任何一层被突破不致全面失守
- **最小攻击面**: 减少容器内的工具和文件

### 3. 容器威胁模型
- **镜像威胁**: 含CVE的基础镜像、恶意层、泄露的密钥
- **运行时威胁**: 容器逃逸、特权提升、敏感挂载
- **网络威胁**: 容器间无限制通信、暴露管理端口
- **编排威胁**: Kubernetes API未授权访问、RBAC过宽

## 实战代码示例

### 安全Dockerfile(最佳实践)

```dockerfile
# ======= 构建阶段 =======
FROM python:3.12-slim AS builder

WORKDIR /build

# 先复制依赖文件(利用缓存)
COPY pyproject.toml uv.lock ./
RUN pip install uv && uv sync --no-dev --frozen

# 复制源代码
COPY super_dev/ ./super_dev/

# ======= 运行阶段 =======
FROM python:3.12-slim AS runtime

# 安全加固
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y --no-install-recommends tini && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* && \
    # 移除不需要的工具
    rm -f /usr/bin/wget /usr/bin/curl && \
    # 创建非root用户
    groupadd -r appuser && \
    useradd -r -g appuser -d /app -s /sbin/nologin appuser

WORKDIR /app

# 从构建阶段复制
COPY --from=builder /build/.venv /app/.venv
COPY --from=builder /build/super_dev /app/super_dev

# 设置环境
ENV PATH="/app/.venv/bin:$PATH" \
    PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1

# 非root用户运行
USER appuser

# 使用tini作为PID 1进程
ENTRYPOINT ["tini", "--"]

# 健康检查
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD python -c "import httpx; httpx.get('http://localhost:8000/health').raise_for_status()"

EXPOSE 8000

CMD ["uvicorn", "super_dev.web.api:app", "--host", "0.0.0.0", "--port", "8000"]
```

### 镜像扫描CI集成

```yaml
# .github/workflows/container-security.yml
name: Container Security
on:
  push:
    paths:
      - 'Dockerfile*'
      - '*.lock'
      - 'pyproject.toml'

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build image
        run: docker build -t myapp:${{ github.sha }} .

      # Trivy扫描
      - name: Trivy vulnerability scan
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: 'myapp:${{ github.sha }}'
          format: 'sarif'
          output: 'trivy-results.sarif'
          severity: 'HIGH,CRITICAL'
          exit-code: '1'  # 发现高危漏洞时失败

      # Dockle最佳实践检查
      - name: Dockle lint
        uses: erzz/dockle-action@v1
        with:
          image: 'myapp:${{ github.sha }}'
          exit-code: '1'
          failure-threshold: 'WARN'

      # Hadolint Dockerfile静态分析
      - name: Hadolint
        uses: hadolint/hadolint-action@v3
        with:
          dockerfile: Dockerfile
          failure-threshold: warning

      - name: Upload scan results
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: 'trivy-results.sarif'
```

### Kubernetes安全Pod配置

```yaml
# 安全的Pod配置
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
  namespace: production
spec:
  replicas: 3
  selector:
    matchLabels:
      app: api-server
  template:
    metadata:
      labels:
        app: api-server
    spec:
      # 使用服务账号(非default)
      serviceAccountName: api-server-sa
      automountServiceAccountToken: false  # 不自动挂载SA token

      # Pod级安全上下文
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        runAsGroup: 1000
        fsGroup: 1000
        seccompProfile:
          type: RuntimeDefault

      containers:
        - name: api
          image: myregistry.com/api-server:v1.2.0@sha256:abc123...  # 使用digest
          imagePullPolicy: Always

          # 容器级安全上下文
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: true
            capabilities:
              drop:
                - ALL
              # 只添加需要的capability
              # add:
              #   - NET_BIND_SERVICE

          # 资源限制(防止资源滥用)
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 512Mi
              ephemeral-storage: 100Mi

          # 探针
          livenessProbe:
            httpGet:
              path: /health
              port: 8000
            initialDelaySeconds: 10
            periodSeconds: 30
          readinessProbe:
            httpGet:
              path: /ready
              port: 8000
            initialDelaySeconds: 5
            periodSeconds: 10

          # 环境变量(密钥从Secret读取)
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: db-credentials
                  key: url
            - name: ENV
              value: "production"

          # 挂载
          volumeMounts:
            - name: tmp
              mountPath: /tmp
            - name: app-config
              mountPath: /app/config
              readOnly: true

          ports:
            - containerPort: 8000
              protocol: TCP

      volumes:
        - name: tmp
          emptyDir:
            sizeLimit: 100Mi
        - name: app-config
          configMap:
            name: api-config

      # 拓扑分散(高可用)
      topologySpreadConstraints:
        - maxSkew: 1
          topologyKey: kubernetes.io/hostname
          whenUnsatisfiable: DoNotSchedule
          labelSelector:
            matchLabels:
              app: api-server
```

### seccomp配置

```json
{
  "defaultAction": "SCMP_ACT_ERRNO",
  "architectures": ["SCMP_ARCH_X86_64"],
  "syscalls": [
    {
      "names": [
        "accept4", "access", "arch_prctl", "bind", "brk",
        "clock_gettime", "clone", "close", "connect",
        "dup", "dup2", "epoll_create1", "epoll_ctl", "epoll_wait",
        "exit", "exit_group", "fchmod", "fchown", "fcntl",
        "fstat", "futex", "getcwd", "getdents64", "getegid",
        "geteuid", "getgid", "getpid", "getppid", "getsockname",
        "getsockopt", "getuid", "ioctl", "listen", "lseek",
        "madvise", "mmap", "mprotect", "munmap", "nanosleep",
        "newfstatat", "openat", "pipe2", "poll", "pread64",
        "pwrite64", "read", "readlink", "recvfrom", "recvmsg",
        "rt_sigaction", "rt_sigprocmask", "rt_sigreturn",
        "sendmsg", "sendto", "set_robust_list", "set_tid_address",
        "setsockopt", "shutdown", "sigaltstack", "socket",
        "stat", "sysinfo", "tgkill", "uname", "unlink",
        "wait4", "write", "writev"
      ],
      "action": "SCMP_ACT_ALLOW"
    }
  ]
}
```

### 准入控制(Kyverno策略)

```yaml
# 强制安全基线
apiVersion: kyverno.io/v1
kind: ClusterPolicy
metadata:
  name: pod-security-baseline
spec:
  validationFailureAction: Enforce
  rules:
    # 禁止特权容器
    - name: deny-privileged
      match:
        resources:
          kinds: ["Pod"]
      validate:
        message: "Privileged containers are not allowed"
        pattern:
          spec:
            containers:
              - securityContext:
                  privileged: "false"

    # 要求非root运行
    - name: require-non-root
      match:
        resources:
          kinds: ["Pod"]
      validate:
        message: "Containers must run as non-root"
        pattern:
          spec:
            securityContext:
              runAsNonRoot: true
            containers:
              - securityContext:
                  allowPrivilegeEscalation: false

    # 要求资源限制
    - name: require-resource-limits
      match:
        resources:
          kinds: ["Pod"]
      validate:
        message: "CPU and memory limits are required"
        pattern:
          spec:
            containers:
              - resources:
                  limits:
                    memory: "?*"
                    cpu: "?*"

    # 禁止hostPath挂载
    - name: deny-host-path
      match:
        resources:
          kinds: ["Pod"]
      validate:
        message: "HostPath volumes are not allowed"
        pattern:
          spec:
            =(volumes):
              - X(hostPath): "null"

    # 要求只读根文件系统
    - name: require-readonly-rootfs
      match:
        resources:
          kinds: ["Pod"]
      validate:
        message: "Root filesystem must be read-only"
        pattern:
          spec:
            containers:
              - securityContext:
                  readOnlyRootFilesystem: true

    # 要求镜像使用digest
    - name: require-image-digest
      match:
        resources:
          kinds: ["Pod"]
      validate:
        message: "Images must use digest (sha256)"
        pattern:
          spec:
            containers:
              - image: "*@sha256:*"

---
# 自动注入安全默认值
apiVersion: kyverno.io/v1
kind: ClusterPolicy
metadata:
  name: add-security-defaults
spec:
  rules:
    - name: add-seccomp-profile
      match:
        resources:
          kinds: ["Pod"]
      mutate:
        patchStrategicMerge:
          spec:
            securityContext:
              seccompProfile:
                type: RuntimeDefault
```

### 运行时监控(Falco规则)

```yaml
# Falco自定义规则
- rule: Container Shell Spawned
  desc: Detect shell spawned in container
  condition: >
    spawned_process and container and
    proc.name in (bash, sh, zsh, dash) and
    not proc.pname in (entrypoint.sh, tini)
  output: >
    Shell spawned in container (user=%user.name container=%container.name
    shell=%proc.name parent=%proc.pname cmdline=%proc.cmdline)
  priority: WARNING

- rule: Sensitive File Access
  desc: Detect access to sensitive files
  condition: >
    open_read and container and
    fd.name in (/etc/shadow, /etc/passwd, /proc/1/environ)
  output: >
    Sensitive file accessed (user=%user.name file=%fd.name
    container=%container.name)
  priority: CRITICAL

- rule: Outbound Connection to Unusual Port
  desc: Detect connections to non-standard ports
  condition: >
    outbound and container and
    not fd.sport in (80, 443, 8080, 8443, 5432, 6379, 9092)
  output: >
    Unusual outbound connection (container=%container.name
    connection=%fd.name port=%fd.sport)
  priority: WARNING

- rule: Package Management in Container
  desc: Detect package installation in running container
  condition: >
    spawned_process and container and
    proc.name in (apt, apt-get, yum, dnf, apk, pip, npm)
  output: >
    Package management detected in running container
    (container=%container.name command=%proc.cmdline)
  priority: ERROR
```

## 最佳实践

### 1. 镜像安全
- 使用最小基础镜像(distroless/alpine/slim)
- 多阶段构建,运行镜像不含构建工具
- 定期扫描并更新基础镜像
- 使用镜像digest而非tag(防篡改)
- 签名验证(cosign/Notary)

### 2. 运行时加固
- 非root运行(runAsNonRoot: true)
- 只读根文件系统(readOnlyRootFilesystem: true)
- 禁止特权提升(allowPrivilegeEscalation: false)
- 删除所有Linux Capabilities(drop: ALL)
- 设置资源限制(CPU/内存/存储)

### 3. 密钥管理
- 不要在镜像中存储密钥
- 使用Kubernetes Secrets(配合加密)
- 推荐: External Secrets Operator + Vault/AWS SM
- 密钥自动轮换

### 4. 网络安全
- 默认拒绝所有流量(NetworkPolicy)
- 按服务对精确放行
- 出站流量同样限制
- 使用Service Mesh加密服务间通信

### 5. 审计与监控
- 启用Kubernetes审计日志
- 部署运行时安全监控(Falco)
- 监控异常行为(shell执行/网络异常)
- 容器镜像持续扫描(新CVE)

## 常见陷阱

### 陷阱1: 以root运行容器
```dockerfile
# 错误: 默认root运行
FROM python:3.12
COPY . /app
CMD ["python", "app.py"]

# 正确: 创建并使用非root用户
FROM python:3.12-slim
RUN useradd -r -s /sbin/nologin appuser
USER appuser
COPY --chown=appuser:appuser . /app
CMD ["python", "app.py"]
```

### 陷阱2: 使用latest标签
```yaml
# 错误: tag可能被覆盖
image: myapp:latest

# 正确: 使用不可变的digest
image: myapp@sha256:abc123def456...
```

### 陷阱3: 在镜像中包含密钥
```dockerfile
# 错误: 密钥在镜像层中永久存在
COPY .env /app/.env
# 即使后面删除也在中间层可见

# 正确: 通过环境变量或Volume注入
# 使用Kubernetes Secrets
```

### 陷阱4: 不限制资源
```yaml
# 错误: 无资源限制,可能OOM Kill其他Pod
containers:
  - name: app
    image: myapp:v1

# 正确: 设置合理的资源限制
containers:
  - name: app
    image: myapp:v1
    resources:
      requests:
        cpu: 100m
        memory: 128Mi
      limits:
        cpu: 500m
        memory: 512Mi
```

### 陷阱5: 自动挂载ServiceAccount Token
```yaml
# 错误: 默认挂载SA Token,被入侵后可调用K8s API
# 正确: 除非需要,否则禁用
spec:
  automountServiceAccountToken: false
```

## Agent Checklist

### 镜像安全
- [ ] 使用最小基础镜像
- [ ] 多阶段构建已使用
- [ ] 镜像扫描集成到CI
- [ ] 使用digest引用镜像
- [ ] 无密钥/凭证包含在镜像中

### 运行时安全
- [ ] 容器以非root运行
- [ ] 只读根文件系统
- [ ] 特权提升已禁止
- [ ] Capabilities全部drop
- [ ] 资源限制已设置

### 网络与通信
- [ ] NetworkPolicy默认拒绝已应用
- [ ] 服务间通信加密(mTLS)
- [ ] 出站流量受控
- [ ] 管理端口不暴露

### 编排安全
- [ ] RBAC最小权限
- [ ] SA Token不自动挂载
- [ ] 准入控制策略已部署
- [ ] 运行时监控已启用(Falco等)
