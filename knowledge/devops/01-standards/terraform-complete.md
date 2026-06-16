---
id: terraform-complete
title: Terraform 完整指南
domain: devops
category: 01-standards
difficulty: intermediate
tags: [complete, devops, terraform, 实战配置, 工作流, 核心概念, 概述, 模块设计]
quality_score: 70
last_updated: 2026-06-15
---
# Terraform 完整指南

> 文档版本: v1.0 | 最后更新: 2026-03-28 | 适用范围: Terraform 1.6+ / OpenTofu 1.6+

---

## 目录

1. [概述](#概述)
2. [核心概念](#核心概念)
3. [HCL 语法详解](#hcl-语法详解)
4. [状态管理](#状态管理)
5. [模块设计](#模块设计)
6. [工作流](#工作流)
7. [AWS 实战配置](#aws-实战配置)
8. [安全](#安全)
9. [CI/CD 集成](#cicd-集成)
10. [团队协作](#团队协作)
11. [性能优化](#性能优化)
12. [常见陷阱与反模式](#常见陷阱与反模式)
13. [Agent Checklist](#agent-checklist)

---

## 概述

### IaC 理念

Infrastructure as Code (IaC) 将基础设施的定义、部署和管理全部代码化，像管理应用代码一样管理基础设施。核心原则：

- **声明式定义**：描述期望状态而非操作步骤，Terraform 自动计算差异并执行变更
- **版本控制**：所有基础设施变更通过 Git 追踪，可审计、可回溯
- **可重复性**：同一套代码在任意环境产生一致的结果，消除手动配置漂移
- **自文档化**：代码本身就是基础设施的文档，无需额外维护配置清单
- **协作友好**：变更通过 PR 审查，团队共享统一的基础设施定义

### Terraform vs 竞品对比

| 维度 | Terraform / OpenTofu | Pulumi | CloudFormation | Ansible |
|------|---------------------|--------|----------------|---------|
| 语言 | HCL (声明式 DSL) | Python/TS/Go/C# 等通用语言 | JSON/YAML | YAML (过程式) |
| 云支持 | 多云 (3000+ Provider) | 多云 | 仅 AWS | 多云 (侧重配置管理) |
| 状态管理 | 显式 State 文件 | 托管 / 自管 State | AWS 托管 | 无状态 (幂等模块) |
| 学习曲线 | 中等 (HCL 专用语法) | 低 (复用已有语言) | 中高 (AWS 绑定) | 低 (YAML 编排) |
| Plan/Preview | `terraform plan` 原生支持 | `pulumi preview` | Change Sets | `--check` 模式 |
| 模块生态 | Terraform Registry 海量模块 | Pulumi Registry | AWS 嵌套栈 / Modules | Ansible Galaxy |
| 适合场景 | 基础设施编排 (网络/计算/存储) | 复杂逻辑 + 基础设施 | 纯 AWS 环境 | 配置管理 + 应用部署 |
| 开源协议 | BSL 1.1 (Terraform) / MPL 2.0 (OpenTofu) | Apache 2.0 | 闭源 | GPL 3.0 |

**选型建议**：
- 多云 / 混合云基础设施编排 → Terraform / OpenTofu
- 团队已有强 Python/TS 背景且逻辑复杂 → Pulumi
- 纯 AWS 且已深度使用 AWS 服务 → CloudFormation
- 服务器配置管理 + 应用部署 → Ansible
- Terraform 负责基础设施层，Ansible 负责配置层，二者常配合使用

---

## 核心概念

### Provider

Provider 是 Terraform 与外部 API 交互的插件。每个云平台、SaaS 服务或内部系统都有对应的 Provider。

```hcl
# 声明 Provider 及版本约束
terraform {
  required_version = ">= 1.6.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.40"  # 允许 5.40.x 补丁更新
    }
    random = {
      source  = "hashicorp/random"
      version = ">= 3.6.0, < 4.0.0"
    }
  }
}

# 配置 Provider
provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Environment = var.environment
      ManagedBy   = "terraform"
      Project     = var.project_name
    }
  }
}

# 多 Provider 实例 (alias)
provider "aws" {
  alias  = "us_west"
  region = "us-west-2"
}
```

### Resource

Resource 是 Terraform 管理的核心对象，代表一个基础设施组件。

```hcl
resource "aws_instance" "web" {
  ami           = data.aws_ami.ubuntu.id
  instance_type = var.instance_type
  subnet_id     = aws_subnet.public[0].id

  vpc_security_group_ids = [aws_security_group.web.id]

  root_block_device {
    volume_size = 20
    volume_type = "gp3"
    encrypted   = true
  }

  tags = {
    Name = "${var.project_name}-web"
  }

  lifecycle {
    create_before_destroy = true
    prevent_destroy       = false
    ignore_changes        = [ami]  # AMI 更新由其他流程管理
  }
}
```

### Data Source

Data Source 用于查询外部信息，不创建资源。

```hcl
# 查询最新 Ubuntu AMI
data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"]  # Canonical

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-*-22.04-amd64-server-*"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

# 查询当前 AWS 账户信息
data "aws_caller_identity" "current" {}

# 查询可用区
data "aws_availability_zones" "available" {
  state = "available"
}
```

### Variable

输入变量定义模块的参数化接口。

```hcl
variable "environment" {
  description = "Deployment environment (dev/staging/prod)"
  type        = string

  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be one of: dev, staging, prod."
  }
}

variable "instance_count" {
  description = "Number of EC2 instances to create"
  type        = number
  default     = 2

  validation {
    condition     = var.instance_count >= 1 && var.instance_count <= 20
    error_message = "Instance count must be between 1 and 20."
  }
}

variable "allowed_cidrs" {
  description = "List of CIDR blocks allowed to access the service"
  type        = list(string)
  default     = []
}

variable "tags" {
  description = "Additional tags to apply to all resources"
  type        = map(string)
  default     = {}
}
```

### Output

输出值暴露模块的计算结果，供其他模块引用或展示给用户。

```hcl
output "instance_ids" {
  description = "IDs of created EC2 instances"
  value       = aws_instance.web[*].id
}

output "load_balancer_dns" {
  description = "DNS name of the load balancer"
  value       = aws_lb.main.dns_name
}

output "database_endpoint" {
  description = "RDS instance endpoint"
  value       = aws_db_instance.main.endpoint
  sensitive   = true  # 不在 CLI 输出中显示
}
```

### Local

Local 值用于中间计算，简化重复表达式。

```hcl
locals {
  # 通用标签合并
  common_tags = merge(var.tags, {
    Environment = var.environment
    ManagedBy   = "terraform"
    Project     = var.project_name
  })

  # 根据环境选择实例规格
  instance_type = {
    dev     = "t3.micro"
    staging = "t3.small"
    prod    = "t3.medium"
  }[var.environment]

  # 可用区列表
  azs = slice(data.aws_availability_zones.available.names, 0, 3)

  # CIDR 计算
  private_subnets = [for i, az in local.azs : cidrsubnet(var.vpc_cidr, 8, i)]
  public_subnets  = [for i, az in local.azs : cidrsubnet(var.vpc_cidr, 8, i + 100)]
}
```

### Module

模块是可复用的 Terraform 配置包，封装一组相关资源。

```hcl
# 调用本地模块
module "vpc" {
  source = "./modules/vpc"

  vpc_cidr     = "10.0.0.0/16"
  environment  = var.environment
  project_name = var.project_name
}

# 调用 Registry 模块
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.0"

  cluster_name    = "${var.project_name}-${var.environment}"
  cluster_version = "1.29"
  vpc_id          = module.vpc.vpc_id
  subnet_ids      = module.vpc.private_subnet_ids
}
```

---

## HCL 语法详解

### 类型系统

HCL 具备完整的类型系统，支持原始类型和复合类型。

```hcl
# 原始类型
variable "name"    { type = string }
variable "count"   { type = number }
variable "enabled" { type = bool }

# 集合类型
variable "cidrs"       { type = list(string) }
variable "ports"       { type = set(number) }
variable "tags"        { type = map(string) }

# 结构化类型
variable "database" {
  type = object({
    engine         = string
    engine_version = string
    instance_class = string
    storage_gb     = number
    multi_az       = bool
    backup_retention = optional(number, 7)  # 可选字段 + 默认值
  })
}

# 元组类型 (固定长度、异构)
variable "rule" {
  type = tuple([string, number, string])
  # 例: ["tcp", 443, "0.0.0.0/0"]
}

# any 类型 (推迟到运行时推断)
variable "flexible_input" {
  type    = any
  default = null
}
```

### 表达式

```hcl
# 字符串插值
resource "aws_instance" "app" {
  tags = {
    Name = "${var.project_name}-${var.environment}-app-${count.index + 1}"
  }
}

# 多行字符串 (heredoc)
resource "aws_iam_policy" "example" {
  policy = <<-EOT
    {
      "Version": "2012-10-17",
      "Statement": [
        {
          "Effect": "Allow",
          "Action": "s3:GetObject",
          "Resource": "arn:aws:s3:::${var.bucket_name}/*"
        }
      ]
    }
  EOT
}

# 引用其他资源
resource "aws_security_group_rule" "ingress" {
  security_group_id = aws_security_group.web.id   # 引用另一个资源的属性
  source_security_group_id = module.alb.security_group_id  # 引用模块输出
}
```

### 内置函数

```hcl
locals {
  # 字符串函数
  upper_env   = upper(var.environment)           # "PROD"
  name_parts  = split("-", var.resource_name)    # ["my", "app"]
  joined      = join(", ", var.cidrs)            # "10.0.0.0/8, 172.16.0.0/12"
  trimmed     = trimspace("  hello  ")           # "hello"
  replaced    = replace(var.name, "/", "-")      # 替换字符

  # 数值函数
  max_val     = max(5, 12, 9)                    # 12
  min_val     = min(var.min_size, 10)            # 取较小值
  ceiling     = ceil(7.3)                        # 8

  # 集合函数
  flat_list   = flatten([var.public_subnets, var.private_subnets])
  unique_list = distinct(var.regions)
  merged_map  = merge(local.default_tags, var.extra_tags)
  keys_list   = keys(var.tags)
  values_list = values(var.tags)
  sorted      = sort(var.names)
  lookup_val  = lookup(var.instance_map, "web", "t3.micro")  # 带默认值的 map 查找
  contains_it = contains(var.allowed_envs, "prod")

  # 编码函数
  json_policy = jsonencode({
    Version   = "2012-10-17"
    Statement = [{ Effect = "Allow", Action = "*", Resource = "*" }]
  })
  b64_encoded = base64encode("hello world")
  yaml_config = yamlencode({ key = "value", list = [1, 2, 3] })

  # 文件函数
  user_data   = file("${path.module}/scripts/init.sh")
  template    = templatefile("${path.module}/templates/config.tpl", {
    db_host = aws_db_instance.main.address
    db_port = aws_db_instance.main.port
  })

  # 网络函数
  subnet_cidr = cidrsubnet("10.0.0.0/16", 8, 1)   # "10.0.1.0/24"
  host_ip     = cidrhost("10.0.1.0/24", 5)         # "10.0.1.5"

  # 类型转换
  str_to_num  = tonumber("42")
  num_to_str  = tostring(42)
  to_set      = toset(["a", "b", "a"])              # 去重
}
```

### 动态块 (Dynamic Block)

```hcl
# 动态生成安全组规则
resource "aws_security_group" "web" {
  name        = "${var.project_name}-web-sg"
  description = "Security group for web tier"
  vpc_id      = module.vpc.vpc_id

  dynamic "ingress" {
    for_each = var.ingress_rules
    content {
      description = ingress.value.description
      from_port   = ingress.value.port
      to_port     = ingress.value.port
      protocol    = ingress.value.protocol
      cidr_blocks = ingress.value.cidr_blocks
    }
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# 对应的变量定义
variable "ingress_rules" {
  type = list(object({
    description = string
    port        = number
    protocol    = string
    cidr_blocks = list(string)
  }))
  default = [
    { description = "HTTPS", port = 443, protocol = "tcp", cidr_blocks = ["0.0.0.0/0"] },
    { description = "HTTP",  port = 80,  protocol = "tcp", cidr_blocks = ["0.0.0.0/0"] },
  ]
}
```

### for_each vs count

```hcl
# --- count: 基于数量的简单迭代 ---
resource "aws_instance" "worker" {
  count = var.worker_count

  ami           = data.aws_ami.ubuntu.id
  instance_type = "t3.medium"
  subnet_id     = element(module.vpc.private_subnet_ids, count.index)

  tags = {
    Name = "${var.project_name}-worker-${count.index + 1}"
  }
}

# --- for_each: 基于集合的迭代 (推荐) ---
# 使用 map 实现不同配置
variable "instances" {
  type = map(object({
    instance_type = string
    subnet_tier   = string
  }))
  default = {
    web = { instance_type = "t3.small", subnet_tier = "public" }
    api = { instance_type = "t3.medium", subnet_tier = "private" }
    worker = { instance_type = "t3.large", subnet_tier = "private" }
  }
}

resource "aws_instance" "app" {
  for_each = var.instances

  ami           = data.aws_ami.ubuntu.id
  instance_type = each.value.instance_type
  subnet_id     = each.value.subnet_tier == "public" ? module.vpc.public_subnet_ids[0] : module.vpc.private_subnet_ids[0]

  tags = {
    Name = "${var.project_name}-${each.key}"
    Role = each.key
  }
}

# --- for 表达式 ---
locals {
  # 列表推导
  instance_ids = [for inst in aws_instance.app : inst.id]

  # Map 推导
  instance_ip_map = { for k, inst in aws_instance.app : k => inst.private_ip }

  # 带过滤的推导
  public_instances = { for k, v in var.instances : k => v if v.subnet_tier == "public" }

  # 嵌套推导
  sg_rules = flatten([
    for name, config in var.services : [
      for port in config.ports : {
        name = name
        port = port
      }
    ]
  ])
}
```

### 条件表达式

```hcl
# 三元运算符
resource "aws_instance" "app" {
  instance_type = var.environment == "prod" ? "t3.large" : "t3.micro"
}

# 条件创建资源 (count)
resource "aws_cloudwatch_metric_alarm" "high_cpu" {
  count = var.environment == "prod" ? 1 : 0

  alarm_name  = "${var.project_name}-high-cpu"
  namespace   = "AWS/EC2"
  metric_name = "CPUUtilization"
  # ...
}

# 条件创建资源 (for_each)
resource "aws_route53_record" "alias" {
  for_each = var.create_dns ? toset(["main"]) : toset([])

  zone_id = var.zone_id
  name    = var.domain_name
  type    = "A"
  # ...
}

# 条件输出
output "bastion_ip" {
  value = var.enable_bastion ? aws_instance.bastion[0].public_ip : null
}
```

---

## 状态管理

### Remote State

**State 文件包含敏感信息，严禁提交到 Git。** 生产环境必须使用 Remote Backend。

```hcl
# S3 Backend (推荐用于 AWS)
terraform {
  backend "s3" {
    bucket         = "mycompany-terraform-state"
    key            = "prod/vpc/terraform.tfstate"
    region         = "ap-northeast-1"
    encrypt        = true
    dynamodb_table = "terraform-state-lock"  # State Locking
    # 启用版本控制以支持状态回滚
  }
}

# 创建 Backend 基础设施 (Bootstrap, 单独管理)
resource "aws_s3_bucket" "terraform_state" {
  bucket = "mycompany-terraform-state"

  lifecycle {
    prevent_destroy = true
  }
}

resource "aws_s3_bucket_versioning" "terraform_state" {
  bucket = aws_s3_bucket.terraform_state.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "terraform_state" {
  bucket = aws_s3_bucket.terraform_state.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "aws:kms"
    }
  }
}

resource "aws_dynamodb_table" "terraform_lock" {
  name         = "terraform-state-lock"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "LockID"

  attribute {
    name = "LockID"
    type = "S"
  }
}
```

### State Locking

State Locking 防止多人同时操作导致状态损坏。

- **S3 Backend**：DynamoDB 表实现锁
- **GCS Backend**：原生支持锁
- **Terraform Cloud / Enterprise**：内置锁管理
- **强制解锁**（仅在锁残留时使用）：`terraform force-unlock <LOCK_ID>`

### State 迁移

```bash
# 从 local 迁移到 S3
# 1. 添加 backend "s3" 配置
# 2. 执行初始化迁移
terraform init -migrate-state

# 在 backend 之间迁移
terraform init -migrate-state -force-copy

# 导出当前状态
terraform state pull > terraform.tfstate.backup

# 推送状态 (谨慎使用)
terraform state push terraform.tfstate
```

### State 操作

```bash
# 列出所有资源
terraform state list

# 查看资源详情
terraform state show aws_instance.web

# 移动资源 (重命名/移入模块)
terraform state mv aws_instance.web aws_instance.app
terraform state mv aws_instance.web module.compute.aws_instance.web

# 从状态中移除 (不删除实际资源)
terraform state rm aws_instance.legacy

# 导入已有资源到 Terraform 管理
terraform import aws_instance.web i-1234567890abcdef0

# Terraform 1.5+ 声明式 import
import {
  to = aws_instance.web
  id = "i-1234567890abcdef0"
}

# taint: 标记资源需要重建 (下次 apply 时)
terraform taint aws_instance.web

# untaint: 取消 taint 标记
terraform untaint aws_instance.web

# Terraform 1.6+ 推荐使用 -replace 替代 taint
terraform apply -replace="aws_instance.web"
```

---

## 模块设计

### 标准模块结构

```
modules/
└── vpc/
    ├── main.tf          # 核心资源定义
    ├── variables.tf     # 输入变量
    ├── outputs.tf       # 输出值
    ├── versions.tf      # Provider 版本约束
    ├── locals.tf        # 本地变量
    ├── data.tf          # Data Source
    ├── README.md        # 模块文档 (terraform-docs 自动生成)
    └── examples/        # 使用示例
        └── simple/
            └── main.tf
```

### 模块输入输出设计

```hcl
# modules/vpc/variables.tf
variable "vpc_cidr" {
  description = "CIDR block for the VPC"
  type        = string

  validation {
    condition     = can(cidrhost(var.vpc_cidr, 0))
    error_message = "Must be a valid CIDR block."
  }
}

variable "enable_nat_gateway" {
  description = "Whether to create NAT Gateway for private subnets"
  type        = bool
  default     = true
}

variable "single_nat_gateway" {
  description = "Use a single NAT Gateway instead of one per AZ (cost saving for non-prod)"
  type        = bool
  default     = false
}

# modules/vpc/outputs.tf
output "vpc_id" {
  description = "The ID of the VPC"
  value       = aws_vpc.main.id
}

output "private_subnet_ids" {
  description = "List of private subnet IDs"
  value       = aws_subnet.private[*].id
}

output "public_subnet_ids" {
  description = "List of public subnet IDs"
  value       = aws_subnet.public[*].id
}

output "nat_gateway_ips" {
  description = "Elastic IPs of the NAT Gateways"
  value       = aws_eip.nat[*].public_ip
}
```

### 版本约束

```hcl
# 精确版本
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "5.5.3"
}

# 补丁版本范围
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.8"  # >= 20.8.0, < 21.0.0
}

# 版本范围
module "rds" {
  source  = "terraform-aws-modules/rds/aws"
  version = ">= 6.0.0, < 7.0.0"
}

# Git 源 (含版本标签)
module "custom" {
  source = "git::https://github.com/myorg/terraform-modules.git//modules/custom?ref=v2.1.0"
}
```

### 模块组合模式

```hcl
# 根模块组合多个子模块
module "networking" {
  source = "./modules/vpc"

  vpc_cidr           = var.vpc_cidr
  environment        = var.environment
  enable_nat_gateway = var.environment == "prod"
  single_nat_gateway = var.environment != "prod"
}

module "database" {
  source = "./modules/rds"

  vpc_id             = module.networking.vpc_id
  subnet_ids         = module.networking.private_subnet_ids
  instance_class     = local.instance_type
  environment        = var.environment
  multi_az           = var.environment == "prod"
}

module "compute" {
  source = "./modules/ecs"

  vpc_id             = module.networking.vpc_id
  private_subnet_ids = module.networking.private_subnet_ids
  public_subnet_ids  = module.networking.public_subnet_ids
  db_endpoint        = module.database.endpoint
  environment        = var.environment
}
```

---

## 工作流

### 核心命令

```bash
# 初始化: 下载 Provider + 模块 + 初始化 Backend
terraform init
terraform init -upgrade          # 升级 Provider 到约束范围内最新版
terraform init -reconfigure      # 重新配置 Backend (忽略已有配置)

# 格式化
terraform fmt                    # 格式化当前目录
terraform fmt -recursive         # 递归格式化所有子目录
terraform fmt -check             # 仅检查格式 (CI 用)

# 验证
terraform validate               # 语法 + 类型检查 (不连接远端)

# 计划
terraform plan                   # 预览变更
terraform plan -out=tfplan       # 保存计划到文件 (推荐用于 CI/CD)
terraform plan -target=aws_instance.web  # 仅计划特定资源
terraform plan -var="environment=prod"   # 传入变量

# 执行
terraform apply                  # 交互式确认后执行
terraform apply tfplan           # 执行保存的计划 (无需再次确认)
terraform apply -auto-approve    # 跳过确认 (仅用于自动化流水线)
terraform apply -replace="aws_instance.web"  # 强制重建指定资源

# 销毁
terraform destroy                # 销毁所有资源
terraform destroy -target=aws_instance.web   # 仅销毁指定资源

# 输出
terraform output                 # 显示所有输出
terraform output -json           # JSON 格式输出
terraform output database_endpoint  # 查看单个输出

# 依赖图
terraform graph | dot -Tpng > graph.png  # 生成资源依赖图
```

### Workspace

Workspace 用于在同一配置中管理多个环境的状态（轻量级方案）。

```bash
# 创建并切换到新 workspace
terraform workspace new dev
terraform workspace new staging
terraform workspace new prod

# 列出所有 workspace
terraform workspace list

# 切换 workspace
terraform workspace select prod

# 在配置中使用 workspace
# main.tf
locals {
  env = terraform.workspace
}
```

**注意**：对于复杂项目，推荐使用目录分离（`environments/dev/`、`environments/prod/`）而非 workspace，因为不同环境往往有不同的资源组合。

---

## AWS 实战配置

### VPC 完整配置

```hcl
resource "aws_vpc" "main" {
  cidr_block           = var.vpc_cidr
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = merge(local.common_tags, {
    Name = "${var.project_name}-${var.environment}-vpc"
  })
}

# 公有子网 (多 AZ)
resource "aws_subnet" "public" {
  count = length(local.azs)

  vpc_id                  = aws_vpc.main.id
  cidr_block              = local.public_subnets[count.index]
  availability_zone       = local.azs[count.index]
  map_public_ip_on_launch = true

  tags = merge(local.common_tags, {
    Name = "${var.project_name}-public-${local.azs[count.index]}"
    Tier = "public"
    "kubernetes.io/role/elb" = "1"  # EKS ALB 发现标签
  })
}

# 私有子网 (多 AZ)
resource "aws_subnet" "private" {
  count = length(local.azs)

  vpc_id            = aws_vpc.main.id
  cidr_block        = local.private_subnets[count.index]
  availability_zone = local.azs[count.index]

  tags = merge(local.common_tags, {
    Name = "${var.project_name}-private-${local.azs[count.index]}"
    Tier = "private"
    "kubernetes.io/role/internal-elb" = "1"
  })
}

# Internet Gateway
resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id

  tags = merge(local.common_tags, {
    Name = "${var.project_name}-igw"
  })
}

# NAT Gateway (每个 AZ 一个，生产环境高可用)
resource "aws_eip" "nat" {
  count  = var.single_nat_gateway ? 1 : length(local.azs)
  domain = "vpc"

  tags = merge(local.common_tags, {
    Name = "${var.project_name}-nat-eip-${count.index + 1}"
  })
}

resource "aws_nat_gateway" "main" {
  count = var.single_nat_gateway ? 1 : length(local.azs)

  allocation_id = aws_eip.nat[count.index].id
  subnet_id     = aws_subnet.public[count.index].id

  tags = merge(local.common_tags, {
    Name = "${var.project_name}-nat-${count.index + 1}"
  })

  depends_on = [aws_internet_gateway.main]
}

# 路由表
resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }

  tags = merge(local.common_tags, { Name = "${var.project_name}-public-rt" })
}

resource "aws_route_table" "private" {
  count  = var.single_nat_gateway ? 1 : length(local.azs)
  vpc_id = aws_vpc.main.id

  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.main[var.single_nat_gateway ? 0 : count.index].id
  }

  tags = merge(local.common_tags, { Name = "${var.project_name}-private-rt-${count.index + 1}" })
}

resource "aws_route_table_association" "public" {
  count          = length(local.azs)
  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}

resource "aws_route_table_association" "private" {
  count          = length(local.azs)
  subnet_id      = aws_subnet.private[count.index].id
  route_table_id = aws_route_table.private[var.single_nat_gateway ? 0 : count.index].id
}
```

### RDS 配置

```hcl
resource "aws_db_subnet_group" "main" {
  name       = "${var.project_name}-db-subnet"
  subnet_ids = aws_subnet.private[*].id

  tags = local.common_tags
}

resource "aws_db_instance" "main" {
  identifier = "${var.project_name}-${var.environment}"

  engine               = "postgres"
  engine_version       = "16.2"
  instance_class       = var.environment == "prod" ? "db.r6g.large" : "db.t3.micro"
  allocated_storage    = 20
  max_allocated_storage = var.environment == "prod" ? 200 : 50  # Auto-scaling 上限

  db_name  = var.db_name
  username = var.db_username
  password = var.db_password  # 建议通过 Secrets Manager 管理

  db_subnet_group_name   = aws_db_subnet_group.main.name
  vpc_security_group_ids = [aws_security_group.db.id]

  multi_az               = var.environment == "prod"
  storage_encrypted      = true
  deletion_protection    = var.environment == "prod"
  skip_final_snapshot    = var.environment != "prod"
  final_snapshot_identifier = var.environment == "prod" ? "${var.project_name}-final-snapshot" : null

  backup_retention_period = var.environment == "prod" ? 35 : 7
  backup_window           = "03:00-04:00"
  maintenance_window      = "Mon:04:00-Mon:05:00"

  performance_insights_enabled = var.environment == "prod"

  tags = local.common_tags

  lifecycle {
    ignore_changes = [password]  # 密码由外部管理
  }
}
```

### S3 配置

```hcl
resource "aws_s3_bucket" "assets" {
  bucket = "${var.project_name}-${var.environment}-assets"

  tags = local.common_tags
}

resource "aws_s3_bucket_versioning" "assets" {
  bucket = aws_s3_bucket.assets.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "assets" {
  bucket = aws_s3_bucket.assets.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
    bucket_key_enabled = true
  }
}

resource "aws_s3_bucket_public_access_block" "assets" {
  bucket = aws_s3_bucket.assets.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_lifecycle_configuration" "assets" {
  bucket = aws_s3_bucket.assets.id

  rule {
    id     = "transition-to-ia"
    status = "Enabled"

    transition {
      days          = 90
      storage_class = "STANDARD_IA"
    }

    transition {
      days          = 180
      storage_class = "GLACIER"
    }

    noncurrent_version_expiration {
      noncurrent_days = 90
    }
  }
}
```

### IAM 配置

```hcl
# ECS Task Role
resource "aws_iam_role" "ecs_task" {
  name = "${var.project_name}-${var.environment}-ecs-task"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ecs-tasks.amazonaws.com"
        }
      }
    ]
  })

  tags = local.common_tags
}

# 最小权限策略
resource "aws_iam_role_policy" "ecs_task" {
  name = "${var.project_name}-ecs-task-policy"
  role = aws_iam_role.ecs_task.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:ListBucket"
        ]
        Resource = [
          aws_s3_bucket.assets.arn,
          "${aws_s3_bucket.assets.arn}/*"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "secretsmanager:GetSecretValue"
        ]
        Resource = [
          "arn:aws:secretsmanager:${var.aws_region}:${data.aws_caller_identity.current.account_id}:secret:${var.project_name}/*"
        ]
      }
    ]
  })
}
```

### ALB 配置

```hcl
resource "aws_lb" "main" {
  name               = "${var.project_name}-${var.environment}-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets            = aws_subnet.public[*].id

  enable_deletion_protection = var.environment == "prod"

  access_logs {
    bucket  = aws_s3_bucket.alb_logs.id
    prefix  = "alb"
    enabled = true
  }

  tags = local.common_tags
}

resource "aws_lb_listener" "https" {
  load_balancer_arn = aws_lb.main.arn
  port              = 443
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"
  certificate_arn   = var.acm_certificate_arn

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.app.arn
  }
}

resource "aws_lb_listener" "http_redirect" {
  load_balancer_arn = aws_lb.main.arn
  port              = 80
  protocol          = "HTTP"

  default_action {
    type = "redirect"
    redirect {
      port        = "443"
      protocol    = "HTTPS"
      status_code = "HTTP_301"
    }
  }
}

resource "aws_lb_target_group" "app" {
  name        = "${var.project_name}-${var.environment}-tg"
  port        = 8080
  protocol    = "HTTP"
  vpc_id      = aws_vpc.main.id
  target_type = "ip"

  health_check {
    enabled             = true
    healthy_threshold   = 3
    unhealthy_threshold = 3
    timeout             = 5
    interval            = 30
    path                = "/health"
    matcher             = "200"
  }

  deregistration_delay = 60

  tags = local.common_tags
}
```

### EKS 配置

```hcl
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.8"

  cluster_name    = "${var.project_name}-${var.environment}"
  cluster_version = "1.29"

  vpc_id     = module.networking.vpc_id
  subnet_ids = module.networking.private_subnet_ids

  # 集群访问控制
  cluster_endpoint_public_access  = true
  cluster_endpoint_private_access = true
  cluster_endpoint_public_access_cidrs = var.environment == "prod" ? var.allowed_cidrs : ["0.0.0.0/0"]

  # 集群插件
  cluster_addons = {
    coredns = {
      most_recent = true
    }
    kube-proxy = {
      most_recent = true
    }
    vpc-cni = {
      most_recent              = true
      service_account_role_arn = module.vpc_cni_irsa.iam_role_arn
    }
  }

  # 托管节点组
  eks_managed_node_groups = {
    general = {
      instance_types = ["t3.large"]
      capacity_type  = var.environment == "prod" ? "ON_DEMAND" : "SPOT"
      min_size       = var.environment == "prod" ? 3 : 1
      max_size       = var.environment == "prod" ? 10 : 3
      desired_size   = var.environment == "prod" ? 3 : 1

      labels = {
        Environment = var.environment
        NodeGroup   = "general"
      }

      tags = local.common_tags
    }
  }

  tags = local.common_tags
}
```

---

## 安全

### Secrets 管理

```hcl
# 严禁在代码中硬编码密码 -- 使用 AWS Secrets Manager
resource "aws_secretsmanager_secret" "db_password" {
  name                    = "${var.project_name}/${var.environment}/db-password"
  recovery_window_in_days = var.environment == "prod" ? 30 : 0
  tags                    = local.common_tags
}

resource "aws_secretsmanager_secret_version" "db_password" {
  secret_id     = aws_secretsmanager_secret.db_password.id
  secret_string = random_password.db.result
}

resource "random_password" "db" {
  length  = 32
  special = true
  override_special = "!#$%^&*()-_=+"
}

# 在 RDS 中引用
resource "aws_db_instance" "main" {
  # ...
  password = random_password.db.result
  # ...
}
```

### Sensitive 变量

```hcl
variable "db_password" {
  description = "Database master password"
  type        = string
  sensitive   = true  # 不会出现在 plan / apply 输出中
}

output "db_connection_string" {
  value     = "postgresql://${var.db_username}:${var.db_password}@${aws_db_instance.main.endpoint}/${var.db_name}"
  sensitive = true
}
```

### Vault 集成

```hcl
provider "vault" {
  address = var.vault_address
  # 认证通过环境变量 VAULT_TOKEN 或 AppRole
}

data "vault_generic_secret" "db" {
  path = "secret/data/${var.environment}/database"
}

resource "aws_db_instance" "main" {
  # ...
  username = data.vault_generic_secret.db.data["username"]
  password = data.vault_generic_secret.db.data["password"]
}
```

### Policy as Code

#### Sentinel (Terraform Cloud / Enterprise)

```python
# sentinel/enforce-encryption.sentinel
import "tfplan/v2" as tfplan

# 强制 S3 Bucket 开启加密
s3_buckets = filter tfplan.resource_changes as _, rc {
    rc.type is "aws_s3_bucket" and
    rc.mode is "managed" and
    (rc.change.actions contains "create" or rc.change.actions contains "update")
}

encryption_enforced = rule {
    all s3_buckets as _, bucket {
        bucket.change.after.server_side_encryption_configuration is not null
    }
}

main = rule {
    encryption_enforced
}
```

#### OPA (Open Policy Agent)

```rego
# policy/enforce_tags.rego
package terraform.analysis

import input as tfplan

# 要求所有资源必须有 Environment 和 ManagedBy 标签
required_tags := ["Environment", "ManagedBy"]

deny[msg] {
    resource := tfplan.resource_changes[_]
    resource.change.actions[_] == "create"
    tags := resource.change.after.tags
    required_tag := required_tags[_]
    not tags[required_tag]
    msg := sprintf("Resource '%s' is missing required tag '%s'", [resource.address, required_tag])
}

# 禁止使用过大的实例类型
deny[msg] {
    resource := tfplan.resource_changes[_]
    resource.type == "aws_instance"
    resource.change.actions[_] == "create"
    instance_type := resource.change.after.instance_type
    startswith(instance_type, "x1")
    msg := sprintf("Instance type '%s' is not allowed (too expensive). Use t3/m6i family.", [instance_type])
}
```

### RBAC (Terraform Cloud)

```hcl
# 团队权限配置
resource "tfe_team" "developers" {
  name         = "developers"
  organization = var.tfc_organization
}

resource "tfe_team_access" "developers_dev" {
  access       = "write"       # plan + apply
  team_id      = tfe_team.developers.id
  workspace_id = tfe_workspace.dev.id
}

resource "tfe_team_access" "developers_prod" {
  access       = "plan"        # 仅 plan，不能 apply
  team_id      = tfe_team.developers.id
  workspace_id = tfe_workspace.prod.id
}
```

---

## CI/CD 集成

### GitHub Actions

```yaml
# .github/workflows/terraform.yml
name: Terraform

on:
  pull_request:
    paths:
      - 'infrastructure/**'
  push:
    branches: [main]
    paths:
      - 'infrastructure/**'

permissions:
  id-token: write   # OIDC
  contents: read
  pull-requests: write

jobs:
  plan:
    name: Terraform Plan
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    defaults:
      run:
        working-directory: infrastructure/

    steps:
      - uses: actions/checkout@v4

      - uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: ${{ secrets.AWS_ROLE_ARN }}
          aws-region: ap-northeast-1

      - uses: hashicorp/setup-terraform@v3
        with:
          terraform_version: 1.7.4

      - name: Terraform Init
        run: terraform init -no-color

      - name: Terraform Format Check
        run: terraform fmt -check -recursive

      - name: Terraform Validate
        run: terraform validate -no-color

      - name: Terraform Plan
        id: plan
        run: terraform plan -no-color -out=tfplan
        continue-on-error: true

      - name: Comment PR
        uses: actions/github-script@v7
        with:
          script: |
            const output = `#### Terraform Plan 📋
            \`\`\`
            ${{ steps.plan.outputs.stdout }}
            \`\`\`
            *Pushed by: @${{ github.actor }}*`;

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: output
            });

      - name: Plan Status
        if: steps.plan.outcome == 'failure'
        run: exit 1

  apply:
    name: Terraform Apply
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    environment: production
    defaults:
      run:
        working-directory: infrastructure/

    steps:
      - uses: actions/checkout@v4

      - uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: ${{ secrets.AWS_ROLE_ARN }}
          aws-region: ap-northeast-1

      - uses: hashicorp/setup-terraform@v3
        with:
          terraform_version: 1.7.4

      - name: Terraform Init
        run: terraform init -no-color

      - name: Terraform Apply
        run: terraform apply -auto-approve -no-color
```

### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - validate
  - plan
  - apply

variables:
  TF_ROOT: infrastructure/
  TF_STATE_NAME: default

.terraform_base:
  image: hashicorp/terraform:1.7.4
  before_script:
    - cd ${TF_ROOT}
    - terraform init -no-color

validate:
  extends: .terraform_base
  stage: validate
  script:
    - terraform fmt -check -recursive
    - terraform validate -no-color
  rules:
    - changes:
        - infrastructure/**

plan:
  extends: .terraform_base
  stage: plan
  script:
    - terraform plan -no-color -out=tfplan
  artifacts:
    paths:
      - ${TF_ROOT}/tfplan
    expire_in: 1 week
  rules:
    - if: '$CI_PIPELINE_SOURCE == "merge_request_event"'
      changes:
        - infrastructure/**

apply:
  extends: .terraform_base
  stage: apply
  script:
    - terraform apply -auto-approve -no-color tfplan
  dependencies:
    - plan
  rules:
    - if: '$CI_COMMIT_BRANCH == "main"'
      changes:
        - infrastructure/**
  when: manual
  environment:
    name: production
```

### Atlantis

```yaml
# atlantis.yaml
version: 3
projects:
  - name: vpc
    dir: infrastructure/vpc
    workspace: default
    terraform_version: v1.7.4
    autoplan:
      when_modified:
        - "*.tf"
        - "../modules/vpc/**"
      enabled: true
    apply_requirements:
      - approved    # PR 需要审批
      - mergeable   # PR 可合并

  - name: app
    dir: infrastructure/app
    workspace: default
    terraform_version: v1.7.4
    autoplan:
      when_modified:
        - "*.tf"
        - "../modules/ecs/**"
      enabled: true
    apply_requirements:
      - approved
      - mergeable
```

### Terraform Cloud

```hcl
# 配置 Terraform Cloud 作为 Backend
terraform {
  cloud {
    organization = "mycompany"

    workspaces {
      tags = ["app:myproject", "env:prod"]
    }
  }
}
```

---

## 团队协作

### 推荐目录结构

```
infrastructure/
├── environments/
│   ├── dev/
│   │   ├── main.tf           # 调用模块，dev 环境参数
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   ├── terraform.tfvars  # dev 环境变量值
│   │   └── backend.tf        # dev 环境 state 配置
│   ├── staging/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   ├── terraform.tfvars
│   │   └── backend.tf
│   └── prod/
│       ├── main.tf
│       ├── variables.tf
│       ├── outputs.tf
│       ├── terraform.tfvars
│       └── backend.tf
├── modules/                   # 可复用模块
│   ├── vpc/
│   ├── ecs/
│   ├── rds/
│   ├── s3/
│   └── iam/
├── policies/                  # OPA / Sentinel 策略
│   ├── enforce_tags.rego
│   └── enforce_encryption.rego
└── scripts/
    ├── bootstrap.sh           # 初始化 S3 Backend + DynamoDB Lock
    └── import.sh              # 批量导入已有资源
```

### 命名规范

```hcl
# 资源命名: <project>-<environment>-<component>-<qualifier>
# 例: myapp-prod-web-sg, myapp-dev-db-primary

# Terraform 资源名 (代码内标识符): 使用下划线，语义清晰
resource "aws_security_group" "web_ingress" { }       # 好
resource "aws_security_group" "sg1" { }                # 差：无语义
resource "aws_security_group" "web-ingress-sg" { }     # 差：HCL 用下划线

# 变量名: 使用下划线，名词短语
variable "vpc_cidr" { }
variable "enable_nat_gateway" { }     # bool 用 enable_ 前缀
variable "instance_count" { }         # 数量用 _count 后缀

# 输出名: 使用下划线，描述所输出的值
output "vpc_id" { }
output "private_subnet_ids" { }
output "database_endpoint" { }

# 模块名: 使用下划线
module "networking" { }
module "application_cluster" { }
```

### 代码审查要点

1. **Plan 输出审查**：每个 PR 必须附带 `terraform plan` 输出
2. **破坏性变更识别**：关注 `destroy` 和 `replace` 操作，尤其是数据库和持久化存储
3. **安全审查**：检查安全组规则、IAM 策略是否遵循最小权限
4. **标签完整性**：所有资源必须包含 Environment、ManagedBy、Project 标签
5. **硬编码检查**：不应出现 IP 地址、密码、Account ID 等硬编码值
6. **State 影响**：注意资源地址变更（重命名/移入模块）是否会导致重建
7. **Provider 版本**：版本约束是否合理，是否会引入破坏性更新

### 变更管理

```bash
# 安全的变更流程
# 1. 创建特性分支
git checkout -b feat/add-redis-cluster

# 2. 修改配置
# 3. 本地验证
terraform fmt -check -recursive
terraform validate
terraform plan -out=tfplan

# 4. 审查 plan 输出
# 5. 提交 PR，附带 plan 输出
# 6. 代码审查 + plan 审查
# 7. 合并到 main，自动触发 apply (或手动 approve)

# 紧急变更回滚
# 方式 1: git revert + apply
git revert HEAD
terraform apply

# 方式 2: 恢复 State 到之前版本 (S3 版本控制)
aws s3api list-object-versions --bucket mycompany-terraform-state --prefix prod/app/terraform.tfstate
aws s3api get-object --bucket mycompany-terraform-state --key prod/app/terraform.tfstate --version-id <version-id> terraform.tfstate.restore
terraform state push terraform.tfstate.restore
```

---

## 性能优化

### 并行度控制

```bash
# 默认并行度为 10，可根据 API 限制调整
terraform apply -parallelism=20     # 提高并行度 (资源量大时)
terraform apply -parallelism=3      # 降低并行度 (API 限流时)
```

### Targeted Apply

```bash
# 仅操作特定资源 (调试/紧急修复时使用)
terraform plan -target=module.networking
terraform apply -target=aws_instance.web

# 注意: targeted apply 会跳过依赖检查，仅用于临时操作
# 完成后应执行完整的 terraform plan 确认状态一致
```

### 模块拆分策略

```
# 按生命周期拆分 State，减少 plan/apply 范围

infrastructure/
├── networking/          # 变更频率低 - VPC/Subnet/NAT
│   └── terraform.tfstate
├── data-stores/         # 变更频率低 - RDS/ElastiCache/S3
│   └── terraform.tfstate
├── compute/             # 变更频率高 - ECS/EC2/ASG
│   └── terraform.tfstate
├── monitoring/          # 变更频率中 - CloudWatch/Alarms
│   └── terraform.tfstate
└── dns/                 # 变更频率低 - Route53/ACM
    └── terraform.tfstate
```

跨 State 引用使用 `terraform_remote_state` Data Source：

```hcl
# compute/main.tf - 引用 networking 的输出
data "terraform_remote_state" "networking" {
  backend = "s3"
  config = {
    bucket = "mycompany-terraform-state"
    key    = "prod/networking/terraform.tfstate"
    region = "ap-northeast-1"
  }
}

resource "aws_instance" "web" {
  subnet_id = data.terraform_remote_state.networking.outputs.private_subnet_ids[0]
}
```

### 大规模部署优化

```bash
# 使用 -refresh=false 跳过 refresh (状态与实际一致时)
terraform plan -refresh=false

# 生成 plan 文件后使用 terraform show 审查
terraform plan -out=tfplan
terraform show -json tfplan | jq '.resource_changes | length'

# Provider 镜像 (加速 init)
# ~/.terraformrc
provider_installation {
  filesystem_mirror {
    path    = "/usr/share/terraform/providers"
    include = ["registry.terraform.io/hashicorp/*"]
  }
  direct {
    exclude = ["registry.terraform.io/hashicorp/*"]
  }
}
```

---

## 常见陷阱与反模式

### 1. State 漂移

**问题**：手动在控制台修改了资源，Terraform State 与实际状态不一致。

```bash
# 检测漂移
terraform plan -refresh-only

# 同步 State 到实际状态 (不修改资源)
terraform apply -refresh-only

# 预防: 禁止手动操作，所有变更通过 Terraform
# 预防: 定期运行 drift detection (Terraform Cloud 内置支持)
```

### 2. 循环依赖

**问题**：资源 A 依赖 B 的属性，B 又依赖 A 的属性。

```hcl
# 错误: 安全组循环引用
resource "aws_security_group" "a" {
  ingress {
    security_groups = [aws_security_group.b.id]  # A 依赖 B
  }
}

resource "aws_security_group" "b" {
  ingress {
    security_groups = [aws_security_group.a.id]  # B 依赖 A → 循环!
  }
}

# 正确: 使用独立的 rule 资源打破循环
resource "aws_security_group" "a" {
  name   = "sg-a"
  vpc_id = aws_vpc.main.id
}

resource "aws_security_group" "b" {
  name   = "sg-b"
  vpc_id = aws_vpc.main.id
}

resource "aws_security_group_rule" "a_from_b" {
  type                     = "ingress"
  security_group_id        = aws_security_group.a.id
  source_security_group_id = aws_security_group.b.id
  from_port                = 443
  to_port                  = 443
  protocol                 = "tcp"
}

resource "aws_security_group_rule" "b_from_a" {
  type                     = "ingress"
  security_group_id        = aws_security_group.b.id
  source_security_group_id = aws_security_group.a.id
  from_port                = 8080
  to_port                  = 8080
  protocol                 = "tcp"
}
```

### 3. Provider 版本锁定

**问题**：未锁定版本导致团队成员使用不同版本，产生不一致的 plan。

```hcl
# 错误: 无版本约束
terraform {
  required_providers {
    aws = {
      source = "hashicorp/aws"
    }
  }
}

# 正确: 明确版本约束 + 提交 .terraform.lock.hcl
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.40"
    }
  }
}

# .terraform.lock.hcl 必须提交到 Git
# 更新锁文件: terraform init -upgrade
```

### 4. count vs for_each 陷阱

```hcl
# 错误: 使用 count + 列表，删除中间元素导致后续资源全部重建
variable "subnets" {
  default = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
}

resource "aws_subnet" "bad" {
  count      = length(var.subnets)
  cidr_block = var.subnets[count.index]  # 删除第 2 个 → 第 3 个变为 index 1 → 重建!
}

# 正确: 使用 for_each + map/set，键稳定不受顺序影响
variable "subnets" {
  default = {
    az-a = "10.0.1.0/24"
    az-b = "10.0.2.0/24"
    az-c = "10.0.3.0/24"
  }
}

resource "aws_subnet" "good" {
  for_each   = var.subnets
  cidr_block = each.value  # 删除 az-b → 仅影响 az-b，az-a 和 az-c 不变
}
```

### 5. 过大的 State 文件

**问题**：所有资源放在一个 State 中，plan/apply 极慢，爆炸半径大。

**解法**：按生命周期和变更频率拆分模块（见性能优化章节）。每个 State 管理 50-200 个资源为宜。

### 6. 硬编码 Provider 配置

```hcl
# 错误: 硬编码 Region 和 Account ID
provider "aws" {
  region = "ap-northeast-1"
}

resource "aws_iam_role" "bad" {
  assume_role_policy = jsonencode({
    Statement = [{
      Principal = { AWS = "arn:aws:iam::123456789012:root" }  # 硬编码
    }]
  })
}

# 正确: 参数化
provider "aws" {
  region = var.aws_region
}

data "aws_caller_identity" "current" {}

resource "aws_iam_role" "good" {
  assume_role_policy = jsonencode({
    Statement = [{
      Principal = { AWS = "arn:aws:iam::${data.aws_caller_identity.current.account_id}:root" }
    }]
  })
}
```

### 7. 忽略 lifecycle 规则

```hcl
# 数据库等有状态资源必须设置 prevent_destroy
resource "aws_db_instance" "main" {
  # ...
  lifecycle {
    prevent_destroy = true  # 防止意外 destroy
  }
}

# 蓝绿部署场景使用 create_before_destroy
resource "aws_instance" "web" {
  # ...
  lifecycle {
    create_before_destroy = true  # 先建新实例再删旧实例
  }
}
```

### 8. 敏感数据泄漏

```hcl
# 错误: 在 terraform.tfvars 中存储密码并提交到 Git
# terraform.tfvars
db_password = "my-secret-password"  # 绝不要这样做

# 正确: 使用环境变量
# export TF_VAR_db_password="my-secret-password"

# 正确: 使用 Secrets Manager / Vault
data "aws_secretsmanager_secret_version" "db" {
  secret_id = aws_secretsmanager_secret.db.id
}

# .gitignore 必须包含:
# *.tfstate
# *.tfstate.*
# *.tfvars      (如果包含敏感值)
# .terraform/
```

### 9. 未使用 moved 块处理重构

```hcl
# Terraform 1.1+ 使用 moved 块安全重命名资源
# 避免 destroy + create，保留已有资源
moved {
  from = aws_instance.web
  to   = aws_instance.application
}

moved {
  from = aws_instance.app
  to   = module.compute.aws_instance.app
}
```

### 10. 忽视 Plan 输出

**反模式**：直接 `terraform apply -auto-approve` 而不审查 plan。

**规范**：
- 开发环境：本地 `terraform plan` 审查后再 apply
- 生产环境：CI 生成 plan → PR 审查 plan 输出 → 人工批准后 apply saved plan
- 永远使用 `terraform plan -out=tfplan` + `terraform apply tfplan`，确保执行的是审查过的计划

---

## Agent Checklist

以下检查清单供 AI Agent 在生成或审查 Terraform 代码时使用：

### 基础规范
- [ ] `terraform fmt` 格式化通过
- [ ] `terraform validate` 验证通过
- [ ] 所有 Provider 声明了版本约束
- [ ] `.terraform.lock.hcl` 提交到 Git
- [ ] `.gitignore` 包含 `*.tfstate`、`.terraform/`、敏感 `.tfvars`

### 安全
- [ ] 无硬编码密码、密钥、Account ID
- [ ] 敏感变量标记 `sensitive = true`
- [ ] 密码通过 Secrets Manager / Vault 管理
- [ ] S3 Bucket 开启加密 + 阻止公开访问
- [ ] IAM 策略遵循最小权限原则
- [ ] 安全组规则无不必要的 `0.0.0.0/0` 入站
- [ ] RDS 开启 `storage_encrypted = true`

### 状态管理
- [ ] 使用 Remote Backend (S3/GCS/Terraform Cloud)
- [ ] 启用 State Locking (DynamoDB/原生)
- [ ] State Bucket 开启版本控制
- [ ] State 按生命周期合理拆分

### 模块设计
- [ ] 模块有完整的 `variables.tf` + `outputs.tf`
- [ ] 变量有 `description` 和 `type`
- [ ] 关键变量有 `validation` 块
- [ ] 使用 `for_each` 而非 `count`（除非纯数量迭代）
- [ ] 模块版本使用 `~>` 约束

### 生产就绪
- [ ] 数据库等有状态资源设置 `prevent_destroy`
- [ ] 所有资源有 Environment / ManagedBy / Project 标签
- [ ] 生产环境 RDS 开启 `multi_az` 和足够的 `backup_retention_period`
- [ ] ALB 使用 TLS 1.3 策略
- [ ] HTTP 自动重定向到 HTTPS
- [ ] EKS 节点组设置合理的 `min_size` / `max_size`
- [ ] NAT Gateway 生产环境每 AZ 一个

### CI/CD
- [ ] PR 自动运行 `terraform plan`
- [ ] Plan 输出作为 PR 评论
- [ ] 生产 Apply 需要人工批准
- [ ] 使用 OIDC 而非长期 Access Key 进行认证
- [ ] Plan 文件保存为 CI Artifact

### 变更安全
- [ ] 审查 plan 中的 `destroy` / `replace` 操作
- [ ] 资源重命名使用 `moved` 块
- [ ] 未使用 `-auto-approve`（除自动化流水线外）
- [ ] 大规模变更分批执行
