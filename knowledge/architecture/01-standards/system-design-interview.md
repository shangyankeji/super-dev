---
id: system-design-interview
title: 系统设计面试指南
domain: architecture
category: 01-standards
difficulty: intermediate
tags: [agent, architecture, checklist, design, interview, system, 实战系统设计, 常见陷阱]
quality_score: 70
last_updated: 2026-06-15
---
# 系统设计面试指南

## 概述
系统设计是评估高级工程师能力的核心考察方式。本指南覆盖URL短链、推送系统、Feed流、搜索引擎、支付系统、聊天系统、视频流媒体七大经典场景,提供结构化的分析方法和可落地的架构方案。

## 核心方法论

### 1. 系统设计四步法
1. **需求澄清(5分钟)**: 功能需求/非功能需求/规模估算
2. **高层设计(10分钟)**: 核心组件/数据流/API设计
3. **深入设计(20分钟)**: 关键组件详细设计/数据模型/算法
4. **总结与扩展(5分钟)**: 瓶颈分析/扩展方案/权衡讨论

### 2. 规模估算速查

| 指标 | 数量级 | 说明 |
|------|--------|------|
| 日活用户(DAU) | 问面试官 | 通常1亿=大规模 |
| QPS | DAU * 操作次数 / 86400 | 峰值约平均的2-3倍 |
| 存储 | 对象大小 * 数量 * 保留时间 | 1年=365天 |
| 带宽 | QPS * 响应大小 | 注意上下行差异 |

速算表:
- 1天 ≈ 10万秒(86400)
- 1年 ≈ 3000万秒
- 1KB * 1亿 ≈ 100GB
- 1MB * 1亿 ≈ 100TB

## 实战系统设计

### 设计1: URL短链系统

```
需求:
- 长URL→短URL(7位字符)
- 短URL→重定向到长URL
- 可选:点击统计、过期设置
- 规模: 1亿URL/天写入, 10:1读写比

规模估算:
- 写QPS: 1亿/86400 ≈ 1200/s, 峰值 3600/s
- 读QPS: 12000/s, 峰值 36000/s
- 存储: 1亿/天 * 100B * 365天 * 5年 ≈ 18TB
```

```
高层架构:

Client → Load Balancer → API Server → Cache(Redis)
                                    ↓
                               Database(MySQL/DynamoDB)
                                    ↓
                          Analytics Service(Kafka→ClickHouse)
```

```python
# 核心API设计
# POST /api/shorten {"url": "https://example.com/very/long/url"}
# → {"short_url": "https://s.co/Ab3xK7z"}
#
# GET /{short_code} → 301 Redirect

# 短码生成策略
import hashlib
import base64

class URLShortener:
    # 方案1: Base62编码自增ID
    BASE62 = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"

    @staticmethod
    def id_to_short_code(id: int) -> str:
        """自增ID转Base62短码"""
        if id == 0:
            return URLShortener.BASE62[0]
        code = []
        while id:
            code.append(URLShortener.BASE62[id % 62])
            id //= 62
        return ''.join(reversed(code))

    # 方案2: MD5哈希截取
    @staticmethod
    def hash_to_short_code(url: str) -> str:
        """MD5哈希取前7位Base62"""
        hash_bytes = hashlib.md5(url.encode()).digest()
        encoded = base64.b64encode(hash_bytes).decode()
        # 去掉特殊字符,取前7位
        code = encoded.replace('+', '').replace('/', '').replace('=', '')
        return code[:7]

# 数据模型
# urls表: id(bigint) | short_code(varchar7, unique index) | original_url(text) |
#          user_id | created_at | expires_at | click_count

# 缓存策略
# Redis: short_code → original_url, TTL=24h
# 布隆过滤器: 快速判断short_code是否存在(防止缓存穿透)
```

### 设计2: 消息推送系统

