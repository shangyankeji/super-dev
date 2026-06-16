---
id: incident-response-complete
title: 事故响应与复盘完整指南
domain: incident
category: 01-standards
difficulty: intermediate
tags: [complete, incident, response, 事故分级, 事故响应流程, 初始通知, 发现后5分钟内, 恢复通知]
quality_score: 70
last_updated: 2026-06-15
---
# 事故响应与复盘完整指南

## 概述

事故响应是运维工程的核心能力。一个成熟的事故响应流程能将 MTTR（平均恢复时间）从小时级降低到分钟级。本指南覆盖事故分级、响应流程、通信模板、复盘方法和预防措施。

---

## 事故分级

| 级别 | 影响 | 响应时间 | 通知范围 | 示例 |
|------|------|----------|----------|------|
| **P0 (Critical)** | 全站不可用/数据丢失 | < 5分钟 | CEO + 全体工程 | 数据库崩溃、安全漏洞被利用 |
| **P1 (Major)** | 核心功能不可用 | < 15分钟 | VP Eng + On-call | 支付系统故障、API 大面积超时 |
| **P2 (Minor)** | 部分功能降级 | < 30分钟 | Team Lead + On-call | 搜索变慢、部分用户无法登录 |
| **P3 (Low)** | 非关键功能异常 | < 2小时 | On-call | 管理后台报错、日志丢失 |

---

## 事故响应流程

### Phase 1: 检测与告警 (0-5分钟)

```yaml
# Prometheus 告警规则示例
groups:
  - name: critical-alerts
    rules:
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m]))
          / sum(rate(http_requests_total[5m])) > 0.05
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "错误率超过 5%"
          runbook: "https://runbook.internal/high-error-rate"

      - alert: HighLatency
        expr: |
          histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) > 2
        for: 3m
        labels:
          severity: warning
        annotations:
          summary: "P99 延迟超过 2s"
```

### Phase 2: 分类与升级 (5-15分钟)

```
告警触发
  │
  ├── On-call 工程师确认
  │     │
  │     ├── 是否影响用户? ──── 否 → P3 (记录并排期修复)
  │     │
  │     └── 是 → 影响范围?
  │           │
  │           ├── 部分用户 → P2
  │           ├── 核心功能 → P1
  │           └── 全站 → P0 (立即升级)
  │
  └── 指派事故指挥官 (Incident Commander)
```

### Phase 3: 诊断与修复 (15分钟-2小时)

```bash
# 快速诊断脚本
#!/bin/bash
echo "=== 系统状态 ==="
kubectl get pods --all-namespaces | grep -v Running
kubectl top nodes
kubectl top pods --sort-by=memory

echo "=== 最近事件 ==="
kubectl get events --sort-by='.lastTimestamp' | tail -20

echo "=== 错误日志 ==="
kubectl logs -l app=api-server --tail=50 | grep -i error

echo "=== 数据库连接 ==="
psql -c "SELECT count(*) FROM pg_stat_activity WHERE state = 'active';"
psql -c "SELECT query, calls, mean_exec_time FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 5;"

echo "=== Redis 状态 ==="
redis-cli info memory | grep used_memory_human
redis-cli info clients | grep connected_clients
```

### Phase 4: 缓解措施

```python
# 常见缓解策略
MITIGATION_PLAYBOOK = {
    "高错误率": [
        "1. 检查最近部署 → 回滚 (kubectl rollout undo)",
        "2. 检查外部依赖 → 开启降级模式",
        "3. 检查数据库 → 杀慢查询, 扩连接池",
        "4. 检查内存/CPU → 水平扩容",
    ],
    "高延迟": [
        "1. 检查数据库慢查询 → 优化或杀掉",
        "2. 检查缓存命中率 → 预热缓存",
        "3. 检查连接池耗尽 → 扩大池大小",
        "4. 检查 GC 暂停 → 调整 JVM/Python GC",
    ],
    "服务不可用": [
        "1. 检查 Pod 状态 → 重启或扩容",
        "2. 检查证书过期 → 更新证书",
        "3. 检查 DNS → 验证解析",
        "4. 检查磁盘 → 清理或扩容",
    ],
    "数据异常": [
        "1. 停止写入 → 防止进一步损坏",
        "2. 评估影响范围 → 确定受影响的数据",
        "3. 从备份恢复 → PITR 到故障前时间点",
        "4. 验证数据完整性 → 校验和对账",
    ],
}
```

### Phase 5: 通信

```markdown
# 事故通信模板

## 初始通知 (发现后5分钟内)
**[P1 事故] API 响应超时**
- 影响: 部分用户无法完成支付
- 发现时间: 2026-03-28 14:30 UTC
- 当前状态: 调查中
- 事故指挥官: @alice
- 下次更新: 15分钟后

## 进展更新 (每15-30分钟)
**[P1 更新] API 响应超时 - 根因已定位**
- 根因: 数据库连接池耗尽 (活跃连接 500/500)
- 缓解措施: 已扩大连接池至 1000，正在重启服务
- 预计恢复: 15分钟内
- 下次更新: 15分钟后

## 恢复通知
**[P1 已恢复] API 响应超时**
- 恢复时间: 2026-03-28 15:15 UTC
- 影响时长: 45分钟
- 根因: 新功能引入了 N+1 查询导致连接池耗尽
- 永久修复: 已合并 PR #1234 (查询优化)
- 复盘时间: 2026-03-29 10:00 UTC
```

