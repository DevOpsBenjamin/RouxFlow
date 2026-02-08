<script setup lang="ts">
import TimerDisplay from '../timer/TimerDisplay.vue'
import SplitStats from '../session/SplitStats.vue'
import MoveList from '../session/MoveList.vue'
import SyncStatus from '../cube/SyncStatus.vue'
import SessionPicker from '../session/SessionPicker.vue'
import ScrambleDisplay from '../session/ScrambleDisplay.vue'
import { useTimerStore } from '../../stores/timer'

const timer = useTimerStore()
</script>

<template>
  <div class="w-full h-full flex flex-col p-[2vmin] overflow-hidden gap-[2vh]">

    <!-- Session Content -->
    <div class="flex-1 flex flex-col overflow-hidden gap-[2vh] animate-in fade-in duration-1000">
      <!-- Top Bar -->
      <div class="flex justify-between items-center h-[8vh] px-[2vw]">
        <SessionPicker class="scale-[1.2] origin-left" />
      </div>

      <!-- Main Content Area: Responsive Grid -->
      <div class="flex-1 grid grid-cols-1 lg:grid-cols-4 gap-[2vmin] px-[2vw] overflow-hidden">
        <!-- Left Sidebar -->
        <aside class="hidden lg:flex flex-col gap-[2vh] bg-slate-900/30 rounded-[2vmin] p-[2vmin] border border-slate-800/50">
          <h3 class="text-[1.5vmin] text-slate-500 uppercase font-black tracking-widest">Session Stats</h3>
          <div class="flex-1 overflow-y-auto">
            <div class="text-[2vmin] text-slate-300">Ao5: 8.42</div>
          </div>
        </aside>

        <!-- Center Logic: Scramble + Timer -->
        <main class="lg:col-span-2 flex flex-col items-center justify-between py-[4vh] gap-[4vh]">
          <div class="w-full">
            <!-- Idle + not solved: prompt user -->
            <div v-if="timer.flowState === 'Idle' && !timer.isCubeSolved"
                 class="w-full text-center p-[2vmin] bg-amber-500/10 rounded-[2vmin] border border-amber-500/20">
              <div class="text-amber-400 text-[3vmin] font-bold">Solve your cube to start</div>
              <div class="text-amber-500/60 text-[1.5vmin] mt-[1vh]">The cube must be in solved state to begin a scramble</div>
            </div>

            <!-- Scrambling / Inspection: show scramble display -->
            <ScrambleDisplay
              v-else-if="timer.flowState === 'Scrambling' || timer.flowState === 'Inspection' || timer.flowState === 'Idle'"
              class="transform transition-all"
            />
          </div>

          <div class="flex-1 flex flex-col items-center justify-center">
            <TimerDisplay class="text-[15vmin] leading-none" />
            <SyncStatus class="mt-[2vh] scale-[1.5]" />
          </div>
        </main>

        <!-- Right Sidebar: Splits & Moves -->
        <aside class="flex flex-col gap-[2vh] bg-slate-900/30 rounded-[2vmin] p-[2vmin] border border-slate-800/50 overflow-hidden">
          <div class="flex-1 flex flex-col gap-[2vh] overflow-hidden">
            <div class="h-[40%] overflow-hidden">
               <h3 class="text-[1.5vmin] text-slate-500 uppercase font-black tracking-widest mb-2">Splits</h3>
               <SplitStats :phases="['FB', 'SB', 'CMLL', 'LSE']" class="h-full" />
            </div>
            <div class="flex-1 overflow-hidden">
               <h3 class="text-[1.5vmin] text-slate-500 uppercase font-black tracking-widest mb-2">Live Moves</h3>
               <MoveList class="h-full" />
            </div>
          </div>
        </aside>
      </div>
    </div>

  </div>
</template>
