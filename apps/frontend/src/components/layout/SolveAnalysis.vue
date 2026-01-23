<script setup lang="ts">
import { useUIStore } from '../../stores/ui'
import { useTimerStore } from '../../stores/timer'
import { computed } from 'vue'

const props = defineProps<{
  solveId?: string
}>()

const ui = useUIStore()
const timer = useTimerStore()

// Use real data from the timer store for the last solve
const solve = computed(() => ({
  id: props.solveId || 'current',
  time: timer.time,
  moves: timer.currentMoves,
  date: Date.now(),
  // Phase data will come from the Rust core in the future
  phases: {
    FB: 0,
    SB: 0,
    CMLL: 0,
    LSE: 0
  }
}))

function formatTime(ms: number) {
  return (ms / 1000).toFixed(2)
}
</script>

<template>
  <div class="w-full max-w-[80vw] bg-slate-900/50 rounded-[4vmin] border border-slate-800 p-[5vmin] space-y-[4vh] animate-in fade-in slide-in-from-bottom-4 duration-700">
    <div class="flex justify-between items-start">
      <div>
        <h2 class="text-[5vmin] font-black italic text-white tracking-tight leading-none">SOLVE ANALYSIS</h2>
        <p class="text-slate-500 font-mono text-[1.5vmin] mt-[1vh] uppercase tracking-widest">{{ solve.id }}</p>
      </div>
      <button 
        @click="ui.setActiveSession()"
        class="p-[2vmin] bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-[2vmin] transition-all text-[2vmin]"
      >
        ← Back to Session
      </button>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-[4vmin]">
      <!-- Summary Card -->
      <div class="bg-indigo-600/10 border border-indigo-500/20 rounded-[3vmin] p-[4vmin] flex flex-col items-center justify-center gap-[2vh]">
        <p class="text-[1.5vmin] text-indigo-400 font-bold uppercase tracking-widest">Final Time</p>
        <p class="text-[12vmin] font-black text-white italic leading-none tabular-nums">{{ formatTime(solve.time) }}s</p>
      </div>

      <!-- Phase Breakdown -->
      <div class="space-y-[2vh]">
        <h3 class="text-[1.5vmin] text-slate-500 font-bold uppercase tracking-widest border-b border-slate-800 pb-[1vh]">Roux Breakdown</h3>
        <div class="grid gap-[1.5vmin]">
          <div v-for="(time, phase) in solve.phases" :key="phase" class="flex justify-between items-center bg-slate-800/30 p-[2vmin] rounded-[1.5vmin]">
            <span class="font-bold text-slate-200 text-[2vmin]">{{ phase }}</span>
            <span class="font-mono text-indigo-400 text-[2.5vmin]">
              {{ time > 0 ? formatTime(time) + 's' : '--' }}
            </span>
          </div>
        </div>
        <p v-if="Object.values(solve.phases).every(t => t === 0)" class="text-[1.2vmin] text-slate-600 italic text-center pt-2">
          Phase detection requires Cube State Tracking (coming soon)
        </p>
      </div>
    </div>

    <!-- Move Sequence -->
    <div class="space-y-[2vh]">
      <h3 class="text-[1.5vmin] text-slate-500 font-bold uppercase tracking-widest border-b border-slate-800 pb-[1vh]">Move Sequence ({{ solve.moves.length }} moves)</h3>
      <div v-if="solve.moves.length > 0" class="flex flex-wrap gap-[0.8vmin] font-mono">
        <span v-for="(move, i) in solve.moves" :key="i" class="px-[1.5vmin] py-[0.8vmin] bg-slate-800 rounded-[1vmin] text-slate-300 text-[1.8vmin]">
          {{ move }}
        </span>
      </div>
      <div v-else class="text-[1.5vmin] text-slate-600 italic text-center py-[4vh]">
        No moves recorded for this solve.
      </div>
    </div>
  </div>
</template>
