# P5 源码研究

来源：父任务 research/followup-design-research.md；2026-10-06。仅为只读研究，行号在实施前复核。未运行产品验收。

 快照文件命名评估

- auth_service.rs:187、oauth_token_service.rs:122、quota_service.rs:539 都由账号别名拼接 name.json。validate_account_name（auth_service.rs:1597）限制 ASCII 字母/数字/_/- 和长度、保留 default；审计未发现路径安全缺口。
- 仅用 identity_key.base64url 作为唯一文件名会让相同完整身份的多个别名共享同一快照，而当前别名可各自保存不同 token 新鲜度、描述、api_base_url、provider 配置。需要先决定去重记录、共享快照引用计数和删除语义；不能把 P1 的 deterministic target 直接当作“一身份一文件”的批准。
- API key/provider 没有 OAuth user/account pair；缺身份 OAuth 旧文件也无法命名。不能用密钥明文派生文件名；base64url 可逆，不提供身份隐私。若将来需要与显示别名解耦，随机 opaque snapshot ID 往往比完整身份编码更易支持多个别名，但属于另一个设计选择。
- 迁移涉及三个路径 resolver、rename/delete/force overwrite、导入导出、backup、repair 和外部用户手工恢复流程。需要 manifest、旧/新布局双读、迁移前备份、每项中断恢复、回滚映射、版本门和 unknown-field 保留；文件移动成功不等于 registry 提交成功。
- 建议本轮结论：暂不实施。P1 已修复同步身份混淆，别名文件名在现有校验下无已证实安全缺口；重复别名、API key 和缺身份的迁移成本超过当前证据支持的收益。
- 交付白名单仅 P5 任务评估文件；评估 AC 通过不要求无代码变化时重复 cargo/just 门。报告明确布局迁移、回滚和 native 行为 NOT_RUN。

