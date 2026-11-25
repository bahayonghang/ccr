# #252 多账户日常路径和默认发布版拒网

在 Windows 11 build 26300 上，对源码提交 `131d2acef02133022b0f59e051c8a49c942cd2f4` 的两个独立构建运行原生验收。两个窗口都由独立 Windows 桌面中的进程绘制，以指定窗口为目标发送键盘和鼠标消息，全程保持共享机器当前前台桌面。实际窗口为中文浅色、14 px 字号和 120 DPI，逻辑客户区 1280×800，物理客户区 1600×1000。截图使用 PrintWindow(2) 读取 DWM 窗框，保留 1602×1039 像素原尺寸。

## 开发构建的日常路径

debug 二进制启用 `native-checkin-fixture`，SHA-256 为 `0f4da49d1edcf3357fe806aa3eac98774065c4dfd52583ddfddf812053116e5e`。在证据目录执行 `python checkin_consent_fixture.py --repo "<候选构建使用的检出根>" --delay-ms 700`，再用脚本打印的 `--fixture-checkin-loopback --fixture-profiles <fixture>/profiles.toml --fixture-registry <fixture>/config.toml` 启动。夹具 `checkin-ycr9j1n5` 位于该检出的 `outputs/desktop`，含值为 `ccr-gpui-checkin-fixture-v1` 的 `.ccr-gpui-checkin-fixture` 文件。两个提供商均为无内置身份的自定义数值回环端点。该次导入文件及脚本的 SHA-256，以及真实端口、确认对象、SQLite 前后读数、请求方法和四类状态均记录在[开发构建收据](daily.json)。

| 连续操作 | 观察 |
| --- | --- |
| 从侧栏进入签到管理 → 导入导出 → 预览本地文件 → 跳过冲突 → 提交 | 初次进入时 Windows UIA 显示名为「本机签到」的分组。共 5 个合成账户，导入和浏览期间请求 0。 |
| 账户 → 搜索 Expired → 该行「签到」→ Esc 取消 → Enter 重开 → Tab、Enter 确认 | 筛选后仅一行。取消后请求仍为 0。两次确认均仅含 Expired、Check-in demo 和 `fixture-103`。执行后 GET 和 POST 各 1 次，均为 401，行内显示 `cookie_expired`。 |
| 保持筛选 → 一键签到 → 确认 4 个启用账户 | 操作期间一键签到禁用。Ready 成功（GET、POST 200），Today 今日已签到（GET 200），Expired 失败（GET、POST 401），Paused provider 发出 0 个请求，结果为跳过 `provider_disabled`。Disabled 未入列。[四态与筛选行](daily-checkin-results.jpg) |
| 清空搜索 → 选择 Ready 和 Expired → 刷新所选余额 → 确认 | Ready 实际 GET 200，余额 1 USD、总额度 1.2、已用 0.2；样本与账户成功刷新时间同为 `2026-11-25T15:13:18.691210100+00:00`，汇总为 1/5 有样本。Expired 的 GET 为 401，返回合法 JSON，失败行显示「失败 · Expired · Check-in demo (fixture-103) · cookie_expired · API error: 余额请求失败（HTTP 401 Unauthorized）」。它在操作前后均无余额样本，也没有成功刷新时间。[混合余额](daily-balance-results.jpg) |
| 签到记录 → 失败 → 输入 `fixture-103` → 修复凭据 → Esc → Enter | 两条失败记录均属于 Expired。第一次打开和取消后第一次按 Enter 再开，都核对了编辑器载入的账户名；原修复按钮返回前后 Y 为 701→701 像素。逐项检查编辑框的可读值，Session 与 API 用户标识均未出现明文。[失败记录](daily-failed-records.jpg) · [掩码编辑器](daily-masked-editor.jpg) |
| 账户 → Ready → 详情 → Enter 返回 → Enter 重开 → Enter 返回 | 当日精确值为「2026-11-25 · 已签到 · 实测 · 1 USD · 额度 1.2 · 已用 0.2 · 来源 2026-11-25 · 奖励 1 USD」。窗口中返回按钮实测宽 84 物理像素，客户区宽 1600。历史筛选、编辑取消和查看详情期间新增 0 请求，签到和余额记录一致。[账户详情及键盘焦点](daily-account-detail.jpg) |

合成服务在整个开发路径收到 9 个请求。Windows UIA 读取了 65 次，检索 Name、HelpText、ItemStatus、Value 和可读取的 Text 等属性，未发现合成 Cookie 明文。两个敏感框在截图中呈圆点，额外的输入框检查也未读得 API 用户标识或 Cookie 明文。Windows UIA 对两框的 IsPassword 都返回 false，屏幕阅读器如何提示这两个输入框仍未验证。

## 默认发布版的两项独立检查

同一源码的默认 release，SHA-256 为 `49915ac8ee44ad0ab7bdbe43954f0797c752a179c70417123724cba4e369edc6`。另一个新启动的仓库合成服务生成了 `checkin-_ea2yvyi`，marker 和配对精确匹配。[默认构建收据](release.json)

1. 提供专用开发 flag 和完整配对参数，进程退出码为 2，标准错误只列出默认构建接受的参数，签到数据库未创建，服务新增请求为 0。
2. 用同一对合法 fixture 文件，省略专用开发 flag，启动默认版真实窗口。从侧栏进入，预览并提交五账户配置，搜索 Ready，点击并确认单行签到。终态是「失败 · Ready · Check-in demo (fixture-101) · 演示数据模式禁止请求提供商。 · 提供商明细未知」。服务整个生存期收到 0 个请求，SQLite 中的签到记录为 0。[默认发布版真实拒网](release-network-denied.jpg)

默认窗口共 17 次 UIA 观察。两个构建的应用标准输出／错误日志均为空，夹具数据库文件也未发现合成 Cookie 明文；标准错误中的命令行用法提示单独记录在 flag 测试。两窗口及监听器已退出，两套临时夹具已删除。该记录限于 Windows 合成服务和真实原生窗口，生产提供商、独立屏幕阅读器及其他操作系统未实测。

## 普通偏好 Cookie 变体

本次选择该变体以检查普通偏好会不会改变认证错误的归类或覆盖奖励中的数字。Ready 与 Expired 保留官方的合成 Session，并额外发送 `consent=1`。包装器调用仓库原准备、监听和应答逻辑，在原临时目录配置两个附加偏好，保留配对路径、marker 和数值回环绑定。仓库演示服务器原本按单一 Session 的完整字符串识别请求。包装器先校验来请求的两项 Cookie 都准确，再将 Session 交给原处理器返回原有响应。其余账户直接使用原处理器。该独立辅助文件的散列已与实际执行件核对。[运行用包装器](checkin_consent_fixture.py)

共 8 个涉及这两个账户的真实请求均带齐两项。Expired 发出 3 次 GET、2 次 POST，401 仍归类为 `cookie_expired` 并能进入正确编辑器。Ready 发出 2 次 GET、1 次 POST；SQLite 的成功签到奖励和原生当日历史标签均为 1 USD。Today 的另一次 GET 构成开发路径第 9 个请求。默认版和密钥回归直接使用仓库原始配置。[包装器散列和逐请求记录](daily.json)

设备密钥错置、恢复后重试，以及单账户密文损坏后的凭据修复入口，在另一个全新夹具中核验。[两项原生回归与截图](regression/review.md)
