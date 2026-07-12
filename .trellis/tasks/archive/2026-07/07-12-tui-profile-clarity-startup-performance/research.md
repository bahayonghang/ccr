# Research: TUI Profile 信息层级与启动性能

## 1. 截图观察

目标截图为 1680x1279 的宽屏布局，当前页面是 Codex Profile。

### 信息层级

- `Focus` 中的 `Name: owlc-sub` 与 `Status: Current · Enabled`，在 `Context / Overview` 中再次出现为 name/current/enabled。
- `model` 和 `base_url` 因 label 命中启发式规则而显示为 info 色，但 description、tags、switch_count 和 Usage 数值基本同权重。
- `auth_mode = no_auth` 只显示为普通 info 色，缺少对路由风险的明确层级。
- `approx_cost = $1601` 与普通计数同权重，扫描时不突出。
- `model_reasoning_effort` 完全缺失，用户无法从 TUI 确认 profile 应用后的推理强度。

### 空间分配

- 宽屏左侧列表占 54%，右侧详情占 46%；详情包含 URL、Usage title 和长 note，实际更需要宽度。
- `Context` 内容结束后仍有较大空白，底部又固定占 3 行空 `Status`。
- `detail_line` 固定将 label pad 到 16 列；短中文标签显得稀疏，长本地化标签可能挤压 value。
- 左侧 `Selection` 是有意义的导航摘要，应保留；主要浪费来自右侧重复和空 status。

## 2. 字段数据流

```text
profiles.toml
  -> ccr-config::load_profiles_from_toml
  -> ProfileConfig.platform_data["model_reasoning_effort"]
  -> ccr-codex::CodexPlatform::resolve_model_reasoning_effort
  -> SwitchSpec.reasoning_effort
  -> Codex config.toml model_reasoning_effort

ProfileConfig
  -> ccr-tui::codex_profile_detail_lines
  -> 当前没有读取 platform_data 中的该 key
```

结论：字段在 profile 解析和 apply 链路中都存在，缺口仅位于 TUI projection/rendering。显示层应保留 raw value 以便诊断，并将已知枚举映射到显式的视觉强度；不得调用 apply 私有校验逻辑或伪造默认值。

## 3. 当前样式与布局实现

- `detail_line(label, value)` 把 label 固定 pad 到 16 列。
- `detail_value_style` 先匹配 yes/no/current/missing，再检查 label 是否包含 auth/provider/login/base_url/model/account。
- 该机制把“字段类别”和“值状态”混在一起，无法表示 `reasoning_effort` 的等级，也会让新增字段依赖命名偶然性。
- `render_profile_context_workspace` 在 wide 模式固定切成 summary / detail / 3-line status。
- `profile_summary_strings` 和各 detail builder 同时渲染身份与状态。

建议改为 typed presentation model，例如 `DetailField { key, label, value, tone, importance }`，由每个平台的 builder 显式指定；通用 renderer 只负责本地化、列宽和 Span 样式。

## 4. 启动链路

```text
process start
  -> clap parse
  -> init_file_only_logger
  -> dispatch no-subcommand
  -> run_tui
     -> initialize_from_config                # 第一次读取 tui.toml
     -> TerminalGuard::new
        -> theme::init_theme
           -> termbg::theme(timeout = 100ms)  # 同步、首帧前
        -> raw mode + alternate screen
     -> App::with_task_executor
        -> load_tui_config                    # 第二次读取 tui.toml
        -> Claude current/runtime/profiles    # 同步
        -> Codex current/runtime/profiles     # 同步
     -> run_loop
        -> first terminal.draw
```

### 本机测量

探针使用 release 构建，直接调用当前代码，不进入完整交互循环。第一次 release 编译未计入产品基线。

| 阶段 | 样本结果 | 结论 |
|---|---:|---|
| 已安装 `ccr --version` 热启动 | 23.8-29.0ms（首个样本 41.3ms） | OS + CLI/logger 基线不大 |
| `App::with_task_executor` 10 次 | 9.0-12.2ms | 当前 15 个 Codex profile 下不是主因 |
| `theme::init_theme`，stdout 非 TTY | 0.024ms | 非交互路径直接跳过 |
| `theme::init_theme`，交互 console | 124.8ms | 命中/接近 100ms termbg 超时，是固定卡顿主因 |

