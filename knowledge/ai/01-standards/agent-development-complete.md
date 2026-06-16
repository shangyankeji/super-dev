---
id: agent-development-complete
title: Agent 开发完整指南
domain: ai
category: 01-standards
difficulty: intermediate
tags: [agent, ai, complete, development, memory, multi-agent, react, tool]
quality_score: 70
last_updated: 2026-06-15
---
# Agent 开发完整指南

## 概述

AI Agent 是能自主规划、使用工具并完成复杂任务的 LLM 应用。本指南覆盖 ReAct 框架、Tool Use、Multi-Agent 协作、Memory 管理、Planning 策略及安全防护，适用于构建生产级 Agent 系统。

### Agent 系统架构

```
Agent 核心架构:
├── 感知层 (Perception)
│   ├── 用户输入解析
│   ├── 环境状态读取
│   └── 工具输出解析
├── 推理层 (Reasoning)
│   ├── 任务规划 (Planning)
│   ├── 决策 (Decision Making)
│   └── 反思 (Reflection)
├── 行动层 (Action)
│   ├── 工具调用 (Tool Use)
│   ├── 代码执行
│   └── API 调用
├── 记忆层 (Memory)
│   ├── 短期记忆 (对话上下文)
│   ├── 工作记忆 (当前任务状态)
│   └── 长期记忆 (知识库/向量存储)
└── 安全层 (Safety)
    ├── 权限管控
    ├── 操作审计
    └── 熔断降级
```

---

## 1. ReAct 框架

### 1.1 ReAct 核心循环

```
ReAct 循环: Thought -> Action -> Observation -> Thought -> ...

Thought: 分析当前状态，决定下一步行动
Action: 选择并执行工具调用
Observation: 获取工具输出结果
Repeat: 直到任务完成或达到最大步数
```

### 1.2 ReAct 实现

```python
from anthropic import Anthropic
import json

class ReActAgent:
    """基于 ReAct 框架的 Agent 实现。"""

    def __init__(self, tools: dict, max_steps: int = 10):
        self.client = Anthropic()
        self.tools = tools
        self.max_steps = max_steps
        self.trajectory: list[dict] = []

    def run(self, task: str) -> str:
        """执行 ReAct 循环直到任务完成。"""
        messages = [{"role": "user", "content": task}]

        tool_definitions = [
            {
                "name": name,
                "description": tool["description"],
                "input_schema": tool["schema"],
            }
            for name, tool in self.tools.items()
        ]

        for step in range(self.max_steps):
            response = self.client.messages.create(
                model="claude-sonnet-4-5-20250929",
                max_tokens=4096,
                system=self._system_prompt(),
                tools=tool_definitions,
                messages=messages,
            )

            # 记录轨迹
            self.trajectory.append({
                "step": step,
                "stop_reason": response.stop_reason,
                "content": [b.model_dump() for b in response.content],
            })

            # 任务完成
            if response.stop_reason == "end_turn":
                return self._extract_final_answer(response)

            # 处理工具调用
            if response.stop_reason == "tool_use":
                messages.append({"role": "assistant", "content": response.content})
                tool_results = self._execute_tools(response.content)
                messages.append({"role": "user", "content": tool_results})

        return "达到最大步数限制，任务未完成"

    def _execute_tools(self, content) -> list[dict]:
        """执行所有工具调用并返回结果。"""
        results = []
        for block in content:
            if block.type == "tool_use":
                tool_name = block.name
                tool_input = block.input
                try:
                    output = self.tools[tool_name]["function"](**tool_input)
                    results.append({
                        "type": "tool_result",
                        "tool_use_id": block.id,
                        "content": str(output),
                    })
                except Exception as e:
                    results.append({
                        "type": "tool_result",
                        "tool_use_id": block.id,
                        "content": f"错误: {e}",
                        "is_error": True,
                    })
        return results

    def _system_prompt(self) -> str:
        return """你是一个能使用工具完成任务的 Agent。

规则:
1. 每次行动前先思考 (Thought) 再决定行动 (Action)
2. 仔细观察工具输出 (Observation) 再决定下一步
3. 不确定时优先请求更多信息
4. 任务完成时给出明确的最终答案
5. 遇到错误时分析原因并尝试替代方案"""

    def _extract_final_answer(self, response) -> str:
        for block in response.content:
            if hasattr(block, "text"):
                return block.text
        return ""
```

