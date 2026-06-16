---
id: elasticsearch-complete
title: Elasticsearch 数据领域完整指南
domain: data
category: 01-standards
difficulty: intermediate
tags: [complete, data, elasticsearch, mapping, 性能优化, 查询, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Elasticsearch 数据领域完整指南

> 文档版本: v1.0 | 最后更新: 2026-03-28

## 概述

Elasticsearch 是一款基于 Apache Lucene 构建的分布式搜索与分析引擎，以近实时（NRT）的全文搜索能力和水平扩展性著称。它不仅是日志分析（ELK Stack）的核心组件，还广泛应用于站内搜索、推荐系统、安全分析、APM（应用性能监控）和向量检索等场景。Elasticsearch 8.x 支持原生向量搜索（kNN）、无需安全配置即默认启用 TLS + 认证、以及 ES|QL 查询语言。

### 全文搜索 vs 结构化搜索

| 维度 | 全文搜索 | 结构化搜索 |
|------|----------|------------|
| 数据类型 | 非结构化文本（文章、日志、评论） | 精确值（状态码、ID、日期范围） |
| 匹配方式 | 分词 -> 倒排索引 -> 相关性评分 | 精确匹配 / 范围过滤，无评分 |
| 典型查询 | `match`、`multi_match`、`match_phrase` | `term`、`range`、`exists`、`bool filter` |
| 是否评分 | 是（`_score`） | 否（filter context，可缓存） |
| 性能特征 | CPU 密集（分词 + 评分） | I/O 密集但可高度缓存 |

### ES 适用场景

| 场景 | 说明 |
|------|------|
| 站内搜索 | 电商商品、文档知识库、CMS 内容检索 |
| 日志与可观测性 | ELK/EFK Stack，集中式日志查询与告警 |
| 安全分析（SIEM） | Elastic Security，威胁检测与事件溯源 |
| APM | 分布式链路追踪、服务拓扑、错误追踪 |
| 推荐/个性化 | 基于用户行为的实时推荐，结合 function_score |
| 向量检索 | 8.x kNN 搜索，语义检索 + 传统检索混合排序 |
| 地理信息 | geo_point / geo_shape 查询，LBS 应用 |
| 指标聚合 | 实时仪表盘、多维度统计分析 |

### 何时不应选择 ES

- **强事务需求**: ES 不支持 ACID 事务，写入后需等待 refresh 才可见（默认 1s）
- **频繁全量更新**: ES 的 update 本质是 delete + reindex，写放大严重
- **主数据存储**: ES 不应作为唯一数据源（Source of Truth），需搭配关系型数据库
- **小数据量精确查询**: 数据量小于百万级且仅需精确查询时，RDBMS 更合适

---

## 核心概念

### 1. Index（索引）

ES 中的索引类似于关系型数据库中的"表"，是文档（Document）的逻辑容器。每个索引有自己的 Mapping（模式定义）和 Settings（分片数、副本数、分析器等）。

```json
// 创建索引
PUT /products
{
  "settings": {
    "number_of_shards": 3,
    "number_of_replicas": 1,
    "refresh_interval": "5s",
    "analysis": {
      "analyzer": {
        "product_analyzer": {
          "type": "custom",
          "tokenizer": "ik_max_word",
          "filter": ["lowercase", "synonym_filter"]
        }
      }
    }
  }
}
```

### 2. Document（文档）

ES 中的最小数据单元，以 JSON 格式存储。每个文档属于一个索引，拥有唯一的 `_id`。

```json
// 索引一个文档
PUT /products/_doc/1
{
  "name": "Apple iPhone 15 Pro",
  "category": "electronics",
  "price": 7999,
  "description": "A17 Pro 芯片，钛金属设计，4800 万像素主摄",
  "tags": ["smartphone", "apple", "5g"],
  "created_at": "2024-09-15T10:30:00Z"
}
```

### 3. Shard（分片）

索引可以被分割为多个分片，每个分片是一个独立的 Lucene 实例。分片是 ES 实现水平扩展的基础。

- **主分片（Primary Shard）**: 数据写入的目标，索引创建后数量不可更改（除非 Reindex 或 Split）
- **副本分片（Replica Shard）**: 主分片的拷贝，提供读取负载均衡和容灾能力

```
Index: products (3P + 1R)
├── Shard 0 (Primary) -> Node A
│   └── Shard 0 (Replica) -> Node B
├── Shard 1 (Primary) -> Node B
│   └── Shard 1 (Replica) -> Node C
└── Shard 2 (Primary) -> Node C
    └── Shard 2 (Replica) -> Node A
```

**分片大小指导原则**:
- 单个分片建议 10GB–50GB（日志场景可放宽至 50GB–80GB）
- 每个节点的分片数量不宜超过 20 个/GB 堆内存
- 避免过度分片（over-sharding），空分片也消耗资源

### 4. Replica（副本）

副本是主分片的完整拷贝，作用包括：

- **高可用**: 主分片所在节点故障时，副本自动提升为主分片
- **读扩展**: 搜索请求可以路由到副本，分摊读压力
- **副本数量可动态调整**:

```json
PUT /products/_settings
{
  "number_of_replicas": 2
}
```

### 5. Segment（段）与近实时搜索

每个分片由多个不可变的 Segment 组成。文档写入流程：

```
写入请求 -> Memory Buffer -> refresh (默认1s) -> 新 Segment (可搜索)
                          -> Translog (持久化保障)
                                    -> flush -> Segment 持久化到磁盘
                                              -> Translog 清空
```

- **refresh**: 将内存缓冲区的数据写入新 Segment，使其可被搜索（默认每 1 秒）
- **flush**: 将 Segment 持久化到磁盘并清空 Translog（由 ES 自动管理）
- **merge**: 后台定期合并小 Segment 为大 Segment，回收已删除文档的空间

```json
// 手动 refresh（通常不需要，测试场景使用）
POST /products/_refresh

// 调整 refresh 间隔（大批量写入时可临时关闭）
PUT /products/_settings
{
  "refresh_interval": "-1"
}
```

### 6. 倒排索引（Inverted Index）原理

倒排索引是 ES 全文搜索的核心数据结构，将分词后的 Term 映射到包含该 Term 的文档列表。

```
原始文档:
  Doc 1: "Elasticsearch 是一个搜索引擎"
  Doc 2: "Elasticsearch 支持全文搜索"
  Doc 3: "Lucene 是搜索引擎的基础"

分词后的倒排索引:
  Term            -> Posting List (DocID, Position, Frequency)
  ─────────────────────────────────────────────────────────
  elasticsearch   -> [{doc:1, pos:0, freq:1}, {doc:2, pos:0, freq:1}]
  搜索引擎        -> [{doc:1, pos:1, freq:1}, {doc:3, pos:1, freq:1}]
  全文搜索        -> [{doc:2, pos:1, freq:1}]
  lucene          -> [{doc:3, pos:0, freq:1}]
  基础            -> [{doc:3, pos:2, freq:1}]
```

倒排索引的关键组件：
- **Term Dictionary**: 所有分词后的词项，排序存储
- **Posting List**: 每个词项对应的文档ID列表 + 位置信息 + 词频
- **Doc Values**: 列式存储，用于排序和聚合（非倒排索引的一部分，但同样重要）
- **Stored Fields**: 原始字段值的行式存储（用于 `_source` 返回）

---

## Mapping 设计

Mapping 定义了索引中每个字段的数据类型和索引方式，类似于 RDBMS 中的 Schema。

### 常用字段类型

| 类型 | 说明 | 索引方式 |
|------|------|----------|
| `text` | 全文搜索字段，会被分词 | 倒排索引 |
| `keyword` | 精确值，不分词 | 倒排索引（整体作为单一 Term） |
| `long` / `integer` / `short` / `byte` | 整数 | BKD Tree |
| `float` / `double` / `half_float` / `scaled_float` | 浮点数 | BKD Tree |
| `date` | 日期时间 | BKD Tree |
| `boolean` | 布尔值 | 倒排索引 |
| `object` | JSON 对象（扁平化存储） | 各子字段独立索引 |
| `nested` | 嵌套对象（保持内部字段关联性） | 独立隐藏文档 |
| `geo_point` | 经纬度坐标 | BKD Tree |
| `geo_shape` | 任意 GeoJSON 几何形状 | BKD Tree |
| `dense_vector` | 密集向量（用于 kNN 搜索） | HNSW 图索引 |
| `ip` | IPv4 / IPv6 地址 | BKD Tree |
| `completion` | 自动补全（FST 结构） | 专用索引 |
| `join` | 父子关系 | 路由至同一分片 |

### 动态映射（Dynamic Mapping）

ES 默认开启动态映射，自动推断字段类型。生产环境建议关闭或设为 strict：

```json
PUT /orders
{
  "mappings": {
    "dynamic": "strict",
    "properties": {
      "order_id": { "type": "keyword" },
      "amount": { "type": "scaled_float", "scaling_factor": 100 },
      "status": { "type": "keyword" },
      "created_at": { "type": "date", "format": "yyyy-MM-dd'T'HH:mm:ssZ||epoch_millis" }
    }
  }
}
```

动态映射策略：
- `true`（默认）: 自动添加新字段
- `false`: 新字段被忽略（不索引，但仍存储在 `_source`）
- `strict`: 遇到未定义字段直接报错
- `runtime`: 新字段作为运行时字段（不持久化索引）

### 多字段（Multi-fields）

同一字段可以用不同方式索引，满足不同查询需求：

```json
{
  "mappings": {
    "properties": {
      "product_name": {
        "type": "text",
        "analyzer": "ik_max_word",
        "search_analyzer": "ik_smart",
        "fields": {
          "keyword": {
            "type": "keyword",
            "ignore_above": 256
          },
          "pinyin": {
            "type": "text",
            "analyzer": "pinyin_analyzer"
          },
          "suggest": {
            "type": "completion"
          }
        }
      }
    }
  }
}
```

查询时通过 `product_name`（全文搜索）、`product_name.keyword`（精确匹配/聚合）、`product_name.pinyin`（拼音搜索）访问不同子字段。

### 分析器链路（Analyzer Chain）

分析器由三部分组成，按顺序处理文本：

```
原始文本 -> Character Filter -> Tokenizer -> Token Filter -> Terms
           (字符过滤)          (分词器)      (词项过滤)
```

- **Character Filter**: 在分词前处理原始文本（如去除 HTML 标签、字符映射）
- **Tokenizer**: 将文本拆分为词项（Token）
- **Token Filter**: 对词项进行转换（如小写、同义词、停用词移除、词干提取）

### 自定义分析器

```json
PUT /articles
{
  "settings": {
    "analysis": {
      "char_filter": {
        "html_strip_filter": {
          "type": "html_strip",
          "escaped_tags": ["b", "em"]
        },
        "ampersand_mapping": {
          "type": "mapping",
          "mappings": ["& => and", "| => or"]
        }
      },
      "tokenizer": {
        "my_edge_ngram": {
          "type": "edge_ngram",
          "min_gram": 2,
          "max_gram": 15,
          "token_chars": ["letter", "digit"]
        }
      },
      "filter": {
        "my_stopwords": {
          "type": "stop",
          "stopwords": ["的", "了", "是", "在", "和"]
        },
        "synonym_filter": {
          "type": "synonym_graph",
          "synonyms_path": "analysis/synonyms.txt",
          "updateable": true
        },
        "my_pinyin": {
          "type": "pinyin",
          "keep_full_pinyin": true,
          "keep_joined_full_pinyin": true,
          "keep_original": true,
          "limit_first_letter_length": 16,
          "remove_duplicated_term": true
        }
      },
      "analyzer": {
        "article_analyzer": {
          "type": "custom",
          "char_filter": ["html_strip_filter", "ampersand_mapping"],
          "tokenizer": "ik_max_word",
          "filter": ["lowercase", "my_stopwords", "synonym_filter"]
        },
        "search_analyzer": {
          "type": "custom",
          "tokenizer": "ik_smart",
          "filter": ["lowercase", "synonym_filter"]
        },
        "autocomplete_analyzer": {
          "type": "custom",
          "tokenizer": "my_edge_ngram",
          "filter": ["lowercase"]
        }
      }
    }
  },
  "mappings": {
    "properties": {
      "title": {
        "type": "text",
        "analyzer": "article_analyzer",
        "search_analyzer": "search_analyzer"
      },
      "content": {
        "type": "text",
        "analyzer": "article_analyzer"
      }
    }
  }
}
```

### 中文分词：IK Analyzer

IK 是 ES 中使用最广泛的中文分词插件，提供两种分词模式：

| 模式 | 说明 | 适用场景 |
|------|------|----------|
| `ik_max_word` | 最细粒度切分，穷尽所有可能的组合 | 索引时使用，提高召回率 |
| `ik_smart` | 最粗粒度切分，不重复 | 搜索时使用，提高精确率 |

```bash
# 安装 IK 插件
./bin/elasticsearch-plugin install https://github.com/medcl/elasticsearch-analysis-ik/releases/download/v8.x.x/elasticsearch-analysis-ik-8.x.x.zip

# 测试分词效果
POST /_analyze
{
  "analyzer": "ik_max_word",
  "text": "中华人民共和国国歌"
}
# 结果: ["中华人民共和国", "中华人民", "中华", "华人", "人民共和国", "人民", "共和国", "共和", "国歌"]
```

**自定义词典**:

```xml
<!-- config/IKAnalyzer.cfg.xml -->
<properties>
    <entry key="ext_dict">custom/custom_dict.dic</entry>
    <entry key="ext_stopwords">custom/custom_stopwords.dic</entry>
    <entry key="remote_ext_dict">http://dict-server/hot_words.txt</entry>
    <entry key="remote_ext_stopwords">http://dict-server/stop_words.txt</entry>
</properties>
```

### 同义词配置

```
# analysis/synonyms.txt
# 格式一：等价同义词
手机,手提电话,移动电话
电脑,计算机,PC

# 格式二：单向映射
iPhone => 苹果手机
MacBook => 苹果笔记本

# 格式三：缩写展开
ES => Elasticsearch
K8s => Kubernetes
```

同义词过滤器建议在 search_analyzer 中使用而非 index_analyzer，以便更新同义词时无需重建索引。

---

## 查询 DSL

ES 查询分为两种上下文：
- **Query Context**: 计算相关性评分（`_score`），用于全文搜索
- **Filter Context**: 不计算评分，结果可缓存，用于精确过滤

### match 查询

```json
// 基础 match：对搜索词分词后查询
GET /products/_search
{
  "query": {
    "match": {
      "description": {
        "query": "苹果手机 拍照",
        "operator": "or",
        "minimum_should_match": "75%"
      }
    }
  }
}

// match_phrase：短语匹配，保持词序
GET /products/_search
{
  "query": {
    "match_phrase": {
      "description": {
        "query": "钛金属设计",
        "slop": 1
      }
    }
  }
}

// multi_match：跨多字段搜索
GET /products/_search
{
  "query": {
    "multi_match": {
      "query": "苹果手机",
      "fields": ["name^3", "description^1", "tags^2"],
      "type": "best_fields",
      "tie_breaker": 0.3
    }
  }
}
```

`multi_match` type 说明：
- `best_fields`: 取最佳匹配字段的得分（默认）
- `most_fields`: 所有匹配字段得分之和
- `cross_fields`: 将多字段视为一个大字段
- `phrase`: 对每个字段执行 match_phrase
- `phrase_prefix`: 对每个字段执行 match_phrase_prefix

### term 查询

```json
// term：精确匹配（不分词）
GET /products/_search
{
  "query": {
    "term": {
      "status": {
        "value": "active"
      }
    }
  }
}

// terms：多值精确匹配（类似 SQL IN）
GET /products/_search
{
  "query": {
    "terms": {
      "category": ["electronics", "accessories"]
    }
  }
}

// exists：字段存在性检查
GET /products/_search
{
  "query": {
    "exists": {
      "field": "discount_price"
    }
  }
}
```

> **警告**: 不要对 `text` 类型字段使用 `term` 查询。`text` 字段在索引时会被分词，而 `term` 不分词，几乎不会匹配。应使用 `keyword` 子字段。

### bool 查询

```json
GET /products/_search
{
  "query": {
    "bool": {
      "must": [
        { "match": { "description": "智能手机" } }
      ],
      "must_not": [
        { "term": { "status": "discontinued" } }
      ],
      "should": [
        { "term": { "brand.keyword": "Apple" } },
        { "term": { "brand.keyword": "Samsung" } }
      ],
      "minimum_should_match": 1,
      "filter": [
        { "range": { "price": { "gte": 3000, "lte": 10000 } } },
        { "term": { "in_stock": true } }
      ]
    }
  }
}
```

- `must`: 必须匹配，贡献评分
- `must_not`: 必须不匹配，不贡献评分（filter context）
- `should`: 可选匹配，贡献评分。若无 must/filter，至少一个 should 必须匹配
- `filter`: 必须匹配，不贡献评分（filter context，结果可缓存）

### range 查询

```json
GET /orders/_search
{
  "query": {
    "range": {
      "created_at": {
        "gte": "2024-01-01",
        "lt": "2024-07-01",
        "format": "yyyy-MM-dd",
        "time_zone": "+08:00"
      }
    }
  }
}
```

### nested 查询

当使用 `nested` 类型存储嵌套对象时，必须使用 `nested` 查询：

```json
// Mapping
PUT /orders
{
  "mappings": {
    "properties": {
      "order_id": { "type": "keyword" },
      "items": {
        "type": "nested",
        "properties": {
          "product_name": { "type": "text", "analyzer": "ik_smart" },
          "quantity": { "type": "integer" },
          "unit_price": { "type": "float" }
        }
      }
    }
  }
}

// 查询：找到有某个商品且数量 > 5 的订单
GET /orders/_search
{
  "query": {
    "nested": {
      "path": "items",
      "query": {
        "bool": {
          "must": [
            { "match": { "items.product_name": "键盘" } },
            { "range": { "items.quantity": { "gt": 5 } } }
          ]
        }
      },
      "inner_hits": {
        "size": 3,
        "_source": ["items.product_name", "items.quantity"]
      }
    }
  }
}
```

> **注意**: `object` 类型会扁平化存储嵌套对象的字段，导致不同对象的字段之间产生错误交叉匹配。如果需要保持嵌套对象内部字段的关联性，必须使用 `nested` 类型。

### has_child / has_parent 查询

```json
// Mapping（父子关系）
PUT /qa_forum
{
  "mappings": {
    "properties": {
      "relation_type": {
        "type": "join",
        "relations": {
          "question": "answer"
        }
      },
      "title": { "type": "text" },
      "content": { "type": "text" },
      "votes": { "type": "integer" }
    }
  }
}

// 索引父文档（问题）
PUT /qa_forum/_doc/q1
{
  "title": "如何优化 Elasticsearch 查询性能？",
  "relation_type": "question"
}

// 索引子文档（答案），routing 到父文档所在分片
PUT /qa_forum/_doc/a1?routing=q1
{
  "content": "首先确保使用了 filter context...",
  "votes": 42,
  "relation_type": {
    "name": "answer",
    "parent": "q1"
  }
}

// has_child：找到有高赞答案的问题
GET /qa_forum/_search
{
  "query": {
    "has_child": {
      "type": "answer",
      "query": {
        "range": { "votes": { "gte": 10 } }
      },
      "score_mode": "max",
      "min_children": 1
    }
  }
}

// has_parent：找到某个问题的所有答案
GET /qa_forum/_search
{
  "query": {
    "has_parent": {
      "parent_type": "question",
      "query": {
        "match": { "title": "Elasticsearch 性能" }
      }
    }
  }
}
```

### function_score / script_score

```json
// function_score：自定义评分
GET /products/_search
{
  "query": {
    "function_score": {
      "query": { "match": { "name": "手机" } },
      "functions": [
        {
          "filter": { "term": { "is_promoted": true } },
          "weight": 5
        },
        {
          "field_value_factor": {
            "field": "sales_count",
            "modifier": "log1p",
            "factor": 0.5
          }
        },
        {
          "gauss": {
            "created_at": {
              "origin": "now",
              "scale": "30d",
              "offset": "7d",
              "decay": 0.5
            }
          }
        },
        {
          "random_score": {
            "seed": 12345,
            "field": "_seq_no"
          }
        }
      ],
      "score_mode": "sum",
      "boost_mode": "multiply",
      "max_boost": 100
    }
  }
}

// script_score：脚本评分（向量相似度等复杂场景）
GET /products/_search
{
  "query": {
    "script_score": {
      "query": { "match_all": {} },
      "script": {
        "source": """
          double textScore = _score;
          double popularity = doc['popularity'].value;
          double recency = decayDateLinear(params.origin, params.scale, params.offset, params.decay, doc['created_at'].value);
          return textScore * 0.5 + Math.log1p(popularity) * 0.3 + recency * 0.2;
        """,
        "params": {
          "origin": "2024-06-01",
          "scale": "90d",
          "offset": "7d",
          "decay": 0.5
        }
      }
    }
  }
}
```

### 聚合（Aggregation）

聚合是 ES 的强大分析功能，支持嵌套组合。

#### Bucket 聚合

```json
// terms 聚合：按类别分组
GET /products/_search
{
  "size": 0,
  "aggs": {
    "by_category": {
      "terms": {
        "field": "category.keyword",
        "size": 20,
        "order": { "_count": "desc" },
        "min_doc_count": 1
      }
    }
  }
}

// date_histogram 聚合：按时间分桶
GET /orders/_search
{
  "size": 0,
  "aggs": {
    "monthly_sales": {
      "date_histogram": {
        "field": "created_at",
        "calendar_interval": "month",
        "format": "yyyy-MM",
        "time_zone": "+08:00",
        "min_doc_count": 0,
        "extended_bounds": {
          "min": "2024-01",
          "max": "2024-12"
        }
      },
      "aggs": {
        "total_revenue": {
          "sum": { "field": "amount" }
        },
        "avg_order_value": {
          "avg": { "field": "amount" }
        }
      }
    }
  }
}

// range 聚合：自定义范围分桶
GET /products/_search
{
  "size": 0,
  "aggs": {
    "price_ranges": {
      "range": {
        "field": "price",
        "ranges": [
          { "key": "budget", "to": 1000 },
          { "key": "mid", "from": 1000, "to": 5000 },
          { "key": "premium", "from": 5000, "to": 10000 },
          { "key": "luxury", "from": 10000 }
        ]
      }
    }
  }
}
```

#### Metrics 聚合

```json
GET /orders/_search
{
  "size": 0,
  "aggs": {
    "revenue_stats": {
      "stats": { "field": "amount" }
    },
    "unique_customers": {
      "cardinality": {
        "field": "customer_id",
        "precision_threshold": 10000
      }
    },
    "percentile_response_time": {
      "percentiles": {
        "field": "response_ms",
        "percents": [50, 90, 95, 99]
      }
    },
    "top_orders": {
      "top_hits": {
        "size": 3,
        "sort": [{ "amount": "desc" }],
        "_source": ["order_id", "amount", "customer_name"]
      }
    }
  }
}
```

#### Pipeline 聚合

```json
GET /orders/_search
{
  "size": 0,
  "aggs": {
    "monthly": {
      "date_histogram": {
        "field": "created_at",
        "calendar_interval": "month"
      },
      "aggs": {
        "monthly_revenue": {
          "sum": { "field": "amount" }
        },
        "revenue_derivative": {
          "derivative": {
            "buckets_path": "monthly_revenue"
          }
        },
        "cumulative_revenue": {
          "cumulative_sum": {
            "buckets_path": "monthly_revenue"
          }
        },
        "moving_avg_revenue": {
          "moving_avg": {
            "buckets_path": "monthly_revenue",
            "window": 3,
            "model": "simple"
          }
        }
      }
    },
    "max_monthly_revenue": {
      "max_bucket": {
        "buckets_path": "monthly>monthly_revenue"
      }
    }
  }
}
```

---

## 索引管理

### 索引别名（Alias）

别名是指向一个或多个索引的虚拟名称，是零停机切换索引的关键手段。

```json
// 创建别名
POST /_aliases
{
  "actions": [
    { "add": { "index": "products_v2", "alias": "products" } },
    { "remove": { "index": "products_v1", "alias": "products" } }
  ]
}

// 过滤别名（虚拟子集视图）
POST /_aliases
{
  "actions": [
    {
      "add": {
        "index": "logs-2024",
        "alias": "logs-error",
        "filter": { "term": { "level": "error" } }
      }
    }
  ]
}

// 写入别名（指定写入目标）
POST /_aliases
{
  "actions": [
    {
      "add": {
        "index": "products_v2",
        "alias": "products_write",
        "is_write_index": true
      }
    }
  ]
}
```

### 滚动索引（Rollover）

```json
// 创建初始索引并关联别名
PUT /logs-000001
{
  "aliases": {
    "logs-write": { "is_write_index": true },
    "logs-read": {}
  },
  "settings": {
    "number_of_shards": 2,
    "number_of_replicas": 1
  }
}

// 手动 Rollover
POST /logs-write/_rollover
{
  "conditions": {
    "max_age": "7d",
    "max_docs": 10000000,
    "max_primary_shard_size": "50gb"
  }
}
// 结果: 创建 logs-000002，logs-write 指向新索引
```

### ILM 生命周期管理

```json
PUT /_ilm/policy/logs_policy
{
  "policy": {
    "phases": {
      "hot": {
        "min_age": "0ms",
        "actions": {
          "rollover": {
            "max_primary_shard_size": "50gb",
            "max_age": "7d"
          },
          "set_priority": { "priority": 100 }
        }
      },
      "warm": {
        "min_age": "30d",
        "actions": {
          "shrink": { "number_of_shards": 1 },
          "forcemerge": { "max_num_segments": 1 },
          "allocate": {
            "require": { "data": "warm" }
          },
          "set_priority": { "priority": 50 }
        }
      },
      "cold": {
        "min_age": "90d",
        "actions": {
          "allocate": {
            "require": { "data": "cold" }
          },
          "freeze": {},
          "set_priority": { "priority": 0 }
        }
      },
      "delete": {
        "min_age": "365d",
        "actions": {
          "delete": {}
        }
      }
    }
  }
}

// 将策略应用到索引模板
PUT /_index_template/logs_template
{
  "index_patterns": ["logs-*"],
  "template": {
    "settings": {
      "number_of_shards": 2,
      "number_of_replicas": 1,
      "index.lifecycle.name": "logs_policy",
      "index.lifecycle.rollover_alias": "logs-write"
    }
  }
}
```

### Reindex

```json
// 基础 Reindex
POST /_reindex
{
  "source": {
    "index": "products_v1",
    "query": {
      "range": { "created_at": { "gte": "2024-01-01" } }
    }
  },
  "dest": {
    "index": "products_v2",
    "pipeline": "enrich_product"
  }
}

// 远程 Reindex（跨集群迁移）
POST /_reindex
{
  "source": {
    "remote": {
      "host": "https://old-cluster:9200",
      "username": "user",
      "password": "pass"
    },
    "index": "products",
    "size": 5000
  },
  "dest": {
    "index": "products"
  }
}

// 异步 Reindex（大数据量场景）
POST /_reindex?wait_for_completion=false&slices=auto
{
  "source": { "index": "big_index" },
  "dest": { "index": "big_index_v2" }
}
// 返回 task_id，通过 GET /_tasks/<task_id> 查看进度
```

### Split / Shrink

```json
// Split：增加分片数（必须是原分片数的整数倍）
POST /products/_split/products_split
{
  "settings": {
    "index.number_of_shards": 6
  }
}

// Shrink：减少分片数（必须是原分片数的因数）
// 前置条件：索引只读 + 所有分片迁移至同一节点
PUT /logs-000001/_settings
{
  "index.routing.allocation.require._name": "shrink_node",
  "index.blocks.write": true
}

POST /logs-000001/_shrink/logs-000001-shrunk
{
  "settings": {
    "index.number_of_shards": 1,
    "index.number_of_replicas": 1,
    "index.routing.allocation.require._name": null,
    "index.blocks.write": null
  }
}
```

---

## 性能优化

### 分片策略

| 原则 | 说明 |
|------|------|
| 单分片大小 | 10–50GB（搜索场景），50–80GB（日志场景） |
| 总分片数 | 每节点分片数 ≤ 20 × JVM 堆内存(GB) |
| 避免过度分片 | 1000 个 1MB 分片远不如 1 个 1GB 分片高效 |
| 时间序列 | 使用 ILM + Rollover，按时间自动拆分 |
| 搜索并行度 | 分片数 = 预期并发搜索数 × 响应时间要求 |

### Routing

默认情况下，文档通过 `hash(_id) % number_of_shards` 路由到分片。自定义 routing 可以将相关文档定位到同一分片，减少搜索时的分片扇出：

```json
// 按租户 ID 路由
PUT /multi_tenant_logs/_doc/1?routing=tenant_abc
{
  "tenant_id": "tenant_abc",
  "message": "User login successful",
  "timestamp": "2024-06-15T10:30:00Z"
}

// 搜索时指定 routing，只查询目标分片
GET /multi_tenant_logs/_search?routing=tenant_abc
{
  "query": {
    "match": { "message": "login" }
  }
}

// Mapping 中强制 routing
PUT /multi_tenant_logs
{
  "mappings": {
    "_routing": { "required": true },
    "properties": {
      "tenant_id": { "type": "keyword" },
      "message": { "type": "text" }
    }
  }
}
```

### Bulk API

批量操作是 ES 写入性能的关键优化手段：

```json
POST /_bulk
{"index": {"_index": "products", "_id": "1"}}
{"name": "Product A", "price": 100}
{"index": {"_index": "products", "_id": "2"}}
{"name": "Product B", "price": 200}
{"update": {"_index": "products", "_id": "1"}}
{"doc": {"price": 150}}
{"delete": {"_index": "products", "_id": "3"}}
```

Bulk 最佳实践：
- **批次大小**: 5–15MB 每批次（而非按文档数），具体需基准测试
- **并发写入**: 使用多线程/多进程并发写入，通常 3–8 个并发线程
- **关闭副本**: 大批量写入前可临时将 `number_of_replicas` 设为 0，完成后恢复
- **关闭 refresh**: 写入期间设 `refresh_interval: -1`，完成后手动 refresh
- **错误处理**: 逐条检查 bulk 响应中的 `errors` 字段，部分失败需重试

### scroll vs search_after

| 方式 | 适用场景 | 特点 |
|------|----------|------|
| `from + size` | 浅分页（前 100 页） | 简单，但 `from + size ≤ 10000`（默认） |
| `scroll` | 全量导出/遍历 | 快照语义，占用资源，非实时 |
| `search_after` | 深分页/实时翻页 | 无状态，实时，需排序字段 |
| PIT + search_after | 一致性深分页 | 最佳方案，保持一致视图 |

```json
// scroll 方式（全量导出）
POST /products/_search?scroll=5m
{
  "size": 1000,
  "query": { "match_all": {} },
  "sort": ["_doc"]
}
// 后续请求
POST /_search/scroll
{
  "scroll": "5m",
  "scroll_id": "<scroll_id>"
}
// 用完清理
DELETE /_search/scroll
{ "scroll_id": "<scroll_id>" }

// search_after 方式（推荐的深分页）
// 第一页
GET /products/_search
{
  "size": 20,
  "query": { "match": { "category": "electronics" } },
  "sort": [
    { "created_at": "desc" },
    { "_id": "asc" }
  ]
}
// 下一页：使用上一页最后一条记录的 sort 值
GET /products/_search
{
  "size": 20,
  "query": { "match": { "category": "electronics" } },
  "sort": [
    { "created_at": "desc" },
    { "_id": "asc" }
  ],
  "search_after": ["2024-06-15T10:30:00.000Z", "product_999"]
}

// PIT (Point in Time) + search_after（一致性深分页）
POST /products/_pit?keep_alive=5m
// 返回 { "id": "<pit_id>" }

GET /_search
{
  "size": 20,
  "query": { "match": { "category": "electronics" } },
  "pit": {
    "id": "<pit_id>",
    "keep_alive": "5m"
  },
  "sort": [
    { "created_at": "desc" },
    { "_id": "asc" }
  ],
  "search_after": ["2024-06-15T10:30:00.000Z", "product_999"]
}
```

### 缓存与预热

ES 内置多级缓存：

| 缓存类型 | 说明 | 失效条件 |
|----------|------|----------|
| Node Query Cache | 缓存 filter context 的结果（bitset） | Segment 合并 / 索引更新 |
| Shard Request Cache | 缓存聚合结果和 `size=0` 的搜索 | refresh 时失效 |
| Fielddata Cache | `text` 字段排序/聚合时加载（避免使用） | 手动清理或内存压力 |
| OS Page Cache | 操作系统文件系统缓存 | 内存不足时 LRU 淘汰 |

```json
// 预热：在索引 settings 中配置
PUT /products/_settings
{
  "index.queries.cache.enabled": true
}

// 手动预热关键查询
GET /products/_search?request_cache=true
{
  "size": 0,
  "aggs": {
    "popular_categories": {
      "terms": { "field": "category.keyword", "size": 50 }
    }
  }
}

// 清理缓存（谨慎使用）
POST /products/_cache/clear
POST /_cache/clear?query=true&fielddata=true&request=true
```

### Mapping 优化与关闭不需要的特性

```json
PUT /optimized_logs
{
  "mappings": {
    "properties": {
      "message": {
        "type": "text",
        "norms": false,
        "index_options": "freqs"
      },
      "trace_id": {
        "type": "keyword",
        "doc_values": false,
        "norms": false
      },
      "raw_body": {
        "type": "keyword",
        "index": false,
        "doc_values": false
      },
      "metadata": {
        "type": "object",
        "enabled": false
      }
    },
    "_source": {
      "excludes": ["raw_body"]
    }
  }
}
```

优化说明：
- `norms: false` — 不需要评分时关闭（节省约 1 byte/doc/field）
- `doc_values: false` — 不需要排序/聚合的 keyword 字段关闭
- `index: false` — 只存储不索引的字段（如原始日志）
- `enabled: false` — 完全跳过解析和索引（仅存储在 `_source`）
- `index_options: freqs` — 不需要位置信息时降低索引精度
- `_source.excludes` — 排除大字段，减少 `_source` 存储和网络传输

---

## 集群运维

### 节点角色

| 角色 | 配置 | 职责 |
|------|------|------|
| `master` | `node.roles: [master]` | 集群状态管理、分片分配决策 |
| `data` | `node.roles: [data]` | 存储数据、执行搜索和聚合 |
| `data_hot` | `node.roles: [data_hot]` | 存储热数据（SSD，高 I/O） |
| `data_warm` | `node.roles: [data_warm]` | 存储温数据（HDD，中等 I/O） |
| `data_cold` | `node.roles: [data_cold]` | 存储冷数据（大容量 HDD） |
| `data_frozen` | `node.roles: [data_frozen]` | 可搜索快照（S3/共享存储） |
| `ingest` | `node.roles: [ingest]` | 数据预处理（Pipeline） |
| `ml` | `node.roles: [ml]` | 机器学习任务 |
| `coordinating` | `node.roles: []` | 请求路由、结果合并（无数据） |
| `transform` | `node.roles: [transform]` | 数据转换任务 |

生产集群最小部署：
- 3 个 dedicated master 节点（避免脑裂，保障选主稳定）
- N 个 data 节点（按存储量和性能需求扩展）
- 1–2 个 coordinating 节点（高并发搜索场景）

### 容量规划

```
存储容量计算:
  原始数据大小 × (1 + 副本数) × 1.1 (索引膨胀) × 1.15 (OS/临时空间)

内存规划:
  JVM Heap ≤ 50% 物理内存 且 ≤ 30GB（压缩指针边界）
  剩余 50%+ 留给 OS Page Cache（文件系统缓存至关重要）
  Heap 分配示例：64GB 物理内存 -> 30GB Heap + 34GB Page Cache

分片规划:
  总分片数 = 总数据量 / 目标分片大小(30GB)
  每节点分片数 ≤ 20 × JVM堆(GB)

示例:
  1TB 原始日志/天，保留 30 天，1 副本
  存储 = 1TB × 30 × 2 × 1.1 × 1.15 ≈ 76TB
  分片 = 76TB / 30GB ≈ 2534 个分片
  节点数 = 2534 / (20 × 30) ≈ 5 个 data 节点（每节点 30GB Heap）
```

### 集群健康

```json
// 集群健康状态
GET /_cluster/health
// green: 所有分片已分配
// yellow: 主分片已分配，存在未分配的副本
// red: 存在未分配的主分片（数据丢失风险）

// 查看未分配分片原因
GET /_cluster/allocation/explain
{
  "index": "products",
  "shard": 0,
  "primary": true
}

// 节点统计
GET /_nodes/stats/jvm,os,process,fs

// 分片分布
GET /_cat/shards?v&s=index,shard

// 热线程诊断
GET /_nodes/hot_threads

// Pending tasks
GET /_cluster/pending_tasks
```

### Hot-Warm-Cold 架构

```yaml
# elasticsearch.yml (Hot 节点)
node.roles: [data_hot, ingest]
node.attr.data: hot

# elasticsearch.yml (Warm 节点)
node.roles: [data_warm]
node.attr.data: warm

# elasticsearch.yml (Cold 节点)
node.roles: [data_cold]
node.attr.data: cold
```

配合 ILM 策略实现数据自动流转（参见上文 ILM 章节）。架构优势：
- **Hot 节点**: 高性能 SSD，处理最新数据的写入和高频查询
- **Warm 节点**: 普通 SSD/HDD，中频查询的历史数据
- **Cold 节点**: 大容量 HDD，低频查询的归档数据
- **Frozen 节点**: 可搜索快照（Searchable Snapshot），数据在对象存储（S3）上

### 跨集群搜索（Cross-Cluster Search）

```json
// 配置远程集群
PUT /_cluster/settings
{
  "persistent": {
    "cluster": {
      "remote": {
        "cluster_us": {
          "seeds": ["us-node1:9300", "us-node2:9300"],
          "transport.compress": true,
          "skip_unavailable": true
        },
        "cluster_eu": {
          "seeds": ["eu-node1:9300"],
          "skip_unavailable": true
        }
      }
    }
  }
}

// 跨集群搜索
GET /local_index,cluster_us:remote_index,cluster_eu:remote_index/_search
{
  "query": {
    "match": { "message": "critical error" }
  }
}
```

### 快照备份

```json
// 注册快照仓库（S3）
PUT /_snapshot/s3_backup
{
  "type": "s3",
  "settings": {
    "bucket": "es-snapshots",
    "region": "ap-east-1",
    "base_path": "production",
    "compress": true,
    "max_restore_bytes_per_sec": "200mb",
    "max_snapshot_bytes_per_sec": "200mb"
  }
}

// 创建快照
PUT /_snapshot/s3_backup/snapshot_20240615
{
  "indices": "products,orders-*",
  "ignore_unavailable": true,
  "include_global_state": false
}

// 查看快照状态
GET /_snapshot/s3_backup/snapshot_20240615/_status

// 恢复快照
POST /_snapshot/s3_backup/snapshot_20240615/_restore
{
  "indices": "products",
  "rename_pattern": "(.+)",
  "rename_replacement": "restored_$1"
}

// SLM（快照生命周期管理）
PUT /_slm/policy/nightly_backup
{
  "schedule": "0 0 2 * * ?",
  "name": "<nightly-snap-{now/d}>",
  "repository": "s3_backup",
  "config": {
    "indices": "*",
    "ignore_unavailable": true,
    "include_global_state": false
  },
  "retention": {
    "expire_after": "30d",
    "min_count": 7,
    "max_count": 60
  }
}
```

---

## 安全

### X-Pack Security（8.x 默认启用）

ES 8.x 首次启动时自动生成 CA 证书、节点证书和 elastic 超级用户密码。

```yaml
# elasticsearch.yml
xpack.security.enabled: true
xpack.security.enrollment.enabled: true

xpack.security.transport.ssl:
  enabled: true
  verification_mode: certificate
  keystore.path: certs/transport.p12
  truststore.path: certs/transport.p12

xpack.security.http.ssl:
  enabled: true
  keystore.path: certs/http.p12
```

### 角色与用户管理

```json
// 创建角色
POST /_security/role/product_reader
{
  "cluster": ["monitor"],
  "indices": [
    {
      "names": ["products*"],
      "privileges": ["read", "view_index_metadata"],
      "field_security": {
        "grant": ["name", "category", "price", "description"]
      },
      "query": {
        "term": { "status": "active" }
      }
    }
  ],
  "run_as": []
}

// 创建用户
POST /_security/user/product_app
{
  "password": "strong_password_here",
  "roles": ["product_reader"],
  "full_name": "Product App Service Account",
  "email": "product-app@example.com"
}
```

### API Key

```json
// 创建 API Key
POST /_security/api_key
{
  "name": "product-search-key",
  "expiration": "90d",
  "role_descriptors": {
    "product_search": {
      "cluster": [],
      "index": [
        {
          "names": ["products*"],
          "privileges": ["read"]
        }
      ]
    }
  },
  "metadata": {
    "application": "product-search-service",
    "team": "search-platform"
  }
}
// 返回 { "id": "...", "api_key": "...", "encoded": "base64..." }
// 使用: curl -H "Authorization: ApiKey <encoded>"

// 撤销 API Key
DELETE /_security/api_key
{
  "ids": ["<api_key_id>"]
}
```

### 字段级安全（Field Level Security）

在角色定义中通过 `field_security` 限制可见字段：

```json
{
  "indices": [
    {
      "names": ["customers*"],
      "privileges": ["read"],
      "field_security": {
        "grant": ["name", "email", "tier"],
        "except": ["ssn", "credit_card"]
      }
    }
  ]
}
```

### 文档级安全（Document Level Security）

在角色定义中通过 `query` 限制可见文档：

```json
{
  "indices": [
    {
      "names": ["orders*"],
      "privileges": ["read"],
      "query": {
        "bool": {
          "filter": [
            { "term": { "region": "asia-pacific" } },
            { "range": { "created_at": { "gte": "now-1y" } } }
          ]
        }
      }
    }
  ]
}
```

文档级安全和字段级安全可以组合使用，实现细粒度的数据访问控制。

---

## 与应用集成

### Python elasticsearch-py

```python
from elasticsearch import Elasticsearch, helpers
from datetime import datetime

# 连接（8.x 推荐方式）
es = Elasticsearch(
    "https://localhost:9200",
    api_key="base64_encoded_key",
    ca_certs="/path/to/http_ca.crt",
    request_timeout=30,
    max_retries=3,
    retry_on_timeout=True
)

# 连接验证
print(es.info())

# 索引单个文档
doc = {
    "name": "Elasticsearch Guide",
    "category": "book",
    "price": 59.99,
    "created_at": datetime.now()
}
resp = es.index(index="products", id="book_001", document=doc)
print(f"Indexed: {resp['result']}")  # created / updated

# 搜索
resp = es.search(
    index="products",
    query={
        "bool": {
            "must": [{"match": {"name": "Elasticsearch"}}],
            "filter": [{"range": {"price": {"lte": 100}}}]
        }
    },
    size=10,
    source=["name", "price"]
)
for hit in resp["hits"]["hits"]:
    print(f"{hit['_id']}: {hit['_source']}")

# 聚合
resp = es.search(
    index="products",
    size=0,
    aggs={
        "by_category": {
            "terms": {"field": "category.keyword", "size": 20},
            "aggs": {
                "avg_price": {"avg": {"field": "price"}}
            }
        }
    }
)
for bucket in resp["aggregations"]["by_category"]["buckets"]:
    print(f"{bucket['key']}: {bucket['doc_count']} items, avg={bucket['avg_price']['value']:.2f}")

# Bulk 批量写入
def generate_actions():
    for i in range(10000):
        yield {
            "_index": "products",
            "_id": f"product_{i}",
            "_source": {
                "name": f"Product {i}",
                "price": round(10 + i * 0.1, 2),
                "category": f"cat_{i % 10}"
            }
        }

success, errors = helpers.bulk(
    es,
    generate_actions(),
    chunk_size=500,
    max_retries=3,
    raise_on_error=False
)
print(f"Bulk indexed: {success} success, {len(errors)} errors")

# Async 版本
from elasticsearch import AsyncElasticsearch
import asyncio

async def async_search():
    es_async = AsyncElasticsearch(
        "https://localhost:9200",
        api_key="base64_encoded_key",
        ca_certs="/path/to/http_ca.crt"
    )
    resp = await es_async.search(
        index="products",
        query={"match_all": {}},
        size=5
    )
    await es_async.close()
    return resp

asyncio.run(async_search())
```

### REST API 直接调用

```bash
# 基础搜索
curl -X GET "https://localhost:9200/products/_search" \
  -H "Content-Type: application/json" \
  -H "Authorization: ApiKey <encoded_key>" \
  --cacert /path/to/http_ca.crt \
  -d '{
    "query": { "match": { "name": "手机" } },
    "size": 10
  }'

# 集群健康
curl -s "https://localhost:9200/_cluster/health?pretty" \
  -H "Authorization: ApiKey <encoded_key>" \
  --cacert /path/to/http_ca.crt

# Cat APIs（运维常用）
curl -s "https://localhost:9200/_cat/indices?v&s=store.size:desc" \
  -H "Authorization: ApiKey <encoded_key>" \
  --cacert /path/to/http_ca.crt
```

### Logstash

```ruby
# logstash.conf
input {
  beats {
    port => 5044
  }
  kafka {
    bootstrap_servers => "kafka1:9092,kafka2:9092"
    topics => ["app-logs"]
    group_id => "logstash-consumers"
    codec => json
  }
}

filter {
  if [type] == "nginx" {
    grok {
      match => { "message" => "%{COMBINEDAPACHELOG}" }
    }
    date {
      match => ["timestamp", "dd/MMM/yyyy:HH:mm:ss Z"]
    }
    geoip {
      source => "clientip"
      target => "geoip"
    }
  }

  mutate {
    remove_field => ["agent", "ecs", "host"]
    rename => { "clientip" => "client_ip" }
  }
}

output {
  elasticsearch {
    hosts => ["https://es-node1:9200", "https://es-node2:9200"]
    index => "logs-%{[type]}-%{+YYYY.MM.dd}"
    api_key => "id:api_key"
    ssl_certificate_authorities => ["/path/to/http_ca.crt"]
    manage_template => true
    template_name => "logs"
    ilm_enabled => true
    ilm_rollover_alias => "logs-write"
    ilm_policy => "logs_policy"
  }
}
```

### Filebeat

```yaml
# filebeat.yml
filebeat.inputs:
  - type: log
    enabled: true
    paths:
      - /var/log/app/*.log
    multiline:
      pattern: '^\d{4}-\d{2}-\d{2}'
      negate: true
      match: after
    fields:
      app: my-service
      env: production

  - type: container
    paths:
      - /var/lib/docker/containers/*/*.log
    processors:
      - add_kubernetes_metadata:
          host: ${NODE_NAME}
          matchers:
            - logs_path:
                logs_path: "/var/lib/docker/containers/"

processors:
  - drop_fields:
      fields: ["agent.ephemeral_id", "agent.hostname"]
  - add_host_metadata: ~

output.elasticsearch:
  hosts: ["https://es-node1:9200"]
  api_key: "id:api_key"
  ssl.certificate_authorities: ["/path/to/http_ca.crt"]
  index: "filebeat-%{+yyyy.MM.dd}"

setup.ilm.enabled: true
setup.ilm.rollover_alias: "filebeat"
setup.ilm.policy_name: "filebeat-policy"

monitoring.enabled: true
monitoring.elasticsearch:
  hosts: ["https://es-monitoring:9200"]
```

### Kibana

Kibana 是 ES 的官方可视化平台，核心功能：

- **Discover**: 日志搜索与浏览，支持 KQL/Lucene 查询语法
- **Dashboard**: 可视化仪表盘，支持 30+ 图表类型
- **Lens**: 拖拽式可视化创建工具
- **Dev Tools**: 交互式 REST API 控制台（开发调试必备）
- **Index Management**: 索引、模板、ILM 策略可视化管理
- **Security**: 用户、角色、API Key 管理界面
- **Alerting**: 基于条件的告警规则（Watcher / Rules）
- **Canvas**: 像素级报告和展示面板

```yaml
# kibana.yml 关键配置
server.host: "0.0.0.0"
server.port: 5601
server.publicBaseUrl: "https://kibana.example.com"

elasticsearch.hosts: ["https://es-node1:9200"]
elasticsearch.serviceAccountToken: "<token>"
elasticsearch.ssl.certificateAuthorities: ["/path/to/http_ca.crt"]

xpack.encryptedSavedObjects.encryptionKey: "min-32-char-encryption-key-here!"
xpack.security.encryptionKey: "min-32-char-encryption-key-here!"
xpack.reporting.encryptionKey: "min-32-char-encryption-key-here!"
```

---

## 常见陷阱

### 1. Mapping Explosion（映射爆炸）

**问题**: 动态映射开启时，大量不同字段名被自动创建，导致集群元数据膨胀、内存溢出。

**典型场景**: 将用户自定义属性或日志的任意 JSON 键直接索引。

```json
// 反模式：每个用户的自定义属性都成为独立字段
{"user_attr_color": "red", "user_attr_size": "L", "user_attr_custom_12345": "value"}

// 正确做法：使用 nested 或 flattened 类型
{
  "user_attributes": [
    {"key": "color", "value": "red"},
    {"key": "size", "value": "L"}
  ]
}
```

**防御措施**:
- 设置 `dynamic: strict` 或 `dynamic: false`
- 配置 `index.mapping.total_fields.limit`（默认 1000）
- 使用 `flattened` 类型存储任意 JSON
- 对动态模板使用 `path_match` / `unmatch` 精确控制

### 2. 深分页（Deep Pagination）

**问题**: `from + size` 超过 `max_result_window`（默认 10000）时报错。即使增大限制，深分页也会消耗大量内存。

**原因**: ES 需要从每个分片取 `from + size` 条记录，协调节点需要在内存中排序 `shards × (from + size)` 条记录。

```
分页到第 1000 页（每页 20 条）: from=19980, size=20
5 分片集群实际排序: 5 × 20000 = 100,000 条记录
```

**解决方案**:
- 浅分页（< 100 页）: `from + size` 即可
- 深分页: 使用 `search_after` + PIT（参见性能优化章节）
- 全量导出: 使用 `scroll`（但注意资源消耗）
- 产品设计: 引导用户通过筛选缩小范围，而非无限翻页

### 3. 高基数聚合（High Cardinality Aggregation）

**问题**: 对高基数字段（如 `user_id`、`ip`、`url`）做 `terms` 聚合，内存和 CPU 消耗极大。

```json
// 危险操作：对百万级 user_id 做 terms 聚合
{
  "aggs": {
    "all_users": {
      "terms": { "field": "user_id", "size": 1000000 }
    }
  }
}
```

**解决方案**:
- 使用 `cardinality` 聚合做近似去重计数（HyperLogLog++，误差 < 1%）
- `terms` 聚合的 `size` 保持合理范围（通常 < 1000）
- 使用 `composite` 聚合分页获取所有桶
- 降低基数：使用 `script` 将 URL 归一化，或按前缀分组
- 预聚合：使用 Transform 定期将细粒度数据聚合为摘要索引

### 4. 分片过多（Over-Sharding）

**问题**: 过多的小分片导致集群元数据膨胀、搜索延迟升高、master 节点压力增大。

**典型场景**: 按天创建日志索引，每个索引默认 5 分片，但日志量很小。

```
365天 × 5分片 × 3索引类型 = 5475 个分片（绝大多数 < 100MB）
```

**解决方案**:
- 小数据量索引使用 1 个分片
- 使用 ILM + Rollover 基于大小/时间自动滚动
- 定期 Shrink 旧索引减少分片数
- 使用 Data Stream 替代手动管理的时间序列索引

### 5. GC 压力（Garbage Collection Pressure）

**问题**: JVM 堆内存不足或使用不当导致频繁 GC（尤其是 Old GC / Stop-the-World），集群响应变慢甚至节点脱离。

**常见原因**:
- JVM Heap 设置过大（> 30GB，失去压缩指针优势）
- 大量 fielddata 加载（对 `text` 字段排序/聚合）
- 高基数 terms 聚合
- 巨大的 bulk 请求
- 过多的 pending tasks 和 in-flight requests

**解决方案**:
- JVM Heap ≤ 50% 物理内存且 ≤ 30GB
- 避免对 `text` 字段聚合/排序（使用 `keyword` 子字段 + `doc_values`）
- 监控 `jvm.gc.collectors.old.collection_time_in_millis`
- 设置 Circuit Breaker：

```json
PUT /_cluster/settings
{
  "persistent": {
    "indices.breaker.total.limit": "70%",
    "indices.breaker.fielddata.limit": "40%",
    "indices.breaker.request.limit": "60%",
    "network.breaker.inflight_requests.limit": "100%"
  }
}
```

- 使用 G1GC（ES 8.x 默认）并确保充足的堆外内存

### 6. 其他常见问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| Yellow 状态 | 副本无法分配（节点不足） | 增加节点或减少副本数 |
| 写入被拒绝 | 线程池队列满 | 控制并发写入数，增大队列（谨慎） |
| 搜索超时 | 查询过重或数据量大 | 优化查询、增加分片、使用 `terminate_after` |
| 磁盘水位线 | 磁盘使用超过 85%/90%/95% | 清理旧索引、扩容磁盘 |
| Unassigned 分片 | 节点故障/磁盘满/分配规则冲突 | `_cluster/allocation/explain` 诊断 |
| 版本升级兼容性 | 跨大版本升级 | 逐版本滚动升级（7→8），不可跳版本 |

---

## Agent Checklist

### Mapping 与索引设计
- [ ] 生产环境已设置 `dynamic: strict` 或 `dynamic: false`
- [ ] 字段类型明确定义，不依赖动态映射推断
- [ ] `text` 字段配置了合适的 `analyzer` 和 `search_analyzer`
- [ ] 需要聚合/排序的字段使用 `keyword` 类型或 `keyword` 子字段
- [ ] `_source` 排除了不需要返回的大字段
- [ ] 不需要评分的字段已关闭 `norms`
- [ ] 不需要排序/聚合的字段已关闭 `doc_values`
- [ ] 嵌套对象的关联性需求已评估（`object` vs `nested`）
- [ ] `index.mapping.total_fields.limit` 已根据实际需求调整

### 查询优化
- [ ] 精确过滤条件放在 `filter` context（可缓存，无评分开销）
- [ ] 避免对 `text` 字段使用 `term` 查询
- [ ] `multi_match` 的 type 根据场景选择（`best_fields` / `cross_fields`）
- [ ] 深分页使用 `search_after` + PIT 而非 `from + size`
- [ ] 高基数字段的 `terms` 聚合 size 已控制在合理范围
- [ ] 评分公式使用 `function_score` 而非 `script_score`（性能更优）
- [ ] 频繁执行的聚合查询使用 `request_cache=true`

### 写入优化
- [ ] Bulk 批次大小为 5–15MB（而非固定文档数）
- [ ] 大批量写入前临时关闭 `refresh_interval` 和减少 `number_of_replicas`
- [ ] Bulk 响应中的 errors 字段有检查和重试逻辑
- [ ] 使用 `_routing` 将相关文档路由到同一分片（多租户场景）
- [ ] 文档 `_id` 使用业务键或自动生成（避免随机 UUID 影响写入性能）

### 集群运维
- [ ] Master 节点为 dedicated 角色（不承担 data/ingest）
- [ ] JVM Heap ≤ 50% 物理内存且 ≤ 30GB
- [ ] 已配置 Hot-Warm-Cold 架构（数据量 > 1TB 场景）
- [ ] ILM 策略已配置并正常运行
- [ ] 快照备份已配置（SLM 或定时任务）
- [ ] 集群健康状态监控告警已就位（`_cluster/health`）
- [ ] 慢查询日志已开启（`index.search.slowlog.threshold`）
- [ ] Circuit Breaker 参数已合理配置
- [ ] 磁盘水位线告警已配置（85% / 90% / 95%）

### 安全
- [ ] X-Pack Security 已启用（8.x 默认启用）
- [ ] TLS 加密已配置（传输层 + HTTP 层）
- [ ] 遵循最小权限原则，应用使用独立角色而非 elastic 超级用户
- [ ] API Key 有合理的过期时间和权限范围
- [ ] 需要时配置了字段级安全和/或文档级安全
- [ ] `elastic` 超级用户密码已修改且安全存储

### 中文搜索
- [ ] IK 分词插件已安装且版本与 ES 匹配
- [ ] 索引时使用 `ik_max_word`，搜索时使用 `ik_smart`
- [ ] 自定义词典已配置（行业术语、品牌名、新词等）
- [ ] 同义词文件已配置在 search_analyzer 中
- [ ] 停用词列表已根据业务场景定制

### 监控与可观测性
- [ ] 关键指标已接入监控系统（Prometheus / Datadog / 自带 Monitoring）
- [ ] JVM GC 时间和频率有告警阈值
- [ ] 搜索延迟 P99 有 SLO 和告警
- [ ] 索引速率下降有告警
- [ ] 未分配分片有即时告警
- [ ] Kibana Dev Tools 可用于日常运维调试

---

**知识ID**: `elasticsearch-complete`
**领域**: data
**类型**: standards
**难度**: intermediate-advanced
**质量分**: 95
**维护者**: data-team@superdev.com
**最后更新**: 2026-03-28
