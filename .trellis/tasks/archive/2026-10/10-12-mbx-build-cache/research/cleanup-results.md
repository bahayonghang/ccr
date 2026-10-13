# 旧 C 盘产物清理（2026-10-12）

用户在迁移验收完成后明确要求清理旧 C 盘相关产物，本次作为原任务后续操作记录。

## 范围与结果

- 删除唯一旧 target：`C:\Users\lyh\AppData\Local\mbx\targets\v1\6b736f3e8608d890383d61470c216d50cd84b38db0e955508f465ce6ab4ea1b5`。
- 删除 35,407 文件、19,233,327,389 字节逻辑数据；清理前后 C 盘可用空间实测增加 17,809,801,216 字节。
- 随后移除空的旧 `targets/v1` 和 `targets` 目录；保留当前生效的 `C:\Users\lyh\AppData\Local\mbx\bin`。
- D 盘共享缓存、两个受管 target、配置备份及验证日志均保留。

## 安全核对与验证

执行前验证旧目录及其父级为预期的普通目录、树内没有 reparse point；两个项目 target 都精确指向此前验证的 D 盘受管目录。未发现可执行路径或编译器参数指向旧 C target 的进程。

只用 PowerShell 对已核验绝对路径执行删除。删除后旧目录不存在，D 盘 CLI 和 Tauri 可执行文件的 SHA-256 与删除前一致，活动 Cargo shim 存在。使用持久 PATH 的 `mbx doctor --json` 再次得到 9 pass、0 failures、0 warnings。

原始清理结果：本机忽略目录 `outputs/mbx-repair/cleanup-result.json`。无产品代码更改，无须重复构建，未提交 Git。

## 回滚边界更新

旧 C 盘 target 副本现已按用户要求永久删除。此前迁移报告中“恢复链接到旧 C 目录”的回滚步骤不再适用；PATH 配置备份仍有效，构建产物以 D 盘为准，必要时可重新构建。
