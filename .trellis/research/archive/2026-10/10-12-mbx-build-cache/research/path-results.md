# 持久 PATH 修复结果

2026-10-12 用户批准实施后，主会话完成本机入口修复。

- 本机备份：`D:\mbx-cache\repair-backups\ccr-20261012-path\environment.json` 与 `mbx-config.toml`；完整私人 PATH 不提交。
- 普通进程无法写 HKLM，错误为 `Requested registry access is not allowed`。经 Windows UAC 授权后，已准备的本地修复脚本退出 0。
- Machine PATH 前置 `C:\Users\lyh\AppData\Local\mbx\bin`，原字符串其余部分完全不变；保留 `ExpandString` 类型，User PATH 未改。
- 注册表逐项读回：`ExactMachineChange=true`、`RegistryTypePreserved=true`、`UserPathUnchanged=true`。
- 已发送 `WM_SETTINGCHANGE` / `Environment` 广播，调用返回成功。
- 补充验证正常加载 profile 的 PowerShell 子进程也解析到同一 mbx shim；`where.exe cargo` 顺序为 mbx shim 第一、Rustup Cargo 第二。
- 从真实持久 Machine + User PATH 构造新环境，启动 `pwsh -NoProfile -NonInteractive`：`Get-Command cargo` 指向 mbx shim；`cargo --version` 为 1.98.0，与修复前相同。
- 两个 target 迁移完成后，在同一持久环境下运行 `mbx doctor --json`：9 个检查项全部 pass，`failures=0`、`warnings=0`。根和 Tauri target 分别解析到 `D:\mbx-cache\targets\v1` 下各自的 checkout digest。

## 生效边界

当前已运行的 Codex 保留原进程环境，不能因注册表或广播而自动替换其 PATH。因此本轮工具构建显式前置同一 shim；用户应重启既有 Codex/IDE/终端后使用普通 Cargo。已验证的是持久配置及使用该配置的新进程，没有声称正在运行的桌面进程已重启。

## 回滚

以管理员权限读取上述备份，恢复 Machine PATH 的原始字符串和 `machineKind` 类型，再发送环境变化通知并重启相关应用。User PATH 与 mbx config 未修改，无需恢复；不要把备份写入仓库。具体修复脚本位于本机忽略目录 `outputs/mbx-repair/repair-path.ps1`。
