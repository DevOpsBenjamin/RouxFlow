<script setup lang="ts">
import { useBluetoothStore, type BluetoothDevice } from '../../stores/bluetooth'
import { useUIStore } from '../../stores/ui'
import { CubeBridge } from '../../services/cube/bridge'

const bt = useBluetoothStore()
const ui = useUIStore()

async function selectDevice(device: BluetoothDevice) {
  bt.isConnecting = true
  try {
    await CubeBridge.finalConnect(device)
    bt.showPicker = false
    ui.setActiveSession()
  } catch (e: any) {
    bt.setError(e.message || 'Failed to connect')
  } finally {
    bt.isConnecting = false
  }
}
</script>

<template>
  <Transition 
    enter-active-class="transition duration-500 ease-out"
    enter-from-class="opacity-0 scale-95"
    enter-to-class="opacity-100 scale-100"
    leave-active-class="transition duration-300 ease-in"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-95"
  >
    <div v-if="bt.showPicker" class="fixed inset-0 z-[100] bg-slate-950/80 backdrop-blur-md flex items-center justify-center p-[5vmin]">
      <div class="bg-slate-900 border border-slate-800 rounded-[4vmin] p-[6vmin] max-w-[50vw] w-full shadow-2xl flex flex-col gap-[4vh]">
        <header class="flex justify-between items-start border-b border-slate-800 pb-[2vh]">
          <div>
            <h2 class="text-[3vmin] font-black italic text-white tracking-tighter uppercase leading-none">Select Cube</h2>
            <p class="text-[1.2vmin] text-slate-500 font-bold uppercase tracking-widest mt-2 flex items-center gap-2">
              <span v-if="bt.isScanning" class="w-2 h-2 bg-indigo-500 rounded-full animate-pulse"></span>
              {{ bt.isScanning ? 'Scanning for cubes...' : 'Scan Paused' }}
            </p>
          </div>
          <div class="flex gap-[1vw]">
             <button 
               @click="CubeBridge.connect()" 
               :disabled="bt.isScanning"
               class="text-indigo-400 hover:text-indigo-300 text-[1.5vmin] font-bold uppercase disabled:opacity-30"
             >
               {{ bt.isScanning ? 'Scanning...' : 'Refresh' }}
             </button>
             <button @click="bt.showPicker = false" class="text-slate-500 hover:text-white transition-colors text-[2vmin]">✕</button>
          </div>
        </header>

        <!-- Device List -->
        <div class="flex-1 overflow-y-auto max-h-[50vh] pr-[2vmin] space-y-[2vh]">
          <div v-if="bt.scannedDevices.length === 0" class="text-center py-[10vh] border-2 border-dashed border-slate-800 rounded-[3vmin] flex flex-col items-center gap-[2vh]">
             <span class="text-[6vmin] opacity-20">📡</span>
             <div class="space-y-1">
               <p class="text-slate-400 font-bold text-[2vmin]">{{ bt.isScanning ? 'Searching for cubes...' : 'No devices found' }}</p>
               <p class="text-slate-600 text-[1.4vmin] max-w-[20vw] mx-auto">Make sure your Bluetooth is ON and your cube is awake.</p>
             </div>
             <button 
               v-if="!bt.isScanning" 
               @click="CubeBridge.connect()"
               class="mt-[2vh] px-[4vw] py-[1.5vh] rounded-[2vmin] bg-slate-800 text-white text-[1.8vmin] hover:bg-slate-700 transition-all"
             >
               Try Again
             </button>
          </div>
          
          <button 
            v-for="device in bt.scannedDevices" 
            :key="device.id"
            @click="selectDevice(device)"
            :disabled="bt.isConnecting"
            class="w-full flex justify-between items-center p-[3vmin] rounded-[2vmin] bg-slate-800/50 border border-slate-700/50 hover:bg-slate-800 hover:border-indigo-500/50 transition-all group"
          >
            <div class="text-left">
              <div class="text-[2vmin] font-bold text-white group-hover:text-indigo-400 transition-colors">{{ device.name }}</div>
              <div class="text-[1.2vmin] text-slate-500 font-mono">{{ device.id }}</div>
            </div>
            <div class="text-[1.5vmin] text-slate-400 font-mono group-hover:text-white">
              {{ device.rssi }} dBm
            </div>
          </button>
        </div>

        <footer v-if="bt.error" class="bg-red-500/10 border border-red-500/20 p-[2vmin] rounded-[1.5vmin] text-red-400 text-[1.4vmin]">
          {{ bt.error }}
        </footer>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
::-webkit-scrollbar {
  width: 0.5vmin;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: #1e293b;
  border-radius: 1vmin;
}
::-webkit-scrollbar-thumb:hover {
  background: #334155;
}
</style>
