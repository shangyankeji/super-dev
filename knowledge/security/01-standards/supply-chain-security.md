---
id: supply-chain-security
title: 软件供应链安全
domain: security
category: 01-standards
difficulty: intermediate
tags: [agent, chain, checklist, security, supply, 实战代码示例, 常见陷阱, 最佳实践]
quality_score: 70
last_updated: 2026-06-15
---
# 软件供应链安全

## 概述
软件供应链攻击通过篡改构建流程、注入恶意依赖或利用已知漏洞来危害系统安全。本指南覆盖依赖扫描、SBOM、签名验证、Lock文件、CVE监控等完整供应链安全体系。

## 核心概念

### 1. 供应链攻击向量
- **依赖混淆(Dependency Confusion)**: 公共仓库中注册与内部包同名的恶意包
- **Typosquatting**: 注册与流行包名相似的恶意包(如requets vs requests)
- **恶意维护者**: 合法包的维护者被收买或账号被盗
- **构建系统入侵**: CI/CD流水线被注入恶意步骤
- **已知漏洞利用**: 使用含CVE的过期依赖

### 2. 防护层次
| 层次 | 防护措施 | 工具 |
|------|----------|------|
| 依赖选择 | 评估包的可信度 | Socket.dev/Snyk Advisor |
| 依赖锁定 | Lock文件固定版本和哈希 | pip freeze/poetry.lock/npm lockfile |
| 漏洞扫描 | 持续扫描已知CVE | Snyk/Trivy/Dependabot/OSV |
| SBOM | 生成软件物料清单 | Syft/CycloneDX/SPDX |
| 签名验证 | 验证包的完整性和来源 | Sigstore/cosign/pip --require-hashes |
| 构建安全 | 可重现构建/最小权限 | SLSA/GitHub Actions |

### 3. SLSA框架(Supply chain Levels for Software Artifacts)
- **Level 1**: 构建过程有文档记录
- **Level 2**: 版本控制和构建服务
- **Level 3**: 安全的构建平台,不可篡改的出处证明
- **Level 4**: 双人审核,密封可重现构建

## 实战代码示例

### 依赖锁定(Python)

```toml
# pyproject.toml — 指定版本范围
[project]
dependencies = [
    "fastapi>=0.100,<1.0",
    "pydantic>=2.0,<3.0",
    "httpx>=0.25,<1.0",
]
```

```bash
# 使用pip-compile生成精确锁文件
pip install pip-tools

# 生成requirements.txt(含哈希)
pip-compile --generate-hashes pyproject.toml -o requirements.txt

# 生成的requirements.txt包含哈希验证
# fastapi==0.109.0 \
#     --hash=sha256:abcdef... \
#     --hash=sha256:123456...

# 安装时验证哈希
pip install --require-hashes -r requirements.txt
```

```bash
# Poetry锁定
poetry lock
poetry install --no-root  # 精确安装lockfile中的版本

# uv锁定
uv lock
uv sync
```

### 依赖扫描CI集成

```yaml
# .github/workflows/security.yml
name: Supply Chain Security
on:
  push:
    branches: [main]
  pull_request:
  schedule:
    - cron: '0 8 * * 1'  # 每周一扫描

jobs:
  dependency-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Python依赖扫描
      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          scan-ref: '.'
          format: 'sarif'
          output: 'trivy-results.sarif'
          severity: 'HIGH,CRITICAL'

      - name: Upload Trivy scan results
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: 'trivy-results.sarif'

      # npm依赖审计
      - name: npm audit
        working-directory: frontend
        run: npm audit --audit-level=high

      # 使用OSV-Scanner
      - name: OSV Scanner
        uses: google/osv-scanner-action/osv-scanner-action@v1
        with:
          scan-args: |-
            --lockfile=requirements.txt
            --lockfile=frontend/package-lock.json

  sbom-generation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Generate SBOM
        uses: anchore/sbom-action@v0
        with:
          format: cyclonedx-json
          output-file: sbom.json

      - name: Upload SBOM
        uses: actions/upload-artifact@v4
        with:
          name: sbom
          path: sbom.json

  license-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - name: Check licenses
        run: |
          pip install pip-licenses
          pip install -e .
          pip-licenses --fail-on="GPL-3.0;AGPL-3.0" --format=json > licenses.json
```

