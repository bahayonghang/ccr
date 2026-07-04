# Design: Secret 掩码 newtype

## 0. 探索结论修正（相对 prd.md 的事实校准）

实现前全量核实了 PRD 定位的算法与字段，以下事实需要修正或补充，不影响任务成立：

1. **实际是 4 处掩码实现，不是 3 处**：PRD 之外还有 `ccr-ui/src-tauri/src/commands/config.rs:59 mask_token`，其算法与 `ccr_core::mask_sensitive` 逐字相同（纯重复代码），一并删除、零行为变化。
2. **`mask_api_key` 已是死代码**：`#[allow(dead_code)]`，全仓仅自身测试引用，删除零迁移成本。
3. **`mask_sensitive`/`mask_if_sensitive` 是冻结公共 API**：`crates/ccr/src/lib.rs:180` 根 facade re-export 被 `public_api_compat.rs:73` 逐字断言，受 `public-api-boundary.md` 保护至 breaking release。因此 PRD"保留为 Secret 内部实现或直接吸收"只能取**保留**分支：`utils::mask::mask_sensitive` 就是全仓唯一掩码算法，`Secret` 的 Debug/Display/默认 Serialize 全部委托它。`Secret` 本身**不加入** `ccr` 根 facade（冻结不加新条目），消费方从 `ccr_core` 导入。
4. **claude/codex profile 详情 IPC 今天故意下发明文 token**（`ccr-ui claude.rs:456` / `codex.rs:1629` `profile_to_json`，编辑表单预填用）。若被 Secret 默认掩码序列化悄悄改掉 → 用户保存表单时把掩码串写回磁盘 → **配置损坏**（比日志泄漏更严重）。这两处必须显式 `expose()` 保持现行为并加注释；换 `has_password` 模式留给 07-03-arch-typed-ipc。
5. **sync 侧 UI 无回环风险**：IPC 响应已是"永不下发密码"（`SyncStatusInfo.has_password: bool`、`WebDavConfigDetails` 无密码字段），密码仅在 `WebDavConfigInput`（入参）出现。
6. **`sync_folders.toml` 含 WebDAV 密码但未享受 0o600**：guarded-write 任务只给 `sync.toml` 设了 `secret: true`。本任务补上（正是 PRD Notes 预言的"0o600 判定复用类型信息"）。
7. **新发现同类债务**：`ccr-cli/src/managers/temp_override.rs:202` 持久化 auth_token 且裸 `fs::write`（guarded-write §8 已记 ccr-cli 范围外）。本任务把其**字段** Secret 化（掩码面），**写路径**不改（一次一个 concern，记入 §7）。

## 1. 范围与验收口径

- **"掩码算法只剩 1 处"口径**：掩码*变换算法*唯一实现 = `ccr_core::utils::mask::mask_sensitive`。`Secret` 的 Display/Debug/默认 Serialize、`mask_if_sensitive`（变量名判定后委托）都是它的调用方。ccr-checkin 的 cookies 掩码*显示*是"JSON map 迭代 + 每值走 Secret Display"的格式化，不含算法，允许存在。
- **Secret 化字段清单（AC#3 口径）**：
  - `ccr-config`：`ConfigSection.auth_token`、`ProfileConfig.auth_token`（均 `Option<Secret>`）
  - `ccr-sync`：`SyncConfig.password`、`WebDavConfig.password`（均 `Secret`）
  - `ccr-cli`：`TempOverride.auth_token`（`Option<Secret>`，持久化凭据结构）
  - `ccr-db/ccr-checkin`：`CreateAccountRequest.cookies_json`、`UpdateAccountRequest.cookies_json`（`Secret`，防请求 Debug 泄漏）；`CryptoManager::decrypt` 返回 `Secret`（解密即包裹）
