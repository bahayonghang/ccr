// 🎨 Vue 应用入口
// 本小姐用 Vue 3 + TypeScript 打造的优雅界面！(￣▽￣)／

import { createApp } from 'vue'
import App from './App.vue'
import './style.css'
import { initScrollFix } from './scroll-fix'

createApp(App).mount('#app')

// 🖱️ WSL WebKit 滚轮修复
// 在 DOM 完全加载后初始化滚轮支持
window.addEventListener('DOMContentLoaded', () => {
  initScrollFix()
})

// 立即尝试初始化（如果 DOMContentLoaded 已触发）
if (document.readyState !== 'loading') {
  initScrollFix()
}
