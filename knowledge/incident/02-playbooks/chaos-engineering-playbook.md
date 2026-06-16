---
title: 混沌工程作战手册
version: 1.0.0
last_updated: 2026-03-28
owner: sre-team
tags: [chaos-engineering, Chaos-Monkey, LitmusChaos, Gremlin, fault-injection, resilience, gameday]
status: production
domain: incident
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）
# 功能：混沌工程全流程作战手册
# 作用：指导团队安全地实施混沌实验以验证系统韧性
# 创建时间：2026-03-28
# 最后修改：2026-03-28

## 目标

建立混沌工程标准化实践，确保：
- 系统韧性通过主动故障注入而非被动等待来验证
- 每个实验有明确假设、安全边界和回滚机制
- 实验结果量化并转化为可落地的改进项
- 团队对故障场景建立肌肉记忆（GameDay 演练）
- 生产环境混沌实验安全可控（爆炸半径最小化）

## 适用场景

- 验证高可用架构（多副本/主从切换/跨可用区）
- 验证降级与熔断策略
- 验证自动伸缩与自愈能力
- 验证监控告警覆盖度
- 验证应急预案有效性
- 新系统上线前韧性验收

## 前置条件

### 环境要求

| 项目 | 要求 |
|------|------|
| 可观测性 | 监控（Prometheus/Grafana）+ 日志（ELK）+ 追踪（Jaeger）已部署 |
| 告警 | 核心服务告警规则已配置且验证过可触发 |
| 容器编排 | Kubernetes 1.24+（LitmusChaos 要求） |
| 回滚能力 | 服务可在 1 分钟内回滚到上一版本 |
| 团队准备 | SRE + 开发 + 运维已完成混沌工程培训 |

### 工具链安装

```bash
# LitmusChaos（Kubernetes 原生，开源）
# 安装 LitmusChaos Control Plane
kubectl apply -f https://litmuschaos.github.io/litmus/3.0.0/litmus-3.0.0.yaml
# 安装 Chaos Runner
kubectl apply -f https://hub.litmuschaos.io/api/chaos/3.0.0/install

# Chaos Mesh（CNCF 项目，Kubernetes 原生）
helm repo add chaos-mesh https://charts.chaos-mesh.org
helm install chaos-mesh chaos-mesh/chaos-mesh -n chaos-mesh --create-namespace \
  --set chaosDaemon.runtime=containerd \
  --set chaosDaemon.socketPath=/run/containerd/containerd.sock

# Gremlin（商业版，支持物理机/容器/云）
# 安装 Agent
curl -sL https://apt.gremlin.com/install.sh | sudo bash

# Toxiproxy（网络故障注入）
docker run -d --name toxiproxy -p 8474:8474 -p 8000-9000:8000-9000 ghcr.io/shopify/toxiproxy

# stress-ng（资源压力测试）
sudo apt install stress-ng
```

### 安全前提

- [ ] 混沌实验章程已获管理层批准
- [ ] 实验范围（命名空间/服务/环境）已明确
- [ ] 生产环境实验需至少 2 人确认（四眼原则）
- [ ] 紧急停止机制已验证可用（Kill Switch）
- [ ] 实验窗口避开业务高峰（通常选工作日上午 10:00-12:00）
- [ ] 客服团队已提前通知

---

## 一、实验设计

### 1.1 混沌实验方法论

```yaml
混沌工程四步法:
  1. 建立稳态假设:
     定义系统正常行为的量化指标
     示例:
       - "API P99 延迟 < 500ms"
       - "订单成功率 > 99.5%"
       - "错误率 < 0.1%"

  2. 引入现实世界变量:
     注入模拟真实故障场景的干扰
     示例:
       - 杀死一个 Pod
       - 注入网络延迟
       - 耗尽 CPU 资源

  3. 观察系统行为:
     对比稳态指标在实验期间的变化
     关键问题:
       - 系统是否自动恢复？
       - 恢复用了多长时间？
       - 用户是否感知到故障？

  4. 验证或推翻假设:
     假设成立 → 系统韧性验证通过
     假设推翻 → 发现弱点 → 创建改进项
```

### 1.2 故障注入类型

