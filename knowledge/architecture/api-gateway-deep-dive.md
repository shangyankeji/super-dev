---
id: api-gateway-deep-dive
title: API网关深度指南
domain: architecture
category: api-gateway-deep-dive.md
difficulty: intermediate
tags: [api, architecture, deep, dive, gateway, 主流api网关对比, 安全防护, 性能优化]
quality_score: 70
last_updated: 2026-06-15
---
# API网关深度指南

## 概述

API网关是微服务架构中的关键基础设施,作为系统的统一入口,负责请求路由、协议转换、认证授权、限流熔断、日志监控等功能。它屏蔽了后端微服务的复杂性,为客户端提供统一、简化的API接口。

## 核心功能

### 1. 路由转发
- 请求路由与负载均衡
- 服务发现集成
- 动态路由配置

### 2. 协议转换
- HTTP/HTTPS转换
- REST/GraphQL转换
- HTTP/gRPC转换

### 3. 认证授权
- 统一认证入口
- Token验证
- 权限校验

### 4. 流量控制
- 限流与熔断
- 请求重试
- 超时控制

### 5. 安全防护
- 请求验证
- SQL注入防护
- XSS防护

### 6. 可观测性
- 请求日志
- 性能监控
- 分布式追踪

## 主流API网关对比

### Kong

#### 架构
```
Kong Server:
- OpenResty + Nginx
- Lua插件机制
- 高性能

数据存储:
- PostgreSQL(推荐)
- Cassandra
- 无存储模式(声明式配置)
```

#### 核心特性
```
优点:
- 高性能(基于Nginx)
- 插件丰富(100+)
- 云原生,支持Kubernetes
- 管理界面(Kong Enterprise)
- 活跃社区

缺点:
- 学习曲线陡峭
- 企业版收费昂贵
- 配置复杂度高

适用场景:
- 大规模微服务
- 需要丰富插件
- Kubernetes环境
```

#### 部署示例
```yaml
# docker-compose.yml
version: '3.8'
services:
  kong-database:
    image: postgres:13
    environment:
      POSTGRES_USER: kong
      POSTGRES_DB: kong
      POSTGRES_PASSWORD: kong
    volumes:
      - kong-data:/var/lib/postgresql/data

  kong-migration:
    image: kong:latest
    command: kong migrations bootstrap
    depends_on:
      - kong-database
    environment:
      KONG_DATABASE: postgres
      KONG_PG_HOST: kong-database
      KONG_PG_PASSWORD: kong

  kong:
    image: kong:latest
    depends_on:
      - kong-migration
    environment:
      KONG_DATABASE: postgres
      KONG_PG_HOST: kong-database
      KONG_PG_PASSWORD: kong
      KONG_PROXY_ACCESS_LOG: /dev/stdout
      KONG_ADMIN_ACCESS_LOG: /dev/stdout
      KONG_PROXY_ERROR_LOG: /dev/stderr
      KONG_ADMIN_ERROR_LOG: /dev/stderr
      KONG_ADMIN_LISTEN: '0.0.0.0:8001'
    ports:
      - "8000:8000"   # HTTP代理
      - "8443:8443"   # HTTPS代理
      - "8001:8001"   # Admin API
      - "8444:8444"   # Admin HTTPS
```

#### 配置示例
```bash
# 添加服务
curl -i -X POST http://localhost:8001/services \
  -d "name=order-service" \
  -d "url=http://order-service:8080"

# 添加路由
curl -i -X POST http://localhost:8001/services/order-service/routes \
  -d "paths[]=/orders"

# 添加插件(JWT认证)
curl -i -X POST http://localhost:8001/routes/order-route/plugins \
  -d "name=jwt"

# 添加插件(限流)
curl -i -X POST http://localhost:8001/services/order-service/plugins \
  -d "name=rate-limiting" \
  -d "config.minute=100" \
  -d "config.policy=local"

# 声明式配置(deck)
_format_version: "3.0"
services:
  - name: order-service
    url: http://order-service:8080
    routes:
      - name: order-route
        paths:
          - /orders
    plugins:
      - name: jwt
      - name: rate-limiting
        config:
          minute: 100
          policy: local
```

