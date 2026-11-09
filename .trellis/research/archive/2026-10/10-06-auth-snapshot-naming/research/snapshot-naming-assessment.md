# 快照文件命名评估

状态：CURRENT_SOURCE_REVIEWED；等待独立检查。日期：2026-10-06。已按当前 P1、P3、P4 工作树代码复核路径、身份与账号操作。本任务只交付评估，未迁移产品布局。源码 SHA256 与文档检查见 `implementation-validation.md`。

## 结论

建议暂缓迁移，是否实施仍待用户单独决定。P1 已把 OAuth 同步、修复与别名选择改为完整身份匹配；这些代码继续读取别名路径。文件名与别名解耦可减少重命名时的文件操作，但还需决定重复别名、缺身份记录、跨平台文件名冲突与恢复语义，当前没有充分依据直接采用“一身份一文件”。

现有名称校验排除了目录分隔符、点号与非 ASCII 字符。该校验不保证物理路径唯一性。P3 独立检查报告的 Windows 大小写别名问题仍见当前代码，须保留为后续设计约束；本评估没有修复该行为。

## 当前源码证据

| 对象 | 当前行为与锚点 |
| --- | --- |
| AuthService resolver | `crates/ccr-codex/src/services/codex_auth_service.rs:191` 返回 `auth_storage_dir()/format!("{}.json", name)`；`:175` 的存储目录为 `ccr_codex_dir/auth`。 |
| OAuthTokenService resolver | `crates/ccr-codex/src/services/codex_oauth_token_service.rs:153` 返回 `ccr_codex_dir/auth/<name>.json`。 |
| QuotaService resolver | `crates/ccr-codex/src/services/codex_quota_service.rs:854` 返回相同的别名路径。三处均未使用 `identity_key` 生成文件名。 |
| 名称校验 | `crates/ccr-codex/src/services/codex_auth_service.rs:1734` 拒绝空名；`:1739` 只保留精确小写 `default`；`:1748` 允许 ASCII 字母、数字、`_`、`-`；`:1755` 限制最多 32 字节。允许的字符都是 ASCII，合法名称长度为 1–32 个字符，大小写均可用。 |
| 完整 OAuth 身份 | `crates/ccr-codex/src/services/codex_auth_identity.rs:20` 拼接 `chatgpt_user_id::chatgpt_account_id`；`:24` 从 id/access JWT 的顶层或命名空间读取 user，冲突时返回未知；`:45` 优先使用 `tokens.account_id`。JWT 解码只作本地关联，不验证签名。 |
| 缺身份与非 OAuth | `crates/ccr-codex/src/services/codex_auth_identity.rs:61` 对非空 OpenAI API key 返回未知身份；缺 user/account 的 tokens 也不能构成完整键。`crates/ccr-codex/src/services/codex_auth_service.rs:457` 和 `:461` 保留 API key/provider 指纹形式的 `account_id`，这些值不能作为 OAuth 完整键。 |
| 旧记录关联 | `crates/ccr-codex/src/services/codex_oauth_token_service.rs:856` 从每个别名自己的快照推导身份；缺失、损坏或与 registry 键冲突的快照不参与关联。`:880` 仅在允许写入的执行观察点补齐缺失键，只读 registry 跳过。 |
| 重复别名 | `crates/ccr-codex/src/services/codex_auth_service.rs:917` 按名称检查冲突，`:1008` 按名称插入记录，没有按完整身份去重。`crates/ccr-codex/src/services/codex_oauth_token_service.rs:907` 按 `current_auth`、最新 `last_used`、插入顺序选择同步目标。 |

`CodexAuthAccount` 在 `crates/ccr-codex/src/models/codex_auth.rs:239` 保存逐别名的描述、Base URL、provider 名称和时间元数据。重复身份可以有多个文件和不同元数据。P1 的 `sync_registered_aliases_locked`（`crates/ccr-codex/src/services/codex_oauth_token_service.rs:348`）已在身份和新鲜度检查后回写其他别名的 tokens，仍逐文件处理；该流程没有建立共享快照引用或删除最后一个引用的规则。

