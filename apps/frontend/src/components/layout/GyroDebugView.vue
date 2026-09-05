<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import Cube3D from '../cube/Cube3D.vue'
import { cm_get_orientation_debug, cm_is_connected, cm_force_home } from '../../services/cube/bridge'

const currentOrientation = ref({ 
  raw: [0,0,0,1] as [number, number, number, number], 
  home: null as [number, number, number, number] | null, 
  shell: [0,0,0,1] as [number, number, number, number],
  posture: "Unknown" as any
})
let timer: number | null = null

const activePostureName = computed(() => {
  if (!currentOrientation.value.home) return "Please Set Home first";

  if (currentOrientation.value.posture === "Unknown") {
      return "Unknown Posture (Transitioning...)";
  }

  // Find short name for posture mapping
  const top = currentOrientation.value.posture.top;
  const front = currentOrientation.value.posture.front;

  const mapping: Record<string, string> = {
      "White": "W", "Yellow": "Y", "Green": "G", "Blue": "B", "Red": "R", "Orange": "O"
  };

  const shortName = `${mapping[top]}/${mapping[front]}`;
  
  if (shortName === "W/G") return "W/G (Home)";
  return shortName;
})

function updateData() {
  if (cm_is_connected()) {
    try {
      const dbg = JSON.parse(cm_get_orientation_debug())
      currentOrientation.value = dbg
    } catch (e) {
      // Ignore parse errors if wasm not ready
    }
  }
}

function handleSetHome() {
  cm_force_home()
}

onMounted(() => {
  timer = window.setInterval(updateData, 50)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <div class="gyro-debug-page p-4 md:p-8 min-h-screen bg-slate-900 text-white flex flex-col items-center">
    <div class="max-w-5xl w-full">
      <header class="mb-8 flex flex-col sm:flex-row justify-between items-start sm:items-end gap-4">
        <div>
          <h1 class="text-3xl font-bold mb-2">Posture Tracker Debug</h1>
          <p class="text-slate-400">Set home as W/G, then test absolute posture locking.</p>
        </div>
        <button 
          @click="handleSetHome"
          class="px-6 py-3 font-bold bg-indigo-600 hover:bg-indigo-500 rounded-xl transition-colors shadow-lg shadow-indigo-500/20"
        >
          FORCE SET HOME (W/G)
        </button>
      </header>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <!-- Status Panel -->
        <div class="bg-slate-800 p-8 rounded-3xl shadow-xl shadow-slate-900 overflow-hidden flex flex-col justify-center items-center text-center border-2 border-slate-700 h-96">
            <template v-if="!currentOrientation.home">
                <div class="w-16 h-16 rounded-full bg-slate-700 flex items-center justify-center mb-6">
                    <span class="text-3xl text-slate-500">?</span>
                </div>
                <h2 class="text-2xl font-bold text-slate-300 mb-2">No Home Set</h2>
                <p class="text-slate-500">Hold your cube in the White Top / Green Front position and press the "Force Set Home" button.</p>
            </template>
            <template v-else>
                <div 
                    class="w-32 h-32 rounded-full flex items-center justify-center mb-6 transition-colors duration-300"
                    :class="activePostureName.includes('Unknown') ? 'bg-amber-500/20 border-4 border-amber-500/50 text-amber-500' : 'bg-emerald-500/20 border-4 border-emerald-500/50 text-emerald-500'"
                >
                    <svg v-if="!activePostureName.includes('Unknown')" xmlns="http://www.w3.org/2000/svg" class="h-16 w-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                    </svg>
                    <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-16 w-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                </div>
                <h2 class="text-4xl font-black mb-2 tracking-tight">{{ activePostureName }}</h2>
                <p class="text-slate-400 font-medium uppercase tracking-widest text-sm">Active Posture</p>
            </template>
        </div>

        <!-- Live View & Raw Monitor -->
        <div class="space-y-8 flex flex-col h-full">
          <div class="cube-preview flex-grow aspect-video bg-black rounded-3xl overflow-hidden border-4 border-slate-800 shadow-2xl relative">
             <Cube3D :flatLighting="true" />
             <div class="absolute bottom-4 left-4 bg-black/60 backdrop-blur-md px-3 py-1 rounded-full text-xs font-mono text-white/80 border border-white/10">3D Mirror</div>
          </div>

          <div class="data-panel bg-slate-800 p-6 rounded-2xl border border-slate-700">
            <div class="flex justify-between items-center mb-4">
                <h3 class="text-sm font-semibold text-slate-500 uppercase tracking-wider">Quaternion Streams</h3>
                <div class="flex gap-2">
                    <span class="w-2 h-2 rounded-full" :class="cm_is_connected() ? 'bg-emerald-500 animate-pulse' : 'bg-red-500'"></span>
                </div>
            </div>
            
            <div class="space-y-4 font-mono text-sm">
              <div class="bg-slate-900/50 p-3 rounded-lg border border-slate-700/50 flex justify-between items-center">
                <div class="text-slate-400 font-bold w-20 shrink-0">RAW</div>
                <div class="grid grid-cols-4 gap-2 text-right flex-grow">
                  <div class="w-16 ml-auto">{{ currentOrientation.raw[0].toFixed(3) }}</div>
                  <div class="w-16 ml-auto">{{ currentOrientation.raw[1].toFixed(3) }}</div>
                  <div class="w-16 ml-auto">{{ currentOrientation.raw[2].toFixed(3) }}</div>
                  <div class="w-16 ml-auto text-slate-500">{{ currentOrientation.raw[3].toFixed(3) }}</div>
                </div>
              </div>
              
              <div class="bg-slate-900/50 p-3 rounded-lg border border-slate-700/50 flex justify-between items-center">
                <div class="text-slate-400 font-bold w-20 shrink-0">HOME</div>
                <div v-if="currentOrientation.home" class="grid grid-cols-4 gap-2 text-right flex-grow text-emerald-400">
                  <div class="w-16 ml-auto">{{ currentOrientation.home[0].toFixed(3) }}</div>
                  <div class="w-16 ml-auto">{{ currentOrientation.home[1].toFixed(3) }}</div>
                  <div class="w-16 ml-auto">{{ currentOrientation.home[2].toFixed(3) }}</div>
                  <div class="w-16 ml-auto opacity-70">{{ currentOrientation.home[3].toFixed(3) }}</div>
                </div>
                <div v-else class="text-slate-500 italic text-right flex-grow">Not calibrated</div>
              </div>

              <div class="bg-slate-900/50 p-3 rounded-lg border border-slate-700/50 flex justify-between items-center">
                <div class="text-slate-400 font-bold w-20 shrink-0">REL (SHELL)</div>
                <div class="grid grid-cols-4 gap-2 text-right flex-grow text-amber-400">
                  <div class="w-16 ml-auto">{{ currentOrientation.shell[0].toFixed(3) }}</div>
                  <div class="w-16 ml-auto">{{ currentOrientation.shell[1].toFixed(3) }}</div>
                  <div class="w-16 ml-auto">{{ currentOrientation.shell[2].toFixed(3) }}</div>
                  <div class="w-16 ml-auto opacity-70">{{ currentOrientation.shell[3].toFixed(3) }}</div>
                </div>
              </div>
            </div>

          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.cube-preview {
  box-shadow: 0 0 100px rgba(0,0,0,0.8) inset;
}
</style>
