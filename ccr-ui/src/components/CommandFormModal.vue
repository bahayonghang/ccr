<template>
  <BaseModal
    :model-value="visible"
    size="md"
    scrollable
    surface="solid"
    :title="isEditing ? $t('common.edit') : $t('common.add')"
    @close="close"
  >
    <!-- 模态框标题 -->
    <template #header="{ titleId }">
      <h3
        :id="titleId"
        class="text-lg font-semibold text-text-primary"
      >
        {{ isEditing ? $t('common.edit') : $t('common.add') }}
      </h3>
    </template>

    <!-- 表单内容 -->
    <form @submit.prevent="handleSubmit">
      <div class="space-y-4">
        <!-- 名称 -->
        <div>
          <label
            class="block text-sm font-medium mb-1"
            :style="{ color: 'var(--color-text-primary)' }"
          >
            {{ $t('common.name') }}
          </label>
          <input
            v-model="form.name"
            type="text"
            required
            :disabled="isEditing"
            class="w-full px-3 py-2 rounded-lg text-sm focus:outline-none focus:ring-2"
            :style="{
              border: '1px solid var(--color-border-default)',
              background: 'var(--color-bg-surface)',
              color: 'var(--color-text-primary)',
              '--tw-ring-color': 'var(--color-accent-primary)'
            }"
            :placeholder="$t('slashCommands.namePlaceholder')"
          >
        </div>

        <!-- 命令 -->
        <div>
          <label
            class="block text-sm font-medium mb-1"
            :style="{ color: 'var(--color-text-primary)' }"
          >
            {{ $t('common.command') }}
          </label>
          <input
            v-model="form.command"
            type="text"
            required
            class="w-full px-3 py-2 rounded-lg text-sm focus:outline-none focus:ring-2"
            :style="{
              border: '1px solid var(--color-border-default)',
              background: 'var(--color-bg-surface)',
              color: 'var(--color-text-primary)',
              '--tw-ring-color': 'var(--color-accent-primary)'
            }"
            :placeholder="$t('slashCommands.commandPlaceholder')"
          >
        </div>

        <!-- 描述 -->
        <div>
          <label
            class="block text-sm font-medium mb-1"
            :style="{ color: 'var(--color-text-primary)' }"
          >
            {{ $t('common.description') }}
          </label>
          <textarea
            v-model="form.description"
            rows="3"
            required
            class="w-full px-3 py-2 rounded-lg text-sm focus:outline-none focus:ring-2 resize-y min-h-[80px]"
            :style="{
              border: '1px solid var(--color-border-default)',
              background: 'var(--color-bg-surface)',
              color: 'var(--color-text-primary)',
              '--tw-ring-color': 'var(--color-accent-primary)'
            }"
            :placeholder="$t('slashCommands.descriptionPlaceholder')"
          />
        </div>

        <!-- 文件夹 -->
        <div>
          <label
            class="block text-sm font-medium mb-1"
            :style="{ color: 'var(--color-text-primary)' }"
          >
            {{ $t('common.folder') }}
          </label>
          <select
            v-model="form.folder"
            required
            class="w-full px-3 py-2 rounded-lg text-sm focus:outline-none focus:ring-2"
            :style="{
              border: '1px solid var(--color-border-default)',
              background: 'var(--color-bg-surface)',
              color: 'var(--color-text-primary)',
              '--tw-ring-color': 'var(--color-accent-primary)'
            }"
          >
            <option
              value=""
              disabled
            >
              {{ $t('slashCommands.selectFolder') }}
            </option>
            <option
              v-for="folder in folders"
              :key="folder"
              :value="folder"
            >
              {{ folder }}
            </option>
          </select>
        </div>
      </div>

      <!-- 表单按钮 -->
      <div class="flex justify-end gap-3 mt-6">
        <button
          type="button"
          class="px-4 py-2 rounded-lg transition-colors hover:opacity-80"
          :style="{
            background: 'var(--color-bg-surface)',
            color: 'var(--color-text-secondary)',
            border: '1px solid var(--color-border-default)'
          }"
          @click="close"
        >
          {{ $t('common.cancel') }}
        </button>
        <button
          type="submit"
          :disabled="loading"
          class="px-4 py-2 rounded-lg inline-flex items-center transition-colors hover:opacity-90"
          :style="{
            background: 'var(--color-accent-primary)',
            color: '#fff',
            opacity: loading ? 0.7 : 1,
            cursor: loading ? 'not-allowed' : 'pointer'
          }"
        >
          <SIcon
            v-if="loading"
            name="RefreshCw"
            size="w-4 h-4"
            class="animate-spin mr-2"
          />
          {{ isEditing ? $t('common.update') : $t('common.create') }}
        </button>
      </div>
    </form>
  </BaseModal>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, watch } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import type { SlashCommand, SlashCommandRequest } from '@/types/platform'

// Props
interface Props {
  visible: boolean
  editingCommand: SlashCommand | null
  folders: string[]
}

const props = defineProps<Props>()

// Emits
interface Emits {
  (e: 'update:visible', value: boolean): void
  (e: 'update:editingCommand', value: SlashCommand | null): void
  (e: 'submit', data: SlashCommandRequest): void
}

const emit = defineEmits<Emits>()

// 状态
const loading = ref(false)
const form = ref<SlashCommandRequest>({
  name: '',
  command: '',
  description: '',
  folder: ''
})

// 计算属性
const isEditing = computed(() => !!props.editingCommand)

// Close handler
const close = () => {
  emit('update:visible', false)
  emit('update:editingCommand', null)
  resetForm()
}

// 方法
const resetForm = () => {
  form.value = {
    name: '',
    command: '',
    description: '',
    folder: ''
  }
}

const handleSubmit = async () => {
  loading.value = true
  try {
    emit('submit', { ...form.value })
    close()
  } finally {
    loading.value = false
  }
}

// 监听编辑命令变化
watch(
  () => props.editingCommand,
  (cmd) => {
    if (cmd) {
      form.value = {
        name: cmd.name,
        command: cmd.command,
        description: cmd.description,
        folder: cmd.folder
      }
    } else {
      resetForm()
    }
  },
  { immediate: true }
)
</script>
