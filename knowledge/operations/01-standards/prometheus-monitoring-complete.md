---
id: prometheus-monitoring-complete
title: Prometheus监控完整指南
domain: operations
category: 01-standards
difficulty: intermediate
tags: [complete, grafana仪表盘, kubernetes监控, monitoring, operations, prometheus, prometheus核心概念, promql查询语言]
quality_score: 70
last_updated: 2026-06-15
---
# Prometheus监控完整指南

## 概述

可观测性(Observability)是现代分布式系统运维的核心能力,由三大支柱构成:

1. **指标(Metrics)** — 可聚合的数值型时间序列数据,回答"系统状态如何"
2. **日志(Logs)** — 离散事件记录,回答"发生了什么"
3. **追踪(Traces)** — 跨服务请求链路,回答"请求经过了哪些路径"

Prometheus专注于指标采集与查询,是CNCF毕业项目,已成为云原生监控事实标准。它采用拉取(Pull)模型主动抓取目标暴露的指标端点,配合Alertmanager实现告警,配合Grafana实现可视化。

### 核心架构

```
┌──────────────┐     scrape      ┌──────────────┐
│  应用/Exporter │ ◄──────────── │  Prometheus   │
│  /metrics端点  │               │  Server       │
└──────────────┘               │  - TSDB       │
                                │  - PromQL     │
┌──────────────┐     scrape     │  - Rules      │
│  Node Exporter │ ◄──────────── │              │
└──────────────┘               └──────┬───────┘
                                       │
                          ┌────────────┼────────────┐
                          ▼            ▼            ▼
                   ┌───────────┐ ┌──────────┐ ┌──────────────┐
                   │Alertmanager│ │ Grafana  │ │ Remote Write │
                   │ 告警路由   │ │ 可视化   │ │ 远程存储     │
                   └───────────┘ └──────────┘ └──────────────┘
```

---

## Prometheus核心概念

### 1. 指标类型(Metric Types)

#### Counter(计数器)
单调递增值,只能增加或重置为零。适用于请求总数、错误总数、处理字节数等。

```promql
# 指标示例
http_requests_total{method="GET", handler="/api/users", status="200"} 1027
http_requests_total{method="POST", handler="/api/users", status="201"} 83

# 计算速率(每秒请求数)
rate(http_requests_total[5m])

# 计算增量
increase(http_requests_total[1h])
```

#### Gauge(仪表盘)
可任意增减的瞬时值。适用于温度、内存使用量、当前连接数、队列深度等。

```promql
# 指标示例
node_memory_AvailableBytes 4294967296
go_goroutines 42
queue_depth{queue="orders"} 156

# 直接查询当前值
node_memory_AvailableBytes

# 计算变化趋势
delta(node_memory_AvailableBytes[1h])
deriv(node_memory_AvailableBytes[1h])
```

#### Histogram(直方图)
将观测值分布到可配置的桶(bucket)中,同时记录总和与计数。适用于请求延迟、响应大小等需要分位数计算的场景。

```promql
# 指标示例(自动生成三组时间序列)
http_request_duration_seconds_bucket{le="0.005"} 24054
http_request_duration_seconds_bucket{le="0.01"}  33444
http_request_duration_seconds_bucket{le="0.025"} 100392
http_request_duration_seconds_bucket{le="0.05"}  129389
http_request_duration_seconds_bucket{le="0.1"}   133988
http_request_duration_seconds_bucket{le="+Inf"}  144320
http_request_duration_seconds_sum 53.2
http_request_duration_seconds_count 144320

# 计算P99延迟
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))

# 计算平均延迟
rate(http_request_duration_seconds_sum[5m]) / rate(http_request_duration_seconds_count[5m])
```

#### Summary(摘要)
在客户端直接计算分位数,无法跨实例聚合。除非有特殊需求,优先使用Histogram。

```promql
# 指标示例
rpc_duration_seconds{quantile="0.5"} 0.023
rpc_duration_seconds{quantile="0.9"} 0.056
rpc_duration_seconds{quantile="0.99"} 0.148
rpc_duration_seconds_sum 1.7560473e+04
rpc_duration_seconds_count 2693
```

### 2. 标签(Labels)

标签是Prometheus的核心维度建模机制,每一组唯一标签组合构成一条独立的时间序列。

```yaml
# 好的标签设计 — 低基数、高区分度
http_requests_total{method="GET", status="200", service="user-api"}
http_requests_total{method="POST", status="500", service="user-api"}

# 坏的标签设计 — 高基数,导致时间序列爆炸
http_requests_total{user_id="abc123"}    # 用户ID作标签 → 百万级序列
http_requests_total{request_id="..."}     # 请求ID作标签 → 无限序列
http_requests_total{ip="10.0.0.1"}        # IP地址作标签 → 高基数
```

**标签基数控制原则:**
- 标签值的势(cardinality)应控制在数百以内
- 避免将用户ID、IP地址、请求ID、trace ID等放入标签
- 使用日志或追踪系统处理高基数维度

### 3. Scrape配置

```yaml
# prometheus.yml
global:
  scrape_interval: 15s          # 全局抓取间隔
  evaluation_interval: 15s      # 规则评估间隔
  scrape_timeout: 10s           # 抓取超时

# 抓取目标配置
scrape_configs:
  # 静态目标
  - job_name: "web-api"
    metrics_path: /metrics       # 默认 /metrics
    scheme: https                # 默认 http
    static_configs:
      - targets:
          - "api-server-1:8080"
          - "api-server-2:8080"
        labels:
          env: production
          team: backend

  # 带认证的目标
  - job_name: "secure-service"
    bearer_token_file: /etc/prometheus/token
    tls_config:
      ca_file: /etc/prometheus/ca.pem
      insecure_skip_verify: false
    static_configs:
      - targets: ["secure-svc:9090"]

  # 指标重标记(relabeling)
  - job_name: "node-exporter"
    static_configs:
      - targets: ["node1:9100", "node2:9100"]
    metric_relabel_configs:
      - source_labels: [__name__]
        regex: "node_cpu_seconds_total"
        action: keep
      - source_labels: [mode]
        regex: "idle"
        action: drop
```

