<template>
  <div
    class="apple-background"
    aria-hidden="true"
  >
    <div class="apple-background__base" />
    <div class="apple-background__halo apple-background__halo--left" />
    <div class="apple-background__halo apple-background__halo--right" />
    <div class="apple-background__grain" />
  </div>
</template>

<style scoped>
.apple-background {
  position: fixed;
  inset: 0;
  overflow: hidden;
  pointer-events: none;
  z-index: -50;
  background: var(--color-bg-base);
}

.apple-background__base,
.apple-background__halo,
.apple-background__grain {
  position: absolute;
  inset: 0;
}

.apple-background__base {
  background:
    radial-gradient(circle at top left, rgb(var(--color-accent-primary-rgb) / 6%) 0%, transparent 28%),
    radial-gradient(circle at 78% 12%, rgb(var(--color-accent-secondary-rgb) / 5%) 0%, transparent 24%),
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 100%), rgb(var(--color-bg-base-rgb) / 94%));
}

.apple-background__halo {
  filter: blur(88px);
  opacity: 0.75;
}

.apple-background__halo--left {
  inset: -8% auto auto -10%;
  width: 44vw;
  height: 44vw;
  border-radius: 50%;
  background: rgb(var(--color-accent-primary-rgb) / 8%);
}

.apple-background__halo--right {
  inset: auto -10% -14% auto;
  width: 38vw;
  height: 38vw;
  border-radius: 50%;
  background: rgb(var(--color-premium-blue-rgb) / 42%);
}

.apple-background__grain {
  opacity: 0.028;
  background-image:
    radial-gradient(rgb(var(--color-text-primary-rgb) / 26%) 0.7px, transparent 0.7px);
  background-size: 18px 18px;
  mask-image: linear-gradient(180deg, rgb(0 0 0 / 68%), transparent);
}

[data-theme='dark'] .apple-background__base {
  background:
    radial-gradient(circle at top left, rgb(var(--color-accent-primary-rgb) / 11%) 0%, transparent 30%),
    radial-gradient(circle at 82% 10%, rgb(var(--color-accent-secondary-rgb) / 9%) 0%, transparent 28%),
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 100%), rgb(var(--color-bg-base-rgb) / 92%));
}

[data-theme='dark'] .apple-background__halo--left {
  background: rgb(var(--color-accent-primary-rgb) / 14%);
}

[data-theme='dark'] .apple-background__halo--right {
  background: rgb(var(--color-premium-blue-rgb) / 66%);
}

[data-theme='dark'] .apple-background__grain {
  opacity: 0.04;
}

@media (prefers-reduced-motion: reduce) {
  .apple-background__halo {
    display: none;
  }
}
</style>
