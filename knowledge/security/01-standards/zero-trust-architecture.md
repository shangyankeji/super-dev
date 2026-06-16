---
id: zero-trust-architecture
title: 零信任架构指南
domain: security
category: 01-standards
difficulty: intermediate
tags: [agent, architecture, checklist, security, trust, zero, 实战代码示例, 常见陷阱]
quality_score: 70
last_updated: 2026-06-15
---
# 零信任架构指南

## 概述
零信任(Zero Trust)是一种安全模型,核心原则是"永不信任,始终验证"。传统边界安全模型假设内网可信,而零信任消除了隐式信任,对每次访问都进行身份验证、授权和加密。本指南覆盖零信任原则、实现路径、mTLS、身份验证和微分段。

## 核心概念

### 1. 零信任核心原则
- **永不信任,始终验证**: 无论请求来自内网还是外网,一视同仁
- **最小权限**: 仅授予完成工作所需的最低权限
- **假设已被入侵**: 设计系统时假定攻击者已在内部
- **显式验证**: 基于所有可用数据点(身份/位置/设备/行为)验证
- **微分段**: 将网络划分为小的隔离区域,限制横向移动

### 2. 零信任架构组件
| 组件 | 功能 | 实现技术 |
|------|------|----------|
| 身份提供者(IdP) | 集中身份管理和认证 | Okta/Auth0/Keycloak/Azure AD |
| 策略引擎 | 访问决策 | OPA/Cedar/Zanzibar |
| 策略执行点(PEP) | 拦截和执行策略 | API Gateway/Service Mesh Sidecar |
| 设备信任评估 | 验证设备安全状态 | MDM/EDR/设备证书 |
| 网络微分段 | 隔离网络区域 | Kubernetes NetworkPolicy/Calico |
| 加密通信 | 端到端加密 | mTLS/WireGuard |

### 3. 零信任成熟度模型
- **Level 1**: 基础 — 强身份认证(MFA)、基础网络分段
- **Level 2**: 进阶 — 设备信任评估、基于策略的访问控制
- **Level 3**: 优化 — 持续验证、行为分析、自动化响应
- **Level 4**: 全面 — 完整的微分段、端到端加密、AI驱动的风险评估

## 实战代码示例

### mTLS服务间认证

```python
# FastAPI mTLS服务端配置
import ssl
import uvicorn

def create_ssl_context():
    """创建mTLS SSL上下文"""
    ssl_context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    ssl_context.load_cert_chain(
        certfile="/certs/server.crt",
        keyfile="/certs/server.key",
    )
    # 要求客户端证书
    ssl_context.verify_mode = ssl.CERT_REQUIRED
    ssl_context.load_verify_locations("/certs/ca.crt")
    return ssl_context

if __name__ == "__main__":
    uvicorn.run(
        "app:app",
        host="0.0.0.0",
        port=8443,
        ssl_certfile="/certs/server.crt",
        ssl_keyfile="/certs/server.key",
        ssl_ca_certs="/certs/ca.crt",
        ssl_cert_reqs=ssl.CERT_REQUIRED,
    )

# 从客户端证书提取身份
from fastapi import Request

async def get_client_identity(request: Request) -> dict:
    """从mTLS证书提取客户端身份"""
    cert = request.scope.get("tls_client_cert")
    if not cert:
        raise HTTPException(401, "Client certificate required")

    subject = dict(x[0] for x in cert.get("subject", []))
    return {
        "service_name": subject.get("commonName"),
        "organization": subject.get("organizationName"),
        "serial_number": cert.get("serialNumber"),
    }
```

```python
# mTLS客户端调用
import httpx

class SecureServiceClient:
    """带mTLS的服务客户端"""

    def __init__(self, service_url: str):
        self.client = httpx.AsyncClient(
            base_url=service_url,
            cert=("/certs/client.crt", "/certs/client.key"),
            verify="/certs/ca.crt",
            timeout=10.0,
        )

    async def call(self, path: str, method: str = "GET", **kwargs):
        response = await self.client.request(method, path, **kwargs)
        response.raise_for_status()
        return response.json()
```

### 基于OPA的策略引擎

