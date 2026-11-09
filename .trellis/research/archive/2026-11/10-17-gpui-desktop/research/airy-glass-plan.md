# GPUI 轻盈毛玻璃增量规划

日期：2026-10-27。范围：优化已有 GPUI 任务，尚未实施本次视觉增量。沿用父任务与 01–10，不创建平行主题项目。原 `paused_by_user`、04 暂停交接和窗口尺寸/DPI 的 `user-deferred` 保持；本次请求不解释为恢复产品实现或 UI 操作。

## 来源与优先级

- 视觉参照：`ccr-ui/DESIGN.md` 的 Creative North Star、Colors、Elevation & Depth、Shapes；具体 field/sheet 数值来自 `ccr-ui/src/styles/tokens.css` 的 `--surface-app-*`。这是现有仓库实现参照，不代表 Tauri 全部原生验收已完成。
- 当前 GPUI：`ccr-desktop/DESIGN.md` 仍为历史 Codex 设计；`src/theme.rs::apply_palette` 使用黑白/中性配色且关闭阴影；`src/shell/mod.rs::Shell` 持有主题、焦点和页面实体；`src/lib.rs` 持有窗口选项。已检查的这些路径没有实现本计划的单层纸面材质。不能据此断言 GPUI 库不支持模糊。
- 本计划及父/子 PRD 是此次变更的目标合同；`ccr-desktop/DESIGN.md` 暂时描述旧版。实施获准后，03 首先合并本合同到该文件，再同步唯一 Rust token 层。原规范中的“不使用玻璃”“黑白主操作”届时由本合同替代，输入、密度、数据和安全约束保留。
- 不改写既有 research/evidence 或原始截图，不把旧 3.4 的无模糊验收转记为新材质通过。既有业务批准仍有效；新增视觉范围待本次规划审阅后实施。

## 可观察目标与非目标

GPUI 与 Tauri 呈现同一轻盈工作台语言：静态薄荷绿到淡紫场景、单张纸面、明确而克制的层次、绿色动作与焦点、紫色明确输出。保留 GPUI 的原生系统字体、CJK fallback、14px 正文、36px 固定列表行、现有最小窗口及键盘导航，不逐像素复制 Web 布局。

只更换视觉呈现，不增加主题商店、flavor/accent/font 偏好体系，不复制 Tauri CSS/provider，不更改配置读写、CAS、认证、命令、用量 SQL 或 job 生命周期。Tauri 的偏好及自身源码保持；GPUI 已有 System/Light/Dark 与语言行为保持。不新增 WebView、操作系统壁纸透视、Mica/Acrylic/vibrancy 或透明窗口需求；应用内纸面材质与 OS 窗口背景是两种能力。

## V1：材质、颜色与密度合同（03 所有，04–09 消费）

| 角色 | Light | Dark | 约束 |
| --- | --- | --- | --- |
| 静态场景 field | #e7f6f1 → #ede7f8 | #102b26 → #25213d | 160deg；不移动、不每帧更新 |
| application sheet | white / 78% | #172225 / 90% | 唯一允许背景模糊的非滚动层 |
| sheet 的实色降级 | #f2f7f5 | #172225 | alpha=1，无模糊 |
| base / elevated | #e7f1ee / #f2f7f5 | #10191b / #172225 | 实色 |
| reading / overlay | #ffffff / #e3ece8 | #1d292c / #233033 | 列表、输入、图表、菜单、对话框均实色 |
| primary / secondary / muted text | #1e2932 / #404e59 / #5c6b76 | #f0f8f5 / #c5d5d0 / #9fafa9 | 按实际背景组合计算，不假定全色表任意组合合格 |
| action/link/focus | #156f4e，hover #146a4a | #85d5b1，hover #9ce0c3 | 保持独立的 on-action 对比色 |
| explicit output | #6d4bd6 | #b9a4f4 | 不拿紫色表示所有选中或未知状态 |

success/warning/danger/info 和平台/来源身份保持独立角色，附文字或图标；不得统一染成绿色。新增材质参数仅属于 sheet，不污染 Kit 的 list/input/popover/background 通用别名。仍使用 `src/theme.rs` 映射 Kit Theme Base/Component，不建立第二套主题所有者。

纸面目标为 20 logical px 圆角、单层轻阴影和细描边，参考阴影为 light `0 20 48 / 8%`、dark `0 20 48 / 24%`；内部密集控件沿用尺寸，不传播 20px 圆角。只给外场景预留必要边距，不能挤掉最小窗口的操作；滚动 owner、虚拟行估高、stable ID、详情返回与 resize hit area 均保持。对话框、粘性工具栏、托盘独立窗口不用第二层玻璃；遮罩可半透明但不模糊。

普通文字至少 4.5:1，焦点/关键边界至少 3:1；主/次/辅助文本角色分别以 12:1、7:1、4.5:1 为目标。必须测实际使用组合及 sheet 与场景两端、中点、最差位置的复合颜色；必要时调整语义 ink，不通过降低阈值过关。禁用整组内容不整体降 alpha。

## V2：原生材质能力与降级（03 探针，09 系统接线，10 复验）

先针对锁定的 GPUI/Kit 版本核查源码和可用 API，再制作最小原生探针。记录 API/源码位置、后端、OS/GPU/驱动、版本、二进制 hash 和截图。目标是应用内 scene 下的单 sheet 背景采样，不能拿 `WindowOptions` 的透明窗口标志、普通 alpha 或模糊整棵子树代替。不能模糊文字、输入和滚动内容。

Web 的 `blur(20px) saturate(1.15)` 是视觉参照，不是已确认可直接调用的 Rust API。验证原生等效实现、采样范围、裁剪和失效更新后才启用。无可用原生能力时使用上述实色方案；若必须升级依赖、引入新绘制后端或自研 shader，先记录阻塞和成本，回到范围评审，不在视觉任务中暗加图形框架。

