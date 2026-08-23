import { grokAuthConfig } from '@/configs/auth'
import { BaseAuth } from '@/features/platform'

export function GrokAuthView() {
  return <BaseAuth config={grokAuthConfig} />
}
