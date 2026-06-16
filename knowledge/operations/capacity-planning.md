---
id: capacity-planning
title: capacity-planning
domain: operations
category: capacity-planning.md
difficulty: intermediate
tags: [capacity, operations, planning, 容量规划报告, 容量规划流程, 当前状态, 执行摘要, 核心概念]
quality_score: 70
last_updated: 2026-06-15
---
# 开发：Excellent（11964948@qq.com）
# 功能：容量规划实践指南
# 作用：提供系统容量预测、规划和优化的完整方法论
# 创建时间：2026-03-20
# 最后修改：2026-03-20

## 目标
建立科学的容量规划体系,预测业务增长带来的资源需求,提前规划扩容,避免性能瓶颈和资源浪费,实现成本与性能的最优平衡。

## 适用范围
- 计算资源（CPU、内存、容器/Pod）
- 存储资源（数据库、对象存储、文件系统）
- 网络资源（带宽、连接数、CDN）
- 中间件资源（消息队列、缓存、数据库连接池）

## 核心概念

### 容量规划三要素
1. **需求预测**：基于历史数据和业务增长预测未来负载
2. **资源供给**：评估当前资源容量和可扩展性
3. **供需匹配**：平衡性能需求与成本约束

### 关键指标

#### 1. 利用率（Utilization）
**定义**：资源使用量 / 资源总量

**示例**：
```
CPU 利用率 = 实际 CPU 使用 / 总 CPU 容量
内存利用率 = 实际内存使用 / 总内存容量
存储利用率 = 已用存储空间 / 总存储空间
```

**目标范围**：
- CPU：60-80%（过低浪费,过高风险）
- 内存：70-85%
- 存储：< 80%（预留空间给快照/日志）

#### 2. 饱和度（Saturation）
**定义**：资源排队/等待程度

**示例**：
```
CPU 饱和度 = 负载均值 / CPU 核心数
磁盘 I/O 饱和度 = I/O 等待时间 / 总时间
网络饱和度 = 丢包率 + 重传率
```

**阈值**：
- CPU 负载 < 核心数 * 1.5
- I/O 等待 < 20%
- 丢包率 < 0.1%

#### 3. 性能基线（Performance Baseline）
**定义**：系统在正常负载下的性能指标

**基线内容**：
```yaml
# 订单服务性能基线
qps_baseline: 1000
latency_p95_baseline: 150ms
latency_p99_baseline: 300ms
cpu_usage_baseline: 45%
memory_usage_baseline: 60%
db_connections_baseline: 80
```

#### 4. 峰值因子（Peak Factor）
**定义**：峰值流量 / 平均流量

**示例**：
```
日均 QPS: 1000
峰值 QPS: 3000
峰值因子: 3000 / 1000 = 3x
```

**常见场景**：
- 电商大促：10-100x
- 工作日早晚高峰：2-3x
- 季节性业务：5-10x

### 扩容策略

#### 1. 垂直扩容（Scale Up）
**定义**：增加单机资源（CPU、内存、磁盘）

**优点**：
- 实施简单
- 无需修改应用

**缺点**：
- 成本高（高端服务器）
- 有上限（单机最大配置）
- 单点风险

**适用场景**：
- 数据库（单机性能要求高）
- 单体应用
- 短期快速扩容

#### 2. 水平扩容（Scale Out）
**定义**：增加服务器/实例数量

**优点**：
- 线性扩展
- 成本可控（普通服务器）
- 高可用（无单点）

**缺点**：
- 需要应用支持分布式
- 复杂度增加（负载均衡、数据分片）

**适用场景**：
- 微服务架构
- 无状态应用
- 大规模系统

#### 3. 弹性扩容（Auto Scaling）
**定义**：根据负载自动调整资源

**策略**：
- 基于指标：CPU > 70% 扩容
- 基于时间：每天 8:00-22:00 保持高配
- 基于预测：AI 预测负载峰值提前扩容

**优点**：
- 自动化
- 成本优化（按需使用）

**缺点**：
- 响应延迟（扩容需要时间）
- 冷启动问题

## 容量规划流程

### 步骤 1：数据收集

