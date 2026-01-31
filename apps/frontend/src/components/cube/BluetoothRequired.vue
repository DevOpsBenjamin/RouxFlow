<script setup lang="ts">
import { useBluetoothStore } from '../../stores/bluetooth'

const bt = useBluetoothStore()
</script>

<template>
  <div class="flex-1 flex flex-col items-center justify-center p-12 text-center space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-1000">
    <div class="relative">
      <div class="absolute inset-0 bg-indigo-500/20 blur-3xl rounded-full"></div>
      <div class="relative text-8xl animate-bounce duration-[2000ms]">🧊</div>
    </div>
    
    <div class="max-w-md space-y-4">
      <h2 class="text-4xl font-black italic tracking-tighter text-white uppercase leading-none">
        Bluetooth Cube <span class="text-indigo-400">Required</span>
      </h2>
      <p class="text-slate-400 font-medium text-sm md:text-base">
        RouxFlow is designed to work exclusively with smart Bluetooth cubes to provide real-time phase analysis and move tracking.
      </p>
    </div>

    <div class="flex flex-col gap-4 w-full max-w-xs">
      <button 
        @click="bt.startScan()"
        class="group relative py-4 px-8 rounded-2xl bg-indigo-600 text-white font-black uppercase tracking-widest text-sm shadow-xl shadow-indigo-600/20 hover:bg-indigo-500 hover:scale-105 active:scale-95 transition-all"
      >
        Connect Your Cube
        <div class="absolute inset-0 rounded-2xl border-2 border-indigo-400/50 scale-110 opacity-0 group-hover:scale-100 group-hover:opacity-100 transition-all"></div>
      </button>
      
      <p class="text-[10px] text-slate-600 font-bold uppercase tracking-[0.2em]">
        Supports MoYu AI, GAN, and more
      </p>
    </div>
  </div>
</template>

<style scoped>
.animate-in {
  animation: fadeIn 0.8s ease-out;
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
