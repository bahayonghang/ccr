# DC-02 权限修复及定向验证

作者：/root/check_t01。root 在旧 Windows ci-final 结束后授权修改；该增量需 root 独立审查。

原 Linux coverage-rust 与精确红例均因语义不变的 mutation 跳过 secret 权限保障失败。保留原 base.rs 0600 断言及失败日志。

修复范围为 6 份现有文件。repository 的 no-op 分支进入 core 的叶锁与 handle CAS 检查，再执行同一 Unix mode 策略。内容、inode、mtime 与 backup 集合不变；0644→0600、0400保留、特殊位按原writer规则归一化。Windows既有DACL保留。

metadata-only调用不登记content rollback entry。已有content entry继续按记录的metadata恢复；本次不修改T05补偿策略。

定向验证：ccr-config 104通过/1子进程helper ignored；core guarded 27通过；core atomic 9通过。2包all-targets/all-features Clippy -D warnings、cargo check、4文件rustfmt check、限定diff-check均退出0。源文件及generated在5份最终定向回执前后无变化。

注入policy fault仅证明权限修改前失败边界，不等价于真实OS chmod拒绝。原生CSP第6次执行及旧Windows ci-final仍归属旧d8ee10源码冻结；新epoch完整门禁由root执行。

完整6文件SHA、before原文、delta与命令日志指纹见 remaining-permissions-fix.json。
