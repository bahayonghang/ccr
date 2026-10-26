# GPUI 04 实施证据

日期：2026-10-18。状态：in_progress；本文件不代表本任务验收通过。

## 恢复与依赖

- 已完整读取自身prd/design/implement与两份curated manifests、02 profiles-source-map（42 command / 5 route）、相关layer indexes与父依赖规则。02已完整验收；03可消费源码与用户指定核心原生流程已独立检查通过，未完成的UIA/IME/延期矩阵/几何差异保留在03，不假记完整完成。
- git status --short、get_context --mode packages、get_context --mode phase --step2.1 --platform codex均实际退出0；保留01–03和其他既有修改。未重建规划、未重复请求实施批准。
- `python .trellis/scripts/task.py start .trellis/tasks/10-17-gpui-04-profiles`实际退出0，planning -> in_progress，session active04。curated上下文现各14条，追加core锁、profile-off、Codex bearer、raw/confirmation契约。

## 最小行为差距与负责人

- 原生当前只读配置；04补真实编辑/原始revision保存/冲突、应用与停用、备份导入导出恢复、配置历史与已有转换器，域内SQL/持久化/认证/日志安全owner保持不变。
- profiles_lifecycle / trellis-implement唯一拥有既有ccr-cli生命周期与必要ccr-config/ccr-codex构造注入：旧API兼容，新增原始draft-token版本检查在事务内，保留journal/replay/补偿/warning/recovery。
- 已确认仅构造注入仍会由CatalogGuard/guarded writer调用默认锁目录。将现有ccr-core/core/lock.rs的窄同步thread-local RAII scope交同一owner，覆盖整个既有单worker生命周期；默认行为/资源hash/锁顺序不变，不跨await/spawn，异常恢复并验证线程隔离。生产捕获既有默认目录，fixture显式私有目录，不新增UI任意锁目录入口或全局HOME修改。
- profiles_services / trellis-implement唯一拥有实际共享ccr-app config策略、必要Tauri adapter委托与全部root/Tauri/desktop Cargo manifests/locks/dependencies记录；不复制领域owner同名API。typed patch/raw/managed backup策略移除旧端真实重复后供两端调用，Tauri wire DTO/export保持。
- profiles_ui / trellis-implement唯一拥有native runtime/profiles与features configs/profiles/converter、app/shell/lib/locales接线及实际控件测试/README/DESIGN事实更新。所有IO在有界worker，draft/response绑定原context+revision，敏感editor scoped且清理。
- 主会话负责任务/台账/上下文、文件写权协调与原生测试；check在明确冻结后接手。所有参与者共享工作区，不回退别人修改，不递归派发implement/check。

## 已审阅归并和边界

- 同一Claude catalog可用一个原生列表承载 /configs 与 /claude-code/profiles，前提是所有generic/dedicated动作可发现，保留泛型redacted export与专用secret canonical export的不同语义；完成前route仍不计通过。Codex/Grok使用各自独立catalog和当前状态owner。
- fixture保留显式profiles/registry配对，不静默忽略registry参数；派生辅助/平台根均为声明的私有fixture路径，default constructors和锁/历史/认证不能间接逃逸。读不mkdir、不初始化或迁移catalog；构造和query纯读取。
- 继续只验证核心编辑/保存/CAS/应用/错误/取消/退出；不执行缩放/DPI矩阵、不用真实用户配置或账户测试、不自动提交/推送/发布/签名/归档。

## 验证

实施中。各owner实际命令/退出码/失败修复记录将写research/{lifecycle,services,ui}-evidence.md。未运行项不计通过；03检查证据不冒称04写入/生命周期验收。

## 本轮恢复与核心审查（实施中）