```yaml
基础设施层:
  节点故障:
    - 节点下线（drain + cordon）
    - 节点重启
    - 节点网络不可达

  资源耗尽:
    - CPU 100%
    - 内存 OOM
    - 磁盘空间满
    - 文件描述符耗尽

应用层:
  Pod/容器故障:
    - 随机杀 Pod
    - 容器 OOM Kill
    - 容器启动失败（镜像拉取失败）
    - 就绪探针失败

  进程故障:
    - 主进程 Kill
    - JVM Full GC（Java）
    - 线程池耗尽

网络层:
  连通性:
    - 网络分区（Pod A 无法访问 Pod B）
    - DNS 解析失败
    - 端口不可达

  质量退化:
    - 延迟注入（100ms ~ 5s）
    - 丢包（1% ~ 50%）
    - 带宽限制
    - 连接重置（RST）

依赖层:
  数据库:
    - 主库宕机
    - 从库延迟增大
    - 连接池耗尽
    - 慢查询

  缓存:
    - Redis 不可达
    - 缓存数据清空
    - 缓存延迟增大

  消息队列:
    - Kafka Broker 宕机
    - 消费延迟
    - Topic 不可用

  外部服务:
    - 第三方 API 超时
    - 第三方 API 返回错误
    - 第三方 API 限流
```

### 1.3 实验模板

```yaml
# 实验卡片模板
实验 ID: CHAOS-2026-001
实验名称: 订单服务单 Pod 故障恢复验证
实验日期: 2026-03-28 10:00-12:00
负责人: SRE-张三
参与人: 开发-李四, 运维-王五

稳态假设:
  - 订单 API P99 延迟 < 500ms
  - 订单创建成功率 > 99.5%
  - 告警在 2 分钟内触发

故障类型: Pod Kill（随机杀死 1/3 订单服务 Pod）
爆炸半径: 仅订单服务，最多影响 1 个 Pod
持续时间: 5 分钟

前提条件:
  - 订单服务当前 3 副本运行正常
  - 监控大盘已打开
  - 回滚命令已准备

中止条件（任一触发立即停止）:
  - 订单创建成功率 < 95%
  - P99 延迟 > 2s 持续 3 分钟
  - 出现数据不一致

预期结果:
  - Kubernetes 自动重建 Pod（< 60s）
  - 期间请求被负载均衡到存活 Pod
  - 用户无感知或仅有短暂延迟上升

实际结果: [实验后填写]
改进项: [实验后填写]
```

---

## 二、工具实战

### 2.1 LitmusChaos 实验

```yaml
# Pod Kill 实验
apiVersion: litmuschaos.io/v1alpha1
kind: ChaosEngine
metadata:
  name: order-service-pod-kill
  namespace: production
spec:
  appinfo:
    appns: production
    applabel: app=order-service
    appkind: deployment
  engineState: active
  chaosServiceAccount: litmus-admin
  experiments:
    - name: pod-delete
      spec:
        components:
          env:
            - name: TOTAL_CHAOS_DURATION
              value: '300'        # 5 分钟
            - name: CHAOS_INTERVAL
              value: '60'         # 每 60 秒杀一次
            - name: FORCE
              value: 'true'
            - name: PODS_AFFECTED_PERC
              value: '33'         # 杀 1/3 的 Pod
        probe:
          - name: order-api-health
            type: httpProbe
            httpProbe/inputs:
              url: http://order-service.production:8080/health
              method:
                get:
                  criteria: ==
                  responseCode: '200'
            mode: Continuous
            runProperties:
              probeTimeout: 5
              retry: 3
              interval: 10
```

```yaml
# 网络延迟实验
apiVersion: litmuschaos.io/v1alpha1
kind: ChaosEngine
metadata:
  name: order-db-network-latency
  namespace: production
spec:
  appinfo:
    appns: production
    applabel: app=order-service
    appkind: deployment
  engineState: active
  chaosServiceAccount: litmus-admin
  experiments:
    - name: pod-network-latency
      spec:
        components:
          env:
            - name: TOTAL_CHAOS_DURATION
              value: '180'         # 3 分钟
            - name: NETWORK_LATENCY
              value: '500'         # 注入 500ms 延迟
            - name: JITTER
              value: '100'         # 抖动 ±100ms
            - name: DESTINATION_IPS
              value: '10.0.1.100'  # 数据库 IP
            - name: DESTINATION_PORTS
              value: '5432'        # PostgreSQL 端口
```

### 2.2 Chaos Mesh 实验

