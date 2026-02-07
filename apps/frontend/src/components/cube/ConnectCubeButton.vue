<script setup lang="ts">
import { ref } from 'vue'
import { useBluetoothStore } from '../../stores/bluetooth'
import { reset_gyro } from '../../services/cube/bridge'
import { logger } from '../../utils/logger'

const bt = useBluetoothStore()
const showCubeManager = ref(false)

async function handleConnect() {
  if (bt.isConnected) {
    // Already connected - open cube manager
    showCubeManager.value = true
  } else {
    // Not connected - trigger connection flow
    await bt.startScan()
  }
}

defineExpose({ showCubeManager })
</script>

<template>
  <div>
    <!-- Not Connected: Big "Connect a Cube" Button -->
    <button
      v-if="!bt.isConnected && bt.savedCubes.length === 0"
      @click="handleConnect"
      :disabled="bt.isConnecting"
      class="group relative px-6 py-3 rounded-2xl bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white font-bold text-sm shadow-lg shadow-indigo-500/30 transition-all transform hover:scale-105 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
    >
      <span v-if="!bt.isConnecting" class="flex items-center gap-2">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16l2.879-2.879m0 0a3 3 0 104.243-4.242 3 3 0 00-4.243 4.242zM21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        Connect a Cube
      </span>
      <span v-else class="flex items-center gap-2">
        <svg class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        Connecting...
      </span>
    </button>

    <!-- Has Saved Cubes but Not Connected: Reconnect Button -->
    <button
      v-else-if="!bt.isConnected && bt.savedCubes.length > 0"
      @click="handleConnect"
      :disabled="bt.isConnecting"
      class="group px-5 py-2.5 rounded-xl bg-slate-800/50 hover:bg-slate-800 border border-slate-700 hover:border-indigo-500/50 text-white font-semibold text-sm transition-all flex items-center gap-3"
    >
      <div class="flex items-center gap-2">
        <div class="w-2 h-2 rounded-full bg-red-500 animate-pulse"></div>
        <span v-if="!bt.isConnecting">No Cube Connected</span>
        <span v-else>Connecting...</span>
      </div>
      <svg class="w-4 h-4 text-slate-400 group-hover:text-indigo-400 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
    </button>

    <!-- Connected: Cube Status Bar (clickable) -->
    <button
      v-else
      @click="showCubeManager = true"
      class="group px-4 py-2 rounded-xl bg-gradient-to-r from-emerald-500/10 to-cyan-500/10 hover:from-emerald-500/20 hover:to-cyan-500/20 border border-emerald-500/20 hover:border-emerald-500/40 transition-all flex items-center gap-3"
    >
      <div class="flex items-center gap-2">
        <div class="w-2 h-2 rounded-full bg-emerald-500 shadow-lg shadow-emerald-500/50"></div>
        <span class="text-white font-semibold text-sm">{{ bt.connectedDeviceName || 'Connected' }}</span>
      </div>

      <div class="flex items-center gap-2 text-xs text-slate-400 group-hover:text-slate-300 transition-colors">
        <span v-if="bt.deviceInfo?.battery_level" class="flex items-center gap-1">
          🔋 {{ bt.deviceInfo.battery_level }}%
        </span>
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>
      </div>
    </button>

    <!-- Cube Manager Drawer -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition duration-300 ease-out"
        enter-from-class="translate-x-full"
        enter-to-class="translate-x-0"
        leave-active-class="transition duration-200 ease-in"
        leave-from-class="translate-x-0"
        leave-to-class="translate-x-full"
      >
        <div
          v-if="showCubeManager"
          class="fixed inset-y-0 right-0 w-full max-w-md bg-slate-900 border-l border-slate-800 shadow-2xl z-50 flex flex-col"
        >
          <!-- Header -->
          <div class="flex items-center justify-between p-6 border-b border-slate-800">
            <div>
              <h2 class="text-xl font-bold text-white">Cube Manager</h2>
              <p class="text-xs text-slate-400 mt-1">Manage your connected cube</p>
            </div>
            <button
              @click="showCubeManager = false"
              class="p-2 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-white transition-colors"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <!-- Content -->
          <div class="flex-1 overflow-y-auto p-6 space-y-6">
            <!-- Current Cube Info -->
            <div class="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
              <div class="flex items-start justify-between mb-3">
                <div>
                  <h3 class="text-sm font-bold text-slate-400 uppercase tracking-wide">Connected Cube</h3>
                  <p class="text-lg font-bold text-white mt-1">{{ bt.connectedDeviceName || 'Unknown' }}</p>
                </div>
                <div class="w-2 h-2 rounded-full bg-emerald-500 shadow-lg shadow-emerald-500/50 mt-2"></div>
              </div>

              <div class="space-y-2 text-sm">
                <div class="flex justify-between text-slate-400">
                  <span>Protocol</span>
                  <span class="text-white font-mono">{{ bt.deviceInfo?.protocol_name || 'Unknown' }}</span>
                </div>
                <div class="flex justify-between text-slate-400">
                  <span>MAC Address</span>
                  <span class="text-white font-mono text-xs">{{ bt.deviceInfo?.mac_address || 'Unknown' }}</span>
                </div>
                <div v-if="bt.deviceInfo?.battery_level" class="flex justify-between text-slate-400">
                  <span>Battery</span>
                  <span class="text-white font-semibold">{{ bt.deviceInfo.battery_level }}%</span>
                </div>
                <div class="flex justify-between text-slate-400">
                  <span>Gyroscope</span>
                  <span :class="bt.deviceInfo?.has_gyro ? 'text-emerald-400' : 'text-slate-500'">
                    {{ bt.deviceInfo?.has_gyro ? '✓ Supported' : '✗ Not Available' }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Gyro Controls (if supported) -->
            <div v-if="bt.deviceInfo?.has_gyro" class="space-y-3">
              <h3 class="text-sm font-bold text-slate-400 uppercase tracking-wide">Gyroscope</h3>

              <div class="bg-indigo-500/10 rounded-xl p-4 border border-indigo-500/20">
                <p class="text-sm text-slate-300 mb-3">
                  Reset the gyroscope orientation to match your physical cube position.
                </p>
                <button
                  @click="() => { try { reset_gyro(); logger.info('Gyro reset'); } catch(e) { logger.error('Gyro reset failed', e); } }"
                  class="w-full py-2.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-semibold text-sm transition-colors flex items-center justify-center gap-2"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                  Reset Gyro Orientation
                </button>
              </div>
            </div>

            <!-- Disconnect Button -->
            <div class="pt-4 border-t border-slate-800">
              <button
                @click="async () => { await bt.disconnect(); showCubeManager = false }"
                class="w-full py-2.5 rounded-lg bg-red-500/10 hover:bg-red-500/20 border border-red-500/20 hover:border-red-500/40 text-red-400 hover:text-red-300 font-semibold text-sm transition-colors"
              >
                Disconnect Cube
              </button>
            </div>

            <!-- More Actions Menu -->
            <details class="group/menu">
              <summary class="flex items-center justify-between p-3 rounded-lg bg-slate-800/30 hover:bg-slate-800/50 cursor-pointer transition-colors list-none">
                <span class="text-sm font-semibold text-slate-300">More Actions</span>
                <svg class="w-4 h-4 text-slate-400 transition-transform group-open/menu:rotate-180" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
              </summary>

              <div class="mt-2 space-y-2 pl-3">
                <button
                  @click="bt.startScan()"
                  class="w-full text-left p-3 rounded-lg hover:bg-slate-800/50 text-sm text-slate-300 hover:text-white transition-colors flex items-center gap-2"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                  </svg>
                  Connect Another Cube
                </button>

                <button
                  v-if="bt.savedCubes.length > 1"
                  class="w-full text-left p-3 rounded-lg hover:bg-slate-800/50 text-sm text-slate-300 hover:text-white transition-colors flex items-center gap-2"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
                  </svg>
                  Switch Cube
                </button>
              </div>
            </details>

            <!-- Saved Cubes List -->
            <div v-if="bt.savedCubes.length > 0" class="space-y-3">
              <h3 class="text-sm font-bold text-slate-400 uppercase tracking-wide">Saved Cubes ({{ bt.savedCubes.length }})</h3>

              <div class="space-y-2">
                <div
                  v-for="cube in bt.savedCubes"
                  :key="cube.id"
                  class="p-3 rounded-lg bg-slate-800/30 border border-slate-700/50 hover:border-slate-700 transition-colors"
                >
                  <div class="flex items-center justify-between">
                    <div class="flex-1 min-w-0">
                      <p class="text-sm font-semibold text-white truncate">{{ cube.name }}</p>
                      <p class="text-xs text-slate-500 font-mono">{{ cube.mac_address }}</p>
                    </div>
                    <button
                      @click="bt.deleteCube(cube.id, cube.user_id)"
                      class="p-1.5 rounded hover:bg-red-500/10 text-slate-600 hover:text-red-400 transition-colors"
                      title="Forget this cube"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Transition>

      <!-- Backdrop -->
      <Transition
        enter-active-class="transition duration-300 ease-out"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition duration-200 ease-in"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div
          v-if="showCubeManager"
          @click="showCubeManager = false"
          class="fixed inset-0 bg-slate-950/80 backdrop-blur-sm z-40"
        ></div>
      </Transition>
    </Teleport>
  </div>
</template>
