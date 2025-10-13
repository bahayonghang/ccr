// 🖱️ WSL WebKit 滚轮事件修复
// 强制启用鼠标滚轮和触摸滚动

export function initScrollFix() {
  console.log('🖱️ 初始化滚轮事件修复...')

  // 1. 确保 body 可滚动
  document.body.style.overflow = 'auto'
  ;(document.body.style as any).webkitOverflowScrolling = 'touch'
  document.body.style.touchAction = 'pan-y'

  // 2. 监听滚轮事件（防止被阻止）
  const wheelHandler = (e: WheelEvent) => {
    // 如果事件被阻止，强制滚动
    const target = e.target as HTMLElement
    const scrollableParent = findScrollableParent(target)
    
    if (scrollableParent && e.deltaY !== 0) {
      const delta = e.deltaY
      const currentScroll = scrollableParent.scrollTop
      const newScroll = currentScroll + delta
      
      // 使用 requestAnimationFrame 平滑滚动
      requestAnimationFrame(() => {
        scrollableParent.scrollTop = newScroll
      })
    }
  }

  // 3. 在捕获阶段监听滚轮事件
  window.addEventListener('wheel', wheelHandler, { passive: false, capture: true })
  window.addEventListener('mousewheel', wheelHandler as any, { passive: false, capture: true })

  // 4. 触摸事件支持（触摸板）
  let touchStartY = 0
  window.addEventListener('touchstart', (e) => {
    touchStartY = e.touches[0].clientY
  }, { passive: true })

  window.addEventListener('touchmove', (e) => {
    const touchY = e.touches[0].clientY
    const delta = touchStartY - touchY
    touchStartY = touchY

    const target = e.target as HTMLElement
    const scrollableParent = findScrollableParent(target)
    
    if (scrollableParent) {
      scrollableParent.scrollTop += delta
    }
  }, { passive: true })

  // 5. 强制所有可滚动容器启用滚轮
  const enableScrollOnElement = (selector: string) => {
    const elements = document.querySelectorAll(selector)
    elements.forEach((el) => {
      const element = el as HTMLElement
      element.style.overflow = 'auto'
      ;(element.style as any).webkitOverflowScrolling = 'touch'
      element.style.touchAction = 'pan-y'
    })
  }

  // 监听 DOM 变化，动态启用滚动
  const observer = new MutationObserver(() => {
    enableScrollOnElement('.configs-section')
    enableScrollOnElement('.sidebar-right')
    enableScrollOnElement('.modal-content')
    enableScrollOnElement('.tab-content')
  })

  observer.observe(document.body, {
    childList: true,
    subtree: true
  })

  // 初始化时启用一次
  setTimeout(() => {
    enableScrollOnElement('.configs-section')
    enableScrollOnElement('.sidebar-right')
    enableScrollOnElement('.modal-content')
    enableScrollOnElement('.tab-content')
    enableScrollOnElement('body')
    enableScrollOnElement('#app')
  }, 100)

  console.log('✅ 滚轮事件修复已启用')
}

// 查找最近的可滚动父元素
function findScrollableParent(element: HTMLElement | null): HTMLElement | null {
  if (!element) return document.body

  const parent = element.parentElement
  if (!parent) return document.body

  const overflow = window.getComputedStyle(parent).overflow
  if (overflow === 'auto' || overflow === 'scroll') {
    return parent
  }

  return findScrollableParent(parent)
}

// 强制刷新滚动容器
export function refreshScrollContainers() {
  const scrollableSelectors = [
    'body',
    '#app',
    '.app',
    '.container',
    '.configs-section',
    '.sidebar-right',
    '.modal-content',
    '.tab-content'
  ]

  scrollableSelectors.forEach(selector => {
    const elements = document.querySelectorAll(selector)
    elements.forEach(el => {
      const element = el as HTMLElement
      if (element) {
        // 强制重新计算滚动
        element.style.overflow = 'auto'
        element.scrollTop = element.scrollTop // 触发重绘
      }
    })
  })
}

