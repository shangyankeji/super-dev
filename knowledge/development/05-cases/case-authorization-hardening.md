---
id: case-authorization-hardening
title: 案例研究：权限体系加固——从越权漏洞到零信任授权
domain: development
category: 05-cases
difficulty: intermediate
tags: [agent, authorization, case, checklist, development, hardening, 元数据]
quality_score: 70
last_updated: 2026-06-15
---
# 案例研究：权限体系加固——从越权漏洞到零信任授权

## 元数据

| 字段 | 值 |
|------|------|
| 行业 | 企业级 HR SaaS 平台 |
| 系统规模 | 企业客户 5,000+，终端用户 200 万 |
| 技术栈 | Java Spring Boot + PostgreSQL + Redis |
| 团队规模 | 后端 22 人，安全 3 人 |
| 加固周期 | 10 周（2024-02 至 2024-04） |
| 核心目标 | 消除 IDOR/越权漏洞，建立细粒度权限模型 |

---

## 一、背景

### 1.1 业务概述

某 HR SaaS 平台提供薪资管理、考勤、招聘、绩效等功能模块。数据敏感度极高：

- 薪资数据：员工工资、奖金、社保明细
- 人事数据：身份证号、银行卡号、家庭住址
- 考勤数据：打卡记录、请假审批
- 绩效数据：评分、晋升建议

### 1.2 触发事件

2024 年 1 月外部渗透测试发现 **12 个 IDOR（Insecure Direct Object Reference）漏洞**：

| 编号 | 漏洞 | 严重性 | 说明 |
|------|------|--------|------|
| V-01 | 越权查看他人薪资 | Critical | 修改 URL 中的 employee_id 可查看任意员工薪资 |
| V-02 | 越权下载工资条 | Critical | 工资条 PDF 的 URL 可预测，无鉴权 |
| V-03 | 跨租户数据访问 | Critical | 修改请求中的 company_id 可访问其他企业数据 |
| V-04 | 越权审批请假 | High | 非直属上级可审批任意员工的请假申请 |
| V-05 | 越权修改考勤 | High | HR 角色可修改非本部门员工的考勤记录 |
| V-06-12 | 其他 IDOR | Medium | 涉及绩效、招聘、通知等模块 |

### 1.3 现有权限模型

```
当前模型：简单 RBAC（Role-Based Access Control）

角色：
├── super_admin    → 全部权限
├── company_admin  → 企业内全部权限
├── hr_manager     → HR 模块权限
├── department_mgr → 部门管理权限
└── employee       → 个人数据权限

问题：
1. 角色粒度太粗：hr_manager 能访问所有员工数据，无部门隔离
2. 无资源归属校验：接口只检查"角色是否有权限"，不检查"该资源是否属于该用户"
3. 无租户隔离层：租户 ID 由前端传递，后端不做强制校验
4. 权限硬编码：权限逻辑散落在 80+ 个 Controller 中
```

---

## 二、挑战

### 2.1 技术挑战

1. **存量代码庞大**：120 个 API 端点需要逐一加固，改动面广
2. **权限逻辑复杂**：HR 场景的权限关系多维——角色 x 部门 x 数据范围 x 操作类型
3. **性能约束**：权限校验不能显著增加接口延迟（P99 增加 < 20ms）
4. **数据隔离**：5,000+ 企业客户共享数据库实例，租户隔离必须无漏洞

### 2.2 业务挑战

1. **不能停服**：改造期间平台正常运营
2. **向下兼容**：现有客户的权限配置不能丢失
3. **灵活性**：不同企业客户有不同的权限需求（有的部门间可见，有的不可见）

---

## 三、方案设计

### 3.1 权限模型升级：RBAC → ABAC + RBAC 混合

```
新模型：RBAC（角色）+ ABAC（属性）+ 资源归属校验

三层权限校验：
Layer 1: 租户隔离（Tenant Isolation）
  → 强制校验当前用户的 tenant_id 与目标资源的 tenant_id
  → 在数据库查询层面自动注入 tenant_id 条件

Layer 2: 角色权限（RBAC）
  → 校验用户角色是否有目标操作的权限
  → 支持细粒度操作：read/write/approve/export/delete

Layer 3: 资源归属（ABAC / Ownership）
  → 校验用户与目标资源的关系
  → 基于属性：部门归属、直属关系、数据范围策略
```

### 3.2 权限策略引擎

引入轻量级策略引擎，将权限规则从代码中抽离：

```java
// 权限策略定义（YAML 配置）
policies:
  - name: salary-read
    resource: "salary"
    action: "read"
    rules:
      - role: "employee"
        condition: "resource.employee_id == subject.employee_id"
        # 员工只能看自己的薪资

      - role: "hr_manager"
        condition: "resource.department_id IN subject.managed_departments"
        # HR 经理只能看所管辖部门的薪资

      - role: "company_admin"
        condition: "resource.company_id == subject.company_id"
        # 企业管理员可看本企业所有薪资

  - name: leave-approve
    resource: "leave_request"
    action: "approve"
    rules:
      - role: "department_mgr"
        condition: >
          resource.applicant.direct_manager_id == subject.employee_id
          OR resource.applicant.department_id IN subject.managed_departments
        # 直属上级或部门经理可审批
```

