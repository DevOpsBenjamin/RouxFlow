<script setup lang="ts">
import { computed } from 'vue'
import TimerDisplay from '../timer/TimerDisplay.vue'
import SplitStats from '../session/SplitStats.vue'
import MoveList from '../session/MoveList.vue'
import SyncStatus from '../cube/SyncStatus.vue'
import BluetoothManager from '../cube/BluetoothManager.vue'
import SessionPicker from '../session/SessionPicker.vue'
import ScrambleDisplay from '../session/ScrambleDisplay.vue'
import { useTimerStore } from '../../stores/timer'
import { useSessionStore } from '../../stores/session'
import { useUIStore } from '../../stores/ui'

const ui = useUIStore()
const timer = useTimerStore()
const sessionStore = useSessionStore()

const isWCAMode = computed(() => sessionStore.activeSession?.session_type === 'WCA')

useTimerStore()
</script>

<template>
  <div class="w-full h-full flex flex-col p-[2vmin] overflow-hidden gap-[2vh]">
    <!-- Top Bar -->
    <div class="flex justify-between items-center h-[8vh] px-[2vw]">
      <SessionPicker class="scale-[1.2] origin-left" />
      <BluetoothManager class="scale-[1.2] origin-right" />
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
          <ScrambleDisplay 
            scramble="U' R2 U B2 D2 F2 L2 D' R2 F2 U L' F D2 B' D2 R' U B' L F'" 
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

    <!-- Free Mode Decision Overlay -->
    <Transition 
      enter-active-class="transition duration-500 ease-out"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition duration-300 ease-in"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div v-if="timer.flowState === 'Summary' && !isWCAMode" class="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-md flex items-center justify-center p-[5vmin]">
        <div class="bg-slate-900 border border-slate-800 rounded-[5vmin] p-[8vmin] max-w-[60vw] w-full shadow-2xl space-y-[6vh] text-center animate-in fade-in zoom-in duration-500">
           <header>
             <h2 class="text-[5vmin] font-black italic text-white leading-none tracking-tighter uppercase">Solve Finished</h2>
             <p class="text-[1.5vmin] text-slate-500 mt-[1vh] font-bold uppercase tracking-widest">Nice Work!</p>
           </header>
           
           <div class="text-[15vmin] font-light text-indigo-400 leading-none tabular-nums italic">
             {{ timer.formattedTime }}s
           </div>

           <div class="flex gap-[4vmin] justify-center w-full">
             <button 
                @click="ui.openAnalysis('current')" 
                class="flex-1 py-[3vh] px-[4vw] rounded-[3vmin] bg-slate-800 text-slate-300 font-bold text-[3vmin] hover:bg-slate-700 hover:text-white transition-all transform hover:scale-105 active:scale-95"
             >
                Deep Analysis
             </button>
             <button 
                @click="timer.reset()" 
                class="flex-1 py-[3vh] px-[4vw] rounded-[3vmin] bg-indigo-600 text-white font-bold text-[3vmin] shadow-lg shadow-indigo-600/20 hover:bg-indigo-500 transition-all transform hover:scale-105 active:scale-95"
             >
                Next Scramble
             </button>
           </div>
        </div>
      </div>
    </Transition>
  </div>
</template>