### 4. 服务发现(Service Discovery)

```yaml
scrape_configs:
  # Kubernetes服务发现
  - job_name: "kubernetes-pods"
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names: ["production", "staging"]
    relabel_configs:
      # 仅抓取有prometheus.io/scrape注解的Pod
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      # 使用注解覆盖端口
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        target_label: __address__
        regex: (.+)
        replacement: $1
      # 保留Pod标签
      - source_labels: [__meta_kubernetes_namespace]
        target_label: namespace
      - source_labels: [__meta_kubernetes_pod_name]
        target_label: pod

  # Consul服务发现
  - job_name: "consul-services"
    consul_sd_configs:
      - server: "consul.service.consul:8500"
        services: ["web", "api", "worker"]
    relabel_configs:
      - source_labels: [__meta_consul_tags]
        regex: ".*,monitor,.*"
        action: keep

  # DNS服务发现
  - job_name: "dns-services"
    dns_sd_configs:
      - names: ["_prometheus._tcp.example.com"]
        type: SRV
        refresh_interval: 30s

  # 文件服务发现(适合动态环境)
  - job_name: "file-sd"
    file_sd_configs:
      - files:
          - "/etc/prometheus/targets/*.json"
        refresh_interval: 5m
```

---

## PromQL查询语言

### 1. 数据类型

| 类型 | 说明 | 示例 |
|------|------|------|
| 即时向量(Instant Vector) | 每条序列的单个最新样本 | `http_requests_total` |
| 范围向量(Range Vector) | 每条序列的一段时间范围样本 | `http_requests_total[5m]` |
| 标量(Scalar) | 单个浮点数值 | `42`, `3.14` |
| 字符串(String) | 字符串值(极少用) | `"hello"` |

### 2. 选择器与匹配器

```promql
# 精确匹配
http_requests_total{method="GET"}

# 不等于
http_requests_total{status!="200"}

# 正则匹配
http_requests_total{handler=~"/api/.*"}

# 正则不匹配
http_requests_total{handler!~"/health|/ready"}

# 组合匹配
http_requests_total{method="GET", status=~"5..", service="user-api"}
```

### 3. 范围向量与偏移

```promql
# 最近5分钟的样本
http_requests_total[5m]

# 1小时前的即时值
http_requests_total offset 1h

# 1小时前的5分钟范围
http_requests_total[5m] offset 1h

# 支持的时间单位: ms s m h d w y
```

### 4. 聚合运算符

```promql
# 求和(按维度聚合)
sum(rate(http_requests_total[5m])) by (service)

# 平均值
avg(node_cpu_seconds_total{mode="idle"}) by (instance)

# 最大/最小值
max(node_memory_AvailableBytes) by (instance)
min(node_filesystem_avail_bytes) by (mountpoint)

# 计数
count(up == 1) by (job)

# 标准差
stddev(rate(http_request_duration_seconds_sum[5m]))

# 分位数
quantile(0.95, rate(http_request_duration_seconds_sum[5m]))

# Top-K / Bottom-K
topk(5, rate(http_requests_total[5m]))
bottomk(3, node_filesystem_avail_bytes)

# without — 排除指定维度后聚合
sum without (instance) (rate(http_requests_total[5m]))

# count_values — 统计各值出现的次数
count_values("version", build_info)
```

### 5. 二元运算符

```promql
# 算术运算
node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes
(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100

# 比较运算(过滤)
http_requests_total > 1000
node_filesystem_avail_bytes / node_filesystem_size_bytes < 0.1

# 比较运算(布尔模式,返回0或1)
http_requests_total > bool 1000

# 向量匹配
# one-to-one
method:http_requests:rate5m{method="GET"} / ignoring(method) group_left sum(method:http_requests:rate5m)

# many-to-one
node_cpu_seconds_total * on(instance) group_left(nodename) node_uname_info
```

### 6. 常用函数

```promql
# 速率计算(Counter必用)
rate(http_requests_total[5m])          # 每秒平均速率,自动处理重置
irate(http_requests_total[5m])         # 瞬时速率,取最后两个样本

# 增量
increase(http_requests_total[1h])      # 1小时内增加量
delta(temperature_celsius[1h])         # Gauge的变化量(可负)

# 直方图分位数
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# 预测
predict_linear(node_filesystem_avail_bytes[6h], 24*3600)  # 预测24小时后的磁盘可用空间

# 缺失检测
absent(up{job="api"})                  # 目标不存在时返回1
absent_over_time(up{job="api"}[5m])    # 5分钟内无数据时返回1

# 时间函数
time()                                 # 当前Unix时间戳
timestamp(up)                          # 样本的时间戳
day_of_week()                          # 星期几(0=Sunday)
hour()                                 # 当前小时

# 标签操作
label_replace(up, "host", "$1", "instance", "(.+):.+")
label_join(up, "full_name", "-", "job", "instance")

# 排序
sort(node_memory_AvailableBytes)
sort_desc(rate(http_requests_total[5m]))

# 截断
clamp(cpu_usage, 0, 100)
clamp_min(value, 0)
clamp_max(value, 100)

# 聚合窗口
avg_over_time(node_cpu_seconds_total[5m])
max_over_time(node_memory_AvailableBytes[1h])
min_over_time(node_filesystem_avail_bytes[1h])
count_over_time(http_requests_total[5m])
```

### 7. 常用查询模板

