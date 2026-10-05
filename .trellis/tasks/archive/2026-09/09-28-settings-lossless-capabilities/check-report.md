# T08 独立检查报告

日期：2026-09-28。基线 commit：`34d8a85e0e48b793733835e0304c8ed33940fcee`。

本轮完成 T08 范围检查，并修复无损补丁、清空反馈和字段初始化问题。最终相关前端检查为 **32 files / 119 tests passed**，TypeScript、生产前端构建、受控 ESLint、i18n 和 Grok 16 项测试通过。正式 `lint:ci` 仍被两份原有临时脚本的 5 条 `no-console` 阻断。T09 状态机、Web 视觉、原生桌面与完整集成门禁仍需后续验收。未修改任务状态，未提交或归档。

## Findings (fixed)

### F1：Codex 补丁重送未编辑的快照字段

- 文件：`ccr-ui/src/configs/settings-codex-map.ts:149`、`settings-patch.ts`、`tests/platforms/settings-lossless.smoke.test.ts:24`。
- 问题：原 fixture 将 Codex 模拟为顶层对象替换。实际 `commands/codex.rs:1241` 的读取函数仅投影已知字段，`:1150` 的 `apply_tui_setting` 逐叶合并。原 mapper 将 snapshot 的所有 TUI sibling 重送，能覆盖加载后由外部修改的 notifications。
- 修复：Codex 使用空 source 构造仅含 dirty leaf 的补丁。未知扩展由后端保留。Claude/OpenCode 继续按其真实顶层对象替换契约保留 snapshot sibling。
- 验证：校正 Codex fixture 的已知字段投影、逐叶合并和 null 删除行为。新增加载后外部修改 notifications/custom 的反例；修复前补丁包含旧 notifications，修复后只含 `{ tui: { animations: false } }`。初始通知数组、未知枚举、未知字段和原始 snapshot 均保持。

### F2：Codex 清空可选字段后报告成功但未删除

- 文件：`ccr-ui/src/configs/settings-codex-map.ts:153`、`settings-codex.ts:36`。
- 问题：空文本或数字编成 `undefined`，domain JSON 转换会省略该项，导致空补丁保存并重新显示旧值。后端已有 `null -> None` 契约，见 `commands/codex.rs:848`、`:908`、`:931`。
- 修复：只将 Codex dirty patch 中的 undefined leaf 转为 null。未编辑项保持缺席；推理枚举和 notifications 控件提供明确的“未设置”选项，null 读取值显示为空。
- 验证：`settings-lossless.smoke.test.ts:115` 经真实 mapper/domain wrapper 执行清空 model、整数、枚举、嵌套文本和 notifications → 保存 → 重读。请求含对应 null，重读为空，其他字段不变。

### F3：Claude/OpenCode 不支持的顶层移除被报告为成功

- 文件：`ccr-ui/src/configs/settings-claude.ts:180`、`settings-opencode.ts:100`、`settings-types.ts:59`、`features/platform/settings/BaseSettings.tsx:58`。
- 问题：Claude 的 model/noOverride、三个数字和其他可空顶层字段，以及 OpenCode 顶层文本，会产生被序列化省略的 undefined。现有 JSON 顶层替换接口没有已验证的键删除协议；写空串不能证明“移除覆盖”的语义。
- 修复：根据主会话确认的能力边界，在任何 mutation 前拒绝这类 patch。`SettingsValidationError` 经 Base 的翻译和既有通知链显示说明。Claude 提示从“源文件”删除键；OpenCode 说明 typed API 的限制。草稿保留，未发成功通知。未新增删除协议或向其他平台套用 null。
- 验证：mapper 用例覆盖 Claude model、maxThinkingTokens、maxOutputTokens、cleanupPeriodDays，以及 OpenCode model、smallModel、defaultAgent、theme。路由测试挂载真实 `ToastContainer`，检查中文错误说明在 DOM 中可见、输入保留、mutation 为 0、success 为 0。另验证受支持的嵌套 section 删除、空数组和空对象仍保存。
- 文案：`src/i18n/locales/{zh-CN,en-US}.ts` 各新增两条说明；`tests/i18n.test.cjs` 的精确 leaf count 从 4501 同步为 4503。

### F4：稳定 t 引用下普通输入框没有正确初始化

