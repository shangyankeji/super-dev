---
id: cloud-native-patterns
title: 云原生模式
domain: architecture
category: 01-standards
difficulty: intermediate
tags: [agent, architecture, checklist, cloud, native, patterns, 实战代码示例, 常见陷阱]
quality_score: 70
last_updated: 2026-06-15
---
# 云原生模式

## 概述
云原生(Cloud Native)是一种充分利用云计算优势的软件设计方法论。本指南覆盖12-Factor App、Sidecar、Ambassador、Init Container、CRD等核心模式,帮助团队构建可扩展、弹性、可观测的云原生应用。

## 核心概念

### 1. 12-Factor App原则
| 因素 | 原则 | 说明 |
|------|------|------|
| 1. Codebase | 一份代码,多份部署 | Git仓库与环境无关 |
| 2. Dependencies | 显式声明依赖 | requirements.txt/package.json |
| 3. Config | 配置存储在环境变量 | 不硬编码配置 |
| 4. Backing Services | 后端服务作为附加资源 | DB/Cache/MQ通过URL连接 |
| 5. Build/Release/Run | 严格分离构建和运行 | CI/CD流水线 |
| 6. Processes | 无状态进程 | 状态存储在外部服务 |
| 7. Port Binding | 通过端口绑定提供服务 | 自包含HTTP服务器 |
| 8. Concurrency | 通过进程模型扩展 | 水平扩展而非垂直 |
| 9. Disposability | 快速启动,优雅停止 | 信号处理/连接排空 |
| 10. Dev/Prod Parity | 环境一致性 | Docker/容器化 |
| 11. Logs | 日志作为事件流 | 输出到stdout |
| 12. Admin Processes | 管理任务作为一次性进程 | Job/CronJob |

### 2. Kubernetes设计模式

| 模式 | 描述 | 用例 |
|------|------|------|
| Sidecar | 辅助容器扩展主容器功能 | 日志收集/代理/监控 |
| Ambassador | 代理容器处理外部通信 | 负载均衡/mTLS/限流 |
| Adapter | 适配容器标准化输出 | 日志格式转换/指标适配 |
| Init Container | 初始化容器在主容器前运行 | 数据迁移/配置下载/等待依赖 |
| Leader Election | 选举主节点处理单例任务 | 定时任务/全局调度 |

### 3. 云原生技术栈
- **容器运行时**: Docker/containerd/CRI-O
- **编排**: Kubernetes
- **服务网格**: Istio/Linkerd/Cilium
- **CI/CD**: ArgoCD/Flux/Tekton
- **可观测性**: Prometheus/Grafana/Jaeger/OpenTelemetry
- **密钥管理**: Vault/Sealed Secrets/External Secrets

## 实战代码示例

### 12-Factor配置管理

```python
# 基于环境变量的配置(Factor 3)
from pydantic_settings import BaseSettings
from functools import lru_cache

class Settings(BaseSettings):
    """应用配置(从环境变量读取)"""
    # 基础
    app_name: str = "my-service"
    environment: str = "development"
    debug: bool = False
    log_level: str = "INFO"

    # 服务端口(Factor 7)
    port: int = 8000
    host: str = "0.0.0.0"

    # 后端服务URL(Factor 4)
    database_url: str = "postgresql://localhost:5432/mydb"
    redis_url: str = "redis://localhost:6379/0"
    rabbitmq_url: str = "amqp://guest:guest@localhost:5672/"

    # 外部服务
    auth_service_url: str = "http://auth-service:8080"
    payment_gateway_url: str = "https://api.stripe.com"
    payment_gateway_key: str = ""

    # 安全
    secret_key: str = "change-me-in-production"
    cors_origins: list[str] = ["http://localhost:3000"]

    model_config = {
        "env_file": ".env",
        "env_file_encoding": "utf-8",
        "case_sensitive": False,
    }

@lru_cache()
def get_settings() -> Settings:
    return Settings()
```

