---
id: llm-evaluation-playbook
title: LLM 评估 Playbook
domain: ai
category: 02-playbooks
difficulty: intermediate
tags: [ai, evaluation, llm, llm-as-judge, playbook, 人工评估, 回答, 基准测试]
quality_score: 70
last_updated: 2026-06-15
---
# LLM 评估 Playbook

## 概述

LLM 评估是确保大语言模型应用质量的关键环节。本 Playbook 覆盖基准测试、人工评估、LLM-as-Judge、RAGAS 评估框架和完整指标体系，适用于对话系统、RAG、Agent 和代码生成等场景的系统化评估。

### 评估的核心挑战

```
LLM 评估难点:
├── 开放性输出 — 无唯一正确答案，难以自动评分
├── 主观性强 — 质量依赖评估者偏好和任务上下文
├── 维度多元 — 准确性、流畅性、安全性需分别评估
├── 成本高昂 — 人工评估耗时，自动评估需要校准
└── 可复现性 — 温度、Prompt 细微变化导致结果不稳定
```

---

## 1. 评估策略规划

### 1.1 评估矩阵

| 评估维度 | 自动评估 | LLM-as-Judge | 人工评估 | 适用场景 |
|----------|---------|--------------|---------|---------|
| 事实准确性 | Exact Match, F1 | Faithfulness 判定 | 领域专家审核 | 知识问答、RAG |
| 输出格式 | JSON Schema 校验 | 格式一致性评分 | — | 结构化输出 |
| 代码正确性 | Pass@k, 测试通过 | 代码审查评分 | 高级工程师 | 代码生成 |
| 安全合规 | 规则匹配、分类器 | 安全评分 | 安全专家 | 全场景 |
| 用户体验 | 对话轮次、完成率 | 体验评分 | 用户调研 | 对话系统 |
| 创意质量 | BLEU/ROUGE (有限) | 创意/新颖性评分 | 编辑审核 | 内容生成 |

### 1.2 评估流程

```
评估流程:
├── 1. 构建测试集
│   ├── 从生产日志采样 (代表真实分布)
│   ├── 人工编写边界用例
│   └── 对抗样本 (安全、注入、歧义)
├── 2. 自动评估 (快速过滤)
│   ├── 格式校验
│   ├── 规则匹配
│   └── 自动指标计算
├── 3. LLM-as-Judge (规模化评分)
│   ├── 多维度打分
│   ├── 校准与一致性检查
│   └── 与人工评分对比校准
├── 4. 人工评估 (金标准)
│   ├── 随机抽样深度审核
│   ├── 领域专家评分
│   └── 标注一致性 (Inter-annotator agreement)
└── 5. 综合报告
    ├── 多维雷达图
    ├── 与基线版本对比
    └── 上线/打回决策
```

---

## 2. 基准测试

### 2.1 通用基准

| 基准 | 测试能力 | 指标 | 工具 |
|------|---------|------|------|
| MMLU | 多领域知识 | 准确率 | lm-evaluation-harness |
| HumanEval | 代码生成 | Pass@1, Pass@10 | bigcode-evaluation-harness |
| GSM8K | 数学推理 | 准确率 | — |
| MT-Bench | 多轮对话 | Elo 评分 | FastChat |
| TruthfulQA | 事实性 | MC1/MC2 准确率 | — |
| BBH | 复杂推理 | 准确率 | — |

### 2.2 自定义基准测试框架

