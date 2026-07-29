# 实施计划

## 代码步骤

1. 为 credentials/snapshot 建立内存身份令牌与已存快照匹配 helper;补未保存登录守卫。
2. 调整 runtime auth 读取:匹配快照元数据优先,state_file 仅为未匹配登录回退;补 A/B 回归。
3. 用 `write_guarded(secret:true, backup:None)` 替换 AuthService 全部 durable write;移除手搓 tempfile。
4. 在 `SettingsManager` 实现同步/异步 CAS-RMW + 3 次冲突重放 + 集中备份;保留明确的 replace/restore API。
5. 迁移 ccr-cli 所有 Claude settings 的 load->mutate->save 生产调用,并用 rg 审计遗漏。
6. 为 Tauri Claude settings 提供 local shared mutation helper,迁移 agents/hooks/plugins/slash/settings/statusline 等写入口;远程环境维持现状。
7. 更新 LocalEnvironment 的 Claude settings 兜底写策略,禁止 same-dir backup。
8. 为 macOS save/switch 增加显式不支持守卫与跨端错误传播测试。
9. 更新 atomic-writer 与 ccr-cli Claude auth/settings 规范。

## 验证顺序

```powershell
cargo test -p ccr-cli claude_auth -- --test-threads=1
cargo test -p ccr-cli managers::settings -- --test-threads=1
cargo test -p ccr-ui --manifest-path ccr-ui/src-tauri/Cargo.toml claude -- --test-threads=1
cargo test -p ccr-core guarded_write -- --test-threads=1
just fmt-check
just lint-strict
just test
just frontend-check-quick
git diff --check
```

Unix 0600 断言在 Unix CI/可用环境执行;Windows 本地记录 ACL 为继承语义,不得冒充 0600 已验证。

## 风险与停止点

- 先落凭据保护和窄测试,再迁移广泛 settings 调用;每批迁移后重新 rg 调用点。
- CAS mutation 必须可重复重放且不得执行外部副作用;发现非幂等 closure 时拆成“准备数据/纯 mutation/提交”三段。
- 任何日志/错误测试都搜索假 token,确保没有泄露。
- 不修改现有 Profiles Vue/CSS/utility 脏文件;UI 仅触碰 Tauri Claude settings/auth 命令及必要独立认证 DTO。
