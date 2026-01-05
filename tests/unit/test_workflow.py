# -*- coding: utf-8 -*-
"""
Super Dev 工作流引擎单元测试
"""

import pytest
from pathlib import Path

from super_dev.orchestrator import (
    WorkflowEngine,
    Phase,
    PhaseResult,
    WorkflowContext
)
from super_dev.config import ConfigManager, ProjectConfig


class TestPhase:
    """测试 Phase 枚举"""

    def test_phase_values(self):
        """测试阶段值"""
        assert Phase.DISCOVERY.value == "discovery"
        assert Phase.INTELLIGENCE.value == "intelligence"
        assert Phase.DRAFTING.value == "drafting"
        assert Phase.REDTEAM.value == "redteam"
        assert Phase.QA.value == "qa"
        assert Phase.DELIVERY.value == "delivery"
        assert Phase.DEPLOYMENT.value == "deployment"


class TestPhaseResult:
    """测试 PhaseResult"""

    def test_successful_result(self):
        """测试成功结果"""
        result = PhaseResult(
            phase=Phase.DISCOVERY,
            success=True,
            duration=1.5,
            quality_score=85.0,
            output={"data": "test"}
        )

        assert result.phase == Phase.DISCOVERY
        assert result.success
        assert result.duration == 1.5
        assert result.quality_score == 85.0
        assert len(result.errors) == 0

    def test_failed_result(self):
        """测试失败结果"""
        result = PhaseResult(
            phase=Phase.INTELLIGENCE,
            success=False,
            duration=0.5,
            errors=["Error 1", "Error 2"]
        )

        assert not result.success
        assert len(result.errors) == 2


class TestWorkflowContext:
    """测试 WorkflowContext"""

    def test_context_creation(self, temp_project_dir: Path):
        """测试上下文创建"""
        config = ProjectConfig(name="test")
        context = WorkflowContext(
            project_dir=temp_project_dir,
            config=config  # type: ignore
        )

        assert context.project_dir == temp_project_dir
        assert context.config.name == "test"
        assert len(context.results) == 0
        assert len(context.metadata) == 0

    def test_context_shared_data(self, temp_project_dir: Path):
        """测试共享数据"""
        config = ProjectConfig(name="test")
        context = WorkflowContext(
            project_dir=temp_project_dir,
            config=config  # type: ignore
        )

        context.user_input["requirement"] = "test input"
        context.research_data["market"] = "test data"

        assert context.user_input["requirement"] == "test input"
        assert context.research_data["market"] == "test data"


