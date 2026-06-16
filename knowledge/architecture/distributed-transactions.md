---
id: distributed-transactions
title: 分布式事务处理完全指南
domain: architecture
category: distributed-transactions.md
difficulty: intermediate
tags: [architecture, distributed, transactions, 分布式事务框架, 分布式事务模式, 参考资源, 常见问题, 最佳实践]
quality_score: 70
last_updated: 2026-06-15
---
# 分布式事务处理完全指南

## 概述

分布式事务是指跨越多个网络节点(服务、数据库)的事务操作,需要保证这些操作的原子性、一致性、隔离性和持久性(ACID)。在微服务架构中,由于每个服务拥有独立的数据库,传统的事务机制不再适用,需要采用专门的分布式事务解决方案。

## 核心挑战

### 1. ACID特性保障困难
```
原子性(Atomicity):
- 跨服务操作难以保证全部成功或全部失败
- 网络分区导致部分操作无法执行

一致性(Consistency):
- 多个数据库间数据同步延迟
- 不同服务间的业务规则一致性

隔离性(Isolation):
- 跨数据库的并发控制复杂
- 锁机制难以协调

持久性(Durability):
- 多节点故障恢复
- 数据不一致的修复
```

### 2. CAP理论权衡
```
Consistency(一致性): 强一致性 vs 最终一致性
Availability(可用性): 高可用 vs 数据准确
Partition Tolerance(分区容错): 网络分区时系统行为

现实选择:
- 金融场景: CP(一致性优先)
- 社交场景: AP(可用性优先)
- 一般业务: 平衡AP和CP
```

### 3. 网络不确定性
```
超时:
- 请求发出后未收到响应
- 无法判断操作是否成功

分区:
- 网络中断,节点间无法通信
- 脑裂问题

延迟:
- 跨数据中心调用延迟高
- 影响事务响应时间
```

## 分布式事务模式

### 1. 两阶段提交(2PC, Two-Phase Commit)

#### 工作原理
```
阶段一(准备阶段):
1. 协调者向所有参与者发送准备请求
2. 参与者执行本地事务,但不提交
3. 参与者返回准备就绪或失败

阶段二(提交阶段):
- 如果所有参与者都准备就绪:
  1. 协调者发送提交命令
  2. 参与者提交事务
  3. 返回提交成功

- 如果有参与者失败:
  1. 协调者发送回滚命令
  2. 参与者回滚事务
  3. 返回回滚成功
```

#### 实现示例
```java
// 协调者
public class TransactionCoordinator {
    public boolean commitDistributedTransaction(List<Participant> participants) {
        // 阶段一: 准备
        boolean allPrepared = true;
        for (Participant p : participants) {
            if (!p.prepare()) {
                allPrepared = false;
                break;
            }
        }

        // 阶段二: 提交或回滚
        if (allPrepared) {
            for (Participant p : participants) {
                p.commit();
            }
            return true;
        } else {
            for (Participant p : participants) {
                p.rollback();
            }
            return false;
        }
    }
}

// 参与者
public class Participant {
    public boolean prepare() {
        try {
            // 执行本地事务,不提交
            connection.setAutoCommit(false);
            executeLocalTransaction();
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    public void commit() {
        connection.commit();
    }

    public void rollback() {
        connection.rollback();
    }
}
```

#### 优缺点
```
优点:
- 强一致性保证
- 数据不丢失
- 逻辑清晰

缺点:
- 同步阻塞: 参与者锁定资源等待
- 单点故障: 协调者故障导致系统阻塞
- 数据不一致: 阶段二部分提交失败
- 性能差: 网络往返次数多

适用场景:
- 对一致性要求极高
- 参与者数量少
- 网络稳定的环境
```

### 2. 三阶段提交(3PC, Three-Phase Commit)

#### 改进点
```
引入CanCommit阶段:
1. CanCommit: 询问是否可以执行事务
2. PreCommit: 执行事务,准备提交
3. DoCommit: 正式提交

超时机制:
- 参与者超时后自动提交或回滚
- 减少阻塞时间
```

