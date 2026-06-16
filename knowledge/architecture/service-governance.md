---
id: service-governance
title: 服务治理完全指南
domain: architecture
category: service-governance.md
difficulty: intermediate
tags: [architecture, governance, service, 服务注册与发现, 核心能力, 概述, 熔断与降级, 负载均衡]
quality_score: 70
last_updated: 2026-06-15
---
# 服务治理完全指南

## 概述

服务治理是一套用于管理微服务架构中服务间交互的方法论和技术体系。它涵盖服务注册发现、负载均衡、流量控制、熔断降级、服务容错、服务监控等各个方面,确保微服务系统的稳定性、可靠性和高性能。

## 核心能力

### 1. 服务生命周期管理
- 服务注册与发现
- 服务上下线管理
- 服务健康检查

### 2. 流量治理
- 负载均衡
- 流量分配
- 灰度发布
- 流量镜像

### 3. 可靠性保障
- 熔断降级
- 限流控制
- 超时控制
- 重试机制

### 4. 安全治理
- 认证授权
- 访问控制
- 通信加密
- 审计日志

### 5. 可观测性
- 指标监控
- 日志采集
- 分布式追踪
- 服务依赖分析

## 服务注册与发现

### 核心概念
```
服务提供者(Provider): 提供服务的应用
服务消费者(Consumer): 调用服务的应用
服务注册中心(Registry): 存储服务实例信息
```

### 注册中心对比

#### Consul
```
架构:
- Server节点(Raft协议)
- Client节点
- Agent(每个节点运行)

特性:
- 服务发现
- 健康检查
- KV存储
- 多数据中心
- DNS/HTTP接口

优点:
- 功能全面
- 支持多语言
- 配置中心集成

缺点:
- 需要部署Agent
- 运维复杂度

适用场景:
- 异构技术栈
- 多数据中心
- 配置管理需求
```

#### Nacos
```
架构:
- Nacos Server(集群)
- Nacos Client
- 数据存储(MySQL/嵌入式)

特性:
- 服务发现
- 配置管理
- 动态DNS
- 服务元数据管理

优点:
- 阿里开源,生产验证
- Spring Cloud Alibaba集成
- 中文文档完善
- 简单易用

缺点:
- 社区生态相对较小
- 性能不如Consul

适用场景:
- Spring Cloud生态
- 国内环境
- 中小型系统
```

#### Eureka
```
架构:
- Eureka Server(集群)
- Eureka Client
- AP架构

特性:
- 服务注册发现
- 自我保护模式
- 区域感知

优点:
- Spring Cloud原生支持
- 高可用设计
- 简单易用

缺点:
- 已进入维护模式
- 功能单一
- 性能一般

适用场景:
- Spring Cloud Netflix生态
- 已有Eureka系统
```

#### ZooKeeper
```
架构:
- Leader节点
- Follower节点
- Observer节点
- CP架构

特性:
- 服务注册
- 配置管理
- 分布式锁
- Leader选举

优点:
- 成熟稳定
- 强一致性
- 功能丰富

缺点:
- 重,复杂
- 临时节点会导致注册频繁
- 运维成本高

适用场景:
- Hadoop生态
- 已有ZK集群
```

### 实现示例(Spring Cloud + Nacos)

#### 服务注册
```java
// application.yml
spring:
  application:
    name: order-service
  cloud:
    nacos:
      discovery:
        server-addr: localhost:8848
        namespace: dev
        group: DEFAULT_GROUP
        metadata:
          version: 1.0.0
          region: cn-east-1

// 启动类
@SpringBootApplication
@EnableDiscoveryClient
public class OrderServiceApplication {
    public static void main(String[] args) {
        SpringApplication.run(OrderServiceApplication.class, args);
    }
}
```

#### 服务发现
```java
@Service
public class OrderService {
    @Autowired
    private DiscoveryClient discoveryClient;

    @Autowired
    private RestTemplate restTemplate;

    public List<ServiceInstance> getInventoryServiceInstances() {
        return discoveryClient.getInstances("inventory-service");
    }

    public String callInventoryService() {
        // 使用负载均衡
        ServiceInstance instance = loadBalancer.choose("inventory-service");
        String url = instance.getUri().toString() + "/inventory/check";

        return restTemplate.getForObject(url, String.class);
    }
}

// 使用Feign(自动集成负载均衡)
@FeignClient(name = "inventory-service")
public interface InventoryClient {
    @GetMapping("/inventory/check")
    String checkInventory();
}
```

