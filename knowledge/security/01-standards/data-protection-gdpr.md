---
id: data-protection-gdpr
title: 数据保护与GDPR合规
domain: security
category: 01-standards
difficulty: intermediate
tags: [agent, checklist, data, gdpr, protection, security, 实战代码示例, 常见陷阱]
quality_score: 70
last_updated: 2026-06-15
---
# 数据保护与GDPR合规

## 概述
数据保护是现代软件系统的法律和道德义务。本指南覆盖GDPR(通用数据保护条例)核心要求、数据分类、加密、脱敏、审计日志和DPIA(数据保护影响评估),帮助团队构建合规的数据处理系统。

## 核心概念

### 1. GDPR核心原则
- **合法性、公正性、透明性**: 有合法依据处理数据,对用户透明
- **目的限制**: 只为明确声明的目的收集数据
- **数据最小化**: 只收集必要的最少数据
- **准确性**: 保持数据准确并及时更新
- **存储限制**: 不超过必要期限保留数据
- **完整性与保密性**: 适当的安全措施保护数据
- **问责制**: 能证明合规

### 2. 数据分类

| 分类 | 示例 | 保护级别 | 处理要求 |
|------|------|----------|----------|
| 公开数据 | 公司名称、产品描述 | 基础 | 无特殊要求 |
| 内部数据 | 内部文档、配置 | 标准 | 访问控制 |
| 机密数据 | 客户信息、财务数据 | 高 | 加密+访问控制+审计 |
| 敏感个人数据 | 健康/种族/宗教/性取向 | 最高 | 明确同意+加密+DPIA |
| PII(个人身份信息) | 姓名/邮箱/手机/身份证 | 高 | 加密+最小化+删除权 |

### 3. 用户权利(GDPR)
- **访问权**: 用户可以获取其数据副本
- **更正权**: 用户可以修正不准确的数据
- **删除权(被遗忘权)**: 用户可以要求删除其数据
- **可携带权**: 用户可以获取结构化机器可读格式的数据
- **限制处理权**: 用户可以限制数据的处理方式
- **反对权**: 用户可以反对某些类型的处理
- **不受自动化决策约束的权利**: 拒绝纯自动化决策

### 4. 合规框架对比
| 法规 | 地区 | 罚款 | 关键差异 |
|------|------|------|----------|
| GDPR | 欧盟/EEA | 最高2000万欧元或4%全球营收 | 最严格,域外适用 |
| CCPA/CPRA | 加州 | 每项违规$7,500 | 侧重消费者权利 |
| PIPL | 中国 | 最高5000万元或5%营收 | 数据本地化要求 |
| LGPD | 巴西 | 最高营收2% | 类似GDPR |

## 实战代码示例

### 数据加密

```python
# 字段级加密
from cryptography.fernet import Fernet
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
from cryptography.hazmat.primitives import hashes
import base64
import os

class FieldEncryptor:
    """字段级加密器"""

    def __init__(self, master_key: str):
        # 从主密钥派生加密密钥
        kdf = PBKDF2HMAC(
            algorithm=hashes.SHA256(),
            length=32,
            salt=b"field-encryption-salt",  # 实际应用中使用随机盐并存储
            iterations=480000,
        )
        key = base64.urlsafe_b64encode(kdf.derive(master_key.encode()))
        self.cipher = Fernet(key)

    def encrypt(self, plaintext: str) -> str:
        """加密字段值"""
        return self.cipher.encrypt(plaintext.encode()).decode()

    def decrypt(self, ciphertext: str) -> str:
        """解密字段值"""
        return self.cipher.decrypt(ciphertext.encode()).decode()

# SQLAlchemy加密列
from sqlalchemy import TypeDecorator, String

class EncryptedString(TypeDecorator):
    """透明加密的字符串列"""
    impl = String
    cache_ok = True

    def __init__(self, encryptor: FieldEncryptor, *args, **kwargs):
        self.encryptor = encryptor
        super().__init__(*args, **kwargs)

    def process_bind_param(self, value, dialect):
        if value is not None:
            return self.encryptor.encrypt(value)
        return value

    def process_result_value(self, value, dialect):
        if value is not None:
            return self.encryptor.decrypt(value)
        return value

# 使用
encryptor = FieldEncryptor(os.environ["ENCRYPTION_KEY"])

class User(Base):
    __tablename__ = "users"
    id = Column(Integer, primary_key=True)
    name = Column(String(100))                                    # 非敏感,不加密
    email = Column(EncryptedString(encryptor, length=500))        # PII,加密
    phone = Column(EncryptedString(encryptor, length=500))        # PII,加密
    email_hash = Column(String(64), index=True)                   # 用于查询的哈希
```