---

## 复盘 (Postmortem)

### 复盘模板

```markdown
# 事故复盘: [事故标题]

## 概要
- **日期**: 2026-03-28
- **时长**: 45分钟 (14:30 - 15:15 UTC)
- **影响**: 约 5000 用户无法完成支付
- **级别**: P1
- **根因**: N+1 查询导致数据库连接池耗尽

## 时间线
| 时间 (UTC) | 事件 |
|------------|------|
| 14:25 | 部署 v2.3.1 (包含订单列表新功能) |
| 14:30 | 错误率告警触发 (>5%) |
| 14:32 | On-call @bob 确认事故，升级为 P1 |
| 14:35 | 事故指挥官 @alice 就位，开始诊断 |
| 14:45 | 定位根因: pg_stat_activity 显示 500 活跃连接 |
| 14:50 | 尝试回滚 v2.3.1 → 失败 (数据库迁移不可逆) |
| 14:55 | 扩大连接池至 1000，重启 API 服务 |
| 15:05 | 错误率降至 1%，支付功能恢复 |
| 15:15 | 错误率降至 0.1%，确认完全恢复 |

## 根因分析
订单列表 API 在新功能中引入了 N+1 查询问题:
- 每个订单额外查询 3 次数据库 (用户信息、商品详情、物流状态)
- 一个列表请求触发 30+ 数据库查询
- 高峰期请求并发导致连接池快速耗尽

## 5 Whys 分析
1. **为什么支付系统不可用?** → 数据库连接池耗尽
2. **为什么连接池耗尽?** → 新功能引入了 N+1 查询
3. **为什么 N+1 查询没被发现?** → 代码审查未检查数据库查询数量
4. **为什么没有查询数量的监控?** → 监控只覆盖了响应时间，未覆盖查询数量
5. **为什么测试没有发现?** → 测试数据量太小 (10条 vs 生产 10000条)

## 行动项
| 优先级 | 行动 | 负责人 | 截止日期 |
|--------|------|--------|----------|
| P0 | 修复 N+1 查询 (select_related) | @charlie | 2026-03-29 |
| P1 | 添加数据库查询数量监控 | @dave | 2026-04-01 |
| P1 | CI 中添加查询数量断言 | @eve | 2026-04-05 |
| P2 | 增加测试数据量 (>1000条) | @charlie | 2026-04-10 |
| P2 | 连接池告警 (>80%使用率) | @dave | 2026-04-05 |

## 经验教训
### 做得好的
- 告警触发迅速 (部署后5分钟)
- 事故响应流程清晰，通信及时
- 团队协作高效

### 需要改进的
- 代码审查需要关注数据库查询模式
- 需要更真实的测试数据
- 回滚失败时需要备选方案
```

### 无责文化 (Blameless Culture)

```
✅ 正确的复盘态度:
- "系统允许了这个错误发生" (而非 "某人犯了错")
- "我们的流程缺失了什么?" (而非 "谁没做好?")
- "如何让这个错误不可能再次发生?" (而非 "下次小心点")

❌ 错误的复盘态度:
- "这是 @bob 的错"
- "应该更小心"
- "下次注意"
```

---

## On-Call 最佳实践

### 轮值制度

```
每周轮值:
├── 主 On-Call (Primary): 接收所有告警，第一响应
├── 副 On-Call (Secondary): 主 On-Call 15分钟无响应时升级
└── 经理 On-Call: P0/P1 事故自动通知

轮值规则:
- 每次轮值 1 周 (周一 10:00 → 下周一 10:00)
- 轮值前一天交接会 (15分钟)
- 夜间告警 (22:00-08:00) 必须可在 5 分钟内响应
- 连续超过 3 次夜间告警 → 次日可延迟上班
```

### 告警疲劳防治

```
✅ 健康的告警:
- 每周夜间告警 < 5 次
- 每条告警都需要人为干预
- 告警有明确的 Runbook 链接

❌ 告警疲劳信号:
- 超过 50% 的告警被忽略
- On-Call 开始"批量确认"告警
- 关键告警淹没在噪音中
```

---

## Agent Checklist

Agent 在设计系统时必须检查的事故预防要点:

- [ ] 是否有完善的监控和告警 (错误率/延迟/资源使用)?
- [ ] 告警是否有 Runbook 链接?
- [ ] 是否有事故分级标准和升级流程?
- [ ] 部署是否支持快速回滚?
- [ ] 是否有数据库慢查询监控?
- [ ] 是否有连接池/线程池使用率告警?
- [ ] 是否有事故通信模板?
- [ ] 是否定期进行复盘 (所有 P0/P1)?
- [ ] 是否有混沌工程实践 (故障注入测试)?
- [ ] On-Call 轮值是否合理 (不过度疲劳)?

---

## 参考资料

- [Google SRE Book - Chapter 14: Managing Incidents](https://sre.google/sre-book/managing-incidents/)
- [PagerDuty Incident Response Guide](https://response.pagerduty.com/)
- [Atlassian Incident Management Handbook](https://www.atlassian.com/incident-management)

---

**文档版本**: v1.0
**最后更新**: 2026-03-28
**质量评分**: 93/100
