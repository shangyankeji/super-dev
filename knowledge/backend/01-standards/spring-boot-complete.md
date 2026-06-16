---
id: spring-boot-complete
title: Spring Boot 完整指南
domain: backend
category: 01-standards
difficulty: intermediate
tags: [backend, boot, complete, data, rest, security, spring, 与依赖注入]
quality_score: 70
last_updated: 2026-06-15
---
# Spring Boot 完整指南

## 概述

Spring Boot 是基于 Spring Framework 的快速开发框架，通过自动配置和约定优于配置原则，大幅简化 Spring 应用的搭建和开发。Spring Boot 3.x 基于 Java 17+，支持 GraalVM 原生镜像、虚拟线程和 Jakarta EE 命名空间。

### 核心特性

- **自动配置**: 根据 classpath 自动配置 Bean
- **起步依赖**: spring-boot-starter-* 预定义依赖集
- **内嵌服务器**: Tomcat/Jetty/Undertow 内嵌运行
- **Actuator**: 生产就绪的监控和管理端点
- **Spring Security**: 企业级安全框架
- **Spring Data JPA**: 声明式数据访问
- **Spring Cloud**: 微服务基础设施

---

## IoC 与依赖注入

### Bean 定义与注入

```java
// 服务层 - 构造函数注入（推荐）
@Service
public class UserService {
    private final UserRepository userRepository;
    private final PasswordEncoder passwordEncoder;

    // Spring 自动注入（单构造函数可省略 @Autowired）
    public UserService(UserRepository userRepository, PasswordEncoder passwordEncoder) {
        this.userRepository = userRepository;
        this.passwordEncoder = passwordEncoder;
    }

    public User createUser(CreateUserRequest request) {
        if (userRepository.existsByEmail(request.email())) {
            throw new DuplicateEmailException(request.email());
        }
        var user = User.builder()
            .email(request.email())
            .password(passwordEncoder.encode(request.password()))
            .role(Role.USER)
            .build();
        return userRepository.save(user);
    }
}
```

### 配置类

```java
@Configuration
public class AppConfig {

    @Bean
    public PasswordEncoder passwordEncoder() {
        return new BCryptPasswordEncoder(12);
    }

    @Bean
    @Profile("production")
    public CacheManager cacheManager(RedisConnectionFactory factory) {
        return RedisCacheManager.builder(factory)
            .cacheDefaults(RedisCacheConfiguration.defaultCacheConfig()
                .entryTtl(Duration.ofMinutes(30))
                .serializeValuesWith(
                    SerializationPair.fromSerializer(new GenericJackson2JsonRedisSerializer())
                ))
            .build();
    }
}
```

---

## AOP (面向切面编程)

```java
@Aspect
@Component
@Slf4j
public class PerformanceAspect {

    @Around("@annotation(com.example.annotation.Timed)")
    public Object measureTime(ProceedingJoinPoint joinPoint) throws Throwable {
        String method = joinPoint.getSignature().toShortString();
        long start = System.nanoTime();
        try {
            return joinPoint.proceed();
        } finally {
            long duration = (System.nanoTime() - start) / 1_000_000;
            log.info("Method {} took {}ms", method, duration);
            if (duration > 500) {
                log.warn("Slow method detected: {} ({}ms)", method, duration);
            }
        }
    }
}

// 审计日志切面
@Aspect
@Component
public class AuditAspect {

    @AfterReturning(pointcut = "@annotation(auditable)", returning = "result")
    public void auditAction(JoinPoint joinPoint, Auditable auditable, Object result) {
        var auth = SecurityContextHolder.getContext().getAuthentication();
        String user = auth != null ? auth.getName() : "anonymous";
        log.info("AUDIT: user={} action={} resource={}", user, auditable.action(), auditable.resource());
    }
}
```

---

## Spring Security

### Security 配置

