# CCR Desktop (GPUI)

独立原生桌面客户端，Cargo package / bin 为 `ccr-gpui`，窗口标题为
`CCR Desktop (GPUI)`，应用身份为 `io.github.ccr.gpui`。它使用自己的 workspace、
Cargo.lock 和 `target/`；根 CLI 与现有 Tauri 命令继续独立构建。

当前默认界面接入原生外壳与 Local 的 Claude / Codex / Grok 配置集。读取、筛选、校验和写入都由有界后台 worker 调用现有领域 owner；列表按稳定名称虚拟化，凭据显示通过领域脱敏。结构化编辑保留读取时的 revision，省略字段表示保留，明确 null 表示清除；冲突保留草稿，只能重读或取消。

三个平台的「配置方案」会在页头显示当前环境与运行文件的核对结果。Claude 只比较 CCR 标记方案的托管字段，订阅模式仅报告覆盖是否清除；Codex 对照运行配置，Grok 同时区分偏离和缺少恢复基线。打开卡片可预览，卡尾的「应用」或「启用并应用」直接作用于该卡。宽窗可并排看目录和详情，窄窗按 Esc 回到原卡。原生样例使用隔离的 27 条方案。[Windows 夹具与实测记录](../.trellis/research/archive/2026-11/gpui-profiles-auth-220/native.md)

「账户」先显示当前环境的凭据文件观测，再列出已保存账户。选择账户后可以明确切换；保存当前登录时才展开命名表单。清除只针对正在运行的认证，删除针对确认框点名的已存账户。不支持的存储仍可查看已存账户和刷新，文件写入操作禁用。

Claude 页提供“配置集 / 配置目录”操作模式，共享列表并保留各自校验与删除策略。配置目录还提供复制、受管备份恢复、导入、脱敏导出和操作历史；配置集提供明确警告后的规范明文源码/导出。Codex 支持受控环境变量与脚本导出预览。Grok 保留分阶段改名结果和激活/漂移/入口状态缺失的删除保护，强制停用并删除需要单独确认，入口状态缺失不提供强制。

本地源码和导出文本仅在当前 feature 的临时实体中保存，取消或离开页面即清除；离开不把已开始的持久写入伪报为取消。转换页只预览现有 Claude Code JSON 与 Codex TOML 转换，不写文件或执行脚本。收藏/最近命令继续由 ccr-db owner 管理，未制造命令执行记录。

底部状态栏切换全局「当前执行环境」，保存本地或 SSH 主机选择；重启后对保存的 SSH 主机自动连接一次。连接失败仍停留在该主机，随环境页面显示原因与重试入口，不回落本地数据。全局设置分为外观、工作台偏好、执行环境、诊断四页；执行环境页管理主机、信任指纹以及 agent / 私钥 / 密码认证。密码与私钥口令只进入系统凭据库，不参与同步或导出。

GPUI 使用 russh 持久连接与 SFTP，平台设置/方案/账户、指令/扩展、MCP、Agent 会话和命令/安装均通过已捕获的环境权限路由。尚不支持的细分操作明确拒绝；不把安装记录当作客户端已加载。SFTP 写入保留版本比较、前像备份与原子替换；服务端必须支持 `posix-rename@openssh.com`。它不提供服务端原子 CAS，不能排除比较到 rename 之间的外部写入。重连后旧请求和草稿快照不得取得新会话权限。Tauri 的系统 OpenSSH 实现保持独立。WSL 作为可切换环境仍延期，设置不自动探测发行版；fixture 不连接真实主机。

平台页已接入固定路径下的设置字段补丁、原始版本源码编辑、Claude/Codex/Grok 账户与 OpenCode/Codex 供应商操作。用量与会话页通过共享应用层读取 Home/Usage 和归档会话，保留精确数值、过滤、分页与 provider fidelity。Codex OAuth 已接入共享控制器、手动回调和取消清理；用量导入已接入安装的 llmusage CLI、状态与取消，Codex quota 请求沿用现有服务并明确触发。Codex 账户页提供登录、添加／导入、配额和回调诊断四个入口。展开只显示任务内容，退出前可保留或丢弃草稿。三种配额目标和 OAuth 的回调、提交、取消均用隔离替身验证，Windows 原生窗口另行检查了按键和焦点。演示模式禁用真实授权与外部配额请求。

