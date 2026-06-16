---
title: Kubernetes 故障排查作战手册
version: 1.0.0
last_updated: 2026-03-28
owner: platform-team
tags: [kubernetes, troubleshooting, diagnostics, pod, network, storage, node]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：Kubernetes 全场景故障排查作战手册
# 作用：指导 K8s 集群各类故障的诊断、根因分析与修复
# 创建时间：2026-03-28
# 最后修改：2026-03-28

## 目标

建立 Kubernetes 故障排查标准化流程，确保：
- 故障快速定位，MTTR 控制在 15 分钟以内
- 诊断流程可复现、可追溯
- 运维人员按手册即可独立完成 90% 常见故障修复
- 故障根因归档，形成持续改进闭环

## 适用场景

- Pod 生命周期异常（CrashLoopBackOff / ImagePullBackOff / OOMKilled / Pending / Evicted）
- 网络连通性故障（Service 不可达 / DNS 解析失败 / NetworkPolicy 阻断 / Ingress 5xx）
- 存储故障（PVC Pending / 挂载失败 / 权限问题）
- 节点故障（NotReady / 磁盘压力 / 内存压力 / PID 压力）
- 资源瓶颈（CPU Throttling / 内存泄漏 / HPA 不生效）
- 部署故障（滚动更新卡住 / 回滚 / ConfigMap/Secret 更新不生效）

---

## 一、通用诊断入口

在定位任何故障之前，先执行以下命令建立全局视图：

```bash
# 集群整体健康状态
kubectl get nodes -o wide
kubectl get cs                          # 控制平面组件状态（1.19+ 已废弃，改用下方）
kubectl get --raw='/readyz?verbose'     # API Server 就绪探针

# 当前命名空间异常资源速查
kubectl get pods --field-selector=status.phase!=Running,status.phase!=Succeeded
kubectl get events --sort-by='.lastTimestamp' | tail -30

# 资源总览
kubectl top nodes
kubectl top pods --sort-by=memory
```

> **原则**：先看 Events，再看 Describe，最后看 Logs。90% 的故障在 Events 阶段即可定位。

---

## 二、Pod 故障排查

### 2.1 CrashLoopBackOff

**症状**

- Pod 反复重启，`RESTARTS` 计数持续增长
- `kubectl get pods` 显示状态 `CrashLoopBackOff`
- 退避时间从 10s 递增至 5min

**诊断命令**

```bash
# 1. 查看 Pod 事件
kubectl describe pod <pod-name> -n <namespace>

# 2. 查看当前容器日志
kubectl logs <pod-name> -n <namespace> --tail=100

# 3. 查看上一次崩溃的日志（关键）
kubectl logs <pod-name> -n <namespace> --previous --tail=200

# 4. 如果是多容器 Pod
kubectl logs <pod-name> -c <container-name> --previous

# 5. 查看退出码
kubectl get pod <pod-name> -o jsonpath='{.status.containerStatuses[0].lastState.terminated}'
```

**退出码速查表**

| 退出码 | 含义 | 常见原因 |
|--------|------|----------|
| 0 | 正常退出 | 一次性任务完成但 restartPolicy=Always |
| 1 | 应用错误 | 未捕获异常、配置错误 |
| 126 | 权限不足 | 二进制文件不可执行 |
| 127 | 命令未找到 | entrypoint/cmd 路径错误 |
| 137 | SIGKILL (OOM) | 内存超限被 cgroup 杀死 |
| 139 | SIGSEGV | 段错误，原生代码 bug |
| 143 | SIGTERM | 正常终止信号，但进程未优雅关闭 |

**根因分析**

1. **应用代码崩溃**：查看 `--previous` 日志中的异常栈
2. **配置缺失**：环境变量、ConfigMap、Secret 未挂载或值错误
3. **依赖服务不可用**：数据库 / 消息队列 / 外部 API 连接失败
4. **健康检查过严**：livenessProbe 超时导致容器被反复杀死
5. **启动顺序依赖**：initContainer 未正确配置

**解决方案**

```yaml
# 场景：livenessProbe 过于激进导致循环重启
# 修复：增大 initialDelaySeconds 和 timeoutSeconds
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 30      # 给应用足够的启动时间
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3
  successThreshold: 1

# 场景：依赖服务未就绪
# 修复：使用 initContainer 等待依赖
initContainers:
- name: wait-for-db
  image: busybox:1.36
  command: ['sh', '-c', 'until nc -z db-service 5432; do echo waiting; sleep 2; done']
```

**预防措施**

- [ ] 所有服务必须配置合理的 startupProbe（慢启动应用）
- [ ] livenessProbe 的 initialDelaySeconds >= 应用平均启动时间的 1.5 倍
- [ ] 关键依赖使用 initContainer 做就绪等待
- [ ] 应用内部实现优雅降级，避免依赖不可用直接 crash

---

### 2.2 ImagePullBackOff

**症状**

- Pod 状态卡在 `ImagePullBackOff` 或 `ErrImagePull`
- Events 中出现 `Failed to pull image` 或 `unauthorized`

**诊断命令**

```bash
# 1. 查看详细错误信息
kubectl describe pod <pod-name> -n <namespace> | grep -A 10 "Events:"

# 2. 检查镜像地址是否正确
kubectl get pod <pod-name> -o jsonpath='{.spec.containers[*].image}'

# 3. 检查 imagePullSecrets
kubectl get pod <pod-name> -o jsonpath='{.spec.imagePullSecrets}'

# 4. 检查 Secret 内容
kubectl get secret <secret-name> -n <namespace> -o jsonpath='{.data.\.dockerconfigjson}' | base64 -d

# 5. 在节点上手动拉取验证
crictl pull <image-url>
```

**根因分析**

1. **镜像不存在**：tag 拼写错误、镜像已被删除
2. **认证失败**：imagePullSecret 过期或配置错误
3. **网络不通**：节点无法访问镜像仓库（防火墙 / 代理）
4. **仓库限流**：Docker Hub 匿名拉取限制（100次/6小时）
5. **镜像架构不匹配**：ARM 节点拉取 AMD64 镜像

**解决方案**

```bash
# 创建/更新 imagePullSecret
kubectl create secret docker-registry regcred \
  --docker-server=registry.example.com \
  --docker-username=user \
  --docker-password=pass \
  --docker-email=user@example.com \
  -n <namespace> --dry-run=client -o yaml | kubectl apply -f -

# 配置 ServiceAccount 默认拉取凭证（推荐）
kubectl patch serviceaccount default -n <namespace> \
  -p '{"imagePullSecrets": [{"name": "regcred"}]}'

# Docker Hub 限流解决：配置镜像代理
# 在 containerd 配置中添加 mirror
# /etc/containerd/config.toml
# [plugins."io.containerd.grpc.v1.cri".registry.mirrors."docker.io"]
#   endpoint = ["https://mirror.example.com"]
```

**预防措施**

- [ ] CI/CD 流水线中验证镜像推送成功后再触发部署
- [ ] 使用确定性 tag（如 sha256 digest），禁止线上使用 `latest`
- [ ] imagePullSecret 通过 Sealed Secrets 或 External Secrets Operator 管理
- [ ] 建立私有镜像仓库，避免公网依赖

---

### 2.3 OOMKilled

**症状**

- Pod 状态 `OOMKilled`，退出码 137
- `kubectl describe pod` 中 `Reason: OOMKilled`
- 节点 dmesg 中出现 `oom-kill` 日志

**诊断命令**

```bash
# 1. 确认 OOM 事件
kubectl get pod <pod-name> -o jsonpath='{.status.containerStatuses[0].lastState.terminated.reason}'

# 2. 查看当前内存用量
kubectl top pod <pod-name> --containers

# 3. 查看 resource limits
kubectl get pod <pod-name> -o jsonpath='{.spec.containers[0].resources}'

# 4. 节点级别 OOM 日志
kubectl get events --field-selector reason=OOMKilling -A

# 5. 查看 cgroup 内存统计（需要 SSH 到节点）
cat /sys/fs/cgroup/memory/kubepods/pod<uid>/<container-id>/memory.max_usage_in_bytes
```