```promql
# === 错误率 ===
# HTTP 5xx错误率
sum(rate(http_requests_total{status=~"5.."}[5m]))
  / sum(rate(http_requests_total[5m])) * 100

# 按服务的错误率
sum by (service) (rate(http_requests_total{status=~"5.."}[5m]))
  / sum by (service) (rate(http_requests_total[5m])) * 100

# === 延迟分位数 ===
# P50 / P90 / P99延迟
histogram_quantile(0.50, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))
histogram_quantile(0.90, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))
histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))

# 按服务的P99延迟
histogram_quantile(0.99, sum by (le, service) (rate(http_request_duration_seconds_bucket[5m])))

# === 资源使用率 ===
# CPU使用率(%)
100 - (avg by (instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# 内存使用率(%)
(1 - node_memory_AvailableBytes / node_memory_MemTotal_bytes) * 100

# 磁盘使用率(%)
(1 - node_filesystem_avail_bytes{fstype!~"tmpfs|overlay"}
     / node_filesystem_size_bytes) * 100

# 磁盘IO利用率
rate(node_disk_io_time_seconds_total[5m]) * 100

# === 吞吐量 ===
# 每秒请求数(QPS)
sum(rate(http_requests_total[5m]))

# 按接口的QPS
sum by (handler) (rate(http_requests_total[5m]))

# 网络吞吐(MB/s)
rate(node_network_receive_bytes_total{device!="lo"}[5m]) / 1024 / 1024
rate(node_network_transmit_bytes_total{device!="lo"}[5m]) / 1024 / 1024

# === 饱和度 ===
# Go协程数量
go_goroutines{job="api"}

# 文件描述符使用率
process_open_fds / process_max_fds * 100

# 连接池使用率
db_pool_active_connections / db_pool_max_connections * 100
```

---

## 告警规则

### 1. 告警规则定义

```yaml
# rules/application-alerts.yml
groups:
  - name: application.rules
    interval: 30s    # 评估间隔(可选,默认使用全局值)
    rules:
      # 高错误率
      - alert: HighErrorRate
        expr: |
          sum by (service) (rate(http_requests_total{status=~"5.."}[5m]))
          / sum by (service) (rate(http_requests_total[5m])) > 0.05
        for: 5m        # 持续5分钟才触发
        labels:
          severity: critical
          team: backend
        annotations:
          summary: "服务 {{ $labels.service }} 错误率过高"
          description: "错误率已达 {{ $value | humanizePercentage }},超过5%阈值,持续5分钟"
          runbook_url: "https://wiki.example.com/runbooks/high-error-rate"
          dashboard: "https://grafana.example.com/d/svc-overview?var-service={{ $labels.service }}"

      # 高延迟
      - alert: HighLatencyP99
        expr: |
          histogram_quantile(0.99, sum by (le, service)
            (rate(http_request_duration_seconds_bucket[5m]))) > 1.0
        for: 10m
        labels:
          severity: warning
          team: backend
        annotations:
          summary: "服务 {{ $labels.service }} P99延迟过高"
          description: "P99延迟为 {{ $value | humanizeDuration }},超过1秒阈值"

      # 实例宕机
      - alert: InstanceDown
        expr: up == 0
        for: 3m
        labels:
          severity: critical
        annotations:
          summary: "实例 {{ $labels.instance }} 不可达"
          description: "任务 {{ $labels.job }} 的实例 {{ $labels.instance }} 已宕机超过3分钟"

  - name: infrastructure.rules
    rules:
      # 磁盘空间预警
      - alert: DiskSpaceRunningOut
        expr: |
          predict_linear(node_filesystem_avail_bytes{fstype!~"tmpfs|overlay"}[6h], 24*3600) < 0
        for: 30m
        labels:
          severity: warning
        annotations:
          summary: "主机 {{ $labels.instance }} 磁盘空间即将耗尽"
          description: "挂载点 {{ $labels.mountpoint }} 预计24小时内磁盘空间耗尽,当前可用 {{ $value | humanize1024 }}B"

      # 内存使用率过高
      - alert: HighMemoryUsage
        expr: (1 - node_memory_AvailableBytes / node_memory_MemTotal_bytes) > 0.9
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "主机 {{ $labels.instance }} 内存使用率超过90%"

      # CPU使用率持续过高
      - alert: HighCPUUsage
        expr: |
          100 - (avg by (instance) (irate(node_cpu_seconds_total{mode="idle"}[10m])) * 100) > 85
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "主机 {{ $labels.instance }} CPU使用率持续超过85%"

      # 目标抓取失败
      - alert: PrometheusTargetMissing
        expr: up == 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Prometheus抓取目标丢失: {{ $labels.job }}/{{ $labels.instance }}"
```

### 2. Alertmanager配置

```yaml
# alertmanager.yml
global:
  resolve_timeout: 5m
  smtp_smarthost: "smtp.example.com:587"
  smtp_from: "alertmanager@example.com"
  smtp_auth_username: "alertmanager"
  smtp_auth_password_file: /etc/alertmanager/smtp_password
  slack_api_url_file: /etc/alertmanager/slack_webhook

# 路由树
route:
  receiver: "default-slack"
  group_by: ["alertname", "service", "namespace"]
  group_wait: 30s           # 同组告警等待聚合的时间
  group_interval: 5m        # 同组已发送后再次发送的间隔
  repeat_interval: 4h       # 未恢复告警重复通知的间隔

  routes:
    # 关键告警 → PagerDuty
    - match:
        severity: critical
      receiver: "pagerduty-critical"
      group_wait: 10s
      repeat_interval: 1h
      continue: false

    # 警告级别 → Slack
    - match:
        severity: warning
      receiver: "team-slack"
      group_wait: 1m
      repeat_interval: 8h

    # 按团队路由
    - match_re:
        team: "frontend|mobile"
      receiver: "frontend-slack"
    - match:
        team: backend
      receiver: "backend-slack"

# 抑制规则
inhibit_rules:
  # critical触发时抑制同服务的warning
  - source_match:
      severity: critical
    target_match:
      severity: warning
    equal: ["alertname", "service"]

  # 集群级告警抑制节点级告警
  - source_match:
      scope: cluster
    target_match:
      scope: node
    equal: ["cluster"]

# 接收器
receivers:
  - name: "default-slack"
    slack_configs:
      - channel: "#alerts-default"
        title: '[{{ .Status | toUpper }}] {{ .CommonLabels.alertname }}'
        text: >-
          *摘要:* {{ .CommonAnnotations.summary }}
          *描述:* {{ .CommonAnnotations.description }}
          *详情:*
          {{ range .Alerts }}
            - *{{ .Labels.instance }}*: {{ .Annotations.description }}
          {{ end }}
        send_resolved: true

  - name: "team-slack"
    slack_configs:
      - channel: "#alerts-team"
        send_resolved: true

  - name: "frontend-slack"
    slack_configs:
      - channel: "#alerts-frontend"
        send_resolved: true

  - name: "backend-slack"
    slack_configs:
      - channel: "#alerts-backend"
        send_resolved: true

  - name: "pagerduty-critical"
    pagerduty_configs:
      - routing_key_file: /etc/alertmanager/pagerduty_key
        severity: critical
        description: '{{ .CommonAnnotations.summary }}'
        details:
          firing: '{{ .Alerts.Firing | len }}'
          resolved: '{{ .Alerts.Resolved | len }}'
          dashboard: '{{ (index .Alerts 0).Annotations.dashboard }}'

  - name: "webhook-custom"
    webhook_configs:
      - url: "https://hooks.example.com/alertmanager"
        send_resolved: true
        max_alerts: 10
        http_config:
          bearer_token_file: /etc/alertmanager/webhook_token
```

