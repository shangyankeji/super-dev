---
id: mlops-complete
title: MLOps 完整指南
domain: ai
category: 01-standards
difficulty: intermediate
tags: [agent, ai, checklist, complete, mlops, 实验跟踪, 数据漂移检测, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# MLOps 完整指南

## 概述

MLOps (Machine Learning Operations) 是将机器学习模型从实验到生产的工程化实践体系。本指南覆盖实验跟踪、模型注册、部署策略、生产监控、数据漂移检测和 A/B 测试，适用于传统 ML 和 LLM 应用的全生命周期管理。

### MLOps 成熟度模型

```
MLOps 成熟度等级:
├── Level 0 — 手工作坊
│   手动训练、手动部署、无版本管理
├── Level 1 — 管道自动化
│   自动训练管道、实验跟踪、模型注册
├── Level 2 — CI/CD for ML
│   自动测试、自动部署、特征存储
├── Level 3 — 全自动化
│   自动重训练、漂移检测、自动回滚
└── Level 4 — 自适应
    自动特征工程、模型选择、超参优化
```

---

## 1. 实验跟踪

### 1.1 实验跟踪架构

```
实验跟踪要素:
├── 代码版本 — Git commit hash
├── 数据版本 — 数据集 hash 或 DVC 版本
├── 超参数 — 所有训练参数
├── 指标 — 训练和验证指标曲线
├── 模型产物 — 权重文件、配置
├── 环境 — Python 版本、依赖包版本、GPU 型号
└── 元数据 — 实验者、时间、备注
```

### 1.2 MLflow 实验跟踪

```python
import mlflow
from mlflow.tracking import MlflowClient

class ExperimentTracker:
    """MLflow 实验跟踪封装。"""

    def __init__(self, experiment_name: str,
                 tracking_uri: str = "http://mlflow:5000"):
        mlflow.set_tracking_uri(tracking_uri)
        mlflow.set_experiment(experiment_name)
        self.client = MlflowClient(tracking_uri)

    def start_run(self, run_name: str, params: dict,
                  tags: dict | None = None) -> str:
        """开始一次实验记录。"""
        run = mlflow.start_run(run_name=run_name, tags=tags or {})
        mlflow.log_params(params)
        # 记录代码版本
        import subprocess
        git_hash = subprocess.check_output(
            ["git", "rev-parse", "HEAD"]
        ).decode().strip()
        mlflow.set_tag("git_commit", git_hash)
        return run.info.run_id

    def log_metrics(self, metrics: dict, step: int | None = None):
        """记录指标。"""
        for key, value in metrics.items():
            mlflow.log_metric(key, value, step=step)

    def log_model(self, model, model_name: str, input_example=None):
        """记录模型产物。"""
        mlflow.sklearn.log_model(
            model, model_name,
            input_example=input_example,
            registered_model_name=model_name,
        )

    def end_run(self):
        mlflow.end_run()

    def compare_runs(self, metric: str, top_k: int = 5) -> list[dict]:
        """比较实验结果，返回 Top-K 最佳运行。"""
        experiment = mlflow.get_experiment_by_name(
            mlflow.get_experiment(
                mlflow.active_run().info.experiment_id
            ).name
        )
        runs = self.client.search_runs(
            experiment_ids=[experiment.experiment_id],
            order_by=[f"metrics.{metric} DESC"],
            max_results=top_k,
        )
        return [
            {
                "run_id": r.info.run_id,
                "run_name": r.info.run_name,
                "params": r.data.params,
                "metrics": r.data.metrics,
            }
            for r in runs
        ]
```

### 1.3 LLM 实验跟踪 (特殊需求)

```python
class LLMExperimentTracker:
    """LLM 应用专用实验跟踪。"""

    def __init__(self, storage):
        self.storage = storage

    def log_prompt_experiment(self, experiment: dict):
        """记录 Prompt 实验。"""
        record = {
            "experiment_id": str(uuid.uuid4()),
            "timestamp": datetime.utcnow().isoformat(),
            "prompt_version": experiment["prompt_version"],
            "prompt_template": experiment["template"],
            "model": experiment["model"],
            "temperature": experiment["temperature"],
            "test_cases": experiment["test_cases"],
            "results": experiment["results"],
            "metrics": {
                "accuracy": experiment.get("accuracy"),
                "latency_p50_ms": experiment.get("latency_p50"),
                "latency_p95_ms": experiment.get("latency_p95"),
                "avg_input_tokens": experiment.get("avg_input_tokens"),
                "avg_output_tokens": experiment.get("avg_output_tokens"),
                "cost_per_request": experiment.get("cost_per_request"),
            },
            "evaluator": experiment.get("evaluator", "auto"),
        }
        self.storage.insert("llm_experiments", record)
        return record["experiment_id"]
```

---

## 2. 模型注册

### 2.1 模型注册中心

```python
class ModelRegistry:
    """模型注册中心: 版本管理、阶段转换、元数据管理。"""

    STAGES = ["development", "staging", "production", "archived"]

    def __init__(self, mlflow_client: MlflowClient):
        self.client = mlflow_client

    def register(self, model_name: str, run_id: str,
                 description: str = "") -> str:
        """注册新模型版本。"""
        result = self.client.create_model_version(
            name=model_name,
            source=f"runs:/{run_id}/model",
            run_id=run_id,
            description=description,
        )
        return result.version

    def promote(self, model_name: str, version: str,
                target_stage: str, approval: dict | None = None):
        """推进模型到下一阶段。"""
        if target_stage not in self.STAGES:
            raise ValueError(f"无效阶段: {target_stage}")

        # 生产阶段需要审批
        if target_stage == "production":
            if not approval or not approval.get("approved_by"):
                raise PermissionError("推进到生产阶段需要审批")

        self.client.transition_model_version_stage(
            name=model_name,
            version=version,
            stage=target_stage,
        )

    def get_production_model(self, model_name: str) -> dict | None:
        """获取当前生产版本。"""
        versions = self.client.get_latest_versions(
            model_name, stages=["production"]
        )
        if versions:
            v = versions[0]
            return {
                "version": v.version,
                "run_id": v.run_id,
                "source": v.source,
                "created_at": v.creation_timestamp,
            }
        return None
```

### 2.2 模型卡片 (Model Card)

```yaml
# model_card.yaml — 每个注册模型必须附带
name: "fraud-detection-v3"
version: "3.2.1"
description: "基于 XGBoost 的交易欺诈检测模型"

model_details:
  type: "binary_classification"
  framework: "xgboost"
  training_data: "transactions_2024_q1_q3"
  features: 47
  training_samples: 2_800_000

performance:
  metrics:
    - name: "AUC-ROC"
      value: 0.9834
      dataset: "test_2024_q4"
    - name: "Precision@0.95_Recall"
      value: 0.89
      dataset: "test_2024_q4"
    - name: "F1"
      value: 0.92
      dataset: "test_2024_q4"

fairness:
  evaluated_groups: ["gender", "age_bucket", "region"]
  max_disparity: 0.03

limitations:
  - "对新型欺诈模式 (训练数据中未出现) 检出率可能较低"
  - "高峰时段延迟可能超过 SLA (50ms)"

ethical_considerations:
  - "模型决策影响用户交易，需要人工复核通道"
  - "年龄和地区不应成为主要判别因子"
```

---

## 3. 部署策略

### 3.1 部署模式对比

| 模式 | 延迟 | 吞吐量 | 适用场景 |
|------|------|--------|---------|
| REST API | 10-100ms | 中 | 在线推理，低并发 |
| gRPC | 5-50ms | 高 | 在线推理，高并发 |
| Batch | 分钟级 | 极高 | 离线批量处理 |
| Streaming | 首 Token < 200ms | 中 | LLM 生成式输出 |
| Edge | 1-10ms | 低 | 端侧推理，离线场景 |

### 3.2 模型服务部署

```python
# FastAPI 模型服务
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import time

app = FastAPI(title="Model Serving API")

class PredictionRequest(BaseModel):
    features: dict
    model_version: str | None = None

class PredictionResponse(BaseModel):
    prediction: float
    confidence: float
    model_version: str
    latency_ms: float

class ModelServer:
    """模型服务器: 支持多版本、金丝雀和回滚。"""

    def __init__(self, registry: ModelRegistry):
        self.registry = registry
        self.loaded_models: dict[str, object] = {}
        self.active_version: str | None = None
        self.canary_version: str | None = None
        self.canary_ratio: float = 0.0

    def load_model(self, model_name: str, version: str):
        """加载模型到内存。"""
        import mlflow.pyfunc
        model = mlflow.pyfunc.load_model(
            model_uri=f"models:/{model_name}/{version}"
        )
        self.loaded_models[version] = model

    def predict(self, request: PredictionRequest) -> PredictionResponse:
        version = self._select_version(request.model_version)
        model = self.loaded_models.get(version)
        if not model:
            raise HTTPException(404, f"模型版本 {version} 未加载")

        start = time.monotonic()
        result = model.predict(request.features)
        latency = (time.monotonic() - start) * 1000

        return PredictionResponse(
            prediction=float(result["prediction"]),
            confidence=float(result["confidence"]),
            model_version=version,
            latency_ms=round(latency, 2),
        )

    def _select_version(self, requested: str | None) -> str:
        """选择模型版本: 支持金丝雀流量分配。"""
        if requested:
            return requested
        if self.canary_version and random.random() < self.canary_ratio:
            return self.canary_version
        return self.active_version
```

### 3.3 蓝绿部署与金丝雀

```yaml
# kubernetes deployment — canary strategy
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: model-serving
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: model-serving
  progressDeadlineSeconds: 600
  service:
    port: 8080
  analysis:
    interval: 1m
    threshold: 5
    maxWeight: 50
    stepWeight: 10
    metrics:
      - name: request-success-rate
        thresholdRange:
          min: 99
        interval: 1m
      - name: prediction-latency-p99
        thresholdRange:
          max: 100
        interval: 1m
```

---

## 4. 生产监控

### 4.1 监控指标体系

```
ML 监控指标:
├── 模型性能指标
│   ├── 准确率/精确率/召回率 (实时 vs 离线)
│   ├── 预测分布 (与训练集对比)
│   └── 置信度分布
├── 系统性能指标
│   ├── 推理延迟 (P50/P95/P99)
│   ├── 吞吐量 (QPS)
│   ├── 错误率
│   └── GPU/CPU/内存使用率
├── 数据质量指标
│   ├── 特征缺失率
│   ├── 异常值比例
│   └── 数据新鲜度
└── 业务指标
    ├── 转化率/点击率变化
    ├── 用户反馈
    └── 人工干预率
```

### 4.2 监控实现

```python
from prometheus_client import Counter, Histogram, Gauge, Summary
import numpy as np

# Prometheus 指标定义
PREDICTION_COUNTER = Counter(
    "model_predictions_total",
    "模型预测总数",
    ["model_name", "model_version", "result_class"],
)
PREDICTION_LATENCY = Histogram(
    "model_prediction_latency_seconds",
    "模型预测延迟",
    ["model_name"],
    buckets=[0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0],
)
PREDICTION_CONFIDENCE = Summary(
    "model_prediction_confidence",
    "预测置信度分布",
    ["model_name"],
)
DATA_DRIFT_SCORE = Gauge(
    "model_data_drift_score",
    "数据漂移得分",
    ["model_name", "feature_name"],
)

class ModelMonitor:
    """模型生产监控。"""

    def __init__(self, model_name: str, reference_data):
        self.model_name = model_name
        self.reference_stats = self._compute_stats(reference_data)
        self.prediction_buffer: list[dict] = []

    def record_prediction(self, features: dict, prediction: float,
                          confidence: float, latency_s: float):
        """记录单次预测并更新监控指标。"""
        result_class = "positive" if prediction > 0.5 else "negative"
        PREDICTION_COUNTER.labels(
            self.model_name, "current", result_class
        ).inc()
        PREDICTION_LATENCY.labels(self.model_name).observe(latency_s)
        PREDICTION_CONFIDENCE.labels(self.model_name).observe(confidence)

        self.prediction_buffer.append({
            "features": features,
            "prediction": prediction,
            "confidence": confidence,
        })

        # 每 1000 次预测检查漂移
        if len(self.prediction_buffer) >= 1000:
            self._check_drift()
            self.prediction_buffer = []

    def _check_drift(self):
        """检查数据漂移 (使用 PSI 方法)。"""
        for feature_name in self.reference_stats:
            current_values = [
                p["features"].get(feature_name)
                for p in self.prediction_buffer
                if p["features"].get(feature_name) is not None
            ]
            if not current_values:
                continue
            psi = self._calculate_psi(
                self.reference_stats[feature_name],
                np.array(current_values),
            )
            DATA_DRIFT_SCORE.labels(self.model_name, feature_name).set(psi)

    @staticmethod
    def _calculate_psi(reference: np.ndarray, current: np.ndarray,
                       bins: int = 10) -> float:
        """计算 Population Stability Index (PSI)。"""
        breakpoints = np.percentile(reference,
                                     np.linspace(0, 100, bins + 1))
        ref_counts = np.histogram(reference, bins=breakpoints)[0] / len(reference)
        cur_counts = np.histogram(current, bins=breakpoints)[0] / len(current)

        # 避免除零
        ref_counts = np.clip(ref_counts, 1e-6, None)
        cur_counts = np.clip(cur_counts, 1e-6, None)

        psi = np.sum((cur_counts - ref_counts) * np.log(cur_counts / ref_counts))
        return float(psi)

    @staticmethod
    def _compute_stats(data) -> dict:
        """计算参考数据集的统计特征。"""
        stats = {}
        for col in data.columns:
            if data[col].dtype in [np.float64, np.int64]:
                stats[col] = data[col].values
        return stats
```

---

## 5. 数据漂移检测

### 5.1 漂移类型与检测方法

| 漂移类型 | 描述 | 检测方法 | 告警阈值 |
|----------|------|---------|----------|
| 数据漂移 (Data Drift) | 输入特征分布变化 | PSI, KS Test | PSI > 0.2 |
| 概念漂移 (Concept Drift) | 特征与标签关系变化 | 性能指标下降 | AUC 下降 > 5% |
| 预测漂移 (Prediction Drift) | 预测分布变化 | PSI on predictions | PSI > 0.15 |
| 标签漂移 (Label Drift) | 标签分布变化 | Chi-squared test | p-value < 0.05 |

### 5.2 自动漂移检测管道

```python
from scipy import stats

class DriftDetector:
    """自动漂移检测管道。"""

    def __init__(self, reference_data, alert_callback=None):
        self.reference = reference_data
        self.alert = alert_callback

    def detect(self, current_data) -> dict:
        """运行全量漂移检测。"""
        report = {"features": {}, "overall_drift": False, "alerts": []}

        for feature in self.reference.columns:
            if self.reference[feature].dtype in [np.float64, np.int64]:
                result = self._test_numeric(feature, current_data[feature])
            else:
                result = self._test_categorical(feature, current_data[feature])

            report["features"][feature] = result
            if result["drifted"]:
                report["overall_drift"] = True
                report["alerts"].append(
                    f"特征 {feature} 漂移: {result['method']}={result['score']:.4f}"
                )

        if report["overall_drift"] and self.alert:
            self.alert(report)
        return report

    def _test_numeric(self, feature: str, current) -> dict:
        ref = self.reference[feature].dropna()
        cur = current.dropna()

        # KS Test
        ks_stat, ks_pvalue = stats.ks_2samp(ref, cur)
        # PSI
        psi = ModelMonitor._calculate_psi(ref.values, cur.values)

        return {
            "method": "KS+PSI",
            "ks_statistic": float(ks_stat),
            "ks_pvalue": float(ks_pvalue),
            "psi": float(psi),
            "score": float(psi),
            "drifted": psi > 0.2 or ks_pvalue < 0.01,
        }

    def _test_categorical(self, feature: str, current) -> dict:
        ref_counts = self.reference[feature].value_counts(normalize=True)
        cur_counts = current.value_counts(normalize=True)

        # 对齐类别
        all_categories = set(ref_counts.index) | set(cur_counts.index)
        ref_freq = [ref_counts.get(c, 1e-6) for c in all_categories]
        cur_freq = [cur_counts.get(c, 1e-6) for c in all_categories]

        chi2, pvalue = stats.chisquare(cur_freq, ref_freq)
        return {
            "method": "chi-squared",
            "chi2": float(chi2),
            "pvalue": float(pvalue),
            "score": float(chi2),
            "drifted": pvalue < 0.05,
        }
```

### 5.3 自动重训练触发

```python
class RetrainingTrigger:
    """基于漂移检测的自动重训练触发器。"""

    def __init__(self, drift_detector: DriftDetector,
                 training_pipeline, config: dict):
        self.detector = drift_detector
        self.pipeline = training_pipeline
        self.config = config
        self.consecutive_drift_count = 0

    def evaluate_and_trigger(self, current_data) -> dict:
        """评估漂移并决定是否触发重训练。"""
        drift_report = self.detector.detect(current_data)

        if drift_report["overall_drift"]:
            self.consecutive_drift_count += 1
        else:
            self.consecutive_drift_count = 0

        action = "none"
        if self.consecutive_drift_count >= self.config.get("drift_patience", 3):
            action = "retrain"
            self.pipeline.trigger(
                reason="data_drift",
                drift_report=drift_report,
                data_window=self.config.get("training_window", "90d"),
            )
            self.consecutive_drift_count = 0

        return {
            "drift_detected": drift_report["overall_drift"],
            "consecutive_count": self.consecutive_drift_count,
            "action": action,
            "report": drift_report,
        }
```

---

## 6. A/B 测试

### 6.1 A/B 测试框架

```python
class ModelABTest:
    """模型 A/B 测试框架。"""

    def __init__(self, control_model: str, treatment_model: str,
                 traffic_split: float = 0.1):
        self.control = control_model
        self.treatment = treatment_model
        self.split = traffic_split
        self.results = {"control": [], "treatment": []}

    def assign_group(self, user_id: str) -> str:
        """确定性分组: 同一用户始终进入同一组。"""
        bucket = int(hashlib.md5(user_id.encode()).hexdigest(), 16) % 100
        return "treatment" if bucket < self.split * 100 else "control"

    def record_outcome(self, group: str, prediction: float,
                       actual: float, latency_ms: float):
        """记录实验结果。"""
        self.results[group].append({
            "prediction": prediction,
            "actual": actual,
            "latency_ms": latency_ms,
            "correct": (prediction > 0.5) == (actual > 0.5),
        })

    def analyze(self) -> dict:
        """分析实验结果并判断统计显著性。"""
        control = self.results["control"]
        treatment = self.results["treatment"]

        if len(control) < 100 or len(treatment) < 100:
            return {"status": "insufficient_data",
                    "message": "样本量不足，继续收集数据"}

        c_acc = np.mean([r["correct"] for r in control])
        t_acc = np.mean([r["correct"] for r in treatment])

        c_latency = np.mean([r["latency_ms"] for r in control])
        t_latency = np.mean([r["latency_ms"] for r in treatment])

        # 双样本 Z 检验
        z_stat, p_value = self._proportion_z_test(
            sum(r["correct"] for r in control), len(control),
            sum(r["correct"] for r in treatment), len(treatment),
        )

        significant = p_value < 0.05
        winner = "treatment" if t_acc > c_acc and significant else "control"

        return {
            "status": "complete" if significant else "not_significant",
            "control_accuracy": round(c_acc, 4),
            "treatment_accuracy": round(t_acc, 4),
            "accuracy_lift": round((t_acc - c_acc) / c_acc * 100, 2),
            "control_latency_ms": round(c_latency, 1),
            "treatment_latency_ms": round(t_latency, 1),
            "p_value": round(p_value, 4),
            "significant": significant,
            "recommendation": f"推荐 {winner}",
            "sample_sizes": {
                "control": len(control),
                "treatment": len(treatment),
            },
        }

    @staticmethod
    def _proportion_z_test(x1: int, n1: int,
                           x2: int, n2: int) -> tuple[float, float]:
        p1 = x1 / n1
        p2 = x2 / n2
        p_pool = (x1 + x2) / (n1 + n2)
        se = np.sqrt(p_pool * (1 - p_pool) * (1/n1 + 1/n2))
        z = (p2 - p1) / se if se > 0 else 0
        p_value = 2 * (1 - stats.norm.cdf(abs(z)))
        return float(z), float(p_value)
```

### 6.2 LLM A/B 测试 (特殊考虑)

```python
class LLMABTest:
    """LLM 应用 A/B 测试: 除准确率外还关注体验指标。"""

    METRICS = [
        "task_completion_rate",  # 任务完成率
        "user_satisfaction",     # 用户满意度 (thumbs up/down)
        "avg_turns",             # 平均对话轮次
        "avg_latency_ms",        # 平均延迟
        "cost_per_session",      # 每会话成本
        "hallucination_rate",    # 幻觉率
    ]

    def analyze_llm(self, control_sessions: list[dict],
                    treatment_sessions: list[dict]) -> dict:
        """LLM 多维指标对比分析。"""
        results = {}
        for metric in self.METRICS:
            c_values = [s.get(metric, 0) for s in control_sessions]
            t_values = [s.get(metric, 0) for s in treatment_sessions]

            t_stat, p_value = stats.ttest_ind(c_values, t_values)
            results[metric] = {
                "control_mean": round(np.mean(c_values), 4),
                "treatment_mean": round(np.mean(t_values), 4),
                "p_value": round(p_value, 4),
                "significant": p_value < 0.05,
            }
        return results
```

---

## 7. CI/CD for ML

### 7.1 ML 管道定义

```yaml
# .github/workflows/ml-pipeline.yml
name: ML Pipeline
on:
  push:
    paths: ["models/**", "features/**", "training/**"]

jobs:
  data-validation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Validate data schema
        run: python scripts/validate_data.py
      - name: Check data quality
        run: python scripts/check_data_quality.py

  training:
    needs: data-validation
    runs-on: [self-hosted, gpu]
    steps:
      - name: Train model
        run: python training/train.py --config configs/prod.yaml
      - name: Evaluate model
        run: python training/evaluate.py
      - name: Register model
        run: python scripts/register_model.py
        if: ${{ env.EVAL_PASSED == 'true' }}

  model-testing:
    needs: training
    steps:
      - name: Unit tests
        run: pytest tests/model/
      - name: Integration tests
        run: pytest tests/integration/
      - name: Performance benchmark
        run: python scripts/benchmark.py --threshold-latency-p99 100

  deploy-staging:
    needs: model-testing
    steps:
      - name: Deploy to staging
        run: ./scripts/deploy.sh staging
      - name: Smoke test
        run: python scripts/smoke_test.py --env staging
      - name: Shadow traffic test
        run: python scripts/shadow_test.py --duration 30m
```

---

## Agent Checklist

- [ ] 每次训练记录代码版本、数据版本、超参数和指标
- [ ] 模型注册中心运行，生产推进需要审批
- [ ] 每个模型附带 Model Card (性能、公平性、局限性)
- [ ] 部署支持金丝雀发布和自动回滚
- [ ] 推理延迟 P95/P99 有监控和告警
- [ ] 数据漂移检测管道运行，PSI > 0.2 触发告警
- [ ] 概念漂移有性能指标监控，AUC 下降 > 5% 触发告警
- [ ] 自动重训练管道就绪，连续漂移可触发
- [ ] A/B 测试框架支持确定性分组和统计显著性检验
- [ ] ML CI/CD 包含数据校验、训练、评估、测试和部署
- [ ] LLM 特有指标 (幻觉率、成本、满意度) 纳入监控
- [ ] 模型退役有归档流程和文档记录
