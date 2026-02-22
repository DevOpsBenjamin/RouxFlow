<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import Cube3D from '../cube/Cube3D.vue'
import { cm_get_orientation, cm_is_connected } from '../../services/cube/bridge'
import { logger } from '../../utils/logger'

// Calibration steps
const steps = [
  { id: 'WG', target: 'W/G', instruction: 'Hold cube with White Top, Green Front' },
  { id: 'WR', target: 'W/R', instruction: 'Rotate to White Top, Red Front (y rotation)' },
  { id: 'RY', target: 'R/Y', instruction: 'Rotate to Red Top, Yellow Front (x rotation)' },
  { id: 'BY', target: 'B/Y', instruction: 'Rotate to Blue Top, Yellow Front (z\' rotation)' },
  { id: 'BR', target: 'B/R', instruction: 'Rotate to Blue Top, Red Front (y\' rotation)' },
  { id: 'RG', target: 'R/G', instruction: 'Rotate to Red Top, Green Front (x\' rotation)' },
]

const currentStepIdx = ref(0)
const recordings = ref<any[]>([])
const currentOrientation = ref({ w: 1, x: 0, y: 0, z: 0 })
let timer: number | null = null

const formatQuat = (v: number) => v.toFixed(4)

const currentStep = computed(() => steps[currentStepIdx.value])
const isFinished = computed(() => currentStepIdx.value >= steps.length)

function updateData() {
  if (cm_is_connected()) {
    try {
      const [qx, qy, qz, qw] = JSON.parse(cm_get_orientation()) as [number, number, number, number]
      currentOrientation.value = { w: qw, x: qx, y: qy, z: qz }
    } catch (e) {
      // Ignore parse errors if wasm not ready
    }
  }
}

function handleContinue() {
  if (isFinished.value || !currentStep.value) return

  // Record current state
  recordings.value.push({
    step: currentStep.value.target,
    quaternion: { ...currentOrientation.value }
  })

  // Advance
  currentStepIdx.value++
  
  if (isFinished.value) {
    logger.info("Calibration sequence finished", recordings.value)
  }
}

function resetCalibration() {
  currentStepIdx.value = 0
  recordings.value = []
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
      <header class="mb-8 flex justify-between items-end">
        <div>
          <h1 class="text-3xl font-bold mb-2">Gyro Calibration & Debug</h1>
          <p class="text-slate-400">Verify IMU mapping through a guided sequence.</p>
        </div>
        <button 
          @click="resetCalibration"
          class="px-4 py-2 text-sm bg-slate-700 hover:bg-slate-600 rounded-lg transition-colors"
        >
          Reset
        </button>
      </header>

      <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <!-- Main Controls / Instructions -->
        <div class="lg:col-span-2 space-y-8">
          <div v-if="!isFinished && currentStep" class="bg-indigo-600 p-8 rounded-3xl shadow-xl shadow-indigo-500/20">
            <div class="flex items-center gap-4 mb-4">
              <span class="w-10 h-10 rounded-full bg-white/20 flex items-center justify-center font-bold">
                {{ currentStepIdx + 1 }}
              </span>
              <h2 class="text-2xl font-bold">Target: {{ currentStep.target }}</h2>
            </div>
            <p class="text-indigo-100 text-lg mb-8">{{ currentStep.instruction }}</p>
            
            <button 
              @click="handleContinue"
              class="w-full py-4 bg-white text-indigo-600 font-bold rounded-2xl hover:scale-[1.02] active:scale-[0.98] transition-all shadow-lg text-xl"
            >
              {{ currentStepIdx === 0 ? 'Set Home & Start' : 'Log Orientation & Continue' }}
            </button>
          </div>

          <div v-else class="bg-emerald-600 p-8 rounded-3xl shadow-xl shadow-emerald-500/20">
            <h2 class="text-2xl font-bold mb-4">Calibration Finished!</h2>
            <p class="text-emerald-50 text-lg mb-4">You have recorded {{ recordings.length }} points. Analysis below.</p>
            <button 
              @click="resetCalibration"
              class="px-6 py-3 bg-white text-emerald-600 font-bold rounded-xl hover:scale-105 transition-all"
            >
              Restart Test
            </button>
          </div>

          <!-- Recorded Data Table -->
          <div class="bg-slate-800 rounded-2xl border border-slate-700 overflow-hidden">
            <table class="w-full text-left text-sm">
              <thead class="bg-slate-900/50 text-slate-400 border-b border-slate-700">
                <tr>
                  <th class="px-6 py-4 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">Target</th>
                  <th class="px-6 py-4 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">W</th>
                  <th class="px-6 py-4 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">X (Pitch)</th>
                  <th class="px-6 py-4 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">Y (Yaw)</th>
                  <th class="px-6 py-4 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">Z (Roll)</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-800">
                <tr v-for="rec in recordings" :key="rec.step" class="hover:bg-gray-800/50 transition-colors">
                  <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-white">{{ rec.step }}</td>
                  <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-300">{{ formatQuat(rec.quaternion.w) }}</td>
                  <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-300">{{ formatQuat(rec.quaternion.x) }}</td>
                  <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-300">{{ formatQuat(rec.quaternion.y) }}</td>
                  <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-300">{{ formatQuat(rec.quaternion.z) }}</td>
                </tr>
                <tr v-if="recordings.length === 0">
                  <td colspan="5" class="p-8 text-center text-slate-500 italic">No points recorded yet.</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- Live View & Raw Monitor -->
        <div class="space-y-8">
          <div class="cube-preview aspect-square bg-black rounded-3xl overflow-hidden border-4 border-slate-800 shadow-2xl">
             <Cube3D />
          </div>

          <div class="data-panel bg-slate-800 p-6 rounded-2xl border border-slate-700">
            <h3 class="text-sm font-semibold text-slate-500 uppercase tracking-wider mb-4">Live Raw Monitor</h3>
            <div class="space-y-3">
              <div v-for="(val, key) in currentOrientation" :key="key" class="flex justify-between items-center bg-slate-900/50 p-2 rounded-lg px-4 border border-slate-700/50">
                <span class="text-slate-400 font-bold">{{ key.toUpperCase() }}</span>
                <span class="font-mono text-lg" :class="Math.abs(val) > 0.8 ? 'text-indigo-400' : ''">
                  {{ val.toFixed(4) }}
                </span>
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
