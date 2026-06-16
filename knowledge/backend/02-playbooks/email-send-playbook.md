---
id: email-send-playbook
title: 邮件发送实战手册（Transactional Email）
domain: backend
category: 02-playbooks
difficulty: intermediate
tags: [email, smtp, sendgrid, ses, transactional, template, queue, retry, dkim, spf, enterprise]
quality_score: 91
maintainer: platform-team@superdev.com
last_updated: 2026-06-15
---

# 邮件发送实战手册（Transactional Email）

## 服务商选择

| 服务 | 特点 | 适合 |
|------|------|------|
| AWS SES | 便宜（$0.10/1000封） | 大量发送、AWS 生态 |
| SendGrid | 功能丰富（模板/A-B 测试） | SaaS 产品 |
| Resend | 开发者体验好 | 新项目快速集成 |
| Postmark | 事务邮件送达率高 | 关键邮件（验证码/订单） |
| 自建 SMTP | 可控但维护重 | 合规要求 |

## 异步队列发送（必须！）

```python
# ❌ 同步发送（请求阻塞 3-5 秒等待 SMTP）
@app.post("/register")
def register(data):
    user = create_user(data)
    send_email(user.email, "Welcome!")  # SMTP 3-5s → 用户等
    return user

# ✅ 异步队列（请求毫秒返回，后台发邮件）
@app.post("/register")
def register(data):
    user = create_user(data)
    enqueue_job("send_welcome_email", user.id)  # 入队毫秒
    return user

@celery.task(bind=True, max_retries=3)
def send_welcome_email(self, user_id):
    user = db.query(User).get(user_id)
    try:
        email_service.send(
            to=user.email,
            template="welcome",
            data={"name": user.name},
        )
    except SMTPException as e:
        raise self.retry(exc=e, countdown=60 * (2 ** self.request.retries))
        # 指数退避重试：60s → 120s → 240s
```

## HTML 邮件模板

```html
<!-- ✅ 邮件兼容 HTML（不是现代 CSS！ Outlook 用 Word 引擎） -->
<table role="presentation" width="100%" cellpadding="0" cellspacing="0"
       style="background-color:#f4f4f5;padding:24px;font-family:Arial,sans-serif;">
  <tr><td align="center">
    <table role="presentation" width="600" style="background:#ffffff;border-radius:8px;padding:32px;">
      <tr><td>
        <h1 style="color:#1a1a1a;font-size:24px;margin:0 0 16px;">
          Welcome to Acme!
        </h1>
        <p style="color:#52525b;line-height:1.6;">
          Hi {{name}},<br><br>
          Thanks for signing up. Click below to verify your email:
        </p>
        <!-- 按钮：用 table 包裹（邮件兼容） -->
        <table role="presentation" cellpadding="0" cellspacing="0" style="margin:24px 0;">
          <tr><td style="background:#2563eb;border-radius:6px;">
            <a href="{{verify_url}}" style="display:inline-block;padding:12px 32px;
               color:#ffffff;text-decoration:none;font-weight:600;">
              Verify Email
            </a>
          </td></tr>
        </table>
      </td></tr>
    </table>
  </td></tr>
</table>
```

### 邮件 HTML 规则
- 用 `<table>` 布局（不用 flex/grid）
- 内联样式（不用 `<style>` 标签 + class）
- 用 `role="presentation"`（屏幕阅读器跳过布局 table）
- 宽度 600px（移动端友好）
- 图片用绝对 URL（`https://cdn.example.com/logo.png`）
- 纯文本版本（multipart，防垃圾邮件过滤）

## DNS 认证（送达率必须）

### SPF（发件服务器授权）
```
# DNS TXT 记录
example.com.  TXT  "v=spf1 include:_spf.google.com include:amazonses.com ~all"
# ~all = 软失败（不严格拒绝），-all = 硬失败
```

### DKIM（邮件签名验证）
```
# DNS TXT 记录（由邮件服务商提供）
selector._domainkey.example.com.  TXT  "v=DKIM1; k=rsa; p=MIGfMA0GCS..."
# 收件方用公钥验证邮件签名
```

### DMARC（策略汇总）
```
# DNS TXT 记录
_dmarc.example.com.  TXT  "v=DMARC1; p=quarantine; rua=mailto:dmarc@example.com"
# p=none（只报告）/ quarantine（隔离）/ reject（拒绝）
```

## 生产检查清单
- [ ] 异步队列发送（不阻塞请求）
- [ ] 指数退避重试（3 次：60s/120s/240s）
- [ ] SPF + DKIM + DMARC DNS 配置
- [ ] HTML 邮件用 table 布局（不用 flex/grid）
- [ ] 内联 CSS（不用 style 标签）
- [ ] 同时发纯文本版本（multipart）
- [ ] 图片用绝对 CDN URL
- [ ] 取消订阅链接（营销邮件必须）
- [ ] 退信处理（bounce/complaint → 标记不发）
- [ ] 速率限制（防 SMTP 服务商封禁）
- [ ] 邮件模板预览（开发环境真实渲染）
- [ ] 发送日志 + 送达率监控
