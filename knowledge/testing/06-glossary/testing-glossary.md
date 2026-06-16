---
id: testing-glossary
title: 测试术语表 (Testing Glossary)
domain: testing
category: 06-glossary
difficulty: intermediate
tags: [agent, checklist, glossary, testing, 测试度量, 测试方法论, 测试替身与隔离, 测试环境与基础设施]
quality_score: 70
last_updated: 2026-06-15
---
# 测试术语表 (Testing Glossary)

> 适用场景：测试策略制定、测试方案评审、QA 团队 Onboarding、开发团队测试培训。
> 涵盖范围：测试方法论、测试类型、测试工具概念、测试度量指标等核心术语。

---

## 测试方法论

| 术语 | 英文 | 定义 |
|------|------|------|
| 测试驱动开发 | Test-Driven Development (TDD) | 先写测试再写实现的开发方法。循环：Red（写失败测试）→ Green（写最小实现使测试通过）→ Refactor（重构代码保持测试通过）。 |
| 行为驱动开发 | Behavior-Driven Development (BDD) | 基于业务行为描述编写测试的方法，使用 Given-When-Then 格式，促进开发、测试和业务的沟通。工具：Cucumber / Behave / Jest-Cucumber。 |
| 验收测试驱动开发 | Acceptance Test-Driven Development (ATDD) | 在开发前由产品、开发和测试共同定义验收标准和验收测试，确保交付符合业务需求。 |
| 契约测试 | Contract Testing | 验证服务提供者和消费者之间接口契约一致性的测试方法，防止接口变更导致集成失败。工具：Pact。 |
| 基于属性的测试 | Property-Based Testing | 定义输入数据的属性和输出应满足的不变量，框架自动生成大量随机输入进行验证。工具：Hypothesis (Python) / fast-check (JS)。 |

---

## 测试类型

| 术语 | 英文 | 定义 |
|------|------|------|
| 单元测试 | Unit Test | 测试最小可测试单元（函数/方法/类）的独立行为，不依赖外部系统。执行速度快（毫秒级），数量最多。 |
| 集成测试 | Integration Test | 验证多个模块或服务之间的交互是否正确，通常涉及数据库、消息队列等真实或模拟的外部依赖。 |
| 端到端测试 | End-to-End Test (E2E) | 模拟真实用户操作，验证完整业务流程从前端到后端到数据库的全链路功能。工具：Playwright / Cypress / Selenium。 |
| 回归测试 | Regression Test | 在代码变更后重新执行已有测试，确保新变更没有破坏已有功能。通常自动化执行。 |
| 冒烟测试 | Smoke Test | 部署后快速验证系统核心功能是否可用的最小测试集，通常在 CI/CD 部署后自动执行。 |
| 金丝雀测试 | Canary Test | 在生产环境中针对小比例流量运行的测试，验证新版本在真实条件下是否正常，异常则自动回滚。 |
| 压力测试 | Stress Test | 在超出正常负载的条件下测试系统行为，找到系统的极限和断裂点。 |
| 负载测试 | Load Test | 在预期负载条件下验证系统性能指标（响应时间、吞吐量）是否满足要求。工具：k6 / Locust / JMeter。 |
| 性能测试 | Performance Test | 测试系统在不同条件下的响应速度、吞吐量、资源利用率等性能指标的总称。 |
| 安全测试 | Security Test | 识别系统安全漏洞的测试，包括渗透测试、SAST、DAST、依赖扫描等。 |
| 可访问性测试 | Accessibility Test (a11y) | 验证应用是否对残障用户友好，符合 WCAG 标准。工具：axe / Lighthouse / Pa11y。 |
| 视觉回归测试 | Visual Regression Test | 通过截图对比检测 UI 的非预期视觉变化。工具：Percy / Chromatic / BackstopJS。 |
| 混沌测试 | Chaos Test | 主动向系统注入故障（网络延迟、服务宕机、磁盘满）验证系统的容错和恢复能力。工具：Chaos Monkey / Litmus。 |

---

## 测试替身与隔离

| 术语 | 英文 | 定义 |
|------|------|------|
| 测试替身 | Test Double | 替代真实依赖的测试用对象的总称，包括 Dummy、Stub、Spy、Mock、Fake 五种类型。 |
| Mock | Mock | 预设期望行为并验证交互的测试替身。可以断言被调用的次数、参数等。工具：unittest.mock / Jest mock / Mockito。 |
| Stub | Stub | 返回预设固定值的测试替身，不验证交互。用于隔离被测代码的外部依赖。 |
| Spy | Spy | 包装真实对象的测试替身，记录调用信息但仍执行真实逻辑。用于验证交互同时保留真实行为。 |
| Fake | Fake | 具有简化但可工作实现的测试替身。如内存数据库替代 PostgreSQL、本地文件系统替代 S3。 |
| Fixture | Fixture | 测试运行前的预设环境和数据，包括数据库记录、配置文件、临时目录等。通过 `setUp` / `@pytest.fixture` 管理。 |
| Test Factory | Test Factory | 创建测试数据的工厂函数或类，提供合理的默认值并允许按需覆盖。工具：Factory Boy (Python) / Fishery (JS)。 |

