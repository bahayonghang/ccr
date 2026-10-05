# Web 预览初步检查

2026-09-28。用户已批准包含 UI 验证的父子任务规划。使用 ccr-ui-visual-workflow 和 tabbit；未读取或修改真实用户配置。

- 启动：`bun run dev:web -- --host 127.0.0.1 --strictPort`，实际地址 `http://127.0.0.1:5173/`。根会话进程 session 57820，后续验收沿用，不重复启动。
- Tabbit 任务：`CCR 架构验收`，新建独立标签页。原有用户标签页未被接管。
- 已通过 DOM 观察访问 `/`、`/claude-code`、`/claude-code/settings`。1718×1015 视口未出现 document 横向溢出。
- 总览明确显示 Web 预览无法调用 Tauri 后端。Claude Settings 最终为错误和 Retry 状态，原因文本为 `Cannot read properties of undefined (reading 'invoke')`。该预览不能验证 Settings 的真实读取、保存和 IPC，不据此标记后端链路通过。
- Settings 展示 `⟦claudeSettings.subtitle⟧`。HEAD 的 settings-claude.ts 已包含该 key，当前两种 locale 无对应字段；属于本轮实施前已有的文案缺失，单独记录，不把环境错误与文案缺失合并归因。
- 初次 link 定位使用不含空白的名称正则，等待超时后确认 URL 未变化。随后依据已观察到的 href 直接进入 Settings。该选择器错误没有计为产品缺陷。
- Tabbit `page.screenshot` 在字体加载后等待截图超时，返回 `SCREENSHOT_CAPTURE_FAILED`；没有取得可审阅图片，不声称完成视觉验收。DOM 检查可继续，最终 UI 验证在 T07/T09 后重新收敛。

本文件为中期证据。最终结果由后续完整 UI 检查记录补充，不能以本文件替代浏览器成功交互或原生 WebView 验收。

## 后续工具状态

尝试用 Tauri 官方 mockIPC 做浏览器内合成 fixture 时，Tabbit 返回新 task ID，当前页为 about:blank，模块导入在首步失败，fixture 未安装、没有执行保存。原任务消失原因未查明。按 recovery 指引核对 inventory 并重取原标签页，工具随后报告 ENTRY_TAB_NOT_ATTACHED，重新选择也未发现该页；未将这些错误视为产品失败。

已执行 finish --task 'CCR 架构验收' --discard，返回 finished=true、forcedCleanup=false，关闭本任务新建的空白页 1608964205。本轮没有取得合成 fixture 的浏览器交互结果。后续 UI 验证可在所有前端改动收敛后重新执行，不得宣称本次保存、截图或原生验收通过。

## 同日浏览器补验

使用独立 Codex in-app browser 标签页 1，沿用端口 5173 的现有预览进程。`cua_repl` 已实际返回运行概览与 `/claude-code/settings` 的 AX tree 和页面截图（823×916）。当前视口下设置错误卡片和 Retry 按钮可见，没有覆盖标题或页面主要内容。截图以工具输出留存；没有将其声称为已保存到文件的产物。

设置页仍出现既有 subtitle key 缺失及缺少 Tauri invoke 的错误。没有读取真实配置、执行保存或安装mock IPC；该结果仅补充 Web 错误状态的视觉证据，不替代 T08 正常表单、T07 作业交互、T09 编辑会话及 native IPC 验收。首次 AX 元素点击因元素属于前一页面被工具拒绝，随后读取新状态并使用已观察路径导航；没有把工具索引错误视作产品缺陷。

## T07 源码冻结后的 Web 检查

2026-09-28，沿用 in-app browser 标签页 1，从可见导航进入 `/commands`。AX tree 与截图均显示命令面板、编排区和运行账本；页面明确标明 Web 预览无法启动 Tauri 后台任务，运行按钮 disabled。DOM 读取的 CSS viewport 为 866×900，document scrollWidth 为 866，无页面横向溢出。截图只保留在工具输出，没有本地截图文件。

没有点击运行、修改真实配置或安装 IPC mock。该项验证覆盖空闲 Web 布局及不可用状态，不代替真实运行、取消、路由恢复和原生事件交付验收。T07 的行为验证以真实 React 页面配合受控 IPC 的自动化及 T11 后端测试为证据，T09 完成后另行核对受影响页面。


## T09 Web Auth error surface — 2026-09-28

Root used the existing in-app browser tab and local preview at http://127.0.0.1:5173/grok/auth during T09 implementation. Accessibility and screenshot show the page title, explicit `Tauri runtime is unavailable for get_current_environment` error, and one visible Retry action. No signed-out success state or indefinite loading was displayed. DOM viewport width and document scrollWidth both equal 866 px. Screenshot exists only in the tool response, not as a saved local artifact.

The zh-CN page used an English `Retry` default label; root reported the newly visible label to the T09 owner for localization. This observation is pre-freeze and only covers a plain-browser missing-runtime state. No real credentials, backend mutation, or remote configuration was accessed. It does not prove native login, refresh success, or filesystem behavior. Navigation clicks from the Grok overview did not change the route in this session; direct navigation to the observed auth link succeeded. The reason for the click behavior was not established.


## T09 frozen Web error surfaces

After the frontend source freeze, root reloaded `/grok/auth`: the explicit missing-runtime error remains visible and the action now reads `重试` in zh-CN. Root also visited the observed `/grok/settings` route. Accessibility/screenshot show the Settings title, explicit environment error, and Chinese retry action. DOM reports zero editable main inputs; viewport width and scrollWidth are both 866 px. Screenshots are tool response evidence only, not saved files.

The temporary agent-created browser tab was closed. The agent-owned Vite preview session 57820 was stopped with Ctrl-C (exit 1 from interruption), after these observations. No product files or user configuration were changed by browser verification. Native WebView, successful real IPC, accounts, remote files and interactive save remain unverified in this Web check.
