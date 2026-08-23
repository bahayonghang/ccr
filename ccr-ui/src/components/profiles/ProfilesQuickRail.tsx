import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from 'react'
import type { ProfilesQuickSwitch } from '@/composables/useProfilesQuickSwitch'
import { PROFILES_PIN_CAP } from '@/features/profiles/stores'
import { useShellT } from '@/shell/i18n'
import { SIcon } from '@/ui'
import './profiles-shared.css'

/** QuickRail 直接读取的最小 profile 形状（两平台共有字段） */
export interface QuickRailProfile {
  name: string
  enabled?: boolean | null
  description?: string | null
}

export interface ProfilesQuickRailProps<T extends QuickRailProfile> {
  profiles: T[]
  currentName: string | null
  /** i18n key 前缀，例如 'claudeProfiles' / 'codex.profiles' */
  i18nPrefix: string
  disabled?: boolean
  busyName?: string | null
  /** 快速切换状态（useProfilesQuickSwitch 返回值） */
  quickSwitch: ProfilesQuickSwitch
  /** 栏容量之外的可用 profile 数（>0 时渲染「+N more → ⌘K」入口） */
  moreCount?: number
  onApply: (name: string) => void
  onMore: () => void
}

interface SwitchChip {
  name: string
  pinned: boolean
  /** 钉选 chip 的序号（1..n）；recent chip 恒为 null（只展示不编号） */
  number: number | null
  /** 名称不在当前列表或已禁用：chip 置灰不可 Apply，但不自动移除 */
  unavailable: boolean
  title: string
}

const buildRailChips = <T extends QuickRailProfile>(
  profiles: T[],
  quickSwitch: ProfilesQuickSwitch,
): SwitchChip[] => {
  const byName = new Map(profiles.map((profile) => [profile.name, profile]))
  const toChip = (name: string, pinned: boolean, number: number | null): SwitchChip => {
    const profile = byName.get(name)
    return {
      name,
      pinned,
      number,
      unavailable: !profile || profile.enabled === false,
      title: profile?.description || name,
    }
  }
  const chips = quickSwitch.pinned.map((name, index) => toChip(name, true, index + 1))
  for (const name of quickSwitch.recentNotPinned) {
    if (chips.length >= PROFILES_PIN_CAP) break
    chips.push(toChip(name, false, null))
  }
  return chips
}

const nextFocusIndex = (eventKey: string, current: number, count: number): number | null => {
  if (eventKey === 'ArrowRight') return (current + 1) % count
  if (eventKey === 'ArrowLeft') return (current - 1 + count) % count
  if (eventKey === 'Home') return 0
  if (eventKey === 'End') return count - 1
  return null
}

/** 快速切换条：钉选项稳定编号，最近项不编号。 */
export function ProfilesQuickRail<T extends QuickRailProfile>({
  profiles,
  currentName,
  i18nPrefix,
  disabled = false,
  busyName = null,
  quickSwitch,
  moreCount = 0,
  onApply,
  onMore,
}: ProfilesQuickRailProps<T>) {
  const t = useShellT()
  const switchListRef = useRef<HTMLDivElement | null>(null)
  const [switchFocusIdx, setSwitchFocusIdx] = useState(0)

  const railChips = useMemo(
    () => buildRailChips(profiles, quickSwitch),
    [profiles, quickSwitch],
  )

  useEffect(() => {
    if (switchFocusIdx >= railChips.length) {
      setSwitchFocusIdx(Math.max(0, railChips.length - 1))
    }
  }, [railChips.length, switchFocusIdx])

  const focusSwitchChip = (index: number) => {
    setSwitchFocusIdx(index)
    const chips = switchListRef.current?.querySelectorAll<HTMLElement>('.cp-chip--switch')
    chips?.[index]?.focus()
  }

  const onSwitchKeydown = (event: KeyboardEvent<HTMLElement>) => {
    const count = railChips.length
    if (count === 0) return
    const next = nextFocusIndex(event.key, switchFocusIdx, count)
    if (next === null) return
    event.preventDefault()
    focusSwitchChip(next)
  }

  if (railChips.length === 0) return null

  return (
    <div className="cp-rail surface-workspace">
      <div className="cp-rail__head">
        <SIcon name="Sparkles" size="w-3.5 h-3.5" className="cp-rail__head-icon" />
        {t(`${i18nPrefix}.quickSwitch`)}
      </div>
      <div
        ref={switchListRef}
        className="cp-rail__list"
        role="toolbar"
        aria-label={t(`${i18nPrefix}.quickSwitch`)}
        onKeyDown={onSwitchKeydown}
      >
        {railChips.map((chip, index) => {
          const active = chip.name === currentName
          const chipClass = [
            'cp-chip',
            'cp-chip--switch',
            active ? 'cp-chip--active' : '',
            busyName === chip.name ? 'cp-chip--busy' : '',
          ]
            .filter(Boolean)
            .join(' ')
          const pinLabel = chip.pinned
            ? t(`${i18nPrefix}.unpinProfile`, { name: chip.name })
            : t(`${i18nPrefix}.pinProfile`, { name: chip.name })
          return (
            <span key={chip.name} className="cp-chip-wrap">
              <button
                type="button"
                className={chipClass}
                tabIndex={index === switchFocusIdx ? 0 : -1}
                disabled={disabled || chip.unavailable}
                aria-pressed={active}
                title={chip.title}
                onClick={() => {
                  setSwitchFocusIdx(index)
                  onApply(chip.name)
                }}
                onFocus={() => setSwitchFocusIdx(index)}
              >
                <span className={active ? 'cp-chip__dot' : 'cp-chip__dot cp-chip__dot--off'} />
                <span className="cp-chip__name">{chip.name}</span>
                {chip.number !== null ? <span className="cp-chip__kbd">{chip.number}</span> : null}
              </button>
              <button
                type="button"
                className={chip.pinned ? 'cp-chip__pin cp-chip__pin--on' : 'cp-chip__pin'}
                tabIndex={-1}
                aria-label={pinLabel}
                onClick={() => quickSwitch.togglePin(chip.name)}
              >
                <SIcon name={chip.pinned ? 'PinOff' : 'Pin'} size="w-3 h-3" />
              </button>
            </span>
          )
        })}
        {moreCount > 0 ? (
          <button type="button" className="cp-chip cp-chip--more" tabIndex={-1} onClick={onMore}>
            <span className="cp-chip__name">{t(`${i18nPrefix}.quickRailMore`, { count: moreCount })}</span>
            <kbd className="cp-chip__kbd">{quickSwitch.modifier}K</kbd>
          </button>
        ) : null}
      </div>
      <div className="cp-rail__hint">
        {t(`${i18nPrefix}.quickRailModifierHint`, { modifier: quickSwitch.modifier })}
      </div>
    </div>
  )
}
