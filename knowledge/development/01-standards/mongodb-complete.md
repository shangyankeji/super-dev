---
id: mongodb-complete
title: MongoDB完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, mongodb, python客户端, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# MongoDB完整指南

## 概述
MongoDB是NoSQL文档数据库,存储JSON格式的文档,支持灵活的schema、水平扩展和丰富的查询语言。本指南覆盖CRUD操作、聚合管道、索引和最佳实践。

## 核心概念

### 1. 文档模型

**数据库和集合**:
```javascript
// 创建/切换数据库
use mydb

// 创建集合
db.createCollection('users')

// 插入文档(自动创建集合)
db.users.insertOne({
  name: 'Alice',
  age: 30,
  email: 'alice@example.com',
  tags: ['developer', 'python'],
  address: {
    city: 'Beijing',
    country: 'China'
  }
})

// 批量插入
db.users.insertMany([
  { name: 'Bob', age: 25 },
  { name: 'Carol', age: 28 }
])
```

### 2. CRUD操作

**查询**:
```javascript
// 查询所有
db.users.find()

// 条件查询
db.users.find({ age: { $gte: 25 } })

// 投影(只返回指定字段)
db.users.find({}, { name: 1, email: 1, _id: 0 })

// 排序
db.users.find().sort({ age: -1 })  // 降序

// 分页
db.users.find().skip(10).limit(10)

// 比较运算符
db.users.find({
  age: { 
    $eq: 30,    // 等于
    $ne: 25,    // 不等于
    $gt: 20,    // 大于
    $gte: 25,   // 大于等于
    $lt: 40,    // 小于
    $lte: 35    // 小于等于
  }
})

// 逻辑运算符
db.users.find({
  $and: [
    { age: { $gte: 25 } },
    { status: 'active' }
  ]
})

db.users.find({
  $or: [
    { age: { $lt: 20 } },
    { age: { $gt: 30 } }
  ]
})

// 数组查询
db.users.find({ tags: 'python' })  // 包含
db.users.find({ tags: { $all: ['python', 'mongodb'] } })  // 包含所有
db.users.find({ tags: { $size: 3 } })  // 数组长度

// 嵌套文档查询
db.users.find({ 'address.city': 'Beijing' })
```

**更新**:
```javascript
// 更新单个文档
db.users.updateOne(
  { name: 'Alice' },
  { $set: { age: 31, status: 'active' } }
)

// 更新多个文档
db.users.updateMany(
  { age: { $lt: 30 } },
  { $set: { category: 'young' } }
)

// 替换文档
db.users.replaceOne(
  { name: 'Alice' },
  { name: 'Alice', age: 32 }
)

// 更新操作符
db.users.updateOne(
  { name: 'Alice' },
  {
    $set: { age: 32 },           // 设置字段
    $unset: { tempField: 1 },    // 删除字段
    $inc: { score: 10 },         // 增加数值
    $push: { tags: 'golang' },   // 添加到数组
    $pull: { tags: 'old' },      // 从数组删除
    $addToSet: { tags: 'unique' } // 添加唯一值
  }
)
```

**删除**:
```javascript
// 删除单个文档
db.users.deleteOne({ name: 'Alice' })

// 删除多个文档
db.users.deleteMany({ status: 'inactive' })

// 删除所有文档
db.users.deleteMany({})

// 删除集合
db.users.drop()
```

### 3. 聚合管道

```javascript
// 基本聚合
db.orders.aggregate([
  // 匹配阶段
  { $match: { status: 'completed' } },
  
  // 分组阶段
  {
    $group: {
      _id: '$customerId',
      totalAmount: { $sum: '$amount' },
      count: { $sum: 1 },
      avgAmount: { $avg: '$amount' }
    }
  },
  
  // 排序
  { $sort: { totalAmount: -1 } },
  
  // 限制
  { $limit: 10 }
])

// lookup(关联查询)
db.orders.aggregate([
  {
    $lookup: {
      from: 'customers',
      localField: 'customerId',
      foreignField: '_id',
      as: 'customer'
    }
  },
  {
    $unwind: '$customer'  // 展开数组
  },
  {
    $project: {
      orderId: '$_id',
      amount: 1,
      customerName: '$customer.name'
    }
  }
])

// 文本搜索
db.articles.createIndex({ content: 'text' })

db.articles.aggregate([
  {
    $match: {
      $text: { $search: 'mongodb database' }
    }
  },
  {
    $addFields: {
      score: { $meta: 'textScore' }
    }
  },
  {
    $sort: { score: -1 }
  }
])
```