## 负载均衡

### 负载均衡策略

#### 轮询(Round Robin)
```
特点:
- 依次分发请求
- 简单公平

适用:
- 服务器性能相近
- 无状态服务
```

#### 加权轮询(Weighted Round Robin)
```
特点:
- 根据权重分配流量
- 权重高的实例获得更多请求

适用:
- 服务器性能不均
- 灰度发布
```

#### 最少连接(Least Connections)
```
特点:
- 选择当前连接数最少的服务器
- 动态调整

适用:
- 长连接场景
- 请求处理时间差异大
```

#### 一致性哈希(Consistent Hash)
```
特点:
- 根据请求特征(如用户ID)哈希
- 同一特征请求路由到同一服务器

适用:
- 有状态服务
- 缓存场景
```

#### 随机(Random)
```
特点:
- 随机选择服务器
- 简单

适用:
- 无特殊要求场景
```

### 客户端负载均衡

#### Spring Cloud LoadBalancer
```java
// 配置
@Configuration
public class LoadBalancerConfig {
    @Bean
    ReactorLoadBalancer<ServiceInstance> randomLoadBalancer(
            Environment environment,
            LoadBalancerClientFactory factory) {
        String serviceId = environment.getProperty(LoadBalancerClientFactory.PROPERTY_NAME);
        return new RandomLoadBalancer(
            factory.getLazyProvider(serviceId, ServiceInstanceListSupplier.class),
            serviceId
        );
    }
}

// 自定义负载均衡策略
public class CustomLoadBalancer implements ReactorServiceInstanceLoadBalancer {
    @Override
    public Mono<Response<ServiceInstance>> choose(Request request) {
        ServiceInstanceListSupplier supplier = serviceInstanceListSupplierProvider
            .getIfAvailable(NoopServiceInstanceListSupplier::new);

        return supplier.get()
            .next()
            .map(instances -> {
                // 自定义选择逻辑
                ServiceInstance instance = selectInstance(instances);
                return new DefaultResponse(instance);
            });
    }

    private ServiceInstance selectInstance(List<ServiceInstance> instances) {
        // 根据实例权重、响应时间等选择
        // ...
    }
}
```

### 服务端负载均衡

#### Nginx
```nginx
upstream backend {
    # 加权轮询
    server backend1.example.com weight=5;
    server backend2.example.com weight=3;
    server backend3.example.com backup;

    # 健康检查
    server backend4.example.com max_fails=3 fail_timeout=30s;

    # 一致性哈希
    hash $request_uri consistent;
}

server {
    location / {
        proxy_pass http://backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## 熔断与降级

### 熔断器模式(Circuit Breaker)

#### 状态机
```
关闭状态(Closed):
- 正常调用
- 监控失败率

打开状态(Open):
- 快速失败,不调用下游
- 等待超时后进入半开

半开状态(Half-Open):
- 允许少量请求
- 测试下游是否恢复
- 成功则关闭,失败则打开
```

#### 实现示例(Resilience4j)
```java
// 配置
@Bean
public CircuitBreakerConfig circuitBreakerConfig() {
    return CircuitBreakerConfig.custom()
        .failureRateThreshold(50) // 失败率阈值50%
        .waitDurationInOpenState(Duration.ofMillis(1000)) // 开启状态等待时间
        .permittedNumberOfCallsInHalfOpenState(2) // 半开状态允许的调用次数
        .slidingWindowSize(10) // 滑动窗口大小
        .slidingWindowType(SlidingWindowType.COUNT_BASED)
        .build();
}

// 使用
@Service
public class OrderService {
    private final CircuitBreaker circuitBreaker;

    public OrderService(CircuitBreakerRegistry registry) {
        this.circuitBreaker = registry.circuitBreaker("inventoryService");
    }

    public InventoryResponse checkInventory(String productId) {
        return circuitBreaker.executeSupplier(() -> {
            return inventoryClient.checkInventory(productId);
        });
    }

    // 带降级方法
    public InventoryResponse checkInventoryWithFallback(String productId) {
        return circuitBreaker.executeSupplier(
            () -> inventoryClient.checkInventory(productId),
            () -> fallbackCheckInventory(productId)
        );
    }

