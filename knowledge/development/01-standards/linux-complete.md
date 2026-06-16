---
id: linux-complete
title: Linux命令行完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, linux, 学习路径, 最佳实践, 核心命令, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Linux命令行完整指南

## 概述
Linux命令行是系统管理和开发的核心工具。本指南覆盖常用命令、Shell脚本和最佳实践。

## 核心命令

### 1. 文件操作

```bash
# 列出文件
ls -la              # 详细列表
ls -lh              # 人类可读大小
tree -L 2           # 树状结构

# 创建/删除
mkdir -p dir1/dir2  # 创建多级目录
touch file.txt      # 创建文件
rm -rf directory    # 强制删除目录
cp -r src dest      # 递归复制
mv old new          # 移动/重命名

# 查看文件
cat file.txt        # 查看全部
less file.txt       # 分页查看
head -n 10 file.txt # 前10行
tail -f logfile     # 实时查看日志

# 搜索
find . -name "*.py"         # 按名称搜索
grep -r "pattern" .         # 搜索内容
grep -i "error" /var/log/*  # 不区分大小写
```

### 2. 权限管理

```bash
# 修改权限
chmod 755 script.sh      # rwxr-xr-x
chmod +x script.sh       # 添加执行权限
chmod -R 644 directory   # 递归修改

# 修改所有者
chown user:group file
chown -R user:group dir

# 查看权限
ls -l file.txt
# -rwxr-xr-x 1 user group 1234 Jan 1 12:00 file.txt
```

### 3. 进程管理

```bash
# 查看进程
ps aux                  # 所有进程
ps aux | grep python    # 过滤进程
top                     # 实时监控
htop                    # 增强版监控

# 后台运行
nohup python app.py &   # 后台运行
bg                      # 后台运行
fg                      # 前台运行
jobs                    # 查看任务

# 结束进程
kill PID                # 优雅终止
kill -9 PID             # 强制终止
pkill -f "python"       # 按名称终止
```

### 4. 网络命令

```bash
# 连接测试
ping google.com
curl -I https://example.com
wget https://example.com/file

# 端口查看
netstat -tulpn          # 查看端口
ss -tulpn               # 现代版
lsof -i :8000           # 查看端口占用

# SSH
ssh user@server.com
scp file.txt user@server:/path/
rsync -avz local/ user@server:/remote/
```

### 5. 磁盘管理

```bash
# 查看磁盘
df -h                   # 磁盘使用
du -sh directory        # 目录大小
du -h --max-depth=1     # 逐层查看

# 挂载
mount /dev/sdb1 /mnt
umount /mnt
```

### 6. 压缩解压

```bash
# tar
tar -czf archive.tar.gz dir/    # 压缩
tar -xzf archive.tar.gz         # 解压

# zip
zip -r archive.zip dir/
unzip archive.zip

# 查看压缩包
tar -tzf archive.tar.gz
unzip -l archive.zip
```

### 7. Shell脚本

```bash
#!/bin/bash

# 变量
NAME="Alice"
echo "Hello, $NAME"

# 条件
if [ -f "file.txt" ]; then
    echo "File exists"
elif [ -d "dir" ]; then
    echo "Directory exists"
else
    echo "Not found"
fi

# 循环
for i in {1..5}; do
    echo $i
done

# 函数
function greet() {
    local name=$1
    echo "Hello, $name"
}

greet "Alice"

# 参数
echo "Script: $0"
echo "First arg: $1"
echo "All args: $@"
```

## 最佳实践

### ✅ DO

1. **使用tab补全**
```bash
cd /path/to/<TAB>
```

2. **使用历史命令**
```bash
history | grep command
!123  # 执行第123条命令
```

3. **使用alias**
```bash
alias ll='ls -lah'
alias gs='git status'
```

### ❌ DON'T

1. **不要滥用sudo**
```bash
# ❌ 危险
sudo rm -rf /

# ✅ 谨慎使用
sudo rm -rf /specific/path
```

2. **不要忽略错误**
```bash
# ❌ 差
rm file.txt

# ✅ 好
rm file.txt || exit 1
```

## 学习路径

### 初级 (1-2周)
1. 基础命令
2. 文件操作
3. 权限管理

### 中级 (2-3周)
1. Shell脚本
2. 进程管理
3. 网络命令

### 高级 (2-4周)
1. 系统调优
2. 自动化脚本
3. 故障排查

---

**知识ID**: `linux-complete`  
**领域**: development  
**类型**: standards  
**难度**: beginner  
**质量分**: 94  
**维护者**: devops-team@superdev.com  
**最后更新**: 2026-03-28
