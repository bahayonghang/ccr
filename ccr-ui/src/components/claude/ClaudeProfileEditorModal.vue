<!-- Claude Profile 编辑器模态：从 ClaudeCodeProfilesView 内联块抽取，
     props/emit 架构对齐 CodexProfileEditorModal。
     视图保留数据加载与 buildRequest；模态负责段导航 scroll-spy、保存前校验汇总与外壳皮肤。
     样式统一消费共享编辑器基底（pe-*），不再维护平行的 --editor-* 令牌体系。 -->
<template>
  <BaseModal
    :model-value="modelValue"
    :persistent="saving"
    :show-close="false"
    size="5xl"
    content-class="pe-modal rounded-2xl"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <template #header="{ titleId }">
      <div class="pe-modal__head">
        <div class="flex min-w-0 items-start gap-4">
          <div class="pe-panel-icon flex h-12 w-12 shrink-0 items-center justify-center">
            <SIcon
              name="Layers"
              size="w-6 h-6"
            />
          </div>
          <div class="min-w-0">
            <p class="pe-modal__eyebrow">
              {{ eyebrow }}
            </p>
            <div class="mt-1.5 flex flex-wrap items-center gap-2">
              <h2
                :id="titleId"
                class="pe-modal__title"
              >
                {{ title }}
              </h2>
              <span
                class="pe-pill"
                :class="statusPillClass"
              >{{ statusLabel }}</span>
            </div>
            <p class="pe-modal__desc mt-1.5 max-w-3xl">
              {{ description }}
            </p>
          </div>
        </div>

        <button
          type="button"
          class="pe-icon-btn shrink-0"
          :aria-label="t('claudeProfiles.closeModal')"
          :disabled="saving"
          @click="emit('close')"
        >
          <SIcon
            name="X"
            size="w-4 h-4"
          />
        </button>
      </div>
    </template>

    <div class="pe-shell max-h-[calc(90vh-9rem)] overflow-hidden">
      <div
        v-if="showValidation && validationErrors.length > 0"
        class="pe-summary"
        role="alert"
      >
        <SIcon
          name="AlertTriangle"
          size="w-4 h-4"
        />
        <span>{{ validationErrors[0].message }}</span>
        <button
          type="button"
          class="pe-summary__jump"
          @click="scrollToSection(validationErrors[0].section)"
        >
          {{ t('claudeProfiles.validationJump') }}
        </button>
      </div>

      <div
        class="pe-nav"
        role="tablist"
        :aria-label="t('claudeProfiles.editorSectionsTitle')"
      >
        <button
          v-for="section in sectionItems"
          :key="section.id"
          type="button"
          role="tab"
          class="pe-nav__item"
          :class="{ 'pe-nav__item--active': activeSectionId === section.id }"
          :aria-selected="activeSectionId === section.id"
          @click="scrollToSection(section.id)"
        >
          {{ section.title }}
        </button>
      </div>

      <div
        ref="scrollRef"
        class="pe-scroll"
      >
        <ProviderTemplateSelector
          class="mb-4"
          platform="claude"
          :selected-template-id="selectedProviderTemplate"
          :selected-endpoint="selectedProviderEndpoint"
          :draft-context="templateDraft"
          :label="t('claudeProfiles.providerTemplateLabel')"
          :helper="t('claudeProfiles.providerTemplateHelper')"
          @select="emit('select-template', $event)"
          @manual="emit('manual-template')"
        />

        <ClaudeProfileEditorSections
          :editing-name="editingName"
          :form="form"
          :is-editing="isEditing"
          :monospace-field-class="MONOSPACE_FIELD_CLASS"
          :parsed-form-tags="parsedFormTags"
          :register-modal-section-ref="registerSectionRef"
          :save-error="saveError"
          :textarea-class="TEXTAREA_CLASS"
          :text-field-class="TEXT_FIELD_CLASS"
          :update-form-field="updateFormField"
        />
      </div>

      <div class="pe-footer">
        <p class="mr-auto text-xs text-text-secondary">
          {{ t('claudeProfiles.modalFooterHint') }}
        </p>
        <button
          type="button"
          class="pe-btn"
          :disabled="saving"
          @click="emit('close')"
        >
          {{ t('claudeProfiles.cancel') }}
        </button>
        <button
          type="button"
          class="pe-btn pe-btn--primary"
          :disabled="saving"
          @click="handleSaveClick"
        >
          <SIcon
            v-if="saving"
            name="RefreshCw"
            size="w-3.5 h-3.5"
            class="animate-spin"
          />
          {{ isEditing ? t('claudeProfiles.save') : t('claudeProfiles.create') }}
        </button>
      </div>
    </div>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import ClaudeProfileEditorSections from '@/components/claude/ClaudeProfileEditorSections.vue'
