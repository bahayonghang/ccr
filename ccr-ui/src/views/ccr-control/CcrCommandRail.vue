<template>
  <aside class="flex w-full flex-none flex-col gap-4 animate-slide-in-left xl:w-80">
    <Card
      variant="glass"
      class="flex flex-col !p-0 overflow-hidden neon-card xl:flex-1"
      padding="none"
      body-class="h-full min-h-[22rem] max-h-[60vh] xl:max-h-none flex flex-col"
    >
      <div class="flex p-2 gap-1 border-b border-border-color bg-bg-secondary/50">
        <button
          v-for="tab in sidebarTabs"
          :key="tab.id"
          class="flex-1 flex items-center justify-center gap-2 py-2 rounded-lg text-xs font-bold transition-colors duration-300 relative overflow-hidden group"
          :class="activeTab === tab.id
            ? 'bg-accent-primary/10 text-accent-primary shadow-neon-jade-sm'
            : 'text-text-muted hover:bg-bg-hover hover:text-white'"
          @click="activeTab = tab.id"
        >
          <SIcon
            :name="tab.icon"
            size="w-4 h-4"
          />
          <span>{{ $t(tab.labelKey) }}</span>
          <div
            v-if="activeTab === tab.id"
            class="absolute inset-0 bg-gradient-to-t from-accent-primary/10 to-transparent opacity-50"
          />
        </button>
      </div>

      <div class="flex-1 min-h-0 overflow-hidden relative">
        <Transition
          name="fade-slide"
          mode="out-in"
        >
          <div
            v-if="activeTab === 'commands'"
            key="commands"
            class="h-full flex flex-col"
          >
            <div class="px-3 py-3 border-b border-border-color">
              <div class="flex gap-2 overflow-x-auto custom-scrollbar pb-1">
                <button
                  v-for="mod in modules"
                  :key="mod.id"
                  class="flex-shrink-0 px-3 py-1.5 rounded-lg text-xs font-bold transition-colors border border-transparent"
                  :class="selectedModuleId === mod.id
                    ? 'bg-accent-primary/20 text-accent-primary border-accent-primary/30'
                    : 'bg-bg-secondary text-text-muted hover:bg-bg-hover hover:text-white'"
                  @click="$emit('selectModule', mod.id)"
                >
                  {{ mod.name }}
                </button>
              </div>
            </div>

            <div class="flex-1 overflow-y-auto custom-scrollbar p-2 space-y-2">
              <div
                v-for="cmd in selectedModule?.commands"
                :key="cmd.command"
                class="group relative rounded-xl border border-transparent transition-colors duration-300 hover:bg-bg-hover hover:border-accent-primary/20"
                :class="selectedCommand?.command === cmd.command ? 'bg-accent-primary/10 border-accent-primary/40 shadow-neon-jade-sm' : ''"
              >
                <button
                  type="button"
                  class="flex w-full items-start gap-3 rounded-xl p-3 pr-12 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/35 focus-visible:ring-offset-2 focus-visible:ring-offset-bg-primary"
                  :aria-label="`Select command ${cmd.name}: ccr ${cmd.command}`"
                  @click="$emit('selectCommand', cmd)"
                >
                  <div
                    class="mt-0.5 w-7 h-7 rounded-lg bg-bg-secondary flex items-center justify-center group-hover:scale-110 transition-transform"
                    :class="selectedCommand?.command === cmd.command ? 'bg-accent-primary text-white' : 'text-text-muted group-hover:text-accent-primary'"
                  >
                    <SIcon
                      name="Terminal"
                      size="w-4 h-4"
                    />
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center justify-between mb-0.5">
                      <span
                        class="text-sm font-bold truncate"
                        :class="selectedCommand?.command === cmd.command ? 'text-accent-primary' : 'text-white'"
                      >{{ cmd.name }}</span>
                    </div>
                    <div class="text-[10px] font-mono opacity-60 mb-1 text-text-primary">
                      ccr {{ cmd.command }}
                    </div>
                    <p class="text-[10px] text-text-muted line-clamp-2 leading-relaxed">
                      {{ cmd.description }}
                    </p>
                    <div class="mt-2 flex flex-wrap items-center gap-1.5">
                      <span
                        class="rounded-full border px-1.5 py-0.5 text-[9px] font-bold uppercase tracking-wide"
                        :class="cmd.executable ? 'border-accent-success/30 bg-accent-success/10 text-accent-success' : 'border-accent-warning/30 bg-accent-warning/10 text-accent-warning'"
                      >
                        {{ cmd.executable ? $t('ccrControl.executable') : $t('ccrControl.unsupported') }}
                      </span>
                      <span
                        v-if="cmd.dangerous"
                        class="rounded-full border border-accent-danger/30 bg-accent-danger/10 px-1.5 py-0.5 text-[9px] font-bold uppercase tracking-wide text-accent-danger"
                      >
                        {{ $t('ccrControl.dangerous') }}
                      </span>
                    </div>
                  </div>
                </button>
                <div class="absolute right-3 top-3 flex items-center gap-1">
                  <SIcon
                    v-if="cmd.dangerous"
                    name="AlertTriangle"
                    size="w-3 h-3"
                    class="text-accent-danger animate-pulse"
                  />
                  <button
                    type="button"
                    class="rounded-md p-1 text-text-muted transition-transform hover:scale-125 hover:text-accent-warning focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-warning/40"
                    :aria-label="isFavorite(cmd.command) ? `Remove ${cmd.name} from favorites` : `Add ${cmd.name} to favorites`"
                    @click="$emit('toggleFavorite', cmd)"
                  >
                    <SIcon
                      name="Star"
                      size="w-3 h-3"
                      :class="isFavorite(cmd.command) ? 'fill-accent-warning text-accent-warning' : ''"
                    />
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div
            v-else-if="activeTab === 'favorites'"
            key="favorites"
            class="h-full overflow-y-auto custom-scrollbar p-2 space-y-2"
          >
            <div
              v-if="favorites.length === 0"
              class="h-full flex flex-col items-center justify-center text-text-muted"
            >
              <SIcon
                name="Star"
                size="w-8 h-8"
                class="opacity-20 mb-2"
              />
              <span class="text-xs">{{ $t('ccrControl.noFavorites') }}</span>
            </div>
            <div
              v-for="fav in favorites"
              :key="fav.id"
              class="relative rounded-xl border border-border-color bg-bg-secondary transition-[border-color,box-shadow] hover:border-accent-warning/30 hover:shadow-neon-gold-sm group"
            >
              <button
                type="button"
                class="w-full rounded-xl p-3 pr-10 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-warning/35 focus-visible:ring-offset-2 focus-visible:ring-offset-bg-primary"
                :aria-label="`Run favorite command ${fav.display_name || fav.command}`"
                @click="$emit('executeFavorite', fav)"
              >
                <div class="mb-2 flex items-center justify-between">
                  <span class="text-xs font-bold text-accent-warning">{{ fav.display_name || fav.command }}</span>
                </div>
                <div class="mb-2 text-[10px] font-mono text-text-primary">
                  ccr {{ fav.command }}
                </div>
                <div class="flex justify-end text-accent-warning">
                  <SIcon
                    name="Play"
                    size="w-3 h-3"
                    class="fill-current"
                  />
                </div>
              </button>
              <button
                type="button"
                class="absolute right-3 top-3 rounded-md p-1 text-text-muted transition-colors hover:text-accent-danger focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-danger/40"
                :aria-label="`Remove favorite ${fav.display_name || fav.command}`"
                @click="$emit('removeFavorite', fav.id)"
              >
                <SIcon
                  name="X"
                  size="w-3 h-3"
                />
              </button>
            </div>
          </div>

          <div
            v-else-if="activeTab === 'history'"
            key="history"
            class="h-full flex flex-col"
          >
            <div class="p-2 border-b border-border-color flex justify-end">
              <button
                v-if="history.length > 0"
                class="text-[10px] flex items-center gap-1 text-text-muted hover:text-accent-danger px-2 py-1 hover:bg-bg-hover rounded transition-colors"
                @click="$emit('clearHistory')"
              >
                <SIcon
                  name="Trash2"
                  size="w-3 h-3"
                />
                {{ $t('ccrControl.clearHistory') }}
              </button>
            </div>
            <div class="flex-1 overflow-y-auto custom-scrollbar p-2 space-y-2">
              <div
                v-if="history.length === 0"
                class="h-full flex flex-col items-center justify-center text-text-muted"
              >
                <SIcon
                  name="History"
                  size="w-8 h-8"
                  class="opacity-20 mb-2"
                />
                <span class="text-xs">{{ $t('ccrControl.noHistory') }}</span>
              </div>
              <button
                v-for="item in history"
                :key="item.id"
                type="button"
                class="flex w-full items-center gap-3 rounded-lg border border-border-color bg-bg-secondary p-2.5 text-left transition-colors hover:bg-bg-hover focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/35 focus-visible:ring-offset-2 focus-visible:ring-offset-bg-primary group"
                :aria-label="`Run history command ${item.command}`"
                @click="$emit('executeHistory', item)"
              >
                <div
                  class="w-2 h-2 rounded-full flex-shrink-0"
                  :class="item.success ? 'bg-accent-success shadow-neon-jade-sm' : 'bg-accent-danger shadow-neon-danger-sm'"
                />
                <div class="flex-1 min-w-0">
                  <div class="text-xs font-mono font-bold truncate text-white">
                    {{ item.command }}
                  </div>
                  <div class="text-[10px] text-text-muted flex items-center gap-2">
                    <span>{{ formatTime(item.executed_at) }}</span>
                    <span>{{ item.duration_ms }}ms</span>
                  </div>
                </div>
                <SIcon
                  name="Play"
                  size="w-3 h-3"
                  class="text-text-muted opacity-0 group-hover:opacity-100 transition-opacity"
                />
              </button>
            </div>
          </div>
        </Transition>
      </div>
    </Card>
  </aside>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import Card from '@/components/ui/Card.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { CcrCommand, CcrModule, CommandHistory, FavoriteCommand } from '@/api/ccr-control'

