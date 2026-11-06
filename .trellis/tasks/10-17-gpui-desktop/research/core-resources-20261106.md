# 2026-11-06 资源、生命周期与敏感同步实现

基于 0404fa2a，完整注册变更经独立 trellis-check 检查；未注册的下一批签到与平台文件不属于本批。

- Claude rules/output styles 文件和 agents/slash/plugins settings 集合 CRUD；Codex 本地 agents、source catalog、安装/同步冲突决策；原始版本、未知字段和省略凭据保留。模板只填充当前指令草稿，显式保存才写文件。
- MCP 五项内置预设走现有编辑确认；跨源复制使用不向 UI 暴露凭据的会话。仅转换已知兼容字段，不支持的协议/字段明确拒绝；单项操作不等于批量安装/同步。
- 五类固定敏感资产接通现有 v2 加密推拉、备份与逐项结果。明文 v1 需单独确认，只导入本地，不声称远端升级。口令不写配置或日志。
- 同一窗口异步恢复并观察 bounds/maximized，防越界回退；关闭等待最终保存终态。拥塞时有界取消排队保存后继续正常 runtime shutdown。损坏偏好保留原字节；祖先 junction/reparse 禁止读写。系统直接 App::quit 无 veto、托盘恢复仍受依赖限制。

## 验证

- desktop-fmt-check=0、desktop-lint=0：batch2-check-*-final.log。
- desktop-test 原样重试=0：134 库测试、8 native_shell、2 native_ui、2 shutdown（1 helper ignored）、15 Python、3 AccessKit。首次仅既有 Insights SQLite 字节比较偶发失败；允许的一次重试通过，未改断言或并行。
- Tauri bin cargo check=0：tauri-resources-check-final.log；55 个修改的 Tauri 命令签名与 0404fa2a 一致。
- 定向领域/交互检查覆盖 Codex copy 原始版本同锁核对、源切换保留草稿/清除旧结果、Claude 祖先链接、Windows junction、MCP 跨源冲突与秘密、敏感同步 loopback v2/v1 合同。

日志位于 outputs/desktop/core-completion-20261106/。这些是隔离 fixture/headless 证据，不是原生实机、真实外部服务或 macOS 验收；01/09/父任务保持 in_progress。