**根因分析**

1. **limits 设置过低**：应用正常峰值超过 memory limits
2. **内存泄漏**：应用长时间运行后内存持续增长
3. **突发流量**：瞬时请求量导致内存激增
4. **JVM 堆外内存**：Java 应用 native memory 未计入 -Xmx
5. **子进程内存**：容器内启动的子进程超出预期

**解决方案**

```yaml
# 1. 调整 limits（基于实际观测值 + 20% 缓冲）
resources:
  requests:
    memory: "512Mi"
  limits:
    memory: "768Mi"       # requests 的 1.5 倍

# 2. Java 应用示例：限制堆 + 堆外
env:
- name: JAVA_OPTS
  value: >-
    -Xms256m -Xmx512m
    -XX:MaxMetaspaceSize=128m
    -XX:MaxDirectMemorySize=64m
    -XX:+UseContainerSupport
    -XX:MaxRAMPercentage=75.0

# 3. Go 应用：设置 GOMEMLIMIT
env:
- name: GOMEMLIMIT
  value: "600MiB"         # limits 的 ~80%
```

**预防措施**

- [ ] 所有容器必须设置 memory requests 和 limits
- [ ] 生产环境通过压测确定资源基线，limits = P99 * 1.5
- [ ] Java 应用必须配置 `-XX:+UseContainerSupport`
- [ ] 配置 Prometheus 告警：容器内存使用率 > 80% 触发预警
- [ ] 定期执行内存 profiling（pprof / async-profiler）

---

### 2.4 Pending

**症状**

- Pod 长时间处于 `Pending` 状态
- `kubectl describe pod` 的 Events 中出现调度失败信息

**诊断命令**

```bash
# 1. 查看调度事件
kubectl describe pod <pod-name> -n <namespace> | grep -A 20 "Events:"

# 2. 查看节点可分配资源
kubectl describe nodes | grep -A 5 "Allocated resources"

# 3. 查看 Pod 的资源请求
kubectl get pod <pod-name> -o jsonpath='{.spec.containers[*].resources.requests}'

# 4. 查看 nodeSelector / affinity / tolerations
kubectl get pod <pod-name> -o yaml | grep -A 10 'nodeSelector\|affinity\|tolerations'

# 5. 检查 PVC 绑定状态
kubectl get pvc -n <namespace>
```

**根因分析**

1. **资源不足**：集群无节点满足 CPU/Memory requests
2. **nodeSelector 无匹配**：标签选择器与节点标签不匹配
3. **亲和性冲突**：requiredDuringScheduling 规则过严
4. **Taint 未容忍**：节点有 taint 但 Pod 无对应 toleration
5. **PVC 未绑定**：StorageClass 无法动态供给或 PV 不足
6. **Pod 数量限制**：节点 maxPods 已满（默认 110）
7. **ResourceQuota 超限**：命名空间配额已用完

**解决方案**

```bash
# 场景：资源不足 → 查看各节点可调度余量
kubectl get nodes -o custom-columns=\
  NAME:.metadata.name,\
  CPU_ALLOC:.status.allocatable.cpu,\
  MEM_ALLOC:.status.allocatable.memory,\
  PODS:.status.allocatable.pods

# 场景：Taint 导致 → 添加 toleration
# kubectl taint nodes <node> key=value:NoSchedule
# 在 Pod spec 中添加：
# tolerations:
# - key: "key"
#   operator: "Equal"
#   value: "value"
#   effect: "NoSchedule"

# 场景：ResourceQuota 超限
kubectl get resourcequota -n <namespace>
kubectl describe resourcequota <name> -n <namespace>
```

**预防措施**

- [ ] 配置 Cluster Autoscaler，资源不足时自动扩节点
- [ ] 为非核心负载配置 PriorityClass，资源紧张时可被抢占
- [ ] 定期审计 ResourceQuota 和 LimitRange 配置
- [ ] 使用 `kubectl-resource-capacity` 插件监控集群利用率

---

### 2.5 Evicted

**症状**

- Pod 状态 `Evicted`，大量 Evicted Pod 残留
- Events 中出现 `The node was low on resource: ephemeral-storage / memory`

**诊断命令**

```bash
# 1. 查看被驱逐的 Pod
kubectl get pods --field-selector=status.phase=Failed -A | grep Evicted

# 2. 查看驱逐原因
kubectl get pod <evicted-pod> -o jsonpath='{.status.reason} {.status.message}'

# 3. 节点资源压力状态
kubectl describe node <node-name> | grep -A 5 "Conditions:"

# 4. 批量清理 Evicted Pod
kubectl get pods -A --field-selector=status.phase=Failed | grep Evicted | \
  awk '{print "kubectl delete pod " $2 " -n " $1}' | sh
```

**根因分析**

1. **临时存储超限**：容器写入日志 / 临时文件超过 ephemeral-storage limits
2. **节点磁盘压力**：节点磁盘使用率超过驱逐阈值（默认 85%）
3. **节点内存压力**：系统内存不足触发 kubelet 驱逐
4. **镜像 / 容器垃圾积累**：未配置 GC 导致磁盘空间耗尽

**解决方案**

```yaml
# 设置 ephemeral-storage 限制
resources:
  requests:
    ephemeral-storage: "1Gi"
  limits:
    ephemeral-storage: "2Gi"

# 配置 kubelet 垃圾回收（节点级别）
# /var/lib/kubelet/config.yaml
# imageGCHighThresholdPercent: 85
# imageGCLowThresholdPercent: 80
# evictionHard:
#   memory.available: "100Mi"
#   nodefs.available: "10%"
#   imagefs.available: "15%"
```

**预防措施**

- [ ] 日志输出到 stdout，由日志采集系统收集，不写本地文件
- [ ] 配置 ephemeral-storage limits 防止磁盘滥用
- [ ] 节点磁盘使用率告警阈值设为 70%
- [ ] 定期清理无用镜像和已完成的 Job/Pod

---

## 三、网络故障排查

### 3.1 Service 不可达

**症状**

- Pod 内 `curl <service-name>:<port>` 超时或连接拒绝
- 外部流量无法到达后端 Pod

**诊断命令**

```bash
# 1. 检查 Service 是否存在且端口正确
kubectl get svc <service-name> -n <namespace> -o wide

# 2. 检查 Endpoints 是否有后端 Pod
kubectl get endpoints <service-name> -n <namespace>

# 3. 检查 Pod 标签是否与 Service selector 匹配
kubectl get pods -n <namespace> -l <selector-key>=<selector-value>

# 4. 从另一个 Pod 测试连通性
kubectl run debug --rm -it --image=nicolaka/netshoot -- bash
# 在 debug Pod 中
curl -v <service-name>.<namespace>.svc.cluster.local:<port>
nslookup <service-name>.<namespace>.svc.cluster.local

# 5. 检查 iptables/ipvs 规则（在节点上）
iptables -t nat -L KUBE-SERVICES | grep <service-cluster-ip>
ipvsadm -Ln | grep <service-cluster-ip>
```

**根因分析**

1. **Endpoints 为空**：Pod 标签与 Service selector 不匹配
2. **Pod 未就绪**：readinessProbe 失败导致从 Endpoints 中移除
3. **端口映射错误**：Service port / targetPort / containerPort 不一致
4. **kube-proxy 异常**：iptables/ipvs 规则未同步
5. **容器端口未监听**：应用绑定了 127.0.0.1 而非 0.0.0.0

**解决方案**

