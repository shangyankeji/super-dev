---
id: security-glossary
title: 安全术语表 (Security Glossary)
domain: security
category: 06-glossary
difficulty: intermediate
tags: [glossary, security, 加密与认证, 合规与标准, 安全架构与策略, 安全测试方法, 安全运营, 攻击技术与威胁]
quality_score: 70
last_updated: 2026-06-15
---
# 安全术语表 (Security Glossary)

> 收录 50+ 核心安全术语，覆盖漏洞管理、攻防技术、安全运营、合规标准等领域。
> 适用于安全审计、架构评审、团队培训等场景。

---

## 漏洞与威胁管理

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| OWASP | Open Worldwide Application Security Project | 全球性非营利安全组织，维护 OWASP Top 10（Web 应用十大安全风险清单）、ZAP、Dependency-Check 等开源项目，是 Web 应用安全领域的事实标准制定者。 |
| CVE | Common Vulnerabilities and Exposures | 公开已知安全漏洞的标准化编号系统，由 MITRE 维护。格式为 CVE-年份-序号（如 CVE-2021-44228）。每个 CVE 对应一个唯一的漏洞标识，用于跨组织沟通和跟踪。 |
| CWE | Common Weakness Enumeration | 软件和硬件安全弱点的分类体系，由 MITRE 维护。与 CVE 的区别：CWE 描述的是弱点类型（如 CWE-79 XSS），CVE 描述的是具体漏洞实例。 |
| CVSS | Common Vulnerability Scoring System | 漏洞严重程度评分标准（0.0-10.0），由 FIRST 维护。评分维度包括攻击向量、攻击复杂度、权限要求、用户交互、影响范围（机密性/完整性/可用性）。当前版本 CVSS v4.0。 |
| 零日漏洞 | Zero-Day Vulnerability | 软件厂商尚未知晓或尚未发布补丁的安全漏洞。"零日"指厂商发现漏洞后用于修复的时间为零天。零日漏洞在黑市价值极高，是 APT 攻击的重要武器。 |
| NVD | National Vulnerability Database | 美国国家漏洞数据库，由 NIST 维护。基于 CVE 数据，补充 CVSS 评分、CWE 分类、修复建议等信息。是全球最权威的漏洞信息源之一。 |
| PoC | Proof of Concept | 漏洞利用概念验证代码，用于证明漏洞可被实际利用。安全研究人员编写 PoC 后通常进行负责任的披露（Responsible Disclosure）。 |
| Exploit | Exploit / 漏洞利用 | 利用安全漏洞执行未授权操作的代码或技术。按成熟度分为 PoC、武器化 Exploit（Weaponized）、打包 Exploit Kit 等级别。 |

---

