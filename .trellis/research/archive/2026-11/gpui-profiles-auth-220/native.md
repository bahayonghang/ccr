# 配置方案与账户 · Windows 原生验收

## 对照条件

Windows，CCR 7.5.1，默认简体中文。宽窗客户区 1280 × 800 逻辑像素、浅色，窄窗 960 × 700、深色。基线截图包含系统标题栏。直接采集客户区的图片使用 Windows Graphics Capture，系统缩放为 125%。用窗口图像和 UI Automation 分别查验画面与控件文本。归档 WebP 保留采集图像解码后的所有像素。

基线来自 `12e8bb2a`，debug 可执行文件 SHA-256 为 `3196fd2210e4088004cd7b711ff9f210c4a5328e7b99dc41d2cd676e11fd921e`。使用 [测试夹具生成器](../../../../../ccr-desktop/tests/profile_account_fixture.rs) 通过领域服务创建三个独立目录，每个平台有 27 条方案，其中 A-Primary 已应用，B-Preview、C-Apply 可用，D-Disabled 已禁用。账户分别保存 Account-B、Account-A，并令运行文件保留 A。每次从同一个 seed 复制新目录，只使用演示入口。

| 旧版入口 | 宽窗浅色 | 窄窗深色 |
| --- | --- | --- |
| Claude 配置方案 | [首屏](before-profiles-wide-light.webp) | [首屏](before-profiles-narrow-dark.webp) |
| Claude 账户 | [首屏](before-accounts-wide-light.webp) | [首屏](before-accounts-narrow-dark.webp) |
| Codex 账户 | [首屏，含常驻 OAuth 与导入区](before-codex-accounts-wide-light.webp) | |

在仓库根目录用 PowerShell 重建一份演示数据（生成器要求使用全新的子目录）。第二次启动前可另复制整个 seed，便于对照写入范围。

```powershell
$parent = New-Item -ItemType Directory -Force -Path outputs/desktop/profiles-auth-qa
$env:CCR_NATIVE_FIXTURE_ROOT = Join-Path $parent.FullName 'seed'
python ccr-desktop/scripts/desktop.py cargo test --locked --test profile_account_fixture -- --ignored
just gpui-dev --fixture-profiles (Join-Path $env:CCR_NATIVE_FIXTURE_ROOT 'profiles.toml') --fixture-registry (Join-Path $env:CCR_NATIVE_FIXTURE_ROOT 'config.toml')
```

## #221 配置状态

在同一窗口经侧栏依次打开三个平台的「配置方案」。每页各显示 27 条，行内的 A-Primary 与页头的运行配置匹配一致。Claude 额外指出 CCR 标记。此轮观测的 debug 构建 SHA-256 为 `894e87fa2ad8dedb6d57a5d5e7dd3632c521417e03cb0fe35d200466f549ea25`。[Claude](221-claude-matched.webp) · [Codex](221-codex-matched.webp) · [Grok](221-grok-matched.webp)

从夹具外部单独改写 Claude 的托管模型值，再破坏其 JSON，最后将文件路径替换为目录。真实刷新依次显示「运行配置偏离」「无法核对 · 格式无效」「无法核对 · 来源不可读」。页头保留 A 的标记，行内同步反映原因，三次仍能浏览 27 条。六个原生状态的 UIA 树均没有合成凭据或原始差异值。[偏离](221-claude-drift.webp) · [格式无效](221-claude-invalid.webp) · [不可读](221-claude-unreadable.webp)

比对目录、配置、标记、历史及已存凭据等 31 个既有文件的 SHA-256，首次三平台浏览后均一致。SQLite 读连接新增了零字节的 WAL。后续三次人工改造分别在改造后、点击刷新前重新取快照，每次刷新前后的 SHA-256 清单完全一致。清单不计窗口偏好、锁文件与 SQLite 共享内存读者索引。

## #222 方案目录与卡内应用

1280 × 800 浅色首屏为三列。点击 B-Preview 后出现两列卡片和 B 的详情，当前仍为 A-Primary。点击 C-Apply 的「应用」后页头与卡片均指向 C，右栏保留 B，中文等待与成功回执点名 C。SQLite 标记、Claude 托管模型和凭据分别与 C 一致。点击 D 的「启用并应用」，D 变为启用、标记与运行设置均切至 D。通过目录浏览与预览前后对照，31 个既有文件内容一致。[首屏](222-profiles-wide-light.webp) · [A 当前、B 预览](222-a-current-b-preview.webp) · [应用 C](222-c-applied-b-preview.webp) · [启用并应用 D](222-d-enabled.webp)

