export const CHART_RESIZE_THROTTLE_MS = 150

/** 节流窗口缩放回调，避免连续 resize 触发图表重建。 */
export function createThrottledResize(
  onResize: () => void,
  waitMs = CHART_RESIZE_THROTTLE_MS,
): () => void {
  let lastRun = 0
  let timer: ReturnType<typeof setTimeout> | null = null

  return () => {
    const now = Date.now()
    const remaining = waitMs - (now - lastRun)

    const run = () => {
      lastRun = Date.now()
      timer = null
      onResize()
    }

    if (remaining <= 0) {
      if (timer) {
        clearTimeout(timer)
        timer = null
      }
      run()
      return
    }

    if (timer) return
    timer = setTimeout(run, remaining)
  }
}
