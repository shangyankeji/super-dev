---
id: graphql-complete
title: GraphQL完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, graphql, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# GraphQL完整指南

## 概述
GraphQL是API查询语言,允许客户端精确指定数据需求。本指南覆盖Schema、Query、Mutation、订阅和最佳实践。

## 核心概念

### 1. Schema定义

**类型定义**:
```graphql
type User {
  id: ID!
  name: String!
  email: String!
  age: Int
  posts: [Post!]!
}

type Post {
  id: ID!
  title: String!
  content: String!
  author: User!
  createdAt: String!
}

type Query {
  users: [User!]!
  user(id: ID!): User
  posts: [Post!]!
}

type Mutation {
  createUser(input: CreateUserInput!): User!
  updateUser(id: ID!, input: UpdateUserInput!): User
  deleteUser(id: ID!): Boolean!
}

input CreateUserInput {
  name: String!
  email: String!
  age: Int
}

input UpdateUserInput {
  name: String
  email: String
  age: Int
}
```

### 2. 查询(Query)

**基础查询**:
```graphql
query {
  users {
    id
    name
    email
  }
}

# 带参数
query GetUser($id: ID!) {
  user(id: $id) {
    id
    name
    email
    posts {
      title
    }
  }
}

# 变量
{
  "id": "1"
}
```

**嵌套查询**:
```graphql
query {
  users {
    name
    posts {
      title
      author {
        name
      }
    }
  }
}
```

### 3. 变更(Mutation)

**创建数据**:
```graphql
mutation CreateUser($input: CreateUserInput!) {
  createUser(input: $input) {
    id
    name
    email
  }
}

# 变量
{
  "input": {
    "name": "Alice",
    "email": "alice@example.com",
    "age": 30
  }
}
```

**更新数据**:
```graphql
mutation UpdateUser($id: ID!, $input: UpdateUserInput!) {
  updateUser(id: $id, input: $input) {
    id
    name
    email
  }
}
```

### 4. Python实现

```python
import strawberry
from typing import List, Optional
from strawberry.fastapi import GraphQLRouter

@strawberry.type
class User:
    id: strawberry.ID
    name: str
    email: str
    age: Optional[int]
    
    @strawberry.field
    def posts(self) -> List['Post']:
        return db.get_user_posts(self.id)

@strawberry.type
class Query:
    @strawberry.field
    def users(self) -> List[User]:
        return db.get_all_users()
    
    @strawawberry.field
    def user(self, id: strawberry.ID) -> Optional[User]:
        return db.get_user(id)

@strawberry.type
class Mutation:
    @strawberry.mutation
    def create_user(self, name: str, email: str, age: Optional[int] = None) -> User:
        user = db.create_user(name, email, age)
        return user

schema = strawberry.Schema(query=Query, mutation=Mutation)
graphql_app = GraphQLRouter(schema)
```

### 5. FastAPI集成

```python
from fastapi import FastAPI
from strawberry.fastapi import GraphQLRouter

app = FastAPI()

# 挂载GraphQL端点
app.include_router(graphql_app, prefix='/graphql')

# 可选: 添加Playground
@app.get('/graphql')
async def graphql_playground():
    return {'message': 'GraphQL endpoint at /graphql'}
```

### 6. 订阅(Subscription)

```python
import strawberry
from strawberry.subscriptions import Subscription

@strawberry.type
class Subscription:
    @strawberry.subscription
    async def user_created(self) -> User:
        # 实时推送新用户
        async for user in user_created_stream():
            yield user

schema = strawberry.Schema(
    query=Query,
    mutation=Mutation,
    subscription=Subscription
)
```

## 最佳实践

### ✅ DO

1. **使用描述和弃用**
```graphql
type User {
  id: ID!
  "用户全名"
  name: String! @deprecated(reason: "Use displayName instead")
  displayName: String!
}
```

2. **输入验证**
```python
@strawberry.mutation
def create_user(self, input: CreateUserInput) -> User:
    if not validate_email(input.email):
        raise ValueError("Invalid email")
    return db.create_user(input)
```

### ❌ DON'T

1. **不要过度嵌套**
```graphql
# ❌ 差
query {
  users {
    posts {
      comments {
        author {
          posts {
            # 无限嵌套
          }
        }
      }
    }
  }
}
```

2. **不要忽略N+1问题**
```python
# ✅ 好: 使用DataLoader
from strawberry.dataloader import DataLoader

@strawberry.type
class User:
    @strawberry.field
    async def posts(self, loader: DataLoader) -> List[Post]:
        return await loader.load(self.id)
```

## 学习路径

### 初级 (1-2周)
1. Schema定义
2. Query查询
3. Mutation变更

### 中级 (2-3周)
1. 订阅
2. 错误处理
3. 认证授权

### 高级 (2-4周)
1. DataLoader
2. 联邦
3. 性能优化

---

**知识ID**: `graphql-complete`  
**领域**: development  
**类型**: standards  
**难度**: intermediate  
**质量分**: 93  
**维护者**: api-team@superdev.com  
**最后更新**: 2026-03-28
