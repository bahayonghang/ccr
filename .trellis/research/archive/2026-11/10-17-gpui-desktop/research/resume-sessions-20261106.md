# Codex 会话操作续作（2026-11-06）

实际缺口：原生会话页只有索引和正文读取，尚无现有 Codex Markdown 导出、克隆和删除的产品入口。

## 实施边界

- 选中的 opaque archive ID 经共享 sessions resolver 和 provider registry 验证后，才在 bounded worker 恢复真实来源；UI 不传任意物理来源路径。仅 Codex live file 支持这些现有操作，其他提供方/归档目录不错误映射。
- Markdown 复用 `CodexSessionService`，显示建议文件名和领域截断状态；显式 Copy Markdown 写入剪贴板，不隐式落盘。预览及原始正文离页/切换读取即清除。
- 克隆显式确认，核对原来源版本，使用既有克隆服务生成新标识并刷新 CCR 索引。副作用由 app-owned runtime 持有；离页仅移除观察订阅。
- 原生删除遵循现行 recoverable-session-trash 规范，按钮为“移入 CCR 废纸篓”，复用 `CodexSessionTrashService`。新增窄 versioned 入口验证原路径/来源 token，保留原有清单、rollout 副本、会话索引移除与 CLI 恢复流程；源文件删除使用 guarded CAS。失败保留 active 来源或报告可能部分完成，不虚构取消。
- 操作 token 读取限制 64 MiB；linked/reparse 来源或废纸篓/索引祖先拒绝。fixture 写锁在实际 blocking worker 内绑定隔离目录。成功移动后用原 `usage_repo` 更新归档删除状态；源操作成功但归档更新失败单独反馈。
- 确认绑定原 archive identity、路径和版本；焦点延迟回调另有 opening generation。保留现有 Tauri 命令签名/载荷和永久删除入口语义，不新增 restore UI。

## 验证

- `cargo test --locked --lib session_operations -- --skip export_bindings`（`ccr-desktop/`）exit=0，2 tests，1.00s：opaque 校验、来源冲突不写、clone/export/trash、CCR 归档标记、实际 CLI owner 恢复；原生控件 headless 导出/剪贴板、取消与离页 plaintext 清除。
- `cargo test -p ccr-codex --locked confirmed_source_version -- --skip export_bindings` exit=0，1 test，0.05s：领域 source-version 过期保留原记录，成功后的同内容恢复。root Cargo lock 的 sandbox 拒绝经自动批准后对同一 fixture 命令执行通过。
- 1 MiB 栈复验：先正常 `cargo test --locked --lib --no-run --message-format=json` 得到 test executable，再仅对测试进程设置 `RUST_MIN_STACK=1048576`，执行 `selected_codex_controls --skip export_bindings`。初次真实测试进程复现新 renderer 栈溢出，按原顺序拆为 confirmation/actions/feedback/export 四个自然区块后，同一命令 exit=0、1 test、0.91s；没有增加产品栈或变更控件树/回调。
- 更早一次把 `RUST_MIN_STACK` 施加到 Cargo 导致 **rustc 编译线程**栈溢出，测试未执行；该尝试与后续实际测试进程复现分开记录。
- 无 computer use、真实用户会话/外部服务、WSL 或 SSH 操作。Headless/fixture 证据不替代原生平台收据。

## 独立检查修复：克隆绑定同一份字节

独立检查发现 `clone_session_versioned` 原先比较 token 后调用旧克隆入口，后者会重新读取正在写入的会话，可能克隆未经确认的追加内容。现改为 bounded source snapshot 只读取一次，token 核对、原会话标识解析和克隆转换均消费同一份字节。既有无版本入口继续原读取流程，两入口共用原目标分配/标识与时间戳重写/缓存更新；仅将既有 parser 的 reader 循环抽为私有入口以复用原解析规则。

`cargo test -p ccr-codex --locked clone_session -- --skip export_bindings` exit=0，2 tests、0.01s。新回归覆盖外部改动拒绝，及捕获后源文件标识变化并追加消息时，只克隆确认内容、保留未知字段、原文件保持最新外部内容；旧无版本克隆回归同时通过。没有增加 test hook 或第二套转换逻辑。

## 低栈回归持久化

既有 Windows `environment_views_fit_native_windows_stack_with_isolated_locks` 子进程测试列表追加实际 session controls、observer subscription setup、Codex API account confirmation 三个测试全名；保留原 1 MiB 栈及 child-only 默认锁 poison，不新增框架或产品改动。`cargo test --locked --lib features::environments::tests::environment_views_fit_native_windows_stack_with_isolated_locks -- --skip export_bindings`（`ccr-desktop/`）exit=0，helper 1/1、3.59s；内部五个实际测试逐个成功，避免普通测试线程较大栈再次掩盖新核心控件溢出。
