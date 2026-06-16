---
id: governance-capabilities
title: Super Dev 治理能力全景图
domain: 00-governance
category: governance-capabilities.md
difficulty: intermediate
tags: [00-governance, capabilities, engine, governance, knowledge, rule, tracker, validation]
quality_score: 70
last_updated: 2026-06-15
---
# Super Dev 治理能力全景图

> 版本: 2.1.6+ | 最后更新: 2026-03-28

## 1. 概述

Super Dev 的核心定位是 **AI Coding 治理层** —— 它不拥有模型端点，而是在 AI 编码宿主（Claude Code、Cursor、Windsurf 等）之上提供标准化的工程流水线协议、质量门禁和交付审计。

2.1.6 版本引入了 5 项治理能力增强，将治理从"阶段性检查"升级为"全链路可编程治理"：

| 能力 | 模块 | 核心价值 |
|------|------|----------|
| Validation Rule Engine | `super_dev/reviewers/validation_rules.py` | 可编程质量规则，YAML 定义，pipeline 任意阶段插拔 |
| Knowledge Tracker | `super_dev/knowledge_tracker.py` | 知识引用透明化，覆盖率可量化 |
| Prompt Templates | `super_dev/creators/prompt_templates.py` | Prompt 版本化管理，可追溯可回滚 |
| ADR Generator | `super_dev/creators/adr_generator.py` | 架构决策自动记录，技术选型有据可查 |
| Pipeline Metrics | `super_dev/metrics/pipeline_metrics.py` | DORA + Rework Rate，效能可度量 |

统一由 **Pipeline Governance** (`super_dev/orchestrator/governance.py`) 集成管理。

---

## 2. 设计理念

### 2.1 可编程优于硬编码

所有治理规则通过 YAML 或 Markdown 文件定义，而非代码内嵌。这意味着：
- 项目团队可以在不修改 Super Dev 源码的情况下调整治理策略
- 规则变更可以通过 Git 追踪和 Code Review
- 不同项目可以有不同的治理配置

### 2.2 透明优于黑盒

每项治理能力都会生成可读的报告和审计记录：
- 验证规则执行结果有详细的通过/失败说明
- 知识引用有完整的引用链路
- 度量数据可导出为 JSON 供外部系统消费

### 2.3 渐进式采纳

所有治理能力默认启用但不阻断，团队可以逐步提高要求：
- 先观察（报告模式）
- 再告警（warning 模式）
- 最后强制（fail 模式）

### 2.4 与 Pipeline 深度集成

治理能力不是独立的工具集，而是嵌入 pipeline 的每个阶段：

```
discovery ──> intelligence ──> drafting ──> redteam ──> qa ──> delivery ──> deployment
   │              │               │            │         │        │            │
   ▼              ▼               ▼            ▼         ▼        ▼            ▼
 知识追踪     知识追踪+       验证规则     验证规则   质量门禁   度量收集     度量报告
              Prompt模板      ADR生成      红队规则   验证规则   ADR归档     最终审计
```

---

## 3. Validation Rule Engine — 验证规则引擎

### 3.1 概述

验证规则引擎允许通过 YAML 文件定义质量检查规则，并在 pipeline 的指定阶段自动执行。内置 14 条默认规则，覆盖文档质量、代码规范、安全基线和架构合规。

### 3.2 内置规则分类

| 类别 | 规则数 | 示例 |
|------|--------|------|
| 文档质量 | 4 | PRD 必须包含用户故事、架构文档必须有组件图 |
| 代码规范 | 3 | 函数复杂度上限、测试覆盖率下限、导入排序 |
| 安全基线 | 4 | 无硬编码密钥、依赖漏洞扫描、OWASP Top 10 检查 |
| 架构合规 | 3 | 分层依赖方向、API 契约一致性、数据库迁移脚本存在 |

### 3.3 自定义规则编写指南

自定义规则文件位置: `.super-dev/rules/custom_rules.yaml`

#### 规则结构

