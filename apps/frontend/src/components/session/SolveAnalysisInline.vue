<script setup lang="ts">
import { computed } from 'vue'
import { useSessionStore } from '../../stores/session'

const sessionStore = useSessionStore()

const solve = computed(() => sessionStore.selectedSolve)

function formatTime(ms: number): string {
    return (ms / 1000).toFixed(2)
}

function formatDate(timestamp: number): string {
    return new Date(timestamp).toLocaleString()
}

function tps(solve: any): string {
    if (!solve || solve.time === 0) return '0.0'
    return (solve.moves.length / (solve.time / 1000)).toFixed(1)
}
</script>

<template>
  <div v-if="solve" class="w-full p-[2vmin] bg-slate-900/50 rounded-[2vmin] border border-indigo-500/20 backdrop-blur-sm">
    <div class="flex items-start justify-between mb-[1.5vh]">
      <div>
        <div class="text-[5vmin] font-mono font-bold text-slate-50 leading-none">{{ formatTime(solve.time) }}</div>
        <div class="text-[1.3vmin] text-slate-500 mt-[0.5vh]">
          {{ solve.moves.length }} moves &middot; {{ tps(solve) }} TPS &middot; {{ formatDate(solve.date) }}
        </div>
      </div>
      <button
        @click="sessionStore.selectSolve(null)"
        class="text-slate-500 hover:text-slate-300 transition-colors p-[0.5vmin]"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-[2.5vmin] h-[2.5vmin]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <!-- Scramble -->
    <div v-if="solve.scramble" class="mb-[1.5vh]">
      <div class="text-[1.2vmin] text-slate-500 font-bold uppercase tracking-wider mb-[0.5vh]">Scramble</div>
      <div class="text-[1.8vmin] font-mono text-slate-400 tracking-wide">{{ solve.scramble }}</div>
    </div>

    <!-- Move list -->
    <div>
      <div class="text-[1.2vmin] text-slate-500 font-bold uppercase tracking-wider mb-[0.5vh]">Solution</div>
      <div class="flex flex-wrap gap-[0.5vmin]">
        <span
          v-for="(move, idx) in solve.moves"
          :key="idx"
          class="text-[1.5vmin] font-mono text-indigo-300 px-[0.6vmin] py-[0.2vh] bg-indigo-500/5 rounded-[0.3vmin]"
        >{{ move }}</span>
      </div>
    </div>
  </div>
</template>
