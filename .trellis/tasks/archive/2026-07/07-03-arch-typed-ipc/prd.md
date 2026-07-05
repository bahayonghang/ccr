# Tauri IPC seam 类型化

## Goal

244/312 命令返回 `Result<Value, String>`，IPC 往返零测试。用 specta/ts-rs 生成共享 DTO 使 Rust↔TS 漂移变编译错误；命令体下沉为 State-free service 函数以便无 Tauri app 单测。**先从单个 domain 试点**，验证收益后再推广。审查候选 4（Worth exploring）。

## Requirements

### 现状（探索报告定位）

- 312 个命令（Windows 320）、30 个注册表模块；244 个返回 `Result<Value, String>`（78% 无类型）。TS 侧 ~294 个 `<T = UnknownRecord>` 转发 wrapper + tauri.ts 手搓 `asRecord/asArray` 强转。
- 加一个命令的最小链条 4 个文件：命令 fn + handler_registry 条目 + 计数测试 + TS wrapper。
- 测试缺口：81 个前端 smoke 测试无一 mock invoke（结构守卫而非行为测试）；Rust 侧 16 个命令文件的 mod tests 只测抽出的纯 helper；24 个命令文件耦合 `State<'_, AppState>`，无运行中的 app + DB pool 即不可测 → IPC 往返两侧均零行为覆盖，shape 漂移无编译器兜底。
- 重灾区：codex domain 63 命令 / codex.rs 76K；usage 家族 4 组命令（stats 10 + stats_extended 7 + claude_observer 9 + usage_v2 17）互有重叠。

### 要做的（试点范围）

1. 选定一个 domain 试点（usage 或 codex 的一个子集，设计阶段定），引入 DTO 代码生成（specta 或 ts-rs，二选一在设计阶段对比）：Rust 定义一次，TS 类型生成，接入 type-check。
2. 试点 domain 的命令体重构为 thin adapter：`#[tauri::command]` 只做 DTO/State 提取，业务逻辑下沉到 State-free service 函数。
3. 为下沉后的 service 函数补单元测试；为试点 domain 的 DTO 加生成漂移守卫（生成物过期则 CI 失败）。
4. 产出推广评估：试点结论（成本/收益/生成物工程化问题）写入任务 notes，决定是否为其余 29 个 domain 立后续任务。
5. 顺带盘点 usage 4 命令家族的重叠，标记可被 usage_v2 吸收的冗余命令（只标记，删除另立任务）。

### 约束

- 遵守 `tauri-handler-registry.md`：注册仍走 define_command_registry!，计数测试有意更新。
- 遵守 `api-facade-boundary.md`：生成的 TS 客户端/新 wrapper 不进 tauri.ts；生成物漂移检查独立于该手工 facade 守卫。
- 不破坏未试点 domain 的现状；试点期间新旧风格并存是预期状态。

## Acceptance Criteria

- [ ] 试点 domain 的命令返回类型全部为具名 DTO（该 domain 内 `Result<Value, String>` 清零）。
- [ ] TS 侧该 domain 的 wrapper 使用生成类型，`<T = UnknownRecord>` 在该 domain 清零；`bun run type-check` 通过。
- [ ] 试点 domain 的业务逻辑可在无 Tauri app 下单测（`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml <domain>`）。
- [ ] DTO 生成漂移守卫接入（生成物与源不一致时可检测）。
- [ ] handler_registry 计数测试、api-facade-boundary smoke、`just frontend-check-quick` 全绿。
- [ ] 推广/放弃决策与理由记录在案；若放弃推广，理由写入 spec（trellis-update-spec）防止重复提议。

## Notes

- 复杂任务：`task.py start` 前需补 design.md（specta vs ts-rs 对比、试点 domain 选择、生成物落盘位置与 CI 接线）与 implement.md。
- 若 07-03-arch-usage-projection 先行完成，usage domain 命令会变薄，届时以 usage 为试点更顺——软依赖，非阻塞。
