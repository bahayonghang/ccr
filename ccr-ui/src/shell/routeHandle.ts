import { useMatches } from 'react-router'

/** 原 vue-router `RouteMeta` 八个字段，字段名保持不变。 */
export interface RouteHandle {
  cache?: boolean
  cacheKey?: string
  hideGlobalBackground?: boolean
  stream?: boolean
  depth?: number
  group?: string
  hideSidebar?: boolean
  deferLocaleHydration?: boolean
}

export const ROUTE_HANDLE_KEYS = [
  'cache',
  'cacheKey',
  'hideGlobalBackground',
  'stream',
  'depth',
  'group',
  'hideSidebar',
  'deferLocaleHydration',
] as const

export type RouteHandleKey = (typeof ROUTE_HANDLE_KEYS)[number]

const HANDLE_KEY_SET = new Set<string>(ROUTE_HANDLE_KEYS)

/** 断言 handle 的自有键都在允许集合内；空 handle 合法。 */
export function assertHandleKeys(handle: unknown, routeId: string): string[] {
  if (handle == null) return []
  if (typeof handle !== 'object') {
    return [`${routeId}: handle 不是对象`]
  }
  return Object.keys(handle)
    .filter((key) => !HANDLE_KEY_SET.has(key))
    .map((key) => `${routeId}: 非法 handle 字段 ${key}`)
}

/**
 * Read the leaf route handle and narrow it to RouteHandle.
 * Missing handle resolves to an empty object.
 */
export function useRouteHandle(): RouteHandle {
  const matches = useMatches()
  const leaf = matches[matches.length - 1]
  const handle = leaf?.handle
  if (handle && typeof handle === 'object') {
    return handle as RouteHandle
  }
  return {}
}