## 攻击技术与威胁

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| APT | Advanced Persistent Threat | 高级持续性威胁，通常由国家或大型组织支持的攻击团体实施。特点：长期潜伏（数月至数年）、多阶段攻击链、定制化攻击工具、目标为高价值数据。知名 APT 组织如 APT28（俄罗斯）、APT41（中国）、Lazarus Group（朝鲜）。 |
| 社会工程 | Social Engineering | 通过心理操纵而非技术手段获取敏感信息或访问权限的攻击方式。常见形式：钓鱼邮件（Phishing）、电话诈骗（Vishing）、物理跟踪进入（Tailgating）、诱饵攻击（Baiting）。据统计，超过 90% 的安全事件涉及社会工程。 |
| 钓鱼攻击 | Phishing | 通过伪装成可信来源（银行、同事、供应商）的电子邮件、网站或消息，诱骗目标泄露凭据或安装恶意软件。变体包括鱼叉钓鱼（Spear Phishing，针对特定个人）和鲸钓（Whaling，针对高管）。 |
| 勒索软件 | Ransomware | 加密受害者文件并要求支付赎金的恶意软件。现代勒索软件采用双重勒索（加密 + 数据泄露威胁）甚至三重勒索（加 DDoS 威胁）。RaaS（勒索软件即服务）降低了攻击门槛。 |
| DDoS | Distributed Denial of Service | 分布式拒绝服务攻击，利用大量受控设备（僵尸网络）向目标发送海量请求，导致服务不可用。按攻击层分为：L3/L4 流量型（UDP Flood、SYN Flood）和 L7 应用型（HTTP Flood、Slowloris）。 |
| 中间人攻击 | Man-in-the-Middle (MITM) | 攻击者在通信双方之间截取和篡改通信内容，而双方不知情。常见场景：公共 WiFi 监听、ARP 欺骗、DNS 劫持、SSL 剥离。HTTPS 和证书锁定（Certificate Pinning）是主要防御手段。 |
| 供应链攻击 | Supply Chain Attack | 通过入侵软件供应链的某个环节（开发工具、第三方库、更新服务器）来攻击最终用户。典型案例：SolarWinds 事件、CodeCov 事件、Log4Shell。 |
| 横向移动 | Lateral Movement | 攻击者在初始入侵后，利用已获取的凭据和权限在内网中从一台主机移动到另一台主机的过程。常用技术：Pass-the-Hash、RDP、PsExec、WMI。 |
| 提权 | Privilege Escalation | 攻击者从低权限账户获取更高权限（如管理员/root）的过程。分为水平提权（获取同级别其他用户权限）和垂直提权（获取更高级别权限）。 |
| 注入攻击 | Injection Attack | 将恶意输入插入到应用程序执行的命令或查询中的攻击方式。包括 SQL 注入、命令注入、LDAP 注入、XSS（HTML/JS 注入）、模板注入（SSTI）等。OWASP Top 10 长期排名第一的风险类别。 |

---

## 防御技术与工具

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| WAF | Web Application Firewall | Web 应用防火墙，部署在 Web 服务器前端，分析 HTTP/HTTPS 流量并拦截恶意请求（如 SQL 注入、XSS）。部署模式：反向代理、透明代理、云 WAF。代表产品：Cloudflare WAF、AWS WAF、ModSecurity。 |
| IDS | Intrusion Detection System | 入侵检测系统，监控网络流量或主机行为，检测可疑活动并发出告警。分为网络型 IDS（NIDS，如 Snort、Suricata）和主机型 IDS（HIDS，如 OSSEC）。仅检测不阻断。 |
| IPS | Intrusion Prevention System | 入侵防御系统，在 IDS 基础上增加主动阻断能力。可自动丢弃恶意数据包或阻断连接。通常部署在网络关键路径上（inline 模式）。 |
| SIEM | Security Information and Event Management | 安全信息和事件管理系统，集中收集、关联分析来自各安全设备和系统的日志，提供实时安全监控、事件关联分析和合规报告。代表产品：Splunk、QRadar、Elastic SIEM、Microsoft Sentinel。 |
| SOAR | Security Orchestration, Automation and Response | 安全编排自动化与响应平台，通过 Playbook 自动化安全事件的检测、分类、响应流程。减少安全团队的手动重复工作，缩短事件响应时间（MTTR）。 |
| EDR | Endpoint Detection and Response | 端点检测与响应，部署在终端设备上，持续监控进程、文件、网络活动，检测高级威胁并提供远程响应能力。代表产品：CrowdStrike Falcon、Microsoft Defender for Endpoint、Carbon Black。 |
| XDR | Extended Detection and Response | 扩展检测与响应，在 EDR 基础上整合网络、云、邮件等多源数据，提供跨层的威胁检测和自动化响应能力。是 EDR 的演进方向。 |
| 蜜罐 | Honeypot | 故意暴露的虚假系统或服务，用于诱骗攻击者并收集攻击情报。分为低交互蜜罐（模拟服务）和高交互蜜罐（真实系统环境）。蜜罐网络（Honeynet）由多个蜜罐组成，用于研究攻击模式。 |
| 蜜令牌 | Honeytoken | 故意放置的虚假凭据或数据（如假 API Key、假数据库记录），当被访问时触发告警，说明系统已被入侵。比蜜罐更轻量，适用于检测内部威胁。 |
| RASP | Runtime Application Self-Protection | 运行时应用自我保护，嵌入到应用运行时中，在应用内部检测和阻断攻击。与 WAF 的区别：RASP 在应用内部工作，能感知应用上下文，误报率更低。 |
| DLP | Data Loss Prevention | 数据防泄露系统，监控和阻止敏感数据的未授权传输。部署在端点、网络和云端，识别并保护结构化数据（数据库）和非结构化数据（文件、邮件）。 |

