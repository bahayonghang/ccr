<template>
  <div class="checkin-manage-view">
    <div class="top-nav">
      <button
        class="back-button"
        @click="goBack"
      >
        <span class="text-lg">←</span>
        {{ t('checkin.backToConfigs') }}
      </button>
      <h1 class="page-title">
        {{ t('checkin.title') }}
      </h1>
    </div>

    <div class="dashboard-content">
      <div class="stats-section">
        <CheckinStats />
      </div>

      <div class="main-section">
        <div class="section-card">
          <AccountManager />
        </div>

        <div class="section-card">
          <TokenConfig />
        </div>

        <div class="section-card full-width">
          <CheckinHistory />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import CheckinStats from './components/CheckinStats.vue'
import AccountManager from './components/AccountManager.vue'
import TokenConfig from './components/TokenConfig.vue'
import CheckinHistory from './components/CheckinHistory.vue'

const router = useRouter()
const { t } = useI18n()

const goBack = () => {
  router.push({ name: 'configs' })
}
</script>

<style scoped>
.checkin-manage-view {
  min-height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 2rem;
}

.top-nav {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 2rem;
}

.back-button {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 0.5rem;
  color: white;
  cursor: pointer;
  transition: all 0.3s ease;
}

.back-button:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: translateX(-2px);
}

.page-title {
  font-size: 1.875rem;
  font-weight: 700;
  color: white;
  margin: 0;
}

.dashboard-content {
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

.stats-section {
  width: 100%;
}

.main-section {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 1.5rem;
}

.section-card {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  border-radius: 1rem;
  padding: 1.5rem;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

.section-card.full-width {
  grid-column: 1 / -1;
}

@media (prefers-color-scheme: dark) {
  .section-card {
    background: rgba(30, 30, 40, 0.95);
  }
}

@media (max-width: 768px) {
  .checkin-manage-view {
    padding: 1rem;
  }

  .page-title {
    font-size: 1.5rem;
  }

  .main-section {
    grid-template-columns: 1fr;
  }
}
</style>
