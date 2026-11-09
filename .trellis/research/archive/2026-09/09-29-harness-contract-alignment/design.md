# T02 设计

## 责任与方案
AGENTS.md 保持共享规则入口，CLAUDE.md 保持真实 @AGENTS.md import。修正直接 Rust 命令为默认并行且跳过 export_bindings。分开列出官方能力、当前机器文件、Git 交付和原生运行证据。Kimi 优先描述项目自定义 agent 的实际存在；对未生成项目 agent 的工作区保留显式 pull 技能 fallback。新增静态契约检查只读取已跟踪的共享事实和受控夹具，不强制个人 dot 目录存在，不伪造 hook 已加载。只读 reviewer 与可写 trellis-check 分开，模型预算与工具权限分开。

## 文件白名单
- `AGENTS.md`
- `CLAUDE.md`
- `docs/agents/harnesses.md`
- `docs/en/agents/harnesses.md`
- `.codex/skills/ccr-gate-recovery/SKILL.md`
- `.trellis/workflow.md`
- `scripts/quality/check_harness_contracts.py`（新增）
- `scripts/quality/test_check_harness_contracts.py`（新增）

## 兼容与回滚
保持既有命令、共享接口和数据安全规则；所列新增脚本均为检查器或测试。提交实施前重新核对当前文件和基线。回滚只撤销本任务差异，不 reset/clean 或覆盖他人工作。生成/ignored 文件的交付边界必须由父任务强模型复核。

## 依赖
T01。父子层级不代替执行顺序。
