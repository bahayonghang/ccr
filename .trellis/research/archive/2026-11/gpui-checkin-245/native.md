# 签到管理工作台的 Windows 原生验收

Windows 11 build 26300 上，从真实侧栏完成了首次手动开户和五账户日常签到。两条路径使用同一开发候选。开户前的浏览与取消保留无库状态，作业的成功、部分失败及跳过均能和本机服务的逐请求日志、SQLite 记录对应。默认发布构建在独立窗口中验证了夹具拒网。[首次开户和 SSH](onboarding/onboarding.md) · [多账户、历史与默认构建](daily/journey.md)

## 候选、夹具与复核方法

| 项目 | 实际条件 |
| --- | --- |
| 开发候选 | Windows MSVC debug，提交 `131d2acef02133022b0f59e051c8a49c942cd2f4`，feature `native-checkin-fixture`，exe SHA-256 `0f4da49d1edcf3357fe806aa3eac98774065c4dfd52583ddfddf812053116e5e`。 |
| 启动 | 在编译该 exe 的检出根运行 `python ccr-desktop/scripts/checkin_fixture.py`。使用生成的 `--fixture-checkin-loopback --fixture-profiles <fixture>/profiles.toml --fixture-registry <fixture>/config.toml`，其中 `<fixture>` 位于该检出的 `outputs/desktop`，文件 `.ccr-gpui-checkin-fixture` 的字节为 `ccr-gpui-checkin-fixture-v1`。两条路线各使用全新目录、独立密钥与 SQLite 库。日常路径另用测试适配器给两个合成账户添加普通偏好 Cookie，仍使用同一 marker、端口及官方响应逻辑。详见[启动命令和适配器](daily/journey.md)。 |
| 服务 | 脚本绑定 `127.0.0.1` 随机端口。请求来自合成自定义提供商，服务端只记录方法、合成账户及状态码。开发客户端禁用代理和重定向。[准入边界](jobs/verification-248.md) |
| 图形会话 | Windows 私有桌面，拥有可操作的 CCR Desktop (GPUI) HWND。通过窗口消息点击、输入和按键，通过 UI Automation 核对可访问名称、启停及遮蔽，再用 PrintWindow(2) 和 DWM 窗框采集实际画面。125% 缩放。窗口条件及文件散列随每份收据保存。 |
| 默认构建 | 相同源码、默认 feature、release 可执行文件 SHA-256 `49915ac8ee44ad0ab7bdbe43954f0797c752a179c70417123724cba4e369edc6`。正确 marker 与配对路径下，传开发开关时退出 2；省略开关后从真实侧栏提交导入、确认行内签到，服务收到 0 次请求，结果说明演示模式禁止访问提供商。[独立构建收据](daily/release.json) |

## 从侧栏走通的两条路径

1. **首次配置。** 从无库首屏打开提供商表单后按 Escape，再预览本地配置并取消。数据库、WAL、SHM、密钥、迁移目录和锁目录均未产生。手动保存 loopback 提供商后，打开并取消账户表单，数据库字节和密钥状态未变。再手动保存首个账户，进入其详情。切换到不可达的测试 SSH 主机后，用键盘取消一次确认，再确认签到。本机服务收到 GET、POST 各一次，均为 200，本机库中新增一条成功记录。详情返回后，第一次 Enter 重新打开原账户。[逐步读数和截图](onboarding/onboarding.md)
2. **日常操作。** 导入五个合成账户，在筛选后的行上签到，再从页头确认所有启用账户。逐项核对成功、今日已签到、凭据错误和提供商停用，忙碌期间重复发起按钮禁用。刷新所选的 Ready 和 Expired 后，前者出现余额及采样时间，后者保留无快照状态并显示实际错误。到签到记录按状态及账户 ID 筛选，打开正确的凭据编辑器，取消后第一次 Enter 再次打开原对象。查看账户详情、逐日数据，再用键盘返回列表。[请求、记录和返回位置](daily/journey.md)

## 首屏、视觉与键盘

八组截图分别对照同一数据状态、主题、尺寸和滚动起点。基线为 `4f0a02387a221f9f1822eede2787ebe050aa75da`，输入包括无库和三个合成账户。宽窗 1280×800、窄窗 960×640，默认字号 14/12。新首屏无需滚动即可辨认本机范围、开户入口或作业按钮和首批账户。配置态按 CNY 和 USD 分行；实测 0 CNY 有数值，缺快照的账户给出文字。窄窗保持单列，工具栏换行。浅深两套截图实测了主按钮文字、次要文字与输入框外缘的对比。[八组前后原图、逐图 JSON 与取样值](paired/comparison.md)

独立走查还将中英文界面字号各提高到 20px，在 960×640 中打开 119 字符的英文账户名。详情显示完整名称，返回时恢复滚动位置并显示焦点；账户表单可滚到保存和取消。旧构建中详情返回按钮的宽度为 929 物理像素，修复后中文为 120、英文为 195。修复前的普通 Label 在 Windows 原始 UIA 树中消失，最终构建在两种语言及 SSH 场景均能定位有名称的作用域 Group。暗色表单的横向四点均为相同的实色，输入框也独立取样。[大字号、实色表单、焦点和像素测量](visual/visual-review.md)

