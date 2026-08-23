import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from 'react'
import type { IconName } from '@/config/icons'
import { useShellT } from '@/shell/i18n'
import { BaseModal, SIcon } from '@/ui'
import './profiles-shared.css'

/** 调用方注入的命令项：label 走 i18n key，图标固定，点击直接执行 handler */
export interface ProfilesCommandPaletteAction {
  id: string
  icon: IconName
  labelKey: string
  handler: () => void
}

/** 调用方注入的 profile 策略：谁能被切换 + 副标题文案来源 */
export interface ProfilesCommandPaletteDescriptor<T> {
  isEnabled: (profile: T) => boolean
  hint: (profile: T) => string | undefined
}

export interface ProfilesCommandPaletteProps<T extends { name: string }> {
  open: boolean
  profiles: T[]
  descriptor: ProfilesCommandPaletteDescriptor<T>
  actions: ProfilesCommandPaletteAction[]
  /** i18n key 前缀，指向 commandPalette 子对象，例如 'codex.profiles.commandPalette' */
  i18nPrefix: string
  onUpdateOpen: (value: boolean) => void
  onApply: (name: string) => void
}

interface PaletteItem {
  id: string
  kind: 'cmd' | 'switch'
  label: string
  hint?: string
  icon: IconName
  action: () => void
}

interface IndexedPaletteItem extends PaletteItem {
  index: number
}

interface PaletteGroup {
  id: 'commands' | 'profiles'
  title: string
  items: IndexedPaletteItem[]
}

const optionId = (index: number) => `cp-command-palette-option-${index}`

const matchesQuery = (item: PaletteItem, query: string) => {
  const q = query.trim().toLowerCase()
  if (!q) return true
  return `${item.label} ${item.hint ?? ''}`.toLowerCase().includes(q)
}

const stepActive = (key: string, current: number, count: number): number | null => {
  if (count === 0) return null
  if (key === 'ArrowDown') return Math.min(count - 1, current + 1)
  if (key === 'ArrowUp') return Math.max(0, current - 1)
  return null
}