```yaml
rules:
  - id: custom-001
    name: "API 响应时间限制"
    description: "所有 API 端点响应时间必须低于 500ms"
    severity: error          # error | warning | info
    phase: qa                # 执行阶段: discovery | drafting | redteam | qa | delivery
    category: performance
    condition:
      type: metric_threshold
      metric: api_response_time_p95
      operator: lt
      value: 500
    message: "API P95 响应时间 {{actual}}ms 超过阈值 500ms"

  - id: custom-002
    name: "必须包含 CHANGELOG"
    description: "交付包必须包含 CHANGELOG.md"
    severity: warning
    phase: delivery
    category: documentation
    condition:
      type: file_exists
      path: CHANGELOG.md
    message: "交付包缺少 CHANGELOG.md 文件"

  - id: custom-003
    name: "禁止使用 eval()"
    description: "代码中不得包含 eval() 调用"
    severity: error
    phase: redteam
    category: security
    condition:
      type: pattern_absent
      glob: "**/*.py"
      pattern: "eval\\("
    message: "检测到 eval() 调用: {{file}}:{{line}}"
```

#### 条件类型 (condition.type)

| 类型 | 说明 | 必填参数 |
|------|------|----------|
| `file_exists` | 检查文件是否存在 | `path` |
| `file_absent` | 检查文件不存在 | `path` |
| `pattern_present` | 检查文件内容包含正则 | `glob`, `pattern` |
| `pattern_absent` | 检查文件内容不包含正则 | `glob`, `pattern` |
| `metric_threshold` | 检查度量值 | `metric`, `operator`, `value` |
| `dependency_check` | 检查依赖版本 | `package`, `version_constraint` |
| `custom_script` | 执行自定义脚本 | `script`, `expected_exit_code` |

#### 跳过规则

在 `super-dev.yaml` 中配置:

```yaml
validation_rules:
  skip_rules:
    - custom-001    # 按 ID 跳过
    - "doc-*"       # 按通配符跳过
```

---

## 4. Knowledge Tracker — 知识引用追踪

### 4.1 概述

Knowledge Tracker 记录 pipeline 运行过程中引用了哪些知识文件，生成引用报告和覆盖率分析。这解决了"AI 到底参考了哪些资料"的透明性问题。

### 4.2 追踪范围

- `knowledge/` 目录下的所有文件
- `output/knowledge-cache/*-knowledge-bundle.json` 缓存文件
- Pipeline 各阶段的知识读取操作

### 4.3 引用报告结构

```json
{
  "run_id": "20260328-143022",
  "total_knowledge_files": 142,
  "referenced_files": 37,
  "coverage": 0.26,
  "by_domain": {
    "security": { "total": 18, "referenced": 12, "coverage": 0.67 },
    "architecture": { "total": 15, "referenced": 8, "coverage": 0.53 },
    "frontend": { "total": 22, "referenced": 5, "coverage": 0.23 }
  },
  "unreferenced_critical": [
    "knowledge/security/01-standards/owasp-top10.md",
    "knowledge/architecture/04-antipatterns/monolith-trap.md"
  ],
  "reference_chain": [
    {
      "phase": "intelligence",
      "file": "knowledge/security/01-standards/web-security-complete.md",
      "sections_used": ["Authentication", "CSRF Protection"],
      "downstream_artifacts": ["output/proj-architecture.md"]
    }
  ]
}
```

### 4.4 使用方法

```bash
# 生成知识引用报告
super-dev governance knowledge-report

# 查看覆盖率摘要
super-dev governance knowledge-report --summary

# 检查是否达到最低覆盖率
super-dev governance knowledge-report --check --min-coverage 0.6
```

### 4.5 最佳实践

1. **在 CI 中集成覆盖率检查**: 确保关键领域（安全、架构）的知识覆盖率不低于 60%
2. **定期审查未引用文件**: 未引用的知识文件可能已过时，需要更新或归档
3. **利用引用链路做根因分析**: 当交付质量出问题时，追溯知识引用链路可以发现是否遗漏了关键输入

