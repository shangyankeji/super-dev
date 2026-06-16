---
id: configuration-management
title: 配置管理完全指南
domain: architecture
category: configuration-management.md
difficulty: intermediate
tags: [architecture, configuration, management, 参考资源, 核心需求, 概述, 选型建议, 配置中心对比]
quality_score: 70
last_updated: 2026-06-15
---
# 配置管理完全指南

## 概述

配置管理是微服务架构中的关键基础设施,负责集中化管理应用配置,实现配置的动态更新、版本管理、环境隔离和审计追踪。优秀的配置管理方案能够显著提升运维效率,降低配置错误风险,支持应用的快速迭代和灵活部署。

## 核心需求

### 1. 集中化管理
- 统一配置存储
- 集中控制与分发
- 减少配置碎片化

### 2. 动态更新
- 配置热更新
- 无需重启应用
- 实时生效

### 3. 环境隔离
- 开发/测试/生产环境
- 多数据中心
- 多租户隔离

### 4. 版本管理
- 配置历史记录
- 版本回滚
- 变更审计

### 5. 安全性
- 敏感信息加密
- 访问权限控制
- 操作审计日志

## 配置中心对比

### Nacos

#### 架构
```
Server端:
- Nacos Server集群(Raft协议)
- 配置存储(MySQL/嵌入式数据库)
- 长轮询推送

Client端:
- SDK集成
- 配置监听
- 本地缓存
```

#### 核心特性
```
优点:
- 配置管理+服务发现一体化
- 动态配置推送(秒级)
- 多环境多命名空间
- 配置回滚
- 灰度发布
- 阿里开源,生产验证
- Spring Cloud Alibaba集成
- 中文文档完善

缺点:
- 性能不如Apollo(大规模场景)
- 社区生态相对较小
- 企业版收费

适用场景:
- Spring Cloud生态
- 中小型系统
- 国内环境
- 配置+服务发现一体化需求
```

#### 实现示例

##### 服务端部署
```yaml
# docker-compose.yml
version: '3.8'
services:
  nacos-mysql:
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: root
      MYSQL_DATABASE: nacos
    volumes:
      - ./mysql-data:/var/lib/mysql

  nacos:
    image: nacos/nacos-server:latest
    environment:
      MODE: standalone
      SPRING_DATASOURCE_PLATFORM: mysql
      MYSQL_SERVICE_HOST: nacos-mysql
      MYSQL_SERVICE_PORT: 3306
      MYSQL_SERVICE_DB_NAME: nacos
      MYSQL_SERVICE_USER: root
      MYSQL_SERVICE_PASSWORD: root
    ports:
      - "8848:8848"
    depends_on:
      - nacos-mysql
```

##### 客户端集成
```xml
<!-- Maven依赖 -->
<dependency>
    <groupId>com.alibaba.cloud</groupId>
    <artifactId>spring-cloud-starter-alibaba-nacos-config</artifactId>
</dependency>
```

```yaml
# bootstrap.yml
spring:
  application:
    name: order-service
  profiles:
    active: dev
  cloud:
    nacos:
      config:
        server-addr: localhost:8848
        namespace: dev
        group: DEFAULT_GROUP
        file-extension: yaml
        shared-configs:
          - data-id: common.yaml
            group: DEFAULT_GROUP
            refresh: true
        extension-configs:
          - data-id: redis.yaml
            group: DEFAULT_GROUP
            refresh: true
          - data-id: mysql.yaml
            group: DEFAULT_GROUP
            refresh: true
        refresh-enabled: true
```

```java
// 配置类
@RefreshScope
@Configuration
public class OrderConfig {
    @Value("${order.timeout:3000}")
    private int timeout;

    @Value("${order.max-retry:3}")
    private int maxRetry;

    // 配置变更时自动刷新
    public int getTimeout() {
        return timeout;
    }
}

// 配置监听
@Component
public class ConfigListener {
    @NacosConfigListener(dataId = "order-service.yaml", groupId = "DEFAULT_GROUP")
    public void onConfigChange(String newConfig) {
        log.info("配置变更: {}", newConfig);
        // 自定义处理逻辑
    }
}
```

### Apollo

#### 架构
```
核心组件:
- Config Service: 配置读取、推送
- Admin Service: 配置管理
- Meta Server: 服务注册发现
- Portal: 管理界面
- Client: SDK

数据存储:
- MySQL(配置、审计日志)
- Eureka(服务发现)
```

