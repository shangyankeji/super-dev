---
id: rag-architecture-complete
title: RAG 架构完整指南
domain: ai
category: 01-standards
difficulty: intermediate
tags: [ai, architecture, complete, embedding, rag, reranking, 向量数据库, 文档分块策略]
quality_score: 70
last_updated: 2026-06-15
---
# RAG 架构完整指南

## 概述

检索增强生成 (Retrieval-Augmented Generation, RAG) 是将外部知识库与 LLM 结合的核心架构模式。本指南覆盖文档分块策略、Embedding 模型选择、向量数据库、检索策略、重排序、评估指标及生产级部署实践。

### RAG 系统架构全景

```
RAG 系统核心组件:
├── 数据摄入层 (Ingestion)
│   ├── 文档解析 (PDF/HTML/Markdown/Office)
│   ├── 文本清洗与预处理
│   ├── 分块策略 (Chunking)
│   └── Embedding 生成
├── 索引层 (Indexing)
│   ├── 向量数据库存储
│   ├── 元数据索引
│   ├── 全文搜索索引
│   └── 增量更新管道
├── 检索层 (Retrieval)
│   ├── 查询理解与改写
│   ├── 向量检索
│   ├── 混合检索 (向量+关键词)
│   ├── 重排序 (Reranking)
│   └── 结果过滤与去重
└── 生成层 (Generation)
    ├── 上下文组装
    ├── Prompt 构建
    ├── LLM 调用
    └── 引用追溯与幻觉检测
```

---

## 1. 文档分块策略

### 1.1 分块方法对比

| 方法 | 适用场景 | 优点 | 缺点 |
|------|---------|------|------|
| 固定大小分块 | 通用文本 | 实现简单，长度可控 | 可能切断语义 |
| 递归字符分割 | 结构化文档 | 保留段落边界 | 需要调参 |
| 语义分块 | 高质量需求 | 语义完整性最好 | 计算成本高 |
| 文档结构分块 | Markdown/HTML | 保留标题层级 | 依赖文档格式 |
| 滑动窗口分块 | 上下文敏感 | 保留跨块上下文 | 存储冗余 |

### 1.2 递归分块实现

```python
from langchain.text_splitter import RecursiveCharacterTextSplitter

def create_splitter(doc_type: str = "general") -> RecursiveCharacterTextSplitter:
    """根据文档类型创建分块器。"""
    configs = {
        "general": {
            "chunk_size": 512,
            "chunk_overlap": 64,
            "separators": ["\n\n", "\n", "。", ".", " "],
        },
        "code": {
            "chunk_size": 1024,
            "chunk_overlap": 128,
            "separators": ["\nclass ", "\ndef ", "\n\n", "\n"],
        },
        "legal": {
            "chunk_size": 768,
            "chunk_overlap": 96,
            "separators": ["\n第.*条", "\n\n", "\n", "。"],
        },
    }
    cfg = configs.get(doc_type, configs["general"])
    return RecursiveCharacterTextSplitter(**cfg)
```

### 1.3 语义分块

```python
import numpy as np

def semantic_chunking(text: str, max_chunk_size: int = 512,
                      similarity_threshold: float = 0.75) -> list[str]:
    """基于语义相似度的分块，在语义断点处切分。"""
    sentences = split_into_sentences(text)
    embeddings = get_embeddings_batch(sentences)

    chunks = []
    current_chunk = [sentences[0]]
    current_size = len(sentences[0])

    for i in range(1, len(sentences)):
        sim = cosine_similarity(embeddings[i - 1], embeddings[i])
        sentence_len = len(sentences[i])

        # 语义断裂或超出大小限制时切分
        if sim < similarity_threshold or current_size + sentence_len > max_chunk_size:
            chunks.append("".join(current_chunk))
            current_chunk = [sentences[i]]
            current_size = sentence_len
        else:
            current_chunk.append(sentences[i])
            current_size += sentence_len

    if current_chunk:
        chunks.append("".join(current_chunk))
    return chunks
```

### 1.4 分块质量评估