#### 1.1 业务指标
**关键指标**：
- 用户数（DAU/MAU）
- 请求数（QPS/RPS）
- 数据量（订单数、交易额）
- 业务增长率

**数据来源**：
- 业务数据库
- BI 系统
- 产品运营数据

#### 1.2 技术指标
**关键指标**：
- 资源利用率（CPU、内存、磁盘、网络）
- 性能指标（延迟、吞吐量、错误率）
- 中间件指标（数据库连接数、缓存命中率）

**数据来源**：
- Prometheus
- Grafana
- APM 系统（Datadog/Jaeger）

**数据采集脚本**：
```python
import pandas as pd
import requests

def collect_metrics(service, start_time, end_time, step='5m'):
    """
    从 Prometheus 采集指标数据
    """
    prometheus_url = "http://prometheus.example.com/api/v1/query_range"

    metrics = [
        'cpu_usage',
        'memory_usage',
        'http_requests_total',
        'http_request_duration_seconds'
    ]

    data_frames = []
    for metric in metrics:
        query = f'{metric}{{service="{service}"}}'
        params = {
            'query': query,
            'start': start_time,
            'end': end_time,
            'step': step
        }

        response = requests.get(prometheus_url, params=params)
        result = response.json()['data']['result'][0]

        df = pd.DataFrame(result['values'], columns=['timestamp', metric])
        df['timestamp'] = pd.to_datetime(df['timestamp'], unit='s')
        df[metric] = df[metric].astype(float)

        data_frames.append(df)

    # 合并所有指标
    merged_df = data_frames[0]
    for df in data_frames[1:]:
        merged_df = pd.merge(merged_df, df, on='timestamp')

    return merged_df

# 使用示例
df = collect_metrics('order-service', '2026-01-01', '2026-03-20')
df.to_csv('order_service_metrics.csv', index=False)
```

### 步骤 2：趋势分析

#### 2.1 时间序列分解
**方法**：将时间序列分解为趋势、季节性、残差

**工具**：Statsmodels、Prophet

**实现**：
```python
from statsmodels.tsa.seasonal import seasonal_decompose
import pandas as pd

def decompose_time_series(data, period=7):
    """
    分解时间序列
    """
    # 转换为时间序列
    ts = pd.Series(data['qps'].values, index=pd.to_datetime(data['timestamp']))

    # 分解
    decomposition = seasonal_decompose(ts, model='multiplicative', period=period)

    # 绘图
    import matplotlib.pyplot as plt
    fig, axes = plt.subplots(4, 1, figsize=(12, 10))

    decomposition.observed.plot(ax=axes[0], title='原始数据')
    decomposition.trend.plot(ax=axes[1], title='趋势')
    decomposition.seasonal.plot(ax=axes[2], title='季节性')
    decomposition.resid.plot(ax=axes[3], title='残差')

    plt.tight_layout()
    plt.savefig('time_series_decomposition.png')

    return decomposition

# 使用示例
df = pd.read_csv('order_service_metrics.csv')
decomposition = decompose_time_series(df, period=7)
```

#### 2.2 增长率计算
**方法**：计算周环比、月环比、年同比增长率

**实现**：
```python
def calculate_growth_rate(data, metric='qps'):
    """
    计算增长率
    """
    df = data.copy()
    df['timestamp'] = pd.to_datetime(df['timestamp'])
    df = df.set_index('timestamp')

    # 周环比
    df['wow_growth'] = df[metric].pct_change(periods=7) * 100

    # 月环比
    df['mom_growth'] = df[metric].pct_change(periods=30) * 100

    # 年同比增长
    df['yoy_growth'] = df[metric].pct_change(periods=365) * 100

    return df

# 使用示例
df_with_growth = calculate_growth_rate(df, 'qps')
print(df_with_growth[['qps', 'wow_growth', 'mom_growth', 'yoy_growth']].tail(30))
```

#### 2.3 异常值处理
**方法**：识别并移除异常值（促销/故障导致的流量异常）

