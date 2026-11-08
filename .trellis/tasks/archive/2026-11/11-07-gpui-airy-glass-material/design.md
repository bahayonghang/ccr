# 技术设计：GPUI 轻盈毛玻璃材质

## 1. 材质成立论证（替换旧“实色降级”结论）

旧结论（`archive/.../10-17-gpui-03-shell/research/airy-glass-capabilities.md`）：没有 backdrop 采样 API，所以只能做实色降级。这里的推理默认把“玻璃”等同于“采样任意背景并模糊”。

新论证：材质背后的像素完全由本应用绘制，而且只有两类低频内容：
- 160° 两色线性渐变（`theme::field`）。
- 若干静态柔光斑（用大 blur 阴影绘制，本身已经高度模糊）。

对这些内容再做一次高斯模糊，结果与原图的差异低于 8-bit 量化误差。因此 `alpha < 1` 的纸面直接合成到场景上，在像素上等价于 `backdrop-filter: blur()`。**成立条件**（写入 DESIGN.md，代码评审时强制执行）：

1. 半透明材质的父链上不能存在文字、图标、图片、列表等高频内容。
2. 半透明材质不能互相叠放（chrome 与 sheet 并列，而不是嵌套）。
3. 承载正文的阅读面（表格、输入、弹层、代码区）一律实色。

这不是“假玻璃开关”，而是在已知背景下精确实现的材质。OS 级 Mica / Blurred 仍然不用。

### 已核实的 GPUI 0.3.9 能力

| 能力 | 证据 | 用途 |
| --- | --- | --- |
| Hsla alpha 背景 | 现有 `hairline()` 已在用 | chrome / sheet 半透明 |
| 多重 BoxShadow + `inset` | `gpui-pre-0.3.9/src/style.rs:349-360`、`styled.rs:34 box_shadow_style_methods!` | 亮边高光（inset 0 1px 白）+ 漫射阴影 |
| `linear_gradient` | `color.rs:865` | 场景、亮边渐隐 |
| 无 `radial_gradient` | `color.rs` 中无此函数 | 光斑改用“小圆 div + 大 blur 同色阴影” |
| `font_features` | `styled.rs:714`、`gpui-pre-windows direct_write.rs:456-502` | 尝试 `tnum`，实现前先做原生探针 |
| 圆角裁剪为矩形 | DESIGN.md 已记录 | 子背景仍需自行设置内侧圆角 |

## 2. Token 设计（`src/theme.rs`）

新增 `Material` 结构，替换 `solid_sheet`：

```rust
pub struct Material {
    pub scene: Background,          // 渐变场
    pub glows: [(Point<f32>, f32, Hsla); 2], // 相对坐标、半径、颜色（静态）
    pub chrome: Hsla,               // 侧栏/工具胶囊：light 白 40–48%，dark #1d292c 55%
    pub sheet: Hsla,                // 主纸面：light 白 72–78%，dark #172225 82–88%
    pub reading: Hsla,              // 实色阅读面：light #fff，dark #1d292c
    pub highlight: Hsla,            // 亮边：light 白 80%，dark 白 8%
    pub sheet_shadow: Vec<BoxShadow>, // inset 0 1 highlight + 0 20 48 漫射 + 0 2 6 近影
    pub inset: Pixels,              // 场景边距
}
```

场景边距公式：`clamp((min(spare_w, spare_h)) / 8, 0, 20)`，其中 spare = viewport − MIN。1280×800 时约 20px，960×640 时为 0。

光斑：light 用 mint `#bfeadb` 和 lavender `#d9cdf5`，alpha 约 0.55；dark 用 `#1f4a40` 和 `#3a2f66`，alpha 约 0.5。位置固定在左上（侧栏后方）和右下，半径约为视口短边的 0.35。这样侧栏透出薄荷色，右下纸面透出淡紫色，形成方向光。

数值是起始值，以合成对比度测试和原生截图为准微调，最终值同步到 DESIGN.md frontmatter。

### 对比度测试改造

`contrast()` 增加 `composite(fg_alpha_color, bg)`。对每个半透明面，分别在 `field[0]`、`field[1]`、中点、光斑峰值色四个背景上合成，然后断言：`text ≥ 12`、`secondary ≥ 7`、`muted / primary / 状态色 ≥ 4.5`、`control ≥ 3`。光斑峰值是最不利情况，必须覆盖。

## 3. Shell 结构

```
root (scene 渐变)
 ├─ glow × 2（absolute，pointer 透传，无 a11y 节点）
 └─ frame（inset 边距，flex row，gap 0）
     ├─ sidebar  — chrome 材质，左侧圆角 20，右侧亮边 1px
     └─ sheet    — sheet 材质，右侧圆角 20，sheet_shadow
          ├─ page header 行（标题 + lens 胶囊 + 页面操作）← 由各页面提供，或 shell 提供标题
          ├─ body（阅读面实色在各 feature 内部）
          └─ status bar（透明，继承 sheet）
```

