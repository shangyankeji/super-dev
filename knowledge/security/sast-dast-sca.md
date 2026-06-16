---
id: sast-dast-sca
title: SAST/DAST/SCA 完整指南
domain: security
category: sast-dast-sca.md
difficulty: intermediate
tags: [dast, sast, sca, security, 三者对比, 动态应用安全测试, 报告和度量, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# SAST/DAST/SCA 完整指南

## 概述
SAST(静态应用安全测试)、DAST(动态应用安全测试)和 SCA(软件成分分析)是应用安全的三大核心测试方法,覆盖代码、运行时和依赖的安全检查。

## 三者对比

| 维度 | SAST | DAST | SCA |
|------|------|------|-----|
| 测试阶段 | 编码/构建 | 运行时 | 构建/部署 |
| 扫描对象 | 源代码 | 运行应用 | 第三方依赖 |
| 检测类型 | 代码漏洞 | 运行时漏洞 | 依赖漏洞 |
| 误报率 | 中-高 | 低 | 低 |
| 修复成本 | 低 | 高 | 中 |
| 自动化程度 | 高 | 中 | 高 |

## SAST - 静态应用安全测试

### 工作原理
```
源代码 -> 词法分析 -> 语法树(AST) -> 数据流分析 -> 污点追踪 -> 漏洞报告
```

### 支持的漏洞类型
```yaml
输入验证:
  - SQL 注入
  - XSS(跨站脚本)
  - 命令注入
  - LDAP 注入
  - XPath 注入

认证授权:
  - 硬编码密码
  - 弱加密算法
  - 不安全的随机数
  - Session 管理缺陷

配置安全:
  - 不安全的配置
  - 敏感信息泄露
  - 调试信息暴露
  - 不安全的默认值

并发安全:
  - 竞态条件
  - 死锁风险
  - 资源泄露
```

### 工具选型

#### Semgrep(推荐)
```yaml
# .semgrepignore
node_modules/
dist/
build/
*.min.js

# semgrep.yaml
rules:
  - id: sql-injection
    patterns:
      - pattern: |
          cursor.execute($QUERY, ...)
      - pattern-not: |
          cursor.execute("...", ...)
    message: 检测到 SQL 注入风险
    severity: ERROR
    languages: [python]
```

```bash
# 运行 Semgrep
semgrep --config=auto --json --output=semgrep-report.json

# 使用自定义规则
semgrep --config=semgrep.yaml --severity ERROR
```

#### SonarQube
```yaml
# sonar-project.properties
sonar.projectKey=myapp
sonar.sources=src/
sonar.tests=tests/
sonar.exclusions=**/*.test.js,**/*.spec.js
sonar.javascript.lcov.reportPaths=coverage/lcov.info
sonar.security.hotspots.review=true
sonar.qualitygate.wait=true
```

```bash
# 运行 SonarQube 扫描
sonar-scanner \
  -Dsonar.host.url=http://sonarqube:9000 \
  -Dsonar.login=$SONAR_TOKEN
```

#### Bandit(Python)
```bash
# 安装
pip install bandit

# 扫描
bandit -r src/ -f json -o bandit-report.json

# 仅报告高危
bandit -r src/ -ll -ii
```

#### ESLint Security(JavaScript)
```javascript
// .eslintrc.js
module.exports = {
  plugins: ['security'],
  extends: ['plugin:security/recommended'],
  rules: {
    'security/detect-eval-with-expression': 'error',
    'security/detect-non-literal-fs-filename': 'warn',
    'security/detect-unsafe-regex': 'error'
  }
};
```

### SAST 最佳实践

#### 1. 增量扫描
```yaml
# GitLab CI 增量扫描
sast:
  script:
    - |
      CHANGED_FILES=$(git diff --name-only $CI_COMMIT_BEFORE_SHA $CI_COMMIT_SHA | grep '\.py$')
      if [ -n "$CHANGED_FILES" ]; then
        bandit -r $CHANGED_FILES -f json -o bandit-report.json
      fi
```

#### 2. 误报处理
```python
# Python 示例 - 抑制误报
password = os.getenv('DB_PASSWORD')  # nosec: B105  # 从环境变量读取,非硬编码

# 或使用配置文件
# .bandit
[bandit]
exclude_dirs = ["/tests/", "/migrations/"]
skips = ["B101", "B601"]
```

#### 3. 质量门禁
```yaml
# GitLab CI
sast_gate:
  script:
    - |
      HIGH_VULNS=$(jq '.vulnerabilities | map(select(.severity == "HIGH")) | length' sast-report.json)
      CRITICAL_VULNS=$(jq '.vulnerabilities | map(select(.severity == "CRITICAL")) | length' sast-report.json)

      if [ "$HIGH_VULNS" -gt 0 ] || [ "$CRITICAL_VULNS" -gt 0 ]; then
        echo "发现 $CRITICAL_VULNS 个严重漏洞和 $HIGH_VULNS 个高危漏洞"
        exit 1
      fi
```

#### 4. IDE 集成
```json
// VS Code settings.json
{
  "semgrep.trace.server": "verbose",
  "semgrep.path": "/usr/local/bin/semgrep",
  "sonarlint.ls.javaHome": "/usr/lib/jvm/java-17",
  "sonarlint.rules": {
    "javascript:S2077": "error",
    "python:S5146": "error"
  }
}
```

## DAST - 动态应用安全测试

### 工作原理
```
HTTP 请求 -> 爬虫 -> 漏洞扫描 -> Payload 注入 -> 响应分析 -> 漏洞报告
```

### 支持的漏洞类型
```yaml
注入攻击:
  - SQL 注入
  - 命令注入
  - XPath 注入
  - NoSQL 注入

客户端攻击:
  - XSS(反射型、存储型)
  - CSRF
  - 点击劫持
  - 开放重定向

认证会话:
  - 弱密码策略
  - 会话固定
  - 认证绕过
  - 权限提升

配置安全:
  - 不安全的 HTTP 方法
  - 敏感信息泄露
  - 缺少安全头
  - CORS 配置错误
```

### 工具选型

#### OWASP ZAP(推荐)
```yaml
# ZAP 自动化配置
environments:
  - name: production
    urls:
      - https://api.example.com

scanners:
  - name: sql_injection
    enabled: true
    strength: high
    threshold: medium

  - name: xss
    enabled: true
    strength: high
    threshold: medium

authentication:
  type: bearer
  token: ${AUTH_TOKEN}

report:
  format: json
  output: zap-report.json
```

```bash
# 命令行扫描
zap-baseline.py \
  -t https://api.example.com \
  -r zap-report.html \
  --hook=/zap/scripts/auth_hook.py

# 全量扫描
zap-full-scan.py \
  -t https://api.example.com \
  -r zap-report.html \
  -d \
  -a \
  -l PASS
```

#### Burp Suite
```python
# Burp Suite REST API
import requests

# 启动扫描
response = requests.post(
    'http://burp:8080/api/v0.1/scan',
    json={
        'urls': ['https://api.example.com'],
        'scope': {
            'include': ['https://api.example.com/.*']
        }
    }
)

scan_id = response.json()['scan_id']

# 获取结果
issues = requests.get(f'http://burp:8080/api/v0.1/scan/{scan_id}').json()
```

#### Nuclei
```yaml
# nuclei-templates/http/vulnerabilities/sql-injection.yaml
id: sql-injection-generic
info:
  name: SQL Injection Detection
  severity: high

http:
  - method: GET
    path:
      - "{{BaseURL}}/search?q=test'"
    matchers:
      - type: regex
        regex:
          - "SQL syntax.*MySQL"
          - "Warning.*pg_.*"
          - "ORA-01756"
```

```bash
# 运行 Nuclei 扫描
nuclei -u https://api.example.com -t cves/ -severity critical,high -o nuclei-report.txt
```

### DAST 最佳实践

#### 1. 认证扫描
```python
# ZAP 认证脚本
from zapv2 import ZAPv2

zap = ZAPv2(apikey='zap-api-key')

# 配置认证
zap.authentication.set_authentication_method(
    contextid=1,
    authmethodname='formBasedAuthentication',
    authmethodconfigparams='loginUrl=https://example.com/login loginRequestData=username={%username%}&password={%password%}'
)

# 配置用户
zap.users.new_user(contextid=1, username='testuser')
zap.users.set_authentication_credentials(
    contextid=1,
    userid=1,
    authcredentialsconfigparams='username=testuser&password=testpass'
)

# 启用用户
zap.users.set_user_enabled(contextid=1, userid=1, enabled=True)
```

#### 2. API 扫描
```yaml
# OpenAPI 规范扫描
openapi_scan:
  target: https://api.example.com
  spec: openapi.yaml
  auth:
    type: bearer
    token: ${API_TOKEN}

  scan_config:
    inject_payloads:
      - "' OR '1'='1"
      - "<script>alert(1)</script>"
      - "../../../../etc/passwd"
```

#### 3. 排除路径
```yaml
# ZAP 排除配置
zap.b
  -t https://api.example.com \
  -r report.html \
  --exclude=".*logout.*" \
  --exclude=".*delete.*"
```

#### 4. 报告分级
```yaml
# 质量门禁
dast_gate:
  script:
    - |
      HIGH_VULNS=$(jq '.site[0].alerts[] | select(.riskdesc | startswith("High")) | .riskdesc' zap-report.json | wc -l)

      if [ "$HIGH_VULNS" -gt 0 ]; then
        echo "发现 $HIGH_VULNS 个高危漏洞,禁止部署"
        exit 1
      fi
```

## SCA - 软件成分分析

### 工作原理
```
依赖清单 -> 解析依赖树 -> 漏洞数据库匹配 -> 许可证检查 -> 风险报告
```

### 支持的检查类型
```yaml
漏洞检测:
  - CVE 漏洞
  - 依赖版本漏洞
  - 传递依赖漏洞
  - 已知恶意包

许可证合规:
  - GPL 许可证冲突
  - 商业许可证
  - 弃用许可证
  - 许可证兼容性

代码质量:
  - 弃用包
  - 维护状态
  - 下载量
  - 社区活跃度
```

### 工具选型

#### Snyk(推荐)
```yaml
# .snyk
version: v1.13.0
ignore:
  CVE-2021-44228:
    - "*":
        reason: 已应用补丁
        expires: 2025-12-31
patch:
  CVE-2021-23456:
    - lodash:
        patched: "4.17.21"
```

```bash
# 运行 Snyk 扫描
snyk test --severity-threshold=high --json > snyk-report.json

# 监控项目
snyk monitor --org=myorg

# 自动修复
snyk fix
```

#### Trivy
```bash
# 扫描文件系统
trivy fs --severity HIGH,CRITICAL --ignore-unfixed .

# 扫描镜像
trivy image --severity HIGH,CRITICAL myapp:latest

# 生成 SBOM
trivy fs --format spdx-json --output sbom.spdx.json .

# SBOM 漏洞扫描
trivy sbom sbom.spdx.json --severity HIGH,CRITICAL
```

#### Dependabot
```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "npm"
    directory: "/"
    schedule:
      interval: "daily"
    open-pull-requests-limit: 10
    reviewers:
      - "myorg/security-team"
    labels:
      - "security"
      - "dependencies"
    commit-message:
      prefix: "fix"
      include: "scope"

  - package-ecosystem: "docker"
    directory: "/"
    schedule:
      interval: "weekly"
```

#### Grype
```bash
# 扫描目录
grype dir:. --fail-on high

# 扫描 SBOM
grype sbom:sbom.json --output json > grype-report.json

# 仅显示已修复漏洞
grype dir:. --only-fixed
```

### SCA 最佳实践

#### 1. 锁定依赖版本
```json
// package.json
{
  "dependencies": {
    "lodash": "4.17.21",        // 精确版本
    "express": "~4.18.2",       // 补丁版本
    "mongoose": "^7.0.0"        // 次版本
  },
  "devDependencies": {
    "typescript": "5.3.3"
  }
}

// package-lock.json 或 yarn.lock 必须提交
```

#### 2. 私有仓库镜像
```bash
# 使用镜像仓库
npm config set registry https://registry.npmmirror.com

# 或使用 .npmrc
registry=https://registry.npmmirror.com
```

#### 3. 持续监控
```yaml
# GitLab CI
dependency_scanning:
  stage: test
  script:
    - npm audit --json > npm-audit.json
  artifacts:
    reports:
      dependency_scanning: npm-audit.json
  rules:
    - if: $CI_PIPELINE_SOURCE == "schedule"  # 定期扫描
    - if: $CI_COMMIT_BRANCH == "main"
```

#### 4. 修复策略
```yaml
# 修复优先级
priority_matrix:
  critical:
    max_age: 7 days
    auto_pr: true
    block_deploy: true

  high:
    max_age: 30 days
    auto_pr: true
    block_deploy: false

  medium:
    max_age: 90 days
    auto_pr: false
    block_deploy: false

  low:
    max_age: 180 days
    auto_pr: false
    block_deploy: false
```

## 集成到 CI/CD

### 完整流水线示例
```yaml
# GitLab CI 完整安全流水线
stages:
  - test
  - scan
  - gate
  - deploy

# SAST 扫描
sast:
  stage: test
  image: returntocorp/semgrep:latest
  script:
    - semgrep --config=auto --json --output=semgrep-report.json
  artifacts:
    reports:
      sast: semgrep-report.json

# SCA 扫描
dependency_scanning:
  stage: test
  image: aquasec/trivy:latest
  script:
    - trivy fs --format json --output=trivy-report.json --severity HIGH,CRITICAL .
  artifacts:
    reports:
      dependency_scanning: trivy-report.json

# DAST 扫描
dast:
  stage: scan
  image: zaproxy/zap-baseline:latest
  script:
    - zap-baseline.py -t $TARGET_URL -r zap-report.html -j
  artifacts:
    reports:
      dast: zap-report.html
  only:
    - main

# 质量门禁
security_gate:
  stage: gate
  script:
    - |
      # 检查 SAST
      SAST_HIGH=$(jq '.vulnerabilities | map(select(.severity == "HIGH" or .severity == "CRITICAL")) | length' semgrep-report.json)

      # 检查 SCA
      SCA_HIGH=$(jq '.Results[] | select(.Vulnerabilities != null) | .Vulnerabilities | length' trivy-report.json)

      # 检查 DAST
      DAST_HIGH=$(grep -c 'riskcode="3"' zap-report.html || echo 0)

      # 决策
      if [ "$SAST_HIGH" -gt 0 ] || [ "$SCA_HIGH" -gt 0 ] || [ "$DAST_HIGH" -gt 0 ]; then
        echo "安全门禁失败"
        echo "SAST 高危: $SAST_HIGH"
        echo "SCA 高危: $SCA_HIGH"
        echo "DAST 高危: $DAST_HIGH"
        exit 1
      fi
  only:
    - main

deploy:
  stage: deploy
  script:
    - kubectl apply -f k8s/
  only:
    - main
```

## 漏洞修复流程

### 1. 漏洞分级
```yaml
严重程度:
  CRITICAL:
    cvss: 9.0 - 10.0
    response_time: 24 hours
    block_deploy: true

  HIGH:
    cvss: 7.0 - 8.9
    response_time: 7 days
    block_deploy: true

  MEDIUM:
    cvss: 4.0 - 6.9
    response_time: 30 days
    block_deploy: false

  LOW:
    cvss: 0.1 - 3.9
    response_time: 90 days
    block_deploy: false
```

### 2. 修复 SLA
```yaml
sla:
  critical:
    acknowledgment: 4 hours
    fix_development: 24 hours
    fix_production: 48 hours

  high:
    acknowledgment: 24 hours
    fix_development: 7 days
    fix_production: 14 days

  medium:
    acknowledgment: 72 hours
    fix_development: 30 days
    fix_production: 60 days

  low:
    acknowledgment: 1 week
    fix_development: 90 days
    fix_production: 180 days
```

### 3. 例外管理
```yaml
# .vulnerability-exceptions.yaml
exceptions:
  - id: CVE-2021-44228
    reason: 已应用 WAF 规则缓解
    expires: 2025-06-30
    approved_by: security-team
    mitigation: WAF 规则 #1234
    risk_accepted: true

  - id: CVE-2023-12345
    reason: 依赖暂无修复版本
    expires: 2025-03-31
    approved_by: cto
    mitigation: 网络隔离
    risk_accepted: true
```

## 报告和度量

### 关键指标
```yaml
sast_metrics:
  - coverage: 95%  # 代码覆盖率
  - scan_frequency: per_commit
  - false_positive_rate: < 10%
  - mttr_high: 7 days
  - mttr_critical: 24 hours

dast_metrics:
  - coverage: 80%  # 应用覆盖率
  - scan_frequency: weekly
  - authenticated_scan: 100%
  - mttr_high: 14 days
  - mttr_critical: 48 hours

sca_metrics:
  - coverage: 100%  # 依赖覆盖率
  - scan_frequency: daily
  - sbom_availability: 100%
  - license_compliance: 100%
  - mttr_high: 7 days
  - mttr_critical: 48 hours
```

### 仪表板
```yaml
# Grafana Dashboard
panels:
  - title: 漏洞趋势
    type: graph
    targets:
      - expr: sum(vulnerabilities_total{severity="critical"})

  - title: 修复时间
    type: stat
    targets:
      - expr: avg(vulnerability_mttr_hours{severity="high"})

  - title: 扫描覆盖率
    type: gauge
    targets:
      - expr: (sum(scanned_repos) / sum(total_repos)) * 100
```

## 实施检查清单

- [ ] SAST 工具集成到 CI
- [ ] SCA 依赖扫描自动化
- [ ] DAST 定期扫描计划
- [ ] 质量门禁配置
- [ ] 漏洞修复流程
- [ ] 例外管理流程
- [ ] 报告和度量
- [ ] 安全培训
- [ ] 定期审查和优化
- [ ] SBOM 生成
- [ ] 依赖锁定
- [ ] IDE 插件部署

## 参考资料
- [OWASP Source Code Analysis](https://owasp.org/www-community/Source_Code_Analysis_Tools)
- [OWASP Web Security Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)
- [OWASP Dependency-Check](https://owasp.org/www-project-dependency-check/)
- [NIST National Vulnerability Database](https://nvd.nist.gov/)
