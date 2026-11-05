# GPUI 10 本机 13 路由截图

日期：2026-11-05。机器：Windows，主显示器 125%。PRD 10.1–10.6 不记通过。没有归档。尺寸/DPI 矩阵和其他操作系统仍未跑。纸面是实色，这次不声称背景模糊。

## 启动

仓库根目录、无额外参数：

```text
just desktop-dev
```

该命令没有传入 fixture 路径、`CCR_DESKTOP_LOCAL_PLATFORM_PROBE` 或演示开关。进程环境只有 Windows 兼容层 `__COMPAT_LAYER=DPIUNAWARE`，用来让加载器在创建窗口前把进程标成 DPI 无关。`ccr-gpui.exe` 的内嵌清单仍写着 `PerMonitorV2`；兼容层覆盖的是这一次启动，没有改清单。窗口标题是 `CCR Desktop (GPUI)`。启动时外壳已经是浅色。没有用 fixture 切主题。

`just desktop-dev` 在窗口收到 `WM_CLOSE` 后退出码 0。结束后没有残留 `ccr-gpui`。

## 1280×800 设备像素的做法

没有把更大的位图缩放后冒充原生图，也没有对 Per-Monitor V2 窗口调用 `SetWindowPos` 把客户区改成 1280×800 物理像素。那样会把逻辑布局收成 1024×640。

兼容层生效后，在 DPI 无关线程里读到 `GetDpiForWindow = 96`，`GetClientRect = 1280×800`。逻辑客户区和位图是同一个 1280×800。抓取使用 `PrintWindow` 的 `PW_RENDERFULLCONTENT`，再按 `ClientToScreen` 裁到客户区。外框是 1296×839，裁切起点是 (8, 31)，源矩形和目标矩形都是 1280×800，没有缩放。左上角像素是场景底色（228, 243, 238）。

UI Automation 会把调用线程改成每监视器感知。若在那之后直接 `PrintWindow`，同一扇窗口会报出 1600×1000，位图右下角是黑边。所以每次打印前用 `SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_UNAWARE)` 把线程拨回无关，再读客户区并打印。

## 侧栏

上一轮 1024×640 逻辑布局里，运行概览留着绿色焦点环，当前路由只有灰色底，设置项被状态条挡住。原因有两处，都在 `ccr-desktop`：

- 侧栏按钮按下时不接收键盘焦点，所以已经落在「Overview」上的焦点环不会跟着路由走。路由变化时，如果焦点还在侧栏里，就沿侧栏 Tab 顺序把焦点移到当前项。
- 当前项再用绿色描边标出。设置项固定在侧栏底部，上面的路由放进可滚动区域，避免最短窗口把设置项画进状态条。

这一轮 13 张图里，打开的路由和侧栏选中项一致。设置页的设置项完整露在状态条上方，并且是选中状态。离开概览后，「Overview」不再带着绿色选中环。

## 截图

目录：`.trellis/tasks/10-17-gpui-10-acceptance/research/screenshots/`。13 张都是 1280×800。

| 文件 | 路由 | 像素 | 画面 |
| --- | --- | --- | --- |
| `01-dashboard.png` | `/` | 1280×800 | 真实本机用量。报告日 2026-11-05。7 日窗口：请求 2,957，token 472,285,568，费用 $264.98，会话 139。Claude Code 与 Codex 有记录。Antigravity、OpenCode 写明该窗口无记录，请求和 token 为 0。 |
| `02-agent-sessions.png` | `/agent-sessions` | 1280×800 | 真实归档。本页 100 条已索引会话。标题和路径来自本机。 |
| `03-mcp-manager.png` | `/mcp-manager` | 1280×800 | 真实本地 MCP 文件。侧栏选中 MCP Manager。 |
| `04-claude-code.png` | `/claude-code` | 1280×800 | 真实配置目录。侧栏选中 Claude Code。 |
| `05-codex.png` | `/codex` | 1280×800 | 真实配置目录。侧栏选中 Codex。 |
| `06-grok.png` | `/grok` | 1280×800 | 真实配置目录。侧栏选中 Grok。 |
| `07-antigravity.png` | `/antigravity` | 1280×800 | 诚实状态：找到 legacy settings.json，类型化设置不支持，值保持隐藏。账户不支持，不报告登录状态。 |
| `08-opencode.png` | `/opencode` | 1280×800 | 真实本地 OpenCode 表面。侧栏选中 OpenCode。没有展示密钥。 |
| `09-commands.png` | `/commands` | 1280×800 | 真实命令目录，28 条。当前是 status，参数 `["--json"]`，还没有任务。不可用命令标了 Not available here。 |
| `10-checkin.png` | `/checkin` | 1280×800 | 诚实空状态：本地库没有已保存的签到账户。没有编造余额。 |
| `11-sync.png` | `/sync` | 1280×800 | 诚实缺失：没有找到规范同步配置。不读旧文件，没有测试远程访问。 |
| `12-usage.png` | `/usage` | 1280×800 | 真实用量库。请求 16256，token 2882413684，USD 1021.42082559，上次同步 2026-10-22T03:20:45Z，提供方维度可用。 |
| `13-settings.png` | `/settings` | 1280×800 | 外观与语言。浅色为选中。设置项完整可见并选中。文案说明偏好只在当前窗口会话生效。 |

导航点的是真实侧栏。按钮没有 InvokePattern，所以按 UIA 边界的中心做了鼠标点击，同名按钮取最靠左的那个。离开概览后，当前按钮的 ToggleState 是 On，「Overview」是 Off。没有出现 “Home insights”、“Apply / refresh”、双语斜杠调试句或示例数字。

概览在库缺失时的源码常量仍是一句中英斜杠说明，并含 “sample numbers”。这台机器已经读到用量库，13 张图里没有这句话。

## 验证

| 命令 | 退出码 |
| --- | --- |
| `just desktop-fmt-check` | 0 |
| `just desktop-lint` | 0 |
| `just desktop-test` | 0 |
| `just desktop-dev` | 0 |

用量 SQLite 字节断言这一次都通过。没有重跑，也没有放宽断言。

没有改 `ccr-ui`、Tauri wire 或 `crates/ccr-usage`。没有回滚 `.trellis/spec/**`、`.trellis/tasks/10-17-gpui-desktop/**` 或 `code_map.md`。没有跑 `just ci`。

## 仍未运行

PRD 10.1–10.6 保持未通过。DPI 矩阵、macOS、Linux、性能、打包和 `just ci` 仍是未运行项。不要归档。
