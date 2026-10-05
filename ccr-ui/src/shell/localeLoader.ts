import type { LoaderFunctionArgs } from 'react-router'
import { flattenCatalog } from './routeCatalog'
import { hydrateShellLocale } from './i18n'

const deferPaths = new Set(
  flattenCatalog()
    .filter((route) => route.handle?.deferLocaleHydration)
    .map((route) => route.path),
)

/** 非 defer 路由阻塞到完整 locale 就绪；defer 路由让首帧先用 boot 文案。 */
export async function localeWarmupLoader({ request }: LoaderFunctionArgs): Promise<null> {
  const pathname = new URL(request.url).pathname
  if (deferPaths.has(pathname)) return null
  await hydrateShellLocale()
  return null
}