defineProps<{
  modules: CcrModule[]
  selectedModuleId: string
  selectedModule?: CcrModule
  selectedCommand: CcrCommand | null
  favorites: FavoriteCommand[]
  history: CommandHistory[]
  isFavorite: (command: string) => boolean
}>()

defineEmits<{
  selectModule: [moduleId: string]
  selectCommand: [command: CcrCommand]
  toggleFavorite: [command: CcrCommand]
  executeFavorite: [favorite: FavoriteCommand]
  removeFavorite: [id: string]
  executeHistory: [historyItem: CommandHistory]
  clearHistory: []
}>()

const { t } = useI18n()
const activeTab = ref<'commands' | 'favorites' | 'history'>('commands')
const sidebarTabs: { id: 'commands' | 'favorites' | 'history'; labelKey: string; icon: string }[] = [
  { id: 'commands', labelKey: 'ccrControl.commands', icon: 'List' },
  { id: 'favorites', labelKey: 'ccrControl.favorites', icon: 'Star' },
  { id: 'history', labelKey: 'ccrControl.history', icon: 'History' },
]

const formatTime = (dateStr: string) => {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  if (diff < 60000) return t('ccrControl.time.justNow')
  if (diff < 3600000) return t('ccrControl.time.minutesAgo', { count: Math.floor(diff / 60000) })
  if (diff < 86400000) return t('ccrControl.time.hoursAgo', { count: Math.floor(diff / 3600000) })
  return date.toLocaleDateString()
}
</script>
