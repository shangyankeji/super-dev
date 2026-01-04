# Super Dev 快速开始指南

> 5 分钟上手 Super Dev，从想法到代码！

---

## 📋 目录

- [安装](#-安装-5-分钟)
- [场景选择](#-选择你的使用场景)
- [场景 1：从 0 到 1（全新项目）](#-场景-1从-0-到-1全新项目)
- [场景 2：从 1 到 N+1（现有项目）](#-场景-2从-1-到-n1现有项目)
- [在 Claude Code 中使用](#-在-claude-code 中使用)
- [常见问题](#-常见问题)

---

## 🚀 安装（5 分钟）

### 步骤 1：安装 Python

确保你的系统已安装 Python 3.10 或更高版本：

```bash
# 检查 Python 版本
python --version
# 或
python3 --version
```

如果未安装，访问 [python.org](https://www.python.org/downloads/) 下载安装。

### 步骤 2：安装 Super Dev

**方式 1：使用 uv（推荐）** ⚡

```bash
# 安装 uv（如果还没安装）
pip install uv

# 使用 uv 安装 Super Dev（快 10-100 倍！）
uv pip install super-dev

# 验证安装
super-dev --version
```

**方式 2：使用 pip（传统方式）**

```bash
# 使用 pip 安装
pip install super-dev

# 或用户模式安装（不需要 sudo）
pip install --user super-dev

# 验证安装
super-dev --version
```

**预期输出**：
```
Super Dev v1.0.1
```

### uv vs pip：为什么推荐 uv？

| 特性 | pip | uv |
|:---|:---|:---|
| **安装速度** | 基准 | **10-100x 更快** ⚡ |
| **依赖解析** | 较慢 | **极快（Rust 实现）** |
| **磁盘使用** | 较高 | **更低** |
| **兼容性** | 完全兼容 | 完全兼容 pip |
| **推荐场景** | 任何环境 | **新项目、CI/CD** |

**提示**：如果你已经有 pip，也可以直接使用 `pip install super-dev`，两种方式安装的包完全一样！

### 步骤 3：初始化项目（可选）

如果你要开始一个新项目，可以先创建项目目录：

```bash
# 创建项目目录
mkdir my-project
cd my-project

# 初始化 Super Dev 项目
super-dev init my-project
```

**预期输出**：
```
✓ 项目已初始化
✓ 创建 .super-dev/project.md
✓ 创建 .super-dev/specs/
```

---

## 🎯 选择你的使用场景

Super Dev 支持两种主要使用场景：

| 场景 | 说明 | 适用情况 |
|:---|:---|:---|
| **从 0 到 1** | 全新项目，从想法到完整代码 | 新产品、新功能、独立项目 |
| **从 1 到 N+1** | 现有项目，添加新功能或迭代 | 维护现有项目、功能扩展 |

**不确定？** 从 0 到 1 开始，完整体验 Super Dev 的能力！

---

## 🌱 场景 1：从 0 到 1（全新项目）

### 完整流程概览

```
想法 → CLI 生成文档 → Claude Code 实现 → 代码完成
```

### 步骤 1：准备你的想法

用一句话描述你想做什么：

**示例**：
- "一个类似 Todoist 的任务管理应用"
- "用户登录系统，包含注册、登录、密码重置"
- "电商网站的商品展示和购物车功能"

### 步骤 2：使用 CLI 生成完整文档

```bash
# 基础用法
super-dev create "你的功能描述"

# 完整用法（推荐）
super-dev create "用户登录系统" \
  --platform web \
  --frontend react \
  --backend node
```

**参数说明**：
- `--platform`: 平台类型（web/mobile/wechat/desktop）
- `--frontend`: 前端框架（react/vue/angular/svelte/flutter/swift/kotlin）
- `--backend`: 后端框架（node/python/go/java/rust/php）

**预期输出**：
```
✓ 正在分析需求...
✓ 生成 PRD 文档...
✓ 生成架构设计文档...
✓ 生成 UI/UX 设计文档...
✓ 创建 Spec 规范...
✓ 生成 AI 提示词...

📁 所有文档已生成到 output/ 目录

下一步：
1. 查看 output/ 目录下的文档
2. 在 Claude Code 中说："请阅读 output/ 目录的文档并帮我实现代码"
3. Claude Code 会根据文档自动生成代码
```

### 步骤 3：查看生成的文档

```bash
# 进入项目目录
cd my-project

# 查看生成的文档
ls output/
```

**你会看到**：
```
output/
├── 用户登录系统-prd.md              # PRD 文档（需求分析）
├── 用户登录系统-architecture.md     # 架构设计文档
├── 用户登录系统-uiux.md             # UI/UX 设计文档
└── 用户登录系统-ai-prompt.md        # AI 提示词（给 Claude 用）
```

### 步骤 4：在 Claude Code 中实现代码

**方式 1：直接对话（推荐）**

在 Claude Code Chat 中：

```
我使用 Super Dev 生成了项目文档，在 output/ 目录。
请阅读所有文档（PRD、架构、UI/UX）并帮我实现代码。
```

**Claude 会做什么**：
1. 阅读所有文档
2. 根据架构设计创建项目结构
3. 根据 UI/UX 设计实现界面
4. 根据 PRD 实现所有功能
5. 确保代码符合质量标准

**方式 2：使用 AI 提示词**

```bash
# 复制 AI 提示词（macOS）
cat output/*-ai-prompt.md | pbcopy

# 复制 AI 提示词（Linux）
cat output/*-ai-prompt.md | xclip
```

然后在 Claude Code 中粘贴并说：

```
请根据这个提示词生成代码
```

### 完整示例：创建 Todo List 应用

```bash
# 1. 创建项目目录
mkdir todo-app
cd todo-app

# 2. 使用 Super Dev 生成文档
super-dev create "Todo List 应用" \
  --platform web \
  --frontend react \
  --backend node

# 3. 查看生成的文档
ls output/

# 4. 在 Claude Code 中说：
# "我使用 Super Dev 生成了项目文档，在 output/ 目录。请阅读所有文档并帮我实现代码。"

# 5. Claude Code 会自动：
# - 创建项目结构
# - 实现 React 组件
# - 实现 Node.js API
# - 配置数据库
# - 编写测试
```

---

## 🔄 场景 2：从 1 到 N+1（现有项目）

### 完整流程概览

```
现有项目 → 添加新功能 → CLI 生成文档 → Claude Code 实现
```

### 步骤 1：进入你的项目目录

```bash
cd /path/to/your/existing-project
```

### 步骤 2：使用 Super Dev 添加新功能

```bash
# 为现有项目添加新功能
super-dev create "添加用户个人资料编辑功能" \
  --platform web \
  --frontend react \
  --backend node
```

**预期输出**：
```
✓ 检测到现有项目
✓ 分析项目结构...
✓ 生成新功能文档...
✓ 更新 Spec 规范...

📁 新功能文档已生成到 output/ 目录
```

### 步骤 3：查看生成的文档

```bash
ls output/
```

**你会看到**：
```
output/
├── 用户个人资料编辑功能-prd.md
├── 用户个人资料编辑功能-architecture.md
├── 用户个人资料编辑功能-uiux.md
└── 用户个人资料编辑功能-ai-prompt.md
```

### 步骤 4：在 Claude Code 中实现新功能

在 Claude Code Chat 中：

```
我使用 Super Dev 为现有项目生成了新功能的文档，在 output/ 目录。

新功能是：用户个人资料编辑功能

请：
1. 阅读现有项目代码（在 src/ 目录）
2. 阅读 output/ 目录的新功能文档
3. 根据文档将新功能集成到现有项目中
```

**Claude 会做什么**：
1. 分析现有项目结构
2. 阅读新功能文档
3. 在不破坏现有代码的前提下添加新功能
4. 确保新代码与现有代码风格一致
5. 运行测试确保没有破坏现有功能

### 完整示例：为电商网站添加支付功能

```bash
# 1. 进入现有项目
cd /path/to/my-ecommerce-site

# 2. 使用 Super Dev 生成支付功能文档
super-dev create "支付功能：支持支付宝和微信支付" \
  --platform web \
  --frontend react \
  --backend node

# 3. 查看生成的文档
ls output/

# 4. 在 Claude Code 中说：
# "我使用 Super Dev 为现有电商网站生成了支付功能的文档，在 output/ 目录。
#  请阅读现有项目代码和新功能文档，帮我实现支付功能。"

# 5. Claude Code 会：
# - 分析现有电商网站结构
# - 集成支付宝和微信支付 SDK
# - 添加支付 API 端点
# - 实现支付界面
# - 更新数据库 schema
# - 添加支付测试
```

---

## 💻 在 Claude Code 中使用

### 方式 1：自动激活 Agent Skills（推荐）

**直接描述你的任务**，Claude 会自动激活 Super Dev Skill：

```
我想开发一个用户认证系统，帮我规划一下
```

**Claude 会做什么**：
1. 自动检测到这是产品规划任务
2. 激活 Super Dev 的产品经理专家
3. 引导你完成需求分析（"关键 7 问"）
4. 生成完整的 PRD 文档

### 方式 2：手动触发 Agent Skill

如果自动检测没有激活，你可以：

```
请使用 Super Dev Skill 来帮我设计这个功能的架构
```

或明确指定专家：

```
请激活 Super Dev 的架构师专家，帮我设计这个系统的技术架构
```

### 方式 3：结合 CLI 使用

```bash
# 1. 在终端使用 CLI 生成文档
super-dev create "用户登录系统" --platform web --frontend react

# 2. 在 Claude Code 中使用生成的文档
# "我使用 Super Dev 生成了项目文档，在 output/ 目录。请阅读所有文档并帮我实现代码。"
```

---

## 📚 常用命令速查

```bash
# ===== 核心命令 =====
super-dev create "功能描述"              # 生成完整文档
super-dev pipeline "功能描述"            # 运行完整 8 阶段流水线
super-dev init <项目名>                  # 初始化新项目

# ===== Spec 管理 =====
super-dev spec list                      # 列出所有 specs
super-dev spec show <spec-name>          # 查看 spec 详情
super-dev spec update <spec-name>        # 更新 spec

# ===== 设计引擎 =====
super-dev design search "关键词"         # 搜索设计资产
super-dev design generate                # 生成完整设计系统

# ===== 质量检查 =====
super-dev quality check                  # 运行质量检查
super-dev quality score                  # 评分项目质量

# ===== 部署配置 =====
super-dev deploy --cicd github           # 生成 CI/CD 配置
super-dev deploy --docker                # 生成 Dockerfile

# ===== 帮助 =====
super-dev --help                         # 查看帮助
super-dev <command> --help               # 查看命令帮助
```

---

## ❓ 常见问题

### Q1：我应该在什么时候使用 Super Dev？

**A**：
- ✅ 开始新项目时
- ✅ 添加新功能时
- ✅ 需要架构设计时
- ✅ 需要代码审查时
- ✅ 需要部署配置时

### Q2：生成的文档在哪里？

**A**：默认在 `output/` 目录。你可以通过 `--output` 参数指定：

```bash
super-dev create "功能" --output docs/
```

### Q3：如何查看生成的文档？

**A**：
```bash
# 使用 cat 查看
cat output/*-prd.md

# 使用编辑器打开
code output/*-prd.md        # VS Code
open output/*-prd.md        # macOS
```

### Q4：Claude Code 不理解生成的文档怎么办？

**A**：尝试更明确的提示：

```
请：
1. 阅读 output/ 目录下的所有文档
2. 理解项目的 PRD、架构和 UI/UX 设计
3. 根据这些文档实现代码

如果文档中有不清楚的地方，请告诉我。
```

### Q5：Agent Skills 没有自动激活怎么办？

**A**：手动触发：

```
请使用 Super Dev Skill 的 [专家名称] 专家帮我 [具体任务]

示例：
"请使用 Super Dev Skill 的架构师专家帮我设计这个系统的技术架构"
```

### Q6：我可以选择不同的专家吗？

**A**：可以！Super Dev 有 10 位专家：

| 专家 | 用途示例 |
|:---|:---|
| **产品经理 (PM)** | 需求分析、PRD 编写 |
| **架构师** | 系统设计、技术选型 |
| **UI 设计师** | 界面设计、颜色搭配 |
| **UX 设计师** | 交互流程、用户体验 |
| **安全专家** | 安全审计、威胁建模 |
| **代码审查官** | 代码审查、性能优化 |
| **DBA** | 数据库设计、SQL 优化 |
| **QA 工程师** | 测试策略、自动化测试 |
| **DevOps** | 部署配置、CI/CD |
| **故障侦探** | 问题排查、根因分析 |

### Q7：生成文档后可以修改吗？

**A**：当然可以！生成的文档是 Markdown 格式，你可以：

```bash
# 使用编辑器修改
code output/*-prd.md

# 修改后重新生成（会保留你的修改）
super-dev update
```

### Q8：如何分享生成的文档？

**A**：
- 文档是 Markdown 格式，可以直接分享
- 可以转换成 PDF：`pandoc output/*-prd.md -o prd.pdf`
- 可以托管到 GitHub 或 GitLab
- 可以复制到 Notion、Confluence 等工具

### Q9：Super Dev 支持哪些技术栈？

**A**：支持几乎所有主流技术栈：

**前端**：React, Vue, Angular, Svelte, Flutter, Swift, Kotlin
**后端**：Node.js, Python, Go, Java, Rust, PHP
**数据库**：PostgreSQL, MySQL, MongoDB, Redis
**平台**：Web, Mobile, WeChat, Desktop

### Q10：生成文档需要多长时间？

**A**：通常 10-30 秒：

```bash
time super-dev create "用户登录"

# 实际输出时间
# ✓ 完成！耗时 15.2 秒
```

---

## 🎓 下一步

- 📖 [阅读完整工作流教程](WORKFLOW_GUIDE.md)
- 🤖 [了解 AI 工具集成](INTEGRATION_GUIDE.md)
- 🎨 [探索设计引擎](DESIGN_ENGINE.md)
- 📋 [查看命令参考](COMMAND_REFERENCE.md)

---

## 💡 提示

1. **从简单开始**：先用小功能测试 Super Dev
2. **阅读文档**：生成的文档是你的技术规范
3. **迭代优化**：根据实际需求调整文档
4. **善用 Claude Code**：让它帮你实现代码
5. **保持沟通**：不清楚时随时向 Claude 提问

---

## 🆘 需要帮助？

- GitHub Issues: [https://github.com/shangyankeji/super-dev/issues](https://github.com/shangyankeji/super-dev/issues)
- 文档: [https://github.com/shangyankeji/super-dev#readme](https://github.com/shangyankeji/super-dev#readme)

---

**准备好了吗？开始你的第一个项目！**

```bash
super-dev create "我的第一个项目" --platform web --frontend react
```