### 3.3 技术实现架构

```
请求流程：
Client → API Gateway → Auth Filter → Controller → Service → DB

Auth Filter 处理流程：
1. 提取 JWT 中的 user_id + tenant_id
2. 从 Redis 缓存加载用户权限上下文（角色 + 部门 + 管辖范围）
3. Layer 1: 验证 tenant_id（注入到所有 DB 查询）
4. Layer 2: 匹配 RBAC 角色权限
5. Layer 3: 评估 ABAC 策略（资源归属校验）
```

#### 租户隔离实现

```java
// MyBatis 拦截器：自动注入 tenant_id 条件
@Intercepts({
    @Signature(type = Executor.class, method = "query", args = {
        MappedStatement.class, Object.class, RowBounds.class, ResultHandler.class
    })
})
public class TenantInterceptor implements Interceptor {

    @Override
    public Object intercept(Invocation invocation) throws Throwable {
        MappedStatement ms = (MappedStatement) invocation.getArgs()[0];
        BoundSql boundSql = ms.getBoundSql(invocation.getArgs()[1]);

        String originalSql = boundSql.getSql();
        Long tenantId = TenantContext.getCurrentTenantId();

        if (tenantId == null) {
            throw new SecurityException("Tenant context is missing");
        }

        // 自动追加 tenant_id 条件
        String newSql = addTenantCondition(originalSql, tenantId);
        // ... 反射替换 SQL
        return invocation.proceed();
    }
}
```

#### 资源归属校验注解

```java
// 声明式资源归属校验
@RestController
@RequestMapping("/api/v1/salary")
public class SalaryController {

    @GetMapping("/{employeeId}")
    @RequirePermission(resource = "salary", action = "read")
    @OwnershipCheck(
        resourceType = "employee",
        resourceIdParam = "employeeId",
        rules = {
            @Rule(role = "employee", condition = "self"),
            @Rule(role = "hr_manager", condition = "managed_department"),
            @Rule(role = "company_admin", condition = "same_tenant")
        }
    )
    public ResponseEntity<SalaryDTO> getSalary(
            @PathVariable Long employeeId,
            @AuthUser UserContext user) {
        return ResponseEntity.ok(salaryService.getByEmployeeId(employeeId));
    }
}
```

#### 权限上下文缓存

```java
// 用户权限上下文（Redis 缓存，TTL 5 分钟）
public class UserPermissionContext {
    private Long userId;
    private Long tenantId;
    private Long employeeId;
    private Set<String> roles;                    // ["hr_manager", "employee"]
    private Set<Long> managedDepartmentIds;       // 管辖部门 ID
    private Set<Long> directReportEmployeeIds;    // 直属下属 ID
    private Map<String, Set<String>> permissions; // resource -> [actions]

    // 缓存 Key: perm:user:{userId}
    // TTL: 5 分钟
    // 失效时机：角色变更/部门调整时主动清除
}
```

---

## 四、实施步骤

### 4.1 Phase 1：紧急修复（Week 1-2）

```
Week 1: 修复 12 个已知漏洞
  - 逐个接口添加资源归属校验
  - 热修复方式上线，每个修复独立 PR + Review

Week 2: 租户隔离加固
  - 上线 TenantInterceptor（MyBatis 拦截器）
  - 全量 SQL 审计：确认所有查询都经过拦截器
  - 补充 14 个被拦截器遗漏的原生 SQL 查询
```

### 4.2 Phase 2：权限框架搭建（Week 3-5）

```
Week 3: 权限模型设计
  - 梳理 120 个 API 端点的资源/操作/归属关系
  - 设计权限策略 YAML 配置格式
  - 实现策略引擎核心（规则解析 + 条件评估）

Week 4: 框架开发
  - 开发 @RequirePermission 注解 + AOP 处理器
  - 开发 @OwnershipCheck 注解 + 校验器
  - 开发 UserPermissionContext + Redis 缓存
  - 性能优化：批量预加载、布隆过滤器快速拒绝

Week 5: 框架测试
  - 单元测试：200+ 测试用例覆盖各种权限组合
  - 性能测试：权限校验增加延迟 < 8ms（P99）
```

### 4.3 Phase 3：全量接入（Week 6-8）

```
Week 6: 核心模块接入（薪资/人事/考勤）
  - 40 个 API 端点迁移到新权限框架
  - 双重校验期：新旧权限逻辑同时运行，记录差异日志

Week 7: 扩展模块接入（绩效/招聘/通知）
  - 50 个 API 端点迁移
  - 清理旧权限代码

Week 8: 剩余模块 + 全量验证
  - 30 个 API 端点迁移
  - 全量回归测试
  - 渗透测试验证
```

### 4.4 Phase 4：加固与审计（Week 9-10）

