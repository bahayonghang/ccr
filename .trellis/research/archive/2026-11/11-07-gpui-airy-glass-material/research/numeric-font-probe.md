# 数值字体探针：系统 UI 字体 + `tnum`

日期：2026-11-07，Windows 11 Pro 10.0.26300，gpui-pre 0.3.9 / gpui-pre-windows 0.3.9。

## 结论

Overview 的 KPI 读数与精确值表改用系统 UI 字体并开启 OpenType `tnum`（`theme::tabular_figures()`），不再使用带斜杠零的系统 mono。Windows 上已通过真实 DirectWrite 整形确认数字等宽；未做原生像素截图，macOS/Linux 未验证（字体不支持 `tnum` 时该特性为 no-op，退回字体默认数字）。

## 证据

1. **代码路径**：`Styled::font_features`（`gpui-pre-0.3.9/src/styled.rs:714`）写入 `TextStyle.font_features`，随 `Font.features` 参与字体解析；`gpui-pre-windows-0.3.9/src/direct_write.rs:456-511` 为每个 FontInfo 生成 `IDWriteTypography`，`apply_font_features`（:1797-1830）对 `liga/clig/calt` 以外的任意标签调用 `AddFontFeature`，并在 :577/:615 `SetTypography` 到文本区间。`tnum` 未被过滤。
2. **字体数据**（fontTools 读取 `C:/Windows/Fonts`）：
   - `SegUIVar.ttf` 默认数字宽度 `1104, 776, 1104…`（“1”为比例宽），GSUB 含 `tnum`，映射后全部为 1104。
   - `segoeui.ttf` 默认数字已全部为 1104（默认即等宽），同样含 `tnum`。
   - `consola.ttf` 等宽，无 `tnum`。
3. **真实 DirectWrite 整形**（临时 example 用 `gpui_kit::platform::application()` 的平台文字系统，不打开窗口，`TextSystem::layout_width` 在 100px 下逐字测量；运行后已删除 example）：

   | 请求族名 | 解析结果 | 默认 “1” | `tnum` “1” | 其他数字 |
   | --- | --- | --- | --- | --- |
   | Segoe UI Variable | Segoe UI | 53.91 | 53.91 | 53.91 |
   | Segoe UI Variable Text | Segoe UI Variable Text | 37.89 | 53.91 | 53.91 |
   | Segoe UI Variable Display | Segoe UI Variable Display | 37.45 | 53.91 | 53.91 |
   | Consolas | Consolas | 54.98 | 54.98 | 54.98 |

   注意：`headless()` 平台使用 Noop 文字系统（所有字符 0.6em、族名回落），不能作为证据；Kit headless 测试同理，只能断言 `font_features` 已设置，不能证明字形宽度。

4. **附带发现**：`all_font_names()` 列出的是 `Segoe UI Variable Text/Display/Small`，不含裸 `Segoe UI Variable`，因此 `theme::native_font` 当前在 Windows 上选中 `Segoe UI`（默认数字已等宽）。本轮不改字体选择（范围外）；`tnum` 让未来切换到 Variable 实例时仍保持等宽。

## 未运行

原生像素截图、macOS（SF / PingFang）与 Linux（Noto / DejaVu）`tnum` 行为、CJK 混排数字宽度。