```java
@Configuration
@EnableWebSecurity
@EnableMethodSecurity
public class SecurityConfig {

    @Bean
    public SecurityFilterChain filterChain(HttpSecurity http, JwtAuthFilter jwtFilter) throws Exception {
        return http
            .csrf(csrf -> csrf.disable())
            .cors(cors -> cors.configurationSource(corsConfigurationSource()))
            .sessionManagement(sm -> sm.sessionCreationPolicy(STATELESS))
            .authorizeHttpRequests(auth -> auth
                .requestMatchers("/api/auth/**", "/actuator/health").permitAll()
                .requestMatchers("/api/admin/**").hasRole("ADMIN")
                .anyRequest().authenticated()
            )
            .addFilterBefore(jwtFilter, UsernamePasswordAuthenticationFilter.class)
            .exceptionHandling(ex -> ex
                .authenticationEntryPoint((req, res, e) ->
                    res.sendError(HttpServletResponse.SC_UNAUTHORIZED))
            )
            .build();
    }

    @Bean
    public CorsConfigurationSource corsConfigurationSource() {
        var config = new CorsConfiguration();
        config.setAllowedOrigins(List.of("https://app.example.com"));
        config.setAllowedMethods(List.of("GET", "POST", "PUT", "DELETE"));
        config.setAllowedHeaders(List.of("*"));
        config.setAllowCredentials(true);
        var source = new UrlBasedCorsConfigurationSource();
        source.registerCorsConfiguration("/api/**", config);
        return source;
    }
}
```

### JWT 过滤器

```java
@Component
@RequiredArgsConstructor
public class JwtAuthFilter extends OncePerRequestFilter {

    private final JwtService jwtService;
    private final UserDetailsService userDetailsService;

    @Override
    protected void doFilterInternal(HttpServletRequest request,
                                     HttpServletResponse response,
                                     FilterChain filterChain) throws ServletException, IOException {
        String header = request.getHeader("Authorization");
        if (header == null || !header.startsWith("Bearer ")) {
            filterChain.doFilter(request, response);
            return;
        }

        String token = header.substring(7);
        String username = jwtService.extractUsername(token);

        if (username != null && SecurityContextHolder.getContext().getAuthentication() == null) {
            UserDetails userDetails = userDetailsService.loadUserByUsername(username);
            if (jwtService.isTokenValid(token, userDetails)) {
                var authToken = new UsernamePasswordAuthenticationToken(
                    userDetails, null, userDetails.getAuthorities());
                authToken.setDetails(new WebAuthenticationDetailsSource().buildDetails(request));
                SecurityContextHolder.getContext().setAuthentication(authToken);
            }
        }
        filterChain.doFilter(request, response);
    }
}
```

---

## Spring Data JPA

### Repository

```java
public interface UserRepository extends JpaRepository<User, Long> {

    Optional<User> findByEmail(String email);

    boolean existsByEmail(String email);

    @Query("SELECT u FROM User u WHERE u.role = :role AND u.createdAt > :since")
    List<User> findRecentByRole(@Param("role") Role role, @Param("since") LocalDateTime since);

    @Query(value = "SELECT * FROM users WHERE LOWER(name) LIKE LOWER(CONCAT('%', :keyword, '%'))",
           nativeQuery = true)
    Page<User> searchByName(@Param("keyword") String keyword, Pageable pageable);
}
```

### Entity

```java
@Entity
@Table(name = "users", indexes = {
    @Index(name = "idx_user_email", columnList = "email", unique = true),
    @Index(name = "idx_user_role", columnList = "role"),
})
@Getter @Setter @Builder @NoArgsConstructor @AllArgsConstructor
public class User {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(nullable = false, unique = true, length = 255)
    private String email;

    @Column(nullable = false)
    @JsonIgnore
    private String password;

    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 20)
    private Role role;

    @CreationTimestamp
    @Column(updatable = false)
    private LocalDateTime createdAt;

    @UpdateTimestamp
    private LocalDateTime updatedAt;
}
```

---

## REST API 设计

```java
@RestController
@RequestMapping("/api/v1/users")
@RequiredArgsConstructor
@Validated
public class UserController {

    private final UserService userService;

    @GetMapping
    public Page<UserResponse> listUsers(
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "20") int size,
            @RequestParam(required = false) String search) {
        return userService.listUsers(search, PageRequest.of(page, size));
    }

    @GetMapping("/{id}")
    public UserResponse getUser(@PathVariable Long id) {
        return userService.getUser(id);
    }

    @PostMapping
    @ResponseStatus(HttpStatus.CREATED)
    public UserResponse createUser(@Valid @RequestBody CreateUserRequest request) {
        return userService.createUser(request);
    }

    @PutMapping("/{id}")
    public UserResponse updateUser(@PathVariable Long id,
                                    @Valid @RequestBody UpdateUserRequest request) {
        return userService.updateUser(id, request);
    }

    @DeleteMapping("/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    @PreAuthorize("hasRole('ADMIN')")
    public void deleteUser(@PathVariable Long id) {
        userService.deleteUser(id);
    }
}
```