#### 优缺点
```
优点:
- 减少阻塞范围
- 引入超时机制

缺点:
- 仍可能数据不一致
- 网络往返次数更多
- 实现复杂

适用场景:
- 对2PC的改进方案
- 要求降低阻塞时间
```

### 3. TCC(Try-Confirm-Cancel)

#### 工作原理
```
Try阶段:
- 资源预留和锁定
- 业务检查
- 不执行真正业务

Confirm阶段:
- 确认执行业务操作
- 使用Try阶段预留的资源
- 不做业务检查

Cancel阶段:
- 取消业务操作
- 释放Try阶段预留的资源
```

#### 实现示例
```java
// 转账服务
public class TransferService {
    // Try: 冻结转账金额
    public boolean tryTransfer(String fromAccount, String toAccount, BigDecimal amount) {
        Account from = accountRepository.findByAccountNumber(fromAccount);
        if (from.getBalance().compareTo(amount) < 0) {
            return false;
        }

        // 冻结金额
        from.setFrozenAmount(from.getFrozenAmount().add(amount));
        from.setBalance(from.getBalance().subtract(amount));
        accountRepository.save(from);

        return true;
    }

    // Confirm: 确认转账
    public void confirmTransfer(String fromAccount, String toAccount, BigDecimal amount) {
        Account from = accountRepository.findByAccountNumber(fromAccount);
        Account to = accountRepository.findByAccountNumber(toAccount);

        // 清除冻结
        from.setFrozenAmount(from.getFrozenAmount().subtract(amount));

        // 入账
        to.setBalance(to.getBalance().add(amount));

        accountRepository.save(from);
        accountRepository.save(to);
    }

    // Cancel: 取消转账
    public void cancelTransfer(String fromAccount, String toAccount, BigDecimal amount) {
        Account from = accountRepository.findByAccountNumber(fromAccount);

        // 释放冻结,恢复余额
        from.setFrozenAmount(from.getFrozenAmount().subtract(amount));
        from.setBalance(from.getBalance().add(amount));

        accountRepository.save(from);
    }
}

// TCC事务协调器
public class TccTransactionCoordinator {
    public void executeTransfer(String from, String to, BigDecimal amount) {
        try {
            // Try
            if (!transferService.tryTransfer(from, to, amount)) {
                throw new RuntimeException("Try阶段失败");
            }

            // Confirm
            transferService.confirmTransfer(from, to, amount);

        } catch (Exception e) {
            // Cancel
            transferService.cancelTransfer(from, to, amount);
            throw e;
        }
    }
}
```

#### 数据库设计
```sql
-- 账户表
CREATE TABLE account (
    id BIGINT PRIMARY KEY,
    account_number VARCHAR(32) UNIQUE NOT NULL,
    balance DECIMAL(19,2) NOT NULL,
    frozen_amount DECIMAL(19,2) DEFAULT 0,
    version INT DEFAULT 0,
    INDEX idx_account_number (account_number)
);

-- TCC事务日志
CREATE TABLE tcc_transaction_log (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    transaction_id VARCHAR(64) NOT NULL,
    branch_id VARCHAR(64) NOT NULL,
    status VARCHAR(20) NOT NULL, -- TRY, CONFIRM, CANCEL
    business_key VARCHAR(128),
    context TEXT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_transaction (transaction_id),
    INDEX idx_status (status)
);
```

#### 优缺点
```
优点:
- 最终一致性
- 锁粒度小,性能好
- 灵活性高,业务可控

缺点:
- 业务侵入性强
- 实现复杂,需要三个接口
- 需要幂等性处理
- 需要防悬挂(空回滚)

适用场景:
- 金融转账
- 库存预占
- 优惠券发放
```

#### 关键问题解决

##### 幂等性
```java
public boolean tryTransfer(String transactionId, String from, String to, BigDecimal amount) {
    // 检查是否已执行
    TccTransactionLog log = logRepository.findByTransactionId(transactionId);
    if (log != null && "TRY".equals(log.getStatus())) {
        return true; // 已执行,直接返回成功
    }

    // 执行业务
    // ...

    // 记录日志
    logRepository.save(new TccTransactionLog(transactionId, "TRY"));
    return true;
}
```