```
需求:
- 支持百万级设备推送(iOS/Android/Web)
- 延迟: P99 < 5秒
- 支持: 单播/组播/广播
- 可靠性: 消息不丢失,支持离线推送

高层架构:

App Server → Push API → Message Queue(Kafka)
                              ↓
                    Push Worker(消费者组)
                    ├── APNs Gateway(iOS)
                    ├── FCM Gateway(Android)
                    └── WebSocket Server(Web)
                              ↓
                    Device Token Store(Redis+DB)
                    Delivery Status(Kafka→Analytics)
```

```python
# 核心设计

class PushService:
    async def send_to_user(self, user_id: str, message: dict):
        """单播: 推送给用户的所有设备"""
        devices = await device_store.get_user_devices(user_id)
        for device in devices:
            await self.enqueue(device, message)

    async def send_to_segment(self, segment_id: str, message: dict):
        """组播: 推送给用户分群"""
        # 分批获取用户,避免内存溢出
        async for batch in user_segment_store.iter_users(segment_id, batch_size=1000):
            tasks = [self.send_to_user(uid, message) for uid in batch]
            await asyncio.gather(*tasks)

    async def send_broadcast(self, message: dict):
        """广播: 推送给所有用户"""
        # 使用topic广播,各worker分片处理
        await kafka.send("push.broadcast", message)

    async def enqueue(self, device: Device, message: dict):
        """入队: 按平台路由到不同topic"""
        topic = f"push.{device.platform}"  # push.ios / push.android / push.web
        await kafka.send(topic, {
            "device_token": device.token,
            "payload": self._build_payload(device.platform, message),
            "priority": message.get("priority", "normal"),
            "ttl": message.get("ttl", 86400),
        })

# 设备Token存储
# Redis: user:{user_id}:devices → Set of device_ids
# DB: devices表: device_id | user_id | platform | token | last_active | created_at

# 离线消息
# 设备离线时,消息存入pending queue
# 设备上线后,拉取pending消息
```

### 设计3: 新闻Feed流

```
需求:
- 用户发布内容,关注者的Feed中显示
- Feed按时间排序(可扩展为算法排序)
- 规模: 5亿用户, DAU 1亿, 每人关注平均200人

核心问题: 推(Fanout on Write) vs 拉(Fanout on Read)

推模式: 发布时写入所有关注者的Feed
- 优点: 读取快(O(1))
- 缺点: 大V发布慢(百万关注者), 存储大
- 适合: 关注者少的普通用户

拉模式: 读取时聚合关注者的内容
- 优点: 写入快, 存储小
- 缺点: 读取慢(聚合N个列表)
- 适合: 大V(百万关注者)

混合方案(推荐):
- 普通用户: 推模式
- 大V(关注者>10000): 拉模式
- 读取时: 合并推模式Feed + 拉取大V最新内容
```

```python
# Feed写入(发布内容)
class FeedService:
    async def publish(self, user_id: str, content: dict):
        """发布内容"""
        post = await post_store.create(user_id, content)

        followers = await social_graph.get_followers(user_id)

        if len(followers) > 10000:
            # 大V: 不fanout,读取时拉取
            return post

        # 普通用户: Fanout写入关注者Feed
        for batch in chunked(followers, 1000):
            await kafka.send("feed.fanout", {
                "post_id": post.id,
                "author_id": user_id,
                "follower_ids": batch,
                "timestamp": post.created_at,
            })

        return post

    async def get_feed(self, user_id: str, cursor: str = None, limit: int = 20):
        """获取Feed"""
        # 1. 从Redis获取推模式的Feed(已排序)
        pushed_posts = await redis.zrevrange(
            f"feed:{user_id}",
            start=0,
            end=limit * 2,  # 多取一些用于合并
        )

        # 2. 拉取关注的大V最新内容
        celebrities = await social_graph.get_followed_celebrities(user_id)
        celebrity_posts = []
        for celeb_id in celebrities:
            posts = await post_store.get_recent(celeb_id, limit=5)
            celebrity_posts.extend(posts)

        # 3. 合并排序
        all_posts = merge_sorted(pushed_posts, celebrity_posts, key="timestamp")

        # 4. 分页
        return paginate(all_posts, cursor=cursor, limit=limit)

# 数据存储
# Posts: post_id | user_id | content | media_urls | created_at (分片by user_id)
# Feed: Redis Sorted Set — feed:{user_id} → {post_id: timestamp}
# Social Graph: follower | following 双向表 (分片by user_id)
```

