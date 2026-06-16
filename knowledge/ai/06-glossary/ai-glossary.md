---
id: ai-glossary
title: AI/ML 术语表 (AI/ML Glossary)
domain: ai
category: 06-glossary
difficulty: intermediate
tags: [ai, glossary, prompt, transformer, 与注意力机制, 基础概念, 大语言模型, 工程]
quality_score: 70
last_updated: 2026-06-15
---
# AI/ML 术语表 (AI/ML Glossary)

> 适用场景：AI 项目沟通对齐、新成员 Onboarding、技术方案评审中的概念统一。
> 涵盖范围：深度学习基础、大语言模型、训练优化、推理部署、Agent 与 RAG 等 40+ 核心术语。

---

## 基础概念

| 术语 | 英文 | 定义 | 关联术语 |
|------|------|------|----------|
| 机器学习 | Machine Learning (ML) | 通过数据训练算法使其自动学习规律并进行预测，无需显式编程。分为监督学习、无监督学习、强化学习三大范式。 | Deep Learning, Supervised Learning |
| 深度学习 | Deep Learning (DL) | 基于多层神经网络的机器学习方法，能自动从原始数据中学习层次化特征表示。 | Neural Network, Feature Learning |
| 神经网络 | Neural Network (NN) | 受生物神经元启发的计算模型，由输入层、隐藏层和输出层组成，通过反向传播学习权重。 | Backpropagation, Activation Function |
| 损失函数 | Loss Function | 衡量模型预测值与真实值之间差距的函数，训练目标是最小化损失。常见有交叉熵、MSE 等。 | Optimization, Gradient Descent |
| 过拟合 | Overfitting | 模型在训练集上表现好但在新数据上表现差，学习了噪声而非规律。对策包括正则化、Dropout、数据增强。 | Underfitting, Regularization |
| 特征工程 | Feature Engineering | 从原始数据中构造有助于模型学习的输入特征的过程。在传统 ML 中极其关键，深度学习中部分被自动化替代。 | Feature Selection, Embedding |

---

## Transformer 与注意力机制

| 术语 | 英文 | 定义 | 关联术语 |
|------|------|------|----------|
| Transformer | Transformer | Google 2017 年提出的序列模型架构，基于自注意力机制，支持并行计算，取代 RNN 成为 NLP 主流。所有现代 LLM 均基于此架构。 | Self-Attention, Encoder-Decoder |
| 注意力机制 | Attention Mechanism | 允许模型在处理某个位置时动态关注输入序列中最相关部分的机制。计算 Query-Key-Value 的加权和。 | Self-Attention, Multi-Head Attention |
| 自注意力 | Self-Attention | 输入序列对自身的注意力计算，每个 Token 关注同一序列中所有其他 Token，捕获长距离依赖。 | Attention, Transformer |
| 多头注意力 | Multi-Head Attention | 将注意力计算拆分为多个并行的"头"，每个头学习不同的注意力模式，最后拼接合并。 | Self-Attention, Transformer |
| 位置编码 | Positional Encoding | 向 Token Embedding 中注入位置信息的技术，因为 Transformer 本身不感知序列顺序。包括正弦编码和 RoPE 等。 | Transformer, RoPE |

---

## 大语言模型 (LLM)

| 术语 | 英文 | 定义 | 关联术语 |
|------|------|------|----------|
| 大语言模型 | Large Language Model (LLM) | 参数量在数十亿以上、基于 Transformer 的语言模型，通过大规模文本预训练获得通用语言理解和生成能力。 | GPT, Claude, Llama |
| Token | Token | 文本被分词器切分后的最小单位，可以是单词、子词或字符。LLM 以 Token 为单位处理和生成文本。 | Tokenizer, BPE |
| 上下文窗口 | Context Window | LLM 单次推理能处理的最大 Token 数量。超出窗口的内容无法被模型感知。现代模型范围从 4K 到 1M+。 | Token, Long Context |
| 温度 | Temperature | 控制 LLM 输出随机性的参数。温度越低输出越确定（贪心），温度越高输出越多样（创意）。 | Top-P, Sampling |
| 幻觉 | Hallucination | LLM 生成看似合理但事实错误的内容。根因是模型基于概率生成而非事实检索。 | Grounding, RAG |
| 接地 | Grounding | 将 LLM 输出与外部可信数据源关联验证的技术，减少幻觉。常见方式包括 RAG、工具调用、引用标注。 | Hallucination, RAG |

---

## 训练方法

