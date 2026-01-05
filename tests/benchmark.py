# -*- coding: utf-8 -*-
"""
Super Dev 性能基准测试
用于测量 CLI 命令和工作流的执行性能
"""

import asyncio
import time
import statistics
from pathlib import Path
from typing import Callable, List

try:
    from rich.console import Console
    from rich.table import Table
    RICH_AVAILABLE = True
except ImportError:
    RICH_AVAILABLE = False

from super_dev.config import ConfigManager, ProjectConfig
from super_dev.orchestrator import WorkflowEngine, Phase, WorkflowContext


class PerformanceBenchmark:
    """性能基准测试"""

    def __init__(self):
        self.console = Console() if RICH_AVAILABLE else None
        self.results = {}

    def benchmark(
        self,
        name: str,
        func: Callable,
        iterations: int = 10,
        warmup: int = 3
    ) -> dict:
        """
        执行基准测试

        Args:
            name: 测试名称
            func: 要测试的函数
            iterations: 迭代次数
            warmup: 预热次数

        Returns:
            性能指标字典
        """
        # 预热
        for _ in range(warmup):
            func()

        # 正式测试
        times = []
        for _ in range(iterations):
            start = time.perf_counter()
            func()
            end = time.perf_counter()
            times.append(end - start)

        # 计算统计数据
        result = {
            "name": name,
            "iterations": iterations,
            "min": min(times),
            "max": max(times),
            "mean": statistics.mean(times),
            "median": statistics.median(times),
            "stdev": statistics.stdev(times) if len(times) > 1 else 0,
            "percentile_95": statistics.quantiles(times, n=20)[18] if len(times) > 1 else times[0],
            "percentile_99": statistics.quantiles(times, n=100)[98] if len(times) > 1 else times[0],
        }

        self.results[name] = result
        return result

    def print_results(self):
        """打印测试结果"""
        if not RICH_AVAILABLE:
            print("性能基准测试结果:")
            for name, result in self.results.items():
                print(f"\n{name}:")
                print(f"  平均: {result['mean']*1000:.2f}ms")
                print(f"  中位数: {result['median']*1000:.2f}ms")
                print(f"  95th: {result['percentile_95']*1000:.2f}ms")
            return

        table = Table(title="性能基准测试结果")
        table.add_column("测试", style="cyan")
        table.add_column("平均", style="green")
        table.add_column("中位数", style="blue")
        table.add_column("最小", style="yellow")
        table.add_column("最大", style="red")
        table.add_column("95th", style="magenta")

        for name, result in self.results.items():
            table.add_row(
                name,
                f"{result['mean']*1000:.2f}ms",
                f"{result['median']*1000:.2f}ms",
                f"{result['min']*1000:.2f}ms",
                f"{result['max']*1000:.2f}ms",
                f"{result['percentile_95']*1000:.2f}ms"
            )

        self.console.print(table)

    def check_thresholds(self, thresholds: dict) -> bool:
        """
        检查是否满足性能阈值

        Args:
            thresholds: {test_name: max_time_ms}

        Returns:
            是否所有测试都满足阈值
        """
        all_pass = True

        if RICH_AVAILABLE:
            from rich.panel import Panel

        for name, max_ms in thresholds.items():
            if name not in self.results:
                if self.console:
                    self.console.print(f"[yellow]⚠[/yellow] {name}: 未运行")
                continue

            actual_ms = self.results[name]["mean"] * 1000
            passed = actual_ms <= max_ms

            if passed:
                if self.console:
                    self.console.print(
                        f"[green]✓[/green] {name}: {actual_ms:.2f}ms ≤ {max_ms}ms"
                    )
            else:
                all_pass = False
                if self.console:
                    self.console.print(
                        f"[red]✗[/red] {name}: {actual_ms:.2f}ms > {max_ms}ms (超出 {(actual_ms-max_ms):.2f}ms)"
                    )

        return all_pass


# ==================== 基准测试套件 ====================

class BenchmarkSuite:
    """基准测试套件"""

    def __init__(self, temp_dir: Path):
        self.temp_dir = temp_dir
        self.benchmark = PerformanceBenchmark()

    def run_all(self):
        """运行所有基准测试"""
        if self.benchmark.console:
            self.benchmark.console.print("\n[bold cyan]Super Dev 性能基准测试[/bold cyan]\n")

        self.benchmark_config_load()
        self.benchmark_config_save()
        self.benchmark_config_validate()
        self.benchmark_workflow_engine_init()
        self.benchmark_phase_execution()

        self.benchmark.print_results()

        # 检查性能目标
        return self.benchmark.check_thresholds({
            "Config Load": 50,
            "Config Save": 50,
            "Config Validate": 100,
            "Engine Init": 100,
            "Phase Execution": 5000,
        })

    def benchmark_config_load(self):
        """测试配置加载"""
        config_path = self.temp_dir / "super-dev.yaml"

        def setup():
            import yaml
            config_path.write_text(yaml.dump({"name": "test", "platform": "web"}))

        def func():
            manager = ConfigManager(self.temp_dir)
            _ = manager.config

        setup()
        self.benchmark.benchmark("Config Load", func, iterations=20)

    def benchmark_config_save(self):
        """测试配置保存"""
        def func():
            manager = ConfigManager(self.temp_dir)
            config = ProjectConfig(name="bench-test")
            manager.save(config)

        self.benchmark.benchmark("Config Save", func, iterations=20)

    def benchmark_config_validate(self):
        """测试配置验证"""
        def func():
            manager = ConfigManager(self.temp_dir)
            manager.create(name="test")
            _ = manager.validate()

        self.benchmark.benchmark("Config Validate", func, iterations=10)

    def benchmark_workflow_engine_init(self):
        """测试工作流引擎初始化"""
        def func():
            engine = WorkflowEngine(self.temp_dir)
            _ = engine.project_dir

        self.benchmark.benchmark("Engine Init", func, iterations=50)

    def benchmark_phase_execution(self):
        """测试阶段执行"""
        engine = WorkflowEngine(self.temp_dir)

        async def test_handler(context):
            # 模拟简单处理
            await asyncio.sleep(0.01)
            return {"result": "ok"}

        engine.register_phase_handler(Phase.DISCOVERY, test_handler)

        async def func():
            config = ConfigManager(self.temp_dir).create(name="bench-test")
            context = WorkflowContext(
                project_dir=self.temp_dir,
                config=config  # type: ignore
            )
            await engine._run_phase(Phase.DISCOVERY, context)

        def wrapper():
            asyncio.run(func())

        self.benchmark.benchmark("Phase Execution", wrapper, iterations=5)


# ==================== 主函数 ====================

def main():
    """主函数"""
    import tempfile
    import shutil

    # 创建临时目录
    temp_dir = Path(tempfile.mkdtemp())

    try:
        # 运行基准测试
        suite = BenchmarkSuite(temp_dir)
        all_pass = suite.run_all()

        # 打印结论
        if suite.benchmark.console:
            if all_pass:
                suite.benchmark.console.print("\n[green]✓[/green] 所有性能测试通过")
            else:
                suite.benchmark.console.print("\n[yellow]⚠[/yellow] 部分性能测试未达标")

    finally:
        # 清理
        shutil.rmtree(temp_dir, ignore_errors=True)


if __name__ == "__main__":
    main()
