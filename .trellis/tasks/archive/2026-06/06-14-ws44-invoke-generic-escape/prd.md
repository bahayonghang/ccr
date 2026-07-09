# WS4.4 invoke 泛型逃逸收敛 — 前提勘误与重新定范围

## 结论

**不执行 PRD「短期方案：`<T = Default>(...)：Promise<T>` → 固定 `Promise<Default>`，312→0」。**
该方案前提错误且不闭合真正的类型洞，改为推荐两条真正有价值的路径（见下）。

## 前提勘误（2026-06-14 实测）

PRD 4.4 短期方案的依据是「泛型参数实际调用方几乎不用」。实测**为假**：

- `src/api/domains/*` 共 293 处 `<T = UnknownRecord>`。
- 排除 `listen`（Tauri 事件，合法泛型）与组合式（useProfilesFilter/useFuzzySearch/usePolledData/useCachedFetch）后，
  仍有 **~100+ 调用点显式传入 `<ConcreteType>`**：`getOpenCodeConfig<…>`×14、`listConfigs`/`listCheckinRecords`/
  `getUsageDashboardV2`/`listCodexProfiles`/`listOpenCodeMcpServers`… 长尾各 1-3 处。
  （CodexAuthView 一个文件就有 `listCodexAuthAccounts<CodexAuthListResponse>` /
  `getCodexAuthCurrent<…>` / `getCodexAllQuotas<…>` / `codexIsOAuthPortInUse<boolean>` /
  `codexReleaseOAuthPort<number>` 等。）

删除泛型 → 这 ~100 处全部 `Expected 0 type arguments` 报错，需逐处改成
`(await fn()) as ConcreteType` 强转。**强转与泛型默认同样不做校验**，安全性无实质提升，
反而 call-site 更难读。即「312→0」既破坏调用方，又不闭合真正的洞。

## 真正的洞 & 正确方向

危险不在「默认是 UnknownRecord 还是 unknown」，而在 `invoke()` 返回 `Promise<unknown>`
被无校验地静默转型为 `T`。无论 T 是默认还是调用方指定，都没有运行时校验。

- **非破坏的真实价值（可增量做）**：对关键入口用 `src/api/_shared.ts` 的 `isRecord/asRecord/pickArray`
  做运行时收窄。现状已部分落地（claude 16、codex 13、opencode 11、unifiedMcp 5、checkin 5、config 3…处），
  可继续向 codex/claude/checkin 的高频读取入口扩面。
- **根治（PRD 自身 Non-Goal / 长期）**：specta + tauri-specta 以 Rust 为源生成 TS 类型，
  消除手抄与无校验转型。这是唯一真正闭合的方案。

## 验收处置

- AC#7 中「`<T = Default>` 逃逸按 4.4 短期方案收敛（→0）」标注为**前提不成立、主动放弃机械删除**。
- 若需推进，按「运行时收窄扩面（增量、非破坏）」执行，并把 specta 作为独立任务评估。

> 状态（2026-06-14）：评估完成，机械删除方案否决。运行时收窄扩面可作为后续增量 subtask。
