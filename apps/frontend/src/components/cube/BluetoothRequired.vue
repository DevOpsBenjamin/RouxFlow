<script setup lang="ts">
import { watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useBluetoothStore } from '../../stores/bluetooth'

const bt = useBluetoothStore()
const router = useRouter()
const route = useRoute()

// When cube connects, navigate back to the origin route
watch(() => bt.isConnected, (connected) => {
  if (connected) {
    const from = route.query.from as string
    if (from) {
      router.push({ name: from })
    } else {
      router.push({ name: 'Session' })
    }
  }
})
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

    <div class="flex flex-col gap-4 items-center">
      <p class="text-slate-500 text-sm">
        Use the <span class="text-indigo-400 font-bold">🔌 Cubes</span> menu in the header to connect your cube.
      </p>

      <button
        @click="router.push({ name: 'SupportedCubes' })"
        class="text-indigo-400 hover:text-indigo-300 font-semibold text-sm underline underline-offset-4 transition-colors"
      >
        View Supported Cubes →
      </button>
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
