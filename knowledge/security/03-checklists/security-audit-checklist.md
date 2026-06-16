---
id: security-audit-checklist
title: 安全审计检查清单
domain: security
category: 03-checklists
difficulty: intermediate
tags: [audit, authentication, authorization, checklist, security, 依赖与供应链安全, 加密与密钥管理, 基础设施安全]
quality_score: 70
last_updated: 2026-06-15
---
# 安全审计检查清单

> 适用范围：Web 应用、API 服务、微服务架构、云原生基础设施
> 维护周期：每季度审查一次，重大安全事件后立即更新
> 严重级别标注：🔴 Critical | 🟠 High | 🟡 Medium | 🟢 Low

---

## 1. 认证安全 (Authentication)

### 1.1 OAuth 2.0 / OpenID Connect

- [ ] 🔴 Authorization Code + PKCE 流程用于公共客户端，禁止 Implicit 流程
- [ ] 🔴 Access Token 有效期不超过 15 分钟
- [ ] 🔴 Refresh Token 启用轮换（Rotation），使用后旧 Token 立即失效
- [ ] 🟠 Token 中不包含敏感业务数据（如密码、身份证号）
- [ ] 🟠 严格验证 redirect_uri，使用精确匹配而非通配符
- [ ] 🟠 state 参数绑定会话，防止 CSRF 攻击
- [ ] 🟡 配置 Token 撤销端点（Revocation Endpoint）
- [ ] 🟡 Authorization Server 支持 Token Introspection
- [ ] 🟢 客户端凭证（client_secret）存储于密钥管理系统，不硬编码

### 1.2 JWT (JSON Web Token)

- [ ] 🔴 验证签名算法，拒绝 `alg: none`
- [ ] 🔴 使用 RS256 或 ES256 非对称算法，避免 HS256 共享密钥
- [ ] 🔴 严格校验 `iss`、`aud`、`exp`、`nbf` 声明
- [ ] 🟠 JWT 密钥定期轮换（至少每 90 天）
- [ ] 🟠 敏感操作需要验证 JWT 中的 `jti`（唯一标识）防重放
- [ ] 🟡 JWK Set (JWKS) 端点启用缓存并设置合理 TTL
- [ ] 🟡 JWT Payload 不存储超过授权所需的最小信息
- [ ] 🟢 日志记录 JWT 验证失败事件，包含来源 IP

### 1.3 多因素认证 (MFA)

- [ ] 🔴 管理员账户强制启用 MFA
- [ ] 🔴 支持 TOTP（如 Google Authenticator）或硬件密钥（FIDO2/WebAuthn）
- [ ] 🟠 MFA 恢复码加密存储，仅生成一次并提示用户保存
- [ ] 🟠 MFA 验证失败次数限制（如连续 5 次失败锁定 15 分钟）
- [ ] 🟡 SMS 验证码作为备选而非主要 MFA 手段（SIM Swap 风险）
- [ ] 🟡 MFA 注册/解绑操作需要额外身份验证
- [ ] 🟢 记录所有 MFA 操作的审计日志

### 1.4 密码策略

- [ ] 🔴 密码使用 bcrypt / scrypt / Argon2 哈希存储，禁止 MD5/SHA1
- [ ] 🔴 密码最低 12 位，包含大小写字母、数字、特殊字符
- [ ] 🟠 检查密码是否在已知泄露数据库中（如 HaveIBeenPwned API）
- [ ] 🟠 登录失败锁定策略：5 次失败后锁定 15 分钟
- [ ] 🟡 禁止使用最近 10 次历史密码
- [ ] 🟡 密码重置链接一次性使用，有效期不超过 30 分钟
- [ ] 🟢 密码强度实时反馈（前端 zxcvbn 库）

---

## 2. 授权安全 (Authorization)

### 2.1 RBAC / ABAC

