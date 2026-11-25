# GPUI 命令工作台 · Windows 原生验收

规格 [#239](https://github.com/bahayonghang/ccr/issues/239)（子票 `#240`–`#244`）。集成分支 `feat/gpui-command-workbench-239`，基线为本地 `dev` 的 `d0f82583`。

## 构建与环境

- **构建形态**：Windows 原生 `release` 构建（`just gpui-build`，`ccr-gpui.exe`）
  - 改版前基线（`dev` / `d0f82583`）release SHA-256：`3e42adb3c6c79119ab2d39455185a2864cb7f36f0e7e593986f7f6bcd180b79b`
  - 改版后候选（`a8da12b039a714644b8beeec893fc3c6b4ce9f76`）release SHA-256：`b7c0f192a584ac078fec3f218ca83711cbe826b9a4b0081a44a4bd717ea3fb44`
- **窗口与显示**：逻辑客户区 `1280×800` 与 `960×640`；Windows 主显示器 125% DPI 缩放下 DWM 扩展窗口框实拍物理像素 `1602×1039`（宽窗）与 `1202×839`（窄窗，含原生标题栏与边框）。
- **主题与语言**：浅色（Light）与深色（Dark）；原生实拍采用简体中文（`zh`），英文文案与读数同步由 `TestAppContext` 视图测试与纯模型单测覆盖。
- **隔离夹具**：通过 `--fixture-profiles` 与 `--fixture-registry` 启动独立合成夹具（位于 `outputs/desktop/` 临时子目录，携带 `.ccr-gpui-profile-fixture` 标记以运行合成托管子进程 `--native-command-fixture`，以及未加标记夹具以触发真实 `Unavailable` 终态），不访问真实个人配置与外部网络；验收完毕已清理所有临时夹具目录与进程。

## 改版前后 2×2 对照

| 逻辑客户区 | 浅色（改版前 → 后） | 深色（改版前 → 后） |
| --- | --- | --- |
| 1280×800 | [改版前](before-wide-light.jpg) → [改版后](after-wide-light.jpg) | [改版前](before-wide-dark.jpg) → [改版后](after-wide-dark.jpg) |
| 960×640 | [改版前](before-narrow-light.jpg) → [改版后](after-narrow-light.jpg) | [改版前](before-narrow-dark.jpg) → [改版后](after-narrow-dark.jpg) |

改版前首屏为 24 个横向胶囊墙、JSON 参数框与五颗并列按钮，且收藏/最近分离在旧外壳页签中。改版后宽窗（≥ 1200px）分为左侧实色命令列表（默认「可运行 6」，顶部「目录 24 / 收藏 / 最近」分段与搜索）和右侧「命令编排 + 运行账本」实色阅读面板；窄窗（< 1200px）严格互斥渲染列表与工作列。

## Windows 原生交互与账本状态走查

| 原生场景 | 窗口与主题 | 实测现象与 UIA 核验 | 证据 |
| --- | --- | --- | --- |
| 选择命令、填参并一键收藏 | 1280×800 浅色 | 选 `history`、填 `-l 20`、点「收藏」后计数升至 `收藏 1`，按钮切换为「取消收藏」，`最近 0` 未虚构记录 | [收藏态](244-wide-light-favorite.jpg) |
| 运行中账本与原位取消 | 1280×800 浅色 | 选 `status` 点「运行」，原位替换为 danger 文字「取消」（等宽不跳位），状态为「运行中」+ polite「正在运行…」，流式输出 `out` 行 | [运行中](244-wide-light-running.jpg) |
| 成功终态与自动刷新最近 | 1280×800 浅色 | 约 10s 合成子进程结束后显示「已成功」、`10.1 秒`、退出码 `0`、`out`/`err`（warning 墨色）通道栏，顶部自动更新为 `最近 1` | [成功终态](244-wide-light-success.jpg) |
| 载入收藏不切分段 | 1280×800 浅色 | 切至 `收藏 1` 可见凭据边界提示，点击已存 `ccr history -l 20` 后载入编排区并停留在收藏分段 | [收藏载入](244-wide-light-favorites-load.jpg) |
| 清空最近行内确认与 Esc | 1280×800 浅色 | 切至 `最近 1` 点「清空最近」，展开后果说明与「取消 / 确认删除」；按 Esc 取消不删记录，再次确认后回到「还没有最近命令」空态 | [清空确认](244-wide-light-clear-confirm.jpg) |
| 深色主题运行中取消 | 1280×800 深色 | 启动后中途点击「取消」，账本转为「已取消」、`2.0 秒`、退出码 `1`，长日志行右端优雅省略 | [已取消](244-wide-dark-cancelled.jpg) |
| 帮助与失败/不可用账本 | 1280×800 深色 | 未标记夹具下点「命令帮助」进入「不可用」终态，退出码显示「–」，显示执行器不可用原因并记录 `最近 1` | [帮助不可用](244-wide-dark-help-unavailable.jpg) |
| 窄窗 drill-in 与返回列表 | 960×640 浅色 | 点击 `history` 互斥进入工作列，顶部显示「← 返回列表」，工作列可纵向滚动且账本副标题与空态无重叠；点击「返回列表」回到目录列表 | [窄窗工作列](244-narrow-light-work.jpg) |
| 窄窗运行中返回列表状态行 | 960×640 深色 | 运行 `status` 期间点「返回列表」，列表顶部显示可聚焦状态行「正在运行 ccr status · 查看账本」，点击可回到运行中账本并取消 | [运行中状态行](244-narrow-dark-running-banner.jpg) |

## 首轮实拍发现与返修复检

首轮实拍视觉复核检出两项真机缺陷：
1. 960×640 窄窗工作列缺少纵向滚动且账本头部未设 `flex_shrink_0`，虚线空态框向上压住账本副标题；
2. 宽窗长日志行右端出现字母硬裁切。

修复提交 `a8da12b0` 为窄窗工作列增加 `overflow_y_scroll`、给账本头部与空态容器增加收缩/裁剪保护，并将日志正文改为 `.truncate()` 省略加完整原文 Tooltip。重编 release 并复拍后，视觉复核逐张确认 **PASS**，且全部 17 张截图未出现本机系统用户名、真实个人配置路径或敏感凭据。

## 明确设计边界（D1 / 后端策略）

- **D1（最近不保存参数）**：保留 `ccr-desktop/src/runtime/ui_state.rs` 的静止存储安全决策与回归测试 `terminal_history_preserves_outcome_and_never_persists_arguments`。最近记录仅保存命令名与结果，不保存任意位置参数；最近行显示 `ccr <命令名>` 并载入「命令名 + 空参数」。
- **命令帮助后端白名单**：不修改 `crates/ccr-app` 安全策略；`CommandExecutionRequest::help` 同样校验命令在白名单中，前端不绕过。

## 明确未验证项（Not Verified）

以下能力未在本次 Windows 原生 release 实机走查中直接验证，不计为已通过：
- 系统屏幕阅读器语音播报（NVDA / Narrator 实机朗读；`role=Status` + `Live::Polite` 与 `ListBox` / `ListBoxOption` 语义由 headless 控件测试与 Windows UIA 树核验）
- 参数输入框内的原生中文 IME 组合输入（实机脚本通过剪贴板粘贴验证 ASCII 参数）
- Windows 高对比度模式与系统「减少动态效果」设置
- 真实 CCR release sidecar 二进制执行与真实远程 SSH 主机执行（SSH 缺 CLI 安装导航与离线恢复页由受控隔离 SSH 夹具测试覆盖）
- macOS / Linux 原生构建、渲染与多 DPI 缩放矩阵（`user-deferred`）
- 超过 8 KiB 单行与 500 行截断丢行的原生像素截图（对应截断与丢行提示、贴底跟随与观察失败重试由 `ledger` 单测与 `TestAppContext` 视图测试覆盖）