class TestWorkflowEngine:
    """测试 WorkflowEngine"""

    def test_engine_initialization(self, temp_project_dir: Path):
        """测试引擎初始化"""
        engine = WorkflowEngine(temp_project_dir)

        assert engine.project_dir == temp_project_dir
        assert engine.config_manager is not None

    def test_get_phases_from_config(self, temp_project_dir: Path):
        """测试从配置获取阶段"""
        config_manager = ConfigManager(temp_project_dir)
        config_manager.create(
            name="test",
            phases=["discovery", "intelligence", "drafting"]
        )

        engine = WorkflowEngine(temp_project_dir)
        phases = engine._get_phases_from_config()

        # 获取阶段时会合并默认配置
        assert len(phases) >= 3
        assert Phase.DISCOVERY in phases
        assert Phase.INTELLIGENCE in phases
        assert Phase.DRAFTING in phases

    def test_register_custom_handler(self, temp_project_dir: Path):
        """测试注册自定义处理器"""
        engine = WorkflowEngine(temp_project_dir)

        async def custom_handler(context):
            return {"custom": "result"}

        engine.register_phase_handler(Phase.DISCOVERY, custom_handler)
        assert Phase.DISCOVERY in engine._phase_handlers
        assert engine._phase_handlers[Phase.DISCOVERY] == custom_handler

    @pytest.mark.asyncio
    async def test_run_single_phase(self, temp_project_dir: Path, workflow_context):
        """测试运行单个阶段"""
        engine = WorkflowEngine(temp_project_dir)

        # 注册测试处理器
        async def test_handler(context):
            return {"test": "data"}

        engine.register_phase_handler(Phase.DISCOVERY, test_handler)

        results = await engine.run(phases=[Phase.DISCOVERY], context=workflow_context)

        assert Phase.DISCOVERY in results
        assert results[Phase.DISCOVERY].success
        assert results[Phase.DISCOVERY].output == {"test": "data"}

    @pytest.mark.asyncio
    async def test_run_multiple_phases(self, temp_project_dir: Path, workflow_context):
        """测试运行多个阶段"""
        # 确保使用默认质量门禁 (80)，避免被其他测试影响
        config_manager = ConfigManager(temp_project_dir)
        config_manager.create(name="test", quality_gate=80)

        engine = WorkflowEngine(temp_project_dir)

        # 注册测试处理器
        async def handler(context):
            return {"handled": True}

        for phase in [Phase.DISCOVERY, Phase.INTELLIGENCE]:
            engine.register_phase_handler(phase, handler)

        results = await engine.run(
            phases=[Phase.DISCOVERY, Phase.INTELLIGENCE],
            context=workflow_context
        )

        assert len(results) == 2
        assert all(r.success for r in results.values())

    @pytest.mark.asyncio
    async def test_phase_failure_stops_workflow(self, temp_project_dir: Path, workflow_context):
        """测试阶段失败停止工作流"""
        # 确保使用默认质量门禁 (80)
        config_manager = ConfigManager(temp_project_dir)
        config_manager.create(name="test", quality_gate=80)

        engine = WorkflowEngine(temp_project_dir)

        # 第一个阶段成功
        async def success_handler(context):
            return {"status": "ok"}

        # 第二个阶段失败
        async def fail_handler(context):
            raise Exception("Test failure")

        engine.register_phase_handler(Phase.DISCOVERY, success_handler)
        engine.register_phase_handler(Phase.INTELLIGENCE, fail_handler)

        results = await engine.run(
            phases=[Phase.DISCOVERY, Phase.INTELLIGENCE, Phase.DRAFTING],
            context=workflow_context
        )

        assert results[Phase.DISCOVERY].success
        assert not results[Phase.INTELLIGENCE].success
        # 第三阶段不应该执行
        assert Phase.DRAFTING not in results

    @pytest.mark.asyncio
    async def test_quality_gate_check(self, temp_project_dir: Path):
        """测试质量门禁检查"""
        config_manager = ConfigManager(temp_project_dir)
        config_manager.create(name="test", quality_gate=90)

        engine = WorkflowEngine(temp_project_dir)
        context = WorkflowContext(
            project_dir=temp_project_dir,
            config=config_manager
        )

        # 返回低分的处理器
        async def low_score_handler(context):
            from super_dev.orchestrator.engine import PhaseResult
            return PhaseResult(
                phase=Phase.DISCOVERY,
                success=True,
                duration=1.0,
                quality_score=75.0,  # 低于门禁
                output={"status": "low_score"}
            )

        # 正常处理器
        async def normal_handler(context):
            from super_dev.orchestrator.engine import PhaseResult
            return PhaseResult(
                phase=Phase.INTELLIGENCE,
                success=True,
                duration=1.0,
                quality_score=85.0,
                output={"status": "normal"}
            )

        engine.register_phase_handler(Phase.DISCOVERY, low_score_handler)
        engine.register_phase_handler(Phase.INTELLIGENCE, normal_handler)

        # Mock calculate_quality_score to return low score for discovery
        original_calculate = engine._calculate_quality_score
        def mock_calculate(phase, context):
            if phase == Phase.DISCOVERY:
                return 75.0
            return 85.0
        engine._calculate_quality_score = mock_calculate

        results = await engine.run(
            phases=[Phase.DISCOVERY, Phase.INTELLIGENCE],
            context=context
        )

        # 第一阶段应该完成但因质量分低导致停止
        assert results[Phase.DISCOVERY].success
        # 第二阶段不应执行（质量门禁停止）
        assert Phase.INTELLIGENCE not in results
        assert results[Phase.DISCOVERY].quality_score == 75.0
