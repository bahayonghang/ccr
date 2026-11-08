# GPUI 轻盈毛玻璃材质重建与侧栏 / Tab / Overview 重排

## 背景与问题诊断

用户 2026-11-07 截图（浅色、英文、约 1918×1374 窗口、Overview 首屏）：GPUI 版“完全没有 design-atlas airy-glass 的轻盈毛玻璃感觉”。

### 根因（不是实现走样，是合同本身）

1. **材质被合同禁止。** `ccr-desktop/DESIGN.md` Elevation 章与 `theme.rs:204-221 solid_sheet` 规定纸面 alpha=1、无模糊；上一轮任务 `archive/2026-11/11-07-gpui-airy-glass-polish/prd.md` 约束写明“不引入假玻璃”。理由是 GPUI 0.3.9 没有元素级 backdrop 采样 API。
2. **这个推理有漏洞。** 纸面后面不是任意内容，而是应用自己绘制的**静态 160° 两色线性渐变**。对平滑渐变做高斯模糊后几乎还是原渐变，因此“半透明纸面 + 已知平滑场景”在像素上约等于真实 backdrop blur。Tauri 版本身就是 `rgb(255 255 255 / 78%)` + `blur(20px)`（`ccr-ui/src/styles/tokens.css:195-199`）。GPUI 不需要采样 API 就能达到相同视觉。
3. **场景几乎不可见。** `solid_sheet` 的 inset 上限 12px，场景只剩窗口边缘一圈；侧栏 `#E7F1EE`、纸面 `#F2F7F5`、面板 `#FFFFFF` 都是不透明薄荷灰，整体读作“灰绿色后台”，没有空气感和光感。
4. **缺少玻璃的三要素：** 透出的色彩场、亮边高光（inner highlight / 白色 hairline）、柔和的大半径漫射阴影。目前三者都没有。

### 侧栏问题

- 实色 `neutral` 底，与纸面同色系硬拼，没有材质分层。
- 选中项是 14% primary tint + 不透明 primary 细边框，读作“输入框/按钮”，偏重；hover / 选中 / 焦点三态区分靠边框。
- 分组标题 hairline 与行间距偏松（13 个目的地 + 分组撑满高度），品牌行与选中行贴得过近。
- 平台图标只有 Claude 着色，其他平台灰色，身份表达不一致。

### Tab（lens 工具行）问题

- `shell/mod.rs:836-856`：分段控件容器带不透明 `control` 边框并 `flex_wrap` 横跨整宽，再加 `border_b`。视觉上像一个**通栏输入框**，两个 tab 挤在左侧，右侧大片空框。
- 选中 tab 为白底 + 不透明 control 边框 + 绿字，与容器边框叠成双线。
- 与页面标题分离成两行 chrome，浪费 48px 首屏高度。

### Overview 问题

- **KPI 层级弱：** Requests / Tokens / Cost / Sessions 是“左标签—右数值”的 4 行，中间是 900px 空白；最重要的 `$1021.42` 与其他行同权重。
- **平台带像表格：** 6 格等宽 + 顶部 tick，Antigravity/Grok/Pi 只显示“Daily split unavailable”，sparkline 高度极小，信息密度低且同形重复。
- **图表空旷：** 90D 下前 2/3 是空白（数据从 10-16 才开始），没有“数据起始”提示；柱子颜色重、无圆角。
- **精确值表默认铺开且从最早日期升序**，首屏下方是一长列 `0`；等宽字体斜杠零 `0` 放大了噪声。
- **Next actions** 在右侧只占上 1/3，下方是死区；与左侧主面板高度不对齐。
- 所有面板都是同一种白底卡片，没有主次材质。

## 目标

用户打开 GPUI 工作台时，第一眼看到**可见的薄荷→淡紫光场**，以及漂浮在上面的**半透明亮边纸面**。侧栏是透出场景色的玻璃导轨，Tab 是紧凑的分段胶囊，Overview 先给出带层级的 KPI，再给出图表和下一步。整体与 Tauri 版 airy-glass 属于同一家族，在 light / dark 两个主题下都成立。

