# GPUI 原生能力探针（S1 研究）

Registry 根 `$REG = C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`。
crate 目录已确认：gpui-pre-0.3.9（lib 名 `gpui`）、gpui-pre-platform-0.3.9、gpui-pre-windows-0.3.9、gpui-component-0.7.3、gpui-kit-0.7.3、gpui-kit-assets-0.7.3。
仓库品牌资产现成：`ccr-ui/public/icons/icon.svg`（1024 viewBox，蓝色互扣环 mark）、`logo.svg`、`src/assets/favicon.svg`；`ccr-ui/src-tauri/icons/icon.ico` 为已生成的多尺寸 ICO，直接复用。

## 1. 自定义 SVG 资产（API 探针，非生产品牌接入方案）
- 结论：`AssetSource` trait 在 gpui 核心；`Application::with_assets` 是唯一注册口，单槽替换式（无 set_asset_source、无内置组合源）。可同时服务 Kit+自定义资产：写一个组合 struct 即可。
- 证据：`$REG/gpui-pre-0.3.9/src/assets.rs:13-19`（`fn load(&self, path:&str)->Result<Option<Cow<'static,[u8]>>>`、`fn list(&self,path:&str)->Result<Vec<SharedString>>`）；`src/gpui.rs:105` `pub use assets::*`（即 `gpui_kit::AssetSource`）；`src/app.rs:203-210` `pub fn with_assets(self, asset_source: impl AssetSource) -> Self`（直接赋值 asset_source 并重建 SvgRenderer，二次调用=替换）；核心仅 `impl AssetSource for ()`（assets.rs:21），无 Composite。
- Kit 范例：`$REG/gpui-kit-assets-0.7.3/src/native_assets.rs:22-37` RustEmbed 实现；注意未知路径返回 **Err**（非 Ok(None)）。`icon_assets!` 宏（lib.rs:71-91）文档明说可与 Assets 组合。
- API 探针：`svg().data(include_bytes!(...))` / `Icon::default().data(...)` 可接受字节，但不适用于当前 embedded-PNG 品牌 carrier。生产侧栏直接嵌入现有 app PNG，见末尾实施结论。以下组合资产源代码仅说明通用 SVG 能力，本轮未采用。
- 降级：`svg().external_path()` 走 fs::read 磁盘加载（svg.rs:46、293-302），仅适合开发期，发布不可靠。
```rust
struct CcrAssets;
impl gpui_kit::AssetSource for CcrAssets {
    fn load(&self, path: &str) -> gpui_kit::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if path == "icons/ccr-mark.svg" {
            return Ok(Some(std::borrow::Cow::Borrowed(include_bytes!("../assets/ccr-mark.svg"))));
        }
        gpui_kit::assets::Assets.load(path).or_else(|_| Ok(None)) // Kit 未知路径报 Err，归一化
    }
    fn list(&self, path: &str) -> gpui_kit::Result<Vec<gpui_kit::SharedString>> {
        gpui_kit::assets::Assets.list(path)
    }
}
```

## 2. Windows 窗口/exe 图标
- 结论：无运行时 API。`WindowOptions.icon` 仅 X11 生效；Windows 窗口类图标来自 **exe 内嵌资源 ID 1**，当前未嵌入→系统默认图标。
- 证据：`$REG/gpui-pre-0.3.9/src/platform.rs:2555-2556` `pub icon: Option<Arc<image::RgbaImage>>`（注释 "X11 only"）；TitlebarOptions 无 icon（platform.rs:2692-2702）；WindowParams.icon 在 Windows 标 dead_code（platform.rs:2611-2613）。`$REG/gpui-pre-windows-0.3.9/src/platform.rs:262-263` `load_icon().unwrap_or_default()`；:1633-1647 `LoadImageW(module, PCWSTR(1 as _), IMAGE_ICON, …)` = MAKEINTRESOURCE(1)；`src/window.rs:1426-1440` `register_window_class` 设 `WNDCLASSW.hIcon`。全 crate 无 WM_SETICON/set_window_icon。gpui 自身 build.rs 只嵌 manifest（resources/windows/gpui.rc：`1 RT_MANIFEST`，与 ICON 类型不冲突）。
- 环境探针：`ccr-desktop/Cargo.lock:2130` 已有 **embed-resource 3.0.12**（gpui-pre/gpui-fast 的 build-dep）；winresource 不在 lock。windres 存在：`C:\Users\lyh\scoop\apps\mingw\current\bin\windres.exe`；rc.exe 存在：`C:/Program Files (x86)/Windows Kits/10/bin/10.0.26100.0/x64/rc.exe`。
- 推荐：ccr-desktop 加 `build.rs` + `[build-dependencies] embed-resource = "=3.0.12"`（版本已锁定，零新增传递依赖），rc 写 `1 ICON "assets/ccr.ico"`；直接复制现有 `ccr-ui/src-tauri/icons/icon.ico`，不新增 SVG 转换或图标生成流程。tradeoff：引入构建期资源编译（rc.exe 由 vswhom 自动探测，本机已具备；windres 可兜底）；一次嵌入同时解决 exe 图标+窗口类图标。
- 降级：不嵌图标则维持系统默认；运行时 WM_SETICON 需 unsafe 拿 HWND 自绘，gpui 未暴露口，不建议。
```rust
// build.rs（Windows-only）
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/ccr.rc", embed_resource::NONE); // ccr.rc: 1 ICON "ccr.ico"
    }
}
```