### 3. 静默与维护窗口

```bash
# 创建静默(通过amtool)
amtool silence add \
  --alertmanager.url=http://localhost:9093 \
  --author="ops-team" \
  --comment="计划维护窗口 2026-03-28 22:00-02:00" \
  --duration=4h \
  alertname="InstanceDown" instance=~"node-[12].*"

# 查看当前静默
amtool silence query --alertmanager.url=http://localhost:9093

# 取消静默
amtool silence expire <silence-id> --alertmanager.url=http://localhost:9093
```

---

## Grafana仪表盘

### 1. 数据源配置

```yaml
# grafana/provisioning/datasources/prometheus.yml
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
    jsonData:
      timeInterval: "15s"        # 与scrape_interval对齐
      httpMethod: POST           # 大查询用POST避免URL长度限制
      exemplarTraceIdDestinations:
        - name: traceID
          datasourceUid: tempo
          urlDisplayLabel: "View in Tempo"
```

### 2. 面板类型与适用场景

| 面板类型 | 适用场景 | 典型查询 |
|---------|---------|---------|
| Time Series | 趋势变化 | `rate(http_requests_total[5m])` |
| Stat | 单值显示 | `sum(up{job="api"})` |
| Gauge | 百分比/阈值 | `(1 - node_memory_AvailableBytes/node_memory_MemTotal_bytes)*100` |
| Bar Chart | 维度比较 | `topk(10, sum by (handler)(rate(http_requests_total[5m])))` |
| Table | 多维明细 | 多指标联合查询 |
| Heatmap | 延迟分布 | `rate(http_request_duration_seconds_bucket[5m])` |
| Logs | 日志面板 | 配合Loki数据源 |
| Node Graph | 拓扑关系 | 配合Tempo数据源 |

### 3. 模板变量

```
# 变量定义(在仪表盘Settings > Variables中配置)

# 数据源变量
Name: datasource
Type: Datasource
Query: prometheus

# 标签值变量
Name: namespace
Type: Query
Query: label_values(kube_pod_info, namespace)
Multi-value: true
Include All: true

# 依赖变量(级联)
Name: service
Type: Query
Query: label_values(kube_pod_info{namespace="$namespace"}, pod)

# 间隔变量
Name: interval
Type: Interval
Values: 1m,5m,10m,30m,1h
Auto: true
Min interval: $__rate_interval

# 在面板查询中使用
sum by (pod) (rate(http_requests_total{namespace="$namespace"}[$interval]))
```

### 4. RED方法仪表盘(面向服务)

RED方法关注三个维度:Rate(速率)、Errors(错误)、Duration(延迟)。

```promql
# --- Rate(请求速率) ---
# 总QPS
sum(rate(http_requests_total{service="$service"}[5m]))
# 按状态码的QPS
sum by (status) (rate(http_requests_total{service="$service"}[5m]))

# --- Errors(错误率) ---
# 错误率百分比
sum(rate(http_requests_total{service="$service", status=~"5.."}[5m]))
/ sum(rate(http_requests_total{service="$service"}[5m])) * 100

# --- Duration(延迟分布) ---
# P50 / P90 / P99
histogram_quantile(0.50, sum by (le) (rate(http_request_duration_seconds_bucket{service="$service"}[5m])))
histogram_quantile(0.90, sum by (le) (rate(http_request_duration_seconds_bucket{service="$service"}[5m])))
histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket{service="$service"}[5m])))
```

### 5. USE方法仪表盘(面向资源)

USE方法关注:Utilization(利用率)、Saturation(饱和度)、Errors(错误数)。

```promql
# --- CPU ---
# Utilization: CPU使用率
100 - avg by (instance) (irate(node_cpu_seconds_total{mode="idle", instance="$instance"}[5m])) * 100
# Saturation: CPU运行队列长度
node_load1{instance="$instance"} / count without(cpu) (node_cpu_seconds_total{mode="idle", instance="$instance"})

# --- Memory ---
# Utilization: 内存使用率
(1 - node_memory_AvailableBytes{instance="$instance"} / node_memory_MemTotal_bytes{instance="$instance"}) * 100
# Saturation: Swap使用
node_memory_SwapTotal_bytes{instance="$instance"} - node_memory_SwapFree_bytes{instance="$instance"}

# --- Disk ---
# Utilization: 磁盘空间使用率
(1 - node_filesystem_avail_bytes{instance="$instance", fstype!~"tmpfs|overlay"}
     / node_filesystem_size_bytes) * 100
# Saturation: 磁盘IO利用率
rate(node_disk_io_time_seconds_total{instance="$instance"}[5m]) * 100

# --- Network ---
# Utilization: 网络带宽使用
rate(node_network_receive_bytes_total{instance="$instance", device!="lo"}[5m]) * 8
rate(node_network_transmit_bytes_total{instance="$instance", device!="lo"}[5m]) * 8
# Errors: 网络错误
rate(node_network_receive_errs_total{instance="$instance"}[5m])
rate(node_network_transmit_errs_total{instance="$instance"}[5m])
```

### 6. Four Golden Signals仪表盘(Google SRE)