    private InventoryResponse fallbackCheckInventory(String productId) {
        // 降级逻辑: 返回默认值或从缓存读取
        return InventoryResponse.defaultResponse();
    }
}
```

#### 配置详解
```yaml
resilience4j:
  circuitbreaker:
    configs:
      default:
        failureRateThreshold: 50
        waitDurationInOpenState: 1000
        slidingWindowSize: 10
        slidingWindowType: COUNT_BASED
        permittedNumberOfCallsInHalfOpenState: 2
        minimumNumberOfCalls: 5
        recordExceptions:
          - java.io.IOException
          - java.net.SocketTimeoutException
        ignoreExceptions:
          - com.example.BusinessException
    instances:
      inventoryService:
        baseConfig: default
        failureRateThreshold: 60
      paymentService:
        baseConfig: default
        waitDurationInOpenState: 5000
```

### 降级策略

#### 返回默认值
```java
public Product getProduct(String productId) {
    return circuitBreaker.executeSupplier(
        () -> productClient.getProduct(productId),
        () -> Product.defaultProduct() // 返回默认商品
    );
}
```

#### 返回缓存数据
```java
public Product getProduct(String productId) {
    return circuitBreaker.executeSupplier(
        () -> {
            Product product = productClient.getProduct(productId);
            cache.put(productId, product);
            return product;
        },
        () -> cache.get(productId) // 降级返回缓存
    );
}
```

#### 返回空数据
```java
public List<Order> getUserOrders(String userId) {
    return circuitBreaker.executeSupplier(
        () -> orderClient.getUserOrders(userId),
        () -> Collections.emptyList() // 降级返回空列表
    );
}
```

#### 页面降级
```java
@Controller
public class PageController {
    @GetMapping("/product/{id}")
    public String getProductPage(@PathVariable String id, Model model) {
        try {
            Product product = productService.getProduct(id);
            model.addAttribute("product", product);
            return "product-detail";
        } catch (Exception e) {
            // 降级到静态页面
            return "product-unavailable";
        }
    }
}
```

## 限流控制

### 限流算法

#### 固定窗口计数器(Fixed Window)
```
原理:
- 将时间划分为固定窗口
- 每个窗口统计请求数
- 超过阈值则拒绝

优点:
- 实现简单
- 内存占用小

缺点:
- 临界时刻可能超限(突刺现象)
- 不够平滑

实现:
if (counter.increment(windowKey) > limit) {
    reject();
}
```

#### 滑动窗口(Sliding Window)
```
原理:
- 将窗口细分为多个小格
- 滑动统计最近N个小格
- 平滑限流

优点:
- 平滑限流
- 避免突刺

缺点:
- 实现复杂
- 内存占用较大

实现(Redis):
current_window = timestamp / granularity
count = sum(redis.get(window_i)) for i in [current_window - window_size, current_window]
if count > limit:
    reject()
```

#### 令牌桶(Token Bucket)
```
原理:
- 以固定速率生成令牌放入桶
- 桶有最大容量
- 请求获取令牌,获取不到则拒绝

优点:
- 允许突发流量
- 平滑限流

缺点:
- 需要维护令牌生成

实现(Guava RateLimiter):
RateLimiter rateLimiter = RateLimiter.create(100); // 100 QPS
if (rateLimiter.tryAcquire()) {
    process();
} else {
    reject();
}
```

#### 漏桶(Leaky Bucket)
```
原理:
- 请求进入桶
- 桶以固定速率流出
- 桶满则拒绝

优点:
- 平滑流量
- 保护下游

缺点:
- 不允许突发流量

实现:
if (queue.offer(request)) {
    // 入队成功
} else {
    // 队列满,拒绝
}
```

### 限流实现

#### 基于Redis + Lua
```java
// Lua脚本
String script =
    "local key = KEYS[1] " +
    "local limit = tonumber(ARGV[1]) " +
    "local window = tonumber(ARGV[2]) " +
    "local current = redis.call('INCR', key) " +
    "if current == 1 then " +
    "    redis.call('EXPIRE', key, window) " +
    "end " +
    "return current <= limit";

