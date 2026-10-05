// 内置站反查辅助：消除按 name join 的断链（provider 改名后仍能正确关联内置站）
//
// 后端等价实现见 crates/ccr-checkin/src/managers/checkin/builtin_providers.rs
// 的 resolve_builtin_for_provider：builtin_id 优先，旧数据（无 builtin_id）回退 name 匹配。

import type { BuiltinProvider, CheckinProvider } from '@/types/checkin'

/** 解析 provider 关联的内置站：builtin_id 优先（改名安全），无/查不到时回退 name 匹配 */
export function resolveBuiltinProvider(
  builtinProviders: BuiltinProvider[],
  provider: Pick<CheckinProvider, 'name' | 'builtin_id'>,
): BuiltinProvider | undefined {
  if (provider.builtin_id) {
    const byId = builtinProviders.find((bp) => bp.id === provider.builtin_id)
    if (byId) return byId
  }
  return builtinProviders.find((bp) => bp.name === provider.name)
}

/** 过滤出尚未添加的内置站：已添加判定按 builtin_id 优先，无 builtin_id 的旧行回退 name 匹配 */
export function filterAvailableBuiltinProviders(
  builtinProviders: BuiltinProvider[],
  providers: Array<Pick<CheckinProvider, 'name' | 'builtin_id'>>,
): BuiltinProvider[] {
  const addedBuiltinIds = new Set<string>()
  const legacyNames = new Set<string>()
  for (const provider of providers) {
    if (provider.builtin_id) {
      addedBuiltinIds.add(provider.builtin_id)
    } else {
      legacyNames.add(provider.name)
    }
  }
  return builtinProviders.filter((bp) => !addedBuiltinIds.has(bp.id) && !legacyNames.has(bp.name))
}