### SBOM生成与验证

```bash
# 使用Syft生成SBOM
syft . -o cyclonedx-json > sbom.json
syft . -o spdx-json > sbom-spdx.json

# 使用Grype扫描SBOM中的漏洞
grype sbom:sbom.json --fail-on high

# Python原生SBOM
pip install cyclonedx-bom
cyclonedx-py environment -o sbom.json --format json
```

```python
# 程序化生成SBOM
from cyclonedx.model.bom import Bom
from cyclonedx.model.component import Component, ComponentType
from cyclonedx.output.json import JsonV1Dot5

def generate_sbom(requirements_file: str) -> str:
    """从requirements.txt生成SBOM"""
    bom = Bom()

    with open(requirements_file) as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            name, _, version = line.partition('==')
            if version:
                component = Component(
                    name=name.strip(),
                    version=version.strip(),
                    type=ComponentType.LIBRARY,
                )
                bom.components.add(component)

    output = JsonV1Dot5(bom)
    return output.output_as_string()
```

### 签名验证

```bash
# 使用cosign签名容器镜像
cosign sign --key cosign.key myregistry.com/myapp:v1.0

# 验证签名
cosign verify --key cosign.pub myregistry.com/myapp:v1.0

# 使用Sigstore无密钥签名(基于OIDC)
cosign sign --identity-token=$(gcloud auth print-identity-token) myregistry.com/myapp:v1.0
cosign verify --certificate-identity=user@example.com \
  --certificate-oidc-issuer=https://accounts.google.com \
  myregistry.com/myapp:v1.0
```

```yaml
# Kubernetes准入控制验证镜像签名
# Kyverno策略
apiVersion: kyverno.io/v1
kind: ClusterPolicy
metadata:
  name: verify-image-signature
spec:
  validationFailureAction: Enforce
  rules:
    - name: verify-cosign-signature
      match:
        resources:
          kinds:
            - Pod
      verifyImages:
        - imageReferences:
            - "myregistry.com/*"
          attestors:
            - entries:
                - keys:
                    publicKeys: |-
                      -----BEGIN PUBLIC KEY-----
                      MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE...
                      -----END PUBLIC KEY-----
```

### 依赖混淆防护

```ini
# pip.conf — 配置内部仓库优先
[global]
index-url = https://internal.pypi.example.com/simple/
extra-index-url = https://pypi.org/simple/

# 更安全: 只使用内部仓库(内部仓库代理外部包)
[global]
index-url = https://internal.pypi.example.com/simple/
```

```bash
# npm — 配置scope到内部仓库
# .npmrc
@myorg:registry=https://npm.internal.example.com/
registry=https://registry.npmjs.org/
```

```python
# 自动检测可疑依赖
import subprocess
import json

def audit_new_dependencies(requirements_before: str, requirements_after: str):
    """检测新增依赖是否可疑"""
    before = set(parse_requirements(requirements_before))
    after = set(parse_requirements(requirements_after))
    new_deps = after - before

    alerts = []
    for dep in new_deps:
        info = get_pypi_info(dep)
        if not info:
            alerts.append(f"Package {dep} not found on PyPI")
            continue

        # 检查可疑指标
        if info["downloads_last_month"] < 100:
            alerts.append(f"{dep}: Very low downloads ({info['downloads_last_month']})")
        if info["age_days"] < 30:
            alerts.append(f"{dep}: Very new package ({info['age_days']} days)")
        if info["maintainer_count"] == 1:
            alerts.append(f"{dep}: Single maintainer")

    return alerts
```

### Dependabot配置