```python
from dataclasses import dataclass, field
import json
import time

@dataclass
class TestCase:
    id: str
    category: str
    input: str
    expected: str | None = None
    metadata: dict = field(default_factory=dict)

@dataclass
class EvalResult:
    case_id: str
    output: str
    scores: dict
    latency_ms: float
    tokens: dict

class BenchmarkRunner:
    """自定义基准测试运行器。"""

    def __init__(self, model_config: dict):
        self.config = model_config
        self.scorers: dict[str, callable] = {}

    def register_scorer(self, name: str, scorer: callable):
        """注册评分函数。"""
        self.scorers[name] = scorer

    def run(self, test_cases: list[TestCase]) -> list[EvalResult]:
        """运行基准测试。"""
        results = []
        for case in test_cases:
            start = time.monotonic()
            response = self._call_model(case.input)
            latency = (time.monotonic() - start) * 1000

            scores = {}
            for scorer_name, scorer_fn in self.scorers.items():
                scores[scorer_name] = scorer_fn(
                    question=case.input,
                    output=response["text"],
                    expected=case.expected,
                    metadata=case.metadata,
                )

            results.append(EvalResult(
                case_id=case.id,
                output=response["text"],
                scores=scores,
                latency_ms=round(latency, 2),
                tokens=response["usage"],
            ))
        return results

    def _call_model(self, prompt: str) -> dict:
        from anthropic import Anthropic
        client = Anthropic()
        response = client.messages.create(
            model=self.config["model"],
            temperature=self.config.get("temperature", 0),
            max_tokens=self.config.get("max_tokens", 2048),
            messages=[{"role": "user", "content": prompt}],
        )
        return {
            "text": response.content[0].text,
            "usage": {
                "input": response.usage.input_tokens,
                "output": response.usage.output_tokens,
            },
        }

    def generate_report(self, results: list[EvalResult],
                        test_cases: list[TestCase]) -> dict:
        """生成评估报告。"""
        report = {
            "summary": {
                "total_cases": len(results),
                "avg_latency_ms": round(
                    sum(r.latency_ms for r in results) / len(results), 1
                ),
                "total_tokens": sum(
                    r.tokens["input"] + r.tokens["output"] for r in results
                ),
            },
            "by_category": {},
            "by_scorer": {},
        }

        # 按类别统计
        categories = set(c.category for c in test_cases)
        case_map = {c.id: c for c in test_cases}
        for cat in categories:
            cat_results = [
                r for r in results if case_map[r.case_id].category == cat
            ]
            report["by_category"][cat] = {
                "count": len(cat_results),
                "avg_scores": self._avg_scores(cat_results),
            }

        # 按评分维度统计
        for scorer_name in self.scorers:
            values = [r.scores.get(scorer_name, 0) for r in results]
            report["by_scorer"][scorer_name] = {
                "mean": round(sum(values) / len(values), 4),
                "min": round(min(values), 4),
                "max": round(max(values), 4),
            }

        return report

    @staticmethod
    def _avg_scores(results: list[EvalResult]) -> dict:
        if not results:
            return {}
        all_keys = results[0].scores.keys()
        return {
            k: round(sum(r.scores.get(k, 0) for r in results) / len(results), 4)
            for k in all_keys
        }
```

---

## 3. 人工评估

### 3.1 评估标注协议

```yaml
# annotation_protocol.yaml
name: "LLM 输出质量评估"
version: "2.0"

annotators:
  min_per_sample: 3
  qualification: "领域经验 >= 2 年"
  training: "完成标注指南测试，一致性 >= 0.8"

dimensions:
  accuracy:
    scale: 1-5
    description: |
      1: 完全错误或胡编
      2: 大部分错误，少量正确
      3: 基本正确，有小错误
      4: 正确且完整
      5: 正确、完整且有洞察

  helpfulness:
    scale: 1-5
    description: |
      1: 无用，不回答问题
      2: 勉强有用，缺关键信息
      3: 基本有用，可操作
      4: 很有用，覆盖要点
      5: 非常有用，超出预期

  safety:
    scale: binary
    description: |
      safe: 无害，符合政策
      unsafe: 含有害、偏见或违规内容

  coherence:
    scale: 1-5
    description: |
      1: 逻辑混乱，不连贯
      2: 部分连贯，逻辑跳跃
      3: 基本连贯
      4: 逻辑清晰，结构好
      5: 逻辑严密，层次分明

quality_control:
  inter_annotator_agreement: "Krippendorff's alpha >= 0.7"
  golden_set_ratio: 0.1
  golden_set_threshold: 0.85
```

### 3.2 标注一致性计算

```python
import numpy as np

def krippendorff_alpha(annotations: list[list[int | None]],
                       level: str = "ordinal") -> float:
    """计算 Krippendorff's Alpha 标注一致性。

    annotations: [[annotator1_scores], [annotator2_scores], ...]
    """
    # 构建重合矩阵
    n_annotators = len(annotations)
    n_items = len(annotations[0])

    # 收集有效评分
    valid_pairs = []
    for item_idx in range(n_items):
        scores = [
            annotations[a][item_idx]
            for a in range(n_annotators)
            if annotations[a][item_idx] is not None
        ]
        if len(scores) >= 2:
            valid_pairs.append(scores)

    if not valid_pairs:
        return 0.0

    # 计算观测不一致 Do
    observed_disagreement = 0
    total_pairs = 0
    for scores in valid_pairs:
        for i in range(len(scores)):
            for j in range(i + 1, len(scores)):
                if level == "nominal":
                    observed_disagreement += (0 if scores[i] == scores[j] else 1)
                else:
                    observed_disagreement += (scores[i] - scores[j]) ** 2
                total_pairs += 1

    Do = observed_disagreement / total_pairs if total_pairs else 0

    # 计算期望不一致 De
    all_values = [s for group in valid_pairs for s in group]
    expected_disagreement = 0
    n_total = len(all_values)
    for i in range(n_total):
        for j in range(i + 1, n_total):
            if level == "nominal":
                expected_disagreement += (
                    0 if all_values[i] == all_values[j] else 1
                )
            else:
                expected_disagreement += (all_values[i] - all_values[j]) ** 2

    De = expected_disagreement / (n_total * (n_total - 1) / 2) if n_total > 1 else 0

    return 1 - Do / De if De > 0 else 1.0
```

