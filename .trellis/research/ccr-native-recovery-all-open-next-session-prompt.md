# 下一对话 Prompt：恢复原生通道并完成 CCR 全部未关闭票

将下文整段交给下一对话执行。

---

请在 `D:\Documents\Code\Github\ccr` 继续已有生产实施，使用 `$wayfinder` 衔接现有地图与决策。目标是：**定位并修复阻碍操作的 Computer Use 原生通道故障，完成真实原生验收，继续处理整个 CCR 仓库全部未关闭 issue，直到逐票满足实现、回归、必需验收和交付条件。**

这是执行续做，不是重新规划。不要重新选择原型、重走设计地图，也不要以检查报告、journal、构建成功或重复的 `os error 2` 回包作为任务完成。

## 1. 保留工作区并读取实际交接

开始时简述执行顺序，然后直接推进，不逐阶段等待批准。

- 先核对 `git status`、diff、HEAD 和实际 release 身份。上一轮交付时为 `dev`，HEAD `4696c36c5315f92f62afa6129c38889fd419e575`，106 个变更路径，暂存区为空；加上新建的本 Prompt 为 107 个，以实际状态为准。此前已有 65 个变更路径，上轮起点为 68 个，不能全部算作本次新增。
- 不 reset、checkout、clean、stash、覆盖已有改动，不使用 `git add -A`。默认仍不 commit、push、发布或上传截图；“处理所有问题”不自动扩大为这些授权。
- 根/Tauri 两份安全 lockfile、桌面原有 Cargo.toml/Cargo.lock、CONTEXT、pastedcontent 的保留结果见最后的 preservation 收据。journal 已到 Session 169；保留已有字节前缀和 index 旧会话记录。
- 依次读根 `AGENTS.md`、`CLAUDE.md`、`code_map.md` 及适用目录指令；`docs/agents/issue-tracker.md`；`.agents/skills/wayfinder/SKILL.md`；桌面 `DESIGN.md`、`CONTEXT.md`；根与桌面 justfile；相关共享服务、配置、用量投影、任务生命周期和夹具合同。
- 读取 `.trellis/research/gpui-claude-redesign-161-implementation.md`，特别是 Session 169 全仓续做和最终组合验证；读取 `.trellis/workspace/lyh/journal-3.md` 的 Session 166–169。
- 本地证据在 `ccr-desktop/outputs/claude-code-redesign/implementation/`，最新一轮在其 `related-closeout/`：`commands-final.json`、`ci-final.log`、`desktop-ci.log`、`build-identity.json`、`validation-inputs.json`、`native-recheck.json`、`preservation-final.json`、`open-issues-final.json`。保留首次失败日志，不能与最终结果混用。
- 已批准原型仍在 `ccr-desktop/outputs/claude-code-redesign/`；初始 diff 是 `implementation/baseline.patch`。忽略目录文件缺失时按 PRD、合同和源码继续，不因此停止。

## 2. 全仓范围与 Wayfinder 边界

重新查询整个 `bahayonghang/ccr` 的 open issues，读取完整正文、评论、标签和依赖关系。交接时只有七张，按实际状态继续：

