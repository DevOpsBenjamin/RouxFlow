<script setup lang="ts">
import { useTimerStore } from '../../stores/timer'
const timer = useTimerStore()
</script>

<template>
  <div class="w-full h-full flex flex-wrap content-start gap-[0.5vmin] p-[1vmin] overflow-y-auto">
    <div v-if="timer.currentMoves.length === 0" class="text-slate-600 italic text-[1.2vmin] w-full text-center py-[2vh]">
      No moves detected...
    </div>
    <span v-for="(move, idx) in timer.currentMoves" :key="idx" 
          class="text-[1.8vmin] font-mono text-indigo-300 px-[0.8vmin] py-[0.2vh] bg-indigo-500/5 rounded-[0.5vmin] animate-in fade-in zoom-in duration-200">
      {{ move }}
    </span>
  </div>
</template>
