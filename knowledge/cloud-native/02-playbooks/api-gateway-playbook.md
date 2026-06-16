---
id: api-gateway-playbook
title: API 网关实战手册（Kong/APISIX）
domain: cloud-native
category: 02-playbooks
difficulty: advanced
tags: [api-gateway, kong, apisix, rate-limiting, routing, authentication, cors, tls, proxy, microservices, enterprise]
quality_score: 93
maintainer: platform-team@superdev.com
last_updated: 2026-06-15
---

# API 网关实战手册（Kong / Apache APISIX）

> 基于 [APISIX vs Kong](https://apisix.apache.org/learning-center/apisix-vs-kong/) + [OneUptime Gateway Config](https://oneuptime.com/blog/post/2026-01-25-api-gateway-configuration/view) + [Moesif Gateway Comparison](https://www.moesif.com/blog/technical/api-gateways/How-to-Choose-The-Right-API-Gateway-For-Your-Platform-Comparison-Of-Kong-Tyk-Apigee-And-Alternatives/)

## 网关核心职责

```
客户端 → API Gateway → 后端微服务
         │
         ├─ 认证（JWT/OAuth2 验证）
         ├─ 限流（rate limiting）
         ├─ 路由（path → service 映射）
         ├─ TLS 终止
         ├─ CORS
         ├─ 请求转换（header 注入）
         ├─ 日志（访问日志 → Loki/ELK）
         └─ 熔断（circuit breaker）
```

## Kong 配置

### 路由 + 认证 + 限流
```yaml
# kong.yml（声明式配置）
services:
  - name: user-service
    url: http://user-service:3000
    routes:
      - name: users
        paths: ["/api/users"]
        strip_path: false
    plugins:
      - name: jwt-auth              # JWT 认证
      - name: rate-limiting          # 限流
        config:
          minute: 100                # 每分钟 100 次
          hour: 1000
          policy: redis              # 多节点共享限流计数
          redis_host: redis
      - name: cors                   # CORS
        config:
          origins: ["https://app.example.com"]
          methods: [GET, POST, PATCH, DELETE]
      - name: prometheus             # 指标暴露

  - name: order-service
    url: http://order-service:3000
    routes:
      - name: orders
        paths: ["/api/orders"]
    plugins:
      - name: jwt-auth
      - name: rate-limiting
        config:
          minute: 50                 # 订单 API 更严格
```

## Apache APISIX 配置

```yaml
# config.yaml — 路由 + 插件
routes:
  - uri: /api/users/*
    upstream:
      type: roundrobin
      nodes:
        "user-service:3000": 1
    plugins:
      jwt-auth: {}                   # JWT 认证
      limit-req:                     # 限流（漏桶）
        rate: 10                     # 10 请求/秒
        burst: 5                     # 突发 5
        rejected_code: 429
      cors:
        allow_origins: "https://app.example.com"
        allow_methods: "GET,POST,PATCH,DELETE"
      prometheus:
        prefer_name: true

# 消费者（API Key 认证）
consumers:
  - username: mobile-app
    plugins:
      key-auth:
        key: secret-api-key-xxx
```

## 限流策略

| 策略 | 场景 | 实现 |
|------|------|------|
| 固定窗口 | 简单限制（100/min） | `rate-limiting` (Kong) / `limit-count` (APISIX) |
| 滑动窗口 | 精确限制（防边界突发） | Redis sorted set |
| 漏桶 | 平滑流量（10/s + 突发 5） | `limit-req` (APISIX) |
| 令牌桶 | 允许突发 + 平均限制 | 自定义 Redis Lua |
| 并发限制 | 防资源耗尽（100 并发） | `limit-conn` (APISIX) |

### 分级限流
```yaml
# 按用户角色分级
plugins:
  rate-limiting:
    config:
      # admin: 1000/min, user: 100/min, anon: 10/min
      limits:
        admin: { minute: 1000 }
        user: { minute: 100 }
        anonymous: { minute: 10 }
      policy: redis
```

## 熔断（Circuit Breaker）

```yaml
# APISIX circuit-breaker 插件
plugins:
  circuit-breaker:
    break_condition: "responses[503].ratio > 0.5"  # 503 > 50%
    default_return_code: 503
    max_breaker_sec: 30     # 熔断 30s
    unhealthy: { http_statuses: [500, 503], successes: 0, http_Requests: 3, time_Window: 10 }
    healthy: { http_statuses: [200], successes: 2, http_Requests: 3, time_Window: 10 }
```

## 网关选择
| 维度 | Kong | APISIX |
|------|------|--------|
| 架构 | OpenResty (Nginx) | OpenResty (Nginx) + etcd |
| 配置 | DB-backed（PG/Postgres） | etcd（动态热更新） |
| 性能 | 优秀 | 略快（Radix tree 路由） |
| 插件 | 丰富（企业版更多） | 丰富 + AI 插件 |
| 生态 | 成熟（Konnect 企业版） | Apache 社区 + API7 企业版 |

## 生产检查清单
- [ ] 所有端点经网关（不直接暴露后端）
- [ ] TLS 终止在网关
- [ ] 认证在网关层（JWT/OAuth2）
- [ ] 限流（按用户/API 分级）
- [ ] CORS 配置（精确 origin）
- [ ] 熔断（防级联故障）
- [ ] 访问日志 → 日志系统
- [ ] Prometheus 指标暴露
- [ ] 网关自身高可用（≥2 副本）