/** ⌘K 命令面板：模糊搜索切换 profile 与执行常用命令。 */
export function ProfilesCommandPalette<T extends { name: string }>({
  open,
  profiles,
  descriptor,
  actions,
  i18nPrefix,
  onUpdateOpen,
  onApply,
}: ProfilesCommandPaletteProps<T>) {
  const t = useShellT()
  const [query, setQuery] = useState('')
  const [activeIdx, setActiveIdx] = useState(0)
  const inputRef = useRef<HTMLInputElement | null>(null)
  const listRef = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    if (!open) return
    setQuery('')
    setActiveIdx(0)
    inputRef.current?.focus()
  }, [open])

  const baseCommands = useMemo<PaletteItem[]>(
    () =>
      actions.map((action) => ({
        id: action.id,
        kind: 'cmd' as const,
        label: t(action.labelKey),
        icon: action.icon,
        action: action.handler,
      })),
    [actions, t],
  )

  const switchItems = useMemo<PaletteItem[]>(
    () =>
      profiles
        .filter((profile) => descriptor.isEnabled(profile))
        .map((profile) => ({
          id: profile.name,
          kind: 'switch' as const,
          label: t(`${i18nPrefix}.actionApply`, { name: profile.name }),
          hint: descriptor.hint(profile),
          icon: 'Play' as IconName,
          action: () => onApply(profile.name),
        })),
    [descriptor, i18nPrefix, onApply, profiles, t],
  )

  const filteredBuckets = useMemo(
    () => ({
      commands: baseCommands.filter((item) => matchesQuery(item, query)),
      profiles: switchItems.filter((item) => matchesQuery(item, query)),
    }),
    [baseCommands, query, switchItems],
  )

  const items = useMemo<PaletteItem[]>(
    () => [...filteredBuckets.commands, ...filteredBuckets.profiles],
    [filteredBuckets],
  )

  const groups = useMemo<PaletteGroup[]>(() => {
    const commandItems = filteredBuckets.commands.map((item, index) => ({ ...item, index }))
    const profileItems = filteredBuckets.profiles.map((item, index) => ({
      ...item,
      index: commandItems.length + index,
    }))
    return [
      { id: 'commands', title: t(`${i18nPrefix}.groupCommands`), items: commandItems },
      { id: 'profiles', title: t(`${i18nPrefix}.groupProfiles`), items: profileItems },
    ]
  }, [filteredBuckets, i18nPrefix, t])

  const resultSummary = t(`${i18nPrefix}.resultSummary`, {
    commands: filteredBuckets.commands.length,
    profiles: filteredBuckets.profiles.length,
  })

  const activeOptionId = items.length > 0 ? optionId(activeIdx) : undefined

  useEffect(() => {
    if (activeIdx >= items.length) {
      setActiveIdx(Math.max(0, items.length - 1))
    }
  }, [activeIdx, items.length])

  const fire = (item: PaletteItem) => {
    item.action()
    onUpdateOpen(false)
  }

  const scrollActiveIntoView = (index: number) => {
    const el = listRef.current?.querySelector<HTMLElement>(`[data-index="${index}"]`)
    el?.scrollIntoView({ block: 'nearest' })
  }

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    const stepped = stepActive(event.key, activeIdx, items.length)
    if (stepped !== null) {
      event.preventDefault()
      setActiveIdx(stepped)
      scrollActiveIntoView(stepped)
      return
    }
    if (event.key === 'Enter') {
      event.preventDefault()
      const item = items[activeIdx]
      if (item) fire(item)
      return
    }
    if (event.key === 'Escape') {
      event.preventDefault()
      event.stopPropagation()
      onUpdateOpen(false)
    }
  }

  return (
    <BaseModal
      modelValue={open}
      onUpdateModelValue={onUpdateOpen}
      description={resultSummary}
      size="full"
      surface="solid"
      contentClass="cp-palette-modal"
      showClose={false}
      header={({ titleId }) => (
        <div className="cp-palette__header">
          <div className="cp-palette__heading">
            <div className="cp-palette__eyebrow">{t(`${i18nPrefix}.eyebrow`)}</div>
            <h2 id={titleId} className="cp-palette__title">
              {t(`${i18nPrefix}.title`)}
            </h2>
          </div>
          <div className="cp-palette__summary" aria-live="polite">
            {resultSummary}
          </div>
        </div>
      )}
      footer={
        <div className="cp-palette__foot">
          <span>
            <kbd className="cp-palette__kbd">↵</kbd> {t(`${i18nPrefix}.execute`)}
          </span>
          <span>
            <kbd className="cp-palette__kbd">↑↓</kbd> {t(`${i18nPrefix}.select`)}
          </span>
          <span>
            <kbd className="cp-palette__kbd">Esc</kbd> {t(`${i18nPrefix}.close`)}
          </span>
        </div>
      }
    >
      <div className="cp-palette">
        <label className="sr-only" htmlFor="cp-command-palette-search">
          {t(`${i18nPrefix}.placeholder`)}
        </label>
        <div className="cp-palette__search-wrap">
          <SIcon name="Search" size="w-4 h-4" className="cp-palette__search-icon" />
          <input
            id="cp-command-palette-search"
            ref={inputRef}
            value={query}
            className="cp-palette__search"
            placeholder={t(`${i18nPrefix}.placeholder`)}
            aria-label={t(`${i18nPrefix}.placeholder`)}
            aria-activedescendant={activeOptionId}
            aria-controls="cp-command-palette-list"
            onInput={(event) => setQuery(event.currentTarget.value)}
            onKeyDown={onKeyDown}
          />
        </div>

        <div
          id="cp-command-palette-list"
          ref={listRef}
          className="cp-palette__list"
          role="listbox"
          aria-label={t(`${i18nPrefix}.title`)}
        >
          {groups.map((group) =>
            group.items.length > 0 ? (
              <section key={group.id} className="cp-palette__group" aria-label={group.title}>
                <div className="cp-palette__group-head">
                  <span>{group.title}</span>
                  <span className="cp-palette__group-count">{group.items.length}</span>
                </div>
                {group.items.map((item) => (
                  <div
                    id={optionId(item.index)}
                    key={item.id}
                    data-index={item.index}
                    className={
                      item.index === activeIdx ? 'cp-palette__row cp-palette__row--active' : 'cp-palette__row'
                    }
                    role="option"
                    aria-selected={item.index === activeIdx}
                    onMouseEnter={() => setActiveIdx(item.index)}
                    onClick={() => fire(item)}
                  >
                    <span className="cp-palette__icon-box">
                      <SIcon
                        name={item.icon}
                        size="w-3.5 h-3.5"
                        className={item.index === activeIdx ? 'cp-palette__icon--accent' : 'cp-palette__icon'}
                      />
                    </span>
                    <div className="cp-palette__main">
                      <div
                        className={
                          item.kind === 'switch' ? 'cp-palette__label cp-palette__label--mono' : 'cp-palette__label'
                        }
                      >
                        {item.label}
                      </div>
                      {item.hint ? <div className="cp-palette__sub">{item.hint}</div> : null}
                    </div>
                    <span className="cp-palette__badge">
                      {item.kind === 'switch' ? t(`${i18nPrefix}.kindSwitch`) : t(`${i18nPrefix}.kindCommand`)}
                    </span>
                  </div>
                ))}
              </section>
            ) : null,
          )}

          {items.length === 0 ? <div className="cp-palette__empty">{t(`${i18nPrefix}.empty`)}</div> : null}
        </div>
      </div>
    </BaseModal>
  )
}