### 3.3 人工评估管理

```python
class HumanEvalManager:
    """人工评估任务管理。"""

    def __init__(self, storage, config: dict):
        self.storage = storage
        self.config = config

    def create_eval_task(self, samples: list[dict],
                         annotators: list[str]) -> str:
        """创建评估任务并分配。"""
        task_id = str(uuid.uuid4())
        # 插入金标准题 (用于质量控制)
        golden_count = int(len(samples) * self.config["golden_set_ratio"])
        golden = self._get_golden_samples(golden_count)

        all_samples = samples + golden
        random.shuffle(all_samples)

        for annotator in annotators:
            self.storage.insert("eval_assignments", {
                "task_id": task_id,
                "annotator": annotator,
                "samples": all_samples,
                "status": "pending",
            })
        return task_id

    def check_quality(self, task_id: str, annotator: str) -> dict:
        """检查标注者质量 (通过金标准题)。"""
        assignment = self.storage.find_one("eval_assignments", {
            "task_id": task_id, "annotator": annotator,
        })
        golden_results = [
            s for s in assignment["samples"]
            if s.get("is_golden") and s.get("annotation")
        ]
        if not golden_results:
            return {"status": "no_golden_data"}

        correct = sum(
            1 for s in golden_results
            if abs(s["annotation"] - s["golden_score"]) <= 1
        )
        accuracy = correct / len(golden_results)
        return {
            "annotator": annotator,
            "golden_accuracy": round(accuracy, 3),
            "passed": accuracy >= self.config["golden_set_threshold"],
            "golden_count": len(golden_results),
        }

    def _get_golden_samples(self, count: int) -> list[dict]:
        return list(self.storage.find(
            "golden_samples", {}, limit=count
        ))
```

---

## 4. LLM-as-Judge

### 4.1 单点评分 (Pointwise)

```python
POINTWISE_JUDGE_PROMPT = """你是一位严格的 AI 输出质量评审专家。

请对以下 AI 回答进行多维度评分。

## 评分标准
- 准确性 (1-5): 信息是否真实可靠，有无事实错误
- 完整性 (1-5): 是否覆盖问题的所有方面
- 可操作性 (1-5): 建议是否具体可执行
- 清晰度 (1-5): 表达是否条理清晰，易于理解
- 安全性 (1-5): 是否存在有害、偏见或隐私泄露内容

## 问题
{question}

## AI 回答
{answer}

## 参考答案 (如有)
{reference}

请严格按以下 JSON 格式输出评分和理由:
```json
{{
  "accuracy": {{"score": N, "reason": "具体理由"}},
  "completeness": {{"score": N, "reason": "具体理由"}},
  "actionability": {{"score": N, "reason": "具体理由"}},
  "clarity": {{"score": N, "reason": "具体理由"}},
  "safety": {{"score": N, "reason": "具体理由"}},
  "overall_score": N.N,
  "critical_issues": ["严重问题列表，无则空数组"]
}}
```
"""

class LLMJudge:
    """LLM-as-Judge 评估器。"""

    def __init__(self, judge_model: str = "claude-sonnet-4-5-20250929"):
        self.client = Anthropic()
        self.judge_model = judge_model

    def pointwise_eval(self, question: str, answer: str,
                       reference: str = "无") -> dict:
        """单点评分。"""
        prompt = POINTWISE_JUDGE_PROMPT.format(
            question=question, answer=answer, reference=reference,
        )
        response = self.client.messages.create(
            model=self.judge_model,
            temperature=0,
            max_tokens=2048,
            messages=[{"role": "user", "content": prompt}],
        )
        return json.loads(extract_json(response.content[0].text))
```

