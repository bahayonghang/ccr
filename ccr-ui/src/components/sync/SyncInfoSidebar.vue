<template>
  <div class="space-y-6">
    <!-- 账号卡片 -->
    <div class="glass-card p-6 transition-[transform,box-shadow] duration-300 hover:scale-[1.01]">
      <div class="mb-5 flex items-center justify-between gap-3">
        <div class="flex items-center gap-3">
          <div
            class="rounded-2xl p-3"
            :style="{ background: 'rgb(var(--color-accent-primary-rgb) / 10%)' }"
          >
            <SIcon
              name="Cloud"
              size="w-6 h-6"
              :style="{ color: 'var(--accent-primary)' }"
            />
          </div>
          <h2
            class="text-xl font-bold"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ $t('sync.webdav.title') }}
          </h2>
        </div>

        <!-- 连接状态 chip（仅已配置时显示） -->
        <span
          v-if="props.syncStatus?.configured"
          class="connection-chip"
          :class="`chip--${chipState}`"
        >
          <SIcon
            :name="chipIcon"
            size="w-3.5 h-3.5"
          />
          {{ chipText }}
        </span>
      </div>

      <!-- 已配置：账号详情 + 操作按钮组 -->
      <div
        v-if="props.syncStatus?.configured"
        class="space-y-4"
      >
        <div class="account-details">
          <div class="detail-row">
            <div class="detail-label">
              {{ $t('sync.webdav.server') }}
            </div>
            <div class="detail-value">
              {{ props.syncStatus.webdav_url }}
            </div>
          </div>
          <div class="detail-row">
            <div class="detail-label">
              {{ $t('sync.webdav.username') }}
            </div>
            <div class="detail-value">
              {{ props.syncStatus.username }}
            </div>
          </div>
          <div class="detail-row">
            <div class="detail-label">
              {{ $t('sync.webdav.remotePath') }}
            </div>
            <div class="detail-value">
              {{ props.syncStatus.remote_path }}
            </div>
          </div>
        </div>

        <div class="action-row">
          <Button
            variant="primary"
            size="sm"
            @click="openDialog('edit')"
          >
            <template #leading>
              <SIcon
                name="Edit"
                size="w-4 h-4"
              />
            </template>
            {{ $t('sync.account.editBtn') }}
          </Button>
          <Button
            variant="secondary"
            size="sm"
            :loading="testing"
            @click="onTestExisting"
          >
            {{ $t('sync.account.testBtn') }}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            @click="confirmingDisconnect = true"
          >
            <template #leading>
              <SIcon
                name="Trash2"
                size="w-4 h-4"
              />
            </template>
            {{ $t('sync.account.disconnectBtn') }}
          </Button>
        </div>
      </div>

      <!-- 未配置：CTA + 折叠 CLI 提示 -->
      <div
        v-else
        class="space-y-4"
      >
        <div class="empty-banner">
          <SIcon
            name="AlertCircle"
            size="w-5 h-5"
          />
          <div class="empty-text">
            <strong>{{ $t('sync.webdav.notConfigured') }}</strong>
            <span class="empty-hint">{{ $t('sync.account.emptyHint') }}</span>
          </div>
        </div>
        <Button
          variant="primary"
          size="lg"
          block
          @click="openDialog('add')"
        >
          <template #leading>
            <SIcon
              name="Plus"
              size="w-5 h-5"
            />
          </template>
          {{ $t('sync.account.addCta') }}
        </Button>
        <details class="cli-fallback">
          <summary>{{ $t('sync.account.cliHint') }}</summary>
          <code class="cli-command">{{ $t('sync.webdav.configureCommand') }}</code>
        </details>
      </div>
    </div>

    <!-- 功能说明 -->
    <div class="glass-card p-6 transition-[transform,box-shadow] duration-300 hover:scale-[1.01]">
      <div class="mb-6 flex items-center gap-3">
        <div
          class="rounded-2xl p-3"
          :style="{ background: 'rgb(var(--color-accent-tertiary-rgb) / 10%)' }"
        >
          <SIcon
            name="BookOpen"
            size="w-6 h-6"
            :style="{ color: 'var(--accent-tertiary)' }"
          />
        </div>
        <h2
          class="text-xl font-bold"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ $t('sync.features.title') }}
        </h2>
      </div>

      <div
        class="space-y-4 text-sm"
        :style="{ color: 'var(--text-secondary)' }"
      >
        <div>
          <h4
            class="mb-2 font-bold"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ $t('sync.features.presetPlatform') }}
          </h4>
          <p>{{ $t('sync.features.presetPlatformDesc') }}</p>
        </div>
        <div>
          <h4
            class="mb-2 font-bold"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ $t('sync.features.independentManagement') }}
          </h4>
          <p>{{ $t('sync.features.independentManagementDesc') }}</p>
        </div>
        <div>
          <h4
            class="mb-2 font-bold"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ $t('sync.features.smartFiltering') }}
          </h4>
          <p>{{ $t('sync.features.smartFilteringDesc') }}</p>
        </div>
        <div>
          <h4
            class="mb-2 font-bold"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ $t('sync.features.sensitiveMasking') }}
          </h4>
          <p>{{ $t('sync.features.sensitiveMaskingDesc') }}</p>
        </div>
      </div>
    </div>

    <!-- 支持服务 -->
    <div class="glass-card p-6 transition-[transform,box-shadow] duration-300 hover:scale-[1.01]">
      <div class="mb-6 flex items-center gap-3">
        <div
          class="rounded-2xl p-3"
          :style="{ background: 'rgb(var(--color-success-rgb) / 10%)' }"
        >
          <SIcon
            name="Server"
            size="w-6 h-6"
            :style="{ color: 'var(--accent-success)' }"
          />
        </div>
        <h2
          class="text-xl font-bold"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ $t('sync.supportedServices.title') }}
        </h2>
      </div>

      <div
        class="space-y-3 text-sm"
        :style="{ color: 'var(--text-secondary)' }"
      >
        <div
          v-for="service in serviceItems"
          :key="service"
          class="flex items-center gap-2"
        >
          <SIcon
            name="CheckCircle"
            size="w-4 h-4"
            :style="{ color: 'var(--accent-success)' }"
          />
          <span>{{ service }}</span>
        </div>
      </div>
    </div>

    <!-- 账号弹窗 -->
    <SyncAccountDialog
      v-model="dialogOpen"
      :mode="dialogMode"
      :initial="props.syncStatus"
      @saved="onSaved"
    />

    <!-- 断开二次确认 -->
    <BaseModal
      v-model="confirmingDisconnect"
      :title="$t('sync.account.disconnectConfirmTitle')"
      size="sm"
      surface="glass"
      :close-on-backdrop="!disconnecting"
      :close-on-escape="!disconnecting"
    >
      <p class="text-sm text-text-secondary">
        {{ $t('sync.account.disconnectConfirmBody') }}
      </p>
      <template #footer>
        <Button
          variant="ghost"
          :disabled="disconnecting"
          @click="confirmingDisconnect = false"
        >
          {{ $t('sync.account.cancelBtn') }}
        </Button>
        <Button
          variant="danger"
          :loading="disconnecting"
          @click="onDisconnect"
        >
          {{ $t('sync.account.disconnectConfirmBtn') }}
        </Button>
      </template>
    </BaseModal>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import SyncAccountDialog from '@/components/sync/SyncAccountDialog.vue'