```python
def evaluate_chunking(chunks: list[str], queries: list[str],
                      ground_truth: list[str]) -> dict:
    """评估分块策略的检索质量。"""
    chunk_embeddings = get_embeddings_batch(chunks)
    metrics = {"hit_rate": 0, "avg_rank": 0, "semantic_coherence": 0}

    for query, expected in zip(queries, ground_truth):
        query_emb = get_embedding(query)
        scores = [cosine_similarity(query_emb, ce) for ce in chunk_embeddings]
        ranked = sorted(enumerate(scores), key=lambda x: x[1], reverse=True)

        # 检查 top-5 是否包含相关块
        top_5_texts = [chunks[idx] for idx, _ in ranked[:5]]
        if any(expected in t for t in top_5_texts):
            metrics["hit_rate"] += 1

    metrics["hit_rate"] /= len(queries)

    # 语义连贯性: 相邻块的平均相似度
    coherence_scores = []
    for i in range(len(chunk_embeddings) - 1):
        coherence_scores.append(
            cosine_similarity(chunk_embeddings[i], chunk_embeddings[i + 1])
        )
    metrics["semantic_coherence"] = float(np.mean(coherence_scores))

    return metrics
```

---

## 2. Embedding 模型选择

### 2.1 主流模型对比

| 模型 | 维度 | 中文支持 | MTEB 排名 | 特点 |
|------|------|---------|-----------|------|
| text-embedding-3-large | 3072 | 良好 | Top 5 | OpenAI 旗舰，支持维度裁剪 |
| text-embedding-3-small | 1536 | 良好 | Top 20 | 性价比高 |
| voyage-3 | 1024 | 良好 | Top 3 | 代码和多语言优秀 |
| bge-large-zh-v1.5 | 1024 | 优秀 | 中文 Top 3 | 开源，中文专项优化 |
| jina-embeddings-v3 | 1024 | 优秀 | Top 10 | 开源，多语言，长文本 |
| GTE-Qwen2-7B | 3584 | 优秀 | 中文 Top 1 | 开源大模型 Embedding |

### 2.2 Embedding 生成管道

```python
from typing import Protocol
import hashlib

class EmbeddingProvider(Protocol):
    def embed(self, texts: list[str]) -> list[list[float]]: ...

class EmbeddingPipeline:
    """生产级 Embedding 生成管道，支持缓存和批量处理。"""

    def __init__(self, provider: EmbeddingProvider, cache_db=None,
                 batch_size: int = 64):
        self.provider = provider
        self.cache = cache_db
        self.batch_size = batch_size

    def embed_documents(self, documents: list[str]) -> list[list[float]]:
        """批量生成文档 Embedding，支持缓存加速。"""
        results: list[list[float] | None] = [None] * len(documents)
        to_embed: list[tuple[int, str]] = []

        # 查缓存
        for i, doc in enumerate(documents):
            key = self._cache_key(doc)
            if self.cache:
                cached = self.cache.get(key)
                if cached:
                    results[i] = cached
                    continue
            to_embed.append((i, doc))

        # 批量调用
        for batch_start in range(0, len(to_embed), self.batch_size):
            batch = to_embed[batch_start:batch_start + self.batch_size]
            texts = [t for _, t in batch]
            embeddings = self.provider.embed(texts)

            for (idx, doc), emb in zip(batch, embeddings):
                results[idx] = emb
                if self.cache:
                    self.cache.set(self._cache_key(doc), emb, ttl=86400 * 7)

        return results  # type: ignore

    @staticmethod
    def _cache_key(text: str) -> str:
        return f"emb:{hashlib.sha256(text.encode()).hexdigest()[:16]}"
```

---

## 3. 向量数据库

### 3.1 主流方案对比

| 数据库 | 类型 | 最大向量数 | 特色功能 | 适用场景 |
|--------|------|-----------|---------|---------|
| Pinecone | 全托管 | 数十亿 | 命名空间、稀疏向量 | 生产首选，免运维 |
| Weaviate | 自托管/云 | 数十亿 | GraphQL API、多模态 | 复杂查询场景 |
| Qdrant | 自托管/云 | 数十亿 | 过滤性能优秀、Rust 实现 | 高性能需求 |
| Milvus | 自托管 | 千亿级 | 分布式、GPU 加速 | 超大规模场景 |
| pgvector | PostgreSQL 扩展 | 千万级 | 与现有 PG 集成 | 中小规模、已有 PG |
| ChromaDB | 嵌入式 | 百万级 | API 简洁、零配置 | 原型和小项目 |

