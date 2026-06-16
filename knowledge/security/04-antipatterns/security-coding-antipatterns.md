---
id: security-coding-antipatterns
title: 安全编码反模式库
domain: security
category: 04-antipatterns
difficulty: intermediate
tags: [antipatterns, coding, concatenation, hardcoded, password, plaintext, secrets, security]
quality_score: 70
last_updated: 2026-06-15
---
# 安全编码反模式库

> 覆盖 OWASP Top 10 高频漏洞对应的编码反模式，每个反模式包含描述、漏洞代码、修复代码和检测工具。

---

## 反模式 1：硬编码密钥 (Hardcoded Secrets)

### 描述

将 API 密钥、数据库密码、Token 等敏感凭据直接写入源代码或配置文件并提交到版本控制。一旦代码泄露（公开仓库、离职员工、供应链攻击），攻击者可直接获取凭据访问后端资源。

### 风险等级

**严重 (Critical)** — CWE-798

### 漏洞代码

```python
# bad: 密钥直接硬编码
import boto3

AWS_ACCESS_KEY = "AKIAIOSFODNN7EXAMPLE"
AWS_SECRET_KEY = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"

client = boto3.client(
    "s3",
    aws_access_key_id=AWS_ACCESS_KEY,
    aws_secret_access_key=AWS_SECRET_KEY,
)
```

```javascript
// bad: 数据库连接串包含明文密码
const mongoose = require("mongoose");
mongoose.connect("mongodb://admin:P@ssw0rd123@prod-db:27017/myapp");
```

### 修复代码

```python
# good: 从环境变量或密钥管理服务获取
import os
import boto3

client = boto3.client(
    "s3",
    aws_access_key_id=os.environ["AWS_ACCESS_KEY_ID"],
    aws_secret_access_key=os.environ["AWS_SECRET_ACCESS_KEY"],
)
```

```javascript
// good: 使用环境变量 + dotenv（.env 加入 .gitignore）
require("dotenv").config();
const mongoose = require("mongoose");
mongoose.connect(process.env.MONGODB_URI);
```

### 检测工具

| 工具 | 类型 | 说明 |
|------|------|------|
| **git-secrets** | Pre-commit Hook | AWS 官方密钥扫描 |
| **TruffleHog** | 历史扫描 | 扫描 Git 历史中的高熵字符串 |
| **detect-secrets** (Yelp) | CI 集成 | 基于插件的秘密检测 |
| **GitHub Secret Scanning** | SaaS | 自动扫描公开仓库 |
| **HashiCorp Vault** | 运行时 | 动态秘密发放与轮转 |

---

## 反模式 2：SQL 拼接 (SQL Concatenation)

### 描述

将用户输入直接拼接进 SQL 语句，导致 SQL 注入攻击。攻击者可绕过认证、窃取数据、删库或执行系统命令。

### 风险等级

**严重 (Critical)** — CWE-89

### 漏洞代码

```python
# bad: 字符串拼接构造 SQL
def get_user(username):
    query = f"SELECT * FROM users WHERE username = '{username}'"
    cursor.execute(query)
    return cursor.fetchone()

# 攻击输入: username = "' OR '1'='1' --"
# 实际执行: SELECT * FROM users WHERE username = '' OR '1'='1' --'
```

```java
// bad: Java 中的字符串拼接
String query = "SELECT * FROM orders WHERE user_id = " + userId;
Statement stmt = connection.createStatement();
ResultSet rs = stmt.executeQuery(query);
```

### 修复代码

```python
# good: 参数化查询
def get_user(username):
    query = "SELECT * FROM users WHERE username = %s"
    cursor.execute(query, (username,))
    return cursor.fetchone()
```

```python
# good: ORM（SQLAlchemy）
user = session.query(User).filter(User.username == username).first()
```

```java
// good: PreparedStatement
String query = "SELECT * FROM orders WHERE user_id = ?";
PreparedStatement pstmt = connection.prepareStatement(query);
pstmt.setInt(1, userId);
ResultSet rs = pstmt.executeQuery();
```

### 检测工具

| 工具 | 类型 | 说明 |
|------|------|------|
| **SQLMap** | DAST | 自动化 SQL 注入检测与利用 |
| **Bandit** | SAST (Python) | 检测 SQL 拼接模式 |
| **SonarQube** | SAST | 多语言 SQL 注入规则 |
| **Semgrep** | SAST | 自定义规则检测拼接模式 |

