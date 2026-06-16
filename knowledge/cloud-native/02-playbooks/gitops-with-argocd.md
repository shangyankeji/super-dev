---
title: GitOps ArgoCD 作战手册
version: 1.0.0
last_updated: 2025-03-20
owner: platform-team
tags: [gitops, argocd, continuous-deployment, kubernetes]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：GitOps 与 ArgoCD 实施作战手册
# 作用：指导使用 ArgoCD 实现声明式持续部署
# 创建时间：2025-03-20
# 最后修改：2025-03-20

## 目标

建立 GitOps 标准化部署流程，确保：
- Git 作为单一事实来源
- 声明式配置管理
- 自动化同步和漂移检测
- 安全可控的发布流程

## 适用场景

- Kubernetes 应用持续部署
- 多环境配置管理
- 基础设施即代码
- 多集群统一管理

## 执行清单

### 部署前准备

- [ ] 确定 Git 仓库结构（单仓库 vs 多仓库）
- [ ] 配置 Git 访问凭证
- [ ] 规划环境隔离策略
- [ ] 设计目录结构
- [ ] 制定回滚流程

### ArgoCD 安装

- [ ] 安装 ArgoCD 控制平面
- [ ] 配置 SSO 集成
- [ ] 创建项目和权限
- [ ] 配置仓库访问
- [ ] 安装 CLI 工具

### 应用接入

- [ ] 创建 Application 资源
- [ ] 配置同步策略
- [ ] 配置健康检查
- [ ] 设置自动同步
- [ ] 配置通知

## 核心配置

### 1. ArgoCD 安装

```bash
# 创建命名空间
kubectl create namespace argocd

# 安装 ArgoCD
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# 获取初始密码
kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d
```

### 2. ArgoCD 配置

```yaml
# argocd-cm 配置
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-cm
  namespace: argocd
  labels:
    app.kubernetes.io/name: argocd-cm
    app.kubernetes.io/part-of: argocd
data:
  # URL 配置
  url: https://argocd.example.com
  # 禁用内置用户（使用 SSO）
  accounts.enabled: "false"
  # SSO 配置
  dex.config: |
    connectors:
    - type: oidc
      id: okta
      name: Okta
      config:
        issuer: https://your-org.okta.com
        clientID: $oidc.okta.clientID
        clientSecret: $oidc.okta.clientSecret
        preferredEmailDomains:
        - example.com
  # 资源剔除
  resource.exclusions: |
    - apiGroups:
      - ""
      kinds:
      - Event
      clusters:
      - "*"
  # 仓库凭证模板
  repositories: |
    - url: https://github.com/org/k8s-configs
      name: k8s-configs
      type: git
```

```yaml
# argocd-rbac-cm 配置
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-rbac-cm
  namespace: argocd
data:
  policy.csv: |
    # 只读用户组
    g, read-only, role:readonly
    # 开发者组 - 特定命名空间权限
    g, developers, role:developer
    # 管理员组
    g, admins, role:admin
    # 项目级别策略
    p, role:developer, applications, get, production/*, allow
    p, role:developer, applications, sync, staging/*, allow
  policy.default: role:readonly
```

### 3. 项目配置

```yaml
apiVersion: argoproj.io/v1alpha1
kind: AppProject
metadata:
  name: production
  namespace: argocd
spec:
  description: Production environment project
  # 源仓库
  sourceRepos:
  - https://github.com/org/k8s-configs
  - https://github.com/org/helm-charts
  # 目标集群和命名空间
  destinations:
  - namespace: production
    server: https://kubernetes.default.svc
  - namespace: monitoring
    server: https://kubernetes.default.svc
  # 允许的资源
  clusterResourceWhitelist:
  - group: ''
    kind: Namespace
  - group: rbac.authorization.k8s.io
    kind: ClusterRole
  - group: rbac.authorization.k8s.io
    kind: ClusterRoleBinding
  # 命名空间资源白名单
  namespaceResourceWhitelist:
  - group: '*'
    kind: '*'
  # 同步窗口（维护窗口）
  syncWindows:
  - kind: deny
    schedule: '0 0 * * *'
    duration: 1h
    namespaces:
    - production
  # 角色配置
  roles:
  - name: developer
    description: Developer access
    policies:
    - p, proj:production:developer, applications, get, production/*, allow
    - p, proj:production:developer, applications, sync, production/*, allow
    groups:
    - developers
```

