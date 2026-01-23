<script setup lang="ts">
import { useTimerStore } from '../../stores/timer'

const timer = useTimerStore()
</script>

<template>
  <div class="relative group">
    <div class="absolute -inset-1 bg-gradient-to-r from-indigo-500 to-cyan-500 rounded-full blur opacity-25 group-hover:opacity-40 transition duration-1000"></div>
    <div class="relative px-20 py-12 bg-slate-900/80 backdrop-blur-xl rounded-3xl border border-white/5 flex flex-col items-center">
      <div class="text-8xl md:text-9xl font-mono font-light tracking-tighter tabular-nums text-slate-50">
        {{ timer.formattedTime }}
      </div>
      <div class="mt-4 flex flex-col items-center gap-2">
        <div class="text-slate-500 uppercase tracking-widest text-xs font-bold">
          {{ timer.isRunning ? 'Solving...' : 'Ready to flow' }}
        </div>
        <div v-if="timer.useGyroTiming" 
             :class="['text-[10px] px-2 py-0.5 rounded-full border transition-colors', 
                      timer.isHandheld ? 'border-indigo-500 text-indigo-400' : 'border-slate-700 text-slate-500']">
          {{ timer.isHandheld ? 'Picked Up' : 'On Table' }}
        </div>
      </div>
    </div>
  </div>
</template>
