<script setup lang="ts">
import { computed } from 'vue'
import TimerDisplay from '../timer/TimerDisplay.vue'
import MoveList from '../session/MoveList.vue'
import SessionPicker from '../session/SessionPicker.vue'
import ScrambleDisplay from '../session/ScrambleDisplay.vue'
import SessionStatsCard from '../session/SessionStatsCard.vue'
import SolveList from '../session/SolveList.vue'
import SolveAnalysisInline from '../session/SolveAnalysisInline.vue'
import { useTimerStore } from '../../stores/timer'
import { useSessionStore } from '../../stores/session'

const timer = useTimerStore()
const sessionStore = useSessionStore()

const wcaAo5Display = computed(() => {
    const stats = sessionStore.sessionStats
    if (!stats?.wca_complete) return null
    if (stats.current_ao5_ms != null) {
        return (stats.current_ao5_ms / 1000).toFixed(2)
    }
    return 'DNF'
})
</script>

<template>
  <div class="w-full h-full flex p-[2vmin] overflow-hidden gap-[2vmin]">

    <!-- Left Sidebar: Session Picker + Stats -->
    <aside class="hidden lg:flex flex-col w-[20vw] min-w-[200px] max-w-[280px] bg-slate-900/30 rounded-[2vmin] p-[2vmin] border border-slate-800/50 overflow-hidden gap-[1.5vh]">
      <SessionPicker class="shrink-0" />
      <div class="w-full h-px bg-slate-800/50 shrink-0" />
      <SessionStatsCard />
    </aside>

    <!-- Center: Scramble + Timer + Analysis -->
    <main class="flex-1 flex flex-col items-center justify-between py-[2vh] gap-[3vh] overflow-hidden">
      <div class="w-full shrink-0 px-[2vw]">
        <!-- WCA session complete: show Ao5 result -->
        <div v-if="timer.isWcaFull"
             class="w-full text-center p-[4vmin] bg-amber-500/5 rounded-[2vmin] border border-amber-500/20">
          <div class="text-amber-400 text-[2vmin] font-bold uppercase tracking-widest mb-[2vh]">WCA Session Complete</div>
          <div class="text-[6vmin] font-mono font-black"
               :class="wcaAo5Display === 'DNF' ? 'text-rose-400' : 'text-emerald-400'">
            {{ wcaAo5Display }}
          </div>
          <div class="text-slate-500 text-[1.8vmin] mt-[1vh]">Average of 5</div>
          <div class="text-slate-600 text-[1.3vmin] mt-[2vh]">Create a new WCA session to start another round.</div>
        </div>

        <!-- Idle + not solved: prompt user -->
        <div v-else-if="timer.flowState === 'Idle' && !timer.isCubeSolved"
             class="w-full text-center p-[2vmin] bg-amber-500/10 rounded-[2vmin] border border-amber-500/20">
          <div class="text-amber-400 text-[3vmin] font-bold">Solve your cube to start</div>
          <div class="text-amber-500/60 text-[1.5vmin] mt-[1vh]">The cube must be in solved state to begin a scramble</div>
        </div>

        <!-- Scrambling / Inspection / Idle+solved: show scramble display -->
        <ScrambleDisplay
          v-else-if="timer.flowState === 'Scrambling' || timer.flowState === 'Inspection' || timer.flowState === 'Idle'"
          class="transform transition-all"
        />
      </div>

      <!-- Timer -->
      <div class="flex flex-col items-center justify-center"
           :class="timer.flowState === 'Solving' || timer.flowState === 'Inspection' ? 'flex-1' : ''">
        <TimerDisplay :compact="timer.flowState !== 'Solving' && timer.flowState !== 'Inspection'" />
      </div>

      <!-- Live moves during solve -->
      <div v-if="timer.flowState === 'Solving'" class="w-full shrink-0 max-h-[15vh] px-[2vw]">
        <h3 class="text-[1.3vmin] text-slate-500 uppercase font-black tracking-widest mb-[0.5vh]">Live Moves</h3>
        <MoveList class="max-h-[12vh]" />
      </div>

      <!-- Inline solve analysis (when a solve is selected and not actively solving) -->
      <SolveAnalysisInline
        v-if="sessionStore.selectedSolveId && timer.flowState !== 'Solving' && timer.flowState !== 'Inspection'"
        class="w-full shrink-0 px-[2vw]"
      />
    </main>

    <!-- Right Sidebar: Solve List -->
    <aside class="flex flex-col w-[20vw] min-w-[200px] max-w-[280px] bg-slate-900/30 rounded-[2vmin] p-[2vmin] border border-slate-800/50 overflow-hidden">
      <SolveList />
    </aside>

  </div>
</template>
