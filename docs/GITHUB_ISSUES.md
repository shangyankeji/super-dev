# GitHub 仓库问题修复

> 修复 GitHub 上显示的两个主要问题

---

## 📋 问题列表

### ✅ 问题 1：License 配置警告（已修复）

**警告信息**：
```
SetuptoolsDeprecationWarning: `project.license` as a TOML table is deprecated
Please use a simple string containing a SPDX license expression for `project.license`.
```

**原因**：使用了旧的 license 配置格式

**修复**：
```toml
# 旧格式（已弃用）
license = {text = "MIT"}

# 新格式（推荐）
license = "MIT"
```

**状态**：✅ 已修复

---

### ⚠️ 问题 2：Workflow 权限警告

**警告信息**：
> This repository does not have the `workflow` scope enabled

**原因**：尝试添加 `.github/workflows/publish.yml` 时，GitHub OAuth token 缺少 `workflow` 权限。

**影响**：
- ❌ 无法通过 Git 推送 GitHub Actions workflow 文件
- ✅ 可以正常推送代码和其他文件
- ✅ Super Dev 本身完全正常工作

**解决方案**：

**选项 A：不使用 GitHub Actions 自动发布**（推荐，当前方案）

- ✅ 使用手动发布脚本 `./scripts/publish.sh`
- ✅ 用户可以直接从 GitHub 安装：`pip install git+https://github.com/shangyankeji/super-dev.git`
- ✅ 无需配置 workflow 权限

**选项 B：启用 workflow 权限**（如果需要自动发布）

1. 访问 GitHub token 设置页面
2. 编辑 OAuth token 权限
3. 勾选 `workflow` scope
4. 保存并重新推送

**状态**：⚠️ 已知晓，不影响使用（使用选项 A）

---

## 🎯 推荐做法

### 当前方案（推荐）

✅ **不使用 GitHub Actions 自动发布**

理由：
1. Super Dev 已经可以从 GitHub 直接安装
2. 用户无需等待 PyPI 发布即可使用
3. 减少配置复杂度
4. 避免权限问题

安装方式：
```bash
pip install git+https://github.com/shangyankeji/super-dev.git
```

### 未来方案（可选）

如果需要更标准的发布流程，可以：

1. 获取 PyPI API token
2. 配置 GitHub Secret: `PYPI_API_TOKEN`
3. 手动在 GitHub 网页界面创建 workflow 文件
4. 启用 workflow 权限

---

## 📝 已修复的配置

### pyproject.toml

```toml
[project]
name = "super-dev"
version = "1.0.1"
license = "MIT"  # ✅ 使用新格式
classifiers = [
    # ✅ 移除了旧的 "License :: OSI Approved :: MIT License"
    "Development Status :: 4 - Beta",
    "Intended Audience :: Developers",
    # ... 其他 classifiers
]
```

---

## ✅ 验证修复

```bash
# 清理旧构建
rm -rf dist/ build/ *.egg-info

# 重新构建（应该没有 license 警告）
uv build

# 检查构建结果
twine check dist/*
```

**预期结果**：
- ✅ 没有 `SetuptoolsDeprecationWarning` 警告
- ✅ 只剩下 byte-compiling 警告（正常）
- ✅ 包检查通过

---

## 🎉 总结

**已修复**：
- ✅ License 配置警告（使用新格式）
- ✅ 移除弃用的 license classifier

**已说明**：
- ⚠️ Workflow 权限警告（不影响使用）
- ✅ 推荐使用手动发布和 GitHub 直接安装

**用户可以**：
- ✅ 正常使用 Super Dev
- ✅ 从 GitHub 直接安装
- ✅ 无需关心这些警告
