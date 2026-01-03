<template>
  <div class="token-config">
    <h2>{{ t('checkin.token_config.title') }}</h2>
    <p class="info-text">
      {{ t('checkin.token_config.integrated_hint') }}
    </p>
    <button
      class="go-button"
      @click="goToCheckinManage"
    >
      {{ t('checkin.token_config.go_account_manage_arrow') }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

const router = useRouter()
const { t } = useI18n()

const goToCheckinManage = () => {
  router.push({ name: 'checkin' })
}
</script>

<style scoped>
.token-config {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.token-config h2 {
  margin: 0;
  font-size: 1.25rem;
  color: #1f2937;
}

.info-text {
  font-size: 0.875rem;
  color: #6b7280;
  line-height: 1.5;
}

.go-button {
  align-self: flex-start;
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  border: 1px solid #d1d5db;
  background: white;
  cursor: pointer;
  font-size: 0.875rem;
}

.go-button:hover {
  background: #f3f4f6;
}
</style>
