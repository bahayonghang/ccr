// zod 试点（08-22-react-foundation 批次 7）：
// 为最小页面调用的 check_version IPC 返回值建立运行时 schema，
// 与 ts-rs 生成的 VersionInfo 类型做编译期一致性锚定。
// 推广结论与测量数据见 .trellis/tasks/08-22-react-foundation/zod-pilot.md。
import { z } from 'zod'

export const versionInfoSchema = z.object({
  current: z.string(),
  latest: z.string().nullable(),
  update_available: z.boolean(),
})

export type VersionInfoParsed = z.infer<typeof versionInfoSchema>
