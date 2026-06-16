---
title: Serverless 模式作战手册
version: 1.0.0
last_updated: 2025-03-20
owner: platform-team
tags: [serverless, knative, functions, faas]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：Serverless 架构模式作战手册
# 作用：指导 Knative、云函数等 Serverless 实践
# 创建时间：2025-03-20
# 最后修改：2025-03-20

## 目标

建立 Serverless 标准化开发和运维流程，确保：
- 事件驱动架构设计
- 弹性伸缩效率
- 冷启动优化
- 成本可控可观测

## 适用场景

- 事件驱动处理
- API 网关后端
- 定时任务
- 数据处理管道
- 突发流量场景

## 执行清单

### 架构设计

- [ ] 评估是否适合 Serverless（无状态、快速响应）
- [ ] 选择合适的 Serverless 平台
- [ ] 设计事件源和触发器
- [ ] 规划函数粒度
- [ ] 设计冷启动优化策略

### Knative 部署

- [ ] 安装 Knative Serving
- [ ] 安装 Knative Eventing
- [ ] 配置网络层
- [ ] 配置自动伸缩
- [ ] 配置域名和 TLS

### 函数开发

- [ ] 实现函数处理逻辑
- [ ] 配置依赖和构建
- [ ] 设置资源限制
- [ ] 实现健康检查
- [ ] 配置日志和监控

## 核心配置

### 1. Knative Serving 安装

```yaml
# Knative Serving 配置
apiVersion: operator.knative.dev/v1beta1
kind: KnativeServing
metadata:
  name: knative-serving
  namespace: knative-serving
spec:
  version: "1.12.0"
  config:
    network:
      ingress-class: "kourier.ingress.networking.knative.dev"
    autoscaler:
      enable-scale-to-zero: "true"
      scale-to-zero-pod-retention-period: "60s"
      pod-autoscaler-class: "kpa.autoscaling.knative.dev"
    defaults:
      revision-timeout-seconds: "300"
      container-concurrency: "100"
    deployment:
      progress-deadline: "600s"
```

### 2. Knative Service 配置

```yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: api-function
  namespace: production
  labels:
    app: api-function
  annotations:
    # 最小实例数（避免冷启动）
    autoscaling.knative.dev/min-scale: "1"
    # 最大实例数
    autoscaling.knative.dev/max-scale: "100"
    # 目标并发
    autoscaling.knative.dev/target: "80"
    # 容器并发限制
    autoscaling.knative.dev/container-concurrency: "100"
spec:
  template:
    metadata:
      annotations:
        # 修订版本超时
        autoscaling.knative.dev/revision-timeout-seconds: "300"
    spec:
      containerConcurrency: 100
      timeoutSeconds: 300
      containers:
      - image: registry.example.com/api-function:v1.0.0
        ports:
        - containerPort: 8080
        env:
        - name: LOG_LEVEL
          value: "info"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
        resources:
          requests:
            cpu: "100m"
            memory: "128Mi"
          limits:
            cpu: "1000m"
            memory: "512Mi"
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
```

### 3. Knative Eventing 配置

```yaml
# Broker 配置
apiVersion: eventing.knative.dev/v1
kind: Broker
metadata:
  name: default
  namespace: production
spec:
  config:
    apiVersion: v1
    kind: ConfigMap
    name: config-br-default-channel
    namespace: knative-eventing

---
# Trigger 配置
apiVersion: eventing.knative.dev/v1
kind: Trigger
metadata:
  name: order-created-trigger
  namespace: production
spec:
  broker: default
  filter:
    attributes:
      type: order.created
      source: order-service
  subscriber:
    ref:
      apiVersion: serving.knative.dev/v1
      kind: Service
      name: order-processor

---
# 事件源配置
apiVersion: sources.knative.dev/v1
kind: ApiServerSource
metadata:
  name: k8s-events-source
  namespace: production
spec:
  mode: Resource
  resources:
  - apiVersion: v1
    kind: Event
  serviceAccountName: events-sa
  sink:
    ref:
      apiVersion: eventing.knative.dev/v1
      kind: Broker
      name: default
```

### 4. 函数代码示例

