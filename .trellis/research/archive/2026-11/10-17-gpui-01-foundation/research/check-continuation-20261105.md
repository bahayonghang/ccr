# GPUI 01 文档续作独立检查（2026-11-05）

日期使用 America/New_York。基线为 `dev` / `0bf9ba48938fb286a581f4e9eb51e07f3273c629`；范围只含本任务 PRD、执行计划、evidence、本轮 continuation 与本报告。没有修改产品、依赖、锁文件、历史原始收据或 task.json，没有提交或归档。

## 已核对与自修

- 按 check.jsonl 读取三个 curated 引用、PRD/design/implement/evidence 和本轮 continuation；依赖治理规范过大导致注入截断，已分段直接补读。
- 当前 Tauri main 的 logger/catalog/pool/llmusage 初始化顺序、默认 preferences 读写、monitoring 全局池、局部 CCR_ROOT/LLMUSAGE_HOME seam 与 WebView 默认身份陈述符合所引 owning source。独立 WebView `data_directory` / `dataDirectory` 在所查 Rust 启动源码与 Tauri 配置中没有配置，不能以业务 DB override 代替隔离。
- **修正过强措辞**：`main.rs:208–209` 确实调用 WSL 探测入口，但 `platform/wsl.rs:231–251` 在有效磁盘缓存命中时直接返回，未命中才到 `:288–292` 执行 WSL。已在 implement/evidence/continuation 同步为“调用带缓存入口，未命中才执行”，本轮仍不启动应用。
- **修正源码定位**：补全 catalog migration 引用为 `crates/ccr-config/src/managers/config/catalog/migrate.rs:29`。其余路径/行号经源码核对，包括 GPUI launch mode、fixture UI-state pool、usage schema 19 seed API、archive upsert、Web telemetry 与窗口验证脚本。
- GPUI 已有 profiles/registry、Insights 和 UI-state 显式路径；文档没有继续沿用“所有 fixture seam 不存在”的旧判断。现有 seam 仍不是两端同负载 S0/S1/S2 或 OS process-create → first-present/action-ready 的采样收据。

## 宿主与证据边界

- 独立执行 `rustc -vV` 与 `rustup target list --installed`，均退出 0：Rust 1.98.0、LLVM 22.1.8，host 与唯一 installed target 都是 `x86_64-pc-windows-msvc`。
- 本任务现有 research 的 macOS / Darwin 相关记录均为缺证说明；`outputs/desktop` 没有匹配 macOS/Darwin/Apple 名称的新增收据。本轮没有提供或生成 macOS 编译收据，不据此断言系统上任意位置绝不存在其他工具。AC1.5 保持开放。
- Windows AC1.1–1.4/native 输入继续引用历史收据；历史 WSL Linux 编译不转记为独立 Linux 桌面验收。性能 `not_run`，相对指标 `not_comparable`；没有新样本、原生截图或百分位数。
- 产品候选 `0bf9ba48` 的 desktop-fmt/lint/test 和根 `just ci`（0，14:13.540）属于前序 [09 独立检查](../../10-17-gpui-09-environments/research/check-20261105.md)。本轮 01 仅修改文档，没有重跑这些命令，也没有把该收据作为 01 性能或 macOS 通过证据。
- 没有运行 WSL/doctor、启动 Tauri/GPUI、安装 target、连接 SSH 或读取身份文件。

## 本轮适当检查

| 检查 | 退出码 / 结果 |
| --- | --- |
| `python ./.trellis/scripts/task.py validate .trellis/research/archive/2026-11/10-17-gpui-01-foundation` | 0；两个 manifest 各 3 项引用有效；既有大文件注入警告保留 |
| 五份当前 Markdown 尾随空白、相对链接和源码引用范围检查 | 0；12 个本地链接、21 处源码引用 |
| task.json `in_progress` / `completedAt=null`，PRD AC1.5 未勾选断言 | 0 |
| 当前改动仅属于本任务 Markdown，产品及历史 baseline 文件无差异 | 0 |
| `git diff --check` | 0 |
| desktop-fmt/lint/test、根 `just ci` | 本轮 `not_run`；没有产品改动，不重复前序门禁 |

文档切片通过独立检查，可交主会话本地提交。任务保持 `in_progress`，不满足归档条件；父任务条件也没有因本报告改变。检查完成时没有范围外脏文件。
