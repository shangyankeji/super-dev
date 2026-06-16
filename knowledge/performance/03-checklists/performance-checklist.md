---
id: performance-checklist
title: 性能审查清单
domain: performance
category: 03-checklists
difficulty: intermediate
tags: [performance, checklist, database, frontend, cache, index, bundle, lcp, lazy-loading, query, redis]
quality_score: 88
maintainer: platform-team@superdev.com
last_updated: 2024-06-14
---

# 性能审查清单

## 数据库
- [ ] 没有 Seq Scan（全表扫描）→ 所有查询命中索引
- [ ] 没有 N+1 查询 → 用 JOIN / eager loading / batch
- [ ] list 端点有 LIMIT + 分页
- [ ] COUNT(*) 大表用估算
- [ ] 连接池大小合理（20 基础 + 10 突发）
- [ ] 查询有超时（statement_timeout 30s）
- [ ] 慢查询日志已启用

## 缓存
- [ ] 热点数据有缓存（Redis / 内存）
- [ ] 缓存有 TTL（不永久缓存）
- [ ] 写操作更新缓存（cache-aside 或 write-through）
- [ ] 缓存击穿防护（singleflight 或互斥锁）
- [ ] 缓存有降级策略（Redis 挂了仍能服务）

## 前端
- [ ] JS bundle < 150KB（gzip）
- [ ] 路由级代码分割（lazy import）
- [ ] 图片用 AVIF/WebP + 响应式 srcset
- [ ] 图片懒加载（loading="lazy"）
- [ ] 字体用 font-display: swap
- [ ] 关键 CSS 内联（首屏不阻塞）

## API 响应
- [ ] 启用 gzip/brotli 压缩
- [ ] 响应有 ETag + Cache-Control
- [ ] 大响应支持字段过滤（?fields=id,name）
- [ ] p95 < 200ms
- [ ] 无阻塞调用（重操作走异步队列）

## Core Web Vitals
- [ ] LCP < 2.5s
- [ ] INP < 200ms
- [ ] CLS < 0.1
- [ ] TTFB < 600ms

## 资源使用
- [ ] CPU 使用率 < 70%（常态）
- [ ] 内存使用率 < 80%（常态）
- [ ] 数据库连接池 < 80%
- [ ] 磁盘 IOPS 在预算内
