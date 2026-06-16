---
id: airflow-complete
title: Apache Airflow完整指南
domain: data-engineering
category: 01-standards
difficulty: intermediate
tags: [airflow, complete, connections和hooks, dag最佳实践, data-engineering, sensor等待任务, 变量和配置, 执行器配置]
quality_score: 70
last_updated: 2026-06-15
---
# Apache Airflow完整指南

## 概述
Apache Airflow是一个开源的工作流编排平台,用于开发和调度数据管道。使用Python代码定义工作流,支持复杂的依赖关系和分布式执行。

## 核心概念

### 1. DAG (Directed Acyclic Graph)
DAG是有向无环图,定义了任务的依赖关系和执行顺序。

```python
from airflow import DAG
from airflow.operators.python import PythonOperator
from datetime import datetime

# 定义DAG
with DAG(
    dag_id='example_dag',
    start_date=datetime(2024, 1, 1),
    schedule_interval='@daily',
    catchup=False
) as dag:
    
    task1 = PythonOperator(
        task_id='task1',
        python_callable=lambda: print("Task 1")
    )
    
    task2 = PythonOperator(
        task_id='task2',
        python_callable=lambda: print("Task 2")
    )
    
    task1 >> task2  # task1完成后执行task2
```

### 2. Task (任务)
Task是DAG中的基本执行单元。

**常用Operator**:
- **PythonOperator**: 执行Python函数
- **BashOperator**: 执行Bash命令
- **SQLOperator**: 执行SQL查询
- **DockerOperator**: 执行Docker容器
- **KubernetesPodOperator**: 执行K8s Pod

```python
from airflow.operators.bash import BashOperator
from airflow.operators.python import PythonOperator

# Python任务
def my_function(**kwargs):
    print(kwargs['ds'])

python_task = PythonOperator(
    task_id='python_task',
    python_callable=my_function,
    provide_context=True
)

# Bash任务
bash_task = BashOperator(
    task_id='bash_task',
    bash_command='echo "Hello Airflow"'
)
```

### 3. XCom (跨任务通信)
XCom用于任务间共享数据。

```python
def push_function(ti):
    ti.xcom_push(key='my_key', value='my_value')

def pull_function(ti):
    value = ti.xcom_pull(key='my_key', task_ids='push_task')
    print(value)

push_task = PythonOperator(
    task_id='push_task',
    python_callable=push_function
)

pull_task = PythonOperator(
    task_id='pull_task',
    python_callable=pull_function
)
```

## DAG最佳实践

### 1. 结构化DAG定义

```python
from airflow import DAG
from airflow.operators.python import PythonOperator
from airflow.utils.dates import days_ago
import logging

default_args = {
    'owner': 'airflow',
    'depends_on_past': False,
    'email': ['airflow@example.com'],
    'email_on_failure': True,
    'email_on_retry': False,
    'retries': 3,
    'retry_delay': timedelta(minutes=5),
}

with DAG(
    'etl_pipeline',
    default_args=default_args,
    description='ETL pipeline for data warehouse',
    schedule_interval='0 2 * * *',  # 每天凌晨2点
    start_date=days_ago(1),
    catchup=False,
    tags=['etl', 'warehouse'],
) as dag:
    
    def extract(**kwargs):
        logging.info("Extracting data...")
        return {'data': 'extracted'}
    
    def transform(**kwargs):
        ti = kwargs['ti']
        extracted_data = ti.xcom_pull(task_ids='extract')
        logging.info(f"Transforming {extracted_data}")
        return {'data': 'transformed'}
    
    def load(**kwargs):
        ti = kwargs['ti']
        transformed_data = ti.xcom_pull(task_ids='transform')
        logging.info(f"Loading {transformed_data}")
    
    extract_task = PythonOperator(
        task_id='extract',
        python_callable=extract,
        provide_context=True
    )
    
    transform_task = PythonOperator(
        task_id='transform',
        python_callable=transform,
        provide_context=True
    )
    
    load_task = PythonOperator(
        task_id='load',
        python_callable=load,
        provide_context=True
    )
    
    extract_task >> transform_task >> load_task
```

