# Super Dev 项目配置

此目录包含 Super Dev 项目的运行时配置和状态。

## 目录结构

```
.super-dev/
├── project.md          # 项目元数据（自动生成）
├── specs/              # OpenSpec 规范文件
├── changes/            # 变更记录（按功能组织）
└── README.md           # 本文件
```

## project.md 说明

`project.md` 是项目的单源真相（Single Source of Truth），包含：

- 项目基本信息（名称、描述、版本）
- 技术栈配置
- 规范定义
- 文档引用

## 规范驱动开发 (SDD)

Super Dev 采用 OpenSpec 风格的规范驱动开发：

1. **Single Source of Truth**: `project.md` 作为唯一真相源
2. **Spec 驱动**: 所有开发基于 spec 规范
3. **文档同步**: PRD、架构、UI/UX 文档自动同步

## 变更记录

`changes/` 目录按功能组织变更：

```
changes/
├── 用户登录功能/
│   ├── prd.md
│   ├── architecture.md
│   └── uiux.md
└── 支付功能/
    ├── prd.md
    ├── architecture.md
    └── uiux.md
```

## 自动化

Super Dev CLI 会自动维护此目录：

- 运行 `super-dev init` 时创建 `project.md`
- 运行 `super-dev create` 时更新规范和文档
- 运行 `super-dev update` 时同步变更

**不要手动编辑此目录下的文件**，除非你清楚知道自己在做什么。