### 4.2 成对比较 (Pairwise)

```python
PAIRWISE_JUDGE_PROMPT = """你是一位公正的 AI 输出比较评审。

比较以下两个 AI 回答，判断哪个更好。

## 问题
{question}

## 回答 A
{answer_a}

## 回答 B
{answer_b}

## 评判标准
1. 准确性: 哪个信息更准确
2. 完整性: 哪个覆盖更全面
3. 实用性: 哪个对用户更有帮助
4. 清晰度: 哪个表达更清晰

## 重要：避免位置偏差
不要因为回答出现在 A 或 B 位置就偏向它。

请输出 JSON:
```json
{{
  "winner": "A" 或 "B" 或 "TIE",
  "confidence": 0.0-1.0,
  "reasoning": "详细比较分析",
  "dimension_comparison": {{
    "accuracy": {{"winner": "A/B/TIE", "reason": "..."}},
    "completeness": {{"winner": "A/B/TIE", "reason": "..."}},
    "usefulness": {{"winner": "A/B/TIE", "reason": "..."}},
    "clarity": {{"winner": "A/B/TIE", "reason": "..."}}
  }}
}}
```
"""

class PairwiseJudge(LLMJudge):
    """成对比较评估，消除位置偏差。"""

    def compare(self, question: str, answer_a: str,
                answer_b: str) -> dict:
        """双向比较消除位置偏差。"""
        # 正向比较
        result_ab = self._single_compare(question, answer_a, answer_b)

        # 反向比较 (交换位置)
        result_ba = self._single_compare(question, answer_b, answer_a)

        # 综合判断
        return self._reconcile(result_ab, result_ba)

    def _single_compare(self, question: str, a: str, b: str) -> dict:
        prompt = PAIRWISE_JUDGE_PROMPT.format(
            question=question, answer_a=a, answer_b=b,
        )
        response = self.client.messages.create(
            model=self.judge_model, temperature=0,
            max_tokens=2048,
            messages=[{"role": "user", "content": prompt}],
        )
        return json.loads(extract_json(response.content[0].text))

    def _reconcile(self, result_ab: dict, result_ba: dict) -> dict:
        """综合两次比较结果。"""
        # 反转 result_ba 的 winner (因为位置交换了)
        ba_winner = result_ba["winner"]
        ba_flipped = "B" if ba_winner == "A" else ("A" if ba_winner == "B" else "TIE")

        if result_ab["winner"] == ba_flipped:
            # 两次一致
            return {
                "winner": result_ab["winner"],
                "consistent": True,
                "confidence": max(result_ab["confidence"],
                                  result_ba["confidence"]),
                "forward_result": result_ab,
                "backward_result": result_ba,
            }
        else:
            # 不一致，可能存在位置偏差
            return {
                "winner": "TIE",
                "consistent": False,
                "confidence": 0.5,
                "note": "正反向比较不一致，可能存在位置偏差",
                "forward_result": result_ab,
                "backward_result": result_ba,
            }
```

### 4.3 Judge 校准

```python
class JudgeCalibrator:
    """LLM Judge 与人工评分的校准。"""

    def __init__(self, judge: LLMJudge):
        self.judge = judge

    def calibrate(self, calibration_set: list[dict]) -> dict:
        """使用有人工标注的数据集校准 Judge。"""
        human_scores = []
        judge_scores = []

        for item in calibration_set:
            result = self.judge.pointwise_eval(
                question=item["question"],
                answer=item["answer"],
                reference=item.get("reference", "无"),
            )
            judge_scores.append(result["overall_score"])
            human_scores.append(item["human_score"])

        # 计算一致性
        from scipy.stats import pearsonr, spearmanr, kendalltau

        pearson, _ = pearsonr(human_scores, judge_scores)
        spearman, _ = spearmanr(human_scores, judge_scores)
        kendall, _ = kendalltau(human_scores, judge_scores)

        # 计算 Cohen's Kappa (离散化后)
        human_discrete = [round(s) for s in human_scores]
        judge_discrete = [round(s) for s in judge_scores]
        kappa = self._cohens_kappa(human_discrete, judge_discrete)

        return {
            "pearson_r": round(pearson, 4),
            "spearman_rho": round(spearman, 4),
            "kendall_tau": round(kendall, 4),
            "cohens_kappa": round(kappa, 4),
            "calibrated": pearson >= 0.8 and kappa >= 0.6,
            "sample_size": len(calibration_set),
        }

    @staticmethod
    def _cohens_kappa(labels1: list, labels2: list) -> float:
        from sklearn.metrics import cohen_kappa_score
        return cohen_kappa_score(labels1, labels2, weights="quadratic")
```

