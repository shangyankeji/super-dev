---
title: 云原生词汇表
version: 1.0.0
last_updated: 2025-03-20
owner: platform-team
tags: [cloud-native, glossary, terminology]
status: production
domain: cloud-native
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：云原生核心术语定义
# 作用：统一术语理解，便于沟通和学习
# 创建时间：2025-03-20
# 最后修改：2025-03-20

## A

### API Server
Kubernetes 控制平面组件，提供 RESTful API 接口，是集群管理的入口。

### Admission Controller
Kubernetes API 请求的拦截插件，用于验证和修改请求。

### AppArmor
Linux 安全模块，通过配置文件限制程序的能力。

### ArgoCD
声明式 GitOps 持续部署工具，以 Git 为单一事实来源。

### Autoscaler
自动扩缩容组件，包括 HPA（Pod 级别）和 Cluster Autoscaler（节点级别）。

## B

### Blue-Green Deployment
蓝绿部署，维护两套完全相同的环境，通过切换流量实现零停机部署。

### Broker
消息代理，在发布-订阅模式中接收和分发消息。

## C

### Canary Release
金丝雀发布，逐步将流量导向新版本，降低发布风险。

### cgroups (Control Groups)
Linux 内核功能，限制、记录和隔离进程组使用的物理资源。

### CI/CD
持续集成/持续部署，自动化软件交付流程。

### Cluster
集群，一组节点（物理机或虚拟机）的集合，运行容器化应用。

### Cluster Autoscaler
Kubernetes 组件，根据资源需求自动调整节点数量。

### ConfigMap
Kubernetes 资源，用于存储非敏感配置数据。

### Container
容器，轻量级、可执行的独立软件包，包含运行所需的所有内容。

### Container Runtime
容器运行时，负责运行容器的软件（如 containerd、CRI-O）。

### ContainerD
高性能容器运行时，Docker 项目的核心组件。

### Control Plane
控制平面，Kubernetes 集群的大脑，管理集群状态。

### CRI (Container Runtime Interface)
容器运行时接口，Kubernetes 与容器运行时交互的标准。

### CSI (Container Storage Interface)
容器存储接口，Kubernetes 与存储系统交互的标准。

### CNI (Container Network Interface)
容器网络接口，配置容器网络的标准。

## D

### DaemonSet
Kubernetes 资源，确保每个节点运行一个 Pod 副本。

### Deployment
Kubernetes 资源，管理无状态应用的部署和更新。

### Desired State
期望状态，系统应该达到的目标配置。

### Distroles
极简容器镜像，仅包含应用程序及其运行时依赖。

### Docker
容器化平台，用于构建、分发和运行容器。

## E

### etcd
分布式键值存储，用于存储 Kubernetes 集群的所有数据。

### Event
Kubernetes 事件，记录集群中发生的操作和状态变化。

### External Secrets
Kubernetes 扩展，从外部密钥管理系统（如 Vault）同步密钥。

## F

### Falco
云原生运行时安全工具，检测异常行为。

### Federation
集群联邦，跨多个 Kubernetes 集群管理资源。

## G

### Gateway
服务网格入口点，处理南北向流量。

### GitOps
使用 Git 作为单一事实来源的基础设施和应用管理方法。

### gRPC
高性能 RPC 框架，使用 Protocol Buffers 序列化。

## H

### Helm
Kubernetes 包管理器，使用 Chart 管理应用。

### Horizontal Pod Autoscaler (HPA)
Kubernetes 资源，根据 CPU/内存使用率自动扩缩 Pod 数量。

## I

### IaC (Infrastructure as Code)
基础设施即代码，使用代码管理和配置基础设施。

### Image
容器镜像，包含应用程序及其依赖的只读模板。

### Ingress
Kubernetes 资源，管理外部访问集群内服务的规则。

### Istio
开源服务网格，提供流量管理、安全、可观测性。

## J

### Jaeger
分布式追踪系统，用于监控和故障排查。

## K

### kubectl
Kubernetes 命令行工具，用于与集群交互。

### Kubelet
Kubernetes 节点代理，负责 Pod 生命周期管理。

### kube-proxy
Kubernetes 网络代理，实现 Service 的负载均衡。

### Kubernetes (K8s)
开源容器编排平台，自动化部署、扩展和管理容器化应用。

