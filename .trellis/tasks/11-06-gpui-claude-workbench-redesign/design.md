# Claude 首页布局与数据映射

## 首屏草图

```text
Claude Code                                [打开认证] [管理 Profiles]
当前配置    <当前 Profile>    <本地凭据状态>               [刷新状态]
本地设置    <模型及其他已读取的设置>

用量洞察                                   [完整用量报告] [订阅设置]
今日 · USD             本月 · USD             全部历史 · USD
<金额>                 <金额>                 <金额>
<金额来源；有订阅时另列月费>
近 30 天 · <请求数> · <token 数> · <记录费用>

近 30 天每日费用 · USD
<纵轴金额，横轴日期；有缺日则断线>
```

- 用 GPUI 的一张实色工作面组织内容。页面标题 20、分区 16、正文 14、辅助 12、主要金额 24，使用现有主题。数值用系统等宽字体，对齐同一基线；本月可单独使用低强调底色。图 2 的米色与金色属于该截图主题。
- 空间不足时先换行操作和指标，再纵向滚动。复用 Kit 按钮和焦点反馈，键盘操作即时生效。趋势图提供稀疏日期刻度、明确的 USD 轴和可读明细入口。
- 顶部平台导航是完整子页目录。认证和 Profiles 属于页头，完整报告和订阅属于洞察区。Tauri 的 `ccr current` 提示在原生首页对应真实的配置摘要，不增加命令执行入口。

## 源码与边界

| 用途 | 来源与约束 |
| --- | --- |
| 原版结构 | `ccr-ui/src/features/claude/ClaudeCodeView.tsx` 的 PageHeader、配置提示与 UsageInsightPanel；同模块 `observer/ObserverChrome.tsx` 的金额与报告/订阅操作；`observer/CostAttributionTab.tsx` 的每日费用图。 |
| 原生页面 | `ccr-desktop/src/features/workbench.rs` 中的 `open`、`current`、`usage` 与 `render`。仅 Claude 使用专属布局，复用已有读取、焦点、挂起和 generation 校验。 |
| 指标 | Profile、CredentialState 和设置字段来自平台读操作。三段金额使用 `observer::Report.insight` 的 `today_value_usd`、`month_value_usd`、`total_value_usd`，月费来自独立的 `subscription` 结果。30 天摘要及曲线使用已限定 Claude 与日期的 `InsightsSnapshot::Usage` 和 `trends[].cost_usd`。按共享投影的真实含义描述费用，不从 Tauri 截图移植价格版本或价目表声明。 |
| 部分可用 | `ccr-desktop/src/runtime/insights/observer.rs` 支持 usage 成功、insight 缺失、subscription 未配置的组合，各区域独立显示状态。以完整 30 天窗口定横轴，对投影没有返回的日期断线，保留实际记录的零点。 |
| 路由 | `ccr-desktop/src/shell/platform_navigation.rs` 的 `link_lens` 负责现有路由。报告走 Claude 的 `Lens::Usage`，订阅走 `Lens::Observer`。复用报告的逐日明细，验证键盘能读到日期与金额。 |
| 原生绘图 | `ccr-desktop/src/theme.rs` 提供字号与间距；`ccr-desktop/src/features/overview.rs` 可参考图例、刻度和空态。优先检查锁定 Kit 的原生绘图能力，本任务无需 WebView 或新的图表运行时。 |

回退边界是 Claude 专属渲染分支及其测试。此次布局无需迁移数据、写入配置或升级协议。