```bash
# 验证端口映射链：Service.port → Service.targetPort → Container.containerPort
kubectl get svc <svc> -o jsonpath='{.spec.ports}'
kubectl get pod <pod> -o jsonpath='{.spec.containers[*].ports}'

# 验证应用监听地址
kubectl exec <pod> -- ss -tlnp
# 确保监听 0.0.0.0:<port> 而非 127.0.0.1:<port>

# 重启 kube-proxy 刷新规则
kubectl rollout restart daemonset kube-proxy -n kube-system
```

**预防措施**

- [ ] 所有 Service 定义后立即验证 Endpoints 不为空
- [ ] readinessProbe 必须准确反映服务可用性
- [ ] 应用监听地址统一使用 `0.0.0.0`
- [ ] 使用 headless Service 时确认 DNS 解析行为

---

### 3.2 DNS 解析失败

**症状**

- Pod 内 `nslookup` / `dig` 无法解析 Service 名称
- 应用日志出现 `Name or service not known` / `no such host`

**诊断命令**

```bash
# 1. 检查 CoreDNS 运行状态
kubectl get pods -n kube-system -l k8s-app=kube-dns

# 2. 查看 CoreDNS 日志
kubectl logs -n kube-system -l k8s-app=kube-dns --tail=50

# 3. 从 Pod 内测试 DNS
kubectl exec <pod> -- nslookup kubernetes.default.svc.cluster.local
kubectl exec <pod> -- cat /etc/resolv.conf

# 4. 直接查询 CoreDNS ClusterIP
kubectl exec <pod> -- nslookup <service-name> $(kubectl get svc -n kube-system kube-dns -o jsonpath='{.spec.clusterIP}')

# 5. 检查 CoreDNS ConfigMap
kubectl get cm coredns -n kube-system -o yaml
```

**根因分析**

1. **CoreDNS Pod 异常**：OOM 或 CrashLoopBackOff
2. **resolv.conf 错误**：Pod 的 DNS 配置未指向 CoreDNS
3. **ndots 过高**：默认 ndots=5 导致多余的搜索查询超时
4. **上游 DNS 不通**：CoreDNS forward 到的外部 DNS 不可达
5. **CoreDNS 过载**：大量 DNS 查询导致延迟或丢弃

**解决方案**

```yaml
# 优化 DNS 查询（减少无效搜索）
spec:
  dnsConfig:
    options:
    - name: ndots
      value: "2"           # 默认 5，降为 2 减少搜索域拼接
    - name: single-request-reopen
      value: ""            # 避免 A/AAAA 并发导致的 conntrack 冲突

# CoreDNS 扩容
kubectl scale deployment coredns -n kube-system --replicas=3

# CoreDNS 启用缓存（在 Corefile 中确认）
# .:53 {
#     cache 30
#     ...
# }
```

**预防措施**

- [ ] CoreDNS 至少 2 副本，配置 PDB 保证可用性
- [ ] 高频 DNS 查询场景配置 NodeLocal DNSCache
- [ ] 监控 CoreDNS 的 QPS 和延迟指标
- [ ] 应用层面启用 DNS 连接池 / 缓存

---

### 3.3 NetworkPolicy 阻断

**症状**

- Pod 间通信突然中断
- `curl` 超时但 DNS 解析正常
- 新部署的 Pod 无法访问已有服务

**诊断命令**

```bash
# 1. 列出命名空间内所有 NetworkPolicy
kubectl get networkpolicy -n <namespace>

# 2. 查看 NetworkPolicy 详情
kubectl describe networkpolicy <name> -n <namespace>

# 3. 检查 Pod 的标签是否被 NetworkPolicy 选中
kubectl get pod <pod-name> --show-labels

# 4. 模拟验证（使用 debug Pod）
kubectl run debug --rm -it --image=nicolaka/netshoot -n <namespace> -- bash
# 在 debug Pod 中
curl -v --connect-timeout 3 <target-service>:<port>

# 5. 检查 CNI 插件是否支持 NetworkPolicy
kubectl get pods -n kube-system | grep -E 'calico|cilium|weave'
```

**根因分析**

1. **默认拒绝策略**：命名空间设了 default-deny 但未为新 Pod 添加 allow 规则
2. **标签不匹配**：NetworkPolicy 的 podSelector 或 namespaceSelector 不正确
3. **端口遗漏**：仅允许了部分端口，遗漏了实际使用的端口
4. **CNI 不支持**：使用了不支持 NetworkPolicy 的 CNI（如 Flannel 默认模式）
5. **egress 规则遗漏**：只配了 ingress，忘记配 egress

**解决方案**

```yaml
# 允许特定命名空间的 Pod 访问
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-from-frontend
  namespace: backend
spec:
  podSelector:
    matchLabels:
      app: api-server
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          tier: frontend
      podSelector:
        matchLabels:
          app: web
    ports:
    - protocol: TCP
      port: 8080

# 允许 DNS 出站（常被遗忘导致网络全断）
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-dns-egress
spec:
  podSelector: {}
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector: {}
    ports:
    - protocol: UDP
      port: 53
    - protocol: TCP
      port: 53
```

**预防措施**

- [ ] 每条 default-deny 策略必须配套 allow-dns-egress
- [ ] NetworkPolicy 变更后必须在 staging 环境验证连通性
- [ ] 使用 `kubectl-np-viewer` 可视化网络策略
- [ ] CI 中集成 NetworkPolicy 合规性检查

---

### 3.4 Ingress 502 / 504

**症状**

- 外部请求通过 Ingress 返回 502 Bad Gateway 或 504 Gateway Timeout
- 直接访问 Service ClusterIP 正常

**诊断命令**

```bash
# 1. 检查 Ingress 配置
kubectl get ingress <name> -n <namespace> -o yaml

# 2. 检查 Ingress Controller 日志
kubectl logs -n ingress-nginx -l app.kubernetes.io/component=controller --tail=100

# 3. 检查后端 Service 和 Endpoints
kubectl get svc <backend-svc> -n <namespace>
kubectl get endpoints <backend-svc> -n <namespace>

# 4. 测试 Ingress Controller 到后端的连通性
kubectl exec -n ingress-nginx <controller-pod> -- curl -v http://<service-cluster-ip>:<port>/

# 5. 查看 Ingress Controller 的 nginx.conf
kubectl exec -n ingress-nginx <controller-pod> -- cat /etc/nginx/nginx.conf | grep -A 20 "upstream"
```

**根因分析**

**502 Bad Gateway**：
1. 后端 Pod 全部不可用（0 Endpoints）
2. 后端 Pod 正在滚动更新，旧 Pod 已终止，新 Pod 未就绪
3. Service port 与 Ingress backend port 不一致
4. 后端返回的 HTTP 响应格式异常

**504 Gateway Timeout**：
1. 后端处理时间超过 Ingress Controller 的 proxy-read-timeout（默认 60s）
2. 网络延迟或丢包导致连接超时
3. 后端应用死锁或线程池耗尽

**解决方案**

```yaml
# 调整超时时间（nginx-ingress 注解）
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  annotations:
    nginx.ingress.kubernetes.io/proxy-connect-timeout: "10"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "120"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "120"
    nginx.ingress.kubernetes.io/proxy-body-size: "50m"

# 配置优雅终止，避免滚动更新时 502
# 在 Deployment 中设置
spec:
  template:
    spec:
      terminationGracePeriodSeconds: 60
      containers:
      - name: app
        lifecycle:
          preStop:
            exec:
              command: ["/bin/sh", "-c", "sleep 15"]   # 等待 Ingress 摘流
```

**预防措施**

- [ ] 后端 readinessProbe 必须准确，确保流量只到就绪 Pod
- [ ] 配置 PodDisruptionBudget 避免滚动更新时全部不可用
- [ ] 长耗时接口配置合理的 proxy-read-timeout
- [ ] Ingress Controller 启用访问日志用于事后分析

---

## 四、存储故障排查

