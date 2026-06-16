---
id: security-review-checklist
title: 安全评审检查清单 (Security Review Checklist)
domain: development
category: 03-checklists
difficulty: intermediate
tags: [checklist, development, owasp, review, security, 依赖与供应链安全, 基础设施安全, 安全]
quality_score: 70
last_updated: 2026-06-15
---
# 安全评审检查清单 (Security Review Checklist)

> 适用场景：新功能安全评审、定期安全审计、第三方集成安全评估、合规检查。
> 通过标准：所有 Critical / High 项全部通过，Medium 项覆盖率 >= 90%，Low 项记录并排期修复。

---

## 1. 认证（Authentication）

### 1.1 身份验证机制

- [ ] **认证方案已确定** — 使用标准协议（OAuth 2.0 / OpenID Connect / SAML），非自研认证方案。
- [ ] **密码策略已实施** — 最小长度 >= 8 字符，要求包含大小写字母 + 数字 + 特殊字符，禁止常见弱密码。
- [ ] **密码存储安全** — 使用 bcrypt / scrypt / Argon2 哈希存储，cost factor >= 10，不使用 MD5 / SHA1。
- [ ] **多因素认证已支持** — 高权限账户强制 MFA（TOTP / WebAuthn / SMS），普通用户可选开启。
- [ ] **登录失败锁定已配置** — 连续失败 >= 5 次锁定账户 15 分钟，或启用渐进式延迟。
- [ ] **会话管理安全** — Session ID 随机生成（>= 128 bit），登录后重新生成，HttpOnly + Secure + SameSite 属性已设置。

### 1.2 Token 安全

- [ ] **JWT 签名算法安全** — 使用 RS256 / ES256，禁止 none / HS256（对称密钥场景除外）。
- [ ] **Token 有效期合理** — Access Token <= 15 分钟，Refresh Token <= 7 天，支持主动撤销。
- [ ] **Token 存储安全** — 前端存储于 HttpOnly Cookie 或内存中，不使用 localStorage。
- [ ] **Token 刷新机制安全** — Refresh Token 轮转（使用后失效），检测重放攻击时撤销整个会话。

---

## 2. 授权（Authorization）

- [ ] **权限模型已实施** — RBAC / ABAC 权限体系已落地，角色与权限映射已文档化。
- [ ] **最小权限原则** — 用户 / 服务账号仅拥有完成职能所需的最小权限集。
- [ ] **越权访问已防护** — 水平越权（访问同级别其他用户数据）和垂直越权（提升自身权限）均有检查。
- [ ] **资源级别权限校验** — 每次资源访问都验证当前用户是否有权操作该具体资源，不仅检查角色。
- [ ] **管理接口隔离** — Admin API 独立部署或网络隔离，不对公网暴露。
- [ ] **权限变更有审计** — 角色分配、权限调整操作记录审计日志，支持事后追溯。
- [ ] **服务间认证已实施** — 微服务间调用使用 mTLS / Service Mesh / API Key，不允许无认证内部调用。

---

## 3. 输入验证与注入防护

### 3.1 SQL 注入

- [ ] **参数化查询已全面使用** — 所有数据库操作使用 ORM 或参数化查询（Prepared Statement），无字符串拼接 SQL。
- [ ] **数据库账号权限最小化** — 应用使用的数据库账号无 DROP / GRANT / CREATE USER 权限。
- [ ] **动态排序/分页字段白名单** — ORDER BY / LIMIT 等动态字段通过白名单校验，不直接拼接用户输入。

### 3.2 XSS（跨站脚本）

- [ ] **输出编码已实施** — 所有动态内容输出时根据上下文进行 HTML / JS / URL / CSS 编码。
- [ ] **CSP 头已配置** — Content-Security-Policy 限制脚本来源，禁止 inline script（或使用 nonce）。
- [ ] **富文本输入已净化** — 使用 DOMPurify / bleach 等白名单净化库处理 HTML 输入。
- [ ] **Cookie 设置 HttpOnly** — 敏感 Cookie 不可被 JavaScript 访问。

### 3.3 其他注入

- [ ] **SSRF 防护已实施** — 用户可控 URL 经过内网地址 / 回环地址 / 元数据端点过滤。
- [ ] **命令注入已防护** — 避免直接调用 shell 命令，必须调用时使用参数数组而非字符串拼接。
- [ ] **路径遍历已防护** — 文件操作路径经过规范化处理，禁止 `../` 遍历。
- [ ] **XML 外部实体（XXE）已禁用** — XML 解析器禁用 DTD 和外部实体加载。
- [ ] **反序列化安全** — 不反序列化不可信数据，或使用安全的序列化格式（JSON 而非 Pickle / Java Serialization）。

### 3.4 通用输入校验

- [ ] **白名单优于黑名单** — 输入校验优先使用允许列表，拒绝不在列表中的输入。
- [ ] **文件上传已限制** — 文件类型通过 MIME + 魔数校验，大小有上限，存储路径不可被直接访问。
- [ ] **请求体大小已限制** — 配置最大请求体大小（如 10MB），防止大负载 DoS。

---

## 4. 数据保护

### 4.1 传输加密

- [ ] **TLS 1.2+ 已强制** — 禁用 TLS 1.0 / 1.1 / SSLv3，仅允许安全密码套件。
- [ ] **HSTS 已启用** — Strict-Transport-Security 头 max-age >= 31536000，包含 includeSubDomains。
- [ ] **内部通信加密** — 微服务间通信使用 mTLS 或加密通道，不传输明文敏感数据。

### 4.2 存储加密

