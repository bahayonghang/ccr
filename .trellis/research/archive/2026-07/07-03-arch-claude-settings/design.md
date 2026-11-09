# Design: 合并 ClaudeSettings

## 侦查结论（修正 PRD 预设）

1. **UI 侧已是富类型 adapter**。`ccr-ui/src-tauri/src/commands/claude.rs:27` 起明确注释"使用 ccr_types::ClaudeSettings 而非 ccr::ClaudeSettings"，`claude_update_settings` 已走 merge→from_value 验证→to_value 写回。PRD 第 2 条"Tauri settings 命令改为 adapter"实际已就位，本任务 UI 侧只需更新注释。真正的工作面在 CLI 侧。
2. **`ConfigSection` 定义于 ccr-config**（`crates/ccr-config/src/managers/config/types.rs:45`），ccr-cli 经 `managers::config` re-export。ccr-config 目前不依赖 ccr-types。
3. **`Validatable` trait 在 ccr-core**，ccr-types 是纯 leaf（仅 serde/chrono/uuid）。orphan rule 禁止在 ccr-cli 为 `ccr_types::ClaudeSettings` impl `ccr_core::Validatable`。全仓无泛型 `T: Validatable` 消费 ClaudeSettings（doctor/validate/tests 均为直接方法调用）→ 固有方法即可，调用点文本不变。
4. **无代码遍历贫瘠版 `ClaudeSettings.other`**：rg 命中的 3 处 `.other.get("auth_mode")` 全是 `ConfigSection.other`（TOML）。贫瘠→富切换后 other 语义收窄（仅未知字段）不影响任何现有读取。UI 的 statusline 读写 `.other.get("statusline")`（富类型，statusline 非已知字段）语义不变。
5. **公共 API 快照零文本改动**：`public_api_compat.rs` 锁的是 `crates/ccr/src/lib.rs` 的 re-export 源码行。`pub use managers::{... ClaudeSettings ...}` 文本不变，`managers::settings` 内部改为 re-export ccr-types 即完成指向切换。
6. **ccr-config 已有映射预览重复**：`ConfigSection::to_anthropic_env_status`（types.rs:260）注释自述"与 ClaudeSettings::update_from_config 保持一致"——正是 PRD 所述跨 shape 漂移的实例。

## 目标形态

```
ccr-types  (leaf)      ClaudeSettings 唯一定义 + 全部纯数据变更/查询/验证方法 + 托管 env key 常量
   ↑              ↑
ccr-config (新增依赖边)  ConfigSection::to_managed_env_pairs() —— ConfigSection→env 的唯一映射
   ↑
ccr-cli                SettingsManager / CachedSettingsManager 纯 IO adapter（load/save/backup/restore）
                       调用点组合：settings.apply_managed_env(section.to_managed_env_pairs())
ccr (根)               re-export 链路不变，指向自动切换
ccr-ui src-tauri       已是富类型 adapter，仅注释更新
```

### A. ccr-types：`claude_settings.rs` 新增（纯数据，无新依赖）

- 托管 env key 常量 pub 化：`ANTHROPIC_BASE_URL` 等 18 个键 + `NON_ANTHROPIC_MANAGED_KEYS` 表（自 ccr-cli settings.rs 迁入，供 ccr-config 映射引用，杜绝字面量漂移）。
- 固有方法（语义逐字迁自 ccr-cli 贫瘠版）：
  - `new()`（等价 `Default::default()`，保住全仓 `ClaudeSettings::new()` 调用点）
  - `clear_anthropic_vars(&mut self)`
  - `clear_managed_vars(&mut self)`
  - `apply_managed_env(&mut self, pairs: impl IntoIterator<Item = (String, String)>)` —— 先 `clear_managed_vars` 再逐对 insert；这是原 `update_from_config` 去掉 ConfigSection 知识后的纯数据核
  - `anthropic_env_status(&self) -> HashMap<String, Option<String>>`
  - `has_anthropic_overrides(&self) -> bool`
  - `validate_api_key_mode(&self) -> Result<(), String>` —— 错误文案保持现有中文原文
  - `validate(&self) -> Result<(), String>` —— 语义同原 `Validatable::validate`（无 overrides 即 Ok，否则走 api_key_mode）；固有方法优先解析，CLI 调用点 `settings.validate()` 文本不变
- 错误用 `String` 而非新错误枚举：仅有两个消费点且均 `to_string()`/`format!`，调用点以 `map_err(CcrError::ValidationError)` 包装后**错误类型与文案与现状完全一致**。避免为单用途引入公共错误类型（Simplicity First）。

### B. ccr-config：ConfigSection 映射（新增 `ccr-types` path 依赖）

- `ConfigSection::to_managed_env_pairs(&self) -> Vec<(String, String)>`：原 `update_from_config` 的 18 键映射逐字迁入（含 `auth_token.expose()` 及其"合法明文消费点"注释）。None 字段跳过的语义不变。
- `to_anthropic_env_status` **不动**（4 键展示预览，surgical）；仅更新其注释指向 `to_managed_env_pairs`。
- 依赖边方向：config→types，types 仍为 leaf，无环。