---

## 反模式 3：明文密码存储 (Plaintext Password Storage)

### 描述

将用户密码以明文或简单哈希（MD5/SHA1）存储在数据库中。数据库泄露后攻击者可直接获取或通过彩虹表快速破解所有用户密码。

### 风险等级

**严重 (Critical)** — CWE-256, CWE-916

### 漏洞代码

```python
# bad: 明文存储
def register(username, password):
    db.execute("INSERT INTO users (username, password) VALUES (%s, %s)",
               (username, password))

# bad: 使用 MD5（无盐值，可彩虹表破解）
import hashlib
def register(username, password):
    hashed = hashlib.md5(password.encode()).hexdigest()
    db.execute("INSERT INTO users (username, password) VALUES (%s, %s)",
               (username, hashed))
```

### 修复代码

```python
# good: 使用 bcrypt（自带盐值 + 自适应代价因子）
import bcrypt

def register(username, password):
    salt = bcrypt.gensalt(rounds=12)
    hashed = bcrypt.hashpw(password.encode("utf-8"), salt)
    db.execute("INSERT INTO users (username, password_hash) VALUES (%s, %s)",
               (username, hashed.decode("utf-8")))

def verify(username, password):
    row = db.fetchone("SELECT password_hash FROM users WHERE username = %s", (username,))
    return bcrypt.checkpw(password.encode("utf-8"), row["password_hash"].encode("utf-8"))
```

```python
# good: 使用 argon2（OWASP 推荐首选）
from argon2 import PasswordHasher

ph = PasswordHasher(time_cost=3, memory_cost=65536, parallelism=4)

def register(username, password):
    hashed = ph.hash(password)
    db.execute("INSERT INTO users (username, password_hash) VALUES (%s, %s)",
               (username, hashed))

def verify(username, password):
    row = db.fetchone("SELECT password_hash FROM users WHERE username = %s", (username,))
    return ph.verify(row["password_hash"], password)
```

### 检测工具

| 工具 | 类型 | 说明 |
|------|------|------|
| **Semgrep** | SAST | 检测 hashlib.md5/sha1 用于密码场景 |
| **SonarQube** | SAST | 弱哈希使用规则 |
| **CrackStation** | 离线测试 | 验证哈希抗彩虹表强度 |

---

## 反模式 4：过度权限 (Excessive Privileges)

### 描述

应用使用数据库 root 账户运行、服务账号拥有 admin 权限、IAM 策略使用 `*` 通配符。违反最小权限原则，一旦应用被攻破，攻击面覆盖全部资源。

### 风险等级

**高 (High)** — CWE-250, CWE-269

### 漏洞代码

```yaml
# bad: AWS IAM 策略给予所有权限
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": "*",
    "Resource": "*"
  }]
}
```

```python
# bad: 应用使用 root 连接数据库
DB_USER = "root"
DB_PASS = "rootpassword"
connection = psycopg2.connect(host="db", user=DB_USER, password=DB_PASS, dbname="app")
```

### 修复代码

```yaml
# good: 最小权限 IAM 策略
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": [
      "s3:GetObject",
      "s3:PutObject"
    ],
    "Resource": "arn:aws:s3:::my-bucket/uploads/*"
  }]
}
```

```sql
-- good: 创建专用应用账户并限制权限
CREATE USER app_service WITH PASSWORD 'generated_password';
GRANT SELECT, INSERT, UPDATE ON orders, products, users TO app_service;
-- 不授予 DELETE、DROP、ALTER 等破坏性权限
```

### 检测工具

| 工具 | 类型 | 说明 |
|------|------|------|
| **IAM Access Analyzer** | AWS 原生 | 检测过度开放的 IAM 策略 |
| **Prowler** | 云安全审计 | CIS Benchmark 检查 |
| **Checkov** | IaC 扫描 | Terraform/CloudFormation 权限审计 |
| **pgAudit** | 数据库 | PostgreSQL 权限使用审计 |

---

## 反模式 5：不安全的反序列化 (Insecure Deserialization)

### 描述

直接反序列化不可信数据（用户输入、网络传入、外部文件），攻击者可构造恶意 payload 实现远程代码执行（RCE）。Java 的 `ObjectInputStream`、Python 的 `pickle`、PHP 的 `unserialize` 是高危入口。

### 风险等级

**严重 (Critical)** — CWE-502

