---
title: 多云治理作战手册
version: 1.0.0
last_updated: 2025-03-20
owner: platform-team
tags: [multicloud, governance, aws, azure, gcp]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：多云环境治理作战手册
# 作用：指导多云架构的统一管理、成本优化和风险控制
# 创建时间：2025-03-20
# 最后修改：2025-03-20

## 目标

建立多云治理标准化流程，确保：
- 统一身份和访问管理
- 集中式成本管控
- 一致的安全策略
- 灾备和故障转移能力

## 适用场景

- 多云架构部署
- 云间迁移和灾备
- 厂商锁定规避
- 合规性要求

## 执行清单

### 治理规划

- [ ] 定义云策略（主力云、备份云）
- [ ] 设计统一身份认证
- [ ] 规划成本分摊机制
- [ ] 制定安全基线
- [ ] 设计监控和告警体系

### 环境配置

- [ ] 配置多云访问凭证
- [ ] 部署统一管理平台
- [ ] 配置网络互联
- [ ] 设置 DNS 和域名
- [ ] 配置证书管理

### 持续运维

- [ ] 定期成本审查
- [ ] 安全审计
- [ ] 性能监控
- [ ] 合规性检查
- [ ] 灾备演练

## 核心配置

### 1. 多云身份管理

```yaml
# 使用 OIDC 联合身份
# AWS IAM OIDC Provider
apiVersion: iam.aws.crossplane.io/v1beta1
kind: OIDCProvider
metadata:
  name: corporate-idp
spec:
  forProvider:
    region: us-east-1
    clientIDList:
    - sts.amazonaws.com
    thumbprintList:
    - 9e99a48a9960b14926bb7f3b02e22da2b0ab7280
    url: https://oidc.example.com

---
# Azure Service Principal
apiVersion: azure.microsoft.com/v1beta1
kind: ProviderConfig
metadata:
  name: azure-provider
spec:
  credentials:
    source: Secret
    secretRef:
      namespace: crossplane-system
      name: azure-credentials
      key: credentials

---
# GCP Workload Identity
apiVersion: cloudplatform.gcp.crossplane.io/v1beta1
kind: ProviderConfig
metadata:
  name: gcp-provider
spec:
  projectID: my-project
  credentials:
    source: Secret
    secretRef:
      namespace: crossplane-system
      name: gcp-credentials
      key: credentials
```

### 2. 统一资源管理（Crossplane）

```yaml
# AWS S3 Bucket
apiVersion: s3.aws.crossplane.io/v1beta1
kind: Bucket
metadata:
  name: data-bucket-aws
  labels:
    cloud: aws
    environment: production
spec:
  forProvider:
    region: us-east-1
    acl: private
    versioningConfiguration:
      status: Enabled
    serverSideEncryptionConfiguration:
      rules:
      - applyServerSideEncryptionByDefault:
          sseAlgorithm: AES256
    publicAccessBlockConfiguration:
      blockPublicAcls: true
      blockPublicPolicy: true
      ignorePublicAcls: true
      restrictPublicBuckets: true
  providerConfigRef:
    name: aws-provider

---
# Azure Storage Account
apiVersion: storage.azure.microsoft.com/v1beta1
kind: StorageAccount
metadata:
  name: datastorageazure
  labels:
    cloud: azure
    environment: production
spec:
  forProvider:
    resourceGroupName: production-rg
    location: eastus
    sku:
      name: Standard_GRS
    kind: StorageV2
    accessTier: Hot
    enableHttpsTrafficOnly: true
    minimumTlsVersion: TLS1_2
    networkRule:
      defaultAction: Deny
      bypass: AzureServices
  providerConfigRef:
    name: azure-provider

---
# GCP Cloud Storage Bucket
apiVersion: storage.gcp.crossplane.io/v1beta1
kind: Bucket
metadata:
  name: data-bucket-gcp
  labels:
    cloud: gcp
    environment: production
spec:
  forProvider:
    location: US
    storageClass: STANDARD
    versioning:
      enabled: true
    uniformBucketLevelAccess:
      enabled: true
    encryption:
      defaultKmsKeyName: projects/my-project/locations/us/keyRings/my-keyring/cryptoKeys/my-key
  providerConfigRef:
    name: gcp-provider
```

