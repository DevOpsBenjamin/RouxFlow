<script setup lang="ts">
import { useSessionStore } from '../../stores/session'
import { deleteSolve } from '../../services/cube/bridge'

const sessionStore = useSessionStore()

function formatTime(ms: number): string {
    return (ms / 1000).toFixed(2)
}

function formatTps(tps: number): string {
    return tps.toFixed(1)
}

async function handleDelete(event: Event, solveId: string) {
    event.stopPropagation()
    await deleteSolve(solveId)
    if (sessionStore.selectedSolveId === solveId) {
        sessionStore.selectSolve(null)
    }
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
      <div class="flex items-center gap-[1.5vmin] px-[1.5vmin] py-[0.5vh] text-[1.5vmin] text-slate-600 font-bold uppercase tracking-wider shrink-0 border-b border-slate-800/50 mb-[0.5vh]">
        <span class="w-[3.5vmin] text-right shrink-0">#</span>
        <span class="flex-1">Time</span>
        <span class="w-[6vmin] text-right">Moves</span>
        <span class="w-[6vmin] text-right">TPS</span>
        <span class="w-[3vmin]"></span>
      </div>

      <!-- Solve rows -->
      <div class="flex-1 overflow-y-auto space-y-[0.3vh]">
        <div
          v-for="entry in sessionStore.solveList"
          :key="entry.id"
          @click="sessionStore.selectSolve(sessionStore.selectedSolveId === entry.id ? null : entry.id)"
          :class="[
            'w-full flex items-center gap-[1.5vmin] px-[1.5vmin] py-[1.2vmin] rounded-[1vmin] text-left transition-colors cursor-pointer',
            sessionStore.selectedSolveId === entry.id
              ? 'bg-indigo-500/15 border border-indigo-500/30'
              : 'bg-white/3 border border-transparent hover:bg-white/5'
          ]"
        >
          <span class="text-[1.6vmin] text-slate-600 font-mono w-[3.5vmin] text-right shrink-0">#{{ entry.index }}</span>
          <span :class="['text-[2.5vmin] font-mono font-bold flex-1', entry.penalty === 'DNF' ? 'text-rose-400' : entry.is_best ? 'text-emerald-400' : 'text-slate-200']">
            {{ entry.penalty === 'DNF' ? 'DNF' : formatTime(entry.time_ms) }}
          </span>
          <span class="text-[1.8vmin] text-slate-400 font-mono w-[6vmin] text-right">{{ entry.penalty === 'DNF' ? '-' : entry.turns }}</span>
          <span class="text-[1.8vmin] text-slate-500 font-mono w-[6vmin] text-right">{{ entry.penalty === 'DNF' ? '-' : formatTps(entry.tps) }}</span>
          <button
            @click="handleDelete($event, entry.id)"
            class="w-[3vmin] h-[3vmin] flex items-center justify-center text-slate-600 hover:text-rose-400 transition-colors rounded shrink-0"
            title="Delete solve"
          >
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-[2vmin] h-[2vmin]">
              <path fill-rule="evenodd" d="M8.75 1A2.75 2.75 0 006 3.75v.443c-.795.077-1.584.176-2.365.298a.75.75 0 10.23 1.482l.149-.022 1.005 11.36A2.75 2.75 0 007.77 20h4.46a2.75 2.75 0 002.751-2.689l1.005-11.36.149.022a.75.75 0 00.23-1.482A41.03 41.03 0 0014 4.193V3.75A2.75 2.75 0 0011.25 1h-2.5zM10 4c.84 0 1.673.025 2.5.075V3.75c0-.69-.56-1.25-1.25-1.25h-2.5c-.69 0-1.25.56-1.25 1.25v.325C8.327 4.025 9.16 4 10 4zM8.58 7.72a.75.75 0 00-1.5.06l.3 7.5a.75.75 0 101.5-.06l-.3-7.5zm4.34.06a.75.75 0 10-1.5-.06l-.3 7.5a.75.75 0 101.5.06l.3-7.5z" clip-rule="evenodd" />
            </svg>
          </button>
        </div>
      </div>
    </template>
  </div>
</template>
