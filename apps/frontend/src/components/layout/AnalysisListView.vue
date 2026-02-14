<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useSessionStore } from '../../stores/session'

const router = useRouter()
const sessionStore = useSessionStore()

function formatTime(ms: number, penalty?: string | null): string {
    if (penalty === 'DNF') return 'DNF'
    const base = (ms / 1000).toFixed(2)
    if (penalty === '+2') return base + '+'
    return base
}

function openSolve(id: string) {
    router.push({ name: 'AnalysisDetail', params: { solveId: id } })
}
</script>

<template>
  <div class="w-full max-w-3xl mx-auto p-[4vmin]">
    <div class="flex items-center justify-between mb-[3vh]">
      <h1 class="text-[4vmin] font-black text-white tracking-tight">Analysis</h1>
      <button
        @click="router.push({ name: 'Session' })"
        class="text-slate-500 hover:text-slate-300 transition-colors text-[1.5vmin] font-medium"
      >
        Back to Session
      </button>
    </div>

    <!-- Session name -->
    <div v-if="sessionStore.activeSession" class="mb-[2vh]">
      <span class="text-[1.3vmin] text-slate-500 uppercase tracking-wider font-bold">
        {{ sessionStore.activeSession.name }}
      </span>
      <span class="text-[1.2vmin] text-slate-600 ml-[1vmin]">
        {{ sessionStore.solveList.length }} solve{{ sessionStore.solveList.length !== 1 ? 's' : '' }}
      </span>
    </div>

    <!-- Solve list -->
    <div v-if="sessionStore.solveList.length > 0" class="space-y-[0.8vmin]">
      <button
        v-for="entry in sessionStore.solveList"
        :key="entry.id"
        @click="openSolve(entry.id)"
        class="w-full flex items-center gap-[2vmin] p-[1.5vmin] bg-slate-900/50 rounded-[1.5vmin] border border-slate-800/50 hover:border-indigo-500/30 hover:bg-slate-900/70 transition-all text-left group"
      >
        <!-- Index -->
        <span class="text-[1.3vmin] text-slate-600 font-mono w-[3vmin] text-right shrink-0">
          {{ entry.index }}
        </span>

        <!-- Time -->
        <span
          class="text-[2.5vmin] font-mono font-bold min-w-[10vmin]"
          :class="entry.penalty === 'DNF' ? 'text-red-400' : 'text-slate-100'"
        >
          {{ formatTime(entry.time_ms, entry.penalty) }}
        </span>

        <!-- Penalty badge -->
        <span v-if="entry.penalty === '+2'" class="text-[1.1vmin] bg-amber-500/15 text-amber-400 px-[0.6vmin] py-[0.2vh] rounded font-bold">+2</span>
        <span v-if="entry.penalty === 'DNF'" class="text-[1.1vmin] bg-red-500/15 text-red-400 px-[0.6vmin] py-[0.2vh] rounded font-bold">DNF</span>

        <!-- Move count + TPS -->
        <span class="text-[1.2vmin] text-slate-500 font-mono ml-auto">
          {{ entry.turns }}m / {{ entry.tps.toFixed(1) }} tps
        </span>

        <!-- Arrow -->
        <svg class="w-[1.5vmin] h-[1.5vmin] text-slate-700 group-hover:text-indigo-400 transition-colors shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
        </svg>
      </button>
    </div>

    <!-- Empty state -->
    <div v-else class="text-center py-[10vh]">
      <div class="text-[3vmin] text-slate-700 mb-[1vh]">No solves yet</div>
      <p class="text-[1.3vmin] text-slate-600">Complete some solves in the Timer to see analysis here.</p>
    </div>
  </div>
</template>
