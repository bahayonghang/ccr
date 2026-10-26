# GPUI 暂停点与下一对话续作契约

记录日期：2026-10-18。用户明确说明主动按 Escape，并要求暂停当前任务、提供下个对话的续作 Prompt。**当前暂停，不继续实施或 Computer Use。** 原父任务及十个子任务的实施批准仍有效，后续明确要求继续时恢复，无需重新批准方案。

## 当前核实状态

- 工作区 `D:\Documents\Code\Github\ccr`，分支 `dev`，HEAD `1dfe776b`。`get_context.py` 报告68项未提交路径；包括此前01–04及既有修改。没有自动提交、push、发布或归档。
- session active task 是 `.trellis/tasks/10-17-gpui-04-profiles`。父任务及01/03/04的 task status 为 `in_progress`；02为 `completed`；05–10为 `planning`。暂停单独记录在 metadata/evidence，不伪造完成状态。
- 01独立工程可用，02共享服务与核心原生验收已完成。03源码、独立检查和用户指定核心原生流程已通过；UIA/IME、其他平台、性能等开放项继续保留，不能宣称03全部完成。
- 04配置/Profile实现、两个独立check已完成并冻结。42命令/5路由有实际代码映射，但尚未做04最终原生验收及逐项证据整合，不能标04完成。
- 05–10尚未启动；05、06有可复用的前置研究，不必重新做同一份广泛调查。
- 暂停检查时 `Get-Process -Name cargo,rustc,ccr-gpui` 无结果；本会话子agent均已完成，无后台实现或检查工作。

## 最新构建和验证

- **最终 debug 构建已经成功，不再是“仍在运行”。** `just desktop-build-debug` 缓存访问重试退出0，完成时间 `2026-10-18T07:20:22.4307061+00:00`；这等于 America/New_York 的2026-10-18 03:20:22。
- 最初 sandbox 尝试退出1，是既有 mbx shim metadata不可访问；使用获准的缓存访问重试成功，没有修改全局缓存配置。两次原始日志分别保留。
- 二进制 `ccr-desktop/target/debug/ccr-gpui.exe` SHA256：`2d7b4271c07dbf0e14fd301ee3df7b6f8ba73996ac570eacf8b947645a606e26`。本次暂停又实际核对一致。
- 构建凭据：`outputs/desktop/profiles/native-build-final-cache-access.json` 与同名 `.log`。
- native最终fmt/check/strictlint/locked全Rust测试/Rust1.95全目标检查均0；库15、native_shell3、native_ui2通过。Python12/AccessKit3在未改动支持层的上次完整desktop-test通过。报告 `research/native-check-final.md`；原始命令/退出码 `outputs/desktop/profiles/native-check-accepted-gates.json`，冻结文件hash `native-check-freeze.json`。
- 领域/shared/Tauri独立检查报告 `research/check-final.md`；冻结receipt `outputs/desktop/profiles/check-frozen-receipt.json`。实际修复并复验Grok stale revision/rename TOML保真、Codex off补偿和首次创建catalog事务发布等问题。
- 根 `just version-check`、`just fmt-check`、`just lint-strict`、`just test` 均0。完整根测试2258通过、0失败、26忽略、33过滤、48 suites，保留默认并行与 `--skip export_bindings`。见 `root-test-summary.json`、`root-gates-receipt.json`、对应原始logs。
- **`just ci` 未运行；完整三平台、独立打包、Tauri/GPUI性能比较未完成。** Tauri release编译不等于安全完整应用基线，不能用空窗口/debug/受控延迟代替性能结果。

## 明确保留的问题

1. bindings checker曾实际出现11个生成文件漂移，后续三次通过但根因未确定；不是“已修复”。230原始生成文件与三份lock全部恢复且hash一致。失败日志、tagged后续凭据和方法在 `research/check-final.md`；不要覆盖失败、静默rebaseline或盲目重复。
2. 04台账仍待整合。父 `research/migration-inventory.json` 原始348命令/76路由记录/18重定向必须保留，不能重跑 capture_inventory.py 覆盖实施证据。按钮存在/headless通过不等于所有命令逐项原生通过。
3. `code_map.md` 还把desktop描述成只读；04验收整合时按真实结果窄更新，暂停时未改产品或该文档。
4. 01/03其他平台、UIA/IME、性能等开放项不能因04继续而消失。用户已明确只做核心流程，不做缩放/DPI/窗口尺寸矩阵，本轮及下一步不得重启这些检查；这不是自动满足原父AC。
5. 现有 `C:\Users\lyh\.codex\RTK.md` 引用实际不存在，已核实；不要编造已读或因此阻止无关续作。

## 下一步：先做04最终核心原生验收

在新的用户继续指令后恢复。先读本文、04自身prd/design/implement及implement.jsonl/check.jsonl、两份check-final、ui-evidence和父文档。通过正常 `task.py start .trellis/tasks/10-17-gpui-04-profiles` 绑定新会话，核对当前git/源码freeze/hash；未变源码复用已有门禁，不无故重跑全部测试。源码或exe变化则重新构建并记录新hash。