```python
# Python 函数示例（Flask）
from flask import Flask, request, jsonify
import logging
import os

app = Flask(__name__)
logger = logging.getLogger(__name__)

@app.route('/health/live')
def liveness():
    return jsonify({"status": "alive"})

@app.route('/health/ready')
def readiness():
    # 检查依赖是否就绪
    return jsonify({"status": "ready"})

@app.route('/', methods=['POST'])
def handle_event():
    """处理 CloudEvents 格式的事件"""
    try:
        # 解析 CloudEvent
        event_data = request.get_json()
        event_type = request.headers.get('Ce-Type')
        event_source = request.headers.get('Ce-Source')

        logger.info(f"Received event: type={event_type}, source={event_source}")

        # 处理事件
        result = process_event(event_type, event_data)

        return jsonify({
            "status": "success",
            "result": result
        })
    except Exception as e:
        logger.exception("Event processing failed")
        return jsonify({
            "status": "error",
            "message": str(e)
        }), 500

def process_event(event_type, data):
    """事件处理逻辑"""
    if event_type == "order.created":
        return process_order(data)
    elif event_type == "user.registered":
        return process_user(data)
    else:
        raise ValueError(f"Unknown event type: {event_type}")

def process_order(order_data):
    """处理订单事件"""
    order_id = order_data.get('order_id')
    # 业务逻辑
    return {"order_id": order_id, "status": "processed"}

def process_user(user_data):
    """处理用户事件"""
    user_id = user_data.get('user_id')
    # 业务逻辑
    return {"user_id": user_id, "status": "processed"}

if __name__ == '__main__':
    port = int(os.environ.get('PORT', 8080))
    app.run(host='0.0.0.0', port=port)
```

### 5. 云函数示例

```yaml
# AWS Lambda 函数配置
# serverless.yml
service: api-function

provider:
  name: aws
  runtime: python3.11
  region: us-east-1
  timeout: 30
  memorySize: 256
  environment:
    LOG_LEVEL: info
    DATABASE_URL: ${ssm:/api-function/database-url}
  iam:
    role:
      statements:
      - Effect: Allow
        Action:
        - dynamodb:Query
        - dynamodb:GetItem
        Resource: arn:aws:dynamodb:*:*:table/orders

functions:
  processOrder:
    handler: handler.process_order
    events:
    - http:
        path: orders
        method: post
        cors: true
    - sqs:
        arn: arn:aws:sqs:*:*:order-queue
        batchSize: 10

  scheduledTask:
    handler: handler.scheduled_task
    events:
    - schedule:
        rate: cron(0 * * * ? *)
        enabled: true
```

```python
# AWS Lambda 函数代码
import json
import logging
import boto3

logger = logging.getLogger()
logger.setLevel(logging.INFO)

dynamodb = boto3.resource('dynamodb')
table = dynamodb.Table('orders')

def process_order(event, context):
    """处理 HTTP 请求或 SQS 消息"""
    try:
        # HTTP 请求
        if 'body' in event:
            body = json.loads(event['body']) if isinstance(event['body'], str) else event['body']
            order_id = body.get('order_id')

            # 处理订单
            result = {
                'order_id': order_id,
                'status': 'processed'
            }

            return {
                'statusCode': 200,
                'body': json.dumps({
                    'status': 'success',
                    'result': result
                })
            }

        # SQS 消息
        elif 'Records' in event:
            for record in event['Records']:
                message = json.loads(record['body'])
                process_order_message(message)

            return {'statusCode': 200}

    except Exception as e:
        logger.exception("Error processing event")
        return {
            'statusCode': 500,
            'body': json.dumps({
                'status': 'error',
                'message': str(e)
            })
        }

def process_order_message(message):
    """处理 SQS 消息"""
    order_id = message.get('order_id')
    logger.info(f"Processing order: {order_id}")

    # 更新数据库
    table.update_item(
        Key={'order_id': order_id},
        UpdateExpression='SET #status = :status',
        ExpressionAttributeNames={'#status': 'status'},
        ExpressionAttributeValues={':status': 'processed'}
    )

def scheduled_task(event, context):
    """定时任务"""
    logger.info("Running scheduled task")

    # 执行定时任务逻辑
    # ...

    return {'status': 'completed'}
```

### 6. 冷启动优化

