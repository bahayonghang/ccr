import { fireEvent, render } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ProfilesQuickRail } from '@/components/profiles'
import { useProfilesQuickSwitch } from '@/composables/useProfilesQuickSwitch'
import { useProfilesQuickSwitchStore } from '@/features/profiles/stores'

const profiles = [
  { name: 'alpha', enabled: true, description: 'Alpha relay' },
  { name: 'beta', enabled: true },
  { name: 'gamma', enabled: false },
  { name: 'delta', enabled: true },
]

function RailHarness(props: {
  moreCount?: number
  onMore?: () => void
  onApply?: (name: string) => void
}) {
  const quickSwitch = useProfilesQuickSwitch({
    platform: 'rail-test',
    getProfileNames: () => profiles.map((profile) => profile.name),
  })
  return (
    <ProfilesQuickRail
      profiles={profiles}
      currentName="alpha"
      i18nPrefix="claudeProfiles"
      quickSwitch={quickSwitch}
      moreCount={props.moreCount ?? 0}
      onApply={props.onApply ?? (() => undefined)}
      onMore={props.onMore ?? (() => undefined)}
    />
  )
}

describe('ProfilesQuickRail quickSwitch mode smoke', () => {
  beforeEach(() => {
    localStorage.clear()
    useProfilesQuickSwitchStore.setState({
      pinnedByPlatform: { 'rail-test': ['alpha', 'gamma'] },
      recentByPlatform: { 'rail-test': ['beta', 'delta'] },
    })
  })

  it('numbers only pinned chips; recent chips stay unnumbered', () => {
    render(<RailHarness />)
    const chips = Array.from(document.querySelectorAll<HTMLButtonElement>('.cp-chip--switch'))
    expect(chips.map((chip) => chip.querySelector('.cp-chip__name')?.textContent)).toEqual([
      'alpha',
      'gamma',
      'beta',
      'delta',
    ])
    expect(chips[0]?.querySelector('.cp-chip__kbd')?.textContent).toBe('1')
    expect(chips[1]?.querySelector('.cp-chip__kbd')?.textContent).toBe('2')
    expect(chips[2]?.querySelector('.cp-chip__kbd')).toBeNull()
    expect(chips[3]?.querySelector('.cp-chip__kbd')).toBeNull()
    expect(chips[1]?.disabled).toBe(true)
  })

  it('keeps a single tab stop and roves focus with arrow/Home/End keys', () => {
    render(<RailHarness />)
    const list = document.querySelector('.cp-rail__list')
    const chips = () => Array.from(document.querySelectorAll<HTMLButtonElement>('.cp-chip--switch'))
    expect(chips().map((chip) => chip.tabIndex)).toEqual([0, -1, -1, -1])
    fireEvent.keyDown(list as HTMLElement, { key: 'ArrowRight' })
    expect(chips().map((chip) => chip.tabIndex)).toEqual([-1, 0, -1, -1])
    fireEvent.keyDown(list as HTMLElement, { key: 'End' })
    expect(chips().map((chip) => chip.tabIndex)).toEqual([-1, -1, -1, 0])
    fireEvent.keyDown(list as HTMLElement, { key: 'Home' })
    expect(chips()[0]?.tabIndex).toBe(0)
  })

  it('renders the more entry and emits more when clicked', () => {
    const onMore = vi.fn()
    render(<RailHarness moreCount={12} onMore={onMore} />)
    const more = document.querySelector('.cp-chip--more') as HTMLButtonElement
    expect(more).toBeTruthy()
    fireEvent.click(more)
    expect(onMore).toHaveBeenCalledTimes(1)
  })
})