```yaml
# Kubernetes ConfigMap和Secret
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
data:
  APP_NAME: "my-service"
  ENVIRONMENT: "production"
  LOG_LEVEL: "INFO"
  AUTH_SERVICE_URL: "http://auth-service:8080"

---
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
stringData:
  DATABASE_URL: "postgresql://user:pass@postgres:5432/mydb"
  SECRET_KEY: "super-secret-production-key"
  PAYMENT_GATEWAY_KEY: "sk_live_..."
```

### 优雅停止(Factor 9)

```python
# 优雅停止和健康检查
import signal
import asyncio
from contextlib import asynccontextmanager

class GracefulShutdown:
    """优雅停止管理器"""

    def __init__(self):
        self.is_shutting_down = False
        self._tasks: set[asyncio.Task] = set()

    def setup_signals(self):
        """注册信号处理"""
        loop = asyncio.get_event_loop()
        for sig in (signal.SIGTERM, signal.SIGINT):
            loop.add_signal_handler(sig, self._handle_signal)

    def _handle_signal(self):
        self.is_shutting_down = True
        logger.info("Shutdown signal received, draining connections...")

shutdown = GracefulShutdown()

@asynccontextmanager
async def lifespan(app):
    """应用生命周期管理"""
    # 启动
    logger.info("Starting application...")
    shutdown.setup_signals()

    # 初始化资源
    await db_pool.connect()
    await redis_pool.connect()
    await consumer.start()

    logger.info("Application started", port=settings.port)

    yield

    # 停止
    logger.info("Shutting down...")

    # 1. 停止接收新请求(K8s已通过readiness探针处理)
    # 2. 等待进行中的请求完成
    await asyncio.sleep(5)  # 给K8s时间更新endpoint

    # 3. 关闭消费者(停止消费新消息)
    await consumer.stop()

    # 4. 等待进行中的任务完成
    if shutdown._tasks:
        await asyncio.gather(*shutdown._tasks, return_exceptions=True)

    # 5. 关闭连接池
    await db_pool.disconnect()
    await redis_pool.disconnect()

    logger.info("Shutdown complete")

app = FastAPI(lifespan=lifespan)

# 健康检查端点
@app.get("/health/live")
async def liveness():
    """存活探针:进程是否运行"""
    return {"status": "alive"}

@app.get("/health/ready")
async def readiness():
    """就绪探针:是否可以接收流量"""
    if shutdown.is_shutting_down:
        return JSONResponse(status_code=503, content={"status": "shutting_down"})

    # 检查依赖服务
    checks = {}
    try:
        await db_pool.execute("SELECT 1")
        checks["database"] = "ok"
    except Exception:
        checks["database"] = "error"
        return JSONResponse(status_code=503, content=checks)

    try:
        await redis_pool.ping()
        checks["redis"] = "ok"
    except Exception:
        checks["redis"] = "error"
        return JSONResponse(status_code=503, content=checks)

    return {"status": "ready", "checks": checks}

@app.get("/health/startup")
async def startup():
    """启动探针:应用是否完成初始化"""
    if not db_pool.is_connected:
        return JSONResponse(status_code=503, content={"status": "initializing"})
    return {"status": "started"}
```

### Sidecar模式

