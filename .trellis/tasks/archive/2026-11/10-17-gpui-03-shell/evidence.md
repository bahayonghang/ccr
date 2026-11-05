# GPUI 03 实施证据

日期：2026-10-18。状态：in_progress；以下是恢复、边界与实际执行记录，尚不代表本任务验收完成。

## 恢复与批准范围

- 已读取自身 prd/design/implement、implement/check curated manifests，以及父设计、ccr-desktop/DESIGN.md 和 02 的 shell-source-map / shell-a11y-seam。02 AC2.1–2.5 已独立通过；01 可用工程依赖与未完成的平台/性能验收按父 implement.md 分别跟踪。
- `python .trellis/scripts/task.py start .trellis/tasks/10-17-gpui-03-shell` 实际退出 0，planning -> in_progress。未新建计划或重复请求实施批准。
- 最新用户指令仅验证核心功能；不执行缩放/窗口尺寸/DPI 矩阵。AC3.1 对应验证保留 user-deferred；仍实现原有自适应边界。未运行项不计通过，父完整验收不据此完成。

## 写入负责人

- shell_ui / trellis-implement：app/lib/theme/shell/locales、必要的 native ui_state 呈现、native_shell/UI 测试、README 与 DESIGN Known Gaps；不写 Cargo 或领域持久化。
- shell_domain / trellis-implement：ccr-db UiStateManager 的注入和 fallible read、desktop runtime/ui_state 与 mod 接线，以及全部 desktop Cargo manifests/lock/dependencies；不写 UI 文件。
- 主会话：任务/父台账/证据、API 协调与核心原生操作。所有参与者共享工作区、保留他人修改；同一文件仅一个当前 owner。独立 check 在实现冻结后接手检查与必要修复。

## 核心边界

- 真实 Local/Claude 配置、命令收藏/已完成命令历史、实际主题/语言设置；不把 route bookmark、伪造历史或尚未接入业务页装成能力。
- 复用 ccr-db 的 schema/SQL/迁移/标识和持久化；fixture 使用显式私有 pool，禁止默认全局 pool 或修改 HOME。所有数据库/文件操作在 02 后台运行时执行。
- 单一 semantic theme 映射、真实键盘区域/焦点、稳定列表 ID、同一原生节点的 role/name/state。Kit input 的 IME 路径保留；headless 语义测试不视为 OS 屏幕阅读器证据。
- 命令完成历史写入由 08 的真实完成事件接入；03 不制造可计数的虚构 job 完成。

## 验证

实施中，实际命令、退出码、截图及未运行项待实现与独立检查后补入。领域与 UI 原始结果分别写 research/domain-evidence.md 与 research/ui-evidence.md。

### 主会话：原生 fixture 预备

- Python 显式复制 02 领域生成的合成 profiles.db / config.toml / fixture-manifest.toml 到独立 outputs/desktop/shell/native-fixture，退出 0；拒绝覆盖已存在目标，无用户目录读取或 HOME 改写。
- profiles.db SHA256 为 1efbf727f1946196bba2d50e6528c1d4b1fe79e54be943b1d031bd63eedd32d5；完整文件来源、大小和 hash 在 outputs/desktop/shell/fixture-copy.json。后续 ui-state.db 由领域迁移在此独立 fixture 根初始化，不能落入真实 .ccr-ui。
- 复用严密 Process.ExitCode launcher 到 outputs/desktop/shell/launch-native-fixture.ps1；尚未启动 03 原生应用，准备成功不是 UI 验收。
- 任务和台账一致性脚本退出 0：保留 348 command / 76 route / 18 redirect，只有 02 的 list_configs 为 verified_core，route 完成数为 0。03 两份 manifest 各 9 条有效；04 已补共享服务、原子写、fixture 与 profile 编辑语义，现各 9 条有效引用。

### 领域冻结与独立预检

- shell_domain 已明确冻结产品文件和 manifest/lock；实现者记录 ccr-db 127 项、后台适配器 2 项、严格 lint 与局部格式检查通过，详见 research/domain-evidence.md。独立 trellis-check 随后再次运行完整 ccr-db 127 项与 all-features clippy，均退出 0；没有发现领域缺陷或修改产品。
- 领域原有日志两处改用 record ID，原始 args/full_command 不进入列表投影。既有 redactor 不识别任意 token=value 自由文本；最初错误哨兵断言失败被保留，不能声称任意用户标签自动去密。真实凭据不得由应用代码填入可见标签。
- 主会话仅修改 child justfile 的 desktop-test-ui 一行，使它实际包含 native_shell 和 native_ui；CLI/ui/tauri 入口未改。
- 05 的 54 command / 8 route 源码研究已保存 research/platforms-source-map.md 并加入 05 implement/check curated context（各 7 条）。这是只读预研，不是提前实施、激活或验收 05。

