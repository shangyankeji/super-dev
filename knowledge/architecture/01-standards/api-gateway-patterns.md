---
id: api-gateway-patterns
title: API Gateway模式
domain: architecture
category: 01-standards
difficulty: intermediate
tags: [agent, api, architecture, checklist, gateway, patterns, 实战代码示例, 常见陷阱]
quality_score: 70
last_updated: 2026-06-15
---
# API Gateway模式

## 概述
API Gateway是微服务架构的统一入口,负责路由、限流、认证、缓存、聚合等横切关注点。本指南覆盖Gateway核心功能模式、BFF(Backend for Frontend)、主流Gateway选型和实战配置。

## 核心概念

### 1. API Gateway职责
- **路由(Routing)**: 将请求转发到正确的后端服务
- **认证授权(Auth)**: 集中处理认证,传递身份信息
- **限流(Rate Limiting)**: 防止API滥用和DDoS
- **缓存(Caching)**: 缓存高频响应减少后端压力
- **聚合(Aggregation)**: 合并多个后端调用为单个响应
- **协议转换**: REST↔gRPC、HTTP↔WebSocket
- **监控**: 请求日志、指标收集、分布式追踪
- **安全**: TLS终止、CORS、安全头、WAF

### 2. Gateway模式

| 模式 | 描述 | 适用场景 |
|------|------|----------|
| 单一Gateway | 所有流量经过一个Gateway | 小型系统 |
| BFF(Backend for Frontend) | 每种客户端一个Gateway | 多端(Web/App/小程序) |
| 分层Gateway | 外部Gateway + 内部Gateway | 企业级多层安全 |
| Sidecar Gateway | 每个服务旁挂代理 | Service Mesh(Envoy) |

### 3. 主流Gateway对比

| 特性 | Kong | APISIX | Envoy | Nginx | AWS API GW | Traefik |
|------|------|--------|-------|-------|------------|---------|
| 语言 | Lua/Go | Lua | C++ | C | 托管 | Go |
| 性能 | 高 | 极高 | 极高 | 极高 | 中 | 高 |
| 插件生态 | 丰富 | 丰富 | 过滤器 | 模块 | 有限 | 中等 |
| K8s原生 | Ingress | Ingress | Envoy Gateway | Ingress | — | Ingress |
| 管理UI | 有(Enterprise) | Dashboard | — | — | Console | Dashboard |
| 适用场景 | 通用API管理 | 高性能/动态 | Service Mesh | 传统负载均衡 | AWS生态 | 容器编排 |

## 实战代码示例

### Kong Gateway配置

```yaml
# kong.yml — 声明式配置
_format_version: "3.0"

services:
  - name: user-service
    url: http://user-service:8080
    connect_timeout: 5000
    read_timeout: 30000
    write_timeout: 10000
    retries: 3
    routes:
      - name: user-routes
        paths:
          - /api/v1/users
          - /api/v2/users
        methods:
          - GET
          - POST
          - PUT
          - DELETE
        strip_path: false

  - name: order-service
    url: http://order-service:8080
    routes:
      - name: order-routes
        paths:
          - /api/v1/orders
        methods:
          - GET
          - POST

plugins:
  # 全局限流
  - name: rate-limiting
    config:
      second: 50
      minute: 1000
      hour: 10000
      policy: redis
      redis_host: redis
      redis_port: 6379

  # JWT认证
  - name: jwt
    service: user-service
    config:
      claims_to_verify:
        - exp
      header_names:
        - Authorization

  # 请求转换
  - name: request-transformer
    service: user-service
    config:
      add:
        headers:
          - "X-Gateway: kong"
          - "X-Request-Start: $(now)"

  # 响应缓存
  - name: proxy-cache
    service: user-service
    route: user-routes
    config:
      response_code:
        - 200
      request_method:
        - GET
      content_type:
        - application/json
      cache_ttl: 300
      strategy: memory

  # CORS
  - name: cors
    config:
      origins:
        - https://app.example.com
        - https://admin.example.com
      methods:
        - GET
        - POST
        - PUT
        - DELETE
      headers:
        - Authorization
        - Content-Type
      max_age: 3600
      credentials: true

  # 日志
  - name: http-log
    config:
      http_endpoint: http://log-collector:8080/logs
      method: POST
      content_type: application/json
```

### BFF模式实现

