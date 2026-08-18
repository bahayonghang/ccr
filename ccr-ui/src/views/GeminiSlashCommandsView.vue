<template>
  <PageShell class="gemini-slash-view">
    <template #header>
      <PageHeader
        :title="$t('gemini.slashCommands.pageTitle')"
        :description="$t('gemini.slashCommands.subtitle')"
      />
    </template>

    <template #subnav>
      <ModuleSubnav module="antigravity" />
    </template>

    <BaseSlashCommands
      :config="geminiConfig"
      hide-chrome
    />
  </PageShell>
</template>

<script setup lang="ts">
import { geminiConfig } from '@/configs/slashCommands'
import BaseSlashCommands from '@/components/BaseSlashCommands.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
</script>