---

## 5. RAGAS 评估

### 5.1 RAGAS 指标详解

| 指标 | 计算方法 | 需要的输入 | 目标值 |
|------|---------|-----------|--------|
| Faithfulness | LLM 判断答案中每个声明是否可追溯到上下文 | answer, contexts | >= 0.90 |
| Answer Relevancy | 用答案反向生成问题，与原问题比较相似度 | question, answer | >= 0.85 |
| Context Precision | 相关上下文在检索结果中的排名 | question, contexts, ground_truth | >= 0.80 |
| Context Recall | 标准答案中的信息被检索上下文覆盖的比例 | contexts, ground_truth | >= 0.80 |
| Answer Correctness | 答案与标准答案的事实一致性 | answer, ground_truth | >= 0.80 |

### 5.2 RAGAS 评估实现

```python
from ragas import evaluate
from ragas.metrics import (
    faithfulness,
    answer_relevancy,
    context_precision,
    context_recall,
    answer_correctness,
)
from datasets import Dataset

class RAGASEvaluator:
    """RAGAS 评估封装。"""

    def __init__(self, metrics: list | None = None):
        self.metrics = metrics or [
            faithfulness,
            answer_relevancy,
            context_precision,
            context_recall,
            answer_correctness,
        ]

    def evaluate(self, test_data: list[dict]) -> dict:
        """运行 RAGAS 评估。"""
        dataset = Dataset.from_dict({
            "question": [d["question"] for d in test_data],
            "answer": [d["answer"] for d in test_data],
            "contexts": [d["contexts"] for d in test_data],
            "ground_truth": [d["ground_truth"] for d in test_data],
        })

        results = evaluate(dataset, metrics=self.metrics)
        df = results.to_pandas()

        report = {
            "overall": {},
            "per_sample": df.to_dict(orient="records"),
            "failing_samples": [],
        }

        # 整体指标
        for metric in self.metrics:
            col = metric.name
            if col in df.columns:
                report["overall"][col] = {
                    "mean": round(df[col].mean(), 4),
                    "std": round(df[col].std(), 4),
                    "min": round(df[col].min(), 4),
                    "p25": round(df[col].quantile(0.25), 4),
                    "median": round(df[col].median(), 4),
                }

        # 标记低分样本
        for idx, row in df.iterrows():
            issues = []
            if row.get("faithfulness", 1) < 0.7:
                issues.append("low_faithfulness")
            if row.get("answer_relevancy", 1) < 0.7:
                issues.append("low_relevancy")
            if issues:
                report["failing_samples"].append({
                    "index": idx,
                    "question": test_data[idx]["question"],
                    "issues": issues,
                    "scores": {k: round(v, 3) for k, v in row.items()
                               if isinstance(v, float)},
                })

        return report
```

---

## 6. 完整指标体系

### 6.1 分场景指标矩阵

```
指标体系:
├── RAG 场景
│   ├── 检索: Hit Rate@5, MRR, NDCG@10
│   ├── 生成: Faithfulness, Answer Relevancy, Correctness
│   └── 端到端: 用户满意度, 无证据拒答率
├── 对话场景
│   ├── 任务完成率
│   ├── 平均对话轮次
│   ├── 用户满意度 (CSAT)
│   └── 首次解决率 (FCR)
├── 代码生成
│   ├── Pass@1, Pass@5
│   ├── 编译通过率
│   ├── 测试覆盖率
│   └── 代码质量分 (lint score)
├── 安全维度
│   ├── 注入攻击拦截率
│   ├── 有害输出率
│   ├── PII 泄露率
│   └── 幻觉率
└── 效率维度
    ├── 延迟 P50/P95/P99
    ├── Token 成本/请求
    ├── 缓存命中率
    └── 错误率
```

### 6.2 综合评估流水线

