# -*- coding: utf-8 -*-
"""
Super Dev 分析器单元测试
"""

import json
import pytest
from pathlib import Path

from super_dev.analyzer import (
    ProjectAnalyzer,
    ProjectCategory,
    ProjectType,
    ArchitectureReport,
    Dependency,
    DesignPattern,
    PatternType,
    TechStack,
    detect_project_type,
    detect_tech_stack,
)


class TestProjectCategory:
    """测试项目分类枚举"""

    def test_category_values(self):
        """测试分类值"""
        assert ProjectCategory.FRONTEND.value == "frontend"
        assert ProjectCategory.BACKEND.value == "backend"
        assert ProjectCategory.FULLSTACK.value == "fullstack"
        assert ProjectCategory.MOBILE.value == "mobile"
        assert ProjectCategory.DESKTOP.value == "desktop"
        assert ProjectCategory.SERVERLESS.value == "serverless"
        assert ProjectCategory.UNKNOWN.value == "unknown"


class TestDependency:
    """测试依赖模型"""

    def test_dependency_creation(self):
        """测试依赖创建"""
        dep = Dependency(name="react", version="18.0.0", type="prod")

        assert dep.name == "react"
        assert dep.version == "18.0.0"
        assert dep.type == "prod"

    def test_dependency_str(self):
        """测试依赖字符串表示"""
        dep = Dependency(name="react", version="18.0.0")
        assert str(dep) == "react@18.0.0"

        dep_no_version = Dependency(name="vue")
        assert str(dep_no_version) == "vue"


class TestDesignPattern:
    """测试设计模式模型"""

    def test_pattern_creation(self):
        """测试模式创建"""
        pattern = DesignPattern(
            name=PatternType.SINGLETON,
            location=Path("/test/file.py"),
            description="单例模式实现",
            confidence=0.9,
        )

        assert pattern.name == PatternType.SINGLETON
        assert pattern.location == Path("/test/file.py")
        assert pattern.description == "单例模式实现"
        assert pattern.confidence == 0.9

    def test_pattern_to_dict(self):
        """测试模式转换为字典"""
        pattern = DesignPattern(
            name=PatternType.OBSERVER,
            location=Path("/test/observer.py"),
            description="观察者模式",
            confidence=0.8,
        )

        result = pattern.to_dict()

        assert result["name"] == "observer"
        assert result["location"] == "/test/observer.py"
        assert result["description"] == "观察者模式"
        assert result["confidence"] == 0.8


class TestTechStack:
    """测试技术栈模型"""

    def test_tech_stack_creation(self):
        """测试技术栈创建"""
        from super_dev.analyzer import FrameworkType

        stack = TechStack(
            category=ProjectCategory.FRONTEND,
            language="typescript",
            framework=FrameworkType.REACT,
            ui_library="@mui/material",
            state_management="zustand",
            dependencies=[],
        )

        assert stack.category == ProjectCategory.FRONTEND
        assert stack.language == "typescript"
        assert stack.ui_library == "@mui/material"

    def test_tech_stack_to_dict(self):
        """测试技术栈转换为字典"""
        stack = TechStack(
            category=ProjectCategory.FULLSTACK,
            language="javascript",
            framework="react",
            ui_library="antd",
            dependencies=[
                Dependency(name="react", version="18.0.0"),
                Dependency(name="antd", version="5.0.0"),
            ],
        )

        result = stack.to_dict()

        assert result["category"] == "fullstack"
        assert result["language"] == "javascript"
        assert result["framework"] == "react"
        assert result["ui_library"] == "antd"
        assert len(result["dependencies"]) == 2


class TestArchitectureReport:
    """测试架构报告模型"""

    def test_report_creation(self):
        """测试报告创建"""
        from super_dev.analyzer import FrameworkType

        report = ArchitectureReport(
            project_path=Path("/test/project"),
            category=ProjectCategory.BACKEND,
            tech_stack=TechStack(
                category=ProjectCategory.BACKEND,
                language="python",
                framework=FrameworkType.UNKNOWN,
                dependencies=[],
            ),
            file_count=100,
            total_lines=5000,
            languages_used={"Python": 5000},
        )

        assert report.project_path == Path("/test/project")
        assert report.category == ProjectCategory.BACKEND
        assert report.file_count == 100
        assert report.total_lines == 5000

    def test_report_to_dict(self):
        """测试报告转换为字典"""
        report = ArchitectureReport(
            project_path=Path("/test/project"),
            category=ProjectCategory.FRONTEND,
            tech_stack=TechStack(
                category=ProjectCategory.FRONTEND,
                language="typescript",
                framework="react",
                dependencies=[],
            ),
            file_count=50,
            total_lines=2500,
            languages_used={"TypeScript": 2000, "CSS": 500},
        )

        result = report.to_dict()

        assert result["project_path"] == "/test/project"
        assert result["category"] == "frontend"
        assert result["file_count"] == 50
        assert result["total_lines"] == 2500
        assert result["languages_used"]["TypeScript"] == 2000

    def test_report_to_markdown(self):
        """测试报告生成 Markdown"""
        report = ArchitectureReport(
            project_path=Path("/test/project"),
            category=ProjectCategory.FULLSTACK,
            tech_stack=TechStack(
                category=ProjectCategory.FULLSTACK,
                language="typescript",
                framework="react",
                ui_library="antd",
                state_management="zustand",
                testing_framework="jest",
                dependencies=[],
            ),
            file_count=200,
            total_lines=10000,
            languages_used={"TypeScript": 8000, "CSS": 2000},
        )

        markdown = report.to_markdown()

        assert "# 项目架构分析报告" in markdown
        assert "frontend" in markdown or "fullstack" in markdown
        assert "typescript" in markdown.lower()
        assert "200" in markdown  # 文件数量
        assert "10,000" in markdown or "10000" in markdown  # 代码行数


