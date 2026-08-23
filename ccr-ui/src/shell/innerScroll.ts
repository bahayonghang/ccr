/** 内部滚动容器的按路径滚动位置。ScrollRestoration 只管 window，这里接管内容区。 */

const positions = new Map<string, number>()

export const CONTENT_SCROLL_SELECTOR = '.content-scroll-area'

export function saveInnerScroll(pathname: string, top: number): void {
  positions.set(pathname, top)
}

export function readInnerScroll(pathname: string): number | undefined {
  return positions.get(pathname)
}

export function restoreInnerScroll(options: {
  pathname: string
  cache: boolean
  element: HTMLElement | null
}): void {
  const { pathname, cache, element } = options
  if (!element) return
  const nextTop = cache ? (positions.get(pathname) ?? 0) : 0
  element.scrollTop = nextTop
}
