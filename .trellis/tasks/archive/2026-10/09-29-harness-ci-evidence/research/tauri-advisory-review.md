# T07 O01：Tauri 9 条 RustSec 告警诊断

## 结论

9 条告警保持未解决：7 条 unmaintained、2 条 unsound。没有修改 Cargo.toml、Cargo.lock、audit ignore 或全局工具，没有把 cargo-audit 退出 0 当作告警已修复。

本轮诊断在 Windows 主机执行。已捕获 28 条命令，全部退出 0；Cargo 1.98.0、cargo-audit 0.22.2。RustSec 缓存 Git HEAD 为 f23b768236fe2880e4cfa167da662cad8ca79240；缓存存在未跟踪文件 crates/stable-vec/RUSTSEC-0000-0000.md。先前“缓存工作区干净”的表述有误，现已更正。audit 报告加载 1277 条公告，该总数不能等同于固定提交的已跟踪公告数。未 fetch 数据库。

后续逐字节核验确认：本报告消费的九份公告与该固定 revision 的 Git blobs 完全一致，也与最初捕获的 SHA-256 一致。未跟踪残留没有被本报告消费，当前 Tauri lockfile 不含 stable-vec，已捕获 audit 结果不含 RUSTSEC-0000-0000。该残留没有改变本报告采用的九条公告记录或结果；整个缓存仍不能表述为干净。复核只读，未删除、修改或隐藏残留文件。

诊断 audit 命令显式使用 --no-fetch --no-yanked，仅复核已知 informational 告警；没有检查 yanked 状态，不替代正式完整 audit。结果为 vulnerabilities.count=0、unmaintained=7、unsound=2，ignore=[]。原审查完整日志保留在父任务 research/evidence/cargo-audit-tauri.log。

## 目标与 feature 边界

使用当前 Tauri lockfile，对以下 target 分别运行默认 feature 和 --all-features。当前桌面 manifest 只有 default=[] 与 custom-protocol，两种模式的告警包可达性相同。

- Windows：x86_64-pc-windows-msvc。
- Linux：x86_64-unknown-linux-gnu。
- macOS：aarch64-apple-darwin；本轮未覆盖 Intel macOS。

每个 target/mode 均对比 --edges normal,build 与 --edges normal,no-proc-macro。第二组排除 build 和 proc-macro 依赖，用于区分普通目标依赖图与构建链。图中存在包不证明函数执行、代码最终链接或漏洞可利用；跨 target 的静态解析不等于在目标操作系统编译或运行。

| 包及告警 | 类别 | Windows | Linux | macOS ARM | 当前处置 |
| --- | --- | --- | --- | --- | --- |
| fxhash 0.2.1 / RUSTSEC-2025-0057 | unmaintained | 构建链 | 构建链 | 构建链 | 未解决；随 Tauri HTML 构建依赖链替换或上游升级。 |
| proc-macro-error 1.0.4 / RUSTSEC-2024-0370 | unmaintained | 不在所查图中 | proc-macro 构建链 | 不在所查图中 | 未解决；由 GTK/glib 宏依赖链处理。 |
| unic-char-property 0.9.0 / RUSTSEC-2025-0081 | unmaintained | 普通及构建链 | 普通及构建链 | 普通及构建链 | 未解决；随 urlpattern 的 Unicode 依赖迁移。 |
| unic-char-range 0.9.0 / RUSTSEC-2025-0075 | unmaintained | 普通及构建链 | 普通及构建链 | 普通及构建链 | 未解决；随 urlpattern 的 Unicode 依赖迁移。 |
| unic-common 0.9.0 / RUSTSEC-2025-0080 | unmaintained | 普通及构建链 | 普通及构建链 | 普通及构建链 | 未解决；随 urlpattern 的 Unicode 依赖迁移。 |
| unic-ucd-ident 0.9.0 / RUSTSEC-2025-0100 | unmaintained | 普通及构建链 | 普通及构建链 | 普通及构建链 | 未解决；随 urlpattern 的 Unicode 依赖迁移。 |
| unic-ucd-version 0.9.0 / RUSTSEC-2025-0098 | unmaintained | 普通及构建链 | 普通及构建链 | 普通及构建链 | 未解决；随 urlpattern 的 Unicode 依赖迁移。 |
| glib 0.18.5 / RUSTSEC-2024-0429 | unsound | 不在所查图中 | 普通依赖链 | 不在所查图中 | 未解决；Linux 函数级运行可达性仍未知。 |
| rand 0.7.3 / RUSTSEC-2026-0097 | unsound | 构建链 | 构建链 | 构建链 | 未解决；当前图未启用公告所需 log feature。 |

