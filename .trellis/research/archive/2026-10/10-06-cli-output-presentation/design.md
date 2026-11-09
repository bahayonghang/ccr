# CLI 消息展示设计

## 选定方案

用户已选择简洁符号与分组。保留 `ccr_core::core::logging::ColorOutput` 的公共方法，在现有入口集中调整状态展示；CLI 增加一个小型建议区块函数。JSON 继续直接序列化既有 DTO。

最小替换方案只修改标签字面量，不能解决字段、建议和统计混用。选定方案包含调用点语义复核，不引入输出框架、消息事件总线或配置系统。

## 责任边界

```text
既有服务结果 / DTO
        |
        v
CLI handler：选择结果、字段、说明、建议及原有输出流
        |                         |
        v                         v
ColorOutput / CLI 建议区块     既有 JSON 序列化
        |                         |
        v                         v
原有 stdout 或 stderr          stdout
```

- `ccr-core` 的 ColorOutput 负责跨 crate 状态字符、字段样式和环境降级。
- `ccr-cli::commands::common` 负责建议排版；handler 负责内容和条件。
- 服务、认证、诊断报告、错误类型与持久化层保持原样。
- title、separator、banner、表格、确认和脱敏函数保持签名与默认布局。相关 handler 可消除重复空行，不重做这些公共入口。

## 语义与层次

| 语义 | 普通终端 | 纯文本 | 样式 |
| --- | --- | --- | --- |
| 成功完成 / 诊断通过 | `✓ 已保存账号 teacher` | `成功: 已保存账号 teacher` | 标记绿，正文普通 |
| 警告 | `! 警告: <具体情况>` | `警告: <具体情况>` | 标记黄，保留警告文字 |
| 错误 | `× 错误: <具体原因>` | `错误: <具体原因>` | 标记红，保留错误文字 |
| 进度阶段 | `→ <当前步骤>` | `进度: <当前步骤>` | 标记青，无动画 |
| 跳过 | `- 跳过: <原因>` | `跳过: <原因>` | 普通文字 |
| 普通说明 | `<说明>` | 相同正文 | 无日志等级前缀 |
| 字段 | `  邮箱: <既有值>` | 相同字段 | 字段名可粗体，必要值不变暗 |
| 统计 / 取消 / 正常空状态 | `共 3 个账号` / `已取消删除` / `暂无已保存账号` | 相同正文 | 无成功或错误标记 |

尖括号表示设计槽位，不是建议命令。Doctor 保留英文正文和字段，状态前缀使用相同展示规则；不自动翻译整个诊断报告。

一个简单操作包含一个结果行、必要字段和可选建议区块。无起始空行；字段与结果连续；建议前仅一个空行。无建议时无标题、空白尾块。多行字段的续行保持缩进。

## save 示例

普通终端，邮箱为合成数据：

```text
✓ 已保存账号 teacher
  邮箱: tea***@example.test

下一步
  查看账号
    ccr codex auth list
```

NO_COLOR 终端保留字符，去掉 ANSI。重定向或 TERM=dumb 使用 `成功: 已保存账号 teacher` 首行。

save 保存当前登录，成功后不默认建议切换回同一账号。成功优先建议查看结果，默认 1 个、最多 2 个。失败优先恢复操作，取消和未知状态不追加成功结论。必要风险文字保留。

## 最小接口变化

- ColorOutput::success/info/warning/error/step 保留名称、参数与输出流；info 改为无等级前缀的普通说明。字段复用 key_value(key, value, indent)。
- 新增 `commands/common/feedback.rs` 的 `print_next_steps(steps: &[(&str, &str)])`：每项为操作名称、完整命令；空列表无输出。函数只排版，不执行、不验证、不持久化。
- 在现有 logging 模块增加小型 `OutputStatus`（Success/Warning/Error/Step/Skipped）及 `ColorOutput::format_status(status, msg, is_terminal) -> String`。现有打印方法委托该函数，Doctor 传 stdout 能力并继续打印到 stdout；格式函数不选择通道或退出码。
- 内部纯格式函数通过参数接收已解析的展示能力。只覆盖上述已有状态，不建立通用消息对象图；测试不切换全局颜色状态。
- handler 向共享消息传普通文本，移除这些参数中的整行颜色、重复状态和 emoji。表格等独立样式保留。
- 新增 `crates/ccr/tests/commands/output_presentation.rs`，在 commands.rs 注册。现有命令测试补充行为断言，不为每个 info 调用复制实现测试。
- 新命令、配置、环境变量、服务和第三方依赖均为 0。