### 漏洞代码

```python
# bad: 直接 pickle 反序列化用户数据
import pickle
import base64

def load_session(cookie_value):
    data = base64.b64decode(cookie_value)
    return pickle.loads(data)  # RCE 风险！

# 攻击者可构造 pickle payload 执行任意命令
```

```java
// bad: 直接反序列化网络流
ObjectInputStream ois = new ObjectInputStream(socket.getInputStream());
Object obj = ois.readObject();  // 可触发 gadget chain RCE
```

### 修复代码

```python
# good: 使用安全的序列化格式
import json
import hmac
import hashlib

SECRET_KEY = os.environ["SESSION_SECRET"]

def load_session(cookie_value):
    payload, signature = cookie_value.rsplit(".", 1)
    expected_sig = hmac.new(SECRET_KEY.encode(), payload.encode(), hashlib.sha256).hexdigest()
    if not hmac.compare_digest(signature, expected_sig):
        raise ValueError("Invalid session signature")
    return json.loads(base64.b64decode(payload))
```

```java
// good: 使用白名单过滤的反序列化
ObjectInputStream ois = new ValidatingObjectInputStream(inputStream);
((ValidatingObjectInputStream) ois).accept(AllowedClass.class);
Object obj = ois.readObject();
```

### 检测工具

| 工具 | 类型 | 说明 |
|------|------|------|
| **Bandit** | SAST (Python) | B301 规则检测 pickle.loads |
| **ysoserial** | 渗透测试 | Java 反序列化 payload 生成 |
| **Semgrep** | SAST | 多语言反序列化规则 |

---

## 反模式 6：CORS 配置错误 (CORS Misconfiguration)

### 描述

将 `Access-Control-Allow-Origin` 设置为 `*` 或动态反射请求的 Origin 而不验证，允许任意恶意网站发起跨域请求读取响应数据，可窃取用户敏感信息。

### 风险等级

**高 (High)** — CWE-942

### 漏洞代码

```python
# bad: 允许所有来源
from flask import Flask
from flask_cors import CORS

app = Flask(__name__)
CORS(app, origins="*", supports_credentials=True)
# 同时设置 * 和 credentials=True 浏览器会拒绝，
# 但开发者常改为反射 Origin 头来"修复"
```

```javascript
// bad: 动态反射 Origin（等同于 *）
app.use((req, res, next) => {
  res.setHeader("Access-Control-Allow-Origin", req.headers.origin);
  res.setHeader("Access-Control-Allow-Credentials", "true");
  next();
});
```

### 修复代码

```python
# good: 白名单校验
ALLOWED_ORIGINS = [
    "https://app.example.com",
    "https://admin.example.com",
]

CORS(app, origins=ALLOWED_ORIGINS, supports_credentials=True)
```

```javascript
// good: 显式白名单
const allowedOrigins = new Set([
  "https://app.example.com",
  "https://admin.example.com",
]);

app.use((req, res, next) => {
  const origin = req.headers.origin;
  if (allowedOrigins.has(origin)) {
    res.setHeader("Access-Control-Allow-Origin", origin);
    res.setHeader("Access-Control-Allow-Credentials", "true");
  }
  next();
});
```

### 检测工具

| 工具 | 类型 | 说明 |
|------|------|------|
| **OWASP ZAP** | DAST | CORS 策略检测 |
| **Burp Suite** | DAST | CORS 配置审计 |
| **ESLint Plugin Security** | SAST (JS) | 检测宽松 CORS 配置 |

---

## 反模式 7：不验证 JWT (Unvalidated JWT)

### 描述

接收 JWT 后不验证签名、不检查过期时间、信任 `alg: none`、或使用对称密钥但密钥太弱。攻击者可伪造令牌冒充任意用户。

### 风险等级

**严重 (Critical)** — CWE-345, CWE-347

### 漏洞代码

```python
# bad: 不验证签名，只解码
import jwt

def get_current_user(token):
    payload = jwt.decode(token, options={"verify_signature": False})
    return payload["user_id"]  # 攻击者可伪造任意 user_id
```

```javascript
// bad: 允许 alg: none
const decoded = jwt.verify(token, secret, { algorithms: ["HS256", "none"] });
```

```python
# bad: 密钥太弱
SECRET = "secret"  # 可被暴力破解
token = jwt.encode(payload, SECRET, algorithm="HS256")
```

