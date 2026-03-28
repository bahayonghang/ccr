export type RuntimeLimitedFeature = 'skills' | 'usage' | 'commands' | 'sync' | 'generic'

const INVOKE_UNAVAILABLE_PATTERNS = [
  /reading ['"]invoke['"]/i,
  /\binvoke\b/i,
  /__tauri__/i,
  /\btauri\b/i,
]

export const isRuntimeUnavailableError = (value: unknown): boolean => {
  const message = value instanceof Error ? value.message : String(value ?? '')
  return INVOKE_UNAVAILABLE_PATTERNS.some((pattern) => pattern.test(message))
}

export const getRuntimeUnavailableCopy = (feature: RuntimeLimitedFeature) => {
  const common = {
    title: '此功能仅在桌面模式下可用',
    actionLabel: '返回首页',
  }

  switch (feature) {
    case 'skills':
      return {
        ...common,
        description: 'Skills 库依赖本地 CLI 环境、文件系统和 Tauri invoke bridge。Web 预览只验证布局与交互，不执行实际技能仓库读写。',
      }
    case 'usage':
      return {
        ...common,
        description: '使用统计依赖本地 usage 日志导入和桌面端命令桥接。Web 预览中不会读取本机日志，因此只保留页面结构验证。',
      }
    case 'commands':
      return {
        ...common,
        description: '命令执行中心依赖桌面端命令桥接。Web 模式下可以查看工作台布局，但不会执行本地 CLI 命令。',
      }
    case 'sync':
      return {
        ...common,
        description: '云同步需要读取本地目录、配置和桌面端同步命令。Web 预览可查看页面结构，但不会执行同步操作。',
      }
    default:
      return {
        ...common,
        description: '这个模块依赖 Tauri 桌面运行时和本地命令桥接。Web 预览仅用于验证界面结构与视觉效果。',
      }
  }
}