import ProviderTemplateSelector from '@/components/provider-templates/ProviderTemplateSelector.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import type {
  ClaudeProfileEditorForm,
  ClaudeProfileEditorSectionItem,
  ClaudeProfileFormSectionId,
} from '@/types/claudeProfileEditor'
import { CLAUDE_PROFILE_FORM_SECTION_IDS } from '@/types/claudeProfileEditor'
import '@/components/profiles/profile-editor-shell.css'

interface Props {
  modelValue: boolean
  form: ClaudeProfileEditorForm
  updateFormField: (field: keyof ClaudeProfileEditorForm, value: string | boolean) => void
  parsedFormTags: string[]
  isEditing: boolean
  /** 正在编辑的是当前激活 profile（头部状态 pill 用） */
  isEditingCurrent: boolean
  editingName: string
  saving: boolean
  saveError: string | null
  selectedProviderTemplate: string | null
  selectedProviderEndpoint: string
  templateDraft: ProviderTemplateDraftContext
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  close: []
  save: []
  'select-template': [selection: ProviderTemplateSelection]
  'manual-template': []
}>()

const { t } = useI18n()

// 表单控件皮肤：共享编辑器基底类，随模态一起迁移出视图
const TEXT_FIELD_CLASS = 'pe-input'
const MONOSPACE_FIELD_CLASS = 'pe-input pe-input--mono'
const TEXTAREA_CLASS = 'pe-textarea'

/* ========================================================================
 * 头部文案
 * ======================================================================== */

const eyebrow = computed(() =>
  props.isEditing ? t('claudeProfiles.modalEditEyebrow') : t('claudeProfiles.modalNewEyebrow'),
)

const title = computed(() =>
  props.isEditing
    ? props.editingName || t('claudeProfiles.editProfileTitle')
    : t('claudeProfiles.newProfileTitle'),
)

const description = computed(() =>
  props.isEditing
    ? t('claudeProfiles.modalEditDescription')
    : t('claudeProfiles.modalNewDescription'),
)

const statusLabel = computed(() => {
  if (props.isEditingCurrent) return t('claudeProfiles.modalStatusCurrent')
  if (props.isEditing) {
    return props.form.enabled
      ? t('claudeProfiles.modalStatusEditing')
      : t('claudeProfiles.modalStatusDisabled')
  }
  return t('claudeProfiles.modalStatusDraft')
})

const statusPillClass = computed(() => {
  if (props.isEditingCurrent) return 'pe-pill--accent'
  if (props.isEditing && !props.form.enabled) return 'pe-pill--danger'
  if (props.isEditing) return 'pe-pill--accent'
  return ''
})

/* ========================================================================
 * 保存前校验：失败时顶部汇总条 + 可跳转到第一个错误字段所在分段
 * ======================================================================== */

interface ValidationError {
  section: ClaudeProfileFormSectionId
  message: string
}

const showValidation = ref(false)

const validationErrors = computed<ValidationError[]>(() => {
  const errors: ValidationError[] = []
  if (!props.form.name.trim()) {
    errors.push({ section: 'basic', message: t('claudeProfiles.validationNameRequired') })
  }
  return errors
})

const handleSaveClick = () => {
  if (validationErrors.value.length > 0) {
    showValidation.value = true
    scrollToSection(validationErrors.value[0].section)
    return
  }
  showValidation.value = false
  emit('save')
}