- **显式不迁移**（理由记录，避免复审重提）：CLI clap 参数结构（用户输入边界，进模型时转 Secret）；`ClaudeSettings` env map（07-03-arch-claude-settings 域）；`cookies_json_encrypted`（密文非明文）；`ccr-ui ConfigInfo.auth_token`（响应 DTO，本来就只存掩码串）；`ConfigService::ConfigInfo`（展示 DTO，迁 `Option<Secret>` 顺带）。
- **磁盘格式零变化**：TOML/JSON 落盘仍是明文原文（expose 注解），已存在配置无损读入（Deserialize 透明接受明文）。
- **可见的掩码格式变化（有意）**：checkin `cookies_masked` 从 `ab****yz` 变为 `mask_sensitive` 格式（前端只整串显示、无解析依赖；`account.rs` 测试是唯一断言处，随迁更新）。ccr-ui 配置列表掩码格式不变（mask_token 与 mask_sensitive 同算法）。

## 2. Secret 类型契约

```rust
// crates/ccr-core/src/core/secret.rs（公共 API 文档英文，实现注释中文）
/// String newtype whose Debug/Display/default-Serialize are always masked.
/// `expose()` is the only way to read the plaintext.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self;
    pub fn expose(&self) -> &str;      // 唯一取原文出口
    pub fn is_empty(&self) -> bool;    // 迁移高频：完整性检查
}
impl From<String> for Secret; impl From<&str> for Secret;
impl PartialEq<str> / PartialEq<&str> for Secret;   // 测试断言便利（承认 eq-oracle 残余风险）
impl fmt::Debug / fmt::Display  → mask_sensitive(&self.0)
impl Serialize                  → serialize_str(&mask_sensitive(&self.0))   // 默认掩码
impl<'de> Deserialize           → String → Secret                            // 透明读明文

/// Explicit plaintext serializers for persistence paths (greppable opt-in).
pub fn expose_plaintext<S>(secret: &Secret, s: S) -> Result<S::Ok, S::Error>;
pub fn expose_plaintext_option<S>(secret: &Option<Secret>, s: S) -> Result<S::Ok, S::Error>;
```

- **不提供** `Deref`/`AsRef<str>`/`Into<String>`/`as_str`——"expose() 之外无法取到原文"的类型层面验证口径 = 公共 API 面审查（无其他读通道）+ Debug/Display/serde 默认路径掩码有单测。
- **不做泛型 `Secret<T>`**：掩码需要 &str 视图，T 只能是 string-like；当前无非 String 凭据消费方，泛型是无消费者的 API 面（简单优先，PRD 的 `<T = String>` 视为速写）。
- **不做 zeroize-on-drop**：token 广泛存在于 env 构造、HTTP 头、序列化缓冲的 String 拷贝中，仅 newtype zeroize 是安全剧场；真 zeroize 需全链路审计，记 §7。
- 模块落位 `core/secret.rs`，`lib.rs` 根 re-export `Secret` 与两个 expose fn；`utils/mask.rs` 原样保留。

## 3. serde 策略：默认掩码 + 显式 expose 注解（决策）

持久化字段写法（Deserialize 走派生、透明收明文）：

```rust
#[serde(default, skip_serializing_if = "Option::is_none",
        serialize_with = "ccr_core::expose_plaintext_option")]
pub auth_token: Option<Secret>,          // ConfigSection / ProfileConfig / TempOverride

#[serde(serialize_with = "ccr_core::expose_plaintext")]
pub password: Secret,                    // SyncConfig / WebDavConfig
```

- **否决 A（默认明文序列化，仅 Debug/Display 掩码）**：新增的任何含 Secret 的响应/日志 JSON 默认泄明文，与现状同病；违反 PRD"serde 默认掩码"。
- **否决 C（不实现 Serialize，编译期强制选择，secrecy 式）**：所有含 Secret 的派生结构（包括大量只需要掩码输出的响应结构）都被迫加注解，注解噪音换不来额外安全；PRD 明确指定默认掩码。
- **防呆对比**：新代码漏 expose 注解 → 磁盘写入掩码串 → **round-trip 测试与用户首次保存立即暴露**（显性失败）；现状漏调 mask → 明文进日志/导出 → 无任何拦截（静默泄漏）。失败模式从静默泄漏变为显性损坏，且已知持久化结构在本任务内全部注解完毕。
- `expose_plaintext` 命名可 grep：`rg 'expose_plaintext'` 即全部明文落盘点位清单（写进 spec 作为审查入口）。