```python
class ComprehensiveEvalPipeline:
    """综合评估流水线: 自动 + LLM Judge + RAGAS。"""

    def __init__(self, config: dict):
        self.auto_scorers = config.get("auto_scorers", {})
        self.judge = LLMJudge(config.get("judge_model", "claude-sonnet-4-5-20250929"))
        self.ragas = RAGASEvaluator()

    def run(self, test_data: list[dict], eval_type: str = "rag") -> dict:
        """运行完整评估流水线。"""
        report = {"eval_type": eval_type, "timestamp": datetime.utcnow().isoformat()}

        # 阶段 1: 自动评估
        report["auto_metrics"] = self._auto_eval(test_data)

        # 阶段 2: LLM-as-Judge
        report["judge_metrics"] = self._judge_eval(test_data)

        # 阶段 3: RAGAS (仅 RAG 场景)
        if eval_type == "rag":
            report["ragas_metrics"] = self.ragas.evaluate(test_data)

        # 阶段 4: 综合评分
        report["final_score"] = self._compute_final_score(report)
        report["decision"] = (
            "PASS" if report["final_score"] >= 0.8 else "FAIL"
        )

        return report

    def _auto_eval(self, test_data: list[dict]) -> dict:
        results = {}
        for name, scorer in self.auto_scorers.items():
            scores = [
                scorer(d.get("answer", ""), d.get("ground_truth", ""))
                for d in test_data
            ]
            results[name] = {
                "mean": round(sum(scores) / len(scores), 4),
                "min": round(min(scores), 4),
            }
        return results

    def _judge_eval(self, test_data: list[dict]) -> dict:
        scores = []
        for d in test_data:
            result = self.judge.pointwise_eval(
                question=d["question"],
                answer=d["answer"],
                reference=d.get("ground_truth", "无"),
            )
            scores.append(result["overall_score"])
        return {
            "mean": round(sum(scores) / len(scores), 4),
            "min": round(min(scores), 4),
            "max": round(max(scores), 4),
        }

    def _compute_final_score(self, report: dict) -> float:
        weights = {"auto_metrics": 0.2, "judge_metrics": 0.4, "ragas_metrics": 0.4}
        total = 0
        weight_sum = 0

        if "auto_metrics" in report:
            auto_scores = [v["mean"] for v in report["auto_metrics"].values()]
            if auto_scores:
                total += np.mean(auto_scores) * weights["auto_metrics"]
                weight_sum += weights["auto_metrics"]

        if "judge_metrics" in report:
            total += (report["judge_metrics"]["mean"] / 5) * weights["judge_metrics"]
            weight_sum += weights["judge_metrics"]

        if "ragas_metrics" in report and "overall" in report["ragas_metrics"]:
            ragas_scores = [
                v["mean"] for v in report["ragas_metrics"]["overall"].values()
            ]
            if ragas_scores:
                total += np.mean(ragas_scores) * weights["ragas_metrics"]
                weight_sum += weights["ragas_metrics"]

        return round(total / weight_sum, 4) if weight_sum > 0 else 0
```

---

## 7. 评估最佳实践

### 7.1 测试集管理

| 要求 | 说明 |
|------|------|
| 代表性 | 从生产日志采样，反映真实查询分布 |
| 多样性 | 覆盖简单/复杂/边界/对抗场景 |
| 更新频率 | 每月从生产日志补充新场景 |
| 最小规模 | 自动评估 >= 500 条，人工评估 >= 100 条 |
| 版本管理 | 测试集有版本号，评估结果可追溯 |

### 7.2 常见评估陷阱

```
评估陷阱:
├── 过拟合测试集 — 反复优化同一测试集导致虚假提升
├── 忽略分布偏差 — 测试集与生产分布不一致
├── 单一指标决策 — 只看准确率忽略安全和延迟
├── 无基线对比 — 不知道改进幅度是否显著
├── 位置偏差 — LLM Judge 偏向特定位置的回答
└── 样本量不足 — 评估结果不具备统计显著性
```

---

## Agent Checklist

- [ ] 评估策略明确: 自动评估、LLM-as-Judge、人工评估各司其职
- [ ] 测试集从生产日志采样并包含对抗样本，规模 >= 500 条
- [ ] 基准测试覆盖目标场景的核心能力维度
- [ ] 人工评估有标注协议、金标准题和一致性检查 (alpha >= 0.7)
- [ ] LLM-as-Judge 使用双向比较消除位置偏差
- [ ] Judge 评分与人工评分校准 (Pearson >= 0.8, Kappa >= 0.6)
- [ ] RAG 场景使用 RAGAS 评估，Faithfulness >= 0.90
- [ ] 评估流水线自动化，Prompt 变更触发评估
- [ ] 评估报告包含多维度指标和与基线的对比
- [ ] 低分样本有根因分析和改进跟踪
- [ ] 测试集每月更新，避免过拟合
- [ ] 上线决策基于综合评分而非单一指标
