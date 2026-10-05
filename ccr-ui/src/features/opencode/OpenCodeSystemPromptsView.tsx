import { SystemPromptsView, systemPromptsConfigs } from '@/features/platform'

export function OpenCodeSystemPromptsView() {
  return <SystemPromptsView config={systemPromptsConfigs.opencode} />
}
