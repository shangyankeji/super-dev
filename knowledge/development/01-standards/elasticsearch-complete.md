---
id: elasticsearch-complete
title: Elasticsearch完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, elasticsearch, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Elasticsearch完整指南

## 概述
Elasticsearch是分布式搜索引擎,基于Lucene构建,支持全文搜索、结构化搜索、分析等。本指南覆盖索引、查询、聚合、性能优化和最佳实践。

## 核心概念

### 1. 文档和索引

**创建文档**:
```json
PUT /users/_doc/1
{
  "name": "Alice",
  "age": 30,
  "email": "alice@example.com",
  "city": "Beijing",
  "interests": ["python", "elasticsearch", "docker"]
}
```

**批量操作**:
```json
POST /_bulk
{"index": {"_index": "users", "_id": "1"}}
{"name": "Alice", "age": 30}
{"index": {"_index": "users", "_id": "2"}}
{"name": "Bob", "age": 25}
{"delete": {"_index": "users", "_id": "3"}}
```

### 2. 映射(Mapping)

**定义映射**:
```json
PUT /articles
{
  "mappings": {
    "properties": {
      "title": {
        "type": "text",
        "analyzer": "standard"
      },
      "content": {
        "type": "text",
        "analyzer": "english"
      },
      "author": {
        "type": "keyword"
      },
      "publish_date": {
        "type": "date"
      },
      "views": {
        "type": "integer"
      },
      "tags": {
        "type": "keyword"
      }
    }
  }
}
```

### 3. 查询DSL

**基本查询**:
```json
GET /articles/_search
{
  "query": {
    "match": {
      "content": "elasticsearch tutorial"
    }
  }
}

// 多字段搜索
GET /articles/_search
{
  "query": {
    "multi_match": {
      "query": "python",
      "fields": ["title^2", "content"],
      "type": "best_fields"
    }
  }
}

// 精确匹配
GET /articles/_search
{
  "query": {
    "term": {
      "status": "published"
    }
  }
}

// 范围查询
GET /articles/_search
{
  "query": {
    "range": {
      "publish_date": {
        "gte": "2024-01-01",
        "lte": "2024-12-31"
      }
    }
  }
}
```

**布尔查询**:
```json
GET /articles/_search
{
  "query": {
    "bool": {
      "must": [
        {"match": {"content": "elasticsearch"}}
      ],
      "should": [
        {"term": {"tags": "tutorial"}},
        {"term": {"tags": "guide"}}
      ],
      "must_not": [
        {"term": {"status": "draft"}}
      ],
      "filter": [
        {"range": {"views": {"gte": 100}}}
      ]
    }
  }
}
```

### 4. 聚合

**桶聚合**:
```json
GET /articles/_search
{
  "size": 0,
  "aggs": {
    "by_author": {
      "terms": {
        "field": "author",
        "size": 10
      }
    },
    "by_tags": {
      "terms": {
        "field": "tags"
      }
    }
  }
}
```

**指标聚合**:
```json
GET /articles/_search
{
  "size": 0,
  "aggs": {
    "avg_views": {
      "avg": {"field": "views"}
    },
    "max_views": {
      "max": {"field": "views"}
    },
    "stats_views": {
      "stats": {"field": "views"}
    }
  }
}
```

**嵌套聚合**:
```json
GET /articles/_search
{
  "size": 0,
  "aggs": {
    "by_author": {
      "terms": {"field": "author"},
      "aggs": {
        "avg_views": {
          "avg": {"field": "views"}
        }
      }
    }
  }
}
```

### 5. Python客户端

```python
from elasticsearch import Elasticsearch

# 连接
es = Elasticsearch(['http://localhost:9200'])

# 索引文档
doc = {
    'title': 'Elasticsearch Guide',
    'content': 'Complete tutorial for Elasticsearch',
    'author': 'Alice',
    'views': 100
}

es.index(index='articles', id=1, document=doc)

# 批量索引
from elasticsearch.helpers import bulk

actions = [
    {
        '_index': 'articles',
        '_id': i,
        '_source': {
            'title': f'Article {i}',
            'views': i * 10
        }
    }
    for i in range(1000)
]

bulk(es, actions)

# 搜索
result = es.search(
    index='articles',
    query={
        'match': {'content': 'elasticsearch'}
    },
    size=10
)

for hit in result['hits']['hits']:
    print(hit['_source'])

# 聚合
result = es.search(
    index='articles',
    size=0,
    aggs={
        'by_author': {
            'terms': {'field': 'author'}
        }
    }
)
```

## 最佳实践

### ✅ DO

1. **使用批量操作**
```python
# ✅ 好
bulk(es, actions)

# ❌ 差
for doc in docs:
    es.index(index='articles', document=doc)
```

2. **合理设置分片数**
```json
PUT /my_index
{
  "settings": {
    "number_of_shards": 3,
    "number_of_replicas": 1
  }
}
```

3. **使用索引别名**
```json
POST /_aliases
{
  "actions": [
    {"add": {"index": "articles_v1", "alias": "articles"}}
  ]
}
```

### ❌ DON'T

1. **不要过度分片**
```json
// ❌ 差: 分片太多
"number_of_shards": 100

// ✅ 好: 合理分片
"number_of_shards": 3
```

2. **不要在text字段使用term查询**
```json
// ❌ 差
{"term": {"content": "elasticsearch"}}

// ✅ 好
{"match": {"content": "elasticsearch"}}
```

## 学习路径

### 初级 (1-2周)
1. 文档和索引
2. 基本查询
3. 映射

### 中级 (2-3周)
1. 复杂查询
2. 聚合
3. 分析器

### 高级 (2-4周)
1. 集群管理
2. 性能优化
3. 安全配置

---

**知识ID**: `elasticsearch-complete`  
**领域**: development  
**类型**: standards  
**难度**: intermediate  
**质量分**: 93  
**维护者**: dev-team@superdev.com  
**最后更新**: 2026-03-28