#### 核心特性
```
优点:
- 统一配置管理平台
- 多环境多集群
- 灰度发布
- 配置版本管理
- 权限管理
- 审计日志
- 高性能(支持10万+客户端)
-携程开源,大规模生产验证

缺点:
- 架构复杂,部署成本高
- 学习曲线陡峭
- 运维成本高

适用场景:
- 大规模微服务(100+服务)
- 需要完善管理界面
- 强审计需求
- 企业级应用
```

#### 实现示例

##### 服务端部署
```yaml
# docker-compose.yml
version: '3.8'
services:
  apollo-mysql:
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: root
    volumes:
      - ./sql:/docker-entrypoint-initdb.d
      - ./mysql-data:/var/lib/mysql

  apollo-configservice:
    image: apolloconfig/apollo-configservice:latest
    environment:
      SPRING_DATASOURCE_URL: jdbc:mysql://apollo-mysql:3306/ApolloConfigDB
      SPRING_DATASOURCE_USERNAME: root
      SPRING_DATASOURCE_PASSWORD: root
    ports:
      - "8080:8080"
    depends_on:
      - apollo-mysql

  apollo-adminservice:
    image: apolloconfig/apollo-adminservice:latest
    environment:
      SPRING_DATASOURCE_URL: jdbc:mysql://apollo-mysql:3306/ApolloConfigDB
      SPRING_DATASOURCE_USERNAME: root
      SPRING_DATASOURCE_PASSWORD: root
    ports:
      - "8090:8090"
    depends_on:
      - apollo-mysql

  apollo-portal:
    image: apolloconfig/apollo-portal:latest
    environment:
      SPRING_DATASOURCE_URL: jdbc:mysql://apollo-mysql:3306/ApolloPortalDB
      SPRING_DATASOURCE_USERNAME: root
      SPRING_DATASOURCE_PASSWORD: root
      APOLLO_PORTAL_ENVS: dev,pro
      DEV_META: http://apollo-configservice:8080
      PRO_META: http://apollo-configservice-pro:8080
    ports:
      - "8070:8070"
    depends_on:
      - apollo-mysql
```

##### 客户端集成
```xml
<!-- Maven依赖 -->
<dependency>
    <groupId>com.ctrip.framework.apollo</groupId>
    <artifactId>apollo-client</artifactId>
    <version>2.1.0</version>
</dependency>
```

```properties
# application.properties
app.id=order-service
apollo.meta=http://localhost:8080
apollo.bootstrap.enabled=true
apollo.bootstrap.namespaces=application
apollo.autoUpdateInjectedSpringProperties=true
```

```java
// 配置类
@Configuration
@EnableApolloConfig
public class AppConfig {
    @Bean
    @RefreshScope
    public OrderConfig orderConfig() {
        return new OrderConfig();
    }
}

@Component
public class OrderConfig {
    @Value("${order.timeout:3000}")
    private int timeout;

    @ApolloConfigChangeListener
    public void onChange(ConfigChangeEvent changeEvent) {
        // 配置变更监听
        for (String key : changeEvent.changedKeys()) {
            ConfigChange change = changeEvent.getChange(key);
            log.info("配置变更 - Key: {}, Old: {}, New: {}",
                key, change.getOldValue(), change.getNewValue());
        }
    }
}

// 手动获取配置
@Service
public class OrderService {
    @ApolloConfig
    private Config config;

    public void processOrder() {
        Integer timeout = config.getIntProperty("order.timeout", 3000);
        // 使用配置
    }
}
```

### Consul

#### 核心特性
```
优点:
- 配置管理+服务发现一体化
- KV存储简单易用
- 多数据中心支持
- DNS接口
- 健康检查
- Go实现,性能好

缺点:
- 配置管理功能相对简单
- 缺乏版本管理
- 无Web管理界面(需第三方)

适用场景:
- 异构技术栈
- 多数据中心
- 简单配置需求
```

#### 实现示例

##### 服务端部署
```yaml
# docker-compose.yml
version: '3.8'
services:
  consul:
    image: consul:latest
    ports:
      - "8500:8500"   # HTTP API
      - "8600:8600/udp" # DNS
    command: agent -server -bootstrap-expect=1 -ui -client=0.0.0.0
```