---

## 2. Tool Use

### 2.1 工具定义规范

```python
TOOL_REGISTRY = {
    "search_codebase": {
        "description": "在代码库中搜索文件或代码片段",
        "schema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "搜索关键词或正则表达式",
                },
                "file_pattern": {
                    "type": "string",
                    "description": "文件名 glob 模式，如 '*.py'",
                },
                "max_results": {
                    "type": "integer",
                    "description": "最大返回结果数",
                    "default": 10,
                },
            },
            "required": ["query"],
        },
        "function": search_codebase_impl,
        "permissions": ["read"],
        "risk_level": "low",
    },
    "execute_command": {
        "description": "执行 Shell 命令",
        "schema": {
            "type": "object",
            "properties": {
                "command": {"type": "string", "description": "要执行的命令"},
                "timeout": {"type": "integer", "default": 30},
            },
            "required": ["command"],
        },
        "function": execute_command_impl,
        "permissions": ["read", "execute"],
        "risk_level": "high",
        "requires_confirmation": True,
    },
}
```

### 2.2 工具安全封装

```python
class SafeToolExecutor:
    """带权限控制和审计日志的工具执行器。"""

    BLOCKED_COMMANDS = [
        "rm -rf /", "mkfs", "dd if=", "> /dev/",
        "chmod 777", "curl | bash", "wget | sh",
    ]

    def __init__(self, allowed_permissions: set[str],
                 audit_logger=None):
        self.permissions = allowed_permissions
        self.audit = audit_logger

    def execute(self, tool_name: str, tool_def: dict,
                inputs: dict, context: dict) -> dict:
        """安全执行工具调用。"""
        # 权限检查
        required = set(tool_def.get("permissions", []))
        if not required.issubset(self.permissions):
            return {
                "error": f"权限不足: 需要 {required - self.permissions}",
                "blocked": True,
            }

        # 危险命令检查
        if tool_name == "execute_command":
            cmd = inputs.get("command", "")
            for pattern in self.BLOCKED_COMMANDS:
                if pattern in cmd:
                    return {"error": f"命令被拦截: 匹配危险模式 '{pattern}'"}

        # 高风险操作需要确认
        if tool_def.get("requires_confirmation"):
            if not context.get("user_confirmed"):
                return {
                    "needs_confirmation": True,
                    "message": f"高风险操作: {tool_name}({inputs})，请确认执行",
                }

        # 执行并记录审计日志
        try:
            result = tool_def["function"](**inputs)
            if self.audit:
                self.audit.log(
                    tool=tool_name, inputs=inputs,
                    result="success", context=context,
                )
            return {"result": result}
        except Exception as e:
            if self.audit:
                self.audit.log(
                    tool=tool_name, inputs=inputs,
                    result=f"error: {e}", context=context,
                )
            return {"error": str(e)}
```

---

## 3. Multi-Agent 协作

### 3.1 协作模式

```
Multi-Agent 模式:
├── 顺序流水线 — Agent A 输出交给 Agent B 处理
├── 并行扇出 — 多个 Agent 同时处理不同子任务
├── 辩论对抗 — Agent 互相评审产出共识
├── 层级委派 — 主 Agent 分配子任务给专家 Agent
└── 黑板模型 — 所有 Agent 共享状态黑板协作
```

### 3.2 层级委派实现