04尚未启动过最终GUI，之前02/03图片不可冒充04证据。恢复调用 `sky.list_windows` 时返回明确的“用户按物理Escape停止”信号，主会话立即停止；用户随后确认是主动停止。不是窗口最小化、权限批准失败或GPUI产品崩溃，也没有04输入/截图/流程通过。

Computer Use必须先加载 `computer-use:computer-use` 技能及其guidance/api/confirmations，经 `@oai/sky` 的 `mcp__node_repl__js` 操作实际Windows原生窗口。`mcp__cua_repl` native API在此环境禁用，不用它替代。新对话先初始化全新选择并唯一匹配实际 `CCR Desktop (GPUI)` 窗口，不能沿用旧句柄、坐标和截图。若再次收到停止信号，立即停，不reset绕过。用户继续授权不是绕过实际停止保护。

### 已准备且实际预检过的fixture工具

- `outputs/desktop/profiles/launch-native-fixture.ps1`：每次 `-RunName <新名称>` 创建全新私有fixture，拒绝覆盖；可选 `-MissingCatalog` 或 `-DelayMs 30000`。传显式profiles/registry配对路径，不修改HOME。保留实际PID、exe hash、stdout/stderr、原生退出码 `native/<run>/receipt.json`。
- `outputs/desktop/profiles/native-seed` 是已关闭的领域生成synthetic数据库/registry/manifest快照，不要覆盖；每次只用新副本。正常fixture有 `a-first` 和 `z-last`，无需真实账户、网络或个人配置。
- 真正第二客户端CAS helper：`outputs/desktop/profiles/services/helper-artifact.json` 指定预编译testexe，直接传 `--exact mutate_native_profile_fixture --ignored --nocapture`。仅子进程环境传 `CCR_APP_PROFILE_MUTATION_ROOT`、`CCR_APP_PROFILE_MUTATION_PROFILES`、`CCR_APP_PROFILE_MUTATION_NAME`，分别为新run的fixture根、逻辑profiles.toml、a-first或z-last。helper验证私有root/marker/路径，使用ConfigManager和与native一致的锁目录，不手写SQL、不复制活跃DB。调用前重新hash exe；预检凭据 `helper-preflight-receipt.json` 可能是根测试重编译前hash，不能挪用为新hash。
- GUI运行时不要运行desktop Cargo：Windows锁定exe，曾有AccessDenied失败；预先构建完成再开窗口。这里是防文件冲突，不是降低测试要求。

### 核心流程

1. 正常新fixture：实际读取/当前标记/脱敏、筛选；编辑description保存并重读；再建立dirty draft，用上述第二客户端提交，实际Save应冲突且保留draft，Cancel/Refresh后显示已提交值。应用 `z-last`（初始a-first已当前）并核对实际当前状态/typed结果。不要打开含plaintext的Inspect/Source/export截图，不必通过UI删除测试数据。
2. 缺catalog新fixture：记录真实错误，向该run专属missing目录复制已关闭seed DB后点Retry，核对真实行恢复，再正常退出；不复制正在使用的catalog。
3. 受控30秒慢读新fixture：确认Loading后在延迟自然结束之前关闭，结合实际receipt寿命 `<30s` 与清理/无残留证明取消。超过30秒只记不确定，不能冒称取消通过。这不是性能样本。
4. 每次保存真正GPUI截图、实际退出码、stderr及fixture状态/hash；最终确认没有残留进程。按新技能文档逐步观察/动作/刷新，避免在其他前台应用上操作。

若 `type_text` 未生效，前次同exe probe和业务输入均出现过该工具差异；实际 `press_key` 可工作。先观察焦点/实际文本，不盲目改Kit输入实现或声称中文IME通过。UIA tree前次为null，不把内部语义测试当OS可访问性通过。正常读取/编辑/保存/取消/退出无需重复实施批准；Windows UI删除等动作仍按技能实际确认策略执行，可使用领域fixture测试覆盖以避免无必要的破坏动作。

04验收整合后：写 `research/native-profiles.md`、core-summary及evidence，按每项实际证据补台账native_action/verification/status，保留开放项；消费独立checker结果，需要修复则经trellis-implement/check闭环。不得仅凭全部按钮或编译就勾全AC。

## 后续顺序和已有研究

04 → `10-17-gpui-05-platforms` → 06resources → 07insights → 08jobs → 09environments → 10acceptance。按各任务已批准依赖和父implement.md的可消费工程/开放验收区分推进，遇外部阻塞记录并继续不依赖它的工作。

