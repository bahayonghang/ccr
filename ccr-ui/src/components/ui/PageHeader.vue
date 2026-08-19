<template>
  <header class="page-header">
    <div
      v-if="$slots.leading"
      class="page-header__leading"
    >
      <slot name="leading" />
    </div>

    <div class="page-header__main">
      <p
        v-if="eyebrow"
        class="page-header__eyebrow"
        :lang="resolvedEyebrowLang"
      >
        {{ eyebrow }}
      </p>
      <h1 class="page-header__title">
        {{ title }}
      </h1>
      <p
        v-if="description"
        class="page-header__description"
      >
        {{ description }}
      </p>
    </div>

    <div
      v-if="$slots.status || $slots.actions"
      class="page-header__aside"
    >
      <div
        v-if="$slots.status"
        class="page-header__status"
      >
        <slot name="status" />
      </div>
      <div
        v-if="$slots.actions"
        class="page-header__actions"
      >
        <slot name="actions" />
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  title: string
  eyebrow?: string
  description?: string
  /** 拉丁短标签应设为 en；中文 eyebrow 不要设，以便继承 :lang(zh)。 */
  eyebrowLang?: string
}

const props = withDefaults(defineProps<Props>(), {
  eyebrow: undefined,
  description: undefined,
  eyebrowLang: undefined,
})

const isLatinEyebrow = (value: string) => /^[\x20-\x7E\s]+$/.test(value)

const resolvedEyebrowLang = computed(() => {
  if (props.eyebrowLang) return props.eyebrowLang
  if (props.eyebrow && isLatinEyebrow(props.eyebrow)) return 'en'
  return undefined
})
</script>

<style scoped>
.page-header {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 1rem;
}

.page-header__leading {
  flex-shrink: 0;
}

.page-header__main {
  flex: 1 1 16rem;
  min-width: 0;
}

.page-header__eyebrow {
  margin: 0 0 0.25rem;
  font-size: 0.75rem;
  font-weight: 600;
  line-height: 1.24;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-text-muted);
}

.page-header__eyebrow:lang(zh),
.page-header__eyebrow:lang(zh-CN) {
  letter-spacing: 0;
  text-transform: none;
}

.page-header__title {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
  line-height: 1.2;
  letter-spacing: -0.01em;
  color: var(--color-text-primary);
}

.page-header__description {
  margin: 0.5rem 0 0;
  max-width: 48rem;
  font-size: 1rem;
  font-weight: 400;
  line-height: 1.56;
  letter-spacing: 0;
  color: var(--color-text-secondary);
}

.page-header__aside {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  gap: 0.75rem;
  margin-left: auto;
}

.page-header__status,
.page-header__actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}

@media (width >= 1024px) {
  .page-header {
    flex-wrap: nowrap;
  }
}
</style>
