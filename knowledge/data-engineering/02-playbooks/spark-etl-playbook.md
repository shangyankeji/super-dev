---
id: spark-etl-playbook
title: Spark ETL开发完整指南
domain: data-engineering
category: 02-playbooks
difficulty: intermediate
tags: [data-engineering, etl, etl流程, playbook, spark, 参考资料, 学习路径, 性能优化]
quality_score: 70
last_updated: 2026-06-15
---
# Spark ETL开发完整指南

## 概述
Apache Spark是一个快速、通用的大数据处理引擎,支持批处理、流处理、SQL查询和机器学习。本指南覆盖Spark ETL(Extract-Transform-Load)的最佳实践。

## 核心概念

### 1. RDD vs DataFrame vs Dataset

**RDD (Resilient Distributed Dataset)**:
- 底层API
- 无模式
- 函数式编程

```python
from pyspark import SparkContext

sc = SparkContext.getOrCreate()

# 创建RDD
rdd = sc.parallelize([1, 2, 3, 4, 5])

# 转换
squared = rdd.map(lambda x: x ** 2)
filtered = rdd.filter(lambda x: x > 10)
```

**DataFrame**:
- 高层API
- 有模式
- SQL风格

```python
from pyspark.sql import SparkSession

spark = SparkSession.builder.appName("ETL").getOrCreate()

# 创建DataFrame
df = spark.createDataFrame(
    [(1, "Alice", 30), (2, "Bob", 25)],
    ["id", "name", "age"]
)

# SQL查询
df.createOrReplaceTempView("users")
result = spark.sql("SELECT * FROM users WHERE age > 25")
```

**Dataset**:
- 类型安全的DataFrame
- Scala/Java优先

### 2. 惰性求值
Spark延迟执行,直到遇到action操作。

```python
# Transformation (惰性)
mapped = df.select("name", "age").filter(df.age > 20)

# Action (触发执行)
result = mapped.collect()  # 触发计算
```

### 3. 分区和并行

```python
# 查看分区数
print(df.rdd.getNumPartitions)

# 重分区
repartitioned = df.repartition(10)

# Coalesce (减少分区)
coalesced = df.coalesce(5)
```

## ETL流程

### 1. Extract (提取)

#### 从文件系统读取

```python
# CSV
df = spark.read.csv("hdfs://path/to/file.csv", header=True, inferSchema=True)

# JSON
df = spark.read.json("hdfs://path/to/file.json")

# Parquet (推荐)
df = spark.read.parquet("hdfs://path/to/file.parquet")

# 分区数据
df = spark.read.parquet("hdfs://path/to/data") \
    .filter("year=2024") \
    .filter("month=03")
```

#### 从数据库读取

```python
# JDBC
df = spark.read \
    .format("jdbc") \
    .option("url", "jdbc:postgresql://localhost:5432/db") \
    .option("dbtable", "users") \
    .option("user", "user") \
    .option("password", "password") \
    .load()

# 带推下谓词
df = spark.read \
    .format("jdbc") \
    .option("url", "jdbc:postgresql://localhost:5432/db") \
    .option("dbtable", "(SELECT * FROM users WHERE active=true) AS users") \
    .option("user", "user") \
    .option("password", "password") \
    .load()
```

#### 从Kafka读取

```python
# 流式读取
df = spark \
    .readStream \
    .format("kafka") \
    .option("kafka.bootstrap.servers", "localhost:9092") \
    .option("subscribe", "topic1,topic2") \
    .load()

# 批量读取
df = spark \
    .read \
    .format("kafka") \
    .option("kafka.bootstrap.servers", "localhost:9092") \
    .option("subscribe", "topic1") \
    .option("startingOffsets", "earliest") \
    .option("endingOffsets", "latest") \
    .load()
```

### 2. Transform (转换)

#### 基础转换

```python
# 选择列
selected = df.select("name", "age")

# 过滤
filtered = df.filter(df.age > 25)

# 去重
distinct = df.dropDuplicates(["user_id"])

# 排序
sorted_df = df.orderBy(df.age.desc())

# 重命名
renamed = df.withColumnRenamed("old_name", "new_name")

# 添加列
with_new = df.withColumn("age_plus_10", df.age + 10)
```

#### 聚合

```python
# 分组聚合
agg_df = df.groupBy("department").agg(
    {"salary": "avg", "age": "max"}
)

# 多聚合
from pyspark.sql.functions import avg, max, count

agg_df = df.groupBy("department").agg(
    avg("salary").alias("avg_salary"),
    max("age").alias("max_age"),
    count("*").alias("count")
)

# Window函数
from pyspark.sql.window import Window
from pyspark.sql.functions import row_number, rank

window_spec = Window.partitionBy("department").orderBy(df.salary.desc())

ranked = df.withColumn("rank", rank().over(window_spec))
```

#### Join

```python
# Inner Join
joined = df1.join(df2, df1.id == df2.user_id, "inner")

# Left Join
left_joined = df1.join(df2, df1.id == df2.user_id, "left")

# Broadcast Join (小表)
from pyspark.sql.functions import broadcast

joined = df1.join(broadcast(small_df), df1.id == small_df.id)

# 多列Join
joined = df1.join(df2, ["id", "date"])
```

#### UDF (用户定义函数)