**实现**：
```python
import numpy as np

def remove_outliers(data, metric='qps', method='iqr', threshold=1.5):
    """
    移除异常值
    """
    df = data.copy()

    if method == 'iqr':
        # 四分位距法
        q1 = df[metric].quantile(0.25)
        q3 = df[metric].quantile(0.75)
        iqr = q3 - q1
        lower = q1 - threshold * iqr
        upper = q3 + threshold * iqr

        df_clean = df[(df[metric] >= lower) & (df[metric] <= upper)]

    elif method == 'zscore':
        # Z-score 法
        mean = df[metric].mean()
        std = df[metric].std()
        df['z_score'] = (df[metric] - mean) / std
        df_clean = df[abs(df['z_score']) < 3]

    return df_clean

# 使用示例
df_clean = remove_outliers(df, 'qps', method='iqr')
print(f"移除 {len(df) - len(df_clean)} 个异常值")
```

### 步骤 3：负载预测

#### 3.1 短期预测（1-7 天）
**方法**：Prophet、ARIMA

**场景**：日常容量规划、自动扩缩容

**实现**：
```python
from fbprophet import Prophet
import pandas as pd

def predict_load_prophet(data, metric='qps', periods=7):
    """
    使用 Prophet 预测负载
    """
    # 准备数据
    df = data[['timestamp', metric]].copy()
    df.columns = ['ds', 'y']
    df['ds'] = pd.to_datetime(df['ds'])

    # 训练模型
    model = Prophet(
        daily_seasonality=True,
        weekly_seasonality=True,
        yearly_seasonality=True
    )
    model.fit(df)

    # 预测
    future = model.make_future_dataframe(periods=periods)
    forecast = model.predict(future)

    # 绘图
    fig = model.plot(forecast)
    fig.savefig('load_forecast_prophet.png')

    return forecast[['ds', 'yhat', 'yhat_lower', 'yhat_upper']].tail(periods)

# 使用示例
forecast = predict_load_prophet(df, 'qps', periods=7)
print(forecast)
```

#### 3.2 中期预测（1-3 个月）
**方法**：线性回归、多项式回归、Prophet

**场景**：季度容量规划、预算制定

**实现**：
```python
from sklearn.linear_model import LinearRegression
import numpy as np

def predict_load_linear(data, metric='qps', days_ahead=90):
    """
    使用线性回归预测负载
    """
    df = data.copy()
    df['timestamp'] = pd.to_datetime(df['timestamp'])
    df = df.set_index('timestamp')

    # 准备特征
    df['days'] = (df.index - df.index[0]).days
    X = df['days'].values.reshape(-1, 1)
    y = df[metric].values

    # 训练模型
    model = LinearRegression()
    model.fit(X, y)

    # 预测
    last_day = df['days'].iloc[-1]
    future_days = np.array(range(last_day + 1, last_day + days_ahead + 1)).reshape(-1, 1)
    predictions = model.predict(future_days)

    # 置信区间（简化）
    residuals = y - model.predict(X)
    std_error = np.std(residuals)
    lower = predictions - 1.96 * std_error
    upper = predictions + 1.96 * std_error

    return {
        'predictions': predictions,
        'lower': lower,
        'upper': upper
    }

# 使用示例
result = predict_load_linear(df, 'qps', days_ahead=90)
print(f"90 天后预测 QPS: {result['predictions'][-1]:.0f}")
print(f"95% 置信区间: [{result['lower'][-1]:.0f}, {result['upper'][-1]:.0f}]")
```

#### 3.3 长期预测（6-12 个月）
**方法**：业务增长模型、S 曲线、市场分析

**场景**：年度容量规划、基础设施投资决策

**实现**：
```python
def predict_load_business_model(
    current_qps,
    monthly_growth_rate,
    months_ahead=12,
    peak_factor=3
):
    """
    基于业务增长模型预测负载
    """
    predictions = []

    for month in range(months_ahead):
        # 月度增长（考虑增长率递减）
        adjusted_growth_rate = monthly_growth_rate * (0.95 ** month)
        future_qps = current_qps * ((1 + adjusted_growth_rate) ** month)

        # 峰值流量
        peak_qps = future_qps * peak_factor

        predictions.append({
            'month': month + 1,
            'avg_qps': future_qps,
            'peak_qps': peak_qps
        })

    return predictions

# 使用示例
predictions = predict_load_business_model(
    current_qps=1000,
    monthly_growth_rate=0.1,  # 10% 月增长
    months_ahead=12,
    peak_factor=3
)

for pred in predictions:
    print(f"第 {pred['month']} 月: 平均 QPS {pred['avg_qps']:.0f}, 峰值 QPS {pred['peak_qps']:.0f}")
```

