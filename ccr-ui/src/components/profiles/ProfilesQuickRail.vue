<!-- 快速切换条：仅展示已启用 profile，附带 ⌘+数字键提示。
     平台差异通过 i18nPrefix 注入；样式靠视图的 --cp-* 继承换肤。 -->
<template>
  <!-- 新模式（quickSwitch 注入时）：钉选编号 chip + recent 无编号 chip + pin 操作 + more 入口。
       编号只出现在钉选 chip 上；recent 只展示不编号。role="toolbar" + roving tabindex。 -->
  <div
    v-if="quickSwitch"
    v-show="railChips.length > 0"
    class="cp-rail surface-workspace"
  >
    <div class="cp-rail__head">
      <SIcon
        name="Sparkles"
        size="w-3.5 h-3.5"
        class="cp-rail__head-icon"
      />
      {{ t(`${i18nPrefix}.quickSwitch`) }}
    </div>
    <div
      ref="switchListRef"
      class="cp-rail__list"
      role="toolbar"
      :aria-label="t(`${i18nPrefix}.quickSwitch`)"
      @keydown="onSwitchKeydown"
    >
      <span
        v-for="(chip, index) in railChips"
        :key="chip.name"
        class="cp-chip-wrap"
      >
        <button
          type="button"
          class="cp-chip cp-chip--switch"
          :class="{
            'cp-chip--active': chip.name === currentName,
            'cp-chip--busy': busyName === chip.name,
          }"
          :tabindex="index === switchFocusIdx ? 0 : -1"
          :disabled="disabled || chip.unavailable"
          :aria-pressed="chip.name === currentName"
          :title="chip.title"
          @click="onSwitchChipClick(index, chip.name)"
          @focus="switchFocusIdx = index"
        >
          <span
            class="cp-chip__dot"
            :class="{ 'cp-chip__dot--off': chip.name !== currentName }"
          />
          <span class="cp-chip__name">{{ chip.name }}</span>
          <span
            v-if="chip.number !== null"
            class="cp-chip__kbd"
          >{{ chip.number }}</span>
        </button>
        <button
          type="button"
          class="cp-chip__pin"
          :class="{ 'cp-chip__pin--on': chip.pinned }"
          tabindex="-1"
          :aria-label="chip.pinned
            ? t(`${i18nPrefix}.unpinProfile`, { name: chip.name })
            : t(`${i18nPrefix}.pinProfile`, { name: chip.name })"
          @click="quickSwitch.togglePin(chip.name)"
        >
          <SIcon
            :name="chip.pinned ? 'PinOff' : 'Pin'"
            size="w-3 h-3"
          />
        </button>
      </span>
      <button
        v-if="moreCount > 0"
        type="button"
        class="cp-chip cp-chip--more"
        tabindex="-1"
        @click="emit('more')"
      >
        <span class="cp-chip__name">{{ t(`${i18nPrefix}.quickRailMore`, { count: moreCount }) }}</span>
        <kbd class="cp-chip__kbd">{{ modifierText }}K</kbd>
      </button>
    </div>
    <div class="cp-rail__hint">
      {{ t(`${i18nPrefix}.quickRailModifierHint`, { modifier: modifierText }) }}
    </div>
  </div>

  <!-- 旧模式（缺省）：渲染与行为保持原样。
       TODO(profiles-redesign): 集成步骤删除 -->
  <div
    v-else-if="enabledProfiles.length > 0"
    class="cp-rail surface-workspace"
  >
    <div class="cp-rail__head">
      <SIcon
        name="Sparkles"
        size="w-3.5 h-3.5"
        class="cp-rail__head-icon"
      />
      {{ t(`${i18nPrefix}.quickSwitch`) }}
    </div>
    <div class="cp-rail__list">
      <button
        v-for="(profile, index) in enabledProfiles"
        :key="profile.name"
        type="button"
        class="cp-chip"
        :class="{
          'cp-chip--active': profile.name === currentName,
          'cp-chip--busy': busyName === profile.name,
        }"
        :disabled="disabled"
        :aria-pressed="profile.name === currentName"
        :title="profile.description || profile.name"
        @click="emit('apply', profile.name)"
      >
        <span
          class="cp-chip__dot"
          :class="{ 'cp-chip__dot--off': profile.name !== currentName }"
        />
        <span class="cp-chip__name">{{ profile.name }}</span>
        <span
          v-if="index < 9"
          class="cp-chip__kbd"
        >{{ index + 1 }}</span>
      </button>
    </div>
    <div class="cp-rail__hint">
      {{ t(`${i18nPrefix}.quickRailHint`) }}
    </div>
  </div>
</template>

<script setup lang="ts" generic="T extends QuickRailProfile">
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import { PROFILES_PIN_CAP, type ProfilesQuickSwitch } from '@/composables/useProfilesQuickSwitch'

/** QuickRail 直接读取的最小 profile 形状（两平台共有字段） */
export interface QuickRailProfile {
  name: string
  enabled?: boolean | null
  description?: string | null
}

interface Props {
  profiles: T[]
  currentName: string | null
  /** i18n key 前缀，例如 'claudeProfiles' / 'codex.profiles' */
  i18nPrefix: string
  disabled?: boolean
  busyName?: string | null
  /** 快速切换状态（useProfilesQuickSwitch 返回值）；注入后启用钉选/最近新模式 */
  quickSwitch?: ProfilesQuickSwitch | null
  /** 栏容量之外的可用 profile 数（>0 时渲染「+N more → ⌘K」入口） */
  moreCount?: number
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  busyName: null,
  quickSwitch: null,
  moreCount: 0,
})

const emit = defineEmits<{
  (e: 'apply', name: string): void
  (e: 'more'): void
}>()

const { t } = useI18n()