```yaml
# Knative 配置 - 最小实例保持
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: api-function
  namespace: production
  annotations:
    # 保持至少 1 个实例（避免冷启动）
    autoscaling.knative.dev/min-scale: "1"
    # 缩容到零的等待时间
    autoscaling.knative.dev/scale-to-zero-pod-retention-period: "5m"
spec:
  template:
    spec:
      containers:
      - image: registry.example.com/api-function:v1.0.0
        # 启动探测
        startupProbe:
          httpGet:
            path: /health/startup
            port: 8080
          initialDelaySeconds: 0
          periodSeconds: 1
          failureThreshold: 30
```

### 7. 函数依赖优化

```dockerfile
# 优化镜像大小和启动时间
FROM python:3.11-slim AS builder

WORKDIR /app

# 安装依赖
COPY requirements.txt .
RUN pip install --no-cache-dir --target=/app/deps -r requirements.txt

# 生产镜像
FROM gcr.io/distroless/python3-debian12

WORKDIR /app

# 复制依赖
COPY --from=builder /app/deps /app/deps
COPY . /app

# 设置 Python 路径
ENV PYTHONPATH=/app/deps

# 非特权用户
USER nonroot:nonroot

EXPOSE 8080

CMD ["python", "/app/main.py"]
```

## 最佳实践

### 1. 函数设计原则

```python
# [DONE] 正确：单一职责、快速响应
def handle_event(event, context):
    """处理单一类型事件"""
    # 1. 验证输入
    validate_event(event)

    # 2. 执行业务逻辑
    result = process_business_logic(event)

    # 3. 返回结果
    return result

# [FAIL] 错误：函数过于复杂、长时间运行
def handle_all_events(event, context):
    """处理所有类型事件"""
    # 长时间运行的任务
    # 复杂的业务逻辑
    # 多个外部调用
    # ...
```

### 2. 事件驱动架构

```yaml
# 生产者配置
apiVersion: eventing.knative.dev/v1
kind: Broker
metadata:
  name: events-broker
  namespace: production
---
# 消费者 Trigger
apiVersion: eventing.knative.dev/v1
kind: Trigger
metadata:
  name: order-events-trigger
  namespace: production
spec:
  broker: events-broker
  filter:
    attributes:
      type: order.created
  subscriber:
    ref:
      apiVersion: serving.knative.dev/v1
      kind: Service
      name: order-processor
```

### 3. 错误处理和重试

```yaml
# 死信队列配置
apiVersion: eventing.knative.dev/v1
kind: Trigger
metadata:
  name: order-events-trigger
  namespace: production
spec:
  broker: events-broker
  filter:
    attributes:
      type: order.created
  subscriber:
    ref:
      apiVersion: serving.knative.dev/v1
      kind: Service
      name: order-processor
  delivery:
    retry: 3
    backoffPolicy: exponential
    backoffDelay: "PT1S"
    deadLetterSink:
      ref:
        apiVersion: serving.knative.dev/v1
        kind: Service
        name: dead-letter-handler
```

```python
# 函数错误处理
import logging

logger = logging.getLogger()

def handle_event(event, context):
    try:
        # 处理事件
        result = process_event(event)

        # 记录成功
        logger.info(f"Event processed successfully: {event.get('id')}")

        return result

    except RetryableError as e:
        # 可重试错误
        logger.warning(f"Retryable error: {e}")
        raise  # 重新抛出以触发重试

    except PermanentError as e:
        # 永久性错误
        logger.error(f"Permanent error: {e}")
        # 发送到死信队列或记录
        send_to_dead_letter_queue(event, e)
        return {"status": "failed", "error": str(e)}
```

### 4. 状态管理

```python
# [DONE] 正确：使用外部状态存储
import redis
import json

redis_client = redis.Redis(
    host=os.environ['REDIS_HOST'],
    port=int(os.environ.get('REDIS_PORT', 6379)),
    decode_responses=True
)

def handle_event(event, context):
    # 从外部存储获取状态
    session_id = event.get('session_id')
    session = redis_client.get(f"session:{session_id}")

    if session:
        session_data = json.loads(session)
    else:
        session_data = {}

    # 处理事件
    session_data.update(event)

    # 保存状态
    redis_client.setex(
        f"session:{session_id}",
        3600,  # 1小时过期
        json.dumps(session_data)
    )

    return {"status": "success", "session": session_data}

# [FAIL] 错误：依赖本地状态
# 全局变量在函数实例间不共享
local_cache = {}

def handle_event(event, context):
    # 本地缓存不可靠
    key = event.get('id')
    if key in local_cache:
        return local_cache[key]
    # ...
```

