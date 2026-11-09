# 设计

## 边界
GrokAuthAction → dispatch → commands/grok/auth.rs → 现有 GrokAuthService。CLI 只处理参数、来源选择、确认和安全输出；凭据/锁/CAS/身份校验留在已有服务。继续使用 platforms/grok/auth/accounts.json，不引入第二份 registry/current 指针。

## 数据流
save 从 read_snapshot 的 sources 选择 scope，单来源自动选择，多来源缺参数返回可用 scope 和错误；将同一 revision 传入 save_current。delete 在确认前读取 revision，确认后调用 delete_account；分发层将全局 CLI -y 与 --force 合并为跳过确认参数，不读取或初始化 profile 配置。switch 复用 switch_account，显示 outgoing_saved/warnings，不自动登录、profile off 或重试冲突。

list JSON 以显式 DTO 输出 accounts、sources、runtime_present、runtime_error，禁止输出 credential/revision。current 保持 existing logged_in 的文件存在性语义，避免库损坏改变既有查询行为。off 继续协调保存可识别原账号再整文件登出；帮助需说明这一副作用。

## 取舍与风险
参考 Codex 核心命令命名和 force 行为，不复制特有 repair/备份机制，也不复制打印错误后返回成功的 handler。无格式迁移、无新依赖。fixture 只能证明本地行为，不证明原生 Grok 或服务端认证。PATH 安装二进制不会随源码自动更新；本任务不自动替换用户全局程序。回滚仅限本任务 diff。
