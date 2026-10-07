# CLI 信息展示：当前源码验收

状态：LOCAL_GATES_PASS_AC7_UNVERIFIED。2026-10-07。HEAD `528d4bae1b145fd74d7bbc97454caac249a4b896`，分支 `dev`。展示和已批准的 Windows 路径修复已实施；父任务及 C1–C3 保持 `in_progress`，未新增提交、推送或归档。

## 实施与独立检查

- C1：共享状态、字段、下一步与启动 `TERM=dumb` 降级已通过。主体提交 `c72ba99f` 保留，补充仍在工作区。
- C2：21 份 Auth 源码边界复核通过，空导入结果重复 warning 已修复。展示 binary 22/22、Grok/Claude/Codex 既有回归 8/10/14 通过。Unknown 和完整密码交互的运行边界单列。
- C3：45 份非 Auth 源码及 4 份活动文档完成复查。Doctor 四状态、JSON、stdout、exit 0/1、文件字节/时间/目录清单由实际 binary 验证。文档 build/audit 通过。
- 已批准的路径修复：非空 CCR_ROOT 选择日志根；ConflictChecker 复用平台路径；project/help 子进程夹具补临时 root。独立 logging 13、ConflictChecker 5、隔离 logger 2 项通过。writer/filter/14 天清理/redaction/bridge、冲突算法与 DTO 保持。
- 已解除展示测试 17+5 个临时 Windows ignore。有意 child probes 继续由实际父测试启动。
- 展示契约、路径优先级及 fresh Windows metadata 已回写 spec。931 基线调用和 452 基线语义入口保留；当前 1091 引用和 961 语义入口另存，72 个清单源码 hash 一致。最终改变文件清单为 82 个产品、测试、文档和规范文件。

## 当前证据

| 检查 | 结果 | 回执 |
| --- | --- | --- |
| 独立路径安全 | PASS，13+5+2；跨根哨兵 bytes/mtime/目录项保持 | C3 `2026-10-07-path-safety-independent-*`、`2026-10-07-path-isolation-independent-review.md` |
| 展示 binary / 既有回归 | PASS，22/22；8/10/14，0 ignored | C2 `2026-10-07-path-safe-*` |
| 补充实际进程 | PASS_BOUNDED，24 case、8 JSON parsed；退出码按既有分支 | `2026-10-07-isolation-process-first.json`；C3 `2026-10-07-independent-process-pty-audit.json` |
| Windows 原生 handler | PASS，18 格 × 9 case = 162；复制执行 36 次 | `native-2026-10-07-isolation-first-*`；C3 `2026-10-07-independent-native-acceptance.json` |
| Windows 原生混合流 | PASS，12 格，4 模式 × stdout/stderr/both 重定向 | `2026-10-07-isolation-native-stream-*`；C3 `2026-10-07-independent-mixed-stream-audit.json` |
| 完整 Rust workspace | PASS，2101 passed、0 failed、19 ignored、33 filtered；默认并行 | `2026-10-07-isolation-workspace-test-result.json` |
| version / fmt / strict lint / workspace check | PASS，各 exit 0；正式 CI 复测前 7 步通过 | `2026-10-07-isolation-just-ci-retest.txt` |
| 清单与源码身份 | PASS，931/1091、452/961、72 hash、82 最终文件 | `2026-10-07-isolation-inventory-verification.json` |
| scoped git diff --check | PASS；raw 回执不改写 CRLF | 主会话与独立 scoped 回执 |
| 完整 just ci | PASS，16/16、exit 0、20:36.985；首失败保留 | `2026-10-07-isolation-just-ci-{first,retest}.txt` 及 result.json |

24-case smoke 的 Codex current 为 unsaved，Grok JSON 为 logged_in=false；这些回执不代表全部 Saved/已登录分支。其他状态依据独立 binary 断言。native current 验证了实际 unsaved warning。

原生矩阵为 Windows 实际 console：40/80/120 列 × dark/light × normal/NO_COLOR/dumb。Win32 字元、palette 和颜色属性验证首行、完整长字段、多行、缺字段、正文默认颜色和复制命令。每格保留完整 PTY；清屏和主题操作属于 harness，不能对整份 PTY 无条件断言无 ANSI。

