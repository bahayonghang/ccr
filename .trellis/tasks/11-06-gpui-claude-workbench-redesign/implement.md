# 实施与核验

- [x] **定位基线。** 以用户图 1 为改前依据，从正常导航打开 Claude 工作台，记录可比的原生窗口、语言与主题。对照 Tauri 的 `ClaudeCodeView`、`ObserverChrome` 与 `CostAttributionTab`，确认已有原生绘图能力与逐日明细入口。
- [x] **重组页面（R1、R4）。** 将 Claude 标题、配置摘要和洞察区按设计稿排列，更新刷新位置，移除页底重复链接。验证主次操作、焦点、读取生命周期及其他平台的默认首页。
- [x] **接入费用与图表（R2、R3）。** 复用已加载快照，分别渲染三个时段、订阅和 30 天摘要。用原生控件绘制带日期与金额轴的走势，核对每个点与报告。隔离 fixture 覆盖超过 7 点、缺日、单点、零值、空与部分失败。
- [x] **执行核心回归。** 在 `ccr-desktop/` 优先扩充 `tests/native_shell.rs` 的 `platform_workbench_is_default_and_scoped_navigation_keeps_platform` 同组用例。先跑受影响行为，再跑 UI 套件。
  ```powershell
  python scripts/desktop.py cargo test --locked --test native_shell platform_workbench -- --skip export_bindings
  just desktop-test-ui
  just desktop-fmt-check
  just desktop-lint
  ```
  若新增库内图表投影单测，单独执行对应测试并记录实际条数。
- [x] **原生视觉走查。** 从同目录运行 `just desktop-dev`，进入「Claude Code → 工作台」。用常用窗口比较 R1 的首屏、R2 的金额层级、R3 的刻度与缺口、R4 的操作与焦点。记录前后截图、窗口尺寸、语言与主题；若数据变化，在对照说明里标出。窗口不可观察时标记为待验；headless 控件断言单独记录。范围限本页及直接受影响的路由。
