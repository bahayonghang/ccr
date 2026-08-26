import type { ProfileDisplayRecord } from '@/configs/profileDisplayRecord'
import type { ProfilePresentationView } from '@/configs/profilePresentation'

export interface ProfileRowState {
  dotTone: 'active' | 'idle'
  badge: { textKey: string; tone: 'accent' | 'neutral' }
  applyLabelKey: string
  applyTone: 'accent-soft' | 'neutral'
  emphasized: boolean
}

/** 卡片与表格共用的行状态：只读 current 标记，不比较平台名。 */
export function resolveRowState(
  record: ProfileDisplayRecord,
  presentation: ProfilePresentationView,
): ProfileRowState {
  if (record.current) {
    return {
      dotTone: 'active',
      badge: { textKey: 'profilesSurface.statusActive', tone: 'accent' },
      applyLabelKey: 'profilesSurface.stop',
      applyTone: 'accent-soft',
      emphasized: true,
    }
  }
  return {
    dotTone: 'idle',
    badge: { textKey: presentation.nameKey, tone: 'neutral' },
    applyLabelKey: 'profilesSurface.apply',
    applyTone: 'neutral',
    emphasized: false,
  }
}
