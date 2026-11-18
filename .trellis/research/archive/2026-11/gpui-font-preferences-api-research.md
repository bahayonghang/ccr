# GPUI 本机字体与字号 API 调研

日期：2026-11-12（America/New_York；UTC 2026-11-13）。对应决策票：[锁定 GPUI 栈支持哪些本机字体与字号能力](https://github.com/bahayonghang/ccr/issues/165)，地图：[GPUI 工作台留白、设置侧栏与字体字号优化地图](https://github.com/bahayonghang/ccr/issues/162)。

基线为 `3870df6b38ec7f1ce64669fae28918abda3a312f`；研究分支 `research/gpui-font-preferences-20261112`。只读产品源码，本分支只保存本报告，不开 PR、不合并。

## 规划可采用的结论

- 现有栈足以提供两套用途独立的 family/字号，复用一个 family 目录和可搜索选择器，并展示预览；**没有完整可靠的“仅等宽字体”公共分类 API**。建议采用共享目录加预览，不能把名字含 Mono 当成分类，也不能把有限样本测量称为认证等宽。[字体 API][text-api]、[类型][text-source]、[主题][component-theme]、[Select][select]
- 主题字号不等于全应用缩放：常规 rem 控件跟随窗口基准，Editor 有独立 mono 字号；CCR 显式 px 和其它语义字号 token 仍需接入。切换主题后必须重新应用同一偏好来源。[root][component-root]、[sizing][component-sizing]、[Editor][editor]、[主题投影][component-theme]、[CCR theme][ccr-theme]
- 本报告仅证明源码/API 能力；实际 Windows/macOS/Linux 字体效果均为 **not_run**。目录存在、family 解析成功、混排不缺字、数字列对齐是不同验收目标。[解析/测量 API][text-api]、[tnum 官方定义][tnum]

## 锁版本与证据取得

[CCR manifest][ccr-manifest] 固定 `gpui-kit =0.7.3`，启用 `component`、`assets`，未启用 `gpui-fast`；[Cargo.lock][ccr-lock] 对应 `gpui-pre 0.3.9`、`gpui-kit/gpui-base/gpui-component 0.7.3`。[Kit 发布清单][kit-manifest] 同样映射到 `gpui-pre =0.3.9`。

直接下载 crates.io 发布源码并计算 SHA-256，以下 9 包均与该 lock checksum 完全一致：`gpui-pre`、`gpui-pre-windows/macos/linux/apple/wgpu` 的 `0.3.9`；`gpui-kit/base/component` 的 `0.7.3`。其中 core SHA 为 `34f61c1cfd36fc9c436af09308c963822ae27a45a84feb21a77ba5c9e051f1c0`，Kit SHA 为 `0fac401f6b149bf33794da0942510240afd29b7b18556574e4cf17196b647066`。

归档格式为 `https://static.crates.io/crates/<package>/<package>-<version>.crate`，例如 [core 发布归档](https://static.crates.io/crates/gpui-pre/gpui-pre-0.3.9.crate)。源码缓存未提交。部分 docs.rs 具体页抓取失败，结论采用 checksum 匹配的发布源码；来源链接固定为相同版本/文件，不以 upstream main 代替。GPUI 发布包记录的上游快照为 `zed@93d745668805029476e6d7fbb19652116fed51de`，仍以发布包实际内容为准。[发布清单][pre-manifest]

## 字体目录与线程

`all_font_names(&self) -> Vec<String>` 是排序去重的可用 family 名，包含 `add_fonts` 注册字体；没有文件路径、face/style、等宽标志或字形覆盖元数据。Windows 合并系统/custom collection 并返回本地化 family 名，因此不能靠英语名字严格匹配证明安装/可用；目录条目也不能证明选后实际用了该 face。`text_system.rs:280–289`、`direct_write.rs:1285–1293,1758–1779`。[API][text-api]、[Windows][windows-text]

`App::text_system()` 返回 `&Arc<TextSystem>`；官方 rustdoc 明确 `TextSystem: Send + Sync`，平台 trait 同样要求 `Send + Sync`。可在前台克隆 Arc 后交给后台枚举/测量/预热，再回前台更新 Entity/Theme；不把 App、Window 或控件移到工作线程。预热文档明确建议后台执行。`app.rs:325–328,2339–2342`、`platform.rs:1494–1535`。[App][app-source]、[自动 trait/预热][text-api]、[平台 trait][platform-source]

没有统一字体目录变化订阅或强制重扫 API。Apple 每次创建 CoreText collection 并附加内存 family；Windows 读已保存 collection，解析失败时尝试刷新；Cosmic 读 FontSystem 数据库。`add_fonts` 会清解析缓存并推进字体 generation，不是系统目录刷新入口。不能承诺热安装/卸载立即更新。建议目录首次使用时后台加载并缓存，本轮不承诺热更新。Kit 私有默认 mono helper 的 `OnceLock` 也只是进程缓存，不能证明 OS 字体永不变化。`Apple text_system.rs:108–139`、`direct_write.rs:339–353`、`cosmic_text_system.rs:131–187`、core `text_system.rs:291–303`、`theme/mono_font.rs:48–83`。[Apple][apple-text]、[Windows][windows-text]、[Cosmic][cosmic-text]、[注册 API][text-api]、[默认 helper][default-mono]

## 等宽分类与测量

公共 `Font` 只有 family/features/fallbacks/weight/style；`FontMetrics` 不包含 monospace 标志；`PlatformTextSystem` 不暴露字体文件、原生 face、OpenType 表或等宽查询。Kit 默认 mono helper 按固定候选名找可用 family，不是全目录分类器。`text_system.rs:1291–1378`、`platform.rs:1494–1535`、`theme/mono_font.rs:19–98`。[类型][text-source]、[trait][platform-source]、[helper][default-mono]

`resolve_font` 后可用 `advance(font_id, size, ch)` 比较样本字形的 advance，缺 glyph 返回错误；`layout_width`/窗口 `shape_line` 可观察 shaping 后宽度。`em_width/ch_width` 只看 m/0。有限 ASCII 样本通过不能证明 CJK、符号、组合字、fallback、不同字重/特征也等宽；且 `resolve_font` 可换到默认栈，不能把测到的 fallback 分类贴给请求 family。`text_system.rs:364–483`。[测量/解析][text-api]、[实现][text-source]

因此“代码用途可选任何 family 并预览”和“仅真实等宽列表”是不同承诺。后者需要额外分类政策/适配；Windows 官方有 `IDWriteFont1::IsMonospacedFont`，但当前 GPUI 公共 trait 未暴露它。[DirectWrite][dwrite-mono]、[GPUI trait][platform-source]

## 更新字体与字号的实际覆盖

| 接入点 | 能力及限制 |
| --- | --- |
| `Theme::update` | 设置 `font_family/font_size/mono_font_family/mono_font_size`，同步 Base 投影并刷新窗口；mode 切换会加载配置，应在加载后重应用偏好。`theme/mod.rs:166–183,288–334`。[主题][component-theme] |
| rem 控件 | Component root 以 UI size 调 `set_rem_size` 并设置 family。Input/Button/Select 的 sizing 主要用 `text_xs/sm/base`，它们是 rem；可跟随此基准。`root.rs:438–451`、`styled.rs:534–575`、`sizing.rs:245–251,337–342`。[root][component-root]、[样式][styled]、[sizing][component-sizing]、[Input][input]、[Button][button]、[Select][select] |
| Editor | 默认读取 mono family/size，使用相对行高；调用方样式仍可覆盖。只设置 family 不会自动应用 mono size。`input/editor.rs:136–155`、`styled.rs:706–736`。[Editor][editor]、[样式][styled] |
| 语义 token | `typography_tokens()` 从默认值开始，只覆盖 sans/mono/md.size/mono_md.size；xs/sm/lg/xl 和 line_height 不会等比重算，Base 默认 token 是 px。`theme/mod.rs:592–599`、`theme_tokens.rs:154–187,206–212`。[投影][component-theme]、[token][base-typography] |
| CCR 固定 px | 根 `application-frame` 的正文仍固定 BODY_SIZE，标题/元数据也有固定 px；不能只改 Theme 字段。`shell/mod.rs:1154–1157`。[CCR shell][ccr-shell] |

CCR `theme.rs:80–112` 已枚举候选并构建正文 fallback；`:551–555` 会重置 family/body/mono size。`shell/mod.rs:184–187` 初始化及系统外观变化、`shell/preferences.rs:89` 偏好应用都重进 apply，后续必须统一应用字体偏好，避免主题切换重置。偏好结构当前为 `Clone + Copy`；添加 String 后需处理 ownership。reader 只接受 v1，需让旧 v1 缺少新增字段时仍能按默认值加载。[theme][ccr-theme]、[shell][ccr-shell]、[偏好应用][ccr-preferences-shell]、[偏好文件][ccr-preferences]

## 选择器复用

`SelectState::new(...).searchable(true)` + `Select::new(&state)` + `SearchableVec` 已够用；用 family 作为稳定 Value，通过 `SelectEvent::Confirm(Option<Value>)` 保存，避免持久化搜索后的 index。`set_selected_value` 先清查询再定位。`select.rs:71–78,152–155,300–304,335–353,649–651`。[Select][select]

`SearchableListItem` 默认按 title 做大小写不敏感子串匹配，可自定义 render 放 sample；delegate 可定制搜索/行显示。默认 SearchableVec 不承诺模糊、拼音或别名检索。List 有 Up/Down/Enter/Escape、虚拟列表、滚动条及选中项滚入视口；Select 有取消/失焦恢复已提交选择、菜单宽高、搜索占位和可访问 label。可以直接复用，不必重造弹层。实际设置入口的中文输入、焦点恢复、长名字、长列表与滚动页面中的弹层仍需验证。`searchable_list/delegate.rs:7–49,88–114`、`vec.rs:169–180`、`list/list.rs:25–34,239–265,552–631`、`select.rs:381–429,610–713`。[条目/委托][search-delegate]、[搜索][search-vec]、[List][list]、[Select][select]

## fallback、tnum 与证据限制

- `resolve_font` 失败会尝试默认栈，全部失败会 panic；`FontFallbacks` 是缺字 cascade，不能当作偏好验证器。Windows、Apple、Cosmic 发布源码都有显式 fallback/特征接入；Linux 当前转接到 Cosmic，不能凭 core 的旧 TODO 就判不支持。它们不保证任意用户字体的 CJK/符号/emoji 覆盖。core `text_system.rs:251–263,364–391`、Windows `direct_write.rs:398–454,501–511,577`、Apple `open_type.rs:28–75,105–153`、Linux `text_system.rs:1`、Cosmic `cosmic_text_system.rs:339–446,1148–1161`。[解析][text-api]、[fallback 类型][fallbacks]、[Windows][windows-text]、[Apple][apple-open-type]、[Linux][linux-text]、[Cosmic][cosmic-text]
- `FontFeatures` 可携带 `("tnum",1)`，Styled 可继承，后端会传给 shaping；CCR 已有 `tabular_figures()`，无需再造机制。`tnum` 是数字到表格数字的可选替换，不使整套字体等宽；参数传递成功不证明当前 face/fallback 有该特征，也不保证小数点/货币符号列对齐。`font_features.rs:6–23`、`styled.rs:712–736`、CCR `theme.rs:400–409`。[特征][features]、[样式][styled]、[CCR][ccr-theme]、[官方定义][tnum]
- **`get_font_for_id` 日志不代表真实 OS 字体。** 它只在 GPUI 解析缓存反查请求 Font 描述符，不查询 OS face。CCR requested/resolved 日志不能证明虚拟 alias、本地化 family 或逐 glyph fallback，更不能替代字体验收。core `text_system.rs:352–361`、CCR `theme.rs:520–534`。[缓存实现][text-source]、[CCR 日志][ccr-theme]

## 最小实施建议与交接

保存 UI family/size、代码用途 family/size 四个独立偏好；共享目录加预览；用已有 Select/List，后台加载目录，前台维护选择。mode/config 加载后统一应用四字段，将受影响的 CCR 文字接入语义字号；保留旧 v1 兼容和失效字体默认显示政策。以上是建议，完整等宽过滤与热安装更新若要承诺，需另列范围。用户已确认两套用途独立；本报告不改变设置左侧栏长页滚动锚点、工作台紧凑披露的布局方向。

本轮只完成版本/checksum/源码/入口审阅，未改产品代码、未运行产品测试或真实用户应用、未安装或下载字体。实现后先验四字段独立应用/保存、旧 v1、主题切换保持、失效字体；再验相关 picker 键盘/取消/焦点/滚动，以及选定字体的中文/ASCII/符号和数字读数。各平台实机效果均为 **not_run**；本报告不替代平台验收。

## 固定版本一手来源

上文行号以对应发布归档内文件为准。

[ccr-manifest]: https://github.com/bahayonghang/ccr/blob/3870df6b38ec7f1ce64669fae28918abda3a312f/ccr-desktop/Cargo.toml
[ccr-lock]: https://github.com/bahayonghang/ccr/blob/3870df6b38ec7f1ce64669fae28918abda3a312f/ccr-desktop/Cargo.lock
[ccr-theme]: https://github.com/bahayonghang/ccr/blob/3870df6b38ec7f1ce64669fae28918abda3a312f/ccr-desktop/src/theme.rs
[ccr-shell]: https://github.com/bahayonghang/ccr/blob/3870df6b38ec7f1ce64669fae28918abda3a312f/ccr-desktop/src/shell/mod.rs
[ccr-preferences-shell]: https://github.com/bahayonghang/ccr/blob/3870df6b38ec7f1ce64669fae28918abda3a312f/ccr-desktop/src/shell/preferences.rs
[ccr-preferences]: https://github.com/bahayonghang/ccr/blob/3870df6b38ec7f1ce64669fae28918abda3a312f/ccr-desktop/src/runtime/preferences.rs
[kit-manifest]: https://docs.rs/crate/gpui-kit/0.7.3/source/Cargo.toml
[pre-manifest]: https://docs.rs/crate/gpui-pre/0.3.9/source/Cargo.toml
[text-api]: https://docs.rs/gpui-pre/0.3.9/gpui/struct.TextSystem.html
[text-source]: https://docs.rs/crate/gpui-pre/0.3.9/source/src/text_system.rs
[app-source]: https://docs.rs/crate/gpui-pre/0.3.9/source/src/app.rs
[platform-source]: https://docs.rs/crate/gpui-pre/0.3.9/source/src/platform.rs
[features]: https://docs.rs/crate/gpui-pre/0.3.9/source/src/text_system/font_features.rs
[fallbacks]: https://docs.rs/crate/gpui-pre/0.3.9/source/src/text_system/font_fallbacks.rs
[styled]: https://docs.rs/crate/gpui-pre/0.3.9/source/src/styled.rs
[component-theme]: https://docs.rs/crate/gpui-component/0.7.3/source/src/theme/mod.rs
[default-mono]: https://docs.rs/crate/gpui-component/0.7.3/source/src/theme/mono_font.rs
[component-root]: https://docs.rs/crate/gpui-component/0.7.3/source/src/root.rs
[component-sizing]: https://docs.rs/crate/gpui-component/0.7.3/source/src/sizing.rs
[base-typography]: https://docs.rs/crate/gpui-base/0.7.3/source/src/theme_tokens.rs
[editor]: https://docs.rs/crate/gpui-component/0.7.3/source/src/input/editor.rs
[input]: https://docs.rs/crate/gpui-component/0.7.3/source/src/input/input.rs
[button]: https://docs.rs/crate/gpui-component/0.7.3/source/src/button/button.rs
[select]: https://docs.rs/crate/gpui-component/0.7.3/source/src/select.rs
[list]: https://docs.rs/crate/gpui-component/0.7.3/source/src/list/list.rs
[search-delegate]: https://docs.rs/crate/gpui-component/0.7.3/source/src/searchable_list/delegate.rs
[search-vec]: https://docs.rs/crate/gpui-component/0.7.3/source/src/searchable_list/vec.rs
[windows-text]: https://docs.rs/crate/gpui-pre-windows/0.3.9/source/src/direct_write.rs
[apple-text]: https://docs.rs/crate/gpui-pre-apple/0.3.9/source/src/text_system.rs
[apple-open-type]: https://docs.rs/crate/gpui-pre-apple/0.3.9/source/src/open_type.rs
[linux-text]: https://docs.rs/crate/gpui-pre-linux/0.3.9/source/src/linux/text_system.rs
[cosmic-text]: https://docs.rs/crate/gpui-pre-wgpu/0.3.9/source/src/cosmic_text_system.rs
[dwrite-mono]: https://learn.microsoft.com/en-us/windows/win32/api/dwrite_1/nf-dwrite_1-idwritefont1-ismonospacedfont
[tnum]: https://learn.microsoft.com/en-us/typography/opentype/spec/features_pt#tag-tnum