### 步骤 4：容量计算

#### 4.1 单服务容量计算
**公式**：
```
所需实例数 = (目标 QPS * 单请求 CPU 时间) / (CPU 核心数 * 目标利用率)
```

**示例**：
```python
def calculate_capacity(
    target_qps,
    cpu_per_request_ms,
    cpu_cores_per_instance,
    target_cpu_utilization=0.7
):
    """
    计算所需实例数
    """
    # 单实例 QPS 容量
    qps_per_instance = (
        cpu_cores_per_instance *
        target_cpu_utilization *
        1000 /  # 转换为毫秒
        cpu_per_request_ms
    )

    # 所需实例数
    required_instances = target_qps / qps_per_instance

    # 冗余（N+1 或 N+2）
    redundancy = max(2, required_instances * 0.2)
    total_instances = required_instances + redundancy

    return {
        'required_instances': required_instances,
        'redundancy': redundancy,
        'total_instances': total_instances,
        'qps_per_instance': qps_per_instance
    }

# 使用示例
capacity = calculate_capacity(
    target_qps=3000,
    cpu_per_request_ms=10,  # 每个请求消耗 10ms CPU
    cpu_cores_per_instance=4,
    target_cpu_utilization=0.7
)

print(f"所需实例数: {capacity['required_instances']:.1f}")
print(f"冗余实例数: {capacity['redundancy']:.1f}")
print(f"总实例数: {capacity['total_instances']:.1f}")
```

#### 4.2 数据库容量计算
**公式**：
```
所需存储 = 数据量 * (1 + 增长率)^月数 * 冗余因子
所需连接数 = (实例数 * 每实例连接数) * 峰值因子
```

**示例**：
```python
def calculate_database_capacity(
    current_data_size_gb,
    monthly_growth_rate,
    months_ahead=12,
    redundancy_factor=1.5,
    index_overhead=0.2
):
    """
    计算数据库存储容量
    """
    # 未来数据量
    future_data_size = current_data_size_gb * ((1 + monthly_growth_rate) ** months_ahead)

    # 索引开销
    data_with_index = future_data_size * (1 + index_overhead)

    # 冗余（主从复制、备份空间）
    total_storage = data_with_index * redundancy_factor

    return {
        'current_data_size_gb': current_data_size_gb,
        'future_data_size_gb': future_data_size,
        'data_with_index_gb': data_with_index,
        'total_storage_gb': total_storage
    }

# 使用示例
db_capacity = calculate_database_capacity(
    current_data_size_gb=500,
    monthly_growth_rate=0.15,
    months_ahead=12
)

print(f"当前数据量: {db_capacity['current_data_size_gb']} GB")
print(f"12 个月后数据量: {db_capacity['future_data_size_gb']:.0f} GB")
print(f"含索引: {db_capacity['data_with_index_gb']:.0f} GB")
print(f"总存储需求: {db_capacity['total_storage_gb']:.0f} GB")
```

#### 4.3 网络带宽计算
**公式**：
```
所需带宽 = (QPS * 平均请求大小 * 8) / (带宽利用率 * 1000000)
```

**示例**：
```python
def calculate_bandwidth(
    target_qps,
    avg_request_size_kb,
    peak_factor=3,
    bandwidth_utilization=0.7
):
    """
    计算所需网络带宽
    """
    # 平均带宽
    avg_bandwidth_mbps = (
        target_qps *
        avg_request_size_kb *
        8 /  # 转换为比特
        1000 /  # KB to MB
        bandwidth_utilization
    )

    # 峰值带宽
    peak_bandwidth_mbps = avg_bandwidth_mbps * peak_factor

    return {
        'avg_bandwidth_mbps': avg_bandwidth_mbps,
        'peak_bandwidth_mbps': peak_bandwidth_mbps
    }

# 使用示例
bandwidth = calculate_bandwidth(
    target_qps=1000,
    avg_request_size_kb=50,
    peak_factor=3
)

print(f"平均带宽需求: {bandwidth['avg_bandwidth_mbps']:.0f} Mbps")
print(f"峰值带宽需求: {bandwidth['peak_bandwidth_mbps']:.0f} Mbps")
```