### 4.1 PVC Pending

**症状**

- PVC 长时间处于 `Pending` 状态
- Pod 因 PVC 未绑定而 Pending

**诊断命令**

```bash
# 1. 查看 PVC 状态和事件
kubectl describe pvc <pvc-name> -n <namespace>

# 2. 检查可用 PV
kubectl get pv --sort-by='.spec.capacity.storage'

# 3. 检查 StorageClass
kubectl get storageclass
kubectl describe storageclass <name>

# 4. 检查 CSI 驱动状态
kubectl get csidrivers
kubectl get pods -n kube-system | grep csi

# 5. 查看 provisioner 日志
kubectl logs -n kube-system <csi-provisioner-pod> --tail=100
```

**根因分析**

1. **StorageClass 不存在**：PVC 指定的 StorageClass 未创建
2. **动态供给失败**：CSI 驱动异常或云厂商 API 配额耗尽
3. **容量不足**：存储池空间不够
4. **accessMode 不匹配**：PVC 要求 ReadWriteMany 但 StorageClass 不支持
5. **拓扑约束**：PV 在可用区 A，Pod 被调度到可用区 B
6. **静态 PV selector 不匹配**：PVC 的 selector 没有匹配的 PV

**解决方案**

```bash
# 检查 StorageClass 是否为 default
kubectl get sc -o jsonpath='{range .items[*]}{.metadata.name}:{.metadata.annotations.storageclass\.kubernetes\.io/is-default-class}{"\n"}{end}'

# 手动创建 PV（静态供给场景）
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: PersistentVolume
metadata:
  name: manual-pv
spec:
  capacity:
    storage: 10Gi
  accessModes:
    - ReadWriteOnce
  persistentVolumeReclaimPolicy: Retain
  storageClassName: standard
  hostPath:
    path: /data/manual-pv
EOF

# 跨可用区问题：配置 volumeBindingMode
# storageClassName 的 volumeBindingMode 应为 WaitForFirstConsumer
```

**预防措施**

- [ ] 使用 `WaitForFirstConsumer` 绑定模式避免拓扑冲突
- [ ] 存储容量告警阈值设为 75%
- [ ] CSI 驱动部署 HA，至少 2 副本
- [ ] 定期清理 Released 状态的 PV

---

### 4.2 挂载失败

**症状**

- Pod 卡在 `ContainerCreating`
- Events 中出现 `FailedMount` / `Unable to attach or mount volumes`
- 超时信息 `timeout expired waiting for volumes to attach/mount`

**诊断命令**

```bash
# 1. 查看挂载事件
kubectl describe pod <pod-name> -n <namespace> | grep -A 10 "FailedMount\|Unable to"

# 2. 检查 VolumeAttachment 状态
kubectl get volumeattachment

# 3. 检查节点上的挂载状态
kubectl get csinodes <node-name> -o yaml

# 4. 在节点上检查设备
lsblk
mount | grep <pv-name>

# 5. CSI 驱动详细日志
kubectl logs -n kube-system <csi-node-pod> -c <driver-container> --tail=100
```

**根因分析**

1. **多节点挂载冲突**：ReadWriteOnce 的 PV 被另一个节点占用
2. **设备繁忙**：上一个 Pod 未正常卸载导致设备 busy
3. **CSI 驱动版本不兼容**：节点上 CSI 驱动未更新
4. **云盘跨可用区**：EBS/Disk 不支持跨 AZ 挂载
5. **subPath 错误**：subPath 指定的路径不存在

**解决方案**

```bash
# 强制分离卡住的 VolumeAttachment
kubectl delete volumeattachment <name> --force --grace-period=0

# 迁移 Pod 到 PV 所在的可用区
# 使用 nodeAffinity 固定
# 或使用支持跨 AZ 的存储（如 EFS / NFS）

# subPath 不存在时使用 subPathExpr + initContainer
# initContainers:
# - name: init-data-dir
#   image: busybox
#   command: ['mkdir', '-p', '/data/subdir']
#   volumeMounts:
#   - name: data
#     mountPath: /data
```

**预防措施**

- [ ] ReadWriteOnce 卷仅用于 Deployment replicas=1 或 StatefulSet
- [ ] 需要多 Pod 共享存储时使用 ReadWriteMany（NFS / CephFS / EFS）
- [ ] Pod 配置 terminationGracePeriodSeconds 确保优雅卸载
- [ ] 云盘快照定期备份

---

### 4.3 权限问题

**症状**

- 容器日志出现 `Permission denied`
- 文件写入失败但挂载本身成功
- 应用以非 root 用户运行时无法访问存储

**诊断命令**

```bash
# 1. 查看容器内文件权限
kubectl exec <pod> -- ls -la /data/

# 2. 查看 securityContext 配置
kubectl get pod <pod> -o jsonpath='{.spec.securityContext}'
kubectl get pod <pod> -o jsonpath='{.spec.containers[0].securityContext}'

# 3. 查看容器进程运行的 UID/GID
kubectl exec <pod> -- id

# 4. 查看挂载点的文件系统信息
kubectl exec <pod> -- stat /data/
kubectl exec <pod> -- df -h /data/
```

**根因分析**

1. **UID 不匹配**：容器进程以 UID 1000 运行，但文件属主是 root
2. **fsGroup 未设置**：PV 上的文件不归 Pod 的 group 所有
3. **readOnly 挂载**：volumeMount 设置了 readOnly: true
4. **NFS root_squash**：NFS 服务端将 root 映射为 nobody
5. **SELinux / AppArmor**：安全模块阻止了文件访问

**解决方案**

```yaml
# 使用 securityContext 统一 UID/GID
spec:
  securityContext:
    runAsUser: 1000
    runAsGroup: 1000
    fsGroup: 1000              # 挂载卷的 group ownership
    fsGroupChangePolicy: "OnRootMismatch"  # 仅在不匹配时修改，加速启动
  containers:
  - name: app
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true    # 强制只读根文件系统
    volumeMounts:
    - name: data
      mountPath: /data
    - name: tmp
      mountPath: /tmp                  # 可写临时目录

# 使用 initContainer 修复权限
initContainers:
- name: fix-permissions
  image: busybox:1.36
  command: ['sh', '-c', 'chown -R 1000:1000 /data']
  volumeMounts:
  - name: data
    mountPath: /data
  securityContext:
    runAsUser: 0                       # initContainer 以 root 运行修复权限
```

**预防措施**

- [ ] 所有 Pod 配置 securityContext.fsGroup
- [ ] Dockerfile 中使用 `USER` 指令明确非 root 用户
- [ ] NFS 服务端配置 `no_root_squash` 或使用确定性 UID 映射
- [ ] 对敏感卷使用 readOnly 挂载

---

## 五、节点故障排查

### 5.1 NotReady

**症状**

- `kubectl get nodes` 显示某节点 `NotReady`
- 该节点上的 Pod 状态变为 `Unknown` 或被驱逐

**诊断命令**

```bash
# 1. 查看节点状态条件
kubectl describe node <node-name> | grep -A 20 "Conditions:"

# 2. 查看节点事件
kubectl get events --field-selector involvedObject.name=<node-name> --sort-by='.lastTimestamp'

# 3. 检查 kubelet 状态（SSH 到节点）
systemctl status kubelet
journalctl -u kubelet --since "10 minutes ago" --no-pager | tail -50

# 4. 检查容器运行时
systemctl status containerd
crictl ps

# 5. 检查网络连通性（节点到 API Server）
curl -k https://<api-server>:6443/healthz

# 6. 检查证书过期
openssl x509 -in /var/lib/kubelet/pki/kubelet-client-current.pem -noout -dates
```

**根因分析**

