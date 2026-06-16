---
id: websocket-realtime-playbook
title: WebSocket 实时通信实战手册
domain: backend
category: 02-playbooks
difficulty: advanced
tags: [websocket, socketio, realtime, redis, pub-sub, scaling, sticky-session, reconnection, chat, notification, enterprise]
quality_score: 93
maintainer: platform-team@superdev.com
last_updated: 2026-06-15
---

# WebSocket 实时通信实战手册

> 基于 [Ably: Scaling Socket.IO](https://ably.com/topic/scaling-socketio) + [OneUptime WebSocket Scaling 2026](https://oneuptime.com/blog/post/2026-01-26-websocket-scaling/view) + [websocket.org Scale Guide](https://websocket.org/guides/websockets-at-scale/)

## 单节点 Socket.IO

```typescript
import { Server } from "socket.io";

const io = new Server(httpServer, {
  cors: { origin: "https://app.example.com", credentials: true },
});

// 认证中间件
io.use((socket, next) => {
  const token = socket.handshake.auth.token;
  if (!verifyJWT(token)) return next(new Error("Unauthorized"));
  socket.userId = decodeJWT(token).sub;
  next();
});

io.on("connection", (socket) => {
  // 加入用户专属 room（方便定向推送）
  socket.join(`user:${socket.userId}`);

  socket.on("message", (data) => {
    // 广播到房间（聊天场景）
    socket.to(`room:${data.roomId}`).emit("message", data);
  });

  socket.on("disconnect", () => {
    console.log(`User ${socket.userId} disconnected`);
  });
});
```

## 多节点扩展（Redis Pub/Sub）

```typescript
// ❌ 多节点不共享状态——用户 A 连到 Node 1，B 连到 Node 2
// Node 1 发消息，Node 2 的用户收不到！

// ✅ Redis Adapter——跨节点同步事件
import { createAdapter } from "@socket.io/redis-adapter";
import { createClient } from "redis";

const pubClient = createClient({ url: "redis://redis:6379" });
const subClient = pubClient.duplicate();
await Promise.all([pubClient.connect(), subClient.connect()]);

io.adapter(createAdapter(pubClient, subClient));
// 现在 Node 1 发消息 → Redis → Node 2 的用户也能收到
```

### 负载均衡配置
```nginx
# nginx — WebSocket 需要 sticky session 或 IP hash
upstream websocket {
  ip_hash;  # 同一用户始终连同一节点（减少 Redis 转发）
  server node1:3000;
  server node2:3000;
  server node3:3000;
}

# WebSocket upgrade
location /socket.io/ {
  proxy_pass http://websocket;
  proxy_http_version 1.1;
  proxy_set_header Upgrade $http_upgrade;
  proxy_set_header Connection "upgrade";
  proxy_read_timeout 86400;  # 长连接超时（24h）
}
```

## 断线重连 + 消息队列

```typescript
// 前端：断线时消息入队，重连后补发
const messageQueue = [];
let isConnected = false;

socket.on("connect", () => {
  isConnected = true;
  // 重连后补发队列消息
  while (messageQueue.length > 0) {
    socket.emit("message", messageQueue.shift());
  }
});

socket.on("disconnect", () => {
  isConnected = false;
});

function sendMessage(data) {
  if (isConnected) {
    socket.emit("message", data);
  } else {
    messageQueue.push(data);  // 离线入队
  }
}

// 自动重连配置
const socket = io({
  reconnection: true,
  reconnectionAttempts: Infinity,
  reconnectionDelay: 1000,      // 首次重连 1s
  reconnectionDelayMax: 30000,  // 最大 30s
});
```

## 心跳检测

```typescript
// 服务端配置心跳（默认已开启）
const io = new Server(httpServer, {
  pingInterval: 25000,   // 每 25s 发 ping
  pingTimeout: 20000,    // 20s 没回 pong = 断开
});
// 自动检测死连接，触发 disconnect 事件清理资源
```

## 连接数优化（百万连接）

| 调优项 | 推荐值 | 说明 |
|--------|--------|------|
| 文件描述符 | `ulimit -n 1000000` | 每连接占一个 fd |
| 端口范围 | `net.ipv4.ip_local_port_range` | 出站连接端口 |
| TCP keepalive | 60s | 检测死连接 |
| 内存 | 50KB/连接 | 100 万连接 ≈ 50GB |
| CPU | event-loop 不阻塞 | 重操作走 worker |

## 生产检查清单
- [ ] Redis Adapter（多节点事件同步）
- [ ] 连接认证（JWT 验证 handshake）
- [ ] 心跳检测（pingInterval + pingTimeout）
- [ ] 断线重连 + 消息队列（前端补发）
- [ ] 连接清理（disconnect 移除 room/session）
- [ ] 负载均衡 sticky session（IP hash）
- [ ] nginx WebSocket upgrade 配置
- [ ] 文件描述符调优（ulimit）
- [ ] 监控并发连接数 + 消息吞吐
- [ ] 限流（防恶意连接洪水）
