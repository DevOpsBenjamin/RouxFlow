<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  cm_set_inspection_duration,
  cm_get_inspection_duration,
  cm_get_pickup_mode,
  cm_set_pickup_mode,
  cm_save_active_session,
} from '../../services/cube/bridge'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ (e: 'close'): void }>()

const inspectionOptions = [
  { label: 'Infinite', value: 0 },
  { label: '15s (WCA)', value: 15 },
]

const pickupOptions = [
  { label: 'None', value: 'None', desc: 'Timer starts at first move' },
  { label: 'Fixed (+400ms)', value: 'Fixed', desc: 'Adds 200ms pickup + 200ms putdown' },
  { label: 'Gyro', value: 'Gyro', desc: 'Detects pickup/putdown via gyroscope' },
]

const inspectionDuration = ref(15)
const pickupMode = ref('None')

watch(() => props.open, (isOpen) => {
  if (isOpen) {
    inspectionDuration.value = cm_get_inspection_duration()
    pickupMode.value = cm_get_pickup_mode()
  }
})

function setInspection(seconds: number) {
  inspectionDuration.value = seconds
  cm_set_inspection_duration(seconds)
}

async function setPickupMode(mode: string) {
  pickupMode.value = mode
  cm_set_pickup_mode(mode)
  await cm_save_active_session()
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-[200] flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in duration-200" @click.self="emit('close')">
      <div class="bg-slate-900 border border-indigo-500/20 rounded-[3vmin] p-[4vmin] max-w-[50vw] shadow-2xl animate-in zoom-in-95 duration-300">
        <div class="flex items-center justify-between mb-[3vh]">
          <h2 class="text-[3vmin] font-black text-indigo-400 uppercase tracking-wider">Session Settings</h2>
          <button @click="emit('close')" class="text-slate-500 hover:text-white transition-colors p-[1vmin]">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-[3vmin] h-[3vmin]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="space-y-[3vh]">
          <!-- Inspection Time -->
          <div>
            <div class="text-[1.5vmin] text-slate-500 font-bold uppercase tracking-wider mb-[1.5vh]">Inspection Time</div>
            <div class="flex flex-wrap gap-[1vmin]">
              <button
                v-for="opt in inspectionOptions"
                :key="opt.value"
                @click="setInspection(opt.value)"
                :class="[
                  'px-[2vmin] py-[1vh] rounded-[1vmin] text-[1.6vmin] font-bold transition-all',
                  inspectionDuration === opt.value
                    ? 'bg-indigo-500 text-white'
                    : 'bg-white/5 text-slate-400 hover:bg-white/10'
                ]"
              >{{ opt.label }}</button>
            </div>
            <p class="text-[1.2vmin] text-slate-600 mt-[1vh]">
              Time to inspect the cube after completing the scramble. Infinite = no timeout.
            </p>
          </div>

          <!-- Pickup Mode -->
          <div>
            <div class="text-[1.5vmin] text-slate-500 font-bold uppercase tracking-wider mb-[1.5vh]">Pickup Time</div>
            <div class="flex flex-wrap gap-[1vmin]">
              <button
                v-for="opt in pickupOptions"
                :key="opt.value"
                @click="setPickupMode(opt.value)"
                :class="[
                  'px-[2vmin] py-[1vh] rounded-[1vmin] text-[1.6vmin] font-bold transition-all',
                  pickupMode === opt.value
                    ? 'bg-indigo-500 text-white'
                    : 'bg-white/5 text-slate-400 hover:bg-white/10'
                ]"
              >{{ opt.label }}</button>
            </div>
            <p class="text-[1.2vmin] text-slate-600 mt-[1vh]">
              {{ pickupOptions.find(o => o.value === pickupMode)?.desc ?? '' }}
            </p>
          </div>
        </div>

        <div class="mt-[4vh] pt-[2vh] border-t border-slate-800">
          <button @click="emit('close')" class="w-full py-[1.5vh] bg-indigo-500 hover:bg-indigo-400 text-white font-bold rounded-[1.5vmin] text-[1.8vmin] transition-all">
            Done
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
