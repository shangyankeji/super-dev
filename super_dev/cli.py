# -*- coding: utf-8 -*-
"""
开发：Excellent（11964948@qq.com）
功能：Super Dev CLI 主入口
作用：提供命令行界面，统一访问所有功能
创建时间：2025-12-30
最后修改：2025-12-30
"""

import sys
import argparse
from pathlib import Path
from typing import Optional

try:
    from rich.console import Console
    from rich.panel import Panel
    from rich.text import Text
    RICH_AVAILABLE = True
except ImportError:
    RICH_AVAILABLE = False

from . import __version__, __description__
from .config import get_config_manager, ConfigManager
from .orchestrator import WorkflowEngine, Phase


class SuperDevCLI:
    """Super Dev 命令行接口"""

    def __init__(self):
        self.console = Console() if RICH_AVAILABLE else None
        self.parser = self._create_parser()

    def _create_parser(self) -> argparse.ArgumentParser:
        """创建命令行参数解析器"""
        parser = argparse.ArgumentParser(
            prog="super-dev",
            description=__description__,
            formatter_class=argparse.RawDescriptionHelpFormatter,
            epilog="""
示例:
  super-dev init my-project        初始化新项目
  super-dev analyze [path]         分析现有项目
  super-dev workflow               运行完整工作流
  super-dev expert PM              调用产品经理专家
  super-dev quality                运行质量检查
  super-dev preview                生成原型预览
  super-dev deploy                 生成部署配置
            """
        )

        parser.add_argument(
            "-v", "--version",
            action="version",
            version=f"%(prog)s {__version__}"
        )

        # 子命令
        subparsers = parser.add_subparsers(
            dest="command",
            title="可用命令",
            description="使用 'super-dev <command> -h' 查看帮助"
        )

        # init 命令
        init_parser = subparsers.add_parser(
            "init",
            help="初始化新项目",
            description="创建一个新的 Super Dev 项目"
        )
        init_parser.add_argument(
            "name",
            help="项目名称"
        )
        init_parser.add_argument(
            "-d", "--description",
            default="",
            help="项目描述"
        )
        init_parser.add_argument(
            "-p", "--platform",
            choices=["web", "mobile", "wechat", "desktop"],
            default="web",
            help="目标平台"
        )
        init_parser.add_argument(
            "-f", "--frontend",
            choices=[
                "next", "remix", "react-vite", "gatsby",
                "nuxt", "vue-vite",
                "angular",
                "sveltekit",
                "astro", "solid", "qwik",
                "none"
            ],
            default="next",
            help="前端框架"
        )
        init_parser.add_argument(
            "--ui-library",
            choices=[
                "mui", "ant-design", "chakra-ui", "mantine", "shadcn-ui", "radix-ui",
                "element-plus", "naive-ui", "vuetify", "primevue", "arco-design",
                "angular-material", "primeng",
                "skeleton-ui", "svelte-material-ui",
                "tailwind", "daisyui"
            ],
            help="UI 组件库"
        )
        init_parser.add_argument(
            "--style",
            choices=["tailwind", "css-modules", "styled-components", "emotion", "scss", "less", "unocss"],
            help="样式方案"
        )
        init_parser.add_argument(
            "--state",
            choices=["react-query", "swr", "zustand", "redux-toolkit", "jotai", "pinia", "xstate"],
            action="append",
            help="状态管理方案 (可多选)"
        )
        init_parser.add_argument(
            "--testing",
            choices=["vitest", "jest", "playwright", "cypress", "testing-library"],
            action="append",
            help="测试框架 (可多选)"
        )
        init_parser.add_argument(
            "-b", "--backend",
            choices=["node", "python", "go", "java", "none"],
            default="node",
            help="后端框架"
        )
        init_parser.add_argument(
            "--domain",
            choices=["", "fintech", "ecommerce", "medical", "social", "iot", "education"],
            default="",
            help="业务领域"
        )

        # analyze 命令
        analyze_parser = subparsers.add_parser(
            "analyze",
            help="分析现有项目",
            description="自动检测和分析现有项目的结构、技术栈和架构模式"
        )
        analyze_parser.add_argument(
            "path",
            nargs="?",
            default=".",
                       help="项目路径 (默认为当前目录)"
        )
        analyze_parser.add_argument(
            "-o", "--output",
            help="输出报告文件路径 (Markdown 格式)"
        )
        analyze_parser.add_argument(
            "-f", "--format",
            choices=["json", "markdown", "text"],
            default="text",
            help="输出格式"
        )
        analyze_parser.add_argument(
            "--json",
            action="store_true",
            help="以 JSON 格式输出"
        )

        # workflow 命令
        workflow_parser = subparsers.add_parser(
            "workflow",
            help="运行工作流",
            description="执行 Super Dev 6 阶段工作流"
        )
        workflow_parser.add_argument(
            "--phase",
            choices=["discovery", "intelligence", "drafting", "redteam", "qa", "delivery", "deployment"],
            nargs="*",
            help="指定要执行的阶段"
        )
        workflow_parser.add_argument(
            "-q", "--quality-gate",
            type=int,
            help="质量门禁阈值 (0-100)"
        )

        # expert 命令
        expert_parser = subparsers.add_parser(
            "expert",
            help="调用专家",
            description="直接调用特定专家"
        )
        expert_parser.add_argument(
            "--list",
            action="store_true",
            help="列出所有可用专家"
        )
        expert_parser.add_argument(
            "expert_name",
            nargs="?",
            choices=["PM", "ARCHITECT", "UI", "UX", "SECURITY", "CODE", "DBA", "QA", "DEVOPS", "RCA"],
            help="专家名称"
        )
        expert_parser.add_argument(
            "prompt",
            nargs="*",
            help="提示词"
        )

        # quality 命令
        quality_parser = subparsers.add_parser(
            "quality",
            help="质量检查",
            description="运行质量检查脚本"
        )
        quality_parser.add_argument(
            "-t", "--type",
            choices=["prd", "architecture", "ui", "ux", "code", "all"],
            default="all",
            help="检查类型"
        )

        # preview 命令
        preview_parser = subparsers.add_parser(
            "preview",
            help="生成原型",
            description="从 UI 设计生成可交互的原型"
        )
        preview_parser.add_argument(
            "-o", "--output",
            default="preview.html",
            help="输出文件路径"
        )

        # deploy 命令
        deploy_parser = subparsers.add_parser(
            "deploy",
            help="生成部署配置",
            description="生成 Dockerfile 和 CI/CD 配置"
        )
        deploy_parser.add_argument(
            "--docker",
            action="store_true",
            help="生成 Dockerfile"
        )
        deploy_parser.add_argument(
            "--cicd",
            choices=["github", "gitlab", "jenkins", "azure", "bitbucket"],
            help="生成 CI/CD 配置"
        )

        # config 命令
        config_parser = subparsers.add_parser(
            "config",
            help="配置管理",
            description="查看和修改项目配置"
        )
        config_parser.add_argument(
            "action",
            choices=["get", "set", "list"],
            help="操作"
        )
        config_parser.add_argument(
            "key",
            nargs="?",
            help="配置键"
        )
        config_parser.add_argument(
            "value",
            nargs="?",
            help="配置值"
        )

        # create 命令 - 一键创建项目
        create_parser = subparsers.add_parser(
            "create",
            help="一键创建项目 (从想法到规范)",
            description="从一句话描述自动生成 PRD、架构、UI/UX 文档并创建 Spec"
        )
        create_parser.add_argument(
            "description",
            help="功能描述 (如 '用户认证系统')"
        )
        create_parser.add_argument(
            "-p", "--platform",
            choices=["web", "mobile", "wechat", "desktop"],
            default="web",
            help="目标平台"
        )
        create_parser.add_argument(
            "-f", "--frontend",
            choices=["react", "vue", "angular", "svelte", "none"],
            default="react",
            help="前端框架"
        )
        create_parser.add_argument(
            "-b", "--backend",
            choices=["node", "python", "go", "java", "none"],
            default="node",
            help="后端框架"
        )
        create_parser.add_argument(
            "-d", "--domain",
            choices=["", "fintech", "ecommerce", "medical", "social", "iot", "education", "auth", "content"],
            default="",
            help="业务领域"
        )
        create_parser.add_argument(
            "--name",
            help="项目名称 (默认根据描述生成)"
        )
        create_parser.add_argument(
            "--skip-docs",
            action="store_true",
            help="跳过文档生成，只创建 Spec"
        )

        # design 命令 - 设计智能引擎
        design_parser = subparsers.add_parser(
            "design",
            help="设计智能引擎 - 超越 UI UX Pro Max",
            description="搜索设计资产、生成设计系统、创建 design tokens"
        )
        design_subparsers = design_parser.add_subparsers(
            dest="design_command",
            title="设计命令",
            description="使用 'super-dev design <command> -h' 查看帮助"
        )

        # design search
        design_search_parser = design_subparsers.add_parser(
            "search",
            help="搜索设计资产",
            description="搜索 UI 风格、配色、字体、组件等设计资产"
        )
        design_search_parser.add_argument(
            "query",
            help="搜索关键词"
        )
        design_search_parser.add_argument(
            "-d", "--domain",
            choices=["style", "color", "typography", "component", "layout", "animation", "ux", "chart", "product"],
            help="搜索领域 (默认自动检测)"
        )
        design_search_parser.add_argument(
            "-n", "--max-results",
            type=int,
            default=5,
            help="最大结果数 (默认: 5)"
        )

        # design generate
        design_generate_parser = design_subparsers.add_parser(
            "generate",
            help="生成完整设计系统",
            description="基于产品类型和行业生成完整的设计系统"
        )
        design_generate_parser.add_argument(
            "--product",
            required=True,
            help="产品类型 (SaaS, E-commerce, Portfolio, Dashboard)"
        )
        design_generate_parser.add_argument(
            "--industry",
            required=True,
            help="行业 (Fintech, Healthcare, Education, Gaming)"
        )
        design_generate_parser.add_argument(
            "--keywords",
            nargs="+",
            help="关键词列表"
        )
        design_generate_parser.add_argument(
            "-p", "--platform",
            choices=["web", "mobile", "desktop"],
            default="web",
            help="目标平台 (默认: web)"
        )
        design_generate_parser.add_argument(
            "-a", "--aesthetic",
            help="美学方向 (可选)"
        )
        design_generate_parser.add_argument(
            "-o", "--output",
            default="output/design",
            help="输出目录 (默认: output/design)"
        )

        # design tokens
        design_tokens_parser = design_subparsers.add_parser(
            "tokens",
            help="生成 design tokens",
            description="生成 CSS 变量、Tailwind 配置等 design tokens"
        )
        design_tokens_parser.add_argument(
            "--primary",
            required=True,
            help="主色 (hex 值，如 #3B82F6)"
        )
        design_tokens_parser.add_argument(
            "--type",
            choices=["monochromatic", "analogous", "complementary", "triadic"],
            default="monochromatic",
            help="调色板类型 (默认: monochromatic)"
        )
        design_tokens_parser.add_argument(
            "--format",
            choices=["css", "json", "tailwind"],
            default="css",
            help="输出格式 (默认: css)"
        )
        design_tokens_parser.add_argument(
            "-o", "--output",
            help="输出文件路径"
        )

        # design landing - Landing 页面模式
        design_landing_parser = design_subparsers.add_parser(
            "landing",
            help="Landing 页面模式生成",
            description="搜索和推荐 Landing 页面布局模式"
        )
        design_landing_parser.add_argument(
            "query",
            nargs="?",
            help="搜索关键词（可选）"
        )
        design_landing_parser.add_argument(
            "--product-type",
            help="产品类型 (SaaS, E-commerce, Mobile, etc.)"
        )
        design_landing_parser.add_argument(
            "--goal",
            help="目标 (signup, purchase, demo, etc.)"
        )
        design_landing_parser.add_argument(
            "--audience",
            help="受众 (B2B, B2C, Enterprise, etc.)"
        )
        design_landing_parser.add_argument(
            "-n", "--max-results",
            type=int,
            default=5,
            help="最大结果数 (默认: 5)"
        )
        design_landing_parser.add_argument(
            "--list",
            action="store_true",
            help="列出所有可用模式"
        )

        # design chart - 图表类型推荐
        design_chart_parser = design_subparsers.add_parser(
            "chart",
            help="图表类型推荐",
            description="根据数据类型推荐最佳图表类型"
        )
        design_chart_parser.add_argument(
            "data_description",
            help="数据描述（如 'time series sales data'）"
        )
        design_chart_parser.add_argument(
            "-f", "--framework",
            choices=["react", "vue", "svelte", "angular", "next", "vanilla"],
            default="react",
            help="前端框架 (默认: react)"
        )
        design_chart_parser.add_argument(
            "-n", "--max-results",
            type=int,
            default=3,
            help="最大结果数 (默认: 3)"
        )
        design_chart_parser.add_argument(
            "--list",
            action="store_true",
            help="列出所有图表类型"
        )

        # design ux - UX 指南查询
        design_ux_parser = design_subparsers.add_parser(
            "ux",
            help="UX 指南查询",
            description="查询 UX 最佳实践和反模式"
        )
        design_ux_parser.add_argument(
            "query",
            help="搜索查询"
        )
        design_ux_parser.add_argument(
            "-d", "--domain",
            help="领域过滤 (Animation, A11y, Performance, etc.)"
        )
        design_ux_parser.add_argument(
            "-n", "--max-results",
            type=int,
            default=5,
            help="最大结果数 (默认: 5)"
        )
        design_ux_parser.add_argument(
            "--quick-wins",
            action="store_true",
            help="显示快速见效的改进建议"
        )
        design_ux_parser.add_argument(
            "--checklist",
            action="store_true",
            help="显示 UX 检查清单"
        )
        design_ux_parser.add_argument(
            "--list-domains",
            action="store_true",
            help="列出所有领域"
        )

        # design stack - 技术栈最佳实践
        design_stack_parser = design_subparsers.add_parser(
            "stack",
            help="技术栈最佳实践",
            description="查询技术栈最佳实践、性能优化和设计模式"
        )
        design_stack_parser.add_argument(
            "stack",
            help="技术栈名称 (Next.js, React, Vue, SvelteKit, etc.)"
        )
        design_stack_parser.add_argument(
            "query",
            nargs="?",
            help="搜索查询（可选）"
        )
        design_stack_parser.add_argument(
            "-c", "--category",
            help="类别过滤 (architecture, performance, state_management, etc.)"
        )
        design_stack_parser.add_argument(
            "--patterns",
            action="store_true",
            help="显示设计模式"
        )
        design_stack_parser.add_argument(
            "--performance",
            action="store_true",
            help="显示性能优化建议"
        )
        design_stack_parser.add_argument(
            "--quick-wins",
            action="store_true",
            help="显示快速见效的性能优化"
        )
        design_stack_parser.add_argument(
            "-n", "--max-results",
            type=int,
            default=5,
            help="最大结果数 (默认: 5)"
        )
        design_stack_parser.add_argument(
            "--list",
            action="store_true",
            help="列出所有支持的技术栈"
        )

        # design codegen - 代码片段生成
        design_codegen_parser = design_subparsers.add_parser(
            "codegen",
            help="代码片段生成",
            description="生成多框架的 UI 组件代码片段"
        )
        design_codegen_parser.add_argument(
            "component",
            help="组件名称 (button, card, input, etc.)"
        )
        design_codegen_parser.add_argument(
            "-f", "--framework",
            choices=["react", "nextjs", "vue", "svelte", "html", "tailwind"],
            default="react",
            help="目标框架 (默认: react)"
        )
        design_codegen_parser.add_argument(
            "-o", "--output",
            help="输出文件路径"
        )
        design_codegen_parser.add_argument(
            "--list",
            action="store_true",
            help="列出所有可用组件"
        )
        design_codegen_parser.add_argument(
            "--search",
            help="搜索组件"
        )

        # spec 命令 - Spec-Driven Development
        spec_parser = subparsers.add_parser(
            "spec",
            help="规范驱动开发 (SDD)",
            description="管理规范和变更提案"
        )
        spec_subparsers = spec_parser.add_subparsers(
            dest="spec_action",
            title="SDD 命令",
            description="使用 'super-dev spec <command> -h' 查看帮助"
        )

        # spec init
        spec_init_parser = spec_subparsers.add_parser(
            "init",
            help="初始化 SDD 目录结构"
        )

        # spec list
        spec_list_parser = spec_subparsers.add_parser(
            "list",
            help="列出所有变更"
        )
        spec_list_parser.add_argument(
            "--status",
            choices=["draft", "proposed", "approved", "in_progress", "completed", "archived"],
            help="按状态过滤"
        )

        # spec show
        spec_show_parser = spec_subparsers.add_parser(
            "show",
            help="显示变更详情"
        )
        spec_show_parser.add_argument(
            "change_id",
            help="变更 ID"
        )

        # spec propose
        spec_propose_parser = spec_subparsers.add_parser(
            "propose",
            help="创建变更提案"
        )
        spec_propose_parser.add_argument(
            "change_id",
            help="变更 ID (如 add-user-auth)"
        )
        spec_propose_parser.add_argument(
            "--title",
            required=True,
            help="变更标题"
        )
        spec_propose_parser.add_argument(
            "--description",
            required=True,
            help="变更描述"
        )
        spec_propose_parser.add_argument(
            "--motivation",
            help="变更动机/背景"
        )
        spec_propose_parser.add_argument(
            "--impact",
            help="影响范围"
        )

        # spec add-req
        spec_add_req_parser = spec_subparsers.add_parser(
            "add-req",
            help="向变更添加需求"
        )
        spec_add_req_parser.add_argument(
            "change_id",
            help="变更 ID"
        )
        spec_add_req_parser.add_argument(
            "spec_name",
            help="规范名称 (如 auth, user-profile)"
        )
        spec_add_req_parser.add_argument(
            "req_name",
            help="需求名称"
        )
        spec_add_req_parser.add_argument(
            "description",
            help="需求描述"
        )

        # spec archive
        spec_archive_parser = spec_subparsers.add_parser(
            "archive",
            help="归档已完成的变更"
        )
        spec_archive_parser.add_argument(
            "change_id",
            help="变更 ID"
        )
        spec_archive_parser.add_argument(
            "-y", "--yes",
            action="store_true",
            help="跳过确认"
        )

        # spec validate
        spec_validate_parser = spec_subparsers.add_parser(
            "validate",
            help="验证规格格式和结构"
        )
        spec_validate_parser.add_argument(
            "change_id",
            nargs="?",
            help="变更 ID (留空则验证所有变更)"
        )
        spec_validate_parser.add_argument(
            "-v", "--verbose",
            action="store_true",
            help="显示详细信息"
        )

        # spec view
        spec_view_parser = spec_subparsers.add_parser(
            "view",
            help="交互式仪表板 - 显示所有规范和变更"
        )
        spec_view_parser.add_argument(
            "--refresh",
            action="store_true",
            help="强制刷新数据"
        )

        # pipeline 命令 - 完整流水线
        pipeline_parser = subparsers.add_parser(
            "pipeline",
            help="运行完整流水线 (从想法到部署)",
            description="执行完整的开发流水线：文档 → Spec → 红队审查 → 质量门禁 → 代码审查指南 → CI/CD"
        )
        pipeline_parser.add_argument(
            "description",
            help="功能描述 (如 '用户认证系统')"
        )
        pipeline_parser.add_argument(
            "-p", "--platform",
            choices=["web", "mobile", "wechat", "desktop"],
            default="web",
            help="目标平台"
        )
        pipeline_parser.add_argument(
            "-f", "--frontend",
            choices=["react", "vue", "angular", "svelte", "none"],
            default="react",
            help="前端框架"
        )
        pipeline_parser.add_argument(
            "-b", "--backend",
            choices=["node", "python", "go", "java", "none"],
            default="node",
            help="后端框架"
        )
        pipeline_parser.add_argument(
            "-d", "--domain",
            choices=["", "fintech", "ecommerce", "medical", "social", "iot", "education", "auth", "content"],
            default="",
            help="业务领域"
        )
        pipeline_parser.add_argument(
            "--name",
            help="项目名称 (默认根据描述生成)"
        )
        pipeline_parser.add_argument(
            "--cicd",
            choices=["github", "gitlab", "jenkins", "azure", "bitbucket"],
            default="github",
            help="CI/CD 平台"
        )
        pipeline_parser.add_argument(
            "--skip-redteam",
            action="store_true",
            help="跳过红队审查"
        )
        pipeline_parser.add_argument(
            "--quality-threshold",
            type=int,
            default=80,
            help="质量门禁阈值 (默认 80)"
        )

        return parser

    def run(self, args: Optional[list] = None) -> int:
        """
        运行 CLI

        Args:
            args: 命令行参数

        Returns:
            退出码
        """
        parsed_args = self.parser.parse_args(args)

        if parsed_args.command is None:
            self._print_banner()
            self.parser.print_help()
            return 0

        # 路由到对应命令
        command_handler = getattr(self, f"_cmd_{parsed_args.command}", None)
        if command_handler is None:
            self.console.print(f"[red]未知命令: {parsed_args.command}[/red]")
            return 1

        try:
            return command_handler(parsed_args)
        except Exception as e:
            self.console.print(f"[red]错误: {e}[/red]")
            return 1

    # ==================== 命令处理器 ====================

    def _cmd_init(self, args) -> int:
        """初始化项目"""
        config_manager = get_config_manager()

        # 检查是否已初始化
        if config_manager.exists():
            self.console.print("[yellow]项目已初始化，使用 'super-dev config set' 修改配置[/yellow]")
            return 0

        # 创建配置
        config = config_manager.create(
            name=args.name,
            description=args.description,
            platform=args.platform,
            frontend=args.frontend,
            backend=args.backend,
            domain=args.domain,
            ui_library=getattr(args, 'ui_library', None),
            style_solution=getattr(args, 'style', None),
            state_management=getattr(args, 'state', []) or [],
            testing_frameworks=getattr(args, 'testing', []) or [],
        )

        # 创建输出目录
        output_dir = Path.cwd() / config.output_dir
        output_dir.mkdir(exist_ok=True)

        self.console.print(f"[green]✓[/green] 项目已初始化: {config.name}")
        self.console.print(f"  平台: {config.platform}")
        self.console.print(f"  前端框架: {config.frontend}")
        if config.ui_library:
            self.console.print(f"  UI 组件库: {config.ui_library}")
        if config.style_solution:
            self.console.print(f"  样式方案: {config.style_solution}")
        if config.state_management:
            self.console.print(f"  状态管理: {', '.join(config.state_management)}")
        if config.testing_frameworks:
            self.console.print(f"  测试框架: {', '.join(config.testing_frameworks)}")
        self.console.print(f"  后端: {config.backend}")
        if config.domain:
            self.console.print(f"  领域: {config.domain}")

        self.console.print(f"\n[dim]下一步:[/dim]")
        self.console.print(f"  1. 编辑 super-dev.yaml 配置项目详情")
        self.console.print(f"  2. 运行 'super-dev workflow' 开始开发")

        return 0

    def _cmd_analyze(self, args) -> int:
        """分析现有项目"""
        from .analyzer import ProjectAnalyzer

        project_path = Path(args.path).resolve()

        if not project_path.exists():
            self.console.print(f"[red]项目不存在: {project_path}[/red]")
            return 1

        self.console.print(f"[cyan]正在分析项目: {project_path}[/cyan]")

        try:
            analyzer = ProjectAnalyzer(project_path)
            report = analyzer.analyze()

            # 根据格式输出
            output_format = "json" if args.json else args.format

            if output_format == "json":
                import json
                output = json.dumps(report.to_dict(), indent=2, ensure_ascii=False)

                if args.output:
                    Path(args.output).write_text(output, encoding="utf-8")
                    self.console.print(f"[green]报告已保存到: {args.output}[/green]")
                else:
                    self.console.print(output)

            elif output_format == "markdown":
                output = report.to_markdown()

                if args.output:
                    Path(args.output).write_text(output, encoding="utf-8")
                    self.console.print(f"[green]报告已保存到: {args.output}[/green]")
                else:
                    self.console.print(output)

            else:  # text
                self.console.print(f"[cyan]项目分析报告[/cyan]")
                self.console.print(f"  路径: {report.project_path}")
                self.console.print(f"  类型: {report.category.value}")
                self.console.print(f"  语言: {report.tech_stack.language}")
                self.console.print(f"  框架: {report.tech_stack.framework.value}")
                if report.tech_stack.ui_library:
                    self.console.print(f"  UI 库: {report.tech_stack.ui_library}")
                if report.tech_stack.state_management:
                    self.console.print(f"  状态管理: {report.tech_stack.state_management}")
                self.console.print(f"  文件数: {report.file_count}")
                self.console.print(f"  代码行数: {report.total_lines:,}")
                self.console.print(f"  依赖数: {len(report.tech_stack.dependencies)}")

                if args.output:
                    Path(args.output).write_text(report.to_markdown(), encoding="utf-8")
                    self.console.print(f"[green]报告已保存到: {args.output}[/green]")

            return 0

        except Exception as e:
            self.console.print(f"[red]分析失败: {e}[/red]")
            import traceback
            self.console.print(traceback.format_exc())
            return 1

    def _cmd_workflow(self, args) -> int:
        """运行工作流"""
        config_manager = get_config_manager()

        if not config_manager.exists():
            self.console.print("[red]未找到项目配置，请先运行 'super-dev init'[/red]")
            return 1

        # 更新质量门禁
        if args.quality_gate is not None:
            config_manager.update(quality_gate=args.quality_gate)

        # 确定要执行的阶段
        phases = None
        if args.phase:
            phase_map = {
                "discovery": Phase.DISCOVERY,
                "intelligence": Phase.INTELLIGENCE,
                "drafting": Phase.DRAFTING,
                "redteam": Phase.REDTEAM,
                "qa": Phase.QA,
                "delivery": Phase.DELIVERY,
                "deployment": Phase.DEPLOYMENT,
            }
            phases = [phase_map[p] for p in args.phase]

        # 运行工作流
        import asyncio
        engine = WorkflowEngine()
        results = asyncio.run(engine.run(phases=phases))

        # 检查是否全部成功
        all_success = all(r.success for r in results.values())

        return 0 if all_success else 1

    def _cmd_expert(self, args) -> int:
        """调用专家"""
        # 处理 --list 选项
        if args.list:
            self.console.print("[cyan]可用专家列表:[/cyan]")
            experts = [
                ("PM", "产品经理 - 战略、市场契合度、优先级"),
                ("ARCHITECT", "架构师 - 扩展性、权衡取舍、可用性"),
                ("UI", "UI 设计师 - 设计系统、视觉打磨"),
                ("UX", "UX 设计师 - 流程、用户心理学"),
                ("SECURITY", "安全红队 - 渗透测试、威胁建模"),
                ("CODE", "代码审查官 - 安全、性能优化"),
                ("DBA", "数据架构 - 范式、索引、一致性"),
                ("QA", "测试开发 - 自动化、质量门禁"),
                ("DEVOPS", "基础设施 - IaC、容器化、流水线"),
                ("RCA", "故障侦探 - 复盘、5 Whys、监控"),
            ]
            for code, desc in experts:
                self.console.print(f"  [green]{code:<10}[/green] {desc}")
            return 0

        # 如果没有提供专家名称，显示帮助
        if not args.expert_name:
            self.console.print("[yellow]请提供专家名称或使用 --list 查看可用专家[/yellow]")
            return 1

        prompt = " ".join(args.prompt) if args.prompt else ""
        self.console.print(f"[cyan]调用专家: {args.expert_name}[/cyan]")
        self.console.print(f"[dim]提示词: {prompt or '(无)'}[/dim]")

        # 这里会调用对应的专家
        # 暂时只打印消息
        self.console.print("[yellow]专家功能正在实现中...[/yellow]")

        return 0

    def _cmd_quality(self, args) -> int:
        """质量检查"""
        self.console.print(f"[cyan]运行质量检查: {args.type}[/cyan]")

        # 这里会调用 quality_check.py
        self.console.print("[yellow]质量检查功能正在实现中...[/yellow]")

        return 0

    def _cmd_preview(self, args) -> int:
        """生成原型"""
        self.console.print(f"[cyan]生成原型: {args.output}[/cyan]")

        # 这里会调用 generate_preview.py
        self.console.print("[yellow]原型生成功能正在实现中...[/yellow]")

        return 0

    def _cmd_deploy(self, args) -> int:
        """生成部署配置"""
        if args.docker:
            self.console.print("[cyan]生成 Dockerfile...[/cyan]")
        if args.cicd:
            self.console.print(f"[cyan]生成 CI/CD 配置: {args.cicd}[/cyan]")

        # 这里会调用 generate_dockerfile.py, generate_ci_cd.py
        self.console.print("[yellow]部署配置功能正在实现中...[/yellow]")

        return 0

    def _cmd_create(self, args) -> int:
        """一键创建项目 - 从想法到规范"""
        from .creators import ProjectCreator

        self.console.print(f"[cyan]Super Dev 项目创建器[/cyan]")
        self.console.print(f"[dim]描述: {args.description}[/dim]")
        self.console.print(f"[dim]平台: {args.platform} | 前端: {args.frontend} | 后端: {args.backend}[/dim]")
        self.console.print("")

        # 确定项目名称
        project_name = args.name
        if not project_name:
            # 从描述生成项目名称
            import re
            words = re.findall(r'[\w]+', args.description)
            if words:
                project_name = '-'.join(words[:3]).lower()
            else:
                project_name = "my-project"

        # 创建项目目录
        project_dir = Path.cwd()

        try:
            creator = ProjectCreator(
                project_dir=project_dir,
                name=project_name,
                description=args.description,
                platform=args.platform,
                frontend=args.frontend,
                backend=args.backend,
                domain=args.domain,
                ui_library=getattr(args, 'ui_library', None),
                style_solution=getattr(args, 'style', None),
                state_management=getattr(args, 'state', []) or [],
                testing_frameworks=getattr(args, 'testing', []) or [],
            )

            # 1. 生成文档
            if not args.skip_docs:
                self.console.print("[cyan]第 1 步: 生成专业文档...[/cyan]")
                docs = creator.generate_documents()
                self.console.print(f"  [green]✓[/green] PRD: {docs['prd']}")
                self.console.print(f"  [green]✓[/green] 架构: {docs['architecture']}")
                self.console.print(f"  [green]✓[/green] UI/UX: {docs['uiux']}")
                self.console.print("")

            # 2. 创建 Spec
            self.console.print("[cyan]第 2 步: 创建 Spec 规范...[/cyan]")
            change_id = creator.create_spec()
            self.console.print(f"  [green]✓[/green] 变更 ID: {change_id}")
            self.console.print("")

            # 3. 生成 AI 提示词
            self.console.print("[cyan]第 3 步: 生成 AI 提示词...[/cyan]")
            prompt_file = creator.generate_ai_prompt()
            self.console.print(f"  [green]✓[/green] 提示词: {prompt_file}")
            self.console.print("")

            # 完成
            self.console.print("[green]✓ 项目创建完成！[/green]")
            self.console.print("")
            self.console.print("[cyan]下一步:[/cyan]")
            self.console.print(f"  1. 查看生成的文档:")
            self.console.print(f"     - PRD: output/{project_name}-prd.md")
            self.console.print(f"     - 架构: output/{project_name}-architecture.md")
            self.console.print(f"     - UI/UX: output/{project_name}-uiux.md")
            self.console.print(f"  2. 查看规范: super-dev spec show {change_id}")
            self.console.print(f"  3. 复制 AI 提示词: cat {prompt_file}")
            self.console.print(f"  4. 开始开发: 回复 '开始' 或运行 'super-dev spec start {change_id}'")

        except Exception as e:
            self.console.print(f"[red]创建失败: {e}[/red]")
            import traceback
            self.console.print(traceback.format_exc())
            return 1

        return 0

    def _cmd_design(self, args) -> int:
        """设计智能引擎命令"""
        from .design import DesignIntelligenceEngine, DesignSystemGenerator, TokenGenerator

        if args.design_command == "search":
            # 搜索设计资产
            self.console.print(f"[cyan]搜索设计资产: {args.query}[/cyan]")

            engine = DesignIntelligenceEngine()

            result = engine.search(
                query=args.query,
                domain=args.domain,
                max_results=args.max_results,
            )

            # 显示结果
            if "error" in result:
                self.console.print(f"[red]搜索失败: {result['error']}[/red]")
                return 1

            domain_name = {
                "style": "风格",
                "color": "配色",
                "typography": "字体",
                "component": "组件",
                "layout": "布局",
                "animation": "动画",
                "ux": "UX 指南",
                "chart": "图表",
                "product": "产品",
            }.get(result["domain"], result["domain"])

            self.console.print(f"\n[green]找到 {result['count']} 个{domain_name}结果:[/green]\n")

            for idx, item in enumerate(result["results"], 1):
                relevance_color = {
                    "high": "green",
                    "medium": "yellow",
                    "low": "dim",
                }.get(item.get("relevance", "low"), "dim")

                self.console.print(f"[{relevance_color}]{idx}.[/] [bold]{item.get('name', item.get('Style Category', item.get('Font Pairing Name', 'N/A')))}[/] (相关度: {item.get('relevance', 'N/A')})")

                # 显示关键信息
                if "description" in item:
                    self.console.print(f"    {item['description']}")
                if "keywords" in item:
                    self.console.print(f"    关键词: {item['keywords']}")
                if "primary_colors" in item:
                    self.console.print(f"    色彩: {item['primary_colors']}")
                if "mood" in item:
                    self.console.print(f"    风格: {item['mood']}")

                self.console.print()

            return 0

        elif args.design_command == "generate":
            # 生成完整设计系统
            self.console.print(f"[cyan]生成设计系统[/cyan]")
            self.console.print(f"  产品: {args.product}")
            self.console.print(f"  行业: {args.industry}")
            self.console.print(f"  关键词: {' '.join(args.keywords) if args.keywords else 'N/A'}")
            self.console.print(f"  平台: {args.platform}")
            self.console.print()

            generator = DesignSystemGenerator()

            design_system = generator.generate(
                product_type=args.product,
                industry=args.industry,
                keywords=args.keywords or [],
                platform=args.platform,
                aesthetic=args.aesthetic,
            )

            self.console.print(f"[green]设计系统已生成:[/green]\n")
            self.console.print(f"  名称: {design_system.name}")
            self.console.print(f"  描述: {design_system.description}")

            if design_system.aesthetic:
                self.console.print(f"  美学方向: {design_system.aesthetic.name}")
                self.console.print(f"  差异化特征: {design_system.aesthetic.differentiation}")

            self.console.print(f"\n[cyan]生成文件...[/cyan]")

            output_dir = Path(args.output)
            generated_files = generator.generate_documentation(design_system, output_dir)

            for file_path in generated_files:
                self.console.print(f"  [green]✓[/green] {file_path}")

            self.console.print(f"\n[dim]下一步:[/dim]")
            self.console.print(f"  1. 查看 {output_dir / 'DESIGN_SYSTEM.md'} 了解设计系统")
            self.console.print(f"  2. 使用 {output_dir / 'design-tokens.css'} 导入 CSS 变量")
            self.console.print(f"  3. 使用 {output_dir / 'tailwind.config.json'} 配置 Tailwind")

            return 0

        elif args.design_command == "tokens":
            # 生成 design tokens
            self.console.print(f"[cyan]生成 design tokens[/cyan]")
            self.console.print(f"  主色: {args.primary}")
            self.console.print(f"  类型: {args.type}")
            self.console.print()

            token_gen = TokenGenerator()

            if args.format == "css":
                tokens = token_gen.generate_all_tokens(args.primary, args.type)

                css_content = [":root {"]
                css_content.append("  /* Colors */")

                for name, value in tokens["colors"].items():
                    css_content.append(f"  --color-{name}: {value};")

                css_content.append("")
                css_content.append("  /* Spacing */")

                for name, value in tokens["spacing"].items():
                    css_content.append(f"  --space-{name}: {value};")

                css_content.append("")
                css_content.append("  /* Shadows */")

                for name, value in tokens["shadows"].items():
                    css_content.append(f"  --shadow-{name}: {value};")

                css_content.append("}")

                output = "\n".join(css_content)

                if args.output:
                    Path(args.output).write_text(output, encoding="utf-8")
                    self.console.print(f"[green]✓[/green] 已保存到 {args.output}")
                else:
                    self.console.print(output)

            elif args.format == "json":
                tokens = token_gen.generate_all_tokens(args.primary, args.type)
                output = json.dumps(tokens, indent=2)

                if args.output:
                    Path(args.output).write_text(output, encoding="utf-8")
                    self.console.print(f"[green]✓[/green] 已保存到 {args.output}")
                else:
                    self.console.print(output)

            elif args.format == "tailwind":
                tokens = token_gen.generate_all_tokens(args.primary, args.type)

                tailwind_config = {
                    "theme": {
                        "extend": {
                            "colors": {f"{k}": v for k, v in tokens["colors"].items()},
                            "spacing": tokens["spacing"],
                            "boxShadow": tokens["shadows"],
                        }
                    }
                }

                output = json.dumps(tailwind_config, indent=2)

                if args.output:
                    Path(args.output).write_text(output, encoding="utf-8")
                    self.console.print(f"[green]✓[/green] 已保存到 {args.output}")
                else:
                    self.console.print(output)

            return 0

        elif args.design_command == "landing":
            # Landing 页面模式生成
            from .design import get_landing_generator

            landing_gen = get_landing_generator()

            # 列出所有模式
            if hasattr(args, 'list') and args.list:
                patterns = landing_gen.list_patterns()
                self.console.print(f"\n[green]可用的 Landing 页面模式 ({len(patterns)} 个):[/green]\n")
                for i, pattern in enumerate(patterns, 1):
                    self.console.print(f"  {i}. {pattern}")
                self.console.print()
                return 0

            # 智能推荐
            if hasattr(args, 'product_type') and args.product_type:
                self.console.print(f"[cyan]智能推荐 Landing 页面模式[/cyan]")
                self.console.print(f"  产品类型: {args.product_type}")
                self.console.print(f"  目标: {args.goal if hasattr(args, 'goal') and args.goal else 'N/A'}")
                self.console.print(f"  受众: {args.audience if hasattr(args, 'audience') and args.audience else 'N/A'}")
                self.console.print()

                recommended = landing_gen.recommend(
                    product_type=args.product_type,
                    goal=args.goal if hasattr(args, 'goal') and args.goal else "",
                    audience=args.audience if hasattr(args, 'audience') and args.audience else ""
                )

                if recommended:
                    self.console.print(f"[green]推荐模式: {recommended.name}[/green]")
                    self.console.print(f"  {recommended.description}")
                    self.console.print(f"  适合: {', '.join(recommended.best_for)}")
                    self.console.print(f"  复杂度: {recommended.complexity}")
                    self.console.print()
                    return 0

            # 搜索模式
            query = args.query if hasattr(args, 'query') and args.query else ""
            if query:
                self.console.print(f"[cyan]搜索 Landing 页面模式: {query}[/cyan]\n")

                results = landing_gen.search(query, max_results=args.max_results)

                if not results:
                    self.console.print("[yellow]未找到匹配的模式[/yellow]")
                    return 1

                self.console.print(f"[green]找到 {len(results)} 个结果:[/green]\n")

                for idx, pattern in enumerate(results, 1):
                    self.console.print(f"[cyan]{idx}. {pattern.name}[/cyan]")
                    self.console.print(f"    {pattern.description}")
                    self.console.print(f"    适合: {', '.join(pattern.best_for)}")
                    self.console.print(f"    复杂度: {pattern.complexity}")
                    self.console.print()

                return 0

            # 默认显示所有模式
            patterns = landing_gen.list_patterns()
            self.console.print(f"\n[green]可用的 Landing 页面模式 ({len(patterns)} 个):[/green]\n")
            for i, pattern in enumerate(patterns, 1):
                self.console.print(f"  {i}. {pattern}")
            self.console.print()
            return 0

        elif args.design_command == "chart":
            # 图表类型推荐
            from .design import get_chart_recommender

            chart_recommender = get_chart_recommender()

            # 列出所有图表类型
            if hasattr(args, 'list') and args.list:
                chart_types = chart_recommender.list_chart_types()
                categories = chart_recommender.list_categories()
                self.console.print(f"\n[green]可用的图表类型 ({len(chart_types)} 个, {len(categories)} 个类别):[/green]\n")
                for category in sorted(categories):
                    types = [ct for ct in chart_types if ct in [c.name for c in chart_recommender.chart_types if c.category.value == category]]
                    self.console.print(f"  [cyan]{category}:[/cyan]")
                    for ct in sorted(types):
                        self.console.print(f"    - {ct}")
                self.console.print()
                return 0

            # 推荐图表类型
            data_description = args.data_description if hasattr(args, 'data_description') else ""
            if data_description:
                self.console.print(f"[cyan]推荐图表类型[/cyan]")
                self.console.print(f"  数据描述: {data_description}")
                self.console.print(f"  框架: {args.framework}")
                self.console.print()

                recommendations = chart_recommender.recommend(
                    data_description=data_description,
                    framework=args.framework,
                    max_results=args.max_results
                )

                if not recommendations:
                    self.console.print("[yellow]未找到合适的图表类型[/yellow]")
                    return 1

                self.console.print(f"[green]推荐结果:[/green]\n")

                for idx, rec in enumerate(recommendations, 1):
                    confidence_pct = int(rec.confidence * 100)
                    self.console.print(f"[cyan]{idx}. {rec.chart_type.name}[/cyan] (置信度: {confidence_pct}%)")
                    self.console.print(f"    {rec.chart_type.description}")
                    self.console.print(f"    理由: {rec.reasoning}")
                    self.console.print(f"    推荐库: {rec.library_recommendation}")
                    self.console.print(f"    无障碍: {rec.chart_type.accessibility_notes}")
                    if rec.alternatives:
                        self.console.print(f"    替代方案: {', '.join([a.name for a in rec.alternatives])}")
                    self.console.print()

                return 0

            # 默认显示所有图表类型
            chart_types = chart_recommender.list_chart_types()
            self.console.print(f"\n[green]可用的图表类型 ({len(chart_types)} 个):[/green]\n")
            for i, ct in enumerate(chart_types, 1):
                self.console.print(f"  {i}. {ct}")
            self.console.print()
            return 0

        elif args.design_command == "ux":
            # UX 指南查询
            from .design import get_ux_guide

            ux_guide = get_ux_guide()

            # 列出所有领域
            if hasattr(args, 'list_domains') and args.list_domains:
                domains = ux_guide.list_domains()
                self.console.print(f"\n[green]UX 指南领域 ({len(domains)} 个):[/green]\n")
                for i, domain in enumerate(domains, 1):
                    self.console.print(f"  {i}. {domain}")
                self.console.print()
                return 0

            # 快速见效的改进
            if hasattr(args, 'quick_wins') and args.quick_wins:
                self.console.print(f"[cyan]快速见效的 UX 改进建议[/cyan]\n")

                quick_wins = ux_guide.get_quick_wins(max_results=args.max_results)

                if not quick_wins:
                    self.console.print("[yellow]未找到快速见效的建议[/yellow]")
                    return 1

                for idx, rec in enumerate(quick_wins, 1):
                    self.console.print(f"[cyan]{idx}. {rec.guideline.topic}[/cyan] ({rec.guideline.domain.value})")
                    self.console.print(f"    [green]最佳实践:[/green] {rec.guideline.best_practice}")
                    self.console.print(f"    [red]反模式:[/red] {rec.guideline.anti_pattern}")
                    self.console.print(f"    影响: {rec.guideline.impact}")
                    self.console.print(f"    优先级: {rec.priority} | 实现难度: {rec.implementation_effort} | 用户影响: {rec.user_impact}")
                    if rec.resources:
                        self.console.print(f"    资源: {', '.join(rec.resources)}")
                    self.console.print()

                return 0

            # 检查清单
            if hasattr(args, 'checklist') and args.checklist:
                self.console.print(f"[cyan]UX 检查清单[/cyan]\n")

                checklist = ux_guide.get_checklist(domains=[args.domain] if hasattr(args, 'domain') and args.domain else None)

                for domain, items in sorted(checklist.items()):
                    self.console.print(f"[cyan]{domain}:[/cyan]")
                    for item in items:
                        self.console.print(f"  {item}")
                    self.console.print()

                return 0

            # 搜索 UX 指南
            query = args.query if hasattr(args, 'query') and args.query else ""
            if query:
                self.console.print(f"[cyan]搜索 UX 指南: {query}[/cyan]\n")

                recommendations = ux_guide.search(
                    query=query,
                    domain=args.domain if hasattr(args, 'domain') else None,
                    max_results=args.max_results
                )

                if not recommendations:
                    self.console.print("[yellow]未找到匹配的 UX 指南[/yellow]")
                    return 1

                self.console.print(f"[green]找到 {len(recommendations)} 个结果:[/green]\n")

                for idx, rec in enumerate(recommendations, 1):
                    self.console.print(f"[cyan]{idx}. {rec.guideline.topic}[/cyan] ({rec.guideline.domain.value})")
                    self.console.print(f"    [green]最佳实践:[/green] {rec.guideline.best_practice}")
                    self.console.print(f"    [red]反模式:[/red] {rec.guideline.anti_pattern}")
                    self.console.print(f"    影响: {rec.guideline.impact}")
                    self.console.print(f"    优先级: {rec.priority} | 实现难度: {rec.implementation_effort} | 用户影响: {rec.user_impact}")
                    if rec.resources:
                        self.console.print(f"    资源: {', '.join(rec.resources)}")
                    self.console.print()

                return 0

            # 默认显示所有领域
            domains = ux_guide.list_domains()
            self.console.print(f"\n[green]UX 指南领域 ({len(domains)} 个):[/green]\n")
            for i, domain in enumerate(domains, 1):
                self.console.print(f"  {i}. {domain}")
            self.console.print()
            return 0

        elif args.design_command == "stack":
            # 技术栈最佳实践
            from .design import get_tech_stack_engine

            tech_engine = get_tech_stack_engine()

            # 列出所有技术栈
            if hasattr(args, 'list') and args.list:
                stacks = tech_engine.list_stacks()
                self.console.print(f"\n[green]支持的技术栈 ({len(stacks)} 个):[/green]\n")
                for i, stack in enumerate(stacks, 1):
                    self.console.print(f"  {i}. {stack}")
                self.console.print()
                return 0

            # 查询参数
            stack_name = args.stack
            query = args.query if hasattr(args, 'query') and args.query else None
            category = args.category if hasattr(args, 'category') else None

            # 显示设计模式
            if hasattr(args, 'patterns') and args.patterns:
                patterns = tech_engine.get_patterns(stack_name)

                if not patterns:
                    self.console.print(f"[yellow]未找到 {stack_name} 的设计模式[/yellow]")
                    return 1

                self.console.print(f"\n[cyan]{stack_name} 设计模式 ({len(patterns)} 个):[/cyan]\n")

                for idx, pattern in enumerate(patterns, 1):
                    self.console.print(f"[cyan]{idx}. {pattern.name}[/cyan]")
                    self.console.print(f"    描述: {pattern.description}")
                    self.console.print(f"    使用场景: {pattern.use_case}")
                    if pattern.pros:
                        self.console.print(f"    优点: {', '.join(pattern.pros)}")
                    if pattern.cons:
                        self.console.print(f"    缺点: {', '.join(pattern.cons)}")
                    self.console.print()

                return 0

            # 显示性能优化建议
            if hasattr(args, 'performance') and args.performance:
                tips = tech_engine.get_performance_tips(stack_name)

                if not tips:
                    self.console.print(f"[yellow]未找到 {stack_name} 的性能建议[/yellow]")
                    return 1

                self.console.print(f"\n[cyan]{stack_name} 性能优化建议 ({len(tips)} 个):[/cyan]\n")

                for idx, tip in enumerate(tips, 1):
                    self.console.print(f"[cyan]{idx}. {tip.topic} - {tip.technique}[/cyan]")
                    self.console.print(f"    描述: {tip.description}")
                    self.console.print(f"    影响: {tip.impact} | 实施难度: {tip.effort}")
                    if tip.code_snippet:
                        self.console.print(f"    代码示例:\n    [dim]{tip.code_snippet}[/dim]")
                    self.console.print()

                return 0

            # 快速见效的性能优化
            if hasattr(args, 'quick_wins') and args.quick_wins:
                tips = tech_engine.get_quick_wins(stack_name)

                if not tips:
                    self.console.print(f"[yellow]未找到 {stack_name} 的快速性能优化[/yellow]")
                    return 1

                self.console.print(f"\n[cyan]{stack_name} 快速见效的性能优化 ({len(tips)} 个):[/cyan]\n")

                for idx, tip in enumerate(tips, 1):
                    self.console.print(f"[cyan]{idx}. {tip.topic} - {tip.technique}[/cyan]")
                    self.console.print(f"    描述: {tip.description}")
                    if tip.code_snippet:
                        self.console.print(f"    代码示例:\n    [dim]{tip.code_snippet}[/dim]")
                    self.console.print()

                return 0

            # 搜索最佳实践
            self.console.print(f"[cyan]搜索 {stack_name} 最佳实践[/cyan]\n")

            if query:
                self.console.print(f"查询: {query}\n")

            recommendations = tech_engine.search_practices(
                stack=stack_name,
                query=query,
                category=category,
                max_results=args.max_results
            )

            if not recommendations:
                self.console.print("[yellow]未找到匹配的最佳实践[/yellow]")
                return 1

            for idx, rec in enumerate(recommendations, 1):
                self.console.print(f"[cyan]{idx}. {rec.practice.topic}[/cyan] ({rec.practice.category.value})")
                self.console.print(f"    [green]最佳实践:[/green] {rec.practice.practice}")
                self.console.print(f"    [red]反模式:[/red] {rec.practice.anti_pattern}")
                self.console.print(f"    好处: {rec.practice.benefits}")
                self.console.print(f"    优先级: {rec.priority} | 复杂度: {rec.practice.complexity}")
                if rec.context:
                    self.console.print(f"    上下文: {rec.context}")
                if rec.alternatives:
                    self.console.print(f"    替代方案: {', '.join(rec.alternatives)}")
                if rec.resources:
                    self.console.print(f"    资源: {', '.join(rec.resources)}")
                if rec.practice.code_example:
                    self.console.print(f"    代码示例:\n    [dim]{rec.practice.code_example[:200]}...[/dim]")
                self.console.print()

            return 0

        elif args.design_command == "codegen":
            # 代码片段生成
            from .design import get_code_generator
            from .design.codegen import Framework

            codegen = get_code_generator()

            # 列出所有可用组件
            if hasattr(args, 'list') and args.list:
                components = codegen.get_available_components(
                    framework=Framework(args.framework) if hasattr(args, 'framework') else None
                )

                self.console.print(f"\n[green]可用组件 ({args.framework or 'all'}):[/green]\n")

                for category, comp_list in sorted(components.items()):
                    self.console.print(f"[cyan]{category}:[/cyan]")
                    for comp in comp_list:
                        self.console.print(f"  - {comp}")
                    self.console.print()

                return 0

            # 搜索组件
            if hasattr(args, 'search') and args.search:
                results = codegen.search_components(
                    query=args.search,
                    framework=args.framework if hasattr(args, 'framework') else None
                )

                if not results:
                    self.console.print(f"[yellow]未找到匹配的组件: {args.search}[/yellow]")
                    return 1

                self.console.print(f"\n[green]找到 {len(results)} 个组件:[/green]\n")

                for idx, snippet in enumerate(results, 1):
                    self.console.print(f"[cyan]{idx}. {snippet.name}[/cyan] ({snippet.framework.value})")
                    self.console.print(f"    类别: {snippet.category.value}")
                    self.console.print(f"    描述: {snippet.description}")
                    self.console.print(f"    依赖: {', '.join(snippet.dependencies)}")
                    if snippet.preview:
                        self.console.print(f"    预览: [dim]{snippet.preview}[/dim]")
                    self.console.print()

                return 0

            # 生成组件代码
            component_name = args.component
            framework = args.framework if hasattr(args, 'framework') else "react"

            self.console.print(f"[cyan]生成 {component_name} 组件 ({framework})[/cyan]\n")

            component = codegen.generate_component(
                component_name=component_name,
                framework=Framework(framework)
            )

            if not component:
                self.console.print(f"[yellow]未找到组件: {component_name}[/yellow]")
                self.console.print(f"使用 --list 查看可用组件")
                return 1

            self.console.print(f"[green]组件名称:[/green] {component_name}")
            self.console.print(f"[green]描述:[/green] {component.description}\n")

            self.console.print(f"[cyan]代码:[/cyan]")
            self.console.print(f"```{framework}")
            self.console.print(component.code)
            self.console.print("```\n")

            if component.imports:
                self.console.print(f"[cyan]导入语句:[/cyan]")
                for imp in component.imports:
                    self.console.print(f"  {imp}")
                self.console.print()

            if component.dependencies:
                self.console.print(f"[cyan]依赖:[/cyan]")
                self.console.print(f"  {', '.join(component.dependencies)}")
                self.console.print()

            if component.usage_example:
                self.console.print(f"[cyan]使用示例:[/cyan]")
                self.console.print(f"  [dim]{component.usage_example}[/dim]")

            # 输出到文件
            if hasattr(args, 'output') and args.output:
                output_path = Path(args.output)
                output_path.parent.mkdir(parents=True, exist_ok=True)

                with open(output_path, 'w', encoding='utf-8') as f:
                    f.write(component.code)

                self.console.print(f"\n[green]已保存到: {output_path}[/green]")

            return 0

        else:
            self.console.print("[yellow]请指定设计子命令[/yellow]")
            self.console.print("  可用命令: search, generate, tokens, landing, chart, ux, stack, codegen")
            self.console.print("  使用 'super-dev design <command> -h' 查看帮助")
            return 1

    def _cmd_pipeline(self, args) -> int:
        """运行完整流水线 - 从想法到部署"""
        # 确定项目名称
        project_name = args.name
        if not project_name:
            import re
            words = re.findall(r'[\w]+', args.description)
            if words:
                project_name = '-'.join(words[:3]).lower()
            else:
                project_name = "my-project"

        tech_stack = {
            "platform": args.platform,
            "frontend": args.frontend,
            "backend": args.backend,
            "domain": args.domain,
        }

        project_dir = Path.cwd()

        self.console.print(f"[cyan]{'=' * 60}[/cyan]")
        self.console.print(f"[cyan]Super Dev 完整流水线[/cyan]")
        self.console.print(f"[cyan]{'=' * 60}[/cyan]")
        self.console.print(f"[dim]项目: {project_name}[/dim]")
        self.console.print(f"[dim]技术栈: {args.platform} | {args.frontend} | {args.backend}[/dim]")
        self.console.print("")

        try:
            # ========== 第 1 阶段: 生成文档 ==========
            self.console.print("[cyan]第 1 阶段: 生成专业文档...[/cyan]")
            from .creators import DocumentGenerator

            doc_generator = DocumentGenerator(
                name=project_name,
                description=args.description,
                platform=args.platform,
                frontend=args.frontend,
                backend=args.backend,
                domain=args.domain
            )

            # 生成文档内容
            prd_content = doc_generator.generate_prd()
            arch_content = doc_generator.generate_architecture()
            uiux_content = doc_generator.generate_uiux()

            # 创建输出目录并写入文件
            output_dir = project_dir / "output"
            output_dir.mkdir(parents=True, exist_ok=True)

            prd_file = output_dir / f"{project_name}-prd.md"
            arch_file = output_dir / f"{project_name}-architecture.md"
            uiux_file = output_dir / f"{project_name}-uiux.md"

            prd_file.write_text(prd_content, encoding="utf-8")
            arch_file.write_text(arch_content, encoding="utf-8")
            uiux_file.write_text(uiux_content, encoding="utf-8")

            self.console.print(f"  [green]✓[/green] PRD: {prd_file}")
            self.console.print(f"  [green]✓[/green] 架构: {arch_file}")
            self.console.print(f"  [green]✓[/green] UI/UX: {uiux_file}")
            self.console.print("")

            # ========== 第 2 阶段: 创建 Spec ==========
            self.console.print("[cyan]第 2 阶段: 创建 Spec 规范...[/cyan]")
            from .creators import SpecBuilder

            spec_builder = SpecBuilder(
                project_dir=project_dir,
                name=project_name,
                description=args.description
            )

            requirements = doc_generator.extract_requirements()
            change_id = spec_builder.create_change(requirements, tech_stack)

            self.console.print(f"  [green]✓[/green] 变更 ID: {change_id}")
            self.console.print(f"  [green]✓[/green] Spec: .super-dev/changes/{change_id}/")
            self.console.print("")

            # ========== 第 3 阶段: 红队审查 ==========
            redteam_report = None
            if not args.skip_redteam:
                self.console.print("[cyan]第 3 阶段: 红队审查...[/cyan]")
                from .reviewers import RedTeamReviewer

                reviewer = RedTeamReviewer(
                    project_dir=project_dir,
                    name=project_name,
                    tech_stack=tech_stack
                )
                redteam_report = reviewer.review()

                # 保存红队审查报告
                redteam_file = project_dir / "output" / f"{project_name}-redteam.md"
                redteam_file.parent.mkdir(parents=True, exist_ok=True)
                redteam_file.write_text(redteam_report.to_markdown(), encoding="utf-8")

                self.console.print(f"  [green]✓[/green] 安全问题: {sum(1 for i in redteam_report.security_issues if i.severity in ('critical', 'high'))} high/critical")
                self.console.print(f"  [green]✓[/green] 性能问题: {sum(1 for i in redteam_report.performance_issues if i.severity in ('critical', 'high'))} high/critical")
                self.console.print(f"  [green]✓[/green] 架构问题: {sum(1 for i in redteam_report.architecture_issues if i.severity in ('critical', 'high'))} high/critical")
                self.console.print(f"  [green]✓[/green] 总分: {redteam_report.total_score}/100")
                self.console.print(f"  [green]✓[/green] 报告: {redteam_file}")
                self.console.print("")
            else:
                self.console.print("[yellow]第 3 阶段: 红队审查 (跳过)[/yellow]")
                self.console.print("")

            # ========== 第 4 阶段: 质量门禁 ==========
            self.console.print("[cyan]第 4 阶段: 质量门禁检查...[/cyan]")
            from .reviewers import QualityGateChecker

            gate_checker = QualityGateChecker(
                project_dir=project_dir,
                name=project_name,
                tech_stack=tech_stack
            )

            gate_result = gate_checker.check(redteam_report)

            # 显示场景信息
            scenario_label = "0-1 新建项目" if gate_result.scenario == "0-1" else "1-N+1 增量开发"
            if gate_result.scenario == "0-1":
                self.console.print(f"  [dim]场景: {scenario_label} (使用放宽标准)[/dim]")

            # 保存质量门禁报告
            gate_file = project_dir / "output" / f"{project_name}-quality-gate.md"
            gate_file.parent.mkdir(parents=True, exist_ok=True)
            gate_file.write_text(gate_result.to_markdown(), encoding="utf-8")

            status = "[green]通过[/green]" if gate_result.passed else "[red]未通过[/red]"
            self.console.print(f"  {status} 总分: {gate_result.total_score}/100")
            self.console.print(f"  [green]✓[/green] 报告: {gate_file}")
            self.console.print("")

            # 质量门禁未通过，停止流水线
            if not gate_result.passed:
                self.console.print("[red]质量门禁未通过，流水线终止[/red]")
                self.console.print("[cyan]请修复以下问题后重新运行:[/cyan]")
                for failure in gate_result.critical_failures:
                    self.console.print(f"  - {failure}")
                return 1

            # ========== 第 5 阶段: 代码审查指南 ==========
            self.console.print("[cyan]第 5 阶段: 生成代码审查指南...[/cyan]")
            from .reviewers import CodeReviewGenerator

            review_gen = CodeReviewGenerator(
                project_dir=project_dir,
                name=project_name,
                tech_stack=tech_stack
            )

            review_guide = review_gen.generate()
            review_file = project_dir / "output" / f"{project_name}-code-review.md"
            review_file.write_text(review_guide, encoding="utf-8")

            self.console.print(f"  [green]✓[/green] 代码审查指南: {review_file}")
            self.console.print("")

            # ========== 第 6 阶段: AI 提示词 ==========
            self.console.print("[cyan]第 6 阶段: 生成 AI 提示词...[/cyan]")
            from .creators import AIPromptGenerator

            prompt_gen = AIPromptGenerator(
                project_dir=project_dir,
                name=project_name
            )

            prompt_content = prompt_gen.generate()
            prompt_file = project_dir / "output" / f"{project_name}-ai-prompt.md"
            prompt_file.write_text(prompt_content, encoding="utf-8")

            self.console.print(f"  [green]✓[/green] AI 提示词: {prompt_file}")
            self.console.print("")

            # ========== 第 7 阶段: CI/CD 配置 ==========
            self.console.print(f"[cyan]第 7 阶段: 生成 CI/CD 配置 ({args.cicd.upper()})...[/cyan]")
            from .deployers import CICDGenerator

            cicd_gen = CICDGenerator(
                project_dir=project_dir,
                name=project_name,
                tech_stack=tech_stack,
                platform=args.cicd
            )

            cicd_files = cicd_gen.generate()

            for file_path, content in cicd_files.items():
                full_path = project_dir / file_path
                full_path.parent.mkdir(parents=True, exist_ok=True)
                full_path.write_text(content, encoding="utf-8")
                self.console.print(f"  [green]✓[/green] {file_path}")

            self.console.print("")

            # ========== 第 8 阶段: 数据库迁移 ==========
            self.console.print("[cyan]第 8 阶段: 生成数据库迁移脚本...[/cyan]")
            from .deployers import MigrationGenerator

            migration_gen = MigrationGenerator(
                project_dir=project_dir,
                name=project_name,
                tech_stack=tech_stack
            )

            migration_files = migration_gen.generate()

            for file_path, content in migration_files.items():
                full_path = project_dir / file_path
                full_path.parent.mkdir(parents=True, exist_ok=True)
                full_path.write_text(content, encoding="utf-8")
                self.console.print(f"  [green]✓[/green] {file_path}")

            self.console.print("")

            # ========== 完成 ==========
            self.console.print(f"[cyan]{'=' * 60}[/cyan]")
            self.console.print("[green]✓ 流水线完成！[/green]")
            self.console.print(f"[cyan]{'=' * 60}[/cyan]")
            self.console.print("")
            self.console.print("[cyan]生成的文件:[/cyan]")
            self.console.print("  文档:")
            self.console.print(f"    - PRD: output/{project_name}-prd.md")
            self.console.print(f"    - 架构: output/{project_name}-architecture.md")
            self.console.print(f"    - UI/UX: output/{project_name}-uiux.md")
            if not args.skip_redteam:
                self.console.print(f"    - 红队审查: output/{project_name}-redteam.md")
            self.console.print(f"    - 质量门禁: output/{project_name}-quality-gate.md")
            self.console.print(f"    - 代码审查: output/{project_name}-code-review.md")
            self.console.print(f"    - AI 提示词: output/{project_name}-ai-prompt.md")
            self.console.print("")
            self.console.print("  CI/CD:")
            for file_path in cicd_files.keys():
                self.console.print(f"    - {file_path}")
            self.console.print("")
            self.console.print("  数据库迁移:")
            for file_path in migration_files.keys():
                self.console.print(f"    - {file_path}")
            self.console.print("")
            self.console.print("[cyan]下一步:[/cyan]")
            self.console.print("  1. 查看生成的文档和审查报告")
            self.console.print("  2. 复制 AI 提示词给 AI 编码助手开始开发")
            self.console.print("  3. 使用代码审查指南进行代码审查")
            self.console.print("  4. 配置 CI/CD 平台 (设置 secrets/credentials)")
            self.console.print("  5. 运行数据库迁移脚本")
            self.console.print("  6. 推送代码触发 CI/CD 流水线")
            self.console.print("")

        except Exception as e:
            self.console.print(f"[red]流水线失败: {e}[/red]")
            import traceback
            self.console.print(traceback.format_exc())
            return 1

        return 0

    def _cmd_config(self, args) -> int:
        """配置管理"""
        config_manager = get_config_manager()

        if not config_manager.exists():
            self.console.print("[red]未找到项目配置[/red]")
            return 1

        if args.action == "list":
            # 列出所有配置
            config = config_manager.config
            self.console.print(f"[cyan]项目配置:[/cyan]")
            for key, value in config.__dict__.items():
                if not key.startswith("_"):
                    self.console.print(f"  {key}: {value}")

        elif args.action == "get":
            if not args.key:
                self.console.print("[red]请指定配置键[/red]")
                return 1
            value = config_manager.get(args.key)
            self.console.print(f"{args.key}: {value}")

        elif args.action == "set":
            if not args.key or not args.value:
                self.console.print("[red]请指定配置键和值[/red]")
                return 1
            config_manager.update(**{args.key: args.value})
            self.console.print(f"[green]✓[/green] {args.key} = {args.value}")

        return 0

    def _cmd_spec(self, args) -> int:
        """Spec-Driven Development 命令"""
        from .specs import SpecGenerator, ChangeManager, SpecManager
        from .specs.models import ChangeStatus

        project_dir = Path.cwd()

        if args.spec_action == "init":
            # 初始化 SDD 目录结构
            generator = SpecGenerator(project_dir)
            agents_path, project_path = generator.init_sdd()

            self.console.print("[green]✓[/green] SDD 目录结构已初始化")
            self.console.print(f"  [dim].super-dev/specs/[/dim] - 当前规范")
            self.console.print(f"  [dim].super-dev/changes/[/dim] - 变更提案")
            self.console.print(f"  [dim].super-dev/archive/[/dim] - 已归档变更")
            self.console.print("")
            self.console.print(f"[cyan]下一步:[/cyan]")
            self.console.print(f"  1. 编辑 .super-dev/project.md 填写项目上下文")
            self.console.print(f"  2. 运行 'super-dev spec propose <id>' 创建变更提案")

        elif args.spec_action == "list":
            # 列出所有变更
            manager = ChangeManager(project_dir)
            status_filter = None
            if args.status:
                status_filter = ChangeStatus(args.status)

            changes = manager.list_changes(status=status_filter)

            if not changes:
                self.console.print("[dim]没有找到变更[/dim]")
                return 0

            self.console.print(f"[cyan]变更列表:[/cyan]")
            for change in changes:
                status_color = {
                    ChangeStatus.DRAFT: "dim",
                    ChangeStatus.PROPOSED: "yellow",
                    ChangeStatus.APPROVED: "blue",
                    ChangeStatus.IN_PROGRESS: "cyan",
                    ChangeStatus.COMPLETED: "green",
                    ChangeStatus.ARCHIVED: "dim",
                }.get(change.status, "white")

                self.console.print(
                    f"  [{status_color}]{change.id}[/] - {change.title} "
                    f"({change.status.value})"
                )
                if change.tasks:
                    rate = change.completion_rate
                    self.console.print(f"    [dim]进度: {rate:.0f}% ({sum(1 for t in change.tasks if t.status.value == 'completed')}/{len(change.tasks)} 任务)[/dim]")

        elif args.spec_action == "show":
            # 显示变更详情
            manager = ChangeManager(project_dir)
            change = manager.load_change(args.change_id)

            if not change:
                self.console.print(f"[red]变更不存在: {args.change_id}[/red]")
                return 1

            self.console.print(f"[cyan]变更详情: {change.id}[/cyan]")
            self.console.print(f"  标题: {change.title}")
            self.console.print(f"  状态: {change.status.value}")

            if change.proposal:
                self.console.print("")
                self.console.print("[cyan]提案:[/cyan]")
                if change.proposal.description:
                    self.console.print(f"  {change.proposal.description}")
                if change.proposal.motivation:
                    self.console.print(f"[dim]动机: {change.proposal.motivation}[/dim]")

            if change.tasks:
                self.console.print("")
                self.console.print("[cyan]任务:[/cyan]")
                for task in change.tasks:
                    checkbox = "[x]" if task.status.value == "completed" else "[ ]"
                    self.console.print(f"  {checkbox} {task.id}: {task.title}")

            if change.spec_deltas:
                self.console.print("")
                self.console.print("[cyan]规范变更:[/cyan]")
                for delta in change.spec_deltas:
                    self.console.print(f"  - {delta.spec_name} ({delta.delta_type.value})")

        elif args.spec_action == "propose":
            # 创建变更提案
            generator = SpecGenerator(project_dir)
            change = generator.create_change(
                change_id=args.change_id,
                title=args.title,
                description=args.description,
                motivation=args.motivation or "",
                impact=args.impact or ""
            )

            self.console.print(f"[green]✓[/green] 变更提案已创建: {change.id}")
            self.console.print(f"  [dim].super-dev/changes/{change.id}/[/dim]")
            self.console.print("")
            self.console.print(f"[cyan]下一步:[/cyan]")
            self.console.print(f"  1. 运行 'super-dev spec add-req {change.id} <spec> <req> <desc>' 添加需求")
            self.console.print(f"  2. 或 'super-dev spec show {change.id}' 查看详情")

        elif args.spec_action == "add-req":
            # 向变更添加需求
            generator = SpecGenerator(project_dir)
            delta = generator.add_requirement_to_change(
                change_id=args.change_id,
                spec_name=args.spec_name,
                requirement_name=args.req_name,
                description=args.description
            )

            self.console.print(f"[green]✓[/green] 需求已添加到变更")
            self.console.print(f"  规范: {delta.spec_name}")
            self.console.print(f"  需求: {args.req_name}")

        elif args.spec_action == "archive":
            # 归档变更
            if not args.yes:
                self.console.print(f"[yellow]即将归档变更: {args.change_id}[/yellow]")
                self.console.print("[dim]这将把规范增量合并到主规范中[/dim]")
                response = input("确认? (y/N): ")
                if response.lower() != "y":
                    self.console.print("[dim]已取消[/dim]")
                    return 0

            change_manager = ChangeManager(project_dir)
            spec_manager = SpecManager(project_dir)

            try:
                change = change_manager.archive_change(args.change_id, spec_manager)
                self.console.print(f"[green]✓[/green] 变更已归档: {change.id}")
                self.console.print(f"  [dim].super-dev/archive/{change.id}/[/dim]")
            except FileNotFoundError as e:
                self.console.print(f"[red]{e}[/red]")
                return 1
            except Exception as e:
                self.console.print(f"[red]归档失败: {e}[/red]")
                return 1

        elif args.spec_action == "validate":
            # 验证规格格式
            from .specs import SpecValidator

            validator = SpecValidator(project_dir)

            if args.change_id:
                # 验证单个变更
                result = validator.validate_change(args.change_id)
                self.console.print(f"[cyan]验证变更: {args.change_id}[/cyan]")
            else:
                # 验证所有变更
                result = validator.validate_all()
                self.console.print("[cyan]验证所有变更[/cyan]")

            self.console.print(result.to_summary())

            if args.verbose or (not result.is_valid):
                # 显示详细信息
                for error in result.errors:
                    self.console.print(
                        f"  [red]错误[/red]: {error.message}"
                    )
                    if error.line > 0:
                        self.console.print(
                            f"    [dim]{error.file}:{error.line}[/dim]"
                        )

                for warning in result.warnings:
                    self.console.print(
                        f"  [yellow]警告[/yellow]: {warning.message}"
                    )
                    if warning.line > 0:
                        self.console.print(
                            f"    [dim]{warning.file}:{warning.line}[/dim]"
                        )

            return 0 if result.is_valid else 1

        elif args.spec_action == "view":
            # 交互式仪表板
            from rich.console import Console
            from rich.table import Table
            from rich.panel import Panel
            from rich.text import Text

            console = Console()
            change_manager = ChangeManager(project_dir)
            spec_manager = SpecManager(project_dir)

            # 获取所有变更和规范
            changes = change_manager.list_changes()
            specs = spec_manager.list_specs()

            # 标题
            title = Text.assemble(
                ("Super Dev ", "bold cyan"),
                ("Spec Dashboard", "bold white"),
            )
            console.print(Panel(title, padding=(0, 1)))

            # 变更统计
            if changes:
                table = Table(title="活跃变更", show_header=True, header_style="bold magenta")
                table.add_column("ID", style="cyan", width=20)
                table.add_column("标题", style="white", width=30)
                table.add_column("状态", style="yellow", width=12)
                table.add_column("进度", style="green", width=10)
                table.add_column("任务", style="blue", width=8)

                for change in changes:
                    progress = f"{change.completion_rate:.0f}%"
                    tasks = f"{sum(1 for t in change.tasks if t.status.value == 'completed')}/{len(change.tasks)}"
                    table.add_row(
                        change.id,
                        change.title or "(无标题)",
                        change.status.value,
                        progress,
                        tasks
                    )

                console.print(table)
            else:
                console.print("[dim]没有活跃变更[/dim]")

            # 规范列表
            if specs:
                console.print("")
                specs_table = Table(title="当前规范", show_header=True, header_style="bold green")
                specs_table.add_column("规范名称", style="cyan", width=30)
                specs_table.add_column("文件路径", style="dim", width=50)

                for spec_name in specs:
                    spec_path = spec_manager.get_spec_path(spec_name)
                    specs_table.add_row(spec_name, str(spec_path.relative_to(project_dir)))

                console.print(specs_table)

            # 统计信息
            console.print("")
            stats_table = Table(show_header=False, box=None)
            stats_table.add_column("指标", style="bold white")
            stats_table.add_column("数量", style="cyan")

            stats_table.add_row("活跃变更", str(len(changes)))
            stats_table.add_row("规范文件", str(len(specs)))
            stats_table.add_row("待处理任务", str(sum(1 for c in changes for t in c.tasks if t.status.value == "pending")))

            console.print(stats_table)

            return 0

        else:
            self.console.print("[yellow]请指定 SDD 命令[/yellow]")
            return 1

        return 0

    # ==================== 辅助方法 ====================

    def _print_banner(self) -> None:
        """打印欢迎横幅"""
        if self.console:
            banner = Text()
            banner.append("Super Dev ", style="bold cyan")
            banner.append(f"v{__version__}\n", style="dim")
            banner.append(__description__, style="white")

            self.console.print(Panel.fit(
                banner,
                title="Super Dev",
                border_style="cyan"
            ))


def main() -> int:
    """主入口"""
    cli = SuperDevCLI()
    return cli.run()


if __name__ == "__main__":
    sys.exit(main())