const enabledProfiles = computed(() =>
  props.profiles.filter(p => p.enabled !== false),
)

/* ========================================================================
 * quickSwitch 新模式：钉选编号 + recent 无编号 + roving tabindex
 * ======================================================================== */

interface SwitchChip {
  name: string
  pinned: boolean
  /** 钉选 chip 的序号（1..n）；recent chip 恒为 null（只展示不编号） */
  number: number | null
  /** 名称不在当前列表或已禁用：chip 置灰不可 Apply，但不自动移除 */
  unavailable: boolean
  title: string
}

const railChips = computed<SwitchChip[]>(() => {
  const quickSwitch = props.quickSwitch
  if (!quickSwitch) return []
  const byName = new Map(props.profiles.map(profile => [profile.name, profile]))
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
  const chips = quickSwitch.pinned.value.map((name, index) => toChip(name, true, index + 1))
  for (const name of quickSwitch.recentNotPinned.value) {
    if (chips.length >= PROFILES_PIN_CAP) break
    chips.push(toChip(name, false, null))
  }
  return chips
})

const modifierText = computed(() => props.quickSwitch?.modifier.value ?? '')

const switchListRef = ref<HTMLElement | null>(null)
const switchFocusIdx = ref(0)

watch(railChips, (chips) => {
  if (switchFocusIdx.value >= chips.length) {
    switchFocusIdx.value = Math.max(0, chips.length - 1)
  }
})

const focusSwitchChip = (index: number) => {
  switchFocusIdx.value = index
  void nextTick(() => {
    const chips = switchListRef.value?.querySelectorAll<HTMLElement>('.cp-chip--switch')
    chips?.[index]?.focus()
  })
}

const onSwitchKeydown = (event: KeyboardEvent) => {
  const count = railChips.value.length
  if (count === 0) return
  if (event.key === 'ArrowRight') {
    event.preventDefault()
    focusSwitchChip((switchFocusIdx.value + 1) % count)
  } else if (event.key === 'ArrowLeft') {
    event.preventDefault()
    focusSwitchChip((switchFocusIdx.value - 1 + count) % count)
  } else if (event.key === 'Home') {
    event.preventDefault()
    focusSwitchChip(0)
  } else if (event.key === 'End') {
    event.preventDefault()
    focusSwitchChip(count - 1)
  }
}

const onSwitchChipClick = (index: number, name: string) => {
  switchFocusIdx.value = index
  emit('apply', name)
}
</script>

<style scoped>
.cp-rail {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  margin-bottom: 14px;

  /* 背景/边框由 surface-workspace 工具类提供 */
  border-radius: 12px;
}

.cp-rail__head {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  color: var(--cp-ink-2);
  font-size: 12px;
  font-weight: 600;
}

.cp-rail__head-icon { color: var(--cp-accent); }

.cp-rail__list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.cp-rail__hint {
  flex-shrink: 0;
  color: var(--cp-ink-4);
  font-size: 10.5px;
  font-family: var(--cp-mono);
}

.cp-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  border-radius: 6px;
  border: 1px solid var(--cp-line-2);
  background: var(--cp-bg-2);
  color: var(--cp-ink-1);
  font-family: var(--cp-mono);
  font-size: 12px;
  cursor: pointer;
  transition: background 120ms ease, border-color 120ms ease, color 120ms ease;
}

.cp-chip:hover:not(:disabled, .cp-chip--active) {
  background: var(--cp-bg-3);
  color: var(--cp-ink-0);
}

.cp-chip:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.cp-chip--active {
  background: var(--cp-accent);
  border-color: var(--cp-accent);
  color: var(--cp-on-accent);
  font-weight: 600;
}

.cp-chip--busy { opacity: 0.7; }

/* quickSwitch 新模式：chip 外壳承载 pin 小操作 */
.cp-chip-wrap {
  position: relative;
  display: inline-flex;
}

.cp-chip--switch:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.cp-chip__pin {
  position: absolute;
  top: -6px;
  right: -6px;
  display: grid;
  place-items: center;
  width: 16px;
  height: 16px;
  padding: 0;
  border-radius: 999px;
  border: 1px solid var(--cp-line-2);
  background: var(--cp-bg-1);
  color: var(--cp-ink-3);
  cursor: pointer;
  opacity: 0;
  transition: opacity 120ms ease, color 120ms ease, border-color 120ms ease;
}

.cp-chip-wrap:hover .cp-chip__pin,
.cp-chip-wrap:focus-within .cp-chip__pin,
.cp-chip__pin--on {
  opacity: 1;
}

.cp-chip__pin--on {
  color: var(--cp-accent);
  border-color: var(--cp-accent-line);
}

.cp-chip__pin:hover {
  color: var(--cp-ink-0);
  border-color: var(--cp-accent-line);
}

.cp-chip--more {
  border-style: dashed;
  color: var(--cp-ink-2);
}

.cp-chip--more:hover:not(:disabled) {
  background: var(--cp-bg-3);
  color: var(--cp-ink-0);
}

.cp-chip__dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--cp-good);
  flex-shrink: 0;
}

.cp-chip__dot--off { background: var(--cp-ink-4); }

.cp-chip--active .cp-chip__dot { background: var(--cp-on-accent); }

.cp-chip__name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
}

.cp-chip__kbd {
  margin-left: 2px;
  padding: 0 4px;
  border-radius: 3px;
  background: rgb(0 0 0 / 25%);
  color: inherit;
  font-size: 10px;
  opacity: 0.65;
}

.cp-chip--active .cp-chip__kbd {
  background: rgb(0 0 0 / 18%);
  opacity: 0.8;
}

@media (prefers-reduced-motion: reduce) {
  .cp-chip, .cp-chip__pin { transition: none; }
}
</style>
