<script setup lang="ts">
import { computed } from 'vue'
import { useTimerStore } from '../../stores/timer'

const props = withDefaults(defineProps<{
    compact?: boolean
}>(), {
    compact: false
})

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
    <!-- Glow effect (only in full mode) -->
    <div v-if="!compact" class="absolute -inset-[0.2vmin] bg-gradient-to-r from-indigo-500 to-cyan-500 rounded-[5vmin] blur opacity-25 group-hover:opacity-40 transition duration-1000"></div>
    <div :class="[
      'relative flex flex-col items-center transition-all duration-300',
      compact
        ? 'px-[4vmin] py-[2vmin]'
        : 'px-[10vmin] py-[6vmin] bg-slate-900/80 backdrop-blur-xl rounded-[4vmin] border border-white/5'
    ]">

      <!-- Inspection countdown -->
      <template v-if="timer.flowState === 'Inspection'">
        <div :class="['font-mono font-black tracking-tighter tabular-nums leading-none transition-colors duration-300', inspectionColor, compact ? 'text-[8vmin]' : 'text-[15vmin]']">
          {{ inspectionSeconds }}
        </div>
        <div class="mt-[1vh]">
          <span class="text-slate-500 uppercase tracking-widest text-[1.5vmin] font-bold">
            Inspection
          </span>
        </div>
      </template>

      <!-- Solve timer -->
      <template v-else>
        <div :class="['font-mono tracking-tighter tabular-nums text-slate-50 leading-none', compact ? 'text-[6vmin] font-light' : 'text-[15vmin] font-light']">
          {{ timer.formattedTime }}
        </div>
        <div v-if="!compact || timer.flowState === 'Solving'" class="mt-[1vh] flex items-center justify-center gap-[1vmin]">
          <span class="text-slate-500 uppercase tracking-widest text-[1.5vmin] font-bold">
            {{ timer.flowState === 'Solving' ? 'Solving...' :
               timer.flowState === 'Summary' ? 'Complete' :
               timer.flowState === 'Scrambling' ? 'Scrambling...' :
               'Ready to flow' }}
          </span>
        </div>
      </template>
    </div>
  </div>
</template>
