# Super Dev 商用就绪验收清单

> 把"可商用"从开放目标转成可逐条勾选的交付项。每项标注现状(✅/🟡/🔴)+ 证据。
> 更新日期:2026-06-17。

---

## 一、核心闭环(用户从想法到上线)

| # | 验收项 | 现状 | 证据 |
|---|--------|------|------|
| 1 | 用户能安装(npm/brew/脚本) | ✅ | `npm/super-dev/` 5 平台预编译包 |
| 2 | 首启有选择器(宿主/第三方API/离线) | ✅ | `app.rs` 分组 picker |
| 3 | 输入需求触发 9 阶段流水线 | ✅ | `runner.rs` PHASE_CHAIN |
| 4 | 宿主(claude code/codex)真正写代码 | ✅ | `experts.rs` "Create REAL CODE FILES" |
| 5 | 两道人工 gate 审核关键节点 | ✅ | DocsConfirm / PreviewConfirm |
| 6 | 前端生成后可预览(/preview) | ✅ | app.rs slash_preview + lib.rs StartPreview |
| 7 | delivery 指挥生产构建 + 部署指令 | ✅ | delivery_prompt + runner 调宿主 |
| 8 | 一键部署(/deploy) | ✅ | app.rs slash_deploy + RunDeploy |
| 9 | 中断可恢复(状态持久化) | ✅ | workflow-state.json + rollback |
| 10 | 优雅退出(不留孤儿进程) | ✅ | lib.rs quit 时 kill dev server |

## 二、质量与安全(治理层)

| # | 验收项 | 现状 | 证据 |
|---|--------|------|------|
| 11 | emoji 作图标实时拦截(全局) | ✅ | check_emoji 25 种文件类型 + 17 段正则 |
| 12 | 强制免费图标库(Lucide/Heroicons/Tabler) | ✅ | SPEC_PREAMBLE + frontend prompt |
| 13 | 敏感路径写操作拦截(bypass-immune) | ✅ | check_sensitive_path SD-SEC-001 |
| 14 | 硬编码颜色拦截 | ✅ | check_color_tokens SD-CODE-002 |
| 15 | AI slop 拦截 | ✅ | check_ai_slop |
| 16 | 质量门(构建/测试评分) | ✅ | quality 阶段 |
| 17 | 合规审计(SOC2/ISO27001/EU AI Act) | ✅ | compliance.rs |

## 三、工程化(稳定性/可维护)

| # | 验收项 | 现状 | 证据 |
|---|--------|------|------|
| 18 | 全量测试绿 | ✅ | 714 tests / 0 fail |
| 19 | clippy pedantic clean | ✅ | -D warnings |
| 20 | fmt clean | ✅ | cargo fmt --check |
| 21 | doctor 自检 | ✅ | 7 项含部署就绪检测(delivery notes/部署命令/平台 CLI) |
| 22 | e2e 端到端测试 | ✅ | tests/e2e.rs 18 项 |
| 23 | 错误恢复(重试) | ✅ | max_retries=3 + 指数退避(2s/4s/8s)+ 瞬态检测(429/5xx) |

## 四、文档与上手(可商用必备)

| # | 验收项 | 现状 | 证据 |
|---|--------|------|------|
| 24 | README 安装→上线完整教程 | ✅ | README「从安装到上线」9 步教程 |
| 25 | 架构文档 | ✅ | docs/ARCHITECTURE_AND_INTERACTION_DESIGN.md |
| 26 | 用户交互文档 | ✅ | 同上 + TUI /help |
| 27 | claude code 学习借鉴记录 | ✅ | docs/CLAUDE_CODE_LEARNINGS.md |

## 五、待推进项(本轮目标)

- [x] **#21** doctor 加部署就绪检测 ✅
- [x] **#24** README 补从安装到上线完整教程 ✅
- [x] **#23** 重试指数退避 ✅
