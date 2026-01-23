<script setup lang="ts">
import { useTimerStore } from '../../stores/timer'

const timer = useTimerStore()
</script>

<template>
  <div class="relative group">
    <div class="absolute -inset-[0.2vmin] bg-gradient-to-r from-indigo-500 to-cyan-500 rounded-[5vmin] blur opacity-25 group-hover:opacity-40 transition duration-1000"></div>
    <div class="relative px-[10vmin] py-[6vmin] bg-slate-900/80 backdrop-blur-xl rounded-[4vmin] border border-white/5 flex flex-col items-center">
      <div class="text-[15vmin] font-mono font-light tracking-tighter tabular-nums text-slate-50 leading-none">
        {{ timer.formattedTime }}
      </div>
      <div class="mt-[2vh] flex flex-col items-center gap-[1vh]">
        <div class="text-slate-500 uppercase tracking-widest text-[1.5vmin] font-bold">
          {{ timer.isRunning ? 'Solving...' : 'Ready to flow' }}
        </div>
        <div v-if="timer.useGyroTiming" 
             :class="['text-[1.2vmin] px-[2vmin] py-[0.5vh] rounded-full border transition-colors', 
                      timer.isHandheld ? 'border-indigo-500 text-indigo-400' : 'border-slate-700 text-slate-500']">
          {{ timer.isHandheld ? 'Picked Up' : 'On Table' }}
        </div>
      </div>
    </div>
  </div>
</template>