native 使用经 SHA-256 固定的临时 ccr.exe 副本，身份见 `2026-10-07-isolation-binary-identity.json`，构建源码见 `2026-10-07-isolation-source-manifest.json`。最终 `2026-10-07-isolation-final-source-manifest.json` 只追加 help 测试断言变化，产品源码无漂移。all-features/release 构建不能覆盖 native artifact 身份。

## AC1–AC10

| AC | 状态 | 依据与边界 |
| --- | --- | --- |
| AC1 | PASS | 18 格 save 首行为 ✓ 或 dumb 的成功:；掩码邮箱、下一步与缺字段由真实 console/binary 验证。 |
| AC2 | PASS_BOUNDED | shared/Doctor 四状态、计数/取消/空状态、实际 warning/error/skip 与原退出码；未触发状态见 AC7。 |
| AC3 | PASS | 标记/正文颜色属性、重复标记审查、共享格式和完整处置；保留布局有范围理由。 |
| AC4 | PASS_BOUNDED | 建议源码审查、普通/前导连字符参数与 36 次屏幕复制执行；Unknown 仅源码边界。 |
| AC5 | PASS | 18 格 normal/NO_COLOR/dumb、force 含 dumb binary、共享混合流 12 格；按目标流选择文字/样式。 |
| AC6 | PASS_BOUNDED | DTO/serializer/return/service 边界、22 项展示、既有回归和 8 JSON smoke；未声明所有交互均运行。 |
| AC7 | UNVERIFIED | 已产出状态由合成 binary 验证；部分导入含 service/renderer。Unknown 无 native producer，完整 import 密码交互未运行，不能称全分支进程验收完成。 |
| AC8 | PASS | 基线/当前清单分开保留；每项处置、理由、责任人与 hash 已审计。 |
| AC9 | PASS | 40/80/120 列、明暗背景、长字段、多行、必要建议和复制执行完整，无人为截断。 |
| AC10 | PASS | 正式 just ci 16/16、exit 0；当前源码与HEAD固定。其他OS/hosted/native边界单列，未冒充通过。 |

## 首失败与未运行边界

- 旧隔离缺口和验收报告保留于 `2026-10-07-pre-path-verification.md`；旧失败不被新 PASS 改写。
- 新隔离 binary 首测 21/22：Windows DirEntry::metadata 缓存目录时间。无 CLI 探针 20/20 复现缓存差异，fresh metadata 前后稳定；只改夹具为 fs::metadata(&path)，全部 bytes/mtime/目录等式保持，复测 22/22。
- just ci 首轮 root commands 138/139：旧 help 断言要求 profile init 建议。只改为完整分组，退役命令缺席检查保留。exact all-features 1/1 后重跑完整 CI。
- 读取 wrapper 曾使用默认 GBK；另一 wrapper 首次预期了错误的 probe message。显式 UTF-8 和当前 probe 消息复核通过，错误说明存于 stream verification；均未发生产品测试失败。
- Unknown native、完整加密密码交互、Linux/macOS、hosted、真实账号和在线 Provider/WebDAV 为 NOT_RUN。ccr codex auth import 仍扫描 Downloads；该路径不在已批准扩展范围，未为验收读取真实 Downloads。
- 发现缺口前的进程对真实日志的实际变更范围仍未审计；没有读取真实日志、配置或账号补证。Windows 权限 helper 仍为 no-op。

回退只恢复批准源码、测试、文档和规范；没有数据迁移，不恢复、删除或修改真实日志或账号。提交、推送、发布与归档仍未授权。

最终门槛：2026-10-07-isolation-final-gate-verification.json 确认16步PASS、HEAD和82个最终文件hash无变化；原始摘要和退出结果见isolation-just-ci-retest.txt/result.json。Rust workspace Test通过2101项，19 ignored和33 filtered另记。AC7保持UNVERIFIED；没有新增提交或归档。

最终范围复核：2026-10-07-isolation-final-scope-verification.json 和 isolation-final-diff-check.txt 确认82个范围内文件hash一致、无额外产品路径、无staged或deleted文件，scoped diff exit0。