```python
from pyspark.sql.functions import udf
from pyspark.sql.types import StringType

@udf(returnType=StringType())
def categorize_age(age):
    if age < 18:
        return "minor"
    elif age < 65:
        return "adult"
    else:
        return "senior"

df = df.withColumn("age_category", categorize_age(df.age))

# Pandas UDF (更快)
import pandas as pd
from pyspark.sql.functions import pandas_udf

@pandas_udf("string")
def categorize_age_udf(age_series: pd.Series) -> pd.Series:
    return age_series.apply(lambda age: "minor" if age < 18 else "adult")
```

### 3. Load (加载)

#### 写入文件系统

```python
# Parquet (推荐)
df.write.parquet("hdfs://path/to/output", mode="overwrite")

# 分区写入
df.write.partitionBy("year", "month").parquet("hdfs://path/to/output")

# CSV
df.write.csv("hdfs://path/to/output.csv", header=True, mode="overwrite")

# JSON
df.write.json("hdfs://path/to/output.json", mode="overwrite")
```

#### 写入数据库

```python
df.write \
    .format("jdbc") \
    .option("url", "jdbc:postgresql://localhost:5432/db") \
    .option("dbtable", "output_table") \
    .option("user", "user") \
    .option("password", "password") \
    .mode("append") \
    .save()
```

#### 写入Kafka

```python
# 流式写入
query = df \
    .writeStream \
    .format("kafka") \
    .option("kafka.bootstrap.servers", "localhost:9092") \
    .option("topic", "output_topic") \
    .option("checkpointLocation", "/path/to/checkpoint") \
    .start()

# 批量写入
df \
    .selectExpr("CAST(key AS STRING)", "CAST(value AS STRING)") \
    .write \
    .format("kafka") \
    .option("kafka.bootstrap.servers", "localhost:9092") \
    .option("topic", "output_topic") \
    .save()
```

## 性能优化

### 1. 缓存

```python
# 缓存DataFrame
df.cache()

# 持久化到磁盘
from pyspark import StorageLevel

df.persist(StorageLevel.MEMORY_AND_DISK)

# 解除缓存
df.unpersist()
```

### 2. 分区优化

```python
# 查看分区数
print(df.rdd.getNumPartitions())

# 增加分区
repartitioned = df.repartition(100, "user_id")

# 减少分区
coalesced = df.coalesce(10)

# 自定义分区
from pyspark.sql.functions import spark_partition_id

df.withColumn("partition_id", spark_partition_id())
```

### 3. 广播变量

```python
# 广播小数据集
broadcast_var = spark.sparkContext.broadcast({"key": "value"})

# 使用
def my_udf(value):
    return broadcast_var.value.get(value)

# 清理
broadcast_var.unpersist()
```

### 4. 累加器

```python
# 创建累加器
acc = spark.sparkContext.accumulator(0)

def add_to_acc(value):
    acc.add(value)

# 使用
df.foreach(lambda row: add_to_acc(row["amount"]))

print(acc.value)
```

## 监控和调优

### 1. Spark UI

访问 `http://localhost:4040` 查看:
- Jobs
- Stages
- Storage
- Environment
- Executors

### 2. 执行计划

```python
# 查看逻辑计划
df.explain(True)

# 查看物理计划
df.explain()
```

### 3. 内存调优

```python
# spark-submit参数
--executor-memory 8G \
--executor-cores 4 \
--driver-memory 4G \
--conf spark.sql.shuffle.partitions=200
```

## 最佳实践

### ✅ DO

1. **使用Parquet格式**
```python
# ✅ 列式存储,性能好
df.write.parquet("output")
```

2. **合理分区**
```python
# ✅ 按常用查询字段分区
df.write.partitionBy("date", "hour").parquet("output")
```

3. **使用广播Join**
```python
# ✅ 小表广播
joined = large_df.join(broadcast(small_df), "id")
```

4. **缓存重用数据**
```python
# ✅ 多次使用的数据
df.cache()
df.count()
df.show()
```

### ❌ DON'T

1. **不要使用collect()获取大数据**
```python
# ❌ 内存溢出
data = df.collect()

# ✅ 使用limit
data = df.limit(1000).collect()
```

2. **不要过度分区**
```python
# ❌ 太多小分区
df.repartition(1000)

# ✅ 合理分区数
df.repartition(200)
```

3. **不要使用row count做检查**
```python
# ❌ 触发完整计算
if df.count() > 0:
    df.show()

# ✅ 使用take
if df.head(1):
    df.show()
```

## 学习路径

### 初级 (1-2周)
1. Spark架构和RDD基础
2. DataFrame和SQL
3. 基础ETL操作

### 中级 (2-3周)
1. 聚合和Window函数
2. UDF和性能优化
3. 流处理(Structured Streaming)

### 高级 (2-4周)
1. 自定义数据源
2. 集群调优
3. 生产部署

### 专家级 (持续)
1. 性能调优和故障排查
2. 多租户资源管理
3. 实时Lambda架构

## 参考资料

### 官方文档
- [Spark官方文档](https://spark.apache.org/docs/latest/)
- [PySpark文档](https://spark.apache.org/docs/latest/api/python/)

### 教程
- [Spark编程指南](https://spark.apache.org/docs/latest/rdd-programming-guide.html)
- [Spark SQL指南](https://spark.apache.org/docs/latest/sql-programming-guide.html)

---

**知识ID**: `spark-etl-playbook`  
**领域**: data-engineering  
**类型**: playbooks  
**难度**: intermediate  
**质量分**: 90  
**维护者**: data-team@superdev.com  
**最后更新**: 2026-03-28