import { clearWebdavConfig } from '@/api'
import type { SyncStatusView } from '@/types/syncSelection'
import { logger } from '@/utils/logger'

interface Props {
  syncStatus: SyncStatusView | null
}

const props = defineProps<Props>()
const emit = defineEmits<{
  'status-refresh': []
}>()

const { t } = useI18n()

const dialogOpen = ref(false)
const dialogMode = ref<'add' | 'edit'>('add')
const confirmingDisconnect = ref(false)
const disconnecting = ref(false)
const testing = ref(false)

const openDialog = (mode: 'add' | 'edit') => {
  dialogMode.value = mode
  dialogOpen.value = true
}

const onSaved = () => {
  emit('status-refresh')
}

const onTestExisting = () => {
  testing.value = true
  emit('status-refresh')
  // 父组件 fetchSyncStatus 内部已调 test_connection，syncStatus 更新后 chip 自动重渲
  setTimeout(() => {
    testing.value = false
  }, 600)
}

const onDisconnect = async () => {
  disconnecting.value = true
  try {
    await clearWebdavConfig()
    confirmingDisconnect.value = false
    emit('status-refresh')
  } catch (err) {
    logger.error('clear_webdav_config failed:', err)
  } finally {
    disconnecting.value = false
  }
}

const serviceItems = computed(() => [
  t('sync.supportedServices.nutstore'),
  t('sync.supportedServices.nextcloud'),
  t('sync.supportedServices.owncloud'),
  t('sync.supportedServices.any'),
])