```python
class OrchestratorAgent:
    """主控 Agent: 分析任务、分配给专家 Agent、综合结果。"""

    def __init__(self):
        self.experts: dict[str, ReActAgent] = {}

    def register_expert(self, role: str, agent: ReActAgent):
        self.experts[role] = agent

    def execute(self, task: str) -> dict:
        # 步骤 1: 任务分解
        subtasks = self._decompose(task)

        # 步骤 2: 分配并执行
        results = {}
        for subtask in subtasks:
            role = subtask["assigned_to"]
            if role in self.experts:
                results[role] = self.experts[role].run(subtask["description"])

        # 步骤 3: 综合结果
        synthesis = self._synthesize(task, results)
        return {"subtasks": subtasks, "results": results, "synthesis": synthesis}

    def _decompose(self, task: str) -> list[dict]:
        """使用 LLM 将任务分解为子任务并分配角色。"""
        prompt = f"""将以下任务分解为子任务，并分配给合适的专家:

可用专家: {list(self.experts.keys())}

任务: {task}

输出 JSON:
```json
[
  {{"description": "子任务描述", "assigned_to": "专家角色", "priority": 1}}
]
```"""
        response = client.messages.create(
            model="claude-sonnet-4-5-20250929",
            messages=[{"role": "user", "content": prompt}],
            max_tokens=2048,
        )
        return json.loads(extract_json(response.content[0].text))

    def _synthesize(self, task: str, results: dict) -> str:
        prompt = f"""综合以下专家的分析结果，给出最终结论。

原始任务: {task}

专家结果:
{json.dumps(results, ensure_ascii=False, indent=2)}

请给出:
1. 综合结论
2. 各专家结论的共识点
3. 存在分歧的点及推荐意见"""
        response = client.messages.create(
            model="claude-sonnet-4-5-20250929",
            messages=[{"role": "user", "content": prompt}],
            max_tokens=4096,
        )
        return response.content[0].text
```

### 3.3 Agent 间通信协议

```python
from dataclasses import dataclass, field
from enum import Enum
from datetime import datetime

class MessageType(Enum):
    TASK = "task"
    RESULT = "result"
    QUERY = "query"
    FEEDBACK = "feedback"
    STATUS = "status"

@dataclass
class AgentMessage:
    """Agent 间标准消息格式。"""
    sender: str
    receiver: str
    msg_type: MessageType
    content: dict
    correlation_id: str
    timestamp: datetime = field(default_factory=datetime.utcnow)
    priority: int = 0

class MessageBus:
    """Agent 消息总线。"""

    def __init__(self):
        self._queues: dict[str, list[AgentMessage]] = {}
        self._handlers: dict[str, callable] = {}

    def register(self, agent_id: str, handler: callable):
        self._queues[agent_id] = []
        self._handlers[agent_id] = handler

    def send(self, message: AgentMessage):
        receiver = message.receiver
        if receiver in self._queues:
            self._queues[receiver].append(message)
            self._handlers[receiver](message)

    def broadcast(self, sender: str, msg_type: MessageType, content: dict):
        for agent_id in self._queues:
            if agent_id != sender:
                self.send(AgentMessage(
                    sender=sender, receiver=agent_id,
                    msg_type=msg_type, content=content,
                    correlation_id=str(uuid.uuid4()),
                ))
```

---

## 4. Memory 管理

### 4.1 三级记忆架构

```python
class AgentMemory:
    """三级记忆系统: 短期 + 工作 + 长期。"""

    def __init__(self, vector_store, max_short_term: int = 20):
        self.short_term: list[dict] = []  # 最近对话轮次
        self.working: dict = {}           # 当前任务状态
        self.vector_store = vector_store  # 长期向量记忆
        self.max_short_term = max_short_term

    def add_interaction(self, role: str, content: str):
        """添加到短期记忆，超出限制时压缩转存长期记忆。"""
        self.short_term.append({"role": role, "content": content})
        if len(self.short_term) > self.max_short_term:
            self._compress_and_archive()

    def update_working(self, key: str, value):
        """更新工作记忆 (当前任务状态)。"""
        self.working[key] = value

    def recall(self, query: str, top_k: int = 5) -> list[dict]:
        """从长期记忆中检索相关信息。"""
        return self.vector_store.search(
            collection="agent_memory",
            query_vector=get_embedding(query),
            top_k=top_k,
        )

    def _compress_and_archive(self):
        """压缩早期对话并存入长期记忆。"""
        to_archive = self.short_term[:10]
        self.short_term = self.short_term[10:]

        summary = self._summarize(to_archive)
        self.vector_store.upsert("agent_memory", [{
            "text": summary,
            "embedding": get_embedding(summary),
            "metadata": {"type": "conversation_summary",
                         "timestamp": datetime.utcnow().isoformat()},
        }])

    def _summarize(self, messages: list[dict]) -> str:
        text = "\n".join(f"{m['role']}: {m['content']}" for m in messages)
        response = client.messages.create(
            model="claude-sonnet-4-5-20250929",
            messages=[{"role": "user",
                       "content": f"简洁总结以下对话的关键信息:\n{text}"}],
            max_tokens=512,
        )
        return response.content[0].text

    def get_context(self, query: str) -> str:
        """构建完整上下文: 工作记忆 + 短期记忆 + 长期检索。"""
        parts = []

        if self.working:
            parts.append(f"当前任务状态:\n{json.dumps(self.working, ensure_ascii=False)}")

        if self.short_term:
            recent = self.short_term[-5:]
            parts.append("最近对话:\n" + "\n".join(
                f"{m['role']}: {m['content'][:200]}" for m in recent
            ))

        long_term = self.recall(query, top_k=3)
        if long_term:
            parts.append("相关历史:\n" + "\n".join(
                d["text"][:200] for d in long_term
            ))

        return "\n\n---\n\n".join(parts)
```