材质选择顺序：减少透明度/高对比度要求 → 不支持或能力未知 → 性能预算失败 → 实色；只有已验证支持且满足预算才开启单层玻璃。系统设置能读取时必须遵从并响应变化；不可读取时记录限制，采用保守实色。测试可注入能力/偏好以验证选择逻辑，但注入结果不算系统设置联动通过。

Tauri 的 <960 CSS px 规则不能直接变成 GPUI 新导航断点：GPUI 最小宽度仍为 960 logical px。若系统给出小于此宽度的有效内容区，材质保护降为实色，不承诺新增小屏布局。原有 1040/1200 布局断点保持。尺寸/DPI 原生矩阵继续 `user-deferred`，未来恢复需用户明确指令。

能力记录至少包括：Windows/macOS/Linux 各自的内景模糊、减少透明度/高对比设置读取、原生呈现结果和 fallback 原因。初始状态均 `NOT_RUN`。不支持记 `UNSUPPORTED` 并说明影响；`fallback PASS` 与 `glass PASS` 分栏，fallback 可支持该平台预览，不等同于毛玻璃完整验收。

## V3：按业务面消费（04–09 所有）

| 子任务 | 必查场景 | 保持的行为 |
| --- | --- | --- |
| 04 Profiles | 列表/详情、编辑器、冲突/部分完成、确认、源码临时区 | draft、CAS、脱敏、离页清理、焦点恢复 |
| 05 Platforms | 设置、账户状态、认证错误/超时/取消 | capability、managed lock、dirty-key 与秘密边界 |
| 06 Resources | 搜索列表、详情编辑、安装/失败/取消 | stable ID、虚拟化、权限及长名称可达性 |
| 07 Insights | Usage/Insights、会话、图表/精确值表、空/未索引 | 固定行高、图表有界、来源色与真实零值语义 |
| 08 Jobs | 收藏/历史、运行日志、同步/签到、cancel_requested/terminal | 有界日志、背压、取消清理、页内错误 |
| 09 Environments | 环境切换、断连、隐藏恢复、托盘、系统外观变化 | draft/job 不串环境，独立托盘实色，退出 owner 不变 |

各域复用 03 token/材质能力，不自行增加 blur 或改变行高。已交付域补视觉回归；未交付域在业务实施时直接采用新版合同，不创建第二轮全量页面重写。

## V4：证据与性能（03 建基线，10 全量复验）

1. 03 改动前固定同机 release、隔离 fixture、lock/hash、主题、窗口、GPU/驱动，保留当前 GPUI 的输入/滚动/空闲样本；Tauri 比较另沿用 `performance.md`，不能混用 Web 测量与 native 采样。
2. 探针先验证有无真实背景模糊；主题切换、焦点、loading/empty/error/disabled、确认与 Esc 在真实原生窗口复核。截图必须注明页面、主题、材质模式/原因、构建 hash 和 fixture；只有当前已接入页面能计入覆盖。
3. 同方法比较改造前、glass、solid，覆盖 S1/S2 滚动、日志、导航、输入、CPU/内存；仍满足原性能合同全部预算。模糊只作用于非滚动 sheet；无装饰循环、整窗重复采样或隐藏页刷新。
4. 10 汇总 03–09，每个页面/状态关联业务 owner、native 证据与材质模式；未来页面、未测 OS、系统设置 API 缺口不得用当前单机截图补齐。
5. 结果使用 `PASS` / `FAIL` / `NOT_RUN` / `UNSUPPORTED` / `NOT_COMPARABLE`，尺寸/DPI 附 `user-deferred` 原因。历史失败和旧截图保留。新的性能门失败时关闭玻璃并记录，不放宽预算或声称完整通过。

## 实施顺序与回退

- 此轮仅更新任务文档及 implement/check context。产品源码、运行中进程、依赖、锁文件、原始证据和 task status 不变。
- 审阅增量且明确恢复后，03 先完成 V2 能力探针及 before 基线，再更新 `ccr-desktop/DESIGN.md` → `src/theme.rs` → shell 材质/组件；保持页面 entity 生命周期，不用重建实体实现换肤。
- 04 在新材质下复核现有配置闭环并完成原欠缺验收；05–09 按既有代码依赖推进。03 的平台缺证不阻断可消费工程，但发现焦点/IME/数据契约回归必须先修复。
- 最后由 10 整合 V1–V4、原生证据及性能。保留既有 UIA/IME、平台与 DPI 开放项，不因设计更新清零。
- 回退先关闭材质增强到 solid，保留新配色及同一实体树；必要时回退本次视觉补丁，保留既有业务实现和 Tauri/CLI 入口，不迁移/删除数据。

## 本轮规划校验（2026-10-27）

- PASS：11 个既有任务的 `task.py validate`；18 份受影响的 implement/check manifest 均包含本合同及 Tauri 参照。
- PASS：父子关系、三类计划文档、引用路径及需求所有者核对；task status、完成时间、原暂停 metadata 与历史证据保持。
- PASS：`git diff --check` 与新增文档空白检查；产品源码、依赖和锁文件无修改。
- 既有注入警告：`dependency-governance.md` 为 36,323 bytes，`profiles-source-map.md` 为 32,888 bytes，超过 32,768 bytes 自动注入上限。后续涉及这些文件的执行者须分段直接补读完整内容，不把被截断的注入当完整契约；本轮不重写这些历史材料。
- NOT_RUN：产品构建、原生窗口、模糊探针、性能与系统设置联动；本轮是文档规划验收，不能推导产品验收。尺寸/DPI 仍 user-deferred。