---

## 5. Prompt Templates — Prompt 模板版本管理

### 5.1 概述

Pipeline 各阶段使用的 Prompt 模板存储为版本化的 Markdown 文件，位于 `super_dev/templates/` 目录。这使得 Prompt 变更可追踪、可回滚、可 A/B 测试。

### 5.2 目录结构

```
super_dev/templates/
├── discovery/
│   ├── requirement_analysis.v1.md
│   └── requirement_analysis.v2.md
├── intelligence/
│   ├── research_prompt.v1.md
│   └── knowledge_synthesis.v1.md
├── drafting/
│   ├── prd_generation.v1.md
│   ├── architecture_generation.v1.md
│   └── uiux_generation.v1.md
├── redteam/
│   ├── security_review.v1.md
│   └── performance_review.v1.md
└── qa/
    ├── quality_gate.v1.md
    └── code_review.v1.md
```

### 5.3 模板格式

```markdown
---
id: prd_generation
version: 2
created: 2026-03-15
author: super-dev
variables:
  - name: project_name
    required: true
  - name: research_summary
    required: true
  - name: knowledge_constraints
    required: false
    default: "无特殊约束"
---

# PRD Generation Prompt

你正在为 {{project_name}} 生成产品需求文档。

## 背景信息

{{research_summary}}

## 约束条件

{{knowledge_constraints}}

## 输出要求

...
```

### 5.4 版本策略

| 策略 | 说明 | 适用场景 |
|------|------|----------|
| `semver` | 语义化版本 (v1, v2, v3) | 默认，适合大多数项目 |
| `date` | 日期版本 (20260328) | 频繁迭代的团队 |
| `hash` | Git commit hash | 需要精确追溯的场景 |

### 5.5 最佳实践

1. **不要删除旧版本模板**: 保留历史版本用于对比和回滚
2. **在模板 frontmatter 中记录变更原因**: 便于团队理解每次改动的动机
3. **先在非关键阶段测试新模板**: 例如先在 discovery 阶段测试，稳定后再推广到 drafting
4. **利用 variables 抽取可变部分**: 避免为不同项目复制整个模板

---

## 6. ADR Generator — 架构决策记录

### 6.1 概述

ADR (Architecture Decision Record) 是记录重要架构决策的轻量级文档。Super Dev 的 ADR Generator 从架构配置（`super-dev.yaml` 和 `output/*-architecture.md`）中自动提取技术选型，生成标准化的 ADR 文档。

### 6.2 自动提取的决策类型

| 决策类型 | 数据来源 | 示例 |
|----------|----------|------|
| 前端框架选择 | `super-dev.yaml: frontend` | ADR-001: 使用 React 作为前端框架 |
| 后端框架选择 | `super-dev.yaml: backend` | ADR-002: 使用 Node.js + Express |
| 数据库选择 | `super-dev.yaml: database` | ADR-003: 使用 PostgreSQL |
| 部署平台 | `super-dev.yaml: platform` | ADR-004: 选择 Web 平台部署 |
| 架构模式 | `output/*-architecture.md` | ADR-005: 采用前后端分离架构 |
| 安全方案 | Red-team 报告 | ADR-006: 实施 OWASP Top 10 防护 |

### 6.3 ADR 格式 (MADR)

```markdown
# ADR-001: 使用 React 作为前端框架

## 状态

已接受 (Accepted)

## 背景

项目需要构建交互式 Web 前端，需要选择前端框架。
团队对 React 生态有丰富经验。

## 决策

使用 React 18+ 配合 Vite 构建工具。

## 理由

- 团队熟悉度高，降低学习成本
- 生态成熟，组件库丰富
- Vite 提供快速的开发体验
- TypeScript 支持完善

## 后果

- 正面: 开发效率高，招聘容易
- 负面: 包体积较大，需要关注性能优化
- 风险: React 大版本升级可能带来迁移成本

## 相关 ADR

- ADR-005: 前后端分离架构
```

