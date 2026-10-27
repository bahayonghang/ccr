# 2026-10-27 提交与归档说明

用户明确要求将 Trellis 检查临时 PNG 加入 gitignore、拆分提交所有改动并归档相关轻盈玻璃任务。本次归档是用户指定的行政收尾，不将未完成的桌面验收改写成 PASS。

- 范围：ccr-ui-airy-glass 父任务及 AG01–AG04 四子任务。GPUI 与其他任务不在归档范围。
- 原实现、Web／对比度／布局／已覆盖性能检查已有记录。完整 918 项测试与后续定向检查、75.23% coverage 等结果仍以 integration-summary 的准确口径为准，未重复声称完整套件覆盖之后的所有改动。
- final_acceptance 保留 open；焦点恢复 FAIL_BASELINE、独立 i18n 漂移、原生／真实缩放／无 blur 引擎与未覆盖真实性能场景仍未关闭。参见 `../10-26-ccr-ag04-acceptance/evidence/integration-summary.md` 和 `native-feasibility.md`。
- task.py 归档会机械设置 status=completed；本任务树的该状态表示本次已授权的实现提交与档案整理结束，不表示所有验收标准已通过。原 AC 复选框不补填不存在的通过结果。
- `/.trellis/tasks/**/*.png` 同时覆盖活动与归档位置；782 张新截图不进 Git，文件保留在本地。原有 295 张跟踪截图不取消跟踪，应用图片不受影响。截图文件名和 JSON/Markdown 记录保留，但新克隆不会携带这些本地 PNG。
- 45 份原始检查 `.log` 与五份 product.diff 快照按字节保留；使用明确任务路径强制纳入日志，未将整个 .trellis 强制加入 Git。gitattributes 仅对任务 evidence 原始日志／差异快照禁用换行转换。
- research 下的测量脚本是运行时的历史证据，部分以活动任务层级推导 repo 并输出到原活动路径。归档后不要直接运行并假定它们已适配新目录；重跑应在隔离 checkout 恢复其原相对位置，匹配所记录的源码／浏览器／fixture，再审查其安装和真实数据边界。原脚本不为归档重写，保留可追溯性。
- 未获授权的 push、安装、发布、签名及真实用量重建不执行。
