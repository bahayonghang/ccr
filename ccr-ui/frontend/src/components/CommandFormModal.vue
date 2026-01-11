<template>
  <div
    v-if="visible"
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
  >
    <div
      ref="modalRef"
      role="dialog"
      aria-modal="true"
      :aria-labelledby="titleId"
      class="rounded-2xl max-w-md w-full mx-4 max-h-[90vh] overflow-y-auto"
      :style="{
        background: 'var(--bg-secondary)',
        border: '1px solid var(--border-color)'
      }"
    >
      <!-- 模态框标题 -->
      <div
        class="flex items-center justify-between p-6"
        :style="{ borderBottom: '1px solid var(--border-color)' }"
      >
        <h3
          :id="titleId"
          class="text-lg font-semibold"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ isEditing ? $t('common.edit') : $t('common.add') }}
        </h3>
        <button
          class="p-1 rounded-lg transition-colors hover:opacity-80"
          :style="{ color: 'var(--text-muted)' }"
          :aria-label="$t('common.close')"
          @click="close"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- 表单内容 -->
      <form
        class="p-6"
        @submit.prevent="handleSubmit"
      >
        <div class="space-y-4">
          <!-- 名称 -->
          <div>
            <label
              class="block text-sm font-medium mb-1"
              :style="{ color: 'var(--text-primary)' }"
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
                border: '1px solid var(--border-color)',
                background: 'var(--bg-primary)',
                color: 'var(--text-primary)',
                '--tw-ring-color': 'var(--accent-primary)'
              }"
              :placeholder="$t('slashCommands.namePlaceholder')"
            >
          </div>

          <!-- 命令 -->
          <div>
            <label
              class="block text-sm font-medium mb-1"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('common.command') }}
            </label>
            <input
              v-model="form.command"
              type="text"
              required
              class="w-full px-3 py-2 rounded-lg text-sm focus:outline-none focus:ring-2"
              :style="{
                border: '1px solid var(--border-color)',
                background: 'var(--bg-primary)',
                color: 'var(--text-primary)',
                '--tw-ring-color': 'var(--accent-primary)'
              }"
              :placeholder="$t('slashCommands.commandPlaceholder')"
            >
          </div>

          <!-- 描述 -->
          <div>
            <label
              class="block text-sm font-medium mb-1"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('common.description') }}
            </label>
            <textarea
              v-model="form.description"
              rows="3"
              required
              class="w-full px-3 py-2 rounded-lg text-sm focus:outline-none focus:ring-2 resize-y min-h-[80px]"
              :style="{
                border: '1px solid var(--border-color)',
                background: 'var(--bg-primary)',
                color: 'var(--text-primary)',
                '--tw-ring-color': 'var(--accent-primary)'
              }"
              :placeholder="$t('slashCommands.descriptionPlaceholder')"
            />
          </div>

          <!-- 文件夹 -->
          <div>
            <label
              class="block text-sm font-medium mb-1"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('common.folder') }}
            </label>
            <select
              v-model="form.folder"
              required
              class="w-full px-3 py-2 rounded-lg text-sm focus:outline-none focus:ring-2"
              :style="{
                border: '1px solid var(--border-color)',
                background: 'var(--bg-primary)',
                color: 'var(--text-primary)',
                '--tw-ring-color': 'var(--accent-primary)'
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
              background: 'var(--bg-secondary)',
              color: 'var(--text-secondary)',
              border: '1px solid var(--border-color)'
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
              background: 'var(--accent-primary)',
              color: '#fff',
              opacity: loading ? 0.7 : 1,
              cursor: loading ? 'not-allowed' : 'pointer'
            }"
          >
            <RefreshCw
              v-if="loading"
              class="w-4 h-4 animate-spin mr-2"
            />
            {{ isEditing ? $t('common.update') : $t('common.create') }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { X, RefreshCw } from 'lucide-vue-next'
import { useFocusTrap, useEscapeKey, useUniqueId } from '@/composables/useAccessibility'

interface SlashCommand {
  name: string
  command: string
  description: string
  folder: string
  enabled: boolean
}

interface SlashCommandRequest {
  name: string
  command: string
  description: string
  folder: string
}

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

// Accessibility enhancements
const titleId = useUniqueId('command-form-title')
const modalRef = ref<HTMLElement | null>(null)
const visibleRef = ref(props.visible)

watch(() => props.visible, (newValue) => {
  visibleRef.value = newValue
})

// Close handler
const close = () => {
  emit('update:visible', false)
  emit('update:editingCommand', null)
  resetForm()
}

const { focusFirstElement } = useFocusTrap(modalRef, visibleRef)
useEscapeKey(close, visibleRef)

watch(visibleRef, (isOpen) => {
  if (isOpen) {
    setTimeout(() => focusFirstElement(), 100)
  }
})

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
