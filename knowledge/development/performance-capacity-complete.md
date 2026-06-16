---
id: performance-capacity-complete
title: performance-capacity-complete
domain: development
category: performance-capacity-complete.md
difficulty: intermediate
tags: [capacity, complete, development, performance, 性能与容量完整知识库]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 性能与容量完整知识库

### 1. 性能预算
- 定义核心接口 P95、P99、吞吐量目标。
- 定义页面加载与交互时延预算。
- 超预算变更不得直接进入发布。

### 2. 压测方法
- 覆盖基线压测、峰值压测、持续压测。
- 场景必须包含正常流量与异常流量。
- 输出容量瓶颈与扩容建议。

### 3. 热点治理
- 定位热点接口、热点SQL、热点缓存键。
- 对热点路径实施缓存、批处理、异步化。
- 对突发流量实施限流与优先级保护。

### 4. 资源池调优
- 线程池参数按CPU核数与阻塞比例配置。
- 连接池参数按下游容量与峰值QPS配置。
- 防止线程池耗尽和连接池泄漏。

### 5. 运行时优化
- 定期分析GC与内存分配特征。
- 识别对象抖动与频繁序列化瓶颈。
- 关键服务建立性能回归基线。

### 6. 性能门禁
- 发布前必须通过性能回归检查。
- 回归幅度超阈值自动阻断发布。
- 性能问题必须有归因与修复计划。