##### 客户端集成
```xml
<!-- Maven依赖 -->
<dependency>
    <groupId>org.springframework.cloud</groupId>
    <artifactId>spring-cloud-starter-consul-config</artifactId>
</dependency>
```

```yaml
# bootstrap.yml
spring:
  application:
    name: order-service
  cloud:
    consul:
      host: localhost
      port: 8500
      config:
        enabled: true
        format: YAML
        prefix: config
        default-context: application
        profile-separator: ','
        data-key: data
        watch:
          enabled: true
          delay: 1000
```

```java
// 配置类
@RefreshScope
@Configuration
@ConfigurationProperties(prefix = "order")
public class OrderConfig {
    private int timeout = 3000;
    private int maxRetry = 3;

    // getters and setters
}
```

### Spring Cloud Config

#### 核心特性
```
优点:
- Spring Cloud原生
- Git版本管理
- 简单易用
- 与Spring生态无缝集成

缺点:
- 需要重启应用才能刷新配置(需配合Bus)
- 无管理界面
- 功能相对简单

适用场景:
- Spring Cloud生态
- 小型系统
- Git版本管理需求
```

#### 实现示例

##### 服务端
```xml
<!-- Maven依赖 -->
<dependency>
    <groupId>org.springframework.cloud</groupId>
    <artifactId>spring-cloud-config-server</artifactId>
</dependency>
```

```java
@SpringBootApplication
@EnableConfigServer
public class ConfigServerApplication {
    public static void main(String[] args) {
        SpringApplication.run(ConfigServerApplication.class, args);
    }
}
```

```yaml
# application.yml
server:
  port: 8888

spring:
  cloud:
    config:
      server:
        git:
          uri: https://github.com/myorg/config-repo
          search-paths:
            - '{application}'
          username: ${GIT_USERNAME}
          password: ${GIT_PASSWORD}
        encrypt:
          enabled: true
```

##### 客户端
```xml
<!-- Maven依赖 -->
<dependency>
    <groupId>org.springframework.cloud</groupId>
    <artifactId>spring-cloud-starter-config</artifactId>
</dependency>
<dependency>
    <groupId>org.springframework.cloud</groupId>
    <artifactId>spring-cloud-starter-bus-amqp</artifactId>
</dependency>
```

```yaml
# bootstrap.yml
spring:
  application:
    name: order-service
  profiles:
    active: dev
  cloud:
    config:
      uri: http://localhost:8888
      fail-fast: true
      retry:
        initial-interval: 1000
        max-interval: 2000
        max-attempts: 6
```

```java
@RefreshScope
@RestController
public class OrderController {
    @Value("${order.timeout:3000}")
    private int timeout;

    @GetMapping("/timeout")
    public int getTimeout() {
        return timeout;
    }
}
```

### Etcd

#### 核心特性
```
优点:
- 高性能KV存储
- 强一致性(Raft协议)
- Watch机制
- Kubernetes原生支持
- Go实现

缺点:
- 配置管理功能简单
- 无Web界面
- 需要自行实现高级功能

适用场景:
- Kubernetes环境
- Go技术栈
- 简单配置需求
```

#### 实现示例

```java
// Java客户端
public class EtcdConfigClient {
    private final Client client;

    public EtcdConfigClient(String endpoints) {
        this.client = Client.builder()
            .endpoints(endpoints.split(","))
            .build();
    }

    public String getConfig(String key) throws Exception {
        GetResponse response = client.getKVClient()
            .get(ByteSequence.from(key, StandardCharsets.UTF_8))
            .get();

        if (response.getKvs().isEmpty()) {
            return null;
        }

        return response.getKvs().get(0).getValue().toString(StandardCharsets.UTF_8);
    }

    public void watchConfig(String key, Consumer<String> listener) {
        Watch.Watcher watcher = client.getWatchClient()
            .watch(ByteSequence.from(key, StandardCharsets.UTF_8),
                WatchOption.DEFAULT,
                response -> {
                    for (WatchEvent event : response.getEvents()) {
                        String value = event.getKeyValue()
                            .getValue()
                            .toString(StandardCharsets.UTF_8);
                        listener.accept(value);
                    }
                });
    }
}
```

## 配置管理最佳实践

### 1. 配置分类