1. **kubelet 进程异常**：kubelet crash 或 OOM
2. **容器运行时故障**：containerd / docker 无响应
3. **网络断开**：节点与 API Server 失联
4. **证书过期**：kubelet 客户端证书或 CA 过期
5. **磁盘满**：根分区 / Docker 分区满导致 kubelet 无法工作
6. **内核 panic**：节点内核崩溃

**解决方案**

```bash
# 重启 kubelet
systemctl restart kubelet

# 重启容器运行时
systemctl restart containerd

# 清理磁盘空间
crictl rmi --prune                # 清理未使用镜像
journalctl --vacuum-size=500M     # 清理 journal 日志

# 证书过期 → 更新
kubeadm certs renew all
systemctl restart kubelet

# 节点彻底不可恢复 → 安全排水后移除
kubectl drain <node-name> --ignore-daemonsets --delete-emptydir-data --timeout=120s
kubectl delete node <node-name>
```

**预防措施**

- [ ] 配置 kubelet 监控和自动重启（systemd watchdog）
- [ ] 证书到期前 30 天告警
- [ ] 节点磁盘使用率告警阈值 70%
- [ ] 关键工作负载配置 PDB + 多副本跨节点分布

---

### 5.2 磁盘压力（DiskPressure）

**症状**

- 节点 Conditions 中 `DiskPressure=True`
- 新 Pod 无法调度到该节点
- 已有 Pod 可能被驱逐

**诊断命令**

```bash
# 1. 确认磁盘压力状态
kubectl describe node <node-name> | grep DiskPressure

# 2. 检查节点磁盘使用（SSH 到节点）
df -h
du -sh /var/lib/containerd/*
du -sh /var/log/*
du -sh /var/lib/kubelet/*

# 3. 查看容器镜像占用
crictl images | sort -k 4 -h

# 4. 查看容器日志大小
find /var/log/containers -name "*.log" -exec ls -lh {} \; | sort -k5 -h | tail -20
```

**根因分析**

1. **容器日志未限制**：应用大量写 stdout 导致日志文件膨胀
2. **镜像积累**：未清理的旧镜像占满磁盘
3. **emptyDir 滥用**：Pod 的 emptyDir 写入大量临时数据
4. **系统日志堆积**：journal / syslog 未配置轮转

**解决方案**

```bash
# 立即清理
crictl rmi --prune
journalctl --vacuum-size=200M
find /var/log -name "*.gz" -mtime +7 -delete

# 配置 containerd 日志大小限制
# /etc/containerd/config.toml
# [plugins."io.containerd.grpc.v1.cri".containerd]
#   [plugins."io.containerd.grpc.v1.cri".containerd.default_runtime]
#     [plugins."io.containerd.grpc.v1.cri".containerd.default_runtime.options]
#       max-container-log-line-size = 16384

# 配置 kubelet 日志轮转
# containerLogMaxSize: "50Mi"
# containerLogMaxFiles: 3
```

**预防措施**

- [ ] kubelet 配置 containerLogMaxSize 和 containerLogMaxFiles
- [ ] 配置 imagefs 和 nodefs 告警
- [ ] 定期运行镜像垃圾回收 CronJob
- [ ] 使用独立磁盘分区存放容器数据

---

### 5.3 内存压力（MemoryPressure）

**症状**

- 节点 Conditions 中 `MemoryPressure=True`
- 低优先级 Pod 被驱逐
- 系统 OOM Killer 开始杀进程

**诊断命令**

```bash
# 1. 查看节点内存状态
kubectl describe node <node-name> | grep -A 3 "MemoryPressure"
kubectl top node <node-name>

# 2. 节点内存详情（SSH 到节点）
free -h
cat /proc/meminfo | head -20

# 3. 查看内存占用前 10 的进程
ps aux --sort=-%mem | head -10

# 4. 查看 Pod 内存使用
kubectl top pods --sort-by=memory -A | head -20

# 5. 检查 OOM 事件
dmesg | grep -i "oom\|out of memory" | tail -20
```

**根因分析**

1. **资源超卖**：requests 总和远大于节点实际内存
2. **内存泄漏**：某个 Pod 内存持续增长
3. **系统预留不足**：未配置 system-reserved / kube-reserved
4. **缓存未释放**：内核缓存过大，可回收但未触发回收

**解决方案**

```bash
# 手动释放缓存（临时措施）
echo 3 > /proc/sys/vm/drop_caches

# 配置 kubelet 系统预留
# /var/lib/kubelet/config.yaml
# systemReserved:
#   cpu: "500m"
#   memory: "1Gi"
# kubeReserved:
#   cpu: "500m"
#   memory: "512Mi"
# evictionHard:
#   memory.available: "200Mi"
# evictionSoft:
#   memory.available: "500Mi"
# evictionSoftGracePeriod:
#   memory.available: "1m"
```

**预防措施**

- [ ] 节点内存 requests 总和不超过 allocatable 的 85%
- [ ] 配置 system-reserved 和 kube-reserved
- [ ] 内存使用率 > 80% 触发告警
- [ ] 使用 VPA 或定期压测校准 requests

---

### 5.4 PID 压力（PIDPressure）

**症状**

- 节点 Conditions 中 `PIDPressure=True`
- 无法创建新进程 / 容器
- 出现 `cannot allocate memory`（实际是 PID 耗尽）

**诊断命令**

```bash
# 1. 查看节点 PID 状态
kubectl describe node <node-name> | grep PIDPressure

# 2. 当前 PID 使用量
cat /proc/sys/kernel/pid_max
ls /proc | grep -E '^[0-9]+$' | wc -l

# 3. 各容器 PID 使用
for cid in $(crictl ps -q); do
  name=$(crictl inspect $cid | jq -r '.status.labels."io.kubernetes.pod.name"')
  pids=$(crictl inspect $cid | jq '.info.pid')
  echo "$name: PID=$pids"
done

# 4. 查看进程树，找出 fork 炸弹
ps auxf | head -100
```

**根因分析**

1. **Fork 炸弹**：应用 bug 导致无限创建子进程
2. **pidsLimit 未设置**：单个 Pod 耗尽节点全部 PID
3. **pid_max 过低**：系统默认 32768 在高密度节点不够
4. **僵尸进程**：PID 1 进程未正确回收子进程

**解决方案**

```bash
# 提高 pid_max
sysctl -w kernel.pid_max=65536

# 配置 kubelet PID 限制
# /var/lib/kubelet/config.yaml
# podPidsLimit: 1024           # 每 Pod 最多 1024 个 PID
# evictionHard:
#   pid.available: "10%"

# 容器内使用 tini 作为 init 进程回收僵尸进程
# Dockerfile
# RUN apk add --no-cache tini
# ENTRYPOINT ["tini", "--"]
# CMD ["your-app"]
```

**预防措施**

- [ ] 所有容器使用 tini / dumb-init 作为 PID 1
- [ ] kubelet 配置 podPidsLimit
- [ ] 监控节点 PID 使用率
- [ ] 应用代码审查确保无 fork 炸弹风险

---

## 六、资源问题排查

### 6.1 CPU Throttling

**症状**

- 应用响应延迟增高但未 OOM
- Prometheus 中 `container_cpu_cfs_throttled_seconds_total` 持续增长
- 应用日志无异常但性能下降

**诊断命令**

```bash
# 1. 查看 CPU 使用与限制
kubectl top pod <pod-name> --containers

# 2. 查看 cgroup CPU 统计（节点上）
cat /sys/fs/cgroup/cpu/kubepods/pod<uid>/<cid>/cpu.stat
# nr_throttled: 节流次数
# throttled_time: 被节流的总时间（纳秒）

# 3. Prometheus 查询
# rate(container_cpu_cfs_throttled_periods_total[5m])
#   / rate(container_cpu_cfs_periods_total[5m]) > 0.25

# 4. 查看 CPU requests/limits 配置
kubectl get pod <pod-name> -o jsonpath='{.spec.containers[*].resources}'
```

**根因分析**

