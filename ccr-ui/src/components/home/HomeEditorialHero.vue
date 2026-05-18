<template>
  <section
    class="home-hero"
    data-home-hero
  >
    <p class="home-hero__eyebrow">
      <span class="home-hero__eyebrow-anchor" />
      {{ t('home.workbenchEyebrow') }}
    </p>
    <h1 class="home-hero__title">
      {{ t('home.workbenchTitle') }}
    </h1>
    <p class="home-hero__description">
      {{ t('home.workbenchDescription') }}
    </p>
    <div
      class="home-hero__divider"
      aria-hidden="true"
    >
      <span class="home-hero__divider-anchor" />
    </div>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<style scoped>
.home-hero {
  display: grid;
  gap: 0.35rem;
  padding: 1rem 0 0.5rem;
  color: var(--color-text-primary);
}

.home-hero__eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 0.55rem;
  margin: 0;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.home-hero__eyebrow-anchor {
  display: inline-block;
  width: 16px;
  height: 2px;
  border-radius: 2px;
  background: var(--color-accent-primary);
}

.home-hero__title {
  margin: 0;
  color: var(--color-text-primary);
  font-family: var(--font-brand);
  font-size: var(--home-text-display);
  font-weight: 620;
  letter-spacing: var(--home-tracking-display);
  line-height: var(--home-leading-display);
  max-width: 36rem;
}

.home-hero__description {
  max-width: 52rem;
  margin: 0.2rem 0 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-body);
  letter-spacing: var(--home-tracking-body);
  line-height: var(--home-leading-body);
}

.home-hero__divider {
  position: relative;
  margin: 0.65rem 0 0 var(--home-divider-inset);
  height: 1px;
  background: var(--home-border-hairline);
}

.home-hero__divider-anchor {
  position: absolute;
  top: -1px;
  left: 0;
  width: var(--home-divider-anchor-width);
  height: 2px;
  border-radius: 2px;
  background: linear-gradient(
    90deg,
    var(--color-accent-primary) 0%,
    rgb(var(--color-accent-primary-rgb) / 24%) 100%
  );
}

@media (width <= 720px) {
  .home-hero__divider {
    margin-left: 0;
  }
}
</style>
