<script setup lang="ts">
import { ref } from 'vue'
import { CubeBridge } from '../../services/cube/bridge'
import { useUIStore } from '../../stores/ui'

const ui = useUIStore()
const isConnecting = ref(false)
const error = ref('')

async function connect() {
  isConnecting.value = true
  error.value = ''
  try {
    await CubeBridge.connect()
  } catch (e: any) {
    error.value = e.message || 'Failed to connect'
  } finally {
    isConnecting.value = false
  }
}
</script>

<template>
  <div class="w-[100vw] h-[100vh] flex flex-col items-center justify-center gap-[5vh] p-[5vmin] overflow-hidden">
    <!-- Hero Section -->
    <header class="text-center space-y-[2vh]">
      <h1 class="text-[12vmin] font-black tracking-tighter bg-gradient-to-br from-white to-slate-500 bg-clip-text text-transparent italic leading-none">
        ROUXFLOW
      </h1>
      <p class="text-[3vmin] text-slate-400 font-medium max-w-[60vw] mx-auto">
        High-performance Roux method engine designed exclusively for Bluetooth smart cubes.
      </p>
    </header>

    <!-- Main Actions -->
    <div class="flex flex-col md:flex-row gap-[3vmin] w-full max-w-[80vw] justify-center">
      <button 
        @click="connect"
        :disabled="isConnecting"
        class="group relative overflow-hidden rounded-[4vmin] bg-indigo-600 p-px font-semibold text-white transition-all hover:scale-[1.05] active:scale-95 disabled:opacity-50 w-full md:w-[35vw]"
      >
        <div class="relative flex items-center justify-center gap-[2vmin] rounded-[3.8vmin] bg-indigo-600 px-[4vw] py-[3vh] transition-all group-hover:bg-indigo-500">
          <span v-if="isConnecting" class="animate-spin text-[5vmin]">⏳</span>
          <span v-else class="text-[5vmin]">⚡</span>
          <span class="text-[4vmin]">{{ isConnecting ? 'Connecting...' : 'Connect' }}</span>
        </div>
      </button>

      <button 
        @click="ui.setActiveSession()"
        class="flex items-center justify-center gap-[2vmin] py-[3vh] px-[4vw] rounded-[4vmin] border border-slate-800 bg-slate-900/50 text-slate-300 transition-all hover:bg-slate-800 hover:text-white w-full md:w-[35vw]"
      >
        <span class="text-[5vmin]">📊</span>
        <span class="text-[4vmin]">History</span>
      </button>
    </div>

    <!-- Friendly Warning -->
    <footer class="text-center space-y-[1vh] mt-auto pb-[5vh] border-t border-slate-900/50 w-full pt-[4vh]">
      <p class="text-[1.5vmin] text-slate-500 uppercase tracking-[0.2em] font-bold">Disclaimer</p>
      <p class="text-[1.8vmin] text-slate-400 max-w-[70vw] mx-auto leading-relaxed">
        This app utilizes real-time move data for phase analysis. 
        If you're looking for a manual timer, use 
        <a href="https://cubeast.com" target="_blank" class="text-indigo-400 underline decoration-indigo-400/30 underline-offset-4">Cubeast</a> 
        or <a href="https://cstimer.net" target="_blank" class="text-indigo-400 underline decoration-indigo-400/30 underline-offset-4">csTimer</a>.
      </p>
    </footer>

    <div v-if="error" class="fixed top-[5vh] text-red-400 bg-red-400/10 px-[2vw] py-[1vh] rounded-[1vmin] border border-red-400/20 text-[2vmin]">
      {{ error }}
    </div>
  </div>
</template>