### 3. 多云 Kubernetes 集群管理

```yaml
# AWS EKS 集群
apiVersion: eks.aws.crossplane.io/v1beta1
kind: Cluster
metadata:
  name: production-eks
  labels:
    cloud: aws
    environment: production
spec:
  forProvider:
    region: us-east-1
    roleArn: arn:aws:iam::123456789:role/eks-cluster-role
    version: "1.28"
    vpcConfig:
      subnetIds:
      - subnet-abc123
      - subnet-def456
      securityGroupIds:
      - sg-abc123
    encryptionConfig:
    - provider:
        keyArn: arn:aws:kms:us-east-1:123456789:key/abc123
      resources:
      - secrets
    logging:
      clusterLogging:
      - enabled: true
        types:
        - api
        - audit
        - authenticator
        - controllerManager
        - scheduler
  providerConfigRef:
    name: aws-provider

---
# Azure AKS 集群
apiVersion: containerservice.azure.microsoft.com/v1beta1
kind: ManagedCluster
metadata:
  name: production-aks
  labels:
    cloud: azure
    environment: production
spec:
  forProvider:
    resourceGroupName: production-rg
    location: eastus
    dnsPrefix: production-aks
    agentPoolProfiles:
    - name: nodepool1
      count: 3
      vmSize: Standard_D2s_v3
      osDiskSizeGB: 100
      osType: Linux
      mode: System
    identity:
      type: SystemAssigned
    networkProfile:
      networkPlugin: azure
      networkPolicy: azure
      loadBalancerSku: standard
  providerConfigRef:
    name: azure-provider

---
# GCP GKE 集群
apiVersion: container.gcp.crossplane.io/v1beta2
kind: Cluster
metadata:
  name: production-gke
  labels:
    cloud: gcp
    environment: production
spec:
  forProvider:
    location: us-central1
    initialNodeCount: 3
    network: projects/my-project/global/networks/default
    subnetwork: projects/my-project/regions/us-central1/subnetworks/default
    enableBinaryAuthorization: true
    enableIntranodeVisibility: true
    masterAuth:
      clientCertificateConfig:
        issueClientCertificate: false
    ipAllocationPolicy:
      useIpAliases: true
    privateClusterConfig:
      enablePrivateEndpoint: false
      enablePrivateNodes: true
      masterIpv4CidrBlock: 172.16.0.0/28
  providerConfigRef:
    name: gcp-provider
```

### 4. 多云网络互联

```yaml
# AWS VPN 连接
apiVersion: ec2.aws.crossplane.io/v1beta1
kind: VPNConnection
metadata:
  name: aws-to-azure-vpn
spec:
  forProvider:
    region: us-east-1
    customerGatewayId: cgw-abc123
    vpnGatewayId: vgw-abc123
    type: ipsec.1
    options:
      staticRoutesOnly: false
  providerConfigRef:
    name: aws-provider

---
# Azure Virtual Network Gateway
apiVersion: network.azure.microsoft.com/v1beta1
kind: VirtualNetworkGateway
metadata:
  name: azure-vpn-gateway
spec:
  forProvider:
    resourceGroupName: production-rg
    location: eastus
    gatewayType: Vpn
    vpnType: RouteBased
    sku:
      name: VpnGw1
      tier: VpnGw1
    vpnClientConfiguration:
      vpnClientProtocols:
      - IkeV2
  providerConfigRef:
    name: azure-provider
```

### 5. 成本管理

```yaml
# 成本分配标签策略
apiVersion: aws.crossplane.io/v1beta1
kind: ProviderConfig
metadata:
  name: aws-provider
spec:
  tags:
    Environment: production
    CostCenter: "12345"
    Owner: platform-team
    Project: core-platform

---
# Kubecost 多云成本监控
apiVersion: v1
kind: ConfigMap
metadata:
  name: kubecost-config
  namespace: kubecost
data:
  cloud-integration.json: |
    {
      "aws": {
        "serviceKeyName": "AWS_ACCESS_KEY_ID",
        "serviceKeySecret": "AWS_SECRET_ACCESS_KEY",
        "spotDataRegion": "us-east-1"
      },
      "azure": {
        "subscriptionId": "xxx-xxx-xxx",
        "clientId": "xxx-xxx-xxx",
        "clientSecret": "xxx-xxx-xxx",
        "tenantId": "xxx-xxx-xxx"
      },
      "gcp": {
        "projectId": "my-project",
        "billingDataDataset": "billing_data"
      }
    }
```

