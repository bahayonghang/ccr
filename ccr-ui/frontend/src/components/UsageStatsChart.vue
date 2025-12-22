<template>
  <div class="usage-stats-chart h-64 relative select-none">
    <!-- 图例 -->
    <div class="absolute top-0 right-0 flex items-center gap-4 text-xs z-10">
      <div class="flex items-center gap-1.5">
        <span class="w-3 h-3 rounded-[2px] bg-orange-500" />
        <span class="text-guofeng-text-secondary">Codex</span>
      </div>
      <div class="flex items-center gap-1.5">
        <span class="w-3 h-3 rounded-[2px] bg-pink-400" />
        <span class="text-guofeng-text-secondary">Claude</span>
      </div>
      <div class="flex items-center gap-1.5">
        <span class="w-3 h-3 rounded-[2px] bg-blue-500" />
        <span class="text-guofeng-text-secondary">Gemini</span>
      </div>
    </div>
    
    <!-- 柱状图容器 -->
    <div
      class="flex flex-col h-full pt-8 pb-6"
    >
      <div class="flex-1 flex items-end justify-between relative gap-[2px]">
        <!-- 底部轴线 -->
        <div class="absolute bottom-0 left-0 right-0 h-px bg-guofeng-border/50" />

        <div
          v-for="(item, index) in chartData"
          :key="item.date"
          class="flex-1 h-full flex flex-col justify-end items-center cursor-pointer group relative"
          @mouseenter="hoveredIndex = index"
          @mouseleave="hoveredIndex = null"
        >
          <!-- 悬浮高亮背景 -->
          <div 
            class="absolute inset-x-0 top-0 bottom-0 bg-guofeng-text-primary/5 rounded opacity-0 transition-opacity duration-200"
            :class="{ 'opacity-100': hoveredIndex === index }"
          />

          <!-- 堆叠柱子 -->
          <div class="w-full max-w-[80%] flex flex-col-reverse items-center relative z-10 transition-all duration-300">
            <!-- Gemini (蓝色) - Top -->
            <div
              v-if="getValue(item, 'gemini') > 0"
              class="w-full bg-blue-500 transition-all duration-500 ease-out"
              :style="{ height: getBarHeight(item, 'gemini') + 'px' }"
              :class="[
                { 'opacity-90': hoveredIndex !== null && hoveredIndex !== index },
                getCornerClass(item, 'gemini')
              ]"
            />
            <!-- Claude (粉色) - Middle -->
            <div
              v-if="getValue(item, 'claude') > 0"
              class="w-full bg-pink-400 transition-all duration-500 ease-out"
              :style="{ height: getBarHeight(item, 'claude') + 'px' }"
              :class="[
                { 'opacity-90': hoveredIndex !== null && hoveredIndex !== index },
                getCornerClass(item, 'claude')
              ]"
            />
            <!-- Codex (橙色) - Bottom -->
            <div
              v-if="getValue(item, 'codex') > 0"
              class="w-full bg-orange-500 transition-all duration-500 ease-out"
              :style="{ height: getBarHeight(item, 'codex') + 'px' }"
              :class="[
                { 'opacity-90': hoveredIndex !== null && hoveredIndex !== index },
                getCornerClass(item, 'codex')
              ]"
            />
          </div>
          
          <!-- 底部刻度线 -->
          <div 
            class="w-px bg-guofeng-border/50 mt-[1px]" 
            :class="shouldShowDate(index, item.date) ? 'h-1.5' : 'h-1'"
          />

          <!-- 日期标签 (数字) -->
          <span 
            v-if="shouldShowDate(index, item.date)"
            class="absolute top-[100%] mt-1 text-[10px] text-guofeng-text-muted font-mono"
          >
            {{ getDayLabel(item.date) }}
          </span>

          <!-- 月份标签 (Month Start) -->
          <div 
            v-if="isMonthStart(index, item.date)" 
            class="absolute top-[100%] mt-5 left-0 pl-1 border-l border-guofeng-jade/50 h-3 flex items-center"
          >
            <span class="text-[10px] font-bold text-guofeng-jade ml-1 uppercase tracking-wider whitespace-nowrap">
              {{ formatMonthLabel(item.date) }}
            </span>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Tooltip -->
    <Transition name="fade">
      <div
        v-if="hoveredIndex !== null && chartData[hoveredIndex]"
        class="absolute z-20 px-3 py-2 rounded-lg bg-guofeng-bg-primary/95 border border-guofeng-border shadow-xl backdrop-blur-md text-xs pointer-events-none transform -translate-x-1/2 transition-all duration-75"
        :style="tooltipStyle"
      >
        <div class="font-bold text-guofeng-text-primary mb-1.5 border-b border-guofeng-border/50 pb-1">
          {{ formatDateFull(chartData[hoveredIndex].date) }}
        </div>
        <div class="space-y-1">
          <div class="flex items-center justify-between gap-6">
            <div class="flex items-center gap-1.5">
              <span class="w-2 h-2 rounded-[1px] bg-orange-500" />
              <span class="text-guofeng-text-secondary">Codex</span>
            </div>
            <span class="text-guofeng-text-primary font-mono font-medium">
              {{ formatValue(getValue(chartData[hoveredIndex], 'codex')) }}
            </span>
          </div>
          <div class="flex items-center justify-between gap-6">
            <div class="flex items-center gap-1.5">
              <span class="w-2 h-2 rounded-[1px] bg-pink-400" />
              <span class="text-guofeng-text-secondary">Claude</span>
            </div>
            <span class="text-guofeng-text-primary font-mono font-medium">
              {{ formatValue(getValue(chartData[hoveredIndex], 'claude')) }}
            </span>
          </div>
          <div class="flex items-center justify-between gap-6">
            <div class="flex items-center gap-1.5">
              <span class="w-2 h-2 rounded-[1px] bg-blue-500" />
              <span class="text-guofeng-text-secondary">Gemini</span>
            </div>
            <span class="text-guofeng-text-primary font-mono font-medium">
              {{ formatValue(getValue(chartData[hoveredIndex], 'gemini')) }}
            </span>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { DailyStatsItem, StatsViewMode } from '@/types'

