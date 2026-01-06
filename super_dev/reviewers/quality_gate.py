# -*- coding: utf-8 -*-
"""
质量门禁检查器 - 确保交付物达到质量标准

开发：Excellent（11964948@qq.com）
功能：多维度质量评分和门禁检查
作用：只有达到 80 分以上才能通过质量门禁
创建时间：2025-12-30
"""

from pathlib import Path
from typing import Optional
from dataclasses import dataclass, field
from enum import Enum


class CheckStatus(Enum):
    """检查状态"""
    PASSED = "passed"
    FAILED = "failed"
    WARNING = "warning"


@dataclass
class QualityCheck:
    """质量检查项"""
    name: str
    category: str  # documentation, security, performance, testing, code_quality
    description: str
    status: CheckStatus
    score: int  # 0-100
    weight: float = 1.0  # 权重，用于计算加权总分
    details: str = ""


@dataclass
class QualityGateResult:
    """质量门禁结果"""
    passed: bool
    total_score: int
    weighted_score: float
    checks: list[QualityCheck] = field(default_factory=list)
    critical_failures: list[str] = field(default_factory=list)
    recommendations: list[str] = field(default_factory=list)
    scenario: str = "1-N+1"  # 场景类型: "0-1" 或 "1-N+1"

    @property
    def passed_checks(self) -> list[QualityCheck]:
        return [c for c in self.checks if c.status == CheckStatus.PASSED]

    @property
    def failed_checks(self) -> list[QualityCheck]:
        return [c for c in self.checks if c.status == CheckStatus.FAILED]

    @property
    def warning_checks(self) -> list[QualityCheck]:
        return [c for c in self.checks if c.status == CheckStatus.WARNING]

    def to_markdown(self) -> str:
        """生成 Markdown 报告"""
        status_icon = "通过" if self.passed else "未通过"
        status_color = "green" if self.passed else "red"

        lines = [
            "# 质量门禁报告",
            "",
            f"**场景**: {self.scenario} ({'0-1 新建项目' if self.scenario == '0-1' else '1-N+1 增量开发'})",
            f"**状态**: <span style='color:{status_color}'>{status_icon}</span>",
            f"**总分**: {self.total_score}/100",
            f"**加权分**: {self.weighted_score:.1f}/100",
            "",
            "---",
            "",
            "## 检查结果摘要",
            "",
            f"- 通过: {len(self.passed_checks)} 项",
            f"- 警告: {len(self.warning_checks)} 项",
            f"- 失败: {len(self.failed_checks)} 项",
            "",
        ]

        # 如果是 0-1 场景，添加说明
        if self.scenario == "0-1":
            lines.extend([
                "**注意**: 当前为 0-1 场景（新建项目），质量门禁已自动放宽标准。",
                "在后续迭代中，随着代码和测试的完善，标准将逐步提高。",
                "",
            ])

        if self.critical_failures:
            lines.extend([
                "## 关键失败项",
                "",
            ])
            for failure in self.critical_failures:
                lines.append(f"- {failure}")
            lines.append("")

        # 按类别分组展示
        categories = {}
        for check in self.checks:
            if check.category not in categories:
                categories[check.category] = []
            categories[check.category].append(check)

        lines.extend([
            "## 详细检查结果",
            "",
        ])

        for category, checks in categories.items():
            lines.extend([
                f"### {category}",
                "",
                "| 检查项 | 状态 | 得分 | 说明 |",
                "|:---|:---:|:---:|:---|",
            ])

            for check in checks:
                status_icon = "✓" if check.status == CheckStatus.PASSED else "⚠" if check.status == CheckStatus.WARNING else "✗"
                lines.append(
                    f"| {check.name} | {status_icon} | {check.score}/100 | {check.description} |"
                )

            lines.append("")

        # 改进建议
        if self.recommendations:
            lines.extend([
                "## 改进建议",
                "",
            ])
            for idx, rec in enumerate(self.recommendations, 1):
                lines.append(f"{idx}. {rec}")
            lines.append("")

        # 下一步行动
        lines.extend([
            "---",
            "",
            "## 下一步行动",
            "",
        ])

        if self.passed:
            lines.extend([
                "[通过] 质量门禁已通过，可以继续下一步：",
                "",
                "1. 开始编码实现",
                "2. 设置 CI/CD 流水线",
                "3. 部署到测试环境",
                "",
            ])
        else:
            lines.extend([
                "[未通过] 质量门禁未通过，请完成以下操作后重新检查：",
                "",
            ])

            failed_items = [f"- {c.description}" for c in self.failed_checks]
            lines.extend(failed_items)
            lines.extend([
                "",
                "修复后运行: `super-dev quality-check`",
                "",
            ])

        return "\n".join(lines)