```yaml
# .github/dependabot.yml
version: 2
updates:
  # Python依赖
  - package-ecosystem: "pip"
    directory: "/"
    schedule:
      interval: "weekly"
      day: "monday"
    open-pull-requests-limit: 10
    reviewers:
      - "security-team"
    labels:
      - "dependencies"
      - "security"
    # 自动合并小版本更新
    allow:
      - dependency-type: "direct"
    ignore:
      - dependency-name: "*"
        update-types: ["version-update:semver-major"]

  # npm依赖
  - package-ecosystem: "npm"
    directory: "/frontend"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 10

  # Docker基础镜像
  - package-ecosystem: "docker"
    directory: "/"
    schedule:
      interval: "weekly"

  # GitHub Actions
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "weekly"
```

## 最佳实践

### 1. 依赖管理
- 使用锁文件固定所有依赖(包括传递依赖)的精确版本
- 生产安装使用`--require-hashes`验证完整性
- 定期更新依赖(至少每月一次)
- 审核新增依赖(下载量/维护活跃度/许可证)

### 2. 漏洞管理
- CI中集成漏洞扫描(阻断高危/严重)
- 配置Dependabot/Renovate自动PR
- 建立CVE响应流程(48小时内评估Critical)
- 维护已知漏洞的例外清单(含理由和到期日)

### 3. SBOM实践
- 每次发布生成SBOM
- SBOM存储在制品仓库
- 定期用SBOM扫描新发现的漏洞
- 合规需求时提供SBOM给客户

### 4. 构建安全
- CI/CD使用最小权限(GITHUB_TOKEN scope限制)
- Pin GitHub Actions到commit SHA而非tag
- 构建环境隔离(不共享缓存)
- 审核CI/CD配置变更

### 5. 内部仓库
- 设置内部包仓库代理外部源
- 配置依赖混淆防护(scope/namespace)
- 内部包使用组织scope(如@myorg/package)
- 定期审计仓库中的包

## 常见陷阱

### 陷阱1: 不锁定传递依赖
```bash
# 错误: requirements.txt只列直接依赖
fastapi
pydantic

# 正确: 锁定全部依赖链
fastapi==0.109.0
pydantic==2.5.3
starlette==0.35.1
anyio==4.2.0
# ... 所有传递依赖
```

### 陷阱2: CI中使用@latest
```yaml
# 错误: Action可能被篡改
- uses: actions/checkout@main

# 正确: 锁定到commit SHA
- uses: actions/checkout@b4ffde65f46336ab88eb53be808477a3936bae11  # v4.1.1
```

### 陷阱3: 忽略开发依赖的安全
```bash
# 开发依赖也可能被利用(恶意的eslint插件/pytest插件)
# 对dev依赖同样需要审计和扫描
```

### 陷阱4: 只依赖自动扫描
```python
# 自动扫描只能检测已知CVE
# 零日攻击和恶意包需要人工审核
# 对于新增依赖,应该:
# 1. 检查GitHub Stars/活跃度
# 2. 检查维护者背景
# 3. 检查包的实际代码(尤其是postinstall脚本)
```

## Agent Checklist

### 依赖锁定
- [ ] 所有项目使用锁文件
- [ ] 锁文件包含哈希值
- [ ] 锁文件提交到版本控制
- [ ] CI中安装使用锁文件

### 漏洞扫描
- [ ] CI集成自动漏洞扫描
- [ ] 高危/严重漏洞阻断构建
- [ ] Dependabot/Renovate已配置
- [ ] CVE响应流程已建立

### SBOM与合规
- [ ] 发布时生成SBOM
- [ ] 许可证合规检查已集成
- [ ] SBOM存储在制品仓库
- [ ] 可按需提供SBOM

### 构建安全
- [ ] CI/CD使用最小权限
- [ ] GitHub Actions锁定到SHA
- [ ] 构建环境隔离
- [ ] CI配置变更需要审核

### 仓库安全
- [ ] 配置内部包仓库
- [ ] 依赖混淆防护已启用
- [ ] 新增依赖有审核流程
- [ ] 定期审计依赖列表
