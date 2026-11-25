# GPUI 签到改版前首屏

规格 #245、首屏 #246、原生复验 #252。八张图片均为 Windows 原生「签到管理」窗口，从真实侧栏首次进入并等待读取完成。每张 PNG 的同名 JSON 保存构建、数据、尺寸、截图 SHA-256、页面通过 UIA 暴露的文字，以及读取前后数据库的摘要。

## 同机条件

- 基线源码为 `4f0a02387a221f9f1822eede2787ebe050aa75da`。本地构建检出为 `e108b8eb3e67acc6431cdd9fd5fc88013d761b45`，其相对基线的唯一跟踪文件差异是 `ccr-desktop/DESIGN.md` 增加 10 行。构建前后复核了代码相同。实际编译、版本、特性、可执行文件 SHA-256 和锁文件信息见 [build.json](build.json)。
- Windows 11 专业版 build 26300，DPI 120（125%），中文，浅／深主题，应用默认系统字体，UI 14、等宽 12，作用域「本地」。逻辑客户区 1280×800 与 960×640，物理客户区分别 1600×1000、1200×800，含原生标题栏的 DWM 扩展框分别 1602×1039、1202×839。
- 使用独立的 Windows 桌面对象，未调用 SwitchDesktop。测试进程在独立桌面创建原生窗口，仅抓取该窗口的 DWM 框，不使用全屏截图。用 UI Automation 找到实际侧栏「签到管理」并打开，不滚动账户视图。两条路径均从新启动的应用进入，没有上一次选中项或滚动位置。
- 使用普通 `--fixture-profiles` 与 `--fixture-registry`，不带 feature 或专用 marker，不使用个人配置或密钥。首次打开时没有 `checkin.db`。配置态从基线表定义生成合成数据库，下述三个账户的凭据字段均为空。每次采图均比较数据库读取前后摘要，并检查未生成 WAL/SHM、密钥、锁、迁移目录。每次退出清除单次临时夹具。
- Windows 原生标题栏和右下角时钟属于实拍。复采时钟可不同。旧版无库空态句子可在图中直接读到。此构建未把该段文字暴露为有名称的 UIA 实体，JSON 中记录的是它旁边的管理入口。

| 状态 | 宽浅 | 宽深 | 窄浅 | 窄深 |
| --- | --- | --- | --- | --- |
| 无库首次配置 | [1280 浅](before-unconfigured-wide-light.png) | [1280 深](before-unconfigured-wide-dark.png) | [960 浅](before-unconfigured-narrow-light.png) | [960 深](before-unconfigured-narrow-dark.png) |
| 三个合成账户 | [1280 浅](before-configured-wide-light.png) | [1280 深](before-configured-wide-dark.png) | [960 浅](before-configured-narrow-light.png) | [960 深](before-configured-narrow-dark.png) |

## 数据口径

夹具 ID 为 `checkin-pair-v1`。仅首次配置态缺库，配置态有两个提供商、三个账户、两个余额快照。普通 fixture 的 profile/catalog 路径作为一对明确传入，签到浏览不需要 profile catalog。`fixture_pair.py` 从 Git 历史读取基线 `crates/ccr-db/src/database/schema.rs`，不读取已安装应用的数据文件。

| 账户 | 提供商 | 状态 | 余额 / 额度 / 已用 |
| --- | --- | --- | --- |
| 晨光测试账户 | 极光演示站 | 启用 | 62.5 / 80 / 17.5 USD |
| Harbor International Research Sandbox | Harbor Demo | 启用 | 0 / 20 / 20 CNY |
| 备用账户 | 极光演示站 | 停用 | 无快照 |

两个快照的时间均为 2026-11-20 08:30 UTC。两种货币分别查看。旧版默认展开第一个账户的明细。

## 复采

在可使用本地交互式 Windows 图形会话的计算机上，从仓库根目录执行。需有 Git、PowerShell、能构建仓库的 Rust/Cargo 工具链，以及 Python、Pillow、pywin32。本机版本为 Python 3.14.7、Pillow 13.0.0、pywin32 312。将指南、[夹具生成器](fixture_pair.py)、[原生采图](capture-windows.py) 和 [UIA 读取器](window-tree.ps1) 放在同一目录，下面的 `$evidenceDir` 填该目录路径。

```powershell
$repo = (git rev-parse --show-toplevel).Trim()
$tree = Join-Path $env:TEMP ('ccr-checkin-source-' + [guid]::NewGuid())
git worktree add --detach $tree 4f0a02387a221f9f1822eede2787ebe050aa75da
Push-Location $tree
try {
    just desktop-build
    if ($LASTEXITCODE -ne 0) { throw '桌面基线构建失败' }
} finally { Pop-Location }
$exe = Join-Path $tree 'ccr-desktop/target/release/ccr-gpui.exe'
if (!(Test-Path -LiteralPath $exe -PathType Leaf)) { throw '基线工作树中缺少可执行文件' }
Get-FileHash -Algorithm SHA256 -LiteralPath $exe
$run = Join-Path $env:TEMP ('ccr-checkin-evidence-' + [guid]::NewGuid())
python (Join-Path $evidenceDir 'capture-windows.py') --repo $tree --exe $exe --out $run
```

候选复采沿用同一脚本、夹具 ID 和窗口条件。将 `--repo`、`--exe` 指向候选，并加上 `--prefix after --commit <实际构建的完整 SHA>`。`--state` 接受 `unconfigured` 或 `configured`，`--size` 接受 `wide` 或 `narrow`，`--theme` 接受 `light` 或 `dark`，可用来单独复拍。脚本识别候选空态的「添加提供商」和配置态第一个账户，如候选 UIA 名称变化，先核对实际入口再调整匹配。此普通夹具用于首屏视觉配对。#248 / #252 的回环作业使用各自的配对数据和显式开发参数。

如需单独生成可交互的普通夹具，`--root` 必须是尚不存在的临时子目录。生成器打印应用需要的两个参数。完成后关闭该窗口，删除此次新建的夹具。

```powershell
$fixture = Join-Path $env:TEMP ('ccr-checkin-manual-' + [guid]::NewGuid())
python (Join-Path $evidenceDir 'fixture_pair.py') --repo $repo --root $fixture --state configured --size narrow --theme dark
& $exe --fixture-profiles (Join-Path $fixture 'profiles.toml') --fixture-registry (Join-Path $fixture 'config.toml')
```