##### 防悬挂
```java
public void cancelTransfer(String transactionId, String from, String to, BigDecimal amount) {
    // 检查Try是否已执行
    TccTransactionLog log = logRepository.findByTransactionId(transactionId);
    if (log == null) {
        // Try未执行,记录Cancel日志防止悬挂
        logRepository.save(new TccTransactionLog(transactionId, "CANCEL"));
        return;
    }

    // 执行Cancel逻辑
    // ...
}
```

### 4. Saga模式

#### 编排式Saga(Choreography)

##### 工作原理
```
1. 每个服务执行本地事务
2. 发布事件通知其他服务
3. 其他服务监听事件并执行相应操作
4. 如果某步骤失败,执行补偿事务
```

##### 实现示例
```java
// 订单服务
@Service
public class OrderService {
    @Transactional
    public Order createOrder(OrderRequest request) {
        // 1. 创建订单(待确认状态)
        Order order = new Order();
        order.setStatus("CREATED");
        orderRepository.save(order);

        // 2. 发布订单创建事件
        eventPublisher.publish(new OrderCreatedEvent(order.getId(), request));

        return order;
    }

    @Transactional
    @EventListener
    public void handlePaymentFailed(PaymentFailedEvent event) {
        // 补偿: 取消订单
        Order order = orderRepository.findById(event.getOrderId());
        order.setStatus("CANCELLED");
        orderRepository.save(order);

        // 发布订单取消事件
        eventPublisher.publish(new OrderCancelledEvent(order.getId()));
    }
}

// 支付服务
@Service
public class PaymentService {
    @Transactional
    @EventListener
    public void handleOrderCreated(OrderCreatedEvent event) {
        try {
            // 执行支付
            Payment payment = paymentService.process(event.getPaymentRequest());

            // 发布支付成功事件
            eventPublisher.publish(new PaymentSuccessEvent(event.getOrderId(), payment.getId()));

        } catch (Exception e) {
            // 发布支付失败事件
            eventPublisher.publish(new PaymentFailedEvent(event.getOrderId()));
        }
    }
}

// 库存服务
@Service
public class InventoryService {
    @Transactional
    @EventListener
    public void handlePaymentSuccess(PaymentSuccessEvent event) {
        // 扣减库存
        inventoryService.deduct(event.getProductId(), event.getQuantity());

        // 发布库存扣减成功事件
        eventPublisher.publish(new InventoryDeductedEvent(event.getOrderId()));
    }

    @Transactional
    @EventListener
    public void handleOrderCancelled(OrderCancelledEvent event) {
        // 补偿: 恢复库存
        inventoryService.restore(event.getProductId(), event.getQuantity());
    }
}
```

##### 优缺点
```
优点:
- 简单,无中心协调器
- 服务间松耦合
- 易于扩展

缺点:
- 流程不清晰,难以理解
- 事件循环依赖风险
- 调试困难
- 补偿逻辑分散

适用场景:
- 简单业务流程
- 服务数量少
- 团队自治度高
```

#### 编制式Saga(Orchestration)

##### 工作原理
```
1. 中央协调器(Saga Orchestrator)管理整个流程
2. 协调器按顺序调用各个服务
3. 服务执行成功,继续下一步
4. 服务执行失败,协调器执行补偿操作
```

