# T02 独立检查阶段记录

日期：2026-09-28。检查角色：`/root/check_t02`。状态：只读阶段完成，等待实现者交接；**T02 整体验收尚未通过**。本轮没有修改产品源码、任务状态或既有用户文件。

## 已检查的边界

- 读取本任务 PRD、design、implement、check context，以及实际 application、journal、guarded writer、CLI/TUI/Tauri adapter 和前端 outcome 消费路径。注入上下文通过保存的 hook 输出及相关规范文件补充读取。
- 共享入口为 `ccr_cli::application::profile_lifecycle::{apply_profile, update_profile}`；`ApplyProfileRequest` 和 `ProfileOutcome/Status/Warning` 已形成三端共同接口。实际写入在同一个同步线程执行，没有在 journal 内 await 或派生任务。
- `WriteJournal` 的 `PhantomData<Rc<()>>` 使 guard 不实现 Send/Sync；嵌套 scope 被显式拒绝。preimage 没有 Debug/Serialize 派生，只留在内存。
- `guarded_write::before_write` 在实际叶锁内读取 preimage 并比较版本；成功路径登记 postimage，rollback 逆序 CAS；不同文件的恢复失败不允许覆盖更新的外部内容。相关终态仍需最终测试复验。
- CLI、TUI backend 和 desktop service 的 apply 已调用同一应用用例；Claude/Codex update/rename 已迁入共享生命周期。Grok 既有专用 rename 路径不属于本任务 R5 的 Claude/Codex rename 范围。

## 已向实现者和主代理反馈的问题

| 编号 | 证据与后果 | 当前处理状态 |
| --- | --- | --- |
| C02-01 | `guarded_write::write_locked` 原先仅在 `AtomicWriter::write` 整体成功后登记 journal。`AtomicWriter::write` 在物理替换之后仍可能因 Unix parent-directory fsync 返回错误。因此已经可见的新文件可能没有 journal entry，随后被误报为 Unchanged。该后置失败也由 `atomic-writer.md` 明确说明。 | 实现者已接受；正在补 post-publish 反例和修复。尚未独立验证修复。 |
| C02-02 | journal Entry 原先仅保存字节。删除已有 Unix 0400 或自定义 Windows DACL 文件后再补偿，会按缺失目标的新 secret-file 策略重建，不能证明原权限恢复。现有字节比较测试不覆盖 metadata。 | 实现者已接受；正在补删除/重建 mode 与 DACL 回归。尚未独立验证修复。 |
| C02-03 | 原 `AppliedWithWarning` 文案无条件称 Profile applied；非激活 update 若 history 失败也能到该分支，导致提交语义失真。TUI 同时直接显示领域英文 message，丢失既有双语呈现。 | 实现者已将提交文案调整为中性描述，并准备 TUI 双语映射。前端和 TUI 行为待最终验证。 |
| C02-04 | journal 的 scope/path 查找原先按字面 PathBuf 比较。相对/绝对、Windows case/verbatim alias 的一致性须验证，避免漏记受管目标。 | 实现者已接受规范化检查；相对路径与 Windows alias 反例待验证。当前不能声称 alias 已正确。 |
| C02-05 | mutation 提交后，CLI detail reread、Claude update response reread 或 ancillary root resolution 的 `?` 可能把已提交结果转换成普通 Err。 | 实现者正在将详情读取改为 best effort、前置解析 ancillary root，并移除 Claude mutation response 的秘密内容/提交后查询。最终三端输出测试待验证。 |

以上是源码检查证据；C02-01/02 的真实故障注入结果须由后续日志证明，不能把静态后果推导记成已复现测试。主代理和实现者均已收到消息；本检查角色没有与仍在写入的实现者并发修改这些文件。

## 独立验证

| 命令 | 结果 | 证据 |
| --- | --- | --- |
| `cargo test -p ccr-cli platforms -- --test-threads=1` | 67 passed，exit 0 | `check-platform-validators.json` / `.log` |
| `cargo test -p ccr-codex profile -- --test-threads=1` | 18 passed，exit 0 | `check-codex-profile-validators.json` / `.log` |
| 本轮 platforms 编译产物执行 `--exact application::profile_contract::tests::profile_preflight_contract --test-threads=1` | 1 passed，内部覆盖 Claude/Codex/Grok disabled/deleted/invalid 目标与持久状态不变 | `check-preflight-independent.json` / `.log` |

第三项复用同轮已编译二进制以避开实现者的 core 故障注入编译。SHA-256：`e15a3747aec9196a680eb413f4df811b15f4a73ea6316770366858776ee69f24`。该证据早于后续 journal 修复；最终仍须重新编译并执行完整相关 contract。所有命令会话均已结束，没有待回收进程。

### 下游依赖结论

- **T04 所需契约 scoped 通过**：`PlatformConfig::validate_profile`、Claude effective-auth、Codex switch-spec 校验、Grok profile-auth-mode，以及共享 apply 的 enabled/invalid/deleted 前置策略已有实际独立验证；实现者已确认这些能力不再修改。T04 可在这些冻结契约上实施 diagnostic/report/binary，不能据此改动 application/journal 或声明 T02 整体通过。已通知主代理。
- **T03 依赖仍未通过**：公开 apply/update/outcome 接口可以阅读，但 compensation、rename 和三端提交语义仍待最终独立 contract 验证。不得将接口已经存在当成行为验收通过。

## 实现交接后的检查清单

1. 复核 C02-01 至 C02-05 的最终代码和反例结果；重点验证 post-publish failure、删除后的权限恢复、Windows path alias、首次写前外部替换。
2. 独立运行 core journal、CLI application contract、TUI adapter、Tauri service/真实 update handler 测试；每个故障阶段确认字节、指针、history、usage_count、operation-record replay 和 recovery-required 行为。
3. 核对必要 cleanup 仍由平台 apply 负责。覆盖 Codex auth-mode 切换、Claude managed-env 清理、Grok entry-state 保留；不得只删除 TUI off 调用却缺少成功路径证据。
4. 验证 Claude/Codex rename 的 current/default、新旧名、unknown TOML fields 和 secret store 一致；外部新版本不得被回滚覆盖。
5. 验证真实前端 warning/error 呈现、已提交编辑不重复 activation、secret sentinel 不进入 DTO/Debug/log；TUI 英中呈现和 CLI 提交后错误语义明确。
6. 核对新 application spec、atomic writer/journal spec 和 TUI/profile-off spec 与实际边界一致，提供新 spec 路径供下游 context 使用。
7. 执行本任务相关 strict lint、type-check、format/diff 和绑定漂移检查。正式前端 lint 的两个受保护 `.tmp` 文件 5 条 no-console 是已知工作区基线；不得排除文件后将子集通过写为完整门禁通过。
8. 按主代理统一安排补充 Linux 权限/故障验证、前端浏览器验证；Windows 单平台和模拟 IPC 结果不代表 native WebView/macOS 通过。

## 未授予的结论

本阶段没有授予完整 AC1–AC5、完整 lint/type-check、全量 Rust/Tauri、native desktop、Linux/macOS 或最终 `just ci` 通过。实现者仍在修改本任务文件；最终独立审查在明确交接后继续。没有提交、推送或归档。
