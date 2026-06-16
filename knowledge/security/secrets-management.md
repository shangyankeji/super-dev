---
id: secrets-management
title: 密钥管理完整方案
domain: security
category: secrets-management.md
difficulty: intermediate
tags: [kubernetes, management, secrets, security, 中的使用, 密钥在, 密钥生命周期管理, 密钥管理]
quality_score: 70
last_updated: 2026-06-15
---
# 密钥管理完整方案

## 概述
密钥管理(Secrets Management)是保护敏感信息(密码、API 密钥、证书等)的系统化方法,防止泄露和未授权访问。

## 密钥类型

### 1. 基础设施密钥
- 数据库凭证
- API 密钥
- SSH 密钥
- TLS/SSL 证书
- 云服务访问密钥(AWS Access Key, GCP Service Account)

### 2. 应用程序密钥
- 加密密钥
- 签名密钥
- Session 密钥
- OAuth Client Secret

### 3. 业务敏感信息
- 第三方服务凭证
- 支付网关密钥
- 监控和日志访问令牌

## 密钥生命周期管理

### 1. 生成(Generation)
```bash
# 强密码生成
openssl rand -base64 32

# RSA 密钥对
openssl genrsa -out private.pem 2048
openssl rsa -in private.pem -pubout -out public.pem

# SSH 密钥
ssh-keygen -t ed25519 -C "deploy@company.com" -f deploy_key

# API 密钥
uuidgen | tr -d '-' | lower
```

### 2. 存储(Storage)
```yaml
# HashiCorp Vault 示例
vault kv put secret/myapp \
  db_password="$(openssl rand -base64 32)" \
  api_key="$(uuidgen)" \
  tls_cert=@"cert.pem"
```

### 3. 分发(Distribution)
```yaml
# Kubernetes Secret
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
data:
  db_password: <base64-encoded>
---
# 从 Vault 注入
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
      - name: app
        env:
        - name: DB_PASSWORD
          valueFrom:
            secretKeyRef:
              name: vault-secret
              key: db_password
```

### 4. 轮换(Rotation)
```yaml
# Vault 动态数据库凭证
vault write database/config/myapp \
  plugin_name=postgresql-database-plugin \
  allowed_roles="myapp-role" \
  connection_url="postgresql://{{username}}:{{password}}@db:5432/myapp"

vault write database/roles/myapp-role \
  db_name=myapp \
  creation_statements="CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}';" \
  default_ttl="1h" \
  max_ttl="24h"
```

### 5. 吊销(Revocation)
```bash
# 吊销 Vault 令牌
vault token revoke <token>

# 吊销 PKI 证书
vault write pki/revoke serial_number=<serial>
```

## 密钥管理解决方案

### 1. HashiCorp Vault

#### 架构
```
Client -> Vault Agent -> Vault Server -> Storage Backend
                                |
                                -> Auth Method (AppRole, K8s, OIDC)
                                -> Secrets Engine (KV, Database, PKI)
```

#### 核心功能
```hcl
# 启用审计日志
audit {
  type = "file"
  options = {
    file_path = "/var/log/vault/audit.log"
  }
}

# AppRole 认证
auth "approle" {
  path = "approle"
}

# 数据库密钥引擎
secrets "database" {
  path = "database"
}
```

#### 最佳实践
```yaml
# 1. 使用命名空间隔离
vault namespace create team-a
vault namespace create team-b

# 2. 细粒度策略
path "secret/data/team-a/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

path "secret/data/team-b/*" {
  capabilities = ["deny"]
}

# 3. 响应包装
vault kv get -wrap-ttl=60s secret/myapp
```

### 2. AWS Secrets Manager

```python
import boto3
import json

client = boto3.client('secretsmanager')

# 存储密钥
response = client.create_secret(
    Name='myapp/db-password',
    SecretString=json.dumps({
        'username': 'admin',
        'password': 'secure-password',
        'host': 'db.example.com',
        'port': 5432
    }),
    Tags=[
        {'Key': 'Environment', 'Value': 'production'},
        {'Key': 'Application', 'Value': 'myapp'}
    ]
)

# 自动轮换
response = client.rotate_secret(
    SecretId='myapp/db-password',
    RotationLambdaARN='arn:aws:lambda:region:account:function:rotate',
    RotationRules={
        'AutomaticallyAfterDays': 30
    }
)
```

