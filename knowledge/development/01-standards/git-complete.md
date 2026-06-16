---
id: git-complete
title: Git完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, git, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Git完整指南

## 概述
Git是分布式版本控制系统,用于跟踪代码变更、协作开发。本指南覆盖Git基础、分支管理、团队协作和最佳实践。

## 核心概念

### 1. 基础命令

**初始化和配置**:
```bash
# 初始化仓库
git init

# 克隆远程仓库
git clone https://github.com/user/repo.git

# 配置用户信息
git config --global user.name "Your Name"
git config --global user.email "your@email.com"

# 查看配置
git config --list
```

**基本操作**:
```bash
# 查看状态
git status

# 添加到暂存区
git add filename
git add .  # 添加所有文件

# 提交
git commit -m "Add new feature"

# 查看历史
git log
git log --oneline --graph

# 查看差异
git diff  # 工作区vs暂存区
git diff --staged  # 暂存区vs最新提交
```

### 2. 分支管理

**创建和切换分支**:
```bash
# 创建分支
git branch feature-branch

# 切换分支
git checkout feature-branch

# 创建并切换
git checkout -b feature-branch

# 新语法(推荐)
git switch -c feature-branch

# 查看分支
git branch
git branch -a  # 包括远程分支

# 删除分支
git branch -d feature-branch
git branch -D feature-branch  # 强制删除
```

**合并分支**:
```bash
# 合并到当前分支
git merge feature-branch

# 解决冲突后
git add resolved-file.txt
git commit -m "Resolve merge conflict"

# 变基(推荐)
git checkout feature-branch
git rebase main

# 解决冲突
git add resolved-file.txt
git rebase --continue
```

### 3. 远程仓库

**操作远程仓库**:
```bash
# 查看远程仓库
git remote -v

# 添加远程仓库
git remote add origin https://github.com/user/repo.git

# 推送
git push origin main
git push -u origin main  # 设置上游

# 拉取
git pull origin main

# 抓取(不合并)
git fetch origin

# 查看远程分支
git branch -r
```

### 4. 工作流

**Git Flow**:
```bash
# 主分支
main       # 生产环境
develop    # 开发环境

# 辅助分支
feature/*  # 功能分支
release/*  # 发布分支
hotfix/*   # 紧急修复

# 创建功能分支
git checkout -b feature/login develop

# 完成功能
git checkout develop
git merge --no-ff feature/login
git branch -d feature/login

# 创建发布分支
git checkout -b release/1.0 develop

# 合并到main
git checkout main
git merge --no-ff release/1.0
git tag -a v1.0
```

**GitHub Flow**(简化版):
```bash
# 从main创建分支
git checkout -b feature-branch main

# 开发并提交
git add .
git commit -m "Add feature"

# 推送到远程
git push origin feature-branch

# 在GitHub创建Pull Request
# 代码审查后合并

# 更新本地main
git checkout main
git pull origin main
```

### 5. 撤销操作

**撤销修改**:
```bash
# 撤销工作区修改
git checkout -- filename
git restore filename  # 新语法

# 撤销暂存
git reset HEAD filename
git restore --staged filename  # 新语法

# 修改最后一次提交
git commit --amend

# 回退到指定提交
git reset --hard <commit-hash>

# 撤销已推送的提交
git revert <commit-hash>
```

### 6. 储藏(Stash)

```bash
# 储藏当前修改
git stash
git stash save "WIP: feature login"

# 查看储藏列表
git stash list

# 恢复储藏
git stash pop  # 恢复并删除
git stash apply  # 恢复但不删除

# 删除储藏
git stash drop
```

### 7. 标签

```bash
# 创建标签
git tag v1.0.0
git tag -a v1.0.0 -m "Release version 1.0.0"

# 查看标签
git tag
git show v1.0.0

# 推送标签
git push origin v1.0.0
git push origin --tags

# 删除标签
git tag -d v1.0.0
git push origin --delete v1.0.0
```

### 8. 高级技巧

**交互式变基**:
```bash
# 编辑最近3次提交
git rebase -i HEAD~3

# 压缩提交
pick abc1234 First commit
squash def5678 Second commit
squash ghi9012 Third commit
```

**Cherry-pick**:
```bash
# 选择性合并提交
git cherry-pick <commit-hash>
```

**二分查找**:
```bash
# 查找引入bug的提交
git bisect start
git bisect bad  # 当前提交有bug
git bisect good v1.0  # v1.0没有bug

# Git会自动二分查找
git bisect good  # 标记为好
git bisect bad   # 标记为坏

# 找到后重置
git bisect reset
```

## 最佳实践

### ✅ DO

1. **编写清晰的提交信息**
```bash
# ✅ 好
git commit -m "feat: Add user authentication

- Add login/logout endpoints
- Implement JWT token validation
- Add password hashing with bcrypt

Closes #123"

# ❌ 差
git commit -m "fix"
```

2. **频繁提交,小步前进**
```bash
# ✅ 好: 逻辑完整的提交
git add authentication.py
git commit -m "feat: Add authentication module"

# ❌ 差: 一次提交太多
git add .
git commit -m "Add everything"
```

3. **使用.gitignore**
```
# .gitignore
__pycache__/
*.pyc
.env
venv/
node_modules/
.DS_Store
*.log
```

### ❌ DON'T

1. **不要提交敏感信息**
```bash
# ❌ 差
git add .env
git commit -m "Add environment variables"

# ✅ 好: 使用.env.example
git add .env.example
```

2. **不要强制推送到main**
```bash
# ❌ 危险
git push --force origin main

# ✅ 安全: 创建新提交
git revert <bad-commit>
```

3. **不要提交大文件**
```bash
# ❌ 差
git add large-dataset.csv

# ✅ 好: 使用Git LFS
git lfs track "*.csv"
git add large-dataset.csv
```

## 学习路径

### 初级 (1周)
1. 基础命令
2. 分支管理
3. 远程仓库

### 中级 (1-2周)
1. 工作流
2. 冲突解决
3. 撤销操作

### 高级 (2-3周)
1. 交互式变基
2. Git hooks
3. CI/CD集成

### 专家级 (持续)
1. Git内部原理
2. 性能优化
3. 大规模团队协作

---

**知识ID**: `git-complete`  
**领域**: development  
**类型**: standards  
**难度**: beginner  
**质量分**: 94  
**维护者**: dev-team@superdev.com  
**最后更新**: 2026-03-28