##### 实现示例
```java
// Saga定义
public class CreateOrderSaga {
    private List<SagaStep> steps;

    public CreateOrderSaga() {
        steps = Arrays.asList(
            new CreateOrderStep(),
            new ReserveInventoryStep(),
            new ProcessPaymentStep(),
            new ConfirmOrderStep()
        );
    }

    public void execute(OrderContext context) {
        int currentStep = 0;

        try {
            // 正向执行
            for (; currentStep < steps.size(); currentStep++) {
                steps.get(currentStep).execute(context);
            }
        } catch (Exception e) {
            // 补偿执行(回滚已执行的步骤)
            for (int i = currentStep - 1; i >= 0; i--) {
                steps.get(i).compensate(context);
            }
            throw e;
        }
    }
}

// Saga步骤接口
public interface SagaStep {
    void execute(OrderContext context);
    void compensate(OrderContext context);
}

// 库存预留步骤
public class ReserveInventoryStep implements SagaStep {
    @Override
    public void execute(OrderContext context) {
        InventoryService inventoryService = context.getInventoryService();
        boolean success = inventoryService.reserve(
            context.getProductId(),
            context.getQuantity()
        );

        if (!success) {
            throw new RuntimeException("库存不足");
        }

        context.setInventoryReserved(true);
    }

    @Override
    public void compensate(OrderContext context) {
        if (context.isInventoryReserved()) {
            InventoryService inventoryService = context.getInventoryService();
            inventoryService.release(
                context.getProductId(),
                context.getQuantity()
            );
        }
    }
}

// Saga协调器
@Service
public class SagaOrchestrator {
    @Autowired
    private SagaInstanceRepository sagaRepository;

    public void startSaga(String sagaType, Object payload) {
        // 创建Saga实例
        SagaInstance instance = new SagaInstance();
        instance.setSagaType(sagaType);
        instance.setPayload(JsonUtil.toJson(payload));
        instance.setStatus("RUNNING");
        sagaRepository.save(instance);

        // 执行Saga
        try {
            Saga saga = sagaFactory.createSaga(sagaType);
            saga.execute(payload);

            instance.setStatus("COMPLETED");
        } catch (Exception e) {
            instance.setStatus("FAILED");
            instance.setError(e.getMessage());
        }

        sagaRepository.save(instance);
    }
}
```

##### Saga状态机
```java
public enum SagaState {
    STARTED,
    ORDER_CREATED,
    INVENTORY_RESERVED,
    PAYMENT_PROCESSED,
    ORDER_CONFIRMED,
    COMPLETED,
    COMPENSATING,
    COMPENSATED
}

public class SagaStateMachine {
    private SagaState currentState;

    public void transition(SagaEvent event) {
        switch (currentState) {
            case STARTED:
                if (event == SagaEvent.ORDER_CREATED_SUCCESS) {
                    currentState = SagaState.ORDER_CREATED;
                } else if (event == SagaEvent.ORDER_CREATED_FAIL) {
                    currentState = SagaState.COMPENSATED;
                }
                break;

            case ORDER_CREATED:
                if (event == SagaEvent.INVENTORY_RESERVED_SUCCESS) {
                    currentState = SagaState.INVENTORY_RESERVED;
                } else if (event == SagaEvent.INVENTORY_RESERVED_FAIL) {
                    currentState = SagaState.COMPENSATING;
                }
                break;

            // ... 其他状态转换

            case COMPENSATING:
                if (event == SagaEvent.COMPENSATION_COMPLETE) {
                    currentState = SagaState.COMPENSATED;
                }
                break;
        }
    }
}
```

##### 数据库设计
```sql
-- Saga实例表
CREATE TABLE saga_instance (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    saga_id VARCHAR(64) UNIQUE NOT NULL,
    saga_type VARCHAR(128) NOT NULL,
    status VARCHAR(20) NOT NULL, -- RUNNING, COMPLETED, FAILED, COMPENSATING
    payload TEXT,
    current_step INT DEFAULT 0,
    error_message TEXT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_saga_id (saga_id),
    INDEX idx_status (status)
);

-- Saga步骤执行日志
CREATE TABLE saga_step_log (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    saga_id VARCHAR(64) NOT NULL,
    step_name VARCHAR(128) NOT NULL,
    step_type VARCHAR(20) NOT NULL, -- FORWARD, COMPENSATION
    status VARCHAR(20) NOT NULL, -- SUCCESS, FAILED
    input_data TEXT,
    output_data TEXT,
    error_message TEXT,
    execution_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_saga_id (saga_id)
);
```

