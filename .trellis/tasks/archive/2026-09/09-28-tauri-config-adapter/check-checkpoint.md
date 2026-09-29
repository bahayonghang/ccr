# T03 早期只读检查

日期：2026-09-28。检查者：`/root/check_t03`。基线：`34d8a85e0e48b793733835e0304c8ed33940fcee`。本报告对应实施中的工作区快照；源码指纹见 `research/check-checkpoint-evidence.json`。

## 范围与结论

本轮按主代理授权只读检查已落盘后端和契约。`implement_t03` 仍拥有生产、测试、registry、frontend 和生成文件的写入权。检查者只写本报告及证据文件，未运行 Cargo、lint、type-check、bindings 或 inventory 生成，未修改源码、任务状态或真实用户配置。

共享 enable 的事务结构符合 T02 的责任边界；CRUD 已移入 T01 路径资源 mutation。发现的 enabled 空值缺口已由实现者修复源码，运行结果仍须确认。活动配置回退判定存在待修复的保护缺口。T03 尚不授予最终通过。

## Findings (fixed)

### F1：enabled 的 null 曾被当作省略字段

- 初始源码：`ccr-ui/src-tauri/src/commands/config/adapter.rs:70-72` 的 `Option<bool>` 仅带 `serde(default)`。反序列化 `enabled: null` 会得到 None，随后 patch 忽略该字段。
- 对应反例：`config/contract_tests.rs:63-64` 要求显式 null 被拒绝。已同时通知主代理和实现者。
- 实现者修复：新增 `enabled_field`，使用 `bool::deserialize(...).map(Some)`；字段省略仍由 default 处理。复读确认位于当前 `adapter.rs:38-42,76-78`。
- 本检查者未修改源码，未独立执行回归。`backend-second.log` 仍记录 strict patch 用例失败，不能将本项标为完整运行验收通过。

## Findings (not fixed)

### N1：禁用和删除只保护 raw current，未覆盖 Claude registry 回退

- 证据：`adapter.rs:149-161` 仅在 `config.current_config == name` 时禁止禁用；`:183-185` 委托 `ConfigService::delete_config`。该 service 在 `crates/ccr-config/src/services/config_service.rs:263-271` 仅检查 raw current 和 default。
- 现有权威行为：`crates/ccr-cli/src/platforms/claude.rs:120-129` 通过平台 resolver 读取 file/registry；`crates/ccr-config/src/platforms/base.rs:803-825` 定义有效 file 优先、有效 registry 回退。现有 `claude.rs:596-618` 回归明确允许 file current 为 ghost、registry 为 beta 时实际 current 为 beta，且查询不修复磁盘。
- 可达条件：beta 存在且不是 default，profiles current 为 ghost，registry current 为 beta。generic list 在 `commands/config.rs:185` 将 beta 标为非当前；update enabled=false 和 delete 均不命中上述保护。runtime 设置仍可使用 beta。
- 结论：代码路径已确认；本轮未运行该 fixture，也未声称已在用户配置重现。需要实现者加入实际 handler 回归，并在现有平台 owner 的 current 判定下保护活动配置。避免自行修复 marker 或改变 T01 的纯读/回退规则。
- 未修改原因：本轮只读授权；该问题涉及业务判定，源码由实现者负责。已同时发送主代理和实现者。

### N2：严格 patch 的非对象反例仍需定位

- `backend-second.log` 记录 `strict_patch_rejects_unknown_wrong_types_missing_collision_without_write` 失败，12 passed / 1 failed。该日志属于实现者的运行，本检查者没有启动或重跑。
- `contract_tests.rs:63` 的候选包含 `[]`。派生 struct 的 sequence 反序列化与全 default 字段可能使空数组被接受；`deny_unknown_fields` 只约束字段，不单独保证输入为 object。
- 已通知实现者用不泄露值的索引确认失败输入，并保留空数组应拒绝的反例。本报告不将尚未定位的失败与 F1 合并推断为同一原因。

## 已核对的实现边界

1. **显式平台**：`adapter.rs:22-27` 只接受 claude；缺少平台返回 platform_required，Codex/Grok/未知值返回 config_platform_unsupported。`commands/config.rs:161-267,394-403` 的迁移入口先校验平台，再进入 blocking worker。删除仍先处理确认。
2. **enable 与 apply**：`crates/ccr-cli/src/application/profile_lifecycle.rs:218-245` 先验证目标，再把 enabled=true 和平台 apply 放入同一次 execute。`:269-303` 的 journal 覆盖执行、提交记录和逆向补偿；`:306-370` 的提交后 ancillary 错误转为 warning，没有普通 Err 返回。尚需最终运行证明每个新增写入阶段的结果。
3. **单 worker**：`commands/config.rs:207-217` 将整个共享激活用例放在一个 spawn_blocking 内；没有在 handler 重组 off/save/delete/apply。
4. **资源写入**：add 使用 mutate_or_create；update 使用 mutate_versioned；duplicate 使用 mutate；delete 使用 T01 service；rename 使用 T02 update_profile。源码位置为 `adapter.rs:128-185`。未保留旧 adapter 专属 config 锁或全量 save RMW。
5. **版本与保留字段**：list 返回 `snapshot.version`；update 把 expected_version 传入 T01。T01 `repository.rs:142-155,171-247` 的 token 来源为原始字节，资源锁内校验后执行 guarded CAS。patch 沿原 ConfigSection 修改，duplicate 克隆原 section；TOML datetime 和未编辑扩展依赖已验证的 T01 delta 持久化。
6. **结果秘密边界**：ConfigMutationResult 仅含 platform/name/outcome；ConfigPatchInput 无 Debug。list 继续使用 Secret Display 脱敏。共享 ProfileOutcome 不返回原始 profile 或 I/O 错误。
7. **registry**：`handler_registry.rs:542-556` 将 update_config 移入 typed config 域，命令 ID 保留。旧 config_extended 与新 config 默认 risk 均为 LocalMutation，默认授权/确认/audit 推导相同；module gate 由 config_extended 变为 config。实际 CRUD 互斥仍须以 T01 path resource lock 和并发 fixture 证明，不能仅以 module 名一致认定通过。