## 依赖链、责任层与后续处置

以下责任层为建议归属；本轮没有向个人分派或发送外部消息。项目入口均为 ccr-ui/src-tauri/Cargo.toml 的桌面 Tauri 依赖管理。

### fxhash

代表链：ccr-desktop → tauri-build 2.6.3 / tauri-macros 2.6.3 → tauri-utils 2.9.3 的 html-manipulation → kuchikiki 0.8.8-speedreader → selectors 0.24.0 → fxhash 0.2.1。该链位于桌面构建或宏展开路径；三个普通无宏图均不含 fxhash。

RustSec 将其标为维护停止，patched=[]，建议的替代为 rustc-hash。上游责任链涉及 Tauri、Brave kuchikiki 与 Servo selectors。项目后续应评估 Tauri 构建链提供的兼容替代；直接给 CCR 加另一个哈希 crate 不会移除传递告警。本轮不做替换，也不把维护停止写成已确认运行时攻击路径。

### proc-macro-error

Linux 代表链为 ccr-desktop → tauri / wry / GTK → glib 0.18.5 → glib-macros 0.18.5 → proc-macro-error 1.0.4；gtk 0.18.2 → gtk3-macros 0.18.2 也依赖同一包。实际 features 为 default,syn,syn-error。Windows 与 macOS ARM 两种模式均无该节点。

RustSec patched=[]。建议责任归属为桌面 Linux 平台维护；上游为 gtk-rs 宏 crate 及 proc-macro-error 维护链。后续应结合 glib/GTK 更新评估宏依赖替代，保留 Linux 编译和 proc-macro 检查，不单独改 lockfile 跨越上级范围。

### 五个 unic 包

共同主链为 ccr-desktop → tauri / tauri-runtime / tauri-runtime-wry → tauri-utils 2.9.3 → urlpattern 0.3.0 → unic-ucd-ident 0.9.0。Tauri 构建链也使用 tauri-utils。下游分支为：

- unic-ucd-ident → unic-char-property → unic-char-range；unic-ucd-ident 也直接依赖 unic-char-range。
- unic-ucd-ident → unic-ucd-version → unic-common。

unic-ucd-ident 的 features 为 default,id,xid。五个节点均在三个目标的普通无宏图中，不能按仅构建依赖处理。五份 RustSec 记录均为 unmaintained，patched=[]。公告为部分用途指出 icu_properties、unicode-ident 或标准字符范围等替代方向；这些方向不构成可直接替换所有 API 的证明。

建议责任归属为桌面 Tauri 权限/URL 模式依赖维护；上游为 Tauri tauri-utils、Deno rust-urlpattern、open-i18n rust-unic。后续需通过上级库迁移，并保留 URL 模式及权限语义回归。当前图不证明项目实际执行了每个 Unicode API。

### glib unsound

Linux 图包含多条 GTK 链，代表为 ccr-desktop → tauri 2.11.5 → tauri-runtime-wry 2.11.4 → wry 0.55.1 → webkit2gtk 2.0.2 / gtk 0.18.2 → glib 0.18.5；tray-icon 0.24.2、muda 0.19.2 和 libappindicator 0.9.0 也连接 GTK。实际 glib features 为 default,gio,gio_ffi,v2_58,v2_60,v2_62,v2_64,v2_66,v2_68,v2_70。