侧栏和 Claude Code、Codex、Grok、Antigravity、OpenCode 平台内的「用量」共享概览、明细与记录。概览默认最近 30 天，提供费用、Token、请求、活跃天数、上一等长周期和逐日精确值。明细包含排行与独立覆盖 366 天的请求热力图，记录可翻页和展开。平台内固定本平台。刷新只读取当前报表；同步使用独立的来源和日期范围，明确确认后才启动导入。Windows 合成导入的启动步骤见下方原生 fixture 说明。[用量重设计 #228](https://github.com/bahayonghang/ccr/issues/228)

`just gpui-dev --probe` 保留 **01 原生输入探针**，仅使用内存状态，不读取个人配置。
编译、headless interaction 与原生运行的证据分别记录在
[01 evidence](../.trellis/tasks/10-17-gpui-01-foundation/evidence.md) 和
[02 evidence](../.trellis/tasks/10-17-gpui-02-services/evidence.md)；外壳与配置集实施分别见 [03 evidence](../.trellis/tasks/10-17-gpui-03-shell/evidence.md) 和 [04 evidence](../.trellis/tasks/10-17-gpui-04-profiles/evidence.md)，未运行项不表示支持已验证。

## 开发

在仓库根运行：

```text
just gpui-help
just gpui-doctor
just gpui-dev
just gpui-build-debug
just gpui-build
just gpui-package
just gpui-ci
just gpui-test-ui
just gpui-audit
```

也可进入本目录，将根命令的 `gpui-` 前缀替换为 `desktop-` 后运行，例如 `just desktop-dev`。根只转发至子 justfile，确保 Cargo 从本目录加载
`.cargo/config.toml`。构建与测试使用 `--locked`；构建入口验证 Cargo metadata，
拒绝将 target 重定向到根 CLI、Tauri 或其他目录的环境覆盖。`gpui-clean` 再次验证 workspace / target 的绝对路径及链接目标，仅清理本目录 `target/`。

若官方状态与文件身份确认 PATH 中的 Cargo 是会自动迁移 target 的 mbx shim，
桌面命令仅在子进程中选择当前项目的 rustup Cargo，并配对同一工具链的 rustc/rustdoc。
这保留桌面本地 target，不修改全局 PATH 或 mbx 设置；普通 Cargo 继续原样使用。
无法确定 shim 身份时明确失败，避免静默改用共享目录。边界与真实退出码由 Python 回归覆盖。

`gpui-ci` 顺序执行 fmt-check、check、lint、默认并行 test、release build；
不会隐式执行 fmt 或 version-sync 修复。`gpui-fmt` 是单独的显式修复命令。

`gpui-dev` 将参数作为独立 argv 转发，fixture 路径包含空格时应按当前 shell 的规则引用。
`gpui-package` 生成当前主机的未签名便携 ZIP，输出到独立 `target/package/`。
它先在 `target/sidecar/` 构建同版本 CCR CLI，校验版本，再将 CLI SHA256 仅注入 GPUI
release 编译子进程；包内并排提供两个可执行文件、LICENSE 和双文件哈希清单。
这保留命令工作台的 release sidecar 身份校验，不依赖用户 PATH 中另一份 CCR。
该命令不生成安装器、不签名、不发布，也不代表其他操作系统的构建或原生验收已通过。
`gpui-test-ui` 使用 GPUI Kit 的真实控件和 headless window，不能替代操作系统
IME、屏幕阅读器、DPI 或原生截图验收。所有新入口在 Windows / Unix 都保留失败退出码。

## 平台先决条件

- Windows：MSVC C++ Build Tools、Windows SDK、DirectX 11 GPU / 驱动。
- macOS：Xcode command line tools、Metal GPU / 驱动、图形会话。
- Linux：C/C++ linker、pkg-config、fontconfig、freetype2、xkbcommon、xkbcommon-x11、Wayland / XCB
  开发库、Vulkan GPU / 驱动以及 X11 / Wayland 图形会话。

Linux doctor 同时检查 `xkbcommon` 和 `xkbcommon-x11` 的 pkg-config 开发入口；缺失时
显示错误并返回非零。Ubuntu 的后者由 `libxkbcommon-x11-dev` 提供
`xkbcommon-x11.pc` 和未版本化的 `libxkbcommon-x11.so` 链接入口，参见
[官方文件清单](https://packages.ubuntu.com/noble/amd64/libxkbcommon-x11-dev/filelist)。
仅有 runtime `libxkbcommon-x11.so.0` 不足以完成链接；doctor 不会自动安装开发包。

doctor 只读检查工具与 Cargo 路径；Windows 的链接器/SDK 由实际构建验证，GPU 必须
启动原生窗口验证。它不安装系统软件，不将打印的先决条件清单计为已验证。
项目遵循仓库 Rust 1.98.0 开发工具链，声明 MSRV 1.95；详见
[依赖选择与验证](dependencies.md)。

## 设计与验证

[DESIGN.md](DESIGN.md) 是已批准的 style:airy-glass 原生设计源；唯一 token 映射位于
`src/theme.rs`。侧栏与纸面是已知静态场景（渐变与两个静态光斑）上的半透明材质，成立条件见 `DESIGN.md` Elevation；阅读面保持实色，不使用窗口级 Mica/Blurred，也不宣称背景采样模糊。`src/shell/` 实现连续工作面、真实导航和状态。设置提供 System / Light / Dark 与中英文切换；主题、语言和退出确认保存在独立 `ccr/desktop-gpui/preferences.json`，主题调色板通过 Kit Theme 同步 Base，不建立第二主题 owner。

默认关闭仍需确认；设置可明确选择直接退出。设置按钮、应用菜单与 Ctrl/Cmd+Q 复用同一确认和清理路径。
直接调用平台应用退出的入口受锁定 GPUI 缺少公开 veto hook 限制，不能保证经过窗口确认。
托盘和 Windows 隐藏/恢复仍不受当前窗口栈支持；这些限制不计为验收通过。

按用户 2026-10-18 指令，本轮只验证核心导航、读取筛选、错误重试、取消/退出、中英文、主题和输入。窗口尺寸/缩放/DPI 矩阵明确 user-deferred，未计通过。原生中文 IME、系统字体 fallback、屏幕阅读器与其他 OS 仍分别以实际证据为准。
性能比较必须先采集同机 Tauri release 基线，再测 GPUI release；空探针不代表完整迁移性能。
提供上文的本机便携打包入口；签名、安装器和发布仍未提供。

## 安全 fixture 原生验证

仅对已经由 `ccr-config` API 创建的独立测试 catalog 使用以下参数；不把真实配置复制为测试数据，
不修改 `HOME`。路径须成对传入，profiles 参数传领域 API 的 legacy 路径，领域 owner 会读取其旁的
`profiles.db`。读取本身不创建或迁移 catalog。执行明确的编辑/应用动作会修改这些测试 catalog；辅助运行目录、锁、历史与其他平台 catalog 均限制在 profiles 所在目录下的 `desktop-profile-fixture/`。传入 registry 路径始终原样用于 marker；不要把真实配置路径传作 fixture。同目录通用配置备份位于声明的 profiles 父目录，fixture 的清理入口只由该已声明目录内部绑定，保留原有深度与扩展名规则；运行、锁和历史路径不随清理根改变。

```text
just gpui-dev --fixture-profiles <fixture>/profiles.toml --fixture-registry <fixture>/config.toml
just gpui-dev --fixture-profiles <fixture>/profiles.toml --fixture-registry <fixture>/config.toml --fixture-read-delay-ms 3000
```

fixture 窗口明确标注“演示数据”，慢读取额外标注“受控慢读取”。延迟范围为 0 至 30000ms，
缺少成对 fixture 路径时拒绝使用延迟参数，生产源不会受此参数影响。验证操作：

1. 读取期间在筛选框输入中英文并删除，输入立即呈现；改变筛选会取消旧读取订阅并增加 generation。
2. 读取完成后点击行或用列表焦点下的上下键选择，详情只展示脱敏凭据；刷新重新调用领域查询。
3. 空筛选、无匹配名称、缺少 catalog、格式错误分别检查 loading / empty / error 与重试；
   修复 fixture 后点刷新，不在产品端自动初始化或迁移。
4. 慢读取期间关闭窗口，订阅请求取消，已经开始的领域 I/O 完成后才结束任务。退出 hook 请求清理；
   GPUI 的 hook 只有 200ms 预算，event loop 返回后 runtime owner 仍会最终 join 全部已启动工作。

运行时通知每订阅容量 128，进度 100ms 合并，权威终态独立保留并优先读取，不依赖丢弃式进度通知。
最多接纳 128 个任务，最多 4 个 blocking worker。命令工作台复用共享命令策略和进程网关，保留每流最多500行/512KiB，原生日志按36px行虚拟化，单行显示不超过8KiB。离页后任务由应用持有，返回后自动恢复保留的快照和订阅；终态不会被迟到的运行状态覆盖。
这些机制和 headless 交互测试不等同于原生截图、三平台或性能验收。

观测域的独立数据根必须一次提供三个绝对路径：

```text
just gpui-dev --fixture-profiles <absolute-fixture>/profiles.toml --fixture-registry <absolute-fixture>/config.toml --insights-usage-root <absolute-usage-root> --insights-archive-db <absolute-archive.db> --insights-provider-home <absolute-provider-home>
```

缺少用量数据库时显示明确错误，不伪造零值或自动同步。读取与会话正文均经有界后台运行时；离开页面清除临时正文并拒绝过期结果。打开页面不自动启动用量导入、配额刷新或登录；这些操作需要明确触发，fixture 仍拒绝真实主机导入和外部服务。

原生观测 fixture 的三个路径须原样使用生成的 `fixture.json` 字段；不要重新拼接或移除 Windows `\\?\` 前缀，否则会改变来源身份。

原生同步验收可在上述参数后追加 `--fixture-usage-import cancel` 或
`--fixture-usage-import partial`。profiles 的父目录须位于当前源码 checkout 的
`outputs/desktop/` 下，目录中须有 `.ccr-gpui-usage-import-fixture`，内容为
`ccr-gpui-usage-import-fixture-v1`（UTF-8，无换行）。profiles、registry 和三个观测路径
也须在该目录内。菜单以「合成导入」标示演示；只在确认同步后启动，固定产生所选来源的
演示进度，不启动 llmusage 子进程、连接 SSH 或写入用量库和 Claude 工具记录。
`cancel` 可在运行后 30 秒内点「取消导入」，界面先显示等待清理，再显示已取消。
`partial` 报告新增 2 条和 1 个来源错误，供核对部分更新及报表条件保留。普通 fixture
仍禁用导入。

### 同步工作台的本地 WebDAV 夹具

从普通合成 catalog 创建独立副本，使同一目录内有 `config.toml`、`profiles.db` 和
`.ccr-gpui-sync-fixture`。该目录须位于当前 checkout 的 `outputs/desktop/` 下，
标记内容为 `ccr-gpui-sync-fixture-v1`（UTF-8，无换行）。将测试服务绑定到数值
loopback 地址，并将该地址写入 fixture 的规范文件 `sync_folders.toml`。

```text
just gpui-dev --fixture-sync-loopback --fixture-profiles <fixture>/profiles.toml --fixture-registry <fixture>/config.toml
```

该选项只在启用了 `native-sync-fixture` 的开发构建中可用，`just gpui-dev` 会按参数选择构建。
它使用现有测试用 WebDAV 客户端，拒绝外部地址，禁用代理和重定向，保持普通演示及 SSH
的网络准入规则。空配置首次保存仍遵循正式 HTTPS 校验。需要检查 loopback 时，从应用外
写入合成设置后用「重读」载入，再显式测试连接。记录服务器请求数以检查进入、重读和保存
不触发连接。


指令文件入口已接入四个平台的固定用户文件，使用原始修订令牌保存、冲突保留草稿、离页清空临时明文。MCP 管理支持 Claude 用户/项目、Codex 和 Antigravity 的新增、编辑和删除，保存固定来源与原始版本，省略的凭据不被覆盖。OpenCode MCP 使用平台源文件编辑器；旧 Claude settings MCP 只读。

同步工作台先显示当前执行环境及 WebDAV 状态。本地重读和资产盘点与显式远端探测独立。五项配置资产可按单项或全部选择方向，核对目标与后果后再确认。敏感新写入使用 v2，只有显式允许且实际下载时才可导入明文 v1。普通文件夹按所选或全部已启用项独立传输。最近操作按执行环境恢复，显示逐项结果和可读取的备份。[同步 Windows 原生验收](../.trellis/research/archive/2026-11/gpui-sync-234/native.md)

签到页提供服务商/账户管理、签到、余额和记录，凭据通过现有加密 owner 保存。任务离页后继续由应用持有，返回恢复结果；已接显式 CDK 充值、手工 WAF 管理/校验、记录筛选与账户月度数据；内嵌浏览器 WAF 登录仍受 WebView 依赖限制。

SSH 同步读取该主机的 `~/.ccr/sync_folders.toml`，四个固定文件资产（Claude settings / CLAUDE.md、Codex config / AGENTS.md）通过私有暂存复用 `ccr-sync` 的 HTTPS、冲突和加密协议，再以 SFTP 版本比较、备份、原子替换写回该主机。普通文件夹限定在捕获的远程 HOME 内：portable 路径、最多 128 节点/12 层、每文件 2MiB/总计 16MiB。多文件发布保留已写文件的远端备份，失败会报告部分完成；不承诺跨文件事务。远端 `ccr-platforms` 以及包含 SQLite 数据库或 sidecar 的文件夹明确拒绝，缺少远端数据库事务 owner 时不覆盖活跃 `profiles.db`。SSH 下解绑 WebDAV/清理旧凭据尚不可用；新建纯空目标目录不支持。空来源目录可上传。以上限制不改变本地同步能力。

SSH 用量显式调用该主机已安装的 `llmusage --home <远程HOME>/.llmusage sync --json-events`，缺 CLI 不运行本机工具。报告通过远端 Python3 标准 `sqlite3` 的只读 backup API 捕获 `~/.llmusage/llmusage.db`（包括已提交 WAL），经 SFTP 下载到按 host ID 隔离的本机私有临时缓存，再由 `ccr-usage` 只读投影。需要远端 Python3 带 sqlite3，根目录与源文件不得为符号链接。快照上限 64MiB，backup helper 最多 10 秒，下载绝对期限 10 分钟；取消后只等在途 SFTP 5 秒，无法证明清理时撤销原会话并报告失败。成功或失败都尝试精确清理远端快照，清理失败不发布缓存，可能需要在主机上手动清理残留 `.ccr-snapshot-*` 目录。本机临时缓存随请求释放。原配置与 Agent 会话预算不变。

已发出的私有 backup helper 在原 10 秒期限内结束，以便取得精确清理路径；页面离开取消后不再下载或发布缓存，清理完成再确认取消，不因正常导航撤销共享 SSH 连接。尚未开始时取消不执行 helper。

用量 schema 最低 10，提供商能力最低 14，兼容满足能力表的未来增量 schema；CCR 不迁移该数据库、不链接上游 llmusage Rust crate。NDJSON 使用原协议解析器与单行/尾部预算；`finished` 只是暂态，必须等远端 exec 和双流终态。远端执行取消或断线不能证明子进程树退出时报告 cleanup failure，不提前释放导入准入。未知用量与会话计数不显示为零，也不使用本机归档/Observer 填补。真实 SSH、系统凭据库交互、远端 Python/llmusage 版本、服务端原子 rename 与跨平台行为仍需真机验证。

命令工作台支持前台/后台执行、help、请求取消和自动恢复观察；安装入口复用现有领域服务。fixture 模式只运行标明的合成子进程，默认拒绝同步、签到与 SSH 的真实网络。上述源码与 fixture 验证不能替代真实 sidecar、外部服务或原生操作验收。

## 显式成对性能数据源

生成的 paired seed 使用单独的 `--benchmark-fixture-root <absolute-pair>/<S0|S1|S2>/gpui`，
与 `--probe`、旧 fixture 参数及三个 insights 参数互斥。该参数只加载已经生成的种子，
不会创建数据。旧 Fixture 的目录布局和合成子进程合同保持独立。

启动在创建窗口之前由有界 worker 验证 marker、成对 manifest、来源身份、目录边界和
四个数据库的 artifact hash，再冻结统一的 home/CCR/主库/归档/用量/锁/偏好根。
窗口标题包含 `Fixture benchmark-s0/s1/s2`。合法的本地配置操作仍可写入该根；
数据库输入一旦被操作改变，下次启动会拒绝原始 seed 身份，需恢复原始试验输入。
此模式拒绝 CLI 导入/命令执行、安装、OAuth、quota、SSH 和外部同步/签到请求。

manifest 的 UTC `as_of` 与时区通过共享 `ReportClock` 接入两端默认首页、用量预设、
Observer、最近 24 小时及滚动热图。显式自选日期/时区保留原值；正常 Local 和旧 Fixture
沿用既有时钟。Tauri 在共用窗口 builder 中仅给 Benchmark 注入只读报告上下文，
没有改写全局 Date、系统时间或真实 elapsed/timeout 时钟。

两端显式 Benchmark 启动复用 `ccr-app::benchmark::Seed`，同样遍历目录并完整 hash
四个数据库；非空 WAL/journal 拒绝加载，需先恢复静态原始输入。这些校验成本包含在
真实启动内，不从呈现时间中扣除。相同校验策略和 headless 数据读回不代表真实呈现或
预算已达标；两端目前仍为 `not_comparable`。
S2 的持续流量仅有 `not_emitted` 描述，尚无运行时 emitter；没有合格性能样本。
无窗口领域加载测试不代表原生呈现或真实 first-present 已通过。

## 核心键盘与命令记录

- F6 顺序切换侧栏、主区、状态区；Shift+F6 反向切换。侧栏上下键选择焦点，Enter/Space 激活。Tab/Shift+Tab 保留 Kit 原生遍历。
- 配置列表上下键选行，Enter 打开详情，Escape 或“返回配置列表”返回并保留稳定配置名称。文本输入焦点下不拦截编辑方向键。
- 离开配置或命令记录页面会取消对应读取订阅，过期结果不能覆盖新页面。收藏写任务由应用持有，离开页面不会伪造取消或停止已开始的持久化。
- 收藏表单的参数每行一个，仅保存数据，不经 shell 解析或执行。新建收藏不写入最近执行历史。
- Fixture 的命令记录写入 registry 同级 `ui-state.db`，迁移只使用同级空 `ui-state-migration-home/`；生产通过既有数据库 pool owner，不能用 CCR_ROOT 假定隔离此数据库。

按钮保留 Kit 自身语义节点/动作，应用在同一节点补 disabled/busy 标记；禁用文字仍使用可读的语义色。只对实际可编辑输入使用原编辑器，未复制或改写 IME。Kit observation snapshot 在 synthetic callback 前采样，public debug tree 不导出 disabled/busy，均不能代替原生 UIA/screen-reader 状态验证。

资源页提供 Claude 文件与 settings 集合管理、Codex 本地与来源 agents，以及填充指令草稿的模板。MCP 支持单项预设和兼容源复制，秘密保留在后端会话中。窗口偏好在同一窗口恢复、观察并于关闭前保存；系统直接退出与托盘限制不因此解除。

OpenCode 与 Antigravity 资源使用原有全局/项目目录；插件/扩展只提供既有只读清单。Usage 页可直接进入预算与定价，无须先建立用量数据库。签到导入预检当前密钥与原版本，配置导出不等于包含历史和额外账户设置的全库备份。

侧栏和平台 tab 共用同一用量报表。默认统计报告时区的今天和此前 29 个日历日，可选择时间、时区、来源、模型、提供商与项目。平台 tab 固定本平台。概览显示 llmusage 的 USD 估算、Token、请求与有请求的天数，并在相同条件下对比紧邻的等长上一周期。含今天的区间注明今日尚未结束。全历史、上期无记录、读取失败、基数为零或费用估算覆盖不完整时显示原因。日期或高级条件应用后才改变报告。同步用量单独选择导入范围，提供进度和取消，离页后仍由应用持有。

用量概览默认在全部来源下显示来源排行，在单一来源下显示模型排行。明细提供完整排行和独立滚动 366 天的活动热力图。记录沿用报表筛选，可分页、核对定价细节并返回表格。切换筛选或执行环境时从第一页读取。

Claude 观测与订阅设置使用独立 CCR 主库，打开页面不建库，首次明确保存可初始化，订阅设置不依赖用量数据库。Sessions 提供 Codex Markdown 导出预览、复制、克隆和可恢复回收；回收使用 CLI 的 `ccr codex sessions restore` 恢复机制。

平台页补充认证关闭、Codex 账户重命名/API key/导入、模型供应商库、模型目录与配额查询。Grok 关闭认证保留原服务的轮换凭据保存语义；配额查询可能刷新凭据并保存观测结果，离页不冒充取消。同步页可解绑 WebDAV、批量传输已启用文件夹并分别显示结果和下载备份；保护备份可能仍保存旧凭据。本轮 Windows 原生覆盖见[配置方案与账户验收记录](../.trellis/research/archive/2026-11/gpui-profiles-auth-220/native.md)。真实 OAuth 服务、生产配额与 WebDAV 传输未在本轮验证。
