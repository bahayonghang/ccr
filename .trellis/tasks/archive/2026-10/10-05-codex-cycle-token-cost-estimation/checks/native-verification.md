# 合成原生终端验证

日期：2026-10-06。结论：Windows 原生 PTY 的代表性尺寸、统计与输入检查 PASS。真实账户和其他系统 NOT_RUN。

## 二进制与隔离范围

- 本轮 `just ci` 的 Release Build 已通过，产物为 `target/release/ccr.exe`，19895296 bytes，修改时间 UTC 2026-10-06 08:42:22。
- SHA256：`4BFD82DE7B929D3C96FD13F0D4E4B0306BE00B08343729CE2B3A874DCCE527B6`。
- 批准的3个JS锁条目更新后，第六轮Release Build再次PASS；重新核对长度、mtime和SHA256，二进制身份与本页原生receipt相同。该补丁没有改变CLI二进制，原生结果仍适用于本轮产物。
- `native-smoke.ps1` 为每次运行创建独立临时目录，覆盖 CCR/Codex/Claude/Grok 和进程 USERPROFILE 路径，结束时恢复进程环境。fixture 没有 auth.json，没有读取个人 auth/session/llmusage 数据。
- 原生会话 stdin/stdout 均未重定向。首次继承 TERM=dumb 被产品拒绝，退出74；详见 `native-first-attempt.md`。复测为支持 ANSI 的 PTY 设置进程级 TERM=xterm-256color；未修改产品终端能力检查。
- 捕获的9段原生输出保留于 `native-pty-output.log`，格式为 NDJSON，每条 output 保留原始 ANSI 字符串。该文件是本地检查 receipt。

## 实际检查结果

| 检查 | 140×40 | 100×22 |
| --- | --- | --- |
| 实际 Console WindowWidth×WindowHeight | 140×40 | 100×22 |
| stdin/stdout redirected | false / false | false / false |
| 所选账户 | synthetic | synthetic |
| 最近5h、7d、全历史 | 各10.2M Token、100 records、API $5.15 | 各10.2M Token、100 records、API $5.15 |
| 费用说明 | API equivalent USD，priced/Standard，100/100 records、10.2M/10.2M priced | API equivalent USD，Std；附加信息按紧凑合同折叠 |
| Token 分类 | In10M、Read9M、Write500K、Out200K、Reason100K | 按紧凑合同折叠 |
| quota 本地错误 | 缺少 fixture auth 文件，Unavailable/ERR 可见 | 同左，错误有截断标记 |
| 容量 | 5h/7d Token和USD为N/A/window，样本0/0；joint N/A/scope | 两窗口rem N/A:window；不显示虚构剩余量 |
| Ctrl+L | 切换为简体中文；synthetic及数值保留 | 同左；语言与退出键可见 |
| r | 显示“已刷新账号与统计”；旧快照/刷新中标签出现后清除 | 同左；所选账户和数值保留 |
| q / 外层进程 | Native process exit 0 / exit_code 0 | Native process exit 0 / exit_code 0 |

100个合成 response.completed 分别使用 I=100000、R=90000、W=5000、O=2000、Q=1000，模型gpt-6.1-sol、速度Standard。实测10.2M/$5.15与规划算例一致。quota 在读取缺失的 fixture auth 文件时失败，当前窗口不可知，N/A原因与该条件一致。该运行没有验证正向真实quota或真实账户经验容量。

## 边界

原生结果来自实际 Windows 程序和 ConPTY 输入/输出。六尺寸×双语×18状态的216组布局为 TestBackend 证据，保留独立标签。Windows私有ACL和原子替换失败为领域测试中的原生文件证据。真实OpenAI账户、其他设备/云任务、Linux/macOS、hosted CI：NOT_RUN。没有安装、提交、归档、推送或发布。