- [ ] 🔴 最小权限原则：默认拒绝，显式授权
- [ ] 🔴 角色权限矩阵文档化并定期审查（至少每季度）
- [ ] 🔴 水平越权检查：用户只能访问自己的资源
- [ ] 🔴 垂直越权检查：普通用户不能执行管理员操作
- [ ] 🟠 API 端点级别的权限控制，不仅依赖前端隐藏
- [ ] 🟠 敏感操作（删除、导出、批量修改）需要二次确认或审批流
- [ ] 🟡 服务间调用使用独立的 Service Account，权限范围最小化
- [ ] 🟡 权限变更记录审计日志
- [ ] 🟢 定期清理不活跃账户和过期权限

### 2.2 API 授权

- [ ] 🔴 所有 API 端点需要认证，公开端点显式标注白名单
- [ ] 🟠 API Rate Limiting：按用户/IP/API Key 限流
- [ ] 🟠 GraphQL 查询深度限制和复杂度分析
- [ ] 🟡 API 密钥区分环境（dev/staging/prod），禁止混用
- [ ] 🟡 Webhook 请求验证签名（如 HMAC-SHA256）
- [ ] 🟢 API 文档标注每个端点所需权限级别

---

## 3. 输入验证与注入防护

### 3.1 跨站脚本攻击 (XSS)

- [ ] 🔴 所有用户输入在输出时进行上下文相关编码（HTML/JS/URL/CSS）
- [ ] 🔴 配置 Content-Security-Policy (CSP) 头，禁止 `unsafe-inline`
- [ ] 🔴 富文本编辑器使用白名单过滤（如 DOMPurify）
- [ ] 🟠 Cookie 设置 HttpOnly 和 Secure 标志
- [ ] 🟠 使用框架自带的模板引擎自动转义（React JSX、Vue 模板）
- [ ] 🟡 X-XSS-Protection 和 X-Content-Type-Options 头配置
- [ ] 🟡 SVG 文件上传需要清洗内嵌脚本
- [ ] 🟢 定期使用自动化工具扫描 DOM XSS

### 3.2 SQL 注入 (SQLi)

- [ ] 🔴 全部使用参数化查询 / 预编译语句，禁止字符串拼接
- [ ] 🔴 ORM 框架中禁止使用 raw query，如必须使用需经安全评审
- [ ] 🟠 数据库账户最小权限：应用账户禁止 DROP/GRANT 权限
- [ ] 🟠 错误信息不暴露数据库结构（关闭详细错误页面）
- [ ] 🟡 存储过程中的动态 SQL 同样使用参数化
- [ ] 🟡 定期使用 SQLMap 等工具进行注入测试
- [ ] 🟢 数据库查询日志监控异常模式

### 3.3 服务端请求伪造 (SSRF)

- [ ] 🔴 URL 白名单机制：仅允许访问预定义的外部域名
- [ ] 🔴 禁止访问内网地址段（10.0.0.0/8、172.16.0.0/12、192.168.0.0/16）
- [ ] 🔴 禁止 `file://`、`gopher://`、`dict://` 等危险协议
- [ ] 🟠 DNS 重绑定防护：解析后验证 IP 地址
- [ ] 🟠 响应内容不直接回显给用户
- [ ] 🟡 云环境中禁止访问元数据服务（169.254.169.254）
- [ ] 🟢 请求超时设置不超过 10 秒

### 3.4 其他注入

- [ ] 🔴 命令注入：禁止将用户输入拼接到系统命令中
- [ ] 🔴 LDAP 注入：LDAP 查询参数转义
- [ ] 🟠 XML 外部实体（XXE）：禁用外部实体解析
- [ ] 🟠 路径遍历：文件操作验证并规范化路径，禁止 `../`
- [ ] 🟡 正则表达式拒绝服务（ReDoS）：避免灾难性回溯模式
- [ ] 🟡 模板注入（SSTI）：用户输入不进入模板引擎
- [ ] 🟢 日志注入：用户输入写入日志前转义换行符

---

## 4. 加密与密钥管理

### 4.1 传输加密 (TLS)