```yaml
# CPU 压力实验
apiVersion: chaos-mesh.org/v1alpha1
kind: StressChaos
metadata:
  name: order-service-cpu-stress
  namespace: production
spec:
  mode: one                        # 影响 1 个 Pod
  selector:
    namespaces:
      - production
    labelSelectors:
      app: order-service
  stressors:
    cpu:
      workers: 4                   # 4 个 CPU 工作线程
      load: 90                     # 每个工作线程 90% 负载
  duration: '5m'

---
# 网络分区实验（订单服务 → 支付服务不可达）
apiVersion: chaos-mesh.org/v1alpha1
kind: NetworkChaos
metadata:
  name: order-payment-partition
  namespace: production
spec:
  action: partition
  mode: all
  selector:
    namespaces:
      - production
    labelSelectors:
      app: order-service
  direction: to
  target:
    mode: all
    selector:
      namespaces:
        - production
      labelSelectors:
        app: payment-service
  duration: '3m'

---
# IO 故障实验（磁盘延迟）
apiVersion: chaos-mesh.org/v1alpha1
kind: IOChaos
metadata:
  name: order-db-io-latency
  namespace: production
spec:
  action: latency
  mode: one
  selector:
    namespaces:
      - production
    labelSelectors:
      app: postgresql
  volumePath: /var/lib/postgresql/data
  path: '*'
  delay: '200ms'
  percent: 80                      # 80% 的 IO 操作受影响
  duration: '3m'
```

### 2.3 Gremlin 实验

```bash
# CPU 攻击
gremlin attack cpu --length 300 --cores 2 --percent 90 \
  --target-type kubernetes --namespace production --label app=order-service

# 网络延迟
gremlin attack network latency --length 180 --delay 500 --jitter 100 \
  --target-type kubernetes --namespace production --label app=order-service \
  --port 5432

# 进程 Kill
gremlin attack process kill --length 60 --interval 30 \
  --process java \
  --target-type kubernetes --namespace production --label app=order-service

# DNS 故障
gremlin attack dns --length 120 \
  --domain payment-service.production.svc.cluster.local \
  --target-type kubernetes --namespace production --label app=order-service

# 磁盘空间填充
gremlin attack disk fill --length 180 --dir /tmp --percent 95 \
  --target-type kubernetes --namespace production --label app=order-service
```

### 2.4 Toxiproxy（网络故障注入）

```bash
# 创建代理（模拟 Redis 连接）
toxiproxy-cli create redis-proxy -l 0.0.0.0:6380 -u redis:6379

# 添加延迟
toxiproxy-cli toxic add redis-proxy -t latency -a latency=500 -a jitter=100

# 添加丢包
toxiproxy-cli toxic add redis-proxy -t timeout -a timeout=3000

# 限制带宽
toxiproxy-cli toxic add redis-proxy -t bandwidth -a rate=10  # 10 KB/s

# 连接重置
toxiproxy-cli toxic add redis-proxy -t reset_peer -a timeout=2000

# 查看所有代理状态
toxiproxy-cli list

# 移除故障（恢复正常）
toxiproxy-cli toxic remove redis-proxy -n latency_downstream
```

---

## 三、安全实施

### 3.1 爆炸半径控制

```yaml
最小化影响原则:
  环境分级:
    Level 1 - 开发/测试环境: 任意实验，无需审批
    Level 2 - Staging 环境: 需 SRE 审批，可激进
    Level 3 - 生产环境: 需 2 人审批（四眼原则），严格控制

  逐步升级:
    第 1 周: 开发环境验证实验脚本
    第 2 周: Staging 环境全量实验
    第 3 周: 生产环境最小影响实验（1 个 Pod / 1% 流量）
    第 4 周: 生产环境扩大范围（基于前周结果）

  流量隔离:
    - 使用 Feature Flag 将实验流量标记
    - 实验期间仅影响内部测试流量
    - 生产用户流量通过正常路径

Kill Switch（紧急停止）:
  # LitmusChaos
  kubectl patch chaosengine <name> -n production --type merge -p '{"spec":{"engineState":"stop"}}'

  # Chaos Mesh
  kubectl delete stresschaos <name> -n production

  # Gremlin
  gremlin halt

  # 通用：删除所有混沌实验
  kubectl delete chaosengine --all -n production
  kubectl delete networkchaos,stresschaos,iochaos --all -n production
```

### 3.2 监控与告警