## 3. dashed border
- 结论：**原生支持**，无需等距 div 段 hack。`Styled::border_dashed()` 存在且 Windows DirectX shader 已实现。
- 证据：`$REG/gpui-pre-0.3.9/src/styled.rs:500-503` `fn border_dashed(mut self) -> Self`（设 `BorderStyle::Dashed`）；`src/scene.rs:597-603` `enum BorderStyle { Solid=0, Dashed=1 }`；`$REG/gpui-pre-windows-0.3.9/src/shaders.hlsl:670-699`：dash=2×边框宽、gap=1×边框宽，沿周长均分（1px 边框→2px 段/1px 隙，浏览器行为）。
- 仓库先例：ccr-desktop/src 无 `border_dashed`/dashed 使用（grep 0 命中）。
- 推荐：空基线直接 `div().border_1().border_dashed().border_color(token)`；注意 dash 密度随边框宽缩放，且大圆角与 dashed 组合不佳（window.rs:4682 注释），基线用 1px+小/零圆角。
- 降级：仅当 shader 路径异常才退回 1px div 段行（目前无此必要）。
```rust
div().w_full().border_1().border_dashed().border_color(cx.theme().border)
```

## 4. overflow_x_scroll
- 结论：存在，但挂在 `InteractiveElement`（非 Styled）；需元素有 id（StatefulInteractiveElement 容器）。
- 证据：`$REG/gpui-pre-0.3.9/src/elements/div.rs:768` `pub trait InteractiveElement`；:1529-1532 `fn overflow_x_scroll(mut self) -> Self`（overflow.x=Scroll）；Styled 的 overflow 宏只生成 hidden 系（gpui-pre-macros-0.3.9/src/styles.rs:129-157）。
- 仓库先例：只有 `overflow_y_scroll`（shell/mod.rs:999、features/install/tools.rs:299 等），无 x 向先例。
- 推荐：横向趋势/表格容器 `div().id("…").overflow_x_scroll()`，子内容给 min_w 触发滚动。降级：无。
```rust
div().id("trend-x").overflow_x_scroll().child(div().min_w(px(720.)).child(chart))
```

## 5. pill 按钮
- 结论：单按钮 `.rounded(px(9999.))` 即 pill，不动全局 theme.radius；仓库已有统一入口。
- 证据：`$REG/gpui-component-0.7.3/src/button/button.rs:24-31` `enum ButtonRounded { None, Small, Medium(default), Large, Size(Pixels) }`；:33-37 `From<Pixels>`；:340-343 `pub fn rounded(mut self, rounded: impl Into<ButtonRounded>) -> Self`；:646-652 render 映射：`Size(px)=>px` 原样，其余按 `cx.theme().radius` 倍数。
- 仓库先例：`ccr-desktop/src/shell/semantics.rs:17` 已对装饰按钮统一 `ButtonRounded::Size(px(theme::PILL_RADIUS))`；`theme.rs:22` `PILL_RADIUS=9999.`。
- 推荐：所有 pill 走 semantics::button 装饰器或直接 `.rounded(px(theme::PILL_RADIUS))`。降级：无。
```rust
Button::new("save").rounded(px(theme::PILL_RADIUS)) // 等价 ButtonRounded::Size(px(9999.))
```