- 文件：`ccr-ui/src/features/platform/settings/SettingsFieldControl.tsx:36`。
- 问题：memo 控件中的普通 input/textarea 未订阅字段值；表单 reset 后没有更新注册与默认值。原 smoke 每次返回新的 t 函数，掩盖了该问题。改用稳定 t 后，Grok model、Claude 数字和 OpenCode 文本等 7 项用例出现永久空值。
- 修复：普通控件使用字段级 `useWatch`，并设置 `defaultValue` / `defaultChecked`；保留 RHF 非受控注册，未增加强制重绘。
- 验证：稳定 t 的原失败用例通过。`settings-capabilities.smoke.test.tsx:171` 检查初始 reset 后文本可见、textarea 值可见、编辑跨 tab 保留、保存请求正确、服务端规范化值重读后更新实际 DOM、dirty 状态清除，以及下一次编辑重新启用保存。

### F5：源码冲突和确认顺序缺少可区分的 fixture

- 文件：`ccr-ui/tests/platforms/settings-capabilities.smoke.test.tsx:231`、`:298`。
- 问题：原 raw fixture 直接返回预设 conflict；即时确认只证明回调被调用，不能证明文件读取等到确认完成。
- 修复：raw fixture 比较请求 token 与模拟磁盘 token；在编辑后修改外部内容和 token，断言旧 token 冲突且外部内容不被覆盖。用可控未决 Promise 验证明文确认前不读 raw/layers，确认后只读取一次。
- 保留真实 CodeMirror DOM 和状态，未将编辑器替换为 textarea mock；未降低 CSP nonce/readable stylesheet 检查。

## Findings (not fixed)

- **T09 既定范围**：`BaseSettings.tsx:26`、`:30` 的 query key 尚未包含执行环境身份；`:42` 仍在 snapshot 变化时 reset。dirty refetch、环境切换与迟到响应隔离由 T09 接手。本轮只修控件注册，不改这些状态机。初始 WSL/SSH 阻断测试通过不代表中途切换已验收。
- **正式 lint 基线**：`ccr-ui/.tmp-desktop-probe.mjs:20,24,92` 与 `.tmp-insights-visual.mjs:236,381` 共 5 条 `no-console`。这些用户文件不在 T08 修复范围，没有修改、删除或忽略。
- **能力边界**：Claude/OpenCode 上述顶层移除现在明确拒绝。新增 typed 删除协议需要单独确认领域语义；不能将当前限制描述为已支持删除。
- **验收边界**：未进行浏览器视觉、生产 WebView CSP、原生桌面、真实 WSL/SSH、真实账户配置保存或完整根/Tauri 门禁。主会话已安排 T07/T09 后集中验证；T08 本轮不能宣布发布或全部验收通过。

## 逐 AC 实际证据

| 条目 | 已执行证据 | 结论与边界 |
| --- | --- | --- |
| AC1 / R1 | `settings-lossless.smoke.test.ts:84`、`:102`、`:134`；路由 `settings-capabilities.smoke.test.tsx:142`。真实 mapper/domain wrapper 经 mock transport，验证 Codex 只改 model 时无 notifications/未知枚举字段；OpenCode notify 保留且不进入无关 patch。Codex 外部 sibling、嵌套扩展和 snapshot 不变断言通过。 | 自动化通过。fixture 根据实际后端投影和合并语义；未操作真实用户文件。 |
| AC2 / R2 | `settings-capabilities.smoke.test.tsx:123`、`:154`：managed 输入 disabled，aria-describedby 对应原因文本，Profiles 恢复链接，当前未知枚举，整数边界，unset/false 分离。typed conflict/managed_locked 保留草稿并显示恢复反馈。 | 自动化通过。权限和锁仍由后端承担。 |
| AC3 / R3：入口与冲突 | Claude/Codex/Grok 路由挂载实际编辑器，未决明文确认、拒绝确认、读取 token、外部 token 冲突、invalid 标记、成功关闭并重载表单；Grok WSL/SSH 只调用环境探测，Claude remote raw 不读文件。 | 路由自动化通过。未验证生产 WebView/真实 remote 环境。 |
| AC3 / R3：无备份 | `commands/grok.rs:2198` 的 fixture 在临时目录放置已有 config 和两处备份 sentinel，比较所有非目标 config、非 lock 文件路径与字节。typed/raw 成功，以及 typed invalid、raw invalid、raw stale 均保留备份清单；失败保留配置字节。页面 no-backup/policy 提示断言通过。 | 独立 Grok suite **16 passed**，包含新增 fixture。Windows 临时文件验证通过，不代表全部 OS/原生检查。 |