#### 按环境分类
```
开发环境(DEV):
- 本地开发配置
- 宽松的限流熔断
- 详细的日志

测试环境(TEST):
- 集成测试配置
- Mock外部服务
- 测试数据

预发环境(STAGING):
- 生产配置副本
- 真实外部服务(测试环境)
- 性能测试

生产环境(PROD):
- 生产配置
- 严格的限流熔断
- 关键日志
```

#### 按类型分类
```
基础设施配置:
- 数据库连接
- Redis连接
- 消息队列
- 日志配置

业务配置:
- 功能开关
- 业务规则
- 参数阈值

运营配置:
- 限流策略
- 熔断策略
- 降级策略
```

### 2. 配置结构设计

#### Nacos配置结构
```
命名空间(Namespace):
- dev: 开发环境
- test: 测试环境
- prod: 生产环境

分组(Group):
- DEFAULT_GROUP: 默认分组
- DATABASE_GROUP: 数据库配置
- REDIS_GROUP: Redis配置
- BUSINESS_GROUP: 业务配置

Data ID:
- order-service.yaml: 订单服务配置
- order-service-dev.yaml: 订单服务开发环境配置
- common.yaml: 公共配置
```

#### Apollo配置结构
```
AppId:
- order-service
- inventory-service
- payment-service

环境(Env):
- DEV
- UAT
- PRO

集群(Cluster):
- default: 默认集群
- shanghai: 上海集群
- beijing: 北京集群

命名空间(Namespace):
- application: 默认命名空间
- database.yaml: 数据库配置
- redis.yaml: Redis配置
```

### 3. 配置模板

#### application.yaml模板
```yaml
# 基础配置
server:
  port: 8080

spring:
  application:
    name: ${APP_NAME}
  profiles:
    active: ${ACTIVE_PROFILE:dev}

# 日志配置
logging:
  level:
    root: INFO
    com.example: DEBUG
  file:
    name: /var/log/${spring.application.name}/${spring.application.name}.log
  pattern:
    file: "%d{yyyy-MM-dd HH:mm:ss} [%thread] %-5level %logger{36} - %msg%n"

# 监控配置
management:
  endpoints:
    web:
      exposure:
        include: health,info,metrics,prometheus
  metrics:
    tags:
      application: ${spring.application.name}
      environment: ${spring.profiles.active}

# 业务配置(从配置中心读取)
order:
  timeout: ${ORDER_TIMEOUT:3000}
  max-retry: ${ORDER_MAX_RETRY:3}
  enable-cache: ${ORDER_ENABLE_CACHE:true}
```

#### database.yaml模板
```yaml
spring:
  datasource:
    url: jdbc:mysql://${DB_HOST:localhost}:${DB_PORT:3306}/${DB_NAME:order_db}?useUnicode=true&characterEncoding=utf8&serverTimezone=Asia/Shanghai
    username: ${DB_USERNAME:root}
    password: ${DB_PASSWORD:root}
    driver-class-name: com.mysql.cj.jdbc.Driver
    hikari:
      minimum-idle: 5
      maximum-pool-size: 20
      idle-timeout: 600000
      max-lifetime: 1800000
      connection-timeout: 30000
      pool-name: ${spring.application.name}-HikariCP
```

### 4. 敏感配置加密

#### Jasypt加密
```xml
<!-- Maven依赖 -->
<dependency>
    <groupId>com.github.ulisesbocchio</groupId>
    <artifactId>jasypt-spring-boot-starter</artifactId>
    <version>3.0.5</version>
</dependency>
```

```yaml
# application.yml
jasypt:
  encryptor:
    password: ${JASYPT_ENCRYPTOR_PASSWORD}
    algorithm: PBEWithMD5AndDES

spring:
  datasource:
    username: root
    password: ENC(加密后的密码)
```

```java
// 使用
@SpringBootApplication
@EnableEncryptableProperties
public class Application {
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}
```

#### Nacos加密
```java
// 自定义配置解密
@Component
public class DecryptConfigListener {
    @NacosConfigListener(dataId = "database.yaml")
    public void onReceive(String config) {
        String decrypted = decryptConfig(config);
        // 更新配置
    }

    private String decryptConfig(String encrypted) {
        // 解密逻辑
        return AES.decrypt(encrypted, secretKey);
    }
}
```

### 5. 配置热更新

