# 独立终端证据复核

2026-10-07。状态：PREPARED_NOT_RUN。产品路径隔离修复由其他实施及检查角色负责；该复核未运行 `ccr.exe`，未读取真实配置或日志。

## 已核对脚本

- 父 `checks/native-smoke.ps1` 使用新合成进程报告中的环境覆盖与临时 cwd。每个 case 清空窗口；缺字段 case 在临时 Codex JWT 中省略 email，最后恢复原临时文件 bytes。
- 40/80/120 列、dark/light、normal/no-color/dumb 组成 18 格。每格包含 long/multiline save、duplicate、current、Doctor、missing-fields、leading-seed、duplicate-leading、copy-normal、copy-leading。
- ordinary/leading 建议从实际 `ConsoleEvidence` 屏幕 Rows 提取，分词后执行。40 列时普通命令在 `teach/er` 处折行，前导连字符命令在 `-teac/her` 处折行；Rows 拼接保留这些命令字符。
- `ConsoleEvidence.cs` 调用 Windows console API，保存可见窗口的实际文本、颜色属性与 palette。该文件不绘制 mock。Doctor 长报告可能超出可见窗口，需要同时保存未截断的原始 PTY 回执。
- 父 `checks/process_evidence.py` 捕获两流、退出码和二进制 hash；24 个 case 使用合成平台 runtime，cwd 指向临时 home。Doctor 仅在路径安全门槛后恢复。

以上是源码核对，不能作为产品运行 PASS。

## 独立校验入口

`checks/native_acceptance_validator.py` 只读取回执，不执行产品。入口 `--help` 与 scoped `git diff --check` exit 0。

校验项目：18 格齐全；binary SHA 一致；实际窗口尺寸；stdout/stderr 未重定向；主题 palette；可见文字前景与背景不同；NO_COLOR/dumb 无样式；正文、长描述与邮箱值保持默认样式；长字段和多行字符完整；缺字段省略；覆盖后果与完整建议仍可读；普通及前导连字符命令来自屏幕且实际执行 exit 0。

独立校验仍需结合原始 PTY、共享输出混合流回执、机器 DTO 与状态回归。可见屏幕的字符和颜色属性不能证明 stdout/stderr 的所有 bytes。

## AC 证据边界

- AC1/AC4/AC9：以实际 handler 原生矩阵为最终证据；缺字段和普通/前导连字符建议复制已列入每格。
- AC2/AC3/AC5：结合共享 renderer、实际原生颜色属性、Doctor typed-status 回归、计数/取消/空状态进程断言及 mixed-stream 矩阵。
- AC6：机器 DTO、两流与退出码使用 root commands 回归和进程报告；旧 baseline binary 的源码身份未证明，不能用旧 artifact 宣称当前源码比较。
- AC7：成功、缺字段、重复、未登录、不支持、warning、空列表、取消和失败有批准的真实 CLI binary 夹具。Codex import partial/skip/overwrite/empty 使用合成 service fixture 与独立 renderer 子进程；`import_command` 仍先读取 `dirs::download_dir()`，该读取不属于已批准路径修复，禁止为补证直接扫描真实 Downloads。Unknown LoginState 没有 native producer，保留源码验证与运行未验证标签。
- AC8：最终 inventory hash 必须反映路径修复后的共同源码，保留基线处置身份。
- AC10：最终源码 manifest、binary SHA、required suites、strict lint/type 与完整 `just ci` 必须互相对应。Linux/macOS/hosted、真实账号和旧日志影响不能由本地 fixture PASS 推定。

完整原生及全量回执尚未生成。该文档不关闭最终验收项。
