---
id: database-review-checklist
title: 数据库 Schema 审查清单
domain: database
category: 03-checklists
difficulty: intermediate
tags: [database, sql, postgresql, review, checklist, index, foreign-key, migration, performance, schema]
quality_score: 89
maintainer: platform-team@superdev.com
last_updated: 2026-06-14
---

# 数据库 Schema 审查清单

## Schema 设计
- [ ] 每张表有 UUID 主键 (`id UUID PRIMARY KEY DEFAULT gen_random_uuid()`)
- [ ] 每张表有 `created_at` + `updated_at` (TIMESTAMPTZ NOT NULL DEFAULT now())
- [ ] 表名复数蛇形 (`users`, `order_items`，不用单数/驼峰/前缀)
- [ ] 列名蛇形语义清晰 (`assignee_id`，不用缩写)
- [ ] 软删除用 `deleted_at TIMESTAMPTZ`（可选），不用 `is_deleted BOOLEAN`
- [ ] 金额用 `NUMERIC(19,2)` 或整数分（不用 FLOAT/REAL）
- [ ] 状态用 `TEXT + CHECK` 约束（不用 ENUM 类型）
- [ ] 时间用 `TIMESTAMPTZ`（不用 TIMESTAMP WITHOUT TIME ZONE）

## 外键关系
- [ ] 每个外键列有显式 `FOREIGN KEY` 约束
- [ ] ON DELETE 策略明确（CASCADE/SET NULL/RESTRICT）
- [ ] 自引用关系（如评论树 parent_id）有约束
- [ ] 外键列有索引（FK 列默认无索引！）

## 索引
- [ ] 每个 WHERE/JOIN/ORDER BY 的列有索引
- [ ] 复合索引遵循最左前缀原则
- [ ] JSONB 列用 GIN 索引
- [ ] 大表的部分索引（WHERE deleted_at IS NULL）
- [ ] 没有 unused indexes（`pg_stat_user_indexes` 检查）

## Migration 安全
- [ ] 新增列有 DEFAULT 值（PG 11+ 不锁表）
- [ ] 大表加索引用 `CONCURRENTLY`（不锁写）
- [ ] 删除列分两步：先标弃用 → 下个版本删除
- [ ] 数据回填脚本幂等（可重跑）
- [ ] 有 down migration（可回滚）

## 查询安全
- [ ] 没有 `SELECT *`（只查需要的列）
- [ ] 没有 N+1 查询（用 JOIN/eager loading）
- [ ] list 查询有 LIMIT（无上限 = OOM 风险）
- [ ] 大表 COUNT 用估算（`pg_class.reltuples`）
- [ ] 参数化查询（防 SQL 注入）