```rego
# policy.rego — Open Policy Agent策略定义

package authz

import future.keywords.if
import future.keywords.in

# 默认拒绝
default allow := false

# 管理员允许所有操作
allow if {
    input.user.roles[_] == "admin"
}

# 用户只能访问自己的资源
allow if {
    input.method == "GET"
    input.path = ["api", "users", user_id]
    input.user.id == user_id
}

# 编辑者可以创建和更新
allow if {
    input.method in ["POST", "PUT", "PATCH"]
    input.user.roles[_] == "editor"
    not is_admin_path(input.path)
}

# 服务间调用需要正确的服务身份
allow if {
    input.source_service == "order-service"
    input.target_service == "inventory-service"
    input.method in ["GET", "POST"]
    startswith(input.path_string, "/api/inventory")
}

# 工作时间限制(敏感操作)
allow if {
    input.action == "export_data"
    is_business_hours(input.timestamp)
    input.user.roles[_] == "data_analyst"
    input.user.device_trust_score >= 80
}

is_admin_path(path) if {
    path[0] == "api"
    path[1] == "admin"
}

is_business_hours(ts) if {
    hour := time.clock(time.parse_rfc3339_ns(ts))[0]
    hour >= 9
    hour < 18
}
```

```python
# FastAPI OPA集成
import httpx

class OPAClient:
    def __init__(self, opa_url: str = "http://opa:8181"):
        self.url = opa_url
        self.client = httpx.AsyncClient()

    async def check_access(self, input_data: dict) -> bool:
        response = await self.client.post(
            f"{self.url}/v1/data/authz/allow",
            json={"input": input_data},
        )
        result = response.json()
        return result.get("result", False)

opa = OPAClient()

class ZeroTrustMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request, call_next):
        auth = await authenticate(request)

        # 构建OPA策略输入
        policy_input = {
            "user": {
                "id": auth.user_id,
                "roles": auth.roles,
                "device_trust_score": auth.device_score,
            },
            "method": request.method,
            "path": request.url.path.strip("/").split("/"),
            "path_string": request.url.path,
            "source_ip": request.client.host,
            "timestamp": datetime.utcnow().isoformat(),
            "source_service": request.headers.get("X-Source-Service"),
            "target_service": "current-service",
        }

        allowed = await opa.check_access(policy_input)
        if not allowed:
            logger.warning("Access denied by policy", extra=policy_input)
            raise HTTPException(403, "Access denied by policy")

        return await call_next(request)
```

### Kubernetes网络策略(微分段)

```yaml
# 默认拒绝所有入站流量
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-ingress
  namespace: production
spec:
  podSelector: {}
  policyTypes:
    - Ingress

---
# 只允许API Gateway访问后端服务
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-api-gateway
  namespace: production
spec:
  podSelector:
    matchLabels:
      app: backend-api
  policyTypes:
    - Ingress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: api-gateway
      ports:
        - port: 8080
          protocol: TCP

---
# 只允许后端服务访问数据库
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-backend-to-db
  namespace: production
spec:
  podSelector:
    matchLabels:
      app: postgresql
  policyTypes:
    - Ingress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              tier: backend
      ports:
        - port: 5432
          protocol: TCP

---
# 限制出站流量(只允许访问必要服务)
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: restrict-egress
  namespace: production
spec:
  podSelector:
    matchLabels:
      app: backend-api
  policyTypes:
    - Egress
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: postgresql
      ports:
        - port: 5432
    - to:
        - podSelector:
            matchLabels:
              app: redis
      ports:
        - port: 6379
    - to:
        - namespaceSelector: {}
          podSelector:
            matchLabels:
              k8s-app: kube-dns
      ports:
        - port: 53
          protocol: UDP
```

### 持续验证与行为分析

```python
# 持续风险评估
from dataclasses import dataclass
from enum import Enum

class RiskLevel(str, Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"

@dataclass
class RiskSignals:
    """访问风险信号"""
    ip_reputation_score: float      # 0-100
    device_trust_score: float       # 0-100
    user_behavior_score: float      # 0-100
    geo_anomaly: bool               # 地理位置异常
    time_anomaly: bool              # 时间异常
    impossible_travel: bool         # 不可能的旅行
    failed_attempts_1h: int         # 1小时内失败次数

class RiskEngine:
    """零信任风险评估引擎"""

    def evaluate(self, signals: RiskSignals) -> tuple[RiskLevel, float]:
        """评估访问风险"""
        score = 100.0

        # IP声誉
        score -= max(0, (100 - signals.ip_reputation_score)) * 0.2

        # 设备信任
        score -= max(0, (100 - signals.device_trust_score)) * 0.3

        # 行为评分
        score -= max(0, (100 - signals.user_behavior_score)) * 0.2

        # 异常标志
        if signals.geo_anomaly:
            score -= 15
        if signals.time_anomaly:
            score -= 10
        if signals.impossible_travel:
            score -= 30

        # 失败尝试
        score -= min(30, signals.failed_attempts_1h * 5)

        score = max(0, score)

        if score >= 80:
            return RiskLevel.LOW, score
        elif score >= 60:
            return RiskLevel.MEDIUM, score
        elif score >= 40:
            return RiskLevel.HIGH, score
        else:
            return RiskLevel.CRITICAL, score

    def get_required_actions(self, risk_level: RiskLevel) -> list[str]:
        """根据风险等级确定需要的额外验证"""
        actions = {
            RiskLevel.LOW: [],
            RiskLevel.MEDIUM: ["step_up_mfa"],
            RiskLevel.HIGH: ["step_up_mfa", "manager_approval"],
            RiskLevel.CRITICAL: ["block_access", "alert_security_team"],
        }
        return actions[risk_level]
```

