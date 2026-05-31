<!-- 4 列统计条：当前 profile / 配置总数 / 认证分布 / 最近写入 -->
<template>
  <div class="cp-stats">
    <div class="cp-stat surface-status">
      <div class="cp-stat__head">
        <span
          class="cp-stat__dot"
          :class="{ 'cp-stat__dot--good': Boolean(current) }"
        />
        {{ $t('claudeProfiles.currentProfile') }}
      </div>
      <div class="cp-stat__value cp-stat__value--mono">
        {{ current || $t('claudeProfiles.notSet') }}
      </div>
      <div class="cp-stat__hint">
        {{ $t('claudeProfiles.statStrip.profileSubtitle') }}
      </div>
    </div>

    <div class="cp-stat surface-status">
      <div class="cp-stat__head">
        <SIcon
          name="Folder"
          size="w-3 h-3"
        />
        {{ $t('claudeProfiles.totalCount') }}
      </div>
      <div class="cp-stat__value cp-stat__value--mono">
        {{ total }}
      </div>
      <div class="cp-stat__hint">
        {{ $t('claudeProfiles.statStrip.totalHint', { enabled, disabled: total - enabled }) }}
      </div>
    </div>

    <div class="cp-stat surface-status">
      <div class="cp-stat__head">
        <SIcon
          name="ShieldCheck"
          size="w-3 h-3"
        />
        {{ $t('claudeProfiles.statStrip.authTitle') }}
      </div>
      <div class="cp-stat__value cp-stat__value--mono">
        {{ subscriptionCount }} · {{ apiKeyCount }}
      </div>
      <div class="cp-stat__hint">
        {{ $t('claudeProfiles.statStrip.authSplit', { subscription: subscriptionCount, apiKey: apiKeyCount }) }}
      </div>
    </div>

    <div class="cp-stat surface-status">
      <div class="cp-stat__head">
        <SIcon
          name="RefreshCw"
          size="w-3 h-3"
        />
        {{ $t('claudeProfiles.statStrip.lastWrite') }}
      </div>
      <div class="cp-stat__value cp-stat__value--mono">
        {{ lastWrite || '—' }}
      </div>
      <div class="cp-stat__hint">
        {{ $t('claudeProfiles.statStrip.lastWriteHint') }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'

interface Props {
  current: string | null
  total: number
  enabled: number
  subscriptionCount: number
  apiKeyCount: number
  lastWrite?: string | null
}

withDefaults(defineProps<Props>(), {
  lastWrite: null,
})
</script>

<style scoped>
.cp-stats {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  margin-bottom: 14px;
}

.cp-stat {
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 13px 16px;

  /* 背景/边框由 surface-status 工具类提供 */
  border-radius: 10px;
}

.cp-stat__head {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--cp-ink-3);
  font-size: 10.5px;
  letter-spacing: 0.8px;
  text-transform: uppercase;
}

.cp-stat__dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: var(--cp-ink-4);
}

.cp-stat__dot--good { background: var(--cp-good); }

.cp-stat__value {
  color: var(--cp-ink-0);
  font-size: 19px;
  font-weight: 600;
  letter-spacing: -0.3px;
  word-break: break-all;
}

.cp-stat__value--mono { font-family: var(--cp-mono); }

.cp-stat__hint {
  color: var(--cp-ink-3);
  font-size: 11px;
}

@media (width <= 1024px) {
  .cp-stats { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}
</style>
