# 原生视觉复核（2026-11-07，主会话）

构建：`ccr-desktop/target/release/ccr-gpui.exe`，本机真实只读用量源，浅色 / English，窗口 1938×1416 物理像素。

- `native/after-overview-default.png`：第一次截图实际启动的是旧 release 产物（新构建因生产编译错误失败，见下），仅作 before 参考。
- `native/after-overview-light-en.png`：修复后的新构建。

## 观察（after）
- 场景渐变与光斑在窗口四周与侧栏后方可见；侧栏半透明、纸面带亮边与阴影。
- Tab 胶囊与 Overview 标题同行；7D/30D/90D + Refresh 位于页头右侧。
- KPI 单面板四栏，Cost 为主读数；平台带仅有记录平台成格，Antigravity/OpenCode 合入 Other sources 并写原因。
- 右栏与主列等高，含 Data source 小节；图表已收紧到有数据区间（7D 下无收紧需求）。
- 发现并修复：日均成本显示 `$35.2906/day`（4 位小数）→ ≥$1 时两位小数。
- 待观察：启动后选中的 Overview tab 显示 2px 焦点环（初始焦点落在页头首个 tab stop），属焦点可见合同内行为，未改。
- 主纸面面板下方仍有空白（窗口高 1374 逻辑像素以上时），属内容高度自然结束。

## 发现的构建缺陷
`features/overview.rs` 的 `type Node = gpui_kit::base::test_support::Observed<..>` 只在 `test-support` 特性下存在；fmt/lint/test/test-ui 均通过而 release 构建失败。已改为 `gpui_kit::base::ObservedElement<..>`，并在 spec 记录“以 desktop-ci 为门禁”。

## 未运行
深色主题、中文、Claude workbench / Settings 页截图（不在未授权下操作 UI 切换主题/页面）；DPI 矩阵（user-deferred）；macOS/Linux；读屏 / IME。