### 数据脱敏

```python
# 数据脱敏工具
import re
import hashlib

class DataMasker:
    """数据脱敏工具"""

    @staticmethod
    def mask_email(email: str) -> str:
        """邮箱脱敏: user@example.com → u***@example.com"""
        if not email or "@" not in email:
            return "***"
        local, domain = email.split("@", 1)
        if len(local) <= 1:
            return f"*@{domain}"
        return f"{local[0]}{'*' * (len(local) - 1)}@{domain}"

    @staticmethod
    def mask_phone(phone: str) -> str:
        """手机号脱敏: 13812345678 → 138****5678"""
        digits = re.sub(r'\D', '', phone)
        if len(digits) >= 7:
            return digits[:3] + "****" + digits[-4:]
        return "****"

    @staticmethod
    def mask_id_card(id_card: str) -> str:
        """身份证脱敏: 110101199001011234 → 110101****1234"""
        if len(id_card) >= 8:
            return id_card[:6] + "****" + id_card[-4:]
        return "****"

    @staticmethod
    def mask_name(name: str) -> str:
        """姓名脱敏: 张三 → 张* / Alice Bob → A*** Bob"""
        if not name:
            return "***"
        parts = name.split()
        if len(parts) == 1:
            return name[0] + "*" * (len(name) - 1)
        return parts[0][0] + "***" + " " + parts[-1]

    @staticmethod
    def pseudonymize(value: str, pepper: str) -> str:
        """假名化: 可逆映射(需要pepper)"""
        return hashlib.sha256(f"{pepper}:{value}".encode()).hexdigest()[:16]

# API响应自动脱敏
from pydantic import BaseModel, field_serializer

class UserPublicResponse(BaseModel):
    """面向公众的用户响应(脱敏)"""
    id: int
    name: str
    email: str
    phone: str | None = None

    @field_serializer("email")
    def mask_email_field(self, v: str) -> str:
        return DataMasker.mask_email(v)

    @field_serializer("phone")
    def mask_phone_field(self, v: str | None) -> str | None:
        if v:
            return DataMasker.mask_phone(v)
        return None
```

### 审计日志

```python
# GDPR审计日志系统
from datetime import datetime
from enum import Enum
from pydantic import BaseModel
import json

class AuditAction(str, Enum):
    CREATE = "create"
    READ = "read"
    UPDATE = "update"
    DELETE = "delete"
    EXPORT = "export"
    CONSENT_GRANTED = "consent_granted"
    CONSENT_WITHDRAWN = "consent_withdrawn"
    DATA_ACCESS_REQUEST = "data_access_request"
    DATA_DELETION_REQUEST = "data_deletion_request"

class AuditLog(BaseModel):
    timestamp: datetime
    action: AuditAction
    actor_id: str
    actor_type: str  # "user" | "system" | "admin"
    resource_type: str
    resource_id: str
    data_categories: list[str]
    legal_basis: str
    ip_address: str
    user_agent: str | None = None
    details: dict = {}

class GDPRAuditLogger:
    """GDPR合规审计日志记录器"""

    def __init__(self, storage):
        self.storage = storage

    async def log(self, audit: AuditLog):
        """记录审计日志(不可修改)"""
        record = audit.model_dump()
        record["timestamp"] = record["timestamp"].isoformat()
        # 追加写入,不可修改
        await self.storage.append(record)
        # 同时发送到SIEM
        await self.siem.send(record)

    async def log_data_access(
        self,
        actor_id: str,
        resource_type: str,
        resource_id: str,
        data_fields: list[str],
        purpose: str,
        request: Request,
    ):
        await self.log(AuditLog(
            timestamp=datetime.utcnow(),
            action=AuditAction.READ,
            actor_id=actor_id,
            actor_type="user",
            resource_type=resource_type,
            resource_id=resource_id,
            data_categories=classify_fields(data_fields),
            legal_basis=purpose,
            ip_address=request.client.host,
            user_agent=request.headers.get("user-agent"),
            details={"fields_accessed": data_fields},
        ))

# FastAPI审计中间件
audit_logger = GDPRAuditLogger(storage)

@app.get("/api/users/{user_id}")
async def get_user(user_id: int, auth = Depends(authenticate), request: Request):
    user = await user_repo.get(user_id)

    await audit_logger.log_data_access(
        actor_id=str(auth.user_id),
        resource_type="user",
        resource_id=str(user_id),
        data_fields=["name", "email", "phone"],
        purpose="user_profile_view",
        request=request,
    )

    return user
```

