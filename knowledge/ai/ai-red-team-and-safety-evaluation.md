---
id: ai-red-team-and-safety-evaluation
title: ai-red-team-and-safety-evaluation
domain: ai
category: ai-red-team-and-safety-evaluation.md
difficulty: intermediate
tags: [ai, ai红队测试与安全评估, and, evaluation, red, safety, team]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## AI红队测试与安全评估

### 目标
- 在上线前识别提示注入、越权调用、敏感泄漏、内容安全等高风险问题。

### 适用范围
- 适用于新能力上线前评估、重大变更回归与周期性安全体检。

### 测试维度
- 提示注入与策略绕过。
- 工具越权与命令滥用。
- 数据泄漏与隐私暴露。
- 有害内容生成与安全边界突破。

### 执行清单
- 建立高危攻击语料库并持续更新。
- 对关键路径执行自动化红队回归。
- 每个高危用例必须定义阻断规则与处置动作。

### 验收标准
- 高危安全用例通过率达到阈值。
- 红队阻断策略覆盖关键攻击面。

### 常见失败模式
- 只做一次性安全测试，缺少版本回归。
- 对阻断误杀率无监控，导致业务不可用。

### 回滚策略
- 红队指标恶化时暂停发布并回滚到稳定策略版本。
- 临时收紧策略阈值并启用人工审核兜底。
