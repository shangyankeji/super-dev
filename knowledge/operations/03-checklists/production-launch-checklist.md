---
id: production-launch-checklist
title: 生产上线检查清单 (Production Launch Checklist)
domain: operations
category: 03-checklists
difficulty: intermediate
tags: [application, checklist, data, documentation, infrastructure, launch, monitoring, operations]
quality_score: 70
last_updated: 2026-06-15
---
# 生产上线检查清单 (Production Launch Checklist)

## 概述

本检查清单覆盖系统从预发布到生产上线的全部验证项，确保基础设施、应用、监控、安全、数据和文档六大维度全部就绪。每次上线前必须逐项确认，未通过的关键项（标记 [CRITICAL]）必须修复后才能继续。

## 使用说明

- **[CRITICAL]** 标记项为强制必过门禁，任一未通过则阻断上线
- **[HIGH]** 标记项为高度建议项，需要负责人签字确认风险后方可跳过
- **[MEDIUM]** 标记项为一般建议项，可带风险上线但需在 7 天内补齐
- 每项需由**执行人签字**和**审核人确认**，记录**确认时间**

---

## 一、基础设施 (Infrastructure)

### 1.1 DNS 与域名

- [ ] **[CRITICAL]** 生产域名 DNS 记录已配置并解析正确
- [ ] **[CRITICAL]** DNS TTL 已调低至 60s（上线当天，方便快速切换）
- [ ] **[HIGH]** 备用 DNS 提供商已配置（防止单点故障）
- [ ] **[MEDIUM]** DNS 记录已添加 CAA（Certificate Authority Authorization）
- [ ] **[HIGH]** 子域名规划已确认（api.example.com / cdn.example.com）
- [ ] **[MEDIUM]** DNS 监控已配置（解析延迟 + 可用性）

### 1.2 TLS/SSL 证书

- [ ] **[CRITICAL]** TLS 证书已申请并部署到所有入口节点
- [ ] **[CRITICAL]** 证书链完整（包含中间证书）
- [ ] **[CRITICAL]** 强制 HTTPS 重定向已启用（HTTP 301 -> HTTPS）
- [ ] **[HIGH]** TLS 最低版本为 1.2（禁用 TLS 1.0/1.1）
- [ ] **[HIGH]** 证书自动续期已配置（Let's Encrypt / ACM / cert-manager）
- [ ] **[MEDIUM]** HSTS（HTTP Strict Transport Security）已启用
- [ ] **[MEDIUM]** OCSP Stapling 已启用

### 1.3 CDN

- [ ] **[HIGH]** 静态资源已配置 CDN 加速（CSS/JS/图片/字体）
- [ ] **[HIGH]** CDN 缓存策略已配置（Cache-Control / ETag / 版本化文件名）
- [ ] **[MEDIUM]** CDN 回源失败的降级方案已准备
- [ ] **[MEDIUM]** CDN 边缘节点覆盖目标用户区域
- [ ] **[MEDIUM]** CDN 流量费用预估已确认

### 1.4 负载均衡

- [ ] **[CRITICAL]** 负载均衡器已配置并通过健康检查
- [ ] **[CRITICAL]** 后端服务实例 >= 2（保证高可用）
- [ ] **[HIGH]** 负载均衡算法已选择（Round Robin / Least Connections / IP Hash）
- [ ] **[HIGH]** 会话保持策略已确认（无状态优先，需要时配置 Sticky Session）
- [ ] **[HIGH]** 连接超时 / 读超时 / 写超时已合理配置
- [ ] **[MEDIUM]** 跨可用区负载均衡已启用
- [ ] **[MEDIUM]** WAF（Web Application Firewall）已启用

### 1.5 计算与网络

- [ ] **[CRITICAL]** 生产实例规格已确认（CPU/Memory/Disk）
- [ ] **[CRITICAL]** 安全组 / 防火墙规则仅开放必要端口
- [ ] **[HIGH]** 实例跨可用区部署（至少 2 个 AZ）
- [ ] **[HIGH]** VPC 网络规划已确认（子网划分、CIDR）
- [ ] **[MEDIUM]** 内网 DNS 解析已配置（服务间通信不走公网）
- [ ] **[MEDIUM]** 网络 ACL 已配置（数据库子网不可公网访问）

---

## 二、应用 (Application)

### 2.1 健康检查

