<template>
  <PageShell>
    <template #header>
      <PageHeader
        :title="$t('slashCommands.title')"
        :description="$t('slashCommands.subtitle')"
      />
    </template>
    <BaseSlashCommands :config="claudeCodeConfig" />
  </PageShell>
</template>

<script setup lang="ts">
import { claudeCodeConfig } from '@/configs/slashCommands'
import BaseSlashCommands from '@/components/BaseSlashCommands.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
</script>
