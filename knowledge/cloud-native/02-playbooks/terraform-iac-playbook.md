---
id: terraform-iac-playbook
title: Terraform IaC 生产实战手册
domain: cloud-native
category: 02-playbooks
difficulty: advanced
tags: [terraform, iac, infrastructure-as-code, modules, state, remote-state, aws, azure, gcp, production, enterprise]
quality_score: 94
maintainer: devops-team@superdev.com
last_updated: 2026-06-15
---

# Terraform IaC 生产实战手册

> 基于 [Spacelift 21 Best Practices](https://spacelift.io/blog/terraform-best-practices) + [Terrateam 2025 Practices](https://terrateam.io/blog/terraform-best-practices) + [Dev.to State Management Deep Dive](https://dev.to/zopdev/the-terraform-state-management-challenge-a-deep-dive-into-its-pitfalls-and-solutions-2025)

## State 管理（最关键）

### 远程 State（必须！）
```hcl
# ❌ 本地 state（团队协作灾难：覆盖、丢失、冲突）
terraform apply  # 写 terraform.tfstate 到本地

# ✅ 远程 state（S3 + DynamoDB 锁）
terraform {
  backend "s3" {
    bucket         = "my-tf-state"
    key            = "prod/terraform.tfstate"
    region         = "us-east-1"
    dynamodb_table = "tf-locks"    # 防并发写入
    encrypt        = true           # 加密 state（含密钥！）
  }
}
```

### State 隔离
```hcl
# ❌ 所有环境共用一个 state（改 prod 意外影响 dev）
# prod/terraform.tfstate 包含 dev + staging + prod 资源

# ✅ 按环境隔离 state
# dev/terraform.tfstate
# staging/terraform.tfstate
# prod/terraform.tfstate
# 用 Terragrunt 或 workspaces 管理
```

### State 安全
- [ ] State 文件加密（`encrypt = true`）
- [ ] State 不入 Git（`.gitignore` 加 `*.tfstate`）
- [ ] S3 bucket 版本控制（误删可恢复）
- [ ] DynamoDB 锁（防并发 `apply`）
- [ ] IAM 限制访问（只有 CI 能写 prod state）

## Module 最佳实践

```hcl
# ✅ 模块化（可复用 + 可测试）
module "web_app" {
  source = "./modules/web-app"  # 本地模块
  # 或 source = "terraform-aws-modules/ec2-instance/aws"  # 社区模块

  name          = "prod-app"
  instance_type = "t3.medium"
  min_size      = 2
  max_size      = 10
  environment   = "prod"
}

# modules/web-app/main.tf — 模块内只声明资源
resource "aws_launch_template" "app" {
  name = var.name
  image_id = data.aws_ami.app.id
  instance_type = var.instance_type
  user_data = base64encode(templatefile("${path.module}/userdata.sh", {
    environment = var.environment
  }))
}
```

### 模块原则
- **单一职责**：一个模块管一类资源（VPC / DB / App）
- **版本化**：`source = "git::https://...?ref=v1.2.0"`
- **变量有默认值** + **输出明确**
- **不用 `count` 控制资源有无**（用条件表达式）
- **`terraform validate` + `fmt` 在 CI 强制**

## 变量与环境管理

```hcl
# ✅ 每个环境有独立的 tfvars
# environments/dev.tfvars
instance_type = "t3.small"
min_size      = 1

# environments/prod.tfvars
instance_type = "t3.large"
min_size      = 3

# 部署
terraform plan -var-file="environments/prod.tfvars"
```

### 密钥处理
```hcl
# ❌ 密钥写在 tfvars（会进 state 文件！明文！）
db_password = "super_secret_123"

# ✅ 密钥从 Secrets Manager / SSM 读取（不进 state）
data "aws_secretsmanager_secret_version" "db" {
  secret_id = "prod/db-password"
}
resource "aws_db_instance" "main" {
  password = jsondecode(data.aws_secretsmanager_secret_version.db.secret_string)["password"]
}
```

## CI/CD 集成

```yaml
# .github/workflows/terraform.yml
- name: Terraform Format
  run: terraform fmt -check
- name: Terraform Validate
  run: terraform validate
- name: Terraform Plan
  run: terraform plan -var-file="prod.tfvars"
  # PR 上显示 plan 结果（人类审查）
- name: Terraform Apply (only on merge to main)
  if: github.ref == 'refs/heads/main'
  run: terraform apply -auto-approve -var-file="prod.tfvars"
```

## 生产检查清单
- [ ] 远程 state（S3 + DynamoDB lock）
- [ ] State 加密 + 不入 Git
- [ ] 按环境隔离 state
- [ ] 密钥从 Secrets Manager 读取（不进 tfvars/state）
- [ ] 模块化（自定义模块或社区模块）
- [ ] CI 强制 `fmt` + `validate` + `plan` 审查
- [ ] `terraform import` 导入已有资源（不用手动重建）
- [ ] `terraform state rm` 清理残留
- [ ] 定期 `terraform plan` 检查 drift