- [ ] 🔴 全站 HTTPS，HTTP 自动 301 重定向到 HTTPS
- [ ] 🔴 TLS 1.2 为最低版本，推荐 TLS 1.3
- [ ] 🔴 禁用 SSLv3、TLS 1.0、TLS 1.1
- [ ] 🟠 HSTS 头配置，max-age 至少 1 年，包含 subdomains
- [ ] 🟠 证书使用 2048-bit RSA 或 256-bit ECC
- [ ] 🟠 证书自动续期（如 Let's Encrypt + certbot）
- [ ] 🟡 配置 OCSP Stapling 加速证书验证
- [ ] 🟡 禁用弱密码套件（RC4、DES、3DES、NULL）
- [ ] 🟢 使用 SSL Labs 测试评分达到 A+ 级别

### 4.2 存储加密

- [ ] 🔴 敏感数据（PII、支付信息）数据库字段级加密（AES-256-GCM）
- [ ] 🔴 磁盘全盘加密（LUKS / BitLocker / AWS EBS 加密）
- [ ] 🟠 备份数据同样加密，密钥与主数据密钥分离
- [ ] 🟠 日志中脱敏处理（手机号、身份证号、银行卡号）
- [ ] 🟡 文件上传加密存储，下载时按需解密
- [ ] 🟡 数据库连接使用 TLS，验证服务端证书
- [ ] 🟢 临时文件处理后安全删除（覆写后删除）

### 4.3 密钥管理

- [ ] 🔴 密钥存储于专用 KMS（AWS KMS / HashiCorp Vault / Azure Key Vault）
- [ ] 🔴 代码仓库中不存储任何密钥、密码、Token（git-secrets 扫描）
- [ ] 🔴 密钥轮换策略：对称密钥每 90 天，非对称密钥每年
- [ ] 🟠 密钥访问控制：最小权限 + 审计日志
- [ ] 🟠 环境变量或密钥管理服务注入密钥，不使用配置文件
- [ ] 🟡 密钥泄露应急预案：泄露后 1 小时内完成轮换
- [ ] 🟡 加密算法迁移计划（为后量子密码学做准备）
- [ ] 🟢 密钥使用监控，检测异常访问模式

---

## 5. 依赖与供应链安全

### 5.1 软件成分分析 (SCA)

- [ ] 🔴 CI/CD 管道集成 SCA 工具（Snyk / Dependabot / Renovate）
- [ ] 🔴 已知高危漏洞（CVSS >= 9.0）阻断构建
- [ ] 🟠 依赖锁文件（package-lock.json / poetry.lock）提交到版本控制
- [ ] 🟠 定期更新依赖（至少每月检查一次）
- [ ] 🟡 评估间接依赖（transitive dependencies）的安全状态
- [ ] 🟡 使用私有制品仓库（Nexus / Artifactory）镜像公共包
- [ ] 🟢 记录所有第三方组件的许可证合规性

### 5.2 npm 生态安全

- [ ] 🔴 `npm audit` 集成到 CI，high/critical 级别阻断
- [ ] 🔴 锁定依赖版本，禁止 `*` 或 `latest` 版本号
- [ ] 🟠 检查 npm 包的维护状态（最后更新时间、下载量、maintainer 数）
- [ ] 🟠 启用 npm 2FA 用于发布操作
- [ ] 🟡 使用 `npm-shrinkwrap.json` 确保生产依赖精确锁定
- [ ] 🟡 监控依赖包的所有权转移（包被接管风险）
- [ ] 🟢 检查 postinstall 脚本是否包含恶意代码

### 5.3 Python 生态安全

- [ ] 🔴 `pip-audit` 或 `safety check` 集成到 CI
- [ ] 🔴 使用 `pip install --require-hashes` 验证包完整性
- [ ] 🟠 虚拟环境隔离（venv / conda），禁止系统级安装
- [ ] 🟠 PyPI 包名检查，防止 typosquatting 攻击
- [ ] 🟡 使用 `pip-compile` 生成确定性依赖列表
- [ ] 🟡 私有包使用 `--index-url` 指向内部 PyPI 镜像
- [ ] 🟢 检查 setup.py / pyproject.toml 中的构建脚本

### 5.4 容器镜像安全

