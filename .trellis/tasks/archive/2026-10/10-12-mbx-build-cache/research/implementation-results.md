# 存储迁移与构建验证

> 后续更新：旧 C 盘 target 已按用户要求清理，旧目录回滚步骤不再适用；见 [清理记录](./cleanup-results.md)。下文保留迁移时的历史证据。

## 实施边界

- 用户已批准设计；本记录仅覆盖 target 迁移及构建/缓存验证。持久 PATH 结果见 `path-results.md`。
- 不修改产品源码、Cargo 配置或 justfile；不删除旧 C 盘输出。
- 当前 Codex 进程仍持有旧环境，构建子进程显式前置已安装的 mbx shim。

## 迁移

- 迁移前未发现 cargo/rustc/mbx 构建进程。
- D 盘剩余约 994 GB；旧 C 盘目录含 35,407 个文件、19,233,299,473 字节，没有嵌套 reparse point。
- 旧目录：`C:\Users\lyh\AppData\Local\mbx\targets\v1\6b736f3e8608d890383d61470c216d50cd84b38db0e955508f465ce6ab4ea1b5`。
- 根 target 复制到 `D:\mbx-cache\migration-10-12-ccr\root-target`，耗时 101.10 秒；35,407 个文件逐一比较大小和 SHA-256，全部一致，核验耗时 190.60 秒、核验时总量 19,233,327,389 字节。
- 核验记录保留于 `D:\mbx-cache\migration-10-12-ccr\verified-copy.json`。只解除项目原 symlink，将已验证副本同卷移动后 `mbx adopt` 成功；根链接落点为 `D:\mbx-cache\targets\v1\6b736f3e8608d890383d61470c216d50cd84b38db0e955508f465ce6ab4ea1b5`。
- Tauri `mbx adopt --dry-run` 后同卷接管成功，28.3 GiB logical 内容保留。链接落点：`D:\mbx-cache\targets\v1\44061aa3da5bb563d33700d15e5ae7660d5d3da33a41e506f28329282a48c2b6`。
- 核验期间另一个项目启动了 Cargo：进程树明确指向 `skills-manage-windows` 的 `skillport.exe`。确认与 CCR 目录无关后继续迁移，未停止该进程。

## 构建与共享缓存

本机原始日志及命令结果位于 `D:\mbx-cache\migration-10-12-ccr\verification`，mbx 原始 session 位于 `D:\mbx-cache\actions\sessions\v1`。

| 检查 | 结果 | mbx session |
| --- | --- | --- |
| `just build` | exit 0，外层 28.89 秒，Cargo 23.60 秒；0 hits / 1 miss / 24 unconsulted，写入 412,418,727 字节 | `1791866980979-41064-uehl` |
| `just ui-build-backend` | exit 0，外层 645.65 秒，Cargo 10 分 43 秒；0 hits / 0 misses / 23 unconsulted / 1 bypassed，写入 144,056,329 字节 | `1791867018324-5804-dzim` |
| `cargo build --locked -p ccr-types --target-dir <A>` | exit 0，12.49 秒；0 hits / 35 unconsulted，写入 51,455,534 字节 | `1791867082602-54236-mxub` |
| 同样命令 `--target-dir <B>` | exit 0，2.65 秒；35 hits / 0 misses，`ccr_types` 自身 outcome=hit | `1791867127682-41064-m0rd` |

`<A>` / `<B>` 分别是验证目录下的 `ccr-types-a` / `ccr-types-b`，每次执行前断言目标不存在；相同工作区、工具链和 dev profile，仅 target 路径不同。第二次实际恢复 74 文件、51,284,578 字节，全部采用 copy，hardlink 数量为 0。这个证据来自 mbx session，不能与 Cargo Fresh/no-op 混为一谈。

已观察到的缓存覆盖限制：Tauri 的部分编译结果引用位于 Tauri 根目录之外的 `crates/ccr/assets/featured-skills.json`、`examples/claude/profiles.example.toml`，mbx 提示绝对路径没有稳定映射，因而没有存储这些结果。Tauri session 明确记录 `ccr_desktop` 因 `unmodeled-link-argument` bypass，主 crate 编译约 601 秒；这次成功构建不代表全部桌面编译结果都可恢复。A 首次构建有一次读取 Cargo build-script declarations 不存在的警告；B 成功命中且没有该警告。以上未通过改业务源码或猜测配置规避。

最终 `mbx cache stats --json`：721 objects、81 action results、2 live checkouts、2 managed target directories；action store 607,960,738 字节，managed targets 50,168,226,742 字节，combined 50,776,949,215 字节，仍在既有 100 GiB 总预算内。`mbx cache projects` 同时列出 CCR 根工作区与 Tauri 工作区。根工作区最后一次使用显式 probe target，因此该命令显示其最新 target 的体积；默认 target 的 D 盘路径以文件链接及 managed-target 元数据为准。

最终产物存在且由本次构建更新：根 `target/debug/ccr.exe` 为 28,376,576 字节，Tauri `target/release/ccr-desktop.exe` 为 59,179,520 字节。两条项目链接仍分别指向上述 D 盘受管目录。

本次未改产品/构建源文件，未运行全仓 `just ci`；实际 CLI/Tauri 构建及共享缓存恢复是该本机配置任务的验收范围。

## 回滚

保留旧 C 盘目录。若需回滚根目录，仅解除项目 target 的 symlink，再重新建立指向上述旧目录的链接，不递归删除任何源目录。

Tauri 可在没有构建进程时解除项目 target 链接，将上述 D 盘受管目录移回 `ccr-ui/src-tauri/target`；操作前必须再次核对路径和链接类型，避免跟随链接删除内容。
