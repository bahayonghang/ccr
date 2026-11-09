# Bindings generation preflight

2026-09-28。以下事实分别记录，不能相互代替或用于未经验证的原因归属。

1. T11 多次生成后出现 11/5 个文件尾部残留或空行。失败 diff 与日志保留在 T11。尚未证明并行导出、Bun、mbx 或任何特定进程为原因。
2. T11 独立正式 guard 在 CLI export 测试 exe 启动时返回 `0xc0000005`，该次尚未执行导出；这是另一个失败阶段。T02 同类启动异常后来在无重编译情况下恢复，但首次没有取得 hash，不能声称两次产物 hash 相同。
3. `ccr-ui/justfile` 的 bindings recipes 先删除实际 generated 目录；`check-generated-bindings.mjs` 在 child 非零时直接返回，未恢复目录。T11 此次 227→0 的文件变化和精确恢复记录构成实际证据。T10 应让失败保留现有生成物，并继续非零退出，不能通过吞失败使 guard 通过。
4. root 对 normalizer 同型的 `node:fs/promises.writeFile` 缩短文件操作做隔离探测：Windows Bun 1.4.2 与 Node v26.7.0，各 3 类 ASCII/中文/LF/CRLF 样本、每类 50 次，均未发现尾部残留。探测只创建并删除自有随机临时文件，不读取/修改产品 generated 目录。代码与结果为 `normalizer-write-probe.mjs`、`normalizer-write-probe-{bun,node}.json`。

第4项只是受控探测，不能证明整体生成链正确，也不能排除其他条件下的写入问题。最终应在所有 source owner 冻结后串行执行原本正式 gate，记录完整前后指纹和未通过阶段。
