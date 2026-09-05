<script setup lang="ts">
import { useSessionStore } from '../../stores/session'
const sessionStore = useSessionStore()

defineProps<{
  phases: string[]
}>()
</script>

<template>
  <div class="flex flex-col gap-[2vh] w-full h-full overflow-hidden">
    <!-- Grid of Splits -->
    <div class="grid grid-cols-2 gap-[1vmin]">
      <div v-for="phase in phases" :key="phase" 
           class="p-[1.5vmin] rounded-[1.5vmin] bg-white/5 border border-white/5 hover:border-white/10 transition-colors">
        <div class="text-[1.2vmin] text-slate-500 font-bold mb-[0.2vh]">{{ phase }}</div>
        <div class="text-[2vmin] font-mono text-slate-300">--.--</div>
      </div>
    </div>

    <!-- Session Summary Mini Card -->
    <div class="p-[2vmin] rounded-[2vmin] bg-indigo-500/5 border border-indigo-500/10 flex flex-col justify-between gap-[1vh]">
      <div class="flex justify-between items-baseline">
        <span class="text-slate-500 text-[1.2vmin]">Best</span>
        <span class="text-[2vmin] font-mono text-slate-200">--.--</span>
      </div>
      <div class="flex justify-between items-baseline">
        <span class="text-slate-500 text-[1.2vmin]">Ao5</span>
        <span class="text-[2vmin] font-mono text-slate-200">--.--</span>
      </div>
      <div class="mt-auto text-[1vmin] text-slate-600 font-bold uppercase tracking-widest text-center">
        {{ sessionStore.activeSession?.solves.length }} / {{ sessionStore.activeSession?.session_type === 'WCA' ? '5' : '∞' }} Solves
      </div>
    </div>
  </div>
</template>