---

## 安全运营

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| SOC | Security Operations Center | 安全运营中心，7x24 小时监控组织安全态势的集中化团队和设施。核心职能：安全监控、事件分析、应急响应、威胁狩猎。SOC 分为 L1（告警分流）、L2（深度分析）、L3（高级调查）三级。 |
| 红队 | Red Team | 模拟真实攻击者对组织进行全面安全评估的专业团队。与渗透测试的区别：红队演练更综合，可能包含社会工程、物理安全等，目标是评估整体安全防御能力而非仅寻找技术漏洞。 |
| 蓝队 | Blue Team | 负责防御和检测的安全团队，是红队的对手方。职责包括：安全监控、事件响应、安全加固、威胁情报分析。日常安全运营由蓝队执行。 |
| 紫队 | Purple Team | 红蓝队协作模式，红队分享攻击技术和发现，蓝队据此优化检测和防御能力。紫队演练的目标是最大化红蓝队的协作价值，持续提升安全防御水平。 |
| 渗透测试 | Penetration Testing | 通过模拟攻击来评估系统安全性的授权测试活动。按知识范围分为黑盒（无内部信息）、白盒（完整内部信息）、灰盒（部分信息）。常见标准：PTES、OSSTMM、OWASP Testing Guide。 |
| Bug Bounty | Bug Bounty Program | 漏洞赏金计划，邀请外部安全研究人员在授权范围内寻找漏洞并给予金钱奖励。平台：HackerOne、Bugcrowd、Intigriti。Google、Apple、Microsoft 等大公司均运营 Bug Bounty 计划。 |
| 威胁狩猎 | Threat Hunting | 主动搜索网络中已存在但未被自动化工具检测到的威胁活动。与传统监控的区别：威胁狩猎是主动的、假设驱动的，由分析师主导而非规则驱动。 |
| IOC | Indicator of Compromise | 入侵指标，用于识别系统已被攻击的证据。包括：恶意 IP 地址、域名、文件哈希（MD5/SHA256）、注册表修改、异常进程等。IOC 是威胁情报共享的基本单位。 |
| TTP | Tactics, Techniques, and Procedures | 战术、技术和程序，描述攻击者的行为模式。MITRE ATT&CK 框架将 TTP 系统化分类，是威胁情报和检测工程的标准参考。相比 IOC，TTP 更难被攻击者改变。 |
| MITRE ATT&CK | MITRE ATT&CK Framework | 基于真实攻击案例的对抗战术和技术知识库。分为 Enterprise（企业）、Mobile（移动）、ICS（工控）三个矩阵。涵盖从初始访问到数据渗出的完整攻击链，是安全检测工程的事实标准。 |
| MTTD | Mean Time to Detect | 平均检测时间，从安全事件发生到被检测发现的平均时长。是衡量安全监控能力的核心指标。行业平均 MTTD 约为 200+ 天，优秀组织可控制在数小时内。 |
| MTTR | Mean Time to Respond | 平均响应时间，从安全事件被检测到完成响应处置的平均时长。与 MTTD 一起构成安全运营的核心效率指标。 |

---

