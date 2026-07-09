<template>
  <div class="skills-migration-view">
    <div class="skills-migration-view__shell">
      <section class="skills-migration-view__hero">
        <div class="skills-migration-view__badge">
          {{ tt('Skills 迁移', 'Skills migration') }}
        </div>
        <h1 class="skills-migration-view__title">
          {{ tt('Skills 已从 CCR UI 下线', 'Skills has been removed from CCR UI') }}
        </h1>
        <p class="skills-migration-view__lead">
          {{ tt('CCR UI 现在只保留 CLI 配置管理主线，不再内置 skills 安装、市场和来源管理。', 'CCR UI now only keeps the CLI configuration management path and no longer embeds skills installation, marketplace, or source management.') }}
        </p>
        <p class="skills-migration-view__copy">
          {{ tt('之后请改用独立应用', 'Use the standalone app instead') }}
          <a
            :href="skillportRepoUrl"
            target="_blank"
            rel="noreferrer"
            class="skills-migration-view__link"
          >{{ tt('skillport', 'skillport') }}</a>
          {{ tt('处理 skills。', 'to handle skills.') }}
        </p>

        <div
          class="skills-migration-view__status"
          data-testid="skills-migration-status"
        >
          <span
            class="skills-migration-view__status-pill"
            :class="statusPillClass"
          >
            {{ statusPillLabel }}
          </span>
          <p class="skills-migration-view__status-copy">
            {{ statusSummary }}
          </p>
        </div>

        <div class="skills-migration-view__actions">
          <button
            v-if="isDetecting"
            type="button"
            class="skills-migration-view__primary skills-migration-view__primary--pending"
            data-testid="skills-migration-primary"
            disabled
          >
            {{ tt('检测 skillport…', 'Detecting skillport...') }}
          </button>

          <button
            v-else-if="appStatus.installed"
            type="button"
            class="skills-migration-view__primary"
            data-testid="skills-migration-primary"
            :disabled="isOpening"
            @click="handlePrimaryAction"
          >
            <img
              :src="skillportBadgeUrl"
              alt=""
              class="skills-migration-view__primary-icon"
            >
            <span>{{ isOpening ? tt('正在打开…', 'Opening...') : tt('打开 skillport', 'Open skillport') }}</span>
          </button>

          <a
            v-else
            :href="skillportRepoUrl"
            target="_blank"
            rel="noreferrer"
            class="skills-migration-view__primary"
            data-testid="skills-migration-primary"
          >
            {{ tt('前往 skillport 仓库', 'Go to the skillport repository') }}
          </a>

          <button
            type="button"
            class="skills-migration-view__secondary"
            data-testid="skills-migration-refresh"
            :disabled="isDetecting"
            @click="refreshAppStatus"
          >
            {{ tt('重新检测', 'Recheck') }}
          </button>

          <RouterLink
            to="/configs"
            class="skills-migration-view__secondary"
          >
            {{ tt('返回配置管理', 'Back to config management') }}
          </RouterLink>
        </div>

        <p
          v-if="launchError"
          class="skills-migration-view__error"
          data-testid="skills-migration-error"
        >
          {{ launchError }}
        </p>

        <div class="skills-migration-view__helper-links">
          <a
            :href="skillportRepoUrl"
            target="_blank"
            rel="noreferrer"
            class="skills-migration-view__helper-link"
          >
            {{ tt('查看仓库说明', 'View repository instructions') }}
          </a>
        </div>
      </section>

      <section class="skills-migration-view__grid">
        <article class="skills-migration-view__card">
          <h2>{{ tt('为什么下线', 'Why it was removed') }}</h2>
          <p>
            {{ tt('这一层功能和 CCR UI 的核心定位不一致。继续保留会让路由、状态和桌面后端持续膨胀。', 'This layer did not fit CCR UI’s core scope. Keeping it would keep inflating routes, state, and the desktop backend.') }}
          </p>
        </article>

        <article class="skills-migration-view__card">
          <h2>{{ tt('现在去哪里', 'Where to go now') }}</h2>
          <p>
            {{ tt('skills 的浏览、安装和管理统一改到 skillport。CCR UI 只负责 CLI 配置、运行态和数据面。', 'Browse, install, and manage skills in skillport. CCR UI only handles CLI config, runtime state, and data surfaces.') }}
          </p>
        </article>

        <article class="skills-migration-view__card">
          <h2>{{ tt('怎么开始', 'How to start') }}</h2>
          <p>
            {{ tt('如果本机已安装 skillport，这里会直接显示打开按钮。还没安装时，请先去仓库查看最新安装说明。', 'If skillport is already installed, this page will show an open button. Otherwise, check the repository for install instructions first.') }}
          </p>
        </article>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import skillportBadgeUrl from '@/assets/skillport-badge.svg'
