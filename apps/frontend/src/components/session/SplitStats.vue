<script setup lang="ts">
import { useSessionStore } from '../../stores/session'
const sessionStore = useSessionStore()

defineProps<{
  phases: string[]
}>()
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-2 gap-6 w-full max-w-4xl">
    <!-- Left: Actual Splits -->
    <div class="grid grid-cols-2 gap-4">
      <div v-for="phase in phases" :key="phase" 
           class="p-4 rounded-2xl bg-white/5 border border-white/5 hover:border-white/10 transition-colors">
        <div class="text-xs text-slate-500 font-bold mb-1">{{ phase }}</div>
        <div class="text-xl font-mono text-slate-300">--.--</div>
      </div>
    </div>

    <!-- Right: Session Summary -->
    <div class="p-6 rounded-3xl bg-indigo-500/5 border border-indigo-500/10 flex flex-col justify-between">
      <div>
        <div class="text-xs text-indigo-400 font-bold uppercase tracking-widest mb-4">Session Summary</div>
        <div class="flex justify-between items-baseline mb-2">
          <span class="text-slate-500 text-sm">Best Time</span>
          <span class="text-2xl font-mono text-slate-200">--.--</span>
        </div>
        <div class="flex justify-between items-baseline">
          <span class="text-slate-500 text-sm">Session Ao5</span>
          <span class="text-2xl font-mono text-slate-200">--.--</span>
        </div>
      </div>
      <div class="mt-8 text-[10px] text-slate-600 font-bold uppercase tracking-widest text-center">
        {{ sessionStore.activeSession?.solves.length }} / {{ sessionStore.activeSession?.type === 'WCA' ? '5' : '∞' }} Solves
      </div>
    </div>
  </div>
</template>