## 安全架构与策略

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| 零信任 | Zero Trust Architecture | 安全架构模型，核心原则为"永不信任，始终验证"。不再区分内外网，每次访问请求都需要经过身份验证、授权检查和持续安全评估。关键组件：身份认证、微分段、最小权限、持续验证。NIST SP 800-207 定义了零信任架构参考模型。 |
| 纵深防御 | Defense in Depth | 安全架构策略，在多个层面部署重叠的安全控制措施。即使某一层防御被突破，后续层仍能提供保护。层次通常包括：物理安全、网络安全、主机安全、应用安全、数据安全。 |
| 最小权限 | Principle of Least Privilege | 安全原则，每个主体（用户、进程、服务）仅被授予完成其任务所需的最小权限集。违反最小权限原则是权限提升攻击的主要原因。 |
| STRIDE | STRIDE Threat Model | 微软提出的威胁建模方法，六类威胁：Spoofing（伪装）、Tampering（篡改）、Repudiation（抵赖）、Information Disclosure（信息泄露）、Denial of Service（拒绝服务）、Elevation of Privilege（权限提升）。 |
| 安全左移 | Shift Left Security | 将安全活动提前到软件开发生命周期的早期阶段（需求、设计、编码），而非仅在部署后进行安全测试。包括：安全需求分析、威胁建模、SAST、SCA 等。 |
| DevSecOps | DevSecOps | 将安全实践集成到 DevOps 流程中的文化和方法论。安全不再是开发流程的瓶颈或事后检查，而是贯穿 CI/CD 管道的自动化环节。 |

---

## 加密与认证

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| PKI | Public Key Infrastructure | 公钥基础设施，管理数字证书和公私钥对的体系。核心组件：CA（证书颁发机构）、RA（注册机构）、证书存储库、CRL/OCSP（证书吊销）。HTTPS/TLS 依赖 PKI 体系运作。 |
| HSM | Hardware Security Module | 硬件安全模块，专用加密硬件设备，用于安全生成、存储和使用加密密钥。防篡改设计，密钥在 HSM 内部使用，不可导出。合规场景（PCI DSS、FIPS 140-2）常要求使用 HSM。 |
| MFA | Multi-Factor Authentication | 多因素认证，要求用户提供两种或以上不同类别的身份验证因素：知识因素（密码）、持有因素（手机/硬件密钥）、生物因素（指纹/面部）。显著降低凭据泄露导致的账户接管风险。 |
| FIDO2 | Fast Identity Online 2 | 无密码认证标准，由 FIDO 联盟和 W3C 联合制定。包含 WebAuthn（Web 认证 API）和 CTAP（客户端到认证器协议）。支持硬件安全密钥和平台认证器（如指纹/面部识别），抗钓鱼。 |
| SSO | Single Sign-On | 单点登录，用户只需认证一次即可访问多个关联系统。实现协议：SAML 2.0、OpenID Connect、Kerberos。提升用户体验的同时需注意 SSO 成为单点故障和高价值攻击目标的风险。 |

---