- [ ] **敏感字段加密存储** — 密码、密钥、身份证号、银行卡号等使用 AES-256-GCM 或同等强度加密。
- [ ] **加密密钥独立管理** — 密钥存储于 KMS / Vault，与加密数据物理分离，支持密钥轮转。
- [ ] **备份数据已加密** — 数据库备份、日志归档均加密存储，访问需要独立授权。

### 4.3 敏感信息管理

- [ ] **密钥不在代码中** — 代码库中无硬编码的 API Key / Secret / 密码，CI/CD 使用密钥管理服务注入。
- [ ] **日志中无敏感信息** — 日志输出已脱敏，不包含密码、Token、PII 数据。
- [ ] **错误响应不泄露内部信息** — 生产环境错误响应不包含堆栈追踪、数据库表名、内部 IP。
- [ ] **Git 历史已清理** — 历史提交中无泄露的密钥 / 密码（使用 git-secrets / truffleHog 扫描）。

---

## 5. API 安全

- [ ] **速率限制已实施** — 按用户 / IP / 接口配置请求频率限制，超限返回 HTTP 429。
- [ ] **CORS 策略已配置** — Access-Control-Allow-Origin 明确指定允许的域名，不使用 `*`（公共 API 除外）。
- [ ] **CSRF 防护已实施** — 状态变更操作使用 CSRF Token 或 SameSite Cookie + Origin 头校验。
- [ ] **API 版本化已实施** — API 变更通过版本号管理（URL / Header），废弃版本有迁移期。
- [ ] **请求/响应 Schema 校验** — 使用 JSON Schema / Protobuf 校验请求格式，拒绝异常结构。
- [ ] **GraphQL 安全**（如适用）— 查询深度限制、复杂度限制、内省在生产环境禁用。

---

## 6. 依赖与供应链安全

- [ ] **依赖漏洞扫描已集成** — CI 中运行 Snyk / Trivy / npm audit / pip-audit，Critical / High 阻断发布。
- [ ] **依赖版本已锁定** — 使用 lock 文件（package-lock.json / poetry.lock / go.sum），构建可重现。
- [ ] **基础镜像安全** — Docker 基础镜像使用官方镜像 + 固定版本号，定期更新。
- [ ] **容器以非 root 运行** — Dockerfile 指定非 root 用户，不使用 `--privileged` 模式。
- [ ] **软件物料清单（SBOM）已生成** — 使用 Syft / CycloneDX 生成 SBOM，可追溯所有依赖来源。
- [ ] **第三方库许可证合规** — 无 GPL（如商业项目不兼容）或其他限制性许可证冲突。

---

## 7. 基础设施安全

- [ ] **网络分段已实施** — 数据库 / 缓存 / 内部服务不对公网暴露，通过私有子网 + 安全组隔离。
- [ ] **SSH / 远程访问受控** — 禁用密码登录，使用密钥认证 + 跳板机，会话有审计记录。
- [ ] **Kubernetes 安全策略** — PodSecurityPolicy / OPA / Kyverno 限制特权容器、hostNetwork、hostPID。
- [ ] **镜像签名与验证** — 容器镜像经过签名（Cosign / Notation），部署时验证签名完整性。
- [ ] **Secret 管理** — Kubernetes Secret 加密存储（etcd 加密），或使用 External Secrets Operator 对接 Vault。

---

## 8. 安全监控与响应

- [ ] **安全事件日志集中化** — 认证失败、授权拒绝、异常访问模式日志统一采集到 SIEM。
- [ ] **异常检测规则已配置** — 暴力破解、异地登录、大量 403/401 等模式有自动告警。
- [ ] **安全事件响应流程已定义** — 事件分级、通知链路、处置步骤、事后复盘流程已文档化。
- [ ] **渗透测试已完成** — 上线前完成内部或第三方渗透测试，高危发现已修复。
- [ ] **安全联系方式已公开** — security.txt 或安全响应邮箱已配置，外部可报告漏洞。

---

## 9. OWASP Top 10 对照

| OWASP 风险 | 对应检查项 | 状态 |
|-------------|-----------|------|
| A01: Broken Access Control | 第 2 节（授权）全部 | |
| A02: Cryptographic Failures | 第 4 节（数据保护）全部 | |
| A03: Injection | 第 3 节（输入验证）全部 | |
| A04: Insecure Design | 架构评审检查清单 | |
| A05: Security Misconfiguration | 第 7 节（基础设施安全）| |
| A06: Vulnerable Components | 第 6 节（依赖安全）全部 | |
| A07: Auth Failures | 第 1 节（认证）全部 | |
| A08: Software/Data Integrity | 第 6 节（SBOM + 镜像签名）| |
| A09: Logging/Monitoring Failures | 第 8 节（安全监控）全部 | |
| A10: SSRF | 3.3 节（SSRF 防护）| |

---

## 参考

- OWASP Top 10 (2021) — https://owasp.org/Top10/
- OWASP ASVS (Application Security Verification Standard) — https://owasp.org/www-project-application-security-verification-standard/
- CWE/SANS Top 25 — https://cwe.mitre.org/top25/
- NIST Cybersecurity Framework — https://www.nist.gov/cyberframework

---

## Agent Checklist

供 AI Agent 在执行安全评审时使用的自检项：

- [ ] 已逐项核对 OWASP Top 10 对照表，无遗漏风险类别。
- [ ] 已使用自动化工具扫描（Trivy / Snyk / Semgrep），非纯人工目检。
- [ ] 已检查 Git 历史中是否存在泄露的密钥或凭证。
- [ ] 已验证所有外部输入的处理路径，包括 Header / Cookie / Query / Body / File。
- [ ] 已确认安全配置在所有环境（dev / staging / prod）一致。
- [ ] 已记录每个未通过项的风险等级（Critical / High / Medium / Low）和修复建议。
- [ ] 安全评审报告已存档至 `output/` 目录。