##### 优缺点
```
优点:
- 流程清晰,易于理解
- 集中管理,便于监控
- 补偿逻辑统一
- 支持复杂流程

缺点:
- 协调器单点故障
- 协调器逻辑复杂
- 服务与协调器耦合

适用场景:
- 复杂业务流程
- 需要集中管理的事务
- 企业级应用
```

### 5. 本地消息表(本地事件表)

#### 工作原理
```
1. 业务操作和消息记录在同一个本地事务中
2. 后台任务定期扫描未发送的消息
3. 发送消息到消息队列
4. 消费者处理消息,确保幂等性
5. 生产者收到确认后标记消息已发送
```

#### 实现示例
```java
// 生产者
@Service
public class OrderService {
    @Transactional
    public void createOrder(Order order) {
        // 1. 插入订单
        orderRepository.insert(order);

        // 2. 插入本地消息(同一事务)
        LocalMessage message = new LocalMessage();
        message.setMessageId(UUID.randomUUID().toString());
        message.setTopic("order-created");
        message.setPayload(JsonUtil.toJson(order));
        message.setStatus("PENDING");
        localMessageRepository.insert(message);
    }
}

// 消息发送任务
@Component
public class MessageSender {
    @Scheduled(fixedDelay = 1000)
    public void sendPendingMessages() {
        // 查询待发送消息
        List<LocalMessage> messages = localMessageRepository.findByStatus("PENDING");

        for (LocalMessage message : messages) {
            try {
                // 发送到消息队列
                mqProducer.send(message.getTopic(), message.getPayload());

                // 更新状态为已发送
                localMessageRepository.updateStatus(
                    message.getMessageId(),
                    "SENT"
                );
            } catch (Exception e) {
                log.error("发送消息失败: {}", message.getMessageId(), e);
            }
        }
    }
}

// 消费者
@Service
public class InventoryService {
    @RabbitListener(queues = "order-created")
    @Transactional
    public void handleOrderCreated(String message) {
        Order order = JsonUtil.fromJson(message, Order.class);

        // 幂等性检查
        if (processedMessageRepository.exists(order.getId())) {
            return; // 已处理,跳过
        }

        // 业务处理
        inventoryService.deduct(order.getProductId(), order.getQuantity());

        // 记录已处理的消息
        processedMessageRepository.insert(order.getId());
    }
}
```

#### 数据库设计
```sql
-- 本地消息表
CREATE TABLE local_message (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    message_id VARCHAR(64) UNIQUE NOT NULL,
    topic VARCHAR(128) NOT NULL,
    payload TEXT NOT NULL,
    status VARCHAR(20) NOT NULL, -- PENDING, SENT, FAILED
    retry_count INT DEFAULT 0,
    next_retry_time TIMESTAMP,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_status (status),
    INDEX idx_next_retry (next_retry_time)
);

-- 已处理消息表(消费者端)
CREATE TABLE processed_message (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    message_id VARCHAR(64) UNIQUE NOT NULL,
    consumer_group VARCHAR(128) NOT NULL,
    process_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_message (message_id, consumer_group)
);
```

#### 优缺点
```
优点:
- 最终一致性保证
- 实现简单
- 不依赖复杂的框架
- 容易理解和维护

缺点:
- 消息延迟(定时任务扫描)
- 消息表需要定期清理
- 数据库压力

适用场景:
- 对实时性要求不高
- 需要可靠消息传递
- 中小型系统
```

### 6. 事务消息(RocketMQ)

#### 工作原理
```
1. 发送半消息(Half Message,不可消费)
2. 执行本地事务
3. 提交或回滚消息
   - 提交: 消息变为可消费
   - 回滚: 删除消息
4. 如果未收到提交/回滚,MQ回查事务状态
```

