<template>
  <div
    class="claude-background"
    aria-hidden="true"
  >
    <div class="claude-background__base" />
    <div class="claude-background__halo claude-background__halo--left" />
    <div class="claude-background__halo claude-background__halo--right" />
    <div class="claude-background__grain" />
  </div>
</template>

<style scoped>
.claude-background {
  position: fixed;
  inset: 0;
  overflow: hidden;
  pointer-events: none;
  z-index: -50;
  background: var(--color-bg-base);
}

.claude-background__base,
.claude-background__halo,
.claude-background__grain {
  position: absolute;
  inset: 0;
}

.claude-background__base {
  background:
    radial-gradient(circle at 14% 0%, rgb(var(--color-accent-primary-rgb) / 6%) 0%, transparent 30%),
    radial-gradient(circle at 78% 14%, rgb(var(--color-accent-secondary-rgb) / 6%) 0%, transparent 22%),
    linear-gradient(180deg, rgb(var(--color-premium-pink-rgb) / 40%), transparent 28%),
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 100%), rgb(var(--color-bg-base-rgb) / 95%));
}

.claude-background__halo {
  filter: blur(88px);
  opacity: 0.68;
}

.claude-background__halo--left {
  inset: -8% auto auto -10%;
  width: 44vw;
  height: 44vw;
  border-radius: 50%;
  background: rgb(var(--color-accent-primary-rgb) / 10%);
}

.claude-background__halo--right {
  inset: auto -10% -14% auto;
  width: 38vw;
  height: 38vw;
  border-radius: 50%;
  background: rgb(var(--color-premium-blue-rgb) / 52%);
}

.claude-background__grain {
  opacity: 0.032;
  background-image:
    radial-gradient(rgb(var(--color-text-primary-rgb) / 24%) 0.8px, transparent 0.8px);
  background-size: 16px 16px;
  mask-image: linear-gradient(180deg, rgb(0 0 0 / 68%), transparent);
}

[data-theme='dark'] .claude-background__base {
  background:
    radial-gradient(circle at 16% 4%, rgb(var(--color-accent-primary-rgb) / 12%) 0%, transparent 30%),
    radial-gradient(circle at 82% 10%, rgb(var(--color-accent-secondary-rgb) / 10%) 0%, transparent 28%),
    linear-gradient(180deg, rgb(var(--color-premium-pink-rgb) / 26%), transparent 28%),
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 100%), rgb(var(--color-bg-base-rgb) / 93%));
}

[data-theme='dark'] .claude-background__halo--left {
  background: rgb(var(--color-accent-primary-rgb) / 14%);
}

[data-theme='dark'] .claude-background__halo--right {
  background: rgb(var(--color-premium-blue-rgb) / 72%);
}

[data-theme='dark'] .claude-background__grain {
  opacity: 0.046;
}

@media (prefers-reduced-motion: reduce) {
  .claude-background__halo {
    display: none;
  }
}
</style>