class QualityGateChecker:
    """质量门禁检查器"""

    # 质量门禁阈值
    PASS_THRESHOLD = 80
    WARNING_THRESHOLD = 60
    # 0-1 场景（空项目）的宽松阈值
    PASS_THRESHOLD_ZERO_TO_ONE = 50

    # 检查项配置
    CHECKS_CONFIG = {
        "documentation": {
            "weight": 1.0,
            "required": True,
        },
        "security": {
            "weight": 1.5,  # 安全更重要
            "required": True,
        },
        "performance": {
            "weight": 1.2,
            "required": True,
        },
        "testing": {
            "weight": 1.3,
            "required": True,
        },
        "code_quality": {
            "weight": 1.0,
            "required": False,
        },
    }

    def __init__(self, project_dir: Path, name: str, tech_stack: dict):
        self.project_dir = Path(project_dir).resolve()
        self.name = name
        self.tech_stack = tech_stack
        self.is_zero_to_one = self._detect_zero_to_one_scenario()

    def _detect_zero_to_one_scenario(self) -> bool:
        """
        检测是否为 0-1 场景（空项目/新建项目）

        0-1 场景特征：
        - 没有源代码目录（src/, lib/, app/, server/, client/ 等）
        - 没有配置文件（package.json, requirements.txt, go.mod 等）
        - 只有 output/ 目录（刚生成的文档）

        Returns:
            bool: True 表示 0-1 场景，False 表示 1-N+1 场景
        """
        # 检查常见的源代码目录
        source_dirs = [
            "src", "lib", "app", "server", "client",
            "backend", "frontend", "api", "handlers",
            "models", "views", "controllers", "services"
        ]

        has_source_code = any(
            (self.project_dir / d).exists()
            for d in source_dirs
        )

        # 检查是否有项目配置文件（表明这不是空项目）
        config_files = [
            "package.json", "requirements.txt", "go.mod",
            "Cargo.toml", "pom.xml", "build.gradle"
        ]

        has_project_config = any(
            (self.project_dir / f).exists()
            for f in config_files
        )

        # 如果有源代码或有项目配置，说明不是 0-1 场景
        return not (has_source_code or has_project_config)

    def check(self, redteam_report: Optional["RedTeamReport"] = None) -> QualityGateResult:
        """执行质量门禁检查"""
        checks = []
        recommendations = []

        # 1. 文档质量检查
        checks.extend(self._check_documentation())

        # 2. 安全检查 (基于红队报告)
        checks.extend(self._check_security(redteam_report))

        # 3. 性能检查 (基于红队报告)
        checks.extend(self._check_performance(redteam_report))

        # 4. 测试检查
        checks.extend(self._check_testing())

        # 5. 代码质量检查
        checks.extend(self._check_code_quality())

        # 计算总分和加权分
        total_score = self._calculate_total_score(checks)
        weighted_score = self._calculate_weighted_score(checks)

        # 根据场景选择阈值
        threshold = self.PASS_THRESHOLD_ZERO_TO_ONE if self.is_zero_to_one else self.PASS_THRESHOLD

        # 检查是否通过
        passed = total_score >= threshold

        # 收集关键失败项
        critical_failures = []
        for check in checks:
            config = self.CHECKS_CONFIG.get(check.category, {})
            if config.get("required", False) and check.status == CheckStatus.FAILED:
                critical_failures.append(f"[{check.category}] {check.description}")

        # 生成改进建议
        recommendations = self._generate_recommendations(checks)

        # 确定场景类型
        scenario = "0-1" if self.is_zero_to_one else "1-N+1"

        return QualityGateResult(
            passed=passed,
            total_score=total_score,
            weighted_score=weighted_score,
            checks=checks,
            critical_failures=critical_failures,
            recommendations=recommendations,
            scenario=scenario,
        )

    def _check_documentation(self) -> list[QualityCheck]:
        """检查文档质量"""
        checks = []

        # 检查 PRD 是否存在
        prd_path = self.project_dir / "output" / f"{self.name}-prd.md"
        if prd_path.exists():
            content = prd_path.read_text(encoding="utf-8")
            # 简单检查文档完整性
            has_vision = "产品愿景" in content or "vision" in content.lower()
            has_features = "功能需求" in content or "features" in content.lower()
            has_acceptance = "验收标准" in content or "acceptance" in content.lower()

            score = 100 if has_vision and has_features and has_acceptance else 70
            status = CheckStatus.PASSED if score >= 80 else CheckStatus.WARNING

            checks.append(QualityCheck(
                name="PRD 文档",
                category="documentation",
                description="产品需求文档完整性",
                status=status,
                score=score,
                weight=self.CHECKS_CONFIG["documentation"]["weight"],
                details="包含产品愿景、功能需求和验收标准" if status == CheckStatus.PASSED else "文档内容不完整",
            ))
        else:
            checks.append(QualityCheck(
                name="PRD 文档",
                category="documentation",
                description="产品需求文档存在性",
                status=CheckStatus.FAILED,
                score=0,
                weight=self.CHECKS_CONFIG["documentation"]["weight"],
                details="PRD 文档不存在",
            ))

        # 检查架构文档是否存在
        arch_path = self.project_dir / "output" / f"{self.name}-architecture.md"
        if arch_path.exists():
            content = arch_path.read_text(encoding="utf-8")
            has_tech_stack = "技术栈" in content or "tech stack" in content.lower()
            has_database = "数据库" in content or "database" in content.lower()
            has_api = "API" in content

            score = 100 if has_tech_stack and has_database and has_api else 70
            status = CheckStatus.PASSED if score >= 80 else CheckStatus.WARNING

            checks.append(QualityCheck(
                name="架构文档",
                category="documentation",
                description="架构设计文档完整性",
                status=status,
                score=score,
                weight=self.CHECKS_CONFIG["documentation"]["weight"],
                details="包含技术栈、数据库设计和 API 设计" if status == CheckStatus.PASSED else "文档内容不完整",
            ))
        else:
            checks.append(QualityCheck(
                name="架构文档",
                category="documentation",
                description="架构设计文档存在性",
                status=CheckStatus.FAILED,
                score=0,
                weight=self.CHECKS_CONFIG["documentation"]["weight"],
                details="架构文档不存在",
            ))

        # 检查 UI/UX 文档是否存在
        uiux_path = self.project_dir / "output" / f"{self.name}-uiux.md"
        if uiux_path.exists():
            checks.append(QualityCheck(
                name="UI/UX 文档",
                category="documentation",
                description="UI/UX 设计文档存在性",
                status=CheckStatus.PASSED,
                score=100,
                weight=self.CHECKS_CONFIG["documentation"]["weight"],
                details="UI/UX 文档已创建",
            ))
        else:
            checks.append(QualityCheck(
                name="UI/UX 文档",
                category="documentation",
                description="UI/UX 设计文档存在性",
                status=CheckStatus.WARNING,
                score=50,
                weight=self.CHECKS_CONFIG["documentation"]["weight"],
                details="UI/UX 文档不存在（可选）",
            ))

        return checks

    def _check_security(self, redteam_report: Optional["RedTeamReport"]) -> list[QualityCheck]:
        """检查安全性"""
        checks = []

        if redteam_report:
            critical_count = sum(1 for i in redteam_report.security_issues if i.severity == "critical")
            high_count = sum(1 for i in redteam_report.security_issues if i.severity == "high")

            if critical_count > 0:
                score = max(0, 100 - critical_count * 30)
                status = CheckStatus.FAILED
            elif high_count > 2:
                score = max(0, 100 - high_count * 15)
                status = CheckStatus.WARNING
            else:
                score = 100
                status = CheckStatus.PASSED

            checks.append(QualityCheck(
                name="安全审查",
                category="security",
                description=f"安全检查 ({critical_count} critical, {high_count} high)",
                status=status,
                score=score,
                weight=self.CHECKS_CONFIG["security"]["weight"],
                details=f"发现 {critical_count} 个严重问题和 {high_count} 个高危问题" if critical_count + high_count > 0 else "未发现严重安全问题",
            ))
        else:
            # 未进行红队审查，给警告
            checks.append(QualityCheck(
                name="安全审查",
                category="security",
                description="安全检查状态",
                status=CheckStatus.WARNING,
                score=50,
                weight=self.CHECKS_CONFIG["security"]["weight"],
                details="未进行红队安全审查",
            ))

        return checks

    def _check_performance(self, redteam_report: Optional["RedTeamReport"]) -> list[QualityCheck]:
        """检查性能"""
        checks = []

        if redteam_report:
            critical_count = sum(1 for i in redteam_report.performance_issues if i.severity == "critical")
            high_count = sum(1 for i in redteam_report.performance_issues if i.severity == "high")

            if critical_count > 0:
                score = max(0, 100 - critical_count * 25)
                status = CheckStatus.FAILED
            elif high_count > 2:
                score = max(0, 100 - high_count * 10)
                status = CheckStatus.WARNING
            else:
                score = 100
                status = CheckStatus.PASSED

            checks.append(QualityCheck(
                name="性能审查",
                category="performance",
                description=f"性能检查 ({critical_count} critical, {high_count} high)",
                status=status,
                score=score,
                weight=self.CHECKS_CONFIG["performance"]["weight"],
                details=f"发现 {critical_count} 个严重问题和 {high_count} 个高危问题" if critical_count + high_count > 0 else "未发现严重性能问题",
            ))
        else:
            checks.append(QualityCheck(
                name="性能审查",
                category="performance",
                description="性能检查状态",
                status=CheckStatus.WARNING,
                score=50,
                weight=self.CHECKS_CONFIG["performance"]["weight"],
                details="未进行红队性能审查",
            ))

        return checks

    def _check_testing(self) -> list[QualityCheck]:
        """检查测试策略"""
        checks = []

        # 检查是否有测试配置
        has_jest = (self.project_dir / "package.json").exists() and "jest" in (
            self.project_dir / "package.json"
        ).read_text()
        has_pytest = (self.project_dir / "pytest.ini").exists() or (
            self.project_dir / "pyproject.toml"
        ).exists()

        if has_jest or has_pytest:
            checks.append(QualityCheck(
                name="测试框架",
                category="testing",
                description="测试框架配置",
                status=CheckStatus.PASSED,
                score=100,
                weight=self.CHECKS_CONFIG["testing"]["weight"],
                details="测试框架已配置",
            ))
        else:
            checks.append(QualityCheck(
                name="测试框架",
                category="testing",
                description="测试框架配置",
                status=CheckStatus.WARNING,
                score=50,
                weight=self.CHECKS_CONFIG["testing"]["weight"],
                details="测试框架未配置",
            ))

        return checks

    def _check_code_quality(self) -> list[QualityCheck]:
        """检查代码质量工具"""
        checks = []

        # 检查 Linter
        has_eslint = (self.project_dir / ".eslintrc.js").exists() or (
            self.project_dir / ".eslintrc.json"
        ).exists()
        has_pylint = (self.project_dir / "pylint.ini").exists()
        has_black = (self.project_dir / "pyproject.toml").exists() and "black" in (
            self.project_dir / "pyproject.toml"
        ).read_text()

        if has_eslint or has_pylint or has_black:
            checks.append(QualityCheck(
                name="Linter",
                category="code_quality",
                description="代码静态检查工具",
                status=CheckStatus.PASSED,
                score=100,
                weight=self.CHECKS_CONFIG["code_quality"]["weight"],
                details="Linter 已配置",
            ))
        else:
            checks.append(QualityCheck(
                name="Linter",
                category="code_quality",
                description="代码静态检查工具",
                status=CheckStatus.WARNING,
                score=50,
                weight=self.CHECKS_CONFIG["code_quality"]["weight"],
                details="Linter 未配置",
            ))

        return checks

    def _calculate_total_score(self, checks: list[QualityCheck]) -> int:
        """计算总分"""
        if not checks:
            return 0

        return sum(c.score for c in checks) // len(checks)

    def _calculate_weighted_score(self, checks: list[QualityCheck]) -> float:
        """计算加权分"""
        if not checks:
            return 0.0

        total_weight = sum(c.weight for c in checks)
        if total_weight == 0:
            return 0.0

        weighted_sum = sum(c.score * c.weight for c in checks)
        return weighted_sum / total_weight

    def _generate_recommendations(self, checks: list[QualityCheck]) -> list[str]:
        """生成改进建议"""
        recommendations = []

        for check in checks:
            if check.status == CheckStatus.FAILED:
                recommendations.append(f"修复: {check.description}")
            elif check.status == CheckStatus.WARNING:
                recommendations.append(f"建议: {check.description}")

        return recommendations
