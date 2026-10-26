# CCR Desktop (GPUI)

独立原生桌面客户端，Cargo package / bin 为 `ccr-gpui`，窗口标题为
`CCR Desktop (GPUI)`，应用身份为 `io.github.ccr.gpui`。它使用自己的 workspace、
Cargo.lock 和 `target/`；根 CLI 与现有 Tauri 命令继续独立构建。

当前默认界面接入原生外壳与 Local 的 Claude / Codex / Grok 配置集。读取、筛选、校验和写入都由有界后台 worker 调用现有领域 owner；列表按稳定名称虚拟化，凭据显示通过领域脱敏。结构化编辑保留读取时的 revision，省略字段表示保留，明确 null 表示清除；冲突保留草稿，只能重读或取消。

Claude 页提供“配置集 / 配置目录”操作模式，共享列表并保留各自校验与删除策略。配置目录还提供复制、受管备份恢复、导入、脱敏导出和操作历史；配置集提供明确警告后的规范明文源码/导出。Codex 支持受控环境变量与脚本导出预览。Grok 保留分阶段改名结果和激活/漂移/入口状态缺失的删除保护，强制停用并删除需要单独确认，入口状态缺失不提供强制。

源码和导出文本仅在当前 feature 的临时实体中保存，取消或离开页面即清除；离开不把已开始的持久写入伪报为取消。转换页只预览现有 Claude Code JSON 与 Codex TOML 转换，不写文件或执行脚本。其他格式和远程环境未接入。收藏/最近命令继续由 ccr-db owner 管理，未制造命令执行记录。

`just desktop-dev --probe` 保留 **01 原生输入探针**，仅使用内存状态，不读取个人配置。
编译、headless interaction 与原生运行的证据分别记录在
[01 evidence](../.trellis/tasks/10-17-gpui-01-foundation/evidence.md) 和
[02 evidence](../.trellis/tasks/10-17-gpui-02-services/evidence.md)；外壳与配置集实施分别见 [03 evidence](../.trellis/tasks/10-17-gpui-03-shell/evidence.md) 和 [04 evidence](../.trellis/tasks/10-17-gpui-04-profiles/evidence.md)，未运行项不表示支持已验证。

## 开发

在仓库根运行：

```text
just desktop-help
just desktop-doctor
just desktop-dev
just desktop-build-debug
just desktop-build
just desktop-ci
just desktop-test-ui
just desktop-audit
```

也可进入本目录运行同名命令。根只转发至子 justfile，确保 Cargo 从本目录加载
`.cargo/config.toml`。构建与测试使用 `--locked`；构建入口验证 Cargo metadata，
拒绝将 target 重定向到根 CLI、Tauri 或其他目录的环境覆盖。`desktop-clean` 再次验证
workspace / target 的绝对路径及链接目标，仅清理本目录 `target/`。

若官方状态与文件身份确认 PATH 中的 Cargo 是会自动迁移 target 的 mbx shim，
桌面命令仅在子进程中选择当前项目的 rustup Cargo，并配对同一工具链的 rustc/rustdoc。
这保留桌面本地 target，不修改全局 PATH 或 mbx 设置；普通 Cargo 继续原样使用。
无法确定 shim 身份时明确失败，避免静默改用共享目录。边界与真实退出码由 Python 回归覆盖。

`desktop-ci` 顺序执行 fmt-check、check、lint、默认并行 test、release build；
不会隐式执行 fmt 或 version-sync 修复。`desktop-fmt` 是单独的显式修复命令。
`desktop-test-ui` 使用 GPUI Kit 的真实控件和 headless window，不能替代操作系统
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

[DESIGN.md](DESIGN.md) 是已批准的 style:codex 原生设计源；唯一 token 映射位于
`src/theme.rs`，`src/shell/` 实现连续工作面、真实导航和状态。设置提供 System / Light / Dark 与中英文切换；偏好仅在当前窗口会话生效，主题调色板通过 Kit Theme 同步 Base，不建立第二主题 owner。

按用户 2026-10-18 指令，本轮只验证核心导航、读取筛选、错误重试、取消/退出、中英文、主题和输入。窗口尺寸/缩放/DPI 矩阵明确 user-deferred，未计通过。原生中文 IME、系统字体 fallback、屏幕阅读器与其他 OS 仍分别以实际证据为准。
性能比较必须先采集同机 Tauri release 基线，再测 GPUI release；空探针不代表完整迁移性能。
本阶段没有打包、签名、安装或发布入口。

## 安全 fixture 原生验证

仅对已经由 `ccr-config` API 创建的独立测试 catalog 使用以下参数；不把真实配置复制为测试数据，
不修改 `HOME`。路径须成对传入，profiles 参数传领域 API 的 legacy 路径，领域 owner 会读取其旁的
`profiles.db`。读取本身不创建或迁移 catalog。执行明确的编辑/应用动作会修改这些测试 catalog；辅助运行目录、锁、历史与其他平台 catalog 均限制在 profiles 所在目录下的 `desktop-profile-fixture/`。传入 registry 路径始终原样用于 marker；不要把真实配置路径传作 fixture。同目录通用配置备份位于声明的 profiles 父目录，fixture 的清理入口只由该已声明目录内部绑定，保留原有深度与扩展名规则；运行、锁和历史路径不随清理根改变。

```text
just desktop-dev --fixture-profiles <fixture>/profiles.toml --fixture-registry <fixture>/config.toml
just desktop-dev --fixture-profiles <fixture>/profiles.toml --fixture-registry <fixture>/config.toml --fixture-read-delay-ms 3000
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
最多接纳 128 个任务，最多 4 个 blocking worker。当前没有日志传输，日志显示上限由后续实际日志域接入。
这些机制和 headless 交互测试不等同于原生截图、三平台或性能验收。

## 核心键盘与命令记录

- F6 顺序切换侧栏、主区、状态区；Shift+F6 反向切换。侧栏上下键选择焦点，Enter/Space 激活。Tab/Shift+Tab 保留 Kit 原生遍历。
- 配置列表上下键选行，Enter 打开详情，Escape 或“返回配置列表”返回并保留稳定配置名称。文本输入焦点下不拦截编辑方向键。
- 离开配置或命令记录页面会取消对应读取订阅，过期结果不能覆盖新页面。收藏写任务由应用持有，离开页面不会伪造取消或停止已开始的持久化。
- 收藏表单的参数每行一个，仅保存数据，不经 shell 解析或执行。新建收藏不写入最近执行历史。
- Fixture 的命令记录写入 registry 同级 `ui-state.db`，迁移只使用同级空 `ui-state-migration-home/`；生产通过既有数据库 pool owner，不能用 CCR_ROOT 假定隔离此数据库。

按钮保留 Kit 自身语义节点/动作，应用在同一节点补 disabled/busy 标记；禁用文字仍使用可读的语义色。只对实际可编辑输入使用原编辑器，未复制或改写 IME。Kit observation snapshot 在 synthetic callback 前采样，public debug tree 不导出 disabled/busy，均不能代替原生 UIA/screen-reader 状态验证。