### 首次原生与检查修复

- 首次实际 0efb 产物显示真实 fixture 两行及深色外壳，实际字符按键可见；原生正常关闭 exit0、stderr 空且无残留。profiles/registry 三个复制源 hash 均保持，未访问收藏页时 ui-state.db 未初始化；断言脚本退出0，原始结论 native/shell-core-normal/review.json 明确 result=partial。
- 文字注入与实际字符按键观察不一致，Unicode/IME 和完整筛选不计本轮通过；后续用真实按键/IME与同产物探针对比定位，不盲目改 Kit。具体截图、回执与未运行项见 research/native-shell.md。
- 独立检查发现清空历史与移除收藏缺少已批准设计要求的不可逆确认，已在唯一 owner command_state.rs 补目标/后果、Confirm/Cancel、Escape/导航取消与焦点恢复；尚待最终门禁和修复后原生复验，不请求重复实施批准。
- 原生运行时并行 cargo test 触及同一 exe，Windows 文件占用导致实际 Cargo101；保留失败。已修正协调顺序，退出并确认无残留后才允许任何 desktop Cargo 检查或重建。未把这一协调错误记为产品测试结果。
- 独立根 version-check 与 fmt-check 均退出0，原始日志 outputs/desktop/shell/check-root-{version,fmt}.log；未执行 repair 命令。

## 本阶段核心交接结果

- 独立 check 自修并复验确认流程与整行点击两项实际缺陷；完整桌面检查通过：Python12、AccessKit3、Rust lib8、native_shell2、native_ui2，以及严格lint/check/fmt、当前依赖图Rust1.95。最后整行修改后受影响UI/聚焦回归/lint/check/fmt和debug build再次退出0；无依赖变化，不无理由重跑整个根测试。
- 最新产物 6e89a0990861055ad2a1c766dd9f0d5bf860f2fdd79bcdfabbf772acffb4886d；独立锁 9c8f5b816b01a2db2285d0092fa223e791ffe3f48f02657ec4edad1708c7216f。原生读取、匹配/空筛选、详情/返回、收藏保存重读、错误重试、整行点击、慢读取消和正常退出均有真实观察；截图与逐产物原始回执见 research/native-shell.md。
- 最后取消run生命期26.372718s小于30s人工延迟，实际native0；此前33.8315s仅正常退出、取消时序不充分，保留且不计取消pass。所有6个已完成run实际退出0/空stderr，无残留。隐藏启动未成功的尝试不计通过。
- 独立checker复核所有receipts/8张实际截图后给出可消费工程交接。task仍in_progress：AC3.4/3.5通过；AC3.1 user-deferred，AC3.2原生UIA/读屏未验证，AC3.3当前布局IME组合态未观察，系统外观事件/其他OS/性能和已知几何差异仍开放。
- 父台账03六项已有实际映射/证据或明确08真实完成事件待接理由；get_favorites/add_favorite计verified_core，get_recent/remove/clear为implemented_test_verified，add_recent等待08，不制造历史。累计核心命令3、完整route0，保留348/76/18。
- 04已依正常task.py start激活，沿用父工程依赖/开放验收分开规则，不宣称完整03或父迁移完成，不归档/提交/推送/发布。
# 2026-10-27 resumed airy-glass increment

The user explicitly resumed existing tasks. Original evidence above remains historical. Current solid fallback implementation and independent review are in `research/airy-glass-implementation.md`, `research/airy-glass-check.md` and `research/airy-glass-capabilities.md`. Before-release identity/native reference is in `research/airy-glass-before-native.md`.

Final checked candidate SHA256 `ed00067d7a6a2f88356c69d1b6e24417680ba7f9c3c170e49a3ca893e9d0eaab` passed fmt-check/check/strict lint/full desktop-test/debug-build; source/gates are frozen in `outputs/desktop/airy-glass/2026-10-27/check/candidate-receipt.json`. Headless painted-quad tests reproduced and fixed opaque children masking paper corners. Only an opaque sheet is implemented because the locked renderer lacks in-app backdrop sampling; no fake blur or OS transparency substitution.

