---
id: ai-domain-index-and-checklist
title: ai-domain-index-and-checklist
domain: ai
category: ai-domain-index-and-checklist.md
difficulty: intermediate
tags: [ai, ai领域索引与执行清单, and, checklist, domain, index]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## AI领域索引与执行清单

### 目标
- 为AI需求、方案、上线、运行提供统一入口和核查清单。

### 适用范围
- LLM应用、Agent系统、RAG应用、智能工作流与自动化工具链。

### 执行清单
- 需求阶段：目标指标、风险边界、数据来源合规性已确认。
- 设计阶段：提示词策略、工具权限模型、失败降级路径已定义。
- 开发阶段：结构化输出、重试策略、幂等策略已实现。
- 测试阶段：准确性、安全性、延迟、成本评测已通过。
- 发布阶段：灰度方案、回滚触发条件、发布验收标准已固化。
- 运行阶段：告警分级、值班流程、复盘闭环机制已建立。

### 验收标准
- 关键链路任务成功率达到目标阈值。
- 安全与合规阻断项为0。
- 生产回滚演练在目标时间内完成。

### 常见失败模式
- 只评估模型能力，不评估工具链与系统级风险。
- 缺少在线监控与漂移检测，质量退化发现滞后。

### 回滚策略
- 发现P0错误或越权风险时，立即降级到安全兜底路径。
- 在30分钟内切回上一个稳定模型与提示词版本。