- [ ] 🔴 使用官方基础镜像，标签锁定到具体版本（禁止 `latest`）
- [ ] 🔴 镜像扫描（Trivy / Grype / Snyk Container）集成到 CI
- [ ] 🟠 多阶段构建，最终镜像不包含构建工具和源码
- [ ] 🟠 非 root 用户运行容器进程
- [ ] 🟡 镜像签名（Cosign / Notary）
- [ ] 🟡 定期重建基础镜像以获取安全补丁
- [ ] 🟢 SBOM（Software Bill of Materials）生成并存档

---

## 6. 基础设施安全

### 6.1 网络安全

- [ ] 🔴 网络分段：数据库/缓存不暴露到公网
- [ ] 🔴 防火墙规则：默认拒绝，按需开放端口
- [ ] 🔴 管理端口（SSH 22 / RDP 3389）仅允许跳板机访问
- [ ] 🟠 VPC / VLAN 隔离不同环境（dev / staging / prod）
- [ ] 🟠 WAF（Web Application Firewall）部署并配置规则集
- [ ] 🟠 DDoS 防护（如 Cloudflare / AWS Shield）
- [ ] 🟡 DNS 安全：DNSSEC 启用，防止 DNS 劫持
- [ ] 🟡 出站流量监控，检测数据外泄
- [ ] 🟢 网络拓扑图文档化并定期更新

### 6.2 服务器安全

- [ ] 🔴 操作系统安全补丁及时更新（关键补丁 72 小时内）
- [ ] 🔴 SSH 密钥认证，禁用密码登录
- [ ] 🟠 服务进程使用最小权限用户运行
- [ ] 🟠 文件系统权限最小化（配置文件 640，可执行文件 750）
- [ ] 🟡 内核安全参数加固（sysctl 配置）
- [ ] 🟡 不必要的服务和端口关闭
- [ ] 🟢 CIS Benchmark 基线检查

### 6.3 日志与监控

- [ ] 🔴 集中化日志收集（ELK / Splunk / Datadog）
- [ ] 🔴 安全事件实时告警（认证失败、权限异常、异常流量）
- [ ] 🔴 日志不可篡改（写入后只读 / 签名验证）
- [ ] 🟠 日志保留期至少 180 天（合规要求可能更长）
- [ ] 🟠 日志中不记录敏感信息（密码、Token、信用卡号）
- [ ] 🟡 异常行为检测（UEBA）规则配置
- [ ] 🟡 定期审查日志告警规则有效性
- [ ] 🟢 日志访问控制：仅安全团队可查看完整日志

### 6.4 Kubernetes 安全

- [ ] 🔴 Pod Security Standards 配置（Restricted 级别）
- [ ] 🔴 RBAC 最小权限：Service Account 不使用 cluster-admin
- [ ] 🟠 Network Policy 限制 Pod 间通信
- [ ] 🟠 Secret 加密存储（etcd 加密 / External Secrets Operator）
- [ ] 🟡 Admission Controller（OPA Gatekeeper / Kyverno）策略
- [ ] 🟡 容器运行时安全（Falco / Sysdig）
- [ ] 🟢 Kubernetes 版本及时更新，跟踪 CVE

---

## 7. 合规与隐私

### 7.1 GDPR（通用数据保护条例）

- [ ] 🔴 用户数据处理有合法法律基础（同意 / 合同 / 正当利益）
- [ ] 🔴 隐私政策清晰告知数据收集目的、范围、保留期
- [ ] 🔴 数据主体权利实现：访问权、删除权、可携带权、更正权
- [ ] 🟠 数据处理协议（DPA）与所有第三方处理方签署
- [ ] 🟠 数据泄露 72 小时内通知监管机构
- [ ] 🟡 数据保护影响评估（DPIA）用于高风险处理活动
- [ ] 🟡 指定数据保护官（DPO）或等效负责人
- [ ] 🟢 Cookie Banner 合规（非必要 Cookie 需用户同意）

### 7.2 SOC 2

- [ ] 🔴 安全策略文档化并经管理层批准
- [ ] 🔴 访问控制：入职/离职/调岗的权限变更流程
- [ ] 🟠 变更管理：代码变更需要 PR 审批和 CI 通过
- [ ] 🟠 事件响应计划文档化并定期演练
- [ ] 🟡 供应商风险评估流程
- [ ] 🟡 业务连续性计划和灾难恢复演练
- [ ] 🟢 员工安全意识培训记录

