---
id: graphql-production-playbook
title: GraphQL 生产实战手册
domain: api
category: 02-playbooks
difficulty: advanced
tags: [graphql, api, schema-design, n-plus-1, dataloader, security, introspection, complexity, federation, apollo, enterprise]
quality_score: 93
maintainer: platform-team@superdev.com
last_updated: 2026-06-15
---

# GraphQL 生产实战手册

> 基于 [GraphQL Conf 2025 Schema Design](https://the-guild.dev/graphql/hive/blog/schema-design-best-practices-part-1) + [StackHawk Security](https://www.stackhawk.com/blog/graphql-security/) + [Zuplo API Design](https://zuplo.com/learning-center/graphql-api-design)

## Schema 设计原则

### 类型设计
```graphql
# ✅ 语义化命名，面向能力而非数据库结构
type Order {
  id: ID!
  number: String!          # 订单编号（面向用户）
  status: OrderStatus!     # 枚举而非 String
  total: Money!            # 自定义标量（带货币）
  items: [OrderItem!]!
  customer: Customer!
  createdAt: DateTime!
}

enum OrderStatus {
  PENDING
  PAID
  SHIPPED
  DELIVERED
  CANCELLED
}

# ✅ Money 类型（不用 Float！）
scalar Money  # { amount: 99.50, currency: "USD" }
```

### Nullability 策略
```graphql
# ✅ 有意识地标 nullable/non-null
type User {
  id: ID!              # 必有
  email: String!       # 必有
  avatarUrl: String    # 可选（可能没头像）
  phone: String        # 可选
}
# 非必要不用 ! —— 客户端要处理 null，但不会因 null 崩溃
```

### 分页（Connection 模式）
```graphql
type Query {
  products(first: Int = 20, after: String): ProductConnection!
}

type ProductConnection {
  edges: [ProductEdge!]!
  pageInfo: PageInfo!
}

type ProductEdge {
  node: Product!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  endCursor: String
}
```

## N+1 问题 + DataLoader

```typescript
// ❌ N+1：查 100 个订单 → 100 次查客户
const resolvers = {
  Order: {
    customer: (order) => db.query('SELECT * FROM customers WHERE id = ?', order.customerId),
    // 100 个订单 = 100 次 DB 查询
  }
};

// ✅ DataLoader：批量查询（1 次查 100 个客户）
import DataLoader from 'dataloader';
const customerLoader = new DataLoader(async (customerIds) => {
  // 一次性查所有 customerIds
  const customers = await db.query('SELECT * FROM customers WHERE id IN (?)', [customerIds]);
  return customerIds.map(id => customers.find(c => c.id === id));
});

const resolvers = {
  Order: {
    customer: (order) => customerLoader.load(order.customerId),
    // 100 个订单 = 1 次 DB 查询（batch）
  }
};
```

## 安全

### 生产关闭 Introspection
```typescript
// ❌ 生产开放 introspection（泄露完整 schema）
const server = new ApolloServer({ typeDefs, resolvers });

// ✅ 生产关闭（客户端已有 schema）
const server = new ApolloServer({
  typeDefs,
  resolvers,
  introspection: process.env.NODE_ENV !== 'production',
});
```

### 查询复杂度限制
```typescript
import { createComplexityRule } from 'graphql-query-complexity';

const server = new ApolloServer({
  validationRules: [
    createComplexityRule({
      maximumComplexity: 1000,    // 最大复杂度
      variables: {},               // 传入变量计算
    }),
  ],
});
// 恶意查询 { users { orders { items { product { reviews } } } } } → 拒绝
```

### 字段级授权
```typescript
const resolvers = {
  User: {
    email: (user, _, context) => {
      // 只有本人或 admin 能看 email
      if (context.userId !== user.id && !context.isAdmin) return null;
      return user.email;
    },
    ssn: (user, _, context) => {
      // 只有 admin 能看社保号
      if (!context.isAdmin) throw new ForbiddenError('No access');
      return user.ssn;
    },
  },
};
```

### Persisted Queries（减少攻击面）
```typescript
// 客户端只发 query hash，不发完整 query
// 未注册的 query 直接拒绝
const server = new ApolloServer({
  typeDefs,
  resolvers,
  persistedQueries: {
    cache: new RedisCache(),
  },
});
```

## 生产检查清单
- [ ] 生产关闭 Introspection
- [ ] 查询复杂度限制（防深度/广度攻击）
- [ ] DataLoader 解决 N+1
- [ ] 字段级授权（每个 resolver 检查权限）
- [ ] Persisted Queries（减少攻击面）
- [ ] 速率限制（按用户/IP）
- [ ] 错误信息不泄露内部结构
- [ ] 分页用 Connection 模式（不用全量数组）
- [ ] Subscription 有连接管理（断线清理）
