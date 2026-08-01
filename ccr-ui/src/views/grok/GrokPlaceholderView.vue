<template>
  <div class="grok-placeholder stage-page">
    <div class="grok-placeholder__shell">
      <ModuleSubnav module="grok" />

      <section class="grok-placeholder__content">
        <div class="grok-placeholder__icon">
          <SIcon
            :name="section === 'profiles' ? 'Folders' : 'SlidersHorizontal'"
            size="w-6 h-6"
          />
        </div>
        <p class="grok-placeholder__eyebrow">
          {{ t('grok.placeholder.eyebrow') }}
        </p>
        <h1>{{ title }}</h1>
        <p class="grok-placeholder__description">
          {{ t('grok.placeholder.description') }}
        </p>
        <RouterLink to="/grok">
          <Button variant="secondary">
            <SIcon
              name="ArrowLeft"
              size="w-4 h-4"
              class="mr-2"
            />
            {{ t('grok.placeholder.back') }}
          </Button>
        </RouterLink>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'

const props = defineProps<{
  section: 'profiles' | 'settings'
}>()

const { t } = useI18n()
const title = computed(() => t(
  props.section === 'profiles'
    ? 'grok.placeholder.profilesTitle'
    : 'grok.placeholder.settingsTitle',
))
</script>

<style scoped>
.grok-placeholder {
  min-height: 100%;
  padding: 1.5rem;
}

.grok-placeholder__shell {
  width: min(100%, 80rem);
  margin: 0 auto;
}

.grok-placeholder__content {
  display: flex;
  min-height: 26rem;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  margin-top: 1rem;
  padding: 2rem;
  border-top: 1px solid var(--stage-border-soft);
}

.grok-placeholder__icon {
  display: flex;
  width: 3rem;
  height: 3rem;
  align-items: center;
  justify-content: center;
  margin-bottom: 1.25rem;
  color: var(--color-platform-grok);
  background: rgb(var(--color-platform-grok-rgb) / 12%);
  border: 1px solid rgb(var(--color-platform-grok-rgb) / 24%);
  border-radius: var(--radius-lg);
}

.grok-placeholder__eyebrow {
  margin: 0;
  color: var(--color-platform-grok);
  font-size: 0.8125rem;
  font-weight: 600;
  letter-spacing: 0;
}

.grok-placeholder h1 {
  margin: 0.5rem 0 0;
  color: var(--stage-text-primary);
  font-family: var(--font-brand);
  font-size: 2rem;
  font-weight: 600;
  letter-spacing: 0;
}

.grok-placeholder__description {
  max-width: 34rem;
  margin: 0.75rem 0 1.5rem;
  color: var(--stage-text-secondary);
  font-size: 0.9375rem;
  line-height: 1.6;
}

@media (width <= 640px) {
  .grok-placeholder {
    padding: 1rem;
  }

  .grok-placeholder__content {
    min-height: 22rem;
    padding: 1.25rem 0.5rem;
  }
}
</style>