```yaml
# Sidecar模式 — 日志收集 + 代理
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
spec:
  template:
    spec:
      containers:
        # 主应用容器
        - name: app
          image: myapp:v1.2.0
          ports:
            - containerPort: 8000
          volumeMounts:
            - name: log-volume
              mountPath: /var/log/app
            - name: tmp
              mountPath: /tmp

        # Sidecar 1: Fluent Bit日志收集
        - name: log-collector
          image: fluent/fluent-bit:2.2
          volumeMounts:
            - name: log-volume
              mountPath: /var/log/app
              readOnly: true
            - name: fluent-config
              mountPath: /fluent-bit/etc/
          resources:
            requests:
              cpu: 50m
              memory: 64Mi
            limits:
              cpu: 100m
              memory: 128Mi

        # Sidecar 2: Envoy代理(Service Mesh)
        - name: envoy-proxy
          image: envoyproxy/envoy:v1.28
          ports:
            - containerPort: 9901  # Envoy admin
          volumeMounts:
            - name: envoy-config
              mountPath: /etc/envoy
          resources:
            requests:
              cpu: 100m
              memory: 128Mi

      volumes:
        - name: log-volume
          emptyDir: {}
        - name: tmp
          emptyDir:
            sizeLimit: 100Mi
        - name: fluent-config
          configMap:
            name: fluent-bit-config
        - name: envoy-config
          configMap:
            name: envoy-config
```

### Init Container模式

```yaml
# Init Container — 初始化和依赖等待
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
spec:
  template:
    spec:
      initContainers:
        # Init 1: 等待数据库就绪
        - name: wait-for-db
          image: busybox:1.36
          command:
            - sh
            - -c
            - |
              until nc -z postgres 5432; do
                echo "Waiting for postgres..."
                sleep 2
              done
              echo "Postgres is ready"

        # Init 2: 运行数据库迁移
        - name: db-migrate
          image: myapp:v1.2.0
          command: ["alembic", "upgrade", "head"]
          envFrom:
            - secretRef:
                name: db-credentials
          resources:
            limits:
              cpu: 500m
              memory: 256Mi

        # Init 3: 下载配置文件
        - name: fetch-config
          image: curlimages/curl:8.5.0
          command:
            - sh
            - -c
            - |
              curl -s -o /config/app.yaml \
                http://config-server:8080/api/config/production
          volumeMounts:
            - name: config-volume
              mountPath: /config

      containers:
        - name: app
          image: myapp:v1.2.0
          volumeMounts:
            - name: config-volume
              mountPath: /app/config
              readOnly: true

      volumes:
        - name: config-volume
          emptyDir: {}
```

### CRD自定义资源

```yaml
# CRD定义
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: microservices.platform.example.com
spec:
  group: platform.example.com
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              required: ["image", "port"]
              properties:
                image:
                  type: string
                port:
                  type: integer
                replicas:
                  type: integer
                  default: 2
                autoscaling:
                  type: object
                  properties:
                    minReplicas:
                      type: integer
                      default: 2
                    maxReplicas:
                      type: integer
                      default: 10
                    targetCPU:
                      type: integer
                      default: 70
                database:
                  type: object
                  properties:
                    type:
                      type: string
                      enum: ["postgresql", "mysql", "none"]
                    size:
                      type: string
                      default: "1Gi"
            status:
              type: object
              properties:
                phase:
                  type: string
                readyReplicas:
                  type: integer
                endpoint:
                  type: string
      subresources:
        status: {}
  scope: Namespaced
  names:
    plural: microservices
    singular: microservice
    kind: Microservice
    shortNames:
      - ms

---
# 使用CRD
apiVersion: platform.example.com/v1
kind: Microservice
metadata:
  name: order-service
  namespace: production
spec:
  image: myregistry.com/order-service:v1.2.0
  port: 8080
  replicas: 3
  autoscaling:
    minReplicas: 3
    maxReplicas: 20
    targetCPU: 60
  database:
    type: postgresql
    size: 10Gi
```

### HPA自动扩缩容

```yaml
# 基于CPU和自定义指标的自动扩缩
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-server
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-server
  minReplicas: 3
  maxReplicas: 50
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
        - type: Percent
          value: 100         # 每次最多翻倍
          periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 10          # 每5分钟最多缩10%
          periodSeconds: 300
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
    - type: Pods
      pods:
        metric:
          name: http_requests_per_second
        target:
          type: AverageValue
          averageValue: "1000"
```

### GitOps部署(ArgoCD)