## 反模式

### 禁止操作

```python
# [FAIL] 禁止：长时间运行任务
def handle_event(event, context):
    # 同步等待长时间操作
    time.sleep(300)  # 阻塞 5 分钟
    # 应该使用异步处理或消息队列

# [FAIL] 禁止：阻塞式调用
def handle_event(event, context):
    # 同步调用外部 API
    response = requests.get(url, timeout=300)
    # 应该设置合理超时或使用异步

# [FAIL] 禁止：大内存占用
def handle_event(event, context):
    # 加载大文件到内存
    data = open('large_file.dat').read()
    # 应该使用流式处理

# [FAIL] 禁止：硬编码配置
DATABASE_URL = "postgres://user:pass@host/db"
# 应该使用环境变量

# [FAIL] 禁止：忽略超时
def handle_event(event, context):
    # 无限循环
    while True:
        process_queue()
    # 应该设置超时和退出条件
```

## 实战案例

### 案例 1：图片处理管道

```yaml
# 上传触发处理
apiVersion: sources.knative.dev/v1beta2
kind: ContainerSource
metadata:
  name: s3-event-source
  namespace: production
spec:
  template:
    spec:
      containers:
      - image: gcr.io/knative-releases/knative.dev/eventing/cmd/awssqs
        env:
        - name: AWS_ACCESS_KEY_ID
          valueFrom:
            secretKeyRef:
              name: aws-credentials
              key: access-key-id
        - name: AWS_SECRET_ACCESS_KEY
          valueFrom:
            secretKeyRef:
              name: aws-credentials
              key: secret-access-key
        - name: AWS_REGION
          value: us-east-1
        - name: QUEUE_URL
          value: https://sqs.us-east-1.amazonaws.com/123456789/image-upload-queue
  sink:
    ref:
      apiVersion: eventing.knative.dev/v1
      kind: Broker
      name: default
---
# 图片处理函数
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: image-processor
  namespace: production
spec:
  template:
    spec:
      containers:
      - image: registry.example.com/image-processor:v1.0.0
        env:
        - name: OUTPUT_BUCKET
          value: processed-images
        resources:
          limits:
            cpu: "2000m"
            memory: "2Gi"
```

### 案例 2：定时数据处理

```yaml
# CronJob 源
apiVersion: sources.knative.dev/v1
kind: ApiServerSource
metadata:
  name: cron-events
  namespace: production
spec:
  schedule: "0 */1 * * *"  # 每小时
  sink:
    ref:
      apiVersion: serving.knative.dev/v1
      kind: Service
      name: data-aggregator
---
# 聚合函数
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: data-aggregator
  namespace: production
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/min-scale: "0"  # 允许缩零
        autoscaling.knative.dev/max-scale: "1"  # 单实例
    spec:
      containers:
      - image: registry.example.com/data-aggregator:v1.0.0
        env:
        - name: DB_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
```

## 检查清单

### 设计检查

- [ ] 函数无状态
- [ ] 执行时间 < 5 分钟
- [ ] 内存使用合理（< 512MB）
- [ ] 使用外部状态存储
- [ ] 事件驱动设计
- [ ] 冷启动影响评估

### 实现检查

- [ ] 依赖最小化
- [ ] 错误处理完善
- [ ] 日志记录充分
- [ ] 超时配置合理
- [ ] 环境变量配置
- [ ] 健康检查实现

### 运维检查

- [ ] 监控指标配置
- [ ] 告警规则设置
- [ ] 死信队列配置
- [ ] 成本监控
- [ ] 冷启动优化
- [ ] 安全策略配置

### 安全检查

- [ ] IAM 权限最小化
- [ ] 环境变量加密
- [ ] 输入验证
- [ ] 依赖安全扫描
- [ ] 网络隔离
- [ ] 审计日志

## 参考资料

- [Knative 官方文档](https://knative.dev/docs/)
- [AWS Lambda 最佳实践](https://docs.aws.amazon.com/lambda/latest/dg/best-practices.html)
- [Serverless Framework](https://www.serverless.com/framework/docs/)
- [CloudEvents 规范](https://cloudevents.io/)
- [Serverless 架构模式](https://www.oreilly.com/library/view/serverless-architectures-on/9781491971540/)