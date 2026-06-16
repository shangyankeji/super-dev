---
id: ecommerce-complete
title: 电商系统完整知识体系
domain: industries
category: ecommerce
difficulty: intermediate
tags: [complete, ecommerce, industries, 参考资料, 学习路径, 性能优化, 最佳实践, 核心模块]
quality_score: 70
last_updated: 2026-06-15
---
# 电商系统完整知识体系

## 概述
电商系统包括商品管理、购物车、订单、支付、物流、推荐等核心模块。本指南覆盖电商系统架构、性能优化、高并发处理。

## 核心模块

### 1. 商品管理

**商品信息**:
- SKU (库存单元)
- SPU (标准产品单元)
- 价格策略
- 库存管理
- 商品分类和属性

**实现**:
```python
from pydantic import BaseModel
from typing import Optional, List
from datetime import datetime

class Product(BaseModel):
    id: int
    sku: str
    name: str
    description: str
    price: float
    category_id: int
    attributes: dict
    images: List[str]
    inventory_count: int
    is_active: bool = True
    created_at: datetime
    updated_at: datetime

class ProductVariant(BaseModel):
    id: int
    product_id: int
    sku: str
    price: float
    attributes: dict  # 颜色、尺寸等
    inventory_count: int

# 商品服务
class ProductService:
    def __init__(self, db: AsyncSession):
        self.db = db
    
    async def get_product(self, product_id: int) -> Optional[Product]:
        query = select(Product).where(Product.id == product_id)
        result = await self.db.execute(query)
        return result.scalars().first()
    
    async def search_products(
        self, 
        query: str, 
        category: Optional[int] = None,
        min_price: Optional[float] = None,
        max_price: Optional[float] = None,
        page: int = 1,
        per_page: int = 20
    ) -> List[Product]:
        sql = select(Product).where(Product.is_active == True)
        
        if query:
            sql = sql.where(Product.name.ilike(f"%{query}%"))
        
        if category:
            sql = sql.where(Product.category_id == category)
        
        if min_price is not None:
            sql = sql.where(Product.price >= min_price)
        
        if max_price is not None:
            sql = sql.where(Product.price <= max_price)
        
        sql = sql.offset((page - 1) * per_page).limit(per_page)
        result = await self.db.execute(sql)
        return result.scalars().all()
```

### 2. 购物车系统

**功能**:
- 添加商品
- 修改数量
- 移除商品
- 清空购物车
- 计算总价

**实现**:
```python
from redis import Redis
import json

class CartService:
    def __init__(self, redis: Redis):
        self.redis = redis
        self.ttl = 86400  # 24小时
    
    def _get_cart_key(self, user_id: int) -> str:
        return f"cart:{user_id}"
    
    async def add_item(self, user_id: int, product_id: int, quantity: int = 1):
        key = self._get_cart_key(user_id)
        cart = await self.get_cart(user_id)
        
        if product_id in cart:
            cart[product_id] += quantity
        else:
            cart[product_id] = quantity
        
        await self.redis.setex(key, self.ttl, json.dumps(cart))
    
    async def update_quantity(self, user_id: int, product_id: int, quantity: int):
        if quantity <= 0:
            await self.remove_item(user_id, product_id)
            return
        
        key = self._get_cart_key(user_id)
        cart = await self.get_cart(user_id)
        cart[product_id] = quantity
        await self.redis.setex(key, self.ttl, json.dumps(cart))
    
    async def remove_item(self, user_id: int, product_id: int):
        key = self._get_cart_key(user_id)
        cart = await self.get_cart(user_id)
        cart.pop(product_id, None)
        await self.redis.setex(key, self.ttl, json.dumps(cart))
    
    async def get_cart(self, user_id: int) -> dict:
        key = self._get_cart_key(user_id)
        data = await self.redis.get(key)
        
        if data:
            return json.loads(data)
        return {}
    
    async def clear_cart(self, user_id: int):
        key = self._get_cart_key(user_id)
        await self.redis.delete(key)
```

### 3. 订单系统