- [ ] **[CRITICAL]** Liveness 探针已配置（检测进程存活）
- [ ] **[CRITICAL]** Readiness 探针已配置（检测服务就绪，含依赖检查）
- [ ] **[HIGH]** Startup 探针已配置（慢启动应用）
- [ ] **[HIGH]** 健康检查端点包含依赖状态（DB/Redis/MQ）
- [ ] **[MEDIUM]** 健康检查超时与间隔合理（interval: 10s, timeout: 5s）

```yaml
# Kubernetes 探针配置示例
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 15
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3
readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3
```

### 2.2 优雅停机

- [ ] **[CRITICAL]** 应用支持 SIGTERM 信号处理（优雅关闭连接）
- [ ] **[CRITICAL]** 停机前完成进行中请求的处理（drain）
- [ ] **[HIGH]** preStop hook 已配置（Kubernetes 场景）
- [ ] **[HIGH]** 优雅停机超时合理设置（terminationGracePeriodSeconds >= 30s）
- [ ] **[MEDIUM]** 停机期间从负载均衡器摘除流量

```yaml
# 优雅停机配置示例
lifecycle:
  preStop:
    exec:
      command: ["/bin/sh", "-c", "sleep 10"]  # 等待 LB 摘除
terminationGracePeriodSeconds: 60
```

### 2.3 配置管理

- [ ] **[CRITICAL]** 所有环境变量 / 配置项已通过 ConfigMap / Secret 管理
- [ ] **[CRITICAL]** 敏感信息不在代码仓库中（密码/Token/密钥）
- [ ] **[CRITICAL]** 生产配置已与开发/测试环境隔离
- [ ] **[HIGH]** 配置变更有审计记录
- [ ] **[HIGH]** 配置回滚方案已就绪
- [ ] **[MEDIUM]** 配置热加载已支持（无需重启生效）

### 2.4 资源限制

- [ ] **[CRITICAL]** 容器 CPU/Memory request 和 limit 已设置
- [ ] **[HIGH]** 资源 limit 基于压测数据设定（非拍脑袋）
- [ ] **[HIGH]** Pod Disruption Budget（PDB）已配置
- [ ] **[MEDIUM]** Resource Quota 已配置（防止单 namespace 过度使用）

### 2.5 依赖与启动顺序

- [ ] **[CRITICAL]** 所有外部依赖连通性已验证（DB/Redis/MQ/第三方API）
- [ ] **[HIGH]** 依赖不可用时有降级方案（熔断/缓存/默认值）
- [ ] **[HIGH]** 连接池大小已配置并验证
- [ ] **[MEDIUM]** 服务启动顺序依赖已处理（init container 或重试机制）

---

## 三、监控 (Monitoring)

### 3.1 指标采集

- [ ] **[CRITICAL]** 应用暴露 Prometheus 指标或等效 metrics 端点
- [ ] **[CRITICAL]** 核心业务指标已定义并采集（QPS/延迟/错误率/饱和度）
- [ ] **[HIGH]** RED 指标全覆盖（Rate/Error/Duration）
- [ ] **[HIGH]** USE 指标全覆盖（Utilization/Saturation/Errors）
- [ ] **[MEDIUM]** 自定义业务指标已采集（注册量/订单量/支付成功率）

### 3.2 日志

- [ ] **[CRITICAL]** 应用日志输出到 stdout/stderr（容器最佳实践）
- [ ] **[CRITICAL]** 日志格式为结构化 JSON（便于解析检索）
- [ ] **[HIGH]** 日志级别可动态调整（不重启切换 DEBUG）
- [ ] **[HIGH]** 日志采集管道已配置（Fluentd/Vector -> Elasticsearch/Loki）
- [ ] **[HIGH]** 请求链路追踪 ID 已注入日志（trace_id / request_id）
- [ ] **[MEDIUM]** 日志保留策略已配置（热数据 7 天，冷数据 90 天）
- [ ] **[MEDIUM]** 敏感信息已脱敏（密码/身份证/手机号不入日志）

### 3.3 告警

- [ ] **[CRITICAL]** P0 告警已配置（服务不可用/错误率突增/延迟飙升）
- [ ] **[CRITICAL]** 告警通知渠道已配置并验证（PagerDuty/Slack/钉钉/飞书）
- [ ] **[HIGH]** 告警分级已定义（P0-紧急/P1-高/P2-中/P3-低）
- [ ] **[HIGH]** 告警抑制与聚合规则已配置（避免告警风暴）
- [ ] **[HIGH]** On-call 轮值已安排（上线后至少 72 小时）
- [ ] **[MEDIUM]** 告警 Runbook 链接已附加到每条告警