#### 实现示例
```java
// 生产者
@Service
public class OrderService {
    @Autowired
    private RocketMQTemplate rocketMQTemplate;

    public void createOrder(Order order) {
        // 构建消息
        Message<Order> message = MessageBuilder.withPayload(order).build();

        // 发送事务消息
        rocketMQTemplate.sendMessageInTransaction(
            "order-group",
            "order-created",
            message,
            order
        );
    }
}

// 事务监听器
@RocketMQTransactionListener(rocketMQTemplateBeanName = "rocketMQTemplate")
public class OrderTransactionListener implements RocketMQLocalTransactionListener {
    @Override
    @Transactional
    public RocketMQLocalTransactionState executeLocalTransaction(Message msg, Object arg) {
        try {
            Order order = (Order) arg;

            // 执行本地事务
            orderRepository.insert(order);

            // 返回提交状态
            return RocketMQLocalTransactionState.COMMIT;

        } catch (Exception e) {
            // 返回回滚状态
            return RocketMQLocalTransactionState.ROLLBACK;
        }
    }

    @Override
    public RocketMQLocalTransactionState checkLocalTransaction(Message msg) {
        // 回查本地事务状态
        Order order = JsonUtil.fromJson(
            new String((byte[]) msg.getPayload()),
            Order.class
        );

        if (orderRepository.exists(order.getId())) {
            return RocketMQLocalTransactionState.COMMIT;
        } else {
            return RocketMQLocalTransactionState.ROLLBACK;
        }
    }
}

// 消费者
@Service
@RocketMQMessageListener(
    topic = "order-created",
    consumerGroup = "inventory-group"
)
public class InventoryConsumer implements RocketMQListener<Order> {
    @Override
    @Transactional
    public void onMessage(Order order) {
        // 幂等性检查
        if (processedOrderRepository.exists(order.getId())) {
            return;
        }

        // 扣减库存
        inventoryService.deduct(order.getProductId(), order.getQuantity());

        // 记录已处理
        processedOrderRepository.insert(order.getId());
    }
}
```

#### 优缺点
```
优点:
- 最终一致性
- 消息零丢失
- 实时性高
- 解耦业务与消息

缺点:
- 依赖特定MQ(RocketMQ)
- 实现相对复杂
- 需要回查接口

适用场景:
- 高可靠消息传递
- 订单、支付等核心业务
- 使用RocketMQ的系统
```

## 分布式事务框架

### 1. Seata

#### 架构
```
TC (Transaction Coordinator): 事务协调器
TM (Transaction Manager): 事务管理器
RM (Resource Manager): 资源管理器

工作流程:
1. TM向TC申请开启全局事务
2. TC返回全局事务ID(XID)
3. RM向TC注册分支事务
4. TM通知TC提交/回滚全局事务
5. TC协调所有分支事务提交/回滚
```

#### AT模式
```java
// 开启全局事务
@GlobalTransactional
public void createOrder(OrderRequest request) {
    // 1. 创建订单
    orderService.createOrder(request);

    // 2. 扣减库存
    inventoryService.deduct(request.getProductId(), request.getQuantity());

    // 3. 扣减余额
    accountService.debit(request.getUserId(), request.getAmount());
}

// 配置
@SpringBootApplication
@EnableAutoDataSourceProxy
public class Application {
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}
```

```
优点:
- 无侵入,自动补偿
- 使用简单
- 性能好

缺点:
- 需要数据库支持
- 存在脏回滚风险
- 全局锁可能影响并发

适用场景:
- 大多数业务场景
- Java应用
- 快速迁移
```

#### TCC模式
```java
@LocalTCC
public interface InventoryService {
    @TwoPhaseBusinessAction(
        name = "prepareDeduct",
        commitMethod = "commit",
        rollbackMethod = "rollback"
    )
    boolean prepareDeduct(
        @BusinessActionContextParameter(paramName = "productId") String productId,
        @BusinessActionContextParameter(paramName = "count") int count
    );

    boolean commit(BusinessActionContext context);

    boolean rollback(BusinessActionContext context);
}

// 使用
@GlobalTransactional
public void placeOrder(String productId, int count) {
    inventoryService.prepareDeduct(productId, count);
    // ...
}
```

