# -*- coding: utf-8 -*-
"""
Super Dev 测试配置
"""

import pytest
import tempfile
import shutil
from pathlib import Path
from typing import Generator

from super_dev.config import ConfigManager, ProjectConfig
from super_dev.orchestrator import WorkflowEngine, WorkflowContext


@pytest.fixture(autouse=True)
def reset_global_config_manager():
    """重置全局配置管理器（每个测试前）"""
    from super_dev.config import manager
    manager._global_config_manager = None
    yield
    manager._global_config_manager = None


@pytest.fixture
def temp_project_dir() -> Generator[Path, None, None]:
    """临时项目目录"""
    temp_dir = Path(tempfile.mkdtemp())
    yield temp_dir
    shutil.rmtree(temp_dir, ignore_errors=True)


@pytest.fixture
def sample_config(temp_project_dir: Path) -> ProjectConfig:
    """示例配置"""
    return ProjectConfig(
        name="test-project",
        description="Test project",
        platform="web",
        frontend="react",
        backend="node",
        domain="ecommerce"
    )


@pytest.fixture
def config_manager(temp_project_dir: Path, sample_config: ProjectConfig) -> ConfigManager:
    """配置管理器"""
    manager = ConfigManager(temp_project_dir)
    manager._config = sample_config
    return manager


@pytest.fixture
def workflow_engine(temp_project_dir: Path) -> WorkflowEngine:
    """工作流引擎"""
    return WorkflowEngine(temp_project_dir)


@pytest.fixture
def workflow_context(temp_project_dir: Path, config_manager: ConfigManager) -> WorkflowContext:
    """工作流上下文"""
    return WorkflowContext(
        project_dir=temp_project_dir,
        config=config_manager
    )