## 合规与标准

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| GDPR | General Data Protection Regulation | 欧盟通用数据保护条例，2018 年 5 月生效。保护 EU 居民个人数据，赋予数据主体权利（访问权、删除权、可携带权等）。违规最高罚款：全球年营收 4% 或 2000 万欧元。 |
| SOC 2 | System and Organization Controls 2 | 美国注册会计师协会（AICPA）制定的服务组织审计标准。基于五项信任服务原则：安全性、可用性、处理完整性、保密性、隐私性。SaaS 企业客户通常要求 SOC 2 Type II 报告。 |
| 等保 | 信息安全等级保护 | 中国信息安全等级保护制度，将信息系统按重要程度分为五个安全保护等级。三级及以上系统需进行等保测评并备案。等保 2.0 扩展覆盖云计算、移动互联、物联网、工控等新技术。 |
| PCI DSS | Payment Card Industry Data Security Standard | 支付卡行业数据安全标准，由 PCI SSC 制定。适用于存储、处理或传输信用卡数据的任何组织。12 项核心要求覆盖网络安全、数据保护、访问控制、监控测试等领域。 |
| HIPAA | Health Insurance Portability and Accountability Act | 美国健康保险携带与责任法案，保护受保护健康信息（PHI）的隐私和安全。适用于医疗保健提供者、健康计划和医疗信息交换中心。 |
| ISO 27001 | ISO/IEC 27001 | 国际信息安全管理体系（ISMS）标准，由 ISO 和 IEC 联合发布。规定了建立、实施、维护和持续改进 ISMS 的要求。附录 A 包含 114 项安全控制措施。 |
| NIST CSF | NIST Cybersecurity Framework | 美国国家标准与技术研究院网络安全框架。五项核心功能：识别（Identify）、保护（Protect）、检测（Detect）、响应（Respond）、恢复（Recover）。广泛用于安全态势评估和改进规划。 |
| SLSA | Supply-chain Levels for Software Artifacts | 软件供应链安全分级框架，由 Google 发起。从 L1 到 L4 四个级别，逐级增加对构建过程完整性的保障。目标是防止软件制品被篡改。 |
| SBOM | Software Bill of Materials | 软件物料清单，列出软件产品中所有组件、库和依赖的详细清单。美国行政令 14028 要求联邦供应商提供 SBOM。标准格式：SPDX、CycloneDX。 |

---

## 安全测试方法

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| SAST | Static Application Security Testing | 静态应用安全测试，在不运行代码的情况下分析源代码或字节码中的安全漏洞。优点：早期发现、覆盖全面。缺点：误报率较高、无法检测运行时问题。代表工具：SonarQube、Semgrep、CodeQL、Checkmarx。 |
| DAST | Dynamic Application Security Testing | 动态应用安全测试，对运行中的应用发送构造的请求来检测漏洞。优点：接近真实攻击、低误报。缺点：覆盖率受测试用例限制。代表工具：OWASP ZAP、Burp Suite、Acunetix。 |
| IAST | Interactive Application Security Testing | 交互式应用安全测试，结合 SAST 和 DAST，在应用运行时通过代码插桩（Instrumentation）监控数据流和执行路径。兼具 SAST 的代码级定位和 DAST 的真实环境验证。 |
| SCA | Software Composition Analysis | 软件成分分析，识别项目中使用的开源组件和第三方库，检查已知漏洞和许可证合规性。代表工具：Snyk、Dependabot、Black Duck、OWASP Dependency-Check。 |
| Fuzzing | Fuzz Testing / 模糊测试 | 向程序输入大量随机或半随机数据，观察是否会导致崩溃、内存泄漏或其他异常行为的测试方法。特别适合测试解析器、协议处理器和文件格式处理。代表工具：AFL++、libFuzzer、OSS-Fuzz。 |

---

## 数据安全与隐私

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| PII | Personally Identifiable Information | 个人身份信息，能够单独或与其他数据结合识别特定个人的信息。包括：姓名、身份证号、电话号码、电子邮件、IP 地址、生物特征等。PII 是 GDPR、CCPA 等数据保护法规的核心保护对象。 |
| 数据脱敏 | Data Masking / Anonymization | 将敏感数据替换为虚构或不可逆的值，使其无法关联到特定个人。技术包括：替换（Substitution）、混洗（Shuffling）、加噪（Noise Addition）、截断（Truncation）、令牌化（Tokenization）。静态脱敏用于测试环境，动态脱敏用于生产查询。 |
| 令牌化 | Tokenization | 将敏感数据（如信用卡号）替换为无意义的令牌（Token），原始数据安全存储在令牌库中。与加密的区别：令牌与原始值之间没有数学关系，即使令牌被截获也无法逆推原始值。PCI DSS 场景中广泛使用。 |
| 数据分类分级 | Data Classification | 根据数据敏感度和业务价值将数据划分为不同等级（如公开、内部、机密、绝密），并对不同等级的数据实施相应的保护措施。是数据安全治理的基础工作。 |
| 数据主权 | Data Sovereignty | 数据受其物理存储所在国法律管辖的原则。影响跨国企业的数据存储和传输策略。欧盟 GDPR 限制个人数据向欧盟境外传输，中国《数据安全法》和《个人信息保护法》对数据出境有严格要求。 |

