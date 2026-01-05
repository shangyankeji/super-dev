# -*- coding: utf-8 -*-
"""
Super Dev 配置管理单元测试
"""

import pytest
import yaml
from pathlib import Path

from super_dev.config import ConfigManager, ProjectConfig, get_config_manager


class TestProjectConfig:
    """测试 ProjectConfig"""

    def test_default_config(self):
        """测试默认配置"""
        config = ProjectConfig(name="test")

        assert config.name == "test"
        assert config.platform == "web"
        assert config.frontend == "next"
        assert config.backend == "node"
        assert config.quality_gate == 80

    def test_config_with_custom_values(self):
        """测试自定义配置"""
        config = ProjectConfig(
            name="custom",
            platform="mobile",
            frontend="vue",
            domain="fintech",
            quality_gate=90
        )

        assert config.name == "custom"
        assert config.platform == "mobile"
        assert config.frontend == "vue"
        assert config.domain == "fintech"
        assert config.quality_gate == 90


class TestConfigManager:
    """测试 ConfigManager"""

    def test_init_without_config(self, temp_project_dir: Path):
        """测试无配置文件时初始化"""
        manager = ConfigManager(temp_project_dir)

        assert not manager.exists()
        assert manager.config.name == "my-project"

    def test_load_from_file(self, temp_project_dir: Path):
        """测试从文件加载配置"""
        config_data = {
            "name": "loaded-project",
            "platform": "desktop",
            "quality_gate": 85
        }
        config_path = temp_project_dir / "super-dev.yaml"
        with open(config_path, "w") as f:
            yaml.dump(config_data, f)

        manager = ConfigManager(temp_project_dir)
        config = manager.config

        assert config.name == "loaded-project"
        assert config.platform == "desktop"
        assert config.quality_gate == 85

    def test_save_config(self, temp_project_dir: Path):
        """测试保存配置"""
        manager = ConfigManager(temp_project_dir)
        config = ProjectConfig(name="saved-project")
        manager.save(config)

        config_path = temp_project_dir / "super-dev.yaml"
        assert config_path.exists()

        with open(config_path, "r") as f:
            data = yaml.safe_load(f)
            assert data["name"] == "saved-project"

    def test_create_config(self, temp_project_dir: Path):
        """测试创建新配置"""
        manager = ConfigManager(temp_project_dir)
        config = manager.create(
            name="new-project",
            platform="wechat",
            domain="ecommerce"
        )

        assert config.name == "new-project"
        assert config.platform == "wechat"
        assert config.domain == "ecommerce"
        assert manager.exists()

    def test_update_config(self, temp_project_dir: Path):
        """测试更新配置"""
        manager = ConfigManager(temp_project_dir)
        manager.create(name="original")

        updated = manager.update(
            description="Updated description",
            quality_gate=90
        )

        assert updated.description == "Updated description"
        assert updated.quality_gate == 90

    def test_get_config_value(self, temp_project_dir: Path):
        """测试获取配置值"""
        manager = ConfigManager(temp_project_dir)
        manager.create(name="test", quality_gate=85)

        assert manager.get("name") == "test"
        assert manager.get("quality_gate") == 85
        assert manager.get("nonexistent", "default") == "default"

    def test_validate_valid_config(self, temp_project_dir: Path):
        """测试验证有效配置"""
        manager = ConfigManager(temp_project_dir)
        manager.create(
            name="valid",
            platform="web",
            frontend="react",
            backend="node",
            quality_gate=85
        )

        is_valid, errors = manager.validate()
        assert is_valid
        assert len(errors) == 0

    def test_validate_invalid_platform(self, temp_project_dir: Path):
        """测试验证无效平台"""
        manager = ConfigManager(temp_project_dir)
        manager.create(name="invalid")
        manager._config.platform = "invalid_platform"

        is_valid, errors = manager.validate()
        assert not is_valid
        assert len(errors) > 0

    def test_validate_invalid_quality_gate(self, temp_project_dir: Path):
        """测试验证无效质量门禁"""
        manager = ConfigManager(temp_project_dir)
        manager.create(name="invalid")
        manager._config.quality_gate = 150

        is_valid, errors = manager.validate()
        assert not is_valid


class TestGlobalConfigManager:
    """测试全局配置管理器"""

    def test_get_config_manager_singleton(self, temp_project_dir: Path):
        """测试单例模式"""
        manager1 = get_config_manager(temp_project_dir)
        manager2 = get_config_manager(temp_project_dir)

        assert manager1 is manager2