### 3. Azure Key Vault

```csharp
// C# 示例
using Azure.Identity;
using Azure.Security.KeyVault.Secrets;

var client = new SecretClient(
    new Uri("https://myvault.vault.azure.net/"),
    new DefaultAzureCredential()
);

// 存储密钥
await client.SetSecretAsync("db-password", "secure-password");

// 读取密钥
KeyVaultSecret secret = await client.GetSecretAsync("db-password");
string password = secret.Value;
```

### 4. Google Secret Manager

```python
from google.cloud import secretmanager

client = secretmanager.SecretManagerServiceClient()

# 创建密钥
parent = f"projects/{project_id}"
response = client.create_secret(
    request={
        "parent": parent,
        "secret_id": "api-key",
        "secret": {"replication": {"automatic": {}}}
    }
)

# 添加版本
response = client.add_secret_version(
    request={
        "parent": response.name,
        "payload": {"data": b"my-secret-api-key"}
    }
)
```

## 密钥在 CI/CD 中的使用

### GitLab CI
```yaml
# .gitlab-ci.yml
variables:
  # 从 Vault 获取
  VAULT_ADDR: "https://vault.company.com"

before_script:
  # Vault 认证
  - export VAULT_TOKEN=$(vault write -field=token auth/jwt/login jwt=$CI_JOB_JWT)

deploy:
  stage: deploy
  script:
    - export DB_PASSWORD=$(vault kv get -field=password secret/myapp/db)
    - kubectl create secret generic app-secrets --from-literal=db-password=$DB_PASSWORD
```

### GitHub Actions
```yaml
name: Deploy
on: [push]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v3

      - name: Import Secrets
        uses: hashicorp/vault-action@v2
        with:
          url: https://vault.company.com
          role: myapp
          method: jwt
          secrets: |
            secret/data/myapp db_password | DB_PASSWORD
            secret/data/myapp api_key | API_KEY

      - name: Deploy
        run: |
          echo "Deploying with secrets..."
          kubectl apply -f k8s/
```

### Jenkins Pipeline
```groovy
pipeline {
  agent any

  stages {
    stage('Deploy') {
      steps {
        script {
          // 从 Vault 读取
          withVault(
            vaultBaseUrl: 'https://vault.company.com',
            credentialId: 'vault-approle',
            secrets: [
              [path: 'secret/myapp', secretValues: [
                [envVar: 'DB_PASSWORD', vaultKey: 'db_password']
              ]]
            ]
          ) {
            sh 'kubectl create secret generic app-secrets --from-literal=db-password=$DB_PASSWORD'
          }
        }
      }
    }
  }
}
```

## Kubernetes 密钥管理

### 1. 原生 Secret
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
stringData:
  db_password: "secure-password"  # 自动 base64 编码
```

### 2. External Secrets Operator
```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: app-secrets
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: vault-backend
    kind: ClusterSecretStore
  target:
    name: app-secrets
  data:
    - secretKey: db_password
      remoteRef:
        key: secret/myapp
        property: db_password
```

### 3. Sealed Secrets
```bash
# 加密 Secret
kubeseal --format=yaml < secret.yaml > sealed-secret.yaml

# 部署加密后的 Secret
kubectl apply -f sealed-secret.yaml
```

## 密钥泄露防护

### 1. 预提交检测
```bash
# git-secrets
git secrets --register-aws
git secrets --scan

# Gitleaks
gitleaks detect --source . --config gitleaks.toml

# TruffleHog
trufflehog git file://. --only-verified
```

### 2. 运行时检测
```python
# 密钥泄露检测规则
patterns = [
    r'(?i)aws_access_key_id\s*=\s*[A-Z0-9]{20}',
    r'(?i)aws_secret_access_key\s*=\s*[A-Za-z0-9/+=]{40}',
    r'(?i)password\s*=\s*["\'][^"\']+["\']',
    r'(?i)api_key\s*=\s*["\'][^"\']+["\']',
    r'-----BEGIN (?:RSA |)PRIVATE KEY-----'
]

def scan_for_secrets(content):
    for pattern in patterns:
        if re.search(pattern, content):
            alert_security_team(pattern)
```

### 3. Git 历史清理
```bash
# BFG Repo-Cleaner
bfg --replace-text passwords.txt my-repo.git