### 3.2 向量数据库集成

```python
from qdrant_client import QdrantClient
from qdrant_client.models import (
    Distance, VectorParams, PointStruct, Filter,
    FieldCondition, MatchValue,
)
import uuid

class VectorStore:
    """向量数据库封装层，支持多后端切换。"""

    def __init__(self, url: str = "localhost", port: int = 6333):
        self.client = QdrantClient(host=url, port=port)

    def create_collection(self, name: str, vector_size: int = 1024):
        self.client.create_collection(
            collection_name=name,
            vectors_config=VectorParams(
                size=vector_size, distance=Distance.COSINE
            ),
        )

    def upsert(self, collection: str, documents: list[dict]):
        """批量写入文档及其向量。"""
        points = [
            PointStruct(
                id=str(uuid.uuid4()),
                vector=doc["embedding"],
                payload={
                    "text": doc["text"],
                    "source": doc.get("source", ""),
                    "metadata": doc.get("metadata", {}),
                },
            )
            for doc in documents
        ]
        self.client.upsert(collection_name=collection, points=points)

    def search(self, collection: str, query_vector: list[float],
               top_k: int = 10, filters: dict | None = None) -> list[dict]:
        """向量搜索，支持元数据过滤。"""
        search_filter = None
        if filters:
            conditions = [
                FieldCondition(key=k, match=MatchValue(value=v))
                for k, v in filters.items()
            ]
            search_filter = Filter(must=conditions)

        results = self.client.search(
            collection_name=collection,
            query_vector=query_vector,
            limit=top_k,
            query_filter=search_filter,
        )
        return [
            {
                "text": r.payload["text"],
                "score": r.score,
                "source": r.payload.get("source"),
                "metadata": r.payload.get("metadata"),
            }
            for r in results
        ]
```

---

## 4. 检索策略

### 4.1 混合检索

```python
class HybridRetriever:
    """混合检索: 向量检索 + 关键词检索 + 加权融合。"""

    def __init__(self, vector_store: VectorStore, bm25_index,
                 vector_weight: float = 0.7):
        self.vector_store = vector_store
        self.bm25 = bm25_index
        self.vector_weight = vector_weight
        self.keyword_weight = 1.0 - vector_weight

    def search(self, query: str, collection: str,
               top_k: int = 10) -> list[dict]:
        # 向量检索
        query_emb = get_embedding(query)
        vector_results = self.vector_store.search(
            collection, query_emb, top_k=top_k * 2
        )

        # 关键词检索
        keyword_results = self.bm25.search(query, top_k=top_k * 2)

        # Reciprocal Rank Fusion (RRF)
        return self._rrf_merge(vector_results, keyword_results, top_k)

    def _rrf_merge(self, vector_results: list[dict],
                   keyword_results: list[dict],
                   top_k: int, k: int = 60) -> list[dict]:
        """RRF 融合算法: score = sum(1 / (k + rank))。"""
        scores: dict[str, float] = {}
        doc_map: dict[str, dict] = {}

        for rank, doc in enumerate(vector_results):
            doc_id = doc["text"][:100]  # 用文本前缀做去重 key
            scores[doc_id] = scores.get(doc_id, 0) + (
                self.vector_weight / (k + rank + 1)
            )
            doc_map[doc_id] = doc

        for rank, doc in enumerate(keyword_results):
            doc_id = doc["text"][:100]
            scores[doc_id] = scores.get(doc_id, 0) + (
                self.keyword_weight / (k + rank + 1)
            )
            if doc_id not in doc_map:
                doc_map[doc_id] = doc

        ranked = sorted(scores.items(), key=lambda x: x[1], reverse=True)
        return [doc_map[doc_id] for doc_id, _ in ranked[:top_k]]
```

### 4.2 查询改写

