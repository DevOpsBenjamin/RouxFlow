<script setup lang="ts">
import { useSessionStore } from '../../stores/session'

const sessionStore = useSessionStore()

function formatTime(ms: number): string {
    return (ms / 1000).toFixed(2)
}

function formatTps(tps: number): string {
    return tps.toFixed(1)
}
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">
    <h3 class="text-[1.5vmin] text-slate-500 uppercase font-black tracking-widest mb-[1vh] shrink-0">Solves</h3>

    <div v-if="!sessionStore.solveList.length" class="flex-1 flex items-center justify-center">
      <span class="text-slate-600 italic text-[1.5vmin]">No solves yet</span>
    </div>

    <template v-else>
      <!-- Column headers -->
      <div class="flex items-center gap-[1.5vmin] px-[1.5vmin] py-[0.5vh] text-[1.3vmin] text-slate-600 font-bold uppercase tracking-wider shrink-0 border-b border-slate-800/50 mb-[0.5vh]">
        <span class="w-[3.5vmin] text-right shrink-0">#</span>
        <span class="flex-1">Time</span>
        <span class="w-[5vmin] text-right">Moves</span>
        <span class="w-[5vmin] text-right">TPS</span>
      </div>

      <!-- Solve rows -->
      <div class="flex-1 overflow-y-auto space-y-[0.3vh]">
        <button
          v-for="entry in sessionStore.solveList"
          :key="entry.id"
          @click="sessionStore.selectSolve(sessionStore.selectedSolveId === entry.id ? null : entry.id)"
          :class="[
            'w-full flex items-center gap-[1.5vmin] px-[1.5vmin] py-[1.2vmin] rounded-[1vmin] text-left transition-colors',
            sessionStore.selectedSolveId === entry.id
              ? 'bg-indigo-500/15 border border-indigo-500/30'
              : 'bg-white/3 border border-transparent hover:bg-white/5'
          ]"
        >
          <span class="text-[1.4vmin] text-slate-600 font-mono w-[3.5vmin] text-right shrink-0">#{{ entry.index }}</span>
          <span :class="['text-[2.2vmin] font-mono font-bold flex-1', entry.is_best ? 'text-emerald-400' : 'text-slate-200']">
            {{ formatTime(entry.time_ms) }}
          </span>
          <span class="text-[1.4vmin] text-slate-400 font-mono w-[5vmin] text-right">{{ entry.turns }}</span>
          <span class="text-[1.4vmin] text-slate-500 font-mono w-[5vmin] text-right">{{ formatTps(entry.tps) }}</span>
        </button>
      </div>
    </template>
  </div>
</template>