```yaml
实验期间必看大盘:
  黄金信号（Golden Signals）:
    延迟:
      - API P50 / P99 / P99.9 延迟
      - 数据库查询延迟
      - 上游服务调用延迟

    流量:
      - 请求 QPS（总量/成功/失败）
      - 活跃连接数

    错误:
      - HTTP 5xx 比例
      - 业务错误码比例
      - 超时比例

    饱和度:
      - CPU / 内存 / 磁盘利用率
      - 连接池使用率
      - 消息队列积压量

  Grafana Dashboard 配置:
    # 创建专用混沌实验 Dashboard
    # 包含以下面板：
    - 实验状态指示器（进行中/已完成/已中止）
    - 稳态指标实时对比（实验前基线 vs 当前）
    - 受影响 Pod/节点列表
    - 告警触发时间线

告警规则（实验期间额外启用）:
  # Prometheus 告警规则
  groups:
    - name: chaos-experiment-alerts
      rules:
        - alert: ChaosExperimentSLOBreach
          expr: |
            (sum(rate(http_requests_total{status=~"5.."}[1m]))
            / sum(rate(http_requests_total[1m]))) > 0.05
          for: 2m
          labels:
            severity: critical
            team: sre
          annotations:
            summary: "混沌实验期间错误率超过 5%，考虑中止实验"

        - alert: ChaosExperimentLatencyBreach
          expr: |
            histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[1m])) by (le)) > 2
          for: 3m
          labels:
            severity: warning
          annotations:
            summary: "混沌实验期间 P99 延迟超过 2s"
```

### 3.3 实验执行流程

```yaml
执行前（T-30min）:
  - [ ] 确认实验卡片已审批
  - [ ] 参与人员全部就位
  - [ ] 监控大盘已打开
  - [ ] Kill Switch 命令已准备并测试
  - [ ] 记录当前稳态指标（基线）
  - [ ] 通知相关团队（客服/产品）

执行中:
  - [ ] 按实验卡片注入故障
  - [ ] 持续观察监控大盘
  - [ ] 记录关键事件时间点
  - [ ] 如触发中止条件，立即执行 Kill Switch
  - [ ] 等待故障持续时间结束

执行后:
  - [ ] 确认故障已完全清除
  - [ ] 确认系统恢复到稳态
  - [ ] 收集实验期间的监控数据
  - [ ] 对比实验前后指标
  - [ ] 填写实验结果报告
```

---

## 四、实验场景库

### 4.1 基础场景（入门级）

```yaml
场景 1 - 单 Pod 故障:
  故障类型: 杀死 1 个 Pod
  验证目标: K8s 自动重建 + 流量自动转移
  预期结果: 30s 内新 Pod Running，用户无感知
  适用阶段: 第一次混沌实验

场景 2 - 缓存失效:
  故障类型: Redis 不可达 60s
  验证目标: 缓存降级 → 直接查询 DB
  预期结果: 延迟上升但服务可用，缓存恢复后自动回填
  注意: 关注 DB 连接池是否被打满

场景 3 - 外部 API 超时:
  故障类型: 第三方支付 API 延迟 10s
  验证目标: 熔断器开启 + 友好提示
  预期结果: 熔断后快速失败，不影响其他功能
```

### 4.2 进阶场景（中级）

```yaml
场景 4 - 数据库主从切换:
  故障类型: 杀死 PostgreSQL 主节点
  验证目标: 自动 Failover + 应用自动重连
  预期结果: Failover < 30s，写入中断 < 10s
  风险: 可能有少量事务丢失（取决于复制延迟）

场景 5 - 网络分区:
  故障类型: 可用区 A 与可用区 B 网络隔离
  验证目标: 跨可用区冗余是否生效
  预期结果: 每个可用区独立提供服务
  注意: 分布式锁/选举机制可能受影响

场景 6 - 级联故障:
  故障类型: 订单服务 CPU 100% → 上游超时 → 网关排队
  验证目标: 熔断器 + 限流 + 降级三者协同
  预期结果: 故障被隔离在订单服务，不扩散到全系统
```

### 4.3 高级场景（生产级 GameDay）

```yaml
场景 7 - 全可用区故障:
  故障类型: 模拟一个 AZ 完全不可用
  验证目标: 跨 AZ 容灾能力
  预期结果: 其余 AZ 承接全部流量，性能略降但可用
  前提: 架构设计支持跨 AZ 部署

场景 8 - 依赖方批量故障:
  故障类型: 同时关闭 Redis + Kafka
  验证目标: 多依赖同时故障时的系统行为
  预期结果: 核心读写路径可用（降级模式），异步任务堆积但不丢失

场景 9 - 流量突增:
  故障类型: 10 倍流量突增（模拟秒杀/营销活动）
  验证目标: 自动伸缩 + 限流
  预期结果: HPA 触发扩容，限流保护后端不被打挂
  工具: k6 / Locust 配合 Chaos 实验
```