1. **CPU limits 过低**：峰值 CPU 需求超过 limits
2. **突发性计算**：GC、JIT 编译等突发 CPU 需求被限流
3. **limits/requests 比值过低**：burstable 空间不足
4. **多线程应用**：线程数 > CPU limits 导致分时竞争

**解决方案**

```yaml
# 方案 1：放大 limits（推荐 limits = requests * 2 ~ 5）
resources:
  requests:
    cpu: "500m"
  limits:
    cpu: "2000m"           # 允许 4 倍突发

# 方案 2：移除 CPU limits（争议方案，但 Google/Zalando 推荐）
# 只设 requests，不设 limits，依赖 requests 做公平调度
resources:
  requests:
    cpu: "500m"
  # 不设 limits

# 方案 3：Java 应用设置容器感知
env:
- name: JAVA_OPTS
  value: "-XX:+UseContainerSupport -XX:ActiveProcessorCount=2"
```

**预防措施**

- [ ] 监控 CPU throttling 比率，超过 25% 触发告警
- [ ] 压测确定 CPU 基线后设置 requests，limits 设为 2-5 倍
- [ ] 考虑对延迟敏感的服务不设 CPU limits
- [ ] 避免 CPU limits = requests（Guaranteed QoS 无突发空间）

---

### 6.2 内存泄漏

**症状**

- Pod 内存使用量持续线性增长
- 定期触发 OOMKilled
- 重启后暂时正常，一段时间后再次增长

**诊断命令**

```bash
# 1. 观察内存增长趋势
kubectl top pod <pod-name> --containers
# 间隔 1 分钟执行多次，观察趋势

# 2. Prometheus 查询（过去 24 小时趋势）
# container_memory_working_set_bytes{pod="<pod-name>"}

# 3. 进入容器进行 profiling
# Go 应用
kubectl port-forward <pod> 6060:6060
go tool pprof http://localhost:6060/debug/pprof/heap

# Java 应用
kubectl exec <pod> -- jcmd 1 GC.heap_dump /tmp/dump.hprof
kubectl cp <pod>:/tmp/dump.hprof ./dump.hprof

# Python 应用
kubectl exec <pod> -- python -c "import tracemalloc; tracemalloc.start()"

# Node.js 应用
kubectl exec <pod> -- kill -USR2 1   # 如果启用了 heapdump
```

**根因分析**

1. **缓存无上限**：内存缓存未设置 maxSize 或 TTL
2. **连接池泄漏**：数据库/HTTP 连接未正确释放
3. **事件监听器泄漏**：注册了 listener 但未取消
4. **全局变量累积**：全局 list/map 持续追加
5. **goroutine 泄漏**（Go 应用）：goroutine 未退出

**解决方案**

```bash
# 临时措施：配置定时重启
# 使用 CronJob 或 kubectl rollout restart

# Go goroutine 泄漏诊断
kubectl exec <pod> -- curl localhost:6060/debug/pprof/goroutine?debug=2

# Java 堆分析
# 下载 dump 后用 Eclipse MAT / VisualVM 分析

# 设置内存告警 + 自动重启
# 通过 VPA 或自定义 controller 实现
```

**预防措施**

- [ ] 所有内存缓存必须设置 maxSize 和 TTL
- [ ] 定期进行 memory profiling，纳入 CI 流程
- [ ] 配置 Prometheus 告警：内存 24h 增长率 > 20% 预警
- [ ] Go 应用启用 pprof endpoint，Java 应用启用 JMX

---

### 6.3 HPA 不生效

**症状**

- 负载增加但 Pod 数量不增长
- `kubectl get hpa` 显示 `TARGETS` 为 `<unknown>` 或指标不更新
- 手动扩容有效但自动扩容无反应

**诊断命令**

```bash
# 1. 查看 HPA 状态
kubectl get hpa <name> -n <namespace> -o wide
kubectl describe hpa <name> -n <namespace>

# 2. 检查 metrics-server 是否正常
kubectl top pods                     # 如果失败说明 metrics-server 异常
kubectl get pods -n kube-system | grep metrics-server
kubectl logs -n kube-system -l k8s-app=metrics-server --tail=50

# 3. 检查 Pod 是否设置了 resources.requests
kubectl get pod <pod> -o jsonpath='{.spec.containers[*].resources.requests}'

# 4. 检查自定义指标（如果用 custom metrics）
kubectl get --raw "/apis/custom.metrics.k8s.io/v1beta1" | jq .
kubectl get --raw "/apis/custom.metrics.k8s.io/v1beta1/namespaces/<ns>/pods/*/http_requests_per_second" | jq .

# 5. 查看 HPA 事件
kubectl get events --field-selector involvedObject.name=<hpa-name> --sort-by='.lastTimestamp'
```

**根因分析**

1. **metrics-server 未部署或异常**
2. **Pod 未设置 CPU/Memory requests**（HPA 百分比目标需要 requests 基准）
3. **HPA 指标采集延迟**（默认 15s 采集，30s 计算）
4. **缩放冷却期**：缩容默认 5 分钟冷却，扩容默认 3 分钟
5. **自定义指标 adapter 异常**
6. **minReplicas = maxReplicas**：配置错误导致无法扩缩

**解决方案**

```yaml
# 正确的 HPA 配置
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: app-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: app
  minReplicas: 2
  maxReplicas: 20
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60    # 扩容稳定窗口
      policies:
      - type: Percent
        value: 100                       # 每次最多翻倍
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300    # 缩容稳定窗口 5 分钟
      policies:
      - type: Percent
        value: 10                        # 每次最多缩 10%
        periodSeconds: 60
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70          # 目标 CPU 利用率
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

**预防措施**

- [ ] 所有被 HPA 管理的 Pod 必须设置 CPU requests
- [ ] metrics-server 部署 HA（2 副本 + PDB）
- [ ] HPA 目标利用率设为 60-80%，留出突发缓冲
- [ ] 配置 HPA 事件告警，及时发现指标采集异常

---

## 七、部署故障排查

### 7.1 滚动更新卡住

**症状**

- `kubectl rollout status` 一直等待
- 新版本 Pod 无法就绪，旧版本 Pod 被保留
- Deployment 的 `READY` 列显示不完整（如 2/3）

**诊断命令**

```bash
# 1. 查看 rollout 状态
kubectl rollout status deployment/<name> -n <namespace> --timeout=10s

# 2. 查看 ReplicaSet 状态
kubectl get rs -n <namespace> -l app=<name>

# 3. 查看新旧 Pod 状态
kubectl get pods -n <namespace> -l app=<name> -o wide

# 4. 查看 Deployment 事件
kubectl describe deployment <name> -n <namespace>

# 5. 查看新 Pod 的问题
kubectl describe pod <new-pod> -n <namespace>
kubectl logs <new-pod> -n <namespace> --tail=100
```

**根因分析**

1. **新版本 Pod CrashLoopBackOff**：代码 bug 或配置错误
2. **readinessProbe 失败**：健康检查不通过
3. **资源不足**：无法调度新 Pod
4. **镜像拉取失败**：新版本镜像不存在
5. **maxUnavailable=0 + maxSurge=0**：错误配置导致无法滚动
6. **PDB 阻止**：PodDisruptionBudget 不允许终止旧 Pod

**解决方案**

```bash
# 查看并回滚到上一版本
kubectl rollout undo deployment/<name> -n <namespace>

# 回滚到指定版本
kubectl rollout history deployment/<name> -n <namespace>
kubectl rollout undo deployment/<name> --to-revision=<N> -n <namespace>

# 暂停滚动更新以排查
kubectl rollout pause deployment/<name> -n <namespace>
# 排查完成后继续
kubectl rollout resume deployment/<name> -n <namespace>
```

```yaml
# 合理的滚动更新策略
spec:
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 25%            # 允许超出期望数 25%
      maxUnavailable: 25%      # 允许不可用 25%
  minReadySeconds: 10          # Pod 就绪后等待 10 秒再继续
  progressDeadlineSeconds: 600 # 10 分钟超时自动标记失败
