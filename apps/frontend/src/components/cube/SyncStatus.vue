<script setup lang="ts">
import { useTimerStore } from '../../stores/timer'
const timer = useTimerStore()
</script>

<template>
  <div v-if="timer.isSynced && !timer.isRunning" class="w-full flex flex-col items-center gap-[1vh] bg-indigo-500/5 rounded-[2vmin] border border-indigo-500/20 p-[2vmin]">
    <div class="flex items-center gap-[1vw]">
      <span class="text-[2vmin]">✨</span>
      <span class="text-[1.8vmin] font-medium text-slate-300">Cube Synchronized</span>
    </div>
    
    <div v-if="!timer.isHandheld && timer.useGyroTiming" class="text-[1.5vmin] text-slate-500 animate-pulse">
      Pick up the cube to begin inspection...
    </div>

    <!-- Alert if mixed on connect (Concept) -->
    <div v-if="false" class="p-[2vmin] bg-orange-500/10 border border-orange-500/20 rounded-[1.5vmin] text-orange-200 text-[1.4vmin]">
      ⚠️ Cube is mixed. Please solve or follow the reset guide.
    </div>
  </div>
</template>
