# T02 静态契约实施记录

- 日期：2026-09-30 UTC；输入 HEAD：`fe9d65972376dbb4c332fde9860c1e42ff2f2005`。
- 环境：Windows 11，Python 3.14.7，Bun 1.4.2；项目声明 Bun 1.4.0，版本差异保持可见。
- 状态：本子 agent 负责的检查器、负例与本地验收完成；等待主会话最终审查。没有提交、归档、推送或改变全局设置。

## 所有权与实际变更

- 本子 agent 新增 `scripts/quality/check_harness_contracts.py` 与 `scripts/quality/test_check_harness_contracts.py`，并回写本任务证据和检查清单。
- 主会话负责 AGENTS.md、CLAUDE.md、双语 harness 页和已跟踪 gate skill；检查器读取的是当前磁盘版本。本子 agent 没有覆盖这些共享文档，也没有编辑 T04 的 Copilot 文件。
- 发现 gate skill 的包级 cargo test 示例缺少绑定 skip 后已报告；主会话补齐 `-- --skip export_bindings`，再运行本轮验收。
- T01 的受控注入及交付夹具已报告 14/14 通过；本轮沿用该前置结果，没有重新构建其实现。

## 检查器契约

检查器只读取 6 个已跟踪共享文件：AGENTS.md、CLAUDE.md、两份 harness 页、gate skill、.trellis/workflow.md。运行不调用 Git、网络或客户端，不检查个人 agents、hooks、账户或 trust 设置。

| 编号 | 检查范围 |
|---|---|
| HC001 | 共享文件缺失、空文件和不可读取的 UTF-8 |
| HC002 | 真正独立的 @AGENTS.md import，拒绝 code span、围栏、注释、缩进代码和错误目标 |
| HC003 | 双语集成表各保留五个工具的唯一行；根规则和 skill 覆盖五工具 |
| HC004 | Kimi 项目三角色、pull 上下文、未初始化检出的执行能力 fallback；拒绝否认已观察到的项目 agents |
| HC005 | 默认并行、直接 cargo test 示例的 export_bindings skip、独立绑定生成归属；拒绝线程参数及环境变量覆盖 |
| HC006 | 审批前只读 reviewer、批准后可写自修 check；任务创建不授予实施权限 |
| HC007 | Git 交付、匹配版本的初始化步骤、生成文件和原生加载/trust 的证据边界 |
| HC008 | fmt 与 version-sync 的修改副作用及修复/验证分类 |

检查以导入语法、表格单元格、命令参数和局部契约词为基础，没有新增依赖或配置引擎。静态通过只能证明这些共享说明满足已列规则，不能证明原生客户端已加载或权限在运行时生效。

## 测试与验证

| 命令 | 退出码 | 结果 |
|---|---:|---|
| `python -m unittest scripts.quality.test_check_harness_contracts` | 0 | 20 个测试方法通过，含四类 AC3 负例及全部缺文件负例 |
| `python scripts/quality/check_harness_contracts.py` | 0 | 6 个共享文件、5 个 harness 通过 |
| `just docs-check` | 0 | frozen install 无依赖变化，docs audit、VitePress 构建、ccr-ui docs audit 通过 |
| `just version-check` | 0 | 7.4.0 一致，Bun 声明镜像及原依赖治理门禁通过 |
| `git diff --check` | 0 | 无空白错误 |

测试包含 BOM/CRLF、无关段落增补、不同 fenced code 和 Rust 命令续行。隔离夹具只有六份共享文件和检查器；没有 .git、.claude、.grok、.kimi-code、.omp 或个人 .codex/agents/hooks/config。默认 root 及显式 --root CLI 均验证过。该夹具没有冒充已提交 clean clone 或原生客户端会话。

补充检查确认六份输入均由 Git 跟踪。两个新增 Python 文件另以 `git diff --no-index --check` 检查：与 NUL 有内容差异时 Git 返回 1，但没有 whitespace 输出。首次证据辅助脚本误把该返回码视为失败；调整返回码分类后辅助检查通过。没有修改 Git 配置。

完整命令、退出码和输出见 [verification.json](evidence/verification.json)。

## T06 集成入口

从仓库根目录按顺序运行：

```text
python -m unittest scripts.quality.test_check_harness_contracts
python scripts/quality/check_harness_contracts.py
```

检查器遇到漂移或共享文件缺失返回 1；成功返回 0。受控目录验证可使用 `python scripts/quality/check_harness_contracts.py --root <fixture-root>`。不需要个人工具目录或新运行时依赖。

## 验收与未验证边界

- AC1：两语言 Kimi 集成行、pull fallback、新检出步骤和原生证据边界通过静态检查。
- AC2：并行、绑定生成、审批及修复副作用规则通过；包级命令缺口已由主会话修正。
- AC3：失效导入、错误 Kimi 分派、串行默认命令和审批前可写 check 均有受控负例。
- AC4：本子 agent 没有读取或复制真实个人配置、凭据或模型设置；知识回写由主会话保留在批准的共享文档与 skill。
- 五套原生客户端的启动、hook trust、实际工具权限和账户模型解析仍未验证。托管 CI 和其他 OS 的本轮结果由父任务单独记录。

## 独立审查后补充

独立强模型发现 HC005 漏检独立行的 RUST_TEST_THREADS 赋值，已在原批准的两个 Python 文件内修复。新增 Bash、PowerShell、CMD 共 8 个负例，最新套件为 21/21。初轮 20/20 凭证保留；最终源码与测试证据见 [集成审查](../../../../09-29-evergreen-harness-audit/research/final-integration-review.md)。