960 × 700 深色首屏为两列。按 East 筛选命中 9/27，摘要仍显示 A；滚至 Lab-17、打开单页详情，Esc 返回同一张卡并保留滚动位置及 East 条件。按 Enter 从原卡打开详情，再次返回，Tab 可单独聚焦「应用 Lab-17」。[深色首屏](222-profiles-narrow-dark.webp) · [滚到 Lab-17](222-narrow-scrolled.webp) · [单页详情](222-narrow-detail.webp) · [返回焦点](222-narrow-return-focus.webp)

按 `theme.rs` 的实际 sRGB 颜色计算，在卡面、选中底和悬停底三种背景中取最小对比度。浅色辅助文字为 4.77:1、控件边界 3.51:1、焦点 14.96:1；深色依次为 6.53:1、3.49:1、9.86:1。状态文字使用各自不透明的徽章底色，由主题的 `status_tones_keep_text_contrast_wherever_a_tag_is_placed` 回归覆盖。

第一次窄窗原生键盘验收发现，虚拟行在按键按下与抬起之间重排时会漏掉一次应用。修复后的 debug 构建 `b4486af0d105372c515d91fef0dd81837e5c9d53cc0bf692e8ef26e030a35cfd` 在同一 East 过滤和滚动位置重验。Esc 返回后 Tab 到独立应用按钮，Enter 使 SQLite 标记变为 Lab-17、运行模型变为 demo-model-17，当前列表内可见成功回执。宽窗重验也显示了 B 详情的全掩码及 C 的具名回执。这三张图用 Windows Graphics Capture 直接采客户区；125% 缩放下 960 × 700、1280 × 800 逻辑像素对应 1200 × 875、1600 × 1000 物理像素。[操作焦点](222-retry-action-focus.webp) · [键盘应用结果](222-keyboard-applied.webp) · [C 的焦点与 B 的全掩码详情](222-fully-masked.webp)

同一键盘修复构建在 960 × 700 英文深色、20px 字号验证了 Codex 与 Grok 的配置入口。两者都是 27/27，A 的文件匹配贴近名称；目录收为一列，首卡完整显示字段和独立 Apply，工具栏换行。[Codex 大字号](222-codex-large.webp) · [Grok 大字号](222-grok-large.webp)

## #223 三平台的运行账户与选择

1280 × 800 浅色，从 Claude、Codex、Grok 的账户 tab 分别进入。三者初次均显示运行账户 A，均无预选行。选择 B 后，辅助技术同时读到 A 是当前账户且未选中、B 已选中，以及具名的切换和删除。Codex 还提供重命名。Grok 的摘要陈述运行文件存在。通过 Windows 原生按钮分别切换后重新解析三个运行文件，与各自的已存快照逐一比对：

| 操作后 | Claude | Codex | Grok |
| --- | --- | --- | --- |
| 只浏览或选择 B | A | A | A |
| Claude 切到 B | B | A | A |
| Codex 切到 B | B | B | A |
| Grok 切到 B | B | B | B |

初次浏览和选择时 31 个既有文件的 SHA-256 未变，唯一新项为零字节的 SQLite WAL。显式切换会重读并清除行选择。以上宽窗构建 SHA-256 为 `28035cb46486a430adf90234bf91dd774fb50871095714bfc542ab96fd3fcf57`。[Claude 首屏](223-accounts-wide-light.webp) · [Claude A 当前、B 选中](223-a-current-b-selected.webp) · [Codex](223-codex-selected.webp) · [Grok](223-grok-selected.webp)

960 × 700 深色使用确认框修复构建 `fc3fd07b573c5a4bf7d700ea7ef6057c2b5e92724aed5dedc4f225b1b4715cf1`。保留 A 运行和 B 选中，打开删除可读到「删除已保存账户 B」与保留运行登录的后果。初始焦点在取消，Esc 回到「删除 Account-B」，再按 Enter 重新打开同一确认。清除运行认证的确认点名 **A** 并说明已存账户仍保留。Windows UIA 的 AlertDialog 可直接读出各自对象和后果。展开保存表单，输入合成名称、取消、确认丢弃，焦点回到「保存当前登录」，重新打开的原生输入值为空。上述取消动作前后 31 个既有文件均未变化。[窄窗](223-accounts-narrow-dark.webp) · [删除 B](223-delete-target-dark.webp) · [清除 A](223-clear-target-dark.webp)

