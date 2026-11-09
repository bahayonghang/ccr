---
target: Codex Profile TUI 截图设计评审
total_score: 23
max_score: 40
na_heuristics: 
p0_count: 1
p1_count: 2
timestamp: 2026-08-17T09-37-05Z
slug: crates-ccr-tui-src-tui
---
# CCR Codex Profile TUI — 设计评审

目标：`crates/ccr-tui/src/tui`（Codex Profile 主界面截图 + Rust 源码）
方法：dual-agent（评估A：设计评审子代理 · 评估B：检测器 + 机械证据子代理，两路隔离）

## 设计健康度：23/40（Acceptable，需要明显改进）

| # | 启发式 | 分数 | 关键问题 |
|---|--------|------|----------|
| 1 | 系统状态可见性 | 3 | 有 current 标记、Control plane banner、toast；但成功 toast 仅 3s（toast.rs:29），Enter 应用后立即退出（app.rs:758），成功反馈永远看不到 |
| 2 | 系统与现实匹配 | 2 | 内部术语直出：「Active driver: Runtime/Auth only」「Control plane」（ui.rs:157-193）、snake_case 原始字段名当标签 |
| 3 | 用户控制与自由 | 2 | 无撤销、无确认；Enter=应用并退出；j/k 页边界回绕（selection.rs:12-19）易 overshoot |
| 4 | 一致性与标准 | 3 | 六页外壳与 token 体系一致；但页脚隐瞒 Enter/Space 双语义（ui.rs:2241），「o off」语义含混 |
| 5 | 错误预防 | 1 | 核心失败：切付费 profile 一击即中、零确认（app.rs:927-1002）；Confirm overlay 只用于账号删除（overlay.rs:58-69） |
| 6 | 识别优于回忆 | 3 | 快捷键与图例常驻；但图例只解释 ●/▶（ui.rs:1945），○/✓ 靠猜；Space 应用键不在任何提示里 |
| 7 | 灵活性与效率 | 2 | 22 个 profile 无搜索/过滤/跳转；switch_count 已统计（ui.rs:1312-1319）却仍按字母序 |
| 8 | 美学与极简 | 2 | 同屏 7 个带边框区域；Selection 面板冗余；Routing/Auth 9 行里 7 行是「-」 |
| 9 | 错误恢复 | 3 | 加载失败空态是范本：原因+来源+按 r 重载（ui.rs:2056-2095）；但错误原文直出 |
| 10 | 帮助与文档 | 2 | 页脚即全部文档；无 ? 帮助层、无首次引导；空态指引到 CLI --help 是加分 |
| **总计** | | **23/40** | **Acceptable（20-27 档）** |

## 设计特异性结论

**LLM 评估**：「通用 Catppuccin 外壳 + 领域专属信息架构」的混合体。视觉语言几乎零特异——Mocha 配色、Mauve 选中行、左列表右详情，原样搬进 lazygit/k9s 都成立；CCR 品牌只以纯文本标题出现（ui.rs:398-401），没有可拥有的视觉签名。真正的特异性在数据层：每平台独立的详情 schema（wire_api/requires_openai）、provider 级用量成本、Control plane 运行时模式。为这个产品而作的是数据，不是界面；视觉上是现成皮肤。

**确定性扫描**：detect.mjs 已真实运行（退出码 0，0 发现），但对本目标不可用——检测器仅扫描 Web 标记/样式扩展名（.html/.vue/.tsx 等），不含 .rs，目标目录 14 个 Rust 文件全部跳过；0 发现是"无文件可扫"而非"无问题"。浏览器覆盖层不适用（TUI 无 URL）。替代机械证据：

- Mocha 暗色主题主色对基本健康：正文 11.34:1、选中行紫底黑字 9.23:1、accent 7.79:1、warning 12.91:1、error 7.08:1
- 不达标项：muted `#7f849c` 4.44:1（差 0.06 临界未达 4.5:1）；非聚焦边框 `#585b70` 2.46:1（装饰性，聚焦边框用 accent 7.79:1，可接受）
- **Latte 亮色主题系统性偏低**：muted 2.83、warning 2.31、success 2.96、border 1.91、info 4.34——双主题契约目前只有暗色达标（截图未覆盖 Latte，源码推导值）

**密度机械统计**（佐证认知负荷）：首屏独立信息元素 ≈90+；右侧详情 26 个字段、其中 7 个为「-」占位（Routing/Auth 9 行占 5）；可见选项 >4 的决策点三处（顶部 tab 6、footer 键 8、列表 22）全部超阈值。

**视觉覆盖层**：不可用（终端应用，无可注入页面），替代信号为静态截图 region 放大核对。

## 总体印象

骨架（布局引擎、主题 token、空态）是认真做的，但「最重要的事情最不起眼」：当前生效 profile 这个用户打开工具要回答的第一个问题答得最差，而 $936 的黄色成本数字抢走了全部注意力。最大的机会不在视觉，在于把「切换」这个高风险动作的确认与反馈闭环补上。

## 做得好的

1. **响应式布局真做了功课**：三档 viewport（theme.rs:22-30）、列宽随终端宽度动态计算、CJK 宽度感知省略号截断且配测试（ui.rs:288-312）——「哈基米biu qq pℓus+team」中英混排对齐不乱，TUI 里少见
2. **错误空态教科书级**：加载失败给出原因+来源+确切恢复按键（ui.rs:2056-2095）；真空态引导到具体 CLI 命令
3. **主题 token 架构严谨**：双调色板全字段语义 token（theme.rs:48-89）、focused 面板边框契约（theme.rs:424-434）、平台身份色有测试锁定（theme.rs:790-798）——六页一致是规模化的，不是巧合