| 术语 | 英文 | 定义 | 关联术语 |
|------|------|------|----------|
| 微调 | Fine-tuning | 在预训练模型基础上，使用特定任务数据继续训练以适配目标场景。可以是全量微调或参数高效微调。 | LoRA, PEFT |
| LoRA | Low-Rank Adaptation | 参数高效微调方法，在原始权重旁增加低秩矩阵，仅训练新增参数（通常 < 1%），大幅降低微调成本。 | Fine-tuning, QLoRA, PEFT |
| QLoRA | Quantized LoRA | LoRA 的改进版本，将基础模型量化到 4-bit 后再做 LoRA 微调，使大模型微调可在消费级 GPU 上运行。 | LoRA, Quantization |
| RLHF | Reinforcement Learning from Human Feedback | 通过人类偏好反馈训练奖励模型，再用强化学习（PPO）优化 LLM 输出，使其更符合人类期望。ChatGPT 的核心训练方法之一。 | DPO, Reward Model, PPO |
| DPO | Direct Preference Optimization | RLHF 的简化替代方案，直接用偏好数据优化模型，无需单独训练奖励模型。更稳定且计算成本更低。 | RLHF, Preference Learning |
| 预训练 | Pre-training | 在大规模无标注数据上训练语言模型的初始阶段，通过预测下一个 Token（自回归）或掩码恢复（BERT）学习语言表示。 | Fine-tuning, Self-supervised |
| 蒸馏 | Distillation | 将大模型（Teacher）的知识迁移到小模型（Student）的训练技术，小模型学习大模型的输出分布而非原始标签。 | Compression, Quantization |

---

## 推理与优化

| 术语 | 英文 | 定义 | 关联术语 |
|------|------|------|----------|
| 推理 | Inference | 使用训练好的模型对新输入进行预测/生成的过程。推理阶段的优化重点是延迟和吞吐量。 | Latency, Throughput |
| 延迟 | Latency | 从发送推理请求到收到完整响应的时间。通常关注首 Token 延迟（TTFT）和每 Token 生成时间。 | TTFT, Throughput |
| 吞吐量 | Throughput | 单位时间内系统能处理的请求数或生成的 Token 数。衡量推理服务的处理能力。 | Latency, Batch Size |
| 量化 | Quantization | 将模型权重从高精度（FP32）压缩到低精度（FP16/INT8/INT4）以减少内存占用和加速推理，精度损失通常可控。 | GPTQ, AWQ, Distillation |
| KV Cache | Key-Value Cache | 自回归生成时缓存已计算的 Key 和 Value 矩阵，避免重复计算。是 LLM 推理内存占用的主要来源。 | PagedAttention, Inference |

---

## Prompt 工程

| 术语 | 英文 | 定义 | 关联术语 |
|------|------|------|----------|
| Prompt | Prompt | 输入给 LLM 的指令或上下文文本，引导模型生成期望输出。Prompt 的质量直接影响输出质量。 | System Prompt, Template |
| 少样本学习 | Few-Shot Learning | 在 Prompt 中提供少量（通常 1-5 个）输入-输出示例，引导模型理解任务格式和期望行为。 | Zero-Shot, In-Context Learning |
| 零样本学习 | Zero-Shot Learning | 不提供任何示例，仅通过任务描述让模型完成任务。依赖模型的预训练知识和指令遵循能力。 | Few-Shot, Instruction Tuning |
| 思维链 | Chain-of-Thought (CoT) | 引导 LLM 逐步推理而非直接给出答案的 Prompt 技术，在复杂推理任务上显著提升准确率。 | Few-Shot, Reasoning |
| 思维树 | Tree-of-Thought (ToT) | CoT 的扩展，允许模型探索多个推理路径并评估选择最优解，适用于需要搜索和规划的复杂任务。 | CoT, Self-Consistency |

---

## RAG 与检索

| 术语 | 英文 | 定义 | 关联术语 |
|------|------|------|----------|
| RAG | Retrieval-Augmented Generation | 检索增强生成，将外部知识库的检索结果作为 LLM 的上下文，生成更准确、可溯源的回答。减少幻觉的主流方案。 | Embedding, Vector DB, Grounding |
| 向量嵌入 | Embedding | 将文本/图像等非结构化数据映射为固定维度的稠密向量，使语义相近的内容在向量空间中距离更近。 | Vector DB, Similarity Search |
| 向量数据库 | Vector Database | 专门存储和检索向量嵌入的数据库，支持高效的近似最近邻（ANN）搜索。代表产品：Pinecone / Milvus / Weaviate / Qdrant。 | Embedding, ANN, HNSW |
| 混合检索 | Hybrid Search | 结合稀疏检索（BM25 关键词匹配）和稠密检索（向量语义匹配）的检索策略，兼顾精确匹配和语义理解。 | BM25, Embedding, Reranking |
| 重排序 | Reranking | 对初步检索结果用更精确的模型（如 Cross-Encoder）重新排序，提升 Top-K 结果质量。 | Retrieval, Cross-Encoder |

---

## Agent 与工具使用

