# Linux 验证环境探测

2026-09-28。WSL Ubuntu-24.04 可用，仓库映射为 /mnt/d/Documents/Code/Github/ccr。

- python3 只读探测发现 /home/lyh/.cargo/bin/cargo 存在，但不在 exec 模式 PATH。
- 运行 /home/lyh/.cargo/bin/rustup toolchain list 意外触发 1.98.0 同步及 downloading 5 components；主会话通过该工具会话 Ctrl-C 中止，退出1。不能将该探测描述为无副作用。未执行后续安装或删除缓存。
- 中止后只读列出：stable-x86_64-unknown-linux-gnu 有 rustc/cargo；1.98.0-x86_64-unknown-linux-gnu 无 rustc/cargo；1.88.0有rustc、无cargo。未记录中止前目录清单，不能判断哪些部分文件是本次新增。
- 直接执行 stable/bin/rustc --version，结果 rustc 1.95.0 (59807616e 2026-04-14)，满足项目记录的MSRV。Windows当前已安装1.98.0。
- T06 checker可用已安装stable真实cargo/rustc/rustdoc，局部PATH与RUSTUP_TOOLCHAIN=stable、--offline --locked，独立target-dir=/tmp/ccr-architecture-linux-01a0e781。依赖缓存不足就记录未执行，不自动联网安装。
- 该补验为Linux/MSRV定向契约验证，不等于 pinned 1.98.0 Linux正式全量门禁，更不代表macOS通过。

## 离线结果与后续边界调整

T06 reviewer离线尝试在解析阶段失败：Cargo.lock锁定indexmap 2.14.1，WSL缓存只有2.14.0。root随后允许使用现有真实stable工具链按Cargo.lock恢复构建依赖（保留--locked，去掉--offline），以执行已批准的Unix进程清理回归；仍禁止调用rustup自动安装/更新工具链，不改全局配置。需记录Cargo.lock前后SHA256及真实测试结果。此调整只针对构建依赖恢复。
