# Super Dev 完整使用教程

## 目录

- [0-1 场景：从零开始创建项目](#0-1-场景从零开始创建项目)
- [1-N+1 场景：在现有项目上迭代](#1-n1-场景在现有项目上迭代)
- [工作流详解](#工作流详解)
- [最佳实践](#最佳实践)
- [常见问题](#常见问题)

---

# 0-1 场景：从零开始创建项目

## 场景描述

你有一个产品想法，需要从零开始创建一个完整的商业级项目。Super Dev 帮你自动完成从需求文档到部署配置的所有工作。

## 完整流程

### 第一步：项目初始化

```bash
# 创建项目目录
mkdir my-awesome-project
cd my-awesome-project

# 初始化 Super Dev 项目
super-dev init awesome-app \
  --platform web \
  --frontend react \
  --backend node
```

**生成文件结构**：
```
awesome-app/
├── .super-dev/              # Super Dev 配置目录
│   ├── config.yaml          # 项目配置
│   └── specs/               # 规范目录
├── super-dev.yaml           # 根配置文件
└── README.md                # 项目说明
```

### 第二步：生成完整项目资产

**方式 A：一键生成（推荐）**

```bash
super-dev pipeline "用户认证系统，包括邮箱注册、密码登录、OAuth2 第三方登录、JWT 令牌管理" \
  --platform web \
  --frontend react \
  --backend node \
  --domain auth \
  --cicd github \
  --quality-threshold 85
```

**方式 B：分步生成**

```bash
# 1. 生成核心文档（PRD + 架构 + UI/UX）
super-dev create "用户认证系统" \
  --platform web \
  --frontend react \
  --backend node \
  --domain auth

# 2. 查看 AI 提示词
cat output/*-ai-prompt.md

# 3. 生成 CI/CD 配置
super-dev deploy --cicd github

# 4. 生成数据库迁移脚本
super-dev migrate --orm prisma
```

### 第三步：使用 AI 提示词开发

1. **打开 AI 提示词文件**：
```bash
cat output/用户认证系统-ai-prompt.md
```

2. **复制提示词到 Claude/GPT-4**：
   - 直接复制整个 Markdown 文件内容
   - AI 会根据详细规范生成代码

3. **AI 生成的代码结构示例**：
```
auth-system/
├── frontend/
│   ├── src/
│   │   ├── components/          # UI 组件（基于 UI/UX 文档）
│   │   ├── pages/               # 页面（基于 PRD）
│   │   ├── hooks/               # 自定义 Hooks
│   │   └── utils/               # 工具函数
│   ├── package.json
│   └── tailwind.config.js       # 使用 UI/UX 文档中的配置
├── backend/
│   ├── src/
│   │   ├── routes/              # API 路由（基于架构文档）
│   │   ├── services/            # 业务逻辑
│   │   ├── models/              # 数据模型（基于数据库文档）
│   │   └── middleware/          # 中间件
│   └── package.json
├── prisma/
│   ├── schema.prisma            # 数据库 Schema
│   └── migrations/              # 迁移脚本
└── .github/
    └── workflows/
        ├── ci.yml               # CI 配置
        └── cd.yml               # CD 配置
```

### 第四步：质量检查

```bash
# 运行质量门禁
super-dev quality --type all

# 如果分数低于阈值，查看详细报告
cat output/*-quality-gate.md
```

### 第五步：迭代优化

```bash
# 分析当前项目
super-dev analyze

# 添加新功能（进入 1-N+1 模式）
super-dev spec propose add-2fa \
  --title "添加双因素认证" \
  --description "为用户登录添加 TOTP 双因素认证"
```

---

# 1-N+1 场景：在现有项目上迭代

## 场景描述

你已经有了一个运行中的项目，需要添加新功能或优化现有功能。Super Dev 帮你在现有基础上规范地迭代。

## 完整流程

### 第一步：分析现有项目

```bash
cd your-existing-project

# 自动分析项目结构
super-dev analyze
```

**输出示例**：
```
项目分析报告
================

项目类型: 全栈 Web 应用
前端框架: React 18.2.0
后端框架: Node.js (Express)
数据库: PostgreSQL
主要依赖: react-router, axios, prisma

技术栈健康度: 85/100
建议优化:
- 升级 React 到 18.3.0
- 添加 TypeScript
- 配置 CI/CD 流水线
```

### 第二步：创建变更提案（Spec-Driven Development）

```bash
# 初始化 SDD（如果还没有）
super-dev spec init

# 创建变更提案
super-dev spec propose add-payment-integration \
  --title "集成支付系统" \
  --description "集成 Stripe 支付网关，支持信用卡和支付宝"
```

**生成的提案结构**：
```
.super-dev/changes/add-payment-integration/
├── proposal.md       # 变更提案（为什么做、做什么、怎么做）
├── tasks.md          # 任务清单
├── design.md         # 设计影响分析
└── specs/            # 规范变更
    └── payment/
        └── spec.md   # 支付模块规范
```

### 第三步：添加详细需求

```bash
# 添加功能需求
super-dev spec add-req add-payment-integration payment stripe-integration \
  "系统 SHALL 集成 Stripe Payment Intent API"

super-dev spec add-req add-payment-integration payment alipay-support \
  "系统 SHALL 支持支付宝支付方式"

super-dev spec add-req add-payment-integration payment webhook-handler \
  "系统 SHALL 处理 Stripe Webhook 事件"

# 添加非功能需求
super-dev spec add-req add-payment-integration performance response-time \
  "支付 API 响应时间 SHALL 小于 200ms (P95)"

super-dev spec add-req add-payment-integration security pci-dss \
  "支付流程 SHALL 符合 PCI-DSS 合规要求"
```

### 第四步：生成变更文档

```bash
# 生成变更相关的所有文档
super-dev create "集成 Stripe 支付和支付宝" \
  --platform web \
  --domain ecommerce \
  --skip-docs \
  --output output/add-payment
```

**生成的文档**：
```
output/add-payment/
├── 集成-Stripe-支付和支付宝-prd.md           # 功能需求
├── 集成-Stripe-支付和支付宝-architecture.md  # 架构设计
├── 集成-Stripe-支付和支付宝-uiux.md          # UI/UX 设计
└── 集成-Stripe-支付和支付宝-ai-prompt.md     # AI 提示词
```

### 第五步：审查与批准

```bash
# 查看完整提案
super-dev spec show add-payment-integration

# 验证规范格式
super-dev spec validate add-payment-integration -v

# 团队审查后批准，进入开发
```

### 第六步：AI 辅助实现

1. **复制 AI 提示词到 Claude**：
```bash
cat output/add-payment/*-ai-prompt.md | pbcopy  # macOS
cat output/add-payment/*-ai-prompt.md | xclip  # Linux
```

2. **Claude 生成代码**：
   - 基于现有代码结构
   - 遵循项目代码风格
   - 添加必要的测试

3. **代码审查**：
```bash
# 使用代码审查指南
cat output/add-payment/*-code-review.md

# 运行红队审查
super-dev redteam "支付系统安全审查"
```

### 第七步：质量门禁

```bash
# 检查变更质量
super-dev quality --type all

# 查看详细报告
cat output/*-quality-gate.md
```

**质量门禁检查项**：
- ✅ 需求完整性 (PRD)
- ✅ 架构合理性
- ✅ 设计一致性 (UI/UX)
- ✅ 安全合规性
- ✅ 性能指标
- ✅ 测试覆盖率

### 第八步：归档变更

```bash
# 变更完成后归档
super-dev spec archive add-payment-integration
```

**归档后的结构**：
```
.super-dev/archive/add-payment-integration/
├── proposal.md       # 原始提案
├── tasks.md          # 完成的任务
├── specs/            # 最终规范（成为新的单一事实源）
└── implementation/   # 实现记录
```

---

# 工作流详解

## 8 阶段流水线

```
┌─────────────────────────────────────────────────────────────┐
│  1. 文档生成 (Documentation)                                │
│     ├─ PRD: 产品需求文档                                    │
│     ├─ Architecture: 技术架构设计                           │
│     └─ UI/UX: 界面与交互设计                                │
├─────────────────────────────────────────────────────────────┤
│  2. Spec 创建 (Specification)                              │
│     ├─ proposal.md: 变更提案                               │
│     ├─ tasks.md: 任务清单                                  │
│     └─ spec.md: 规范定义（OpenSpec 格式）                  │
├─────────────────────────────────────────────────────────────┤
│  3. 红队审查 (Red Team Review)                             │
│     ├─ 安全威胁分析                                        │
│     ├─ 性能瓶颈识别                                        │
│     └─ 架构缺陷检查                                        │
├─────────────────────────────────────────────────────────────┤
│  4. 质量门禁 (Quality Gate)                                │
│     ├─ 自动评分 (80+ 分通过)                               │
│     ├─ 详细改进建议                                        │
│     └─ 缺陷修复追踪                                        │
├─────────────────────────────────────────────────────────────┤
│  5. 代码审查 (Code Review)                                 │
│     ├─ 审查清单                                            │
│     ├─ 常见陷阱警告                                        │
│     └─ 最佳实践建议                                        │
├─────────────────────────────────────────────────────────────┤
│  6. AI 提示生成 (AI Prompt Generation)                     │
│     ├─ 结构化提示词                                        │
│     ├─ 上下文注入                                          │
│     └─ 约束条件明确                                        │
├─────────────────────────────────────────────────────────────┤
│  7. CI/CD 配置 (Continuous Integration/Deployment)         │
│     ├─ GitHub Actions / GitLab CI / Jenkins               │
│     ├─ 自动化测试                                          │
│     └─ 自动部署                                            │
├─────────────────────────────────────────────────────────────┤
│  8. 数据库迁移 (Database Migration)                        │
│     ├─ Prisma / TypeORM / Sequelize                       │
│     ├─ SQLAlchemy / Django / Mongoose                     │
│     └─ 版本化迁移脚本                                      │
└─────────────────────────────────────────────────────────────┘
```

## 各阶段详解

### 阶段 1: 文档生成

**目的**: 为项目建立完整、专业的文档基础

**输入**:
- 产品想法（一句话描述）
- 目标平台 (web/mobile/wechat/desktop)
- 前端框架 (react/vue/angular/svelte)
- 后端框架 (node/python/go/java)
- 业务领域 (fintech/ecommerce/medical/social/iot/education/auth/content)

**输出**:
1. **PRD 文档** (XX-prd.md)
   - 项目概述
   - 用户故事
   - 功能需求
   - 非功能需求
   - 验收标准

2. **架构文档** (XX-architecture.md)
   - 系统架构图
   - 技术栈选型
   - 数据模型
   - API 设计
   - 安全设计

3. **UI/UX 文档** (XX-uiux.md)
   - 设计分析（项目特征）
   - 配色方案（基于 150+ 色彩库）
   - 字体配对（基于 80+ 组合）
   - 布局模式
   - UX 最佳实践
   - Tailwind 配置

**示例**:
```bash
super-dev pipeline "任务管理系统，支持项目创建、任务分配、进度跟踪、团队协作" \
  --platform web \
  --frontend react \
  --backend python \
  --domain content
```

### 阶段 2: Spec 创建

**目的**: 创建 OpenSpec 风格的规范，作为开发的单一事实源

**工作流**:
```
Draft → Review & Align → Implement → Archive
```

**关键概念**:
- **单一事实源 (SSOT)**: 所有决策记录在 spec 中
- **Delta 格式**: 只记录变更部分 (ADDED/MODIFIED/REMOVED)
- **可追溯**: 每个需求都有唯一 ID

**示例 Spec**:
```markdown
# Payment Module Spec

## ADDED: Stripe Integration

### REQ-PAY-001: Stripe Payment Intent
**Requirement**: The system SHALL integrate Stripe Payment Intent API

**Rationale**: Stripe provides industry-leading payment processing

**Specification**:
- API Version: 2023-10-16
- Supported Methods: card, alipay
- Webhook Events: payment_intent.succeeded, payment_intent.failed

### REQ-PAY-002: Payment Confirmation
**Requirement**: The system SHALL send email confirmation after successful payment

**Specification**:
- Email Template: payment-confirmation.html
- Delivery Time: < 5 seconds
```

### 阶段 3: 红队审查

**目的**: 从攻击者视角审查设计，发现潜在问题

**审查维度**:
1. **安全**
   - 认证/授权漏洞
   - SQL/XSS/CSRF 注入
   - 敏感数据泄露
   - API 安全

2. **性能**
   - 数据库查询优化
   - 缓存策略
   - 并发处理
   - 资源消耗

3. **架构**
   - 单点故障
   - 可扩展性
   - 技术债务
   - 依赖风险

**输出示例**:
```markdown
# 红队审查报告

## 高危问题 (Critical)

### [SEC-001] JWT 密钥硬编码
**位置**: `src/auth/jwt.ts:15`
**问题**: JWT 密钥直接写在代码中
**影响**: 密钥泄露后攻击者可伪造任意令牌
**修复**: 使用环境变量存储密钥

### [PERF-001] N+1 查询问题
**位置**: `src/api/users.ts:45`
**问题**: 获取用户列表时对每个用户单独查询订单
**影响**: 1000 用户时需要 1001 次数据库查询
**修复**: 使用 eager loading 或 JOIN 查询

## 中危问题 (High)
- [ARCH-001] 缺少 API 限流机制
- [SEC-002] CORS 配置过于宽松
```

### 阶段 4: 质量门禁

**目的**: 自动评估项目质量，确保达到商业级标准

**评分体系**:
```
总分 100 分，80 分以上通过

维度 1: 需求完整性 (20 分)
  ├─ 用户故事清晰度 (5)
  ├─ 功能需求完整性 (5)
  ├─ 非功能需求明确性 (5)
  └─ 验收标准可衡量性 (5)

维度 2: 架构合理性 (20 分)
  ├─ 技术选型合理性 (5)
  ├─ 架构设计清晰度 (5)
  ├─ 数据模型正确性 (5)
  └─ 可扩展性考虑 (5)

维度 3: 设计一致性 (20 分)
  ├─ 视觉风格统一性 (5)
  ├─ 交互逻辑一致性 (5)
  ├─ 无障碍性 (5)
  └─ 响应式设计 (5)

维度 4: 安全合规性 (20 分)
  ├─ 认证授权设计 (5)
  ├─ 数据保护 (5)
  ├─ API 安全 (5)
  └─ 合规性考虑 (5)

维度 5: 可维护性 (20 分)
  ├─ 代码组织 (5)
  ├─ 文档完整性 (5)
  ├─ 测试覆盖 (5)
  └─ 错误处理 (5)
```

**输出示例**:
```markdown
# 质量门禁报告

## 总体评分: 87/100 ✅ 通过

### 分项得分
| 维度 | 得分 | 满分 | 状态 |
|:---|:---|:---|:---|
| 需求完整性 | 18 | 20 | ✅ |
| 架构合理性 | 19 | 20 | ✅ |
| 设计一致性 | 16 | 20 | ⚠️ |
| 安全合规性 | 17 | 20 | ✅ |
| 可维护性 | 17 | 20 | ✅ |

### 改进建议
1. **设计一致性**: 建议统一按钮圆角（当前 4px vs 8px）
2. **可维护性**: 建议添加 API 文档 (Swagger/OpenAPI)
```

### 阶段 5: 代码审查

**目的**: 为代码审查提供专业指导

**审查清单**:
```markdown
# 代码审查指南

## 功能性
- [ ] 代码实现了规范中的所有需求
- [ ] 边界条件处理正确
- [ ] 错误处理完善

## 安全性
- [ ] 输入验证充分
- [ ] SQL/命令注入防护
- [ ] 敏感数据不泄露

## 性能
- [ ] 无明显性能瓶颈
- [ ] 数据库查询优化
- [ ] 缓存使用得当

## 可读性
- [ ] 命名清晰
- [ ] 逻辑易懂
- [ ] 注释充分

## 测试
- [ ] 单元测试覆盖核心逻辑
- [ ] 集成测试覆盖关键路径
```

### 阶段 6: AI 提示生成

**目的**: 为 Claude/GPT-4 等AI生成结构化提示词

**提示词结构**:
```markdown
# 项目上下文
- 项目名称: {name}
- 技术栈: {stack}
- 业务领域: {domain}

# 约束条件
1. 必须遵循 PRD 中定义的所有功能需求
2. 必须实现架构文档中的 API 设计
3. 必须使用 UI/UX 文档中的设计系统
4. 必须修复红队审查中发现的所有高危问题
5. 必须达到质量门禁的所有标准

# 代码规范
- 遵循项目现有代码风格
- 使用 TypeScript 严格模式
- 所有函数必须有 JSDoc 注释
- 测试覆盖率 > 80%

# 任务清单
1. 实现 {feature-1}
2. 实现 {feature-2}
...

# 参考文档
- PRD: {prd-path}
- 架构: {arch-path}
- UI/UX: {uiux-path}
- Spec: {spec-path}
```

### 阶段 7: CI/CD 配置

**支持的平台**:
- GitHub Actions
- GitLab CI
- Jenkins
- Azure DevOps
- Bitbucket Pipelines

**生成内容**:
```yaml
# .github/workflows/ci.yml
name: CI

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      - run: npm ci
      - run: npm run lint
      - run: npm run test
      - run: npm run build

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm audit
      - run: npm run snyk-test
```

### 阶段 8: 数据库迁移

**支持的 ORM**:
- Prisma (TypeScript/Node)
- TypeORM (TypeScript/Node)
- Sequelize (Node)
- SQLAlchemy (Python)
- Django (Python)
- Mongoose (Node/MongoDB)

**生成示例** (Prisma):
```prisma
// prisma/schema.prisma

model User {
  id        String   @id @default(uuid())
  email     String   @unique
  password  String
  name      String?
  createdAt DateTime @default(now())
  updatedAt DateTime @updatedAt

  // Relations
  orders    Order[]
}

model Order {
  id          String   @id @default(uuid())
  userId      String
  status      OrderStatus @default(PENDING)
  total       Decimal
  createdAt   DateTime @default(now())
  updatedAt   DateTime @updatedAt

  // Relations
  user        User     @relation(fields: [userId], references: [id])
  items       OrderItem[]
}

enum OrderStatus {
  PENDING
  PAID
  SHIPPED
  DELIVERED
  CANCELLED
}
```

---

# 最佳实践

## 0-1 项目最佳实践

### 1. 明确产品定位
```bash
# ✅ 好的描述
super-dev pipeline "B2B SaaS 项目管理工具，支持看板视图、甘特图、团队协作、工时统计"

# ❌ 差的描述
super-dev pipeline "做一个项目管理"
```

### 2. 选择正确的业务领域
```bash
# 电商项目
--domain ecommerce

# 金融项目
--domain fintech

# 认证授权
--domain auth
```

### 3. 设置合理的质量标准
```bash
# MVP 阶段可以放宽
--quality-threshold 70

# 生产环境严格标准
--quality-threshold 85
```

## 1-N+1 项目最佳实践

### 1. 小步迭代
```bash
# ✅ 好的变更粒度
super-dev spec propose add-user-profile
super-dev spec propose add-avatar-upload
super-dev spec propose add-social-login

# ❌ 太大的变更
super-dev spec propose huge-refactoring
```

### 2. 保持 Spec 更新
```bash
# 每次变更后
super-dev spec validate

# 定期归档
super-dev spec archive completed-feature
```

### 3. 充分利用专家系统
```bash
# 需求模糊时咨询 PM
super-dev expert PM "这个功能是否必要？"

# 架构不确定时咨询架构师
super-dev expert ARCHITECT "如何设计高并发？"

# 安全审查时调用安全专家
super-dev expert SECURITY "OAuth2 实现是否安全？"
```

## 团队协作最佳实践

### 1. 使用 Git 集成
```bash
# 在 CI 中运行质量检查
- run: super-dev quality --type all

# 在 PR 中强制执行
if [ $(super-dev quality --score) -lt 80 ]; then
  exit 1
fi
```

### 2. 文档版本控制
```bash
# 所有生成的文档纳入 Git
git add output/
git add .super-dev/

# 规范作为单一事实源
git add .super-dev/specs/
```

### 3. 定期回顾
```bash
# 每个迭代结束后
super-dev analyze  # 查看技术债务
super-dev quality  # 检查质量趋势
```

---

# 常见问题

## Q1: Super Dev 生成代码吗？

**A**: Super Dev 本身不生成代码，它生成：
- 完整的项目文档（PRD、架构、UI/UX）
- 规范定义（OpenSpec 格式）
- AI 提示词（给 Claude/GPT-4）
- 配置文件（CI/CD、数据库）

你将提示词复制给 Claude，Claude 根据详细规范生成代码。

## Q2: 如何确保 AI 生成的代码质量？

**A**: Super Dev 提供多层保障：
1. **红队审查**: 发现安全和性能问题
2. **质量门禁**: 80+ 分标准
3. **代码审查指南**: 详细的审查清单
4. **专家系统**: 10 位专家提供专业建议

## Q3: 可以在现有项目上使用吗？

**A**: 可以！支持两种模式：
- **0-1**: 从零开始
- **1-N+1**: 在现有项目上迭代

使用 `super-dev analyze` 分析现有项目，然后用 `super-dev spec` 添加新功能。

## Q4: 生成的文档可以直接用于生产吗？

**A**: 是的，文档达到商业级标准：
- 专业术语和格式
- 完整的需求和架构
- 可执行的设计配置
- 行业最佳实践

建议团队审查后使用。

## Q5: 如何处理特定业务逻辑？

**A**: Super Dev 通过以下方式处理：
1. **业务领域知识库**: 6 个领域专业知识
2. **专家系统**: 咨询领域专家
3. **自定义提示**: 在需求描述中加入具体细节
4. **迭代优化**: 使用 1-N+1 模式持续改进

## Q6: 支持哪些技术栈？

**A**:
- **前端**: React, Vue, Angular, Svelte
- **后端**: Node.js, Python, Go, Java
- **数据库**: PostgreSQL, MySQL, MongoDB, SQLite
- **ORM**: Prisma, TypeORM, Sequelize, SQLAlchemy, Django, Mongoose
- **CI/CD**: GitHub, GitLab, Jenkins, Azure, Bitbucket
- **平台**: Web, Mobile, WeChat, Desktop

## Q7: 如何升级版本？

**A**:
```bash
# 查看当前版本
super-dev --version

# 升级到最新版
git pull origin main
pip install -e .
```

## Q8: 数据安全和隐私？

**A**:
- 所有处理在本地完成
- 不向外部服务发送代码
- 生成的文档可以手动审查敏感信息
- 支持私有化部署

---

# 快速参考

## 常用命令

```bash
# 0-1 场景
super-dev init <name>              # 初始化项目
super-dev pipeline "描述"          # 完整流水线
super-dev create "描述"            # 快速创建

# 1-N+1 场景
super-dev analyze                  # 分析现有项目
super-dev spec init                # 初始化 SDD
super-dev spec propose <id>        # 创建提案
super-dev spec add-req <id> ...    # 添加需求
super-dev spec validate            # 验证规范
super-dev spec archive <id>        # 归档变更

# 设计智能
super-dev design search "关键词"   # 搜索设计资产
super-dev design generate          # 生成设计系统

# 专家系统
super-dev expert --list            # 列出专家
super-dev expert PM "问题"         # 咨询专家

# 质量与部署
super-dev quality --type all       # 质量检查
super-dev deploy --cicd github     # 生成 CI/CD
```

## 工作流速查

```
新项目 (0-1)
  ↓
init → pipeline → AI 实现 → quality → 部署

现有项目 (1-N+1)
  ↓
analyze → spec propose → spec add-req → create → AI 实现 → validate → archive
```

---

**需要帮助？**
- GitHub Issues: https://github.com/shangyankeji/super-dev/issues
- Email: 11964948@qq.com
