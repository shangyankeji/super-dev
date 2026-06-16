---
id: cicd-glossary
title: CI/CD 术语表 (CI/CD Glossary)
domain: cicd
category: 06-glossary
difficulty: intermediate
tags: [agent, checklist, cicd, glossary, 分支模型, 构建策略, 流水线基础, 版本管理]
quality_score: 70
last_updated: 2026-06-15
---
# CI/CD 术语表 (CI/CD Glossary)

> 收录 40+ 核心 CI/CD 术语，覆盖流水线基础、构建策略、部署策略、分支模型和版本管理等领域。
> 适用于 DevOps 评审、Pipeline 设计、团队培训等场景。

---

## 流水线基础

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| Pipeline | Pipeline / 流水线 | 将代码从提交到部署的完整自动化流程。由多个 Stage 组成，按顺序或并行执行。Pipeline 的设计质量直接决定团队交付效率和发布可靠性。 |
| Stage | Stage / 阶段 | Pipeline 中的逻辑分组，如 build、test、deploy。同一 Stage 内的 Job 可并行执行，Stage 之间按顺序执行。典型 Stage 链：lint → build → test → security → deploy。 |
| Job | Job / 任务 | Pipeline 中最小的可执行单元，在一个 Runner 上运行。每个 Job 包含一组 Step/Script。Job 可以设置依赖关系、条件触发和超时策略。 |
| Step | Step / 步骤 | Job 内部的单个命令或 Action。Step 按顺序执行，任一 Step 失败会导致 Job 失败（除非标记为 `continue-on-error`）。 |
| Runner | Runner / 执行器 | 执行 Pipeline Job 的计算环境。可以是 CI 平台提供的云端 Runner，也可以是用户自建的 Self-Hosted Runner。Runner 的性能直接影响构建速度。 |
| Trigger | Trigger / 触发器 | 启动 Pipeline 的事件源。常见触发器：push、pull_request、schedule（定时）、manual（手动）、API 调用、上游 Pipeline 完成。 |
| Artifact | Artifact / 制品 | 构建过程产生的输出文件（二进制包、Docker 镜像、测试报告等）。制品可在 Job 之间传递，也可上传到制品仓库供部署使用。 |
| Cache | Cache / 缓存 | 在 Pipeline 运行之间持久化的文件（如 `node_modules`、`.m2` 目录），用于加速后续构建。与 Artifact 的区别：Cache 用于加速，Artifact 用于传递产出物。 |

---

## 构建策略

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| Matrix Build | Matrix Build / 矩阵构建 | 使用参数组合自动生成多个 Job 实例的构建策略。例如：`os: [ubuntu, macos]` × `node: [18, 20]` 生成 4 个 Job，确保在多种环境下通过测试。 |
| Incremental Build | Incremental Build / 增量构建 | 仅编译和测试受代码变更影响的部分，而非全量构建。通过依赖图分析（如 Nx、Turborepo、Bazel）实现，可将构建时间减少 50-90%。 |
| Reusable Workflow | Reusable Workflow / 可复用工作流 | 可被其他 Pipeline 调用的模板化工作流（GitHub Actions 的 `workflow_call`、GitLab 的 `include`）。避免在多个仓库中重复相同的 CI 配置。 |
| Composite Action | Composite Action / 组合 Action | GitHub Actions 中将多个 Step 封装为一个可复用 Action 的方式。比 Reusable Workflow 更轻量，适合封装常见的构建/测试/部署片段。 |
| Self-Hosted Runner | Self-Hosted Runner / 自建执行器 | 用户自行部署和管理的 Pipeline 执行器。优势：可访问内网资源、更大的计算资源、持久化缓存。劣势：需自行维护安全性和可用性。 |
| Build Cache | Build Cache / 构建缓存 | 存储构建中间产物（编译结果、依赖包）以加速后续构建的技术。包括 Docker Layer Cache、npm/pip 缓存、编译器增量编译缓存等。 |