### Spring Cloud Gateway

#### 架构
```
基于Spring WebFlux:
- Reactor响应式编程
- Netty服务器
- 非阻塞IO

组件:
- Route(路由)
- Predicate(断言)
- Filter(过滤器)
```

#### 核心特性
```
优点:
- Spring生态集成
- 响应式高性能
- 灵活的配置方式
- Java开发友好

缺点:
- 相对年轻,生态不如Kong
- 依赖Spring体系
- 管理界面缺失

适用场景:
- Spring Cloud微服务
- Java技术栈
- 中小型系统
```

#### 实现示例
```java
// 依赖
dependencies {
    implementation 'org.springframework.cloud:spring-cloud-starter-gateway'
    implementation 'org.springframework.boot:spring-boot-starter-webflux'
}

// 配置类
@Configuration
public class GatewayConfig {
    @Bean
    public RouteLocator customRouteLocator(RouteLocatorBuilder builder) {
        return builder.routes()
            .route("order-service", r -> r
                .path("/orders/**")
                .filters(f -> f
                    .stripPrefix(1)
                    .addRequestHeader("X-Gateway", "Spring-Cloud-Gateway")
                    .addResponseHeader("X-Response-Time", System.currentTimeMillis())
                    .requestRateLimiter(c -> c
                        .setRateLimiter(redisRateLimiter())
                    )
                    .circuitBreaker(c -> c
                        .setName("orderCircuitBreaker")
                        .setFallbackUri("forward:/fallback/orders")
                    )
                )
                .uri("lb://order-service")
            )
            .route("inventory-service", r -> r
                .path("/inventory/**")
                .filters(f -> f
                    .stripPrefix(1)
                    .retry(3)
                )
                .uri("lb://inventory-service")
            )
            .build();
    }

    @Bean
    public RedisRateLimiter redisRateLimiter() {
        return new RedisRateLimiter(100, 200); // 100 replenishRate, 200 burstCapacity
    }
}

// 全局过滤器
@Component
public class AuthenticationFilter implements GlobalFilter, Ordered {
    @Override
    public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
        ServerHttpRequest request = exchange.getRequest();

        // 跳过白名单路径
        if (isWhitelisted(request.getPath().value())) {
            return chain.filter(exchange);
        }

        // 验证Token
        String token = request.getHeaders().getFirst("Authorization");
        if (token == null || !validateToken(token)) {
            exchange.getResponse().setStatusCode(HttpStatus.UNAUTHORIZED);
            return exchange.getResponse().setComplete();
        }

        // 添加用户信息到请求头
        ServerHttpRequest mutatedRequest = request.mutate()
            .header("X-User-Id", getUserIdFromToken(token))
            .build();

        return chain.filter(exchange.mutate().request(mutatedRequest).build());
    }

    @Override
    public int getOrder() {
        return -100; // 高优先级
    }
}

// 限流KeyResolver
@Configuration
public class RateLimiterConfig {
    @Bean
    public KeyResolver ipKeyResolver() {
        return exchange -> Mono.just(
            exchange.getRequest().getRemoteAddress().getAddress().getHostAddress()
        );
    }

    @Bean
    public KeyResolver userKeyResolver() {
        return exchange -> Mono.justOrEmpty(
            exchange.getRequest().getHeaders().getFirst("X-User-Id")
        );
    }
}
```