## 4. sync 密码加密存储：否（决策）

- 文件权限已收敛：`sync.toml` 0o600（guarded-write 已做）+ `sync_folders.toml` 0o600（本任务 B1 补）。
- AES-256-GCM 的密钥同机同权限落盘（`~/.ccr/checkin/crypto.key`，0o600），对"攻击者可读本机文件"威胁模型零增益，仅防肩窥/误 cat——Secret 掩码已覆盖后者。
- 结构成本：需 ccr-sync 依赖加密模块或把 `CryptoManager` 迁入 ccr-core，且加密后 sync.toml 不可手工编辑。真升级是 OS keychain，超本任务范围。
- PRD 底线"不再明文出现在日志/Display/序列化默认路径"由 Secret 类型达成。决策连同 keychain 候选记入 §7 与 spec。

## 5. 迁移矩阵

### B1 — ccr-sync（+ ccr-cli sync 命令 + ccr-ui sync 命令）

| 位置 | 现状 | 迁移后 |
|---|---|---|
| `sync/config.rs` `SyncConfig.password` | `String` 明文，Debug 泄漏 | `Secret` + expose 注解落盘 |
| `sync/folder.rs` `WebDavConfig.password` | 同上 | 同上 |
| `sync/folder_manager.rs` `save_config` | `write_toml`（0o644） | `write_toml_opts(secret: true)`（文件含密码） |
| `sync/service.rs:45` reqwest_dav 认证 | `config.password.clone()` | `config.password.expose().to_string()` |
| `ccr-cli sync/commands.rs` 构造/克隆 ×6 | String | `Secret::new(read_password()?)` / clone 不变 |
| `ccr-ui sync.rs` 完整性检查/克隆/has_password ×13 | `!p.trim().is_empty()` | `!p.expose().trim().is_empty()`（clone/字段搬运不变） |

### B2 — ccr-config（+ ccr-cli/ccr-tui/ccr-codex/ccr-ui 消费方）

| 位置 | 现状 | 迁移后 |
|---|---|---|
| `managers/config/types.rs` `ConfigSection.auth_token` | `Option<String>` | `Option<Secret>` + expose 注解 |
| `models/platform.rs` `ProfileConfig.auth_token` | 同上 | 同上 |
| `config_service.rs export_config(include_secrets=false)` | 手工替换字段为掩码串 | `t = Secret::new(t.to_string())`（Display 即掩码，expose 序列化输出掩码串，行为不变） |
| `config_service.rs ConfigInfo.auth_token`（展示 DTO） | `Option<String>` | `Option<Secret>`（CLI 表格直接 Display） |
| `ccr-cli managers/temp_override.rs` | `Option<String>` 落盘 | `Option<Secret>` + expose 注解（写路径不改） |
| CLI 掩码显示位（table/switch/current/add/temp_*） | `ColorOutput::mask_sensitive(token)` | `format!("{}", secret)`（Display）或维持 mask_sensitive(expose())——以最小 diff 为准 |
| env / settings.json / auth.json 构造位（platforms/{claude,gemini,droid}.rs、codex 服务、base.rs 等） | 直接用 String | `expose()`（合法明文消费点） |
| ccr-tui `ui.rs` ×2 完整性检查 | `!token.trim().is_empty()` | `!token.expose().trim().is_empty()` |
| ccr-ui `config.rs` `mask_token` + 列表掩码 | 第 4 套重复实现 | 删除，改 Display（格式不变） |
| ccr-ui `config.rs:229/525` UI 写入路径 | String | `Secret::new(...)` |
| ccr-ui `claude.rs:456` / `codex.rs:1629` profile 详情 | 明文 JSON 下发 | `profile.auth_token.as_ref().map(Secret::expose)` + 注释（§0.4，行为保持） |
| `crates/ccr/tests/` 集成测试 ×16 | 字符串断言 | `PartialEq<&str>` 断言 / `Secret::from` 构造 |

