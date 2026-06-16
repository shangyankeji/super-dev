---
title: 服务网格作战手册
version: 1.0.0
last_updated: 2025-03-20
owner: platform-team
tags: [service-mesh, istio, traffic-management, observability]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：服务网格实施与运维作战手册
# 作用：指导 Istio/Linkerd 服务网格的部署、配置和运维
# 创建时间：2025-03-20
# 最后修改：2025-03-20

## 目标

建立服务网格标准化运维流程，确保：
- 流量管理可控可观测
- 服务间通信安全
- 故障快速定位和恢复
- 渐进式发布能力

## 适用场景

- 微服务流量管理
- 金丝雀/蓝绿发布
- 服务间 mTLS 加密
- 分布式追踪和可观测性
- 故障注入和混沌工程

## 执行清单

### 部署前准备

- [ ] 评估集群资源是否满足 Sidecar 开销（每 Pod 约增加 100MB 内存）
- [ ] 确定 Sidecar 资源配额
- [ ] 规划命名空间启用策略
- [ ] 准备监控和日志基础设施
- [ ] 制定回滚计划

### Istio 安装

- [ ] 下载并验证 Istio 版本
- [ ] 配置 IstioOperator 自定义资源
- [ ] 安装控制平面
- [ ] 验证组件运行状态
- [ ] 配置自动 Sidecar 注入

### 应用接入

- [ ] 标记命名空间启用注入
- [ ] 重启 Pod 注入 Sidecar
- [ ] 验证 Sidecar 运行状态
- [ ] 配置流量路由
- [ ] 启用 mTLS

## 核心配置

### 1. Istio 安装配置

```yaml
apiVersion: install.istio.io/v1alpha1
kind: IstioOperator
metadata:
  name: production-istio
  namespace: istio-system
spec:
  profile: default
  components:
    pilot:
      enabled: true
      k8s:
        resources:
          requests:
            cpu: 500m
            memory: 2Gi
          limits:
            cpu: 2000m
            memory: 4Gi
        hpaSpec:
          minReplicas: 2
          maxReplicas: 5
    ingressGateways:
    - name: istio-ingressgateway
      enabled: true
      k8s:
        resources:
          requests:
            cpu: 500m
            memory: 1Gi
          limits:
            cpu: 2000m
            memory: 2Gi
        hpaSpec:
          minReplicas: 2
          maxReplicas: 10
    cni:
      enabled: true
  meshConfig:
    accessLogFile: /dev/stdout
    enableAutoMtls: true
    defaultConfig:
      proxyMetadata:
        PROXY_CONFIG_TRACE: "verbose"
      tracing:
        zipkin:
          address: zipkin.istio-system:9411
    outboundTrafficPolicy:
      mode: REGISTRY_ONLY
  values:
    global:
      proxy:
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 512Mi
    sidecarInjectorWebhook:
      rewriteAppHTTPProbe: true
```

### 2. 命名空间启用注入

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: production
  labels:
    istio-injection: enabled
```

### 3. Gateway 配置

```yaml
apiVersion: networking.istio.io/v1beta1
kind: Gateway
metadata:
  name: production-gateway
  namespace: production
spec:
  selector:
    istio: ingressgateway
  servers:
  - port:
      number: 80
      name: http
      protocol: HTTP
    tls:
      httpsRedirect: true
    hosts:
    - "*.example.com"
  - port:
      number: 443
      name: https
      protocol: HTTPS
    tls:
      mode: SIMPLE
      credentialName: wildcard-tls
    hosts:
    - "*.example.com"
```

### 4. VirtualService 流量路由

```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: api-service
  namespace: production
spec:
  hosts:
  - api.example.com
  gateways:
  - production-gateway
  http:
  - match:
    - headers:
        x-canary:
          exact: "true"
    route:
    - destination:
        host: api-service
        subset: canary
      headers:
        response:
          add:
            x-version: "v2"
  - route:
    - destination:
        host: api-service
        subset: stable
      weight: 95
    - destination:
        host: api-service
        subset: canary
      weight: 5
    timeout: 30s
    retries:
      attempts: 3
      perTryTimeout: 10s
      retryOn: gateway-error,connect-failure,refused-stream