### 步骤 5：成本优化

#### 5.1 资源利用率分析
**方法**：识别低利用率资源

**实现**：
```python
def analyze_resource_utilization(metrics, thresholds):
    """
    分析资源利用率
    """
    low_utilization = []
    high_utilization = []

    for service, data in metrics.items():
        avg_cpu = data['cpu_usage'].mean()
        avg_memory = data['memory_usage'].mean()

        if avg_cpu < thresholds['low_cpu'] or avg_memory < thresholds['low_memory']:
            low_utilization.append({
                'service': service,
                'avg_cpu': avg_cpu,
                'avg_memory': avg_memory,
                'recommendation': '缩容'
            })

        if avg_cpu > thresholds['high_cpu'] or avg_memory > thresholds['high_memory']:
            high_utilization.append({
                'service': service,
                'avg_cpu': avg_cpu,
                'avg_memory': avg_memory,
                'recommendation': '扩容'
            })

    return {
        'low_utilization': low_utilization,
        'high_utilization': high_utilization
    }

# 使用示例
metrics = {
    'order-service': {
        'cpu_usage': [40, 45, 42, 38, 50],
        'memory_usage': [60, 65, 62, 58, 70]
    },
    'inventory-service': {
        'cpu_usage': [15, 18, 12, 20, 16],
        'memory_usage': [25, 30, 28, 32, 26]
    }
}

thresholds = {
    'low_cpu': 30,
    'low_memory': 40,
    'high_cpu': 80,
    'high_memory': 85
}

result = analyze_resource_utilization(metrics, thresholds)
print("低利用率资源（建议缩容）:")
for item in result['low_utilization']:
    print(f"  {item['service']}: CPU {item['avg_cpu']:.0f}%, 内存 {item['avg_memory']:.0f}%")
```

#### 5.2 实例类型优化
**方法**：选择性价比最优的实例类型

**实现**：
```python
def optimize_instance_type(workload_profile, instance_options):
    """
    优化实例类型选择
    """
    best_instance = None
    best_cost_performance = 0

    for instance in instance_options:
        # 计算性能得分
        if workload_profile['cpu_intensive']:
            performance_score = instance['cpu_cores'] * 0.7 + instance['memory_gb'] * 0.3
        elif workload_profile['memory_intensive']:
            performance_score = instance['cpu_cores'] * 0.3 + instance['memory_gb'] * 0.7
        else:
            performance_score = instance['cpu_cores'] * 0.5 + instance['memory_gb'] * 0.5

        # 计算性价比
        cost_performance = performance_score / instance['price_per_hour']

        if cost_performance > best_cost_performance:
            best_cost_performance = cost_performance
            best_instance = instance

    return best_instance

# 使用示例
workload_profile = {
    'cpu_intensive': False,
    'memory_intensive': True
}

instance_options = [
    {'type': 'c5.2xlarge', 'cpu_cores': 8, 'memory_gb': 16, 'price_per_hour': 0.34},
    {'type': 'r5.2xlarge', 'cpu_cores': 8, 'memory_gb': 64, 'price_per_hour': 0.504},
    {'type': 'm5.2xlarge', 'cpu_cores': 8, 'memory_gb': 32, 'price_per_hour': 0.384}
]

best = optimize_instance_type(workload_profile, instance_options)
print(f"最优实例类型: {best['type']}, 价格: ${best['price_per_hour']}/小时")
```

#### 5.3 预留实例 vs 按需实例
**方法**：根据使用时长选择付费模式