---

## 部署策略

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| Blue-Green Deployment | Blue-Green Deployment / 蓝绿部署 | 维护两套完全相同的生产环境（Blue 和 Green），新版本部署到非活跃环境，验证通过后通过流量切换（如负载均衡器）将用户导向新环境。优势：零停机、秒级回滚。劣势：资源成本翻倍。 |
| Canary Deployment | Canary Deployment / 金丝雀部署 | 先将新版本部署到一小部分实例（如 5%），观察关键指标（错误率、延迟、业务指标）无异常后逐步扩大比例（5% → 25% → 50% → 100%）。比蓝绿部署更节省资源，但实现更复杂。 |
| Rolling Update | Rolling Update / 滚动更新 | 逐步用新版本实例替换旧版本实例，过程中新旧版本共存。Kubernetes 的默认部署策略。通过 `maxSurge` 和 `maxUnavailable` 参数控制更新速度和可用性。 |
| A/B Testing Deployment | A/B Testing / A/B 测试部署 | 基于用户特征（地区、设备、用户分群）将流量路由到不同版本，用于验证功能效果。与 Canary 的区别：A/B 是按用户特征分流，Canary 是按比例随机分流。 |
| Feature Flag | Feature Flag / 功能开关 | 通过配置（非代码部署）控制功能的开启和关闭。允许代码已部署但功能未对用户可见，实现部署与发布解耦。常用平台：LaunchDarkly、Unleash、Flagsmith。 |
| GitOps | GitOps | 以 Git 仓库为单一可信源管理基础设施和应用部署的实践。变更通过 PR 提交到 Git，自动化工具（ArgoCD、Flux）监听并同步到集群。核心原则：声明式、版本化、自动化、自愈。 |
| Immutable Deployment | Immutable Deployment / 不可变部署 | 每次部署创建全新的基础设施实例而非原地更新。部署后的实例不可修改，需要变更时创建新实例并销毁旧实例。消除配置漂移和"雪花服务器"问题。 |

---

## 分支模型

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| Trunk-Based Development | Trunk-Based Development / 主干开发 | 所有开发者直接在 main/trunk 分支提交（或使用极短生命周期的 feature 分支），保持主干始终可发布。要求高质量的自动化测试和 Feature Flag 支持。适合持续部署场景。 |
| Git Flow | Git Flow | Vincent Driessen 提出的分支模型：`main`（生产）、`develop`（开发主线）、`feature/*`（功能分支）、`release/*`（发布准备）、`hotfix/*`（紧急修复）。适合有固定发布周期的项目，但分支管理复杂度较高。 |
| GitHub Flow | GitHub Flow | 简化的分支模型：只有 `main` 分支和 feature 分支。开发在 feature 分支进行，通过 PR 合并到 main，合并后立即部署。比 Git Flow 简单，适合持续交付团队。 |
| Release Branch | Release Branch / 发布分支 | 从主干切出的专用分支，用于发布前的最终测试和 Bug 修复。发布完成后合并回主干并打 Tag。在需要支持多版本并行维护的场景中使用。 |
| Hotfix Branch | Hotfix Branch / 热修复分支 | 从生产分支直接切出的紧急修复分支，用于快速修复生产事故。修复完成后同时合并到生产分支和开发主线，确保修复不丢失。 |

---