在独立副本中，Claude 保留运行凭据并移除已存账户，页面同时显示「没有匹配」和「没有已保存账户」。Codex 移除运行 auth.json 后仍可看到 A/B；Grok 损坏 JSON 后显示无效并保留 A/B。将 Grok 文件替换为目录并手动刷新后，页面显示无法读取，可重试。前面三项浏览前后 27 个既有文件不变，故障注入后、刷新前后的文件清单完全一致。[Claude 空账户](223-edge-claude.webp) · [Codex 缺运行凭据](223-edge-codex.webp) · [Grok 格式无效](223-edge-grok.webp) · [无法读取](223-unreadable.webp)

Codex 夹具将凭据存储显式设为 keyring，同时保留 A/B 两个已存文件。原生页报告「此凭据存储不读取账户凭据」，没有给任何账户加当前标记。[存储不支持](223-store-unsupported.webp)

另取干净副本明确确认两个破坏性动作。删除 Claude 已存 B 后，运行凭据 SHA-256 逐字节未变，仍匹配 A。随后确认清除运行 A，页面显示未检测到凭据，磁盘仍保留 A 的已存快照，Codex/Grok 的运行哈希与账户清单在两次操作中均未变。

在最终 #223 构建 `0af9a6a758c9ae7284cce9af0f614a5cd6442959bef26bebbad7c1f5eb3ad526` 原生重验了只读存储。选择 B 和刷新都可用，Windows UIA 同时报告保存、清除、切换 B、重命名 B、删除 B 五个文件写入动作 disabled。[只读来源的 B 与操作状态](223-readonly-actions.webp)

## #224 Codex 按需账户操作

在 Windows 1280 × 800 浅色和 960 × 700 深色从 Codex 的「账户」进入。首页依次读到运行凭据、已保存账户、四个工具入口。选择 B 后进入配额，目标明确区分当前运行、Account-B 和全部。演示模式下三个外部查询都禁用，初始状态为尚未查询。在窄窗用 Esc 返回后按 Enter 再次打开同一入口。一次早期实现打开此页导致 Windows 栈溢出，保留默认线程栈修复后用 debug 构建重新操作并正常退出。该次截图构建 SHA-256 为 `1c1c0c9e607d4bf074775c8c79d6e20825787604a762fa1af13d320bd536bf6f`。[宽窗首页](224-default-wide.webp) · [窄窗首页](224-home-narrow-dark.webp) · [选中 B 的配额](224-quota.webp) · [窄窗配额](224-quota-narrow-dark.webp)

登录入口显示演示边界，诊断入口显示回调端口预检与自有监听器的清理审阅。展开导入，输入合成 JSON 后显示掩码，Windows 的 ValuePattern 和 TextPattern 均未读到合成凭据。带草稿转到配额先弹出丢弃确认，Esc 回表单，确认才离开。浏览这四项与取消前后，31 个文件的 SHA-256 完全相同。[登录](224-login.webp) · [诊断](224-diagnostics.webp) · [导入草稿](224-masked-draft.webp)

冻结源码的 debug 构建 `f3a7efc2361f0fbda7290e20ff4e8f6c9c3024c39496fe1352f1b60236bf9626` 上，先选 B，在「添加／导入」单独输入 Fixture-Added 和合成 API JSON，再明确确认。新增账户出现在保存列表，运行 A 和既有 A/B 快照的字节均未变。同一窗口从登录按 Esc，再按 Enter 回到登录，退出码为 0。[新增后的账户列表](224-added-account.webp)

页面定向测试 14/14 通过。受控替身统计浏览及展开的请求数为零，验证 OAuth 回调先进入待提交、显式提交才尝试令牌交换、取消等待清理。领域所有者另验证成功提交及在途清理。三个配额目标分别记录，0%、缺失、获取／重置时间、部分失败及保留历史分别断言。覆盖离开后的旧结果、SSH 文件操作的能力边界和诊断归属。冻结版 `just gpui-test-ui` 为 41 项通过，1 项按仓库设置忽略。演示窗口没有连接真实 OAuth 或配额服务。

