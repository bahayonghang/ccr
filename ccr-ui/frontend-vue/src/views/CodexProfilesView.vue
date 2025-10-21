<template>
  <Layout>
    <div class="mb-8">
      <h1 class="text-3xl font-bold mb-2" :style="{ color: 'var(--text-primary)' }">Codex Profiles</h1>
      <p class="text-base" :style="{ color: 'var(--text-secondary)' }">Codex Profile 配置和管理</p>
    </div>

    <!-- Action buttons -->
    <div class="flex flex-wrap gap-3 mb-6">
      <Button variant="primary">
        <Plus class="w-4 h-4 mr-2" />
        添加 Profile
      </Button>
      <Button variant="secondary">
        <RefreshCw class="w-4 h-4 mr-2" />
        刷新
      </Button>
    </div>

    <!-- Codex Profiles table -->
    <Card>
      <Table 
        :columns="columns" 
        :data="profiles" 
        key-field="name"
        :has-actions="true"
      >
        <template #model="{ value }">
          <span :style="{ color: 'var(--text-primary)' }">{{ value || '未设置' }}</span>
        </template>
        
        <template #actions>
          <div class="flex space-x-2">
            <Button variant="ghost" size="sm">
              <Edit2 class="w-4 h-4" />
            </Button>
            <Button variant="ghost" size="sm">
              <Trash2 class="w-4 h-4" />
            </Button>
          </div>
        </template>
      </Table>
    </Card>
  </Layout>
</template>

<script setup lang="ts">
import Layout from '@/components/Layout.vue'
import Card from '@/components/Card.vue'
import Button from '@/components/Button.vue'
import Table from '@/components/Table.vue'
import { Plus, RefreshCw, Edit2, Trash2 } from 'lucide-vue-next'

const columns = [
  { key: 'name', title: '名称' },
  { key: 'model', title: '模型' },
  { key: 'approval_policy', title: '审批策略' },
  { key: 'actions', title: '操作' },
]

const profiles = [
  { name: 'default', model: 'claude-3-opus', approval_policy: 'auto' },
  { name: 'dev', model: 'claude-3-sonnet', approval_policy: 'manual' },
  { name: 'prod', model: 'claude-3-opus', approval_policy: 'strict' },
]
</script>