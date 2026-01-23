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
    
    sessionManager.handle_scramble_move(moveStr, Date.now() / 1000)
    updateState()
  }
})
</script>

<template>
  <div class="max-w-4xl mx-auto text-center p-8 bg-slate-900/50 border-t border-white/5">
    <div class="text-slate-500 text-xs mb-4 font-bold uppercase tracking-widest flex items-center justify-center gap-2">
      <span v-if="isWCAMode" class="px-2 py-0.5 bg-amber-500/10 text-amber-500 border border-amber-500/20 rounded text-[10px]">WCA Integrity Mode</span>
      <span v-else>Next Scramble</span>
    </div>

    <div v-if="isInvalid" class="text-rose-400 font-bold animate-pulse text-lg">
      ⚠️ SCRAMBLE INVALIDATED (Slow Turn)
    </div>
    <div v-else-if="isReady" class="text-emerald-400 font-bold text-lg flex items-center justify-center gap-2">
      <span>✅ Ready to Inspect</span>
    </div>
    <div v-else :class="['font-mono transition-all duration-300', isWCAMode ? 'text-4xl text-indigo-400' : 'text-xl text-slate-300']">
      <template v-if="isWCAMode">
        {{ props.scramble.split(' ')[scrambleIndex] }}
      </template>
      <template v-else>
        {{ scramble }}
      </template>
    </div>

    <div v-if="isWCAMode && scrambleIndex < scrambleLen" class="mt-4 text-[10px] text-slate-600 uppercase font-bold tracking-widest">
      Move {{ scrambleIndex + 1 }} of {{ scrambleLen }}
    </div>
  </div>
</template>