## 6. Kit Icon 自定义 path（API 探针，非生产品牌接入方案）
- 结论：`svg().path()` 依赖资产源（§1），但 `.data(bytes)` 完全绕过；Kit `Icon` 支持 IconName 之外的自定义 path **和** 原始字节 → 侧栏 mark 不被资产注册阻塞。
- 证据：`$REG/gpui-pre-0.3.9/src/elements/svg.rs:53-62` `pub fn data(mut self, data:&[u8])`（hash 缓存键，paint 时直传字节）；:182-186 path 分支→`window.paint_svg`（window.rs:4983）→`svg_renderer.rs:255-261` `asset_source.load(&params.path)?`。`$REG/gpui-component-0.7.3/src/icon.rs:117-120` `pub fn path(impl Into<SharedString>)`（Assets bundle 路径如 `icons/foo.svg`）；:136-139 `pub fn data(&[u8])`（免注册）；`$REG/gpui-kit-assets-0.7.3/src/icon.rs:7-9` `trait IconNamed { fn path(self)->SharedString }` 可为自定义图标集实现，`From<T: IconNamed> for Icon`（component icon.rs:59-63）。
- 生产结论：当前品牌 icon.svg 是 embedded-PNG carrier；GPUI SVG 的着色 mask 不保留该原色方案，因此用现有 PNG，不单色化或重绘。下方 `Icon::data` 代码仅证明 API 字节入口，可用于普通单色功能图标，不能作为 CCR 品牌接入配方。
- 降级：组合资产源 + `Icon::new(…path…)`（§1 snippet）。
```rust
use gpui_kit::component::Icon;
Icon::default().data(include_bytes!("../assets/ccr-mark.svg")).size(px(20.))
```

## 2026-11-07 实施结论（优先于上方 API 探针建议）

- §1/§6 的 `svg().data` / Kit Icon 仅说明 API 可接受字节，不能证明它适合当前品牌：`icon.svg` 是 embedded-PNG carrier，GPUI SVG 是着色 mask。实际品牌使用 `gpui_kit::img(Arc<Image::from_bytes(ImageFormat::Png, include_bytes!(../../../ccr-ui/public/icons/icon.png))>)`，保留原有蓝色及透明角；构建期嵌入，无运行时文件访问/下载/第二资产源。没有 SVG 重绘或单色化。
- 当前 SVG carrier 与初始未跟踪的 `assets/ccr-mark.svg` 字节相同；后者未被代码使用，已删除重复副本。PNG 直接引用仓库现有生成资产，无新增栅格生成流程。
- `assets/ccr.ico` 与 `ccr-ui/src-tauri/icons/icon.ico` SHA-256 均为 `af47b53ab2a9290974bd6a54a09934e1220b7b124e8a12c249189e6338c0926a`。来源与许可：本仓库 MIT 品牌生成资产；生产主图及来源由 `.trellis/spec/ccr-ui/frontend/brand-asset-pipeline.md` 管理。
- Windows `build.rs` 编译 `assets/ccr.rc` 的 ID 1 ICON。资源编译失败会使构建失败，不能静默交付默认图标。`embed-resource = =3.0.12` 已在独立 workspace 锁内，只新增 ccr-gpui -> embed-resource 构建依赖边，不升级传递包。
- `theme::PANEL_RADIUS` 为 12px；`theme::hairline` 是装饰线 alpha，`TabColors::border` / 输入边界仍为不透明 control；`semantics::tab` 保留 selected/toggled、Kit 键盘与焦点行为，使用独立选中面/主色文字/6px 角。通用 ghost selected 也从 sheet 改为 overlay，避免其他局部选择器同色。
- pill 已由现有 `semantics::button` 实现，不改全局 radius。分段控件先经 button 装饰器再经 tab，避免 pill 覆盖 tab 角度。
- 状态栏保留真实环境/来源文案；配置就绪点仅表示已配置，不表示查询/SSH 登录成功；独立 Clock entity 每 30 秒更新 HH:mm，销毁时任务随 entity 取消，不刷新业务页。
- 源码/编译/headless 证据与原生可见性分别记录；本实施未运行 Windows 标题栏/任务栏/Alt-Tab 像素检查，也未运行 macOS/Linux 或延期矩阵。
