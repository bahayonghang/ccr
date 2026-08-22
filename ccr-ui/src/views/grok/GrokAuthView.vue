<template>
  <PageShell class="grok-auth-view">
    <template #header>
      <PageHeader
        :title="t('grok.auth.title')"
        :eyebrow="t('grok.dashboard.header.eyebrow')"
        :description="t('grok.auth.subtitle')"
      >
        <template #actions>
          <Button
            variant="secondary"
            :disabled="loading"
            @click="refresh"
          >
            {{ t('common.refresh') }}
          </Button>
        </template>
      </PageHeader>
    </template>

    <template #subnav>
      <ModuleSubnav module="grok" />
    </template>

    <section
      v-if="localOnly"
      class="grok-auth-view__banner"
    >
      <strong>{{ t('grok.dashboard.localOnly.title') }}</strong>
      <p>{{ t('grok.dashboard.localOnly.description') }}</p>
    </section>

    <section
      v-else
      class="grok-auth-view__panel"
      data-testid="grok-auth-session"
    >
      <p class="grok-auth-view__label">
        {{ t('grok.auth.sessionFile') }}
      </p>
      <p
        class="grok-auth-view__value"
        data-testid="grok-auth-status"
      >
        {{ loggedIn ? t('grok.auth.signedIn') : t('grok.auth.signedOut') }}
      </p>
      <div
        v-if="canAuthOff"
        data-testid="grok-auth-off"
      >
        <Button
          :disabled="loading"
          @click="handleAuthOff"
        >
          {{ t('auth.off') }}
        </Button>
      </div>
    </section>
  </PageShell>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import Button from '@/components/ui/Button.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import { grokAuthCurrent, grokAuthOff } from '@/api/domains/grok'
import { logger } from '@/utils/logger'
import { extractErrorMessage } from '@/utils/errorHandler'
import { useUIStore } from '@/stores/ui'

defineOptions({ name: 'GrokAuthView' })

const { t } = useI18n()
const uiStore = useUIStore()
const loading = ref(false)
const loggedIn = ref(false)
const canAuthOff = ref(false)
const localOnly = ref(false)

const refresh = async () => {
  loading.value = true
  try {
    const response = await grokAuthCurrent()
    if (response.status === 'unsupported_environment') {
      localOnly.value = true
      canAuthOff.value = false
      return
    }
    localOnly.value = false
    loggedIn.value = response.logged_in
    canAuthOff.value = response.can_auth_off
  } catch (error) {
    logger.error('Failed to load Grok auth status:', error)
    uiStore.showError(extractErrorMessage(error) || t('auth.offFailed'))
  } finally {
    loading.value = false
  }
}

const handleAuthOff = async () => {
  const confirmed = await uiStore.requestConfirm({
    title: t('auth.confirmOffTitle'),
    message: t('auth.confirmOffGrok'),
    confirmText: t('auth.off'),
    cancelText: t('common.cancel'),
    type: 'danger',
  })
  if (!confirmed) return

  try {
    loading.value = true
    const result = await grokAuthOff()
    if (result.status === 'unsupported_environment') {
      localOnly.value = true
      return
    }
    if (result.changed) {
      uiStore.showSuccess(t('auth.offSuccess'))
    } else {
      uiStore.showSuccess(t('auth.offUnchanged'))
    }
    await refresh()
  } catch (error) {
    logger.error('Failed to log out Grok official session:', error)
    uiStore.showError(extractErrorMessage(error) || t('auth.offFailed'))
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void refresh()
})
</script>

<style scoped>
.grok-auth-view__panel,
.grok-auth-view__banner {
  border: 1px solid var(--stage-border-soft);
  border-radius: 1rem;
  background: var(--stage-surface-elevated);
  padding: 1rem 1.125rem;
  display: grid;
  gap: 0.75rem;
}

.grok-auth-view__label {
  color: var(--stage-text-quiet);
  font-size: 0.75rem;
  font-weight: 600;
}

.grok-auth-view__value {
  color: var(--stage-text-primary);
  font-size: 1.125rem;
  font-weight: 700;
}
</style>