### 3.4 大盘 (Dashboard)

- [ ] **[CRITICAL]** 系统概览大盘已创建（SLI/SLO 一目了然）
- [ ] **[HIGH]** 每个核心服务独立大盘已创建
- [ ] **[HIGH]** 基础设施大盘已创建（Node/Pod/DB/Redis）
- [ ] **[MEDIUM]** 业务大盘已创建（业务指标实时展示）
- [ ] **[MEDIUM]** 大盘已共享给所有相关团队

---

## 四、安全 (Security)

### 4.1 认证与授权

- [ ] **[CRITICAL]** 所有 API 端点已配置认证（JWT/OAuth2/API Key）
- [ ] **[CRITICAL]** 未授权访问返回 401，权限不足返回 403
- [ ] **[CRITICAL]** 管理后台有独立认证（MFA 必须启用）
- [ ] **[HIGH]** RBAC 权限模型已实现并验证
- [ ] **[HIGH]** API Rate Limiting 已配置
- [ ] **[MEDIUM]** Session/Token 过期策略已确认

### 4.2 数据加密

- [ ] **[CRITICAL]** 传输加密（TLS）已在所有通信链路启用
- [ ] **[CRITICAL]** 敏感数据存储加密（AES-256 或等效）
- [ ] **[HIGH]** 数据库连接使用 SSL
- [ ] **[HIGH]** 密钥管理使用 KMS/Vault（不在环境变量明文存储）
- [ ] **[MEDIUM]** 加密密钥轮换策略已定义

### 4.3 审计日志

- [ ] **[CRITICAL]** 用户登录/登出事件有审计记录
- [ ] **[CRITICAL]** 敏感操作（权限变更/数据删除/配置修改）有审计记录
- [ ] **[HIGH]** 审计日志不可篡改（独立存储，写入后不可修改）
- [ ] **[HIGH]** 审计日志保留期 >= 180 天
- [ ] **[MEDIUM]** 审计日志支持检索和导出

### 4.4 安全扫描

- [ ] **[HIGH]** 依赖漏洞扫描已通过（npm audit / pip-audit / trivy）
- [ ] **[HIGH]** 容器镜像安全扫描已通过
- [ ] **[HIGH]** OWASP Top 10 已验证覆盖
- [ ] **[MEDIUM]** SAST（静态代码分析）已通过
- [ ] **[MEDIUM]** DAST（动态安全测试）已通过

---

## 五、数据 (Data)

### 5.1 备份

- [ ] **[CRITICAL]** 数据库自动备份已配置（频率 >= 每日）
- [ ] **[CRITICAL]** 备份存储在异地（跨区域 / 跨可用区）
- [ ] **[HIGH]** 备份加密已启用
- [ ] **[HIGH]** 备份保留策略已定义（日备份 7 天 + 周备份 4 周 + 月备份 12 月）
- [ ] **[MEDIUM]** 关键配置文件已备份（Nginx/Kubernetes/中间件）

### 5.2 恢复测试

- [ ] **[CRITICAL]** 数据库恢复已实际执行验证（不是"应该能恢复"）
- [ ] **[CRITICAL]** 恢复时间 (RTO) 和恢复点 (RPO) 已确认并满足 SLA
- [ ] **[HIGH]** 恢复操作文档化（Runbook）
- [ ] **[HIGH]** 恢复演练计划已安排（至少每季度一次）
- [ ] **[MEDIUM]** 部分恢复能力已验证（单表/单库恢复）

### 5.3 数据迁移

- [ ] **[HIGH]** 数据库 Schema 迁移脚本已验证（正向 + 回滚）
- [ ] **[HIGH]** 数据迁移有幂等保证（可重复执行）
- [ ] **[HIGH]** 大表迁移有在线方案（不锁表：pt-online-schema-change / gh-ost）
- [ ] **[MEDIUM]** 数据一致性校验工具已准备

---

## 六、文档 (Documentation)

### 6.1 Runbook

- [ ] **[CRITICAL]** 常见故障处理 Runbook 已编写并审核
- [ ] **[CRITICAL]** Runbook 包含：故障现象 -> 诊断步骤 -> 修复操作 -> 验证方法
- [ ] **[HIGH]** Runbook 已分发给 On-call 团队并完成培训
- [ ] **[MEDIUM]** Runbook 与告警关联（告警消息包含 Runbook 链接）

