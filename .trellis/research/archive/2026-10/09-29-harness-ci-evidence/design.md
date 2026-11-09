# T07 设计

## 责任与方案
先复验历史失败对应现有测试与正式 OS/coverage 门禁。当前主干测试已通过，不预先改变生产逻辑。若失败重现，只在已有测试增加有界诊断字段：doctor outcome、退出码、timed_out、stdout/stderr 字节计数及异步清理状态；敏感输出必须脱敏。超过诊断范围的产品修复重新提交变更计划。Tauri 9 条 audit warning 记录依赖链、target 可达性、责任人与后续处置，不新增 ignore。

## 文件白名单
- `crates/ccr-cli/src/commands/codex/fix.rs`
- `ccr-ui/src-tauri/src/process/gateway.rs`
- `ccr-ui/tests/setup/react-cleanup.ts`
- `ccr-ui/vitest.smoke.config.ts`
- `.trellis/spec/ccr/backend/test-fixtures.md`
- `.trellis/spec/ccr/backend/dependency-governance.md`

## 兼容与回滚
保持既有命令、共享接口和数据安全规则；所列新增脚本均为检查器或测试。提交实施前重新核对当前文件和基线。回滚只撤销本任务差异，不 reset/clean 或覆盖他人工作。生成/ignored 文件的交付边界必须由父任务强模型复核。

## 依赖
T06。父子层级不代替执行顺序。

## 2026-10-04 继续实施范围

用户在未完成清单后要求继续实施。新增白名单：`crates/ccr/tests/managers/general.rs`、`crates/ccr/tests/managers/legacy_registry.rs`，仅应用已复核候选的六个命名环境夹具绑定；`.gitignore` 精确忽略本任务的 `research/tauri-advisory-commands.json`。保留原始 JSON、诊断失败、测试并行度与生产锁逻辑。
