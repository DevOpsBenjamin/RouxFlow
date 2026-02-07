<script setup lang="ts">
import { useBluetoothStore } from '../../stores/bluetooth'

const bt = useBluetoothStore()

function closeModal() {
  if (!bt.isConnecting) {
    bt.showPicker = false
  }
}
</script>

<template>
  <!-- Simple modal showing connection status -->
  <!-- Web Bluetooth API shows its own device picker, so we just show loading/error states -->
  <Transition
    enter-active-class="transition duration-300 ease-out"
    enter-from-class="opacity-0 scale-95"
    enter-to-class="opacity-100 scale-100"
    leave-active-class="transition duration-200 ease-in"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-95"
  >
    <div
      v-if="bt.showPicker"
      class="fixed inset-0 z-[100] bg-slate-950/80 backdrop-blur-md flex items-center justify-center p-4"
      @click.self="closeModal"
    >
      <div class="bg-slate-900 border border-slate-800 rounded-2xl p-8 max-w-md w-full shadow-2xl">
        <!-- Connecting State -->
        <div v-if="bt.isConnecting" class="text-center space-y-6">
          <div class="relative w-20 h-20 mx-auto">
            <svg
              class="w-20 h-20 animate-spin text-indigo-500"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          </div>

          <div>
            <h3 class="text-xl font-bold text-white mb-2">Connecting to Cube</h3>
            <p class="text-sm text-slate-400">
              Please wait while we establish connection...
            </p>
          </div>

          <div class="pt-4">
            <div class="flex items-center justify-center gap-2 text-xs text-slate-500">
              <div class="w-2 h-2 rounded-full bg-indigo-500 animate-pulse"></div>
              <span>Initializing WASM protocol handler</span>
            </div>
          </div>
        </div>

        <!-- Error State -->
        <div v-else-if="bt.error" class="text-center space-y-6">
          <div class="w-20 h-20 mx-auto rounded-full bg-red-500/10 flex items-center justify-center">
            <svg class="w-10 h-10 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>

          <div>
            <h3 class="text-xl font-bold text-white mb-2">Connection Failed</h3>
            <p class="text-sm text-slate-400 mb-4">{{ bt.error }}</p>

            <div class="bg-slate-800/50 rounded-lg p-4 text-left space-y-2 text-xs text-slate-400">
              <p class="font-semibold text-slate-300">Troubleshooting:</p>
              <ul class="list-disc list-inside space-y-1 ml-2">
                <li>Make sure your cube is powered on</li>
                <li>Check if Bluetooth is enabled on your device</li>
                <li>Try turning the cube off and on again</li>
                <li>Make sure the cube isn't connected to another device</li>
              </ul>
            </div>
          </div>

          <div class="flex gap-3">
            <button
              @click="bt.startScan()"
              class="flex-1 py-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-semibold text-sm transition-colors"
            >
              Try Again
            </button>
            <button
              @click="closeModal"
              class="flex-1 py-3 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 font-semibold text-sm transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>

        <!-- Success State (briefly shown) -->
        <div v-else class="text-center space-y-6">
          <div class="w-20 h-20 mx-auto rounded-full bg-emerald-500/10 flex items-center justify-center">
            <svg class="w-10 h-10 text-emerald-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
          </div>

          <div>
            <h3 class="text-xl font-bold text-white mb-2">Connected!</h3>
            <p class="text-sm text-slate-400">
              Your cube is ready to use
            </p>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>
