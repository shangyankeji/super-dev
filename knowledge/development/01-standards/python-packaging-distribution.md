---
id: python-packaging-distribution
title: Python打包分发指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [agent, checklist, development, distribution, packaging, python, 实战代码示例, 常见陷阱]
quality_score: 70
last_updated: 2026-06-15
---
# Python打包分发指南

## 概述
Python打包生态经历了从setup.py到pyproject.toml的演进。本指南覆盖现代Python项目的打包、构建、分发全流程,包括setuptools/wheel/twine/uv/poetry等工具链的选型与实战。

## 核心概念

### 1. 打包工具演进
| 阶段 | 工具 | 配置文件 | 状态 |
|------|------|----------|------|
| 传统 | setuptools + setup.py | setup.py/setup.cfg | 仍广泛使用 |
| 过渡 | setuptools + pyproject.toml | pyproject.toml | 推荐 |
| 现代 | Poetry/PDM/Hatch | pyproject.toml | 推荐 |
| 新锐 | uv | pyproject.toml | 快速增长 |

### 2. 核心标准
- **PEP 517**: 构建系统接口标准
- **PEP 518**: pyproject.toml中声明构建依赖
- **PEP 621**: pyproject.toml中的项目元数据标准
- **PEP 660**: 可编辑安装(editable installs)标准

### 3. 分发格式
- **sdist**: 源代码分发包(.tar.gz),包含构建所需全部源文件
- **wheel**: 预构建二进制包(.whl),安装更快、无需编译
- **egg**: 已废弃格式,不应使用

## 实战代码示例

### pyproject.toml完整配置(setuptools)

```toml
[build-system]
requires = ["setuptools>=68.0", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "my-awesome-tool"
version = "1.2.0"
description = "一个示例Python工具"
readme = "README.md"
license = {text = "MIT"}
requires-python = ">=3.10"
authors = [
    {name = "Dev Team", email = "dev@example.com"},
]
keywords = ["tool", "automation", "cli"]
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: MIT License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
]

dependencies = [
    "click>=8.0",
    "httpx>=0.25",
    "pydantic>=2.0",
    "rich>=13.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0",
    "pytest-cov>=4.0",
    "pytest-asyncio>=0.21",
    "ruff>=0.1.0",
    "mypy>=1.5",
    "black>=23.0",
]
docs = [
    "sphinx>=7.0",
    "sphinx-rtd-theme>=1.3",
]
all = ["my-awesome-tool[dev,docs]"]

[project.urls]
Homepage = "https://github.com/org/my-awesome-tool"
Documentation = "https://my-awesome-tool.readthedocs.io"
Repository = "https://github.com/org/my-awesome-tool"
Issues = "https://github.com/org/my-awesome-tool/issues"
Changelog = "https://github.com/org/my-awesome-tool/blob/main/CHANGELOG.md"

[project.scripts]
my-tool = "my_awesome_tool.cli:main"
my-tool-admin = "my_awesome_tool.admin:main"

[project.entry-points."my_tool.plugins"]
json-output = "my_awesome_tool.plugins.json:JsonPlugin"
csv-output = "my_awesome_tool.plugins.csv:CsvPlugin"

[tool.setuptools.packages.find]
where = ["."]
include = ["my_awesome_tool*"]
exclude = ["tests*", "docs*"]

[tool.setuptools.package-data]
my_awesome_tool = ["templates/*.html", "static/*.css", "py.typed"]
```

### Poetry配置

```toml
[tool.poetry]
name = "my-awesome-tool"
version = "1.2.0"
description = "一个示例Python工具"
authors = ["Dev Team <dev@example.com>"]
readme = "README.md"
license = "MIT"
packages = [{include = "my_awesome_tool"}]

[tool.poetry.dependencies]
python = "^3.10"
click = "^8.0"
httpx = "^0.25"
pydantic = "^2.0"

[tool.poetry.group.dev.dependencies]
pytest = "^7.0"
ruff = "^0.1.0"
mypy = "^1.5"

[tool.poetry.scripts]
my-tool = "my_awesome_tool.cli:main"

[build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"
```

### uv工作流

```bash
# 安装uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# 创建项目
uv init my-project
cd my-project

# 添加依赖
uv add click httpx pydantic
uv add --dev pytest ruff mypy

# 锁定依赖
uv lock

# 同步安装
uv sync

# 运行脚本
uv run python -m my_project
uv run pytest

# 构建
uv build

# 发布
uv publish --token $PYPI_TOKEN
```

### 版本管理策略

```python
# 方案1: 单一来源版本(__init__.py)
# my_awesome_tool/__init__.py
__version__ = "1.2.0"

# pyproject.toml
[tool.setuptools.dynamic]
version = {attr = "my_awesome_tool.__version__"}

# 方案2: 使用setuptools-scm从git tag自动获取
[build-system]
requires = ["setuptools>=68.0", "setuptools-scm>=8.0"]

[tool.setuptools_scm]
write_to = "my_awesome_tool/_version.py"

# 方案3: 使用bump2version管理
# .bumpversion.cfg
# [bumpversion]
# current_version = 1.2.0
# commit = True
# tag = True
# [bumpversion:file:pyproject.toml]
# [bumpversion:file:my_awesome_tool/__init__.py]
```

### 构建与发布流程

```bash
# 清理旧构建
rm -rf dist/ build/ *.egg-info

# 构建sdist和wheel
python -m build

# 检查包质量
twine check dist/*

# 发布到TestPyPI
twine upload --repository testpypi dist/*

# 从TestPyPI安装验证
pip install --index-url https://test.pypi.org/simple/ my-awesome-tool

# 正式发布到PyPI
twine upload dist/*
```