import {
  detectSkillportApp,
  isTauriEnvironment,
  openSkillportApp,
  type SkillportAppStatus,
} from '@/api/domains/system'
import { logger } from '@/utils/logger'

const skillportRepoUrl = 'https://github.com/bahayonghang/skills-manage-windows'

const { locale } = useI18n()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)

const isDetecting = ref(false)
const isOpening = ref(false)
const launchError = ref('')
const appStatus = ref<SkillportAppStatus>({
  supported: false,
  installed: false,
  platform: 'other',
  source: 'unsupported',
})

const statusPillLabel = computed(() => {
  if (isDetecting.value) return tt('正在检测', 'Detecting')
  if (appStatus.value.installed) return tt('已检测到安装', 'Installed')
  if (appStatus.value.supported) return tt('未检测到安装', 'Not installed')
  return tt('当前环境不支持', 'Unsupported environment')
})

const statusPillClass = computed(() => {
  if (isDetecting.value) return 'skills-migration-view__status-pill--pending'
  if (appStatus.value.installed) return 'skills-migration-view__status-pill--ready'
  if (appStatus.value.supported) return 'skills-migration-view__status-pill--empty'
  return 'skills-migration-view__status-pill--unsupported'
})

const statusSummary = computed(() => {
  if (isDetecting.value) {
    return tt('正在检查本机是否已经安装 skillport。', 'Checking whether skillport is installed locally.')
  }

  if (appStatus.value.installed) {
    return tt('已检测到本机安装，可以直接从这里拉起独立应用。', 'Skillport is installed locally and can be launched from here.')
  }

  if (appStatus.value.supported) {
    return tt('当前没有检测到本机安装，请先前往仓库查看最新安装说明。', 'No local install was detected. Check the repository instructions first.')
  }

  return tt('当前运行环境暂不支持自动检测，请直接前往仓库查看说明。', 'The current environment does not support auto-detection. Check the repository instructions directly.')
})

/*
 * ========================================================================
 * 步骤1：同步 skillport 探测状态
 * ========================================================================
 * 目标：
 * 1) 在进入迁移页时拿到可渲染的安装状态
 * 2) 让未安装、已安装、当前环境不支持三种分支稳定落地
 * 数据源：
 * 1) Tauri shell 探测命令
 * 2) 当前运行环境是否具备 Tauri 能力
 */
const refreshAppStatus = async (): Promise<void> => {
  logger.info('[skills-migration] 开始探测 skillport 状态')

  // 1.1 清理上一轮错误并进入探测态
  isDetecting.value = true
  launchError.value = ''

  try {
    // 1.2 非 Tauri 环境直接走不支持分支，避免无意义 invoke
    if (!isTauriEnvironment()) {
      appStatus.value = {
        supported: false,
        installed: false,
        platform: 'other',
        source: 'unsupported',
      }
      return
    }

    // 1.3 读取后端探测结果并刷新前端状态
    appStatus.value = await detectSkillportApp()
  } catch (error) {
    // 1.4 探测失败时回退到保守状态，仍保留仓库入口
    appStatus.value = {
      supported: true,
      installed: false,
      platform: 'other',
      source: 'not_found',
    }
  launchError.value = tt('自动检测失败，请先查看仓库说明后再重试。', 'Auto-detection failed. Check the repository instructions first, then retry.')
    logger.warn('[skills-migration] 探测 skillport 状态失败', error)
  } finally {
    // 1.5 结束探测态，允许用户继续操作
    isDetecting.value = false
    logger.info('[skills-migration] skillport 状态探测完成', {
      supported: appStatus.value.supported,
      installed: appStatus.value.installed,
      platform: appStatus.value.platform,
      source: appStatus.value.source,
    })
  }
}

/*
 * ========================================================================
 * 步骤2：拉起已检测到的独立应用
 * ========================================================================
 * 目标：
 * 1) 只在已检测到安装时触发打开动作
 * 2) 打开失败时保留仓库回退入口，不把用户卡死在按钮上
 * 数据源：
 * 1) 当前探测状态
 * 2) Tauri shell 打开命令
 */