## 需求

### R1 材质合同重建（DESIGN.md 先行）

- 修改 `ccr-desktop/DESIGN.md` Elevation 章：从“实色降级”改为“**已知静态场景上的半透明分层材质**”。写明成立条件：材质背后只能是应用自绘的低频场景（渐变 + 柔光斑），不能是文字、图片或列表等高频内容。因此不需要 backdrop 采样，也不构成“伪造模糊”。
- 材质分三档，最多三层，**禁止玻璃叠玻璃**：
  - `scene`：渐变场 + 2 个静态柔光斑（用大模糊阴影或多段渐变实现，无动画）。
  - `chrome`：侧栏 / 顶部工具行，低不透明度，让场景色透出。
  - `sheet`：主纸面，较高不透明度，带 1px 白色亮边高光和大半径漫射阴影。
  - 密集阅读面（表格、输入、弹层）仍为实色，保证对比度。
- 场景边距随窗口放大：默认窗口必须能明显看到场景（目标 ≥ 16–24px），最小窗口 960×640 仍可收敛到 0。
- dark 主题定义对应材质：深青→深紫场景、低亮度亮边、更重阴影。不能只做浅色。
- 对比度合同改成**按合成后颜色计算**：文字落在半透明面上时，用渐变两端和中点分别合成后验证 ≥ 4.5:1（正文）/ 3:1（控件边界）。

### R2 侧栏

- 侧栏改为透出场景的 `chrome` 材质，与纸面之间用亮边分隔，不再是一块实色。
- 选中态：半透明白色 pill 面 + 主色文字/图标 + semibold，**去掉不透明边框**。hover 用更浅的同色阶，键盘焦点保留独立的 2px ring。三态必须可区分。
- 收紧节奏：行高 32 保留，分组标题与上一组间距 12–16px，标题与首行 4px；品牌行与第一项之间留出呼吸空间。
- 平台图标着色规则统一：要么全部使用平台身份色，要么全部中性、仅选中项着色。两种方案在 design.md 中二选一，不能混用。
- Settings dock 与版本行保留，与上方导航用亮边 hairline 分隔。

### R3 Lens Tab

- 分段控件改为**内容宽度的胶囊**（不再通栏），容器是 `chrome` 低透明面加亮边，选中项是浮起的实白 pill（轻阴影）+ 主色文字，未选中项无边框。
- 与页面标题合并到同一行：左侧标题，右侧（或标题下）分段控件，Refresh 等页面操作与之对齐。去掉独立工具行的 `border_b` 和 48px 高度。没有 lens 的页面行为不变。
- 键盘（Tab/方向键/F6）、`selected/toggled` aria、焦点环与现有 test id 不变，所有 lens 工具行（Overview/Monitoring、平台 context tabs、Commands/Usage/MCP）统一受益。

### R4 Overview 版式

- **页头**：标题 + 状态行（数据源 / 时间窗 / 本地时区）+ 右侧 7D/30D/90D 范围切换与 Refresh。范围切换上移到页头，作用于整页。
- **KPI 带**：Cost / Requests / Tokens / Sessions 横排 4 格，大数值（24–32px）+ 小标签 + 次要说明（例如“all time”、平均每日）。Cost 为主读数，必须视觉最强。不做同形卡片墙：用一张面板内的分栏加 hairline 分隔。
- **平台带**：只为有记录的平台显示 sparkline；“daily split unavailable / no requests”压缩成次要文字或合并到一个“其他来源”格。保留“无记录给原因、不铺零”的合同。
- **趋势图**：数据起点前的空白区标注“no data before MM-DD”或自动收紧到首个有数据日（二选一，在 design.md 中定）；柱形加小圆角、降低饱和度，图例与平台身份色一致；高度 200–240px。
- **精确值表**：默认收起；展开后**倒序**（最新在上），默认隐藏全零行或折叠为“N 天无记录”；数字使用表格数字而不是斜杠零 mono。现有精确值行焦点、滚动 reveal 与测试合同保留。
- **右栏**：Next actions 与主列等高，或把下方空间交给摘要（数据源健康、最近同步时间等真实数据），不能留死区，也不能编造数字。
- 窄宽（< 1040）时右栏堆到下方，KPI 带 2×2，不横向滚动整页。

