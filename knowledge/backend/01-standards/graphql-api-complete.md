---
id: graphql-api-complete
title: GraphQL API 完整指南
domain: backend
category: 01-standards
difficulty: intermediate
tags: [api, backend, complete, dataloader, graphql, resolver, schema, subscription]
quality_score: 70
last_updated: 2026-06-15
---
# GraphQL API 完整指南

## 概述

GraphQL 是 Facebook 开发的 API 查询语言和运行时，允许客户端精确指定所需数据。与 REST 相比，GraphQL 解决了 over-fetching 和 under-fetching 问题，提供强类型 Schema、内省能力和实时订阅支持。

### 何时选择 GraphQL

- 多端（Web/iOS/Android）数据需求差异大
- 页面需要聚合多个资源的数据
- 需要实时数据推送（Subscription）
- API 演进频繁，需要向后兼容
- 前端团队需要自主决定数据形状

### 何时选择 REST

- 简单 CRUD 操作
- 文件上传/下载为主
- 缓存需求强（HTTP 缓存天然支持）
- 团队 GraphQL 经验不足

---

## Schema 设计

### SDL (Schema Definition Language)

```graphql
# 类型定义
type User {
  id: ID!
  email: String!
  name: String!
  role: Role!
  posts(first: Int = 10, after: String): PostConnection!
  createdAt: DateTime!
}

type Post {
  id: ID!
  title: String!
  content: String!
  author: User!
  tags: [Tag!]!
  status: PostStatus!
  publishedAt: DateTime
  createdAt: DateTime!
}

type Tag {
  id: ID!
  name: String!
  posts(first: Int = 10, after: String): PostConnection!
}

# 枚举
enum Role {
  USER
  EDITOR
  ADMIN
}

enum PostStatus {
  DRAFT
  PUBLISHED
  ARCHIVED
}

# 自定义标量
scalar DateTime
scalar JSON

# 连接（Relay 规范分页）
type PostConnection {
  edges: [PostEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type PostEdge {
  node: Post!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}

# 输入类型
input CreatePostInput {
  title: String!
  content: String!
  tags: [ID!]
  status: PostStatus = DRAFT
}

input UpdatePostInput {
  title: String
  content: String
  tags: [ID!]
  status: PostStatus
}

# Query
type Query {
  user(id: ID!): User
  me: User!
  posts(
    first: Int = 20
    after: String
    status: PostStatus
    search: String
  ): PostConnection!
  post(id: ID!): Post
}

# Mutation
type Mutation {
  createPost(input: CreatePostInput!): Post!
  updatePost(id: ID!, input: UpdatePostInput!): Post!
  deletePost(id: ID!): Boolean!
  login(email: String!, password: String!): AuthPayload!
}

type AuthPayload {
  token: String!
  user: User!
}

# Subscription
type Subscription {
  postPublished: Post!
  commentAdded(postId: ID!): Comment!
}
```

---

## Resolver 实现

### Node.js (Apollo Server)

```typescript
import { GraphQLResolveInfo } from "graphql";

const resolvers = {
  Query: {
    me: (_parent: unknown, _args: unknown, context: Context) => {
      if (!context.user) throw new AuthenticationError("未认证");
      return context.user;
    },

    posts: async (_parent: unknown, args: PostsArgs, context: Context) => {
      const { first = 20, after, status, search } = args;
      return context.dataSources.postAPI.getPosts({ first, after, status, search });
    },

    post: async (_parent: unknown, { id }: { id: string }, context: Context) => {
      const post = await context.dataSources.postAPI.getPostById(id);
      if (!post) throw new NotFoundError(`Post ${id} not found`);
      return post;
    },
  },

  Mutation: {
    createPost: async (_parent: unknown, { input }: { input: CreatePostInput }, context: Context) => {
      if (!context.user) throw new AuthenticationError("未认证");
      return context.dataSources.postAPI.createPost({
        ...input,
        authorId: context.user.id,
      });
    },

    updatePost: async (_parent: unknown, { id, input }: { id: string; input: UpdatePostInput }, context: Context) => {
      if (!context.user) throw new AuthenticationError("未认证");
      const post = await context.dataSources.postAPI.getPostById(id);
      if (!post) throw new NotFoundError(`Post ${id} not found`);
      if (post.authorId !== context.user.id && context.user.role !== "ADMIN") {
        throw new ForbiddenError("无权修改");
      }
      return context.dataSources.postAPI.updatePost(id, input);
    },
  },

  // 字段级 Resolver
  User: {
    posts: (parent: User, args: ConnectionArgs, context: Context) => {
      return context.dataSources.postAPI.getPostsByAuthor(parent.id, args);
    },
  },

  Post: {
    author: (parent: Post, _args: unknown, context: Context) => {
      // 使用 DataLoader 避免 N+1
      return context.loaders.userLoader.load(parent.authorId);
    },
    tags: (parent: Post, _args: unknown, context: Context) => {
      return context.loaders.tagLoader.load(parent.id);
    },
  },
};
```

---

## DataLoader (N+1 解决方案)