```promql
# 1. Latency(延迟) — 成功请求的延迟 vs 失败请求的延迟
# 成功请求P99
histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket{status!~"5.."}[5m])))
# 失败请求P99
histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket{status=~"5.."}[5m])))

# 2. Traffic(流量) — 系统负载
sum(rate(http_requests_total[5m]))

# 3. Errors(错误) — 失败请求比例
sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))

# 4. Saturation(饱和度) — 最受限资源的利用率
# CPU饱和度
avg(1 - rate(node_cpu_seconds_total{mode="idle"}[5m]))
# 内存饱和度
1 - avg(node_memory_AvailableBytes / node_memory_MemTotal_bytes)
```

---

## 应用埋点

### 1. Python客户端(prometheus_client)

```python
# pip install prometheus-client
from prometheus_client import (
    Counter, Gauge, Histogram, Summary,
    start_http_server, generate_latest, CONTENT_TYPE_LATEST
)
import time

# 定义指标
REQUEST_COUNT = Counter(
    "http_requests_total",
    "Total HTTP requests",
    ["method", "handler", "status"]
)
REQUEST_LATENCY = Histogram(
    "http_request_duration_seconds",
    "HTTP request latency",
    ["method", "handler"],
    buckets=[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
)
IN_PROGRESS = Gauge(
    "http_requests_in_progress",
    "Number of in-progress HTTP requests",
    ["handler"]
)
DB_POOL_SIZE = Gauge(
    "db_connection_pool_size",
    "Database connection pool size",
    ["pool"]
)

# 使用装饰器简化
@REQUEST_LATENCY.labels(method="GET", handler="/api/users").time()
def get_users():
    pass

# 手动埋点
def handle_request(method, handler):
    IN_PROGRESS.labels(handler=handler).inc()
    start = time.time()
    try:
        # ... 业务逻辑
        status = "200"
    except Exception:
        status = "500"
        raise
    finally:
        duration = time.time() - start
        REQUEST_COUNT.labels(method=method, handler=handler, status=status).inc()
        REQUEST_LATENCY.labels(method=method, handler=handler).observe(duration)
        IN_PROGRESS.labels(handler=handler).dec()

# Flask中间件集成
from flask import Flask, request, g
app = Flask(__name__)

@app.before_request
def before_request():
    g.start_time = time.time()

@app.after_request
def after_request(response):
    latency = time.time() - g.start_time
    REQUEST_COUNT.labels(
        method=request.method,
        handler=request.endpoint or "unknown",
        status=response.status_code
    ).inc()
    REQUEST_LATENCY.labels(
        method=request.method,
        handler=request.endpoint or "unknown"
    ).observe(latency)
    return response

@app.route("/metrics")
def metrics():
    return generate_latest(), 200, {"Content-Type": CONTENT_TYPE_LATEST}

# FastAPI集成
from fastapi import FastAPI, Request
from starlette.middleware.base import BaseHTTPMiddleware
from prometheus_client import make_asgi_app

app = FastAPI()

class MetricsMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request: Request, call_next):
        start = time.time()
        response = await call_next(request)
        duration = time.time() - start
        REQUEST_COUNT.labels(
            method=request.method,
            handler=request.url.path,
            status=response.status_code
        ).inc()
        REQUEST_LATENCY.labels(
            method=request.method,
            handler=request.url.path
        ).observe(duration)
        return response

app.add_middleware(MetricsMiddleware)
metrics_app = make_asgi_app()
app.mount("/metrics", metrics_app)

# 独立指标服务器
if __name__ == "__main__":
    start_http_server(8000)  # 在8000端口暴露/metrics
```

### 2. Node.js客户端(prom-client)

```javascript
// npm install prom-client
const client = require("prom-client");

// 启用默认指标(进程级: CPU/内存/GC/事件循环等)
const collectDefaultMetrics = client.collectDefaultMetrics;
collectDefaultMetrics({ prefix: "app_" });

// 自定义指标
const httpRequestsTotal = new client.Counter({
  name: "http_requests_total",
  help: "Total HTTP requests",
  labelNames: ["method", "handler", "status"],
});

const httpRequestDuration = new client.Histogram({
  name: "http_request_duration_seconds",
  help: "HTTP request latency in seconds",
  labelNames: ["method", "handler"],
  buckets: [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10],
});

const activeConnections = new client.Gauge({
  name: "http_active_connections",
  help: "Number of active connections",
});

// Express中间件
const express = require("express");
const app = express();

app.use((req, res, next) => {
  const end = httpRequestDuration.startTimer({
    method: req.method,
    handler: req.route?.path || req.path,
  });
  activeConnections.inc();

  res.on("finish", () => {
    end();
    httpRequestsTotal.inc({
      method: req.method,
      handler: req.route?.path || req.path,
      status: res.statusCode,
    });
    activeConnections.dec();
  });

  next();
});

// 指标端点
app.get("/metrics", async (req, res) => {
  res.set("Content-Type", client.register.contentType);
  res.end(await client.register.metrics());
});
```

### 3. Go客户端(prometheus/client_golang)

```go
package main

import (
    "net/http"
    "time"

    "github.com/prometheus/client_golang/prometheus"
    "github.com/prometheus/client_golang/prometheus/promauto"
    "github.com/prometheus/client_golang/prometheus/promhttp"
)

var (
    httpRequestsTotal = promauto.NewCounterVec(
        prometheus.CounterOpts{
            Name: "http_requests_total",
            Help: "Total HTTP requests",
        },
        []string{"method", "handler", "status"},
    )

    httpRequestDuration = promauto.NewHistogramVec(
        prometheus.HistogramOpts{
            Name:    "http_request_duration_seconds",
            Help:    "HTTP request duration in seconds",
            Buckets: prometheus.DefBuckets,
        },
        []string{"method", "handler"},
    )

    activeRequests = promauto.NewGauge(
        prometheus.GaugeOpts{
            Name: "http_active_requests",
            Help: "Number of active requests",
        },
    )
)

// 中间件
func metricsMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        start := time.Now()
        activeRequests.Inc()
        defer activeRequests.Dec()

        rw := &responseWriter{ResponseWriter: w, statusCode: 200}
        next.ServeHTTP(rw, r)

        duration := time.Since(start).Seconds()
        httpRequestsTotal.WithLabelValues(r.Method, r.URL.Path, http.StatusText(rw.statusCode)).Inc()
        httpRequestDuration.WithLabelValues(r.Method, r.URL.Path).Observe(duration)
    })
}

type responseWriter struct {
    http.ResponseWriter
    statusCode int
}

func (rw *responseWriter) WriteHeader(code int) {
    rw.statusCode = code
    rw.ResponseWriter.WriteHeader(code)
}

func main() {
    mux := http.NewServeMux()
    mux.Handle("/metrics", promhttp.Handler())
    mux.HandleFunc("/api/users", handleUsers)

    server := &http.Server{
        Addr:    ":8080",
        Handler: metricsMiddleware(mux),
    }
    server.ListenAndServe()
}

func handleUsers(w http.ResponseWriter, r *http.Request) {
    w.Write([]byte(`{"users": []}`))
}
```