## 最终检查待验证项

- AC1：React 真实配置页操作，经 generated/domain wrapper 发出明确 claude 与 enable flag；实际 Rust handler 验证 runtime/current/enabled 一致；警告与 recovery 不显示纯成功、不重复激活。
- AC1：enable 每个实际写入阶段故障补偿、replay 和 failure outcome；未编辑 marker/扩展/凭据保持。当前 `profile_contract.rs:407-455` 已包含三平台 enable fixture，独立运行尚未进行。
- AC2：strict patch 每项非法输入、显式清空与省略；missing/collision/stale token 无目标及备份写入；真正经 desktop handler 与共享 service 的并发独立修改不丢失。
- AC2：编辑器发送读时 token；未编辑 masked token 不回写；清空 optional 字段发送 null；未知字段不被消费者静默吞掉。前端仍在实施，本轮未把中间状态列为最终缺陷。
- AC2：补充 N1 的 stale-marker 活动保护 fixture，并保留 inactive 文档行为。
- AC3：所有旧缺平台请求、unsupported、registry metadata、main/tray ACL、确认与实际 resource 互斥；handler/manifest/client 同源无漂移。当前 generated 文件尚由实现者独占生成。
- 正式 bindings guard：T11 已记录 export 进程启动失败和逐字节恢复；不能把该历史失败归因于 T03，也不能在本轮重复生成。最终整体证据由 owner/T10 提供。

## Verification

- Lint：未运行，本轮只读 checkpoint。
- TypeCheck：未运行，本轮只读 checkpoint。
- Tests：未独立运行；仅读取实现者 backend-second.log 的 12 passed / 1 failed 记录。
- Native UI / browser / Linux / macOS：未运行。
- 源码、测试、生成文件修复：检查者无写入。F1 由实现者修复源码；最终复验等待 freeze 后由主代理安排。

## 续轮静态复审

2026-09-28，主代理恢复本检查。后端源码已冻结；主代理接管 inventory/bindings 窗口，实现者继续前端草稿修复。本节保留早期失败记录，不以实施者结果代替独立验证。

- F1/N2 源码已修：ConfigPatchInput 使用 `serde_json::Map` 作为 Deserialize 入口，非对象被拒绝；字段按明确类型取出，enabled:null 被拒绝，optional null 与 absent 分别保留。未知剩余字段返回固定错误。位置：`adapter.rs:66-89`。
- N1 源码已修：list 使用 Claude 既有 resolver 标识 current；update/delete 在同一 application operation lock 内读取 actual current，分别拒绝禁用/删除。位置：`commands/config.rs:167-188`、`adapter.rs:160-174,196-202`。没有引入 query repair。
- 实施者 `backend-third.log`：17 passed / 1 ignored child helper。已审阅源码，包含实际 handler 切换/启用/CRUD、ghost registry current 保护、operation-lock barrier、3 轮 desktop-adapter/service 双进程持资源锁争用、strict patch、CAS 与 TOML extension。ignored helper 由父测试显式启动。该日志仍是实施者证据；独立执行尚未开始。
- 新问题 N3：EditConfigModal 恢复旧全量草稿时，把 baseline/version 改为新读 snapshot。旧草稿中未编辑的旧字段会被当作 patch 并携带新 token，覆盖外部更新而不触发预期冲突。位置：`EditConfigModal.tsx:47-53,79-87`、`configForm.ts:79-93`。已交实现者绑定草稿原版本/基线并补真实表单回归。
- 新问题 N4：同一组件换配置或重新读取失败时，没有清除旧 baseline。必须在新读取开始时使旧基线失效，并禁止无有效 snapshot 的保存。已交实现者补回归。
- AddConfigModal 的草稿仍包含 auth_token，与 EditConfigModal 新增的 secret 排除不同。已要求实现者明确草稿秘密边界；当前未修改该文件。
- 生成前置仍未满足：续轮读取的 generated/config.ts 仍为旧签名，ConfigPatchInput/ConfigPlatform 等文件尚未生成。不能据此给 T09 typed-client 前置通过。
- 当前没有独立测试/lint/type-check 命令会话；所有 Cargo 仍等待主代理当前 inventory exact 命令结束通知。