Design Atlas 对签到视图的 19 个 Rust 源文件未找到可扫描文件，机械门禁没有运行。原生人工走查覆盖浅深阅读面、状态文字、控件外缘、键盘焦点与字号变化。UIA 按属性检查合成会话标记，归档截图另行目视检查。实际覆盖限于已记录的窗口、主题和合成账户。

## 管理、诊断、历史与传输

| 从工作台可达的操作 | 扩展验证 |
| --- | --- |
| 已有库读取、分币种摘要、筛选、真实零与缺值 | [只读首屏验证 #246](first-screen/native-246.md) |
| 提供商与手动账户管理、保护已有凭据、关联删除保护、连接与 CDK 诊断 | [原生账户和提供商检查 #247](account-management/native-247.md) |
| 所选／全部账户、30 秒余额缓存、离开及返回作业、忙碌禁用 | [开发回环请求和原生结果 #248](jobs/native-248.md) |
| 签到记录条件、账户 ID 定位、月历、逐日数值与趋势 | [GPUI 只读历史 #249](history/native-249.md) |
| 配置预览、取消、跳过／覆盖冲突、加密配置与今日统计、恢复 | [本地传输和备份 #250](transfer/native-250.md) |

## Standards

整支差异首轮评审发现提供商回显、WAF 只读访问、余额口径、缺密钥时的账户名称和批量上限五处问题。返修后，用真实窗口验证了 WAF 过期行仍在、孤立余额被排除、原始名称保留、作业及新历史不含测试凭据。作业计划测试覆盖 1,000 个启用目标混入停用账户、1,001 个启用目标和失效 ID。[Windows 返修读数和截图](review/native-reviewfix.md)

## Spec

首轮发现五处与票据不符，其中余额口径与 Standards 重合。其余四项分别涉及目标库密钥校验、并发开户与删除、Cloudflare 缓存状态、作业解密失败后的按 ID 修复。SQLite 双连接验证两种并发次序均无孤立账户。原生复验中，错误密钥未改变目标库、记录数或备份，恢复原密钥后同一配置成功导入。损坏的合成凭据在发请求前报错，能打开该账户编辑器，取消后由原按钮再次进入。复验还发现失败预览旁的旧回执归属不清，现已标注上次导入结果。[导入、修复入口及键盘返回](daily/regression/review.md)

复核时还发现两个能从产品入口触发的边界条件。普通偏好 Cookie 中的数字曾改变 HTTP 401 的显示和凭据错误分类，并会污染已保存的 1 USD 奖励。签到账户写入与提供商删除的关联保护还需覆盖共享手工开户及配置导入。最终回归分别检查了保留的错误码、按 ID 修复入口、SQLite 奖励与逐日显示，以及先删除提供商后的开户和导入结果。[最终日常操作](daily/journey.md)

## 自动验证

| 命令 | 实际结果和源码范围 |
| --- | --- |
| GPUI 默认及开发特性签到测试 | 最终源码 `131d2ace` 的默认配置通过 49 项，启用 `native-checkin-fixture` 后通过 50 项。使用 `ccr-desktop/scripts/desktop.py` 分别运行 `cargo test --locked --lib checkin -- --skip export_bindings` 及其开发特性版本。 |
| `cargo test --locked -p ccr-checkin -p ccr-db -- --skip export_bindings` | 最终源码独立运行 ccr-checkin 100 项、ccr-db 142 项，全数通过。先前 `b6debd5d` 同时覆盖 ccr-core、ccr-checkin、ccr-db，398 项通过，8 项按原设置忽略。 |
| `just desktop-test-ui` | `131d2ace` 通过，124 秒。各程序的通过／忽略数为 24／0、20／1、2／0。 |
| `just desktop-ci` | `131d2ace` 通过，533 秒。包含格式、检查、Clippy、全套桌面测试与默认 release。GPUI 库通过 495 项，忽略 4 项。 |
| 根 `just ci` | `131d2ace` 全部 17 步通过，1231 秒。本机缺少 OMP Trellis 扩展，配方报告 14 项契约测试跳过。 |

以上三项完整门禁均在干净的集成源码上运行。此后的归档提交仅更新本目录的实测材料。[前序命令与覆盖记录](jobs/verification-248.md)

## 验收范围

本次接受 Windows 手动开户、签到与余额、历史、导入导出及其本机作用域。SSH 场景选择了隔离的离线主机。独立屏幕阅读器、系统高对比模式、macOS、Linux、真实 SSH 会话、生产提供商及其浏览器挑战未在本轮实测。签到 OAuth 保持开放、标签为 `needs-info`，需要能够返回可用于签到的凭据的提供商公开协议。[OAuth #251](https://github.com/bahayonghang/ccr/issues/251)