## Windows 大小写边界

P3 报告 `../10-06-auth-destructive-backup/research/independent-check.md:57` 已记录 `Foo`/`foo` 问题。当前保存与重命名使用区分大小写的 registry 键检查；在大小写不敏感的 Windows 文件系统上，两条记录可能解析到同一物理快照。强制重命名在 `crates/ccr-codex/src/services/codex_auth_service.rs:1679` 删除目标后，`:1687` 才检查源是否存在，存在删除同一物理源文件的风险。该结论来自当前源码与 P3 既有发现；P5 未执行原生重命名复现。

P3 在 `crates/ccr-codex/src/services/codex_auth_backup.rs:38` 折叠 Windows 备份匹配的大小写，并在 `:95` 折叠备份池锁资源。该机制协调同一物理别名的备份，不拒绝 registry 中的大小写别名，也不改变快照路径。

未来迁移须先识别物理源路径是否重合。仅有两条别名记录时，不能推断存在两个独立凭据副本。base64url 使用大小写字母，亦不能仅凭编码字符串不同就保证 Windows 物理路径不同；编码长度、平台大小写比较及目标已存在时的处理都需进入命名合同。

## 收益与成本

| 方案 | 收益 | 兼容成本 |
| --- | --- | --- |
| 保留 `{name}.json` | 无布局迁移；保留现有手工恢复、导入与逐别名记录方式 | rename 仍需移动文件；Windows 大小写别名边界仍待处理 |
| base64url 完整身份 | 完整身份与展示名分离；完成关联设计后，rename 可只改元数据 | 同一身份的多个别名会指向一个目标；需决定共享凭据和删除规则。API key/provider 与缺身份旧记录无相应键。编码可逆，不提供身份隐藏；还需处理 Windows 大小写与长度边界 |
| 随机 opaque snapshot ID | 可为每条别名记录保留独立文件，也可覆盖非 OAuth 或缺身份记录；不直接呈现身份 | 新增持久关联字段与 resolver 迁移；需决定 ID 粒度、分配、物理冲突检查和恢复映射。若选共享文件，仍需引用与删除规则；超出本轮指定方案 |

两个新命名方案均仅作评估，未授权实现。不能以密钥明文、token 或其可逆编码派生文件名。P1 的 `identity_key` 是关联元数据；采用该键作为文件名将新增可见身份信息。

## 账号操作与兼容成本

| 操作 | 当前交互与迁移要求 |
| --- | --- |
| 保存与 force 覆盖 | `crates/ccr-codex/src/services/codex_auth_service.rs:924` 锁定 runtime 和别名目标，`:979` 私有写入别名文件，`:992` 构造新记录。采用共享身份目标会影响其他别名的凭据；须决定替换单条引用或替换共享内容。force 重建记录的现有未知字段丢弃语义须单独区分。 |
| rename | `crates/ccr-codex/src/services/codex_auth_service.rs:1675` 在移动前备份目标、源和 registry；`:1693` 移动文件；`:1697` 保持记录顺序，并更新 `current_auth` 与 `usage_ledger.account_name`。P4 移动帮助函数 `:2195` 先私有发布回退目标再删除源；成功移动后的权限或 registry 发布失败仍可能部分完成。新布局须让 rename 更新相同元数据，并定义 force 目标引用的回收。 |
| delete | `crates/ccr-codex/src/services/codex_auth_service.rs:1451` 在 registry 锁内备份并删除别名文件，移除记录并清理相应 `current_auth`。共享目标须先判断其他引用；独立目标也须保留精确备份关联。 |
| export | `crates/ccr-codex/src/services/codex_auth_service.rs:1890` 遍历别名键，跳过旧 provider 记录；`:1895` 由 `include_secrets` 决定是否读取别名快照。导出 DTO（`crates/ccr-codex/src/models/codex_auth.rs:702`）没有 `identity_key` 或未知 registry 字段。加密入口 `codex_auth_service.rs:1959` 复用完整导出。新 resolver 必须保持导出键与凭据选择一致，内部存储 ID 的跨机器导出语义需另定。 |
| import | `crates/ccr-codex/src/services/codex_auth_service.rs:2061` 校验名称并跳过旧 provider 记录；`:2082` 处理 Merge/Replace 与 force；`:2102` 的 force 路径备份并移除既有快照；`:2126` 仅在有 `auth_data` 时写文件；`:2160` 从落盘快照推导完整键，最后 `:2188` 保存 registry。迁移须覆盖无 `auth_data`、缺身份、API key、重复身份、名称冲突与已有目标，不能仅靠导出元数据生成 OAuth 文件名。 |
| backup | `crates/ccr-codex/src/services/codex_auth_service.rs:1481` 用别名选源与账号备份池。`crates/ccr-codex/src/services/codex_auth_backup.rs:19` 保留别名前缀，`:126` 只比较最新匹配备份。P3 保留所有既有文件。新布局须保留旧备份与恢复映射，决定别名池如何关联共享或独立存储 ID。 |
| repair、同步与 quota | `crates/ccr-codex/src/services/codex_oauth_token_service.rs:253` 按完整身份筛选 runtime、runtime backups 和其他注册别名；`:957` 修复仍写指定别名。`crates/ccr-codex/src/services/codex_quota_service.rs:892` 刷新检查目的文件 tokens、身份与版本，再私有写回。三个 resolver、稳定凭据操作锁、新鲜度与版本检查须同时适配布局，缺完整身份的同文件刷新须继续可用。 |

