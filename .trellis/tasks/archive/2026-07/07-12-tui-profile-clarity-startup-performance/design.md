# Design: TUI Profile 信息层级与启动性能优化

## 1. Boundaries

- `ccr-tui/ui.rs` 负责 profile 详情 presentation model、响应式布局和 Ratatui rendering。
- `ccr-tui/app.rs` 继续拥有 profile 数据、selection 和 reload；不在 render loop 增加 I/O。
- `ccr-config::TuiConfigManager` 继续拥有 `tui.toml`；若主题需要持久化，在这里扩展向后兼容字段。
- `ccr-codex` 继续拥有 reasoning effort 的 apply 校验；TUI 只读取 profile raw value并显示。
- `ccr-core/logging.rs` 的 retention 修复作为独立、小范围性能卫生改动，仅在测量确认后纳入实现。

## 2. Profile Detail Presentation Model

以显式字段模型代替 `detail_value_style(label, value)` 的字符串猜测：

```rust
enum DetailTone {
    Primary,
    Accent,
    Info,
    Success,
    Warning,
    Muted,
    Cost,
    Effort(EffortLevel),
}

struct DetailField {
    key: DetailKey,
    value: String,
    tone: DetailTone,
    emphasize_label: bool,
}
```

- `DetailKey` 负责稳定的中英文 label，不用裸字符串分派。
- 每个平台 builder 显式选择 tone；renderer 不推断业务含义。
- status 值仍由小型 typed helper 映射 success/warning/muted。
- `model` 与 `reasoning_effort` 使用平台 accent 且 label 加粗。
- `minimal/low/medium/high/xhigh` 依次使用 muted/info/accent/accent+bold/warning+bold；合法 `xhigh` 不使用 error。
- unknown effort 保留 raw text并使用 warning，帮助发现旧配置或手工编辑错误。
- secret value 先经过现有 masked helper，再进入 `DetailField`。

## 3. Codex Reasoning Effort Projection

新增只读 helper，从 `config.platform_data.get("model_reasoning_effort")` 提取非空 string：

- 不存在/空字符串 -> `-` + muted。
- 已知枚举 -> lowercase display + mapped effort tone。
- 未知/非 string -> raw/debug-safe display 或 `invalid` + warning，具体实现需避免序列化可能包含 secret 的任意对象。

位置放在 Codex `Engine` 分组的 `model` 之后、`small_fast` 之前，使模型选择和推理强度相邻。

## 4. Layout

### Wide

- workspace: list 46% / detail 54%。
- left selection panel保留 5 行。
- Focus 只显示 name + Current/Available + Enabled/Disabled + optional last apply。
- Context 不再重复 name/current/enabled；第一组直接从 description 或 Routing/Auth 开始。
- Status 只有存在 last apply/toast 时才分配 3 行，否则 detail 获得全部剩余高度。

### Standard / Compact

- 保留当前上下布局和 drawer 行为。
- 同样去重 identity/status。
- label width 由本次可见 `DetailKey` 的 display width 计算，clamp 在 10-18 列；compact 使用更小上限。
- 继续使用 `unicode-width` helper，不按 char count 计算。

## 5. Startup

### Confirmed theme policy

- `TuiConfig` 增加可选 theme，默认 `Mocha`。
- `Ctrl+T` 切换 theme 后持久化完整 `TuiConfig`，与 `Ctrl+L` 的保存契约一致。
- `CCR_TUI_THEME=mocha|latte` 始终最高优先级。
- 自动探测改为显式 `CCR_TUI_THEME=auto`，并清楚接受最多约 100ms 的探测等待；默认路径不调用 termbg。

### Construction order

```text
load TuiConfig once
  -> select language and theme
  -> construct App from loaded config
  -> enter TerminalGuard
  -> draw first frame immediately
  -> existing tick starts Usage background load
```

- 为 `App::with_task_executor` 增加接收已加载 `TuiConfig` 的构造入口，旧入口可作为测试/兼容 wrapper。
- App 当前 9-12ms，不立即改成全异步状态机；先把它移到 alternate screen 之前并保留可恢复错误。
- 增加分段 timing span 或 debug-only probe seam，避免永久打印到 TUI。

## 6. Logging

- 修复 retention 时按 rolling appender 的实际前缀 `ccr.log.` 识别日期文件，而不是依赖最后扩展名为 `log`。
- 不在本任务中删除用户日志或改变 14 天保留策略。
- 若 logger 分段计时仍稳定低于预算，只修 correctness，不引入后台 logger 初始化。

## 7. Compatibility

- 旧 `tui.toml` 缺少 theme 时使用新默认，tab order 与 language 不变。
- 未配置 reasoning effort 的 profile 显示 `-`，不修改文件。
- 不改变 Codex apply 规范和合法枚举。
- 主题策略变更需在 Phase 3 更新 `ccr-tui` spec 的 startup/theme contract。

## 8. Rollback

- UI presentation model 可单独回退，不影响 profile 配置和 apply。
- theme persistence 字段使用 serde default，可回退代码且旧版本忽略未知字段前需验证当前 TOML parser 行为。
- construction order 变更若触发 terminal error 文案/语言回归，可保留单次 config load，同时恢复 guard 顺序。