```typescript
import DataLoader from "dataloader";

// 创建 DataLoader
function createLoaders(db: Database) {
  return {
    userLoader: new DataLoader<string, User>(async (ids) => {
      const users = await db.user.findMany({
        where: { id: { in: [...ids] } },
      });
      const userMap = new Map(users.map(u => [u.id, u]));
      return ids.map(id => userMap.get(id) || new Error(`User ${id} not found`));
    }),

    tagLoader: new DataLoader<string, Tag[]>(async (postIds) => {
      const postTags = await db.postTag.findMany({
        where: { postId: { in: [...postIds] } },
        include: { tag: true },
      });
      const tagMap = new Map<string, Tag[]>();
      for (const pt of postTags) {
        const tags = tagMap.get(pt.postId) || [];
        tags.push(pt.tag);
        tagMap.set(pt.postId, tags);
      }
      return postIds.map(id => tagMap.get(id) || []);
    }),
  };
}

// 在 Context 中每请求创建新实例
const server = new ApolloServer({
  typeDefs,
  resolvers,
  context: ({ req }) => ({
    user: getUserFromToken(req.headers.authorization),
    loaders: createLoaders(db),
    dataSources: { postAPI: new PostAPI(db) },
  }),
});
```

---

## 订阅 (Subscription)

```typescript
import { PubSub } from "graphql-subscriptions";

const pubsub = new PubSub();

const resolvers = {
  Mutation: {
    createPost: async (_parent, { input }, context) => {
      const post = await context.dataSources.postAPI.createPost(input);
      if (post.status === "PUBLISHED") {
        pubsub.publish("POST_PUBLISHED", { postPublished: post });
      }
      return post;
    },
  },

  Subscription: {
    postPublished: {
      subscribe: () => pubsub.asyncIterator(["POST_PUBLISHED"]),
    },
    commentAdded: {
      subscribe: (_parent, { postId }) =>
        pubsub.asyncIterator([`COMMENT_ADDED_${postId}`]),
    },
  },
};
```

---

## 安全

### 查询深度与复杂度限制

```typescript
import depthLimit from "graphql-depth-limit";
import { createComplexityLimitRule } from "graphql-validation-complexity";

const server = new ApolloServer({
  typeDefs,
  resolvers,
  validationRules: [
    depthLimit(7),                              // 查询深度限制
    createComplexityLimitRule(1000, {            // 查询复杂度限制
      scalarCost: 1,
      objectCost: 5,
      listFactor: 10,
    }),
  ],
});
```

### 速率限制

```typescript
import { RateLimitDirective } from "graphql-rate-limit-directive";

const typeDefs = gql`
  directive @rateLimit(limit: Int!, duration: Int!) on FIELD_DEFINITION

  type Mutation {
    login(email: String!, password: String!): AuthPayload!
      @rateLimit(limit: 5, duration: 60)
    createPost(input: CreatePostInput!): Post!
      @rateLimit(limit: 10, duration: 60)
  }
`;
```

### 字段级授权

```typescript
const resolvers = {
  User: {
    email: (parent, _args, context) => {
      // 仅自己或管理员可见邮箱
      if (context.user?.id === parent.id || context.user?.role === "ADMIN") {
        return parent.email;
      }
      return null;
    },
  },
};
```

---

## 性能优化

### 持久化查询 (Persisted Queries)

```typescript
// 客户端发送 query hash 而非完整 query
// POST /graphql
// { "extensions": { "persistedQuery": { "version": 1, "sha256Hash": "abc123..." } } }

import { ApolloServerPluginCacheControl } from "@apollo/server/plugin/cacheControl";

const server = new ApolloServer({
  typeDefs,
  resolvers,
  persistedQueries: { ttl: 900 }, // 15 分钟 TTL
  plugins: [ApolloServerPluginCacheControl({ defaultMaxAge: 60 })],
});
```

### 查询批处理

```typescript
// 客户端批量发送多个查询
// POST /graphql
// [{ "query": "..." }, { "query": "..." }]

// Apollo Client 配置
const link = new BatchHttpLink({
  uri: "/graphql",
  batchMax: 5,        // 最多 5 个查询合批
  batchInterval: 20,  // 20ms 窗口
});
```

---

## 常见反模式

| 反模式 | 问题 | 正确做法 |
|--------|------|----------|
| 不用 DataLoader | N+1 查询 | 每请求创建 DataLoader 实例 |
| 无深度/复杂度限制 | DoS 攻击 | 设置 depthLimit + complexityLimit |
| Schema 暴露内部结构 | 信息泄露 | 以客户端需求设计 Schema |
| 巨型 Resolver 函数 | 难维护 | 拆分为 Service + DataSource |
| 忽略错误处理 | 敏感信息泄露 | 统一错误格式，隐藏内部细节 |
| Subscription 用轮询实现 | 浪费资源 | 用 WebSocket + PubSub |

---

## Agent Checklist

- [ ] Schema 遵循 Relay 规范分页（Connection / Edge / PageInfo）
- [ ] 所有关联字段使用 DataLoader 避免 N+1
- [ ] 设置查询深度限制（推荐 <= 7 层）
- [ ] 设置查询复杂度限制
- [ ] 敏感字段实现字段级授权
- [ ] Mutation 使用 Input 类型参数化
- [ ] 错误返回统一格式，不暴露内部堆栈
- [ ] 登录等敏感操作设置速率限制
- [ ] 生产环境禁用 Introspection（或仅限内部）
- [ ] Subscription 使用 WebSocket，生产环境用 Redis PubSub
- [ ] Schema 变更有向后兼容策略（@deprecated 标记）
- [ ] 接入 APM 监控 Resolver 级别性能