### R5 共享 token 与组件

- 所有新增角色（scene 光斑、chrome/sheet alpha、highlight、shadow 组、KPI 字号、tab pill 阴影）只进 `src/theme.rs`，DESIGN.md frontmatter 同步。
- 对比度单测扩展为**合成色**测试，覆盖 light/dark × 渐变两端与中点。
- 数值字体：评估 Segoe UI Variable / 系统 UI 字体的 `tnum` 表格数字特性。可用时，KPI 与表格改用 tabular UI 数字；不可用时保留 mono，但改成不带斜杠零的系统 mono。结论记录在 research。

## 约束

- 不使用 OS 窗口级 Mica / Acrylic / `WindowBackgroundAppearance::Blurred` 透出桌面（本轮范围外，见“开放问题”）。
- 无装饰性动画；场景完全静态，不做每帧重绘。材质不能导致 render 中出现额外分配或计算热点。
- 不改业务数据、领域服务、Tauri 代码；Overview 只能使用已有 report / services 数据，不新增伪造统计。
- 保留键盘、F6、焦点恢复、IME、虚拟化、stable ID、精确值行焦点 reveal；现有测试 id / aria 不破坏，断言更新而不是弱化。
- DPI / 跨显示器矩阵仍为 user-deferred（2026-10-18），不记通过。
- 先改 DESIGN.md，再改 `theme.rs`，最后改视图。

## 验收条件

- [ ] DESIGN.md Elevation / Colors / Components 章节反映新材质合同与成立条件；frontmatter token 同步。
- [ ] 1280×800 浅色：场景渐变与光斑在侧栏和纸面四周肉眼可见；纸面有亮边与漫射阴影；侧栏透出场景色。对照用户截图，原生截图前后对比存档。
- [ ] 深色主题同等成立：无发灰、无脏紫，文字对比度通过。
- [ ] 合成色对比度单测覆盖 light/dark × 场景两端与中点 × 所有承载文字的半透明面。
- [ ] 侧栏选中、hover、键盘焦点三态可区分，选中态无不透明边框；平台图标着色规则一致。
- [ ] Lens tab 为内容宽度胶囊并与标题同行；所有 lens 页面适用；键盘与 aria 测试通过。
- [ ] Overview：KPI 带中 Cost 为主读数；趋势图空白区已处理；精确值表默认收起、倒序、零行折叠；右栏无死区；窄宽堆叠正确。
- [ ] `just desktop-fmt-check`、`just desktop-lint`、`just desktop-test`、`just desktop-test-ui` 全绿。
- [ ] 原生截图：Overview、Claude workbench、Settings 至少三页 × light/dark（en 为主，zh 至少一张验证 CJK）。未运行项明确列出。

## 范围外

OS 窗口级透明 / 系统模糊、真实 backdrop blur 渲染器补丁、macOS/Linux 原生验收、DPI 矩阵、其他页面的版式重排（只吃共享 chrome / token 的收益）、业务逻辑与数据层。

## 已决议（2026-11-07 用户确认）

1. 接受“已知静态低频场景上的半透明材质”论证，推翻上一轮“不引入假玻璃”约束。
2. 不做 Windows 11 Mica / Acrylic / Blurred 窗口级透明探针。
3. 侧栏平台图标默认中性色，hover / 选中时显示平台身份色。
