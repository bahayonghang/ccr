# zod 试点结论（批次 7 / design.md §9）

> 试点对象：最小页面调用的 `check_version` IPC wrapper（`src/api/domains/system.ts`
> 的 `getVersion`），返回值类型为 ts-rs 生成的
> `src/types/generated/system/VersionInfo.ts`：
> `{ current: string, latest: string | null, update_available: boolean }`。

## Schema 代码

`ccr-ui/src/schemas/versionInfo.ts`（12 行）：

```ts
import { z } from 'zod'

export const versionInfoSchema = z.object({
  current: z.string(),
  latest: z.string().nullable(),
  update_available: z.boolean(),
})

export type VersionInfoParsed = z.infer<typeof versionInfoSchema>
```

一致性锚定测试：`tests/zod-pilot.smoke.test.ts`（保留，供 08-22-state-logic-port 参考）。

## 一致性证明方法与结果

1. **编译期**：测试文件内写类型等价断言
   `Equal<ReturnType<typeof versionInfoSchema.parse>, VersionInfo> = true`
   （`Equal` 为标准的互变位比较）。schema 字段增删或可空性漂移会使 `tsc --noEmit`
   失败。结果：**一致，断言通过**。
2. **运行时**：`safeParse` 桌面端实测返回形态
   `{"current":"7.2.0","latest":null,"update_available":false}` → 通过；缺字段、错型
   （current:number、update_available:string）→ 各自拒绝。结果：**行为符合生成类型的形状约束**。

## 成本测量

| 项目 | 数值 |
| --- | --- |
| schema 手写成本 | 12 行（含注释）；一次通过编译期等价断言与运行时用例，无返工 |
| 测试成本 | 约 45 行（Equal 断言 + 3 组 safeParse 用例） |
| bundle 增量（raw） | index chunk 167,158 B → 226,561 B，**+59,403 B（约 58.0 KiB）** |
| bundle 增量（gzip） | 17,102 B → 33,110 B，**+16,008 B（约 15.6 KiB）** |

测量方法：`bun run build` 基线 vs 在 `src/main.tsx` 临时加入
`import { versionInfoSchema } from './schemas/versionInfo'` 后构建，测完即还原
（已还原并复验产物哈希回到基线 `index-B7xwGjUb.js`）。增量主体为 zod v4 核心运行时；
后续每个新增 schema 只增自身定义，边际体积接近零。

## 推广结论

**值得推广到新增 wrapper**（不回填既有 57 个，与父任务 design.md §15 一致）。依据：

- 一次性成本约 16 KiB gzip（zod 核心），摊薄后单 schema 边际成本趋近于零；
- 手写成本低：对照 ts-rs 生成文件逐字段誊写即可，编译期 Equal 断言把「schema 写错」
  变成构建失败而非静默漏检；
- 收益点在 IPC 边界的版本漂移防护：前端与后端非同进程发布（升级窗口期内可能版本错位），
  运行时校验能把错位从深层渲染错误收敛为边界处的可诊断失败；
- 落位约定：schema 放 `src/schemas/<domain>.ts`，wrapper 内以
  `schema.parse(await invoke(...))` 消费；该落位不在冻结的 `src/api/**` 内，无需解冻。