### 4. Application 配置

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: production-app
  namespace: argocd
  finalizers:
  - resources-finalizer.argocd.argoproj.io
spec:
  project: production
  source:
    repoURL: https://github.com/org/k8s-configs
    targetRevision: main
    path: apps/production/api-service
    helm:
      valueFiles:
      - values.yaml
      - values-production.yaml
      parameters:
      - name: image.tag
        value: v1.2.3
  destination:
    server: https://kubernetes.default.svc
    namespace: production
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
      allowEmpty: false
    syncOptions:
    - Validate=true
    - CreateNamespace=true
    - PrunePropagationPolicy=foreground
    - PruneLast=true
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
  ignoreDifferences:
  - group: apps
    kind: Deployment
    jsonPointers:
    - /spec/replicas
  info:
  - name: Team
    value: Backend Team
  - name: On-call
    value: backend@example.com
```

### 5. ApplicationSet（多环境）

```yaml
apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: multi-environment-apps
  namespace: argocd
spec:
  generators:
  - list:
      elements:
      - cluster: staging
        url: https://staging.kubernetes.local
        namespace: staging
      - cluster: production
        url: https://production.kubernetes.local
        namespace: production
  template:
    metadata:
      name: '{{cluster}}-api-service'
    spec:
      project: '{{cluster}}'
      source:
        repoURL: https://github.com/org/k8s-configs
        targetRevision: main
        path: apps/api-service
        helm:
          valueFiles:
          - values.yaml
          - 'values-{{cluster}}.yaml'
      destination:
        server: '{{url}}'
        namespace: '{{namespace}}'
      syncPolicy:
        automated:
          prune: true
          selfHeal: true
```

### 6. Kustomize 应用

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: kustomize-app
  namespace: argocd
spec:
  project: production
  source:
    repoURL: https://github.com/org/k8s-configs
    targetRevision: main
    path: apps/api-service/overlays/production
    kustomize:
      namePrefix: prod-
      images:
      - api-service=v1.2.3
      commonLabels:
        environment: production
      patches:
      - target:
          kind: Deployment
          name: api-service
        patch: |-
          - op: add
            path: /spec/template/spec/containers/0/resources/limits/memory
            value: 4Gi
  destination:
    server: https://kubernetes.default.svc
    namespace: production
```

### 7. 通知配置

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-notifications-cm
  namespace: argocd
data:
  service.slack: |
    token: $slack-token
  template.app-deployed: |
    email:
      subject: Application {{.app.metadata.name}} deployed
    slack:
      attachments: |
        [{
          "title": "{{.app.metadata.name}}",
          "title_link": "{{.context.argocdUrl}}/applications/{{.app.metadata.name}}",
          "color": "#18be52",
          "fields": [
          {
            "title": "Sync Status",
            "value": "{{.app.status.sync.status}}",
            "short": true
          },
          {
            "title": "Revision",
            "value": "{{.app.status.sync.revision}}",
            "short": true
          }
          ]
        }]
  trigger.on-deployed: |
    - description: Application deployed
      send:
      - app-deployed
      when: app.status.operationState.phase in ['Succeeded']
  subscriptions: |
    - recipients:
      - slack:deployments
      triggers:
      - on-deployed
      - on-sync-failed
```

### 8. Image Updater 配置

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: api-service
  namespace: argocd
  annotations:
    argocd-image-updater.argoproj.io/image-list: api=registry.example.com/api-service
    argocd-image-updater.argoproj.io/api.update-strategy: semver
    argocd-image-updater.argoproj.io/api.allow-tags: regexp:^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$
    argocd-image-updater.argoproj.io/write-back-method: git
    argocd-image-updater.argoproj.io/git-branch: main
spec:
  # ... Application spec
```

