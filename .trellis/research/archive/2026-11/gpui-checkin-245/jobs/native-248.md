# #248 签到作业的 Windows 原生复核

从签到管理的实际导入、选择及确认控件完成全量和所选签到、连续两次批量余额刷新。在筛选、忙碌时追加点击、切换页面和普通重读后，界面结果与回环服务的请求计数一致。[逐项读数与请求收据](native-248.json)

## 复现条件

Windows 11 build 26300，120 DPI，中文浅色，逻辑客户区 1280×800。通过独立桌面中的真实 GPUI 窗口执行。源码提交 `8c7885a1c1bdd49627f88336065aa43fe12057ef`，debug feature exe 的 SHA-256 为 `388d4e7d1e5fd92439867a66d7a38b2df9c93bd83bf9e3221e8f4ed353e0106d`。在该源码根用 `python ccr-desktop/scripts/desktop.py cargo build --locked --bin ccr-gpui --features native-checkin-fixture` 构建。二进制副本仍使用编译时检出目录下的配对夹具。

1. 在同一检出的根目录运行 `python ccr-desktop/scripts/checkin_fixture.py --delay-ms 600`。保持服务器运行，按它打印的三个参数启动 feature exe，路径保留原值。脚本只监听数值地址 `127.0.0.1`，在本树 `outputs/desktop` 内创建临时目录和签到 marker。每次启动都会生成新端口、时间与目录。
2. 从侧栏「签到管理」进入「导入导出」，填入脚本打印的 `accounts.json` 绝对路径，预览，选择跳过冲突，再提交导入。该次数据集为 `checkin-3on3_7ji`、标记 `ccr-gpui-checkin-fixture-v1`，生成文件 SHA-256 为 `804f6620c7de2e03a01a8b8e0c2c1908b466f501ca7d1bfc0fd427016f9c0220`。导入五账户未向提供商发送请求。
3. 在账户清单依次执行下表。服务按完成顺序仅记录方法、合成账户与 HTTP 状态。以 UIA、DWM 窗口截图、SQLite 已保存快照及服务器记录相互核对。图片保留原窗口尺寸。

| 路径 | 实测结果 |
| --- | --- |
| 选择 Ready、Today → 刷新所选余额 → 立即重复 | 首次各 1 次 GET，均更新为 1 USD。汇总覆盖 2/5，余额 2、总额度 2.4、已用 0.4；两条账户记录的成功刷新时间和余额记录时间逐项一致。第二次两条结果均为「跳过 · 30 秒内已刷新」，新增 0 个请求，金额和时间均保留。[首次余额刷新](balance-updated.jpg) · [近期跳过](balance-recent.jpg) |
| 只选 Expired → 签到所选 | 确认只含 `fixture-103`、Check-in demo。回环服务只收到 Expired 的 GET 和 POST，各返回 401，共 2 次。 |
| 搜索 Ready → 一键签到 → 追加点击 → 切换用量页并返回 | 确认包含 4 个启用账户及所属提供商。Disabled 没有入列。确认执行后，一键签到呈禁用，再向同一控件发送一次鼠标点击。Ready 成功（GET、POST 各 1），Today 今日已签到（GET 1），Expired 失败（GET、POST 各 1，`cookie_expired`），Paused provider 跳过（0 次，`provider_disabled`）。这批共 5 次请求。离页回来仍可看到四条结果。[筛选时的四账户确认](all-confirm.jpg) |
| Expired 的结果 → 编辑凭据 → 取消 → Enter → 普通重读 | UIA 逐条提供包含状态、账户、提供商、ID 和已有错误／跳过原因的名称；编辑按钮也标出 Expired 和 Check-in demo。进入编辑器核对账户名，取消后原按钮的 Y 坐标为 523→523，第一下 Enter 直接重开同一账户。清单重读后四项未变，请求数仍为 9。[回到原错误项后的焦点](error-return.jpg) |

初版 `467dfcec` 中四条结果均可见，但 Windows UIA 只给出总分组与通用编辑按钮。`53a5dc11` 已验证四个命名逐项分组及按账户命名的编辑按钮。上表及附图来自已合入 #249 历史和 #250 传输的最终构建；在新的五账户夹具上重跑了完整路径。

本次实测使用仓库合成服务。普通/default 构建的拒网路径和其他平台的行为由各自验证记录说明。