### 6.4 使用方法

```bash
# 从当前项目配置生成所有 ADR
super-dev governance adr generate

# 只生成指定类别
super-dev governance adr generate --category frontend,database

# 列出已生成的 ADR
super-dev governance adr list

# 导出为单一文档
super-dev governance adr export --format markdown
```

### 6.5 使用场景

1. **新项目启动时**: 自动为技术选型生成 ADR，建立决策追溯基线
2. **架构变更时**: 在修改 `super-dev.yaml` 或架构文档后重新生成，记录演进历史
3. **交付审计时**: ADR 作为交付证据包的一部分，证明技术决策经过了评估
4. **团队 onboarding**: 新成员通过 ADR 快速了解项目的技术决策和背后的考量

---

## 7. Pipeline Metrics — 交付效能度量

### 7.1 概述

Pipeline Metrics 追踪交付效能指标，包括 DORA 四项指标和 Rework Rate。数据存储在本地，可导出供外部 BI 系统消费。

### 7.2 度量指标

#### DORA 四项指标

| 指标 | 说明 | 数据来源 |
|------|------|----------|
| Deployment Frequency | 部署频率 | pipeline delivery 阶段完成次数 |
| Lead Time for Changes | 变更前置时间 | 从 discovery 到 delivery 的耗时 |
| Change Failure Rate | 变更失败率 | quality gate 未通过的比例 |
| Time to Restore | 恢复时间 | 从失败到下次成功的耗时 |

#### 扩展指标

| 指标 | 说明 |
|------|------|
| Rework Rate | 返工率 — 需要重新执行的阶段占比 |
| Knowledge Coverage | 知识覆盖率 — 来自 Knowledge Tracker |
| Rule Pass Rate | 规则通过率 — 来自 Validation Rule Engine |
| Gate Score Trend | 质量门禁分数趋势 |

### 7.3 数据存储

度量数据以 JSON 格式存储在 `output/metrics/` 目录:

```
output/metrics/
├── pipeline-runs.jsonl        # 每次 pipeline 运行记录 (JSON Lines)
├── dora-summary.json          # DORA 指标汇总
└── weekly-report.json         # 周报数据
```

### 7.4 使用方法

```bash
# 显示当前项目的效能指标
super-dev governance metrics show

# 显示最近 30 天的趋势
super-dev governance metrics show --period 30d

# 导出为 JSON
super-dev governance metrics export --output metrics-export.json

# 生成周报
super-dev governance metrics weekly-report
```

### 7.5 指标解读

| 等级 | Deployment Frequency | Lead Time | Change Failure Rate | Time to Restore |
|------|---------------------|-----------|---------------------|-----------------|
| Elite | 按需 (每天多次) | < 1 天 | < 5% | < 1 小时 |
| High | 每天至每周 | 1 天 - 1 周 | 5% - 10% | < 1 天 |
| Medium | 每周至每月 | 1 周 - 1 月 | 10% - 20% | 1 天 - 1 周 |
| Low | 每月以上 | > 1 月 | > 20% | > 1 周 |

---

## 8. Pipeline Governance 集成层

### 8.1 概述

`super_dev/orchestrator/governance.py` 是治理集成层，负责：
- 在 pipeline 各阶段自动调用对应的治理能力
- 汇总所有治理结果生成统一报告
- 根据配置决定是否阻断 pipeline

### 8.2 阶段与治理能力映射

```yaml
governance_hooks:
  discovery:
    - knowledge_tracker.start_tracking
  intelligence:
    - knowledge_tracker.record_references
    - prompt_templates.load_template
  drafting:
    - validation_rules.check("drafting")
    - prompt_templates.load_template
    - adr_generator.extract_decisions
  redteam:
    - validation_rules.check("redteam")
  qa:
    - validation_rules.check("qa")
    - pipeline_metrics.record_gate_score
  delivery:
    - validation_rules.check("delivery")
    - knowledge_tracker.generate_report
    - adr_generator.finalize
    - pipeline_metrics.record_completion
  deployment:
    - pipeline_metrics.record_deployment
    - pipeline_metrics.generate_summary
```

