<script setup lang="ts">
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
</script>

<template>
  <div class="w-full h-full flex flex-col p-[2vmin] overflow-hidden gap-[2vh]">

    <!-- Session Content -->
    <div class="flex-1 flex flex-col overflow-hidden gap-[2vh] animate-in fade-in duration-1000">
      <!-- Top Bar -->
      <div class="flex justify-between items-center h-[8vh] px-[2vw]">
        <SessionPicker class="scale-[1.2] origin-left" />
      </div>

      <!-- Main Content Area: 3-Column Grid -->
      <div class="flex-1 grid grid-cols-1 lg:grid-cols-4 gap-[2vmin] px-[2vw] overflow-hidden">
        <!-- Left Sidebar: Session Stats -->
        <aside class="hidden lg:flex flex-col bg-slate-900/30 rounded-[2vmin] p-[2vmin] border border-slate-800/50 overflow-hidden">
          <h3 class="text-[1.5vmin] text-slate-500 uppercase font-black tracking-widest mb-[1.5vh] shrink-0">Session Stats</h3>
          <SessionStatsCard />
        </aside>

        <!-- Center: Scramble + Timer + Analysis -->
        <main class="lg:col-span-2 flex flex-col items-center justify-between py-[2vh] gap-[3vh] overflow-hidden">
          <div class="w-full shrink-0">
            <!-- Idle + not solved: prompt user -->
            <div v-if="timer.flowState === 'Idle' && !timer.isCubeSolved"
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
          <div v-if="timer.flowState === 'Solving'" class="w-full shrink-0 max-h-[15vh]">
            <h3 class="text-[1.3vmin] text-slate-500 uppercase font-black tracking-widest mb-[0.5vh]">Live Moves</h3>
            <MoveList class="max-h-[12vh]" />
          </div>

          <!-- Inline solve analysis (when a solve is selected and not actively solving) -->
          <SolveAnalysisInline
            v-if="sessionStore.selectedSolveId && timer.flowState !== 'Solving' && timer.flowState !== 'Inspection'"
            class="w-full shrink-0"
          />
        </main>

        <!-- Right Sidebar: Solve List -->
        <aside class="flex flex-col bg-slate-900/30 rounded-[2vmin] p-[2vmin] border border-slate-800/50 overflow-hidden">
          <SolveList />
        </aside>
      </div>
    </div>

  </div>
</template>
