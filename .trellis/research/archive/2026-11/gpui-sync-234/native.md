# GPUI 同步工作台 · Windows 原生验收

规格 [#234](https://github.com/bahayonghang/ccr/issues/234)。集成分支为
`codex/gpui-sync-234`，基线为本地 dev 的
`916328e5d254f2d7d37d47fa88c51d0b8a170701`。原工作区已有的设计和 Trellis
改动仍在原处。

## 实施前

用 `just gpui-dev` 从基线启动 GPUI，使用复制的合成 catalog、独立偏好与
合成 WebDAV 配置。配置处于已启用、尚未测试状态，普通文件夹含长路径。
Windows Graphics Capture 采集的整窗画面带约 2 像素窗口边框和 31 像素
标题栏，偏好指定的客户区为 1280×800 和 960×640 逻辑像素。四次采集都从左侧
「同步」进入，停在滚动起点。Windows 可访问文本均未找到合成用户名或密码。
debug 构建 SHA-256 为 `7f5b17db59140d5c43523bca32f0045cf097e7720d5dc4f29018029c9b2f4a86`。

| 客户区 | 浅色 | 深色 |
| --- | --- | --- |
| 1280×800 | [基线](before-wide-light.jpg) | [基线](before-wide-dark.jpg) |
| 960×640 | [基线](before-narrow-light.jpg) | [基线](before-narrow-dark.jpg) |

默认宽度首屏只看到第一项配置资产和部分资产操作。最小宽度的文件夹路径溢出阅读区。
源码为 Rust，design-atlas 扫描器退出 2，报告无可扫描文件。

## #235 WebDAV 首屏、设置与执行环境

验收源码 `050021c90c094b37e3c04e8fd946994cb2faf91d`，原生 debug 可执行文件
SHA-256 `4ece5a22832e7d38d1af35017624fdb921a52fffd26059d619ef00abac05fa41`。
通过 `just gpui-dev --fixture-sync-loopback`，为每个窗口传入独立的合成 catalog 和注册表。

- 1280×800 浅色首屏只有一个主标题。当前执行环境、三个独立的设置动作、五项资产和普通文件夹入口可见。测试服务在打开及重读后保持原有的 1 条健康检查记录。显式测试后新增 1 条 `PROPFIND /` 207，设置服务返回 503 后重新测试新增 1 条 503，失败替换先前的成功。去运行概览后返回同步，当前环境仍显示失败。
- 1280×800 在右栏显示断开确认，列出清除凭据、删除旧连接配置及保留文件夹的范围。焦点从确认的「取消」用 Tab 到「确认」，再回到「取消」。Esc 收起右栏，把焦点还给「断开 WebDAV」。
- 960×640 深色首次打开显示「未配置」。在单列设置表单第一次只输入合成地址，提交后显示无效设置。补全合成用户名后 Tab 到「保存设置」，按 Enter 创建规范文件。摘要显示「已保存 · 已停用」，状态提示已保存，焦点环与 Windows 可访问焦点均返回「编辑 WebDAV 设置」。该过程未增加 loopback 请求。
- 960×640 启动预选的不可达合成 SSH 环境，在同步侧栏仍显示离线提示。实际点击「切回本地」才显示该夹具中的本地 WebDAV 摘要，连接状态为尚未测试。已连接 SSH 的权限和确认边界由隔离传输、运行时及 Shell 检查覆盖。
- 窗口快照和可访问读回未出现已保存的合成用户名或密码。实际清理、保留字段与文件夹定义、读取损坏配置及写入修订冲突由隔离运行时和控件测试验证。

| 原生场景 | 证据 |
| --- | --- |
| 宽窗已配置且尚未测试 | [1280×800 浅色](235-wide-light-loopback.jpg) |
| 连接测试 | [成功](235-connection-success.jpg) · [重测失败](235-connection-failed.jpg) |
| 断开确认 | [右栏、安全焦点与后果](235-disconnect-confirm.jpg) · [Esc 后焦点](235-disconnect-focus-return.jpg) |
| 首次配置 | [未配置](235-narrow-unconfigured.jpg) · [窄窗表单](235-narrow-settings.jpg) · [拒绝不完整设置](235-narrow-save-rejected.jpg) · [保存后焦点](235-narrow-save-focus.jpg) |
| 执行环境 | [离线 SSH 保留](235-ssh-offline.jpg) · [显式切回本地](235-return-local.jpg) |

| 本地检查 | 结果 |
| --- | --- |
| `just gpui-lint` | 0。修复前一次聚合门禁报告了仅测试使用的导入，移动该导入后复跑通过。 |
| `just gpui-test-ui` | 0，46 项通过、1 项忽略。 |
| `python ccr-desktop/scripts/desktop.py cargo test --locked --lib sync -- --skip export_bindings` | 0，28 项通过。 |
| 同一入口使用 `--features native-sync-fixture --lib native_fixture_requires_an_isolated_catalog_and_exact_marker` | 0，1 项通过。 |
| `just gpui-ci` | 0，主库 419 项通过、4 项忽略，其他测试目标及 Windows release 构建通过。 |
| `cargo test --locked -p ccr-sync --all-features -- --skip export_bindings` | 0，77 项及 4 个 doctest 通过，1 项忽略。将 Cargo 与其编译器固定在同一工具链，清理集成 worktree 的可重建根 target 后复验通过。 |
| `cargo clippy --locked -p ccr-sync --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | 0。 |

Standards 与 Spec 两位独立审查者复查了修复。确认焦点、保存后焦点、SSH 授权丢失后重测
和路径读取失败均有回归覆盖。人工走查浅深主题的实色阅读面、布局与焦点环，错误和确认风险
带可读文字。系统高对比度、减少动态效果、其他系统、生产 WebDAV 和真实 SSH 主机未实测。
父票还需完整首屏矩阵、大字、英文及资产和文件夹端到端走查。

## #236 配置资产与远端分布

候选 `24bf5e8aba71ac61317e3f528a12383fc0f6c7d2`。`just gpui-dev --fixture-sync-loopback`
启动的候选 debug SHA-256 为 `62ad663a208ad3214a422dc610c233948bef36902b3865f39077c5514b00b0db`。

- 1280×800 浅色从真实侧栏打开同步，首屏含 CCR、Claude Code、Codex 共五项固定资产、保护方式、两种独立检查和文件夹入口。缺少探测时仅显示状态文案。[最终默认首屏](236-final-five-assets-wide.jpg)
- 较早候选 `a5162f4f` 的宽窗检查首先显示本地盘点结果。显式远端检查产生一组 207、200、两次 404 和一次 503。可访问状态为存在 2、缺失 2、未知 1，与五行一致。随后合成服务全部返回 503，旧条和旧计数消失。[宽窗三态](236-mixed-remote-wide.jpg) · [失败](236-remote-failed.jpg)
- 最终候选在 960×640 深色复验独立本地读取、键盘显式远端检查及三态计数。打开详情后显示完整路径和保护方式，焦点进入「返回资产列表」，Esc 还给 `settings.json` 原行。单项传输打开口令表单，Esc 返回传输发起按钮。[窄窗计数](236-final-mixed-narrow.jpg) · [详情](236-narrow-detail-focus.jpg) · [返回](236-narrow-row-return.jpg) · [传输入口](236-transfer-opener-return.jpg)
- 修复前，实际点击资产行后焦点跳到侧栏，Esc 无法返回。修复将焦点安排在详情和返回列表挂载后的帧中。回归测试先执行 Windows 的回调再绘制，复现并验证这一顺序。[修复前证据](236-native-focus-before-fix.jpg)
- 读取异常的本地资产仍可获得独立 WebDAV 证据。无 WebDAV 配置的本地盘点、连接中 SSH、切换环境、探测中配置变更、逐项未知及失败由合成传输与运行时测试覆盖。截图和 Windows 可访问文本未含已存凭据。

`just gpui-lint` 及候选上的 `just gpui-test-ui`、`just gpui-ci` 通过。同步定向 35 项通过，
UI 门禁 46 项通过、1 项忽略。完整 GPUI 门禁主库 426 项通过、4 项忽略，并完成 Windows
release。共享 `ccr-sync` 77 项及 4 个 doctest 通过，1 项忽略，严格 Clippy 通过。
共享 crate 源码和锁文件从复验提交到候选一致。Spec 复查确认远端探测独立于本地路径，
资产入参仍绑定执行环境与 WebDAV 修订，原传输入口保留。

生产 WebDAV、真实 SSH、系统高对比度、减少动态效果与其他系统未实测。

## #238 普通文件夹

候选 `24bf5e8aba71ac61317e3f528a12383fc0f6c7d2`。原生 debug SHA-256 与 #236
相同。Windows 使用同一隔离 WebDAV，并独立准备了三项文件夹，其中一项停用。

- 960×640 深色列表中长路径可省略，打开详情可读取完整本机路径与远端路径。最终候选的焦点进入内容宽度的「返回同步列表」，Esc 返回原来的 Lab notes 详情按钮，并恢复列表位置。[详情焦点](238-final-detail-focus.jpg) · [返回原行](238-final-row-return.jpg)
- 较早候选 `a5162f4f` 中，上传所选、下载全部已启用和移除确认均检查了可见范围、初始「取消」焦点以及 Esc 返回。下载范围只显示已启用的两项，说明 HTTPS 原内容传输和替换前备份。[单项确认](238-single-review.jpg) · [批量确认](238-enabled-batch-review.jpg) · [返回发起按钮](238-batch-review-return.jpg) · [移除确认](238-remove-safe-focus.jpg)
- 最终候选实际执行批量上传，WebDAV 收到两个已启用项的 PUT，分别返回 201 和 503。界面显示 1/2 部分完成和逐项失败原因。请求日志中没有停用目标。[批量结果](238-final-batch-partial.jpg)
- 最终候选在 1280×800 浅色右栏显示完整路径。实际下载所选文件夹得到 GET 200，界面显示 1/1 和备份位置。读回目标与先前上传的合成文本一致，备份 SHA-256 与覆盖前文件一致，为 `F6F83669F5A303B2A7A52F8E416E59DE0E9DDE5724B30F8E7A52557DD1F55B3D`。[宽窗详情](238-final-wide-detail.jpg) · [下载与备份](238-final-wide-backup.jpg)
- 新增、编辑、启停、删除、修订或目标身份失效、本机与 SSH 范围、混合结果由合成控件、运行时和传输测试覆盖。复查确认移除与详情焦点在控件实际挂载后生效。原生验证用的合成服务把 MKCOL 改为返回 201，与仓库受控 WebDAV 测试一致。

同一候选上的同步定向 35 项、`just gpui-test-ui` 46 项和 `just gpui-ci` 主库 426 项
通过，分别忽略 0、1、4 项测试。完整门禁也完成 Windows release。原生验证使用合成 HTTP
loopback 和本机文件。生产 HTTPS WebDAV、真实 SSH、系统高对比度、减少动态效果和其他
系统未实测。


## #237 配置资产传输

候选 `8d6d551855d3e8a2780f1e82add9689a8ebe9f55`。通过 `just gpui-dev --fixture-sync-loopback` 启动，debug SHA-256 为
`DBA131B5928B5BB1228128ED52F58F8962CDA6DAE3B15DBF974F1D16BA75000D`。

- 1280×800 浅色核对 Claude Code 的 `settings.json`、本地环境、WebDAV 地址、资产根路径及上传后果。Enter 进入确认，口令显示圆点，覆盖默认关闭。Windows 可访问文本、配置和运行日志未出现合成口令。[掩码确认与显式覆盖](237-final-wide-confirm.jpg)
- 在较早候选 `68ec2cb3` 完成首次上传，在最终候选显式允许覆盖并复跑。最终合成 WebDAV 收到 MKCOL 201、PUT 201。读回对象为 `ccr-sync-encrypted-asset` v2、AES-256-GCM，资产 ID 为 `claude-settings`。界面显示运行中，离开后回到本地同步页，保留逐项成功 1/1。[运行中](237-final-wide-running.jpg) · [返回原环境](237-final-wide-retained.jpg)
- 在 960×640 深色、英文 UI 字号 20 下，较早候选的 Tab 把焦点移到视口外。最终候选自动滚动，完整显示覆盖、v1 和确认按钮的焦点环。两项权限默认关闭，键盘单独切换覆盖后仍保留 v1 关闭。下载确认分别说明文件备份、CCR Platforms 数据库事务回滚和 v1 仅适用于实际下载。Shift+Tab 回到口令框，Esc 恢复「Transfer all assets…」的焦点和滚动位置。预览、更改选项及取消前后的合成 WebDAV 请求数均为 33。[修复前](237-narrow-focus-before-fix.jpg) · [英文大字](237-final-narrow-large-list.jpg) · [覆盖焦点](237-final-overwrite-focus.jpg) · [v1 焦点](237-final-v1-focus.jpg) · [确认焦点和后果](237-final-confirm-focus.jpg) · [反向 Tab](237-final-return-to-input.jpg) · [Esc 返回](237-final-escape-original.jpg)
- 控件和运行时夹具检查了默认冲突、覆盖后双端均存在时从当前环境发布、批量部分成功与备份、配置失效禁止请求、环境结果、离页及改方向主动清空口令。SSH 夹具覆盖单项入口和全部资产限制。SSH 文件确实替换时，备份留在主机，现有适配器未提供可显示的备份路径。

候选同步定向 39 项通过，`just gpui-test-ui` 46 项通过、1 项忽略，`just gpui-ci`
主库 430 项通过、4 项忽略，并完成 Windows release。`ccr-core` 凭据 8 项测试及严格
Clippy 通过。独立 Standards 与 Spec 确认同主机目标区分、脱敏与口令清理，最终键盘修复
通过 Standards 复查。生产 HTTPS WebDAV、真实 SSH、系统高对比度、减少动态效果与其他
系统未实测。