探针已删除，仓库没有保留临时 example。

### 实施后复测

采用确定性 persisted theme 后，以相同 release 探针在交互 console 连续运行 10 次：

| 阶段 | 实施后结果 | 对比 |
|---|---:|---:|
| `theme::init_theme` | 0.001-0.005ms | 从 124.8ms 降至近似零 |
| `App::with_task_executor` | 6.3-7.8ms | 变更前 9.0-12.2ms |

默认路径不再调用 `termbg`；只有显式 `CCR_TUI_THEME=auto` 才承担终端查询等待。主入口实际使用单次加载的 `TuiConfig` 构造 App，探针为复用公共入口仍包含一次 wrapper 配置读取，因此主入口不会比该结果更差。复测探针已删除。

为补齐首帧证据，使用 release 临时 example 在默认主题路径连续启动 10 个独立进程，依次测量 `build_cli_command` 解析、`init_file_only_logger`、`theme::init_theme`、`App::with_task_executor` 与 140x30 `TestBackend` 首帧。release 编译耗时不计入样本，`CCR_TUI_THEME` 在测量进程中明确未设置。

```text
cli=1.229 logger=4.269 theme=0.003 app=7.321 draw=0.760 total=13.587
cli=0.836 logger=3.827 theme=0.002 app=7.267 draw=0.718 total=12.654
cli=0.820 logger=3.823 theme=0.002 app=7.175 draw=0.678 total=12.500
cli=0.834 logger=3.848 theme=0.003 app=7.437 draw=0.679 total=12.803
cli=0.872 logger=4.036 theme=0.003 app=6.915 draw=0.673 total=12.502
cli=0.797 logger=3.729 theme=0.002 app=7.428 draw=0.719 total=12.678
cli=0.898 logger=3.499 theme=0.002 app=6.953 draw=0.696 total=12.051
cli=0.862 logger=3.743 theme=0.002 app=6.995 draw=0.707 total=12.312
cli=1.076 logger=3.702 theme=0.003 app=7.566 draw=0.683 total=13.032
cli=0.838 logger=3.735 theme=0.002 app=6.923 draw=0.630 total=12.132
```

- CCR 自有同步阶段总计：p50 约 12.58ms，p95 约 13.34ms。
- theme：0.002-0.003ms；App：6.915-7.566ms；首帧：0.630-0.760ms。
- 该结果不含 OS 进程创建，也不声称 TestBackend 等同真实终端写出成本；它用于稳定隔离 CCR 自有阶段。真实交互终端的变更前/后 theme 数据仍用于证明 100ms 固定等待已消除。
- 临时 probe source 已删除；仓库不保留一次性测量入口。

### 次要因素

- `TerminalGuard` 在 App 构造前进入 alternate screen，因而 App 的 9-12ms 或冷盘更慢时间会显示为空白，而不是保留 shell 内容。
- `tui.toml` 在主入口和 App 构造中重复读取。
- `init_file_only_logger` 在分发前同步创建日志 writer 并扫描日志目录。
- 当前 `~/.ccr/logs` 存在大量 `ccr.log.YYYY-MM-DD` 文件；清理函数要求扩展名等于 `log`，与实际 rolling filename 不匹配，过期文件无法被清理。此项是长期目录卫生问题，但不是本次 100ms 固定延迟的首要来源。

## 5. 优先级判断

1. P0: 移除默认首帧前 100ms termbg 同步等待，定义可持久化主题策略。
2. P1: TUI 配置单次加载，并把 App 构造放到切屏前或引入可渲染的 loading shell。
3. P1: 添加 reasoning effort 投影和 typed semantic styling。
4. P1: 去重 Focus/Overview、动态 Status、调整 wide ratio 和 label width。
5. P2: 修复日志 retention 对 rolling filename 的识别，并以独立计时决定是否需要异步化。

## 6. 风险

- 默认取消自动主题探测会改变首次启动的明暗选择策略，需要产品确认和 spec 更新。
- App 全量异步加载会显著扩大状态机和错误处理范围；当前测量仅 9-12ms，不建议在没有冷启动证据时过度重构。
- 样式不能用红色表示合法的 `xhigh`，否则会把强度误读为错误。
- 宽屏比例修改必须与 120 列边界和 CJK 断行测试一起验证。