### 设计4: 搜索引擎

```
需求:
- 全文搜索(文档/商品/用户)
- 支持: 分词、相关性排序、过滤、高亮、自动补全
- 延迟: P99 < 200ms
- 规模: 10亿文档, 10万QPS

高层架构:

写入路径:
App → Write API → Kafka → Index Worker → Elasticsearch

查询路径:
Client → Search API → Elasticsearch Cluster
                   → Cache(Redis)

Elasticsearch Cluster:
├── Master Nodes (3个, 管理集群)
├── Data Nodes (N个, 存储和查询)
├── Coordinating Nodes (路由和聚合)
└── Ingest Nodes (数据预处理)
```

```python
# 搜索API设计
class SearchService:
    async def search(self, query: SearchQuery) -> SearchResult:
        # 缓存命中
        cache_key = f"search:{hash(query)}"
        cached = await redis.get(cache_key)
        if cached:
            return SearchResult.parse_raw(cached)

        # 构建ES查询
        es_query = {
            "bool": {
                "must": [
                    {"multi_match": {
                        "query": query.text,
                        "fields": ["title^3", "description^2", "content"],
                        "type": "best_fields",
                        "fuzziness": "AUTO",
                    }},
                ],
                "filter": [],
            }
        }

        # 过滤条件
        if query.category:
            es_query["bool"]["filter"].append(
                {"term": {"category": query.category}}
            )
        if query.price_range:
            es_query["bool"]["filter"].append(
                {"range": {"price": {
                    "gte": query.price_range[0],
                    "lte": query.price_range[1],
                }}}
            )

        result = await es.search(
            index="products",
            body={
                "query": es_query,
                "highlight": {"fields": {"title": {}, "description": {}}},
                "from": (query.page - 1) * query.size,
                "size": query.size,
                "sort": self._build_sort(query.sort_by),
            },
        )

        search_result = self._format_result(result)
        await redis.setex(cache_key, 300, search_result.json())
        return search_result

    async def autocomplete(self, prefix: str, limit: int = 10):
        """自动补全"""
        result = await es.search(
            index="products",
            body={
                "suggest": {
                    "title_suggest": {
                        "prefix": prefix,
                        "completion": {
                            "field": "title.suggest",
                            "size": limit,
                            "fuzzy": {"fuzziness": 1},
                        },
                    },
                },
            },
        )
        return [s["text"] for s in result["suggest"]["title_suggest"][0]["options"]]
```

### 设计5: 支付系统

```
需求:
- 支持多种支付方式(信用卡/支付宝/微信)
- 强一致性(钱不能多也不能少)
- 幂等性(重复请求不重复扣款)
- 对账(与第三方支付渠道对账)

高层架构:

Client → API Gateway → Payment Service → Payment Router
                                              ├── Stripe Gateway
                                              ├── Alipay Gateway
                                              └── WeChat Pay Gateway
                          ↕
                    Order Service ← → Ledger Service
                          ↕
                    Notification Service

关键组件:
- 支付服务: 统一支付接口,路由到不同渠道
- 账本服务: 复式记账,确保资金平衡
- 对账服务: 定时与渠道对账
```