## 环境矩阵

字符能力与颜色能力分开判断，目标流分别通过标准库 IsTerminal 检查。复用 colored 3.1.1，不复制其变量优先级，不在逐条消息中调用全局 set_override。用户于 2026-10-06 批准补充：CLI 启动处调用 ColorOutput::configure_cli_output() 一次，仅 TERM=dumb 禁用 colored 样式，覆盖保留的标题、分隔线、表格和确认文字；其他条件不设置 override。完整差异与原生证据见 research/term-dumb-acceptance-gap.md。

| 条件 | 字符 | 样式 |
| --- | --- | --- |
| 目标流 TTY，TERM 非 dumb | 简洁符号 | 库允许时仅标记/字段名加样式 |
| 上述条件且 NO_COLOR 非空，无强制颜色 | 简洁符号 | 无 ANSI，包括粗体 |
| 目标流非 TTY | 纯文本状态词 | 默认无 ANSI |
| TERM=dumb | 纯文本状态词 | 无 ANSI |
| stdout TTY、stderr 重定向 | 各流分别判断 | 默认 stderr 无样式 |
| stdout 重定向、stderr TTY | 各流分别判断 | 库依据 stdout，stderr 可保持无色；不补全局 override |
| 显式 CLICOLOR_FORCE=1、TERM 非 dumb | 仍按目标流判断 | 保留库的强制颜色及对 NO_COLOR 的优先级；属于默认规则例外 |

库把空 NO_COLOR 也视为禁用颜色，任务保留既有行为。JSON 不调用人类渲染，强制颜色也不能给 JSON 添加装饰。能力判断不修改系统或终端设置。

## 通道与兼容性

success/info/warning/step 当前为 stdout；error 当前为 stderr（logging.rs:31-49）。保持各 handler 的详情和建议通道。公开指南建议诊断信息使用 stderr；批量通道迁移改变接口，留在范围外。

JSON 路径早于人类渲染，保留 DTO、字段、值、格式和退出码。不得借展示任务修复错误吞掉、改变部分成功或 handle_error。DoctorStatus::label 保持公共兼容，command 层选择新标记，不改报告序列化。

旧人类标签属于主动更新范围，依赖标签的外部文本解析可能受影响。任务不增加 legacy 模式。

## 建议与参数

- 复用 handler 的现有分支，不增加认证检查或服务调用；未知状态保持未知。
- 已校验 Codex/Claude 名称只允许 ASCII 字母、数字、下划线、连字符，最长 32 字符（codex_auth_service.rs:1740、claude_auth_service.rs:625）。位置参数使用 `--` 分界，覆盖示例为 `ccr codex auth save --force -- teacher`，同时说明覆盖后果。
- 未知名称给出可执行的列表或 --help，不把 `<名称>` 当作可复制命令。
- 路径使用既有安全引用；不建立跨 shell 生成框架。密码和密钥不加入建议命令；import/export 使用既有交互或帮助入口。
- 长字段和命令保持完整，不进行宽度对齐或人为截断，允许终端自然折行。

## 阶段与回退

C1 修改共享规则后，未迁移的 handler 仍可运行；C2/C3 再调整字段与建议。阶段间可暂时存在版式差异，但 JSON、退出码和业务行为不得回归。

前提是纳入修改的文本供人阅读；解析旧标签的外部脚本会受影响。回退只恢复批准的展示、测试、规范和文档，不改变账号或日志，不运行 auth restore。无数据迁移。commit、push、PR、发布和归档均未授权。
