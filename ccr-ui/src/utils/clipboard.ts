/**
 * 剪贴板写入工具。
 *
 * 收口此前两个互不知晓的实现（codexHelpers.copyToClipboard / opencode.copyText），
 * 优先使用异步 Clipboard API，失败时降级到 execCommand 以兼容旧 WebView。
 */

/**
 * Copy text to the clipboard, falling back to a hidden textarea + execCommand
 * when the async Clipboard API is unavailable or rejected.
 *
 * @returns whether the copy succeeded
 */
export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    // 降级：旧环境或非安全上下文下 navigator.clipboard 不可用
    try {
      const textarea = document.createElement('textarea')
      textarea.value = text
      textarea.style.position = 'fixed'
      textarea.style.opacity = '0'
      document.body.appendChild(textarea)
      textarea.select()
      document.execCommand('copy')
      document.body.removeChild(textarea)
      return true
    } catch {
      return false
    }
  }
}