RUSTSEC-2024-0429 指向 VariantStrIter 的 Iterator/DoubleEndedIterator 实现。缓存源码 variant_iter.rs:118–130 仍通过不可变指针地址传递 C 可变输出参数，再调用 CStr::from_ptr；与缓存公告所描述的问题一致。公告 patched 范围为 >=0.20.0、unaffected 为 <0.15.0。当前 gtk 0.18.2 声明 glib 0.18；单独锁到 glib 0.20 不满足该依赖范围。

建议责任归属为桌面 Linux 平台与依赖维护；上游为 gtk-rs/gtk-rs-core、GTK3 bindings、Tauri/Wry。后续优先确认 VariantStrIter 的项目及传递调用可达性，并评估兼容上游迁移或经过审查的修复回移。该工作超过本轮诊断范围。没有运行 exploit/UB 复现；Linux 普通图可达不能用于判定已发生崩溃或可利用漏洞。

### rand unsound

代表链：ccr-desktop → Tauri 构建/宏链 → tauri-utils 2.9.3 → kuchikiki 0.8.8-speedreader → selectors 0.24.0 的 build-dependency phf_codegen 0.8.0 → phf_generator 0.8.0 → rand 0.7.3。phf_generator 声明 rand 0.7 加 small_rng；源码使用 SmallRng::seed_from_u64(FIXED_SEED) 生成 PHF。

三个 target 的两种模式均解析出 alloc,default,getrandom,getrandom_package,libc,rand_pcg,small_rng,std，未启用 log。rand 0.7.3 的日志宏由 cfg(feature = "log") 控制；ThreadRng 由 std 提供，因此不能把不存在名为 thread_rng 的 feature 误写成 ThreadRng 被禁用。

公告触发条件还要求自定义 logger 重入 ThreadRng、达到重播种路径，并启用对应 trace 或失败 warn 日志。当前锁定 feature 图缺少必要的 log 条件，且已确认的消费者使用 SmallRng。这些静态证据限定当前条件，未验证未来 feature 统一、其他调用路径或所有构建环境，告警仍保留。

公告 patched 范围为 >=0.10.1、>=0.9.3 <0.10.0、>=0.8.6 <0.9.0；0.7.3 没有在该记录中获得修复版本。phf_generator 0.8.0 的 rand 0.7 范围不接受 0.8.6。建议责任归属为桌面构建链维护；上游为 rust-phf、selectors/kuchikiki、Tauri，并由 rust-random 提供安全修复。后续经批准更新上级构建依赖，再复查 feature 与公告，不能直接跨 major 修改 lock 节点。

## 证据与验证

- tauri-advisory-commands.json：28 条完整命令、UTC 时间、退出码、耗时、Cargo 输入 SHA-256、stdout/stderr 和哈希。含 12 个 target/feature/edge 图、9 个完整反向依赖图、工具及 Git 元数据与一次诊断 audit。
- tauri-advisory-audit.json：缓存 audit 原始 JSON，七条维护停止、两条 unsound、ignore=[]；yanked 检查未执行。
- tauri-advisory-sources.json：九份官方 RustSec 缓存记录的 TOML 字段、来源文件 SHA-256、数据库 Git revision。
- tauri-advisory-cache-verification.json：12 条只读 Git 命令均退出 0；九份公告逐字节对照固定 revision、原始哈希、未跟踪残留状态和影响边界。
- tauri-advisory-reachability.json：逐 target/feature/edge 的包与 feature 结果；输入文件前后哈希完全一致。
- tauri-advisory-source-inspection.json：十二个已锁定上游 manifest 的 repository、依赖范围、features 和 SHA-256。
- tauri-advisory-code-excerpts.json：rand 日志/ThreadRng feature、phf_generator 调用与 glib 受影响实现的来源行号及 SHA-256。
- tauri-advisory-capture.py：可重复的固定快照图采集入口。编写阶段曾出现字符串换行转义语法错误；修正并通过语法检查后才开始上述 28 条命令。该错误没有修改产品或产生 gate 结果。

没有重复 Windows doctor、进程 smoke、Tauri coverage、Linux workspace coverage 或 MSRV suite。本报告不重新判定 H01/H02/H03 的历史根因；这些检查由主会话记录。九条 warning 的状态均为 UNRESOLVED，后续修复需要独立的范围、兼容性与平台验收。
