---
id: concurrency-reliability-complete
title: concurrency-reliability-complete
domain: development
category: concurrency-reliability-complete.md
difficulty: intermediate
tags: [complete, concurrency, development, reliability, 并发与稳定性完整知识库]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 并发与稳定性完整知识库

### 1. 并发模型
- 共享资源必须定义并发访问策略。
- 读多写少场景优先无锁或读写锁优化。
- 避免粗粒度锁导致吞吐下降。

### 2. 分布式锁
- 锁必须有过期时间与续约机制。
- 锁粒度需最小化，避免全局串行化。
- 失败重试必须限制次数并具备退避策略。

### 3. 任务调度与消费
- 定时任务必须具备幂等保证。
- 消费者必须处理重复消息与乱序消息。
- 死信队列需有自动告警与处理流程。

### 4. 稳定性防护
- 限流保护核心资源。
- 熔断阻断异常依赖扩散。
- 降级保障核心链路可用。

### 5. 故障演练
- 周期执行依赖超时、网络抖动、节点宕机演练。
- 验证告警是否有效、恢复是否可执行。
- 演练结果必须回写到runbook与门禁策略。

### 6. 高可用架构
- 核心服务避免单点依赖。
- 明确同城容灾与异地容灾策略。
- 对关键链路定义可降级最小功能集。