---

## 测试度量

| 术语 | 英文 | 定义 |
|------|------|------|
| 代码覆盖率 | Code Coverage | 测试执行时覆盖的代码比例，包括行覆盖率、分支覆盖率、函数覆盖率等。工具：coverage.py / Istanbul / JaCoCo。 |
| 分支覆盖率 | Branch Coverage | 测试覆盖的条件分支比例（if/else、switch/case），比行覆盖率更严格。 |
| 变异测试 | Mutation Testing | 自动修改源代码（如将 `>` 改为 `>=`），验证测试是否能检测到变异。存活的变异体说明测试不够充分。工具：mutmut / Stryker。 |
| Flaky Test | Flaky Test | 不稳定测试，在相同代码上时而通过时而失败。常见原因：时间依赖、竞态条件、外部服务、测试顺序依赖。 |
| 测试金字塔 | Test Pyramid | 测试分布的理想模型：底部大量快速的单元测试，中间适量集成测试，顶部少量 E2E 测试。由 Mike Cohn 提出。 |
| 测试冰淇淋锥 | Test Ice Cream Cone | 测试金字塔的反模式：大量 E2E/手动测试在顶部，少量单元测试在底部。导致测试慢、脆弱、维护成本高。 |

---

## 高级测试技术

| 术语 | 英文 | 定义 |
|------|------|------|
| 模糊测试 | Fuzzing | 向程序输入大量随机或半随机数据，检测崩溃、内存泄漏和安全漏洞。工具：AFL / libFuzzer / Atheris。 |
| 快照测试 | Snapshot Testing | 将组件输出序列化为快照文件，后续测试对比输出是否发生变化。适用于 UI 组件和 API 响应。工具：Jest Snapshot。 |
| 影子测试 | Shadow Testing | 将生产流量复制到新版本服务，对比新旧版本的输出差异，不影响用户。 |
| 数据驱动测试 | Data-Driven Testing | 测试逻辑与测试数据分离，通过外部数据源（CSV/JSON/数据库）驱动测试执行，同一测试逻辑覆盖大量场景。 |
| 参数化测试 | Parameterized Test | 同一测试函数使用不同参数多次执行，避免重复编写结构相同的测试。`@pytest.mark.parametrize` / `@ParameterizedTest`。 |
| 测试左移 | Shift-Left Testing | 将测试活动提前到开发生命周期的更早阶段（需求、设计），而非等到编码完成后才开始测试。 |
| 测试右移 | Shift-Right Testing | 在生产环境中进行测试（如 Canary 测试、A/B 测试、混沌工程），验证系统在真实条件下的行为。 |
| 契约测试 | Contract Testing | 验证服务间接口契约一致性的测试方法。消费者定义期望，提供者验证满足。工具：Pact / Spring Cloud Contract。 |

---

## 测试环境与基础设施

| 术语 | 英文 | 定义 |
|------|------|------|
| 测试环境 | Test Environment | 用于运行测试的独立环境，与生产环境隔离。包括本地环境、CI 环境、Staging 环境等。 |
| 测试容器 | Testcontainers | 使用 Docker 容器为集成测试提供真实依赖（数据库、消息队列等）的库，测试后自动清理。 |
| 测试数据管理 | Test Data Management | 创建、维护和清理测试所需数据的策略和工具。包括 Fixture、Factory、Seed Data、数据脱敏等。 |
| 持续测试 | Continuous Testing | 在 CI/CD 流水线中自动化执行测试的实践，每次代码提交都触发测试，快速反馈质量状态。 |
| 服务虚拟化 | Service Virtualization | 模拟外部依赖服务（第三方 API、后端服务）行为的技术，使测试不依赖真实外部系统。工具：WireMock / MockServer。 |
| 测试隔离 | Test Isolation | 确保每个测试独立运行，不受其他测试的影响。通过独立数据、事务回滚、容器隔离等方式实现。 |

---

## Agent Checklist

- [ ] 项目测试方案中使用的术语与本表定义一致
- [ ] QA 和开发团队对核心术语的理解已对齐
- [ ] 新引入的测试方法/工具已补充到本术语表
- [ ] 测试评审时使用本表作为沟通基准
