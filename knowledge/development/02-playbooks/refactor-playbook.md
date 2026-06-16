---
id: refactor-playbook
title: 重构作战手册 (Refactoring Playbook)
domain: development
category: 02-playbooks
difficulty: intermediate
tags: [agent, checklist, development, playbook, refactor, 前置条件, 回滚方案, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# 重构作战手册 (Refactoring Playbook)

## 概述

重构是在不改变外部行为的前提下改善代码内部结构的系统化过程。本手册覆盖从债务识别、风险评估、安全重构到验证上线的全流程，适用于遗留系统改造、架构升级和持续改善场景。

## 前置条件

### 必须满足

- [ ] 目标模块有 >= 60% 的自动化测试覆盖率（核心路径 >= 80%）
- [ ] 建立了可观测基线：关键 API 响应时间、错误率、吞吐量
- [ ] CI/CD 流水线正常运行且包含回归测试
- [ ] 已获得技术负责人或架构师的重构方案审批
- [ ] 团队有明确的回滚策略和回滚触发条件

### 建议满足

- [ ] 有代码复杂度分析报告（如 radon、SonarQube）
- [ ] 有最近 6 个月的缺陷密度和变更频率数据
- [ ] 相关依赖方已知晓重构计划

---

## 步骤一：债务识别与优先级排序

### 1.1 收集指标

```bash
# Python 项目 - 圈复杂度分析
radon cc src/ -a -nc -s -j > reports/complexity.json
radon mi src/ -s -j > reports/maintainability.json

# 重复代码检测
pylint --disable=all --enable=duplicate-code src/ > reports/duplicates.txt

# Git 变更热点分析 - 找出变更最频繁的文件
git log --since="6 months ago" --pretty=format: --name-only | \
  sort | uniq -c | sort -rg | head -30 > reports/churn.txt

# 缺陷关联分析 - 哪些文件关联最多的 bug fix
git log --since="6 months ago" --grep="fix" --pretty=format: --name-only | \
  sort | uniq -c | sort -rg | head -20 > reports/bug-hotspots.txt
```

### 1.2 债务分类矩阵

| 类型 | 识别信号 | 影响维度 | 优先级权重 |
|------|----------|----------|-----------|
| 结构性债务 | 循环依赖、上帝类、深层嵌套 | 可维护性、扩展性 | 高 |
| 认知性债务 | 命名混乱、魔数、缺少注释 | 开发效率、新人上手 | 中 |
| 测试性债务 | 难以测试、测试脆弱 | 质量、发布速度 | 高 |
| 性能性债务 | N+1 查询、内存泄漏 | 用户体验、成本 | 视影响 |
| 安全性债务 | 过时依赖、硬编码凭证 | 安全合规 | 最高 |

### 1.3 优先级评分

```
重构优先级 = (变更频率 × 0.3) + (缺陷密度 × 0.3) + (复杂度 × 0.2) + (业务影响 × 0.2)
```

输出物：`refactor-backlog.md`，按优先级排列的重构清单。

---

## 步骤二：建立安全护栏

### 2.1 补充测试

```python
# 对即将重构的模块补充特征化测试 (Characterization Tests)
# 目的：捕获当前行为作为基线，而非验证正确性

import pytest
from your_module import LegacyProcessor

class TestLegacyProcessorCharacterization:
    """记录现有行为，确保重构不改变外部表现"""

    def test_normal_input_output(self):
        processor = LegacyProcessor()
        result = processor.process({"type": "order", "amount": 100})
        # 记录当前实际输出，不判断对错
        assert result == {"status": "ok", "total": 100.0, "tax": 13.0}

    def test_edge_case_empty_input(self):
        processor = LegacyProcessor()
        result = processor.process({})
        assert result == {"status": "error", "code": "INVALID_INPUT"}

    def test_boundary_values(self):
        processor = LegacyProcessor()
        # 记录边界行为
        assert processor.process({"amount": 0}) == {"status": "ok", "total": 0.0, "tax": 0.0}
        assert processor.process({"amount": -1}) == {"status": "error", "code": "NEGATIVE_AMOUNT"}
```

### 2.2 建立可观测基线

```bash
# 记录重构前的性能基线
wrk -t12 -c400 -d30s http://localhost:8080/api/target-endpoint > baseline_perf.txt

# 记录内存和 CPU 基线
python -c "
import psutil, json, time
metrics = []
for _ in range(60):
    metrics.append({
        'cpu': psutil.cpu_percent(),
        'memory': psutil.virtual_memory().percent,
        'timestamp': time.time()
    })
    time.sleep(1)
with open('baseline_resources.json', 'w') as f:
    json.dump(metrics, f)
"
```

### 2.3 特性开关

```python
# 使用特性开关实现渐进式切换
from feature_flags import FeatureFlag

class OrderService:
    def calculate_total(self, order):
        if FeatureFlag.is_enabled("use_new_calculator", default=False):
            return self._new_calculate(order)
        return self._legacy_calculate(order)

    def _legacy_calculate(self, order):
        # 旧逻辑保留
        ...

    def _new_calculate(self, order):
        # 重构后逻辑
        ...
```

---

## 步骤三：安全重构执行

### 3.1 重构模式库

#### 提取方法 (Extract Method)

```python
# 重构前 - 方法过长
class ReportGenerator:
    def generate(self, data):
        # 30 行数据清洗逻辑
        cleaned = ...
        # 20 行聚合逻辑
        aggregated = ...
        # 25 行格式化逻辑
        formatted = ...
        return formatted

# 重构后 - 职责清晰
class ReportGenerator:
    def generate(self, data):
        cleaned = self._clean_data(data)
        aggregated = self._aggregate(cleaned)
        return self._format_report(aggregated)

    def _clean_data(self, data):
        ...

    def _aggregate(self, cleaned_data):
        ...

    def _format_report(self, aggregated_data):
        ...
```

#### 替换条件为多态 (Replace Conditional with Polymorphism)

```python
# 重构前 - 大量 if/elif
def calculate_price(product_type, base_price):
    if product_type == "standard":
        return base_price
    elif product_type == "premium":
        return base_price * 1.5
    elif product_type == "enterprise":
        return base_price * 2.0 + 500
    # ... 更多分支

# 重构后 - 策略模式
from abc import ABC, abstractmethod

class PricingStrategy(ABC):
    @abstractmethod
    def calculate(self, base_price: float) -> float: ...

class StandardPricing(PricingStrategy):
    def calculate(self, base_price: float) -> float:
        return base_price

class PremiumPricing(PricingStrategy):
    def calculate(self, base_price: float) -> float:
        return base_price * 1.5

PRICING_STRATEGIES = {
    "standard": StandardPricing(),
    "premium": PremiumPricing(),
}

def calculate_price(product_type: str, base_price: float) -> float:
    strategy = PRICING_STRATEGIES.get(product_type)
    if not strategy:
        raise ValueError(f"Unknown product type: {product_type}")
    return strategy.calculate(base_price)
```

#### 引入中间层消除循环依赖

```python
# 重构前 - A 和 B 循环依赖
# module_a.py: from module_b import B
# module_b.py: from module_a import A

# 重构后 - 引入接口层
# interfaces.py
from abc import ABC, abstractmethod

class Notifier(ABC):
    @abstractmethod
    def notify(self, event: dict) -> None: ...

# module_a.py - 依赖接口而非具体实现
class A:
    def __init__(self, notifier: Notifier):
        self.notifier = notifier

# module_b.py - 实现接口
class B(Notifier):
    def notify(self, event: dict) -> None:
        ...
```

### 3.2 小步提交策略

```bash
# 每个重构步骤单独提交，便于定位问题和回滚
git commit -m "refactor: extract data cleaning into _clean_data method"
git commit -m "refactor: extract aggregation into _aggregate method"
git commit -m "refactor: extract formatting into _format_report method"
# 不要把多个重构步骤合并到一个提交
```

### 3.3 并行验证 (Shadow Mode)

```python
import logging
import json

logger = logging.getLogger("refactor.shadow")

class ShadowComparator:
    """并行运行新旧逻辑并比较结果"""

    def execute_with_shadow(self, legacy_fn, new_fn, *args, **kwargs):
        legacy_result = legacy_fn(*args, **kwargs)
        try:
            new_result = new_fn(*args, **kwargs)
            if legacy_result != new_result:
                logger.warning(
                    "Shadow mismatch: legacy=%s, new=%s, args=%s",
                    json.dumps(legacy_result, default=str),
                    json.dumps(new_result, default=str),
                    json.dumps(args, default=str),
                )
        except Exception as e:
            logger.error("Shadow execution failed: %s", e)
        # 始终返回旧逻辑结果
        return legacy_result
```

---

## 步骤四：渐进式上线

### 4.1 灰度放量策略

```yaml
# 灰度放量阶段
phase_1:
  traffic_percentage: 1%
  duration: 24h
  rollback_trigger: error_rate > 0.1%
phase_2:
  traffic_percentage: 10%
  duration: 48h
  rollback_trigger: error_rate > 0.05%
phase_3:
  traffic_percentage: 50%
  duration: 72h
  rollback_trigger: p99_latency > baseline * 1.2
phase_4:
  traffic_percentage: 100%
  duration: 168h  # 7 天观察期
  rollback_trigger: any_anomaly
```

### 4.2 监控核查清单

```bash
# 核心指标检查脚本
#!/bin/bash
echo "=== 重构后健康检查 ==="
echo "1. 错误率变化:"
curl -s 'http://prometheus:9090/api/v1/query?query=rate(http_errors_total[5m])' | jq '.data.result[0].value[1]'

echo "2. P99 延迟变化:"
curl -s 'http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99,rate(http_duration_seconds_bucket[5m]))' | jq '.data.result[0].value[1]'

echo "3. 吞吐量变化:"
curl -s 'http://prometheus:9090/api/v1/query?query=rate(http_requests_total[5m])' | jq '.data.result[0].value[1]'
```

---

## 步骤五：沉淀与收尾

### 5.1 更新文档

- 更新架构图（如果模块边界变化）
- 更新 API 文档（如果接口签名变化）
- 在 ADR (Architecture Decision Record) 中记录重构决策和理由

### 5.2 清理工作

```bash
# 移除特性开关（重构稳定后）
# 移除旧代码路径
# 移除 Shadow Mode 比较逻辑
# 更新监控告警阈值
```

### 5.3 知识回写

- 总结遇到的反模式，补充到 `04-antipatterns/`
- 总结有效的重构模式，补充到本手册
- 更新编码规范（如果发现新的约束）

---

## 回滚方案

### 快速回滚

```bash
# 通过特性开关即时回滚
curl -X PUT http://feature-flags/api/flags/use_new_calculator \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'

# 通过 Git 回滚（如果没有特性开关）
git revert <refactor-commit-hash>
git push origin main
```

### 回滚触发条件

| 指标 | 阈值 | 动作 |
|------|------|------|
| 错误率 | > 基线 2 倍 | 立即回滚 |
| P99 延迟 | > 基线 1.5 倍 | 立即回滚 |
| 核心测试失败 | 任意失败 | 阻断发布 |
| Shadow 不一致率 | > 1% | 暂停放量，排查 |

---

## Agent Checklist

AI 编码 Agent 在执行重构任务时必须逐项确认：

- [ ] **债务识别完成**：已运行复杂度分析和变更热点分析，明确重构目标
- [ ] **测试护栏就绪**：目标模块测试覆盖率 >= 60%，核心路径 >= 80%
- [ ] **基线已记录**：性能基线和行为基线均有数据存档
- [ ] **方案已审批**：重构方案经技术负责人确认
- [ ] **特性开关就绪**：涉及行为变更的重构已配置特性开关
- [ ] **小步提交**：每个提交仅包含一个逻辑步骤的重构
- [ ] **测试持续通过**：每次提交后所有测试绿色
- [ ] **Shadow 验证**：新旧逻辑并行比较无不一致
- [ ] **灰度放量**：按 1% -> 10% -> 50% -> 100% 阶段放量
- [ ] **监控确认**：错误率、延迟、吞吐量均在基线范围内
- [ ] **旧代码清理**：重构稳定后移除旧路径和特性开关
- [ ] **文档更新**：架构图、API 文档、ADR 已同步更新
- [ ] **知识沉淀**：经验回写到标准和清单