---

## 5. Planning 策略

### 5.1 Plan-and-Execute

```python
class PlanAndExecuteAgent:
    """先规划再执行的 Agent 模式。"""

    def __init__(self, executor: ReActAgent):
        self.executor = executor

    def run(self, task: str) -> dict:
        # 阶段 1: 生成计划
        plan = self._create_plan(task)

        # 阶段 2: 逐步执行
        results = []
        for i, step in enumerate(plan["steps"]):
            result = self.executor.run(step["action"])
            results.append({"step": step, "result": result})

            # 阶段 3: 执行后反思并调整计划
            if self._needs_replan(step, result, plan["steps"][i+1:]):
                remaining = self._replan(task, results, plan["steps"][i+1:])
                plan["steps"] = plan["steps"][:i+1] + remaining

        return {"plan": plan, "results": results}

    def _create_plan(self, task: str) -> dict:
        prompt = f"""为以下任务创建详细的执行计划。

任务: {task}

输出 JSON:
```json
{{
  "goal": "任务目标",
  "steps": [
    {{"id": 1, "action": "具体步骤", "expected_output": "预期结果", "dependencies": []}}
  ],
  "success_criteria": ["完成标准"]
}}
```"""
        response = client.messages.create(
            model="claude-sonnet-4-5-20250929",
            messages=[{"role": "user", "content": prompt}],
            max_tokens=2048,
        )
        return json.loads(extract_json(response.content[0].text))

    def _needs_replan(self, step: dict, result: str,
                      remaining: list) -> bool:
        """判断是否需要调整后续计划。"""
        prompt = f"""评估执行结果是否需要调整后续计划。

已执行步骤: {step['action']}
实际结果: {result}
预期结果: {step['expected_output']}
剩余步骤: {json.dumps(remaining, ensure_ascii=False)}

只回答 YES 或 NO。"""
        response = client.messages.create(
            model="claude-sonnet-4-5-20250929",
            messages=[{"role": "user", "content": prompt}],
            max_tokens=10,
        )
        return "YES" in response.content[0].text.upper()

    def _replan(self, task: str, completed: list, remaining: list) -> list:
        """根据已完成结果重新规划剩余步骤。"""
        prompt = f"""根据已完成的步骤和结果，调整剩余计划。

任务: {task}
已完成: {json.dumps(completed, ensure_ascii=False)}
原剩余计划: {json.dumps(remaining, ensure_ascii=False)}

输出调整后的剩余步骤 (JSON 数组)。"""
        response = client.messages.create(
            model="claude-sonnet-4-5-20250929",
            messages=[{"role": "user", "content": prompt}],
            max_tokens=2048,
        )
        return json.loads(extract_json(response.content[0].text))
```

---

## 6. 安全防护

### 6.1 权限与沙箱

