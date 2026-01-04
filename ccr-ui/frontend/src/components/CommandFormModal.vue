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
      :class="modalClass"
      :style="modalStyle"
    >
      <!-- 模态框标题 -->
      <div
        :class="headerClass"
        :style="headerStyle"
      >
        <h3
          :id="titleId"
          :class="titleClass"
          :style="titleStyle"
        >
          {{ isEditing ? $t('common.edit') : $t('common.add') }}
        </h3>
        <button
          :class="closeButtonClass"
          :style="closeButtonStyle"
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
              :class="labelClass"
              :style="labelStyle"
            >
              {{ $t('common.name') }}
            </label>
            <input
              v-model="form.name"
              type="text"
              required
              :disabled="isEditing"
              :class="inputClass"
              :style="inputStyle"
              :placeholder="$t('slashCommands.namePlaceholder')"
            >
          </div>

          <!-- 命令 -->
          <div>
            <label
              :class="labelClass"
              :style="labelStyle"
            >
              {{ $t('common.command') }}
            </label>
            <input
              v-model="form.command"
              type="text"
              required
              :class="inputClass"
              :style="inputStyle"
              :placeholder="$t('slashCommands.commandPlaceholder')"
            >
          </div>

          <!-- 描述 -->
          <div>
            <label
              :class="labelClass"
              :style="labelStyle"
            >
              {{ $t('common.description') }}
            </label>
            <textarea
              v-model="form.description"
              rows="3"
              required
              :class="textareaClass"
              :style="textareaStyle"
              :placeholder="$t('slashCommands.descriptionPlaceholder')"
            />
          </div>

          <!-- 文件夹 -->
          <div>
            <label
              :class="labelClass"
              :style="labelStyle"
            >
              {{ $t('common.folder') }}
            </label>
            <select
              v-model="form.folder"
              required
              :class="selectClass"
              :style="selectStyle"
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
        <div
          :class="footerClass"
          :style="footerStyle"
        >
          <button
            type="button"
            :class="cancelButtonClass"
            :style="cancelButtonStyle"
            @click="close"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            type="submit"
            :disabled="loading"
            :class="submitButtonClass"
            :style="submitButtonStyle"
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
  theme: 'claude-code' | 'css-variable'
  themeColors: {
    bg: string
    bgSecondary: string
    bgTertiary: string
    primary: string
    secondary: string
    accent: string
    accentBg: string
  }
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

const modalClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'glass-effect rounded-2xl max-w-md w-full mx-4 max-h-[90vh] overflow-y-auto'
  }
  return 'rounded-lg max-w-md w-full mx-4 max-h-[90vh] overflow-y-auto'
})

const modalStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      background: 'var(--bg-secondary)',
      border: '1px solid var(--border-color)'
    }
  }
})

const headerClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'flex items-center justify-between p-6 border-b border-white/20'
  }
  return 'flex items-center justify-between p-6 border-b'
})

const headerStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      borderBottom: '1px solid var(--border-color)'
    }
  }
})

const titleClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'text-lg font-semibold text-guofeng-text-primary'
  }
  return 'text-lg font-semibold'
})

const titleStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      color: 'var(--text-primary)'
    }
  }
})

const closeButtonClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'p-1 rounded-lg text-guofeng-text-muted hover:text-guofeng-text-primary hover:bg-guofeng-bg-tertiary transition-colors'
  }
  return 'p-1 rounded-lg'
})

const closeButtonStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      color: 'var(--text-muted)',
      transition: 'all 0.2s'
    }
  }
})

const labelClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'block text-sm font-medium text-guofeng-text-primary mb-1'
  }
  return 'block text-sm font-medium mb-1'
})

const labelStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      color: 'var(--text-primary)'
    }
  }
})

const inputClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'w-full px-3 py-2 rounded-lg border border-guofeng-border bg-guofeng-bg-primary text-guofeng-text-primary placeholder-guofeng-text-muted focus:outline-none focus:ring-2 focus:ring-guofeng-amber focus:border-transparent'
  }
  return 'w-full px-3 py-2'
})

const inputStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      borderRadius: '6px',
      border: '1px solid var(--border-color)',
      background: 'var(--bg-primary)',
      color: 'var(--text-primary)',
      fontSize: '14px'
    }
  }
})

const textareaClass = computed(() => {
  return inputClass.value
})

const textareaStyle = computed(() => {
  const style = inputStyle.value
  return {
    ...style,
    resize: 'vertical' as const,
    minHeight: '80px'
  }
})

const selectClass = computed(() => {
  return inputClass.value
})

const selectStyle = computed(() => {
  return inputStyle.value
})

const footerClass = computed(() => {
  return 'flex justify-end gap-3 mt-6'
})

const footerStyle = computed(() => {
  return {}
})

const cancelButtonClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'px-4 py-2 rounded-lg border border-guofeng-border bg-guofeng-bg-secondary text-guofeng-text-secondary hover:bg-guofeng-bg-tertiary hover:text-guofeng-text-primary transition-colors'
  }
  return 'px-4 py-2 rounded-lg border'
})

const cancelButtonStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      background: 'var(--bg-secondary)',
      color: 'var(--text-secondary)',
      border: '1px solid var(--border-color)'
    }
  }
})

const submitButtonClass = computed(() => {
  if (props.theme === 'claude-code') {
    return 'px-4 py-2 rounded-lg bg-guofeng-amber text-white hover:bg-guofeng-amber/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed inline-flex items-center'
  }
  return 'px-4 py-2 rounded-lg inline-flex items-center'
})

const submitButtonStyle = computed(() => {
  if (props.theme === 'claude-code') {
    return {}
  } else {
    return {
      background: 'var(--accent-primary)',
      color: '#fff',
      opacity: loading.value ? 0.7 : 1,
      cursor: loading.value ? 'not-allowed' : 'pointer'
    }
  }
})

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