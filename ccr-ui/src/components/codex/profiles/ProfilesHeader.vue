<!-- Codex Profiles 页面顶部标题区：标题/副标题 + 命令/重载/导出/添加 -->
<template>
  <div class="cp-header">
    <div class="cp-header__intro">
      <div class="cp-header__icon">
        <SIcon
          name="Folder"
          size="w-5 h-5"
        />
      </div>
      <div class="cp-header__text">
        <h1 class="cp-header__title">
          {{ $t('codex.profiles.title') }}
        </h1>
        <p class="cp-header__subtitle">
          {{ $t('codex.profiles.subtitle') }}
        </p>
      </div>
    </div>

    <div class="cp-header__actions">
      <RouterLink
        to="/codex"
        class="cp-header__back"
      >
        <button
          type="button"
          class="cp-btn cp-btn--ghost"
        >
          <SIcon
            name="ArrowLeft"
            size="w-3.5 h-3.5"
          />
          <span>{{ $t('codex.profiles.backToCodex') }}</span>
        </button>
      </RouterLink>

      <button
        type="button"
        class="cp-btn cp-btn--ghost"
        :class="{ 'cp-btn--palette-open': paletteOpen }"
        :disabled="loading"
        :aria-pressed="paletteOpen"
        aria-haspopup="dialog"
        :title="$t('codex.profiles.commandPaletteShortcut')"
        @click="emit('openPalette')"
      >
        <SIcon
          name="Command"
          size="w-3.5 h-3.5"
        />
        <span>{{ $t('codex.profiles.commandPaletteButton') }}</span>
        <kbd class="cp-btn__kbd">⌘K</kbd>
      </button>

      <button
        type="button"
        class="cp-btn cp-btn--ghost"
        :disabled="loading"
        @click="emit('reload')"
      >
        <SIcon
          name="RefreshCw"
          size="w-3.5 h-3.5"
          :class="{ 'cp-spin': loading }"
        />
        <span>{{ $t('codex.profiles.reloadAction') }}</span>
      </button>

      <button
        type="button"
        class="cp-btn cp-btn--ghost"
        :disabled="exporting || loading"
        @click="emit('export')"
      >
        <SIcon
          name="Download"
          size="w-3.5 h-3.5"
        />
        <span>{{ $t('common.export') }}</span>
      </button>

      <button
        type="button"
        class="cp-btn cp-btn--primary"
        :disabled="loading"
        @click="emit('add')"
      >
        <SIcon
          name="Plus"
          size="w-3.5 h-3.5"
        />
        <span>{{ $t('codex.profiles.addProfile') }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { RouterLink } from 'vue-router'

interface Props {
  loading?: boolean
  exporting?: boolean
  paletteOpen?: boolean
}

withDefaults(defineProps<Props>(), {
  loading: false,
  exporting: false,
  paletteOpen: false,
})

const emit = defineEmits<{
  (e: 'add'): void
  (e: 'export'): void
  (e: 'reload'): void
  (e: 'openPalette'): void
}>()
</script>

<style scoped>
.cp-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 16px;
  padding: 4px 0 14px;
}

.cp-header__intro {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.cp-header__icon {
  display: grid;
  place-items: center;
  width: 38px;
  height: 38px;
  flex-shrink: 0;
  border-radius: 10px;
  background: var(--cp-accent-soft);
  border: 1px solid var(--cp-accent-line);
  color: var(--cp-accent);
}

.cp-header__text { min-width: 0; }

.cp-header__title {
  font-size: 18px;
  font-weight: 600;
  color: var(--cp-ink-0);
  letter-spacing: -0.2px;
  margin: 0;
  line-height: 1.25;
}

.cp-header__subtitle {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--cp-ink-3);
}

.cp-header__actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.cp-header__back { display: inline-flex; }

.cp-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 12px;
  border-radius: 7px;
  font-size: 12.5px;
  font-weight: 500;
  font-family: inherit;
  background: var(--cp-bg-2);
  border: 1px solid var(--cp-line-2);
  color: var(--cp-ink-1);
  cursor: pointer;
  transition: background 120ms ease, border-color 120ms ease, color 120ms ease;
}

.cp-btn:hover:not(:disabled) {
  background: var(--cp-bg-3);
  color: var(--cp-ink-0);
}

.cp-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.cp-btn--ghost {
  background: transparent;
  border-color: var(--cp-line);
  color: var(--cp-ink-2);
}

.cp-btn--palette-open {
  background: var(--cp-accent-soft);
  border-color: var(--cp-accent-line);
  color: var(--cp-accent);
  box-shadow: inset 0 0 0 1px var(--cp-accent-line);
}

.cp-btn--palette-open .cp-btn__kbd {
  border-color: var(--cp-accent-line);
  color: var(--cp-accent);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
}

.cp-btn--primary {
  background: var(--cp-accent);
  border-color: var(--cp-accent);
  color: var(--cp-on-accent);
  font-weight: 600;
}

.cp-btn--primary:hover:not(:disabled) {
  background: var(--color-accent-primary-hover);
  border-color: var(--color-accent-primary-hover);
}

.cp-btn__kbd {
  margin-left: 4px;
  padding: 1px 5px;
  font-family: var(--cp-mono);
  font-size: 10px;
  color: var(--cp-ink-4);
  background: var(--cp-bg-0);
  border: 1px solid var(--cp-line-2);
  border-radius: 3px;
}

.cp-spin { animation: cp-spin 1s linear infinite; }

@keyframes cp-spin { to { transform: rotate(360deg); } }

@media (prefers-reduced-motion: reduce) {
  .cp-btn { transition: none; }
  .cp-spin { animation: none; }
}
</style>
