# Super Dev + Claude Code 集成说明

## Super Dev 是什么？

**Super Dev** 是一个 **AI 编程流水线辅助工具**，具有两种形态：

### 定位

- ✅ **CLI 工具**：`super-dev` 命令行工具
- ✅ **Agent Skills**：Claude Code 中的增强技能
- ✅ **规范驱动开发**：从 PRD 到代码的全流程支持

### 核心价值

```
用户需求 → Super Dev CLI → 生成文档 + 提示词 → 复制到 Claude Code → 代码实现
         ↓
    Agent Skills 自动增强 → Claude Code 直接调用 Super Dev 能力
```

---

## 架构说明

### 双重架构

**Super Dev** 采用 **CLI 工具 + Agent Skills** 的双重架构：

| 形态 | 用途 | 触发方式 |
|:---|:---|:---|
| **CLI 工具** | 生成 PRD、架构、UI/UX 文档 | 终端执行 `super-dev` |
| **Agent Skills** | 增强 Claude Code 的开发能力 | 自动调用或手动触发 |

### Agent Skills 工作原理

```
.claude/skills/super-dev/
├── SKILL.md              # 技能定义（自动检测）
├── experts/              # 10 位专家（PM、架构、UI、UX、安全、代码、DBA、QA、DevOps、RCA）
├── scripts/              # Python 工具（市场调研、竞品分析、领域研究）
├── knowledge/            # 知识库（平台、行业、组件）
├── quality/              # 质量检查清单
└── wisdom/               # 最佳实践和陷阱
```

**自动检测机制**：
- 当你在 Claude Code 中描述相关任务时，Claude 会自动检测并加载 Super Dev Skill
- 例如："帮我设计一个电商平台的架构" → 自动激活架构师专家
- 例如："帮我评审这段代码的安全性" → 自动激活安全专家

---

## 如何在 Claude Code 中使用？

### 方式 1: 自动调用（推荐）

**Agent Skills 会自动激活**，你只需要：

1. **描述你的任务**：
```
我需要开发一个用户认证系统，包含注册、登录、密码重置功能
```

2. **Claude 自动检测并激活 Super Dev Skill**

3. **Claude 使用 Super Dev 的专家能力**：
- 产品经理专家会问你"关键 7 问"
- 架构师专家设计技术方案
- UI/UX 专家提供设计建议

### 方式 2: CLI 工具 + Claude Code

**步骤 1**: 在终端生成文档

```bash
# 使用 Super Dev CLI 生成完整文档
super-dev create "用户认证系统" \
  --platform web \
  --frontend react \
  --backend node

# 查看生成的文档
ls output/
```

**步骤 2**: 在 Claude Code 中使用

```
我使用 Super Dev 生成了项目文档，在 output/ 目录。
请阅读所有文档（PRD、架构、UI/UX）并帮我实现代码。
```

### 方式 3: 手动触发 Agent Skill

如果自动检测没有激活，你可以：

```
请使用 Super Dev Skill 来帮我设计这个功能的架构
```

或明确指定专家：

```
请激活 Super Dev 的架构师专家，帮我设计这个系统的技术架构
```

---

## 为什么不需要 Slash Command？

**Agent Skills vs Slash Commands**：

| 特性 | Agent Skills | Slash Commands |
|:---|:---|:---|
| **触发方式** | 自动检测 | 手动输入 `/command` |
| **灵活性** | 高（上下文感知） | 低（固定格式） |
| **适用场景** | 复杂任务 | 快速操作 |
| **Super Dev 选择** | ✅ 使用 | ❌ 不需要 |

**原因**：

1. **Agent Skills 更智能**：Claude 会根据任务自动选择合适的专家
2. **更灵活**：不需要记忆命令格式
3. **上下文感知**：根据对话历史自动调整

---

## 快速开始

### 测试 Super Dev Agent Skills

在 Claude Code 中直接对话：

```
我想开发一个 Todo List 应用，帮我规划一下
```

Claude 会自动激活 Super Dev 的产品经理专家，引导你完成需求分析。

### 测试 Super Dev CLI

```bash
# 1. 确认已安装
super-dev --version

# 2. 生成测试项目
super-dev create "Todo List 应用" \
  --platform web \
  --frontend react

# 3. 在 Claude Code 中使用
# "请阅读 output/ 目录的文档并实现代码"
```

---

## 与其他 AI 工具一起使用

Super Dev 生成的文档是**通用的**，可以在任何 AI 工具中使用：

- **Cursor**: 复制提示词到 Composer
- **Windsurf**: 复制提示词到 AI Chat
- **ChatGPT**: 粘贴提示词到对话框
- **Claude Code**: Agent Skills 自动激活或复制提示词
- **Aider**: `aider --prompt "$(cat output/*-ai-prompt.md)"`

详见：[docs/INTEGRATION_GUIDE.md](../docs/INTEGRATION_GUIDE.md)

---

## Agent Skills 专家列表

Super Dev 包含 **10 位专家**，可根据任务自动激活：

| 专家 | 擅长领域 | 触发场景示例 |
|:---|:---|:---|
| **产品经理** | 需求分析、PRD、优先级 | "帮我分析用户需求"、"写一个 PRD" |
| **架构师** | 系统设计、技术选型 | "设计系统架构"、"选择技术栈" |
| **UI 设计师** | 视觉设计、组件库 | "设计 UI 界面"、"选择颜色方案" |
| **UX 设计师** | 交互流程、用户体验 | "优化用户流程"、"提升用户体验" |
| **安全专家** | 安全审计、威胁建模 | "检查代码安全"、"做安全评估" |
| **代码审查官** | 代码质量、最佳实践 | "审查这段代码"、"优化代码结构" |
| **DBA** | 数据库设计、性能优化 | "设计数据库结构"、"优化 SQL" |
| **QA 工程师** | 测试策略、自动化 | "编写测试用例"、"测试覆盖率" |
| **DevOps** | 部署、CI/CD、容器化 | "配置部署流程"、"写 Dockerfile" |
| **故障侦探** | 问题排查、复盘 | "分析系统故障"、"做根因分析" |

---

## 总结

**Super Dev** = CLI 工具 + Agent Skills + 规范驱动开发

**CLI 工具**：
- 生成 PRD、架构、UI/UX 文档
- 生成代码规范和 AI 提示词
- 执行市场调研、竞品分析

**Agent Skills**：
- 自动增强 Claude Code 能力
- 10 位专家自动激活
- 上下文感知的智能辅助

**在 Claude Code 中**：
- ✅ Agent Skills 自动激活
- ✅ 直接对话使用专家能力
- ✅ 或使用 CLI 生成文档后再实现代码

**核心价值**：
- 🎯 从 0 到 1：完整的项目规划和文档生成
- 🚀 从 1 到 N+1：迭代开发和功能扩展
- 🤖 规范驱动：OpenSpec 风格的单源真相
- 👥 专家团队：10 位专家自动协作