### 6.2 架构文档

- [ ] **[HIGH]** 系统架构图已更新（包含所有组件和依赖关系）
- [ ] **[HIGH]** 网络拓扑图已更新
- [ ] **[MEDIUM]** API 文档已更新（OpenAPI/Swagger）
- [ ] **[MEDIUM]** 数据流图已更新

### 6.3 联系人与升级路径

- [ ] **[CRITICAL]** On-call 值班表已发布
- [ ] **[CRITICAL]** 升级路径已定义（P0: 5 分钟通知 -> 15 分钟组建 War Room）
- [ ] **[HIGH]** 关键供应商联系方式已记录（云厂商/CDN/DNS/第三方服务）
- [ ] **[HIGH]** 业务方联系人已确认（上线通知 / 异常沟通）
- [ ] **[MEDIUM]** 管理层升级通道已定义

---

## 七、回滚方案 (Rollback Plan)

### 7.1 回滚策略

- [ ] **[CRITICAL]** 回滚方案已文档化并经过审核
- [ ] **[CRITICAL]** 回滚操作 <= 5 分钟可完成
- [ ] **[CRITICAL]** 回滚触发条件已明确定义

```markdown
## 回滚触发条件
- 核心功能不可用（支付/登录/下单）
- 错误率 > 5%（持续 3 分钟）
- P95 延迟 > SLO 阈值的 2 倍（持续 5 分钟）
- 数据一致性异常
```

### 7.2 回滚执行

- [ ] **[CRITICAL]** 前一版本镜像/制品已保留并可直接部署
- [ ] **[HIGH]** 数据库回滚脚本已准备（如有 Schema 变更）
- [ ] **[HIGH]** 回滚后的验证清单已准备
- [ ] **[HIGH]** 回滚演练已在预发布环境执行

```bash
# Kubernetes 快速回滚
kubectl rollout undo deployment/api-server -n production
kubectl rollout status deployment/api-server -n production

# 验证回滚
kubectl get pods -n production -l app=api-server
curl -s https://api.example.com/healthz | jq .
```

---

## 八、上线执行 (Launch Execution)

### 8.1 上线前确认

- [ ] 所有 [CRITICAL] 项已通过（零例外）
- [ ] 所有 [HIGH] 项已通过或有签字确认的豁免
- [ ] 上线时间窗口已与各方确认（业务/运维/客服）
- [ ] 回滚负责人已指定
- [ ] On-call 人员已就位
- [ ] 客户沟通模板已准备（如需对外公告）

### 8.2 上线后验证

- [ ] 健康检查通过
- [ ] 核心业务流程冒烟测试通过
- [ ] 监控大盘无异常
- [ ] 错误率在基线范围内
- [ ] 延迟在 SLO 范围内
- [ ] 日志无异常错误

### 8.3 上线后观察

- [ ] 上线后 15 分钟：首次确认（健康检查 + 核心流程）
- [ ] 上线后 1 小时：二次确认（指标趋势 + 日志）
- [ ] 上线后 4 小时：三次确认（全量流量表现）
- [ ] 上线后 24 小时：最终确认（完整业务周期）

---

## 签字确认

| 角色 | 姓名 | 签字 | 日期 |
|------|------|------|------|
| 开发负责人 | ___ | ___ | ___ |
| 测试负责人 | ___ | ___ | ___ |
| SRE/运维负责人 | ___ | ___ | ___ |
| 安全负责人 | ___ | ___ | ___ |
| 产品负责人 | ___ | ___ | ___ |

---

## Agent Checklist

- [ ] 所有 [CRITICAL] 检查项已逐一确认并标记通过
- [ ] 所有 [HIGH] 检查项已逐一确认或有书面豁免
- [ ] 基础设施六项（DNS/TLS/CDN/LB/计算/网络）全部验证
- [ ] 应用五项（健康检查/优雅停机/配置管理/资源限制/依赖）全部验证
- [ ] 监控四项（指标/日志/告警/大盘）全部就绪
- [ ] 安全四项（认证/加密/审计日志/安全扫描）全部通过
- [ ] 数据三项（备份/恢复测试/迁移）全部验证
- [ ] 文档三项（Runbook/架构图/联系人）全部就绪
- [ ] 回滚方案已文档化、已演练、可在 5 分钟内执行
- [ ] 上线后观察计划已安排（15 分钟/1 小时/4 小时/24 小时）
- [ ] 签字确认表已收集完毕
