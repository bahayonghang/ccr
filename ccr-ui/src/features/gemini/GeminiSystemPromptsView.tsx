import { SystemPromptsView, systemPromptsConfigs } from '@/features/platform'

export function GeminiSystemPromptsView() {
  return <SystemPromptsView config={systemPromptsConfigs.gemini} />
}