### 4. 索引

```javascript
// 创建索引
db.users.createIndex({ email: 1 })  // 升序
db.users.createIndex({ name: 1, age: -1 })  // 复合索引

// 唯一索引
db.users.createIndex({ email: 1 }, { unique: true })

// 文本索引
db.articles.createIndex({ title: 'text', content: 'text' })

// 地理空间索引
db.places.createIndex({ location: '2dsphere' })

// 查看索引
db.users.getIndexes()

// 删除索引
db.users.dropIndex('index_name')

// 查询计划
db.users.find({ email: 'alice@example.com' }).explain('executionStats')
```

### 5. 事务

```javascript
// MongoDB 4.0+ 支持多文档事务
const session = db.getMongo().startSession()

try {
  session.startTransaction()
  
  const usersCollection = session.getDatabase('mydb').users
  const ordersCollection = session.getDatabase('mydb').orders
  
  // 操作1
  await usersCollection.updateOne(
    { _id: userId },
    { $inc: { balance: -100 } },
    { session }
  )
  
  // 操作2
  await ordersCollection.insertOne(
    { userId, amount: 100 },
    { session }
  )
  
  await session.commitTransaction()
  console.log('Transaction committed')
} catch (error) {
  await session.abortTransaction()
  console.error('Transaction aborted:', error)
} finally {
  await session.endSession()
}
```

## Python客户端

```python
from pymongo import MongoClient
from bson.objectid import ObjectId

# 连接
client = MongoClient('mongodb://localhost:27017/')
db = client['mydb']
users = db['users']

# 插入
user_id = users.insert_one({
    'name': 'Alice',
    'age': 30,
    'email': 'alice@example.com'
}).inserted_id

# 查询
user = users.find_one({'_id': ObjectId(user_id)})
all_users = list(users.find({'age': {'$gte': 25}}))

# 更新
users.update_one(
    {'name': 'Alice'},
    {'$set': {'age': 31}}
)

# 删除
users.delete_one({'name': 'Alice'})

# 聚合
pipeline = [
    {'$match': {'age': {'$gte': 25}}},
    {'$group': {'_id': '$status', 'count': {'$sum': 1}}}
]
results = list(users.aggregate(pipeline))
```

## 最佳实践

### ✅ DO

1. **使用索引**
```javascript
// ✅ 好: 查询走索引
db.users.createIndex({ email: 1 })
db.users.find({ email: 'alice@example.com' })

// ❌ 差: 全表扫描
db.users.find({ $where: 'this.email.length > 10' })
```

2. **使用投影减少数据传输**
```javascript
// ✅ 好
db.users.find({}, { name: 1, email: 1, _id: 0 })

// ❌ 差
db.users.find({})  // 返回所有字段
```

3. **批量操作**
```python
# ✅ 好
users.insert_many([doc1, doc2, doc3])

# ❌ 差
for doc in docs:
    users.insert_one(doc)
```

### ❌ DON'T

1. **不要使用大文档**
```javascript
// ❌ 差: 文档超过16MB
{
  logs: [ /* 数百万条日志 */ ]
}

// ✅ 好: 分离到单独集合
db.logs.insertMany([log1, log2, ...])
```

2. **不要过度使用$lookup**
```javascript
// ❌ 差: 多层嵌套lookup
db.orders.aggregate([
  { $lookup: { ... } },
  { $lookup: { ... } },
  { $lookup: { ... } }
])

// ✅ 好: 考虑嵌入文档或反范式化
```

## 学习路径

### 初级 (1-2周)
1. 文档模型
2. CRUD操作
3. 基本查询

### 中级 (2-3周)
1. 聚合管道
2. 索引优化
3. 数据建模

### 高级 (2-4周)
1. 事务
2. 分片
3. 复制集

### 专家级 (持续)
1. 性能调优
2. 高可用架构
3. 大规模部署

---

**知识ID**: `mongodb-complete`  
**领域**: development  
**类型**: standards  
**难度**: intermediate  
**质量分**: 93  
**维护者**: dba-team@superdev.com  
**最后更新**: 2026-03-28