### Service Mesh mTLS(Istio)

```yaml
# Istio PeerAuthentication — 强制mTLS
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: production
spec:
  mtls:
    mode: STRICT  # 所有服务间通信必须mTLS

---
# Istio AuthorizationPolicy — 服务间授权
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: order-service-policy
  namespace: production
spec:
  selector:
    matchLabels:
      app: order-service
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/production/sa/api-gateway"]
      to:
        - operation:
            methods: ["GET", "POST"]
            paths: ["/api/orders/*"]
    - from:
        - source:
            principals: ["cluster.local/ns/production/sa/payment-service"]
      to:
        - operation:
            methods: ["GET"]
            paths: ["/api/orders/*/status"]
```

## 最佳实践

### 1. 身份管理
- 集中身份管理(使用IdP)
- 所有用户和服务都有唯一身份
- 强制MFA(至少TOTP,推荐Passkey)
- 服务间使用SPIFFE/mTLS身份
- 定期审计和轮换凭证

### 2. 网络安全
- 默认拒绝所有流量(Network Policy)
- 微分段隔离不同服务和环境
- 服务间通信全部加密(mTLS)
- 使用Service Mesh简化mTLS管理
- 出站流量也要控制(防数据外泄)

### 3. 访问控制
- 基于策略的访问控制(OPA/Cedar)
- 最小权限原则(Just-In-Time/Just-Enough)
- 持续验证(不仅在登录时)
- 上下文感知(设备/位置/时间/行为)

### 4. 监控与响应
- 记录所有访问决策(允许和拒绝)
- 实时行为分析(检测异常模式)
- 自动化响应(风险升高→要求额外验证)
- 定期红队演练(测试横向移动能力)

### 5. 渐进式实施
- 从最敏感的系统开始
- 先监控模式,再强制模式
- 分阶段: 身份→网络→数据→设备
- 每个阶段充分测试和培训

## 常见陷阱

### 陷阱1: 只在边界实施
```
# 错误: 只在API Gateway做认证,内部服务之间无验证
# 攻击者一旦进入内网就可以自由横向移动

# 正确: 每个服务都独立验证身份和授权
# 使用Service Mesh(Istio)自动mTLS
```

### 陷阱2: 过粗的网络分段
```yaml
# 错误: 按namespace分段,同namespace内无限制
# 正确: 按服务级别分段
# 每个服务只能访问它需要的其他服务和端口
```

### 陷阱3: 忽略出站流量
```yaml
# 错误: 只限制入站,出站不管
# 攻击者可以从被入侵的Pod向外部发送数据

# 正确: 出站流量同样限制
# 只允许访问已知的外部服务(API/DB等)
```

### 陷阱4: 静态信任评估
```python
# 错误: 只在登录时评估风险
# 整个会话期间使用相同的信任级别

# 正确: 持续评估
# 每次敏感操作都重新评估风险
# 发现异常时要求额外验证或降级权限
```

## Agent Checklist

### 身份与认证
- [ ] 集中身份管理(IdP)已部署
- [ ] 所有用户启用MFA
- [ ] 服务间使用mTLS/SPIFFE身份
- [ ] 凭证自动轮换

### 网络分段
- [ ] 默认拒绝网络策略已应用
- [ ] 服务间通信加密(mTLS)
- [ ] 出站流量受控
- [ ] 环境间完全隔离(dev/staging/prod)

### 策略引擎
- [ ] 基于策略的访问控制已实现
- [ ] 策略定义为代码(Git管理)
- [ ] 策略变更有审计追踪
- [ ] 策略测试覆盖

### 持续验证
- [ ] 风险评估引擎已部署
- [ ] 行为异常检测已启用
- [ ] 自适应认证已实现
- [ ] 安全事件自动响应