### 用户权利实现

```python
# GDPR用户权利API
class GDPRService:
    """GDPR权利服务"""

    async def handle_access_request(self, user_id: int) -> dict:
        """处理数据访问请求(SAR)"""
        # 收集用户所有数据
        data = {
            "personal_info": await user_repo.get(user_id),
            "orders": await order_repo.get_by_user(user_id),
            "activity_logs": await activity_repo.get_by_user(user_id),
            "preferences": await preference_repo.get(user_id),
            "consent_records": await consent_repo.get_by_user(user_id),
        }

        # 审计日志
        await audit_logger.log(AuditLog(
            action=AuditAction.DATA_ACCESS_REQUEST,
            actor_id=str(user_id),
            resource_type="user_data",
            resource_id=str(user_id),
            data_categories=["all"],
            legal_basis="gdpr_article_15",
        ))

        return data

    async def handle_deletion_request(self, user_id: int) -> dict:
        """处理删除请求(被遗忘权)"""
        results = {"deleted": [], "retained": [], "errors": []}

        # 1. 删除非必须保留的数据
        deletable_data = [
            ("user_profile", user_repo.delete_profile),
            ("preferences", preference_repo.delete),
            ("activity_logs", activity_repo.delete_by_user),
            ("sessions", session_repo.delete_by_user),
        ]

        for name, delete_fn in deletable_data:
            try:
                await delete_fn(user_id)
                results["deleted"].append(name)
            except Exception as e:
                results["errors"].append({"data": name, "error": str(e)})

        # 2. 标记必须保留的数据(法律义务)
        retained = [
            ("financial_records", "Tax law requires 7-year retention"),
            ("audit_logs", "Regulatory compliance"),
        ]
        results["retained"] = [
            {"data": name, "reason": reason, "retention_until": calculate_retention_end(name)}
            for name, reason in retained
        ]

        # 3. 匿名化而非删除(保留统计价值)
        await user_repo.anonymize(user_id)

        # 审计日志
        await audit_logger.log(AuditLog(
            action=AuditAction.DATA_DELETION_REQUEST,
            actor_id=str(user_id),
            resource_type="user_data",
            resource_id=str(user_id),
            data_categories=["all"],
            legal_basis="gdpr_article_17",
            details=results,
        ))

        return results

    async def handle_portability_request(self, user_id: int) -> bytes:
        """处理数据可携带请求"""
        data = await self.handle_access_request(user_id)
        # 返回结构化JSON格式
        return json.dumps(data, indent=2, default=str, ensure_ascii=False).encode()

# API端点
@app.post("/api/gdpr/access-request")
async def request_data_access(auth = Depends(authenticate)):
    """提交数据访问请求(SAR)"""
    data = await gdpr_service.handle_access_request(auth.user_id)
    return data

@app.post("/api/gdpr/deletion-request")
async def request_data_deletion(auth = Depends(authenticate)):
    """提交数据删除请求"""
    result = await gdpr_service.handle_deletion_request(auth.user_id)
    return result

@app.get("/api/gdpr/export")
async def export_data(auth = Depends(authenticate)):
    """导出个人数据(可携带权)"""
    data = await gdpr_service.handle_portability_request(auth.user_id)
    return Response(
        content=data,
        media_type="application/json",
        headers={"Content-Disposition": "attachment; filename=my_data.json"},
    )
```

### 同意管理