方案选择：侧栏与纸面**并列**，而不是让侧栏嵌在纸面里，满足“不嵌套玻璃”。替代方案是整张 sheet 半透明、侧栏透明。这样做会让侧栏与内容没有材质差，层次更平，因此不采用。

### Lens 胶囊与标题合并

现状：shell 在 body 上方独立渲染 `lens_bar`（`shell/mod.rs:836-856`），而 Overview 等页面自己渲染标题，所以两者位于不同的 entity。

设计：shell 保持 lens 状态的唯一所有权，但改为输出一个 `Option<AnyElement>` 胶囊；页面头部通过一个共享的 `page_header(title, status, trailing)` helper（新文件 `src/shell/page_header.rs`）渲染。shell 把胶囊作为 `trailing` 的一部分注入页面。为避免跨 entity 传递元素，采用**折中做法**：shell 自己渲染 header 行（标题取 `Destination` 的本地化名称，胶囊在右侧）；页面不再渲染重复的 h1，只保留状态行和页面操作。Overview 的 Refresh 和范围切换放在页面内容的首行右侧，与胶囊视觉对齐。

- 胶囊：容器 `chrome` + 1px highlight 边、radius pill、padding 3px；选中项 `reading` 实白 + 轻阴影（0 1 3 黑 8%）+ primary 文字；未选中项透明，hover 时 chrome 加深一阶。
- `semantics::tab` 的 `selected/toggled`、tab stop 和焦点环逻辑保留，只改颜色和边框。

### 侧栏选中态

`nav_button`（`shell/mod.rs:619`）：selection 改为 `reading.opacity(0.7)` 的 pill 面 + highlight inset 阴影，去掉 primary 边框；文字和图标用 primary；hover 用 `reading.opacity(0.35)`；focus 仍为 2px ring。平台图标默认中性，选中或 hover 时使用身份色（推荐方案，待评审确认）。

## 4. Overview 版式

```
[页头] Overview · ● Local usage · 90 days ending 2026-11-07       [7D|30D|90D] [Refresh]
[KPI 面板 — 单面板 4 栏 hairline 分隔]
  COST $1,021.42 (主)  | REQUESTS 16.3K | TOKENS 2.9B | SESSIONS 417 all time
[平台带] 有记录平台 sparkline 格 + "其他来源" 汇总格
[主列: 趋势图面板 + 折叠精确值表]               [右栏: Next actions + 数据源摘要]
```

- 范围切换从 `usage_panel` 移到页头，`set_days` 逻辑不变。
- KPI：Cost 用 32px/600，其他用 24px/500；标签 12px muted；次要说明行使用已有字段（例如 days、all time），不计算新指标。如果需要显示“日均”，只能由已有总数 ÷ 天数推导，并明确标注。
- 趋势图空白：**推荐“自动收紧到首个有数据日前 1 天”**，并在 x 轴起点标注“since MM-DD”。90D 的含义（查询窗口）保留在状态行。替代方案“灰色区 + 文字”会保留 2/3 空白，不采用。
- 精确值表：`show_exact` 默认 false（已是 false；截图中处于展开状态）；展开后行序倒置，连续全零日折叠为一行“MM-DD – MM-DD · no records”。焦点 reveal 的 index 映射必须随之调整，并补充测试。
- 右栏：`actions` 下追加“数据源”小节，复用 `status_text` 已有信息（configured / last refresh），不新增查询。用 `items_stretch` 让右栏与主列等高。

## 5. 兼容与回滚

- 所有改动位于 `ccr-desktop`：`theme.rs`、`shell/mod.rs`、`shell/semantics.rs`、新文件 `shell/page_header.rs`、`features/overview.rs`、DESIGN.md 和测试。
- 回滚点：材质（theme + shell root）、侧栏 / tab、Overview 三个提交相互独立，可以单独 revert。
- 性能：光斑和阴影都是常量，每帧只多 2 个 shadow primitive 和若干 alpha quad，不新增分配热点。

## 6. 风险

- 半透明 sidebar 上的 muted 文字落在光斑峰值处时对比度可能不足 → 测试覆盖峰值；不足时调低 chrome alpha 的透明度（提高不透明度），而不是加深文字。
- Kit 组件内部背景（Button ghost、List）可能自带不透明底，破坏材质 → 实现时逐个确认 `c.sidebar`、`c.background`、`c.title_bar`、`c.status_bar` 的映射。
- `tnum` 在 Segoe UI Variable 上可能不生效 → 原生探针；失败时保留 mono，但改用无斜杠零字体（Cascadia Mono 默认零形或 Segoe UI Mono），并记录到 research。