### 6. 统一监控

```yaml
# Prometheus 联邦配置
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-federation
  namespace: monitoring
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
    scrape_configs:
    - job_name: 'federate-aws'
      scrape_interval: 15s
      honor_labels: true
      metrics_path: '/federate'
      params:
        'match[]':
        - '{job="kubernetes-pods"}'
        - '{job="kubernetes-services"}'
      static_configs:
      - targets:
        - 'prometheus-aws.monitoring.svc.cluster.local:9090'
        labels:
          cloud: aws

    - job_name: 'federate-azure'
      scrape_interval: 15s
      honor_labels: true
      metrics_path: '/federate'
      params:
        'match[]':
        - '{job="kubernetes-pods"}'
        - '{job="kubernetes-services"}'
      static_configs:
      - targets:
        - 'prometheus-azure.monitoring.svc.cluster.local:9090'
        labels:
          cloud: azure

    - job_name: 'federate-gcp'
      scrape_interval: 15s
      honor_labels: true
      metrics_path: '/federate'
      params:
        'match[]':
        - '{job="kubernetes-pods"}'
        - '{job="kubernetes-services"}'
      static_configs:
      - targets:
        - 'prometheus-gcp.monitoring.svc.cluster.local:9090'
        labels:
          cloud: gcp
```

### 7. 灾备配置

```yaml
# Velero 多云备份
apiVersion: velero.io/v1
kind: BackupStorageLocation
metadata:
  name: aws-backup
  namespace: velero
spec:
  provider: aws
  objectStorage:
    bucket: k8s-backups-aws
  config:
    region: us-east-1
---
apiVersion: velero.io/v1
kind: BackupStorageLocation
metadata:
  name: azure-backup
  namespace: velero
spec:
  provider: azure
  objectStorage:
    bucket: k8s-backups-azure
  config:
    resourceGroup: backup-rg
    storageAccount: backupstorage

---
# 定期备份计划
apiVersion: velero.io/v1
kind: Schedule
metadata:
  name: daily-backup
  namespace: velero
spec:
  schedule: "0 2 * * *"
  template:
    includedNamespaces:
    - production
    - staging
    storageLocation: aws-backup
    ttl: 720h  # 30 天
    snapshotVolumes: true
```

## 最佳实践

### 1. 云选择策略

```yaml
# 工作负载放置策略
workloadPlacement:
  # 主力云 - 常规工作负载
  primary:
    cloud: aws
    workloads:
    - web-services
    - api-gateway
    - general-compute

  # 备份云 - 灾备和特殊需求
  secondary:
    cloud: azure
    workloads:
    - disaster-recovery
    - windows-workloads
    - office-integration

  # 专业云 - 特定服务
  specialized:
    cloud: gcp
    workloads:
    - ml-training
    - big-data-processing
    - kubernetes-native
```

### 2. 成本优化策略

```yaml
# 资源标签标准
tags:
  required:
  - Environment
  - CostCenter
  - Owner
  - Project
  optional:
  - Customer
  - Application
  - Version

---
# 成本告警规则
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: cost-alerts
  namespace: monitoring
spec:
  groups:
  - name: cost.rules
    rules:
    - alert: HighCloudSpend
      expr: |
        sum(cloud_cost_usd) by (cloud) > 10000
      for: 1h
      labels:
        severity: warning
      annotations:
        summary: "High cloud spend detected"
        description: "{{ $labels.cloud }} spend is ${{ $value }}"
```

### 3. 安全策略统一

