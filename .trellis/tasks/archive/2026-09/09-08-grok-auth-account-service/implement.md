# 服务执行计划

## 顺序与独占文件

- [x] 实施授权和停止后切换范围已确认；已启动并读取协议研究、core atomic/lock、Grok profile/auth_off、测试 fixture 规范。
- [x] core 补齐 FileLock 非截断打开、new secret Windows DACL，保留既有继承策略；维护对应规范/测试源码。
- [x] 在 ccr-cli 的 GrokAuthService/内部模块实现 Secret 账号库、snapshot DTO、scope 验证/身份匹配/观察版本。
- [x] 按 Codex 保存语义落实仅复制到 CCR Grok 目录；save 不碰官方锁或原生文件、不要求 Grok 退出。维护官方锁占用、无锁文件及捕获后原生刷新夹具源码；未执行。
- [x] 实现 save/switch/delete 及共享 auth_off 锁协调；维护个人/团队身份、enrichment/歧义、并发与失败夹具源码。
- [x] 同步服务模块导出并静态核对现有 current/off 消费者接口；编译按要求跳过，不新增 CLI save/switch 命令或 Tauri 账号 API。
- [x] 更新 auth-off.md/grok-profile-runtime.md 及 core 规范：允许账号服务受控读写 auth，profile 操作仍不能碰 auth/MCP。
- [x] 独立静态审查秘密流、并发、写后错误和自动回存决策；已修正已确认生产缺陷，API 已接入 TUI。

不修改 TUI/theme 或清理子任务文件；如需接口调整，通知父任务协调。

## 验证

本轮用户要求不测试：只维护必要测试源码并静态审查，不执行下列命令、构建或运行时检查。全部执行项记 SKIPPED（用户要求）；不触碰当前 Grok Build、真实凭据或配置，不安装/替换二进制。

```powershell
rtk cargo test -p ccr-core -- --test-threads=1
rtk cargo clippy -p ccr-core --all-targets --all-features -- -D warnings
rtk cargo test -p ccr-cli grok_auth -- --test-threads=1
rtk cargo test -p ccr-cli auth_off -- --test-threads=1
rtk cargo test -p ccr-cli grok -- --test-threads=1
rtk cargo test -p ccr-cli --test dispatch_routing -- --test-threads=1
rtk cargo test -p ccr --test commands grok_profile -- --test-threads=1
rtk proxy just lint-strict
```

新用例统一可被 grok_auth 过滤捕获，必须确认非零数量；如拆文件导致命名不匹配，补实际命令。Windows ACL 用当前 Windows 测试；Unix mode 测试在 Unix 环境执行，无环境则单列缺失证据，不能以 cfg skip 当通过。不得为此装全局工具。

core 完整工作区 test/build、其他平台回归与 Tauri 编译留作后续验证，本轮不执行父 just ci。本方案保持公共 current 字段。无真实账号操作/网络 token 请求/native UI。

## 失败与回退

先冻结服务契约再交给 TUI。保存库/运行时数据不做发布回滚或用户迁移；失败按设计的原子写/回读机制报告。只回退本任务代码，不撤销其他子任务改动。

已补 Windows runtime 拒绝替换与 DACL 设置失败的故障测试源码，未执行。仅自有文件运行 rustfmt 源码格式化；产品测试/编译/lint/runtime 均 SKIPPED。PRD 行为 AC 不勾选，任务保留 in_progress，未提交/归档。最终审查结果见父任务 research/implementation-review.md。
