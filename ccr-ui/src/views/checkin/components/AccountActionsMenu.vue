<template>
  <Teleport to="body">
    <div
      v-if="activeMenuAccount"
      class="checkin-accounts-tab__menu checkin-accounts-tab__menu--floating"
      :class="`checkin-accounts-tab__menu--${menuPosition.placement}`"
      :style="menuStyle"
      @click.stop
    >
      <button
        class="checkin-accounts-tab__menu-item checkin-accounts-tab__menu-item--top"
        @click="emit('refresh-balance', activeMenuAccount.id); close()"
      >
        {{ t('checkin.actions.refreshBalance') }}
      </button>
      <button
        class="checkin-accounts-tab__menu-item"
        @click="emit('edit', activeMenuAccount); close()"
      >
        {{ t('checkin.accounts.edit') }}
      </button>
      <button
        class="checkin-accounts-tab__menu-item checkin-accounts-tab__menu-item--danger"
        @click="emit('delete', activeMenuAccount.id); close()"
      >
        {{ t('checkin.accounts.delete') }}
      </button>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import type { AccountInfo } from '@/types/checkin'

const props = defineProps<{
  accounts: AccountInfo[]
}>()

const emit = defineEmits<{
  (e: 'refresh-balance', accountId: string): void
  (e: 'edit', account: AccountInfo): void
  (e: 'delete', accountId: string): void
}>()

const { t } = useI18n()

type AccountMenuPlacement = 'top' | 'bottom'

interface AccountMenuPosition {
  top: number
  left: number
  width: number
  maxHeight: number
  placement: AccountMenuPlacement
}

const ACCOUNT_MENU_WIDTH = 168
const ACCOUNT_MENU_ESTIMATED_HEIGHT = 144
const ACCOUNT_MENU_MARGIN = 12
const ACCOUNT_MENU_GAP = 8

const openAccountId = ref<string | null>(null)
const menuPosition = ref<AccountMenuPosition>({
  top: ACCOUNT_MENU_MARGIN,
  left: ACCOUNT_MENU_MARGIN,
  width: ACCOUNT_MENU_WIDTH,
  maxHeight: ACCOUNT_MENU_ESTIMATED_HEIGHT,
  placement: 'bottom',
})

const activeMenuAccount = computed(
  () => props.accounts.find((account) => account.id === openAccountId.value) || null
)

const menuStyle = computed(() => ({
  top: `${menuPosition.value.top}px`,
  left: `${menuPosition.value.left}px`,
  width: `${menuPosition.value.width}px`,
  maxHeight: `${menuPosition.value.maxHeight}px`,
}))

const close = () => {
  openAccountId.value = null
}

const updateMenuPosition = (trigger: HTMLElement) => {
  const rect = trigger.getBoundingClientRect()
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight
  const menuWidth = Math.min(ACCOUNT_MENU_WIDTH, viewportWidth - ACCOUNT_MENU_MARGIN * 2)
  const availableBelow = viewportHeight - rect.bottom - ACCOUNT_MENU_MARGIN
  const availableAbove = rect.top - ACCOUNT_MENU_MARGIN
  const placement: AccountMenuPlacement =
    availableBelow >= ACCOUNT_MENU_ESTIMATED_HEIGHT || availableBelow >= availableAbove
      ? 'bottom'
      : 'top'

  const left = Math.min(
    Math.max(ACCOUNT_MENU_MARGIN, rect.right - menuWidth),
    viewportWidth - menuWidth - ACCOUNT_MENU_MARGIN
  )

  const minimumVisibleHeight = 108
  const top =
    placement === 'bottom'
      ? Math.min(
          Math.max(ACCOUNT_MENU_MARGIN, rect.bottom + ACCOUNT_MENU_GAP),
          viewportHeight - ACCOUNT_MENU_MARGIN - minimumVisibleHeight
        )
      : Math.max(ACCOUNT_MENU_MARGIN, rect.top - ACCOUNT_MENU_ESTIMATED_HEIGHT - ACCOUNT_MENU_GAP)

  const maxHeight = Math.max(
    minimumVisibleHeight,
    placement === 'bottom'
      ? viewportHeight - top - ACCOUNT_MENU_MARGIN
      : rect.top - ACCOUNT_MENU_GAP - ACCOUNT_MENU_MARGIN
  )

  menuPosition.value = {
    top,
    left,
    width: menuWidth,
    maxHeight,
    placement,
  }
}

const toggle = (accountId: string, event: MouseEvent) => {
  if (openAccountId.value === accountId) {
    close()
  } else {
    const trigger = event.currentTarget
    if (!(trigger instanceof HTMLElement)) return
    updateMenuPosition(trigger)
    openAccountId.value = accountId
  }
}

// 点击页面其他地方关闭菜单
const closeMenuOnClickOutside = (e: MouseEvent) => {
  if (
    openAccountId.value &&
    !(e.target as HTMLElement).closest(
      '.checkin-accounts-tab__menu-wrap, .checkin-accounts-tab__menu--floating'
    )
  ) {
    close()
  }
}

onMounted(() => {
  document.addEventListener('click', closeMenuOnClickOutside)
  window.addEventListener('resize', close)
  window.addEventListener('scroll', close, true)
})

onUnmounted(() => {
  document.removeEventListener('click', closeMenuOnClickOutside)
  window.removeEventListener('resize', close)
  window.removeEventListener('scroll', close, true)
})

defineExpose({ openAccountId, toggle, close })
</script>

<style scoped>
.checkin-accounts-tab__menu {
  z-index: var(--z-popover);
  padding: 0.35rem;
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border-default);
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-lg);
  overflow-y: auto;
}

.checkin-accounts-tab__menu--floating {
  position: fixed;
  inset: auto auto auto 0;
}

.checkin-accounts-tab__menu--top {
  transform-origin: bottom right;
}

.checkin-accounts-tab__menu--bottom {
  transform-origin: top right;
}

.checkin-accounts-tab__menu-item {
  width: 100%;
  border-radius: var(--radius-md);
  padding: 0.625rem 0.75rem;
  text-align: left;
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-secondary);
  transition:
    background-color var(--duration-normal) var(--ease-out),
    color var(--duration-normal) var(--ease-out);
}

.checkin-accounts-tab__menu-item:hover {
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: var(--text-primary);
}

.checkin-accounts-tab__menu-item--top {
  margin-bottom: 0.15rem;
}

.checkin-accounts-tab__menu-item--danger {
  color: rgb(var(--color-danger-rgb) / 92%);
}

.checkin-accounts-tab__menu-item--danger:hover {
  background: rgb(var(--color-danger-rgb) / 14%);
  color: rgb(var(--color-danger-rgb) / 100%);
}
</style>
