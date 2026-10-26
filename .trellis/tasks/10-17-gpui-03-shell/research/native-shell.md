# 03 Windows 原生核心验证

日期：2026-10-18。Computer Use 通过既有 @oai/sky / node_repl 操作真实 ccr-gpui 窗口；无浏览器 mock、缩放或 DPI 操作。仅使用 outputs/desktop/shell 下的合成 fixture，无 HOME 改写。

## 首次原生观察：部分结果，仍需修复后复验

- 普通 launcher `outputs/desktop/shell/launch-native-fixture.ps1 -RunName shell-core-normal`，实际 PID 52368；产物 SHA256 0efb818550bfc371f61e9c871bb468d519d92e32f24a710c280116cba95758e8、独立锁 9c8f5b816b01a2db2285d0092fa223e791ffe3f48f02657ec4edad1708c7216f。启动截图显示真实 2 行、z-last Disabled 与 a-first Current、Demo data 标记和深色原生外壳。
- Computer Use list_windows 唯一选取 CCR Desktop (GPUI)，get_window / activate / capture；include_text=true 仍返回 UIA null。没有据此判 native role/name/state 已验证。
- 实际点击筛选框后可见 caret；type_text 的 a-first / -first 未显示，Return 后也未出现。实际 press_key(a) 可见字符 a。此差异尚未归因为应用或注入工具，不能将筛选、Unicode 或 IME 计为本轮通过；下一轮使用实际字符按键与系统 IME 区分。
- 独立 checker 同时发现新 UI 的清空历史缺少已批准设计要求的目标/后果确认；本轮没有执行删除。已交 checker 在关闭窗口后补两步确认、Cancel/Escape 焦点恢复与真实数据不变回归，再重建复验。
- 本轮 Alt+F4 正常关闭，实际 native_exit_code=0、launcher exit=0；helper CCR 窗口列表为空，Get-Process 无 ccr-gpui。首次 CIM 查询因 sandbox 资源访问失败（exit1），采用普通进程只读查询复核（exit0），未要求新增权限。
- 原始 receipt/stdout/stderr 在 outputs/desktop/shell/native/shell-core-normal/；截图 windows-core/read-list-dark.png 与 initial-keyboard-character.png。这里只证明该产物的真实列表、字符按键与正常退出，不宣称全套 03 验收。

## Cargo 与原生窗口占用协调

- 初始以为只读 integration test 不会重写正常二进制；实际 Cargo test 也尝试移除 target/debug/ccr-gpui.exe，因运行中的 Windows 文件锁退出 101，未替换正在验证的产物。此为协调失败，不能计为产品测试失败或通过。
- 已修正顺序：先原生退出并确认无残留，再允许任何 desktop Cargo 门禁/重建；下次原生期间不并行运行 Cargo。失败原始日志由 checker 保留。

## 未运行

修复后完整核心续验、OS UIA/读屏、系统外观事件实际切换、其他 OS 与性能尚未完成。尺寸/缩放/DPI 按用户最新指令 user-deferred，不能标通过。

## 确认修复后 f692 产物：实际核心续验