# git-filter-repo
git filter-repo --invert-paths --path secrets.env
```

## 密钥轮换策略

### 1. 自动轮换
```yaml
# Vault 数据库轮换
vault write database/roles/app \
  db_name=postgres \
  creation_statements="CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}';" \
  revocation_statements="DROP ROLE \"{{name}}\";" \
  default_ttl="1h" \
  max_ttl="24h"
```

### 2. 零停机轮换
```yaml
# 蓝绿密钥策略
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets-blue
data:
  db_password: <current-password>
---
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets-green
data:
  db_password: <new-password>
```

### 3. 渐进式轮换
```python
def rotate_secret(secret_name):
    # 1. 生成新密钥
    new_secret = generate_secret()

    # 2. 写入新版本
    add_secret_version(secret_name, new_secret)

    # 3. 监控应用健康
    if not health_check():
        # 回滚到旧版本
        rollback_secret(secret_name)
        return

    # 4. 标记旧版本为过期
    expire_old_versions(secret_name, keep_last=2)
```

## 审计和合规

### 1. 审计日志
```json
{
  "timestamp": "2025-03-20T10:00:00Z",
  "action": "read",
  "secret_path": "secret/myapp/db",
  "actor": "app-service",
  "ip_address": "10.0.1.100",
  "user_agent": "vault-client/1.0",
  "success": true,
  "response_code": 200
}
```

### 2. 访问控制
```hcl
# Vault 策略示例
# 开发者只读访问开发环境
path "secret/data/dev/*" {
  capabilities = ["read", "list"]
}

# 运维完全访问生产环境
path "secret/data/prod/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

# 审计员只读访问所有
path "sys/audit" {
  capabilities = ["read", "list"]
}
```

### 3. 合规检查
```yaml
# 密钥合规规则
policies:
  - name: ensure-secrets-encrypted
    resource: aws.secretsmanager
    filters:
      - type: value
        key: KmsKeyId
        value: absent

  - name: ensure-rotation-enabled
    resource: aws.secretsmanager
    filters:
      - type: value
        key: RotationEnabled
        value: false

  - name: no-secrets-in-env
    resource: k8s.deployment
    filters:
      - type: env-var
        key: PASSWORD
        value: not-null
```

## 灾难恢复

### 1. 备份策略
```bash
# Vault 备份
vault operator raft snapshot save backup.snap

# 恢复
vault operator raft snapshot restore backup.snap
```

### 2. 多区域复制
```hcl
# Vault 复制配置
replication {
  mode = "primary"
  primary_cluster_addr = "https://primary.vault:8201"

  performance_replication {
    paths = ["secret/data/global/*"]
  }
}
```

### 3. 密钥托管
```yaml
# Shamir Secret Sharing
recovery_shares: 5
recovery_threshold: 3

# 5 人持有密钥分片,至少 3 人才能恢复
```

## 安全最佳实践

### 1. 最小权限
- 应用仅获取所需密钥
- 使用细粒度策略
- 定期审查权限

### 2. 加密传输和存储
- TLS 传输加密
- 密钥静态加密
- 信封加密

### 3. 审计追踪
- 记录所有访问
- 实时告警异常
- 定期审计日志

### 4. 高可用
- 多副本部署
- 自动故障转移
- 定期演练恢复

### 5. 密钥隔离
- 按环境隔离(Dev/Staging/Prod)
- 按团队隔离
- 按应用隔离

## 实施检查清单

- [ ] 选择密钥管理方案(Vault/云原生)
- [ ] 建立命名规范
- [ ] 配置访问控制策略
- [ ] 实现自动轮换
- [ ] 集成 CI/CD 流水线
- [ ] 部署密钥泄露检测
- [ ] 启用审计日志
- [ ] 配置告警通知
- [ ] 定期备份
- [ ] 制定灾难恢复计划
- [ ] 安全培训
- [ ] 定期安全审计

## 参考资料
- [HashiCorp Vault Documentation](https://www.vaultproject.io/docs)
- [AWS Secrets Manager Best Practices](https://docs.aws.amazon.com/secretsmanager/latest/userguide/best-practices.html)
- [OWASP Secrets Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)
- [Kubernetes Secrets Management](https://kubernetes.io/docs/concepts/configuration/secret/)