```python
# 同意管理系统
class ConsentPurpose(str, Enum):
    ESSENTIAL = "essential"           # 必要功能
    ANALYTICS = "analytics"           # 数据分析
    MARKETING = "marketing"           # 营销推广
    PERSONALIZATION = "personalization"  # 个性化
    THIRD_PARTY = "third_party"       # 第三方共享

class ConsentRecord(BaseModel):
    user_id: int
    purpose: ConsentPurpose
    granted: bool
    granted_at: datetime | None
    withdrawn_at: datetime | None
    version: str                      # 隐私政策版本
    method: str                       # "web_form" | "api" | "email"
    ip_address: str

class ConsentManager:
    async def grant_consent(self, user_id: int, purpose: ConsentPurpose, request: Request):
        record = ConsentRecord(
            user_id=user_id,
            purpose=purpose,
            granted=True,
            granted_at=datetime.utcnow(),
            withdrawn_at=None,
            version=CURRENT_PRIVACY_POLICY_VERSION,
            method="web_form",
            ip_address=request.client.host,
        )
        await consent_repo.save(record)
        await audit_logger.log(AuditLog(
            action=AuditAction.CONSENT_GRANTED,
            actor_id=str(user_id),
            resource_type="consent",
            resource_id=purpose,
            legal_basis="gdpr_article_6_1_a",
        ))

    async def check_consent(self, user_id: int, purpose: ConsentPurpose) -> bool:
        """检查用户是否同意特定目的"""
        if purpose == ConsentPurpose.ESSENTIAL:
            return True  # 必要处理不需要同意
        record = await consent_repo.get_latest(user_id, purpose)
        return record is not None and record.granted
```

## 最佳实践

### 1. 数据最小化
- 只收集业务必需的数据
- 注册时不要求不必要的字段
- 定期审查数据收集范围
- 日志中不包含PII(或脱敏)

### 2. 加密策略
- 传输加密: TLS 1.2+全覆盖
- 存储加密: 敏感字段加密(AES-256)
- 数据库加密: 透明数据加密(TDE)
- 密钥管理: 使用KMS(AWS KMS/Vault)

### 3. 数据保留
- 定义每类数据的保留期限
- 自动清理过期数据
- 法律要求的数据标记保留原因
- 保留策略文档化并定期审查

### 4. 隐私设计(Privacy by Design)
- 默认设置最高隐私保护
- 隐私考虑融入开发流程
- DPIA在处理高风险数据前执行
- 第三方数据处理需签署DPA

### 5. 事件响应
- 72小时内向监管机构通报数据泄露
- 高风险泄露需通知受影响用户
- 准备数据泄露应急响应流程
- 定期演练数据泄露场景

## 常见陷阱

### 陷阱1: 日志中泄露PII
```python
# 错误
logger.info(f"User login: {user.email}, IP: {ip}")
logger.error(f"Payment failed for card {card_number}")

# 正确
logger.info("User login", user_id=user.id, ip=mask_ip(ip))
logger.error("Payment failed", user_id=user.id, card_last4=card[-4:])
```

### 陷阱2: 删除数据不彻底
```python
# 错误: 只删除主表,备份和缓存中还有数据
await user_repo.delete(user_id)

# 正确: 删除所有存储位置
await user_repo.delete(user_id)
await cache.delete(f"user:{user_id}")
await search_index.delete(f"user_{user_id}")
await cdn.purge(f"/avatars/{user_id}/*")
# 备份中的数据:标记为待清理,下次备份时排除
```

### 陷阱3: 同意管理不完善
```python
# 错误: 使用预勾选的同意框
# 错误: 一次性同意所有用途
# 错误: 不提供撤回同意的途径

# 正确:
# - 每个目的单独同意
# - 默认不勾选
# - 撤回同意与授予同意一样容易
# - 记录同意的版本、时间、方式
```

### 陷阱4: 跨境数据传输不合规
```python
# 错误: 直接将EU用户数据存储到美国服务器
# 正确:
# - 使用标准合同条款(SCCs)
# - 评估目标国数据保护水平
# - 考虑数据本地化方案
# - 使用EU区域的云服务
```

## Agent Checklist

### 数据分类与保护
- [ ] 所有数据字段已分类(公开/内部/机密/敏感)
- [ ] PII字段已识别并标注
- [ ] 敏感数据字段级加密已实现
- [ ] 数据脱敏在日志/测试/非生产环境中应用

### 用户权利
- [ ] 数据访问请求(SAR)接口已实现
- [ ] 数据删除/匿名化接口已实现
- [ ] 数据导出(可携带权)接口已实现
- [ ] 同意管理系统已实现

### 合规管理
- [ ] 隐私政策已发布并可访问
- [ ] 审计日志覆盖所有数据操作
- [ ] 数据保留策略已定义并自动执行
- [ ] 数据泄露应急响应流程已建立

### 技术措施
- [ ] 全链路TLS加密
- [ ] 密钥管理使用KMS
- [ ] 第三方数据处理有DPA协议
- [ ] DPIA已对高风险处理执行
