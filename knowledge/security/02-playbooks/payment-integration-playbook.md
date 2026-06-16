---
id: payment-integration-playbook
title: 支付集成实战手册（Stripe）
domain: security
category: 02-playbooks
difficulty: advanced
tags: [payment, stripe, idempotency, webhook, billing, subscription, pci-dss, charge, refund, enterprise, money]
quality_score: 94
maintainer: platform-team@superdev.com
last_updated: 2026-06-15
---

# 支付集成实战手册（Stripe）

> 基于 [Stripe 官方文档](https://docs.stripe.com/webhooks) + [Hookdeck Webhook Guide](https://hookdeck.com/webhooks/platforms/guide-to-stripe-webhooks-features-and-best-practices) + [Digital Applied 2026 Guide](https://www.digitalapplied.com/blog/stripe-payment-integration-developer-guide-2026)

## 幂等性（防止重复扣款）

```python
import uuid

# ❌ 无幂等键——网络超时重试 = 重复扣款
@app.post("/charge")
def charge(customer_id, amount):
    return stripe.PaymentIntent.create(
        amount=amount,
        currency="usd",
        customer=customer_id,
    )  # 超时重试 → 可能创建两个 PaymentIntent！

# ✅ 幂等键——同一个键 24h 内只执行一次
@app.post("/charge")
def charge(customer_id, amount):
    idempotency_key = str(uuid.uuid4())  # 存 DB，重试用同一个 key
    return stripe.PaymentIntent.create(
        amount=amount,
        currency="usd",
        customer=customer_id,
    ), stripe_headers={"Idempotency-Key": idempotency_key}
    # Stripe 24h 内对同 key 返回缓存结果（不重复扣款）
```

## Webhook 处理

### 签名验证（必须！）
```python
@app.post("/webhooks/stripe")
async def stripe_webhook(request):
    payload = await request.body()
    sig_header = request.headers.get("stripe-signature")

    # ✅ 验证签名（防伪造）
    try:
        event = stripe.Webhook.construct_event(
            payload, sig_header, WEBHOOK_SECRET
        )
    except stripe.error.SignatureVerificationError:
        raise HTTPException(400, "Invalid signature")

    # ✅ 幂等处理（Stripe 可能重发同一事件）
    if redis.exists(f"stripe_event:{event.id}"):
        return {"status": "already_processed"}  # 去重
    redis.setex(f"stripe_event:{event.id}", 86400, "1")

    # 处理事件
    match event.type:
        case "payment_intent.succeeded":
            handle_payment_success(event.data.object)
        case "invoice.paid":
            handle_subscription_renewed(event.data.object)
        case "customer.subscription.deleted":
            handle_subscription_cancelled(event.data.object)

    return {"status": "processed"}
```

### Webhook 处理原则
1. **快速返回 200** — Stripe 期望 2 秒内响应，否则重发
2. **异步处理** — 重操作走队列，webhook 只入队
3. **幂等** — 用 event.id 去重
4. **验证签名** — 不验证 = 任何人能伪造支付成功

## 订阅生命周期

```python
# 订阅流程
customer = stripe.Customer.create(email=user.email)
# → 创建订阅
subscription = stripe.Subscription.create(
    customer=customer.id,
    items=[{"price": "price_pro_monthly"}],
    payment_behavior="default_incomplete",  # 等支付完成才激活
    expand=["latest_invoice.payment_intent"],
)
# 前端用 subscription.latest_invoice.payment_intent.client_secret 完成支付

# Webhook 处理续费
case "invoice.paid":
    # 续费成功 → 延长用户到期时间
    user.subscription_expires = invoice.period_end
    user.save()

case "invoice.payment_failed":
    # 续费失败 → 通知用户 + 宽限期
    send_email(user, "payment_failed")
    user.grace_period_until = now + timedelta(days=3)
```

## 生产检查清单
- [ ] 所有写操作用幂等键（Idempotency-Key header）
- [ ] Webhook 验证签名
- [ ] Webhook 幂等处理（event.id 去重）
- [ ] Webhook 快速返回 200 + 异步处理重操作
- [ ] 金额用整数分（不用浮点）
- [ ] 不存完整信用卡号（用 Stripe token，PCI 合规）
- [ ] 测试模式用 Stripe test keys + test cards
- [ ] Webhook endpoint 用 HTTPS
- [ ] 退款走 Stripe Dashboard 或 API（不手动改 DB）
- [ ] 监控支付失败率 + 对账
