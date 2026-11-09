# Windows 原生可行性探针

日期：2026-10-17。执行者：主会话，computer-use 技能与独立 node_repl 的 `@oai/sky`。对象是实际编译的 GPUI 窗口，不是浏览器 mock。仅使用合成文本，无业务配置、账户或网络操作。

## 被测产物

- 可执行文件：`ccr-desktop/target/debug/ccr-gpui.exe`，49,963,008 bytes。
- SHA256：`7d58450248e04ac958d464723f12ef01e838dbaf6422de27a1a9f7eb71d9af4d`。
- `ccr-desktop/Cargo.lock` SHA256：`f05306cb38264a91afd699843ab7e67f380e180698ed31b9e39d6f62fd39cacd`。
- 构建模式 debug，只作原生可行性验证；未运行 GPUI 性能采样。
- 默认 client 请求尺寸来自源码 1280×800 logical px；helper 捕获整窗 1282×831（含系统装饰）。未量测 target-window effective DPI，不能据此判定 100% 或 DPI 矩阵通过。

## 实际操作与结果

1. `sky.launch_app` 使用上述明确 exe 路径，无参启动，调用成功。
2. `sky.list_windows` 按精确标题 `CCR Desktop (GPUI)` 唯一选定窗口。helper 将子进程归入父宿主 app identity；实际截图确认目标为本次 GPUI 探针。未操作宿主应用的 UI。
3. 首次 `get_window_state` 报窗口 minimized；按工具说明 `activate_window`、`get_window` 后重新 capture 成功。可见 GPUI 标题、双语说明、真实 Input 控件及预览区。
4. 点击输入框后可见焦点环与插入位置。`sky.type_text('CCR 中文输入')` 后输入与下方预览一致，保存 Unicode 注入截图。此步单独不能证明 IME。
5. 键盘 `n` 输入一个普通字符；`Ctrl+Space` 切换组合模式，再按 `n`、`i`，出现系统中文候选“你”等。输入框带组合串，预览仍保留已提交内容 `CCR 中文输入n`，没有提前提交 `ni`。
6. Space 选择首个候选，候选窗口消失，输入与预览同时变成 `CCR 中文输入n你`。保存组合态与提交后截图。随后再次 Ctrl+Space 恢复该探针的输入模式。
7. Ctrl+A 可见文本选择；Tab 在只有一个输入框的探针内未出现第二个控件。不能据此判完整 shell 的 Tab/Shift+Tab 导航通过。
8. 尝试拖动右下角缩小窗口，capture 尺寸未变化；未把这次尝试记录为最小尺寸验证。四种尺寸矩阵仍未运行。
9. Alt+F4 正常关闭，`sky.list_windows` 返回 0 个同标题窗口；随后 `Get-Process -Name ccr-gpui` 返回 0 个进程，命令退出 0。未强制终止进程。

每次输入均以新 capture 为依据，输入后立即刷新。原生工具调用成功不使用 shell exit code 表示；独立产物/退出进程核对命令退出 0。

## 原始截图和元数据

保存于 `outputs/desktop/native/windows/`（重建产物，未自动提交）：

- `probe-unicode-input.png`
- `probe-ime-composition.png`
- `probe-ime-committed.png`
- `probe-keyboard-selection.png`
- `probe-desktop-dev.png`
- `probe-capture.json`

## 实际公开入口复验

- 仓库根 `just desktop-dev` 首次在 sandbox 内执行：退出 1。路径校验先成功，随后 mbx 读取现有缓存报 `Access denied (os error 5)`；未把此失败记为启动成功。
- 同命令经必要的 require_escalated 重试，使用现有工具与缓存：session 14605。实际执行子工程 verify，再 `cargo run --locked --bin ccr-gpui`，14.73s 构建后运行新产物。
- 通过 `sky.list_windows` 唯一选中本次新窗口，激活并保存 `probe-desktop-dev.png`；截图确认为真实 GPUI 探针。Alt+F4 关闭后同标题窗口数为 0，session 14605 最终退出 **0**。
- 因此已有实际 root public entry -> child workspace -> cargo run -> visible native window -> normal exit 的闭环，而不只是 dry-run 或直接 exe 启动。
- 该次重建后的 debug exe SHA256：`fbcbfb5bd72e87e4d1866cb507cdc147bbe7687ee389b667190a08007a3b80e0`，对应 `probe-desktop-dev.png`。前述 IME 截图对应首次 debug 产物 hash，两次产物分别记录，不混同。
- 同时核对 release exe SHA256：`8d179ea6c5d9b6f5d243c8c0503aabb1c4f568891e328c380591bc980da1f4ec`；lock hash 保持不变。release 只完成构建，本次未启动或测量。

## 仍未验证

- 所有 `include_text:true` 调用的 accessibility 结果均为 null。不能证明原生 role/name/state、屏幕阅读器或焦点树满足合同；03 必须排查并验证。
- 未验证 light/dark 切换、英文整套文案、最小窗口/四种尺寸、100/150/200% effective DPI、跨显示器移动、恢复后的完整焦点导航、IME 取消/更多编辑组合或禁用/错误/加载状态。
- 未采集任何性能原始样本；Tauri 基线因隔离/计时条件缺口仍未采集。不能判性能预算通过或比较改善百分比。
- 未获得 macOS/Linux 构建或原生运行 receipt。Windows 输入探针成功不代表 01 AC1.5 整体通过，更不代表父任务 AC4/AC7 通过。
