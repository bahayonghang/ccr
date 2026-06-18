# 技术设计：TUI Tab 排序

## 结论

实现范围改为“固定默认顺序 + `~/.ccr` 下正式 TOML 配置”。推荐文件名为 `~/.ccr/tui.toml`，而不是 `tab-order.toml`。

## 方案对比

### 方案 A：只做固定重排

- 在 `crates/ccr-tui/src/tui/app.rs` 调整 tab 构建顺序。
- `ui.rs`、键盘循环、鼠标命中测试保持不变。

优点：

- 改动面最小。
- 不引入新配置文件，不增加解析和迁移负担。
- 行为稳定，回滚简单。

缺点：

- 不满足用户已明确提出的配置化范围。

### 方案 B：`~/.ccr/tui.toml`

- 新增 root-level TUI 偏好配置文件：`<CCR_ROOT>/tui.toml`，默认即 `~/.ccr/tui.toml`。
- 配置读取放在 `ccr-config`，TUI 只消费解析后的 tab 顺序。
- 文件内容先只定义：

```toml
tab_order = [
  "codex_profile",
  "claude_profile",
  "codex_auth",
  "claude_auth",
  "opencode_auth",
]
```

- 内置默认顺序与上面一致；配置缺失或非法时直接回退默认。

优点：

- 满足用户希望顺应使用趋势、同时保留自定义能力的目标。
- `tui.toml` 比 `tab-order.toml` 更符合仓库现有根级单领域配置命名模式，如 `sync.toml`、`sync_folders.toml`。
- 同文件未来可承载其他 TUI 偏好，不必再增加新的 root-level 配置文件。

缺点：

- 需要新增配置模型、读取路径、默认值、错误处理和测试。
- 这是跨层改动，必须保持 `ccr-config` 与 `ccr-tui` 边界清晰。

## 推荐边界

- `ccr-config` 负责：
  - `CCR_ROOT` / `~/.ccr` 根目录解析
  - `tui.toml` 的加载与默认值回退
  - 可测试的配置契约
- `ccr-tui` 负责：
  - 定义 tab 标识到运行时 `PlatformTab` 的映射
  - 依据配置顺序重排实际 tabs
  - 对缺失/非法配置消费默认顺序后的结果

不要在 `ccr-tui` 里直接 `read_to_string("~/.ccr/...")`。

## 配置契约

### 文件位置

- 默认：`~/.ccr/tui.toml`
- 测试/自定义根目录：`$CCR_ROOT/tui.toml`

### 值域

- `codex_profile`
- `claude_profile`
- `codex_auth`
- `claude_auth`
- `opencode_auth`

### 回退规则

- 文件不存在：使用默认顺序
- 存在未知值：忽略配置并回退默认顺序
- 存在重复值：忽略配置并回退默认顺序
- 缺少任一必需 tab：忽略配置并回退默认顺序

这里推荐“整份配置原子有效”，而不是“部分合法就部分采用”，因为 tab 数量固定且很少，整份回退更容易理解，也更容易测试。

## 影响面

- `crates/ccr-tui/src/tui/app.rs`
- `crates/ccr-tui/src/tui/ui.rs`
- 相关 tab 顺序/切换测试
- `crates/ccr-config` 的 root 配置读取与测试

## 数据流

`CCR_ROOT or ~/.ccr` -> `ccr-config` 加载 `tui.toml` -> 解析为稳定 tab id 列表 -> `ccr-tui` 构建内置 tabs -> 按配置顺序重排 -> `ui.rs` 原样渲染

## 兼容性

- 预选中入口只按 `variant` / `platform` 找 tab，不依赖固定索引，排序调整不应破坏入口。
- 键盘 `Tab` / `BackTab` 循环和鼠标 header 命中测试应继续按 `tabs.len()` 工作。
- 若以后增加新 tab，必须同步更新默认顺序、允许值枚举和配置校验测试。