```python
# 支付核心设计

class PaymentService:
    async def create_payment(self, request: PaymentRequest) -> Payment:
        """创建支付(幂等)"""
        # 1. 幂等检查
        existing = await payment_store.get_by_idempotency_key(request.idempotency_key)
        if existing:
            return existing

        # 2. 创建支付记录
        payment = Payment(
            id=generate_id(),
            order_id=request.order_id,
            amount=request.amount,
            currency=request.currency,
            method=request.method,
            status="pending",
            idempotency_key=request.idempotency_key,
        )
        await payment_store.save(payment)

        # 3. 路由到支付渠道
        gateway = self.get_gateway(request.method)
        try:
            result = await gateway.charge(
                amount=request.amount,
                currency=request.currency,
                token=request.payment_token,
                idempotency_key=request.idempotency_key,
            )

            # 4. 更新状态
            payment.status = "completed"
            payment.gateway_tx_id = result.transaction_id
            await payment_store.update(payment)

            # 5. 记账
            await ledger.record(
                debit_account=f"user:{request.user_id}",
                credit_account="revenue:sales",
                amount=request.amount,
                reference=payment.id,
            )

            # 6. 通知
            await event_bus.publish("payment.completed", payment.to_dict())

            return payment

        except PaymentDeclinedError as e:
            payment.status = "failed"
            payment.failure_reason = str(e)
            await payment_store.update(payment)
            raise

# 复式记账
class LedgerService:
    async def record(self, debit_account: str, credit_account: str,
                     amount: Decimal, reference: str):
        """复式记账(借贷必须平衡)"""
        async with db.begin() as tx:
            # 借方
            await tx.execute(text("""
                INSERT INTO ledger_entries (account, type, amount, reference, created_at)
                VALUES (:account, 'debit', :amount, :ref, now())
            """), {"account": debit_account, "amount": amount, "ref": reference})

            # 贷方
            await tx.execute(text("""
                INSERT INTO ledger_entries (account, type, amount, reference, created_at)
                VALUES (:account, 'credit', :amount, :ref, now())
            """), {"account": credit_account, "amount": amount, "ref": reference})

# 对账
class ReconciliationService:
    async def reconcile(self, date: str, gateway: str):
        """每日对账"""
        # 1. 获取本地交易记录
        local_txns = await payment_store.get_by_date_and_gateway(date, gateway)

        # 2. 获取渠道交易记录
        remote_txns = await gateway_client.get_settlement(date)

        # 3. 对比
        local_map = {tx.gateway_tx_id: tx for tx in local_txns}
        remote_map = {tx.id: tx for tx in remote_txns}

        discrepancies = []
        for tx_id, remote_tx in remote_map.items():
            local_tx = local_map.get(tx_id)
            if not local_tx:
                discrepancies.append(("missing_local", tx_id))
            elif local_tx.amount != remote_tx.amount:
                discrepancies.append(("amount_mismatch", tx_id))

        for tx_id in set(local_map.keys()) - set(remote_map.keys()):
            discrepancies.append(("missing_remote", tx_id))

        return ReconciliationReport(date=date, discrepancies=discrepancies)
```

### 设计6: 聊天系统

```
需求:
- 一对一和群聊
- 实时消息投递(WebSocket)
- 离线消息存储和推送
- 消息已读回执
- 规模: 1亿DAU, 平均每人发20条/天

高层架构:

Client ←WebSocket→ Gateway Server(有状态,维护连接)
                        ↕
                   Message Service(无状态)
                        ↕
               ┌────────┼────────┐
           Message DB    Redis    Push Service
          (Cassandra)  (在线状态)  (离线通知)
```

```python
# 消息投递核心

class ChatService:
    async def send_message(self, msg: ChatMessage):
        """发送消息"""
        # 1. 持久化
        await message_store.save(msg)

        # 2. 更新会话最新消息
        await conversation_store.update_last_message(msg.conversation_id, msg)

        # 3. 投递给接收方
        if msg.type == "direct":
            await self._deliver_to_user(msg.recipient_id, msg)
        elif msg.type == "group":
            members = await group_store.get_members(msg.group_id)
            for member_id in members:
                if member_id != msg.sender_id:
                    await self._deliver_to_user(member_id, msg)

    async def _deliver_to_user(self, user_id: str, msg: ChatMessage):
        """投递给用户"""
        # 查找用户连接的Gateway服务器
        gateway_server = await redis.get(f"online:{user_id}")

        if gateway_server:
            # 在线: 通过WebSocket实时推送
            await self._push_via_gateway(gateway_server, user_id, msg)
        else:
            # 离线: 存入未读队列 + 推送通知
            await redis.lpush(f"unread:{user_id}", msg.to_json())
            await push_service.send_notification(user_id, {
                "title": msg.sender_name,
                "body": msg.preview_text,
            })

# 消息存储(Cassandra)
# 按conversation_id分区,timestamp排序
# CREATE TABLE messages (
#   conversation_id text,
#   message_id timeuuid,
#   sender_id text,
#   content text,
#   type text,
#   created_at timestamp,
#   PRIMARY KEY (conversation_id, message_id)
# ) WITH CLUSTERING ORDER BY (message_id DESC);
```