## 目录结构

### 单仓库结构

```
k8s-configs/
├── apps/
│   ├── api-service/
│   │   ├── base/
│   │   │   ├── deployment.yaml
│   │   │   ├── service.yaml
│   │   │   ├── configmap.yaml
│   │   │   └── kustomization.yaml
│   │   └── overlays/
│   │       ├── staging/
│   │       │   ├── kustomization.yaml
│   │       │   └── patches/
│   │       └── production/
│   │           ├── kustomization.yaml
│   │           └── patches/
│   └── web-app/
│       └── ...
├── infrastructure/
│   ├── monitoring/
│   ├── ingress-nginx/
│   └── cert-manager/
└── argocd/
    ├── projects/
    │   ├── production.yaml
    │   └── staging.yaml
    └── applications/
        ├── production/
        └── staging/
```

### Helm Chart 结构

```
charts/
├── api-service/
│   ├── Chart.yaml
│   ├── values.yaml
│   ├── values-staging.yaml
│   ├── values-production.yaml
│   └── templates/
│       ├── deployment.yaml
│       ├── service.yaml
│       ├── configmap.yaml
│       └── ingress.yaml
```

## 最佳实践

### 1. 环境隔离

```yaml
# 使用 Kustomize overlays 管理环境差异
# base/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-service
spec:
  replicas: 1
  template:
    spec:
      containers:
      - name: api
        resources:
          requests:
            cpu: 100m
            memory: 256Mi

---
# overlays/production/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
- ../../base
patchesStrategicMerge:
- deployment-patch.yaml

---
# overlays/production/deployment-patch.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-service
spec:
  replicas: 5
  template:
    spec:
      containers:
      - name: api
        resources:
          requests:
            cpu: 500m
            memory: 1Gi
          limits:
            cpu: 2000m
            memory: 4Gi
```

### 2. 渐进式发布

```yaml
# 使用 Argo Rollouts 进行渐进式发布
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: api-service
spec:
  replicas: 10
  strategy:
    canary:
      steps:
      - setWeight: 5
      - pause: {duration: 10m}
      - setWeight: 20
      - pause: {duration: 10m}
      - setWeight: 50
      - pause: {duration: 10m}
      - setWeight: 80
      - pause: {duration: 10m}
      analysis:
        templates:
        - templateName: success-rate
        startingStep: 2
        args:
        - name: service-name
          value: api-service-canary
  selector:
    matchLabels:
      app: api-service
  template:
    # Pod template
```

### 3. 密钥管理

```yaml
# 使用 Sealed Secrets 或 External Secrets
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: api-secrets
  namespace: production
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: vault-backend
    kind: ClusterSecretStore
  target:
    name: api-secrets
    creationPolicy: Owner
  data:
  - secretKey: db-password
    remoteRef:
      key: secret/data/production/api
      property: db_password
```

### 4. 资源钩子

```yaml
# PreSync 钩子 - 数据库迁移
apiVersion: batch/v1
kind: Job
metadata:
  name: db-migration
  annotations:
    argocd.argoproj.io/hook: PreSync
    argocd.argoproj.io/hook-delete-policy: HookSucceeded
spec:
  template:
    spec:
      containers:
      - name: migration
        image: migration-tool:v1.0.0
        command: ["./migrate.sh"]
      restartPolicy: Never
  backoffLimit: 0

---
# PostSync 钩子 - 通知
apiVersion: batch/v1
kind: Job
metadata:
  name: notify-deployment
  annotations:
    argocd.argoproj.io/hook: PostSync
    argocd.argoproj.io/hook-delete-policy: HookSucceeded
spec:
  template:
    spec:
      containers:
      - name: notify
        image: curlimages/curl
        command:
        - curl
        - -X
        - POST
        - -H
        - 'Content-Type: application/json'
        - -d
        - '{"text":"Deployment completed"}'
        - https://hooks.slack.com/services/xxx
      restartPolicy: Never
```