### 8.3 统一治理报告

pipeline 完成后，governance 层生成统一报告 (`output/governance-report.json`):

```json
{
  "run_id": "20260328-143022",
  "overall_status": "passed",
  "validation_rules": {
    "total": 17,
    "passed": 15,
    "warnings": 2,
    "errors": 0
  },
  "knowledge_coverage": 0.72,
  "adr_count": 6,
  "pipeline_duration_minutes": 45,
  "quality_gate_score": 92,
  "rework_count": 1
}
```

---

## 9. Agent Checklist

以下清单供 AI Agent（Claude Code、Cursor 等）在执行 Super Dev pipeline 时参考：

### 9.1 Pipeline 启动前

- [ ] 确认 `super-dev.yaml` 中治理配置已正确设置
- [ ] 确认 `.super-dev/rules/custom_rules.yaml` 存在（如有自定义规则）
- [ ] 确认 `knowledge/` 目录内容为最新版本
- [ ] 确认 `super_dev/templates/` 中模板版本正确

### 9.2 各阶段检查

- [ ] **discovery**: Knowledge Tracker 已启动追踪
- [ ] **intelligence**: 知识引用已记录，Prompt 模板已加载
- [ ] **drafting**: 验证规则已执行，ADR 决策已提取
- [ ] **redteam**: 安全类验证规则已执行
- [ ] **qa**: 质量门禁分数已记录，所有 error 级规则已通过
- [ ] **delivery**: 知识引用报告已生成，ADR 已归档，度量已记录
- [ ] **deployment**: 部署度量已记录，统一治理报告已生成

### 9.3 交付前确认

- [ ] 统一治理报告 (`output/governance-report.json`) 状态为 passed
- [ ] 知识覆盖率达到配置阈值（默认 0.6）
- [ ] 所有 error 级验证规则通过
- [ ] ADR 文档已包含在交付证据包中
- [ ] DORA 指标已更新

### 9.4 常见问题排查

| 问题 | 检查项 | 解决方案 |
|------|--------|----------|
| 验证规则未执行 | `validation_rules.enabled` | 检查 `super-dev.yaml` 配置 |
| 知识覆盖率为 0 | Knowledge Tracker 启动 | 确认 `knowledge/` 目录非空 |
| ADR 生成为空 | 架构配置 | 确认 `super-dev.yaml` 有 frontend/backend/database 配置 |
| 度量数据缺失 | metrics 目录权限 | 确认 `output/metrics/` 目录可写 |
| 自定义规则不生效 | YAML 语法 | 使用 `super-dev governance rules validate` 检查 |

### 9.5 治理能力启用矩阵

| 治理能力 | 默认状态 | 推荐级别 | 企业级别 |
|----------|----------|----------|----------|
| Validation Rules | 启用 (warning) | 启用 (error) | 启用 (error + fail_on_warning) |
| Knowledge Tracker | 启用 | 启用 + 覆盖率检查 | 启用 + min_coverage: 0.8 |
| Prompt Templates | 启用 | 启用 + semver | 启用 + 审批流程 |
| ADR Generator | 启用 | 启用 | 启用 + 强制归档 |
| Pipeline Metrics | 启用 | 启用 + 周报 | 启用 + 外部 BI 集成 |

---

## 10. 参考资料

- [DORA Metrics](https://dora.dev/) — DevOps Research and Assessment
- [MADR](https://adr.github.io/madr/) — Markdown Architectural Decision Records
- [OWASP Top 10](https://owasp.org/www-project-top-ten/) — Web 应用安全风险
- Super Dev 源码: `super_dev/orchestrator/governance.py`
- Super Dev 配置: `super-dev.yaml`
- Super Dev 自定义规则示例: `.super-dev/rules/custom_rules.yaml`
