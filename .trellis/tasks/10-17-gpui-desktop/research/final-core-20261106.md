# 2026-11-06 最后核心接合与整合收据

基线 e8329cd8；本文件区分源码、fixture/headless、原生与外部环境收据。Windows 原生子集与便携包已验证；本次记录的完整 CI 最终失败，后续续作另补最终收据，任务继续 in_progress。

## 已接业务

- OpenCode agents/commands 及 Antigravity commands 使用真实全局/项目目录、格式和原始版本；未知字段保留。OpenCode plugin清单只读metadata，不读取执行代码；Antigravity extensions只有原有只读manifest能力。
- OpenCode TUI使用原编辑器，13项主题选择写入草稿；旧keybindings投影到canonical tui.json，原源和目标版本均须核对。
- Usage主路由提供预算/定价，解除用量数据库存在性依赖。Fixture无声明路径仍拒绝；SSH不能回退Local。
- Claude hooks/statusline归并同一路径设置源编辑器，typed验证但不重序列化原JSON；Codex OAuth浏览器按钮经既有allowlist，fixture禁外部访问。
- 签到已接批量操作、事务删除、余额历史/分币种汇总、今日统计JSON导出、配置导入导出、WAF手工管理/线上校验、显式CDK、真实账户连接测试、记录筛选分页和月度dashboard。
- 配置导入原版本绑定、写前整批解密验证、已有extra_config保留；ImportSession只保无秘密元数据，apply重读hash绑定原文件。导入前verified snapshot不等同多manager原子事务，不自动全库回滚。
- core_check完成全批注册变化审查，58个变更Tauri命令签名保持一致；修复platform backup/locks祖先链接和预览秘密驻留，各窄回归1/1。领域、loopback和实际控件风险回归通过；源码检查没有用这些证据冒领真实外部服务。

## 尚未完成的验收

真实SSH/WebDAV/认证/CDK未运行；macOS机器收据缺失，01 AC1.5不关闭。WSL和窗口尺寸/DPI矩阵按用户延期。托盘/隐藏恢复及系统直接App::quit veto受现有公开API限制；不引入第二套窗口或WebView栈。所有任务与父归档硬门保持开放。


## 原生复现记录（修复前）

已提交资源批次 debug 候选通过显式 synthetic fixture 启动，Overview 显示 Demo data 与13侧栏入口。真实 UIA 点击 Settings 后主线程 stack overflow，进程 exit=1；这是产品缺陷，不能归类外部阻塞或认领原生通过。源二进制/fixture/hash见 `outputs/desktop/core-completion-20261106/native-debug-launch.json`，初始画面 `native-overview.png`，错误 `native-visible.stderr.log`。早先 Hidden 启动参数只使窗口不可见，两个本轮进程已核路径后清理；Visible 启动复现独立于该参数。修复与后续原生结果将在下文补齐。


### 栈溢出定位与低栈回归

Windows PE主线程栈保留为1 MiB；debug编译器输出确认RemoteView::render单帧781760 B。独立函数帧不直接求和冒充同时调用栈。保持Shell/preferences与产品栈大小不变，仅按原顺序拆Remote自然区块后，同一真实Settings操作在1 MiB子进程由stack overflow（3221225725）变为通过。新增Windows回归固定这一限制，不改变全局测试并行。

有限兄弟检查发现CodexAgents::render 1032672 B、Platforms::render 800976 B、Checkin render_extended 804544 B与render_operations 653840 B，同样按原UI区块拆分，保留容器层级/控件顺序/回调。Codex CRUD/源切换、主题保存、敏感确认3条，以及Checkin真实管理/取消/删除/筛选/月历1条，在子进程1 MiB下通过。汇编前后数值与真实Windows重验结果待最终追加；低栈headless不替代原生重验。


### 整合检查中修复的隔离缺口

SSH fixture最初只绑定DB/known_hosts，遗漏了确认、连接和上传暂存的默认锁路径。现已在实际阻塞写线程注入fixture锁；同一完整UI链在无环境覆盖和默认锁poison两种情况下通过，host-key/nonce规则未改。永久低栈回归同时运行Settings及完整SSH fixture，并令默认锁目录指向fixture普通文件；任何未来回落都会失败。

