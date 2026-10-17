# 实施顺序

设计：`design.md`。未得到对规划摘要的明确同意前，不运行 `task.py start`，不改产品代码。

## 顺序

1. 在 `ccr-config` 增加目录库模块：建表、ACL、修订令牌、`extra_toml` 往返、NULL current / usage / enabled。先用临时库做单测，不接生产调用。
2. 实现 `migrate_catalogs()` 的阶段表。权威提交和导入行同一次提交，提交前不移动 TOML。只回收带 attempt 标记且尚未权威的新库。无标记的已有库保持不动。覆盖五个平台，含 Grok `api_key` 和缺字段的 Gemini，并注入「提交后、移动前」「移动后、`source_retired` 前」和「移动失败」。
3. 把 `ConfigManager` 的 load、snapshot、mutate、patch、autofix、clear current 接到库。直接 load 在仅有 TOML 时返回「尚未迁移」，不解析它。无变化的 mutate 不推进 `revision`。同一线程的活动 `CatalogTransaction` 被内层 load、mutate 和 `update_current_config` 复用。
4. 改五个平台的 `load_profiles_from_file`、`base::load_profiles_from_toml`、`load_current_profile_marker`、`update_current_config` 和 `mutate_profiles*`，使移走 TOML 后的列表和切换走库。保留注册表优先和空 current。
5. 改 `profile_lifecycle`：一个目录事务跨过 `execute`，文件日志不再包含 `profiles.toml`。SQLite `COMMIT` 成功后才 `journal.commit()`，成功操作记录写在这两者之后；`COMMIT` 失败时文件仍能回滚，且 operation id 不呈现为已成功。计数在该提交点之后的独立事务里饱和增加。
6. 改初始化、export/import、桌面 raw get/save。raw 冲突不写库。import/export 不创建活 TOML。临时文件、staging 和新库在写密钥前套 `restrict_owner_only`。
7. 改 `profile open`：无 `$VISUAL` / `$EDITOR` 即失败；阻塞编辑器退出后导入并删除临时文件；临时路径避开平台目录。
8. 改同步：锁内 checkpoint，busy 或错误则不发布；backup API 生成快照后再上传。排除 `profiles.toml`、`-wal`、`-shm`。只有上传成功才 DELETE 远端 `profiles.toml`，404 为成功。拉取用事务导入活库，不改名已打开的 `profiles.db`。
9. 更新命令输出、医生检查和参考文档中的活路径。同步更新 `.trellis/spec/ccr-config/backend/backend-guidelines.md` 与 `profile-repository.md` 的存储描述。
10. 跑下面的验证。先窄后宽。修复时不放宽现有纯读、密钥和补偿断言。过滤器执行 0 个测试就算失败。

## 验证

- `cargo test -p ccr-config --lib`。其中迁移阶段、事务复用和权限用例至少各有一个非零结果。
- `cargo test -p ccr-cli --lib application::profile_contract`。必须命中该模块里已有的生命周期夹具，并加上 `COMMIT` 失败后的 operation-id 重放。
- `cargo test -p ccr-cli --lib commands::data::export::test_determine_output_path`
- `cargo test -p ccr-cli --lib commands::data::import::test_merge_configs`
- `cargo test -p ccr-sync --lib`。必须命中 checkpoint busy 不 DELETE、上传失败不 DELETE、404 DELETE，以及打开中的库在拉取失败时 revision 不变。
- `cargo test -p ccr-codex --lib platforms::codex`。结果为 0 个测试则失败。
- `cargo test -p ccr -- --skip export_bindings`。保持默认并行度。
- `cd ccr-ui && bun run tauri:test`。该脚本已跳过 `export_bindings`。
- 受影响包的 clippy，`--all-targets --all-features -- -D warnings`
- `just fmt-check`
- 跨 crate 收尾再跑 `just ci`

直接调用仓库读接口的「不建库」用例要覆盖两者都缺、仅 TOML、已 `source_retired`、损坏输入。入口迁移用例另写，不能拿仓库读接口的结果充当 CLI list。夹具可以先写遗留 TOML，但平台 adapter 的切换断言必须发生在 TOML 已经移走之后。密钥哨兵不得出现在日志或 Debug 捕获里。Unix mode 与 Windows DACL 分平台记录，测不到的一侧写 `NOT_RUN`。

## 风险点

- `crates/ccr-cli/src/application/profile_lifecycle.rs`：SQLite `COMMIT` 先于 `journal.commit()`。`COMMIT` 失败时文件预映像必须还在。
- `crates/ccr-config/src/platforms/base.rs` 和五个平台的 `load_profiles_from_file`：移走 TOML 后不能再返回空目录。
- `crates/ccr-config/src/managers/config/repository.rs`：修订令牌、空 current，以及内层加入同一事务。查询不建库。
- `ccr-ui/src-tauri/src/commands/profile_lifecycle.rs`：raw CAS 从文件字节改为 `revision`。
- `crates/ccr-sync/src/sync/service.rs` 与 `transaction.rs`：快照上传，以及拉取时事务导入而不是改名活库。排除规则要有旧断言的反向更新。
- 迁移后若把备份 TOML 手工放回去，新二进制不会再读它。文档要写明这一点。

## 启动前

`implement.jsonl` 和 `check.jsonl` 已有研究与契约条目。启动时再确认设计文件也在两份清单里。同意规划摘要之后才 `task.py start`。
