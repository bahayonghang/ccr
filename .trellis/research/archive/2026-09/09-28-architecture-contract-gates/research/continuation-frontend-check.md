# T10 本轮前端翻译切片非作者复核

日期：2026-09-28。审查者：`/root/implement_t10_specs`。本轮 5 文件前端增量和 root 后续新增的两处规范条款通过复核。产品阻断问题为 **0 项**；发现并由 root 修正了 1 处规范测试描述，当前未解决发现为 **0 项**。正式前端聚合门禁的既有失败单独保留。

## 范围与修复前字节

审查依据为 `continuation-tauri-lint-implementation.json`、两个 before SHA 文件及 `continuation-source-freeze.json`。5 个当前文件全部匹配实施者 after SHA 和 root freeze；freeze 共记录 262 个文件，本次没有审查全部冻结文件。

| 文件 | 本轮语义增量 |
| --- | --- |
| `ccr-ui/src/i18n/locales/zh-CN.ts` | 新增 Claude subtitle、Codex save 和 4 个推理深度选项，共 6 个叶子键 |
| `ccr-ui/src/i18n/locales/en-US.ts` | 新增对应的 6 个英文叶子键 |
| `ccr-ui/tests/i18n.test.cjs` | EXPECTED_LEAF_COUNT 从 4517 调整为 4523 |
| `ccr-ui/src/configs/settings-codex.ts` | 仅将 4 个 CODEX_EFFORT labelKey 改为完整翻译路径 |
| `ccr-ui/tests/platforms/settings-visible-i18n.smoke.test.tsx` | 新增 1 项真实 Settings View 和译文函数的双语回归测试 |

4 个既有文件的 before 字节已经在内存中还原，SHA 与原 before 记录逐一完全一致。还原包含补丁上下文发生的 CRLF/LF 差异，未向源码写入还原结果。新测试在 before 元数据中标为新文件、before SHA 为 null，目前未被 Git 跟踪。本报告不虚构新文件创建前的字节快照。完整 before/after SHA 和可读的本轮 diff 位于同名 JSON。

同文件中的历史改动均排除：auth stale、settingsSession、raw-source clear、command job/history、profiles outcome 文案，以及 Codex dirty-patch、环境探测、raw source、unset 和 load/save 逻辑。本轮叶子键基数使用 **4517 + 6 = 4523**；Git HEAD 的 4501 基数不属于本轮 before。

## 代码与验收

- Claude `subtitleKey` 的既有消费者为 `settings-claude.ts:44`，`BaseSettings.tsx:60,93` 在正常和不可用状态读取相同翻译键。两种语言均补齐该键，没有改变页面状态逻辑。
- `BaseSettings.tsx:64` 使用 `${config.i18nPrefix}.save`；新增 `codex.settings.save` 与消费者一致。
- `SettingsFieldControl.tsx:31` 调用 `t(option.labelKey)`；4 个 labelKey 使用已存在的 Codex Settings 命名空间。option 的 `minimal/low/medium/high` 值完全不变。
- 新测试挂载真实 `CodexSettingsView`、`BaseSettings` 与翻译实现，仅模拟 Tauri IPC。测试在同一个已加载页面切换 zh-CN/en-US，断言字段标签、Save、4 个已知选项及 `future-effort` 未知值，同时确认只有 1 次 `codex_get_settings`。
- i18n 计数调整对应两种语言各 6 个新增叶子键。测试文件其他断言未改变，未降低检查要求。

未知值验收只覆盖当前表单的值与显示内容。新测试没有执行保存、文件落盘、真实 IPC admission 或原生 WebView。Claude subtitle 在本轮获得目录及消费链核对，未新增 Claude 页面渲染测试。

## 独立执行结果

| 本审查实际运行的命令 | 结果 |
| --- | --- |
| `bun run test:smoke -- tests/platforms/settings-visible-i18n.smoke.test.tsx` | 1 文件、1 测试通过，exit 0 |
| `bun run test:i18n` | 24 项通过；两种语言各 4523 叶子键；key-leak self-test 通过，exit 0 |
| `bun ./node_modules/eslint/bin/eslint.js` 加本轮 5 文件 | exit 0，无输出 |
| `git diff --check` 加本轮 5 文件 | exit 0，LF/CRLF 提示保留 |

3 条 Bun 命令的完整 argv、开始/结束时间、exit code、stdout/stderr 和 SHA 保存在同名 JSON。5 个源文件在执行前后 SHA 完全一致。stdout/stderr 分别采集，其保存顺序不表示事件时序。没有运行 Cargo 或完整 CI。

另外核对了实施者的 4 项前端验证日志 SHA 和原失败日志 SHA，全部匹配。实施者记录的 Settings smoke 为 4 文件/45 测试；本审查独立复跑范围为新增标签测试 1 项，两者没有混计。原红测保留 Save 与 4 个 effort 标签泄漏的失败结果。原失败版本没有完整源字节快照，最终源 SHA 不归属于红测时点。

## 附录：本轮新增规范

只审查 root 新增条款，未重新审查旧规范。两份规范在内存中去除本轮新增条款，并恢复触及上下文的换行后，均精确匹配 root freeze 的原 SHA。

### Dependency Governance

`dependency-governance.md:58,68,81-82,95` 的 5 条新规则与已验证的安全切片一致：独立双锁审计、默认 fetch 与数据库 revision、精确版本/checksum/依赖列表/package 数、保留 warning 与失败记录、禁止新增 ignore，以及锁定依赖不等于 active feature/target graph 的边界。开发工具链通过不替代实际 MSRV 验证的表述准确。

本次仅核对新条款与 `continuation-security-check.md/json` 已验证事实的对应关系。此前由本审查者编写的其他规范内容不计为独立检查。

### Platform Surface Contracts

`platform-surface-contracts.md:98-104` 的 explicit-path persistence 条款与 `codex_settings.rs:25,34-38,46-87` 一致。生产 handler 调用同一个私有 read/merge/write helper；测试使用 TempDir，保存后读取实际文件，检查新 model、完整 notifications 数组，并比较除 model 外的全部 TOML 语义值。条款明确排除 State、IPC admission、cache invalidation 和 native WebView；该边界准确。本审查仅核对该描述，没有重复进行 Rust 产品审查或运行 Cargo。

`platform-surface-contracts.md:105-107` 最初写为断言 visible headings，但新增测试实际断言字段标签。审查已通知 root，root 将表述改为 **visible field labels**。已读回确认，该发现关闭。最终规范与同一会话内的双语切换、保存动作标签、枚举标签和未知值断言一致。

## 正式门禁与剩余边界

已核对父任务 `research/root-continuation-frontend.json/log`：正式 `just frontend-check` 返回 **exit 1**。TypeScript 阶段通过，随后 ESLint 在两份受保护文件报告 **5 条 no-console**：`.tmp-desktop-probe.mjs` 3 条、`.tmp-insights-visual.mjs` 2 条。两文件当前 SHA 均与 continuation-baseline 相同，本审查未修改、忽略或移动这些文件。

本轮切片通过不能替代正式前端门禁通过；根任务的其他 aggregate 结果由 root 管理。本轮没有启动浏览器或桌面，jsdom 测试不构成真实浏览器、原生 WebView 或完整视觉验收。

仅写入本 Markdown 与同名 JSON。没有修改产品源码、规范、任务状态、父汇总或矩阵；没有派生代理、提交、推送或归档。
