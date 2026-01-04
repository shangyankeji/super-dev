# Scripts 目录

此目录包含 Super Dev 的辅助脚本。

## 可用脚本

### publish.sh

**功能**：自动发布 Super Dev 到 PyPI

**使用方法**：

```bash
# 运行发布脚本
./scripts/publish.sh
```

**脚本会自动**：
1. ✅ 检查版本号
2. ✅ 检查 Git 状态（确保没有未提交的更改）
3. ✅ 运行测试
4. ✅ 检查代码质量（ruff）
5. ✅ 清理旧的构建文件
6. ✅ 构建包
7. ✅ 检查包
8. ✅ 发布到 PyPI
9. ✅ 创建并推送 Git tag（可选）

**要求**：
- Python 3.10+
- 已安装 `twine`: `pip install twine`
- 已配置 PyPI API token（参见 `docs/PUBLISHING.md`）

---

## 首次发布配置

### 1. 安装发布工具

```bash
# 安装 uv（推荐）
pip install uv

# 或使用传统工具
pip install build twine
```

### 2. 配置 PyPI Token

```bash
# 创建 ~/.pypirc
cat > ~/.pypirc << 'EOF'
[distutils]
index-servers =
    pypi

[pypi]
username = __token__
password = <your-pypi-token>
EOF

chmod 600 ~/.pypirc
```

### 3. 测试发布脚本

```bash
# 不实际发布，只测试流程
./scripts/publish.sh
```

**详细发布指南**：参见 [`docs/PUBLISHING.md`](../docs/PUBLISHING.md)