### 设计7: 视频流媒体

```
需求:
- 视频上传、转码、存储
- 自适应码率流媒体播放(HLS/DASH)
- 规模: 每天100万视频上传, 1亿次播放

高层架构:

上传路径:
Client → Upload API → Object Storage(S3)
                          ↓
                    Transcode Queue(SQS/Kafka)
                          ↓
                    Transcode Workers(FFmpeg)
                    ├── 1080p/720p/480p/360p
                    ├── 生成HLS分片
                    └── 生成缩略图
                          ↓
                    CDN Origin(S3)

播放路径:
Client → CDN Edge → CDN Origin(S3)
         ↓
   自适应码率(根据带宽自动切换)

关键设计:
- CDN: 全球边缘节点,减少延迟
- 自适应码率: HLS/DASH协议,客户端根据带宽选择清晰度
- 转码: 异步处理,多种分辨率+编码(H.264/H.265/AV1)
- 分片: 视频切成2-10秒小段,支持seek和CDN缓存
```

## 最佳实践

### 1. 面试沟通
- 先问清楚需求(不要假设)
- 做规模估算(展示定量思维)
- 从高层设计开始,再逐步深入
- 讨论权衡(没有完美方案)
- 主动提出可改进的点

### 2. 通用设计原则
- 读写分离(CQRS/主从)
- 缓存分层(CDN/Redis/本地)
- 异步处理(消息队列解耦)
- 分片(按合理的键分布数据)
- 冗余(副本保高可用)

### 3. 数据库选择
- 关系型(MySQL/PG): 事务/复杂查询/一致性强
- 文档型(MongoDB): 灵活Schema/嵌套数据
- 列族(Cassandra): 大规模时序写入
- KV(Redis/DynamoDB): 高性能简单查询
- 搜索(ES): 全文搜索/聚合分析
- 图(Neo4j): 关系查询(社交/推荐)

### 4. 容量规划
- 存储: 数据大小 * 记录数 * 冗余因子
- 带宽: QPS * 平均响应大小
- 缓存: 热点数据 * 20%规则(20%数据覆盖80%请求)
- 节点数: QPS / 单节点能力

## 常见陷阱

### 陷阱1: 上来就画详细架构
```
# 错误: 没有问清需求就开始设计
# 正确: 先花5分钟确认功能需求/非功能需求/规模
```

### 陷阱2: 忽略非功能需求
```
# 错误: 只关注功能,不考虑性能/可用性/一致性
# 正确: 明确SLA(延迟P99/可用性99.9%/一致性模型)
```

### 陷阱3: 过度设计
```
# 错误: 小规模系统用Kafka+ES+Redis+Cassandra
# 正确: 从简单架构开始,说明在什么规模下需要什么组件
```

### 陷阱4: 不讨论权衡
```
# 错误: "这是最好的方案"
# 正确: "方案A的优点是X,缺点是Y;方案B的优点是..."
```

## Agent Checklist

### 设计流程
- [ ] 需求已澄清(功能/非功能/规模)
- [ ] 规模估算已完成(QPS/存储/带宽)
- [ ] 高层架构已画出
- [ ] 关键组件已详细设计

### 技术选型
- [ ] 数据库选择有理由
- [ ] 缓存策略已设计
- [ ] 消息队列需求已评估
- [ ] CDN/负载均衡已考虑

### 可靠性
- [ ] 单点故障已识别并解决
- [ ] 数据冗余/备份策略
- [ ] 降级方案已设计
- [ ] 限流/过载保护

### 可扩展性
- [ ] 水平扩展方案已设计
- [ ] 分片策略已确定
- [ ] 缓存分层已优化
- [ ] 瓶颈已识别并有扩展路径
