# 共享入口设计

采用父任务 design.md 的语义表、环境矩阵和最小接口。父方案为唯一展示契约，不重复定义另一套标签。

## 文件责任

- `crates/ccr-core/src/core/logging.rs`：修改 ColorOutput 展示方法，增加 OutputStatus 和 format_status；追加必要的纯格式测试。logger、guards、redaction 与 file writer 区域保持不变。
- `crates/ccr-cli/src/commands/common/feedback.rs`：新增仅排版的 print_next_steps；不存储配置或调用服务。
- `crates/ccr-cli/src/commands/common/mod.rs`：注册并导出 CLI 内使用的建议 helper。
- `crates/ccr-core/tests/output_presentation.rs`：新建实际 stdout/stderr 输出契约测试，使用测试子进程运行既有函数，无测试专用产品命令。

ColorOutput 仍为公共入口。新增 formatter 服务现有包装方法及 Doctor 两个实际消费者；不把业务类型引入 core。formatter 返回 String，使用调用者提供的目标流 TTY 状态；对颜色与 TERM 的处理遵循父矩阵。

普通 key_value 继续使用现有接口，必要时修正目标流能力和多行缩进。禁止在单条调用期间 set_override(true/false)。纯函数测试传入能力值，环境组合通过隔离子进程验证。

本子任务不迁移全部 handler 字段或建议，不改 title/separator 等无关默认样式。共享入口外 Codex/Sync 调用通过编译和现有回归检查，最终语义复核由 C3 负责。

## 回退

恢复上述展示文件与对应测试即可，不触及用户数据。新增 core API 不要求删除任何现有出口；根公共 API 保持兼容。

## 已批准补充：TERM=dumb 统一降级

用户于 2026-10-06 选择统一降级。增加 ColorOutput::configure_cli_output()，CLI main 启动时调用一次；仅 TERM=dumb 设置 colored override=false。不在逐条消息切换状态。补充责任文件 crates/ccr/src/main.rs；保留普通终端默认布局、logger、输出流和业务结果。独立进程覆盖保留装饰 helper 的无 ANSI 行为。完整差异和已审阅代码提案见父 research/term-dumb-acceptance-gap.md。