### 全局异常处理

```java
@RestControllerAdvice
@Slf4j
public class GlobalExceptionHandler {

    @ExceptionHandler(ResourceNotFoundException.class)
    @ResponseStatus(HttpStatus.NOT_FOUND)
    public ErrorResponse handleNotFound(ResourceNotFoundException ex) {
        return new ErrorResponse("NOT_FOUND", ex.getMessage());
    }

    @ExceptionHandler(MethodArgumentNotValidException.class)
    @ResponseStatus(HttpStatus.BAD_REQUEST)
    public ErrorResponse handleValidation(MethodArgumentNotValidException ex) {
        var errors = ex.getBindingResult().getFieldErrors().stream()
            .collect(Collectors.toMap(FieldError::getField, FieldError::getDefaultMessage));
        return new ErrorResponse("VALIDATION_ERROR", "请求参数验证失败", errors);
    }

    @ExceptionHandler(Exception.class)
    @ResponseStatus(HttpStatus.INTERNAL_SERVER_ERROR)
    public ErrorResponse handleGeneric(Exception ex) {
        log.error("Unhandled exception", ex);
        return new ErrorResponse("INTERNAL_ERROR", "服务器内部错误");
    }
}

public record ErrorResponse(String code, String message, Map<String, String> details) {
    public ErrorResponse(String code, String message) {
        this(code, message, Map.of());
    }
}
```

---

## 微服务配置

### application.yml

```yaml
spring:
  application:
    name: user-service
  datasource:
    url: jdbc:postgresql://${DB_HOST:localhost}:5432/${DB_NAME:userdb}
    username: ${DB_USER:postgres}
    password: ${DB_PASSWORD:password}
    hikari:
      maximum-pool-size: 20
      minimum-idle: 5
      connection-timeout: 30000
  jpa:
    hibernate:
      ddl-auto: validate
    open-in-view: false
    properties:
      hibernate:
        default_batch_fetch_size: 16
        jdbc.batch_size: 50
  cache:
    type: redis
  redis:
    host: ${REDIS_HOST:localhost}
    port: 6379

management:
  endpoints:
    web:
      exposure:
        include: health, info, metrics, prometheus
  endpoint:
    health:
      show-details: when-authorized

server:
  port: 8080
  shutdown: graceful

logging:
  level:
    root: INFO
    com.example: DEBUG
    org.hibernate.SQL: DEBUG
```

---

## 部署

### Dockerfile

```dockerfile
FROM eclipse-temurin:21-jre-alpine
WORKDIR /app
COPY target/*.jar app.jar
RUN addgroup -S spring && adduser -S spring -G spring
USER spring
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s \
  CMD wget -qO- http://localhost:8080/actuator/health || exit 1
ENTRYPOINT ["java", "-XX:+UseZGC", "-XX:MaxRAMPercentage=75", "-jar", "app.jar"]
```

---

## 常见反模式

| 反模式 | 问题 | 正确做法 |
|--------|------|----------|
| 字段注入 @Autowired | 不可测试 | 构造函数注入 |
| open-in-view: true | 延迟查询泄漏到 Controller | 设为 false |
| 不设连接池限制 | 连接耗尽 | 配置 HikariCP 参数 |
| Entity 直接返回 | 序列化循环 / 信息泄露 | 用 DTO/Record 映射 |
| 不做分页 | 大数据量 OOM | 始终用 Pageable |
| 事务范围过大 | 锁竞争 | 最小化 @Transactional 范围 |

---

## Agent Checklist

- [ ] 使用构造函数注入，不使用字段注入
- [ ] spring.jpa.open-in-view 设为 false
- [ ] Entity 与 DTO/Record 分离，API 不直接返回 Entity
- [ ] 所有 API 实现分页（使用 Pageable）
- [ ] 配置全局异常处理（@RestControllerAdvice）
- [ ] Spring Security 配置为 stateless + JWT
- [ ] 敏感配置通过环境变量注入，不硬编码
- [ ] Actuator health/ready 端点启用并接入部署探针
- [ ] 数据库连接池参数明确配置
- [ ] 日志级别按环境区分（dev: DEBUG, prod: INFO）
- [ ] 使用 Flyway/Liquibase 管理数据库迁移
- [ ] 关键操作添加审计日志