#### Saga模式
```yaml
# saga定义
type: "saga"
name: "create-order-saga"
steps:
  - name: "create-order"
    service: "order-service"
    transaction:
      path: "/orders"
      method: "POST"
    compensation:
      path: "/orders/{orderId}"
      method: "DELETE"

  - name: "reserve-inventory"
    service: "inventory-service"
    transaction:
      path: "/inventory/reserve"
      method: "POST"
    compensation:
      path: "/inventory/release"
      method: "POST"

  - name: "process-payment"
    service: "payment-service"
    transaction:
      path: "/payments"
      method: "POST"
    compensation:
      path: "/payments/{paymentId}/refund"
      method: "POST"
```

### 2. DTM

#### 特点
```
- 跨语言支持(Go, Java, Python, PHP等)
- 支持多种模式(Saga, TCC, 2PC)
- 轻量级,易部署
- HTTP/gRPC协议
```

#### Saga示例
```go
// Go示例
func main() {
    // 创建Saga
    saga := dtmcli.NewSaga("http://localhost:36789/api/dtmsvr", "").
        Add("http://localhost:8080/order/create", "http://localhost:8080/order/cancel", &OrderReq{}).
        Add("http://localhost:8080/inventory/deduct", "http://localhost:8080/inventory/restore", &InventoryReq{}).
        Add("http://localhost:8080/payment/process", "http://localhost:8080/payment/refund", &PaymentReq{})

    // 提交Saga
    err := saga.Submit()
    if err != nil {
        panic(err)
    }
}
```

## 选型建议

### 场景对比

```
强一致性要求:
- 2PC/3PC
- Seata AT模式
适用: 金融核心、账户余额

最终一致性:
- TCC
- Saga
- 本地消息表
- 事务消息
适用: 电商订单、库存、积分

简单场景:
- 本地消息表
适用: 非核心业务,流量小

复杂业务流程:
- Saga编制式
适用: 多步骤,有补偿逻辑

高性能:
- TCC
- Saga
适用: 高并发,短事务
```

### 技术栈匹配

```
Java生态:
- Seata(AT/TCC/Saga)
- RocketMQ事务消息
- 本地消息表

Go生态:
- DTM
- 本地消息表

混合技术栈:
- DTM(跨语言)
- HTTP API + 消息队列
```

## 最佳实践

### 1. 幂等性设计
```
唯一ID:
- 业务唯一键(订单号)
- 幂等令牌(Idempotency Key)

实现:
- 数据库唯一索引
- Redis SETNX
- 状态机检查
```

### 2. 补偿事务设计
```
原则:
- 补偿操作必须成功
- 补偿操作可重试
- 补偿操作幂等

实现:
- 补偿失败记录日志
- 人工干预机制
- 定期重试补偿
```

### 3. 事务隔离
```
读已提交:
- 大多数场景适用

串行化:
- 金融核心业务
- 需要防止脏读、不可重复读

实现:
- 数据库隔离级别
- 分布式锁
```

### 4. 超时与重试
```
超时设置:
- 根据业务RT设置合理超时
- 避免过长导致资源占用

重试策略:
- 指数退避
- 最大重试次数
- 死信队列
```

### 5. 监控与告警
```
监控指标:
- 事务成功率
- 事务耗时(P99)
- 补偿次数
- 失败原因分布

告警:
- 事务失败率超过阈值
- 补偿失败
- 长时间未完成的事务
```

## 常见问题

### 1. 事务超时
```
原因:
- 网络延迟
- 服务响应慢
- 数据库锁等待

解决:
- 设置合理超时时间
- 优化慢查询
- 使用异步处理
```

### 2. 数据不一致
```
原因:
- 补偿失败
- 并发冲突
- 网络分区

解决:
- 补偿重试机制
- 对账脚本
- 人工修复工具
```

### 3. 性能问题
```
原因:
- 全局锁竞争
- 长事务
- 同步调用链长

解决:
- 缩短事务范围
- 异步化处理
- 优化热点数据
```

## 参考资源

### 开源框架
- Seata: https://seata.io/
- DTM: https://en.dtm.pub/
- RocketMQ: https://rocketmq.apache.org/

### 学习资料
- 《分布式系统原理》
- 《微服务架构设计模式》
- Martin Fowler: Sagas论文
- Google Spanner论文