- 05预研：`../10-17-gpui-03-shell/research/platforms-source-map.md`，54命令/8路由。已纠正一个关键区别：`auth_off_for_platform(Codex)` 在Keyring/Auto可以调用官方 `codex logout`，`codex_local_auth_off` 则只支持File。保留实际调用方差异，不把本地退出改成官方撤销或反过来；测试用显式路径/假进程，不操作真实账户。
- 06预研：`research/resources-source-map.md`（80命令/38路由=24surface+14redirect）及 `resources-preflight.md`。记录了现有agent/outputstyle路径验证和Skills shell拼接/取消边界，需要在真实领域owner修复后才能暴露原生动作，不复制安全逻辑。Skillport与Skills当前能力按原研究说明，别用占位页面虚报完整CRUD。

## 所有续作必须保留的执行契约

- 遵循AGENTS.md、code_map、Trellis；加载start/continue/before-dev，UI应用ui/design-md及已批准 `ccr-desktop/DESIGN.md`。不用重新brainstorm已批准范围。
- 主会话协调；implement/check通过对应Trellis sub-agent角色。每次派发首行 `Active task: <正常task.py current返回路径>`，明确唯一负责文件/行为/AC/命令及curated上下文；共享工作区不得回退他人修改，共享文件不能两个当前写入owner。子agent不递归派发implement/check。
- 每个新子任务先读自身五份文档/清单并正常激活；父任务负责整合，不一次性重写全模块。
- `ccr-desktop/` 独立workspace/lock/target，package/bin `ccr-gpui`；根仅新增 `desktop-*`，不改旧CLI/ui/tauri语义、不把GPUI拉进根默认图、不删Tauri。
- 复用领域crates，framework-free共享层不依赖tauri/gpui，ccr-usage仍为usage SQL唯一owner；不链接上游llmusage Rust crate。不复制认证/profile/进程安全/数据库逻辑。
- 保留backup/masking/CAS/atomic-write/命令策略/SSH trust/取消清理。只用私有fixtures，不改真实用户配置/账户，不全局改HOME。
- 依赖新增/变动时按实施当日核对兼容稳定版本、MSRV/features/来源/例外、提交应有lock文件内容并 `--locked` 验证；不是每次会话为了“最新”无故升级已冻结的依赖。
- 不在GPUI主线程做同步I/O，保留有界队列、虚拟列表、局部更新、日志上限、过期响应丢弃。
- 根测试默认并行与 `--skip export_bindings`；fmt/version-sync修复与验证分离；不降低覆盖率阈值、不改hosted required context。
- 性能需安全完整Tauri release基线，再同机同数据同操作GPUI，保留原始样本/环境/版本/数据hash；失败先修，不放宽预算。完整平台/打包/just ci未满足不能宣称父AC1–AC8完成。
- 不自动commit、push、发布、用签名密钥或归档未完成任务；Pause不是Finish，不执行会自动归档/提交的finish-work默认动作。

## 可直接发送到新对话的Prompt

```text
在 D:\Documents\Code\Github\ccr 恢复已批准的GPUI桌面迁移。先完整阅读 .trellis/tasks/10-17-gpui-04-profiles/pause-handoff.md，并按其状态和执行契约继续，不重新立项或请求实施批准。

这是对上一会话暂停的明确恢复。上次Escape是我主动按下的；当时没有启动04原生测试窗口。现在可以恢复Computer Use；若工具再次报告用户停止则立即尊重停止，不绕过保护。

先核对git status、Trellis当前任务、进程、源码freeze和最终exe hash。02已完成；01/03的开放验收继续跟踪，不重复施工。04实现和两次独立检查已完成，最终debug build已成功，尚缺最终核心原生验收和42命令/5路由台账整合。没有相关源码变化时复用已有通过门禁，不重跑无关检查。

按交接文档使用已准备的私有fixture launcher和真实第二客户端CAS helper，先完成04的配置读取、筛选、编辑保存重读、冲突处理、应用状态、错误重试、取消和正常退出，记录真正GPUI截图/退出码/日志与数据状态。只测核心流程，不做缩放、DPI或窗口尺寸矩阵，不碰真实用户配置/账户，不全局修改HOME。不得把未运行项记通过。

04验收及evidence/inventory整合后，按依赖顺序继续05平台设置认证、06资源、07观测、08任务、09环境与系统集成、10最终验收。每个子任务先读自身prd/design/implement/implement.jsonl/check.jsonl并用Trellis正常命令激活。复用05/06已有source-map，保留bindings偶发漂移、其他平台/IME/UIA/性能/打包等开放项；有外部阻塞准确记录并继续独立工作，不宣称整个迁移完成。

遵循AGENTS.md和Trellis sub-agent模式，主会话协调trellis-implement/trellis-check；派发首行Active task，明确唯一文件owner、范围、验收和验证命令，加载curated上下文，告知共享工作区不得回退他人修改。保留ccr-gpui独立workspace/lock/target、desktop-*边界、领域owner、备份脱敏CAS原子写和取消语义，Rust默认并行与--skip export_bindings，使用--locked验证且不隐式修复fmt/version-sync。

不要只完成脚手架或单个子任务就结束；继续完成已批准后续工作。不自动提交、push、发布、使用签名密钥或归档未完成任务。最终按实际证据报告范围、产物和阻塞。
```