- 独立检查完成确认、取消和焦点回归后，正常桌面 test/UI/lint/check/fmt、当前完整依赖图 Rust1.95 check 和 debug build 全部退出0。当前产物 SHA256 f69266960cfad286007a4184c42cda8276b8b6987186051683aa3e1b44d99af4，锁未变。
- 同一二进制 `shell-final-probe -Probe`（PID47972）：点击 probe 输入后 type_text(a-first) 同样不显示，实际按键a可见且 preview 同步。正常退出0；profiles/registry inputs=null，无配置读取。原始 receipt 在 native/shell-final-probe，截图 probe-text-injection-not-visible / probe-real-key-visible。对照和只读源码研究没有找到03特有输入回归，不能据此盲目修改 Kit；也没有把失败注入记通过。
- `shell-final-core`（PID76196）：实际 z 筛选为1行z-last，追加q为空状态，Backspace恢复命中；Ctrl+A/Backspace 清除后2行恢复。截图 final-filter-match / final-filter-empty / final-list-dark。
- 未在当前布局观察到中文组合态：Ctrl+Space、一次已有布局切换及实际n仍只观察到普通文本。没有打开OS设置或继续枚举切换；IME仍未验证。ime-composition-not-observed.png 只保留实际观察，不能作为组合输入成功截图。
- 期间一次 capture 被其他前台窗口遮挡；未保存其内容、未针对它操作，立即重新枚举唯一CCR目标、get_window/activate/capture 后继续，旧观察作废。
- 配置详情a-first显示masked凭据、description和model默认；Back返回保留选中项。但实际点行内空白无响应、点名称才打开，选中背景也只覆盖约110px：检查 app.rs 行项目缺 w_full，交独立 checker 补整行命中与实际渲染回归。此缺陷尚未修复前不宣称完整核心验收。截图 final-masked-detail-row-hitbox-gap / final-detail-back-selection。
- 收藏页真实空读取后，实际字符按键输入ccr、点击Add，领域成功后UI显示Saved并重读出ccr；Refresh后仍有该记录。未执行任何删除/清空。截图 final-favorite-saved / final-favorite-reread，DB路径在独立 fixture 根。
- 此run正常Alt+F4，实际native exit0、launcher0；helper无CCR窗口，Get-Process无ccr-gpui。原始receipt/logs native/shell-final-core。所有 Cargo 检查在该窗口退出前保持停止，未再产生exe占用竞态。
- 下一轮仅复验整行点击修复、缺失catalog错误重试及慢读取消/退出，不再扩大输入布局或尺寸/DPI验证。

## 最终核心交接：6e89 产物

- 原生整行命中问题已由真实渲染回归复现（row193px/list1488px，exit101），补 w_full 后同一回归通过，包括右侧空白点击、详情、返回、10k虚拟行范围和稳定选择。受影响 check/lint/fmt/UI与debug build 均退出0，最新 SHA256 6e89a0990861055ad2a1c766dd9f0d5bf860f2fdd79bcdfabbf772acffb4886d。
- shell-row-retry（PID85336）实际显示缺失catalog错误；读取未创建missing目录。主会话只向本run/owned missing路径复制原领域生成合成DB，点击Retry恢复2行；随后在行的右侧空白x903/y279点击，真实打开a-first脱敏详情且选中背景覆盖整行。退出0，stderr空。截图 row-final-missing-catalog / row-final-retry-recovered / row-final-whole-row-opens-detail。
- shell-final-cancel（PID100824）正常退出0，但总生命期33.8315s超过人工延迟30s；只证明退出，取消时序不充分，不记取消pass。一次隐藏包装尝试没有创建run/窗口/receipt，也不记原生通过；仅保留空包装日志与失败观察，不臆断原因。
- shell-timed-cancel（PID97924）重新观察Reading catalog与Controlled slow read，随后正常关闭；总生命期 **26.372718s < 30s**，实际native exit0、stderr空，证明人工延迟尚未结束时关闭取消。截图 timed-pending-cancel.png。此为功能取消验证，不是性能/取消延迟预算采样。
- 六个已完成原生run均有实际Process.ExitCode=0与空stderr，精确逐run hash与时长见 outputs/desktop/shell/native/core-summary.json；未把旧产物截图冒称新产物。原profiles/registry/manifest哈希保持，收藏DB在专用fixture内创建，无ccr-gpui残留。
- 独立 trellis-check 已核对全部原始receipt/log/hash、精确取消时长与8张实际截图，确认03源码和核心工程可供04使用。UIA、IME组合态、延期尺寸/DPI、其他OS/性能与已知设计几何差异继续开放，03不标完整completed、不归档。04按父工程依赖判定正常start。
