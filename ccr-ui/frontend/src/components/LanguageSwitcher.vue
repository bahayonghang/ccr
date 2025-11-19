<template>
  <div class="relative">
    <!-- Language Switcher Button -->
    <button
      class="px-3 py-2 rounded-lg font-semibold text-sm transition-all flex items-center space-x-1.5 hover:scale-105"
      :style="{
        background: 'var(--bg-tertiary)',
        color: 'var(--text-primary)',
        border: '1px solid var(--border-color)'
      }"
      :aria-label="$t('language.switchLanguage')"
      :title="$t('language.switchLanguage')"
      @click="toggleDropdown"
    >
      <Languages class="w-4 h-4" />
      <span class="hidden sm:inline">{{ currentLanguageName }}</span>
      <ChevronDown
        class="w-3 h-3 transition-transform"
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
        class="absolute right-0 mt-2 w-40 rounded-lg overflow-hidden glass-effect z-50"
        :style="{
          border: '1px solid var(--border-color)',
          boxShadow: 'var(--shadow-large)'
        }"
      >
        <button
          v-for="lang in languages"
          :key="lang.code"
          class="w-full px-4 py-2.5 text-left text-sm font-medium transition-all flex items-center justify-between"
          :class="{
            'bg-gradient-to-r from-accent-primary/10 to-accent-secondary/10': currentLocale === lang.code
          }"
          :style="{
            color: currentLocale === lang.code ? 'var(--accent-primary)' : 'var(--text-secondary)',
            backgroundColor: currentLocale === lang.code ? '' : 'transparent'
          }"
          @click="switchLanguage(lang.code)"
          @mouseenter="hoveredLang = lang.code"
          @mouseleave="hoveredLang = null"
        >
          <span class="flex items-center space-x-2">
            <span class="text-lg">{{ lang.flag }}</span>
            <span>{{ lang.name }}</span>
          </span>
          <Check
            v-if="currentLocale === lang.code"
            class="w-4 h-4"
            :style="{ color: 'var(--accent-primary)' }"
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
const hoveredLang = ref<string | null>(null)

interface Language {
  code: string
  name: string
  flag: string
}

const languages: Language[] = [
  { code: 'zh-CN', name: '中文', flag: '🇨🇳' },
  { code: 'en-US', name: 'English', flag: '🇺🇸' },
]

const currentLocale = computed(() => locale.value)

const currentLanguageName = computed(() => {
  const current = languages.find(lang => lang.code === currentLocale.value)
  return current ? current.name : languages[0].name
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
.glass-effect {
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  background: var(--bg-secondary);
}
</style>
