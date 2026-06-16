---
id: security-in-development-complete
title: security-in-development-complete
domain: development
category: security-in-development-complete.md
difficulty: intermediate
tags: [complete, development, security, 开发安全完整知识库]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## 开发安全完整知识库

### 1. 安全左移
- 在需求与设计阶段完成威胁建模。
- 在编码阶段执行安全基线检查。
- 在发布阶段执行漏洞与配置扫描。

### 2. 输入与输出安全
- 输入参数白名单校验与长度限制。
- 输出前进行敏感数据脱敏。
- 文件上传必须限制类型、大小、解析策略。

### 3. 身份与权限
- 最小权限原则覆盖接口、数据、操作。
- 高风险操作必须二次确认或审批。
- 权限变更必须可审计可追溯。

### 4. 依赖与供应链
- 依赖版本锁定与漏洞持续扫描。
- 高危漏洞有时限修复策略。
- 构建制品需签名并可验签。

### 5. 密钥与机密管理
- 禁止把密钥写入代码库。
- 密钥轮换周期与访问审计必须制度化。
- 非必要场景不得下发高权限凭证。

### 6. 事件响应
- 发现漏洞后先止损再修复。
- 关键安全事件必须形成复盘与防复发措施。
- 安全改进项进入发布门禁持续执行。
