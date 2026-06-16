---
id: ai-rag-engineering-playbook
title: ai-rag-engineering-playbook
domain: ai
category: ai-rag-engineering-playbook.md
difficulty: intermediate
tags: [ai, engineering, playbook, rag, rag工程作战手册]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）

## AI RAG工程作战手册

### 目标
- 构建高召回、高精度、低幻觉的检索增强生成系统。

### 适用范围
- 适用于知识问答、客服助手、内部检索助手和文档智能化场景。

### 核心流程
- 数据接入：来源可信校验、清洗去噪、结构化切片。
- 索引构建：Embedding模型评估、分层索引、增量更新。
- 检索策略：关键词+向量混合检索、重排序、查询改写。
- 生成控制：引用约束、答案结构化、无证据拒答策略。

### 执行清单
- 文档分块策略和chunk overlap经过离线评测。
- 召回率、MRR、NDCG等检索指标具备基线。
- 生成结果必须可追溯到引用片段与版本。

### 验收标准
- 高价值问答场景正确率与可解释性达标。
- 无证据回答比例和幻觉率低于阈值。

### 常见失败模式
- 文档切块过粗导致召回不准，过细导致语义破碎。
- 忽略知识库更新延迟，线上内容与实际文档不一致。

### 回滚策略
- 检索异常时回切到上一个稳定索引快照。
- 关闭激进查询改写，启用保守检索策略。
