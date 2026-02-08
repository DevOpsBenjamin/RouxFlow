<script setup lang="ts">
import { useTimerStore } from '../../stores/timer'
const timer = useTimerStore()
</script>

<template>
  <div v-if="timer.isCubeSolved && !timer.isRunning" class="w-full flex flex-col items-center gap-[1vh] bg-indigo-500/5 rounded-[2vmin] border border-indigo-500/20 p-[2vmin]">
    <div class="flex items-center gap-[1vw]">
      <span class="text-[2vmin]">✨</span>
      <span class="text-[1.8vmin] font-medium text-slate-300">Cube Synchronized</span>
    </div>
  </div>
</template>
