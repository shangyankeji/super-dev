---
id: auth-sso-playbook
title: 认证与 SSO 实战手册（Keycloak/Auth0/OIDC）
domain: security
category: 02-playbooks
difficulty: advanced
tags: [authentication, authorization, sso, keycloak, auth0, oidc, oauth2, pkce, jwt, rbac, saml, enterprise, identity]
quality_score: 95
maintainer: security-team@superdev.com
last_updated: 2026-06-15
---

# 认证与 SSO 实战手册

> 基于 [RFC 9700 OAuth 2.0 Security BCP](https://datatracker.ietf.org/doc/html/rfc9700) + [Keycloak 生产配置](https://www.keycloak.org/server/configuration-production) + [Duende 2025 Web Security](https://duendesoftware.com/blog/20250805-best-practices-of-web-application-security-in-2025)

## 2025 认证标准：Authorization Code + PKCE

```
RFC 9700 (2025年1月) 的核心建议：
- 禁止 implicit flow（已弃用）
- 所有客户端必须用 Authorization Code + PKCE
- redirect_uri 精确匹配（不允许通配符）
- token 最小权限 scope
```

### PKCE 流程
```typescript
// 前端：生成 code_verifier + code_challenge
const codeVerifier = generateRandomString(128);
const codeChallenge = base64Url(sha256(codeVerifier));

// 1. 重定向到授权服务器（带 code_challenge）
window.location = `${authServer}/auth?` + new URLSearchParams({
  response_type: 'code',
  client_id: clientId,
  redirect_uri: 'https://app.example.com/callback',
  code_challenge: codeChallenge,
  code_challenge_method: 'S256',
  state: randomState,           // CSRF 防护
  scope: 'openid profile email', // 最小 scope
});

// 2. 回调：用 code + code_verifier 换 token
const tokenResponse = await fetch(`${authServer}/token`, {
  method: 'POST',
  body: new URLSearchParams({
    grant_type: 'authorization_code',
    code: codeFromCallback,
    redirect_uri: 'https://app.example.com/callback',
    client_id: clientId,
    code_verifier: codeVerifier,  // 证明是同一个客户端
  }),
});
const { access_token, id_token, refresh_token } = await tokenResponse.json();
```

## Keycloak vs Auth0 选择

| 维度 | Keycloak | Auth0 |
|------|----------|-------|
| 部署 | 自托管（开源） | 托管 SaaS |
| 成本 | 免费（服务器成本） | 按用户收费 |
| 协议 | OIDC + SAML | OIDC + SAML |
| 定制 | 完全可控 | 有限（Rules/Actions） |
| 运维 | 自己管 HA/备份 | 零运维 |
| 适合 | 大企业/合规要求 | 快速上线/SaaS |

## Keycloak 生产配置

### 高可用部署
```yaml
# K8s 部署：≥2 副本 + 共享 DB
apiVersion: apps/v1
kind: Deployment
metadata:
  name: keycloak
spec:
  replicas: 3                    # 至少 3 个副本
  template:
    spec:
      containers:
      - name: keycloak
        image: quay.io/keycloak/keycloak:latest
        args: ["start"]
        env:
        - name: KC_DB            # 共享 PostgreSQL
          value: postgres
        - name: KC_DB_URL
          value: jdbc:postgresql://pg:5432/keycloak
        - name: KC_HOSTNAME
          value: auth.example.com
        - name: KC_PROXY          # behind reverse proxy
          value: edge
        - name: KC_HTTP_ENABLED
          value: "true"
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
```

### 生产必做
- [ ] HTTPS（TLS 终止在反向代理）
- [ ] 共享数据库（PostgreSQL，不用内嵌 H2）
- [ ] ≥2 副本（HA）
- [ ] 健康检查 + 自动重启
- [ ] 备份 realm 配置（`kcadm.sh export`）
- [ ] 关闭临时账号（admin/admin）
- [ ] 配置 SMTP（密码重置邮件）
- [ ] 密码策略（≥12 字符 + 特殊字符）

## JWT 验证（服务端）

```python
# 每个服务验证 JWT（不信任网关，纵深防御）
from jose import jwt

def verify_token(token: str):
    # 1. 获取 JWKS（Keycloak 公钥）
    jwks = get_jwks(f"{auth_server}/protocol/openid-connect/certs")
    # 2. 验证签名 + 过期 + 受众
    payload = jwt.decode(
        token,
        jwks,
        algorithms=["RS256"],       # 只允许 RS256
        audience="account",          # 验证受众
        issuer=f"{auth_server}/realms/myrealm",  # 验证签发者
    )
    return payload  # 含 sub(用户ID), roles, scope
```

## RBAC 权限模型

```python
# Keycloak realm roles → 应用权限映射
REALM_ROLES = {
    "admin": ["read:any", "write:any", "delete:any", "manage:users"],
    "manager": ["read:team", "write:team"],
    "user": ["read:own", "write:own"],
}

def check_permission(user_roles: list, required: str):
    for role in user_roles:
        if required in REALM_ROLES.get(role, []):
            return True
    raise ForbiddenError(f"Missing permission: {required}")

# 使用
@app.delete("/api/users/{id}")
def delete_user(id, token=Depends(verify_token)):
    check_permission(token["realm_access"]["roles"], "delete:any")
    return db.delete(User, id)
```

## 生产检查清单
- [ ] Authorization Code + PKCE（不用 implicit）
- [ ] redirect_uri 精确匹配（不通配）
- [ ] token 最小 scope
- [ ] JWT 用 RS256（不用 HS256）
- [ ] 服务端验证 JWT 签名 + 过期 + 受众 + 签发者
- [ ] refresh token 轮换（每次刷新换新 token）
- [ ] Keycloak ≥2 副本 + 共享 DB
- [ ] 关闭临时账号 + 配置密码策略
- [ ] 备份 realm 配置
- [ ] MFA（敏感操作）