```yaml
# application.yml配置方式
spring:
  cloud:
    gateway:
      routes:
        - id: order-service
          uri: lb://order-service
          predicates:
            - Path=/orders/**
          filters:
            - StripPrefix=1
            - name: RequestRateLimiter
              args:
                redis-rate-limiter.replenishRate: 100
                redis-rate-limiter.burstCapacity: 200
                key-resolver: "#{@ipKeyResolver}"
            - name: CircuitBreaker
              args:
                name: orderCircuitBreaker
                fallbackUri: forward:/fallback/orders
            - AddRequestHeader=X-Gateway,Spring-Cloud-Gateway
            - AddResponseHeader=X-Response-Time,${spring.cloud.gateway.response.timeout}

        - id: inventory-service
          uri: lb://inventory-service
          predicates:
            - Path=/inventory/**
            - Method=GET,POST
          filters:
            - StripPrefix=1
            - Retry=3

      default-filters:
        - AddRequestHeader=X-Request-Id,${spring.cloud.gateway.request.id}
      globalcors:
        cors-configurations:
          '[/**]':
            allowedOrigins: "*"
            allowedMethods:
              - GET
              - POST
              - PUT
              - DELETE
            allowedHeaders: "*"
            allowCredentials: true

# 熔断配置
resilience4j:
  circuitbreaker:
    configs:
      default:
        failureRateThreshold: 50
        waitDurationInOpenState: 10000
        slidingWindowSize: 10
    instances:
      orderCircuitBreaker:
        baseConfig: default
```

### APISIX

#### 架构
```
Apache APISIX:
- 基于OpenResty + Nginx
- Lua实现
- 高性能

组件:
- APISIX(网关)
- Dashboard(管理界面)
- Admin API(管理接口)
```

#### 核心特性
```
优点:
- 高性能(单核23000 QPS)
- 动态路由,热加载
- 云原生,支持Kubernetes
- 插件热加载
- Dashboard完善
- 中文社区活跃

缺点:
- 相对年轻(2019年开源)
- 企业版收费
- 部分插件不够成熟

适用场景:
- 国内环境
- 云原生架构
- 需要Dashboard
```

#### 配置示例
```yaml
# docker-compose.yml
version: '3.8'
services:
  apisix:
    image: apache/apisix:latest
    volumes:
      - ./apisix/config.yml:/usr/local/apisix/conf/config.yml:ro
      - ./apisix/nginx.conf:/usr/local/apisix/nginx/conf/nginx.conf:ro
    ports:
      - "9080:9080"   # HTTP
      - "9443:9443"   # HTTPS
      - "9180:9180"   # Admin API

  apisix-dashboard:
    image: apache/apisix-dashboard:latest
    volumes:
      - ./dashboard/conf.yml:/usr/local/apisix-dashboard/conf/conf.yml:ro
    ports:
      - "9000:9000"
```

```bash
# 通过Admin API配置
# 添加上游(Upstream)
curl http://127.0.0.1:9180/apisix/admin/upstreams/1 \
  -H "X-API-KEY: edd1c9f034335f136f87ad84b625c8f1" -X PUT -d '
{
  "type": "roundrobin",
  "nodes": {
    "order-service:8080": 1
  }
}'

# 添加路由
curl http://127.0.0.1:9180/apisix/admin/routes/1 \
  -H "X-API-KEY: edd1c9f034335f136f87ad84b625c8f1" -X PUT -d '
{
  "uri": "/orders/*",
  "upstream_id": 1,
  "plugins": {
    "limit-count": {
      "count": 100,
      "time_window": 60,
      "rejected_code": 429,
      "key": "remote_addr"
    },
    "jwt-auth": {},
    "cors": {
      "allow_origins": "*",
      "allow_methods": "GET,POST,PUT,DELETE"
    }
  }
}'
```

### Nginx/OpenResty

#### 核心特性
```
优点:
- 极致性能
- 成熟稳定
- 广泛应用
- 配置灵活

缺点:
- 配置复杂
- 缺乏管理界面
- 动态配置困难
- 需要重启/重载

适用场景:
- 高性能场景
- 简单路由需求
- 已有Nginx基础设施
```