Main then observed the actual candidate in dark/light and English/Chinese, performed configuration metadata/CAS/apply, missing-catalog Retry and pending-read close. Three fixture processes exited0. Exact results/failures are in `../10-17-gpui-04-profiles/research/native-solid-candidate-20261027.md`, summary `outputs/desktop/profiles/native/solid-core-summary-20261027.json`. These do not count as S1/S2 performance or IME/UIA proof.

Native keyboard acceptance is currently FAIL: opening profile Off confirmation from an outside sidebar focus lets Tab escape and Escape fails to dismiss. A separate draft-cancellation status is also stale. A checker is fixing these actual precondition gaps;03/04 are not complete. Deferred size/DPI, OS UIA/IME, other platforms, glass, system-preference integration and full performance remain open.05 and independent07 consume stable services/theme while their own acceptance stays separate. No commit/push/archive was performed.

# 2026-11-05 workbench navigation

Sidebar is the 13 workbench destinations from `ccr-desktop/DESIGN.md`. Content roots are `page-dashboard`, `page-agent-sessions`, `page-mcp-manager`, `page-claude-code`, `page-codex`, `page-grok`, `page-antigravity`, `page-opencode`, `page-commands`, `page-checkin`, `page-sync`, `page-usage`, `page-settings`. Settings stays a bottom dock, not a fifth section. Favorites, recent commands, converter, and install are in-page controls on Commands, MCP Manager, and Usage. Claude/Codex/Grok mount the existing local profile catalog; OpenCode and Antigravity mount the existing local settings reader. MCP Manager, Check-ins, and Sync say they are not connected and do not invent counts. Usage and agent sessions use the existing local reads, not the Home insights bench. Unindexed stays unindexed. The sheet stays the solid fallback. This pass does not claim backdrop blur.

AC 3.1 stays user-deferred. AC 3.2, 3.6, and 3.7 are not passed. No new native screenshot, UIA, IME, DPI, or performance sample was taken.

Commands from the repo root, after `just desktop-fmt` repaired formatting in the files this pass edited:

- `just desktop-fmt-check` exit 0
- `just desktop-lint` exit 0 (`cargo clippy --locked --all-targets -- -D warnings` for ccr-gpui finished clean; a pre-existing ccr-cli compile warning was printed and was not a clippy failure)
- `just desktop-test` exit 0: Python unittest 12 ok, AccessKit harness 3 ok, ccr-gpui lib 58 ok, native_shell 6 ok, native_ui 2 ok

The first `just desktop-fmt-check` before formatting exited 1 on rustfmt diffs only.

# 2026-11-05 check

Independent check of the 13-destination shell. Sidebar order matches `mainLayoutNavSections` plus the settings dock: Overview, Configuration (Agent sessions, MCP Manager), Platforms (Claude Code, Codex, Grok, Antigravity, OpenCode), Tools (Commands, Check-ins, Sync, Usage), then Settings. Favorites, recent commands, converter, and install stay off the sidebar. Each destination keeps its `page-*` root. MCP Manager, Check-ins, and Sync stay explicit not-connected notices. Overview still shows unindexed sessions as unindexed. The sheet remains the opaque fallback. AC 3.1 stays user-deferred. AC 3.2, 3.6, and 3.7 are not passed. No backdrop blur, native screenshot, UIA, IME, DPI, or performance sample was claimed.

Check fixes: instruction-file, install, and command notices now show one language; the usage/session product paths no longer render “Home insights”, “Apply / refresh”, or the bilingual source-refresh sentence. Direct insights tests still exercise the real usage and archive reads.

Commands from the repo root, after the check fixes:

- `just desktop-fmt-check` exit 0
- `just desktop-lint` exit 0 (ccr-gpui clippy clean; the existing ccr-cli compile warning was not a clippy failure)
- `just desktop-test` exit 0: Python unittest 12 ok, AccessKit harness 3 ok, ccr-gpui lib 58 ok, native_shell 6 ok, native_ui 2 ok

The first full `just desktop-test` in this check exited 101 because two insights archive renders required a language global that those tests do not install. The status line now uses the optional language lookup. The suite above is the rerun after that fix.