```yaml
# ArgoCD Application
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: order-service
  namespace: argocd
spec:
  project: production
  source:
    repoURL: https://github.com/org/k8s-manifests.git
    targetRevision: main
    path: services/order-service/overlays/production
  destination:
    server: https://kubernetes.default.svc
    namespace: production
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
      - ServerSideApply=true
    retry:
      limit: 3
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
```

## 最佳实践

### 1. 无状态设计(Factor 6)
- 应用进程不持有本地状态
- 会话状态存储在Redis/数据库
- 文件上传直接到对象存储(S3)
- 任何实例都可以处理任何请求

### 2. 配置外部化(Factor 3)
- 通过环境变量或ConfigMap注入配置
- 密钥使用Secret/Vault管理
- 不同环境用不同ConfigMap,同一镜像
- 配置变更不需要重新构建镜像

### 3. 健康检查三件套
- **Liveness**: 进程是否存活(死了就重启)
- **Readiness**: 是否可以接收流量(没准备好就不转发)
- **Startup**: 是否完成初始化(慢启动不被杀)

### 4. 资源管理
- 所有容器设置requests和limits
- CPU limits可选(可能导致throttling)
- 内存limits必须设置(防OOM影响节点)
- 使用LimitRange设置命名空间默认值

### 5. 可观测性
- 日志输出到stdout(Factor 11)
- 暴露Prometheus指标(/metrics)
- 集成分布式追踪(OpenTelemetry)
- 标准化健康检查端点

## 常见陷阱

### 陷阱1: 容器内存储状态
```python
# 错误: 文件存在容器内,Pod重启就丢失
with open("/app/data/upload.pdf", "wb") as f:
    f.write(data)

# 正确: 使用对象存储
await s3_client.upload_fileobj(data, "uploads", "upload.pdf")
```

### 陷阱2: 优雅停止时间不足
```yaml
# 错误: terminationGracePeriodSeconds太短
# Pod收到SIGTERM后只有30秒(默认)完成清理

# 正确: 根据业务需要设置
spec:
  terminationGracePeriodSeconds: 60
  containers:
    - name: app
      lifecycle:
        preStop:
          exec:
            command: ["sh", "-c", "sleep 5"]  # 等K8s更新endpoint
```

### 陷阱3: 健康检查配置错误
```yaml
# 错误: 存活探针检查数据库
# 数据库短暂不可用就重启Pod,加重数据库压力

livenessProbe:
  httpGet:
    path: /health  # 如果这里检查了DB连接...

# 正确: 存活探针只检查进程健康
livenessProbe:
  httpGet:
    path: /health/live  # 只检查进程是否存活

readinessProbe:
  httpGet:
    path: /health/ready  # 这里检查依赖服务
```

### 陷阱4: 不设置资源限制
```yaml
# 错误: 无限制,可能影响同节点其他Pod
# 正确: 合理设置
resources:
  requests:
    cpu: 100m      # 调度基准
    memory: 128Mi
  limits:
    cpu: 500m      # 上限(可选,有争议)
    memory: 512Mi  # 必须设置,防OOM
```

## Agent Checklist

### 12-Factor合规
- [ ] 配置通过环境变量注入
- [ ] 无状态进程设计
- [ ] 日志输出到stdout
- [ ] 依赖显式声明
- [ ] 端口绑定暴露服务

### Kubernetes部署
- [ ] 健康检查三件套配置
- [ ] 资源requests和limits设置
- [ ] 优雅停止处理
- [ ] 非root容器运行
- [ ] 安全上下文配置

### 可扩展性
- [ ] HPA已配置
- [ ] 无状态可水平扩展
- [ ] Pod反亲和性配置(高可用)
- [ ] PDB(Pod Disruption Budget)已设置

### 运维就绪
- [ ] GitOps部署流程
- [ ] 金丝雀/蓝绿发布策略
- [ ] 回滚机制可用
- [ ] 监控和告警覆盖