```python
# BFF — Web端专用Gateway
from fastapi import FastAPI, Depends, Request
import httpx
from cachetools import TTLCache

web_bff = FastAPI(title="Web BFF")
cache = TTLCache(maxsize=1000, ttl=300)

class ServiceClient:
    """内部服务客户端"""
    def __init__(self):
        self.client = httpx.AsyncClient(timeout=10.0)
        self.services = {
            "user": "http://user-service:8080",
            "order": "http://order-service:8080",
            "product": "http://product-service:8080",
            "review": "http://review-service:8080",
        }

    async def call(self, service: str, path: str, **kwargs) -> dict:
        url = f"{self.services[service]}{path}"
        response = await self.client.get(url, **kwargs)
        response.raise_for_status()
        return response.json()

svc = ServiceClient()

@web_bff.get("/bff/dashboard")
async def web_dashboard(auth = Depends(authenticate)):
    """Web端仪表盘 — 聚合多个服务数据"""
    import asyncio

    # 并行调用多个服务
    user_task = svc.call("user", f"/api/users/{auth.user_id}")
    orders_task = svc.call("order", f"/api/orders?user_id={auth.user_id}&limit=5")
    stats_task = svc.call("order", f"/api/orders/stats?user_id={auth.user_id}")

    user, orders, stats = await asyncio.gather(
        user_task, orders_task, stats_task,
        return_exceptions=True,
    )

    # 降级处理: 某个服务失败不影响整体
    return {
        "user": user if not isinstance(user, Exception) else None,
        "recent_orders": orders if not isinstance(orders, Exception) else [],
        "stats": stats if not isinstance(stats, Exception) else {"error": "unavailable"},
    }

@web_bff.get("/bff/product/{product_id}")
async def web_product_detail(product_id: int):
    """Web端产品详情 — 聚合产品+评论"""
    import asyncio

    cache_key = f"product_detail:{product_id}"
    if cache_key in cache:
        return cache[cache_key]

    product, reviews = await asyncio.gather(
        svc.call("product", f"/api/products/{product_id}"),
        svc.call("review", f"/api/reviews?product_id={product_id}&limit=10"),
    )

    # Web端需要完整信息
    result = {
        "product": product,
        "reviews": reviews,
        "review_summary": {
            "count": len(reviews),
            "average": sum(r["rating"] for r in reviews) / len(reviews) if reviews else 0,
        },
    }

    cache[cache_key] = result
    return result

# BFF — 移动端专用Gateway(精简数据)
mobile_bff = FastAPI(title="Mobile BFF")

@mobile_bff.get("/bff/product/{product_id}")
async def mobile_product_detail(product_id: int):
    """移动端产品详情 — 精简数据,减少传输"""
    product = await svc.call("product", f"/api/products/{product_id}")

    # 移动端只返回必要字段,省流量
    return {
        "id": product["id"],
        "name": product["name"],
        "price": product["price"],
        "image": product["image_url"],
        "rating": product["average_rating"],
        # 评论单独分页加载,不在详情页返回
    }
```

### 请求聚合与响应转换

```python
# GraphQL作为聚合层
import strawberry
from strawberry.fastapi import GraphQLRouter

@strawberry.type
class DashboardData:
    user: "UserSummary"
    notifications: list["Notification"]
    recent_orders: list["OrderSummary"]
    recommendations: list["ProductSummary"]

@strawberry.type
class Query:
    @strawberry.field
    async def dashboard(self, info) -> DashboardData:
        """单次请求获取仪表盘所有数据"""
        user_id = info.context["user_id"]

        # DataLoader批量加载避免N+1
        user = await info.context["user_loader"].load(user_id)
        notifications = await info.context["notification_loader"].load(user_id)
        orders = await info.context["order_loader"].load(user_id)
        recommendations = await info.context["rec_loader"].load(user_id)

        return DashboardData(
            user=user,
            notifications=notifications,
            recent_orders=orders,
            recommendations=recommendations,
        )

schema = strawberry.Schema(query=Query)
graphql_app = GraphQLRouter(schema)
```

### 缓存策略

```python
# Gateway层缓存
from fastapi import Response
from hashlib import sha256

class GatewayCache:
    """Gateway响应缓存"""

    def __init__(self, redis):
        self.redis = redis

    def cache_key(self, request: Request) -> str:
        """生成缓存键"""
        key_parts = [
            request.method,
            request.url.path,
            str(sorted(request.query_params.items())),
        ]
        return f"gw_cache:{sha256(':'.join(key_parts).encode()).hexdigest()}"

    async def get_or_fetch(self, request: Request, fetch_fn, ttl: int = 300):
        """缓存穿透策略"""
        # 只缓存GET请求
        if request.method != "GET":
            return await fetch_fn()

        key = self.cache_key(request)
        cached = await self.redis.get(key)
        if cached:
            return json.loads(cached)

        result = await fetch_fn()
        await self.redis.setex(key, ttl, json.dumps(result))
        return result

    async def invalidate_pattern(self, pattern: str):
        """按模式失效缓存"""
        keys = []
        async for key in self.redis.scan_iter(f"gw_cache:*{pattern}*"):
            keys.append(key)
        if keys:
            await self.redis.delete(*keys)
```

### 健康检查与服务发现