### Kustomize
Kubernetes 原生配置管理工具，支持声明式定制。

## L

### Label
键值对标签，附加到 Kubernetes 对象上用于选择和组织。

### Liveness Probe
存活探针，检测容器是否运行，失败则重启容器。

### LoadBalancer
负载均衡器类型 Service，通过云提供商的负载均衡器暴露服务。

## M

### Microservices
微服务架构，将应用拆分为小型、独立的服务。

### mTLS (Mutual TLS)
双向 TLS，服务间双向认证和加密通信。

### Multicloud
多云策略，使用多个云服务提供商。

## N

### Namespace
命名空间，Kubernetes 集群内的虚拟集群，用于资源隔离。

### Network Policy
网络策略，控制 Pod 间网络流量的规则。

### Node
节点，Kubernetes 集群中的工作机器。

## O

### OPA (Open Policy Agent)
策略引擎，用于声明式策略定义和执行。

### Operator
Kubernetes 扩展模式，使用自定义资源管理复杂应用。

## P

### Persistent Volume (PV)
持久卷，集群级别的存储资源。

### Persistent Volume Claim (PVC)
持久卷声明，用户对存储资源的请求。

### Pod
Kubernetes 最小部署单元，包含一个或多个容器。

### Pod Security Policy (PSP)
Pod 安全策略，控制 Pod 的安全配置（已废弃，使用 Pod Security Standards）。

### Prometheus
开源监控和告警系统，云原生监控标准。

## R

### RBAC (Role-Based Access Control)
基于角色的访问控制，Kubernetes 权限管理机制。

### Readiness Probe
就绪探针，检测容器是否准备好接收流量。

### ReplicaSet
Kubernetes 资源，维护指定数量的 Pod 副本。

### Rolling Update
滚动更新，逐步替换旧版本 Pod 的更新策略。

## S

### Seccomp (Secure Computing Mode)
Linux 安全功能，限制进程可以调用的系统调用。

### Secret
Kubernetes 资源，用于存储敏感信息（密码、密钥等）。

### Selector
选择器，通过标签筛选 Kubernetes 对象。

### Self-Healing
自愈，系统自动检测和修复故障的能力。

### Service
Kubernetes 资源，定义一组 Pod 的访问策略。

### Service Account
服务账户，Pod 用于访问 Kubernetes API 的身份。

### Service Mesh
服务网格，处理服务间通信的基础设施层。

### Sidecar
边车模式，在同一个 Pod 中运行辅助容器。

### StatefulSet
Kubernetes 资源，管理有状态应用的部署。

## T

### Taint
污点，标记节点以阻止 Pod 调度（除非有匹配的容忍度）。

### Toleration
容忍度，允许 Pod 调度到有特定污点的节点。

### Tracing
追踪，跟踪请求在分布式系统中的路径。

## U

### User Namespace
用户命名空间，隔离用户和组 ID。

## V

### Vertical Pod Autoscaler (VPA)
Kubernetes 扩展，自动调整 Pod 的 CPU 和内存资源。

### Virtual Service
Istio 资源，配置服务网格中的流量路由规则。

## W

### Workload
工作负载，运行在 Kubernetes 上的应用程序。

## Z

### Zero Downtime
零停机，部署过程中服务持续可用。

### Zero Trust
零信任，默认不信任任何用户或系统，持续验证。

## 缩写对照表

| 缩写 | 全称 |
|------|------|
| K8s | Kubernetes |
| HPA | Horizontal Pod Autoscaler |
| VPA | Vertical Pod Autoscaler |
| RBAC | Role-Based Access Control |
| PV | Persistent Volume |
| PVC | Persistent Volume Claim |
| CRD | Custom Resource Definition |
| CNI | Container Network Interface |
| CSI | Container Storage Interface |
| CRI | Container Runtime Interface |
| mTLS | Mutual TLS |
| OPA | Open Policy Agent |
| IaC | Infrastructure as Code |
| CI/CD | Continuous Integration/Continuous Deployment |

## 参考资料

- [CNCF Glossary](https://glossary.cncf.io/)
- [Kubernetes 术语表](https://kubernetes.io/zh-cn/docs/reference/glossary/)
- [Istio 术语表](https://istio.io/latest/docs/reference/glossary/)