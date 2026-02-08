<script setup lang="ts">
import { computed } from 'vue'
import { useTimerStore } from '../../stores/timer'

const timer = useTimerStore()

const inspectionSeconds = computed(() => Math.ceil(timer.inspectionRemaining))

const inspectionColor = computed(() => {
    const s = timer.inspectionRemaining
    if (s > 10) return 'text-emerald-400'
    if (s > 5) return 'text-yellow-400'
    return 'text-rose-400'
})
</script>

<template>
  <div class="relative group">
    <div class="absolute -inset-[0.2vmin] bg-gradient-to-r from-indigo-500 to-cyan-500 rounded-[5vmin] blur opacity-25 group-hover:opacity-40 transition duration-1000"></div>
    <div class="relative px-[10vmin] py-[6vmin] bg-slate-900/80 backdrop-blur-xl rounded-[4vmin] border border-white/5 flex flex-col items-center">

      <!-- Inspection countdown -->
      <template v-if="timer.flowState === 'Inspection'">
        <div :class="['text-[15vmin] font-mono font-black tracking-tighter tabular-nums leading-none transition-colors duration-300', inspectionColor]">
          {{ inspectionSeconds }}
        </div>
        <div class="mt-[2vh] text-slate-500 uppercase tracking-widest text-[1.5vmin] font-bold">
          Inspection
        </div>
      </template>

      <!-- Solve timer -->
      <template v-else>
        <div class="text-[15vmin] font-mono font-light tracking-tighter tabular-nums text-slate-50 leading-none">
          {{ timer.formattedTime }}
        </div>
        <div class="mt-[2vh] flex flex-col items-center gap-[1vh]">
          <div class="text-slate-500 uppercase tracking-widest text-[1.5vmin] font-bold">
            {{ timer.flowState === 'Solving' ? 'Solving...' :
               timer.flowState === 'Summary' ? 'Complete' :
               timer.flowState === 'Scrambling' ? 'Scrambling...' :
               'Ready to flow' }}
          </div>
        </div>
      </template>
    </div>
  </div>
</template>