## 反模式

### 禁止操作

```yaml
# [FAIL] 禁止：手动修改集群资源
kubectl edit deployment api-service

# [FAIL] 禁止：禁用自动同步漂移检测
syncPolicy:
  automated:
    selfHeal: false  # 应该为 true

# [FAIL] 禁止：直接提交 Secret 明文
apiVersion: v1
kind: Secret
metadata:
  name: api-secret
stringData:
  password: "plaintext-password"  # 应使用 SealedSecret

# [FAIL] 禁止：使用 latest 标签
image: myapp:latest  # 应使用固定版本

# [FAIL] 禁止：忽略资源冲突
ignoreDifferences:
- group: '*'
  kind: '*'  # 过于宽泛

# [FAIL] 禁止：生产环境自动同步无审批
syncPolicy:
  automated:
    prune: true
    selfHeal: true  # 生产环境应有审批流程
```

## 实战案例

### 案例 1：紧急修复回滚

```bash
# 1. 禁用自动同步
argocd app set production-api --sync-policy none

# 2. 回滚到上一个版本
argocd app rollback production-api

# 3. 或者回滚到特定版本
argocd app history production-api
argocd app rollback production-api <revision>

# 4. 修复代码并推送
git revert <commit>
git push

# 5. 重新启用自动同步
argocd app set production-api --sync-policy automated
```

### 案例 2：多集群部署

```yaml
# 添加远程集群
apiVersion: v1
kind: Secret
metadata:
  name: cluster-production-useast
  namespace: argocd
  labels:
    argocd.argoproj.io/secret-type: cluster
type: Opaque
stringData:
  name: production-useast
  server: https://production-useast.example.com
  config: |
    {
      "bearerToken": "<token>",
      "tlsClientConfig": {
        "insecure": false,
        "caData": "<base64-ca>"
      }
    }

---
# ApplicationSet 多集群部署
apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: multi-cluster-apps
  namespace: argocd
spec:
  generators:
  - clusters:
      selector:
        matchLabels:
          environment: production
  template:
    metadata:
      name: '{{name}}-api-service'
    spec:
      project: production
      source:
        repoURL: https://github.com/org/k8s-configs
        path: apps/api-service
        targetRevision: main
      destination:
        server: '{{server}}'
        namespace: production
```

### 案例 3：漂移检测和自愈

```bash
# 手动触发漂移检测
argocd app diff production-api --refresh

# 查看差异详情
argocd app diff production-api --local ./manifests

# 强制同步修复漂移
argocd app sync production-api --force

# 查看同步状态
argocd app get production-api --refresh
```

## 检查清单

### 初始部署检查

- [ ] Git 仓库可访问
- [ ] ArgoCD 项目配置正确
- [ ] RBAC 权限配置
- [ ] 仓库凭证配置
- [ ] Application 创建成功
- [ ] 首次同步成功

### 日常运维检查

- [ ] 应用状态为 Healthy
- [ ] 同步状态为 Synced
- [ ] 无漂移告警
- [ ] 通知配置正常
- [ ] 日志正常输出
- [ ] 资源配额合理

### 发布检查

- [ ] Git 分支策略正确
- [ ] PR 审核完成
- [ ] CI 测试通过
- [ ] 变更清单确认
- [ ] 回滚方案准备
- [ ] 监控告警配置

### 安全检查

- [ ] RBAC 最小权限
- [ ] Secret 已加密
- [ ] Git 访问受控
- [ ] 审计日志启用
- [ ] SSO 集成正常

## 参考资料

- [ArgoCD 官方文档](https://argo-cd.readthedocs.io/)
- [ArgoCD 最佳实践](https://argo-cd.readthedocs.io/en/stable/user-guide/best_practices/)
- [GitOps 原则](https://opengitops.dev/)
- [Kustomize 文档](https://kubectl.docs.kubernetes.io/guides/introduction/kustomize/)
- [Argo Rollouts](https://argoproj.github.io/argo-rollouts/)