**订单状态**:
- Pending: 待支付
- Paid: 已支付
- Shipped: 已发货
- Delivered: 已送达
- Cancelled: 已取消
- Refunded: 已退款

**实现**:
```python
from enum import Enum
from datetime import datetime

class OrderStatus(str, Enum):
    PENDING = "pending"
    PAID = "paid"
    SHIPPED = "shipped"
    DELIVERED = "delivered"
    CANCELLED = "cancelled"
    REFUNDED = "refunded"

class Order(BaseModel):
    id: int
    user_id: int
    status: OrderStatus = OrderStatus.PENDING
    total_amount: float
    discount_amount: float = 0.0
    final_amount: float
    shipping_address: dict
    billing_address: dict
    items: List[OrderItem]
    created_at: datetime
    updated_at: datetime
    paid_at: Optional[datetime]
    shipped_at: Optional[datetime]
    delivered_at: Optional[datetime]

class OrderItem(BaseModel):
    product_id: int
    product_name: str
    quantity: int
    unit_price: float
    total_price: float

class OrderService:
    def __init__(self, db: AsyncSession, cart_service: CartService):
        self.db = db
        self.cart_service = cart_service
    
    async def create_order(self, user_id: int, order_data: dict) -> Order:
        # 1. 获取购物车
        cart = await self.cart_service.get_cart(user_id)
        
        if not cart:
            raise ValueError("Cart is empty")
        
        # 2. 验证库存
        for product_id, quantity in cart.items():
            await self._validate_inventory(product_id, quantity)
        
        # 3. 计算价格
        items = []
        total_amount = 0.0
        
        for product_id, quantity in cart.items():
            product = await self.product_service.get_product(product_id)
            item_total = product.price * quantity
            items.append(OrderItem(
                product_id=product_id,
                product_name=product.name,
                quantity=quantity,
                unit_price=product.price,
                total_price=item_total
            ))
            total_amount += item_total
        
        # 4. 创建订单
        order = Order(
            user_id=user_id,
            status=OrderStatus.PENDING,
            total_amount=total_amount,
            final_amount=total_amount,
            items=items,
            **order_data
        )
        
        self.db.add(order)
        await self.db.commit()
        await self.db.refresh(order)
        
        # 5. 清空购物车
        await self.cart_service.clear_cart(user_id)
        
        return order
    
    async def cancel_order(self, order_id: int, reason: str):
        order = await self.get_order(order_id)
        
        if order.status not in [OrderStatus.PENDING, OrderStatus.PAID]:
            raise ValueError("Cannot cancel order in current status")
        
        # 退款
        if order.status == OrderStatus.PAID:
            await self._refund_payment(order)
        
        # 恢复库存
        for item in order.items:
            await self._restore_inventory(item.product_id, item.quantity)
        
        # 更新状态
        order.status = OrderStatus.CANCELLED
        order.cancelled_at = datetime.utcnow()
        order.cancel_reason = reason
        
        await self.db.commit()
```

### 4. 推荐系统

**推荐策略**:
- 协同过滤
- 内容推荐
- 热门商品
- 个性化推荐

