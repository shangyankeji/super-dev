---
id: threat-modeling-stride-playbook
title: threat-modeling-stride-playbook
domain: security
category: threat-modeling-stride-playbook.md
difficulty: intermediate
tags: [modeling, playbook, security, stride, threat, 威胁建模手册]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## STRIDE 威胁建模手册

### 目标
- 在需求与设计阶段提前识别安全风险，降低上线后高危漏洞暴露概率。

### STRIDE 维度
- S：身份伪造（Spoofing）
- T：篡改（Tampering）
- R：抵赖（Repudiation）
- I：信息泄露（Information Disclosure）
- D：拒绝服务（Denial of Service）
- E：权限提升（Elevation of Privilege）

### 建模步骤
- 列出核心资产与数据流。
- 为每条数据流逐项评估 STRIDE 风险。
- 为每个高风险点定义检测与防护措施。
- 确定验证方式并纳入测试计划。

### 必做检查项
- 鉴权接口必须验证主体、租户、资源归属。
- 敏感数据接口必须有审计日志与脱敏策略。
- 外部输入点必须有参数白名单与长度限制。
- 高风险操作必须定义速率限制与告警规则。

### 常见失败模式
- 只做一次建模，不随架构演进更新。
- 识别风险后没有落地到门禁和测试。