const chipState = computed<'ok' | 'fail' | 'unknown'>(() => {
  const accessible = props.syncStatus?.remote_accessible
  if (accessible === true) return 'ok'
  if (accessible === false) return 'fail'
  return 'unknown'
})

const chipIcon = computed(() => {
  switch (chipState.value) {
    case 'ok': return 'CheckCircle'
    case 'fail': return 'AlertCircle'
    default: return 'Cloud'
  }
})

const chipText = computed(() => {
  switch (chipState.value) {
    case 'ok': return t('sync.webdav.connected')
    case 'fail': return t('sync.webdav.unreachable')
    default: return t('sync.webdav.untested')
  }
})
</script>

<style scoped>
.connection-chip {
  @apply inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-medium;

  border: 1px solid transparent;
}

.connection-chip.chip--ok {
  border-color: rgb(var(--color-success-rgb) / 28%);
  background: rgb(var(--color-success-rgb) / 12%);
  color: var(--accent-success);
}

.connection-chip.chip--fail {
  border-color: rgb(var(--color-danger-rgb) / 28%);
  background: rgb(var(--color-danger-rgb) / 12%);
  color: var(--accent-danger);
}

.connection-chip.chip--unknown {
  border-color: rgb(var(--color-border-default-rgb) / 40%);
  background: rgb(var(--color-bg-elevated-rgb) / 50%);
  color: var(--text-muted);
}

.account-details {
  @apply space-y-3;
}

.detail-row {
  @apply flex flex-col gap-1;
}

.detail-label {
  @apply text-xs;

  color: var(--text-muted);
}

.detail-value {
  @apply break-all font-mono text-sm;

  color: var(--text-primary);
}

.action-row {
  @apply flex flex-wrap items-center gap-2 pt-2;
}

.empty-banner {
  @apply flex items-start gap-3 rounded-xl px-4 py-3;

  border: 1px solid rgb(var(--color-warning-rgb) / 28%);
  background: rgb(var(--color-warning-rgb) / 10%);
  color: var(--accent-warning);
}

.empty-text {
  @apply flex flex-col gap-0.5 text-sm;
}

.empty-text strong {
  color: var(--text-primary);
}

.empty-hint {
  @apply text-xs;

  color: var(--text-secondary);
}

.cli-fallback {
  @apply rounded-xl px-4 py-2 text-xs;

  border: 1px solid rgb(var(--color-border-default-rgb) / 40%);
  background: rgb(var(--color-bg-elevated-rgb) / 50%);
  color: var(--text-muted);
}

.cli-fallback summary {
  @apply cursor-pointer select-none py-1 font-medium;

  color: var(--text-secondary);
}

.cli-command {
  @apply mt-2 block rounded-lg px-3 py-2 font-mono text-xs;

  background: rgb(var(--color-bg-elevated-rgb) / 70%);
  color: var(--text-primary);
}
</style>