```python
class AgentSandbox:
    """Agent 执行沙箱: 限制文件访问、网络和资源。"""

    def __init__(self, config: dict):
        self.allowed_paths: list[str] = config.get("allowed_paths", [])
        self.allowed_domains: list[str] = config.get("allowed_domains", [])
        self.max_execution_time: int = config.get("max_execution_time", 300)
        self.max_memory_mb: int = config.get("max_memory_mb", 512)
        self.max_tool_calls: int = config.get("max_tool_calls", 50)
        self._tool_call_count = 0

    def check_file_access(self, path: str) -> bool:
        """检查文件路径是否在白名单内。"""
        from pathlib import Path
        resolved = str(Path(path).resolve())
        return any(resolved.startswith(p) for p in self.allowed_paths)

    def check_network_access(self, url: str) -> bool:
        """检查 URL 域名是否在白名单内。"""
        from urllib.parse import urlparse
        domain = urlparse(url).hostname or ""
        return any(domain.endswith(d) for d in self.allowed_domains)

    def check_tool_limit(self) -> bool:
        """检查工具调用次数是否超限。"""
        self._tool_call_count += 1
        return self._tool_call_count <= self.max_tool_calls
```

### 6.2 操作审计

```python
class AgentAuditLog:
    """Agent 操作审计日志。"""

    def __init__(self, storage):
        self.storage = storage

    def log(self, **kwargs):
        entry = {
            "timestamp": datetime.utcnow().isoformat(),
            "agent_id": kwargs.get("agent_id"),
            "action": kwargs.get("tool"),
            "inputs_summary": self._redact(kwargs.get("inputs", {})),
            "result_status": kwargs.get("result", "unknown"),
            "session_id": kwargs.get("context", {}).get("session_id"),
        }
        self.storage.insert(entry)

    def _redact(self, inputs: dict) -> dict:
        """脱敏: 移除密码、密钥等敏感字段。"""
        sensitive_keys = {"password", "token", "secret", "api_key", "credential"}
        return {
            k: "***REDACTED***" if k.lower() in sensitive_keys else v
            for k, v in inputs.items()
        }
```

### 6.3 熔断降级

```python
class AgentCircuitBreaker:
    """Agent 熔断器: 连续失败时自动降级。"""

    def __init__(self, failure_threshold: int = 3,
                 recovery_timeout: int = 60):
        self.failure_threshold = failure_threshold
        self.recovery_timeout = recovery_timeout
        self.failure_count = 0
        self.state = "closed"  # closed / open / half-open
        self.last_failure_time = None

    def call(self, func, *args, **kwargs):
        if self.state == "open":
            if self._should_attempt_recovery():
                self.state = "half-open"
            else:
                raise RuntimeError("Agent 熔断中，请稍后重试")

        try:
            result = func(*args, **kwargs)
            self._on_success()
            return result
        except Exception as e:
            self._on_failure()
            raise

    def _on_success(self):
        self.failure_count = 0
        self.state = "closed"

    def _on_failure(self):
        self.failure_count += 1
        self.last_failure_time = time.monotonic()
        if self.failure_count >= self.failure_threshold:
            self.state = "open"

    def _should_attempt_recovery(self) -> bool:
        if self.last_failure_time is None:
            return True
        return (time.monotonic() - self.last_failure_time) > self.recovery_timeout
```

---

## Agent Checklist

- [ ] Agent 使用 ReAct 框架，每步有显式 Thought-Action-Observation 记录
- [ ] 工具定义包含 description、schema、permissions 和 risk_level
- [ ] 高风险工具 (写入/删除/执行) 有二次确认和审计日志
- [ ] 危险命令有黑名单拦截，文件操作有路径白名单
- [ ] Multi-Agent 有明确的通信协议和角色分工
- [ ] Memory 系统实现短期/工作/长期三级架构
- [ ] 长期记忆有压缩归档策略，避免 Token 溢出
- [ ] Planning 支持执行后反思和动态重新规划
- [ ] 沙箱限制文件访问、网络访问和资源使用
- [ ] 工具调用次数有上限，连续失败有熔断降级
- [ ] 所有操作有审计日志，敏感信息自动脱敏
- [ ] Agent 轨迹 (trajectory) 完整保存，可回溯调试