---

## Kubernetes监控

### 1. 核心组件

```yaml
# kube-prometheus-stack 一键部署(Helm)
# 包含: Prometheus Operator + Prometheus + Alertmanager + Grafana + 预置规则/仪表盘
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm install monitoring prometheus-community/kube-prometheus-stack \
  --namespace monitoring --create-namespace \
  --set prometheus.prometheusSpec.retention=15d \
  --set prometheus.prometheusSpec.resources.requests.memory=2Gi \
  --set prometheus.prometheusSpec.resources.requests.cpu=500m \
  --set alertmanager.alertmanagerSpec.replicas=3
```

### 2. ServiceMonitor与PodMonitor

```yaml
# ServiceMonitor — 通过Service发现并抓取Pod指标
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: user-api-monitor
  namespace: monitoring
  labels:
    release: monitoring    # 必须匹配Prometheus Operator的selector
spec:
  namespaceSelector:
    matchNames: ["production"]
  selector:
    matchLabels:
      app: user-api
  endpoints:
    - port: metrics        # Service中定义的端口名
      interval: 15s
      path: /metrics
      scrapeTimeout: 10s
      metricRelabelings:
        - sourceLabels: [__name__]
          regex: "go_.*"
          action: drop      # 丢弃Go运行时指标以减少存储

---
# PodMonitor — 直接抓取Pod,不需要Service
apiVersion: monitoring.coreos.com/v1
kind: PodMonitor
metadata:
  name: batch-job-monitor
  namespace: monitoring
spec:
  namespaceSelector:
    matchNames: ["batch"]
  selector:
    matchLabels:
      app: batch-worker
  podMetricsEndpoints:
    - port: metrics
      interval: 30s
```

### 3. 关键Kubernetes指标

```promql
# --- kube-state-metrics ---
# Pod状态
kube_pod_status_phase{phase!="Running", phase!="Succeeded"} == 1
# Pod重启次数
rate(kube_pod_container_status_restarts_total[15m]) > 0
# Deployment副本不匹配
kube_deployment_status_replicas_available != kube_deployment_spec_replicas
# HPA当前副本 vs 期望副本
kube_horizontalpodautoscaler_status_current_replicas / kube_horizontalpodautoscaler_spec_max_replicas

# --- node-exporter ---
# 节点CPU/内存/磁盘(前面已列出)

# --- cAdvisor(kubelet内置) ---
# 容器CPU使用率
sum by (pod, container) (rate(container_cpu_usage_seconds_total{container!="POD", container!=""}[5m]))
# 容器内存使用
sum by (pod, container) (container_memory_working_set_bytes{container!="POD", container!=""})
# 容器OOMKill
increase(kube_pod_container_status_last_terminated_reason{reason="OOMKilled"}[1h])
# 容器CPU限流
sum by (pod) (rate(container_cpu_cfs_throttled_periods_total[5m]))
  / sum by (pod) (rate(container_cpu_cfs_periods_total[5m])) * 100

# --- 常用告警规则 ---
# Pod CrashLoopBackOff
kube_pod_container_status_waiting_reason{reason="CrashLoopBackOff"} == 1
# Job失败
kube_job_status_failed > 0
# PVC空间不足
kubelet_volume_stats_available_bytes / kubelet_volume_stats_capacity_bytes < 0.1
# 节点NotReady
kube_node_status_condition{condition="Ready", status="true"} == 0
```

---

## 高可用与长期存储

### 1. Prometheus联邦(Federation)

```yaml
# 全局Prometheus从区域Prometheus采集聚合指标
scrape_configs:
  - job_name: "federate-region-cn"
    honor_labels: true
    metrics_path: /federate
    params:
      "match[]":
        - '{__name__=~"job:.*"}'            # 只采集recording rules产出
        - '{__name__=~"instance:.*"}'
    static_configs:
      - targets: ["prometheus-cn.internal:9090"]
        labels:
          region: cn

  - job_name: "federate-region-us"
    honor_labels: true
    metrics_path: /federate
    params:
      "match[]":
        - '{__name__=~"job:.*"}'
        - '{__name__=~"instance:.*"}'
    static_configs:
      - targets: ["prometheus-us.internal:9090"]
        labels:
          region: us
```

### 2. Thanos架构

```
┌─────────────┐  ┌─────────────┐
│ Prometheus A │  │ Prometheus B │   ← 各集群独立运行
│ + Sidecar   │  │ + Sidecar   │
└──────┬──────┘  └──────┬──────┘
       │                │
       ▼                ▼
  ┌────────────────────────┐
  │     对象存储(S3/GCS)     │   ← 长期存储
  └────────────────────────┘
       ▲                ▲
       │                │
┌──────┴──────┐  ┌──────┴──────┐
│ Thanos Store│  │ Thanos      │
│ Gateway     │  │ Compactor   │   ← 降采样、压缩
└──────┬──────┘  └─────────────┘
       │
┌──────┴──────┐
│ Thanos Query│   ← 统一查询入口,去重
└──────┬──────┘
       │
┌──────┴──────┐
│   Grafana   │
└─────────────┘
```

```yaml
# Thanos Sidecar配置(与Prometheus同Pod)
containers:
  - name: thanos-sidecar
    image: quay.io/thanos/thanos:v0.35.0
    args:
      - sidecar
      - --tsdb.path=/prometheus/data
      - --prometheus.url=http://localhost:9090
      - --objstore.config-file=/etc/thanos/bucket.yml
    volumeMounts:
      - name: prometheus-data
        mountPath: /prometheus/data
      - name: thanos-config
        mountPath: /etc/thanos

# bucket.yml
type: S3
config:
  bucket: thanos-metrics
  endpoint: s3.amazonaws.com
  region: us-east-1
  access_key: ${AWS_ACCESS_KEY_ID}
  secret_key: ${AWS_SECRET_ACCESS_KEY}
```