---

## 五、结果分析

### 5.1 实验结果模板

```markdown
# 混沌实验结果报告

## 实验基本信息
- 实验 ID: CHAOS-2026-001
- 实验名称: 订单服务单 Pod 故障恢复验证
- 执行时间: 2026-03-28 10:00-10:30
- 环境: Staging / Production
- 执行人: SRE-张三

## 稳态假设验证

| 假设 | 基线值 | 实验期间值 | 恢复后值 | 结论 |
|------|--------|-----------|---------|------|
| P99 延迟 < 500ms | 230ms | 480ms（峰值） | 240ms | 通过 |
| 成功率 > 99.5% | 99.98% | 99.2%（最低） | 99.97% | 未通过 |
| 告警 < 2min | - | 1.5min | - | 通过 |

## 时间线
| 时间 | 事件 |
|------|------|
| 10:00 | 实验开始，杀死 Pod order-service-abc |
| 10:00:05 | K8s 检测到 Pod 不健康 |
| 10:00:15 | 新 Pod 开始创建 |
| 10:00:35 | 新 Pod 进入 Running 状态 |
| 10:00:45 | 新 Pod 通过就绪检查，接收流量 |
| 10:01:30 | 告警触发（Pod Restart） |
| 10:05:00 | 指标恢复到稳态 |

## 发现与改进

### 发现 1: 成功率短暂跌破 99.5%
- 原因：Pod 被杀时正在处理的请求直接失败
- 影响：约 50 个请求返回 502
- 改进项：配置 Pod preStop hook，在 SIGTERM 时先从 Service 摘除再优雅停机
- 优先级：P1
- 负责人：开发-李四
- 截止日期：2026-04-05

### 发现 2: 告警触发较慢（1.5min）
- 原因：告警评估间隔为 1 分钟
- 改进项：将 Pod 故障告警评估间隔缩短为 30 秒
- 优先级：P2
- 负责人：运维-王五
- 截止日期：2026-04-10
```

### 5.2 韧性评分卡

```yaml
评分维度（每项 1-5 分）:

自动恢复:
  5: 故障自动恢复，用户完全无感知
  4: 自动恢复，用户有短暂延迟（< 5s）
  3: 自动恢复，用户有明显延迟（5-30s）
  2: 需要人工介入才能恢复
  1: 无法恢复，需要回滚

故障隔离:
  5: 故障完全隔离，不影响其他服务
  4: 轻微影响邻近服务（延迟上升）
  3: 影响部分下游服务的非核心功能
  2: 导致多个服务降级
  1: 级联故障，全系统受影响

可观测性:
  5: 故障在 30s 内被检测到
  4: 1 分钟内检测到
  3: 5 分钟内检测到
  2: 需要人工巡检发现
  1: 故障未被监控覆盖

降级体验:
  5: 降级后功能完整，仅性能略降
  4: 非核心功能不可用，核心功能正常
  3: 核心功能部分可用
  2: 核心功能显著受损
  1: 服务完全不可用

综合评分:
  18-20: 优秀（生产就绪）
  14-17: 良好（可上线，需持续改进）
  10-13: 及格（需修复后复测）
  < 10: 不合格（需重新设计韧性方案）
```

---

## 六、GameDay 演练

### 6.1 GameDay 组织

```yaml
GameDay 定义:
  团队集中进行混沌实验的专项活动日。
  目的是在真实（或接近真实）的环境中验证系统韧性和团队响应能力。

频率: 每季度 1 次（生产环境）/ 每月 1 次（Staging）

角色分工:
  实验设计师（SRE）:
    - 设计实验场景
    - 准备故障注入脚本
    - 控制实验进度

  观察者（开发 + 运维）:
    - 监控系统指标
    - 记录异常行为
    - 执行应急响应

  裁判（Tech Lead）:
    - 判定实验是否需要中止
    - 评估团队响应质量
    - 汇总实验结果

  记录员:
    - 记录完整时间线
    - 记录所有发现
    - 整理实验报告

议程（半天）:
  09:00-09:30: 开场说明 + 回顾上次改进项
  09:30-10:00: 实验场景说明 + 确认安全措施
  10:00-11:30: 实验执行（3-4 个场景）
  11:30-12:00: 即时复盘 + 改进项整理
```