/* ========================================================================
 * 段导航 + scroll-spy（IntersectionObserver，顶部 -140px 锚线）
 * ======================================================================== */

const sectionItems = computed<ClaudeProfileEditorSectionItem[]>(() => ([
  {
    id: 'basic',
    title: t('claudeProfiles.sections.basic.title'),
    description: t('claudeProfiles.sections.basic.description'),
    icon: 'Layers',
  },
  {
    id: 'connection',
    title: t('claudeProfiles.sections.connection.title'),
    description: t('claudeProfiles.sections.connection.description'),
    icon: 'Globe',
  },
  {
    id: 'auth',
    title: t('claudeProfiles.sections.auth.title'),
    description: t('claudeProfiles.sections.auth.description'),
    icon: 'ShieldCheck',
  },
  {
    id: 'status',
    title: t('claudeProfiles.sections.status.title'),
    description: t('claudeProfiles.sections.status.description'),
    icon: 'SlidersHorizontal',
  },
]))

const scrollRef = ref<HTMLElement | null>(null)
const activeSectionId = ref<ClaudeProfileFormSectionId>('basic')
const sectionRefs = ref<Record<ClaudeProfileFormSectionId, HTMLElement | null>>({
  basic: null,
  connection: null,
  auth: null,
  status: null,
})

type SectionRefTarget = Element | { $el?: unknown } | null

const resolveSectionElement = (target: SectionRefTarget): HTMLElement | null => {
  if (!target) return null
  if (target instanceof HTMLElement) return target
  if ('$el' in target) {
    const { $el } = target
    return $el instanceof HTMLElement ? $el : null
  }
  return null
}

const registerSectionRef = (sectionId: ClaudeProfileFormSectionId, target: SectionRefTarget) => {
  sectionRefs.value[sectionId] = resolveSectionElement(target)
}

let sectionObserver: IntersectionObserver | null = null

const teardownSectionObserver = () => {
  sectionObserver?.disconnect()
  sectionObserver = null
}

const setupSectionObserver = () => {
  teardownSectionObserver()
  const container = scrollRef.value
  if (!container || typeof IntersectionObserver === 'undefined') return

  const elementToSection = new Map<Element, ClaudeProfileFormSectionId>()
  CLAUDE_PROFILE_FORM_SECTION_IDS.forEach((sectionId) => {
    const element = sectionRefs.value[sectionId]
    if (element) elementToSection.set(element, sectionId)
  })
  if (elementToSection.size === 0) return

  const visibility = new Map<ClaudeProfileFormSectionId, boolean>()

  sectionObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        const sectionId = elementToSection.get(entry.target)
        if (sectionId) visibility.set(sectionId, entry.isIntersecting)
      }
      const activeId = CLAUDE_PROFILE_FORM_SECTION_IDS.find(id => visibility.get(id))
      if (activeId) activeSectionId.value = activeId
    },
    { root: container, rootMargin: '-140px 0px -70% 0px', threshold: 0 },
  )

  elementToSection.forEach((_sectionId, element) => sectionObserver?.observe(element))
}

const scrollToSection = (sectionId: ClaudeProfileFormSectionId) => {
  activeSectionId.value = sectionId

  const container = scrollRef.value
  const element = sectionRefs.value[sectionId]
  if (!container || !element) return

  container.scrollTo({ top: Math.max(element.offsetTop - 16, 0), behavior: 'smooth' })
}

watch(
  () => props.modelValue,
  (isOpen) => {
    if (!isOpen) {
      showValidation.value = false
      activeSectionId.value = 'basic'
      teardownSectionObserver()
      return
    }

    showValidation.value = false
    activeSectionId.value = 'basic'
    void nextTick(() => {
      scrollRef.value?.scrollTo({ top: 0 })
      setupSectionObserver()
    })
  },
  { immediate: true },
)

// 模板填充切到 connection 段由视图驱动（applyClaudeProviderTemplate），此处响应外部跳转
defineExpose({ scrollToSection })

onBeforeUnmount(teardownSectionObserver)
</script>