- `get_context.py`及`--mode phase --step 2.1 --platform codex`实际退出0，确认session active04、02 completed；03开放验收仍保留。初次人工读取旧`.trellis/.current-task`路径提示不存在，随后使用正常session-aware命令恢复，未创建伪造任务指针。
- Computer Use经`@oai/sky`重新`list_windows`成功，当前没有CCR原生窗口。此前03六次退出记录与core-summary仍保留，02验收证据另存于其任务；用户恢复窗口的协调答复不替代04的新原生证据。未操作其他应用、未启动未冻结二进制。
- 共享服务owner报告21个测试通过及Tauri实际default-feature配置18个通过；领域owner报告新增注入生命周期12/12、scoped-lock2/2通过。原始日志已保存，独立检查和完整受影响回归尚未完成，此处不提前勾选AC。
- 主会话定位Grok private operation与catalog顺序反转，临时shared阶段实现已移除；现由Grok领域owner提供already-locked adapter和真实staged rename/delete策略，保留部分失败和force边界。
- 原生脱敏导出误用secret-only专用source接口、历史`OperationResult::Failure/Warning(String)`被Debug展示、advanced字段可能被静默忽略及generic active-disable保护差异已交唯一UI owner修复并补测试；未把按钮存在算完成。
- Tauri backup cleanup默认root保留历史HOME/.ccr；native使用显式fixture root。没有顺带改变旧端root选择，也未在真实home运行清理。
- 使用trellis-update-spec记录同步scoped-lock目录、注入生命周期、原始draft revision和Grok锁顺序；04 curated contexts现各15条。spec是执行契约，不是尚未运行检查的通过证明。

## 原生验收准备（尚未启动04 GUI）

- 从已关闭的02/03领域API创建的synthetic fixture复制profiles.db、registry config.toml和manifest到`outputs/desktop/profiles/native-seed`，原件不改。哈希在`native-seed-copy.json`；profiles.db SHA256为`1efbf727f1946196bba2d50e6528c1d4b1fe79e54be943b1d031bd63eedd32d5`，与原始seed一致。
- `outputs/desktop/profiles/launch-native-fixture.ps1`为每次运行创建新的私有fixture与准确owner marker，拒绝覆盖既有run，原生参数只传配对显式路径；将保存实际二进制hash/PID/退出码/stdout/stderr，不覆盖HOME。PowerShell Parser实际检查0语法错误，退出0；这不代表GUI通过。
- 为真实原生stale-draft CAS准备的第二客户端是预先编译的ccr-app ignored fixture helper，拒绝outputs/desktop/profiles外、缺marker或catalog逃逸的目标，通过ConfigManager和与native一致的`desktop-profile-fixture/locks`更新description。GUI期间直接调用testexe，不运行Cargo、不复制活跃数据库或手写SQL。
- 在独立`helper-preflight`实际调用`--exact mutate_native_profile_fixture --ignored --nocapture`，退出0，1个fixture helper通过，catalog revision由`1b6a4b7f…`变为`10a15b92…`；输出只有版本/name/status，stderr无诊断。`helper-preflight-receipt.json`记录exe hash和no-HOME事实。此结果只是工具预检，不是04原生冲突验收。

## 独立领域检查与根回归

- trellis-check实际先复现、再修复四个问题：Grok真正提交时的原始revision检查；Grok rename保留exact TOML/datetime；Codex off registry失败恢复legacy入口快照；首次创建事务发布，避免凭据回滚后遗留orphan catalog。详情与失败/修复日志在`research/check-final.md`和`outputs/desktop/profiles/check-*`。旧implementer freeze receipt被新的`check-frozen-receipt.json`源码哈希接续，不拿旧源码结果证明新修复。
- 独立受影响4-crate测试退出0：ccr-app23、CLI409+dispatch12、Codex392、config126、doctest1通过，保留原ignored/export filters；最终catalog专项3项和严格all-target/all-feature Clippy退出0。Tauri frozen config18、Grok16、命令inventory1及secret-write guard通过。native检查仍另行进行。
- bindings原始checker加`--locked`的首跑0，第二proof跑实际1（11文件漂移），随后带差异捕获的三次最终跑0。全部230原始生成文件精确bytes及三份lock恢复且哈希不变；**原因未复现，不宣称稳定性问题已修复**。未改生成类型、未静默rebaseline。当前通过与历史失败均保留，最终整合仍跟踪此风险。
- 主会话使用项目实际Cargo/rustc1.98、仅子进程PATH配对，实际运行原始`just version-check`、`just fmt-check`、`just lint-strict`、`just test`，退出码均0。根完整测试结果见`root-test-summary.json`；命令阶段时间/原始日志见`root-gates-receipt.json`及`root-{fmt-check,lint-strict,test}.log`。Root lock前后SHA256均`f4a932f8ca98b21ec80b4d0d178e6c3a67f7510993da49b91b03a55c07fa3b15`。
- 原始root公开recipes保持原命令语义；依赖锁定额外由前述领域/共享/Tauri/desktop的实际`--locked`检查验证。验证未隐式执行fmt/version-sync修复，不改测试并行、覆盖阈值或required context。`just ci`与父任务完整验收仍未运行/通过。

## 原生独立检查（尚未进行04 Computer Use）