### C. ccr-cli：settings.rs 收缩为 IO adapter

- 删除本地 `ClaudeSettings` struct、全部 inherent impl、`Validatable` impl（约 -310 行），改 `pub use ccr_types::ClaudeSettings;` —— `crate::managers::settings::ClaudeSettings` 与 `ccr::ClaudeSettings` 路径零改动。
- `SettingsManager` / `CachedSettingsManager` 签名自动跟随，IO 逻辑（lock/tempfile/AsyncAtomicWriter/backup/restore）**零改动**。
- 调用点改写（3 处生产代码）：
  - `platforms/claude.rs:348`、`services/settings_service.rs:70,80`：`settings.update_from_config(&section)` → `settings.apply_managed_env(section.to_managed_env_pairs())`
- 验证调用点（2 处）：`doctor_service.rs:931-933`、`commands/lifecycle/validate.rs:190` 追加 `.map_err(CcrError::ValidationError)`（若上下文需要 CcrError）或直接消费 String。
- **不加 `ClaudeSettingsExt` 兼容 trait**：生产调用点仅 3 处，保留 trait 会维持"变更逻辑在 CLI 侧"的假象，与任务目标相悖。`ccr::ClaudeSettings::update_from_config` 方法从公共 API 消失是有意为之（快照只锁 re-export 符号名，不锁方法；PRD 已授权快照有意更新流程，此处连快照都不动）。

### D. ccr 根 crate tests 改写（4 文件）

- `workflows/temp_override.rs:56,103`、`managers/general.rs:231,587`：update_from_config → pairs 组合调用。
- `managers/general.rs` 若 `use ccr::Validatable` 因此 unused 则移除。
- `commands/{doctor,current,validate}.rs` 与 `doctor_service.rs:1819` 的 struct literal `ClaudeSettings { env, other }` → 追加 `..Default::default()`。

### E. ccr-ui src-tauri

- `commands/claude.rs:27-28` 注释更新为统一后事实。其余零改动，前端零改动，Tauri payload serde 形状零变化（UI 本就走富类型）。

## 行为变化与规范化（有意接受，测试固化）

| 场景 | 贫瘠版行为 | 富类型行为 | 判定 |
|---|---|---|---|
| hooks 字段为 string/number/bool | 塞进 `other` 容忍 | 解析报错 | 改进：doctor 更早诊断损坏；restore 拒绝坏备份 |
| legacy hooks 数组 | 原样保留 | 归一化为 canonical object 格式 | ccr-types 既有文档语义；UI 写路径已如此 |
| 磁盘上 `"agents": []` 等空容器/null | 原样往返 | 读后不再写出（skip_serializing_if） | 语义无损的规范化 |
| 未知字段 | flatten 保留 | flatten 保留 | 不变（核心回归风险，往返测试覆盖） |
| `"env"` 字段 | 恒写出 | 恒写出 | 不变 |
| 键顺序 | HashMap 无序 | HashMap 无序 | 不变（现状已重排） |

"无损"验收口径：**非空已知字段语义保留 + 未知字段字节级保留**；空容器消失与 hooks 归一化为可容忍规范化。

## 测试策略

- **ccr-types**：迁入并改造纯数据测试（clear/apply/env_status/has_overrides/validate 系列，pairs 直供不再依赖 ConfigSection）；新增往返保留测试：富字段（mcpServers/hooks/plugins）+ 人为未知字段 + legacy hooks 归一化断言。测试代码禁 `unwrap()`（lint-strict），统一 `expect`。
- **ccr-config**：`to_managed_env_pairs` 18 键映射断言（extended envs / custom model option / runtime envs / fable & model names / None 跳过）；防串档组合测试（apply 两个 profile 断言旧键清除）——直接消费 ccr-types::ClaudeSettings。
- **ccr-cli**：SettingsManager IO 测试原地保留；新增磁盘级往返测试（写含富字段+未知字段 JSON → load → apply_managed_env → save → load → 断言未知字段与非托管 env 保留）。
- **ccr 根**：4 个测试文件改写后全绿，public_api_compat 快照不变即为通过。
- **UI**：`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml settings` + `just frontend-check-quick`（含 claude-settings.smoke）。

## 提交切分（每提交可编译）

1. `feat(types)`: ccr-types 吸收变更/查询/验证方法 + 常量 + 单元测试（纯新增）。
2. `feat(config)`: ccr-config 依赖 ccr-types + `to_managed_env_pairs` + 映射/防串档测试（纯新增）。
3. `refactor(cli)`: 删除贫瘠 shape、切 re-export、改 5 处调用点、迁移/改写测试、UI 注释（一次切换）。

## 回滚

三个提交独立 revert；提交 1、2 为纯新增可单独回滚；提交 3 回滚即恢复双 shape 现状。磁盘 settings.json 格式不变，无数据迁移，无运行时回滚动作。
