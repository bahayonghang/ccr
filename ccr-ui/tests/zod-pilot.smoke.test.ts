import { describe, expect, it } from 'vitest'
import type { VersionInfo } from '@/types/generated/system/VersionInfo'
import { versionInfoSchema } from '@/schemas/versionInfo'

// 编译期类型等价断言：z.infer 与 ts-rs 生成的 VersionInfo 必须互相等价，
// 任一侧漂移（字段增删、可空性变化）都会导致该文件类型检查失败。
type Equal<S, T> =
  (<T1>() => T1 extends S ? 1 : 2) extends <T1>() => T1 extends T ? 1 : 2 ? true : false

const schemaMatchesGeneratedType: Equal<
  ReturnType<typeof versionInfoSchema.parse>,
  VersionInfo
> = true

describe('zod 试点：check_version 返回值（批次 7）', () => {
  it('z.infer 与 ts-rs 生成类型编译期一致', () => {
    // 该断言在编译期已生效；此处仅防被静默删除
    expect(schemaMatchesGeneratedType).toBe(true)
  })

  it('运行时校验接受桌面端实测返回形态，拒绝缺字段/错型载荷', () => {
    // 形态取自 tauri:dev 实测返回：{"current":"7.2.0","latest":null,"update_available":false}
    const valid = versionInfoSchema.safeParse({
      current: '7.2.0',
      latest: null,
      update_available: false,
    })
    expect(valid.success).toBe(true)

    expect(versionInfoSchema.safeParse({ current: '7.2.0' }).success).toBe(false)
    expect(
      versionInfoSchema.safeParse({ current: 7, latest: null, update_available: false }).success,
    ).toBe(false)
    expect(
      versionInfoSchema.safeParse({ current: '7.2.0', latest: '8.0.0', update_available: 'yes' })
        .success,
    ).toBe(false)
  })
})
