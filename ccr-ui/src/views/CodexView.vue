<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <div class="max-w-7xl mx-auto space-y-5">
      <section class="grid grid-cols-1 xl:grid-cols-3 gap-4">
        <Card
          variant="glass"
          class="xl:col-span-2 relative overflow-hidden p-6"
        >
          <div class="absolute inset-y-0 right-0 w-72 bg-gradient-to-l from-pink-500/10 via-purple-500/5 to-transparent pointer-events-none" />

          <div class="relative z-10 space-y-5">
            <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
              <div class="space-y-3">
                <div class="flex items-center gap-3">
                  <div class="w-12 h-12 rounded-2xl bg-pink-500/10 border border-pink-500/20 flex items-center justify-center shadow-lg backdrop-blur-md">
                    <SIcon
                      name="Code2"
                      size="w-6 h-6"
                      class="text-pink-400"
                    />
                  </div>
                  <div>
                    <h1 class="text-3xl font-bold font-display text-white tracking-tight">
                      Codex
                    </h1>
                    <p class="text-sm text-white/70">
                      先看当前账号、配置健康度和下一步，再进入细项管理。
                    </p>
                  </div>
                </div>

                <div class="flex flex-wrap gap-2">
                  <span class="px-3 py-1 rounded-full text-xs font-semibold uppercase tracking-wider bg-pink-500/10 text-pink-300 border border-pink-500/20">
                    workflow first
                  </span>
                  <span class="px-3 py-1 rounded-full text-xs font-semibold uppercase tracking-wider border border-white/10 bg-white/5 text-white/75">
                    {{ versionLabel }}
                  </span>
                  <span class="px-3 py-1 rounded-full text-xs font-semibold uppercase tracking-wider border border-emerald-500/20 bg-emerald-500/10 text-emerald-300">
                    {{ currentProfileLabel }}
                  </span>
                </div>
              </div>

              <div class="flex flex-wrap gap-2 lg:justify-end">
                <RouterLink to="/codex/auth">
                  <Button
                    variant="glass"
                    size="sm"
                  >
                    <SIcon
                      name="KeyRound"
                      size="w-4 h-4"
                      class="mr-2"
                    />
                    账号
                  </Button>
                </RouterLink>
                <RouterLink to="/codex/profiles">
                  <Button
                    variant="glass"
                    size="sm"
                  >
                    <SIcon
                      name="Folders"
                      size="w-4 h-4"
                      class="mr-2"
                    />
                    Profiles
                  </Button>
                </RouterLink>
                <Button
                  variant="ghost"
                  size="sm"
                  :disabled="loading"
                  @click="refresh(true)"
                >
                  <SIcon
                    name="RefreshCw"
                    size="w-4 h-4"
                    class="mr-2"
                    :class="{ 'animate-spin': loading }"
                  />
                  刷新
                </Button>
              </div>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
              <div class="rounded-2xl border border-white/10 bg-white/5 px-4 py-3">
                <p class="text-xs uppercase tracking-[0.2em] text-white/45 mb-1">
                  当前账号
                </p>
                <p
                  class="text-lg font-semibold text-white truncate"
                  :title="currentAccountLabel"
                >
                  {{ currentAccountLabel }}
                </p>
              </div>
              <div class="rounded-2xl border border-white/10 bg-white/5 px-4 py-3">
                <p class="text-xs uppercase tracking-[0.2em] text-white/45 mb-1">
                  累计请求
                </p>
                <p class="text-lg font-semibold text-white">
                  {{ summary?.usage.all_time.total_requests ?? 0 }}
                </p>
              </div>
              <div class="rounded-2xl border border-white/10 bg-white/5 px-4 py-3">
                <p class="text-xs uppercase tracking-[0.2em] text-white/45 mb-1">
                  累计 Tokens
                </p>
                <p class="text-lg font-semibold text-white">
                  {{ usageTotalTokens }}
                </p>
              </div>
            </div>
          </div>
        </Card>

        <Card
          variant="glass"
          class="p-5"
        >
          <div class="flex items-center gap-3 mb-4">
            <div class="w-10 h-10 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center">
              <SIcon
                name="Route"
                size="w-5 h-5"
                class="text-amber-300"
              />
            </div>
            <div>
              <h2 class="text-base font-semibold text-white">
                下一步
              </h2>
              <p class="text-xs text-white/55">
                只保留最该先做的动作
              </p>
            </div>
          </div>

          <div class="space-y-3">
            <RouterLink
              v-for="action in nextActions"
              :key="action.title"
              :to="action.to"
              class="block rounded-2xl border border-white/10 bg-white/5 p-4 transition-all duration-200 hover:border-pink-500/30 hover:bg-white/10"
            >
              <div class="flex items-start gap-3">
                <div
                  class="mt-0.5 flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border"
                  :class="toneClassMap[action.tone]"
                >
                  <SIcon
                    :name="action.icon"
                    size="w-5 h-5"
                  />
                </div>
                <div class="min-w-0 flex-1">
                  <div class="flex items-center justify-between gap-3">
                    <h3 class="text-sm font-semibold text-white">
                      {{ action.title }}
                    </h3>
                    <SIcon
                      name="ArrowRight"
                      size="w-4 h-4"
                      class="text-white/35 shrink-0"
                    />
                  </div>
                  <p class="mt-1 text-sm leading-6 text-white/65">
                    {{ action.description }}
                  </p>
                </div>
              </div>
            </RouterLink>
          </div>
        </Card>
      </section>

      <section class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-3">
        <RouterLink
          v-for="item in healthItems"
          :key="item.key"
          :to="item.to"
          class="group"
        >
          <Card
            variant="elevated"
            hover
            class="h-full p-4 border border-white/10"
          >
            <div class="flex items-start justify-between gap-3">
              <div
                class="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border"
                :class="toneClassMap[item.tone]"
              >
                <SIcon
                  :name="item.icon"
                  size="w-5 h-5"
                />
              </div>
              <span class="text-[11px] uppercase tracking-[0.18em] text-white/35">
                状态
              </span>
            </div>
            <p class="mt-4 text-xs uppercase tracking-[0.2em] text-white/45">
              {{ item.title }}
            </p>
            <p class="mt-1 text-lg font-semibold text-white break-words">
              {{ item.value }}
            </p>
            <p class="mt-2 text-sm leading-6 text-white/60">
              {{ item.detail }}
            </p>
          </Card>
        </RouterLink>
      </section>

      <section class="grid grid-cols-1 xl:grid-cols-3 gap-4">
        <Card
          variant="glass"
          class="xl:col-span-2 p-5"
        >
          <div class="flex items-center justify-between gap-3 mb-4">
            <div>
              <h2 class="text-base font-semibold text-white">
                管理入口
              </h2>
              <p class="text-sm text-white/55">
                把细项管理降级成次级入口，需要时再深入。
              </p>
            </div>
            <RouterLink
              to="/codex/settings"
              class="text-sm text-pink-300 hover:text-pink-200 transition-colors"
            >
              打开设置
            </RouterLink>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
            <RouterLink
              v-for="link in managementLinks"
              :key="link.to"
              :to="link.to"
              class="group"
            >
              <Card
                variant="glass"
                hover
                class="h-full p-4 border border-white/10"
              >
                <div class="flex items-start justify-between gap-3 mb-3">
                  <div
                    class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border"
                    :class="toneClassMap[link.tone]"
                  >
                    <SIcon
                      :name="link.icon"
                      size="w-5 h-5"
                    />
                  </div>
                  <span class="rounded-full border border-white/10 bg-white/5 px-2.5 py-1 text-[11px] text-white/55">
                    {{ link.badge }}
                  </span>
                </div>
                <h3 class="text-sm font-semibold text-white group-hover:text-pink-200 transition-colors">
                  {{ link.title }}
                </h3>
                <p class="mt-2 text-sm leading-6 text-white/60">
                  {{ link.description }}
                </p>
              </Card>
            </RouterLink>
          </div>
        </Card>

        <Card
          variant="glass"
          class="p-5"
        >
          <div class="flex items-center gap-3 mb-4">
            <div class="w-10 h-10 rounded-xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center">
              <SIcon
                name="Sparkles"
                size="w-5 h-5"
                class="text-indigo-300"
              />
            </div>
            <div>
              <h2 class="text-base font-semibold text-white">
                工作流摘要
              </h2>
              <p class="text-xs text-white/55">
                一眼确认是否可以直接开工
              </p>
            </div>
          </div>

          <div
            v-if="error"
            class="rounded-2xl border border-rose-500/20 bg-rose-500/10 p-4 text-sm text-rose-200"
          >
            <p class="font-medium mb-2">
              仪表盘数据加载失败
            </p>
            <p class="text-rose-100/80 break-words">
              {{ error }}
            </p>
          </div>

          <div
            v-else-if="!summary && loading"
            class="space-y-3"
          >
            <div class="h-20 rounded-2xl bg-white/5 animate-pulse" />
            <div class="h-20 rounded-2xl bg-white/5 animate-pulse" />
            <div class="h-20 rounded-2xl bg-white/5 animate-pulse" />
          </div>

          <EmptyState
            v-else-if="!summary"
            icon="Inbox"
            title="暂时还没有仪表盘数据"
            description="可以先刷新一次，或者进入 Auth / Profiles 页面补齐基础配置。"
            action-text="立即刷新"
            action-icon="RefreshCw"
            :on-action="() => refresh(true)"
          />

          <div
            v-else
            class="space-y-3"
          >
            <div class="rounded-2xl border border-white/10 bg-white/5 p-4">
              <p class="text-xs uppercase tracking-[0.2em] text-white/45 mb-1">
                活跃模型
              </p>
              <p class="text-lg font-semibold text-white">
                {{ summary.usage.top_model?.model || summary.config.model || '未识别' }}
              </p>
              <p class="mt-1 text-sm text-white/60 leading-6">
                {{ summary.usage.top_model
                  ? `近阶段请求 ${summary.usage.top_model.total_requests} 次，输出 ${formatTokens(summary.usage.top_model.total_output_tokens)} tokens`
                  : '暂无按模型维度的活跃数据' }}
              </p>
            </div>

            <div class="rounded-2xl border border-white/10 bg-white/5 p-4">
              <p class="text-xs uppercase tracking-[0.2em] text-white/45 mb-1">
                扩展能力库存
              </p>
              <div class="grid grid-cols-2 gap-3 mt-3 text-sm">
                <div>
                  <p class="text-white/40">
                    MCP
                  </p>
                  <p class="text-white font-semibold">
                    {{ summary.inventory.mcp_servers_total }}
                  </p>
                </div>
                <div>
                  <p class="text-white/40">
                    Config Profiles
                  </p>
                  <p class="text-white font-semibold">
                    {{ summary.inventory.config_profiles_total }}
                  </p>
                </div>
                <div>
                  <p class="text-white/40">
                    Agents
                  </p>
                  <p class="text-white font-semibold">
                    {{ summary.inventory.agents_total }}
                  </p>
                </div>
                <div>
                  <p class="text-white/40">
                    Slash Commands
                  </p>
                  <p class="text-white font-semibold">
                    {{ summary.inventory.slash_commands_total }}
                  </p>
                </div>
              </div>
            </div>

            <div class="rounded-2xl border border-white/10 bg-white/5 p-4">
              <p class="text-xs uppercase tracking-[0.2em] text-white/45 mb-1">
                最近活动
              </p>
              <p class="text-sm text-white/70 leading-6">
                {{ summary.usage.last_activity_at
                  ? `最近活动时间 ${formatDateTime(summary.usage.last_activity_at)}`
                  : '尚未发现最近活动记录' }}
              </p>
            </div>
          </div>
        </Card>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onActivated, onMounted } from 'vue'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useCodexDashboard } from '@/composables/useCodexDashboard'

defineOptions({ name: 'CodexView' })

const {
  summary,
  loading,
  error,
  versionLabel,
  currentAccountLabel,
  currentProfileLabel,
  usageTotalTokens,
  healthItems,
  nextActions,
  managementLinks,
  formatTokens,
  formatDateTime,
  refresh
} = useCodexDashboard()

const toneClassMap = {
  success: 'bg-emerald-500/10 border-emerald-500/20 text-emerald-300',
  warning: 'bg-amber-500/10 border-amber-500/20 text-amber-300',
  danger: 'bg-rose-500/10 border-rose-500/20 text-rose-300',
  neutral: 'bg-white/5 border-white/10 text-white/75'
} as const

onMounted(() => {
  void refresh(false)
})

onActivated(() => {
  void refresh(false)
})
</script>