const props = defineProps<{
  data: DailyStatsItem[]
  viewMode: StatsViewMode
}>()

const hoveredIndex = ref<number | null>(null)

// 获取当前视图模式下的值
const getValue = (item: DailyStatsItem, platform: 'claude' | 'codex' | 'gemini'): number => {
  const stats = item[platform]
  switch (props.viewMode) {
    case 'sessions': return stats.sessions
    case 'duration': return stats.duration_seconds
    case 'tokens': return stats.tokens || 0
    default: return stats.sessions
  }
}

// 计算最大值用于缩放
const maxValue = computed(() => {
  let max = 0
  for (const item of props.data) {
    const total = getValue(item, 'claude') + getValue(item, 'codex') + getValue(item, 'gemini')
    if (total > max) max = total
  }
  return max || 1
})

// 计算柱子高度
const getBarHeight = (item: DailyStatsItem, platform: 'claude' | 'codex' | 'gemini'): number => {
  const value = getValue(item, platform)
  const maxHeight = 120 // 调整最大高度以适应新布局
  return Math.max(0, (value / maxValue.value) * maxHeight) // 允许 0 高度
}

// 圆角逻辑：只有最顶部的非零元素才有圆角
const getCornerClass = (item: DailyStatsItem, platform: 'claude' | 'codex' | 'gemini') => {
  const vGemini = getValue(item, 'gemini')
  const vClaude = getValue(item, 'claude')
  // const vCodex = getValue(item, 'codex')

  // 渲染顺序(从上到下): Gemini -> Claude -> Codex (因为是 flex-col-reverse)
  // 实际上 flex-col-reverse 让 DOM order 中的第一个元素在最底部。
  // DOM: Gemini(Top in code due to reverse?), Claude, Codex.
  // Wait, flex-col-reverse means the first child is at the bottom visually?
  // No, flex-col-reverse stack items from bottom to top. 
  // First child in DOM -> Bottom visually.
  // But my code has: Gemini(Div1) -> Claude(Div2) -> Codex(Div3)
  // If flex-col-reverse:
  // Div1 (Gemini) will be at the BOTTOM?
  // Let's verify standard CSS.
  // flex-direction: column-reverse; 
  // The first flex item is displayed at the bottom.
  // So: Gemini (Bottom), Claude (Middle), Codex (Top).
  
  // WAIT. My previous code was:
  // Gemini -> Claude -> Codex.
  // user said chart showed: Purple (Claude) Middle, Blue (Gemini) Top, Orange (Codex) Bottom.
  // If Gemini is Blue and on Top.
  // Then Gemini must be the LAST child in a flex-col-reverse container?
  // OR the FIRST child in a flex-col container.
  
  // Let's check previous code logic:
  // <div class="w-full flex flex-col-reverse items-center">
  //   <Gemini/>  <-- First child
  //   <Claude/>
  //   <Codex/>   <-- Last child
  // </div>
  // In column-reverse, Last child is at the TOP.
  // So Codex (Orange) should be at TOP.
  // But screenshot showed Codex (Orange) at BOTTOM.
  
  // User Screenshot 1: 
  // Top: Blue (Gemini)
  // Mid: Pink (Claude)
  // Bot: Orange (Codex)
  
  // This means the rendering order in previous code:
  // Gemini (Div1) -> Bottom? NO.
  // If Codex (Div3) is at Bottom, then Div3 must have been the First visual element.
  // In flex-col-reverse, visual order is Reverse of DOM order.
  // Visual Bottom = DOM First.
  // Visual Top = DOM Last.
  // So if previous code had Gemini first, it should be at Visual Bottom.
  // But screenshot shows Gemini at Top. 
  
  // Maybe I misread the previous code or CSS behavior.
  // Let's stick to standard flex-col-reverse:
  // DOM: [A, B, C] -> Visual: 
  // C (Top)
  // B
  // A (Bottom)
  
  // I want:
  // Gemini (Blue) -> Top
  // Claude (Pink) -> Middle
  // Codex (Orange) -> Bottom
  
  // So in flex-col-reverse:
  // Gemini must be Last child (C)
  // Claude Middle (B)
  // Codex First child (A)
  
  // So DOM order should be: Codex, Claude, Gemini.
  
  if (platform === 'gemini') {
    // Gemini is Top (Last child). Always rounded top if > 0
    return 'rounded-t-[2px]'
  }
  
  if (platform === 'claude') {
    // Claude is Middle. Rounded top only if Gemini is 0
    return vGemini === 0 ? 'rounded-t-[2px]' : ''
  }
  
  if (platform === 'codex') {
    // Codex is Bottom (First child). Rounded top only if Gemini & Claude are 0
    return (vGemini === 0 && vClaude === 0) ? 'rounded-t-[2px]' : ''
  }
  
  return ''
}

