# Claude Code 工作台验收

## 同机首屏对照

Windows 原生 GPUI 在相同的本机数据、英文、浅色主题及 1538 × 1126 窗口下采集前后截图。原版参照为用户提供的中文 Tauri 截图及其 ClaudeCodeView、ObserverChrome、CostAttributionTab 源码。截图保存在本地 `outputs/desktop/claude-workbench-redesign/`。

| 核对内容 | 改前 | 改后实测 |
| --- | --- | --- |
| 配置与操作 | 凭据、模型等设置向下逐行展开，刷新在页底 | 标题旁有认证与 Profiles，Profile 和本地凭据同处一行，设置与刷新在同一块 |
| 金额层级 | 三段金额排在七条横条之下，约在截图纵坐标 700 px | 三段金额并排在图表上方，数值约在 400 px。今日 $0.0000、本月 $135.74、全部历史 $786.86，与原版截图一致。订阅月费另列 |
| 费用图 | 以请求数计算条长，最多列出七条记录 | 横轴 10 月 8 日至 11 月 6 日，纵轴为 $0、$50、$100、$150。完整 30 天内有 15 个记录日；10 月 23 日及 31 日的空缺在图上断开。10 月 21 日的高点与逐日金额 $143.5663398 对应 |
| 明细入口 | 页底重复平台工具栏 | 在洞察标题旁打开完整报告，来源字段为 claude，默认进入 Daily，焦点落在逐日列表。按 ↓ 和 Enter 展开 10 月 20 日，画面和 Windows 可访问树均给出 USD 95.7127744，随后从 Workbench 返回。订阅设置也实际打开并返回 |

原生窗口文件为 `before.jpg`、`after.jpg`、`daily-detail.jpg`，Tauri 原图为 `tauri-reference.png`。首页与明细在本机实窗核验；认证和 Profiles 的目标路由及跨平台导航另由 headless 测试核验。

## 实施中修正

- 首次实窗的指标只能读出金额，不能读出所属时段。现能从 Windows 可访问树读到 Profile、各时段及金额、汇总、订阅、图表标题、日期区间和记录天数。
- 完整报告的 Sources 页按既有定义比较各平台。Claude 首页入口改为直达逐日列表并主动转移焦点。全局报告的来源比较仍可通过原有导航访问。
- 图表单测覆盖超过 7 条记录、请求数高而费用低、缺日、孤立零值和极端有限金额。原生壳层测试使用隔离数据库核对局部失败、刷新恢复、长中英文 Profile、来源隔离、键盘选中日期及精确 USD。

## 验证范围

桌面工作区的 `just desktop-build-debug`、`desktop-fmt-check`、`desktop-check`、`desktop-lint`、`desktop-test-ui` 和 `desktop-test` 全部通过。最终桌面套件通过 Python 16 项、AccessKit 专项 3 项及桌面 Cargo 各顶层结果合计 254 项，另有 6 项显式忽略。视觉证据对应上述单一 Windows 窗口，其他系统、缩放比例和窗口矩阵不计入此次实测。