## #220 同一 release 的 Profiles → Auth

在 Windows 11 26300 的同一个进程中，经侧栏和真实标签页完成以下操作。使用最终 `just gpui-ci` 构建的 release，SHA-256 为 `fe552d4adbe29c3f16212e4c30cd8cf61c6060d38e3537ae2d997b1eb9fa1a50`。主仓库基线 `12e8bb2aa6b56715d525b3e2734152eb48892f96`。本次工作区未提交。

| 用户操作 | 配置方案页或文件 | 运行账户与文件 |
| --- | --- | --- |
| 首次进入 Codex，再预览 B | SQLite 与 CCR 注册表均为 A-Primary，模型 demo-model-00，预览仍可浏览 B | 仍精确匹配 Account-A，运行文件、注册表和已存账户的摘要均相同 |
| 点 C 卡内的「应用」 | 两处标记变为 C-Apply，模型变为 demo-model-02，界面显示 C 已应用，详情仍是 B | 该合成方案写入自己的 API 凭据，账户页报告未匹配到唯一的已存账户 |
| 在账户页选 B | 两处标记、配置文件及模型与刚应用后相同 | 选中 B，仅浏览，运行凭据及两个已存快照的 SHA-256 一致 |
| 明确切换到 Account-B | 切换按 Codex 既有语义同步登录配置，模型仍是 demo-model-02，标记仍为 C | 运行凭据与已存 B 完全相同，界面显示 B 当前，两个已存快照均未变化 |
| 返回「配置方案」 | 页面重新检查 config.toml，显示「运行配置无匹配」。切换账户已移除自定义 model_provider，模型及历史标记并不足以认定整份运行配置仍匹配 C | 仍匹配 B，返回操作未改前一步文件 |

[预览 B，当前 A](220-release-preview-b.webp) · [独立应用 C](220-release-applied-c.webp) · [从方案进入账户，选 B](220-release-account-preview.webp) · [明确切换后 B 当前](220-release-account-switched.webp)

本次串走另行生成一对 S0 合成目录，两个页面共用 gpui 端的 `home/.codex/config.toml` 与 `home/.codex/auth.json`。用 `ccr-app` 的 `generate_s0_s1_s2_pairs` 生成器产生新根，在该 S0 的 gpui 与 tauri 两端补入相同的 27 条 Codex 方案、A/B 的账户快照、运行配置和注册表。补入前后分别比对两端原清单中四个核心数据库的 SHA-256。通过 `--benchmark-fixture-root <S0/gpui>` 启动；比对工具仅输出方案名、账户标签、模型和文件摘要。窗口正常退出。Windows 宽窗浅色及窄窗深色由子票分别验收，本次跨页联验使用 1280 × 800 浅色。SSH 部分使用隔离传输和错误注入；生产 OAuth、生产配额、真实 SSH 主机与 macOS/Linux 窗口不在本次实测范围。

### 集成检查

`just gpui-test-ui` 41 项通过、1 项按仓库设置忽略；`just gpui-ci` 通过，含 409 个主库测试通过、4 个忽略与 release 构建。`just ci` 17/17 步通过，耗时 23 分 44 秒，含 Rust、Tauri、前端测试／覆盖率、依赖审计及 VS Code 扩展检查。前端 174 个文件、959 项测试通过。Rust 审计报告仓库允许的 9 个 workspace 和 18 个 Tauri 警告。前端审计为 0 条通报、0 个活动例外；本轮删掉了已被上游撤回的 picomatch 例外，保留并验证既有四个依赖补丁。[上游撤回记录](https://github.com/advisories/GHSA-fc5q-39xc-vjpf)

最终三道门禁前后，2,778 个源码及构建输入文件的聚合 SHA-256 相同。本次早期直接使用 Cargo 曾让本机包装器将桌面 target 改为符号链接，已恢复为项目内实体目录。最终检查与构建全部通过仓库自有入口验证其位置。[门禁和构建摘要](checks.json) · [跨页各步的文件摘要](shared-runtime.json)

GitHub 最终逐票读回 #221、#222、#223、#224、#220 均为 CLOSED / COMPLETED。父票的四个子项及两条依赖已另行核对。[GitHub 状态、关闭时间和验收评论](github.json)