### 2. 动态DAG生成

```python
from airflow import DAG
from airflow.operators.python import PythonOperator
from datetime import datetime

def create_dag(dag_id, schedule, config):
    with DAG(
        dag_id=dag_id,
        schedule_interval=schedule,
        start_date=datetime(2024, 1, 1)
    ) as dag:
        
        task = PythonOperator(
            task_id='task',
            python_callable=lambda: print(f"Running {dag_id}")
        )
        
        return dag

# 动态创建多个DAG
for i in range(10):
    dag_id = f'dynamic_dag_{i}'
    schedule = '@daily'
    config = {'id': i}
    globals()[dag_id] = create_dag(dag_id, schedule, config)
```

## Sensor等待任务

### 1. 时间Sensor

```python
from airflow.sensors.time_sensor import TimeSensor
from airflow.sensors.time_delta_sensor import TimeDeltaSensor

# 等待特定时间
wait_for_time = TimeSensor(
    task_id='wait_for_time',
    target_time='10:00:00',
    poke_interval=60
)

# 等待时间增量
wait_for_delta = TimeDeltaSensor(
    task_id='wait_for_delta',
    delta=timedelta(hours=2)
)
```

### 2. 外部Sensor

```python
from airflow.sensors.external_task import ExternalTaskSensor

# 等待另一个DAG的任务完成
wait_for_other_dag = ExternalTaskSensor(
    task_id='wait_for_other_dag',
    external_dag_id='other_dag',
    external_task_id='final_task',
    mode='reschedule',  # 节省资源
    poke_interval=300
)
```

### 3. 文件Sensor

```python
from airflow.sensors.filesystem import FileSensor

# 等待文件出现
wait_for_file = FileSensor(
    task_id='wait_for_file',
    filepath='/path/to/file.csv',
    poke_interval=30
)
```

## Connections和Hooks

### 1. Connection配置

**Web UI配置**:
1. Admin -> Connections
2. 添加Connection:
   - Connection Id: `postgres_default`
   - Connection Type: `Postgres`
   - Host: `localhost`
   - Port: `5432`
   - Database: `airflow`
   - Username: `airflow`
   - Password: `airflow`

### 2. 使用Hook

```python
from airflow.providers.postgres.hooks.postgres import PostgresHook

def query_database():
    hook = PostgresHook(postgres_conn_id='postgres_default')
    df = hook.get_pandas_df("SELECT * FROM users")
    return df.to_dict()

query_task = PythonOperator(
    task_id='query_database',
    python_callable=query_database
)
```

## 变量和配置

### 1. Variables

```python
from airflow.models import Variable

# 设置Variable (Web UI: Admin -> Variables)
# Variable.set('max_items', 100)

# 使用Variable
def use_variable():
    max_items = Variable.get('max_items', default_var=50)
    print(f"Max items: {max_items}")

# JSON Variable
config = Variable.get('config', deserialize_json=True)
```

### 2. Jinja模板

```python
from airflow.operators.bash import BashOperator

templated_task = BashOperator(
    task_id='templated_task',
    bash_command='echo "Running {{ ds }}" with {{ params.my_param }}',
    params={'my_param': 'custom_value'}
)
```

## 执行器配置

### 1. SequentialExecutor (默认)
单进程执行,适合开发。

### 2. LocalExecutor
多进程,适合单机生产。

```python
# airflow.cfg
executor = LocalExecutor
sql_alchemy_conn = postgresql+psycopg2://airflow:airflow@localhost/airflow
```

### 3. CeleryExecutor
分布式,适合大规模生产。