## 迁移边界

正式迁移前必须决定同身份多别名是否共享凭据、删除是否影响其他别名，以及 API key、旧缺身份和物理路径重合记录的命名规则。还需确定迁移触发时机：加载时自动迁移或显式命令。不能静默合并身份未知或内容不同的文件。

执行需要旧/新布局双读和明确读优先级、registry 与全部源/既有目标的迁移前私有备份、旧别名路径到新路径的 manifest、逐项状态与内容版本、可重复的中断恢复和 registry 提交失败的补偿。先验证完整私有目标，再发布 registry 引用，旧文件的退休阶段须有单独恢复状态；多个文件操作没有 OS 事务保证。

P2 的版本门（`crates/ccr-codex/src/services/codex_registry_store.rs:24`）与未知字段保留（`crates/ccr-codex/src/models/codex_auth.rs:288`、`:353`）不能阻止更早的已发布 CCR 按旧别名布局读取或写入。新布局必须决定兼容版本与降级边界，保留正常元数据往返中的未知字段，拒绝只读 registry 迁移，并覆盖外部写入冲突。锁顺序须沿用凭据资源、registry、备份池、guarded writer 叶锁；锁协议不控制外部编辑器或 Codex 写入。

回滚必须由 manifest 和匹配的私有备份还原旧路径、registry、引用与快照字节，并核对当前内容版本以保留迁移后外部修改。若版本冲突阻止恢复，须返回明确的恢复状态及匹配路径，不覆盖新凭据。回滚要覆盖发布前、发布后、旧文件退休中和中断重启；布局恢复不能恢复服务器已撤销或已轮换的 OAuth 授权。应先在合成夹具验证全部中断点，再独立批准真实布局迁移。

## 证据与未运行项

归档审计 F13 与 `design-research.md` 保留规划来源；上文锚点来自当前源码。评估满足 R1 的报告要求，R2/R3 的产品迁移与无需移动文件的 rename 均未实施，产品迁移验收项保持未勾选。

NOT_RUN：快照迁移、回滚执行、合成迁移故障矩阵、真实账号、用户目录、P5 原生 Windows 大小写重命名、Unix 命名与权限、已安装二进制和托管 CI。本评估没有产品改动，按批准的文档范围不重复 Rust 门禁；`cargo test -p ccr-codex`、`just lint-strict`、`just test` 与 `just ci` 均未在 P5 执行。其他任务既有回执不作为迁移验收。是否实施迁移由用户另行决定。
