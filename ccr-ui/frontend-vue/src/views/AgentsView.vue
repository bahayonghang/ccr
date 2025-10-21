<template>
  <Layout>
    <div class="mb-8">
      <h1 class="text-3xl font-bold mb-2" :style="{ color: 'var(--text-primary)' }">Agents</h1>
      <p class="text-base" :style="{ color: 'var(--text-secondary)' }">AI Agent 配置、工具绑定和模型管理</p>
    </div>

    <!-- Action buttons -->
    <div class="flex flex-wrap gap-3 mb-6">
      <Button variant="primary">
        <Plus class="w-4 h-4 mr-2" />
        添加 Agent
      </Button>
      <Button variant="secondary">
        <RefreshCw class="w-4 h-4 mr-2" />
        刷新
      </Button>
    </div>

    <!-- Agents table -->
    <Card>
      <Table 
        :columns="columns" 
        :data="agents" 
        key-field="name"
        :has-actions="true"
      >
        <template #disabled="{ value }">
          <span 
            class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium"
            :style="{
              background: value ? 'rgba(239, 68, 68, 0.1)' : 'rgba(16, 185, 129, 0.1)',
              color: value ? '#ef4444' : '#10b981',
              border: value ? '1px solid rgba(239, 68, 68, 0.3)' : '1px solid rgba(16, 185, 129, 0.3)'
            }"
          >
            {{ value ? '已禁用' : '启用' }}
          </span>
        </template>
        
        <template #folder="{ value }">
          <span :style="{ color: 'var(--text-muted)' }">{{ value || '根目录' }}</span>
        </template>
        
        <template #actions>
          <div class="flex space-x-2">
            <Button variant="ghost" size="sm">
              <Edit2 class="w-4 h-4" />
            </Button>
            <Button variant="ghost" size="sm">
              <Trash2 class="w-4 h-4" />
            </Button>
            <Button variant="ghost" size="sm">
              <Power class="w-4 h-4" />
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
import { Plus, RefreshCw, Edit2, Trash2, Power } from 'lucide-vue-next'

const columns = [
  { key: 'name', title: '名称' },
  { key: 'model', title: '模型' },
  { key: 'folder', title: '文件夹' },
  { key: 'disabled', title: '状态' },
  { key: 'actions', title: '操作' },
]

const agents = [
  { name: 'Code Assistant', model: 'gpt-4', folder: '', disabled: false },
  { name: 'Reviewer', model: 'claude-3', folder: '', disabled: true },
  { name: 'Debugger', model: 'gpt-3.5', folder: 'dev', disabled: false },
  { name: 'Documentation', model: 'llama-2', folder: 'docs', disabled: false },
]
</script>