#### 配置示例
```nginx
# nginx.conf
upstream order_service {
    least_conn;
    server order-service-1:8080 weight=5 max_fails=3 fail_timeout=30s;
    server order-service-2:8080 weight=3;
    server order-service-3:8080 backup;
}

upstream inventory_service {
    least_conn;
    server inventory-service-1:8080;
    server inventory-service-2:8080;
}

# 限流配置
limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
limit_conn_zone $binary_remote_addr zone=conn_limit:10m;

server {
    listen 80;
    server_name api.example.com;

    # 启用gzip
    gzip on;
    gzip_types application/json;

    # 订单服务
    location /orders/ {
        # 限流
        limit_req zone=api_limit burst=20 nodelay;
        limit_conn conn_limit 10;

        # 代理
        proxy_pass http://order_service/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;

        # 超时
        proxy_connect_timeout 3s;
        proxy_send_timeout 5s;
        proxy_read_timeout 5s;

        # 重试
        proxy_next_upstream error timeout http_500 http_502 http_503;
        proxy_next_upstream_tries 3;
    }

    # 库存服务
    location /inventory/ {
        proxy_pass http://inventory_service/;
    }

    # 健康检查(需要OpenResty或Tengine)
    location /health {
        access_log off;
        return 200 "OK";
    }
}
```

### Traefik

#### 核心特性
```
优点:
- 云原生,自动服务发现
- Let's Encrypt自动证书
- 动态配置
- Dashboard友好
- Kubernetes原生

缺点:
- 性能不如Nginx/Kong
- 功能相对简单
- 社区规模较小

适用场景:
- Kubernetes环境
- 容器化应用
- 自动化程度要求高
```

#### 配置示例
```yaml
# traefik.yml
entryPoints:
  web:
    address: ":80"
    http:
      redirections:
        entryPoint:
          to: websecure
          scheme: https

  websecure:
    address: ":443"

providers:
  docker:
    endpoint: "unix:///var/run/docker.sock"
    exposedByDefault: false

  kubernetesIngress:
    enabled: true

certificatesResolvers:
  letsencrypt:
    acme:
      email: admin@example.com
      storage: /letsencrypt/acme.json
      httpChallenge:
        entryPoint: web

api:
  dashboard: true
  insecure: true

metrics:
  prometheus: true
```

```yaml
# Docker Compose
version: '3.8'
services:
  traefik:
    image: traefik:v2.10
    command:
      - "--configFile=/etc/traefik/traefik.yml"
    ports:
      - "80:80"
      - "443:443"
      - "8080:8080"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./traefik.yml:/etc/traefik/traefik.yml
      - ./letsencrypt:/letsencrypt

  order-service:
    image: order-service:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.orders.rule=PathPrefix(`/orders`)"
      - "traefik.http.routers.orders.entrypoints=websecure"
      - "traefik.http.routers.orders.tls.certresolver=letsencrypt"
      - "traefik.http.services.orders.loadbalancer.server.port=8080"
      - "traefik.http.middlewares.orders-ratelimit.ratelimit.average=100"
      - "traefik.http.routers.orders.middlewares=orders-ratelimit"
```

## 网关功能实现

### 1. 认证授权

#### JWT认证
```java
@Component
public class JwtAuthenticationFilter implements GlobalFilter, Ordered {
    @Value("${jwt.secret}")
    private String secret;

    @Override
    public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
        ServerHttpRequest request = exchange.getRequest();
        String path = request.getPath().value();

        // 白名单
        if (isWhitelisted(path)) {
            return chain.filter(exchange);
        }

        // 提取Token
        String authHeader = request.getHeaders().getFirst("Authorization");
        if (authHeader == null || !authHeader.startsWith("Bearer ")) {
            return unauthorized(exchange);
        }

        String token = authHeader.substring(7);

        try {
            // 验证Token
            Claims claims = Jwts.parser()
                .setSigningKey(secret)
                .parseClaimsJws(token)
                .getBody();

            // 添加用户信息到请求头
            ServerHttpRequest mutatedRequest = request.mutate()
                .header("X-User-Id", claims.getSubject())
                .header("X-User-Role", claims.get("role", String.class))
                .build();

            return chain.filter(exchange.mutate().request(mutatedRequest).build());

        } catch (Exception e) {
            return unauthorized(exchange);
        }
    }

    private Mono<Void> unauthorized(ServerWebExchange exchange) {
        exchange.getResponse().setStatusCode(HttpStatus.UNAUTHORIZED);
        exchange.getResponse().getHeaders().setContentType(MediaType.APPLICATION_JSON);

        String body = "{\"error\":\"Unauthorized\",\"message\":\"Invalid token\"}";
        DataBuffer buffer = exchange.getResponse().bufferFactory().wrap(body.getBytes());
        return exchange.getResponse().writeWith(Mono.just(buffer));
    }

    private boolean isWhitelisted(String path) {
        return path.startsWith("/auth/login") ||
               path.startsWith("/auth/register") ||
               path.startsWith("/actuator/health");
    }

    @Override
    public int getOrder() {
        return -100;
    }
}
```

