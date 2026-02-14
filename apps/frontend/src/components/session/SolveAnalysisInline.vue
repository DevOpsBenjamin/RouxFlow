<script setup lang="ts">
import { computed } from 'vue'
import { useSessionStore } from '../../stores/session'
import { analyzeSolve, type SolveAnalysis, type StepSegment } from '../../services/cube/bridge'

const sessionStore = useSessionStore()

const solve = computed(() => sessionStore.selectedSolve)

const analysis = computed<SolveAnalysis | null>(() => {
    const s = solve.value
    if (!s || !s.moves || s.moves.length === 0) return null
    try {
        return analyzeSolve(s)
    } catch {
        return null
    }
})

const stepColors: Record<string, { bg: string; text: string; border: string; label: string }> = {
    FB:   { bg: 'bg-blue-500/10',   text: 'text-blue-300',   border: 'border-blue-500/30',   label: 'FB' },
    SB:   { bg: 'bg-emerald-500/10', text: 'text-emerald-300', border: 'border-emerald-500/30', label: 'SB' },
    CMLL: { bg: 'bg-amber-500/10',  text: 'text-amber-300',  border: 'border-amber-500/30',  label: 'CMLL' },
    LSE:  { bg: 'bg-purple-500/10', text: 'text-purple-300', border: 'border-purple-500/30', label: 'LSE' },
}

// Map each move index to its step color
const moveStepMap = computed<(string | null)[]>(() => {
    const s = solve.value
    const a = analysis.value
    if (!s || !a) return []
    const map: (string | null)[] = new Array(s.moves.length).fill(null)
    for (const step of a.steps) {
        for (let i = step.start_move; i < step.end_move; i++) {
            map[i] = step.step
        }
    }
    return map
})

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

function stepTps(step: StepSegment): string {
    if (!step.time_ms || step.time_ms === 0 || step.move_count === 0) return '-'
    return (step.move_count / (step.time_ms / 1000)).toFixed(1)
}

function moveClasses(idx: number): string[] {
    const step = moveStepMap.value[idx]
    if (step && stepColors[step]) {
        return [stepColors[step].bg, stepColors[step].text]
    }
    return ['bg-slate-500/5', 'text-slate-500']
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

    <!-- Step Analysis -->
    <div v-if="analysis && analysis.steps.length > 0" class="mb-[1.5vh]">
      <div class="text-[1.2vmin] text-slate-500 font-bold uppercase tracking-wider mb-[0.8vh]">
        Steps
        <span v-if="analysis.orientation" class="text-slate-600 font-normal normal-case ml-[1vmin]">{{ analysis.orientation }}</span>
      </div>
      <div class="flex gap-[0.8vmin] flex-wrap">
        <div
          v-for="step in analysis.steps"
          :key="step.step"
          class="flex items-center gap-[0.6vmin] px-[1vmin] py-[0.4vh] rounded-[0.5vmin] border"
          :class="[stepColors[step.step]?.bg, stepColors[step.step]?.border]"
        >
          <span class="text-[1.3vmin] font-bold" :class="stepColors[step.step]?.text">{{ stepColors[step.step]?.label }}</span>
          <span class="text-[1.2vmin] text-slate-400">{{ step.move_count }}m</span>
          <span v-if="step.time_ms != null && step.time_ms > 0" class="text-[1.2vmin] text-slate-500">{{ (step.time_ms / 1000).toFixed(2) }}s</span>
          <span class="text-[1.1vmin] text-slate-600">{{ stepTps(step) }} tps</span>
        </div>
      </div>
    </div>

    <!-- Move list with step colors -->
    <div>
      <div class="text-[1.2vmin] text-slate-500 font-bold uppercase tracking-wider mb-[0.5vh]">Solution</div>
      <div class="flex flex-wrap gap-[0.5vmin]">
        <span
          v-for="(move, idx) in solve.moves"
          :key="idx"
          class="text-[1.5vmin] font-mono px-[0.6vmin] py-[0.2vh] rounded-[0.3vmin]"
          :class="moveClasses(idx as number)"
        >{{ move }}</span>
      </div>
    </div>
  </div>
</template>