```

**预防措施**

- [ ] 配置 progressDeadlineSeconds（默认 600s）
- [ ] 新版本在 staging 验证通过后再上线
- [ ] 使用 Argo Rollouts 实现金丝雀发布
- [ ] PDB 配置确保 minAvailable 小于 replicas

---

### 7.2 回滚操作

**完整回滚流程**

```bash
# Step 1: 确认当前版本和历史
kubectl rollout history deployment/<name> -n <namespace>

# Step 2: 查看特定版本详情
kubectl rollout history deployment/<name> --revision=<N> -n <namespace>

# Step 3: 执行回滚
kubectl rollout undo deployment/<name> -n <namespace>               # 回滚到上一版本
kubectl rollout undo deployment/<name> --to-revision=<N> -n <namespace>  # 回滚到指定版本

# Step 4: 验证回滚状态
kubectl rollout status deployment/<name> -n <namespace>
kubectl get pods -n <namespace> -l app=<name>

# Step 5: 验证应用可用性
kubectl exec <test-pod> -- curl -s http://<service>:<port>/healthz
```

**StatefulSet 回滚注意事项**

```bash
# StatefulSet 不支持 rollout undo，需要手动修改
kubectl get statefulset <name> -o jsonpath='{.spec.updateStrategy}'

# 分区滚动（partition 控制）
kubectl patch statefulset <name> -p '{"spec":{"updateStrategy":{"rollingUpdate":{"partition":3}}}}'
# 从最后一个 Pod 开始更新到 partition 指定的序号

# 恢复完整滚动
kubectl patch statefulset <name> -p '{"spec":{"updateStrategy":{"rollingUpdate":{"partition":0}}}}'
```

**预防措施**

- [ ] 保留足够的 revisionHistoryLimit（默认 10）
- [ ] 每次部署前记录当前版本号
- [ ] 回滚后立即验证核心功能
- [ ] 事后进行 RCA 并修复根因

---

### 7.3 ConfigMap / Secret 更新不生效

**症状**

- 修改了 ConfigMap 或 Secret 但 Pod 行为未变
- 环境变量仍为旧值
- 配置文件内容未更新

**诊断命令**

```bash
# 1. 确认 ConfigMap 已更新
kubectl get cm <name> -n <namespace> -o yaml

# 2. 查看 Pod 内的实际值
kubectl exec <pod> -- env | grep <key>               # 环境变量方式
kubectl exec <pod> -- cat /config/<file>              # 挂载文件方式

# 3. 查看 Pod 创建时间（环境变量只在创建时注入）
kubectl get pod <pod> -o jsonpath='{.metadata.creationTimestamp}'

# 4. 检查挂载方式
kubectl get pod <pod> -o jsonpath='{.spec.volumes}' | jq .
```

**根因分析**

1. **环境变量不会热更新**：env / envFrom 只在 Pod 创建时注入
2. **Volume 挂载有延迟**：kubelet 同步周期默认 60s，可能需 1-2 分钟
3. **subPath 挂载不会更新**：这是 Kubernetes 已知限制
4. **应用未 watch 文件变化**：文件更新了但应用没有重新加载
5. **immutable ConfigMap**：设置了 `immutable: true`

**解决方案**

```bash
# 方案 1：强制重启 Pod 使环境变量生效
kubectl rollout restart deployment/<name> -n <namespace>

# 方案 2：使用 hash 注解实现自动滚动更新
# 在 Deployment template 中添加
# metadata:
#   annotations:
#     checksum/config: {{ sha256sum of configmap }}
# CI/CD 自动计算 hash，ConfigMap 变更触发滚动更新

# 方案 3：使用 Reloader 自动重启
# 安装 stakater/Reloader
# 然后在 Deployment 上添加注解：
# metadata:
#   annotations:
#     reloader.stakater.com/auto: "true"
```

```yaml
# 最佳实践：immutable ConfigMap + 版本化名称
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config-v2          # 版本化命名
immutable: true                 # 防止意外修改
data:
  app.yaml: |
    key: new-value

# Deployment 引用新版本
# volumes:
# - name: config
#   configMap:
#     name: app-config-v2      # 引用新 ConfigMap 触发滚动更新
```

**预防措施**

- [ ] 环境变量型配置必须配合 rollout restart 使用
- [ ] 生产环境使用 immutable ConfigMap + 版本化命名
- [ ] 安装 Reloader 或在 CI 中自动注入 checksum 注解
- [ ] 避免使用 subPath 挂载需要热更新的配置

---

## 八、日志与监控

### 8.1 kubectl logs 高级用法

```bash
# 基础：查看 Pod 日志
kubectl logs <pod> -n <namespace>

# 查看上一个容器实例的日志（crash 场景必用）
kubectl logs <pod> --previous

# 多容器 Pod 指定容器
kubectl logs <pod> -c <container>

# 实时跟踪
kubectl logs <pod> -f --tail=100

# 按时间过滤
kubectl logs <pod> --since=1h
kubectl logs <pod> --since-time='2026-03-28T10:00:00Z'

# 所有副本日志聚合
kubectl logs -l app=<name> --all-containers --max-log-requests=10

# 输出到文件
kubectl logs <pod> --all-containers > /tmp/pod-logs.txt
```

### 8.2 kubectl top 资源监控

```bash
# 节点资源使用
kubectl top nodes
kubectl top nodes --sort-by=cpu
kubectl top nodes --sort-by=memory

# Pod 资源使用
kubectl top pods -n <namespace> --sort-by=memory
kubectl top pods -n <namespace> --containers     # 按容器拆分
kubectl top pods -A --sort-by=cpu | head -20     # 全集群 CPU Top 20
```

### 8.3 kubectl describe 关键信息

```bash
# Pod：重点看 Events、State、Conditions
kubectl describe pod <pod> -n <namespace>

# Node：重点看 Conditions、Allocated resources、Events
kubectl describe node <node>

# Service：重点看 Endpoints
kubectl describe svc <service> -n <namespace>

# PVC：重点看 Events（供给状态）
kubectl describe pvc <pvc> -n <namespace>
```

### 8.4 kubectl events 事件排查

```bash
# 命名空间内所有事件（按时间排序）
kubectl get events -n <namespace> --sort-by='.lastTimestamp'

# 全集群 Warning 事件
kubectl get events -A --field-selector type=Warning --sort-by='.lastTimestamp'

# 特定资源的事件
kubectl get events --field-selector involvedObject.name=<pod-name>

# 监听实时事件
kubectl get events -w -n <namespace>

# 过滤特定原因
kubectl get events --field-selector reason=FailedScheduling -A
kubectl get events --field-selector reason=OOMKilling -A
kubectl get events --field-selector reason=BackOff -A
```

### 8.5 Prometheus 告警规则参考

```yaml
# 关键告警规则（PrometheusRule CRD）
groups:
- name: kubernetes-pod-alerts
  rules:
  - alert: PodCrashLooping
    expr: rate(kube_pod_container_status_restarts_total[15m]) * 60 * 15 > 0
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "Pod {{ $labels.pod }} is crash looping"

  - alert: PodOOMKilled
    expr: kube_pod_container_status_last_terminated_reason{reason="OOMKilled"} > 0
    for: 0m
    labels:
      severity: warning
    annotations:
      summary: "Pod {{ $labels.pod }} was OOM killed"

  - alert: HighCPUThrottling
    expr: >
      rate(container_cpu_cfs_throttled_seconds_total[5m])
      / rate(container_cpu_cfs_periods_total[5m]) > 0.25
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "Container {{ $labels.container }} high CPU throttling"

  - alert: PVCAlmostFull
    expr: kubelet_volume_stats_used_bytes / kubelet_volume_stats_capacity_bytes > 0.85
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "PVC {{ $labels.persistentvolumeclaim }} is 85% full"

  - alert: NodeMemoryPressure
    expr: kube_node_status_condition{condition="MemoryPressure",status="true"} == 1
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "Node {{ $labels.node }} under memory pressure"

  - alert: NodeDiskPressure
    expr: kube_node_status_condition{condition="DiskPressure",status="true"} == 1
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "Node {{ $labels.node }} under disk pressure"

  - alert: HPAMaxedOut
    expr: kube_horizontalpodautoscaler_status_current_replicas == kube_horizontalpodautoscaler_spec_max_replicas
    for: 15m
    labels:
      severity: warning
    annotations:
      summary: "HPA {{ $labels.horizontalpodautoscaler }} at max replicas"