#### OAuth2.0集成
```java
@Configuration
@EnableWebFluxSecurity
public class SecurityConfig {
    @Bean
    public SecurityWebFilterChain securityWebFilterChain(ServerHttpSecurity http) {
        http
            .oauth2ResourceServer()
            .jwt()
            .jwtAuthenticationConverter(jwtAuthenticationConverter());

        http
            .authorizeExchange()
            .pathMatchers("/auth/**").permitAll()
            .pathMatchers("/orders/**").hasRole("USER")
            .pathMatchers("/admin/**").hasRole("ADMIN")
            .anyExchange().authenticated();

        return http.build();
    }

    private Converter<Jwt, ? extends Mono<? extends AbstractAuthenticationToken>>
            jwtAuthenticationConverter() {
        JwtAuthenticationConverter converter = new JwtAuthenticationConverter();
        converter.setJwtGrantedAuthoritiesConverter(new JwtGrantedAuthoritiesConverter());
        return new ReactiveJwtAuthenticationConverterAdapter(converter);
    }
}
```

### 2. 限流熔断

#### Redis限流
```java
@Component
public class RedisRateLimiter {
    @Autowired
    private ReactiveRedisTemplate<String, String> redisTemplate;

    private final String script =
        "local key = KEYS[1] " +
        "local limit = tonumber(ARGV[1]) " +
        "local window = tonumber(ARGV[2]) " +
        "local current = redis.call('INCR', key) " +
        "if current == 1 then " +
        "    redis.call('EXPIRE', key, window) " +
        "end " +
        "return current <= limit";

    public Mono<Boolean> allowRequest(String key, int limit, int window) {
        return redisTemplate.execute(
            RedisScript.of(script, Boolean.class),
            Collections.singletonList(key),
            String.valueOf(limit),
            String.valueOf(window)
        ).next();
    }
}

@Component
public class RateLimitFilter implements GlobalFilter, Ordered {
    @Autowired
    private RedisRateLimiter rateLimiter;

    @Override
    public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
        String clientId = exchange.getRequest().getHeaders().getFirst("X-Client-Id");
        String key = "rate_limit:" + clientId;

        return rateLimiter.allowRequest(key, 100, 60)
            .flatMap(allowed -> {
                if (allowed) {
                    return chain.filter(exchange);
                } else {
                    exchange.getResponse().setStatusCode(HttpStatus.TOO_MANY_REQUESTS);
                    return exchange.getResponse().setComplete();
                }
            });
    }

    @Override
    public int getOrder() {
        return -50;
    }
}
```

#### 熔断降级
```java
@RestController
public class FallbackController {
    @RequestMapping("/fallback/orders")
    public Mono<Map<String, Object>> orderFallback() {
        return Mono.just(Map.of(
            "success", false,
            "message", "订单服务暂时不可用,请稍后重试",
            "timestamp", System.currentTimeMillis()
        ));
    }

    @RequestMapping("/fallback/inventory")
    public Mono<Map<String, Object>> inventoryFallback() {
        return Mono.just(Map.of(
            "success", false,
            "message", "库存服务暂时不可用",
            "timestamp", System.currentTimeMillis()
        ));
    }
}
```

### 3. 请求日志