| 术语 | 英文 | 定义 | 关联术语 |
|------|------|------|----------|
| Agent | Agent (智能体) | 基于 LLM 的自主决策系统，能感知环境、规划任务、调用工具并迭代执行，直到完成目标。 | Tool Use, Planning, ReAct |
| 工具调用 | Tool Use / Function Calling | LLM 通过结构化输出调用外部 API / 函数 / 数据库的能力，扩展模型的行动边界。 | Agent, API, JSON Schema |
| ReAct | Reasoning + Acting | Agent 架构模式，交替进行推理（Thought）和行动（Action），根据观察结果迭代决策。 | Agent, CoT, Tool Use |
| 规划 | Planning | Agent 将复杂目标分解为可执行子任务的能力，包括任务分解、依赖排序和资源分配。 | Agent, Task Decomposition |
| 多 Agent 系统 | Multi-Agent System | 多个具有不同角色和能力的 Agent 协作完成复杂任务的架构，通过消息传递和共享状态协调。 | Agent, Orchestration |
| 记忆 | Memory | Agent 存储和检索历史交互、知识和状态的机制。分为短期记忆（上下文窗口）和长期记忆（外部存储）。 | Agent, Context Window, RAG |

---

## MLOps 与工程化

| 术语 | 英文 | 定义 | 关联术语 |
|------|------|------|----------|
| MLOps | Machine Learning Operations | 将 DevOps 实践应用于 ML 项目的方法论，涵盖数据管理、模型训练、部署、监控的全生命周期自动化。 | CI/CD, Model Registry |
| 模型注册中心 | Model Registry | 集中管理模型版本、元数据、血缘关系的服务。支持模型的注册、审批、部署追踪。代表：MLflow / SageMaker。 | MLOps, Model Versioning |
| 数据漂移 | Data Drift | 生产环境中输入数据分布与训练数据分布发生偏移的现象，会导致模型性能下降，需持续监控。 | Concept Drift, Monitoring |
| 特征存储 | Feature Store | 集中管理和服务化机器学习特征的系统，支持特征复用、一致性保证和在线/离线服务。 | Feature Engineering, MLOps |
| A/B 测试 | A/B Testing | 将用户随机分为实验组和对照组，比较不同模型版本的效果差异，基于统计显著性做上线决策。 | Canary Deploy, Statistical Significance |
| 概念漂移 | Concept Drift | 数据中输入与输出之间的映射关系随时间发生变化，导致模型准确率下降。需要持续监控和定期重训练。 | Data Drift, Model Monitoring |

---

## 安全与对齐

| 术语 | 英文 | 定义 | 关联术语 |
|------|------|------|----------|
| Prompt 注入 | Prompt Injection | 攻击者通过精心构造的输入覆盖或绕过系统 Prompt 约束，使模型执行非预期行为。分为直接注入和间接注入。 | Jailbreak, Security |
| 越狱 | Jailbreak | 通过特殊 Prompt 绕过 LLM 的安全限制，使其生成被禁止的内容。常见手法包括角色扮演、多语言绕过等。 | Prompt Injection, Red Team |
| 红队测试 | Red Teaming | 模拟攻击者对 AI 系统进行对抗性测试，发现安全漏洞、偏见和有害输出。是 AI 安全评估的核心方法。 | Prompt Injection, Safety |
| 对齐 | Alignment | 确保 AI 系统的行为符合人类意图和价值观的研究和工程实践。RLHF 和 Constitutional AI 是常见的对齐方法。 | RLHF, Safety, Ethics |
| 护栏 | Guardrails | 限制 LLM 输入和输出的安全机制，包括输入过滤、输出检测、话题限制等。确保模型行为在预期范围内。 | Safety, Content Filter |
| Constitutional AI | Constitutional AI (CAI) | Anthropic 提出的对齐方法，通过一组明确的原则（宪法）指导模型的自我改进，减少对人工标注的依赖。 | Alignment, RLHF |
| 水印 | Watermarking | 在 AI 生成内容中嵌入不可见标记的技术，用于识别和追踪 AI 生成的文本/图像。 | Detection, Provenance |
| 有害内容过滤 | Content Filtering | 检测和拦截 AI 输出中有害内容（暴力、歧视、虚假信息等）的机制。通常结合分类模型和规则引擎。 | Guardrails, Safety |

---

## 多模态与新兴方向

| 术语 | 英文 | 定义 | 关联术语 |
|------|------|------|----------|
| 多模态 | Multimodal | 能同时处理和理解多种数据类型（文本、图像、音频、视频）的模型能力。代表模型：GPT-4V、Gemini、Claude。 | Vision, Audio |
| 视觉语言模型 | Vision-Language Model (VLM) | 同时具备图像理解和文本生成能力的模型，可以描述图片、回答关于图片的问题、执行 OCR 等任务。 | Multimodal, OCR |
| 合成数据 | Synthetic Data | 由 AI 模型生成的训练数据，用于数据稀缺场景的数据增强。需注意质量控制和分布偏差。 | Data Augmentation, Training |

---

## Agent Checklist

- [ ] 术语在项目文档和代码注释中使用一致，避免混用中英文别名
- [ ] 团队成员已阅读并理解与当前项目相关的核心术语
- [ ] 新增的 AI 术语已补充到本术语表
- [ ] 技术方案评审中引用术语时附带本表链接