// 展示的图表数据 (最近30天)
const chartData = computed(() => {
  return props.data.slice(-30)
})

// 日期标签逻辑 (显示日期数字)
const shouldShowDate = (index: number, dateStr: string): boolean => {
  // 每隔两天显示一次，或者月初，或者最后一天
  if (isMonthStart(index, dateStr)) return true
  if (index === chartData.value.length - 1) return true
  
  const date = new Date(dateStr)
  return date.getDate() % 2 === 0
}

const getDayLabel = (dateStr: string): string => {
  const date = new Date(dateStr)
  return date.getDate().toString()
}

// 月份开始逻辑
const isMonthStart = (index: number, dateStr: string): boolean => {
  if (index === 0) return true
  const curr = new Date(dateStr)
  const prev = new Date(chartData.value[index - 1].date)
  return curr.getMonth() !== prev.getMonth()
}

const formatMonthLabel = (dateStr: string): string => {
  const date = new Date(dateStr)
  return date.toLocaleString('en-US', { month: 'short' })
}

const formatDateFull = (dateStr: string): string => {
  const date = new Date(dateStr)
  return date.toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' })
}

// 格式化数值
const formatValue = (value: number): string => {
  if (props.viewMode === 'duration') {
    if (value < 60) return `${value}s`
    if (value < 3600) return `${Math.floor(value / 60)}m`
    return `${(value / 3600).toFixed(1)}h`
  }
  return value.toLocaleString()
}

// Tooltip 位置
const tooltipStyle = computed(() => {
  if (hoveredIndex.value === null) return {}
  const count = chartData.value.length
  // 简单的位置计算，始终跟随索引
  // 为了防止溢出，可以在左右边缘做检测，这里简化居中处理
  const percent = (hoveredIndex.value / (count - 1)) * 100
  
  // 边界处理
  let left = `${percent}%`
  let transform = 'translateX(-50%)'
  
  if (hoveredIndex.value < 2) {
    left = '0%'
    transform = 'translateX(0)'
  } else if (hoveredIndex.value > count - 3) {
    left = '100%'
    transform = 'translateX(-100%)'
  }

  return {
    left,
    transform,
    bottom: '80%' // 显示在柱子上方
  }
})
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(4px) translateX(-50%);
}
/* Adjust transforms for edge cases if needed in CSS, 
   but inline styles override mostly. */
</style>
