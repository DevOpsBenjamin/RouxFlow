<script setup lang="ts">
import { computed } from 'vue'
import { useSessionStore } from '../../stores/session'

const sessionStore = useSessionStore()

const stats = computed(() => sessionStore.sessionStats)

function formatTime(ms: number | null | undefined): string {
    if (ms == null) return '--'
    return (ms / 1000).toFixed(2)
}

function formatTps(tps: number | null | undefined): string {
    if (tps == null) return '--'
    return tps.toFixed(1)
}

function formatCountdown(ms: number | null | undefined): string {
    if (ms == null) return '--:--'
    const totalSec = Math.floor(ms / 1000)
    const min = Math.floor(totalSec / 60)
    const sec = totalSec % 60
    return `${min}:${sec.toString().padStart(2, '0')}`
}
</script>

<template>
  <div class="flex flex-col gap-[1.5vh] h-full overflow-y-auto">
    <!-- Session type badge + solve count -->
    <div class="flex items-center justify-between">
      <span
        :class="[
          'px-[1vw] py-[0.3vh] rounded-[0.5vmin] text-[1.2vmin] font-bold uppercase tracking-wider',
          stats?.session_type === 'WCA'
            ? 'bg-amber-500/10 text-amber-500 border border-amber-500/20'
            : 'bg-indigo-500/10 text-indigo-400 border border-indigo-500/20'
        ]"
      >{{ stats?.session_type || 'Free' }}</span>
      <span class="text-slate-500 text-[1.3vmin] font-bold">
        {{ stats?.solve_count ?? 0 }} solve{{ (stats?.solve_count ?? 0) !== 1 ? 's' : '' }}
      </span>
    </div>

    <!-- WCA countdown (hidden when session is complete) -->
    <div v-if="stats?.session_type === 'WCA' && !stats?.wca_complete" class="p-[1.5vmin] rounded-[1.5vmin] bg-amber-500/5 border border-amber-500/10 space-y-[0.5vh]">
      <div class="flex justify-between items-center">
        <span class="text-[1.1vmin] text-amber-500/60 font-bold uppercase tracking-wider">Time Left</span>
        <span :class="['text-[2vmin] font-mono font-bold', (stats?.wca_remaining_ms ?? 0) < 300000 ? 'text-rose-400' : 'text-amber-400']">
          {{ stats?.wca_remaining_ms != null ? formatCountdown(stats.wca_remaining_ms) : 'Not started' }}
        </span>
      </div>
      <div class="flex justify-between items-center">
        <span class="text-[1.1vmin] text-amber-500/60 font-bold uppercase tracking-wider">Solves Left</span>
        <span :class="['text-[2vmin] font-mono font-bold', (stats?.wca_solves_remaining ?? 0) === 0 ? 'text-rose-400' : 'text-amber-400']">
          {{ stats?.wca_solves_remaining ?? 0 }} / 5
        </span>
      </div>
    </div>
    <!-- WCA complete badge -->
    <div v-else-if="stats?.wca_complete" class="p-[1.5vmin] rounded-[1.5vmin] bg-emerald-500/5 border border-emerald-500/10 text-center">
      <span class="text-[1.3vmin] text-emerald-400 font-bold uppercase tracking-wider">5/5 Complete</span>
    </div>

    <!-- Best single -->
    <div class="p-[1.5vmin] rounded-[1.5vmin] bg-emerald-500/5 border border-emerald-500/10">
      <div class="text-[1.2vmin] text-slate-500 font-bold uppercase tracking-wider">Best</div>
      <div class="text-[2.5vmin] font-mono text-emerald-400 font-bold">{{ formatTime(stats?.best_ms) }}</div>
    </div>

    <!-- Average -->
    <div class="p-[1.5vmin] rounded-[1.5vmin] bg-white/5 border border-white/5">
      <div class="text-[1.2vmin] text-slate-500 font-bold uppercase tracking-wider">Average</div>
      <div class="text-[2.5vmin] font-mono text-slate-300">{{ formatTime(stats?.average_ms) }}</div>
    </div>

    <!-- Ao5 -->
    <div class="grid grid-cols-2 gap-[1vmin]">
      <div class="p-[1.5vmin] rounded-[1.5vmin] bg-white/5 border border-white/5">
        <div class="text-[1.1vmin] text-slate-500 font-bold uppercase tracking-wider">Ao5</div>
        <div class="text-[2vmin] font-mono text-slate-300">{{ formatTime(stats?.current_ao5_ms) }}</div>
      </div>
      <div class="p-[1.5vmin] rounded-[1.5vmin] bg-indigo-500/5 border border-indigo-500/10">
        <div class="text-[1.1vmin] text-indigo-400/60 font-bold uppercase tracking-wider">Best Ao5</div>
        <div class="text-[2vmin] font-mono text-indigo-400">{{ formatTime(stats?.best_ao5_ms) }}</div>
      </div>
    </div>

    <!-- Ao12 -->
    <div class="grid grid-cols-2 gap-[1vmin]">
      <div class="p-[1.5vmin] rounded-[1.5vmin] bg-white/5 border border-white/5">
        <div class="text-[1.1vmin] text-slate-500 font-bold uppercase tracking-wider">Ao12</div>
        <div class="text-[2vmin] font-mono text-slate-300">{{ formatTime(stats?.current_ao12_ms) }}</div>
      </div>
      <div class="p-[1.5vmin] rounded-[1.5vmin] bg-indigo-500/5 border border-indigo-500/10">
        <div class="text-[1.1vmin] text-indigo-400/60 font-bold uppercase tracking-wider">Best Ao12</div>
        <div class="text-[2vmin] font-mono text-indigo-400">{{ formatTime(stats?.best_ao12_ms) }}</div>
      </div>
    </div>

    <!-- TPS -->
    <div class="p-[1.5vmin] rounded-[1.5vmin] bg-white/5 border border-white/5">
      <div class="text-[1.2vmin] text-slate-500 font-bold uppercase tracking-wider">Mean TPS</div>
      <div class="text-[2vmin] font-mono text-slate-300">{{ formatTps(stats?.mean_tps) }}</div>
    </div>
  </div>
</template>
