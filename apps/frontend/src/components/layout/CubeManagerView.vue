<script setup lang="ts">
import { computed } from 'vue'
import { useBluetoothStore } from '../../stores/bluetooth'
import Cube3D from '../cube/Cube3D.vue'
//import BluetoothRequired from '../cube/BluetoothRequired.vue'

const bt = useBluetoothStore()

// Mock function for now, will bridge to Rust Core later
function resetGyro() {
  console.log('Resetting Gyro orientation...')
  // TODO: Call Rust Core to reset gyro offset
}

const isGyroSupported = computed(() => {
  return bt.deviceInfo?.features?.gyro || true // Assume true for now for testing
})

</script>

<template>
  <div class="w-full h-full p-[2vmin] flex flex-col gap-[2vh] overflow-hidden relative">
    
    <!-- Background Gradient for Premium Feel -->
    <div class="absolute inset-0 bg-gradient-to-br from-slate-950 via-slate-900 to-indigo-950/20 -z-10"></div>

    <Transition 
      enter-active-class="transition duration-700 ease-out"
      enter-from-class="opacity-0 translate-y-8"
      enter-to-class="opacity-100 translate-y-0"
    >
      <div class="flex-1 flex flex-col lg:flex-row gap-[2vmin] h-full overflow-hidden animate-in fade-in duration-700">
        
        <!-- Left Panel: 3D View & Visualization -->
        <section class="lg:flex-none lg:aspect-square lg:h-full lg:w-auto w-full aspect-square relative bg-slate-900/40 backdrop-blur-xl rounded-[3vmin] border border-white/2 overflow-hidden shadow-2xl group flex flex-col items-center justify-center">
          <div class="absolute inset-0 bg-gradient-to-b from-transparent to-slate-950/80 pointer-events-none z-10"></div>
          
          <!-- 3D Canvas -->
          <Cube3D class="absolute inset-0 w-full h-full" />
          
          <!-- Overlay Info -->
          <div class="absolute bottom-0 left-0 right-0 p-[3vmin] z-20 flex justify-between items-end">
             <div>
               <h2 class="text-[3vmin] font-black text-white leading-none">{{ bt.deviceInfo?.name || 'Connected Cube' }}</h2>
               <div class="text-[1.5vmin] text-indigo-400 font-mono mt-1 opacity-80">{{ bt.deviceInfo?.address || 'UNKNOWN MAC' }}</div>
             </div>
             <div class="text-right">
                <div class="text-[1.2vmin] font-bold uppercase tracking-widest text-slate-500 mb-1">Protocol</div>
                <div class="px-3 py-1 bg-white/10 rounded-full text-[1.5vmin] text-white font-bold backdrop-blur-md">
                  {{ bt.deviceInfo?.protocol || 'Unknown' }}
                </div>
             </div>
          </div>
        </section>

        <!-- Right Panel: Controls & Settings -->
        <aside class="flex-1 w-full min-w-0 flex flex-col gap-[3vh]">
          
          <!-- Orientation Control -->
          <div class="bg-indigo-500/10 backdrop-blur-xl rounded-[2.5vmin] p-[3vmin] border border-indigo-500/20 shadow-lg">
             <div class="flex items-center gap-3 mb-[2vh]">
                <div class="p-2 bg-indigo-500 rounded-lg text-white">🧭</div>
                <h3 class="text-[2vmin] font-bold text-white">Orientation</h3>
             </div>
             
             <p class="text-[1.4vmin] text-slate-400 mb-[3vh] leading-relaxed">
               Align your physical cube to match the screen. 
               <span class="text-indigo-300 font-bold">Blue Front</span>, <span class="text-indigo-300 font-bold">White Down</span>.
             </p>

             <button 
               @click="resetGyro"
               :disabled="!isGyroSupported"
               class="w-full py-[2vh] rounded-[1.5vmin] bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-[1.8vmin] shadow-lg shadow-indigo-600/20 transition-all transform hover:scale-[1.02] active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
             >
               <span>🎯</span>
               <span>Reset Gyro</span>
             </button>
             
             <div v-if="!isGyroSupported" class="mt-2 text-[1.2vmin] text-red-400 text-center italic">
               * This cube does not support Gyroscope
             </div>
          </div>

          <!-- Hardware Info -->
          <div class="bg-slate-900/40 backdrop-blur-xl rounded-[2.5vmin] p-[3vmin] border border-white/5 flex-1">
             <h3 class="text-[1.8vmin] font-bold text-slate-400 mb-[2vh] uppercase tracking-wider">Hardware Stats</h3>
             
             <div class="space-y-[1.5vh]">
                <div class="flex justify-between items-center p-[1.5vmin] rounded-[1.5vmin] bg-white/5">
                   <span class="text-[1.4vmin] text-slate-500">Battery</span>
                   <span class="text-[1.4vmin] font-bold text-emerald-400">84%</span>
                </div>
                <!-- Add more stats here later -->
             </div>
          </div>

        </aside>

      </div>
    </Transition>
  </div>
</template>