```java
@Component
@Slf4j
public class RequestLoggingFilter implements GlobalFilter, Ordered {
    @Override
    public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
        ServerHttpRequest request = exchange.getRequest();
        long startTime = System.currentTimeMillis();

        // 生成请求ID
        String requestId = UUID.randomUUID().toString();
        exchange.getAttributes().put("requestId", requestId);

        log.info("[{}] {} {} - Start",
            requestId,
            request.getMethod(),
            request.getPath()
        );

        return chain.filter(exchange).then(Mono.fromRunnable(() -> {
            long duration = System.currentTimeMillis() - startTime;
            ServerHttpResponse response = exchange.getResponse();

            log.info("[{}] {} {} - {} - {}ms",
                requestId,
                request.getMethod(),
                request.getPath(),
                response.getStatusCode(),
                duration
            );
        }));
    }

    @Override
    public int getOrder() {
        return -200; // 最高优先级
    }
}
```

### 4. 请求响应转换

```java
@Component
public class ResponseTransformFilter implements GlobalFilter, Ordered {
    @Override
    public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
        ServerHttpResponse originalResponse = exchange.getResponse();
        DataBufferFactory bufferFactory = originalResponse.bufferFactory();

        ServerHttpResponseDecorator decoratedResponse = new ServerHttpResponseDecorator(originalResponse) {
            @Override
            public Mono<Void> writeWith(Publisher<? extends DataBuffer> body) {
                if (body instanceof Flux) {
                    Flux<? extends DataBuffer> fluxBody = (Flux<? extends DataBuffer>) body;

                    return super.writeWith(fluxBody.buffer().map(dataBuffers -> {
                        // 合并buffer
                        DataBuffer join = bufferFactory.join(dataBuffers);
                        byte[] content = new byte[join.readableByteCount()];
                        join.read(content);

                        // 原始响应
                        String originalResponse = new String(content, StandardCharsets.UTF_8);

                        // 转换响应(包装为统一格式)
                        String transformedResponse = transformResponse(originalResponse, exchange);

                        return bufferFactory.wrap(transformedResponse.getBytes(StandardCharsets.UTF_8));
                    }));
                }
                return super.writeWith(body);
            }
        };

        return chain.filter(exchange.mutate().response(decoratedResponse).build());
    }

    private String transformResponse(String originalResponse, ServerWebExchange exchange) {
        try {
            ObjectMapper mapper = new ObjectMapper();
            Object data = mapper.readValue(originalResponse, Object.class);

            Map<String, Object> wrapper = new HashMap<>();
            wrapper.put("success", true);
            wrapper.put("data", data);
            wrapper.put("timestamp", System.currentTimeMillis());

            return mapper.writeValueAsString(wrapper);
        } catch (Exception e) {
            return originalResponse;
        }
    }

    @Override
    public int getOrder() {
        return -20;
    }
}
```

### 5. 跨域处理

```java
@Configuration
public class CorsConfig {
    @Bean
    public CorsWebFilter corsWebFilter() {
        CorsConfiguration config = new CorsConfiguration();
        config.setAllowCredentials(true);
        config.addAllowedOriginPattern("*");
        config.addAllowedMethod("*");
        config.addAllowedHeader("*");
        config.addExposedHeader("*");
        config.setMaxAge(3600L);

        UrlBasedCorsConfigurationSource source = new UrlBasedCorsConfigurationSource();
        source.registerCorsConfiguration("/**", config);

        return new CorsWebFilter(source);
    }
}
```

## 性能优化

### 1. 连接池优化
```yaml
spring:
  cloud:
    gateway:
      httpclient:
        pool:
          type: ELASTIC
          max-idle-time: 15000
          evict-in-background: 10000
        connect-timeout: 3000
        response-timeout: 5000
```

