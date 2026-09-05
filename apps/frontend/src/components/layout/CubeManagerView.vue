<script setup lang="ts">
import { computed, ref } from 'vue'
import { useBluetoothStore } from '../../stores/bluetooth'
import Cube3D from '../cube/Cube3D.vue'
import { reset_gyro, resetCube } from '../../services/cube/bridge'
import { logger } from '../../utils/logger'

const bt = useBluetoothStore()
const isResetting = ref(false)

function resetGyro() {
  try {
    reset_gyro()
    logger.info('Gyro orientation reset')
  } catch (e) {
    logger.error('Failed to reset gyro:', e)
  }
}

async function handleResetCube() {
  isResetting.value = true
  try {
    await resetCube()
    logger.info('Cube reset command sent')
  } catch (e) {
    logger.error('Failed to reset cube:', e)
  } finally {
    setTimeout(() => { isResetting.value = false }, 1000)
  }
}

const isGyroSupported = computed(() => {
  return bt.deviceInfo?.has_gyro ?? false
})
</script>

<template>
  <div class="w-full h-full p-[2vmin] flex flex-col gap-[2vh] overflow-hidden relative">

    <!-- Background Gradient -->
    <div class="absolute inset-0 bg-gradient-to-br from-slate-950 via-slate-900 to-indigo-950/20 -z-10"></div>

    <Transition
      enter-active-class="transition duration-700 ease-out"
      enter-from-class="opacity-0 translate-y-8"
      enter-to-class="opacity-100 translate-y-0"
    >
      <div class="flex-1 flex flex-col lg:flex-row gap-[2vmin] h-full overflow-hidden animate-in fade-in duration-700">

        <!-- Left Panel: 3D View -->
        <section class="lg:flex-none lg:aspect-square lg:h-full lg:w-auto w-full aspect-square relative bg-slate-900/40 backdrop-blur-xl rounded-[3vmin] border border-white/2 overflow-hidden shadow-2xl group flex flex-col items-center justify-center">
          <div class="absolute inset-0 bg-gradient-to-b from-transparent to-slate-950/80 pointer-events-none z-10"></div>
          <Cube3D class="absolute inset-0 w-full h-full" />

          <div class="absolute bottom-0 left-0 right-0 p-[3vmin] z-20 flex justify-between items-end">
             <div>
               <h2 class="text-[3vmin] font-black text-white leading-none">{{ bt.deviceInfo?.name || 'Connected Cube' }}</h2>
               <div class="text-[1.5vmin] text-indigo-400 font-mono mt-1 opacity-80">{{ bt.deviceInfo?.mac_address || 'UNKNOWN MAC' }}</div>
             </div>
             <div class="text-right">
                <div class="text-[1.2vmin] font-bold uppercase tracking-widest text-slate-500 mb-1">Protocol</div>
                <div class="px-3 py-1 bg-white/10 rounded-full text-[1.5vmin] text-white font-bold backdrop-blur-md">
                  {{ bt.deviceInfo?.protocol_name || 'Unknown' }}
                </div>
             </div>
          </div>
        </section>

        <!-- Right Panel: Controls & Settings -->
        <aside class="flex-1 w-full min-w-0 flex flex-col gap-[3vh]">

          <!-- Controls Row: 2 columns -->
          <div class="grid grid-cols-2 gap-[2vmin]">

            <!-- Left: Gyro Reset -->
            <div class="bg-indigo-500/10 backdrop-blur-xl rounded-[2.5vmin] p-[3vmin] border border-indigo-500/20 shadow-lg flex flex-col">
               <div class="flex items-center gap-2 mb-[1.5vh]">
                  <div class="p-1.5 bg-indigo-500 rounded-lg text-white text-[1.5vmin]">&#x1F9ED;</div>
                  <h3 class="text-[1.8vmin] font-bold text-white">Gyro</h3>
               </div>

               <p class="text-[2vmin] text-slate-400 mb-[2vh] leading-relaxed flex-1">
                 Position your cube with white on top and green facing you, then press the button to sync the 3D view with your cube's orientation.
               </p>

               <button
                 @click="resetGyro"
                 :disabled="!isGyroSupported"
                 class="w-full py-[1.5vh] rounded-[1.5vmin] bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-[1.5vmin] shadow-lg shadow-indigo-600/20 transition-all transform hover:scale-[1.02] active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
               >
                 <span>Reset Gyro</span>
               </button>

               <div v-if="!isGyroSupported" class="mt-1 text-[1.1vmin] text-red-400 text-center italic">
                 No gyroscope
               </div>
            </div>

            <!-- Right: Reset Cube -->
            <div class="bg-red-500/10 backdrop-blur-xl rounded-[2.5vmin] p-[3vmin] border border-red-500/20 shadow-lg flex flex-col">
               <div class="flex items-center gap-2 mb-[1.5vh]">
                  <div class="p-1.5 bg-red-500 rounded-lg text-white text-[1.5vmin]">&#x1F504;</div>
                  <h3 class="text-[1.8vmin] font-bold text-white">Reset</h3>
               </div>

               <p class="text-[2vmin] text-slate-400 mb-[2vh] leading-relaxed flex-1">
                 If your cube is out of sync (e.g. after using it without battery), solve it first, then press this button to reset the synchronization.
               </p>

               <button
                 @click="handleResetCube"
                 :disabled="isResetting"
                 class="w-full py-[1.5vh] rounded-[1.5vmin] bg-red-600 hover:bg-red-500 text-white font-bold text-[1.5vmin] shadow-lg shadow-red-600/20 transition-all transform hover:scale-[1.02] active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
               >
                 <span>{{ isResetting ? 'Resetting...' : 'Reset Cube' }}</span>
               </button>
            </div>

          </div>

          <!-- Hardware Info -->
          <div class="bg-slate-900/40 backdrop-blur-xl rounded-[2.5vmin] p-[3vmin] border border-white/5 flex-1">
             <h3 class="text-[1.8vmin] font-bold text-slate-400 mb-[2vh] uppercase tracking-wider">Hardware Stats</h3>

             <div class="space-y-[1.5vh]">
                <div class="flex justify-between items-center p-[1.5vmin] rounded-[1.5vmin] bg-white/5">
                   <span class="text-[2vmin] text-slate-500">Battery</span>
                   <span class="text-[2vmin] font-bold" :class="bt.deviceInfo?.battery_level != null ? 'text-emerald-400' : 'text-slate-600'">
                     {{ bt.deviceInfo?.battery_level != null ? bt.deviceInfo.battery_level + '%' : 'Polling...' }}
                   </span>
                </div>
                <div class="flex justify-between items-center p-[1.5vmin] rounded-[1.5vmin] bg-white/5">
                   <span class="text-[2vmin] text-slate-500">Firmware</span>
                   <span class="text-[2vmin] font-mono text-white">{{ bt.deviceInfo?.sw_version || '--' }}</span>
                </div>
                <div class="flex justify-between items-center p-[1.5vmin] rounded-[1.5vmin] bg-white/5">
                   <span class="text-[2vmin] text-slate-500">Hardware</span>
                   <span class="text-[2vmin] font-mono text-white">{{ bt.deviceInfo?.hw_version || '--' }}</span>
                </div>
                <div class="flex justify-between items-center p-[1.5vmin] rounded-[1.5vmin] bg-white/5">
                   <span class="text-[2vmin] text-slate-500">Gyroscope</span>
                   <span class="text-[2vmin] font-bold" :class="bt.deviceInfo?.has_gyro ? 'text-emerald-400' : 'text-slate-600'">
                     {{ bt.deviceInfo?.has_gyro ? 'Supported' : 'Not Available' }}
                   </span>
                </div>
             </div>
          </div>

        </aside>

      </div>
    </Transition>
  </div>
</template>