## 优先问题

### [P0] 切换 profile 零确认且 Enter 即退
- **为什么重要**：Enter → ApplyAndQuit（app.rs:758），成功 toast 推入后应用立刻退出，用户永远看不到「已切换到 X」。一次误触就把运行时切到付费 profile，发现时可能已产生费用。overlay.rs 里现成的 Confirm 弹窗只服务账号删除
- **修法**：Enter 改为复用 Overlay::Confirm 的确认框（含目标 profile 名 + provider + model + token 状态）；或把「应用并退出」挪给 Shift+Enter，Enter 应用后停留并在 Focus 区常驻最近切换结果
- **建议命令**：`$impeccable harden`

### [P1] 最重要的状态（当前生效 profile）几乎不可见
- **为什么重要**：用户打开工具要回答的第一个问题「我现在用的是哪个？」答得最差——唯一信号是 jargon 化的「Profile: not bound」（ui.rs:142-176）；图例只解释 ●/▶ 却没解释 ○/✓（ui.rs:1945 vs 333-335）
- **修法**：banner 改常驻 current chip（名称+model，未绑定用大白话「未绑定 · 仅运行时认证」）；删掉与 ● 冗余的 ✓；图例补全或精简标记体系
- **建议命令**：`$impeccable layout` + `$impeccable clarify`

### [P1] Selection 面板纯冗余，制造第七个边框区域
- **为什么重要**：Selected/Name 与 Focus 重复、计数与列表标题重复（ui.rs:1921-1947 vs 526-531），白占 5 行高度，多一个竞争注意力的盒子
- **修法**：整块删除，图例并入列表面板 title_bottom
- **建议命令**：`$impeccable distill`

### [P2] 详情面板倾倒全部字段，空值淹没信号
- **为什么重要**：Routing/Auth 9 行中 7 行是「-」（ui.rs:1229-1309）；`requires_openai` false 渲染成警示黄（ui.rs:1302-1305），把"正常"标记成"有问题"，消耗信任
- **修法**：空值默认折叠为一行 muted「N 项未设置」（按键展开）；布尔 false 改 neutral muted，黄色只留给真异常
- **建议命令**：`$impeccable distill`

### [P2] 页脚标注与实际语义不符
- **为什么重要**：「Enter apply」隐瞒退出行为（ui.rs:2241）；Space 应用键不在任何提示里；「o off」不说明 off 什么——Alex 靠试错发现 Enter 把 app 关了
- **修法**：页脚诚实标注「Enter apply+quit · Space apply」；「o」改「deactivate/解绑」
- **建议命令**：`$impeccable clarify`

## Persona 红旗

**Alex（效率型高级用户）**：22 个 profile 只能 j/k 线性遍历，全仓无搜索/过滤/跳转键位（app.rs:750-763）；switch_count=20 已统计却仍字母序；页内回绕丢位置感；Enter/Space 语义差异无文档靠试错。

**Sam（无障碍依赖）**：面板焦点只靠边框颜色区分（accent vs 灰），色弱用户无法判断活跃面板，无形状冗余；yes=绿/no=黄纯色彩语义，无色盲安全图标；选中行 ●/○ 变深灰-on-Mauve，小字号难分；描述列 muted+italic（CJK 算法倾斜可读性差，4.44:1 临界）；Status strip 条件性出现（ui.rs:650-652），反馈位置不恒定。

**Jordan（新手）**：首屏第一行「Control plane / Active driver: Runtime/Auth only」全是内部语言；「Profile: not bound」无解释无下一步引导；详情键名是原始 snake_case 字段名；22 行全是 ○ 而图例只讲 ●；无 ? 帮助层、无首次运行引导。

## 次要观察

- Toast 插入页脚行首导致整行快捷键右移跳动（ui.rs:2153-2166）
- 「Page: 1/1」单页时是噪声，且与列表标题计数重复
- banner「Auth: OpenAI / API Key」用 success 绿，中性状态占用成功语义
- 双列间无引导点，名称到描述视线跳跃约 10 列（可加 dotted leader）
- Focus 面板增量信息只有 Status 一行，高度可让位给 Context
- 页脚条件显示 page hint（ui.rs:2235-2237）是全 UI 唯一的渐进披露——团队有这个意识，只是没用到详情面板
- 截图右上角「Codex Profile」末字形被切：代码上该 label 右对齐且整图无右边框字符，更可能是截图裁切而非应用溢出——存疑项，需未裁切截图复核

## 值得思考的问题

1. switch_count 已在持久化统计（app.rs:981-986），为什么列表还是字母序？默认按最近/最常用排序，22 个 profile 的导航难题是否大半自动消失——比加搜索框更便宜？
2. 真正需要确认的是「切换」还是「切换并立刻消失」这个组合？如果 Enter 应用后停留并常驻显示新生效状态，P0 的确认框是否就不必要了？
3. $936 是 provider 级全时段累计，不是这个 profile 的成本——它出现在单 profile 详情里能支撑什么决策？如果没有，删掉是否比加一行 muted 解释更有价值？

## Run Notes

- target slug：crates-ccr-tui-src-tui（critique-storage 解析成功）
- ignore list：无（.impeccable/critique/ignore.md 不存在）
- 评估独立性：A/B 双子代理并行隔离，互相不可见对方输出
- CLI 检测器：真实运行一次（退出码 0，0 发现），对本目标不可用（不支持 .rs），空结果=无可扫文件而非无问题
- 浏览器可视化/覆盖层注入：跳过——TUI 无 URL，无可注入页面；替代信号为静态截图 region 放大核对
- live server：未启动（不适用）
- 临时文件：正文临时文件写入快照后已删除