class TestDetectProjectType:
    """测试项目类型检测"""

    def test_detect_react_project(self, temp_project_dir: Path):
        """测试检测 React 项目"""
        # 创建 package.json
        package_json = temp_project_dir / "package.json"
        package_json.write_text(json.dumps({
            "name": "test-react-app",
            "dependencies": {
                "react": "^18.0.0",
                "react-dom": "^18.0.0",
            }
        }))

        category = detect_project_type(temp_project_dir)
        assert category == ProjectCategory.FRONTEND

    def test_detect_fullstack_project(self, temp_project_dir: Path):
        """测试检测全栈项目"""
        package_json = temp_project_dir / "package.json"
        package_json.write_text(json.dumps({
            "name": "test-fullstack",
            "dependencies": {
                "react": "^18.0.0",
                "express": "^4.18.0",
            }
        }))

        category = detect_project_type(temp_project_dir)
        assert category == ProjectCategory.FULLSTACK

    def test_detect_python_project(self, temp_project_dir: Path):
        """测试检测 Python 项目"""
        requirements_txt = temp_project_dir / "requirements.txt"
        requirements_txt.write_text("fastapi==0.100.0\nuvicorn==0.23.0\n")

        category = detect_project_type(temp_project_dir)
        assert category == ProjectCategory.BACKEND

    def test_detect_unknown_project(self, temp_project_dir: Path):
        """测试检测未知项目"""
        category = detect_project_type(temp_project_dir)
        assert category == ProjectCategory.UNKNOWN


class TestDetectTechStack:
    """测试技术栈检测"""

    def test_detect_react_stack(self, temp_project_dir: Path):
        """测试检测 React 技术栈"""
        package_json = temp_project_dir / "package.json"
        package_json.write_text(json.dumps({
            "name": "test",
            "dependencies": {
                "react": "^18.0.0",
                "react-dom": "^18.0.0",
                "antd": "^5.0.0",
                "zustand": "^4.0.0",
            },
            "devDependencies": {
                "vite": "^4.0.0",
                "jest": "^29.0.0",
            }
        }))

        stack = detect_tech_stack(temp_project_dir)

        assert stack.language in ["javascript", "typescript"]
        assert stack.framework.value in ["react"]
        assert stack.ui_library == "antd"
        assert stack.state_management == "zustand"
        assert stack.build_tool == "vite"
        assert stack.testing_framework == "jest"

    def test_detect_python_stack(self, temp_project_dir: Path):
        """测试检测 Python 技术栈"""
        requirements_txt = temp_project_dir / "requirements.txt"
        requirements_txt.write_text(
            "fastapi==0.100.0\n"
            "uvicorn==0.23.0\n"
            "pytest==7.4.0\n"
        )

        stack = detect_tech_stack(temp_project_dir)

        assert stack.language == "python"
        assert stack.framework.value in ["fastapi", "unknown"]
        assert stack.testing_framework == "pytest"