## 版本管理

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| SemVer | Semantic Versioning / 语义化版本 | 版本号格式：`MAJOR.MINOR.PATCH`。MAJOR = 不兼容的 API 变更，MINOR = 向后兼容的功能新增，PATCH = 向后兼容的 Bug 修复。预发布版本附加 `-alpha.1`、`-beta.2` 等后缀。 |
| CalVer | Calendar Versioning / 日历版本 | 基于日期的版本号格式，如 `2024.03.15` 或 `24.3`。适合发布周期固定、不需要表达 API 兼容性的项目（如 Ubuntu: 24.04）。 |
| Changelog | Changelog / 变更日志 | 记录每个版本的变更内容，按版本号倒序排列。遵循 Keep a Changelog 格式：Added / Changed / Deprecated / Removed / Fixed / Security。自动化工具：Conventional Commits + standard-version。 |
| Tag | Tag / 标签 | Git 中标记特定提交的引用，通常用于标记发布版本（如 `v1.2.3`）。分为轻量标签（仅指针）和注释标签（包含作者、日期、消息）。发布版本应使用注释标签。 |
| Conventional Commits | Conventional Commits / 约定式提交 | 提交信息格式规范：`type(scope): description`。type 包括 feat、fix、docs、style、refactor、test、chore 等。支持自动生成 Changelog 和版本号。 |

---

## 质量与安全

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| Quality Gate | Quality Gate / 质量门禁 | Pipeline 中的自动化检查点，代码必须通过指定阈值才能继续。常见门禁：测试覆盖率 ≥ 80%、零 Critical 漏洞、Lint 零错误、代码 Review 至少一人 Approve。 |
| SAST | Static Application Security Testing | 在不运行程序的情况下分析源代码中的安全漏洞。工具：CodeQL、Semgrep、SonarQube。优势：覆盖面广，开发阶段即可发现。劣势：误报率较高。 |
| SCA | Software Composition Analysis | 分析项目依赖中的已知漏洞。工具：Snyk、Trivy、Dependabot。检查范围包括直接依赖和传递依赖，对照 CVE 数据库匹配漏洞。 |
| DAST | Dynamic Application Security Testing | 在运行状态下通过模拟攻击检测应用漏洞。工具：OWASP ZAP、Burp Suite。需要可访问的部署环境，通常在 staging 阶段执行。 |

---

## 运行时与环境

| 术语 | 英文全称 | 定义 |
|------|---------|------|
| Environment Promotion | Environment Promotion / 环境晋级 | 制品从低环境逐步推进到高环境的过程：dev → staging → production。每次晋级前需通过对应环境的质量门禁。核心原则：同一制品跨环境部署，仅配置不同。 |
| Infrastructure as Code | IaC / 基础设施即代码 | 用代码（Terraform、Pulumi、CloudFormation）定义和管理基础设施的实践。IaC 确保环境配置可版本化、可审计、可复现。是 CI/CD 中环境一致性的基础保障。 |
| Container Registry | Container Registry / 容器镜像仓库 | 存储和分发 Docker/OCI 镜像的服务。常用方案：Docker Hub、GitHub Container Registry（ghcr.io）、Harbor（自建）、AWS ECR、Google GCR。CI 构建的镜像推送到 Registry，部署时从 Registry 拉取。 |
| Deployment Slot | Deployment Slot / 部署槽 | Azure App Service 提供的零停机部署机制。新版本部署到预热槽（Staging Slot），验证通过后与生产槽（Production Slot）交换流量。概念类似蓝绿部署但由平台原生支持。 |
| Pipeline as Code | Pipeline as Code / 流水线即代码 | 将 CI/CD Pipeline 定义为代码文件（如 `.github/workflows/*.yml`、`Jenkinsfile`、`.gitlab-ci.yml`）并纳入版本控制的实践。确保 Pipeline 变更可审计、可回滚、可测试。 |

---

## Agent Checklist

- [ ] 术语覆盖所有要求的关键词：Pipeline/Stage/Job/Runner/Artifact/Cache/Matrix Build/Reusable Workflow/Self-Hosted Runner/Blue-Green/Canary/Rolling Update/Feature Flag/Trunk-Based/Git Flow/SemVer/Changelog
- [ ] 每个术语包含英文全称和中文定义
- [ ] 术语按领域分组（流水线基础、构建策略、部署策略、分支模型、版本管理、质量与安全）
- [ ] 使用统一的表格格式
- [ ] 定义准确、专业，包含使用场景和工具推荐
- [ ] 文件超过 100 行