#### @RefreshScope
```java
@RefreshScope
@Configuration
@ConfigurationProperties(prefix = "rate.limit")
public class RateLimitConfig {
    private int qps = 100;
    private int burst = 200;

    // getters and setters
}

@RefreshScope
@Service
public class OrderService {
    @Value("${order.timeout}")
    private int timeout;

    public void processOrder() {
        // 使用最新配置
    }
}
```

#### 配置变更监听
```java
@Component
@Slf4j
public class ConfigChangeHandler {
    @ApolloConfigChangeListener
    public void handleConfigChange(ConfigChangeEvent event) {
        for (String key : event.changedKeys()) {
            ConfigChange change = event.getChange(key);

            log.info("配置变更: {} - {} -> {}",
                key,
                change.getOldValue(),
                change.getNewValue());

            // 根据配置类型处理
            if (key.startsWith("rate.limit.")) {
                handleRateLimitChange(key, change);
            } else if (key.startsWith("feature.toggle.")) {
                handleFeatureToggleChange(key, change);
            }
        }
    }

    private void handleRateLimitChange(String key, ConfigChange change) {
        // 更新限流配置
        rateLimiter.updateConfig(key, change.getNewValue());
    }

    private void handleFeatureToggleChange(String key, ConfigChange change) {
        // 更新功能开关
        featureToggle.update(key, Boolean.parseBoolean(change.getNewValue()));
    }
}
```

### 6. 配置版本管理

#### Git集成(Spring Cloud Config)
```bash
# 配置仓库结构
config-repo/
├── application.yml          # 公共配置
├── application-dev.yml      # 开发环境
├── application-prod.yml     # 生产环境
├── order-service.yml        # 订单服务配置
├── order-service-dev.yml
├── order-service-prod.yml
└── database.yml             # 数据库配置
```

#### Apollo版本管理
```sql
-- 查询配置历史
SELECT
    NamespaceName,
    Key,
    Value,
    Comment,
    DataChange_CreatedBy,
    DataChange_CreatedTime,
    DataChange_LastModifiedBy,
    DataChange_LastTime
FROM Item
WHERE AppId = 'order-service'
  AND NamespaceName = 'application'
ORDER BY DataChange_LastTime DESC;

-- 回滚配置
-- 通过Portal界面操作
```

### 7. 灰度发布配置

#### Nacos灰度发布
```yaml
# order-service.yaml(主配置)
order:
  feature:
    new-algorithm: false

# order-service-gray.yaml(灰度配置)
order:
  feature:
    new-algorithm: true

# 应用配置
spring:
  cloud:
    nacos:
      config:
        shared-configs:
          - data-id: order-service.yaml
          - data-id: order-service-gray.yaml
            refresh: true
```

#### Apollo灰度发布
```java
// 通过Apollo Portal配置灰度规则
// 1. 创建灰度配置
// 2. 配置灰度规则(IP、AppId、标签)
// 3. 发布灰度配置
// 4. 监控灰度效果
// 5. 全量发布或回滚
```

### 8. 配置审计

#### 审计日志
```java
@Component
public class ConfigAuditLogger {
    @Autowired
    private AuditLogRepository auditLogRepository;

    @ApolloConfigChangeListener
    public void auditConfigChange(ConfigChangeEvent event) {
        for (String key : event.changedKeys()) {
            ConfigChange change = event.getChange(key);

            AuditLog log = new AuditLog();
            log.setAppId("order-service");
            log.setConfigKey(key);
            log.setOldValue(change.getOldValue());
            log.setNewValue(change.getNewValue());
            log.setOperator(getCurrentUser());
            log.setOperationTime(LocalDateTime.now());
            log.setClientIp(getClientIp());

            auditLogRepository.save(log);
        }
    }
}
```

## 配置中心高可用

### 1. 集群部署

#### Nacos集群
```yaml
# nacos-cluster.conf
node1:8848
node2:8848
node3:8848

# application.properties
nacos.inetutils.ip-address=节点IP
```

#### Apollo集群
```yaml
# 多机房部署
apollo-configservice-sh: 上海
apollo-configservice-bj: 北京

# 客户端配置
apollo.meta=http://configservice-sh:8080,http://configservice-bj:8080
```

### 2. 数据库高可用

```yaml
# MySQL主从
spring:
  datasource:
    master:
      url: jdbc:mysql://master:3306/nacos
      username: root
      password: root
    slave:
      url: jdbc:mysql://slave:3306/nacos
      username: root
      password: root
```

