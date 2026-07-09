# Secret 掩码 newtype

## Goal

ccr-core 新增 `Secret` 类型：Debug/Display/serde 内置统一掩码，`expose()` 唯一出口。淘汰 checkin `mask_api_key` 与 ccr-db `mask_cookies_json` 两套分叉算法；堵上 SyncConfig/WebDavConfig.password 明文无掩码缺口。审查候选 2（Strong）。

## Requirements

### 现状（探索报告定位）

- 3 套掩码算法：`ccr-core::mask_sensitive`（prefix4…suffix4）、`ccr-checkin/crypto.rs:213 mask_api_key`（prefix-****suffix4）、`ccr-db/models/checkin/account.rs:222 mask_cookies_json`+`mask_value`（prefix2****suffix2）。
- 掩码靠约定：config_service.rs:377-381 必须"记得"调 mask_sensitive；类型层面无强制。
- 缺口：`SyncConfig.password` / `WebDavConfig.password` 明文存储、从不掩码、从不加密——而 ccr-checkin 对 cookies 用 AES-256-GCM，两个凭据存储安全姿态相反。

### 要做的

1. ccr-core 新增 `Secret<T = String>` newtype：`Debug`/`Display` 输出统一掩码；serde 默认序列化为掩码，落盘原文需显式路径；`expose()` 是取原值的唯一出口。
2. 凭据字段迁移到 `Secret`：config 的 auth_token 类字段、checkin cookies/API key、sync WebDAV password。
3. 删除 `mask_api_key`、`mask_cookies_json`/`mask_value`，统一走 `Secret` 的掩码；`mask_sensitive` 保留为 `Secret` 内部实现或直接吸收。
4. sync 密码是否升级为加密存储（对齐 checkin 的 AES-256-GCM）在设计阶段决策；本任务底线是"不再明文出现在日志/Display/序列化默认路径"。

### 约束

- 磁盘格式兼容：已存在的配置文件必须能无损读入（原文反序列化进 Secret）。
- 掩码格式变化会影响 UI 显示与既有测试快照，需盘点 `rg 'mask_sensitive|mask_api_key|mask_cookies'` 的全部消费方。
- 涉及凭据处理，触发 rust-security-reviewer 审查。

## Acceptance Criteria

- [x] 全仓掩码算法只剩 1 处实现；`mask_api_key`、`mask_cookies_json` 删除。
- [x] `Secret` 的 Debug/Display/serde 掩码行为有单元测试；`expose()` 之外无法取到原文（类型层面验证）。
- [x] auth_token、checkin 凭据、WebDAV password 均为 `Secret` 类型；`rg 'password.*String'` 在凭据结构上无裸 String 残留。
- [x] 既有配置文件读写往返（读旧文件→保存→再读）无损，有测试。
- [x] 日志/错误信息中不出现凭据原文（抽查 Display/Debug 路径有测试覆盖）。
- [x] `just lint-strict`、`just test` 通过；rust-security-reviewer 子代理审查通过。
- [x] masking 相关 spec 条目更新（trellis-update-spec）。

## Notes

- 复杂任务：`task.py start` 前需补 design.md（Secret 泛型形状、serde 策略、加密决策）与 implement.md。
- 与 07-03-arch-guarded-write 互不阻塞；若同期进行，secret 文件的 0o600 判定可复用本任务的类型信息。
