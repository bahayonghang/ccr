# C3 当前验收

状态：PASS_LOCAL_GATES。2026-10-07。源码、文档、最小路径修复、独立检查、实际 binary 和 Windows 原生矩阵已通过；完整 just ci 16/16、exit 0，耗时20:36.985。任务保持 in_progress，未提交或归档。

## 证据与责任

- 非 Auth 45 份源码及 4 份中英文活动文档完成独立复查。原有表格、确认、服务/返回、JSON 和 Doctor stdout 边界保持；额外日志/ConflictChecker 路径变化按用户批准方案单独复核。
- 931 基线调用、452 基线语义入口分别保留，当前 1091/961 另存。处置为迁移 494、共享覆盖 302、保留 135；72 源码 hash 与当前清单一致，最终 82 文件 manifest 另存。
- 路径独立检查：logging 13、ConflictChecker 5、logger 子进程 2 通过。15 天 managed log 删除，13 天及 unmanaged 保留，其他 temp root 的 bytes/mtime/目录项不变；真实数据未读取。
- 展示进程 22/22、0 ignored；Doctor JSON/human exit 0/1、四状态、stdout、完整 file/dir bytes/mtime inventory 等式通过。Windows 缓存 metadata 首失败与20轮无CLI探针保留；夹具改用 fresh fs::metadata，断言未减少。
- 既有 Grok/Claude/Codex 回归 8/10/14 通过。完整 root commands 与相关 crate 套件由正式 workspace/all-features/default-parallel/skip-export-bindings Test 步覆盖；2101 passed、0 failed、19 ignored、33 filtered，ignored/filtered 不计通过。
- 真实 Windows console：40/80/120 列 × dark/light × normal/NO_COLOR/dumb，18 格×9=162 case 通过；36 次从屏幕提取普通/前导连字符建议并实际执行。长字段、多行、缺字段、正文/字段值默认色、状态及 palette 由实际 Win32 buffer 和完整 PTY验证。
- 当前共享 native 混合流 12 格通过，覆盖 normal/force/dumb-force/NO_COLOR × stdout/stderr/both；force 例外、dumb 降级、目标流字符和错误 stderr 归属明确。
- final strict lint、fmt、version、workspace check 已通过正式 CI 复测前7步；完整16步正式CI已通过，原始摘要和exit0由父回执记录。docs build/audit 已通过且4份文档未再修改。

## 回执索引

- 路径及局部修复：`2026-10-07-path-isolation-implementation.md`、`2026-10-07-path-isolation-independent-review.md`、`2026-10-07-doctor-inventory-probe-first.txt`、`2026-10-07-gate-help-init-*`。
- 独立实际证据：`2026-10-07-independent-native-acceptance.json`、`2026-10-07-independent-native-checker-retest.json`、`2026-10-07-independent-process-pty-audit.json`、`2026-10-07-independent-mixed-stream-audit.json`。
- 父 checks：`2026-10-07-isolation-{binary-identity,source-manifest,final-source-manifest,workspace-test-result,inventory-verification}.json`、`native-2026-10-07-isolation-first-*`、`2026-10-07-isolation-native-stream-*`、`2026-10-07-isolation-just-ci-{first,retest}.txt`。
- 此前报告原文：`2026-10-07-pre-path-verification.md`。旧 failed/NOT_RUN 状态是此前证据，不覆盖当前通过结果。

## 边界与未运行项

父 AC7 仍 UNVERIFIED：Unknown 无 native producer，完整 import 密码交互未运行；partial/skip/overwrite/empty 含 service/renderer 边界，不能记为全分支CLI运行。24-case smoke 的 Codex current 是 unsaved，Grok JSON 是 logged_in=false；Saved等依据独立 binary。父AC10=PASS；父AC7未关闭。

Linux/macOS、hosted、真实账号和在线Provider/WebDAV为NOT_RUN。发现隔离缺口前的真实日志实际变更范围未经审计；未读取真实配置、账号或日志补证。Windows权限helper仍no-op。预存Vue 3说明与React19冲突及既有失败返回Ok不在批准修复范围，保持原样。

native 使用固定临时ccr.exe SHA26560b7ae47654a54f12e90cc75f99622439bd21a6af04c15c876a139a7475c6；最终源码与构建manifest只差help.rs测试预期，产品无漂移。后续debug/all-features/release构建不替换该身份。回退只恢复批准文件，不操作真实账号/日志；提交、推送、发布、归档未授权。

最终独立全表由2026-10-07-independent-final-acceptance.md记录；父final-gate-verification.json确认16步PASS与82文件hash不变。当前范围内无需进一步源码修改；交付权限未扩大。
