# P3 源码研究

来源：父任务 research/followup-design-research.md；2026-10-06。仅为只读研究，行号在实施前复核。未运行产品验收。

 去重备份、保留池与破坏前置条件

#### 可复用实现

- managers/codex_config.rs:237–277：读取源字节；仅与该池最新备份比较；相同时刷新既有备份 mtime 并返回路径；命中不清理。新备份通过 AtomicWriter.secret(true) 写入。
- :281–297：同秒文件名已存在时追加 _N；:300–320：按 mtime/name 排序；:326–337：保留池 10 份。该 helper 为私有且绑定 runtime backup_dir，不能直接把 CCR auth/backups 送入同一 runtime 保留池。
- codex_registry_store.rs:113–157 的 save/backup 仍忽略自动备份失败、秒级名称+fs::copy。codex_auth_service.rs:1324 的 backup_registry 另有重复实现；:1344 的账号备份已私有原子写，但仍同秒覆盖。
- delete_account（auth_service.rs:1290）直接删除 snapshot；rename_account（:1480）先备份/删除冲突目标，再忽略源 snapshot/registry 备份失败。必须把全部必需备份移到任意删除或移动之前。

#### 建议合同与顺序

- 新增 ccr-codex 私有共享 backup helper（例如 services/codex_auth_backup.rs），供 registry_store 与 auth_service 使用；auth_service.backup_registry 委托 registry_store.backup。保持现有备份根目录与可恢复原始字节。
- 池分为注册表 1 池、每个账号各 1 池。不能用 auth_ 或 auth_account_ 宽前缀旋转，否则用户 A 的历史可删除用户 B 的备份。现有名字可保留；账号名 foo/foo_bar 的池过滤必须匹配完整文件名结构，不能仅 starts_with(auth_account_foo_)。旧文件只在准确归属后参与该池。
- 采用最新相同内容去重、同秒序号、私有原子写。去重返回真实存在且字节相等的路径；读备份或更新时间失败时创建新备份。身份键和凭据不进入备份名称；备份名称仍按安全账号别名。
- 同秒序号选取和创建需在该池的操作锁内完成。现有 exists→AtomicWriter 是 TOCTOU；只序号扫描无法在两个进程并发时保证不覆盖。registry save 已持注册表锁；确定操作锁→backup 池锁→leaf writer 顺序，独立 backup 同样使用池锁，避免递归同一锁。
- 删除：加载/验证可写注册表→验证账号→备份注册表与存在的 snapshot→全部成功→删除 snapshot→移除账号及必要 current_auth→保存。源 snapshot 确实不存在时可返回 None；存在但读/写失败必须中止删除。
- force rename：验证源/目标/版本门→备份存在的目标 snapshot、源 snapshot、原 registry→全部成功→才删除目标、移动源、修改内存 registry 和保存。任何一个备份失败时源/目标 snapshot 和 registry 保持原字节；已成功创建的备份可以保留。
- 自动 registry save 是否将 self.backup() 错误改为传播，需要在设计明确。建议可靠备份为前置条件；不要把失败继续写入描述为已满足 backup-before-destructive-change。
- P3 的 fail-closed 保证范围为备份失败前不破坏。文件移动或最终 registry 保存失败仍可能部分完成，不能宣称多文件事务。若要求补偿，应单独列出最小回滚机制和验证，不自动扩大为通用事务层。
- 保留数量等待用户选择。选择 10 时每池保留 10 份，并明确删除账号的 backup 也属于凭据留存。选择暂不清理时实现去重/防覆盖/私有写和失败中止，保留清理不运行，PRD 不再声称本轮有界保留。

#### 白名单与验证

- services/{mod.rs,codex_auth_backup.rs,codex_registry_store.rs,codex_auth_service.rs}；必要时对应 CLI/TUI EN/ZH 固定失败提示和断言。无需改 runtime CodexConfigManager 的既有保留合同。
- 测试覆盖：连续相同内容路径复用；固定同秒不同内容；多线程/独立进程不覆盖；不同账号及 foo/foo_bar 池隔离；注册表/账号 10 份或不清理的选择；删除前可恢复相同字节；源/目标/registry 逐一备份失败均不删除任何 snapshot；目录变文件的确定性 I/O 失败。
- 采用 tempfile/TestCodexEnv；失败注入限测试 closure/局部函数，不新增全局环境开关。Unix owner-only 与 Windows 新备份 DACL 分平台验证。

