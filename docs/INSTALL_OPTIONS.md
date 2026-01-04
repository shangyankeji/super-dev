# Super Dev 安装方式

> 📦 包已构建成功！选择最适合你的安装方式。

---

## ✅ 验证结果

包已构建并测试通过：
- ✅ `super_dev-1.0.1-py3-none-any.whl` (161 KB)
- ✅ `super_dev-1.0.1.tar.gz` (150 KB)
- ✅ 本地安装测试通过
- ✅ 命令功能正常

---

## 🚀 3 种安装方式

### 方式 1：从 GitHub 直接安装（最简单！）⭐

**推荐新手使用**，无需 PyPI，直接从 GitHub 安装：

```bash
# 安装最新版本
pip install git+https://github.com/shangyankeji/super-dev.git

# 或指定版本
pip install git+https://github.com/shangyankeji/super-dev.git@v1.0.1

# 验证安装
super-dev --version
```

**优点**：
- ✅ 最简单，一条命令搞定
- ✅ 无需 PyPI 账号
- ✅ 总是最新版本
- ✅ 适合快速体验

---

### 方式 2：从源码安装（开发模式）

适合想要修改代码或参与开发的用户：

```bash
# 1. 克隆仓库
git clone https://github.com/shangyankeji/super-dev.git
cd super-dev

# 2. 安装
pip install -e .

# 验证安装
super-dev --version
```

**优点**：
- ✅ 可以修改代码
- ✅ 立即生效修改
- ✅ 适合开发者

---

### 方式 3：从 PyPI 安装（传统方式）

需要先发布到 PyPI（需要 PyPI token），但用户体验最好：

```bash
# 安装
pip install super-dev

# 或使用 uv（更快）
uv pip install super-dev

# 验证安装
super-dev --version
```

**优点**：
- ✅ 标准方式，最熟悉
- ✅ 安装速度快（特别是用 uv）
- ✅ 版本管理清晰

**缺点**：
- ❌ 需要发布到 PyPI
- ❌ 需要配置 token

---

## 🎯 我的推荐

**对于用户**：使用 **方式 1**（从 GitHub 安装）

```bash
pip install git+https://github.com/shangyankeji/super-dev.git
```

**对于开发者**：使用 **方式 2**（从源码安装）

```bash
git clone https://github.com/shangyankeji/super-dev.git
cd super-dev
pip install -e .
```

---

## 📝 更新 README

在 README 中添加这 3 种安装方式：

```markdown
## 安装

### 方式 1：从 GitHub 安装（推荐）⭐

```bash
pip install git+https://github.com/shangyankeji/super-dev.git
```

### 方式 2：从源码安装（开发模式）

```bash
git clone https://github.com/shangyankeji/super-dev.git
cd super-dev
pip install -e .
```

### 方式 3：从 PyPI 安装（需要先发布）

```bash
pip install super-dev
```
```

---

## ❓ 常见问题

### Q: 方式 1 的安装速度慢吗？

A: 不慢，只需要下载一次源码，和从 PyPI 下载差不多。

### Q: 方式 1 可以用 uv 吗？

A: 可以！`uv pip install git+https://github.com/shangyankeji/super-dev.git`

### Q: 一定要发布到 PyPI 吗？

A: **不需要！** 方式 1 和 2 都很好用。发布到 PyPI 只是让安装更标准。

### Q: 如何更新？

A:
- 方式 1: 重新运行安装命令
- 方式 2: `git pull`
- 方式 3: `pip install --upgrade super-dev`

---

## 🎉 总结

**不需要发布到 PyPI**，用户也可以轻松安装 Super Dev！

推荐直接在 README 中写：

```bash
pip install git+https://github.com/shangyankeji/super-dev.git
```

这样最简单！