```python
QUERY_REWRITE_PROMPT = """
你是一个搜索查询优化专家。将用户的自然语言问题改写为更适合检索的查询。

规则:
1. 生成 3 个不同角度的改写查询
2. 提取关键实体和概念
3. 扩展同义词和相关术语

用户问题: {query}

请按 JSON 格式输出:
```json
{
  "original": "原始查询",
  "rewrites": ["改写1", "改写2", "改写3"],
  "key_entities": ["实体1", "实体2"],
  "expanded_terms": ["同义词1", "相关术语"]
}
```
"""

def multi_query_retrieve(query: str, retriever: HybridRetriever,
                         collection: str, top_k: int = 10) -> list[dict]:
    """多查询检索: 改写查询后并行检索，合并去重。"""
    rewrites = generate_query_rewrites(query)
    all_results = []

    for q in [query] + rewrites:
        results = retriever.search(q, collection, top_k=top_k)
        all_results.extend(results)

    # 去重并按分数排序
    seen = set()
    unique = []
    for r in sorted(all_results, key=lambda x: x["score"], reverse=True):
        key = r["text"][:200]
        if key not in seen:
            seen.add(key)
            unique.append(r)
    return unique[:top_k]
```

---

## 5. 重排序 (Reranking)

### 5.1 Cross-Encoder 重排序

```python
from sentence_transformers import CrossEncoder

class Reranker:
    """基于 Cross-Encoder 的重排序器。"""

    def __init__(self, model_name: str = "BAAI/bge-reranker-v2-m3"):
        self.model = CrossEncoder(model_name)

    def rerank(self, query: str, documents: list[dict],
               top_k: int = 5) -> list[dict]:
        """对检索结果重排序。"""
        pairs = [(query, doc["text"]) for doc in documents]
        scores = self.model.predict(pairs)

        for doc, score in zip(documents, scores):
            doc["rerank_score"] = float(score)

        ranked = sorted(documents, key=lambda x: x["rerank_score"],
                        reverse=True)
        return ranked[:top_k]
```

### 5.2 LLM 重排序 (备选)

```python
RERANK_PROMPT = """
对以下文档按与查询的相关性排序 (最相关在前):

查询: {query}

文档列表:
{documents}

请输出排序后的文档编号列表 (如: [3, 1, 5, 2, 4])。只输出列表，不要解释。
"""
```

---

## 6. 生成与引用追溯

### 6.1 上下文组装

```python
def build_rag_prompt(query: str, retrieved_docs: list[dict],
                     max_context_tokens: int = 4000) -> str:
    """组装 RAG Prompt，控制上下文长度并保留引用信息。"""
    context_parts = []
    total_tokens = 0

    for i, doc in enumerate(retrieved_docs):
        doc_tokens = estimate_tokens(doc["text"])
        if total_tokens + doc_tokens > max_context_tokens:
            break
        context_parts.append(
            f"[来源{i+1}] ({doc.get('source', '未知')})\n{doc['text']}"
        )
        total_tokens += doc_tokens

    context = "\n\n---\n\n".join(context_parts)

    return f"""基于以下参考资料回答用户问题。

## 规则
1. 只使用参考资料中的信息回答
2. 每个论点必须标注来源编号 (如: [来源1])
3. 如果参考资料不足以回答，明确说明"根据现有资料无法完整回答"
4. 不要编造参考资料中没有的信息

## 参考资料
{context}

## 用户问题
{query}

## 回答
"""
```

### 6.2 幻觉检测

```python
HALLUCINATION_CHECK_PROMPT = """
检查以下 AI 回答是否忠实于提供的参考资料。

参考资料:
{context}

AI 回答:
{answer}

请逐句检查，输出 JSON:
```json
{
  "sentences": [
    {
      "text": "回答中的句子",
      "supported": true/false,
      "source": "来源编号或 null",
      "issue": "问题描述或 null"
    }
  ],
  "faithfulness_score": 0.0-1.0,
  "hallucinated_claims": ["幻觉内容列表"]
}
```
"""
```

---

## 7. 评估指标

### 7.1 RAG 评估指标体系

| 指标 | 维度 | 计算方法 | 目标值 |
|------|------|---------|--------|
| Hit Rate@k | 检索 | top-k 中包含相关文档的查询比例 | >= 0.90 |
| MRR | 检索 | 第一个相关文档排名的倒数均值 | >= 0.80 |
| NDCG@k | 检索 | 归一化折损累积增益 | >= 0.75 |
| Faithfulness | 生成 | 回答可追溯到参考文档的比例 | >= 0.90 |
| Answer Relevancy | 生成 | 回答与问题的相关性评分 | >= 0.85 |
| Context Precision | 检索+生成 | 检索文档中与回答相关的比例 | >= 0.70 |
| Context Recall | 检索+生成 | 标准答案中被检索文档覆盖的比例 | >= 0.80 |

