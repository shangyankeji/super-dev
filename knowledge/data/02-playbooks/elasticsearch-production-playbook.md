---
id: elasticsearch-production-playbook
title: Elasticsearch 生产实战手册
domain: data
category: 02-playbooks
difficulty: advanced
tags: [elasticsearch, search, mapping, indexing, bulk, query, optimization, production, full-text, analytics, enterprise]
quality_score: 93
maintainer: data-team@superdev.com
last_updated: 2026-06-15
---

# Elasticsearch 生产实战手册

> 基于 [Elastic 官方 Mapping 文档](https://www.elastic.co/docs/manage-data/data-store/mapping) + [Dynamic Mapping](https://www.elastic.co/docs/manage-data/data-store/mapping/dynamic-mapping)

## Mapping 设计（最关键）

### 显式 Mapping（禁止动态推断）
```json
// ❌ 动态 mapping（生产灾难：乱写字段 → mapping 爆炸）
PUT /products  // 不指定 mapping → ES 猜类型 → 字段越来越多

// ✅ 显式 mapping + 禁止动态
PUT /products
{
  "mappings": {
    "dynamic": "strict",          // 未知字段直接拒绝（防止 mapping 爆炸）
    "properties": {
      "name": { "type": "text", "analyzer": "standard" },
      "name_keyword": { "type": "keyword" },  // 文本 + keyword 双字段
      "price": { "type": "scaled_float", "scaling_factor": 100 },  // 分
      "tags": { "type": "keyword" },
      "in_stock": { "type": "boolean" },
      "created_at": { "type": "date" },
      "attrs": { "type": "object", "enabled": false }  // 不索引的大对象
    }
  }
}
```

### 字段类型选择
| 需求 | 类型 | 说明 |
|------|------|------|
| 全文搜索 | text | 分词后索引 |
| 精确匹配/聚合/排序 | keyword | 不分词 |
| 金额 | scaled_float | 精确（不用 double） |
| 时间 | date | 支持 range 查询 |
| 布尔 | boolean | true/false |
| 不搜索的大对象 | object + enabled:false | 省空间 |

### text + keyword 双字段
```json
// ✅ 同一字段既有全文搜索又有精确匹配
"name": {
  "type": "text",           // 全文搜索用 name
  "fields": {
    "keyword": { "type": "keyword" }  // 聚合/排序用 name.keyword
  }
}
```

## 索引策略

### Bulk 批量写入
```python
# ❌ 逐条写入（慢）
for product in products:
    es.index(index="products", body=product)

# ✅ Bulk 批量（快 10-50x）
from elasticsearch.helpers import bulk
actions = [
    {"_index": "products", "_id": p["id"], "_source": p}
    for p in products
]
bulk(es, actions, chunk_size=500)  # 每 500 条一批
```

### 索引别名（零停机重建）
```python
# 用别名而非直接索引名
es.indices.put_alias(index="products_v1", name="products")

# 重建索引时：
es.indices.create("products_v2")  # 新 mapping
# reindex 数据
es.reindex({"source": {"index": "products_v1"}, "dest": {"index": "products_v2"}})
# 切换别名（原子操作，零停机）
es.indices.update_aliases({"actions": [
    {"remove": {"index": "products_v1", "alias": "products"}},
    {"add": {"index": "products_v2", "alias": "products"}},
]})
es.indices.delete("products_v1")  # 删旧索引
```

## 查询优化

```json
// ✅ bool query + filter（filter 不打分，可缓存）
{
  "query": {
    "bool": {
      "must": [
        { "match": { "name": "wireless headphones" } }  // 全文搜索（打分）
      ],
      "filter": [
        { "term": { "tags": "electronics" } },          // 精确过滤（缓存）
        { "range": { "price": { "gte": 50, "lte": 200 } } }
      ]
    }
  },
  "size": 20,           // 分页
  "from": 0,
  "sort": [             // 排序用 keyword 字段
    { "_score": "desc" },
    { "created_at": "desc" }
  ]
}
```

## 生产检查清单
- [ ] 显式 mapping + `dynamic: strict`
- [ ] text + keyword 双字段（搜索 + 聚合）
- [ ] 金额用 scaled_float（不用 double）
- [ ] 大对象用 `enabled: false`
- [ ] Bulk 批量写入
- [ ] 索引别名（零停机重建）
- [ ] filter 替代 query（可缓存）
- [ ] 分页用 search_after（不用大 from）
- [ ] 分片数合理（每分片 10-50GB）
- [ ] 副本至少 1 个（高可用）
