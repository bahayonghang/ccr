# 其余 CLI 迁移设计

## 文件责任

- 父 scope-inventory.md 中 C3 的 CLI 文件，仅状态、字段、提示及相关 import。
- `crates/ccr-cli/src/commands/doctor_cmd.rs`：用共享 formatter 格式化原有 DoctorStatus；继续 println 到 stdout。DoctorStatus::label 和 DTO 保持不变；doctor_service 仅调查，不改诊断行为。
- `crates/ccr-sync/src/sync/content_selector.rs`、`crates/ccr-codex/src/platforms/codex.rs`：复核共享变化；如有重复状态或样式，仅修改相应展示行。
- `crates/ccr/tests/commands/output_presentation.rs` 补非 Auth 场景；现有 clean/current/doctor/project_init/validate/codex_fix/sync_content 命令测试只按需要调整展示断言。
- `docs/reference/commands/clean.md`、`doctor.md` 及 `docs/en/reference/commands/` 对应镜像：只同步受影响状态和示例，不重做文档导航。

## 迁移规则

逐个引用决定状态、字段、普通说明、建议或保留。不得将所有 info 一律改为字段，也不得将所有 warning 改为说明。功能统计和只读列表使用中性文字；确有风险的警告继续保留。

UI 服务启动日志中地址、连接信息和进程生命周期不变；只调整属于用户提示的等级/建议。外部工具继承的 stdout/stderr 不包装或重写。诊断 detail/path/recommendation 内容保留，只调整 status 展示。

命令文本和路径不人为截断；终端自然折行。保持当前表格截断规则和行列，避免将本任务变为全 CLI 布局重写。

## 兼容性与回退

回归比较使用合成 fixture。clean 验收默认 dry-run，不删除真实路径；update 不下载安装真实工具；UI 服务/同步不访问真实远端。确需模拟副作用时仅使用夹具所有的临时目录和假工具。

最终核对父 AC1–AC10，不把 C1 格式测试替代 terminal evidence。回退只恢复批准的文本、formatter 使用和文档；账号、数据、服务和外部状态不变。