**实现**:
```python
from typing import List, Dict
import pandas as pd
from sklearn.metrics.pairwise import cosine_similarity

class RecommendationEngine:
    def __init__(self, db: AsyncSession):
        self.db = db
    
    async def get_personalized_recommendations(
        self, 
        user_id: int, 
        limit: int = 10
    ) -> List[int]:
        """个性化推荐"""
        # 1. 获取用户历史订单
        user_orders = await self._get_user_orders(user_id)
        
        if not user_orders:
            # 新用户: 返回热门商品
            return await self._get_popular_products(limit)
        
        # 2. 协同过滤
        similar_users = await self._find_similar_users(user_id)
        
        # 3. 获取相似用户购买的商品
        recommendations = []
        for similar_user in similar_users:
            products = await self._get_user_products(similar_user)
            recommendations.extend(products)
        
        # 4. 去重和排序
        purchased_products = set(item.product_id for order in user_orders for item in order.items)
        recommendations = [
            p for p in recommendations 
            if p not in purchased_products
        ]
        
        return recommendations[:limit]
    
    async def get_similar_products(self, product_id: int, limit: int = 10) -> List[int]:
        """相似商品推荐"""
        # 基于商品属性和分类
        product = await self.product_service.get_product(product_id)
        
        query = """
        SELECT p.id, 
               ABS(p.category_id - :category_id) AS category_diff,
               ABS(p.price - :price) AS price_diff
        FROM products p
        WHERE p.id != :product_id
          AND p.is_active = true
        ORDER BY category_diff, price_diff
        LIMIT :limit
        """
        
        result = await self.db.execute(
            text(query),
            {"category_id": product.category_id, "price": product.price, "product_id": product_id, "limit": limit}
        )
        
        return [row.id for row in result.scalars().all()]
    
    async def get_frequently_bought_together(self, product_id: int, limit: int = 10) -> List[int]:
        """经常一起购买的商品"""
        query = """
        SELECT oi2.product_id, COUNT(*) as frequency
        FROM order_items oi1
        JOIN order_items oi2 ON oi1.order_id = oi2.order_id
        WHERE oi1.product_id = :product_id
          AND oi2.product_id != :product_id
        GROUP BY oi2.product_id
        ORDER BY frequency DESC
        LIMIT :limit
        """
        
        result = await self.db.execute(
            text(query),
            {"product_id": product_id, "limit": limit}
        )
        
        return [row.product_id for row in result.scalars().all()]
```

### 5. 库存管理

**库存策略**:
- 安库存预警
- 自动补货
- 库存盘点

**实现**:
```python
class InventoryService:
    def __init__(self, db: AsyncSession):
        self.db = db
        self.low_stock_threshold = 10
    
    async def update_inventory(self, product_id: int, quantity_change: int):
        """更新库存"""
        query = """
        UPDATE products 
        SET inventory_count = inventory_count + :quantity_change,
            updated_at = NOW()
        WHERE id = :product_id
        RETURNING inventory_count
        """
        
        result = await self.db.execute(
            text(query),
            {"product_id": product_id, "quantity_change": quantity_change}
        )
        
        new_count = result.scalar()
        
        # 检查低库存
        if new_count <= self.low_stock_threshold:
            await self._send_low_stock_alert(product_id, new_count)
        
        return new_count
    
    async def check_inventory(self, product_id: int, quantity: int) -> bool:
        """检查库存是否充足"""
        product = await self.product_service.get_product(product_id)
        return product.inventory_count >= quantity
    
    async def get_low_stock_products(self) -> List[Product]:
        """获取低库存商品"""
        query = select(Product).where(
            Product.inventory_count <= self.low_stock_threshold,
            Product.is_active == True
        )
        result = await self.db.execute(query)
        return result.scalars().all()
    
    async def _send_low_stock_alert(self, product_id: int, current_count: int):
        """发送低库存警报"""
        product = await self.product_service.get_product(product_id)
        
        # 发送邮件/短信通知
        await send_alert_email(
            subject=f"Low Stock Alert: {product.name}",
            body=f"Product {product.name} (SKU: {product.sku}) has only {current_count} units left"
        )
```

## 性能优化

### 1. 缓存策略

```python
from fastapi_cache import FastAPICache
from fastapi_cache.backends.redis import RedisBackend
from fastapi_cache.decorator import cache

@app.get("/products/{product_id}")
@cache(expire=300)  # 5分钟缓存
async def get_product(product_id: int):
    # 数据库查询
    return await product_service.get_product(product_id)

@app.get("/products/search")
@cache(expire=60)  # 1分钟缓存
async def search_products(query: str):
    # 搜索逻辑
    return await product_service.search_products(query)
```

### 2. 数据库优化