```python
# airflow.cfg
executor = CeleryExecutor
broker_url = redis://localhost:6379/0
result_backend = db+postgresql://airflow:airflow@localhost/airflow
```

### 4. KubernetesExecutor
K8s原生,适合云环境。

```python
# airflow.cfg
executor = KubernetesExecutor
kubernetes_namespace = airflow
```

## 监控和日志

### 1. 日志配置

```python
# airflow.cfg
[core]
logging_level = INFO
fab_logging_level = WARN

[logging]
base_log_folder = /path/to/logs
```

### 2. 自定义日志

```python
import logging

def my_task(**kwargs):
    logger = logging.getLogger(__name__)
    logger.info("This is an info message")
    logger.error("This is an error message")
```

### 3. Email告警

```python
default_args = {
    'email': ['admin@example.com'],
    'email_on_failure': True,
    'email_on_retry': True,
}
```

## 性能优化

### 1. 并行执行

```python
from airflow.operators.python import PythonOperator

# 并行任务
parallel_tasks = [
    PythonOperator(task_id=f'task_{i}', python_callable=lambda i=i: print(f"Task {i}"))
    for i in range(10)
]

# join
parallel_tasks >> final_task
```

### 2. 资源池

```python
resource_task = PythonOperator(
    task_id='resource_task',
    python_callable=my_function,
    pool='limited_pool'  # 限制并发
)
```

### 3. 动态任务映射

```python
def process_file(filename):
    print(f"Processing {filename}")

files = ['file1.csv', 'file2.csv', 'file3.csv']

process_tasks = [
    PythonOperator(
        task_id=f'process_{file}',
        python_callable=process_file,
        op_kwargs={'filename': file}
    )
    for file in files
]
```

## 最佳实践

### ✅ DO

1. **使用DAG上下文管理器**
```python
with DAG(...) as dag:
    # 定义任务
```

2. **提供默认参数**
```python
default_args = {
    'owner': 'team',
    'retries': 3,
    'retry_delay': timedelta(minutes=5),
}
```

3. **添加有意义的标签**
```python
with DAG(..., tags=['etl', 'production', 'critical']):
```

4. **使用XCom传递少量数据**
```python
# ✅ 好: 少量数据
ti.xcom_push(key='record_count', value=1000)

# ❌ 差: 大量数据
ti.xcom_push(key='dataframe', value=large_df)
```

### ❌ DON'T

1. **不要在顶层代码中写业务逻辑**
```python
# ❌ 错误
result = query_database()  # 每次解析DAG都会执行

def my_task():
    # ✅ 正确
    result = query_database()
```

2. **不要使用全局变量**
```python
# ❌ 错误
GLOBAL_VAR = []

def my_task():
    GLOBAL_VAR.append(1)
```

3. **不要硬编码敏感信息**
```python
# ❌ 错误
password = "secret123"

# ✅ 正确: 使用Connection或Variable
password = Variable.get('db_password')
```

## 学习路径

### 初级 (1周)
1. Airflow架构和核心概念
2. 第一个DAG
3. 基础Operator使用

### 中级 (2周)
1. Sensor和XCom
2. 动态DAG生成
3. Hook和Connection

### 高级 (2周)
1. 自定义Operator
2. 分布式执行器
3. 性能优化

### 专家级 (持续)
1. 多租户架构
2. 安全和权限管理
3. 监控和故障排查

## 参考资料

### 官方文档
- [Airflow官方文档](https://airflow.apache.org/docs/)
- [Provider包](https://airflow.apache.org/docs/apache-airflow-providers/index.html)

### 教程
- [Airflow教程](https://airflow.apache.org/docs/tutorial.html)
- [Astronomer指南](https://www.astronomer.io/guides/)

---

**知识ID**: `airflow-complete`  
**领域**: data-engineering  
**类型**: standards  
**难度**: intermediate  
**质量分**: 92  
**维护者**: data-team@superdev.com  
**最后更新**: 2026-03-28
