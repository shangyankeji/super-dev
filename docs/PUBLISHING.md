# 发布指南

> 如何将 Super Dev 发布到 PyPI

---

## 📋 目录

- [准备发布](#准备发布)
- [发布到 PyPI（手动）](#发布到-pypi手动)
- [发布到 PyPI（自动）](#发布到-pypi自动)
- [安装方式](#安装方式)
- [发布后验证](#发布后验证)

---

## 准备发布

### 1. 安装发布工具

```bash
# 使用 uv（推荐）
pip install uv

# 或使用传统方式
pip install build twine
```

### 2. 配置 PyPI Token

**方法 1：使用 API Token（推荐）**

1. 访问 [pypi.org](https://pypi.org) 并登录
2. 进入 Account settings → API tokens
3. 创建一个新的 API token
   - Scope: "Entire account"（第一次发布）或 "Project: super-dev"
   - Token name: "GitHub Actions" 或 "Development"
4. **复制 token**（只显示一次！）

**方法 2：使用 username/password**

```bash
# 创建 ~/.pypirc
cat > ~/.pypirc << 'EOF'
[distutils]
index-servers =
    pypi
    testpypi

[pypi]
username = __token__
password = <your-pypi-token>

[testpypi]
username = __token__
password = <your-testpypi-token>
repository = https://test.pypi.org/legacy/
EOF

chmod 600 ~/.pypirc
```

### 3. 配置 GitHub Actions（可选，用于自动发布）

在 GitHub 仓库中设置 Secret：

1. 进入 Settings → Secrets and variables → Actions
2. 点击 "New repository secret"
3. Name: `PYPI_API_TOKEN`
4. Value: 粘贴你的 PyPI API token
5. 点击 "Add secret"

---

## 发布到 PyPI（手动）

### 步骤 1：更新版本号

```bash
# 编辑 pyproject.toml
vim pyproject.toml

# 更新 version = "1.0.2"（不要忘记更新 CHANGELOG.md）
```

### 步骤 2：构建包

```bash
# 清理旧的构建
rm -rf dist/ build/ *.egg-info

# 使用 uv 构建（推荐）
uv build

# 或使用传统方式
python -m build
```

### 步骤 3：检查包

```bash
# 检查包的元数据
twine check dist/*

# 预览将要发布的内容
ls -lh dist/
```

### 步骤 4：测试发布（可选）

```bash
# 发布到 TestPyPI
twine upload --repository testpypi dist/*

# 测试安装
pip install --index-url https://test.pypi.org/simple/ super-dev
```

### 步骤 5：正式发布

```bash
# 发布到 PyPI
twine upload dist/*
```

**输入**：
- Username: `__token__`
- Password: `<your-pypi-token>`

### 步骤 6：验证发布

```bash
# 等待 1-2 分钟后
pip install super-dev

# 验证安装
super-dev --version
```

---

## 发布到 PyPI（自动）

### 配置 GitHub Actions

**文件：`.github/workflows/publish.yml`**

```yaml
name: Publish to PyPI

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: read
  id-token: write

jobs:
  pypi-publish:
    name: Upload release to PyPI
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: "3.11"

      - name: Install uv
        run: |
          curl -LsSf https://astral.sh/uv/install.sh | sh
          echo "$HOME/.cargo/bin" >> $GITHUB_PATH

      - name: Build package
        run: |
          uv build

      - name: Publish to PyPI
        uses: pypa/gh-action-pypi-publish@release/v1
        with:
          password: ${{ secrets.PYPI_API_TOKEN }}
```

### 使用自动发布

```bash
# 1. 更新版本号
vim pyproject.toml

# 2. 提交更改
git add .
git commit -m "bump: version 1.0.2"

# 3. 创建 Git tag
git tag v1.0.2

# 4. 推送 tag（触发自动发布）
git push origin v1.0.2
```

GitHub Actions 会自动：
1. 构建包
2. 发布到 PyPI
3. 创建 GitHub Release

---

## 安装方式

### 使用 pip（传统方式）

```bash
# 从 PyPI 安装
pip install super-dev

# 升级
pip install --upgrade super-dev

# 开发模式安装
pip install -e .

# 安装开发依赖
pip install -e ".[dev]"
```

### 使用 uv（推荐）

```bash
# 从 PyPI 安装（更快）
uv pip install super-dev

# 升级
uv pip install --upgrade super-dev

# 开发模式安装
uv pip install -e .

# 安装开发依赖
uv pip install -e ".[dev]"
```

### 为什么推荐 uv？

| 特性 | pip | uv |
|:---|:---|:---|
| **安装速度** | 基准 | **10-100x 更快** |
| **依赖解析** | 较慢 | **极快（Rust 实现）** |
| **磁盘使用** | 较高 | **更低** |
| **兼容性** | 完全兼容 | 完全兼容 pip |
| **更新频率** | 较慢 | **活跃开发中** |

---

## 发布后验证

### 1. 检查 PyPI 页面

访问 [https://pypi.org/project/super-dev/](https://pypi.org/project/super-dev/)

确认：
- ✅ 版本号正确
- ✅ 描述显示正常
- ✅ 项目链接有效
- ✅ 文档链接正常

### 2. 测试安装

```bash
# 清理环境（可选）
python -m venv test_env
source test_env/bin/activate  # Windows: test_env\Scripts\activate

# 安装
pip install super-dev

# 验证版本
super-dev --version

# 测试命令
super-dev --help
super-dev expert --list

# 退出测试环境
deactivate
rm -rf test_env
```

### 3. 测试不同安装方式

```bash
# 测试 pip
pip install super-dev

# 测试 uv
uv pip install --force-reinstall super-dev

# 测试开发模式
git clone https://github.com/shangyankeji/super-dev.git
cd super-dev
pip install -e .
```

### 4. 检查文档

确认文档中的安装说明正确：

- ✅ README.md
- ✅ README_EN.md
- ✅ docs/QUICKSTART.md
- ✅ docs/PUBLISHING.md

---

## 常见问题

### Q1：发布时提示 "HTTPError: 400 Bad Request"

**原因**：版本号已存在

**解决**：
```bash
# 更新版本号
vim pyproject.toml  # 改为 1.0.3

# 重新构建
rm -rf dist/
uv build

# 重新发布
twine upload dist/*
```

### Q2：发布时提示 "403 Forbidden"

**原因**：API token 无效或过期

**解决**：
1. 重新生成 API token
2. 更新 ~/.pypirc 或 GitHub Secret
3. 重新发布

### Q3：如何撤销已发布的版本？

**注意**：PyPI **不允许删除已发布的版本**

**解决**：
- 发布新版本修复问题
- 在 PyPI 上标记为 "Yanked"（仅适用于包管理器）
- 紧急情况联系 PyPI 支持

### Q4：如何发布预发布版本？

```bash
# 在 pyproject.toml 中使用预发布版本号
version = "1.0.2a1"  # Alpha
version = "1.0.2b1"  # Beta
version = "1.0.2rc1" # Release Candidate

# 或使用 post-release
version = "1.0.2.post1"
```

### Q5：测试安装时找不到包

**原因**：PyPI 索引更新延迟

**解决**：
```bash
# 清理 pip 缓存
pip cache purge

# 使用 --no-cache-dir
pip install --no-cache-dir super-dev

# 或等待 5-10 分钟后重试
```

---

## 发布检查清单

**发布前**：
- [ ] 更新版本号（pyproject.toml）
- [ ] 更新 CHANGELOG.md
- [ ] 运行所有测试（`pytest`）
- [ ] 检查代码质量（`ruff check .`, `mypy .`）
- [ ] 更新文档（README, QUICKSTART）
- [ ] 创建 Git commit
- [ ] 创建 Git tag

**发布中**：
- [ ] 清理旧的构建文件（`rm -rf dist/ build/`）
- [ ] 构建包（`uv build`）
- [ ] 检查包（`twine check dist/*`）
- [ ] 测试发布（可选，TestPyPI）
- [ ] 正式发布（`twine upload dist/*`）

**发布后**：
- [ ] 验证 PyPI 页面
- [ ] 测试安装（`pip install super-dev`）
- [ ] 测试命令（`super-dev --version`）
- [ ] 推送 Git tag
- [ ] 创建 GitHub Release
- [ ] 通知用户（更新说明）

---

## 资源链接

- **PyPI**: https://pypi.org
- **TestPyPI**: https://test.pypi.org
- **uv 文档**: https://github.com/astral-sh/uv
- **Twine 文档**: https://packaging.python.org/tutorials/distributing-packages/
- **PyPI 发布指南**: https://packaging.python.org/en/latest/guides/distributing-packages-using-setuptools/

---

**准备好发布了吗？**

```bash
# 一键发布脚本
#!/bin/bash
set -e

echo "🚀 开始发布 Super Dev..."

# 1. 更新版本号
echo "📝 请检查 pyproject.toml 中的版本号："
grep "version =" pyproject.toml
read -p "版本号正确吗？(y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ 发布已取消"
    exit 1
fi

# 2. 运行测试
echo "🧪 运行测试..."
pytest

# 3. 清理
echo "🧹 清理旧的构建..."
rm -rf dist/ build/ *.egg-info

# 4. 构建
echo "📦 构建包..."
uv build

# 5. 检查
echo "🔍 检查包..."
twine check dist/*

# 6. 发布
echo "🚀 发布到 PyPI..."
twine upload dist/*

echo "✅ 发布完成！"
echo "📦 请访问: https://pypi.org/project/super-dev/"
```

保存为 `scripts/publish.sh`：

```bash
chmod +x scripts/publish.sh
./scripts/publish.sh
```
