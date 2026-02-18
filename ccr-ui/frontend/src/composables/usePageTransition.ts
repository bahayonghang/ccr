import { ref, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'

// 路由感知过渡动画类型
type TransitionName =
  | 'page-fade'
  | 'page-slide-up'
  | 'page-slide-down'
  | 'page-cross-fade'
  | 'page-slide-lateral'

/**
 * 路由感知的智能过渡动画选择 composable
 *
 * 根据导航上下文（层级深度、分组、浏览器后退）自动选择最合适的过渡效果：
 * - page-fade: 首次加载 / 深层链接
 * - page-slide-up: 向下导航（进入子页面）
 * - page-slide-down: 向上导航（返回上级）
 * - page-cross-fade: 不同组的同级切换（平台间跳转）
 * - page-slide-lateral: 同组内同级切换
 */
export function usePageTransition() {
  const transitionName = ref<TransitionName>('page-fade')
  const router = useRouter()
  let isBack = false

  // 监听 popstate 检测浏览器后退/前进
  const onPopState = () => {
    isBack = true
  }
  window.addEventListener('popstate', onPopState)
  onUnmounted(() => window.removeEventListener('popstate', onPopState))

  router.beforeEach((to, from) => {
    // 首次加载 / 无 from
    if (!from.name) {
      transitionName.value = 'page-fade'
      isBack = false
      return
    }

    const toDepth = (to.meta.depth as number) ?? 1
    const fromDepth = (from.meta.depth as number) ?? 1
    const toGroup = (to.meta.group as string) ?? ''
    const fromGroup = (from.meta.group as string) ?? ''

    if (isBack) {
      // 浏览器后退 → 反向动画
      if (fromDepth > toDepth) {
        transitionName.value = 'page-slide-down'
      } else if (fromDepth < toDepth) {
        transitionName.value = 'page-slide-up'
      } else {
        transitionName.value = 'page-cross-fade'
      }
    } else if (toDepth > fromDepth) {
      // 向下导航（进入子页面）
      transitionName.value = 'page-slide-up'
    } else if (toDepth < fromDepth) {
      // 向上导航（返回上级）
      transitionName.value = 'page-slide-down'
    } else if (toGroup && fromGroup && toGroup === fromGroup) {
      // 同组内同级切换（如 codex/mcp → codex/profiles）
      transitionName.value = 'page-slide-lateral'
    } else {
      // 不同组同级切换（如 claude-code → codex）
      transitionName.value = 'page-cross-fade'
    }

    isBack = false
  })

  return { transitionName }
}
