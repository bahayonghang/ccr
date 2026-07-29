# 诊断 Grok profiles、改进解析错误并支持推理强度

## Goal

准确指出当前系统 Grok `profiles.toml` 无法被 CCR 加载的具体原因，让 CLI/TUI 的配置解析错误包含足以修复问题的安全诊断信息，并让 profile 中的 `reasoning_effort` 在切换时完整写入 Grok 运行时配置。

## Background

- 当前系统文件位于 `C:\Users\lyh\.ccr\platforms\grok\profiles.toml`。
- Python 标准 TOML 解析器可解析该文件，证明文件不一定存在 TOML 词法或语法错误。
- 已安装的 `ccr 7.0.1` 执行 `ccr grok profile list --json` 仍以退出码 14 失败，并只报告“配置包含无效语法”。
- 当前失败项已定位到 `provider_type`：值 `relay` 不属于共享 `ProviderType` 编码；允许值为 `official_relay` 或 `third_party_model`。该 profile 属于第三方中转，应使用 `third_party_model`。当前工作树二进制对真实只读文件的复验定位在第 14 行；测试夹具保留原始第 13 行布局以锁定字段级定位。
- 第 20 行 `reasoning_effort = "high"` 会通过 `ConfigSection.other` 保留，因此不是本次加载失败原因；但 CCR 当前不会验证、展示或应用它，重建 `[model.custom]` 时反而会丢掉运行时已有的推理强度字段。
- 当前 `~/.grok/config.toml` 使用 `model.custom.supports_reasoning_effort = true`、`model.custom.reasoning_effort = "high"` 和 `models.default_reasoning_effort = "high"`。本机 Grok 文档确认全局默认推理强度的配置位置是 `[models].default_reasoning_effort`。
- `ccr-config` 当前依次尝试完整格式和简化格式反序列化，但会丢弃两次底层解析错误并统一包装为“无效语法”；加载层又重复添加路径和错误前缀。
- TUI 已有 `Where`/`What` 分行格式化逻辑；截图中的粘连来自长 Windows 路径在终端边界处换行，且 `What` 再次包含同一路径和嵌套错误前缀。

## Requirements

- R1：使用 CCR 自身的数据模型确定当前文件失败的具体字段、值类型或结构约束，并向用户说明可执行的修复方式。
- R2：当 TOML 文本本身无效时，错误必须保留解析器提供的行号、列号和具体原因。
- R3：当 TOML 语法有效但不符合 CCR profile 数据模型时，错误必须区分“语法错误”和“字段/类型错误”，并尽可能指出字段位置与期望类型。
- R4：文件加载错误只显示一次来源路径和一次原因，不重复嵌套“配置格式无效”或“TOML 解析失败”。
- R5：CLI 与 TUI 通过共享加载路径获得一致的底层诊断；TUI 保留 `Where`/`What` 的可扫描布局。
- R6：错误输出不得回显 API key、token、带凭据的 URL 或完整配置正文。
- R7：保持完整格式和简化格式兼容，不改变有效 profile 的加载结果。
- R8：将 `reasoning_effort` 定义为 Grok profile 的正式平台字段；支持手写 `profiles.toml`、`profile create --reasoning-effort` 和 `profile set-field ... reasoning_effort`。
- R9：切换带 `reasoning_effort` 的第三方 profile 时，在 `[model.custom]` 写入 `supports_reasoning_effort = true` 与 `reasoning_effort = <值>`，并在 `[models]` 写入 `default_reasoning_effort = <值>`；官方 profile 只管理 `[models].default_reasoning_effort`。
- R10：`reasoning_effort` 必须是 Grok Build `ReasoningEffort` 枚举支持的字符串：`none`、`minimal`、`low`、`medium`、`high`、`xhigh` 或 `max`。CCR 忽略首尾空白并规范为小写；其他值必须拒绝，避免生成 Grok 无法反序列化的配置。模型菜单的自定义 `id` 只是展示/输入标识，不能替代其规范 `value`。
- R11：切换到未配置 `reasoning_effort` 的 profile 或执行 `off` 时，恢复进入 CCR profile mode 前的 `models.default_reasoning_effort`；第三方 `[model.custom]` 通过既有整表恢复机制清理，不能残留上一 profile 的推理强度。
- R12：Grok `current/list --json` 与 TUI profile 详情显示 `reasoning_effort`，不展示内部派生的 `supports_reasoning_effort`。
- R13：更新 Grok 示例、CLI 中英文文档和运行时规范，使 `reasoning_effort = "high"` 成为 copy-ready 用法，并锁定字段映射与恢复语义。
- R14：本任务不直接修改或覆盖用户当前的 `profiles.toml` 或 `~/.grok/config.toml`；真实文件只作为只读诊断证据。

## Acceptance Criteria

- [x] AC1：针对当前系统文件的诊断能指出实际不兼容项，而不是笼统归因于“无效语法”。
- [x] AC2：损坏 TOML 的测试断言错误包含文件路径、行列和底层解析原因。
- [x] AC3：语法有效但字段类型不兼容的测试断言错误明确属于结构/类型问题，并定位相关字段或位置。
- [x] AC4：错误文本中路径和错误分类不重复，且不包含测试夹具中的秘密值。
- [x] AC5：现有完整格式、简化格式以及 Grok profile 加载测试继续通过。
- [x] AC6：`ccr-config` 与 `ccr-tui` 的相关测试通过；按影响范围完成 Rust 格式、Clippy 和测试验证。
- [x] AC7：TUI 的 `Where` 与 `What` 标签和值使用稳定的分行布局；长 Windows 路径换行后不会与 `What` 标签视觉粘连。
- [x] AC8：包含 `reasoning_effort = "high"` 的第三方 profile 切换后，运行时同时包含 `model.custom.supports_reasoning_effort = true`、`model.custom.reasoning_effort = "high"` 和 `models.default_reasoning_effort = "high"`。
- [x] AC9：官方 profile 的推理强度只写入 `models.default_reasoning_effort`，不会创建或污染 `[model.custom]`。
- [x] AC10：从 `high` profile 切换到未配置推理强度的 profile，以及执行 `off`，都恢复入口配置的 `models.default_reasoning_effort`；原值缺失时删除该键，其他 `[models]` 字段保持不变。
- [x] AC11：`create`、`set-field`、`current/list --json` 和 TUI 详情覆盖 `reasoning_effort`；空字符串、非字符串及非规范等级被拒绝，7 个合法等级可 round-trip。
- [x] AC12：Grok canonical 示例和中英文命令文档包含 `reasoning_effort = "high"` 或等价 CLI 用法，运行时规范与代码行为一致。

## Out of Scope

- 自动修改或覆盖用户当前的 `profiles.toml`。
- 重设计整个 TUI 空状态页面。
- 自动探测远端模型实际支持哪些 reasoning option；CCR 只校验字段形态，模型能力由 Grok 在运行时判定。
- 把 `supports_reasoning_effort` 暴露为独立 profile 输入；它由是否设置 `reasoning_effort` 自动派生。
