# 隐藏的 platform profile 与顶层旧命令的调用方

研究票 #168（地图 #167）。代码基线 `origin/dev`@`16cbc6fa`。只读调查：没有构建，也没有运行二进制，行为结论都来自源码和测试断言。标了「推断」的地方没有实测。

## 结论

1. **`ccr platform profile …` 可以直接删除，没有调用方卡住它。** 仓库内没有任何运行时调用。`ccr-vscode` 拼的是 `ccr <平台> profile <动作>`（`ccr-vscode/src/services/ccrCliArgs.ts:31-33`），函数名里带 `PlatformProfile`，argv 首项却是 `claude` / `codex` / `grok`。这组命令今天已经只返回迁移错误（`crates/ccr-cli/src/cli/dispatch.rs:462-464`），五个动作的参数只被解析、不被消费。删除时要跟着改的只有三处：两个断言迁移错误的测试（`crates/ccr/tests/commands/legacy_routing.rs:99-107`、`crates/ccr/tests/commands/platform_profile_surface.rs:114-126`），以及规格 `.trellis/spec/ccr-cli/backend/grok-profile-runtime.md:250` 里「`ccr platform switch/profile` 继续返回迁移错误」这句。`ccr platform switch/current/info/init` 同样没有运行时调用方。
2. **顶层旧命令隐藏并提示弃用后**，要看提示写到哪一路输出：
   - **提示必须走 stderr。** `ccr-vscode` 在运行时调用 `ccr export --platform <平台> --stdout` 并把 stdout 整段当 TOML 解析（`ccr-vscode/src/services/catalogReader.ts:77-92`，`:57-59`）。仓库现行约定是警告写 stdout（`.trellis/spec/ccr-cli/backend/cli-output-presentation.md:43`），照这个约定加提示会让扩展读不到 profile。这条路径没有测试覆盖。
   - **测试**：按「只隐藏 + stderr 提示」实施，现有断言预计都不会失败（推断）。要补断言或同步措辞的测试见 [测试](#测试) 一节，核心是 `crates/ccr/tests/commands/help.rs`、`legacy_routing.rs`、`validate.rs`、`crates/ccr/tests/diagnostics_contract.rs`、`crates/ccr-cli/tests/dispatch_routing.rs`，以及 `crates/ccr-cli/src/cli/definitions.rs:593-608` 的解析测试。
   - **文档**：26 个命令页（13 个命令 × 中英）、侧边栏 `docs/.vitepress/config.mjs`、文档审计脚本 `docs/scripts/audit-docs.mjs:209-268`、快速开始与指南、两份 README、`examples/README.md`。清单见 [文档](#文档) 一节。审计脚本按 `Commands` 枚举的每个变体要求一张命令页，不看 `hide`，所以隐藏后命令页不能删，只能改写成弃用说明。
   - **还有两个运行时调用方要一起决定**：`ccr-vscode` 的 `export` / `import`，以及桌面端命令目录（`crates/ccr-app/src/jobs/policy.rs`）里可执行的 `list`、`validate`、`export`、`switch`。隐藏不会让它们失效，但每次执行都会带上弃用提示。

## 命令现状

| 命令 | 定义 | 分发 | 今天的行为 |
|---|---|---|---|
| `ccr platform profile create/set-field/enable/disable/delete` | `crates/ccr-cli/src/cli/subcommands/platform.rs:67-167`（`hide = true`） | `crates/ccr-cli/src/cli/dispatch.rs:462-464` | 返回 `legacy_platform_command_error("profile")`（`crates/ccr-cli/src/commands/migration.rs:15-19`） |
| `ccr platform switch/current/info` | `platform.rs:21-56`（`hide = true`） | `dispatch.rs:450-458` | 同上，迁移错误 |
| `ccr platform init` | `platform.rs:58-65`（`hide = true`） | `dispatch.rs:459-461` | `legacy_platform_init_error`（`migration.rs:21-26`） |
| `ccr switch <名称>` | `crates/ccr-cli/src/cli/definitions.rs:118-121` | `dispatch.rs:139-141` | `legacy_switch_error`（`crates/ccr-cli/src/commands/profile/switch.rs:23-26`） |
| 裸 `ccr <名称>` | `definitions.rs:58-61`（位置参数 `config_name`） | `dispatch.rs:288-293` | `legacy_shortcut_error`（`migration.rs:9-13`） |
| `ccr list/add/delete/enable/disable/init/export/import/optimize/validate/clear` | `definitions.rs:104-299`，都没有 `hide` | `dispatch.rs:125-235` | 仍是真实实现 |

`ccr platform list` 不在调查范围内，它是保留的规范入口。

## 运行时调用方

| 路径 | 调用的命令 | 说明 |
|---|---|---|
| `ccr-vscode/src/services/catalogReader.ts:77-92` | `ccr export --platform <平台> --stdout` | `execFile("ccr", …)`，stdout 直接 `TOML.parse`，stderr 被丢弃。两处使用：SQLite 读失败时的回退（`:57-59`），以及「打开 profiles」命令（`ccr-vscode/src/extension.ts:349`） |
| `ccr-vscode/src/extension.ts:87-93` | `ccr import <临时文件> --platform <平台> --force` | 用户保存上一步导出的临时 TOML 时触发。失败时把 stderr 或 stdout 弹给用户 |
| `crates/ccr-app/src/jobs/policy.rs:351` | `ccr list`（别名 `ls`） | 桌面命令目录，`executable: true`，在白名单内（`:4-19`） |
| `crates/ccr-app/src/jobs/policy.rs:367` | `ccr validate` | 同上，可执行 |
| `crates/ccr-app/src/jobs/policy.rs:508` | `ccr export [-o …] [--no-secrets]` | 同上，可执行 |
| `crates/ccr-app/src/jobs/policy.rs:538` | `ccr switch <config>` | 同上，可执行。CLI 侧已退休，所以这一项今天执行必然得到迁移错误 |
| `crates/ccr-app/src/jobs/policy.rs:627` | `ccr backup` | 可执行、在白名单内，但 CLI 没有 `backup` 子命令。推断：它会落到裸 `ccr <名称>` 路径，得到 `legacy_shortcut_error`。`rename`（`:561`）、`duplicate`（`:594`）、`diff`（`:475`）同样没有对应子命令 |

命令目录本身不直接执行，执行链路是：

- 桌面（GPUI）：`ccr-desktop/src/runtime/jobs.rs:58-84` → `crates/ccr-app/src/jobs/commands.rs:458`（`command.arg(snapshot.command).args(snapshot.args)`，程序固定为 sidecar `ccr`）。
- Tauri：`ccr-ui/src-tauri/src/commands/command_exec.rs:353-361`（后台任务）、`:431-445`（前台执行）、`:468-476`（`ccr help <命令>`）。
- 前端入口：`ccr-ui/src/features/commands/useCommandsPage.ts:126-131`、`ccr-ui/src/configs/commands.ts:81-83`。目录加载失败时的兜底注册表里也有 `switch`（`ccr-ui/src/features/commands/commands-model.ts:34-43`），参数区对 `switch` 做了下拉特判（`ccr-ui/src/features/commands/CommandsComposer.tsx:57`）。

目录里另有只展示、不可执行的条目：`delete`（`policy.rs:643`）、`import`（`:671`）、`add`（`:735`）、`platform`（`:770`，示例文本含 `ccr platform current`，`:775`）。它们的 `executable` 为 `false`，请求会被 `validate_command_request` 拒绝。

没有找到运行时调用方的命令：`add`、`delete`、`enable`、`disable`、`init`、`optimize`、`clear`，以及整个 `ccr platform` 旧子命令组。

只展示命令文本、不执行的界面：

| 路径 | 文本 | 说明 |
|---|---|---|
| `ccr-ui/src/features/claude/home/homeModel.ts:4-5`、`:93-94` | `ccr switch`、`ccr list` | Claude 首页「常用命令」，点击复制到剪贴板（`HomeGrids.tsx:86`） |
| `ccr-vscode/src/providers/statusBarProvider.ts:123`、`profileTreeProvider.ts:395` | `Run 'ccr init' to get started.` | 未初始化时的提示 |

`scripts/`、五个 `justfile`、`.github/` 里没有任何一处调用这两组命令。`justfile` 与 `ccr-ui/scripts` 只跑 `ccr --version`。

## 测试

### `ccr platform` 旧子命令

| 路径 | 命令 | 断言 | 删除 `platform profile` 后 |
|---|---|---|---|
| `crates/ccr/tests/commands/legacy_routing.rs:99-107` | `platform profile create gemini team` | 失败，stderr 含 `legacy`、`current_platform` 和新命令 | 要删或改成断言解析错误 |
| `crates/ccr/tests/commands/platform_profile_surface.rs:114-126` | 同上 | 同上 | 同上 |
| `crates/ccr/tests/commands/help.rs:38-49` | `--help` | 根帮助不含 `ccr platform switch/current/info/init/profile` | 不受影响 |
| `crates/ccr/tests/commands/help.rs:97-110` | `platform --help` | 平台帮助不列出 `switch/current/info/init/profile` | 不受影响 |
| `crates/ccr/tests/commands/legacy_routing.rs:85-97` | `platform switch codex`、`platform current` | 迁移错误 | 不受影响 |
| `crates/ccr/tests/commands/grok_profile.rs:482-486` | `platform switch grok` | 迁移错误，含 `ccr grok profile` | 不受影响 |
| `crates/ccr/tests/commands/help.rs:113-137` | `platform init grok` | 迁移错误，点名三个 `profile init` | 不受影响 |

`crates/ccr/tests/public_api_compat.rs:9-10` 引用的是库函数 `ccr::application::switch_platform`，与 CLI 子命令无关。

### 顶层旧命令

| 路径 | 命令 | 断言 | 隐藏并提示后要做的事 |
|---|---|---|---|
| `crates/ccr/tests/commands/help.rs:27-54` | `--help` | 只断言任务导向文本，不断言旧命令出现 | 补「根帮助不再列出旧命令」的断言 |
| `crates/ccr/tests/commands/help.rs:244-251` | `init --help` | 含 `初始化配置文件`、`--force` | 命令说明若改成弃用措辞，这里要同步 |
| `crates/ccr/tests/commands/help.rs:158-182` | `init` | stdout 含一段固定的「下一步」文本 | 提示写 stdout 且插在这段中间才会失败；补弃用提示断言 |
| `crates/ccr/tests/commands/legacy_routing.rs:59-83` | `switch team`、裸 `team` | stderr 含 `legacy`、`current_platform` 和新命令 | 弃用提示若与退休错误统一措辞，这里要同步 |
| `crates/ccr-cli/tests/dispatch_routing.rs:263-292` | 裸 `ccr team`（进程内） | `ConfigError`，含 `legacy shortcut retired` | 同上 |
| `crates/ccr-cli/tests/dispatch_routing.rs:384-446` | `ccr list`（进程内） | 成功且目录字节不变；缺配置时返回 `ConfigMissing` 且不初始化 | 提示不能写文件，也不能改错误类型 |
| `crates/ccr/tests/commands/validate.rs:129-163` | `validate` | 成功，stdout 含平台名，不含 `ccr switch <config>` | 补弃用提示断言 |
| `crates/ccr/tests/diagnostics_contract.rs:71-96` 及其调用处（`:216-607`） | `validate` | 退出码、分类词、用户文件字节不变 | 提示不能改退出码，不能写文件 |
| `crates/ccr-cli/src/cli/definitions.rs:593-608` | `ccr init --force`（解析） | 解析为 `Commands::Init` | 不受影响；可顺带断言隐藏后仍能解析 |
| `crates/ccr-app/src/jobs/policy.rs:1087-1135`、`ccr-ui/src-tauri/src/commands/command_exec.rs:526-540` | 命令目录 | `delete`、`import`、`add`、`platform` 不可执行 | 只有桌面目录跟着调整时才要改 |
| `ccr-desktop/src/features/jobs/tests.rs:219-265` | 命令目录 | 存在可选中的 `job-command-list` | 同上 |
| `ccr-desktop/src/shell/command_state.rs:546-569`、`:623`，`ccr-desktop/src/runtime/ui_state.rs:301` | 收藏与历史里的 `"list"` | 只当字符串存取，不执行 | 不受影响 |

没有二进制级测试覆盖顶层 `add`、`delete`、`enable`、`disable`、`export`、`import`、`optimize`、`clear`。`ccr-vscode/src/services/ccrCli.test.ts` 只测 `<平台> profile …` 的 argv，不覆盖 `export` / `import`。`ccr-ui/tests/commands/` 用的是 `status` 和虚构的 `delete` 目录项，不依赖真实旧命令。

## 文档

### 命令页与导航

- `docs/reference/commands/` 与 `docs/en/reference/commands/` 下各 13 页：`list`、`switch`、`add`、`delete`、`enable`、`disable`、`init`、`export`、`import`、`optimize`、`validate`、`clear`、`platform`。`switch.md` 和 `platform.md` 已经写成退休说明，其余 11 页仍按现行命令描述。
- 侧边栏：`docs/.vitepress/config.mjs:59-89`（中文）、`:234-264`（英文）。
- 审计脚本：`docs/scripts/audit-docs.mjs:209-268`。`extractTopLevelCommands` 用正则读 `Commands` 枚举的变体，`checkCommandCoverage` 要求每个变体都有中英命令页，并拒绝多余的页。它不识别 `#[command(hide = true)]`。
- 命令总览：`docs/reference/commands/index.md:46,78,83`（及英文同位置）仍把 `ccr init`、`ccr validate` 列为常用命令。

### 仍把旧命令当现行用法的页面

| 路径 | 涉及的命令 |
|---|---|
| `README.md:38-43`、`README_CN.md:38-43` | `init`、`add`、`validate`（快速开始） |
| `docs/index.md:85-90`、`docs/en/index.md:85-90` | `init`、`add`、`validate` |
| `docs/guide/quick-start.md:29,42,63`（及英文） | `init`、`add`、`validate` |
| `docs/guide/cli-workflows.md:8,23,54,60`（英文 `:8,23,44`） | `init`、`add`、`validate` |
| `docs/guide/configuration.md:67-73`（及英文） | `init`、`add`、`validate` |
| `docs/examples/index.md:23,33,41`（及英文） | `validate`、`switch`、`export` |
| `docs/examples/multi-platform-setup.md`（35 处；英文 6 处） | `platform switch/current/info`、`list`、`switch`、`add`、`delete` |
| `docs/examples/troubleshooting.md`（31 处；英文 1 处） | `platform switch/current/info`、`list`、`switch`、`add`、`init`、`export`、`import` |
| `docs/reference/platforms/droid.md`（19 处，中英相同） | `platform switch/current`、`list`、`switch`、`add`、`delete`、`import` |
| `docs/reference/platforms/gemini.md`（10 处，中英相同） | `platform switch`、`list`、`switch`、`add`、`export`、`import`、`validate` |
| `docs/reference/platforms/migration.md`（26 处，中英相同） | `platform switch/current`、`import`、`export`、`list`、`add`、`init`、`validate` |
| `docs/reference/platforms/claude.md:94-96`、`codex.md:127-129` | `platform switch/current`、`switch` |
| `docs/reference/commands/temp.md`、`temp-token.md`（及英文） | `switch`、`add` |
| `docs/reference/commands/sync.md:270-305,407`、`update.md`（12 处）、`clean.md:345`、`doctor.md:108` | `list`、`add`、`export`、`validate`、`switch`、`platform switch` |
| `examples/README.md:72,85,159-161` | `list`、`enable`、`disable` |
| `examples/droid/profiles.toml:9` | `ccr switch -p droid <profile-name>`（这个写法 CLI 里不存在） |

`docs/examples/` 和 `docs/reference/platforms/` 里的 `ccr platform switch`、`ccr switch` 在今天的 CLI 上已经只会报迁移错误，这些页面在本轮之前就过期了。

### 已经按退休口径写的页面

`README.md:9,50-54`、`README_CN.md:9,50-54`、`docs/guide/cli-workflows.md:95-99`（英文 `:77-81`）、`docs/guide/quick-start.md:81-84`、`docs/guide/entrypoints.md:43`、`docs/reference/migration.md:9-13`、`docs/reference/internals/runtime-flows.md:9`、`docs/reference/commands/platform.md:17-38`、`docs/reference/commands/switch.md`，以及各自的英文页。删除 `ccr platform profile` 后，这些迁移表里的对应行可以保留，作为旧命令到新命令的对照。

### 规格

| 路径 | 内容 |
|---|---|
| `.trellis/spec/ccr-cli/backend/grok-profile-runtime.md:250` | `ccr platform switch/profile` 继续返回迁移错误 |
| `.trellis/spec/ccr-cli/backend/backend-guidelines.md:157,396,420` | 断言 `ccr init` 兼容性；`ccr platform init grok` 返回迁移错误 |
| `.trellis/spec/ccr-cli/backend/profile-init.md:52` | 退休的 `ccr platform init` 返回专门的迁移错误 |
| `.trellis/spec/ccr-config/backend/backend-guidelines.md:29` | `profiles.toml` 是 import、export 的交换格式 |
| `.trellis/spec/ccr-cli/backend/cli-output-presentation.md:43` | 警告写 stdout |

## CLI 自己的提示文本

这些不是调用方，但隐藏旧命令后它们会继续把用户引向旧命令：

| 路径 | 文本 |
|---|---|
| `crates/ccr-cli/src/cli/definitions.rs:24-28` | 根命令 `long_about` 的快速开始：`ccr init`、`ccr list`、`ccr switch <名称>`、`ccr anthropic` |
| `crates/ccr-cli/src/cli/definitions.rs:366-368`、`crates/ccr-cli/src/cli/dispatch.rs:938` | 示例与 `ccr version` 输出里的 `ccr platform switch`、`ccr platform current`（已退休） |
| `crates/ccr-core/src/core/error.rs:281-282,302,331,341-344` | 错误建议：`ccr list`、`ccr add`、`ccr validate`、`ccr platform switch`（最后一个已退休） |
| `crates/ccr-cli/src/services/doctor_service.rs:274,297,307,318,338,372` | doctor 建议 `ccr init` |
| `crates/ccr-sync/src/sync/service.rs:1072` | `请先运行 'ccr init'` |
| `crates/ccr-tui/src/tui/runtime.rs:116-128` | 非交互终端时建议 `ccr list` / `ccr current` |
| `crates/ccr-cli/src/commands/profile/{add,delete,enable,disable}.rs`、`lifecycle/clear.rs:147`、`data/import.rs:230`、`provider_cmd.rs:127`、`crates/ccr-cli/src/sync/commands.rs:676` | 「下一步」里的 `ccr list` |
| `crates/ccr-cli/src/commands/temp_cmd.rs:39` | `下次执行 'ccr switch' 时…`（已退休） |

`crates/ccr-tui/src/tui/ui.rs:2385-2410` 的测试已经断言空状态不出现 `ccr platform init` 和 `ccr add`。

## 方法与未核实项

- 搜索范围：`git ls-files` 列出的全部受版本控制文件，排除 `ref/`、`third_party/`、`ccr-desktop/vendor/`、锁文件、`.trellis/workspace/` 和 `.trellis/research/archive/`。隐藏目录（`.github/`、`.trellis/`）单独带 `--hidden` 复查过。
- 搜索方式：整串命令（`ccr <命令>`、`ccr platform <子命令>`）、argv 数组片段（`["platform", "profile", …]`、首元素为旧命令的数组、跨行数组）、`.arg("…")` 链、sidecar 与 `execFile` / `Command::new` 调用点，以及对旧命令实现函数的进程内引用。
- 没有构建或运行 `ccr`。「隐藏后现有断言不会失败」「`ccr backup` 落到裸快捷路径」「删除 `platform profile` 后得到 clap 解析错误」都是从 clap 定义和断言文本推断的，没有实测。
- 只看了本仓库。已发布的 `ccr-vscode` 旧版本、用户脚本和仓库外的调用不在范围内。
