import { ref, computed, onMounted, onUnmounted, type Ref } from 'vue'

export function useAnimationVisibility(targetRef: Ref<HTMLElement | null>) {
  const isInViewport = ref(true)
  const isPageVisible = ref(true)
  const prefersReducedMotion = ref(false)

  const shouldAnimate = computed(
    () => isInViewport.value && isPageVisible.value && !prefersReducedMotion.value,
  )

  let intersectionObserver: IntersectionObserver | null = null
  let motionMediaQuery: MediaQueryList | null = null

  function handleVisibilityChange() {
    isPageVisible.value = document.visibilityState === 'visible'
  }

  function handleMotionChange(e: MediaQueryListEvent) {
    prefersReducedMotion.value = e.matches
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

    // matchMedia: 检测用户 prefers-reduced-motion 偏好
    motionMediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
    prefersReducedMotion.value = motionMediaQuery.matches
    motionMediaQuery.addEventListener('change', handleMotionChange)
  })

  onUnmounted(() => {
    if (intersectionObserver) {
      intersectionObserver.disconnect()
      intersectionObserver = null
    }

    document.removeEventListener('visibilitychange', handleVisibilityChange)

    if (motionMediaQuery) {
      motionMediaQuery.removeEventListener('change', handleMotionChange)
      motionMediaQuery = null
    }
  })

  return { shouldAnimate, isInViewport, isPageVisible, prefersReducedMotion }
}
