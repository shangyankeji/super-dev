---
id: database-engineering-complete
title: database-engineering-complete
domain: development
category: database-engineering-complete.md
difficulty: intermediate
tags: [complete, database, development, engineering, 数据库工程完整知识库]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 数据库工程完整知识库

### 1. 数据模型设计
- 核心表必须定义主键、唯一约束、审计字段。
- 状态字段需有明确状态机规则。
- 跨表关系需明确级联策略与删除策略。

### 2. 索引与查询治理
- 每个高频查询都应有索引策略。
- 慢SQL需定期分析并持续优化。
- 禁止无条件全表扫描进入生产核心链路。

### 3. 事务与一致性
- 明确读写一致性级别与事务边界。
- 关键写流程必须具备失败补偿机制。
- 分布式场景优先最终一致并设计对账策略。

### 4. 分库分表与扩展
- 按业务增长预估选择拆分时机。
- 路由规则必须稳定且可回溯。
- 拆分方案必须附带迁移与回滚路径。

### 5. 缓存一致性
- 采用写后删缓存或订阅失效策略。
- 热点数据必须防击穿、穿透、雪崩。
- 缓存命中率与失效率需持续监控。

### 6. 数据生命周期
- 定义保留、归档、清理规则。
- 敏感数据必须脱敏与访问审计。
- 备份与恢复演练必须周期化执行。
