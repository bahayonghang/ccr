# 普通设置与 MCP 表单实施收据

本批承接父任务原有未提交工作，不创建重复任务、不提交、不操作真实用户配置、不运行原生窗口。HEAD 核对为 `96376961cd49b7914e3457efba2fc92e9d8ae076`。原生交互验收由用户明确延期；下列控件测试是 GPUI Kit headless，不是原生截图或 IME/UIA 收据。

## 实际字段与所有权

| 页面 | 普通控件 | 来源与保存 |
| --- | --- | --- |
| Claude Settings | model、effortLevel、theme、language、autoUpdatesChannel、permissions.defaultMode | 既有 `PlatformSession.fields` 安全投影；`Command::Patch` / `json::save_claude_patch` |
| Codex Settings | model、model_provider、model_reasoning_effort、approval_policy、sandbox_mode、personality | 既有安全投影；`codex::save_patch` |
| Grok Settings | models.default/default_reasoning_effort、ui.theme、session.auto_compact_threshold_percent/load_envrc、cli.auto_update/channel/show_tips、hints.new_session_worktree_mode/fork_worktree_mode | 新增原生 typed-patch 分支；字段/类型/枚举来自 Tauri `commands/grok.rs` 的 SETTINGS_KEYS 与 validate_settings_patch；保存仍走原 Document、managed guard、原版本 CAS |
| OpenCode Settings/TUI | model、small_model、default_agent、share、snapshot、autoupdate（bool 或 notify）；theme、mouse | 既有 JSON dirty-leaf owner；未提交字段、嵌套扩展字段保留 |
| Codex provider library | name、base_url、website_url、api_key_url、替换 api_key、新 api_key_name | 安全元数据读取；原 `CodexModelProviderStoreService::mutate_versioned`；省略元数据取捕获原记录，空替换密钥保留已存密钥 |
| OpenCode provider | name、npm、options.baseURL、替换 options.apiKey | 原 JSON provider merge owner；凭据不预填 |
| MCP editor | 服务器名、stdio command、逐行 args、HTTP URL、逐键 env/headers 替换 | 原 `McpEditSession`，固定来源/范围/原版本；Codex 使用 http_headers，Antigravity 新 HTTP 使用 serverUrl，已存 URL 字段名称保留 |

普通字段是本批明确范围，不宣称覆盖 Tauri 的所有设置字段。其余键继续可由显式 Advanced JSON patch 或原始来源编辑。Source ack、账户/OAuth、模型清单、配置来源只读层、MCP copy/preset/batch 能力均保留。

表单只提交有变化的普通键；未选 bool/enum 不会变成 false。已有 args、HTTP 地址、env、headers、API keys 不载入替换控件。替换密钥/地址/映射值使用遮蔽输入，控件的 accessibility value 不包含秘密；输入缓冲仅留在当前编辑会话，离页/关闭清除。高级草稿往返保留已输入的原始补丁和未知键，非法对象不会静默丢弃。

新增/改变的原生接口：

- `PlatformSession::provider_fields(id) -> Vec<FieldFact>`：仅安全元数据；带用户信息、query、fragment 的已存 URL 不投影。
- `McpEditSession::common_fields(name) -> Result<McpFormFields, McpError>`：仅保守过滤后的 command 与 URL 键名；无已存参数、地址、环境值、header 值。
- Grok `Command::Patch`：只接受确证的嵌套字段/类型/枚举，再执行已有 managed guard 和 Document 原版本保存；不增加新的持久化 owner。
- 原生 Codex MCP patch 对 `http_headers` 在后台合并原映射，避免只替换一项时删除省略的凭据；null 清除沿用高级协议。

## Hunt：真实失败与修复

### MCP 原有 Copy/Edit 被裁切

新批量工具行与详情内容使默认 1280×800 下操作行落在 MCP viewport 外。实际 Copy bounds 为 `y=749.5..777.5`，viewport 为 `y=49..759`；点击中心 `763.5` 位于裁切区域外，因此原有测试随后找不到来源 picker / editor。不是领域复制失败。

`features/mcp/mod.rs` 仅将选中详情的操作行移到状态元数据之前，保留全部详情和批量功能。既有 preset/copy 控件测试新增正向可见边界断言。修复后旧 MCP 四项测试全部通过，没有用放大窗口、重试或测试专用滚动绕过。

### OpenCode 测试备份路径

四平台控件回归中前三个平台成功，OpenCode 返回 Storage 且原文件未变化。确认输入事件已到达、Save 在 viewport 内后，隔离 fixture 的底层 writer probe 得到备份 Win32 OS error 3。该 sandbox Temp 根很长，原始备份路径长度依次为 Claude259、Codex258、OpenCode261；tempfile3.27 Windows persist 将原始 UTF16 路径直接交给 MoveFileExW。

新测试最初没有像既有 native fixtures 一样 canonicalize 根。将实际 fixture base canonicalize，并让 ConfigSource 与 PlatformRoots 始终使用同一根（Windows verbatim 长路径表示）后，两项表单测试通过。临时写诊断 probe 已删除；没有改变生产 writer、关闭 backup 或放宽成功断言。

### 控件秘密与输入引擎

遮蔽 Input 的 accessibility value 为 None 是正确行为；测试通过对应 scoped private buffer 验证冲突草稿保留，并断言 accessibility value 仍未暴露秘密。provider 字段重建后按既有 Kit 控件测试约定先完成一次异步 engine 同步，再输入和验证真实控件值；没有增加重试循环。

## 实际验证

Windows，pinned Rust1.98，独立 desktop target，`--locked --offline`、默认并行与 `--skip export_bindings` 保留。当前 lib test artifact 为 `ccr_gpui-c8abaaf8ffb826a3.exe`。

| 检查 | 结果 |
| --- | --- |
| `features::platforms::forms` | 2/2：四平台真实控件写入、bool、未知/秘密保留、原版本冲突草稿、离页清空；provider metadata-only 修改保留已存 URL/API keys |
| `features::platforms::tests` | 7/7：既有账户、OAuth、source acknowledgement、层清单与 TUI 控件 |
| `features::mcp::tests` | 4/4：空/读、秘密脱敏、preset/copy、真实新增/修改/冲突/删除与确认 |
| `features::mcp::editor::form::tests` | 1/1：非法 URL 不写、typed HTTP 写入、遮蔽、不载入已存秘密、原版本冲突保留私有草稿 |
| `runtime::mcp::editor::tests` | 2/2：原 source-bound 回归；三个 HTTP target 的字段/省略凭据/CAS及非法 Codex header 类型不写 |
| `runtime::platforms::grok::tests` | 1/1：类型/范围/未知键失败保持原字节，未知字段保留、managed 拒绝与原版本 CAS |
| 本批显式文件 rustfmt | PASS（skip_children=true，不格式化整仓） |

独立 check、最终 fmt/lint/整合测试由主会话接收。既有 1MiB guard 已登记本批普通设置/provider/MCP 控件及主会话 Grok/capabilities/session 分页测试名；完整守卫等待最终 session candidate API 稳定后统一运行，本收据不冒认新的 18children 已通过。原生窗口、截图、IME/UIA仍为 user-deferred / not_run。