```

### 5. DestinationRule 配置

```yaml
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: api-service
  namespace: production
spec:
  host: api-service
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
        connectTimeout: 5s
      http:
        h2UpgradePolicy: UPGRADE
        http1MaxPendingRequests: 100
        http2MaxRequests: 1000
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 60s
      maxEjectionPercent: 50
      minHealthPercent: 25
    tls:
      mode: ISTIO_MUTUAL
  subsets:
  - name: stable
    labels:
      version: v1
  - name: canary
    labels:
      version: v2
```

### 6. 服务入口（外部服务）

```yaml
apiVersion: networking.istio.io/v1beta1
kind: ServiceEntry
metadata:
  name: external-api
  namespace: production
spec:
  hosts:
  - external-api.example.com
  ports:
  - number: 443
    name: https
    protocol: HTTPS
  location: MESH_EXTERNAL
  resolution: DNS
```

### 7. 授权策略

```yaml
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: api-service-policy
  namespace: production
spec:
  selector:
    matchLabels:
      app: api-service
  action: ALLOW
  rules:
  - from:
    - source:
        principals:
        - "cluster.local/ns/production/sa/frontend"
        - "cluster.local/ns/production/sa/worker"
    to:
    - operation:
        methods:
        - GET
        - POST
        paths:
        - /api/*
        - /health
```

### 8. PeerAuthentication（mTLS）

```yaml
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: production-mtls
  namespace: production
spec:
  mtls:
    mode: STRICT
```

### 9. 故障注入测试

```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: api-service-fault-injection
  namespace: production
spec:
  hosts:
  - api-service
  http:
  - match:
    - headers:
        x-test-fault:
          exact: "true"
    fault:
      delay:
        percentage:
          value: 100
        fixedDelay: 5s
      abort:
        percentage:
          value: 10
        httpStatus: 500
    route:
    - destination:
        host: api-service
  - route:
    - destination:
        host: api-service
```

### 10. 请求限流

```yaml
apiVersion: networking.istio.io/v1beta1
kind: EnvoyFilter
metadata:
  name: rate-limit
  namespace: production
spec:
  workloadSelector:
    labels:
      app: api-service
  configPatches:
  - applyTo: HTTP_FILTER
    match:
      context: SIDECAR_INBOUND
    patch:
      operation: INSERT_BEFORE
      value:
        name: envoy.filters.http.local_ratelimit
        typed_config:
          "@type": type.googleapis.com/udpa.type.v1.TypedStruct
          type_url: type.googleapis.com/envoy.extensions.filters.http.local_ratelimit.v3.LocalRateLimit
          value:
            stat_prefix: http_local_rate_limiter
            token_bucket:
              max_tokens: 100
              tokens_per_fill: 100
              fill_interval: 60s
            filter_enabled:
              runtime_key: local_rate_limit_enabled
              default_value:
                numerator: 100
                denominator: HUNDRED
            filter_enforced:
              runtime_key: local_rate_limit_enforced
              default_value:
                numerator: 100
                denominator: HUNDRED
            response_headers_to_add:
            - header:
                key: x-rate-limited
                value: "true"
```

## 最佳实践

### 1. 金丝雀发布流程

```yaml
# 第一阶段：1% 流量
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: api-service
spec:
  http:
  - route:
    - destination:
        host: api-service
        subset: stable
      weight: 99
    - destination:
        host: api-service
        subset: canary
      weight: 1

---
# 第二阶段：10% 流量（观察指标后）
# weight: 90 / 10

---
# 第三阶段：50% 流量（继续观察）
# weight: 50 / 50

---
# 第四阶段：100% 流量（全量切换）
# weight: 0 / 100
```

### 2. 熔断配置

```yaml
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: api-service
spec:
  host: api-service
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
      http:
        http1MaxPendingRequests: 100
        http2MaxRequests: 1000
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 60s
      maxEjectionPercent: 50
```

### 3. 可观测性配置

```yaml
# Prometheus 采集配置
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: istio-system
data:
  prometheus.yml: |
    scrape_configs:
    - job_name: 'istio-mesh'
      kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
          - production
      relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        action: keep
```

## 反模式

### 禁止操作

```yaml
# [FAIL] 禁止：所有流量立即切换
route:
- destination:
    host: api-service
    subset: canary
  weight: 100  # 立即 100%

# [FAIL] 禁止：无超时配置
http:
- route:
  - destination:
      host: api-service
  # 缺少 timeout

# [FAIL] 禁止：无熔断配置
# 缺少 outlierDetection

# [FAIL] 禁止：全量启用 mTLS 无灰度
spec:
  mtls:
    mode: STRICT  # 直接全量开启

# [FAIL] 禁止：Sidecar 资源无限制
# 缺少 resources 配置
```

## 实战案例

### 案例 1：渐进式金丝雀发布

```bash
# 1. 部署新版本
kubectl apply -f deployment-v2.yaml

# 2. 配置 1% 流量
kubectl apply -f virtualservice-1pct.yaml

# 3. 监控关键指标 10 分钟
# - 错误率 < 0.1%
# - P99 延迟 < 200ms
# - 业务指标正常

# 4. 逐步放量：1% -> 10% -> 50% -> 100%
for weight in 10 50 100; do
  kubectl patch virtualservice api-service \
    --type=json \
    -p="[{\"op\": \"replace\", \"path\": \"/spec/http/0/route/1/weight\", \"value\": $weight}]"

  # 等待并监控
  sleep 600
done

# 5. 下线旧版本
kubectl scale deployment api-service-v1 --replicas=0
```

### 案例 2：故障注入混沌测试

```yaml
# 注入延迟测试服务韧性
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: api-service-chaos
spec:
  hosts:
  - api-service
  http:
  - match:
    - headers:
        x-chaos-test:
          exact: "latency"
    fault:
      delay:
        percentage:
          value: 100
        fixedDelay: 500ms
    route:
    - destination:
        host: api-service
```

### 案例 3：紧急回滚

```bash
# 发现问题时立即回滚
kubectl patch virtualservice api-service \
  --type=json \
  -p='[{"op": "replace", "path": "/spec/http/0/route/0/weight", "value": 100}]'

kubectl patch virtualservice api-service \
  --type=json \
  -p='[{"op": "replace", "path": "/spec/http/0/route/1/weight", "value": 0}]'

# 缩容新版本
kubectl scale deployment api-service-v2 --replicas=0
```

## 检查清单

### 部署检查

- [ ] Istio 控制平面组件健康
- [ ] Sidecar 注入正常工作
- [ ] Gateway 配置正确
- [ ] VirtualService 路由规则生效
- [ ] mTLS 状态符合预期
- [ ] 授权策略配置正确
- [ ] 监控指标正常采集
- [ ] 追踪数据正常上报

### 流量管理检查

- [ ] 超时配置合理（默认 30s）
- [ ] 重试策略配置正确
- [ ] 熔断配置生效
- [ ] 限流配置合理
- [ ] 金丝雀发布流程定义

### 安全检查

- [ ] mTLS 已启用（STRICT 模式）
- [ ] 授权策略最小权限
- [ ] 外部服务访问受控
- [ ] 敏感流量已加密

### 可观测性检查

- [ ] Prometheus 指标正常
- [ ] Jaeger/Zipkin 追踪正常
- [ ] Kiali 控制台可访问
- [ ] 日志正常输出

## 参考资料

- [Istio 官方文档](https://istio.io/latest/docs/)
- [Istio 最佳实践](https://istio.io/latest/docs/ops/best-practices/)
- [Linkerd 文档](https://linkerd.io/2/overview/)
- [Envoy 配置参考](https://www.envoyproxy.io/docs/envoy/latest/)
- [服务网格模式](https://philcalcado.com/2017/08/03/pattern_service_mesh.html)