- [GPUI Claude Code 页面：修复违反 DESIGN.md 的 7 处缺陷](https://github.com/bahayonghang/ccr/issues/150)
- [用量新鲜度漏算 sync --rebuild，「上次成功同步」早于实际](https://github.com/bahayonghang/ccr/issues/151)
- [Claude 洞察把「项目未归属」显示成「0 个项目」](https://github.com/bahayonghang/ccr/issues/152)
- [Claude 工具与会话的费用列恒为假 0](https://github.com/bahayonghang/ccr/issues/153)
- [Claude 工具调用热力图按 UTC 分桶，窗口与时间戳也有偏差](https://github.com/bahayonghang/ccr/issues/154)
- [规则、输出样式和用户指令文件不跟随 CLAUDE_CONFIG_DIR](https://github.com/bahayonghang/ccr/issues/156)
- [PRD：GPUI Claude Code 页面重设计](https://github.com/bahayonghang/ccr/issues/161)，包括 DESIGN 改写附录。

已有 [GPUI Claude Code 页面重设计地图](https://github.com/bahayonghang/ccr/issues/138) 和设计决策票保持关闭；它们不构成实现或原生验收完成的证据。不要新建重复地图。资源发现研究票保持研究结论，不重开为实施票。

本 Prompt 明确延续全部 open issue 的实施授权，不套用 Wayfinder 默认“只规划、每会话一票”来中断这项已有执行任务。只有实际未决、会改变已批准设计或权限的决定才转给我；不代答 HITL，不擅自扩大产品功能。叙述用票名和链接，避免裸编号堆砌。

## 3. 首要任务：真正处理原生通道故障

上一轮错误是：

```text
Computer Use native pipe is unavailable: failed to connect native pipe:
The system cannot find the file specified. (os error 2)
Browsers: Error: The browser access host is unavailable
```

当时 `cua.getState()` 返回 apps/browsers 空清单和上述错误，`js_reset` 后重新初始化仍相同。**重置 JavaScript kernel 只是一项尝试，不能据此认定宿主或管道已经被恢复。** 当时没有启动夹具 GUI，没有新增原生截图、外窗/客户区尺寸或 DPI 观测。

此次我明确要求继续定位并修复这些阻塞。因此，旧的“有界恢复一次后停止排障”不再是本次原生桥故障的停止条件。它也不意味着允许无限盲重试。

1. 先读取当前安装版本的 computer-use skill、其完整使用/恢复/确认文档，以及当前实际工具文档和工具清单。工具入口可能变化：按本次提供的 API 操作，不混用旧 `sky` 与新 `cua` 的接口，不猜工具名或参数。
2. 建立可重复的最小反馈环：调用当前文档规定的原生清单/连接观测，保留精确错误、时间、调用入口和上下文。区分原生传输缺失、浏览器 host 不可用、会话/连接配置不匹配、目标程序或夹具路径不存在；不能把所有 `os error 2` 都当成 CCR 产品缺陷，也不能先假定某一种原因。
3. 使用适用的诊断技能，针对 Computer Use 相关组件检查官方支持的连接状态、安装版本、宿主状态、会话路由及相关日志。先提出能被验证的原因，再按证据修复。日志和配置只读取必要字段，脱敏，不输出令牌、凭据或完整环境变量。
4. 本任务授权在当前工具和官方文档允许的范围内，对现有可信 Computer Use 集成执行定向的连接恢复、重新初始化和受支持的宿主恢复操作。每次操作都要说明修复了什么，并用同一反馈环验证；同样失败就换有证据支持的路径，不机械重复 `getState`/reset。
5. 不逆向私有管道协议，不构建自制客户端，不搜索并手动启动 `codex-computer-use.exe`，不绕过当前 skill/API 限制。不要改用 PowerShell UI Automation、私有屏幕/输入接口、HTML 或 headless 来伪造原生验收。浏览器恢复也不等于原生恢复；浏览器若不属于必要路径，不扩大为无关环境大修。
6. 需要账户、权限扩张、不可逆系统操作或工具明确要求人工完成的步骤时，遵循实际权限边界。不要伪造批准，不关闭安全检查，不自动重启会导致当前对话或用户工作丢失的宿主。
7. 若证据确认故障位于当前代理无法控制的宿主/平台层，给出精确组件、已验证原因或尚缺证据、已尝试路径，以及最小且具体的人工动作。只有到这一步才请求必要接管，并保留恢复后的接续点；不能再只报告一句“工具不可用”并把任务标完成。不得承诺平台限制一定能由仓库代码修复。
8. 原生恢复的验收必须到达：原生清单调用成功，能绑定实际目标窗口，并取得该窗口真实 AX 状态和截图。一次调用不报错、空清单或浏览器能打开，都不足以证明整个原生验收路径恢复。

可独立的生产修复和 owner 回归继续推进；排障不能成为跳过其他未完成工作的理由。不要修复无关 WSL、系统 PATH、显示器或 DPI 配置。

## 4. 复核实现，发现遗漏就继续改生产代码

上轮已经修复并回归：项目归属完整性、工具/会话 nullable 费用与真实零、默认按调用数排序、报告时钟/窗口/热力图时区、坏时间戳跳过计数与稳定 seq/事务、Tauri 自定义 Claude 配置目录、全局成功同步措辞，以及 ghost、金额和套餐文案遗漏。它们不是重新设计的起点，也不能仅凭交接当作完整覆盖证明。

重点保留并重新核对 PRD 的九 tab、唯一 h1、宽窄导航与选中 reveal、稳定 ID、焦点恢复及 generation 门控；MCP 真正 source/target 权限与非 Codex 废纸篓拒绝；订阅缺键/明确保存默认值/读取失败/CAS；无记录与真实零、精确金额、缓存分母零、输出含推理不重复计数；当前 A 与选中 B 的选择/应用/解除语义；保存、导入、恢复均不应用；未知字段、空凭据、错误草稿、current 与 revision 保护；导入同名默认保留及预览后 CAS；确认初始取消、双向 Tab/Esc/返回触发者、外部焦点打开与晚回包；短/空 pane 收紧、长列表限高、窄窗返回保留状态；扩展只读发现无隐式写入、联网、下载或执行；账户/设置/指令及其他平台实际能力。

工具事件未变文件仍有游标快路；上轮没有重建真实历史缓存或运行真实同步。不能把新解析规则通过说成旧用户数据已清洗。需要进一步处理时使用正确 owner 和隔离回归，不借机操作真实数据。

共享 crate 保持既有消费者兼容，不重设计 Tauri，不在 Claude 首页重新加入不可靠的项目/工具费用模块。

## 5. 使用当前源码候选完成四场真实原生验收

已完成的最新候选：

```text
ccr-desktop/target/release/ccr-gpui.exe
SHA-256: 22ae86c7a775794f74939a5227911fb461d2a5cb5074943f42e302ed5211187a
Bytes: 54184960
```

这是 dirty 工作区构建，不是纯 HEAD 构建。`related-closeout/validation-inputs.json` 记录 807 个共享/桌面/第三方输入的哈希；使用前核对当前源码、二进制和收据。源码或构建输入变化就重新 release 构建，不能继续引用旧哈希。

现有合成夹具为 `outputs/desktop/claude-161-native-20261110`。使用前读取既有 fixture owner，核对 sentinel、启动参数、绝对路径和 roots 隔离。不要覆盖已有夹具；需要新夹具时通过现有 owner 创建新目录。允许启动可见的隔离 CCR 原生窗口执行本任务验收，不操作真实配置。

必须完成：

- 1280×800，浅色中文；
- 2000×1080，浅色英文，同时覆盖工作台与 Profiles 的 DESIGN 视觉要求；
- 960×640，深色英文，重点 Codex 最长导航及窄窗操作；
- 1280×800，深色中文。

验证变更核心及直接受影响回归。记录每场实际外窗、客户区、DPI、构建哈希、夹具和本地截图对应关系；目标尺寸不等于实测尺寸。截图只存本地忽略目录，不上传，不扩大显示器/DPI/全平台矩阵。

禁止真实登录、账户切换、凭据修改、真实方案删除、真实同步或扩展安装。临时修改任何偏好必须备份并在 finally 中逐字节恢复，恢复失败要明确报告并修复。禁止把 headless 控件测试、HTML、源码断言或成功构建当成原生通过。

## 6. 验证与已通过证据

上轮最终结果供定位，不当作后续改动的自动通行证：

- `just desktop-test-ui` 通过：14 shell + 2 input；
- `just desktop-ci` 通过：303 lib、14 shell、2 input、2 shutdown，含 Windows 1 MiB 子进程及新 release；既有 ignored 项没有冒充已运行；
- `just ci` 最终 13m43s、16 阶段全绿；Tauri 336 normal/5 ignored、2 SQL guard、200 exports/inventory；前端 174 文件/951 测试及覆盖率；审计、文档、VS Code 均通过；
- 三处 Rust 1.95 `--locked` 编译检查通过，仅证明 Windows 编译，不证明 1.95 运行测试、Linux TLS 或原生 UI；
- 首轮 CI 因新增六个双语键后漏同步固定计数而失败。两处基线已从 4564 同步到 4570，集合/数量/占位符检查仍严格，最终 CI 已通过。不得反复修复已经修正的旧失败。

按本次实际改动先跑最窄 owner 回归，再完成必要的 `just desktop-test-ui`、`just desktop-ci`、`just ci`。源码未变时可复用有身份对应关系的已有通过证据，不重复运行未受影响外围检查；产品源码变化后补齐相应最终门禁。

保留默认并行和 `--skip export_bindings`，绑定由既有事务生成入口负责；修复命令与验证命令分开，不删除失败测试、不降阈值、不加审计忽略。

系统 bash 指向损坏 WSL，根 CI 只在该命令进程给 PATH 前置 `C:\Program Files\Git\bin`；不修改系统配置。依赖授权仍仅限消除 cargo audit 实际阻塞所需的最小安全修复，遵循 dependency-governance，不全量 cargo update。上轮根/Tauri OpenSSL 安全锁修复已通过，桌面原有 Cargo 改动须保留；只有新证据表明确有阻塞时才继续最小修复。

## 7. 逐票交付和停止条件

对实际 open issue 逐票核对实现、执行入口、回归、必需原生证据和交付条件；不要把某两张票的原生要求套给所有票，也不要把一张票阻塞等同于其他票没有可推进工作。

允许依真实证据维护相关 issue 的处理状态、填写 resolution comment，并关闭已经满足自身全部关闭条件的票。不能把未提交的本地改动描述成远端已有修复；若该票仍缺提交/远端落地或其他交付步骤，保持 open 并准确说明。默认不 commit/push 的限制继续有效，不擅自变更授权。

重设计 PRD 和 DESIGN 合同票的必需原生验收未完成前不能关闭。不要为达到“全关”的数字目标伪造通过、忽略失败或把问题移到新票后宣告完成。

更新真实实施清单、生产合同和 mini-trellis 交接，区分本次新增、继承改动、修复结果、通过命令、原生证据、未运行项和外部阻塞。`add_session.py` 使用 `--commit - --no-commit`，不要自动提交；检查脚本的默认 Completed/规划会话文案，不能因此把整体未验收工作写成完成。

持续执行到可完成工作全部处理、必需验收满足、逐票关闭条件核实。若最后仍有真正不受当前代理控制的宿主或授权前置，留下已经穷尽的受支持路径、最小接管动作与明确接续点，诚实标注 blocked；不把阻塞报告当成已完成，也不靠无限重试代替修复。
