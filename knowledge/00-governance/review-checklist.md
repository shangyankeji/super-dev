---
id: review-checklist
title: 全链路知识评审清单
domain: 00-governance
category: 01-standards
difficulty: intermediate
tags: [00-governance, checklist, review, quality, enterprise]
quality_score: 88
last_updated: 2026-06-16
---

# 全链路知识评审清单

> 用于每次知识库变更（新增/修订/归档）发布前的强制检查。每项必须
> 明确"通过 / 不通过 / 不适用"，不通过则阻断发布。

## A. front-matter 合规性

- [ ] `id` 唯一且与文件名 stem 一致。
- [ ] `title` 为有意义的标题（非 `review-checklist.md` 这样的文件名回退）。
- [ ] `domain` 与文件所在路径的第一段一致。
- [ ] `tags` 非空且 ≤ 8 个，每项为小写 kebab-case。
- [ ] `quality_score` 为 0-100 的整数且 ≥ 50。
- [ ] `difficulty` 为 `beginner` / `intermediate` / `advanced` 之一。
- [ ] `last_updated` 为 ISO 8601 日期（YYYY-MM-DD）。

## B. 内容质量

- [ ] 文件字数 ≥ 200 词（CJK 1.5×）。
- [ ] 包含至少 1 个 H2 章节（`## `）。
- [ ] 没有遗留占位符（`TODO`、`FIXME`、`PLACEHOLDER`、`Lorem ipsum`）。
- [ ] 没有硬编码的颜色十六进制值或 emoji 作为功能图标。
- [ ] 外部引用附明来源（官方文档 / RFC / 论文链接）。
- [ ] 代码示例可执行或有明确的"伪代码"标注。

## C. 结构一致性

- [ ] 域内文件遵循 `0X-standards` / `02-playbooks` / `03-checklists` / `04-antipatterns` 分类。
- [ ] 同域文件之间无冲突结论（同一规则只有一个权威版本）。
- [ ] 跨域引用使用相对路径（`../security/01-standards/...`），非绝对路径。

## D. 产品与设计

- [ ] 是否定义目标用户、关键任务、验收指标。
- [ ] 是否明确范围边界与"不做项"。
- [ ] 是否覆盖正常态与异常态。
- [ ] 是否定义高风险操作确认机制。

## E. 架构与开发

- [ ] 是否定义模块边界与依赖方向。
- [ ] 是否给出容量、可用性与回滚策略。
- [ ] 是否有统一错误模型与日志上下文。
- [ ] 是否包含关键路径幂等与重试策略。

## F. 测试与安全

- [ ] 是否覆盖关键业务闭环与异常分支。
- [ ] 是否沉淀线上事故回归用例。
- [ ] 是否覆盖鉴权、授权、审计与敏感数据保护。
- [ ] 是否有漏洞修复闭环与升级策略。

## G. CI/CD 与运维

- [ ] 是否有强制质量门禁与自动回滚触发条件。
- [ ] 是否保留制品追踪与发布审计信息。
- [ ] 是否有可执行 runbook 与告警分级规则。
- [ ] 是否定义 SLO、错误预算与演练计划。

## H. 检索可达性

- [ ] 用 3 个代表性关键词做 BM25 检索，确认该文件能被召回。
- [ ] 文件的 `quality_score` 不会导致它在 hybrid 模式下被过度降权。
- [ ] 如果是经验文件（`.super-dev/learned/`），确认 `tags` 包含 `lesson` 标记。

## 评审流程

1. 贡献者自查 → 提交 PR。
2. 域维护者逐项打勾 → 标注结果。
3. 不通过项返回贡献者修订；通过后合入。
4. 每季度全库批量执行此清单（通过脚本化扫描 front-matter + 内容占位符）。