// 使用
public boolean allowRequest(String key, int limit, int window) {
    DefaultRedisScript<Boolean> redisScript = new DefaultRedisScript<>(script, Boolean.class);
    return redisTemplate.execute(
        redisScript,
        Collections.singletonList(key),
        String.valueOf(limit),
        String.valueOf(window)
    );
}
```

#### Spring Cloud Gateway限流
```yaml
spring:
  cloud:
    gateway:
      routes:
        - id: order-service
          uri: lb://order-service
          predicates:
            - Path=/orders/**
          filters:
            - name: RequestRateLimiter
              args:
                redis-rate-limiter.replenishRate: 10 # 每秒生成令牌数
                redis-rate-limiter.burstCapacity: 20 # 桶容量
                key-resolver: "#{@userKeyResolver}"

// KeyResolver
@Bean
public KeyResolver userKeyResolver() {
    return exchange -> Mono.just(
        exchange.getRequest().getHeaders().getFirst("X-User-Id")
    );
}
```

#### Sentinel限流
```java
// 配置
FlowRule rule = new FlowRule();
rule.setResource("createOrder");
rule.setGrade(RuleConstant.FLOW_GRADE_QPS);
rule.setCount(100); // 100 QPS
FlowRuleManager.loadRules(Collections.singletonList(rule));

// 使用
public Order createOrder(OrderRequest request) {
    try (Entry entry = SphU.entry("createOrder")) {
        // 业务逻辑
        return orderService.create(request);
    } catch (BlockException e) {
        // 被限流
        throw new RateLimitException("请求过于频繁");
    }
}

// 注解方式
@SentinelResource(value = "createOrder", blockHandler = "handleBlock")
public Order createOrder(OrderRequest request) {
    return orderService.create(request);
}

public Order handleBlock(OrderRequest request, BlockException e) {
    // 限流降级逻辑
    throw new RateLimitException("请求过于频繁");
}
```

### 分布式限流

#### Redis + Lua实现分布式令牌桶
```lua
-- distributed_rate_limiter.lua
local key = KEYS[1]
local permits = tonumber(ARGV[1]) -- 请求数量
local max_burst = tonumber(ARGV[2]) -- 最大突发
local rate = tonumber(ARGV[3]) -- 速率
local now = tonumber(ARGV[4])

local info = redis.call("HMGET", key, "tokens", "last_refill")
local tokens = tonumber(info[1])
local last_refill = tonumber(info[2])

if tokens == nil then
    tokens = max_burst
    last_refill = now
end

-- 计算新令牌
local interval = now - last_refill
local new_tokens = interval * rate
tokens = math.min(max_burst, tokens + new_tokens)

-- 检查是否足够
if tokens < permits then
    return 0 -- 拒绝
end

-- 扣减令牌
tokens = tokens - permits
redis.call("HMSET", key, "tokens", tokens, "last_refill", now)
redis.call("EXPIRE", key, math.ceil(max_burst / rate) + 1)

return 1 -- 允许
```

```java
public class DistributedRateLimiter {
    private RedisTemplate<String, String> redisTemplate;
    private String script;

    public boolean acquire(String key, int permits, int maxBurst, double rate) {
        DefaultRedisScript<Long> redisScript = new DefaultRedisScript<>(script, Long.class);
        Long result = redisTemplate.execute(
            redisScript,
            Collections.singletonList(key),
            String.valueOf(permits),
            String.valueOf(maxBurst),
            String.valueOf(rate),
            String.valueOf(System.currentTimeMillis())
        );
        return result != null && result == 1;
    }
}
```

## 超时控制

### 超时设置原则
```
连接超时(Connection Timeout):
- 建立连接的超时时间
- 建议: 1-3秒

读取超时(Read Timeout):
- 等待响应的超时时间
- 建议: 根据业务RT设置,通常3-10秒

写超时(Write Timeout):
- 发送数据的超时时间
- 建议: 1-3秒
```

### 实现示例

#### RestTemplate
```java
@Bean
public RestTemplate restTemplate() {
    HttpComponentsClientHttpRequestFactory factory =
        new HttpComponentsClientHttpRequestFactory();
    factory.setConnectTimeout(3000); // 连接超时3秒
    factory.setReadTimeout(5000);    // 读取超时5秒
    return new RestTemplate(factory);
}
```

#### Feign
```yaml
feign:
  client:
    config:
      default:
        connectTimeout: 3000
        readTimeout: 5000
      inventory-service:
        connectTimeout: 2000
        readTimeout: 3000
```

#### OkHttp
```java
@Bean
public OkHttpClient okHttpClient() {
    return new OkHttpClient.Builder()
        .connectTimeout(3, TimeUnit.SECONDS)
        .readTimeout(5, TimeUnit.SECONDS)
        .writeTimeout(3, TimeUnit.SECONDS)
        .retryOnConnectionFailure(true)
        .build();
}
```

## 重试机制

### 重试策略

#### 固定间隔重试
```java
@Retryable(
    value = {RemoteServiceException.class},
    maxAttempts = 3,
    backoff = @Backoff(delay = 1000) // 固定1秒
)
public Product getProduct(String productId) {
    return productClient.getProduct(productId);
}
```

#### 指数退避重试
```java
@Retryable(
    value = {RemoteServiceException.class},
    maxAttempts = 3,
    backoff = @Backoff(delay = 1000, multiplier = 2) // 1s, 2s, 4s
)
public Product getProduct(String productId) {
    return productClient.getProduct(productId);
}
```

### 重试注意事项
```
必须条件:
- 幂等性: 重试必须保证操作幂等
- 可重试异常: 只对可恢复异常重试
- 最大重试次数: 避免无限重试
- 退避策略: 避免重试风暴

不适合重试的场景:
- 非幂等操作(如扣款)
- 业务异常(如余额不足)
- 资源不存在(404)
```

## 服务容错

### 舱壁模式(Bulkhead)
```
原理:
- 隔离资源,防止故障扩散
- 为每个服务分配独立资源池

实现(Resilience4j):
@Bean
public BulkheadConfig bulkheadConfig() {
    return BulkheadConfig.custom()
        .maxConcurrentCalls(10) // 最大并发数
        .maxWaitDuration(Duration.ofMillis(500)) // 等待时间
        .build();
}

@Service
public class OrderService {
    @Bulkhead(name = "inventoryService", fallbackMethod = "fallback")
    public InventoryResponse checkInventory(String productId) {
        return inventoryClient.checkInventory(productId);
    }

    public InventoryResponse fallback(String productId) {
        return InventoryResponse.defaultResponse();
    }
}
```

### 故障隔离
```
线程池隔离:
- 每个服务使用独立线程池
- 故障不会影响其他服务

信号量隔离:
- 共享线程池,使用信号量限制并发
- 轻量级,适合内部调用

选择:
- 网络调用: 线程池隔离
- 本地调用: 信号量隔离
```

## 灰度发布

### 基于权重的灰度
```yaml
spring:
  cloud:
    nacos:
      discovery:
        metadata:
          version: v2
          weight: 20 # 20%流量
```

### 基于Header的灰度
```java
@Configuration
public class GrayLoadBalancerConfig {
    @Bean
    ReactorLoadBalancer<ServiceInstance> grayLoadBalancer(
            Environment environment,
            LoadBalancerClientFactory factory) {
        return new GrayLoadBalancer(
            factory.getLazyProvider(environment.getProperty(LoadBalancerClientFactory.PROPERTY_NAME), ServiceInstanceListSupplier.class),
            environment.getProperty(LoadBalancerClientFactory.PROPERTY_NAME)
        );
    }
}

public class GrayLoadBalancer implements ReactorServiceInstanceLoadBalancer {
    @Override
    public Mono<Response<ServiceInstance>> choose(Request request) {
        DefaultRequestContext context = (DefaultRequestContext) request.getContext();
        HttpHeaders headers = (HttpHeaders) context.getClientRequest().getHeaders();

        String version = headers.getFirst("X-Service-Version");

        return serviceInstanceListSupplierProvider.getIfAvailable()
            .get()
            .next()
            .map(instances -> {
                List<ServiceInstance> filtered = instances.stream()
                    .filter(instance -> version == null ||
                            version.equals(instance.getMetadata().get("version")))
                    .collect(Collectors.toList());

                if (filtered.isEmpty()) {
                    filtered = instances;
                }

                ServiceInstance instance = selectInstance(filtered);
                return new DefaultResponse(instance);
            });
    }
}
```

## 服务监控

### 健康检查

#### Spring Boot Actuator
```yaml
management:
  endpoints:
    web:
      exposure:
        include: health,info,metrics
  endpoint:
    health:
      show-details: always
```

#### 自定义健康检查
```java
@Component
public class InventoryServiceHealthIndicator implements HealthIndicator {
    @Autowired
    private InventoryClient inventoryClient;

    @Override
    public Health health() {
        try {
            HealthStatus status = inventoryClient.checkHealth();
            if (status.isHealthy()) {
                return Health.up()
                    .withDetail("inventory-service", "available")
                    .build();
            } else {
                return Health.down()
                    .withDetail("inventory-service", "unavailable")
                    .build();
            }
        } catch (Exception e) {
            return Health.down(e).build();
        }
    }
}
```

### 指标采集

#### Prometheus + Micrometer
```java
// 依赖
implementation 'io.micrometer:micrometer-registry-prometheus'

// 自定义指标
@Service
public class OrderService {
    private final Counter orderCounter;
    private final Timer orderTimer;

    public OrderService(MeterRegistry registry) {
        this.orderCounter = Counter.builder("order.count")
            .description("Total order count")
            .tag("type", "normal")
            .register(registry);

        this.orderTimer = Timer.builder("order.latency")
            .description("Order processing latency")
            .register(registry);
    }

    public Order createOrder(OrderRequest request) {
        return orderTimer.record(() -> {
            Order order = // 业务逻辑
            orderCounter.increment();
            return order;
        });
    }
}
```

```yaml
# application.yml
management:
  endpoints:
    web:
      exposure:
        include: prometheus
  metrics:
    tags:
      application: ${spring.application.name}
    export:
      prometheus:
        enabled: true
```

## 服务网格治理

### Istio流量管理

#### 虚拟服务
```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: order-service
spec:
  hosts:
    - order-service
  http:
    - match:
        - headers:
            x-user-type:
              exact: vip
      route:
        - destination:
            host: order-service
            subset: v2
          weight: 100
    - route:
        - destination:
            host: order-service
            subset: v1
          weight: 90
        - destination:
            host: order-service
            subset: v2
          weight: 10
```

#### 目标规则
```yaml
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: order-service
spec:
  host: order-service
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
      http:
        h2UpgradePolicy: UPGRADE
        http1MaxPendingRequests: 100
        http2MaxRequests: 1000
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
  subsets:
    - name: v1
      labels:
        version: v1
    - name: v2
      labels:
        version: v2
```

### Envoy过滤器
```yaml
apiVersion: networking.istio.io/v1alpha3
kind: EnvoyFilter
metadata:
  name: custom-filter
spec:
  workloadLabels:
    app: order-service
  filters:
    - filterName: envoy.lua
      filterType: HTTP
      filterConfig:
        inline_code: |
          function envoy_on_request(request_handle)
            -- 自定义逻辑
          end
```

## 最佳实践

### 1. 服务治理分层
```
基础设施层:
- Kubernetes Service
- Istio Service Mesh

应用层:
- Spring Cloud
- Dubbo

混合治理:
- 基础设施层 + 应用层
- 渐进式演进
```

### 2. 容错设计原则
```
快速失败(Fail Fast):
- 及时返回错误
- 避免资源占用

优雅降级(Graceful Degradation):
- 提供有损服务
- 保证核心功能

自我保护:
- 限流熔断
- 资源隔离

自我恢复:
- 自动重试
- 熔断器自动恢复
```

### 3. 监控告警
```
关键指标:
- 服务可用性(99.9%+)
- 响应时间(P99 < 500ms)
- 错误率(< 0.1%)
- QPS

告警级别:
- P0: 服务不可用(短信+电话)
- P1: 性能下降(短信)
- P2: 异常趋势(邮件)
```

### 4. 演进策略
```
阶段一: 基础治理
- 服务注册发现
- 负载均衡
- 健康检查

阶段二: 容错治理
- 熔断降级
- 限流控制
- 重试机制

阶段三: 流量治理
- 灰度发布
- 流量镜像
- A/B测试

阶段四: 智能治理
- 自适应限流
- 智能路由
- AIOps
```

## 参考资源

### 开源框架
- Spring Cloud: https://spring.io/projects/spring-cloud
- Dubbo: https://dubbo.apache.org/
- Sentinel: https://sentinelguard.io/
- Resilience4j: https://resilience4j.readme.io/
- Istio: https://istio.io/

### 学习资料
- 《微服务设计》
- 《Release It!》
- Google SRE Book
- Netflix技术博客
