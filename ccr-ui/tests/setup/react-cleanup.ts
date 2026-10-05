// React 测试工具在每个用例后卸载挂载的组件，防止用例间 DOM 与订阅泄漏
import { cleanup } from '@testing-library/react'
import { afterEach } from 'vitest'

afterEach(() => {
  cleanup()
})
