<template>
  <div class="skills-redirect">
    <p>Redirecting to marketplace…</p>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

onMounted(() => {
  void router.replace({ path: '/skills', query: { tab: 'marketplace' } })
})
</script>

<style scoped>
.skills-redirect {
  @apply flex min-h-[40vh] items-center justify-center text-sm text-white/60;
}
</style>
