# -*- coding: utf-8 -*-
"""
Super Dev CLI 集成测试
"""

import os
import pytest
import yaml
from pathlib import Path
from super_dev.cli import SuperDevCLI


class TestCLIInit:
    """测试 init 命令"""

    def test_init_creates_config(self, temp_project_dir: Path):
        """测试 init 命令创建配置文件"""
        # 切换到临时目录
        original_cwd = os.getcwd()
        os.chdir(temp_project_dir)

        try:
            cli = SuperDevCLI()
            result = cli.run([
                "init", "test-project",
                "-p", "web",
                "-f", "next",
                "-b", "node"
            ])

            assert result == 0
            config_path = temp_project_dir / "super-dev.yaml"
            assert config_path.exists()

            with open(config_path, "r") as f:
                config = yaml.safe_load(f)
                assert config["name"] == "test-project"
                assert config["platform"] == "web"
        finally:
            os.chdir(original_cwd)

    def test_init_already_initialized(self, temp_project_dir: Path):
        """测试重复初始化"""
        # 切换到临时目录
        original_cwd = os.getcwd()
        os.chdir(temp_project_dir)

        try:
            # 创建现有配置
            (Path.cwd() / "super-dev.yaml").write_text("name: existing")

            cli = SuperDevCLI()
            result = cli.run(["init", "another-project"])

            assert result == 0  # 应该优雅处理
        finally:
            os.chdir(original_cwd)


class TestCLIConfig:
    """测试 config 命令"""

    def test_config_list(self, temp_project_dir: Path, sample_config):
        """测试列出配置"""
        # 切换到临时目录
        original_cwd = os.getcwd()
        os.chdir(temp_project_dir)

        try:
            # 创建配置文件 (使用当前目录的 super-dev.yaml)
            config_data = {
                "name": sample_config.name,
                "platform": sample_config.platform,
                "frontend": sample_config.frontend,
                "backend": sample_config.backend
            }
            config_path = Path.cwd() / "super-dev.yaml"
            with open(config_path, "w") as f:
                yaml.dump(config_data, f)

            cli = SuperDevCLI()
            result = cli.run(["config", "list"])

            assert result == 0
        finally:
            os.chdir(original_cwd)

    def test_config_get(self, temp_project_dir: Path, sample_config):
        """测试获取配置"""
        # 切换到临时目录
        original_cwd = os.getcwd()
        os.chdir(temp_project_dir)

        try:
            config_data = {
                "name": sample_config.name,
                "quality_gate": 85
            }
            config_path = Path.cwd() / "super-dev.yaml"
            with open(config_path, "w") as f:
                yaml.dump(config_data, f)

            cli = SuperDevCLI()
            result = cli.run(["config", "get", "quality_gate"])

            assert result == 0
        finally:
            os.chdir(original_cwd)

    def test_config_set(self, temp_project_dir: Path):
        """测试设置配置"""
        # 切换到临时目录
        original_cwd = os.getcwd()
        os.chdir(temp_project_dir)

        try:
            config_data = {"name": "test"}
            config_path = Path.cwd() / "super-dev.yaml"
            with open(config_path, "w") as f:
                yaml.dump(config_data, f)

            cli = SuperDevCLI()
            result = cli.run(["config", "set", "quality_gate", "90"])

            assert result == 0

            # 验证配置已更新
            with open(config_path, "r") as f:
                config = yaml.safe_load(f)
                assert config["quality_gate"] == 90
        finally:
            os.chdir(original_cwd)


class TestCLIExpert:
    """测试 expert 命令"""

    def test_expert_pm(self):
        """测试调用 PM 专家"""
        cli = SuperDevCLI()
        result = cli.run(["expert", "PM", "帮我分析需求"])

        assert result == 0

    def test_expert_architect(self):
        """测试调用架构师专家"""
        cli = SuperDevCLI()
        result = cli.run(["expert", "ARCHITECT", "设计系统架构"])

        assert result == 0


class TestCLIPreview:
    """测试 preview 命令"""

    def test_preview_generation(self, temp_project_dir: Path):
        """测试原型生成"""
        cli = SuperDevCLI()
        result = cli.run([
            "preview",
            "-o", str(temp_project_dir / "preview.html")
        ])

        assert result == 0


class TestCLIDeploy:
    """测试 deploy 命令"""

    def test_deploy_docker(self, temp_project_dir: Path):
        """测试生成 Dockerfile"""
        cli = SuperDevCLI()
        result = cli.run(["deploy", "--docker"])

        assert result == 0

    def test_deploy_cicd_github(self):
        """测试生成 GitHub Actions 配置"""
        cli = SuperDevCLI()
        result = cli.run(["deploy", "--cicd", "github"])

        assert result == 0

    def test_deploy_cicd_gitlab(self):
        """测试生成 GitLab CI 配置"""
        cli = SuperDevCLI()
        result = cli.run(["deploy", "--cicd", "gitlab"])

        assert result == 0