```

---

## 九、网络调试工具箱

### 9.1 kubectl exec 调试

```bash
# 进入 Pod 执行调试命令
kubectl exec -it <pod> -- /bin/sh

# 如果容器没有 shell（distroless 镜像）
# 使用 ephemeral container（K8s 1.25+）
kubectl debug -it <pod> --image=nicolaka/netshoot --target=<container>

# 在节点上调试（共享节点网络命名空间）
kubectl debug node/<node-name> -it --image=nicolaka/netshoot
```

### 9.2 nsenter 节点级网络调试

```bash
# 获取容器的 PID
crictl inspect <container-id> | jq '.info.pid'

# 进入容器的网络命名空间
nsenter -t <pid> -n

# 在容器网络命名空间内执行命令
nsenter -t <pid> -n ip addr
nsenter -t <pid> -n ss -tlnp
nsenter -t <pid> -n iptables -L -n
nsenter -t <pid> -n ip route
```

### 9.3 tcpdump 抓包

```bash
# 在 Pod 内抓包（如果有 tcpdump）
kubectl exec <pod> -- tcpdump -i eth0 -nn -c 100 port 8080

# 使用 ephemeral container 抓包
kubectl debug -it <pod> --image=nicolaka/netshoot --target=<container> -- \
  tcpdump -i eth0 -nn -w /tmp/capture.pcap -c 1000

# 抓包并下载分析
kubectl cp <pod>:/tmp/capture.pcap ./capture.pcap -c debugger

# 在节点上抓特定 Pod 的流量
# 先获取 Pod IP
POD_IP=$(kubectl get pod <pod> -o jsonpath='{.status.podIP}')
# 在节点上
tcpdump -i any host $POD_IP -nn -c 200
```

### 9.4 curl / wget 连通性测试

```bash
# 使用 debug Pod 测试
kubectl run debug --rm -it --image=nicolaka/netshoot -- bash

# 测试 Service 连通性
curl -v http://<service>.<namespace>.svc.cluster.local:<port>/healthz
curl -v --connect-timeout 3 --max-time 10 http://<service>:<port>/

# 测试 HTTPS / TLS
curl -vk https://<ingress-host>/
openssl s_client -connect <ingress-host>:443 -servername <host>

# 测试 TCP 端口连通性
nc -zv <host> <port>
nmap -p <port> <host>

# 测试 DNS 解析
dig <service>.<namespace>.svc.cluster.local @<coredns-ip>
dig +short +trace <external-domain>

# 路由追踪
traceroute -n <target-ip>
mtr -n <target-ip>
```

### 9.5 常用调试镜像

| 镜像 | 用途 | 工具集 |
|------|------|--------|
| `nicolaka/netshoot` | 网络调试瑞士军刀 | tcpdump, dig, curl, iperf, nmap, strace |
| `busybox:1.36` | 轻量级调试 | sh, nc, wget, ping, nslookup |
| `curlimages/curl` | HTTP 测试 | curl |
| `alpine:3.19` | 通用调试 | sh, apk 可装任何工具 |
| `ubuntu:22.04` | 完整调试环境 | apt 可装任何工具 |

---

## 十、故障排查决策树

```
Pod 异常
├── Pending
│   ├── Events: FailedScheduling → 检查资源/亲和性/taint
│   ├── Events: no PV found → 检查 PVC/StorageClass
│   └── 无 Events → 检查 API Server / Scheduler 日志
├── CrashLoopBackOff
│   ├── Exit 137 → OOMKilled → 调大 limits / 查内存泄漏
│   ├── Exit 1 → 应用错误 → 看 --previous 日志
│   ├── Exit 127 → 命令不存在 → 检查 entrypoint/CMD
│   └── 无日志 → 检查 image / securityContext
├── ImagePullBackOff
│   ├── unauthorized → 检查 imagePullSecret
│   ├── not found → 检查镜像 tag
│   └── timeout → 检查网络 / 镜像仓库
├── ContainerCreating（超时）
│   ├── FailedMount → 检查 PV/VolumeAttachment
│   └── NetworkNotReady → 检查 CNI 插件
├── Running 但不工作
│   ├── readinessProbe 失败 → 检查探针配置
│   ├── CPU Throttling → 调整 limits
│   └── 网络不通 → 走网络排查流程
└── Evicted
    ├── ephemeral-storage → 清理磁盘 / 设 limits
    └── memory → 检查节点内存压力
```

---

## Agent Checklist

以下清单供 AI Agent 在 Kubernetes 故障排查场景中使用：

### 信息收集阶段

- [ ] 执行 `kubectl get pods -o wide` 确认 Pod 状态和所在节点
- [ ] 执行 `kubectl get events --sort-by='.lastTimestamp'` 获取最新事件
- [ ] 执行 `kubectl describe pod/node/svc` 获取详细状态
- [ ] 确认问题影响范围：单 Pod / 整个 Deployment / 全节点 / 全集群

### Pod 故障

- [ ] CrashLoopBackOff：查看 `--previous` 日志，确认退出码
- [ ] ImagePullBackOff：验证镜像地址、imagePullSecret、网络连通性
- [ ] OOMKilled：对比 `kubectl top` 用量与 limits 设置
- [ ] Pending：检查调度事件、节点资源、亲和性规则、PVC 状态
- [ ] Evicted：检查节点 Conditions，确认是磁盘还是内存压力

### 网络故障

- [ ] 用 `kubectl get endpoints` 验证 Service 后端不为空
- [ ] 用 debug Pod 测试 DNS 解析和 TCP 连通性
- [ ] 检查 NetworkPolicy 是否阻断流量
- [ ] Ingress 502/504：检查 Ingress Controller 日志和后端健康状态

### 存储故障

- [ ] PVC Pending：检查 StorageClass、CSI 驱动、容量配额
- [ ] 挂载失败：检查 VolumeAttachment 和 accessMode
- [ ] 权限问题：对比容器 UID 与文件 ownership，检查 fsGroup

### 节点故障

- [ ] NotReady：检查 kubelet 和 containerd 状态
- [ ] DiskPressure：检查磁盘使用，清理镜像和日志
- [ ] MemoryPressure：检查内存使用，确认 system-reserved 配置
- [ ] PIDPressure：检查 PID 使用量和 podPidsLimit

### 资源问题

- [ ] CPU Throttling：查看 cfs_throttled 指标，评估 limits 合理性
- [ ] 内存泄漏：观察内存增长趋势，进行 profiling
- [ ] HPA 不生效：验证 metrics-server、Pod requests、HPA 配置

### 部署故障

- [ ] 滚动更新卡住：检查新 Pod 状态，必要时 rollout undo
- [ ] ConfigMap/Secret 更新不生效：确认挂载方式，必要时 rollout restart

### 修复验证

- [ ] 修复后确认 Pod 状态恢复 Running/Ready
- [ ] 验证业务功能正常（健康检查通过、端到端请求成功）
- [ ] 检查是否有次生问题（如回滚后配置不一致）
- [ ] 记录根因和修复方案，更新运维文档