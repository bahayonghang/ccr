# #248 作业、余额与夹具验证

行为与原生截图见[Windows 实测](native-248.md)，其中附有逐账户请求收据。以下验证使用临时签到数据库、合成凭据及捕获的临时密钥目录。测试未访问实际提供商或个人签到库。

## 从失败到通过

| 场景 | 复现与回归 |
| --- | --- |
| 签到对象和返回位置 | 在筛选后的账户列表点击一键签到，按启用账户的保存 ID 确认；所选及行内操作只确认指定对象。改变候选身份后提交，作业被拒绝。重复提交、离页重返、编辑凭据并保存后，仍可读到相同账户的上次结果。新增首屏按钮时重现过删除账户后的焦点回退错误，调整顺序后原有回归通过。 |
| 采样的原子性 | 临时数据库在更新账户成功时间时拒绝写入，旧实现仍产生“已更新”。修复后逐项报告失败，账户的成功时间与最近余额样本均保持原值。成功时两处保存同一时间；单账户显式刷新可以在 30 秒内取新样本。 |
| 队列等待期间的变化 | 在首项请求未完成时，另一客户端停用下一账户、给第三个账户保存新样本。旧行为向服务端发送 3 次请求。修复后仅发送 1 次，其余两项分别报告对象变化和近期已刷新。 |
| 提供商准入 | 一个库中历史遗留的内置别名最初进入普通网络错误分支。修复后在创建请求前即被拒绝。相邻用例验证显式内置标识、非数值回环地址、合成提供商端点及重定向；第二个监听端口未收到重定向请求。 |
| 原生辅助功能 | 初版的四行结果可见，Windows UIA 中缺少可定位的逐项名称。为每行赋予 Group 角色，为凭据操作增加账户和提供商名称后，用新构建重新从侧栏进入并验证。 |

## 验证命令

| 命令 | 已验证结果 |
| --- | --- |
| `cargo test -p ccr-checkin -- --skip export_bindings` | 共享 crate 通过 97 项。合入 #249 仅修改独立 GPUI 工程及验收档案，之后共享 crate 源码未变。 |
| `python ccr-desktop/scripts/desktop.py cargo test --locked --features native-checkin-fixture --lib checkin -- --skip export_bindings` | 最终实现提交 `8c7885a1c1bdd49627f88336065aa43fe12057ef` 通过 43 项，包括经脚本创建标记夹具、预览导入、真实服务请求及账户级结果，也覆盖独立只读历史和传输入口。 |
| `python ccr-desktop/scripts/desktop.py cargo build --locked --bin ccr-gpui --features native-checkin-fixture` | 同一提交的 debug 构建通过。Windows 窗口按合成 ID 复核 9 次 HTTP、51 次 UIA 观察，构建散列及截图见[原生收据](native-248.md)。 |
| `just desktop-test-ui` | 同一提交通过 24 项环境测试、20 项 native shell 和 2 项 native UI。native shell 沿用 1 项 ignore。 |
| `just desktop-ci` | 同一提交的 fmt、check、clippy、全量测试、默认 release 均通过。GPUI 库 488 项通过、4 项 ignore。 |
| `just ci` | 单票与 #250 集成期间首轮全部 17 步通过，用时 31:51。修复后从 `53a5dc11b8f2aef11447d0649b171dedce021425` 启动第二轮，全部 17 步通过，用时 23:34。期间吸收的 #249 仅涉及独立的 `ccr-desktop` 工程与原生证据，上述桌面门禁在吸收后重跑。 |

对最终默认 release（SHA-256 `90612125a4704955499f2b2660369772c3bdc46427dab8a1009fa777faf3481c`）独立执行了两次：一次只传 `--fixture-checkin-loopback`，另一次同时传该开关、空的 `profiles.toml` 和 `config.toml` 的绝对路径，目录位于真实 `outputs/desktop`，含准确 marker。两次均退出码 2，在解析参数时拒绝启动。普通 Fixture 和 Benchmark 的请求拒绝、开发模式缺路径、错 marker、越界/错配目录及自定义回环准入由上表的运行时用例验证。

验证范围为 Windows 合成服务、真实 SQLite/临时设备密钥和本地构建。生产提供商、真实挑战、其他操作系统及独立屏幕阅读器未实测。
