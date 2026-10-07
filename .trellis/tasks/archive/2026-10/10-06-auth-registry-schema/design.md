# 设计：注册表版本门与未知字段保留

## 边界

- 所有改动在 `crates/ccr-codex`（模型、注册表存储、`CodexAuthService` 6 个命令、`CodexOAuthTokenService::update_registry_metadata`）与 `crates/ccr-tui/src/tui/codex_auth/app.rs`（toast 本地化）。
- 不改 `ccr-types`、Tauri DTO、ccr-vscode、导出 DTO。不新增 `CcrError` 变体。

## 1. 未知字段保留（R1）

```rust
// models/codex_auth.rs
pub struct CodexAuthRegistry {
    // ... 既有字段 ...
    /// 未识别字段（较新版本写入），load → save 原样保留
    #[serde(flatten)]
    pub extra: toml::Table,
}

pub struct CodexAuthAccount {
    // ... 既有字段 ...
    #[serde(flatten)]
    pub extra: toml::Table,
}
```

- 空 `toml::Table` 不输出任何键，1.x 现有文件的保存结果不变。
- `Default` 与全部结构字面量补 `extra: toml::Table::new()`（生产：`codex_auth_service.rs:931`、`:2004`；其余为测试）。
- `save_current` 覆盖账号时构造新 `CodexAuthAccount`，`extra` 为空，旧记录的未知字段随之移除（R1 的显式语义）。
- TOML flatten 序列化依赖 toml 1.x 的文档缓冲；先例 `ccr-config/src/managers/platform_config.rs:76`。实施第一步用往返测试确认顶层未知表与 `accounts` 子表混排时的输出可再次解析。

## 2. 版本规则与只读判定（R2、R3）

```rust
// codex_registry_store.rs
pub const SUPPORTED_REGISTRY_MAJOR: u32 = 1;
pub const REGISTRY_READ_ONLY_PREFIX: &str = "注册表只读：";

impl CodexAuthRegistry {
    /// 主版本 > 1 或无法解析 → 只读
    pub fn is_read_only(&self) -> bool;
}
```

- 主版本解析：`version.split('.').next()?.trim().parse::<u32>()`；失败视为只读。缺失 `version` 时 serde 默认 `"1.0"`，可写。
- 只比较主版本：次版本变化属于追加字段，由 R1 保护。
- `version` 字段按加载值写回，不改写为 `"1.0"`。

只读错误文本（服务层，CLI 与桌面端原样显示）：

```
注册表只读：auth_registry.toml 版本 {version} 由更新版本的 CCR 写入，当前版本仅支持 1.x；请升级 CCR 后再执行此操作
```

文本只由一个函数生成，并配一个反向提取函数，往返由测试锁定：

```rust
pub fn registry_read_only_message(version: &str) -> String; // 生成上面的完整文本
pub fn registry_read_only_version(error: &str) -> Option<&str>; // 在任意位置找到前缀，返回「版本 」与「 由」之间的值
```

`CcrError::ConfigError` 的 Display 带「配置文件错误: 」前缀（`ccr-core/src/core/error.rs:115`），TUI 拿到的是 Display 文本，因此 `registry_read_only_version` 用 `find` 定位前缀，不用 `strip_prefix`。

## 3. 检查点（R3、R4）

| 调用方 | 检查位置 | 只读时行为 |
| --- | --- | --- |
| `save_current` | 加载注册表后、创建目录与写快照之前（`codex_auth_service.rs:901` 之后） | 返回只读错误 |
| `switch_account` | 方法入口，`ensure_*` 之后、加载与观测点同步之前（`:1169-1175`） | 返回只读错误；不运行切换前同步 |
| `delete_account` | 删除快照之前（`:1279-1293`） | 返回只读错误 |
| `update_account_description` | 加载后（`:1441`） | 返回只读错误 |
| `rename_account` | 任何备份与移动之前（`:1467-1486`） | 返回只读错误 |
| `import_accounts` | 加载后、写入任何快照之前（`:1898`） | 返回只读错误 |
| `CodexRegistryStore::save` | 锁内、备份与写入之前 | 返回只读错误（兜底，覆盖未来新增写入点） |
| `update_registry_metadata` | 调用 `save` 处 | 只读错误 → `warn!` 后 `Ok(())` |
| `sync_current_auth_registry` | 调用 `save` 处 | 只读错误 → `warn!` 后返回计算出的 `new_current` |

- 命令侧统一用私有帮助函数 `ensure_registry_writable(&registry) -> Result<()>`；`save` 兜底复用同一判定。
- 后台调用方用模式匹配判别：`Err(CcrError::ConfigError(msg)) if msg.starts_with(REGISTRY_READ_ONLY_PREFIX)` → `warn!` 后按成功处理；其他错误照常返回。不新增错误变体。
- 只读模式不影响 `CodexOAuthTokenService` 的快照写入与 `commit_plan` 的 runtime 写入（R4）；这些路径只写注册表元数据时才跳过。

## 4. 加载（R5）

```rust
pub fn load(&self) -> Result<CodexAuthRegistry> {
    // 读取文本 → toml::from_str::<CodexAuthRegistry>
    // 失败时：再用 toml::from_str::<toml::Table> 只取 version；
    //   若主版本 > 1 或无法解析 → ConfigError("解析注册表失败: 版本 {v} 由更新版本的 CCR 写入，请升级 CCR（{原错误}）")
    //   否则保持原错误文本「解析注册表失败: {e}」不变
}
```

## 5. TUI 本地化（R6）

- 在 `codex_auth/app.rs` 增加 `fn localized_service_error(error: &str) -> String`：`registry_read_only_version(error)` 命中时返回
  - EN：`registry is read-only: version {v} was written by a newer CCR; upgrade CCR to change accounts`
  - ZH：`注册表只读：版本 {v} 由更新版本的 CCR 写入；升级 CCR 后才能修改账号`
  - 否则返回原文。
- 应用于保存、切换、删除、重命名的错误 toast（`app.rs:1014-1016`、`:1057-1059`、`:1190-1192`、`:1317-1318`）。TUI 没有改描述与导入入口；这两个命令只在 CLI 与桌面端触发，显示服务层中文文本。toast 外层「Save failed: / 保存失败：」等前缀不变。

## 兼容性与回滚

- 1.x 文件：`extra` 为空、版本可写，读写行为与文本不变。
- 回滚：单独回退本任务提交即可；回退后的版本会丢弃 `extra`（与修复前行为相同），不会损坏文件。