```sql
-- 索引
CREATE INDEX idx_products_category ON products(category_id);
CREATE INDEX idx_products_name ON products USING gin(name);
CREATE INDEX idx_orders_user_status ON orders(user_id, status);
CREATE INDEX idx_orders_created ON orders(created_at DESC);

-- 分区 (PostgreSQL)
CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    ...
) PARTITION BY RANGE (created_at);

CREATE TABLE orders_2024_q1 PARTITION OF orders
    FOR VALUES FROM ('2024-01-01') TO ('2024-04-01');

-- 物化视图 (热门商品)
CREATE MATERIALIZED VIEW popular_products AS
SELECT 
    p.id,
    p.name,
    COUNT(o.id) as order_count,
    SUM(oi.quantity) as total_quantity
FROM products p
JOIN order_items oi ON p.id = oi.product_id
JOIN orders o ON oi.order_id = o.id
WHERE o.created_at > NOW() - INTERVAL '7 days'
GROUP BY p.id
ORDER BY order_count DESC
LIMIT 100;

REFRESH MATERIALIZED VIEW popular_products;
```

### 3. 异步处理

```python
from celery import Celery

celery_app = Celery('ecommerce', broker='redis://localhost:6379/0')

@celery_app.task
def process_order_async(order_id: int):
    """异步处理订单"""
    # 1. 发送确认邮件
    send_order_confirmation_email(order_id)
    
    # 2. 更新库存
    update_inventory_for_order(order_id)
    
    # 3. 通知仓库
    notify_warehouse_for_shipping(order_id)
    
    # 4. 更新推荐系统
    update_recommendation_data(order_id)

@celery_app.task
def generate_daily_sales_report():
    """每日销售报告"""
    # 生成报告逻辑
    report = generate_sales_report()
    send_report_email(report)
```

## 最佳实践

### ✅ DO

1. **使用幂等性键**
```python
@app.post("/orders")
async def create_order(order: OrderCreate):
    # ✅ 幂等性检查
    if await redis.exists(f"order:idempotency:{order.idempotency_key}"):
        return await get_existing_order(order.idempotency_key)
    
    # 创建订单
    ...
    await redis.setex(f"order:idempotency:{order.idempotency_key}", 3600, order_id)
```

2. **库存预占**
```python
async def create_order(items: List[OrderItem]):
    # ✅ 预占库存
    for item in items:
        await redis.setnx(f"inventory:lock:{item.product_id}", 1)
        await redis.expire(f"inventory:lock:{item.product_id}", 300)
    
    try:
        # 创建订单
        ...
    finally:
        # 释放锁
        for item in items:
            await redis.delete(f"inventory:lock:{item.product_id}")
```

### ❌ DON'T

1. **不要N+1查询**
```python
# ❌ 错误
async def get_orders(user_id: int):
    orders = await db.execute(select(Order).where(Order.user_id == user_id))
    for order in orders:
        order.items = await db.execute(
            select(OrderItem).where(OrderItem.order_id == order.id)
        )
    return orders

# ✅ 正确
async def get_orders(user_id: int):
    query = select(Order).options_from(Order.items).where(Order.user_id == user_id)
    return await db.execute(query)
```

2. **不要在事务中调用外部API**
```python
# ❌ 错误
async def create_order():
    async with db.begin():
        ...
        await stripe.Charge.create(...)  # 外部API调用
        ...

# ✅ 正确
async def create_order():
    async with db.begin():
        # 先创建订单
        ...
    
    # 事务外调用API
    await stripe.Charge.create(...)
```

## 学习路径

### 初级 (1-2周)
1. 电商系统概述
2. 商品和购物车系统
3. 计算和价格

### 中级 (2-3周)
1. 计单和支付集成
2. 库存管理
3. 推荐系统

### 高级 (2-4周)
1. 高并发处理
2. 分布式事务
3. 数据分析

### 专家级 (持续)
1. 智能定价
2. 供应链优化
3. 全渠道零售

## 参考资料

### 抡术文档
- [电商架构模式](https://microservices.io/patterns/ecommerce.html)
- [电商性能优化](https://www.nginx.com/blog/ecommerce-performance/)

### 案例学习
- [Shopify架构](https://www.shopify.com/blog/engineering/)
- [Amazon电商技术](https://aws.amazon.com/ecommerce/)

---

**知识ID**: `ecommerce-complete`  
**领域**: industries/ecommerce  
**类型**: standards  
**难度**: intermediate  
**质量分**: 92  
**维护者**: ecommerce-team@superdev.com  
**最后更新**: 2026-03-28