### B3 — ccr-checkin + ccr-db

| 位置 | 现状 | 迁移后 |
|---|---|---|
| `ccr-checkin core/crypto.rs mask_api_key` | 死代码（算法 2/4） | 删除（含测试） |
| `ccr-db models/checkin/account.rs mask_cookies_json`/`mask_value` | 算法 3/4，re-export | 删除（含 mod.rs re-export 与测试） |
| `ccr-checkin account_manager.get_info` | `mask_cookies_json(&plaintext)` | 私有 `masked_cookies_display(&Secret)`：JSON map 迭代 + 每值 `Secret` Display；解析失败/列表路径占位 `"****"` 不变 |
| `CryptoManager::decrypt` | 返回 `String` | 返回 `Secret`（解密即包裹）；`encrypt` 收 `&str` 不变（调用方 `expose()`） |
| `CreateAccountRequest.cookies_json` / `UpdateAccountRequest.cookies_json` | `String`（Debug 泄明文） | `Secret`（IPC 明文入参 Deserialize 透明；加密走 `expose()`） |
| `account_manager.get_cookies_json` | 返回 `(String, String)` | 返回 `(Secret, String)`，HTTP 头构造点 `expose()` |

## 6. 测试设计

1. **B0 单元测试**（`core/secret.rs`）：Display/Debug 掩码断言（长/短/空串）；默认 Serialize 掩码（serde_json + toml 双格式）；expose 注解字段序列化 = 原文；Deserialize 明文 → `expose()` 取回原值；`Option<Secret>` 注解 + skip_serializing_if 组合；PartialEq str；is_empty。
2. **round-trip 无损**（AC#4）：手写旧格式 TOML 字符串（含明文 token/password）→ load → save → 重新 load 相等，且磁盘字节含明文（B1 sync.toml/sync_folders.toml、B2 .ccs_config.toml 各一）。
3. **Debug 不泄漏**：`format!("{:?}", SyncConfig/ConfigSection)` 不含明文原文、含掩码串（B1/B2 各一）。
4. **B1 权限**：`sync_folders.toml` 保存后 mode 0o600（`#[cfg(unix)]`，Windows 跳过注释说明）。
5. **B3**：新 cookies 掩码显示格式断言；`format!("{:?}", CreateAccountRequest)` 不含明文 cookies。
6. **回归**：既有全部测试 `-- --test-threads=1` 绿；`public_api_compat.rs` 不动即绿（B0 不碰 ccr 根 facade）。

## 7. 遗留债务（本任务明确不做，回写 spec 时记录）

- `temp_override.rs` 裸 `fs::write` 写路径迁移 guarded write（字段已 Secret 化，写法属 guarded-write 遗留清单 ccr-cli 项）。
- claude/codex profile 详情 IPC 明文下发（已显式 `expose()` 标记）→ 07-03-arch-typed-ipc 换 `has_password` 模式。
- sync 密码 OS keychain 存储（若安全要求提升）；zeroize-on-drop（需全链路审计）。
- `ClaudeSettings` env map 中的 token 值仍为裸 String → 07-03-arch-claude-settings 域。

## 8. 回滚

- B0 单独提交；B1/B2/B3 每批一个提交，互不依赖（都只依赖 B0），任一批可独立 `git revert`。
- 磁盘格式零变化 → 回滚无数据迁移成本；`sync_folders.toml` 的 0o600 对旧代码无害。
- B0 回滚需先回滚已合入的 B1-B3（依赖新类型）。

## 9. Spec 同步

完成后（trellis-update-spec 流程）：

- `ccr-core/backend/backend-guidelines.md`：掩码章节改写为 Secret 契约——唯一算法归属、默认掩码 serde、`expose_plaintext` 注解是唯一明文落盘通道（rg 审查入口）、新凭据字段必须 Secret 的 Wrong/Correct 对照、加密/zeroize 否决理由。
- `ccr-sync` / `ccr-checkin` `backend-guidelines.md`：凭据字段规则与 `sync_folders.toml` secret 写契约一句话引用。