### CI自动发布

```yaml
# .github/workflows/publish.yml
name: Publish to PyPI
on:
  release:
    types: [published]

jobs:
  publish:
    runs-on: ubuntu-latest
    environment: release
    permissions:
      id-token: write  # 用于Trusted Publisher
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      - name: Install build tools
        run: pip install build
      - name: Build package
        run: python -m build
      - name: Check package
        run: |
          pip install twine
          twine check dist/*
      - name: Publish to PyPI
        uses: pypa/gh-action-pypi-publish@release/v1
        # 使用Trusted Publisher,无需token
```

### 项目结构模板

```
my-awesome-tool/
├── pyproject.toml          # 项目元数据和构建配置
├── README.md
├── LICENSE
├── CHANGELOG.md
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── publish.yml
├── my_awesome_tool/        # 源代码包
│   ├── __init__.py         # 版本号在此定义
│   ├── py.typed            # PEP 561类型标记
│   ├── cli.py              # CLI入口
│   ├── core/
│   │   ├── __init__.py
│   │   └── engine.py
│   ├── models/
│   │   ├── __init__.py
│   │   └── schemas.py
│   └── templates/          # 包内数据文件
│       └── default.html
├── tests/
│   ├── conftest.py
│   ├── unit/
│   └── integration/
└── docs/
    ├── conf.py
    └── index.rst
```

### Namespace Packages

```python
# 用于组织内部拆分多个包共享命名空间
# 包1: myorg-core
# myorg/core/__init__.py

# 包2: myorg-utils
# myorg/utils/__init__.py

# pyproject.toml (包1)
[tool.setuptools.packages.find]
include = ["myorg.core*"]

# 用户可以同时安装:
# pip install myorg-core myorg-utils
# import myorg.core
# import myorg.utils
```

### 条件依赖与平台特定

```toml
[project]
dependencies = [
    "colorama; sys_platform == 'win32'",
    "uvloop>=0.17; sys_platform != 'win32'",
    "importlib-metadata>=4.0; python_version < '3.12'",
    "tomllib; python_version < '3.11'",
]
```

## 最佳实践

### 1. pyproject.toml优先
- 新项目必须使用pyproject.toml,不再新建setup.py
- 工具配置(ruff/black/mypy/pytest)统一放pyproject.toml
- 使用PEP 621标准的`[project]`表,而非工具私有格式

### 2. 依赖管理
- 生产依赖使用兼容版本(`>=1.0,<2.0`或`^1.0`)
- 开发依赖放`[project.optional-dependencies]`或poetry的dev group
- 锁文件(poetry.lock/uv.lock)提交到版本控制
- 定期更新依赖并运行测试

### 3. 版本策略
- 遵循语义化版本(SemVer): MAJOR.MINOR.PATCH
- 版本号单一来源,避免多处定义
- 使用git tag触发CI/CD发布

### 4. 包质量检查
- 发布前用`twine check`验证元数据
- 在TestPyPI上测试安装
- 确保README能在PyPI正确渲染
- 包含py.typed标记支持类型检查

### 5. 安全分发
- 使用Trusted Publisher(PyPI OIDC)替代API token
- 启用2FA保护PyPI账户
- 签名发布包(sigstore)

## 常见陷阱

### 陷阱1: MANIFEST.in遗漏文件
```bash
# 错误: sdist中缺少数据文件
# 正确: 在pyproject.toml中声明
[tool.setuptools.package-data]
my_tool = ["templates/**", "static/**", "*.yaml"]
```

### 陷阱2: 混淆install_requires和extras_require
```toml
# 错误: 把测试依赖放入主依赖
dependencies = ["pytest", "ruff"]  # 不应该!

# 正确: 使用optional-dependencies
[project.optional-dependencies]
dev = ["pytest", "ruff"]
```

### 陷阱3: 忘记__init__.py
```python
# Python 3中regular package仍需要__init__.py
# namespace package才不需要
# 确保每个需要导入的目录都有__init__.py
```

### 陷阱4: 版本号不一致
```python
# 错误: pyproject.toml写1.2.0, __init__.py写1.1.0
# 正确: 使用动态版本,单一来源
[project]
dynamic = ["version"]
[tool.setuptools.dynamic]
version = {attr = "my_tool.__version__"}
```

### 陷阱5: 开发安装未用editable模式
```bash
# 错误: pip install .  (每次改代码都要重新安装)
# 正确: pip install -e ".[dev]"  (可编辑安装)
```

## Agent Checklist

### 项目初始化
- [ ] 使用pyproject.toml作为唯一配置文件
- [ ] 遵循PEP 621元数据标准
- [ ] 版本号有单一来源
- [ ] 项目结构符合Python包规范

### 依赖管理
- [ ] 生产/开发依赖分离
- [ ] 版本约束合理(不过宽也不过窄)
- [ ] 锁文件已提交版本控制
- [ ] 平台特定依赖使用环境标记

### 构建发布
- [ ] CI/CD自动化构建和发布
- [ ] twine check通过
- [ ] TestPyPI验证通过
- [ ] 使用Trusted Publisher或安全token

### 包质量
- [ ] README在PyPI正确渲染
- [ ] classifiers和关键词准确
- [ ] 包含LICENSE文件
- [ ] py.typed标记已添加(如支持类型)
- [ ] 数据文件正确包含在分发包中
