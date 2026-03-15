<template>
  <div class="relative w-full">
    <!-- Language Switcher Button -->
    <button
      type="button"
      class="glass-surface flex w-full items-center justify-between gap-2 rounded-lg border border-border-default/60 px-3 py-2 text-sm font-semibold text-text-primary transition-[color,background-color,border-color,transform,box-shadow] hover:border-accent-primary/35 hover:bg-bg-elevated/80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/20"
      :aria-label="$t('common.language.switchLanguage')"
      :aria-expanded="showDropdown"
      aria-haspopup="listbox"
      :title="$t('common.language.switchLanguage')"
      @click="toggleDropdown"
    >
      <span class="flex min-w-0 items-center gap-2">
        <SIcon
          name="Languages"
          size="w-4 h-4"
        />
        <span class="text-left whitespace-normal break-words">
          {{ currentLanguageName }} / {{ targetLanguageName }}
        </span>
      </span>
      <SIcon
        name="ChevronDown"
        size="w-3 h-3"
        class="text-text-muted transition-transform"
        :class="{ 'rotate-180': showDropdown }"
      />
    </button>

    <!-- Dropdown Menu -->
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="transform scale-95 opacity-0"
      enter-to-class="transform scale-100 opacity-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="transform scale-100 opacity-100"
      leave-to-class="transform scale-95 opacity-0"
    >
      <div
        v-if="showDropdown"
        class="glass-surface absolute left-0 z-50 mt-2 w-44 overflow-hidden rounded-xl border border-border-default/70 shadow-2xl"
        role="listbox"
      >
        <button
          v-for="lang in languages"
          :key="lang.code"
          type="button"
          class="lang-option w-full px-4 py-3 text-left text-sm font-medium transition-colors flex items-center justify-between"
          :class="{
            'lang-active': currentLocale === lang.code,
            'lang-inactive': currentLocale !== lang.code
          }"
          role="option"
          :aria-selected="currentLocale === lang.code"
          @click="switchLanguage(lang.code)"
        >
          <span class="flex items-center gap-3">
            <span class="text-lg">{{ lang.flag }}</span>
            <span>{{ lang.name }}</span>
          </span>
          <SIcon
            v-if="currentLocale === lang.code"
            name="Check"
            size="w-4 h-4"
            class="text-text-inverted"
          />
        </button>
      </div>
    </Transition>

    <!-- Click outside to close -->
    <div
      v-if="showDropdown"
      class="fixed inset-0 z-40"
      @click="showDropdown = false"
    />
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { setLocale } from '@/i18n'
import { logger } from '@/utils/logger'

const { locale, t: _t } = useI18n({ useScope: 'global' })

const showDropdown = ref(false)

interface Language {
  code: string
  name: string
  flag: string
}

const languages: Language[] = [
  { code: 'zh-CN', name: '中文', flag: 'CN' },
  { code: 'en-US', name: 'English', flag: 'US' },
]

const currentLocale = computed(() => locale.value)

const currentLanguageName = computed(() => {
  const current = languages.find(lang => lang.code === currentLocale.value)
  return current ? current.name : languages[0].name
})

const targetLanguageName = computed(() => {
  const target = languages.find(lang => lang.code !== currentLocale.value)
  return target ? target.name : languages[0].name
})

const toggleDropdown = () => {
  showDropdown.value = !showDropdown.value
}

const switchLanguage = async (langCode: string) => {
  try {
    await setLocale(langCode)
    locale.value = langCode
  } catch (error) {
    logger.warn('[LanguageSwitcher] failed to switch locale', error)
  }

  showDropdown.value = false
}
</script>

<style scoped>
.lang-option {
  position: relative;
}

.lang-active {
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: var(--color-accent-primary);
  font-weight: var(--font-semibold);
  box-shadow: var(--shadow-glow-primary);
}

.lang-inactive {
  @apply text-text-secondary hover:bg-bg-surface/80 hover:text-text-primary;
}
</style>