```yaml
# OPA Gatekeeper 多云策略
apiVersion: templates.gatekeeper.sh/v1
kind: ConstraintTemplate
metadata:
  name: k8sallowedrepos
spec:
  crd:
    spec:
      names:
        kind: K8sAllowedRepos
      validation:
        openAPIV3Schema:
          type: object
          properties:
            repos:
              type: array
              items:
                type: string
  targets:
    - target: admission.k8s.gatekeeper.sh
      rego: |
        package k8sallowedrepos
        violation[{"msg": msg}] {
          container := input.review.object.spec.containers[_]
          satisfied := [good | repo = input.parameters.repos[_]; good = startswith(container.image, repo)]
          not any(satisfied)
          msg := sprintf("container %v has an invalid image repo %v, allowed repos are %v", [container.name, container.image, input.parameters.repos])
        }
---
apiVersion: constraints.gatekeeper.sh/v1beta1
kind: K8sAllowedRepos
metadata:
  name: allowed-repos
spec:
  match:
    kinds:
    - apiGroups: [""]
      kinds: ["Pod"]
  parameters:
    repos:
    - "registry.example.com/"
    - "gcr.io/my-project/"
    - "123456789.dkr.ecr.us-east-1.amazonaws.com/"
```

## 反模式

### 禁止操作

```yaml
# [FAIL] 禁止：硬编码云服务特定功能
# 直接使用 AWS S3 SDK，无法迁移
s3_client = boto3.client('s3')

# [FAIL] 禁止：分散的身份管理
# 每个云独立管理用户

# [FAIL] 禁止：无成本监控
# 缺少成本告警和预算控制

# [FAIL] 禁止：无灾备方案
# 单云部署无备份

# [FAIL] 禁止：不一致的安全策略
# 不同云使用不同安全标准

# [FAIL] 禁止：无供应商锁定评估
# 使用云特有服务无替代方案
```

## 实战案例

### 案例 1：跨云灾备切换

```yaml
# 主站点（AWS）
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: primary-ingress
  namespace: production
  annotations:
    external-dns.alpha.kubernetes.io/set-identifier: "primary"
    external-dns.alpha.kubernetes.io/aws-weight: "100"
spec:
  rules:
  - host: api.example.com
    http:
      paths:
      - path: /
        backend:
          service:
            name: api-service
            port:
              number: 80

---
# 备份站点（Azure）
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: secondary-ingress
  namespace: production
  annotations:
    external-dns.alpha.kubernetes.io/set-identifier: "secondary"
    external-dns.alpha.kubernetes.io/azure-weight: "0"  # 灾备时改为 100
spec:
  rules:
  - host: api.example.com
    http:
      paths:
      - path: /
        backend:
          service:
            name: api-service
            port:
              number: 80
```

### 案例 2：成本优化实施

```bash
# 1. 识别闲置资源
aws ec2 describe-instances --query 'Reservations[].Instances[?State.Name==`stopped`]'
az vm list --query "[?powerState=='VM deallocated']"

# 2. 调整资源大小
kubectl patch deployment api-service -p '{"spec":{"template":{"spec":{"containers":[{"name":"api","resources":{"requests":{"cpu":"100m"}}}]}}}}'

# 3. 启用自动伸缩
kubectl autoscale deployment api-service --cpu-percent=70 --min=2 --max=10

# 4. 使用 Spot/Preemptible 实例
# 在 node selector 中添加 spot 实例标签

# 5. 预留实例购买
# 根据稳定工作负载购买预留实例
```

## 检查清单

### 治理检查

- [ ] 统一身份认证配置
- [ ] 成本标签策略实施
- [ ] 安全策略一致性
- [ ] 合规性审计通过
- [ ] 访问权限最小化

### 运维检查

- [ ] 多云监控集成
- [ ] 日志集中收集
- [ ] 告警规则统一
- [ ] 备份策略执行
- [ ] 灾备演练完成

### 成本检查

- [ ] 月度成本审查
- [ ] 闲置资源清理
- [ ] 预留实例评估
- [ ] Spot 实例使用
- [ ] 成本告警配置

### 安全检查

- [ ] 网络隔离正确
- [ ] 加密配置一致
- [ ] 访问日志审计
- [ ] 漏洞扫描完成
- [ ] 合规性验证

## 参考资料

- [AWS Well-Architected Framework](https://aws.amazon.com/architecture/well-architected/)
- [Azure Cloud Adoption Framework](https://docs.microsoft.com/azure/cloud-adoption-framework/)
- [Google Cloud Architecture Framework](https://cloud.google.com/architecture/framework)
- [Crossplane 文档](https://crossplane.io/docs/)
- [FinOps Foundation](https://www.finops.org/)
- [多云架构模式](https://www.oreilly.com/library/view/multicloud-architecture/9781492053102/)