---
id: data-engineering-glossary
title: Data Engineering Glossary
domain: data-engineering
category: 06-glossary
difficulty: intermediate
tags: [agent, checklist, data, data-engineering, engineering, glossary, 术语对比速查表, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Data Engineering Glossary

## 概述

数据工程术语表涵盖数据采集、传输、存储、处理和服务全链路的核心概念与工具。本术语表按字母顺序排列，每个词条包含定义、使用场景和关联术语，适用于数据工程师、后端工程师和技术决策者。

---

## A

### Airflow (Apache Airflow)

**定义**：Apache 基金会的开源工作流编排平台，使用 Python 定义 DAG（有向无环图）来描述数据管道的依赖关系和执行顺序。

**使用场景**：ETL/ELT 任务调度、定时数据同步、多步骤数据处理流水线编排。

**核心概念**：DAG（工作流定义）、Operator（任务执行单元）、Scheduler（调度器）、Executor（执行引擎）、XCom（任务间通信）。

**关联术语**：DAG、ETL、Prefect、Dagster

### Avro

**定义**：Apache 开源的行式数据序列化格式，Schema 与数据一起存储，支持 Schema 演进（Schema Evolution），广泛用于 Kafka 消息序列化。

**使用场景**：Kafka 消息编解码、数据交换格式、Schema Registry 配合使用。

**对比**：Avro（行存、适合写入和消息传递） vs Parquet（列存、适合分析查询）。

**关联术语**：Schema Registry、Kafka、Parquet、Protobuf

---

## B

### Batch Processing

**定义**：批处理，指对一段时间内累积的数据进行集中处理的计算模式。与流处理（Stream Processing）相对。

**使用场景**：每日报表生成、历史数据回填、大规模数据转换。

**典型工具**：Spark、Hive、MapReduce。

**关联术语**：Stream Processing、Lambda Architecture、Spark

### Bronze/Silver/Gold (Medallion Architecture)

**定义**：数据湖中的分层架构模式（也称奖牌架构）。Bronze 层存原始数据，Silver 层存清洗后的数据，Gold 层存业务聚合数据。

**使用场景**：数据湖建设、Databricks Lakehouse 架构。

**关联术语**：Data Lake、Data Lakehouse、Delta Lake

---

## C

### CDC (Change Data Capture)

**定义**：变更数据捕获，一种识别和捕获数据库中数据变更（INSERT/UPDATE/DELETE）的技术，将变更实时或近实时地传播到下游系统。

**实现方式**：
- **基于日志**：读取数据库 binlog/WAL（推荐，对源库影响小）
- **基于查询**：定期轮询变更（简单但延迟高、对源库有压力）
- **基于触发器**：数据库触发器捕获变更（侵入性强，不推荐）

**典型工具**：Debezium、Maxwell、Canal（阿里开源）、AWS DMS。

**关联术语**：Debezium、Binlog、WAL、Stream Processing

### Column Store

**定义**：列式存储，按列而非按行组织数据的存储格式。同一列的数据物理上连续存储，适合分析查询（通常只读取少数列）和高压缩比。

**典型系统**：ClickHouse、Apache Druid、Redshift、BigQuery。

**关联术语**：Parquet、ORC、OLAP

---

## D

### DAG (Directed Acyclic Graph)

**定义**：有向无环图，在数据工程中用于描述任务之间的依赖关系和执行顺序。任务 A 完成后才能执行任务 B 即 A -> B 的有向边。

**使用场景**：Airflow 工作流定义、Spark 执行计划、dbt 模型依赖。

**关联术语**：Airflow、dbt、Spark

### Data Catalog

**定义**：数据目录，集中管理组织内所有数据资产的元数据的系统。提供数据发现、数据血缘、数据质量和访问控制功能。

**典型工具**：Apache Atlas、DataHub（LinkedIn 开源）、Amundsen（Lyft 开源）、Google Data Catalog。

**关联术语**：Metadata、Data Lineage、Data Governance

### Data Lake

**定义**：数据湖，以原始格式存储海量结构化、半结构化和非结构化数据的集中式存储系统。强调"先存储，后处理"（Schema-on-Read）。

**典型实现**：AWS S3 + Athena、Azure Data Lake Storage、HDFS。

**优点**：存储成本低、支持多种数据格式、灵活性高。

**风险**：缺乏治理可能退化为"数据沼泽"（Data Swamp）。

**关联术语**：Data Warehouse、Data Lakehouse、Bronze/Silver/Gold

### Data Lakehouse

**定义**：数据湖仓一体，结合数据湖的灵活性和数据仓库的管理能力的新型架构。在数据湖之上增加 ACID 事务、Schema 管理和性能优化。

**典型实现**：Databricks（Delta Lake）、Apache Iceberg、Apache Hudi。

**关联术语**：Data Lake、Data Warehouse、Delta Lake、Iceberg

### Data Lineage

**定义**：数据血缘，追踪数据从源头到最终消费者的完整流转路径。包括数据经过了哪些转换、存储在哪些系统中、被哪些报表使用。

**使用场景**：影响分析（修改上游会影响哪些下游）、合规审计、数据质量根因分析。

**关联术语**：Data Catalog、Metadata、dbt

### Data Mesh

**定义**：一种去中心化的数据架构范式，将数据所有权分散到各业务领域团队，每个团队负责自己的数据产品（Data Product），中台提供自助式基础设施。

**四大原则**：领域所有权、数据即产品、自助式平台、联邦治理。

**关联术语**：Data Product、Domain-Driven Design

### Data Warehouse

**定义**：数据仓库，面向分析场景的集中式数据存储系统。数据在写入时经过清洗和建模（Schema-on-Write），结构化程度高。

**典型系统**：Snowflake、BigQuery、Redshift、ClickHouse。

**对比**：Data Warehouse（结构化、分析优化）vs Data Lake（原始格式、灵活）。

**关联术语**：Data Lake、OLAP、Star Schema、dbt

### dbt (Data Build Tool)

**定义**：数据转换工具，允许数据分析师和工程师使用 SQL 定义数据模型并管理转换逻辑。dbt 处理"T"（Transform），不处理"E"和"L"。

**核心能力**：SQL 模型化、依赖管理（自动 DAG）、测试框架、文档生成、增量处理。

**版本**：dbt Core（开源 CLI）、dbt Cloud（托管平台）。

**关联术语**：ELT、Data Warehouse、DAG

### Debezium

**定义**：Red Hat 开源的分布式 CDC 平台，基于 Kafka Connect 实现。支持 MySQL、PostgreSQL、MongoDB、Oracle 等数据库的变更捕获。

**工作原理**：读取数据库的事务日志（binlog/WAL），将变更事件发布到 Kafka Topic。

**关联术语**：CDC、Kafka Connect、Binlog

### Delta Lake

**定义**：Databricks 开源的存储层，在数据湖（如 S3/HDFS）之上提供 ACID 事务、Schema 演进、时间旅行（Time Travel）和统一的批流处理。

**关联术语**：Data Lakehouse、Iceberg、Hudi、Parquet

---

## E

### ELT (Extract, Load, Transform)

**定义**：先抽取和加载原始数据到目标系统（通常是数据仓库或数据湖），然后在目标系统中进行转换。与 ETL 的区别在于转换发生的位置。

**优势**：利用目标系统的计算能力进行转换，适合云数据仓库场景。

**典型工具链**：Fivetran/Airbyte（EL） + dbt（T） + Snowflake/BigQuery（目标系统）。

**关联术语**：ETL、dbt、Data Warehouse

### ETL (Extract, Transform, Load)

**定义**：数据集成的经典三步流程。从源系统抽取（Extract）数据，在中间层进行清洗转换（Transform），然后加载（Load）到目标系统。

**使用场景**：传统数据仓库建设、跨系统数据同步。

**典型工具**：Informatica、Talend、Apache NiFi、AWS Glue。

**对比**：ETL（转换在中间层）vs ELT（转换在目标系统）。

**关联术语**：ELT、Data Warehouse、Airflow

---

## F

### Flink (Apache Flink)

**定义**：分布式流处理框架，支持有状态的流计算和批处理。以"流优先"设计，将批处理视为有界流的特例。

**核心能力**：精确一次（Exactly-Once）语义、事件时间处理、窗口计算、状态管理、Savepoint/Checkpoint。

**使用场景**：实时数据分析、CEP（复杂事件处理）、实时 ETL、实时风控。

**对比**：Flink（真正的流处理，低延迟）vs Spark Streaming（微批处理，吞吐量高）。

**关联术语**：Stream Processing、Kafka、Exactly-Once、Spark

---

## I

### Iceberg (Apache Iceberg)

**定义**：Netflix 开源的表格式（Table Format），为数据湖提供类似数据仓库的管理能力。支持 ACID 事务、Schema 演进、分区演进和时间旅行。

**对比**：Iceberg vs Delta Lake vs Hudi - 三者功能相似，Iceberg 的引擎无关性最好（支持 Spark/Flink/Trino/Presto）。

**关联术语**：Data Lakehouse、Delta Lake、Hudi、Parquet

### Idempotency

**定义**：幂等性，指同一操作执行多次与执行一次的效果相同。在数据管道中至关重要，因为重试和重放是常见场景。

**实现方式**：使用唯一标识去重、UPSERT 操作、幂等写入（如 Kafka 幂等生产者）。

**关联术语**：Exactly-Once、At-Least-Once

---

## K

### Kafka (Apache Kafka)

**定义**：分布式事件流平台，用于构建实时数据管道和流式应用。消息以 Topic 为单位组织，分区（Partition）实现并行，副本（Replica）保证高可用。

**核心概念**：
- **Producer**：消息生产者
- **Consumer**：消息消费者
- **Consumer Group**：消费者组，实现消息的并行消费
- **Topic**：消息主题，逻辑分类
- **Partition**：分区，物理并行单元
- **Offset**：消息偏移量，消费位置标记

**使用场景**：系统间异步通信、事件驱动架构、日志聚合、CDC 传输、流处理数据源。

**关联术语**：Schema Registry、Kafka Connect、Flink、CDC

### Kafka Connect

**定义**：Kafka 的数据集成框架，提供标准化的 Connector 接口，用于在 Kafka 和外部系统之间批量移动数据。

**类型**：Source Connector（外部 -> Kafka）、Sink Connector（Kafka -> 外部）。

**典型 Connector**：JDBC Source/Sink、Debezium（CDC Source）、Elasticsearch Sink、S3 Sink。

**关联术语**：Kafka、Debezium、CDC

---

## O

### OLAP (Online Analytical Processing)

**定义**：联机分析处理，面向复杂查询和数据分析的数据库处理方式。特点是读多写少、查询涉及大量数据聚合。

**典型系统**：ClickHouse、Apache Druid、StarRocks、BigQuery。

**对比**：OLAP（分析型，列存，复杂聚合）vs OLTP（事务型，行存，高并发读写）。

**关联术语**：OLTP、Column Store、Data Warehouse

### OLTP (Online Transaction Processing)

**定义**：联机事务处理，面向日常业务操作的数据库处理方式。特点是高并发、低延迟、单行读写为主。

**典型系统**：MySQL、PostgreSQL、Oracle、MongoDB。

**关联术语**：OLAP、ACID

### Orchestration

**定义**：编排，在数据工程中指协调和管理多个数据任务的执行顺序、依赖关系和失败处理。

**典型工具**：Airflow、Prefect、Dagster、Argo Workflows。

**关联术语**：Airflow、DAG、Pipeline

---

## P

### Parquet

**定义**：Apache 开源的列式存储文件格式，广泛用于大数据分析场景。支持高效的列裁剪（Column Pruning）和行组过滤（Predicate Pushdown），压缩比高。

**使用场景**：数据湖存储、Spark/Hive 分析、数据归档。

**对比**：Parquet（列存、分析优化）vs Avro（行存、序列化优化）vs ORC（列存、Hive 生态）。

**关联术语**：Avro、ORC、Column Store、Data Lake

### Pipeline

**定义**：数据管道，指数据从源头经过一系列处理步骤到达目标的完整流程。可以是批处理管道或实时流管道。

**关联术语**：ETL、ELT、DAG、Orchestration

---

## S

### Schema Registry

**定义**：Schema 注册中心，集中管理和验证数据 Schema（模式定义）的服务。确保生产者和消费者使用兼容的 Schema，防止数据格式不一致导致的下游故障。

**典型实现**：Confluent Schema Registry（支持 Avro/Protobuf/JSON Schema）。

**核心功能**：
- Schema 版本管理
- 兼容性检查（Backward/Forward/Full）
- Schema 演进支持

**关联术语**：Avro、Kafka、Schema Evolution

### SCD (Slowly Changing Dimension)

**定义**：缓慢变化维度，数据仓库中处理维度表数据随时间变化的策略。

**常用类型**：
- **Type 1**：直接覆盖旧值（不保留历史）
- **Type 2**：新增行保留历史（增加有效日期列）
- **Type 3**：增加列记录新旧值（仅保留一次变更）

**关联术语**：Data Warehouse、Star Schema、Dimension Table

### Spark (Apache Spark)

**定义**：统一的大规模数据处理引擎，支持批处理、流处理（Structured Streaming）、机器学习（MLlib）和图计算（GraphX）。

**核心概念**：RDD、DataFrame、SparkSQL、Catalyst Optimizer。

**使用场景**：大规模 ETL、数据分析、机器学习特征工程。

**关联术语**：Flink、Hadoop、Parquet、Delta Lake

### Star Schema

**定义**：星型模型，数据仓库中最常见的维度建模方式。中心是事实表（Fact Table），周围是维度表（Dimension Table），形如星形。

**优点**：查询简单直观、聚合性能好。

**对比**：Star Schema（简单、冗余）vs Snowflake Schema（规范化、复杂）。

**关联术语**：Data Warehouse、Fact Table、Dimension Table、SCD

### Stream Processing

**定义**：流处理，对持续到达的数据逐条或小批量进行实时处理的计算模式。与批处理相对。

**典型工具**：Flink、Kafka Streams、Spark Structured Streaming。

**核心挑战**：事件时间 vs 处理时间、乱序处理、状态管理、精确一次语义。

**关联术语**：Batch Processing、Flink、Kafka、Event Time

---

## W

### WAL (Write-Ahead Log)

**定义**：预写日志，数据库在修改数据前先将变更写入日志的机制。用于崩溃恢复和数据复制。PostgreSQL 的 WAL 是 CDC 的重要数据源。

**关联术语**：CDC、Binlog、Debezium

### Window Function

**定义**：窗口函数，在流处理中对一段时间范围或一定数量的事件进行聚合计算。

**常见类型**：
- **Tumbling Window**：滚动窗口，固定大小不重叠
- **Sliding Window**：滑动窗口，固定大小可重叠
- **Session Window**：会话窗口，基于活动间隔动态划分

**关联术语**：Flink、Stream Processing、Event Time

---

## 术语对比速查表

| 维度 | 选项 A | 选项 B | 选择依据 |
|------|--------|--------|---------|
| 处理模式 | Batch | Stream | 延迟要求（分钟级 vs 秒级） |
| 集成模式 | ETL | ELT | 目标系统计算能力 |
| 存储格式 | Parquet | Avro | 分析查询 vs 消息传递 |
| 流引擎 | Flink | Spark Streaming | 真流 vs 微批 |
| 表格式 | Iceberg | Delta Lake | 引擎无关性 vs Databricks 生态 |
| 存储架构 | Data Lake | Data Warehouse | 灵活性 vs 管理能力 |
| 编排工具 | Airflow | Prefect | 成熟度 vs 开发体验 |
| CDC 工具 | Debezium | Canal | 通用 vs MySQL 专用 |

---

## Agent Checklist

以下为 AI Agent 在数据工程项目中使用本术语表的要点：

- [ ] 数据管道设计中正确区分 ETL 和 ELT 模式
- [ ] 根据延迟要求选择批处理（Spark）或流处理（Flink/Kafka Streams）
- [ ] CDC 场景优先选择基于日志的方案（Debezium）而非轮询
- [ ] 分析场景使用列式存储格式（Parquet）而非行式格式
- [ ] Kafka 消息使用 Schema Registry 管理 Schema 兼容性
- [ ] 数据湖遵循 Medallion Architecture（Bronze/Silver/Gold）分层
- [ ] 数据仓库使用维度建模（Star Schema）并处理 SCD
- [ ] 数据管道确保幂等性，支持安全重试和重放
- [ ] 使用 dbt 管理 SQL 转换逻辑、测试和文档
- [ ] 建立数据目录和血缘追踪，支持影响分析和合规审计