const handlePrimaryAction = async (): Promise<void> => {
  // 2.1 非已安装分支直接退出，避免误触
  if (!appStatus.value.installed) {
    return
  }

  logger.info('[skills-migration] 开始打开 skillport')

  // 2.2 进入打开态并清理旧错误
  isOpening.value = true
  launchError.value = ''

  try {
    // 2.3 调用后端打开独立应用
    await openSkillportApp()
  } catch (error) {
    // 2.4 打开失败时保留仓库入口，并给出页内错误提示
    launchError.value = tt('已检测到 skillport，但拉起失败。请先查看仓库说明，确认安装是否完整。', 'Skillport was detected, but launch failed. Check the repository instructions and confirm the install is complete.')
    logger.error('[skills-migration] 打开 skillport 失败', error)
  } finally {
    // 2.5 结束打开态，允许再次尝试
    isOpening.value = false
    logger.info('[skills-migration] 打开 skillport 流程结束')
  }
}

onMounted(() => {
  void refreshAppStatus()
})
</script>

<style scoped>
.skills-migration-view {
  @apply min-h-full px-4 py-6 sm:px-6;
}

.skills-migration-view__shell {
  @apply mx-auto flex max-w-5xl flex-col gap-6;
}

.skills-migration-view__hero {
  @apply rounded-2xl border border-border-default/60 bg-bg-elevated/70 p-8;
}

.skills-migration-view__badge {
  @apply inline-flex items-center rounded-full border border-accent-primary/20 bg-accent-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-accent-primary;
}

.skills-migration-view__title {
  @apply mt-4 text-3xl font-semibold tracking-[-0.04em] text-text-primary;
}

.skills-migration-view__lead {
  @apply mt-4 max-w-3xl text-base leading-7 text-text-primary;
}

.skills-migration-view__copy {
  @apply mt-3 max-w-3xl text-sm leading-7 text-text-secondary;
}

.skills-migration-view__link {
  @apply font-semibold text-accent-primary hover:underline;
}

.skills-migration-view__status {
  @apply mt-6 flex flex-col gap-3 rounded-xl border border-border-default/50 bg-bg-surface/70 p-4;
}

.skills-migration-view__status-pill {
  @apply inline-flex w-fit items-center rounded-full px-3 py-1 text-xs font-semibold tracking-[0.12em];
}

.skills-migration-view__status-pill--pending {
  @apply border border-border-default/60 bg-bg-overlay/70 text-text-primary;
}

.skills-migration-view__status-pill--ready {
  @apply border border-emerald-500/20 bg-emerald-500/10 text-emerald-600 dark:text-emerald-300;
}

.skills-migration-view__status-pill--empty {
  @apply border border-amber-500/20 bg-amber-500/10 text-amber-700 dark:text-amber-300;
}

.skills-migration-view__status-pill--unsupported {
  @apply border border-border-default/50 bg-bg-base/60 text-text-secondary;
}

.skills-migration-view__status-copy {
  @apply text-sm leading-7 text-text-secondary;
}

.skills-migration-view__actions {
  @apply mt-6 flex flex-wrap gap-3;
}

.skills-migration-view__primary,
.skills-migration-view__secondary {
  @apply inline-flex min-h-[44px] items-center justify-center rounded-2xl px-5 py-3 text-sm font-semibold transition-colors;
}

.skills-migration-view__primary {
  @apply gap-2 bg-accent-primary text-white hover:bg-accent-primary/90;
}

.skills-migration-view__primary:disabled,
.skills-migration-view__secondary:disabled {
  @apply cursor-not-allowed opacity-70;
}

.skills-migration-view__primary--pending {
  @apply bg-accent-primary/70;
}

.skills-migration-view__primary-icon {
  @apply h-5 w-5 rounded-lg;
}

.skills-migration-view__secondary {
  @apply border border-border-default/60 bg-bg-base/50 text-text-primary hover:bg-bg-surface/70;
}

.skills-migration-view__error {
  @apply mt-4 rounded-2xl border border-rose-500/20 bg-rose-500/10 px-4 py-3 text-sm leading-6 text-rose-700 dark:text-rose-200;
}

.skills-migration-view__helper-links {
  @apply mt-4 flex flex-wrap gap-4;
}

.skills-migration-view__helper-link {
  @apply text-sm font-medium text-text-secondary underline-offset-4 hover:text-text-primary hover:underline;
}

.skills-migration-view__grid {
  @apply grid gap-4 md:grid-cols-3;
}

.skills-migration-view__card {
  @apply rounded-xl border border-border-default/50 bg-bg-surface/70 p-5;
}

.skills-migration-view__card h2 {
  @apply text-sm font-semibold text-text-primary;
}

.skills-migration-view__card p {
  @apply mt-3 text-sm leading-7 text-text-secondary;
}
</style>