### 2. 缓存策略
```java
@Component
public class CacheFilter implements GlobalFilter, Ordered {
    @Autowired
    private ReactiveRedisTemplate<String, String> redisTemplate;

    @Override
    public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
        ServerHttpRequest request = exchange.getRequest();

        // 只缓存GET请求
        if (!HttpMethod.GET.equals(request.getMethod())) {
            return chain.filter(exchange);
        }

        String cacheKey = generateCacheKey(request);

        // 尝试从缓存读取
        return redisTemplate.opsForValue().get(cacheKey)
            .flatMap(cachedResponse -> {
                // 缓存命中
                ServerHttpResponse response = exchange.getResponse();
                response.getHeaders().setContentType(MediaType.APPLICATION_JSON);
                DataBuffer buffer = response.bufferFactory()
                    .wrap(cachedResponse.getBytes(StandardCharsets.UTF_8));
                return response.writeWith(Mono.just(buffer));
            })
            .switchIfEmpty(
                // 缓存未命中,执行请求
                chain.filter(exchange).then(Mono.fromRunnable(() -> {
                    ServerHttpResponse response = exchange.getResponse();
                    // 缓存响应(根据业务设置TTL)
                    // ...
                }))
            );
    }

    private String generateCacheKey(ServerHttpRequest request) {
        return "cache:" + request.getPath().value() + ":" +
               DigestUtils.md5DigestAsHex(
                   request.getQueryParams().toString().getBytes()
               );
    }

    @Override
    public int getOrder() {
        return -30;
    }
}
```

### 3. 压缩
```java
@Component
public class CompressionFilter implements GlobalFilter, Ordered {
    @Override
    public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
        ServerHttpRequest request = exchange.getRequest();
        String acceptEncoding = request.getHeaders().getFirst("Accept-Encoding");

        if (acceptEncoding != null && acceptEncoding.contains("gzip")) {
            // 下游服务返回时压缩
            ServerHttpResponse response = exchange.getResponse();
            response.getHeaders().set("Content-Encoding", "gzip");
        }

        return chain.filter(exchange);
    }

    @Override
    public int getOrder() {
        return 0;
    }
}
```

## 高可用设计

### 1. 网关集群
```
部署方案:
- 多实例部署(至少3个节点)
- 负载均衡(Nginx/SLB/ELB)
- 无状态设计
- 共享配置中心

故障转移:
- 健康检查
- 自动摘除故障节点
- 会话保持(Sticky Session)
```

### 2. 配置中心集成
```java
@Configuration
public class DynamicRouteConfig {
    @Autowired
    private RouteDefinitionLocator routeDefinitionLocator;

    @Bean
    public RouteDefinitionRepository routeDefinitionRepository() {
        // 从Nacos/Consul/Apollo加载路由配置
        return new NacosRouteDefinitionRepository();
    }
}

public class NacosRouteDefinitionRepository implements RouteDefinitionRepository {
    @NacosValue(value = "${gateway.routes}", autoRefreshed = true)
    private String routesConfig;

    @Override
    public Flux<RouteDefinition> getRouteDefinitions() {
        List<RouteDefinition> routeDefinitions = parseRoutes(routesConfig);
        return Flux.fromIterable(routeDefinitions);
    }

    private List<RouteDefinition> parseRoutes(String config) {
        // 解析JSON/YAML配置
        // ...
    }
}
```

### 3. 降级策略
```java
@Component
public class GlobalFallbackHandler implements WebExceptionHandler {
    @Override
    public Mono<Void> handle(ServerWebExchange exchange, Throwable ex) {
        ServerHttpResponse response = exchange.getResponse();

        if (ex instanceof NotFoundException) {
            response.setStatusCode(HttpStatus.NOT_FOUND);
            return writeResponse(response, "Service not found");
        }

        if (ex instanceof ConnectException) {
            response.setStatusCode(HttpStatus.SERVICE_UNAVAILABLE);
            return writeResponse(response, "Service unavailable");
        }

        if (ex instanceof TimeoutException) {
            response.setStatusCode(HttpStatus.GATEWAY_TIMEOUT);
            return writeResponse(response, "Request timeout");
        }

        response.setStatusCode(HttpStatus.INTERNAL_SERVER_ERROR);
        return writeResponse(response, "Internal server error");
    }

    private Mono<Void> writeResponse(ServerHttpResponse response, String message) {
        response.getHeaders().setContentType(MediaType.APPLICATION_JSON);
        String body = String.format("{\"error\":\"%s\",\"timestamp\":%d}",
            message, System.currentTimeMillis());
        DataBuffer buffer = response.bufferFactory().wrap(body.getBytes());
        return response.writeWith(Mono.just(buffer));
    }
}
```

## 安全防护

