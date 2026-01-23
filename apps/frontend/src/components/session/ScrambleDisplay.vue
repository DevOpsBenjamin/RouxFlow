<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useSessionStore } from '../../stores/session'
import { useTimerStore } from '../../stores/timer'
import { sessionManager } from '../../services/cube/bridge'

const props = defineProps<{
  scramble: string
}>()

const sessionStore = useSessionStore()
const timer = useTimerStore()
const isInvalid = ref(false)
const isReady = ref(false)
const scrambleIndex = ref(0)
const scrambleLen = ref(0)

const isWCAMode = computed(() => sessionStore.activeSession?.session_type === 'WCA')

function updateState() {
  if (!sessionManager) return
  isReady.value = sessionManager.is_scramble_ready()
  isInvalid.value = sessionManager.is_scramble_invalid()
  scrambleIndex.value = sessionManager.get_scramble_index()
  scrambleLen.value = sessionManager.get_scramble_len()
}

// Reset when scramble changes
watch(() => props.scramble, () => {
  if (!sessionManager) return
  sessionManager.start_scramble(props.scramble)
  updateState()
})

// Listen to moves from the timer store
watch(() => timer.lastReceivedMove, (m) => {
  if (m && sessionManager) {
    const faceNames = ['U', 'R', 'F', 'D', 'L', 'B']
    const amountStr = m.amount === 1 ? '' : m.amount === -1 ? "'" : '2'
    const moveStr = `${faceNames[m.face]}${amountStr}`
    
    if (isReady.value && timer.flowState === 'Scrambling') {
        timer.startTimer()
    } else {
        sessionManager.handle_scramble_move(moveStr, Date.now() / 1000)
        updateState()
    }
  }
})
</script>

<template>
  <div class="w-full text-center p-[2vmin] bg-slate-900/30 rounded-[2vmin] border border-white/5 backdrop-blur-sm">
    <div class="text-slate-500 text-[1.5vmin] mb-[1vh] font-bold uppercase tracking-widest flex items-center justify-center gap-[1vw]">
      <span v-if="isWCAMode" class="px-[1vw] py-[0.5vh] bg-amber-500/10 text-amber-500 border border-amber-500/20 rounded-[0.5vmin] text-[1.2vmin]">WCA Integrity Mode</span>
      <span v-else>Next Scramble</span>
    </div>

    <div v-if="isInvalid" class="text-rose-400 font-bold animate-pulse text-[3vmin]">
      ⚠️ SCRAMBLE INVALIDATED
    </div>
    <div v-else-if="isReady" class="text-emerald-400 font-bold text-[3vmin] flex items-center justify-center gap-[1vw]">
      <span>✅ Ready to Inspect</span>
    </div>
    <div v-else-if="timer.flowState === 'Summary' && isWCAMode" class="flex flex-col items-center gap-[1vh]">
       <div class="text-[2vmin] text-slate-500 uppercase font-black">Next Scramble Teaser</div>
       <div class="text-[10vmin] text-indigo-400 font-black animate-pulse">
         {{ props.scramble.split(' ')[0] }}
       </div>
       <div class="text-[1.5vmin] text-slate-600 italic">Perform this move to begin next scramble</div>
    </div>
    <div v-else-if="timer.flowState !== 'Summary'" :class="['font-mono transition-all duration-300 transform', isWCAMode ? 'text-[8vmin] text-indigo-400 font-black' : 'text-[3vmin] text-white/90']">
      <template v-if="isWCAMode">
        {{ props.scramble.split(' ')[scrambleIndex] }}
      </template>
      <template v-else>
        <span class="tracking-tight leading-tight">{{ scramble }}</span>
      </template>
    </div>

    <div v-if="isWCAMode && scrambleIndex < scrambleLen" class="mt-[1vh] text-[1.5vmin] text-slate-600 uppercase font-bold tracking-widest">
      Move {{ scrambleIndex + 1 }} of {{ scrambleLen }}
    </div>
  </div>
</template>
