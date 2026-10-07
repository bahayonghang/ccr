# C2 验收状态

2026-10-07，`in_progress`。独立源码和已批准的路径隔离复查完成。实际 Windows 展示进程 22/22 与既有 Grok/Claude/Codex 8/10/14 回归通过；父原生终端和 full CI 由主会话继续。当前证据见 independent-review.md 和 `2026-10-07-path-safe-` 回执。

| PRD 条款 | 已有证据 | 状态与缺口 |
| --- | --- | --- |
| save 首行、字段、缺字段 | 当前 captured binary 与父 18 格 save/缺字段 screen 通过；normal/NO_COLOR 为 ✓，dumb 为成功:；字段缩进、邮箱掩码、完整建议及缺字段省略均核对 | PASS_LOCAL_BINARY_AND_NATIVE；全矩阵汇总由父独立验收 |
| 相关建议最多两项 | 当前代码审查与隔离 save/list/current/switch 断言通过；保留 daemon 分支，总建议不超过两项 | PASS_SOURCE_AND_LOCAL_BINARY |
| 覆盖命令参数与安全说明 | 普通名及 `-teacher` 的实际恢复命令可解析；覆盖后果可见；输出不含合成 token sentinel；双 logger 安全路径回归通过 | PASS_LOCAL_BINARY；实际账号为 NOT_RUN |
| 正确分支状态 | 未登录、keyring unsupported、API Key、Provider Key、空列表、取消、失败与受控运行中警告通过；import renderer 3 项补充部分成功 | PASS_SOURCE_AND_LOCAL_BINARY；Unknown 无 native producer，仅源码复查，runtime NOT_RUN |
| JSON/退出码兼容 | current/rename/update/Grok/off DTO 断言与既有 32 项 Auth/profile 回归通过；重复 save 保留既有 zero exit；serializer/return/service 审查一致 | PASS_SOURCE_AND_LOCAL_BINARY；完整加密密码交互 NOT_RUN |
| 合成 binary 与调用处置 | 原始 203 项处置齐全，Auth 生产源无漂移；显式 CCR root/平台覆盖已生效，22 项 Windows 临时 ignore 全部解除 | PASS_LOCAL_BINARY；历史未隔离证据单独保留 |

## 路径隔离后的当前回执

- `2026-10-07-path-safe-presentation-first.txt`：21 passed、1 failed、0 ignored、exit 101。唯一失败是 Doctor inventory 读取 Windows 缓存目录 mtime。
- `2026-10-07-path-safe-presentation-retest.txt`：22 passed、0 failed、0 ignored、exit 0。包含 warning 子探针实际就绪，Doctor JSON/human exit 0/1、完整文件字节及文件/目录 mtime 等式。
- `2026-10-07-path-safe-{grok,claude,codex}-regression-first.txt`：8/10/14 passed，均 exit 0，默认并行。
- [C3 独立路径报告](../../10-06-cli-feedback-rollout/checks/2026-10-07-path-isolation-independent-review.md)：core logging 13、ConflictChecker 5、双 logger 合成子进程 2 passed；1 个 intentional ignored child 由两个 parent 各实际调用。保留 14 天期限与跨 root byte/mtime/目录项数断言。
- C3 `2026-10-07-doctor-inventory-probe-{first,retest}.txt` / `probe-result.json`：不调用 CCR 的 20 轮合成探针复现 Windows DirEntry 缓存；fresh metadata 20 轮稳定。修复只改变测试 inventory 查询，保留全部等式断言。
- C3 `2026-10-07-path-safety-independent-{fmt,clippy,type-check}.txt`：当前 fmt、三个 affected packages all-targets/all-features strict clippy/type-check 均 exit 0。
- 父 `2026-10-07-isolation-binary-identity.json`、`native-2026-10-07-isolation-first-*-{save,missing-fields}-screen.json` 与 C3 `2026-10-07-independent-native-acceptance.json`：18 个实际 Windows 组合，binary SHA `26560b7ae47654a54f12e90cc75f99622439bd21a6af04c15c876a139a7475c6`。C2 独立逐格检查 save 首行、字段/掩码/建议和缺字段省略；其他 native 条款由父 acceptance_check 汇总。

## 回执索引

- `2026-10-07-independent-presentation-first.txt`：receipt wrapper 错误；产品输出未取得。
- `2026-10-07-independent-presentation-retest-1.txt`：15 项，14 通过、1 失败，exit 101。
- `2026-10-07-independent-presentation-retest-2.txt`：21 项，17 通过、4 失败，exit 101；保存所有首失败与不完整隔离诊断。
- `2026-10-07-independent-codex-unit-first.txt`：lib 6 通过，integration 0 匹配另记。
- `2026-10-07-independent-claude-unit-first.txt`：lib 1 通过，integration 0 匹配另记。
- `2026-10-07-independent-empty-import-first.txt`：新增 empty renderer case 首失败，exit 101。
- `2026-10-07-independent-empty-import-retest.txt`：修复后 3 项通过，exit 0。
- `2026-10-07-independent-{grok,claude,codex}-regression-first.txt`：8、10、14 项通过，exit 0；logger 未隔离。
- `2026-10-07-independent-fmt-check.txt`：PASS，exit 0。
- `2026-10-07-independent-diff-check.txt`：C2 scoped PASS，exit 0。
- `2026-10-07-independent-boundary-audit.json` / `source-manifest.json`：当前源码审查与身份。
- `2026-10-07-independent-receipt-secret-audit.json`：2 份已有 Doctor txt/raw 回执的冲突 detail 布尔审计，敏感关键字值为 false；不读取真实配置、不打印值。
- 父 checks/2026-10-07-continuation-{lint-strict,check-workspace}-first.txt：首轮 PASS，exit 0；历史共同源码结果保留。
- 父 checks/2026-10-07-continuation-{fmt-check,lint-strict,check-workspace}-retest.txt：C3 独立修正与格式修复后的最终共同源码 PASS，各 exit 0；身份见父 continuation-source-manifest.json。

授权前 17 项 Windows ignored 与 `PREPARED_NOT_RUN` 是历史状态，未记录为 PASS。用户批准最小路径修复后，独立安全门槛通过，17+5 个临时 ignore 已解除。core/Doctor 单独 child probes 的 intentional ignore 保留，不将被忽略入口单独计入 passed 数量。

先前 Windows logger 对真实日志的实际变更范围仍未审计，没有读取真实日志补证；修复后的进程使用临时 CCR root。Windows 权限 helper 仍为 no-op。Unknown、完整加密密码交互、其他 OS、hosted 和真实账号为 NOT_RUN；父原生终端与最终 just ci 以主会话独立回执为准。C2 未修改 Doctor service、认证服务、退出码、依赖或系统 Known Folder；提交和归档未授权。

## 2026-10-07 最终联合门槛

父正式 just ci 16/16、exit0、20:36.985，当前HEAD与82个最终文件hash保持。完整workspace Test2101 passed/0 failed/19 ignored/33 filtered；ignored/filtered不计通过。原生handler18格/162case/复制36与共享混合流12已独立复核。父AC7仍UNVERIFIED，Unknown和完整密码交互运行边界未改变。完整回执见父checks/2026-10-07-isolation-final-gate-verification.json与just-ci-retest.txt。此前pending CI段落是阶段证据；当前fullCI已通过，任务仍in_progress，无新增提交或归档。