```python
# Gateway健康检查
class ServiceHealth:
    """后端服务健康状态"""

    def __init__(self):
        self.services: dict[str, dict] = {}
        self.client = httpx.AsyncClient(timeout=5.0)

    async def check_all(self) -> dict:
        results = {}
        for name, url in SERVICE_REGISTRY.items():
            try:
                resp = await self.client.get(f"{url}/health")
                results[name] = {
                    "status": "healthy" if resp.status_code == 200 else "degraded",
                    "latency_ms": resp.elapsed.total_seconds() * 1000,
                }
            except Exception as e:
                results[name] = {
                    "status": "unhealthy",
                    "error": str(e),
                }
        return results

@app.get("/gateway/health")
async def gateway_health():
    """Gateway健康检查(含后端服务状态)"""
    health = ServiceHealth()
    services = await health.check_all()

    all_healthy = all(s["status"] == "healthy" for s in services.values())
    status_code = 200 if all_healthy else 503

    return JSONResponse(
        status_code=status_code,
        content={
            "gateway": "healthy",
            "services": services,
            "timestamp": datetime.utcnow().isoformat(),
        },
    )
```

## 最佳实践

### 1. Gateway职责边界
- Gateway只处理横切关注点(路由/认证/限流/缓存)
- 业务逻辑放在后端服务,不在Gateway中
- 聚合逻辑放BFF层,不放通用Gateway
- 避免Gateway成为单点瓶颈

### 2. 性能优化
- 启用HTTP/2和连接复用
- 合理设置超时(connect/read/write分开)
- 缓存高频GET请求
- 压缩响应(gzip/brotli)
- 连接池管理后端连接

### 3. 高可用
- Gateway多实例部署(至少2个)
- 使用负载均衡器(ALB/NLB)前置
- 后端服务降级(返回缓存/默认值)
- 断路器防止级联故障

### 4. 安全
- TLS终止在Gateway层
- 集中认证,后端信任Gateway传递的身份
- 输入验证在Gateway层做第一道
- 安全头统一在Gateway添加
- 日志不记录敏感数据(token/password)

### 5. 可观测性
- 每个请求分配唯一ID(X-Request-ID)
- 记录请求延迟/状态码/路由目标
- 分布式追踪传播(W3C Trace Context)
- 实时仪表盘监控流量和错误率

## 常见陷阱

### 陷阱1: Gateway承担过多业务逻辑
```
# 错误: 在Gateway中实现业务规则
# Gateway检查库存→计算价格→验证优惠券→创建订单

# 正确: Gateway只做路由和横切关注点
# Gateway认证→路由到order-service→order-service处理业务
```

### 陷阱2: 聚合请求无超时控制
```python
# 错误: 等待所有后端响应,一个慢全部慢
results = await asyncio.gather(svc_a(), svc_b(), svc_c())

# 正确: 设置超时,降级处理
results = await asyncio.gather(
    asyncio.wait_for(svc_a(), timeout=3.0),
    asyncio.wait_for(svc_b(), timeout=3.0),
    asyncio.wait_for(svc_c(), timeout=3.0),
    return_exceptions=True,
)
# 对超时的服务返回降级数据
```

### 陷阱3: 缓存击穿
```python
# 错误: 热点key过期后大量请求穿透到后端
# 正确: 使用singleflight/mutex防止击穿
async def get_with_lock(key: str, fetch_fn, ttl: int):
    cached = await redis.get(key)
    if cached:
        return json.loads(cached)

    lock_key = f"lock:{key}"
    acquired = await redis.set(lock_key, "1", nx=True, ex=10)
    if acquired:
        result = await fetch_fn()
        await redis.setex(key, ttl, json.dumps(result))
        await redis.delete(lock_key)
        return result
    else:
        await asyncio.sleep(0.1)
        return await get_with_lock(key, fetch_fn, ttl)
```

### 陷阱4: 忽略Gateway自身的限流
```
# 错误: 只限制客户端请求,不限制Gateway到后端的请求
# 后端可能被Gateway的重试请求压垮

# 正确: 双向限流
# 客户端→Gateway: 限流
# Gateway→后端: 断路器+限流
```

## Agent Checklist

### Gateway选型
- [ ] 根据需求选择合适的Gateway(通用/BFF/Mesh)
- [ ] 评估性能需求(QPS/延迟)
- [ ] 确认插件生态满足需求
- [ ] 高可用部署方案已设计

### 核心功能
- [ ] 路由规则配置正确
- [ ] 认证授权已集中处理
- [ ] 限流策略已配置(全局+按路由)
- [ ] 缓存策略已优化

### 可靠性
- [ ] 超时配置合理
- [ ] 断路器已配置
- [ ] 降级策略已实现
- [ ] 后端健康检查已启用

### 可观测性
- [ ] 请求ID贯穿全链路
- [ ] 延迟/错误率/流量指标已收集
- [ ] 分布式追踪已启用
- [ ] 告警规则已配置