Windows签到统计导出曾将canonical root与大小写折叠的锁身份路径比较，误拒绝合法路径；修复使用真实canonical路径而非权限不等价的锁identity。Tauri settings_raw两条写入测试改用各自thread-local fixture锁，poison默认锁后整个模块7/7通过，保持默认并行与原断言。

总CI先后定位两处移植测试expect lint与Tauri unit绑定lint，均等价修正；已通过的root Test/Release/Audit/Governance收据保留，但不冒称这些中途失败的CI为最终成功。最终整合结果以最后一次完整命令退出码为准。

## Windows 原生与便携包收据

修复后的 debug 二进制 SHA256 `e65043ce4a14f2583d5f1552e0e0ff7ccbfc446e044f548049b7c7b7ff295db6` 使用本轮 `native-fixture` 显式路径启动，不修改真实 HOME/USERPROFILE。通过 Windows UI 实际操作全部 13 个侧栏入口，Settings、Codex agents、OpenCode/Antigravity 资源和 Checkin 工具均能渲染，无栈溢出。具体写入：预算启用及 dailyLimit=12.5、Claude MCP `native-probe`、OpenCode Markdown `native-agent.md`。MCP 原保存确认打开时请求关闭，选择继续使用后原草稿/确认保留，随后正常保存。

合成命令最初因夹具目录缺少安全 sentinel 被拒绝为 Unavailable，尚未进入进程启动；只在已确认的本轮合成根补齐精确标记后，同一应用实际运行、输出并成功结束。该准备问题不归类为 Windows 进程限制。Ctrl+Q 进入同一确认框，debug 正常 exit=0，stderr 为空。

`just desktop-package` exit=0，生成未签名 `ccr-desktop/target/package/ccr-gpui-7.5.0-windows-amd64.zip`，包含同版本 GPUI/CLI、LICENSE、哈希 manifest。解压到本轮 outputs 后，直接启动包内 GPUI，恢复 dark/zh 偏好及已保存 MCP；合成任务 Running 时 Ctrl+Q 确认退出，release exit=0，stderr 为空，随后只读枚举本轮 GPUI 进程数为 0。包内 GPUI SHA256 `0384afea8e8063593604426677e66c484073d0225cd83aaa2a70b000af554469`，CLI SHA256 `7ae37a26fde9e53ebeafbc5cf0b09fa6da8e47429df815bbe354c26a140795b6`。未启动真实 CCR 业务命令。

本轮原生截图/日志在 `outputs/desktop/core-completion-20261106/`：`native-settings-fixed.png`、`native-budget-saved.png`、`native-cancel-preserves-mcp.png`、`native-mcp-saved.png`、`native-opencode-saved.png`、`native-checkin-tools.png`、`native-portable-restored.png`、`native-portable-mcp-restored.png`、`native-portable-running-quit.png`、两份 `native-*-launch.json` 与退出码、`native-final-processes.json`。原生收据只覆盖上述 Windows 操作，不扩张到真实 SSH、外部服务、OS 通知、IME、DPI 矩阵或其他 OS。

## 集中门禁

- 最终 `desktop-fmt-check`、`desktop-lint` 均 exit=0；`desktop-test` exit=0，146 库测试及原有 integration/Python/AccessKit gates 通过，默认并行不变。日志 `checkin-path-review-{fmt,lint}.log`、`final-desktop-test-fixed.log`。
- `desktop-build-debug`、`desktop-package` exit=0，分别见 `final-desktop-build-debug.log`、`final-desktop-package.log`。
- 完整 CI 后续发现 SyncFolderInfo 源文档注释在 owner 提取时遗漏，导致生成 TS 少三行 JSDoc；恢复原 Rust 注释，字段和生成物基线不变。独立 `tauri-bindings-check` exit=0，日志 `bindings-comment-recovery.log`。
- `final-just-ci-complete.log` 与对应 exit receipt 实际 exit=1：Tauri 测试仍引用已提取的 `llmusage_adapter::cli::spawn_sync_process`。前述 Windows 原生/便携收据早于后来中断的认证、WebDAV 管理和用量导入增量，不验证这些新功能。后续修复与最终门禁见 `resume-summary-20261106.md`（续作收尾时记录）。
