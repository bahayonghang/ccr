# 本机修复设计

## 边界与数据流

正确的持久环境 -> 普通 cargo -> 已安装 mbx shim -> mbx -> Rustup Cargo / rustc -> D 盘 action store 与 managed targets。just、无 profile PowerShell 和 Tauri 的 Cargo 子进程依靠该入口自然接入；不引入项目专属 wrapper 或 mise 依赖。

## 持久环境

先在本机非版本管理目录备份 User/Machine PATH 和 mbx 配置。由于 Machine PATH 已有 `.cargo\bin`，不能仅把 shim 加到 User PATH 就宣称成功。优先在既有 Machine Rust 路径之前插入唯一 shim 目录，保留其他条目原序、内容及类型；该操作只在获得 OS 权限时执行。无管理员能力时明确阻塞持久修复，不用 PowerShell profile 冒充无 profile/GUI 的全局修复。更新后广播环境变化，验证新进程组合环境；当前 Codex 进程需重启的限制单独记录。

不覆盖 rustup 的 cargo.exe，不改 ACL，不更换已经正确的 cache_dir。实际构建可在单次执行进程内显式前置 shim，持久验收则必须单独证明。

## 存储迁移

根 target 是旧 C 盘 mbx symlink。执行前重新检查没有使用本次 CCR 源/目标目录的活动构建；解析并验证 link、旧目录和 D 盘目的路径都等于诊断记录中的受控位置。先复制旧输出到 D 盘任务专用临时保留目录并核验，再只解除根 target 链接，将已复制的输出作为普通 target 放回工作区供 mbx adopt 在 D 盘同卷接管。保留 C 盘旧目录直到迁移和构建通过，不执行自动跨盘删除迁移。

Tauri target 已在 D 盘，可直接使用 `mbx adopt` 接管并保留内容。先 dry-run，再操作；不处理其他 checkout。若原子切换或接管失败，恢复旧 link/目录，保留副本，不盲目递归删除。

迁移后检查 mbx 元数据对两个 workspace 的归属。旧 C 盘保留目录的最终清理是独立回收步骤，不作为本次修复的前置条件。

## 验证

先 doctor / Cargo 解析 / target 实际路径，再构建 CLI 和 UI Rust 后端。用同一小 crate、相同 profile 和工具链在两个全新 target 构建，第二次 mbx 的 session、summary 或 stats 必须有实际命中，区分 Cargo Fresh。完整项目构建为真实入口验收，独立 target 为共享缓存验收。

## 回滚

从本机备份恢复被修改的 PATH；重新启动相关进程。恢复根 target 原 symlink；Tauri 根据迁移日志恢复其原目录位置。禁止清空共享 D 盘 cache 或删除未验证的旧输出。用户私人配置和备份不纳入 Git。