### 3. VictoriaMetrics(高性能替代方案)

```yaml
# 作为Prometheus远程写入目标
# prometheus.yml
remote_write:
  - url: http://victoriametrics:8428/api/v1/write
    queue_config:
      max_samples_per_send: 10000
      capacity: 100000
      max_shards: 30

# VictoriaMetrics部署(单节点)
docker run -d \
  --name victoriametrics \
  -v vmdata:/victoria-data \
  -p 8428:8428 \
  victoriametrics/victoria-metrics:v1.101.0 \
  -retentionPeriod=90d \
  -storageDataPath=/victoria-data \
  -memory.allowedPercent=60
```

### 4. 远程存储配置

```yaml
# prometheus.yml — 远程读写
remote_write:
  - url: http://remote-storage:9201/write
    remote_timeout: 30s
    queue_config:
      capacity: 50000
      max_shards: 20
      min_shards: 1
      max_samples_per_send: 5000
      batch_send_deadline: 5s
      min_backoff: 30ms
      max_backoff: 5s
    write_relabel_configs:
      - source_labels: [__name__]
        regex: "go_.*"
        action: drop              # 不写入Go运行时指标

remote_read:
  - url: http://remote-storage:9201/read
    read_recent: false            # 本地有的数据不走远程读
    required_matchers:
      job: "important-service"
```

---

## 最佳实践

### 1. 命名规范

```
# 格式: <namespace>_<name>_<unit>
# 使用下划线分隔,全小写

# 好的命名
http_requests_total                     # Counter: _total后缀
http_request_duration_seconds           # Histogram: 使用基本单位(秒而非毫秒)
node_memory_AvailableBytes              # Gauge: 使用基本单位(字节而非MB)
process_cpu_seconds_total               # Counter: CPU秒数
myapp_queue_depth                       # Gauge: 队列深度

# 坏的命名
http_requests                           # 缺少_total后缀
request_latency_ms                      # 不要使用毫秒,用秒
memory_usage_mb                         # 不要使用MB,用字节
HttpRequestCount                        # 不要用驼峰
http.requests.total                     # 不要用点号分隔
```

### 2. Recording Rules(预计算)

```yaml
# rules/recording-rules.yml
groups:
  - name: http_recording_rules
    interval: 30s
    rules:
      # 预计算每秒请求速率
      - record: job:http_requests:rate5m
        expr: sum by (job) (rate(http_requests_total[5m]))

      # 预计算错误率
      - record: job:http_errors:ratio5m
        expr: |
          sum by (job) (rate(http_requests_total{status=~"5.."}[5m]))
          / sum by (job) (rate(http_requests_total[5m]))

      # 预计算延迟分位数
      - record: job:http_request_duration_seconds:p99_5m
        expr: |
          histogram_quantile(0.99,
            sum by (job, le) (rate(http_request_duration_seconds_bucket[5m])))

      # 预计算实例级CPU使用率
      - record: instance:node_cpu:ratio
        expr: |
          1 - avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m]))

  - name: resource_recording_rules
    rules:
      # 节点内存使用率
      - record: instance:node_memory_utilization:ratio
        expr: |
          1 - node_memory_AvailableBytes / node_memory_MemTotal_bytes

      # 容器CPU请求使用率
      - record: namespace:container_cpu_usage:sum
        expr: |
          sum by (namespace) (
            rate(container_cpu_usage_seconds_total{container!="POD", container!=""}[5m])
          )
```

### 3. 性能调优

```yaml
# prometheus.yml全局配置
global:
  scrape_interval: 15s          # 通常15s-60s;不要低于10s
  scrape_timeout: 10s           # 必须小于scrape_interval
  evaluation_interval: 15s

# 存储配置(命令行参数)
# --storage.tsdb.retention.time=15d         # 本地保留15天
# --storage.tsdb.retention.size=50GB        # 或按大小保留
# --storage.tsdb.wal-compression            # 启用WAL压缩
# --storage.tsdb.min-block-duration=2h      # 最小block时长
# --storage.tsdb.max-block-duration=36h     # 最大block时长
# --query.max-concurrency=20               # 最大并发查询数
# --query.timeout=2m                        # 查询超时
# --query.max-samples=50000000             # 单次查询最大样本数
```

**性能关键指标自监控:**

```promql
# Prometheus自身健康
prometheus_tsdb_head_series                         # 活跃时间序列数(核心容量指标)
rate(prometheus_tsdb_head_samples_appended_total[5m]) # 每秒写入样本数
prometheus_tsdb_compactions_failed_total             # 压缩失败次数
prometheus_engine_query_duration_seconds             # 查询耗时
rate(prometheus_target_scrapes_exceeded_sample_limit_total[5m])  # 样本超限
prometheus_tsdb_storage_blocks_bytes                  # 存储块大小
```

### 4. 标签基数控制

```yaml
# 在scrape配置中限制每次抓取的样本数
scrape_configs:
  - job_name: "risky-service"
    sample_limit: 5000            # 超过则整次抓取失败
    target_limit: 100             # 限制目标数
    label_limit: 30               # 标签数量上限
    label_name_length_limit: 200  # 标签名长度上限
    label_value_length_limit: 500 # 标签值长度上限

    metric_relabel_configs:
      # 丢弃高基数指标
      - source_labels: [__name__]
        regex: "expensive_metric_.*"
        action: drop
      # 丢弃特定标签(降低基数)
      - regex: "trace_id|span_id"
        action: labeldrop
```

---

## 常见陷阱

### 1. 高基数标签

**问题:** 将高基数维度(用户ID、IP、请求ID)作为标签,导致时间序列爆炸,内存和存储急剧增长。

```promql
# 诊断: 查看每个指标的序列数
topk(20, count by (__name__) ({__name__!=""}))

# 诊断: 查看每个标签的基数
count(count by (user_id) (http_requests_total))  # 如果返回值很大,说明有问题
```