## Verification

前端命令目录为 `ccr-ui/`；Cargo、rustfmt、Git 命令目录为 repository root。

| 检查 | 本轮最终结果 |
| --- | --- |
| `bun run test:smoke -- tests/platforms tests/configs tests/ui/code-source-editor.smoke.test.tsx tests/quality/hardcode-px-rgba.smoke.test.ts tests/api/api-facade-boundary.smoke.test.ts` | **32 files / 119 tests passed**，exit 0。 |
| `bun run type-check` | **通过**，exit 0。 |
| `bun run lint:ci` | **失败**，只有两份原有临时脚本的 5 条 no-console，exit 1。最终修改后复跑仍为相同定位。 |
| 受控 `bunx eslint … --quiet` | **通过**。显式覆盖 T08 的 settings configs、Settings/shared editor、原路径 re-export、相关 smoke、两份 locale 和 i18n test。只作为 T08 诊断，不替代正式 lint。 |
| `bun run lint:style` | **通过**。正式 lint 在 ESLint 停止，因此独立执行。 |
| `bun run check:style-lines` | **通过**；脚本当前匹配 0 个 `.module.css`，覆盖范围有限。 |
| `bun run check:arch-boundaries` | **4 组违规 fixture** 全部按预期拒绝。 |
| `bun run check:cycles` | **731 文件**，无循环依赖。 |
| `bun run test:i18n` | **24 passed**，4503 leaf key；key-leak 自检通过。新增两键后的精确计数已同步。 |
| `bun run build`，仅该命令设置 `CCR_SKIP_ICON_GENERATION=1` | **通过**，exit 0；保留已有 configLoader、Tailwind、大 chunk 提示。没有新增图标改动。 |
| `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::grok::tests -- --test-threads=1` | **16 passed / 529 filtered out**，2 suites，exit 0。 |
| `rustfmt --check --edition 2024 ccr-ui/src-tauri/src/commands/grok.rs` | **通过**。 |
| `git diff --check` | **通过**。LF/CRLF 提示不计为 whitespace error。 |

### 检查期间的失败与修复记录

- Codex 旧 mapper 在忠实 fixture 下发送未编辑 sibling；新断言先失败，再随局部修复通过。
- 稳定 t fixture 使 7 项输入行为用例失败；仅增加等待未解决问题。字段订阅修复后通过，另补初始/后续 reset、tab 与 dirty 行为回归。
- 新增两个 locale key 后，i18n 精确计数仍为 4501，导致 2 项失败；同步计数至 4503 后 24 项通过。
- 首次独立 Grok Cargo 命令在执行用例前遇到 T06 在途编译错误：`llmusage_adapter/cli.rs:16` 无法导入 `ManagedChild`，另有 3 条 `SyncProcess` bound 错误。已通知主会话，由 T06 owner 修复。原始输出保存在 `C:/Users/lyh/AppData/Local/rtk/tee/1790596235_cargo_test.log`。随后独立复跑 16 项通过；未将首轮编译失败记为测试通过。
- 实施报告中的 16 项与强化无备份单例属于实施者记录；本报告的 16 项是本 reviewer 复跑结果，二者明确区分。

## 改动与后续交接

- 检查修复只涉及 T08 mapper/控件/测试/规范，以及主会话明确交给本 reviewer 的两份 locale 和对应精确计数。`commands/grok.rs` 本轮只审阅其测试增量，未改后端生产逻辑。
- `platform-surface-contracts.md` 已补充真实合并语义、Codex null 删除、其他平台删除限制和稳定翻译引用下的字段订阅契约；其余两份实施阶段规范与当前源码核对一致。
- 原始编辑器迁移保留旧 TS/TSX 入口 re-export；没有扩大 feature 跨域 import 豁免。
- 两份用户临时脚本本轮读取的 SHA256 前后相同：`.tmp-desktop-probe.mjs` 为 `02E1AEA2AD36CFD1D56E62852F1462D0F9C96A439A1380EAF03E0D4B14912C85`；`.tmp-insights-visual.mjs` 为 `AC8FE1E9E9DE9738FBC5B2CE44F4D66322EB7D83B83823A38168314C41F47D53`。
- T09 继续处理 draft/refetch、环境身份和语言/错误规范；请保留本轮稳定 t、真实 Toast、清空能力和 reset DOM 回归。T10/root 执行根与 Tauri 完整组合门禁，补 Web/native 验收，并维护父任务 ledger。