**实现**：
```python
def compare_pricing_models(
    on_demand_price,
    reserved_price_1y,
    reserved_price_3y,
    usage_months
):
    """
    比较不同付费模式的成本
    """
    on_demand_cost = on_demand_price * usage_months * 730  # 每月 730 小时
    reserved_cost_1y = reserved_price_1y * min(usage_months, 12) * 730
    reserved_cost_3y = reserved_price_3y * min(usage_months, 36) * 730

    return {
        'on_demand': on_demand_cost,
        'reserved_1y': reserved_cost_1y,
        'reserved_3y': reserved_cost_3y,
        'recommendation': 'reserved_3y' if usage_months >= 24 else
                         'reserved_1y' if usage_months >= 6 else
                         'on_demand'
    }

# 使用示例
comparison = compare_pricing_models(
    on_demand_price=0.10,
    reserved_price_1y=0.06,
    reserved_price_3y=0.04,
    usage_months=18
)

print(f"按需付费成本: ${comparison['on_demand']:.2f}")
print(f"1 年预留实例成本: ${comparison['reserved_1y']:.2f}")
print(f"3 年预留实例成本: ${comparison['reserved_3y']:.2f}")
print(f"推荐: {comparison['recommendation']}")
```

## 自动化扩缩容

### Kubernetes HPA（Horizontal Pod Autoscaler）
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: order-service-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: order-service
  minReplicas: 3
  maxReplicas: 20
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
        - type: Percent
          value: 100
          periodSeconds: 15
        - type: Pods
          value: 4
          periodSeconds: 15
      selectPolicy: Max
```

### Cluster Autoscaler
```yaml
# 集群自动扩缩容配置
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cluster-autoscaler
  namespace: kube-system
spec:
  template:
    spec:
      containers:
        - name: cluster-autoscaler
          image: k8s.gcr.io/autoscaling/cluster-autoscaler:v1.21.0
          command:
            - ./cluster-autoscaler
            - --scale-down-unneeded-time=10m
            - --scale-down-delay-after-add=10m
            - --scale-down-delay-after-failure=3m
            - --scale-down-delay-after-delete=10s
            - --min-nodes=3
            - --max-nodes=50
            - --nodes=1:10:node-group-1
            - --nodes=1:20:node-group-2
```

### 基于预测的自动扩缩容
```python
import requests
from datetime import datetime, timedelta

def predictive_scaling(service, forecast_api, k8s_api):
    """
    基于预测的自动扩缩容
    """
    # 获取预测负载
    forecast = requests.get(f"{forecast_api}/predict/{service}?hours=1").json()
    predicted_qps = forecast['predicted_qps']
    current_qps = forecast['current_qps']

    # 计算所需实例数
    capacity = calculate_capacity(
        target_qps=predicted_qps,
        cpu_per_request_ms=10,
        cpu_cores_per_instance=4
    )

    # 获取当前实例数
    current_replicas = requests.get(f"{k8s_api}/deployments/{service}").json()['replicas']

    # 决策
    if capacity['total_instances'] > current_replicas * 1.2:
        # 提前扩容（预测负载增加）
        action = 'scale_up'
        target_replicas = int(capacity['total_instances'])
    elif capacity['total_instances'] < current_replicas * 0.8:
        # 延迟缩容（避免误判）
        action = 'scale_down'
        target_replicas = int(capacity['total_instances'])
    else:
        action = 'no_action'
        target_replicas = current_replicas

    # 执行扩缩容
    if action != 'no_action':
        requests.patch(
            f"{k8s_api}/deployments/{service}/scale",
            json={'replicas': target_replicas}
        )

    return {
        'action': action,
        'current_replicas': current_replicas,
        'target_replicas': target_replicas,
        'current_qps': current_qps,
        'predicted_qps': predicted_qps
    }
```

## 容量规划报告

### 报告模板
```markdown
# 容量规划报告 - YYYY-MM

## 执行摘要
- 规划周期：YYYY-MM 至 YYYY-MM
- 关键发现：[总结 3-5 个关键点]
- 总投资：$XXX
- 风险等级：[低/中/高]

## 当前状态
### 资源利用率
| 服务 | CPU | 内存 | 存储 | 状态 |
|------|-----|------|------|------|
| 订单服务 | 45% | 62% | 70% | 正常 |
| 支付服务 | 78% | 85% | 65% | 接近阈值 |
| 库存服务 | 30% | 40% | 50% | 低利用率 |

### 性能基线
| 服务 | QPS | P95 延迟 | 错误率 | SLO 达成 |
|------|-----|----------|--------|----------|
| 订单服务 | 1200 | 180ms | 0.05% | 99.9% |
| 支付服务 | 800 | 150ms | 0.08% | 99.8% |