### 7.2 RAGAS 评估实现

```python
from ragas import evaluate
from ragas.metrics import (
    faithfulness, answer_relevancy,
    context_precision, context_recall,
)
from datasets import Dataset

def evaluate_rag_system(test_data: list[dict]) -> dict:
    """使用 RAGAS 框架评估 RAG 系统。"""
    dataset = Dataset.from_dict({
        "question": [d["question"] for d in test_data],
        "answer": [d["generated_answer"] for d in test_data],
        "contexts": [d["retrieved_contexts"] for d in test_data],
        "ground_truth": [d["ground_truth"] for d in test_data],
    })

    results = evaluate(
        dataset,
        metrics=[
            faithfulness,
            answer_relevancy,
            context_precision,
            context_recall,
        ],
    )
    return results.to_pandas().describe().to_dict()
```

### 7.3 端到端评估管道

```python
class RAGEvaluationPipeline:
    """端到端 RAG 评估管道。"""

    def __init__(self, rag_system, test_cases: list[dict]):
        self.rag = rag_system
        self.test_cases = test_cases

    def run(self) -> dict:
        results = {
            "retrieval": {"hit_rate": 0, "mrr": 0},
            "generation": {"faithfulness": 0, "relevancy": 0},
            "latency": {"p50_ms": 0, "p95_ms": 0, "p99_ms": 0},
        }
        latencies = []

        for case in self.test_cases:
            start = time.monotonic()
            answer, contexts = self.rag.query(case["question"])
            elapsed = (time.monotonic() - start) * 1000
            latencies.append(elapsed)

            # 检索质量
            relevant = case["relevant_doc_ids"]
            retrieved_ids = [c["id"] for c in contexts]
            if any(r in retrieved_ids[:5] for r in relevant):
                results["retrieval"]["hit_rate"] += 1
            for rank, doc_id in enumerate(retrieved_ids):
                if doc_id in relevant:
                    results["retrieval"]["mrr"] += 1 / (rank + 1)
                    break

        n = len(self.test_cases)
        results["retrieval"]["hit_rate"] /= n
        results["retrieval"]["mrr"] /= n
        results["latency"]["p50_ms"] = round(np.percentile(latencies, 50))
        results["latency"]["p95_ms"] = round(np.percentile(latencies, 95))
        results["latency"]["p99_ms"] = round(np.percentile(latencies, 99))

        return results
```

---

## 8. 生产部署检查项

### 8.1 索引管理

```
索引管理清单:
├── 增量更新 — 新文档自动进入索引管道
├── 版本控制 — 索引快照可回滚到上一版本
├── 过期清理 — 删除的源文档对应向量也被清理
├── 一致性校验 — 定期校验文档数与向量数一致
└── 监控告警 — 索引延迟、大小、查询延迟有告警
```

### 8.2 性能优化

| 优化项 | 方法 | 效果 |
|--------|------|------|
| 向量量化 | PQ/SQ 量化 | 内存降低 4-8 倍 |
| 索引分片 | 按租户或时间分片 | 查询延迟降低 |
| 缓存热查询 | Redis 缓存高频查询结果 | 命中率 30-50% |
| 预计算 Embedding | 离线批量生成 | 在线延迟降低 |
| 上下文压缩 | 只保留相关段落 | Token 成本降低 40% |

---

## Agent Checklist

- [ ] 文档分块策略经过离线评测，chunk_size 和 overlap 有数据支撑
- [ ] Embedding 模型在目标语言和领域数据上做过对比评测
- [ ] 向量数据库支持元数据过滤和增量更新
- [ ] 检索层采用混合检索 (向量+关键词) 并有 RRF 融合
- [ ] 重排序模型部署并在检索结果上验证提升效果
- [ ] 查询改写管道上线并有多查询检索策略
- [ ] 生成 Prompt 强制要求引用来源编号
- [ ] 幻觉检测流程集成到质量监控中
- [ ] RAGAS 或等价评估框架产出基线指标
- [ ] Hit Rate >= 0.90, MRR >= 0.80, Faithfulness >= 0.90
- [ ] 索引有版本管理、增量更新和一致性校验
- [ ] 查询延迟 P95 < 2s，包含检索+重排+生成全链路
