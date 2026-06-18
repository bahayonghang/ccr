# 实施计划：TUI Tab 排序

## 目标

把默认 tab 顺序调整为 `Codex Profile`、`Claude Code Profile`、`Codex Auth`、`Claude Auth`、`OpenCode Auth`，并正式支持 `<CCR_ROOT>/tui.toml`（默认 `~/.ccr/tui.toml`）中的 `tab_order` 配置。

## 步骤

1. 确认最终方案
   - 确认配置文件采用 `<CCR_ROOT>/tui.toml`，而不是一次性 `tab-order.toml`。
   - 验证：任务接受标准明确且无歧义。

2. 在配置层新增 TUI 偏好读取
   - 文件：`crates/ccr-config`
   - 操作：
     - 新增 `tui.toml` 配置模型与默认值
     - 通过 `CCR_ROOT` / `~/.ccr` 解析配置路径
     - 对非法顺序执行整份回退
   - 验证：
     - 文件不存在返回默认顺序
     - 缺项/重复/未知值回退默认顺序

3. 调整 tab 构建顺序
   - 文件：`crates/ccr-tui/src/tui/app.rs`
   - 操作：
     - 先构建完整 tab 集
     - 再按配置层返回的顺序重排
     - 默认顺序与用户指定趋势一致
   - 验证：
     - 预选中入口仍按 `variant` 找到正确 tab
     - 无配置时顺序即用户要求的默认顺序

4. 补回归测试
   - 覆盖：
     - `ccr-config`：`tui.toml` 默认/合法/非法配置
     - `ccr-tui`：默认 tab 顺序、配置顺序、Tab/BackTab 循环、预选中入口、header 命中测试
   - 验证：`cargo test -p ccr-tui -- --test-threads=1`
   - 验证：`cargo test -p ccr-config -- --test-threads=1`

## 质量门

- `just fmt-check`
- `cargo test -p ccr-config -- --test-threads=1`
- `cargo test -p ccr-tui -- --test-threads=1`
- `cargo test -p ccr -- --test-threads=1`
- `just lint-strict`

## 回滚点

- 若 `tui.toml` 方案出现复杂回归，可先保留 `ccr-config` 默认顺序接口，但在 `ccr-tui` 只消费默认值，临时禁用文件覆盖。
- 完整回滚则恢复 `App::with_task_executor` 的原始 push 顺序，并删除 `tui.toml` 读取路径。