- UI implementer实际完成42command/5route映射、完整选中profile readonly Inspect、generic/dedicated显式模式；`research/ui-evidence.md`明确其结果仅是源码与headless，不冒称逐项headed验证。
- 第二名trellis-check唯一接手native源文件，实际复现确认框打开时底层Save仍执行、禁用按钮仍为Tab stop，以及Grok已成功重试后残留retry动作。已作窄修复：同节点禁用语义、使用Kit既有FocusTrapElement的真实决定子树、精确Restore leaf、committed成功清理旧recovery但保留warning。原始失败和后续验证均由`research/native-check-final.md`记录。
- 已收到该检查者fmt/check/strict lint/full desktop-test退出0：Python12、AccessKit3、Rust lib14、native_shell3、native_ui2通过；Rust1.95完整图验证仍进行。此后还需fixture backup→clean闭环修复/复验，最终源码hash以检查者最终receipt为准，不能拿本条中间结果证明后续改动。
- 发现fixture.paths.root原先仅指aux目录，实际ConfigService通用备份在catalog parent，导致native fixture CleanBackups漏掉自己创建的备份。最初拟修改fixture.paths.root，经领域衍生路径审计发现还会移动off/analytics/registry辅助文件，因此未实施该方案。最终选择仅在native worker从既有ConfigSource绑定cleanup_root：Local沿用context.paths.root，Fixture使用声明的私有catalog parent；paths.root及所有aux路径/锁位置保持不变，不增加UI任意root输入。生产resolver/depth3/扩展名清理集合不变，补真实Services backup→list→CleanBackups及无关文件保留测试。没有扩大生产清理范围。
- 主会话仍未启动04 GUI；全部源代码与编译检查结束后才构建最终debug artifact，保存hash并操作真实窗口。测试期间不运行Cargo，不使用真实用户配置或账户。

## 2026-10-18T07:20Z Computer Use recovery stopped

After consuming the final frozen native-check report, main started `just desktop-build-debug`. The initial sandbox attempt exited1 because the existing mbx shim identity metadata was inaccessible; raw `outputs/desktop/profiles/native-build-final.log` and JSON receipt retained. The cache-access retry is separately logged as `native-build-final-cache-access.*`.

The first recovery call (`sky.list_windows`, before selecting or launching any04 GUI) returned: "Computer Use was stopped by the user with the physical Escape key. Stop your work, do not call further Computer Use tools in this turn, and send a final message noting that the user stopped Computer Use." Main stopped all Computer Use immediately. No04 native fixture window was launched, no input was sent, and no04 screenshot/native flow is marked passed. This is a new explicit stop result after the user's earlier restored-window response; it does not revoke implementation approval. Core native acceptance remains pending. No zoom/DPI checks were performed.

## 2026-10-18 用户要求暂停与交接

- 用户明确确认Escape是主动停止，要求暂停并提供下一对话Prompt。现在不继续产品实施、检查或Computer Use；现有实施批准保留。
- 正常 `get_context.py` / `--mode phase` 查询退出0：session active04，父/01/03/04为in_progress，02 completed，05–10 planning。暂停通过父/04的 `meta.execution_state=paused_by_user` 和交接文档记录，未将未验收任务改为completed或归档。
- 最终构建现已实际结束，`native-build-final-cache-access.json`记录 `just desktop-build-debug` exit0，完成于2026-10-18T07:20:22.4307061+00:00。二进制SHA256又核对为 `2d7b4271c07dbf0e14fd301ee3df7b6f8ba73996ac570eacf8b947645a606e26`，不是旧版03产物。前一回复“正在运行”是当时状态，本条以最终receipt补齐。
- 消费native最终报告及 `native-check-accepted-gates.json`：最终fmt/check/lint/locked Rust tests/MSRV全部0，lib15/native_shell3/native_ui2；最后generic-validation/fixture-cleanup修复已经复验，并非仍在进行。原始中间失败/修复证据保留。
- 暂停时检查 `Get-Process -Name cargo,rustc,ccr-gpui` 无结果；子agent状态均completed，没有后台实现/Cargo/GUI遗留。所有已有工作区修改保留，无commit/push/发布/签名/归档。
- 当前可复用源文件、证据、真实未验收项、fixture工具、04原生步骤及05–10续作Prompt见 `pause-handoff.md`。不执行finish-work的自动归档/提交动作；暂停不代表完成。会话记录使用add_session.py的显式 `--no-commit`。