### 7.3 等保（中国信息安全等级保护）

- [ ] 🔴 系统定级备案（二级/三级根据业务重要性）
- [ ] 🔴 三级系统双因素认证
- [ ] 🔴 安全审计日志保留不少于 180 天
- [ ] 🟠 网络安全区域划分（安全域隔离）
- [ ] 🟠 数据库审计系统部署
- [ ] 🟡 入侵检测/防御系统（IDS/IPS）部署
- [ ] 🟡 定期漏洞扫描和渗透测试（至少每年一次）
- [ ] 🟢 安全运维管理制度文档化

### 7.4 行业特定合规

- [ ] 🟠 支付业务：PCI DSS 合规（卡号不落地、令牌化）
- [ ] 🟠 医疗健康：HIPAA 合规（PHI 加密、审计跟踪）
- [ ] 🟡 金融行业：数据分类分级管理
- [ ] 🟡 跨境数据传输：标准合同条款（SCC）或充分性认定
- [ ] 🟢 未成年人数据保护：COPPA / 个人信息保护法儿童条款

---

## 8. 安全开发生命周期 (SDL)

### 8.1 安全设计

- [ ] 🔴 威胁建模（STRIDE / DREAD）在设计阶段完成
- [ ] 🟠 安全架构评审：新服务/新接口上线前
- [ ] 🟡 安全需求在 PRD 中明确标注
- [ ] 🟢 安全设计文档归档

### 8.2 安全编码

- [ ] 🔴 代码审查包含安全检查项
- [ ] 🔴 SAST 工具（SonarQube / Semgrep / CodeQL）集成到 CI
- [ ] 🟠 敏感数据处理代码由安全团队 Review
- [ ] 🟡 安全编码规范文档化并培训
- [ ] 🟢 代码中的安全注释（`// SECURITY: ...`）标记关键安全决策

### 8.3 安全测试

- [ ] 🔴 DAST 扫描（OWASP ZAP / Burp Suite）在 staging 环境运行
- [ ] 🔴 渗透测试：重大版本发布前执行
- [ ] 🟠 安全回归测试：修复后的漏洞添加自动化测试用例
- [ ] 🟠 模糊测试（Fuzzing）用于解析器和协议处理模块
- [ ] 🟡 API 安全测试：认证绕过、越权、注入
- [ ] 🟢 安全测试覆盖率跟踪

### 8.4 安全运营

- [ ] 🔴 安全事件响应流程（检测 -> 遏制 -> 根除 -> 恢复 -> 复盘）
- [ ] 🔴 漏洞管理：Critical 24h / High 72h / Medium 30d / Low 90d 修复 SLA
- [ ] 🟠 安全监控 7x24 值班或托管 SOC 服务
- [ ] 🟠 红队/蓝队定期对抗演练
- [ ] 🟡 Bug Bounty 计划或 VDP（漏洞披露政策）
- [ ] 🟢 安全度量指标定期汇报（MTTD / MTTR / 漏洞密度）

---

## Agent Checklist

- [ ] 安全审计清单已覆盖 8 个安全域
- [ ] 每个检查项均标注严重级别（Critical/High/Medium/Low）
- [ ] Checkbox 格式可直接用于审计跟踪
- [ ] 认证与授权章节覆盖 OAuth2、JWT、RBAC、MFA
- [ ] 输入验证章节覆盖 XSS、SQLi、SSRF 及其他注入类型
- [ ] 加密章节覆盖传输 TLS、存储 AES、密钥管理
- [ ] 依赖安全章节覆盖 SCA、npm audit、pip-audit、容器镜像
- [ ] 基础设施章节覆盖防火墙、网络隔离、日志审计、K8s 安全
- [ ] 合规章节覆盖 GDPR、SOC2、等保、行业特定合规
- [ ] SDL 章节覆盖安全设计、编码、测试、运营全生命周期