### 1. SQL注入防护
```java
@Component
public class SqlInjectionFilter implements GlobalFilter, Ordered {
    private static final Pattern SQL_PATTERN = Pattern.compile(
        "(?i)(select|insert|update|delete|drop|union|exec|execute|xp_cmdshell)"
    );

    @Override
    public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
        ServerHttpRequest request = exchange.getRequest();

        // 检查查询参数
        if (containsSqlInjection(request.getQueryParams())) {
            return forbidden(exchange, "Invalid query parameters");
        }

        // 检查路径参数
        if (containsSqlInjection(request.getPath().value())) {
            return forbidden(exchange, "Invalid path");
        }

        return chain.filter(exchange);
    }

    private boolean containsSqlInjection(MultiValueMap<String, String> params) {
        return params.values().stream()
            .flatMap(List::stream)
            .anyMatch(this::isSqlInjection);
    }

    private boolean containsSqlInjection(String value) {
        return isSqlInjection(value);
    }

    private boolean isSqlInjection(String value) {
        return SQL_PATTERN.matcher(value).find();
    }

    private Mono<Void> forbidden(ServerWebExchange exchange, String message) {
        exchange.getResponse().setStatusCode(HttpStatus.FORBIDDEN);
        return exchange.getResponse().setComplete();
    }

    @Override
    public int getOrder() {
        return -90;
    }
}
```

### 2. XSS防护
```java
@Component
public class XssFilter implements GlobalFilter, Ordered {
    @Override
    public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
        ServerHttpRequest request = exchange.getRequest();

        ServerHttpRequest mutatedRequest = request.mutate()
            .headers(headers -> {
                // 安全头
                headers.set("X-XSS-Protection", "1; mode=block");
                headers.set("X-Content-Type-Options", "nosniff");
                headers.set("X-Frame-Options", "DENY");
                headers.set("Content-Security-Policy", "default-src 'self'");
            })
            .build();

        return chain.filter(exchange.mutate().request(mutatedRequest).build());
    }

    @Override
    public int getOrder() {
        return -80;
    }
}
```

## 监控指标

### 1. Prometheus集成
```yaml
management:
  endpoints:
    web:
      exposure:
        include: prometheus,health,info,gateway
  metrics:
    tags:
      application: ${spring.application.name}
    export:
      prometheus:
        enabled: true
```

### 2. 自定义指标
```java
@Component
public class MetricsFilter implements GlobalFilter, Ordered {
    private final Counter requestCounter;
    private final Timer requestTimer;

    public MetricsFilter(MeterRegistry registry) {
        this.requestCounter = Counter.builder("gateway.requests")
            .description("Total gateway requests")
            .register(registry);

        this.requestTimer = Timer.builder("gateway.request.duration")
            .description("Gateway request duration")
            .register(registry);
    }

    @Override
    public Mono<Void> filter(ServerWebExchange exchange, GatewayFilterChain chain) {
        requestCounter.increment();

        return requestTimer.record(() -> chain.filter(exchange));
    }

    @Override
    public int getOrder() {
        return -150;
    }
}
```

## 选型建议

### 场景对比
```
Kong:
- 大规模微服务(100+服务)
- 需要丰富插件
- 多语言技术栈
- Kubernetes环境

Spring Cloud Gateway:
- Spring Cloud生态
- Java技术栈
- 中小型系统(10-50服务)
- 响应式架构

APISIX:
- 国内环境
- 需要Dashboard
- 云原生架构
- Apache生态

Nginx:
- 高性能场景
- 简单路由
- 已有Nginx基础设施
- 成本敏感

Traefik:
- Kubernetes环境
- 自动化需求高
- 容器化应用
- Let's Encrypt自动证书
```

## 参考资源

### 官方文档
- Kong: https://docs.konghq.com/
- Spring Cloud Gateway: https://spring.io/projects/spring-cloud-gateway
- APISIX: https://apisix.apache.org/
- Traefik: https://doc.traefik.io/traefik/

### 最佳实践
- 《微服务架构设计模式》
- 《构建高性能Web服务器》
- Nginx官方指南
- Kong最佳实践文档