class TestProjectAnalyzer:
    """测试项目分析器"""

    def test_analyzer_initialization(self, temp_project_dir: Path):
        """测试分析器初始化"""
        analyzer = ProjectAnalyzer(temp_project_dir)
        # 使用 resolve() 来处理 macOS 的 /var 软链接
        assert analyzer.project_path == temp_project_dir.resolve()

    def test_analyzer_nonexistent_path(self):
        """测试分析器处理不存在的路径"""
        with pytest.raises(FileNotFoundError):
            ProjectAnalyzer("/nonexistent/path")

    def test_analyze_simple_project(self, temp_project_dir: Path):
        """测试分析简单项目"""
        # 创建 package.json
        package_json = temp_project_dir / "package.json"
        package_json.write_text(json.dumps({
            "name": "test-app",
            "dependencies": {
                "react": "^18.0.0",
            }
        }))

        # 创建一些源文件
        src_dir = temp_project_dir / "src"
        src_dir.mkdir()

        (src_dir / "App.tsx").write_text("""
export function App() {
    return <div>Hello</div>;
}
""")

        (src_dir / "index.tsx").write_text("""
import { App } from './App';
console.log(App);
""")

        analyzer = ProjectAnalyzer(temp_project_dir)
        report = analyzer.analyze()

        assert report.category == ProjectCategory.FRONTEND
        assert report.file_count >= 2
        assert report.total_lines > 0
        assert len(report.languages_used) > 0

    def test_analyze_python_project(self, temp_project_dir: Path):
        """测试分析 Python 项目"""
        requirements_txt = temp_project_dir / "requirements.txt"
        requirements_txt.write_text("fastapi==0.100.0\n")

        src_dir = temp_project_dir / "src"
        src_dir.mkdir()

        (src_dir / "main.py").write_text("""
from fastapi import FastAPI

app = FastAPI()

@app.get("/")
def read_root():
    return {"Hello": "World"}
""")

        analyzer = ProjectAnalyzer(temp_project_dir)
        report = analyzer.analyze()

        assert report.category == ProjectCategory.BACKEND
        assert "Python" in report.languages_used

    def test_analyzer_get_summary(self, temp_project_dir: Path):
        """测试获取项目摘要"""
        package_json = temp_project_dir / "package.json"
        package_json.write_text(json.dumps({
            "name": "test",
            "dependencies": {"react": "^18.0.0"}
        }))

        src_dir = temp_project_dir / "src"
        src_dir.mkdir()
        (src_dir / "index.ts").write_text("console.log('test');")

        analyzer = ProjectAnalyzer(temp_project_dir)
        summary = analyzer.get_summary()

        assert "项目类型:" in summary
        assert "编程语言:" in summary
        assert "框架:" in summary
        assert "文件数量:" in summary

    def test_analyzer_get_dependencies(self, temp_project_dir: Path):
        """测试获取依赖列表"""
        package_json = temp_project_dir / "package.json"
        package_json.write_text(json.dumps({
            "name": "test",
            "dependencies": {
                "react": "^18.0.0",
                "antd": "^5.0.0",
            },
            "devDependencies": {
                "jest": "^29.0.0",
            }
        }))

        analyzer = ProjectAnalyzer(temp_project_dir)
        analyzer.analyze()

        deps = analyzer.get_dependencies()

        assert len(deps) == 2
        dep_names = [d.name for d in deps]
        assert "react" in dep_names
        assert "antd" in dep_names

    def test_analyzer_get_language_distribution(self, temp_project_dir: Path):
        """测试获取语言分布"""
        package_json = temp_project_dir / "package.json"
        package_json.write_text(json.dumps({
            "name": "test",
            "dependencies": {"react": "^18.0.0"}
        }))

        src_dir = temp_project_dir / "src"
        src_dir.mkdir()
        (src_dir / "index.ts").write_text("console.log('test');\n" * 10)
        (src_dir / "styles.css").write_text(".test { color: red; }\n" * 10)

        analyzer = ProjectAnalyzer(temp_project_dir)
        analyzer.analyze()

        distribution = analyzer.get_language_distribution()

        assert len(distribution) > 0
        # 检查百分比总和约等于100
        total_percentage = sum(distribution.values())
        assert abs(total_percentage - 100.0) < 1.0  # 允许1%误差

    def test_analyzer_directory_structure(self, temp_project_dir: Path):
        """测试目录结构分析"""
        package_json = temp_project_dir / "package.json"
        package_json.write_text(json.dumps({"name": "test"}))

        # 创建目录结构和代码文件
        (temp_project_dir / "src").mkdir()
        (temp_project_dir / "src" / "components").mkdir()
        (temp_project_dir / "src" / "utils").mkdir()
        (temp_project_dir / "public").mkdir()

        # 创建代码文件（只有包含代码文件的目录才会被分析）
        (temp_project_dir / "src" / "App.tsx").write_text("export default function App() {}")
        (temp_project_dir / "src" / "components" / "Button.tsx").write_text("export default function Button() {}")
        (temp_project_dir / "src" / "utils" / "helper.ts").write_text("export function helper() {}")
        (temp_project_dir / "public" / "index.html").write_text("<html></html>")

        analyzer = ProjectAnalyzer(temp_project_dir)
        report = analyzer.analyze()

        # 检查目录结构
        assert "src" in report.directory_structure
        # src 应该有子目录
        assert "components" in report.directory_structure.get("src", {})
        assert "utils" in report.directory_structure.get("src", {})