### 3. 客户端容错

```java
// 本地缓存
@Configuration
public class ConfigCacheConfig {
    @Bean
    public ConfigCache configCache() {
        return new LocalFileConfigCache();
    }
}

public class LocalFileConfigCache {
    private static final String CACHE_DIR = "/var/cache/config/";

    public void save(String key, String value) {
        String filePath = CACHE_DIR + key + ".cache";
        Files.writeString(Path.of(filePath), value);
    }

    public String load(String key) {
        String filePath = CACHE_DIR + key + ".cache";
        if (Files.exists(Path.of(filePath))) {
            return Files.readString(Path.of(filePath));
        }
        return null;
    }
}
```

## 配置管理工具

### 1. 配置校验

```java
@Component
public class ConfigValidator {
    @PostConstruct
    public void validateConfig() {
        validateDatabaseConfig();
        validateRedisConfig();
        validateBusinessConfig();
    }

    private void validateDatabaseConfig() {
        String url = environment.getProperty("spring.datasource.url");
        if (url == null || url.isEmpty()) {
            throw new IllegalStateException("数据库URL未配置");
        }

        int maxPoolSize = environment.getProperty(
            "spring.datasource.hikari.maximum-pool-size",
            Integer.class,
            20
        );
        if (maxPoolSize < 1 || maxPoolSize > 100) {
            throw new IllegalStateException(
                "数据库连接池大小配置错误: " + maxPoolSize
            );
        }
    }
}
```

### 2. 配置迁移工具

```java
public class ConfigMigrationTool {
    public void migrateFromPropertiesToYaml() {
        // 读取.properties文件
        Properties props = loadProperties("application.properties");

        // 转换为YAML
        Map<String, Object> configMap = new HashMap<>();
        props.forEach((key, value) -> {
            String[] keys = key.toString().split("\\.");
            Map<String, Object> current = configMap;
            for (int i = 0; i < keys.length - 1; i++) {
                current = (Map<String, Object>)
                    current.computeIfAbsent(keys[i], k -> new HashMap<>());
            }
            current.put(keys[keys.length - 1], value);
        });

        // 写入YAML文件
        Yaml yaml = new Yaml();
        yaml.dump(configMap, new FileWriter("application.yaml"));
    }
}
```

### 3. 配置对比工具

```java
public class ConfigDiffTool {
    public List<ConfigDiff> diff(String env1, String env2) {
        Map<String, String> config1 = loadConfig(env1);
        Map<String, String> config2 = loadConfig(env2);

        List<ConfigDiff> diffs = new ArrayList<>();

        // 查找新增配置
        config2.forEach((key, value) -> {
            if (!config1.containsKey(key)) {
                diffs.add(new ConfigDiff(key, null, value, "ADDED"));
            }
        });

        // 查找删除配置
        config1.forEach((key, value) -> {
            if (!config2.containsKey(key)) {
                diffs.add(new ConfigDiff(key, value, null, "DELETED"));
            }
        });

        // 查找修改配置
        config1.forEach((key, value1) -> {
            String value2 = config2.get(key);
            if (value2 != null && !value1.equals(value2)) {
                diffs.add(new ConfigDiff(key, value1, value2, "MODIFIED"));
            }
        });

        return diffs;
    }
}
```

## 选型建议

### 场景对比
```
Nacos:
- Spring Cloud Alibaba生态
- 配置管理+服务发现一体化
- 中小型系统(10-100服务)
- 国内环境

Apollo:
- 大规模微服务(100+服务)
- 需要完善管理界面
- 强审计需求
- 企业级应用

Consul:
- 异构技术栈
- 多数据中心
- 简单配置需求
- Go生态

Spring Cloud Config:
- Spring Cloud生态
- Git版本管理需求
- 小型系统
- 简单场景

Etcd:
- Kubernetes环境
- Go技术栈
- 简单配置需求
```

## 参考资源

### 官方文档
- Nacos: https://nacos.io/
- Apollo: https://www.apolloconfig.com/
- Consul: https://www.consul.io/
- Spring Cloud Config: https://spring.io/projects/spring-cloud-config
- Etcd: https://etcd.io/

### 最佳实践
- 《微服务配置管理》
- 12-Factor App配置原则
- Spring Cloud配置管理指南