---

## 网络安全协议

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| TLS | Transport Layer Security | 传输层安全协议，HTTPS 的加密基础。TLS 1.3（2018 年发布）简化了握手流程（1-RTT），移除了不安全的密码套件。TLS 提供三项安全保证：机密性（加密）、完整性（MAC）、身份认证（证书）。 |
| mTLS | Mutual TLS | 双向 TLS 认证，不仅服务器向客户端出示证书，客户端也向服务器出示证书。在微服务架构中实现服务间零信任通信。Service Mesh（如 Istio）可自动管理 mTLS 证书。 |
| CORS | Cross-Origin Resource Sharing | 跨源资源共享，浏览器安全机制，控制哪些外部域可以访问 Web 资源。配置不当（如 `Access-Control-Allow-Origin: *`）会导致敏感数据泄露。需要精确配置允许的源、方法和头信息。 |
| CSP | Content Security Policy | 内容安全策略，通过 HTTP 头指定浏览器允许加载的资源来源，有效防御 XSS 和数据注入攻击。关键指令：`default-src`、`script-src`、`style-src`、`connect-src`。配置 `unsafe-inline` 和 `unsafe-eval` 会显著降低防护效果。 |
| HSTS | HTTP Strict Transport Security | HTTP 严格传输安全，通过响应头强制浏览器仅使用 HTTPS 访问站点。`max-age` 指定生效时间（建议至少 1 年），`includeSubDomains` 覆盖所有子域。可提交到 HSTS Preload List 实现浏览器内置强制。 |
| DNSSEC | Domain Name System Security Extensions | DNS 安全扩展，通过数字签名验证 DNS 应答的真实性和完整性，防止 DNS 缓存投毒和 DNS 劫持。部署链路：根域 → TLD → 权威域名服务器逐级签名。 |
| OAuth 2.0 | Open Authorization 2.0 | 开放授权框架（RFC 6749），允许第三方应用在用户授权下有限访问其在其他服务上的资源，无需共享密码。核心概念：授权码、访问令牌、刷新令牌、作用域。OAuth 2.1 草案整合了安全最佳实践（强制 PKCE、禁止 Implicit 流程）。 |
| SAML | Security Assertion Markup Language | 安全断言标记语言，基于 XML 的企业级单点登录协议。主要用于企业身份提供者（IdP）与服务提供者（SP）之间的身份联邦。相比 OIDC 更适合企业环境，但 XML 格式较复杂。 |

---

## 云安全

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| CSPM | Cloud Security Posture Management | 云安全态势管理，自动检测和修复云基础设施的安全配置错误。覆盖 AWS/Azure/GCP 等多云环境。常见检测项：S3 存储桶公开访问、安全组过度开放、未加密的存储卷。代表工具：Prisma Cloud、Wiz、Prowler。 |
| CWPP | Cloud Workload Protection Platform | 云工作负载保护平台，保护运行在云端的虚拟机、容器和 Serverless 工作负载。功能包括：漏洞管理、运行时保护、网络微分段、合规检查。 |
| CASB | Cloud Access Security Broker | 云访问安全代理，部署在用户与云服务之间，提供可见性、合规性、威胁防护和数据安全。主要解决 Shadow IT 问题（员工未经授权使用云服务）。 |
| IAM | Identity and Access Management | 身份与访问管理，管理数字身份及其对资源的访问权限的框架和技术。云 IAM（如 AWS IAM）是云安全的基础，核心原则：最小权限、职责分离、定期审计。 |
| KMS | Key Management Service | 密钥管理服务，云服务商提供的托管式密钥管理解决方案（如 AWS KMS、Azure Key Vault、GCP Cloud KMS）。支持密钥生成、轮换、审计，与加密服务集成。合规场景建议使用 CloudHSM 获得更高安全保证。 |

