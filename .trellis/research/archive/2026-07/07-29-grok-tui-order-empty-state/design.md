# Design：Grok TUI 顺序与空态迁移指引

## 1. 边界与不变量

本修复不改变 profile 路由架构。现行不变量是：

- Claude、Codex、Grok 的 profile 通过 `ccr <platform> profile ...` 管理。
- `ccr platform list` 仍是受支持的平台 registry 视图。
- `ccr platform switch/current/info/init/profile` 仅保留解析能力，用于返回 `legacy_platform_command_error`；它们不再是可发现或推荐的命令。
- `TuiConfigManager::load()` 保留用户排序，只有缺失的已知页签按 `DEFAULT_TAB_ORDER` 追加。

因此改动集中在配置默认值、TUI 文案和 Clap 帮助展示，不触碰 profile 存储或 runtime 应用逻辑。

## 2. 默认顺序

在 `crates/ccr-config/src/managers/tui_config.rs` 中仅调整 `DEFAULT_TAB_ORDER`：

```text
CodexProfile
ClaudeProfile
GrokProfile
CodexAuth
ClaudeAuth
OpencodeAuth
```

不增加版本字段或落盘迁移：

- 无 `tui.toml`：`TuiConfig::default()` 立即采用新顺序。
- 完整自定义顺序：`missing` 为空，load 后完全保持原顺序。
- 不完整旧顺序：只影响缺失项的补齐相对顺序，用户已列项不动。

更新精确默认顺序测试，并保留现有自定义/缺失项迁移测试作为兼容护栏。

## 3. 空态文案

`render_empty_state` 已能从 `app.current_platform()` 取得 `short_name()`，无需增加 Grok 特判。把成功读取但 profile 列表为空的两条提示改为：

```text
Run 'ccr {short_name} profile create --help' to create a profile
After creating it, press 'r' to reload
```

中文提供等价内容。读取失败的错误空态保持原样。

为减少 Ratatui 渲染测试的脆弱性，可抽取一个只构造 `Vec<Line>` 的小型 helper，由 renderer 和单测共用。测试依次设置英文/简体中文语言，并覆盖 Claude、Codex、Grok，重点断言有效命令、reload 提示和旧命令负断言。

## 4. CLI 帮助与兼容解析

### 4.1 隐藏而不删除

在 `PlatformAction` 的五个退休变体上使用 Clap 的隐藏属性，使它们不出现在 `ccr platform --help` 的 Commands 列表中。变体与 `dispatch_platform` match 臂保留，因此：

```text
ccr platform init grok
  -> parse PlatformAction::Init
  -> dispatch_platform
  -> legacy_platform_command_error("init")
```

不会退化成 parser 的 unknown-subcommand 错误，也不会恢复旧状态写入。

### 4.2 自定义帮助

同步改写 `help_config.rs` 的根帮助和 platform 帮助：

- platform 只描述 registry/list 能力和迁移边界。
- 状态入口使用 `ccr current`。
- profile 入口使用 `ccr claude profile --help`、`ccr codex profile --help`、`ccr grok profile --help`。
- `ccr help platform` 继续与 `ccr platform --help` 完全一致。

不在本任务中清扫公共文档和所有历史输出；这些面包含尚无统一替代语义的平台，作为单独迁移任务处理。

## 5. 测试策略

### ccr-config

- 精确断言完整默认顺序。
- 保持现有完整自定义顺序、旧配置补齐和 round-trip 测试通过。

### ccr-tui

- 英文/中文 Grok 空态包含 `ccr grok profile create --help` 与 `r` reload。
- Claude/Codex 生成各自显式命令。
- 所有成功读取的 Profile 空态都不包含 `ccr platform init` 或 `ccr add`。

### CLI 集成

- `platform --help` 与 `help platform` 相等。
- Commands/after-help 不出现五个退休入口，仍出现 `platform list`、`ccr current` 和显式 profile 帮助。
- 根帮助不出现退休平台操作。
- 真实启动测试 `platform init grok`：非零退出、legacy 文案和 Grok 迁移目标存在。

## 6. 风险与回滚

- 风险：误删 enum 变体会破坏旧脚本的迁移错误。通过“help 隐藏 + 负向执行测试”防止。
- 风险：默认顺序变化误改自定义配置。通过完整自定义顺序 round-trip 测试防止。
- 风险：全局 i18n 状态污染并行测试。沿现有测试模式在用例结束恢复英文，并串行执行相关测试。
- 回滚：顺序、TUI 文案、帮助隐藏均为无数据迁移改动，可按文件回退；不需要恢复用户配置。
