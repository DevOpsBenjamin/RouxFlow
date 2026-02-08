<script setup lang="ts">
import { computed } from 'vue'
import { useSessionStore } from '../../stores/session'
import { useTimerStore } from '../../stores/timer'

const sessionStore = useSessionStore()
const timer = useTimerStore()

const isWCAMode = computed(() => sessionStore.activeSession?.session_type === 'WCA')
const ss = computed(() => timer.scrambleState)
</script>

<template>
  <div class="w-full text-center p-[2vmin] bg-slate-900/30 rounded-[2vmin] border border-white/5 backdrop-blur-sm">
    <div class="text-slate-500 text-[1.5vmin] mb-[1vh] font-bold uppercase tracking-widest flex items-center justify-center gap-[1vw]">
      <span v-if="isWCAMode" class="px-[1vw] py-[0.5vh] bg-amber-500/10 text-amber-500 border border-amber-500/20 rounded-[0.5vmin] text-[1.2vmin]">WCA Integrity Mode</span>
      <span v-else-if="timer.flowState === 'Inspection'">Inspection</span>
      <span v-else>Scramble</span>
    </div>

    <!-- Invalid scramble -->
    <div v-if="ss.is_invalid" class="text-rose-400 font-bold animate-pulse text-[3vmin]">
      SCRAMBLE INVALIDATED
    </div>

    <!-- Inspection countdown -->
    <div v-else-if="timer.flowState === 'Inspection'" class="text-emerald-400 font-bold text-[3vmin]">
      Scramble complete — inspect the cube
    </div>

    <!-- Scrambling: show moves with highlighting -->
    <div v-else-if="ss.total > 0 && timer.flowState !== 'Summary'" class="font-mono transition-all duration-300 transform">
      <!-- WCA mode: one move at a time -->
      <template v-if="isWCAMode">
        <div v-if="ss.correction_move" class="text-[8vmin] text-amber-400 font-black animate-pulse">
          {{ ss.correction_move }}
          <div class="text-[1.5vmin] text-amber-500/60 mt-[1vh]">Undo mistake</div>
        </div>
        <div v-else-if="ss.expected_move" class="text-[8vmin] text-indigo-400 font-black">
          {{ ss.expected_move }}
        </div>
      </template>

      <!-- Free mode: all moves with color coding -->
      <template v-else>
        <div class="tracking-tight leading-tight text-[3vmin] flex flex-wrap justify-center gap-[0.8vmin]">
          <span
            v-for="(move, idx) in ss.scramble"
            :key="idx"
            :class="[
              'transition-colors duration-200 px-[0.3vmin]',
              idx < ss.index ? 'text-emerald-400/70' :
              idx === ss.index && !ss.correction_move ? 'text-white font-bold scale-110' :
              'text-slate-600'
            ]"
          >{{ move }}</span>
        </div>
        <!-- Correction prompt -->
        <div v-if="ss.correction_move" class="mt-[1vh] text-amber-400 text-[2vmin] font-bold animate-pulse">
          Undo: {{ ss.correction_move }}
        </div>
      </template>
    </div>

    <!-- Progress indicator -->
    <div v-if="ss.total > 0 && timer.flowState === 'Scrambling'" class="mt-[1vh] text-[1.5vmin] text-slate-600 uppercase font-bold tracking-widest">
      Move {{ ss.index }} of {{ ss.total }}
    </div>
  </div>
</template>
