<script setup lang="ts">
import { useTimerStore } from '../../stores/timer'
const timer = useTimerStore()
</script>

<template>
  <div v-if="timer.isSynced && !timer.isRunning" class="w-full max-w-4xl p-4 flex flex-col items-center gap-4 bg-indigo-500/5 rounded-2xl border border-indigo-500/20">
    <div class="flex items-center gap-2">
      <span class="text-indigo-400">✨</span>
      <span class="text-sm font-medium text-slate-300">Cube Synchronized</span>
    </div>
    
    <div v-if="!timer.isHandheld && timer.useGyroTiming" class="text-xs text-slate-500 animate-pulse">
      Pick up the cube to begin inspection...
    </div>

    <!-- Alert if mixed on connect (Concept) -->
    <div v-if="false" class="p-4 bg-orange-500/10 border border-orange-500/20 rounded-xl text-orange-200 text-xs">
      ⚠️ Cube is mixed. Please solve or follow the reset guide.
    </div>
  </div>
</template>
