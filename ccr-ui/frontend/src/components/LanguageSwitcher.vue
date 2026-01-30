<template>
  <div class="relative w-full">
    <!-- Language Switcher Button -->
    <button
      class="w-full px-3 py-2 rounded-lg font-semibold text-sm transition-all flex items-center justify-between gap-2 hover:scale-[1.02] bg-bg-surface border border-border-subtle hover:border-accent-primary/50"
      :aria-label="$t('common.language.switchLanguage')"
      :title="$t('common.language.switchLanguage')"
      @click="toggleDropdown"
    >
      <span class="flex items-center gap-2 min-w-0 text-text-primary">
        <Languages class="w-4 h-4" />
        <span class="text-left whitespace-normal break-words">
          {{ currentLanguageName }} / {{ targetLanguageName }}
        </span>
      </span>
      <ChevronDown
        class="w-3 h-3 transition-transform text-text-muted"
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
        class="absolute left-0 mt-2 w-44 rounded-xl overflow-hidden z-50 bg-bg-elevated border border-border-subtle shadow-2xl"
      >
        <button
          v-for="lang in languages"
          :key="lang.code"
          class="lang-option w-full px-4 py-3 text-left text-sm font-medium transition-all flex items-center justify-between"
          :class="{
            'lang-active': currentLocale === lang.code,
            'lang-inactive': currentLocale !== lang.code
          }"
          @click="switchLanguage(lang.code)"
        >
          <span class="flex items-center gap-3">
            <span class="text-lg">{{ lang.flag }}</span>
            <span>{{ lang.name }}</span>
          </span>
          <Check
            v-if="currentLocale === lang.code"
            class="w-4 h-4 text-white"
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
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Languages, ChevronDown, Check } from 'lucide-vue-next'

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

const switchLanguage = (langCode: string) => {
  locale.value = langCode

  // Save to localStorage
  try {
    localStorage.setItem('ccr-ui-locale', langCode)
  } catch (error) {
    console.warn('Failed to save locale preference:', error)
  }

  showDropdown.value = false
}
</script>

<style scoped>
.lang-option {
  position: relative;
}

.lang-active {
  @apply bg-accent-primary text-white font-bold;
}

.lang-inactive {
  @apply text-text-secondary hover:bg-bg-surface hover:text-text-primary;
}
</style>

