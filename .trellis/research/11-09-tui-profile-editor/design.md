# Design：Profile 编辑器

> 承载方式已批准：就地右栏，窄屏退为全屏（PRD E10）。

## 1. 模块边界

新目录 `crates/ccr-tui/src/tui/profile_edit/`，不把编辑器塞进已有 3302 行的 `app.rs` 和 4449 行的 `ui.rs`：

| 文件 | 职责 |
| --- | --- |
| `mod.rs` | `ProfileEditor` 状态、按键处理、`EditorEvent`（Save / Cancel / None） |
| `schema.rs` | 每个平台的字段定义 `FieldSpec { key, label, kind, group }` |
| `field.rs` | 单行文本编辑（光标、按显示宽度移动、粘贴）、密钥字段、开关、枚举 |
| `request.rs` | 草稿 → `EditProfileRequest`：只包含改动字段与密钥改动，不做字段规则判断 |
| `ui.rs` | 渲染（表单、脏标记、字段错误、底部提示） |

`app.rs` 只增加：`profile_editor: Option<ProfileEditor>` 字段、`e` → `Action::EditSelected`、编辑器打开时优先路由按键与粘贴、保存时调度 `MutationRequest::Update`。

## 2. 字段 schema

`FieldKind = Text | Secret | Toggle | Choice(&'static [&'static str]) | Number | Tags`

| 平台 | 字段（键名 → 类型） |
| --- | --- |
| 通用 | `name` Text、`description` Text、`model` Text、`base_url` Text、`provider` Text、`tags` Tags、`enabled` Toggle |
| Claude | `auth_token` Secret；高级组：`small_fast_model`、`default_opus_model`、`default_sonnet_model`、`default_haiku_model`、`default_fable_model`、`subagent_model`、`effort_level`、`api_timeout_ms` Number、`claude_code_auto_compact_window` Number |
| Codex | `auth_token` Secret、`model_reasoning_effort` Choice、`wire_api` Choice、`env_key` Text、`requires_openai_auth` Toggle |
| Grok | `credential` Choice(保留 / 替换 API 密钥 / 改用环境变量 / 清除) + 随选项出现的 `api_key` Secret 或 `env_key` Text、`reasoning_effort` Choice、`api_backend` Choice、`context_window` Number、`supports_backend_search` Toggle |

Choice 的可选值取自各平台 `validate_profile` 接受的集合与现有详情字段帮助函数（`codex_reasoning_effort_field`、`grok_reasoning_effort_field`，`ui.rs:1704`、`ui.rs:1740`），实现时以平台校验器为准，测试锁定。当前值不在集合内时保留原值并标为"自定义"，不强行改写。

## 3. 交互（就地右栏）

- 打开（Wide）：右栏变为表单，标题 ` 编辑 · <name> `，左侧列表变暗且不响应选择键。右栏拆分复用布局子任务的 `DetailLayout`：固定区换成表单（按字段数取高度，超出时表单内滚动并保持当前字段可见），弹性区换成改动预览，复用 `stored_fields.rs` 的键名列与掩码规则，只列出改动键。
- 打开（Standard / Compact）：编辑器取 header 与 footer 之间的整块 Rect，纵向切为表单（`Min`）与预览（`Length(min(改动数 + 2, 8))`），列表不渲染。
- 渲染 Rect 的选择只取决于 `theme::viewport_mode`，与 `render_profile_workspace`（`ui.rs:535`）的模式分支同源，窗口缩放时实时切换，草稿不丢失。
- 键位：`↑↓` / `Tab` / `Shift+Tab` 移动字段；字符直接输入；`←→`、`Home/End`、`Backspace/Delete` 编辑；`Space` 切换开关、`Enter` 或 `←→` 在 Choice 上切换；`Ctrl+D` 在密钥字段上标记"将清除"；`Ctrl+S` 保存；`Esc` 取消（有改动时第一次 `Esc` 只提示"再按 Esc 放弃改动"）。
- 编辑器打开时 `q`、`h/j/k/l`、`o`、`r`、`x` 都是普通字符；`Ctrl+C` 等同 `Esc` 取消而非退出进程。
- 反馈：改动字段名前显示 `•`；当前字段整行用选中底色；字段错误紧随该字段一行，error 色。

## 4. 写入数据流

```
e → 从缓存快照构造草稿(含 revision) → 编辑 → Ctrl+S → 本地校验
  → 确认框(改动字段清单) → y → MutationRequest::Edit(EditProfileRequest)
  → worker: ccr_cli::application::profile_edit::edit_profile(request)
  → MutationOutcome::Edited(EditProfileOutcome)
  → toast + 刷新该平台 + 选中 outcome 中的新名称
```

- `revision`：`read_profiles`（`profile_tasks.rs:37`）在后台额外读取 `ConfigManager::new(&paths.profiles_file).snapshot()?.version`，存进 `ProfileSnapshot` 与 tab 缓存，作为请求里的期望 revision。缺失时 `e` 提示"配置版本未知，按 r 刷新"。
- 字段白名单、取值校验、Grok 凭据互斥、平台分发全部在 `edit_profile` 内；TUI 本地校验（E5）只做即时提示，最终以用例返回的错误为准。
- 不新增 `ccr-app` 依赖，不起 `ccr` 子进程。
- 请求中的密钥以 `ccr_core::Secret` 持有；`MutationRequest` 的 `Debug`（若有）不得打印请求内容。

## 5. 粘贴

- 终端初始化时 `EnableBracketedPaste`，恢复时 `DisableBracketedPaste`（含 panic / 错误退出路径，与现有 raw mode 恢复同处）。
- `Event::Paste(text)` 仅在编辑器打开时消费：去掉 `\r\n`，插入光标处；编辑器未打开时忽略。
- 风险：旧版 Windows 控制台可能不产生 Paste 事件而是逐字符按键；逐字符输入仍能工作，只是慢。Windows Terminal 下需人工验证。

## 6. 契约与 spec

- 编辑器打开期间：不允许 Apply / Off、不允许切到其它 tab；后台读完成照常更新缓存但不覆盖草稿。
- 保存走单写者；写入中编辑器关闭，右栏显示"正在保存…"；`q` 延迟退出规则不变。
- 更新 `.trellis/spec/ccr-tui/backend/profile-workbench.md`：第 2 节新增 `MutationRequest::Edit`，第 3 节新增编辑契约，第 4 节新增版本冲突 / 密钥 / 粘贴行。