## 负载预测
### 业务增长
- 用户增长率：15%/月
- QPS 增长率：12%/月
- 数据增长：20%/月

### 预测结果
| 时间 | QPS | 实例数 | 存储 | 带宽 |
|------|-----|--------|------|------|
| 当前 | 1,200 | 10 | 500 GB | 100 Mbps |
| 3 个月 | 1,700 | 14 | 800 GB | 140 Mbps |
| 6 个月 | 2,400 | 20 | 1,200 GB | 200 Mbps |
| 12 个月 | 4,800 | 40 | 2,500 GB | 400 Mbps |

## 扩容计划
### 短期（0-3 个月）
- 支付服务扩容：5 -> 8 实例
- 数据库存储扩容：500 GB -> 800 GB
- 带宽升级：100 Mbps -> 150 Mbps

### 中期（3-6 个月）
- 订单服务扩容：10 -> 15 实例
- Redis 缓存扩容：16 GB -> 32 GB
- 消息队列扩容：增加 2 个 broker

### 长期（6-12 个月）
- 新增数据中心（异地多活）
- 数据库分库分表
- CDN 节点扩展

## 成本分析
### 当前成本
- 计算资源：$10,000/月
- 存储资源：$2,000/月
- 网络资源：$1,500/月
- 总计：$13,500/月

### 预计成本（12 个月后）
- 计算资源：$35,000/月
- 存储资源：$6,000/月
- 网络资源：$5,000/月
- 总计：$46,000/月

### 成本优化建议
- 预留实例：节省 30-40%
- 竞价实例：非关键服务节省 60-70%
- 自动扩缩容：节省 20-30%

## 风险与缓解
### 风险 1：预测不准确
- 影响：资源不足或浪费
- 缓解：建立监控预警、定期调整预测模型

### 风险 2：供应链延迟（硬件采购）
- 影响：扩容延期
- 缓解：提前 3 个月采购、云资源备份方案

### 风险 3：预算限制
- 影响：扩容计划推迟
- 缓解：分阶段实施、优先核心服务

## 附录
- 详细数据表
- 预测模型说明
- 成本计算明细
```

## 常见失败模式

### 1. 预测过于乐观
**原因**：忽略季节性、促销、突发事件

**后果**：资源不足、性能下降

**解决**：保守预测、保留缓冲（20-30%）

### 2. 扩容响应慢
**原因**：审批流程长、采购周期长

**后果**：错过业务高峰、用户体验差

**解决**：预审批机制、弹性云资源

### 3. 忽略依赖服务
**原因**：只规划应用服务，忽略数据库/缓存/网络

**后果**：木桶效应、单点瓶颈

**解决**：全链路容量规划

### 4. 过度扩容
**原因**：求稳、缺乏成本意识

**后果**：资源浪费、成本失控

**解决**：定期审查利用率、自动缩容

### 5. 缺少回滚计划
**原因**：假设扩容一定成功

**后果**：扩容失败导致故障

**解决**：灰度扩容、快速回滚机制

## 验收标准

### 功能验收
- [ ] 容量规划模型建立
- [ ] 自动化预测系统上线
- [ ] 扩容决策流程文档化
- [ ] 容量规划报告模板

### 性能验收
- [ ] 预测准确率 >= 80%（对比实际负载）
- [ ] 扩容响应时间 < 4 小时（云资源）
- [ ] 资源利用率 60-80%
- [ ] 成本优化 >= 20%

### 运营验收
- [ ] 每月容量规划报告产出
- [ ] 季度容量评审会议
- [ ] 容量告警机制上线
- [ ] 团队培训覆盖率 100%

## 参考资源

### 工具
- Prometheus + Grafana：监控与可视化
- Prophet：时间序列预测
- Kubecost：Kubernetes 成本分析
- CloudHealth：多云成本管理

### 最佳实践
- Google SRE Book - Chapter on Capacity Planning
- AWS Well-Architected Framework - Cost Optimization
- The Art of Capacity Planning（O'Reilly）

### 云服务商
- AWS：Auto Scaling、Capacity Reservations
- Azure：Virtual Machine Scale Sets、Reserved VM Instances
- GCP：Managed Instance Groups、Committed Use Discounts
- Alibaba Cloud：Auto Scaling、Reserved Instances
