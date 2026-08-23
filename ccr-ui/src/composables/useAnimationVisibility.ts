import { ref, computed, onMounted, onUnmounted, type Ref } from 'vue'
import { readPrefersReducedMotion } from '@/utils/reducedMotion'

export function useAnimationVisibility(targetRef: Ref<HTMLElement | null>) {
  const isInViewport = ref(true)
  const isPageVisible = ref(true)
  const prefersReducedMotion = ref(false)

  const shouldAnimate = computed(
    () => isInViewport.value && isPageVisible.value && !prefersReducedMotion.value,
  )

  let intersectionObserver: IntersectionObserver | null = null

  function handleVisibilityChange() {
    isPageVisible.value = document.visibilityState === 'visible'
  }

  onMounted(() => {
    if (typeof window === 'undefined') return

    // IntersectionObserver: 检测元素是否在视口内
    intersectionObserver = new IntersectionObserver(
      (entries) => {
        const entry = entries[0]
        if (entry) {
          isInViewport.value = entry.isIntersecting
        }
      },
      { threshold: 0 },
    )

    if (targetRef.value) {
      intersectionObserver.observe(targetRef.value)
    }

    // document.visibilitychange: 检测标签页是否可见
    isPageVisible.value = document.visibilityState === 'visible'
    document.addEventListener('visibilitychange', handleVisibilityChange)

    // prefers-reduced-motion 的读取点已收敛到 reducedMotion.ts（批次 7 单点）；
    // 本组合式随 08-22-state-logic-port 迁移退役，reduced-motion 职责不再在此持有。
    prefersReducedMotion.value = readPrefersReducedMotion()
  })

  onUnmounted(() => {
    if (intersectionObserver) {
      intersectionObserver.disconnect()
      intersectionObserver = null
    }

    document.removeEventListener('visibilitychange', handleVisibilityChange)

  })

  return { shouldAnimate, isInViewport, isPageVisible, prefersReducedMotion }
}
