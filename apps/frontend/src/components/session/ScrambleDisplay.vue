<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useSessionStore } from '../../stores/session'
import { useTimerStore } from '../../stores/timer'

const sessionStore = useSessionStore()
const timer = useTimerStore()

const isWCAMode = computed(() => sessionStore.activeSession?.session_type === 'WCA')
const ss = computed(() => timer.scrambleState)

/** For WCA: remaining ms for current move (timeout - elapsed). Negative = overtime. */
const moveRemainingMs = computed(() => {
    if (!ss.value.move_timeout_ms) return null
    return ss.value.move_timeout_ms - (ss.value.move_elapsed_ms || 0)
})

function formatMoveTime(ms: number): string {
    const sec = Math.abs(ms) / 1000
    return sec.toFixed(1)
}

/** Green blink on every accepted scramble move in WCA mode (including each half of D2) */
const showMoveBlink = ref(false)
let blinkTimeout: ReturnType<typeof setTimeout> | null = null

watch(() => ss.value.accepted_count, (newCount, oldCount) => {
    if (isWCAMode.value && newCount > (oldCount ?? 0)) {
        showMoveBlink.value = true
        if (blinkTimeout) clearTimeout(blinkTimeout)
        blinkTimeout = setTimeout(() => {
            showMoveBlink.value = false
        }, 200)
    }
})
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
      <template v-if="isWCAMode">SCRAMBLE INVALIDATED</template>
      <template v-else>
        Solve your cube to restart
        <div class="text-[1.5vmin] text-rose-400/60 mt-[1vh] font-normal">Too many wrong moves</div>
      </template>
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
          <!-- Per-move countdown for correction -->
          <div v-if="moveRemainingMs != null"
               :class="['text-[2.5vmin] font-mono mt-[0.5vh]',
                        moveRemainingMs > 3000 ? 'text-slate-500' :
                        moveRemainingMs > 0 ? 'text-amber-400' : 'text-rose-400 animate-pulse']">
            {{ formatMoveTime(moveRemainingMs) }}s
          </div>
        </div>
        <div v-else-if="ss.expected_move" class="relative">
          <!-- Green blink overlay on accepted move -->
          <div v-if="showMoveBlink"
               class="text-[8vmin] text-emerald-400 font-black animate-pulse">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-[8vmin] h-[8vmin] inline-block" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="3">
              <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
            </svg>
          </div>
          <!-- Normal expected move display -->
          <div v-else class="text-[8vmin] text-indigo-400 font-black">
            {{ ss.expected_move }}
            <!-- Per-move countdown -->
            <div v-if="moveRemainingMs != null"
                 :class="['text-[2.5vmin] font-mono mt-[0.5vh]',
                          moveRemainingMs > 3000 ? 'text-slate-500' :
                          moveRemainingMs > 0 ? 'text-amber-400' : 'text-rose-400 animate-pulse']">
              {{ formatMoveTime(moveRemainingMs) }}s
            </div>
          </div>
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

    <!-- Progress indicator + error count (error count WCA only) -->
    <div v-if="ss.total > 0 && timer.flowState === 'Scrambling'" class="mt-[1vh] flex items-center justify-center gap-[2vmin] text-[1.5vmin] uppercase font-bold tracking-widest">
      <span class="text-slate-600">Move {{ ss.index }} of {{ ss.total }}</span>
      <span v-if="isWCAMode && ss.mistake_count > 0" class="text-rose-400">{{ ss.mistake_count }} error{{ ss.mistake_count !== 1 ? 's' : '' }}</span>
    </div>
  </div>
</template>