**解决方案:**
- 移除高基数标签,改用日志或追踪系统
- 使用metric_relabel_configs在采集时丢弃
- 对必要的高基数场景使用recording rules预聚合

### 2. Missing指标(指标缺失)

**问题:** 服务刚启动时,Counter/Histogram尚未被触发过,查询返回空结果,导致告警规则失效。

```python
# Python: 初始化时预设标签组合
REQUEST_COUNT = Counter("http_requests_total", "Total requests", ["method", "status"])
# 启动时初始化所有预期的标签组合
for method in ["GET", "POST", "PUT", "DELETE"]:
    for status in ["200", "400", "404", "500"]:
        REQUEST_COUNT.labels(method=method, status=status)  # 初始值为0
```

```promql
# PromQL: 使用or向量填充
(
  sum by (service) (rate(http_requests_total{status=~"5.."}[5m]))
  / sum by (service) (rate(http_requests_total[5m]))
) or (
  0 * group by (service) (up{job="api"})
)
```

### 3. 告警风暴(Alert Storm)

**问题:** 级联故障导致大量告警同时触发,通知渠道被淹没。

**解决方案:**
- 合理设置`group_by`和`group_wait`,将相关告警聚合
- 使用`inhibit_rules`抑制低级别告警(见Alertmanager配置节)
- 分层告警: 基础设施层 → 平台层 → 应用层,高层告警抑制低层
- 设置合理的`for`持续时间,过滤瞬时抖动
- 使用路由树将不同级别告警发送到不同渠道

### 4. 存储膨胀

**问题:** 指标数量不受控增长,存储成本持续上升。

```promql
# 诊断: 每个job贡献的序列数
count by (job) ({__name__!=""})

# 诊断: 每个指标名贡献的序列数
topk(10, count by (__name__) ({__name__!=""}))

# 诊断: 抓取的样本量
scrape_samples_scraped
```

**解决方案:**
- 定期审计指标,移除不再使用的指标
- 使用`metric_relabel_configs`在采集端丢弃无用指标
- 对高频指标使用recording rules聚合后,丢弃原始细粒度数据
- 合理设置retention(时间或大小)
- 使用Thanos/VictoriaMetrics降采样(downsampling)长期数据

### 5. rate()与irate()误用

**问题:** 对Gauge使用rate(),对需要平滑趋势的Counter使用irate()。

```promql
# 错误: Gauge不应使用rate
rate(node_memory_AvailableBytes[5m])           # 错误
# 正确: Gauge使用delta或deriv
delta(node_memory_AvailableBytes[5m])          # 正确
deriv(node_memory_AvailableBytes[5m])          # 正确

# irate vs rate的选择
rate(http_requests_total[5m])    # 平滑的平均速率,适合告警和趋势
irate(http_requests_total[5m])   # 瞬时速率,适合仪表盘实时展示(但告警中易误报)
```

### 6. histogram_quantile精度陷阱

**问题:** bucket边界设置不合理,导致分位数计算严重偏离真实值。

```python
# 不好: 默认bucket可能不适合你的延迟分布
Histogram("http_duration_seconds", "...", buckets=prometheus_client.DEFAULT_BUCKETS)
# DEFAULT_BUCKETS = (.005, .01, .025, .05, .075, .1, .25, .5, .75, 1.0, 2.5, 5.0, 7.5, 10.0)

# 好: 根据实际SLO和延迟分布定制bucket
Histogram(
    "http_duration_seconds", "...",
    buckets=[0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5]
    # 如果SLO是P99 < 500ms, 在500ms附近需要更密的bucket
)
```

---

## Agent Checklist

以下检查项供Super Dev Agent在监控相关任务中使用:

### 埋点检查
- [ ] 所有HTTP服务已暴露`/metrics`端点
- [ ] 使用了正确的指标类型(Counter用于累计值, Gauge用于瞬时值, Histogram用于分布)
- [ ] 指标命名符合`<namespace>_<name>_<unit>`规范,Counter以`_total`结尾
- [ ] 无高基数标签(用户ID、IP、请求ID等不应作为标签)
- [ ] Histogram bucket边界与SLO对齐,在关键阈值附近有足够精度
- [ ] Counter在服务启动时预初始化所有标签组合,避免指标缺失

### 告警检查
- [ ] 每条告警规则有`for`持续时间(避免瞬时抖动误报)
- [ ] 告警包含`severity`标签,按严重级别路由
- [ ] 告警包含`summary`和`description`注解,附带runbook链接
- [ ] 配置了抑制规则(critical抑制warning,集群级抑制节点级)
- [ ] 告警分组合理,避免告警风暴
- [ ] 已配置发送恢复通知(`send_resolved: true`)

### 基础设施检查
- [ ] Prometheus配置了合理的retention(时间或大小)
- [ ] scrape_interval与Grafana仪表盘的`$__rate_interval`对齐
- [ ] 生产环境Prometheus高可用(至少2副本或使用Thanos/VictoriaMetrics)
- [ ] 使用recording rules预计算高频查询,降低查询延迟
- [ ] 定期审计指标基数,`prometheus_tsdb_head_series`处于合理范围
- [ ] Prometheus自身被监控(元监控),包括抓取延迟、查询耗时、存储使用

### Kubernetes监控检查
- [ ] 部署了kube-state-metrics、node-exporter
- [ ] ServiceMonitor/PodMonitor已创建且selector正确匹配
- [ ] 容器资源指标(CPU/内存/网络/磁盘)已采集
- [ ] Pod重启、CrashLoopBackOff、OOMKilled已配置告警
- [ ] PVC空间和节点磁盘空间有预测性告警(`predict_linear`)

### 仪表盘检查
- [ ] 核心服务有RED方法仪表盘(Rate/Errors/Duration)
- [ ] 基础设施有USE方法仪表盘(Utilization/Saturation/Errors)
- [ ] 仪表盘使用模板变量(namespace、service、instance)支持筛选
- [ ] 关键面板有阈值标记(红/黄/绿)
- [ ] 仪表盘已纳入版本管理(Grafana provisioning或dashboard-as-code)
