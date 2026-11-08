# 执行计划

## 0. 准备

- [ ] `trellis-before-dev`：读取 `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`、`ccr-desktop/DESIGN.md`、`theme.rs`、`shell/mod.rs`、`shell/semantics.rs`、`features/overview.rs`。
- [ ] 原生基线：release 构建后截取 Overview light/dark 1280×800 两张“before”图，存放到 `outputs/desktop/airy-glass-material/before/`。
- [ ] 探针：`tnum` 在 Segoe UI Variable 上是否生效（一个临时 example 或测试窗口），结论写入 `research/numeric-font-probe.md`。

## 1. 材质合同（提交 1：`feat(GPUI): ✨ 重建轻盈毛玻璃材质`）

- [ ] DESIGN.md：更新 Elevation 章（成立条件三条）、frontmatter 材质 token、Don'ts（删除“不加 glass”，改为“不嵌套、不盖高频内容”）。
- [ ] `theme.rs`：用 `Material` 替代 `solid_sheet`；新增 `composite()` 合成对比度测试，覆盖 light/dark × 4 个背景采样点；更新 inset 测试（1280×800 约 20px，960×640 为 0）。
- [ ] `shell/mod.rs` root：场景 + 2 个光斑 + frame（sidebar chrome 并列 sheet）；检查 Kit `sidebar/title_bar/status_bar/background` 映射，避免不透明底穿帮。
- [ ] 验证：`just desktop-fmt-check && just desktop-lint && just desktop-test`。

## 2. 侧栏与 Lens 胶囊（提交 2：`feat(GPUI): ✨ 优化侧栏与分段标签`）

- [ ] `nav_button`：pill 选中面、无边框、hover / focus 三态；平台图标着色规则按评审结论实现。
- [ ] 分组间距与品牌行呼吸空间调整。
- [ ] 新增 `shell/page_header.rs`；shell 渲染“标题 + 胶囊”行，删除独立 lens 工具行的通栏边框和 `border_b`。
- [ ] `semantics::tab` 改为胶囊配色；保留 selected / toggled / tab stop。
- [ ] 各页面去掉重复 h1（只针对已有 lens 的页面；无 lens 的页面通过 page_header 获得统一标题，或保持原样，以 diff 最小为准）。
- [ ] 验证：`just desktop-test-ui`；确认 `tests/native_shell.rs` 中的 lens / 焦点测试为绿。

## 3. Overview 重排（提交 3：`feat(GPUI): ✨ 重排运行概览首页`）

- [ ] 页头：范围切换和 Refresh 上移。
- [ ] KPI 单面板 4 栏（Cost 主读数）。
- [ ] 平台带：有记录的平台单独成格，其余合并为“其他来源”格，并保留原因文字。
- [ ] 趋势图：自动收紧到首个有数据日、柱形圆角、图例颜色一致、高度 200–240。
- [ ] 精确值表：倒序、零行折叠，同步修正焦点 reveal 的 index 映射并补测试；数字字体按探针结论处理。
- [ ] 右栏等高 + 数据源小节；窄宽时堆叠，KPI 2×2。
- [ ] 验证：`just desktop-test-ui`、`cargo test -p ccr-gpui overview`（以实际包名为准）。

## 4. 质量门与原生复核

- [ ] `trellis-check` 全范围检查。
- [ ] `just desktop-fmt-check`、`desktop-lint`、`desktop-test`、`desktop-test-ui` 全绿，收据写入 `research/integration.md`。
- [ ] 原生 after 截图：Overview / Claude workbench / Settings × light/dark（en），以及 Overview zh 一张；与 before 并排比较，写入 `research/native-visual.md`，并列出未运行项（DPI、macOS/Linux、读屏）。
- [ ] 更新 spec：`theme-token-contracts.md` 追加 GPUI 材质成立条件与合成对比度规则。

## 回滚点

三个提交彼此独立：只回滚材质时 revert 提交 1，并让 `Material` 退回 alpha=1 即可；提交 2 和 3 不依赖 alpha 值。
