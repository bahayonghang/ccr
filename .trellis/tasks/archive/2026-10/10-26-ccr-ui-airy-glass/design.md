# CCR Tauri 轻盈玻璃适配设计（2026-10-26已批准）

## 1. 来源和优先级
/design-md 的 style:airy-glass 已通过技能脚本 apply 到 research/catalog/DESIGN.md，保留 provenance。该文件保留为来源快照；实施按本方案的CCR适配同步 ccr-ui/DESIGN.md，不直接复制目录覆盖行为与数据合同。

用户指定新风格优先于旧行情终端审美；数据、安全和行为规范继续有效。批准后AG01同步 ccr-ui/DESIGN.md、ccr-ui/AGENTS.md、ccr-ui/.impeccable/design.json、theme token spec 与相关测试。审计根AGENTS/code_map及ccr-ui/code_map中的方向引用和 .github/skills/ccr-ui-visual-workflow 的说明，仅修改与本次方向冲突的文字；本地忽略技能副本不是仓库交付物。

唯一目标为 ccr-ui React/CSS。GPUI、IPC、SQL和业务服务不属于视觉重构。

## 2. 原始风格的CCR适配（建议）
| 项目 | 方案 | 理由与代价 |
|---|---|---|
| 背景 | 160deg 薄荷 #E7F6F1 → 淡紫 #EDE7F8 静态场景 | 保留所选风格核心；不能只改按钮颜色 |
| 玻璃 | 应用外壳一张纸面，亮色白0.78、20px blur、saturate 1.15、单阴影 | 同屏真实blur最多1层，采用比目录6层上限更严格的预算 |
| 内容/浮层 | 卡片、表格、输入、代码、图表、sticky、dialog/popover均实底 | 保证阅读、滚动与危险状态的稳定性 |
| 降级 | <960 CSS px、reduce transparency、无blur支持时使用alpha=1，blur none | 目录sheet-solid为白0.96仍非不透明，CCR明确加强此降级 |
| 暗色 | 保留light/dark/system，以同构低饱和深色场景补足暗色 | 目录没有暗色；AG01先做合成对比度预检再冻结色值 |
| 偏好 | 保留neutral/clay、clay accent值域/默认值/历史迁移 | 不新增airy主题provider，不重置存储；clay是历史key，可改变显示色值 |
| 语义 | 默认主操作/链接/焦点采用绿，紫仅用于明确输出；错误红/警告黄独立 | 平台色、统计维度不能强行压成绿紫，未知/失败不能涂成成功 |
| 字体 | 保留当前UI/code字体轨道、用户覆盖、数字等宽与tabular nums | 不下载Figtree；不改变字体机制 |
| 几何 | 20px限定应用纸面，16px可用于静态well；密集控件保持高度 | 独立按钮/标签可pill；不将8px最小间隔套到热力图/表格，不改虚拟行高 |
| 动效 | 不缩放整个纸面，不动画blur，无漂浮blob；保留受控按钮按压 | sheet:active范例不适用于整张桌面外壳；继续reduced-motion单点 |
| 图表 | 保留来源色、热度单调、真实比例、精确表格和图例 | 目录没有图表色板，需要对新底色重新验证 |

上述暗色扩展、字体/偏好保留与全前端范围已随用户“批准开始实施”获得批准。目录本身“一次模糊”与“至多六层”的文字跨度，在CCR收敛为一层。

## 3. 令牌和布局
继续使用 src/styles/tokens.css 的语义变量及 core.css 的 Tailwind 映射。通用 bg/text/platform/status 和共享 --surface-workspace-* 保持实色；专用应用纸面别名才允许alpha。不能全局开放 surface-shell/workspace blur：两者均被多处消费（见surface-inventory）。

AG01兼任本次专用令牌治理：先记录变量名集和引用，登记最小 --surface-app-sheet-* 与场景变量族，输出前后差集。不另建主题系统、不散布组件hex、不复活legacy liquid-glass变量。保留 @theme inline 引用关系、唯一主题写入口与runtime custom accent扩展行为。

MainLayout + src/shell/shell.css 使用静态field和唯一非滚动paper，优先以现有根元素/伪元素承载，必要时才加展示包装。纸面之外的渐变应可见，内部实底阅读层不能把材质效果全盖住。保留 content-scroll-area、scrollRef、滚动恢复、Outlet生命周期、sidebar resize与键盘/移动行为，禁止为玻璃重挂载业务树。

hideGlobalBackground/theme-stage 分支逐页检查。backdrop-filter会影响层叠/定位，验证portal、fixed、sticky、tooltip、dialog、移动侧栏和resize hit area。浮层变实底而不是在玻璃纸上再开一层玻璃。

独立 /tray/codex 不在 MainLayout：使用紧凑实底纸面，保留其尺寸/焦点/原生职责，不照搬主窗口的外边距和blur。

首帧同步 index.html、critical CSS、首屏/延迟CSS入口、flavorPreview.ts。核对 nativeWindowAppearance.ts 的macOS底色，仅同步视觉值与mock断言，不改变macOS-only/lazy-import/setTheme机制，不修改transparent:false、权限或窗口装饰。无法实机验证macOS时明确缺口。

## 4. 冻结合同
- API facade、typed IPC、Query keys、CAS/dirty-key patch、遮罩/备份/确认/原子写入不变。
- requestConfirm、focus trap、Esc/关闭、焦点恢复、aria、disabled语义不变。
- 虚拟列表估高/overscan、分页、末项可达、滚动所有者和cache不变。
- Usage visitedTabs/hidden保持挂载；options/series稳定，不替换图表库。
- 首页有界图表、平台条带、状态栏、Insights结构及未知值“—”保留。
- Web不读取本机的提示保持；fixture只用于隔离验证，不混入产品正常数据。
- 不改导航/重定向/query参数、作业取消/恢复、订阅清理、环境隔离。
- /skills仍是Skillport迁移引导，不借视觉改造重建旧管理器。
- 不顺带清理所有legacy命名，只处理本次实际触及的视觉规则。

## 5. 可访问性和测试迁移
现有主题守卫既包含外观快照，也包含功能合同。仅迁移旧颜色/半径/材质角色断言，保留存储、字体、唯一写入口、overlay单实现与motion规则，逐项说明改变原因。

保留primary≥12、secondary≥7、muted≥4.5。普通文字至少4.5，大字和关键非文本控件至少3。目录#5C6B76不能直接当要求7:1的secondary。白色paper边线不能作为输入边界。检查渐变端点/中点/实际最差位置、每种flavor和浮层状态的真实合成背景，而非只算RGB原值。

已有“同屏≤3”注释/正则与规范≤1漂移，统一后增加运行时computed-style计数。静态正则不能代替可读性、blur预算、键盘和截图证据。保留所有现有更严门槛，不以新审美为由删测试。

## 6. 交付与回滚
AG01 → AG02 → AG03 → AG04。先基线后改造，每段代表页验证后扩散。基线必须记录当前脏工作树指纹，不能把HEAD冒充实际运行版本。开始实现前重新检查70项既有改动是否继续变化。

各段保留本任务差异、变更文件清单、原始证据；只回滚本段diff，禁止整库reset/clean。令牌/规范/测试作为一致单元回滚。性能失败先评估实底降级，不静默放松门槛。具体数值调色是AG01可验证技术工作，不是本轮已验证结论。
