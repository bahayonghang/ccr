# 实施计划：GPUI 视觉细节优化

前置：prd.md / design.md / research/visual-audit.md 已评审；`task.py start` 后才动代码。每个 slice 独立 commit，可独立回滚。

## S1 研究收口（能力缺口）

- [x] GPUI 自定义资产路径探针：SVG 是 PNG carrier，侧栏直接嵌入正式 PNG，保留品牌原色。
- [x] Windows exe 图标嵌入路径探针：复用现有 ICO，embed-resource 编译资源 ID 1。
- [x] dashed border 能力探针：锁定 GPUI 已有原生支持，无需 div 段模拟。
- 产出：research/native-capabilities.md，记录结论与降级方案。

## S2 Token 层 + 共享 chrome

- [x] 按 design.md §1 改 ccr-desktop/DESIGN.md（新增角色），再改 src/theme.rs + 对比度单测。
- [x] segmented lens bar 替换两处 lens_button 调用点（shell/mod.rs、shell/platform_navigation.rs）。
- [x] 侧栏品牌行 + mark；侧栏选中 tint+border；状态栏签名组成（状态点/路由 hints/独立 30 秒 clock entity）。
- [x] 核实已有统一主/次按钮 pill；输入/列表角度不变，segmented decorator 最后应用。
- 验证：`just desktop-fmt-check && just desktop-lint && just desktop-test-ui`（在 ccr-desktop 目录）。

## S3 Overview 签名组件

- [x] 平台带横滚 + tick + micro label + 中性 sparkline/峰值 + dashed 空基线（源码）。
- [x] 逐日图 gridlines/段间隙/180px 高度/逐段 tooltip/exact table/零值日基线（源码）。
- [x] 总量列网格；next-actions 去嵌套与 chevron；首屏下真实四来源覆盖摘要 rail（当前窗口 light/dark 已实看，不泛化为尺寸矩阵）。
- [x] 更新 overview.rs 现有 headless 测试断言并新增精确值表键盘路径（不弱化；测试结果见集成收据）。
- 验证：desktop-test + desktop-test-ui；原生截图 overview light/dark × zh/en。

## S4 逐页巡检

- [x] workbench：daily bar 身份色、总量列对齐；保留已完成的 Claude 首页重构。
- [x] insights/usage、settings、commands、mcp、checkin、sync：逐页源码核对，只改呈现；见 research/page-source-audit.md，非原生像素验收。
- [ ] 历史截图矩阵：四页以上 × 两主题 × 两语言 × 1280×800/960×640；未运行，按用户最终归档范围排除，不追加执行、不记通过。
- 验证：`just desktop-ci`（ccr-desktop）；根仓库 `just version-check` 不受影响确认。

## S5 收尾

- [x] trellis-check 全 scope：独立审查及最终 desktop-ci / desktop-test-ui 通过；含 27 子项原 1 MiB 栈保护、Usage 真实切换、30/90 日长表可见性回归。
- [x] spec 更新：theme-token-contracts.md 明确 GPUI 角色/Tab 同色根因；brand-asset-pipeline.md 增补原色 PNG 与 ICO 消费合同。
- [x] 核心原生 QA 收据与截图索引已写入 research/native-core-acceptance-20261107.md；DPI/macOS/Linux 等未运行项不记通过。
- [x] 代码已分批本地提交，并通过 Trellis archive 归档当前任务；会话收尾日志单独记录，不推送。

源码批次自动门禁：desktop-ci exit 0，独立 desktop-test-ui exit 0，根 version-check / fmt-check 与 diff-check 通过。Windows release 内 ID 1 图标六帧原始字节与正式源相同。核心原生续查与最终用户归档授权见下文；不把自动化或窄验收泛化为未运行矩阵通过。

前一源码批次记录：当时复用既有 in_progress 任务，原生截图 AC 与尺寸/DPI 矩阵未运行，不能因源码与自动化通过而归档。该批次检查命令与结果见 research/integration-20261107.md；后续原生状态以下条为准。

后续核心原生验收（最新用户范围）：图标、Tab、Overview 精确值表已实看通过；原生发现的行焦点边框改用逻辑 focus 后，受影响回归、fmt/lint/UI 与 release build 通过，light/dark 新构建复验通过。用户明确不做尺寸/DPI 矩阵，不以此追加阻挡。详细范围、前后截图与最新哈希见 research/native-core-acceptance-20261107.md；不把窄范围结果勾成历史四页矩阵完成。

最终收尾：用户已明确授权中文分批本地提交并归档。代码已提交为 `43d1b9e`（共享外壳与工具行）、`dacaca40`（Overview 与工作台）；不推送、不补矩阵。完成状态与日期由 Trellis archive 命令写入 task.json。

## 验证命令汇总

- cd ccr-desktop && just desktop-fmt-check / desktop-check / desktop-lint / desktop-test / desktop-test-ui / desktop-ci
- 修复类（与验证分开）：just desktop-fmt
- 原生截图：真实窗口采集，禁止浏览器 mock 声称原生验证。

## _review gates_

- S2 结束后一次强模型 review（token 语义/同色根因/权限边界）。
- S3 结束后截图对照 review（签名组件与 Tauri 层级）。
- 最终 desktop-ci + 全 scope check。