```
Week 9: 审计系统
  - 高风险操作审计日志：薪资查看/导出、人事变更、权限变更
  - 异常访问告警：短时间内频繁访问不同员工数据
  - 权限变更追踪：谁在什么时间修改了谁的角色

Week 10: 持续防护
  - 权限配置管理界面：企业管理员可自定义部门间可见性
  - 定期权限审查：每月自动生成权限审查报告
  - CI 集成：新 API 端点必须声明权限注解，否则构建失败
```

---

## 五、结果数据

### 5.1 安全指标

| 指标 | 加固前 | 加固后 |
|------|--------|--------|
| IDOR 漏洞数 | 12 | 0 |
| 越权风险端点 | 38（渗透测试发现） | 0 |
| 租户隔离漏洞 | 3 | 0 |
| 权限绕过路径 | 5 | 0 |
| 权限代码覆盖率 | 40%（散落在 Controller） | 100%（框架化） |

### 5.2 性能影响

| 指标 | 加固前 | 加固后 | 影响 |
|------|--------|--------|------|
| API P50 延迟 | 45ms | 48ms | +3ms |
| API P99 延迟 | 180ms | 188ms | +8ms |
| Redis 缓存命中率 | - | 94% | - |
| 权限校验耗时 P99 | - | 8ms | - |

### 5.3 工程指标

| 指标 | 加固前 | 加固后 |
|------|--------|--------|
| 权限相关代码行数 | 散落 8,000+ 行 | 框架 2,500 行 + 策略配置 800 行 |
| 新端点接入权限时间 | 2 小时（手写逻辑） | 10 分钟（声明注解） |
| 权限变更部署时间 | 需重新部署应用 | 热更新策略配置 |
| 权限审计日志覆盖 | 0% | 100%（高风险操作） |

### 5.4 业务影响

- 通过 ISO 27001 审计中的访问控制条款
- 2 家大型企业客户因权限管控能力提升而签署年度合同
- 客诉中"权限问题"类别从月均 8 次降到 1 次

---

## 六、经验教训

### 6.1 做对的事

1. **框架化而非逐个修补**：虽然 Phase 1 先做了紧急修复，但根本解决靠的是 Phase 2 的权限框架。如果只修补漏洞不建框架，未来每个新接口都可能引入新漏洞
2. **声明式优于命令式**：注解 + 策略配置的方式让权限逻辑一目了然，Code Review 时可直接看到每个接口的权限要求
3. **租户隔离下沉到数据层**：MyBatis 拦截器自动注入 tenant_id，比在每个 Service 方法中手动添加更可靠
4. **双重校验过渡期**：新旧权限同时运行 2 周，捕获了 7 处迁移 Bug
5. **CI 强制检查**：新接口没有权限注解就构建失败，从根源上杜绝了遗漏

### 6.2 做错的事

1. **初期低估了权限场景复杂度**：HR 场景的权限关系比预期复杂（例如：虚线汇报、兼职、代理审批），策略引擎设计时未充分考虑
2. **缓存失效策略不完善**：角色变更后缓存 5 分钟才过期，导致权限变更不即时生效。后来改为变更时主动清除缓存
3. **审计日志量低估**：全量审计导致日志量激增 10 倍，紧急调整为只审计高风险操作

### 6.3 关键认知

- 授权和认证必须分层设计：认证（你是谁）→ 角色权限（你能做什么）→ 资源归属（你能访问哪些数据）
- RBAC 不够用的时候不要硬扩角色数量，引入 ABAC 做属性级控制
- 权限改造不是一次性项目，需要建立持续的权限审查和漏洞检测机制
- 默认拒绝（deny by default）必须是权限框架的基本原则

---

## Agent Checklist

在 AI Agent 辅助执行权限体系加固时，应逐项确认：

- [ ] **权限清单**：是否梳理了所有 API 端点的资源/操作/归属关系
- [ ] **租户隔离**：多租户系统是否在数据层强制隔离（而非仅靠前端传参）
- [ ] **IDOR 检测**：是否对所有带资源 ID 参数的端点进行了越权测试
- [ ] **默认拒绝**：权限框架是否基于 deny-by-default 原则
- [ ] **角色粒度**：角色定义是否足够细，是否需要引入 ABAC 属性控制
- [ ] **资源归属**：是否有独立的资源归属校验层（不仅仅靠角色）
- [ ] **缓存策略**：权限缓存的 TTL 和主动失效机制是否合理
- [ ] **性能测试**：权限校验对接口延迟的影响是否在可接受范围
- [ ] **审计日志**：高风险操作是否记录了完整的审计日志
- [ ] **声明式注解**：新接口是否必须声明权限注解（CI 强制检查）
- [ ] **渗透验证**：加固后是否通过了渗透测试验证
- [ ] **向下兼容**：现有用户的权限配置在迁移后是否保持一致
- [ ] **持续监控**：是否建立了异常访问模式的自动告警
- [ ] **定期审查**：是否有定期的权限审查机制和报告
