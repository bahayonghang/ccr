# Research: CcrError 上帝枚举拆解——数据盘点

- **Query**: 盘点 `crates/ccr-core/src/core/error.rs` CcrError 的 variant 分类、构造点分布、匹配点、转换面、Result 渗透、公开 API 面、文案面、既有先例、依赖图
- **Scope**: internal
- **Date**: 2026-07-05
- **环境说明**: 本机 `rg` = ripgrep 15.1.0。下列命令均可复现(Windows 下路径分隔符显示为 `\`)。

## 1. Variant 全清单与分类

来源:`crates/ccr-core/src/core/error.rs:105-205`。**实际 25 个 variant,不是 PRD 说的约 26**。

| # | Variant | 定义行 | 分类 | 说明 |
|---|---|---|---|---|
| 1 | ConfigError(String) | error.rs:108 | 领域(实际是万能兜底) | 全仓 344 次引用,最大 variant,语义已漂移为通用错误 |
| 2 | ConfigMissing(String) | error.rs:112 | 领域(config) | is_fatal 之一 |
| 3 | ConfigSectionNotFound(String) | error.rs:116 | 领域(config/profile) | |
| 4 | ConfigFormatInvalid(String) | error.rs:120 | 领域(config) | |
| 5 | SettingsError(String) | error.rs:124 | 领域(settings) | |
| 6 | SettingsMissing(String) | error.rs:128 | 领域(settings) | is_fatal 之一 |
| 7 | FileLockError(String) | error.rs:132 | **原语**(锁) | |
| 8 | LockTimeout(String) | error.rs:136 | **原语**(锁) | |
| 9 | JsonError(#[from] serde_json::Error) | error.rs:140 | **原语**(格式) | |
| 10 | TomlError(#[from] toml::de::Error) | error.rs:144 | **原语**(格式) | |
| 11 | IoError(#[from] std::io::Error) | error.rs:148 | **原语**(IO) | is_fatal 之一 |
| 12 | FileIoError(String) | error.rs:152 | **原语**(IO) | 与 IoError 功能重叠(字符串版) |
| 13 | HistoryError(String) | error.rs:156 | 领域(history) | |
| 14 | ValidationError(String) | error.rs:160 | 原语级通用 | ccr-core 自己的 utils/validation.rs 也构造 |
| 15 | SyncError(String) | error.rs:164 | 领域(sync) | |
| 16 | PlatformNotFound(String) | error.rs:168 | 领域(platform) | |
| 17 | PlatformNotSupported(String) | error.rs:172 | 领域(platform) | |
| 18 | ProfileNotFound(String) | error.rs:176 | 领域(profile) | |
| 19 | NetworkError(String) | error.rs:180 | 原语级(传输) | 仅 ccr-cli/ccr-skills 构造 |
| 20 | ResourceNotFound(String) | error.rs:184 | 通用 | |
| 21 | ResourceAlreadyExists(String) | error.rs:188 | 通用 | |
| 22 | DatabaseError(String) | error.rs:192 | 领域(db) | |
| 23 | UpdateError(String) | error.rs:196 | 领域(update) | |
| 24 | UiError(String) | error.rs:200 | 领域(ui) | |
| 25 | ExternalCommandError(String) | error.rs:204 | 原语级(进程) | |

分类小计:**原语/原语级/通用 11 个**(7/8/9/10/11/12/14/19/20/21/25),**领域词汇 14 个**(1-6/13/15-18/22-24)。

附属结构:`exit_codes` 常量模块(error.rs:24-99,25 个退出码)、`exit_code()`(error.rs:213-241,25 臂穷尽 match)、`is_fatal()`(error.rs:251-256)、`user_message()`(error.rs:261-363,11 个 variant 有定制建议文案)、`pub type Result<T>`(error.rs:367)。

复现:`rg -o "^\s+\w+\(" crates/ccr-core/src/core/error.rs`(在 enum 区间内数)或直接读文件。

## 2. 构造点分布

命令:`rg "CcrError::" --type rust -c`(按文件)与 `rg -o "CcrError::\w+" --type rust --no-filename | sort | uniq -c | sort -rn`(按 variant)。

**总量:1082 处 `CcrError::` 引用,分布在 104 个文件**。其中 error.rs 自身 46 处(定义处 match 臂+单测),纯文档注释 3 处(`crates/ccr-types/src/claude_settings.rs:284`、`crates/ccr-cli/src/commands/profile/{enable.rs:22,disable.rs:24}` 的幽灵变体 `ConfigNotFound`——**该 variant 并不存在**,是文档腐化)。

### 按 crate 汇总(全部引用,含测试)

| Crate | 引用数 | 文件数 | 备注 |
|---|---|---|---|
| ccr-cli | 413 | 45 | 最大构造方 |
| ccr-codex | 293 | 15 | 第二大 |
| ccr-skills | 97 | 3 | |
| ccr-core | 84 | 6 | 其中 error.rs 46;其余 38 处**全部是原语 variant** |
| ccr-store | 69 | 7 | |
| ccr-config | 55 | 9 | |
| ccr-sync | 31 | 5 | |
| ccr-tui | 27 | 2 | |
| ccr-ui/src-tauri | 9 | 1 | 仅 commands/config.rs |
| ccr(根) | 1 | 1 | tests/platforms/general.rs |
| ccr-types | 1 | 1 | 纯文档注释 |
| **ccr-db / ccr-usage / ccr-checkin** | **0** | 0 | `rg -c "CcrError" crates/ccr-db/src crates/ccr-usage/src crates/ccr-checkin/src` 均 0 命中 |

### 关键事实:ccr-core 自身(排除 error.rs)只构造原语

`rg -o "CcrError::\w+" crates/ccr-core/src --glob '!**/error.rs' -n` → 38 处,全部为:FileIoError(19,fileio.rs/guarded_write.rs)、IoError(11,atomic_writer.rs)、FileLockError(3,lock.rs)、LockTimeout(3,lock.rs/guarded_write.rs 测试)、ValidationError(2,utils/validation.rs)。**领域词汇 variant 在 ccr-core 内只出现在 error.rs 定义处本身**——"底层 crate 认识全应用概念"仅指定义,不含构造。

### 领域 variant 的跨 crate 构造分布(排除 error.rs,核心表)

命令:`rg -o "CcrError::(SyncError|DatabaseError|...)" --type rust -g '!crates/ccr-core/src/core/error.rs' | sort | uniq -c`

| 领域 variant | 构造分布(次数) | "归属 crate"集中度 |
|---|---|---|
| HistoryError | ccr-store 11(history.rs) | **100% 集中**,唯一干净案例 |
| SyncError | **ccr-cli 21**(sync/commands.rs) + ccr-sync 10(service.rs) | 归属 crate 只占 32% |
| DatabaseError | **ccr-codex 32**(codex_history_sync_service.rs 29 + opencode_usage_service.rs 3) + ccr-store 27(session_store 16 + database 11) | 最大构造方是 codex,不是 store |
| UiError | **ccr-cli 38**(ui_service.rs 37 + ui.rs 1) + ccr-tui 3(runtime.rs) | 主构造方 ccr-cli 无法依赖 ccr-tui(依赖方向相反) |
| SettingsError | **ccr-cli 51**(managers/settings.rs 35、claude_auth_service 7、gemini 5、droid 4)+ ccr-codex 14(codex_config.rs) | Settings 管理器在 ccr-cli,codex 又有自己的 |
| SettingsMissing | ccr-cli 10 | 集中在 cli |
| PlatformNotFound | ccr-cli 8 + ccr-config 4 + ccr-codex 1 + ccr tests 1 | 散 4 处 |
| PlatformNotSupported | ccr-cli 11(qwen.rs 6 等) | 集中在 cli |
| ProfileNotFound | ccr-cli 7 + ccr-codex 4 | 散 2 crate |
| ConfigSectionNotFound | ccr-config 5 + ccr-tui 4 + ccr-cli 3 | 散 3 crate |
| ConfigError | 全部 8 个业务 crate + src-tauri 都构造(cli 89 / codex 158 / config 29 / skills 24 / store 18 / sync 14 / tui 7 / core 3) | 万能兜底,无归属可言 |
| UpdateError | ccr-cli 1 + ccr-codex 1 | 极小 |

**结论性事实**:除 HistoryError 外,没有一个领域 variant 的构造点集中在 PRD 设想的"归属 crate";最大构造方普遍是消费侧 crate(ccr-cli / ccr-codex)。

## 3. 匹配/检查点(迁移破坏面)

命令:`rg "match .*CcrError|if let CcrError|matches!\(.*CcrError" --type rust -g '!crates/ccr-core/src/core/error.rs'`

全仓对 CcrError variant 做模式匹配的位置共 **7 处,其中生产代码仅 1 处**:

| 位置 | 性质 | 匹配的 variant |
|---|---|---|
| `crates/ccr-codex/src/services/codex_history_sync_service.rs:2878-2880`(`is_locked_error`) | **生产代码(唯一)** | FileLockError(原语,任何方案下都留在 core) |
| `crates/ccr-core/src/core/guarded_write.rs:497` | 测试 | LockTimeout |
| `crates/ccr-tui/src/tui/runtime.rs:407` | 测试 | UiError |
| `crates/ccr-skills/src/services/skills_service.rs:2948,2966` | 测试×2 | ValidationError |
| `crates/ccr-cli/src/commands/platform/profile.rs:400` | 测试 | PlatformNotSupported |
| `crates/ccr/tests/platforms/general.rs:513` | 集成测试 | PlatformNotFound(带 guard 匹配) |

行为消费面:`exit_code()/is_fatal()/user_message()` 的调用方全仓仅 1 处——`crates/ccr/src/cli/dispatch.rs:738-747`(`handle_error`,终端渲染 + `std::process::exit`)。复现:`rg "\.exit_code\(\)|\.is_fatal\(\)|\.user_message\(\)" --type rust -g '!**/error.rs'`。

**解读**:1082 处引用中,分支消费只有 7+1 处。CcrError 的 variant 在实践中是「(退出码, 文案前缀) 标签」,不是类型化分支信号。

## 4. From / 转换面

- error.rs 内 `#[from]` 共 3 个:serde_json::Error(:140)、toml::de::Error(:144)、std::io::Error(:148)。
- **全仓 0 处手写 `impl From<...> for CcrError`,0 处 `impl From<CcrError> for ...`**(`rg "impl From<.*> for CcrError|From<CcrError>" --type rust` 无命中)。
- 显式 `CcrError::from` 调用仅 1 处:`crates/ccr-cli/src/commands/lifecycle/init.rs:80`(io::Error)。
- ccr-ui/src-tauri 不包装 CcrError 为 DTO:命令层用 `Result<T, String>`,经 `.map_err(|e| e.to_string())` / `format!` 扁平化(`ccr-ui/src-tauri/src/commands/config.rs:140-215`);仅 config.rs 内部辅助函数直接返回 `Result<_, CcrError>` 并构造 9 处(:76-128)。

## 5. Result 别名渗透

- 别名定义:`crates/ccr-core/src/core/error.rs:367`。
- re-export 链:ccr-core 根(`crates/ccr-core/src/lib.rs:8-9`)、ccr-config 根(`crates/ccr-config/src/lib.rs:8`)、ccr-codex 根(`crates/ccr-codex/src/lib.rs:10`)、ccr 根 legacy(`crates/ccr/src/lib.rs:179`)与 `ccr::prelude`(:159-161)。
- 实际 import 全部走 `ccr_core::` 路径:`rg -l "use ccr_core::.*\b(CcrError|Result)\b|use ccr_core::\{[^}]*\b(CcrError|Result)\b" --type rust` → **142 个文件**(ccr-cli 81 / ccr-codex 18 / ccr-config 11 / ccr-tui 11 / ccr-store 9 / ccr-sync 5 / ccr-skills 3 / ccr-core 2 / ccr 1 / src-tauri 1)。经 `ccr_config::CcrError`、`ccr_codex::CcrError` 转发路径 import 的文件数为 **0**(这两个 re-export 是装饰性的)。
- 签名量级(churn 上界代理):`rg -o "\-> Result<" crates/<c>/src -c` 合计 **约 1142 个 `-> Result<` 签名**:ccr-cli 417 / ccr-codex 258 / ccr-store 122 / ccr-config 92 / ccr-skills 76 / ccr-tui 71 / ccr-core 47 / ccr-sync 41 / ccr 18(少数为 std Result,绝大多数是 CcrError 别名)。

## 6. 公开 API 面

- 快照测试 `crates/ccr/tests/public_api_compat.rs`:
  - `stable_prelude_paths_remain_available`(:24-34):`ccr::prelude::{CcrError, Result, ...}` 必须可用,`size_of::<CcrError>() > 0`。
  - `crate_root_public_reexport_snapshot_is_intentional`(:52-74):根路径快照第 71-74 行冻结 `pub use ccr_core::{... CcrError ... Result ...}`。
- `ccr::prelude` 确认导出 CcrError/Result:`crates/ccr/src/lib.rs:159-161`。
- spec 冻结措辞原文,`.trellis/spec/ccr/backend/public-api-boundary.md`:
  - §2 稳定 prelude 签名块第一组即 `CcrError, Result`(:17)。
  - §3:"Existing root exports ... remain available until an explicit breaking release plan exists"(:33);"In the 6.x line, do not add `#[deprecated]` ... would break `-D warnings` downstream builds"(:34)。
- 当前版本 6.4.3(根 Cargo.toml:21),即处于被冻结的 6.x 线内。
- **其他 spec 文件主动规定"映射到 CcrError 领域 variant"**(落法 A/B 都要改这些行):
  - `.trellis/spec/ccr-sync/backend/backend-guidelines.md:27` "map WebDAV/network/path failures to `CcrError::SyncError`"
  - `.trellis/spec/ccr-store/backend/backend-guidelines.md:31` "Map `rusqlite` failures to `CcrError::DatabaseError`"
  - `.trellis/spec/ccr-config/backend/backend-guidelines.md:41`、`ccr-codex/backend/backend-guidelines.md:34`、`ccr-cli/backend/backend-guidelines.md:37`、`ccr/backend/backend-guidelines.md:27`、`ccr-core/backend/backend-guidelines.md:7,63`、`ccr-types/backend/backend-guidelines.md:49`、`ccr-core/backend/atomic-writer.md:113`

## 7. 用户可见文案面

- 全部 Display 文案为中文,定义于 error.rs `#[error("...")]`(:107-204);`user_message()` 为 11 个 variant 提供多行建议文案(:261-363),含平台列表、命令建议等长文本。
- 终端唯一渲染点:`crates/ccr/src/cli/dispatch.rs:734-748`。
- 测试对文案的耦合**极薄**:error.rs 自身 4 个单测(:374-403);`crates/ccr-sync/src/sync/folder_manager.rs:616` 断言 Display 含"已存在";`crates/ccr/tests/platforms/general.rs:513` 匹配 variant 与载荷(非文案)。crates/ccr/tests 下其余 stderr 断言("legacy"、"current_platform" 等)针对命令引导文案,与 CcrError Display 无关。复现:`rg 'assert.*"(配置|设置|同步|数据库|平台|历史|验证|更新|文件锁|网络)' crates --type rust`(1 命中且无关)。
- 退出码契约:仅 error.rs 注释声明"便于脚本判断";集成测试、ccr-vscode、justfile 均未断言具体退出码值(docs 仅 changelog/doctor.md 提及,doctor 有自己的码)。
- Tauri 侧:CcrError 经 `to_string()`(即 Display)扁平化为 String 传给前端,无 variant 级映射(见 §4)。

## 8. 既有先例(领域自有错误类型)

工作区拆分(2026-03-31, b3d22abe)之后新建的 crate **全部走自有错误路线,零 CcrError 依赖**:

| Crate | 错误类型 | 位置 | 与 CcrError 关系 |
|---|---|---|---|
| ccr-db | DbError / MigrationError / ExecutorError | `crates/ccr-db/src/core/error.rs:9,61,95` | 无(0 引用);依赖 ccr-core 但只用其他设施 |
| ccr-usage | UsageError(结构化载荷,含 SchemaUnsupported{expected,actual}) | `crates/ccr-usage/src/error.rs:6` | 无;甚至不依赖 ccr-core |
| ccr-checkin | CheckinServiceError 等分层错误,`Database(#[from] DbError)` | `crates/ccr-checkin/src/core/error.rs:30`、`managers/checkin/*_manager.rs` | 无(0 引用) |
| src-tauri llmusage_adapter | LlmusageAdapterError,手写 UsageError→AdapterError 逐 variant 映射 | `ccr-ui/src-tauri/src/llmusage_adapter/error.rs:6`、`db.rs:98-108` | 无 |

消费端示范:ccr-tui 直接对 UsageError variant 做类型化分支(`crates/ccr-tui/src/tui/usage/app.rs:122-136`)——这正是 CcrError 从未被这样用过的能力。

## 9. 依赖图事实核查

各 crate Cargo.toml 的内部依赖(`rg "^ccr-[a-z]*\s*=" crates/*/Cargo.toml`):

- **ccr-core 无任何内部依赖(图底)——"ccr-core 不依赖 ccr-types"属实**(`crates/ccr-core/Cargo.toml` 无 ccr-* 条目);ccr-types、ccr-usage 也是零内部依赖的叶子。
- 依赖 ccr-core 的 crate(10 个)+ src-tauri:ccr、ccr-checkin、ccr-cli、ccr-codex、ccr-config、ccr-db、ccr-skills、ccr-store、ccr-sync、ccr-tui、ccr-ui/src-tauri(Cargo.toml:17)。
- 与落法 A 冲突的关键边:**ccr-tui → ccr-cli**(ccr-tui/Cargo.toml:16,方向决定 UiError 无法"上移到 ccr-tui"再被 ccr-cli 构造);**ccr-codex 不依赖 ccr-store/ccr-db/ccr-cli**(DatabaseError、SettingsError 的最大构造方无法引用设想的归属 crate)。

## 10. 变更频率(locality 问题的实际发生率)

`git log --oneline --follow -- crates/ccr-core/src/core/error.rs` → 22 个提交(2025-10-11 起)。

- 最后一次新增 variant:**ba1b9c39 2026-02-04**(一次加 4 个:FileIoError/UpdateError/UiError/ExternalCommandError)。
- error.rs 迁入独立 ccr-core crate:b3d22abe 2026-03-31。
- **拆分后 3 个月内 error.rs 仅 3 次改动且全是文案/更名**(c36cf4d9、c3c271c2 平台更名,a283bf95 2026-05-20 仅 2 行 Gemini→Antigravity 文案)。
- 即:「新增领域错误要改图底 crate」的 locality 倒置自 ccr-core 存在以来**发生次数为 0**。

## Caveats / Not Found

- 引用计数含测试代码与少量文档注释(已单列 3 处纯文档);"构造点"未逐一区分 `Err(CcrError::X)` 与 `matches!` 断言,但 §3 已证明匹配点仅 7 处,误差可忽略。
- `-> Result<` 签名计数含少量 `std::result::Result` 全称写法之外的本地 Result(如 ccr-tui usage 模块用 UsageError),作为量级代理使用。
- 未发现任何外部(仓外)库消费者的证据;公开 API 冻结的实际受益者目前只有 ccr-ui/src-tauri 与假想的下游。
