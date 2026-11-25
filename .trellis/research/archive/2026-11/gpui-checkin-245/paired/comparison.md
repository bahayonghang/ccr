# Windows 签到首屏对照

2026-11-25，在 Windows 11 build 26300 的独立桌面，从真实侧栏首次进入「签到管理」，等待读取完成后截取窗口。八对图在同一台机器上使用相同的中文界面、125% 缩放、14/12 默认字号、逻辑客户区与滚动位置。八份候选 UIA 收据均能定位「本机签到」。数据由基线的 `checkin-pair-v1` 生成。每对已配置图的输入库逐字节相同；每次查看后该库哈希未变。无库场景未创建数据库。十六张图各自的 SHA-256 均已对照同名 JSON 核验。

基线为 `4f0a02387a221f9f1822eede2787ebe050aa75da`，使用普通配对夹具。候选为 `131d2acef02133022b0f59e051c8a49c942cd2f4` 的 Windows MSVC debug 二进制，启用 `native-checkin-fixture`，SHA-256 为 `0f4da49d1edcf3357fe806aa3eac98774065c4dfd52583ddfddf812053116e5e`。候选使用 `--fixture-checkin-loopback` 和同目录的 `--fixture-profiles`、`--fixture-registry`，在编译检出的 `outputs/desktop` 下校验专用 marker。为保持数据完全可比，首屏沿用基线的两个演示提供商、三个合成账户和同一组余额；本组只读浏览没有发起作业。真实回环请求在主验收记录中另有收据。[基线构建与生成方法](../baseline/baseline.md)

| 数据状态 | 逻辑客户区 | 主题 | 改版前 | 改版后 |
| --- | --- | --- | --- | --- |
| 无库 | 1280×800 | 浅色 | [基线](../baseline/before-unconfigured-wide-light.png) | [候选](after-unconfigured-wide-light.png) · [收据](after-unconfigured-wide-light.json) |
| 无库 | 1280×800 | 深色 | [基线](../baseline/before-unconfigured-wide-dark.png) | [候选](after-unconfigured-wide-dark.png) · [收据](after-unconfigured-wide-dark.json) |
| 无库 | 960×640 | 浅色 | [基线](../baseline/before-unconfigured-narrow-light.png) | [候选](after-unconfigured-narrow-light.png) · [收据](after-unconfigured-narrow-light.json) |
| 无库 | 960×640 | 深色 | [基线](../baseline/before-unconfigured-narrow-dark.png) | [候选](after-unconfigured-narrow-dark.png) · [收据](after-unconfigured-narrow-dark.json) |
| 三个账户 | 1280×800 | 浅色 | [基线](../baseline/before-configured-wide-light.png) | [候选](after-configured-wide-light.png) · [收据](after-configured-wide-light.json) |
| 三个账户 | 1280×800 | 深色 | [基线](../baseline/before-configured-wide-dark.png) | [候选](after-configured-wide-dark.png) · [收据](after-configured-wide-dark.json) |
| 三个账户 | 960×640 | 浅色 | [基线](../baseline/before-configured-narrow-light.png) | [候选](after-configured-narrow-light.png) · [收据](after-configured-narrow-light.json) |
| 三个账户 | 960×640 | 深色 | [基线](../baseline/before-configured-narrow-dark.png) | [候选](after-configured-narrow-dark.png) · [收据](after-configured-narrow-dark.json) |

## 首屏观察

无库时，两种尺寸均可直接看到「本机签到」、账户分段、无库说明及唯一的绿色「添加提供商」。基线入口为通栏的「管理提供商与账户」。有账户时，候选在首次进入、未滚动的位置展示「一键签到」「刷新余额」、按币种分行的保存余额、搜索与提供商筛选、三条账户行。基线默认展开第一个账户的详情。

样本的 CNY 为余额 0、额度 20、已用 20，USD 为余额 62.5、额度 80、已用 17.5。两组各覆盖 1/3 的账户，另 1/3 无余额快照。宽窄截屏均保留三组区别。960×640 下清单单列，长英文名、第三行和最右侧行内操作位于客户区内。明细阅读面为实色，标题与金额先于时间及覆盖说明可读。本组只比较仓库支持的两个窗口尺寸。

## 像素对照

在宽窗无损 PNG 中，分别取字形实心笔画与紧邻背景的 RGB，按 sRGB 相对亮度计算对比度。输入框边缘取外侧背景。数据只覆盖下列控件、文字及状态。

| 取样处 | 浅色 前景／背景 | 比值 | 深色 前景／背景 | 比值 |
| --- | --- | ---: | --- | ---: |
| 余额标题和数值 | `#1e2932 / #ffffff` | 14.81:1 | `#f0f8f5 / #1d292c` | 13.84:1 |
| 覆盖数及时间 | `#5c6b76 / #ffffff` | 5.50:1 | `#9fafa9 / #1d292c` | 6.53:1 |
| 一键签到按钮文字 | `#ffffff / #156f4e` | 6.15:1 | `#10191b / #85d5b1` | 10.32:1 |
| 搜索框外缘 | `#787f89 / #f9fcfc` | 3.92:1 | `#737b86 / #172326` | 3.76:1 |

另与前一次原生复验候选 `9fbdc2cb` 的八张对应首屏逐像素比较，排除窗口底部含时钟的 65 物理像素后，七张完全一致。深色窄窗的已配置清单只在列表右侧空白区的 (1188, 633) 出现一个像素变化，绿色通道由 35 变为 34。八组原生 UIA 标签与数据均一致，表中颜色取样也一致。

## 复采

在 Windows 图形会话中使用 Python、Pillow 和 pywin32。以 [capture-paired.py](capture-paired.py) 的 `--repo` 指向构建该 feature exe 的检出根目录，指定 `--exe`、`--out`、`--prefix after`、`--commit <源码 SHA>`。它复用基线的生成器与窗口捕获代码，补齐候选的配对文件和 marker。每张截图保留完整 DWM 窗框及单独收据；成本较高的实际请求走主记录里描述的回环服务。
