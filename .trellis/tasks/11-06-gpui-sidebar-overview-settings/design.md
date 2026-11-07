# 原生设计与实现边界

## 视觉语言

design-atlas 的 style:airy-glass 已落在 `ccr-desktop/DESIGN.md`。原生样式统一经 `ccr-desktop/src/theme.rs` 和 Kit Theme，继续使用实色内容面、系统正文、等宽数字、12/14/20/24 字级与既有间距。Tauri 截图的暖纸色随其主题变化，只提取布局、状态和导航语义。高频操作即时反馈。

## 侧边栏

对照 `ccr-ui/src/shell/MainLayoutNav.tsx` 的分组/平台色标和同目录 `MainLayoutChrome.tsx` 的 settings dock。GPUI 的 `ccr-desktop/src/shell/mod.rs` 中，nav_button 已设 selected、toggled、list_active，routes 和底部设置已有独立容器。先观察“选中无焦点、焦点在另一项、选中且聚焦”三种绘制，再调整导航行自身样式。沿用 32 高行、16 图标和既有导航宽度为起点，图标与文字按列对齐。当前项用轻填充及字重；清晰焦点环只表达焦点。保留稳定 ID、控件角色、设置独立固定区和状态栏。紧凑宽度下先保留名称，再安排偏好摘要。

## 总览

参考 `ccr-ui/src/features/usage/dashboard/DashboardView.tsx` 组合的 PlatformMatrix、UsageMovement、NextActions。原生入口 `ccr-desktop/src/features/overview.rs` 的 header、metrics、platforms、lower、actions。状态区显示真实报告日期与需要处理的原因。平台列共用一个表面；下方主区域组合时间选择、四项指标和逐日图，侧区只给能打开的下一步。空间不足时顺排，长窗口稀疏标注日期。去掉“先看状态，再看数字”和“这些入口只打开已有页面”等说明。

`ccr-desktop/src/runtime/insights.rs` 的 WorkbenchReport 提供 window_days、可选成本、会话口径和未画入的请求数；WorkbenchPlatform 只有请求、Token 和逐日请求。7D 返回窗口会话，30D/90D 的现有会话投影是全历史。复用 ResponseGate 拒绝过期回包，显示旧值时显式标注其报告窗口，或在新报告就绪前收起旧值。保留 SSH 下壳层对本地总览的限制。生产视图只读现有报告，禁止从截图抄数字、推断 CLI 已安装或添加视图层 SQL。

## 设置

参考 `ccr-ui/src/features/configs/AppSettingsView.tsx` 的摘要与章节定位。GPUI 入口为 `ccr-desktop/src/shell/mod.rs` 的 settings，以及同目录 `preferences.rs` 的 preference_controls。保留单一内容滚动区，章节入口将所选内容带入视野并让键盘可继续操作。外观放系统/浅色/深色，语言放中/英，工作台偏好放退出策略；诊断与环境承载现有执行环境、通知及远程管理。退出后果放在退出设置旁；不支持的能力仅在相关章节说明。

摘要与选择标记取当前应用状态，区分系统跟随偏好与实际明暗。继续经 apply_preferences 更新主题、语言和菜单，复用 `ccr-desktop/src/runtime/preferences.rs` 的写盘格式、代次控制和失败提示。远程管理保留已有实体和入口；从 SSH 受限页面进入环境设置时应能直接找到对应内容。