### 6.2 GameDay 评估表

```yaml
团队响应评估:
  | 评估项 | 评分(1-5) | 备注 |
  |--------|----------|------|
  | 故障发现速度 | | |
  | 沟通协作效率 | | |
  | 根因定位速度 | | |
  | 恢复操作正确性 | | |
  | 应急预案执行 | | |
  | 决策质量 | | |

系统评估:
  | 评估项 | 评分(1-5) | 备注 |
  |--------|----------|------|
  | 自动恢复能力 | | |
  | 降级方案有效性 | | |
  | 监控告警覆盖度 | | |
  | 故障隔离能力 | | |
  | 数据一致性保持 | | |
```

---

## 七、验证

### 7.1 混沌工程成熟度模型

```yaml
Level 1 - 起步:
  - 在开发/测试环境进行手动故障注入
  - 基本监控已覆盖
  - 团队了解混沌工程概念

Level 2 - 标准化:
  - 使用混沌工程工具（LitmusChaos / Chaos Mesh）
  - 实验模板化、可复现
  - Staging 环境定期实验

Level 3 - 自动化:
  - 混沌实验集成到 CI/CD（Staging 门禁）
  - 实验结果自动生成报告
  - 改进项自动创建 Ticket

Level 4 - 生产就绪:
  - 生产环境定期 GameDay
  - 自动化混沌实验（非工作时间自动运行）
  - 韧性评分纳入发布门禁

Level 5 - 持续混沌:
  - 生产环境持续混沌（Chaos as a Service）
  - 故障注入自适应（基于系统健康度调整强度）
  - 混沌实验覆盖全部核心路径
```

### 7.2 验证清单

| 指标 | 达标标准 |
|------|---------|
| 核心服务故障恢复时间 | < 60s（自动） |
| 告警触发时间 | < 2 分钟 |
| 熔断器生效时间 | < 30s |
| 降级方案覆盖率 | 所有外部依赖 100% |
| GameDay 频率 | >= 每季度 1 次 |
| 改进项闭环率 | > 90% |

---

## 八、回滚

### 实验回滚

```bash
# 紧急停止所有混沌实验

# LitmusChaos - 停止所有实验
kubectl get chaosengine -n production -o name | xargs -I {} kubectl patch {} -n production --type merge -p '{"spec":{"engineState":"stop"}}'

# Chaos Mesh - 删除所有实验
kubectl delete networkchaos,stresschaos,iochaos,podchaos,dnschaos --all -n production

# Gremlin - 全局停止
gremlin halt

# Toxiproxy - 移除所有 Toxic
for proxy in $(toxiproxy-cli list | tail -n +2 | awk '{print $1}'); do
  toxiproxy-cli toxic remove $proxy --all
done

# 验证系统恢复
kubectl get pods -n production
kubectl top pods -n production
curl -s http://api.target.com/health | jq .
```

### 实验导致真实故障时的处理

```yaml
如果混沌实验导致了预期外的真实故障:

  1. 立即停止实验（Kill Switch）

  2. 按安全事件响应流程处理:
     - 评估影响范围
     - 通知相关团队
     - 执行恢复操作

  3. 如果无法自动恢复:
     - 手动重启受影响服务
     kubectl rollout restart deployment/<service> -n production
     - 如果数据受损，从备份恢复

  4. 事后分析:
     - 为什么实验超出了预期爆炸半径？
     - 安全机制为什么没有生效？
     - 更新实验安全边界
```

---

## Agent Checklist

供自动化 Agent 在执行混沌工程流程时逐项核查：

- [ ] 混沌工程工具已安装并验证可用（LitmusChaos / Chaos Mesh / Gremlin）
- [ ] 可观测性三支柱已就绪（监控/日志/追踪）
- [ ] 实验卡片已填写并审批
- [ ] 稳态假设已明确且有量化指标
- [ ] 爆炸半径已控制（环境/命名空间/Pod 数量）
- [ ] Kill Switch 已测试可用
- [ ] 中止条件已明确定义
- [ ] 参与人员全部就位
- [ ] 监控大盘已打开并记录基线
- [ ] 实验按计划执行
- [ ] 实验期间持续监控关键指标
- [ ] 实验结束后系统恢复到稳态
- [ ] 实验结果已记录（稳态假设验证结果/时间线/发现）
- [ ] 改进项已创建并指定负责人和截止日期
- [ ] 韧性评分已计算
- [ ] 实验报告已归档