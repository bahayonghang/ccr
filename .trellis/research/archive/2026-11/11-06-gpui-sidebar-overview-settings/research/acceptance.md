# 实施验收记录

## 侧边栏

- Windows 原生窗口，1282 × 831，英文浅色，读取本机用量数据。改前 `outputs/desktop/sidebar-overview-settings/before-overview.jpg` 和 `before-settings.jpg` 复现了粗圆框。
- `outputs/desktop/sidebar-overview-settings/sidebar.jpg` 中当前总览有轻填充，平台图标按身份色排列，设置摘要显示 Light · English。使用 Windows Graphics Capture 拍摄。
- `outputs/desktop/sidebar-overview-settings/sidebar-keyboard.jpg` 记录同一窗口按 ↓ 后的状态：焦点到 Agent sessions，总览仍是当前页面；随后按 Enter，原生可访问树出现会话归档控件。
- `native_shell` 通过 9 项，忽略 1 个显式 fixture 准备入口。自动化覆盖 13 个路由、点击、F6、方向键、Tab/Enter 和设置摘要同步。

## 总览

- 首轮 Windows 原生检查发现窗口选择器保留 Kit 粗焦点框、刻度贴图边未对准柱心，以及指标在 Windows 可访问树中只有无名 text。修正后用同一窗口重新拍摄 `overview.jpg`、`overview-30d.jpg`、`overview-90d.jpg`，均在上述 outputs 目录。
- 本机报告日 2026-11-06。7D 原生可访问树显示 Requests 2,673、Tokens 431,314,989、Sessions · 7D 133；30D 显示 Requests 16,256 和 Sessions · all time 417。30D/90D 均提示 2,221 条总数内请求未在逐日图绘制。日期刻度与对应柱位对齐。
- `overview_` 过滤命令通过 5 项，覆盖缺库、报告切换/失败和旧响应、精确数字、已有行动入口。
- 归档读取失败显示不可用，仅在共享 snapshot 成功且未建索引时显示未索引。状态栏的来源路径只能证明已配置；实际读取结果由总览报告。两种归档输入和 UI 精确值回归均通过。

## 设置

- 本机常用窗口的英文浅色截图为 `settings.jpg`，与 `overview.jpg` 同尺寸。第一版用了默认资源包未包含的图标，最终版本换用已打包的 Globe、Info 和 Settings。实窗复核了四个入口、选中勾、主题与语言摘要，以及固定设置入口。
- 实际点击 Diagnostics & environment 后内容定位到执行环境，再按 Tab 聚焦 Post a system notification，见 `settings-environment.jpg`。没有发送通知或建立 SSH 连接。四个章节的首项都通过 headless 焦点与可见边界断言；SSH 受限页的设置深链在隔离回归中定位到环境章节。
- 使用独立 fixture 偏好目录切换为深色、中文和直接退出。新进程恢复相同选项。截图 `settings-zh-dark.jpg`、`settings-restarted.jpg`。存储失败、原文件保护及重试另由自动化覆盖。

## 最终门禁

- 在 `ccr-desktop/` 运行的 `just desktop-fmt-check`、`just desktop-lint`、`just desktop-test-ui`、`just desktop-test` 全部通过。完整测试包含 Python 16 项、AccessKit 专项 3 项、Rust 顶层结果合计 250 项通过及 6 项显式忽略。忽略项为既有基准和辅助进程入口。
- 首次完整测试时原生调试窗口占用了 Cargo 要重建的 exe；该次失败。正常退出后重跑通过，之后的视觉验收运行独立副本。
- 验证对象为 Windows GPUI 的三个改动切片。未进行多平台、DPI 矩阵或全仓 `just ci`。截图为本地交付物，源说明与核对记录随任务保存。
