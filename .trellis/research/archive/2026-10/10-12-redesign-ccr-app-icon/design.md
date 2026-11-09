# 汇流图标

## 行为差

现在的图标是深色圆角方块上的字母 C、双弧和尖括号。选定后的图标是饱和蓝底上的汇流：两条白线收到顶部一个白点。

几何的源在 `branding/*.svg`。Windows 没有 Cairo 时，出图走 `ccr-ui/scripts/generate_icons.py` 的 `render_native_brand_master`，这段代码现在按旧弧线手绘。两处必须使用同一组坐标。

## 几何

画布 `0 0 1024 1024`。

- 蓝 `#1A73E8`，标记 `#FFFFFF`
- 应用圆角方块：`x=48 y=48 width=928 height=928 rx=224`
- 展示圆角方块：`x=32 y=32 width=960 height=960 rx=236`
- 左路线：`M 156 848 C 200 640 370 420 512 292`，线宽 `112`，圆头
- 右路线：`M 868 848 C 824 640 654 420 512 292`，线宽 `112`，圆头
- 汇合点：圆心 `(512, 228)`，半径 `72`，盖住两条路线的交点
- 标记裁在圆角方块内。不画渐变、投影、内描边。
- `vscode-icon.svg` 只用上述路线和圆点，颜色为 `currentColor`，不带蓝底。

## 会改的文件

- `branding/master.svg`、`app-icon.svg`、`display-logo.svg`、`vscode-icon.svg`、`branding/README.md`
- `ccr-ui/scripts/generate_icons.py` 的色板和手绘几何
- 生成器拥有的 PNG、ICO、ICNS、SVG 副本

## 不做

不改界面色板、页面、版本号和 `ccr-ui/ref/`。