### 修复代码

```python
# good: 完整验证
import jwt
from datetime import datetime, timezone

PUBLIC_KEY = open("public.pem").read()

def get_current_user(token):
    try:
        payload = jwt.decode(
            token,
            PUBLIC_KEY,
            algorithms=["RS256"],         # 只允许特定算法
            options={
                "verify_exp": True,       # 验证过期
                "verify_iss": True,       # 验证签发者
                "verify_aud": True,       # 验证受众
            },
            issuer="https://auth.example.com",
            audience="https://api.example.com",
        )
        return payload["user_id"]
    except jwt.ExpiredSignatureError:
        raise AuthError("Token expired")
    except jwt.InvalidTokenError:
        raise AuthError("Invalid token")
```

### 检测工具

| 工具 | 类型 | 说明 |
|------|------|------|
| **jwt_tool** | 渗透测试 | JWT 漏洞全面检测 |
| **Semgrep** | SAST | JWT 验证规则 |
| **Burp JWT Editor** | DAST | JWT 篡改与测试 |

---

## 反模式 8：使用弱加密算法 (Weak Cryptography)

### 描述

使用已被攻破的加密算法（DES、3DES、RC4、MD5 用于完整性校验、SHA1 用于证书签名）或 ECB 模式加密。数据可被解密或伪造。

### 风险等级

**高 (High)** — CWE-327, CWE-328

### 漏洞代码

```python
# bad: 使用 DES 加密
from Crypto.Cipher import DES
cipher = DES.new(b"8bytekey", DES.MODE_ECB)
encrypted = cipher.encrypt(b"sensitiv")  # ECB 模式 + DES = 双重问题
```

```python
# bad: MD5 用于数据完整性校验
import hashlib
checksum = hashlib.md5(file_data).hexdigest()
# MD5 存在碰撞攻击，不能保证完整性
```

```java
// bad: SHA1 用于证书签名
MessageDigest md = MessageDigest.getInstance("SHA-1");
byte[] digest = md.digest(data);
```

### 修复代码

```python
# good: 使用 AES-256-GCM（认证加密）
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
import os

key = AESGCM.generate_key(bit_length=256)
aesgcm = AESGCM(key)
nonce = os.urandom(12)
ciphertext = aesgcm.encrypt(nonce, plaintext, associated_data)
```

```python
# good: SHA-256 用于完整性校验
import hashlib
checksum = hashlib.sha256(file_data).hexdigest()
```

```python
# good: HMAC 用于消息认证
import hmac, hashlib
mac = hmac.new(secret_key, message, hashlib.sha256).hexdigest()
```

### 检测工具

| 工具 | 类型 | 说明 |
|------|------|------|
| **Bandit** | SAST (Python) | B303/B304 弱加密检测 |
| **SonarQube** | SAST | 弱加密算法规则 |
| **ssl-enum-ciphers** (nmap) | 网络扫描 | TLS 弱密码套件检测 |
| **testssl.sh** | 网络扫描 | TLS 配置完整测试 |

---

## 综合防护矩阵

| 反模式 | OWASP 类别 | CWE | 自动化检测可行性 | 修复优先级 |
|--------|-----------|-----|------------------|-----------|
| 硬编码密钥 | A07:2021 | 798 | 高 | P0 |
| SQL 拼接 | A03:2021 | 89 | 高 | P0 |
| 明文密码 | A02:2021 | 256/916 | 中 | P0 |
| 过度权限 | A01:2021 | 250/269 | 中 | P1 |
| 不安全反序列化 | A08:2021 | 502 | 中 | P0 |
| CORS 错误 | A05:2021 | 942 | 高 | P1 |
| JWT 不验证 | A07:2021 | 345/347 | 中 | P0 |
| 弱加密算法 | A02:2021 | 327/328 | 高 | P1 |

---

## Agent Checklist

- [ ] 所有反模式均包含描述、漏洞代码、修复代码和检测工具四部分
- [ ] 代码示例覆盖 Python、JavaScript/Node.js、Java 等主流语言
- [ ] CWE 编号和 OWASP 分类准确对应
- [ ] 修复方案符合当前行业最佳实践（bcrypt/argon2、参数化查询、AES-GCM 等）
- [ ] 检测工具列表包含 SAST、DAST、SCA 多种类型
- [ ] 综合防护矩阵提供优先级排序
- [ ] 文件行数 >= 300 行