---

## 应急响应

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| IR | Incident Response | 安全事件响应，识别、遏制、根除安全事件并从中恢复的系统化流程。NIST SP 800-61 定义了四阶段模型：准备 → 检测与分析 → 遏制/根除/恢复 → 事后活动。 |
| DFIR | Digital Forensics and Incident Response | 数字取证与事件响应，结合数字证据收集/分析与安全事件处置的专业领域。数字取证遵循证据链完整性原则（Chain of Custody），确保证据在法律程序中可采信。 |
| BCP | Business Continuity Plan | 业务连续性计划，确保组织在灾难或中断事件期间维持关键业务运营的策略和程序文档。包含：风险评估、业务影响分析（BIA）、恢复策略、演练计划。 |
| DRP | Disaster Recovery Plan | 灾难恢复计划，BCP 的子集，专注于 IT 系统和数据的恢复。关键指标：RPO（恢复点目标，可接受的数据丢失量）和 RTO（恢复时间目标，系统恢复时间上限）。 |
| 战争室 | War Room | 安全事件期间临时成立的跨职能应急响应协调中心。集结安全、运维、开发、法务、公关等团队，统一指挥事件处置。在重大安全事件（如 Log4Shell）期间，战争室可能持续运作数天至数周。 |

---

## 其他重要术语

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| VDP | Vulnerability Disclosure Policy | 漏洞披露政策，组织公开发布的文档，说明外部安全研究人员如何向该组织报告安全漏洞。VDP 通常不提供金钱奖励（区别于 Bug Bounty），但承诺不对善意报告者采取法律行动。ISO 29147 提供了漏洞披露的标准指南。 |
| 攻击面 | Attack Surface | 系统中所有可被攻击者利用的潜在入口点的集合。包括：开放端口、API 端点、用户输入、第三方集成等。攻击面管理（ASM）是持续发现、分类和监控组织暴露面的实践。减少攻击面是安全加固的核心目标。 |
| 安全基线 | Security Baseline | 系统或网络的最低安全配置标准。包括操作系统加固配置、网络设备安全配置、应用安全配置等。CIS Benchmarks 是业界广泛采用的安全基线标准，覆盖 100+ 种技术平台。 |
| 威胁情报 | Threat Intelligence (TI) | 基于证据的关于威胁的知识，包括上下文、机制、指标、影响和可操作建议。分为战略级（趋势分析）、战术级（TTP）和运营级（IOC）。共享标准：STIX/TAXII。 |
| 证书透明度 | Certificate Transparency (CT) | Google 发起的开放框架，要求 CA 将颁发的所有 TLS 证书记录到公开可审计的 CT 日志中。帮助检测错误颁发或恶意颁发的证书。2018 年起 Chrome 强制要求所有新证书支持 CT。 |
| 安全编排 | Security Orchestration | 将多个安全工具和流程通过自动化工作流连接起来，实现安全事件的自动分流、富化和响应。减少人工操作，提升响应效率。通常作为 SOAR 平台的核心能力。 |

---

## Agent Checklist

- [ ] 术语表覆盖 50+ 个安全术语
- [ ] 分类清晰：漏洞管理、攻击技术、防御工具、安全运营、架构策略、加密认证、合规标准、测试方法
- [ ] 每个术语包含中文名、英文全称、完整定义
- [ ] 表格格式规范，可直接用于文档和培训
- [ ] 覆盖所有指定术语：OWASP/CVE/CWE/CVSS/零日漏洞/APT/社会工程/蜜罐/WAF/IDS/IPS/SIEM/SOC/红队/蓝队/紫队/渗透测试/Bug Bounty/零信任
- [ ] 定义内容准确，包含代表性工具、标准和实际应用场景
- [ ] 术语之间的关联关系在定义中有交叉引用说明
