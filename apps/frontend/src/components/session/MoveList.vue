<script setup lang="ts">
import { useTimerStore } from '../../stores/timer'
const timer = useTimerStore()
</script>

<template>
  <div class="w-full max-w-4xl p-4 min-h-16 flex flex-wrap gap-2 items-center justify-center bg-white/3 rounded-2xl border border-white/5">
    <div v-if="timer.currentMoves.length === 0" class="text-slate-600 italic text-sm">
      No moves yet...
    </div>
    <span v-for="(move, idx) in timer.currentMoves" :key="idx" 
          class="text-xl font-mono text-indigo-300 animate-in fade-in zoom-in duration-200">
      {{ move }}
    </span>
  </div>
</template>
