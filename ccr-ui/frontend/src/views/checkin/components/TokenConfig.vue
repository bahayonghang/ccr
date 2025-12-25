<template>
  <div class="token-config">
    <h2>Token 配置</h2>
    <p class="info-text">
      Token 配置已集成在账号管理中。请在账号管理中配置每个账号的 Cookies 和 API User。
    </p>
    <button
      class="go-button"
      @click="goToCheckinManage"
    >
      前往账号管理 →
    </button>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

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
