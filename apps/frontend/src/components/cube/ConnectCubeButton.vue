<script setup lang="ts">
import { ref } from 'vue'
import { useBluetoothStore } from '../../stores/bluetooth'

const bt = useBluetoothStore()
const showCubeManager = ref(false)
const showDropdown = ref(false)

function handleConnect() {
  if (bt.isConnected) {
    showCubeManager.value = true
  } else if (bt.savedCubes.length > 0) {
    showDropdown.value = !showDropdown.value
  } else {
    bt.startScan()
  }
}

function closeDropdown() {
  showDropdown.value = false
}

async function handleReconnect(cube: typeof bt.savedCubes[number]) {
  showDropdown.value = false
  await bt.reconnectCube(cube)
}

async function handleNewScan() {
  showDropdown.value = false
  await bt.startScan()
}

async function handleDeleteCube(id: string, userId: string | null) {
  await bt.deleteCube(id, userId)
}

defineExpose({ showCubeManager })
</script>

<template>
  <div class="relative">
    <!-- Not Connected, No Saved Cubes: Big "Connect a Cube" Button -->
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

    <!-- Not Connected, Has Saved Cubes: Dropdown Button -->
    <div v-else-if="!bt.isConnected && bt.savedCubes.length > 0">
      <button
        @click="handleConnect"
        :disabled="bt.isConnecting"
        class="group px-5 py-2.5 rounded-xl bg-slate-800/50 hover:bg-slate-800 border border-slate-700 hover:border-indigo-500/50 text-white font-semibold text-sm transition-all flex items-center gap-3"
      >
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full bg-red-500 animate-pulse"></div>
          <span v-if="!bt.isConnecting">No Cube Connected</span>
          <span v-else>Connecting...</span>
        </div>
        <svg class="w-4 h-4 text-slate-400 group-hover:text-indigo-400 transition-all" :class="{ 'rotate-180': showDropdown }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      <!-- Dropdown Menu -->
      <Transition
        enter-active-class="transition duration-150 ease-out"
        enter-from-class="opacity-0 -translate-y-1"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition duration-100 ease-in"
        leave-from-class="opacity-100 translate-y-0"
        leave-to-class="opacity-0 -translate-y-1"
      >
        <div
          v-if="showDropdown"
          class="absolute right-0 top-full mt-2 w-72 rounded-xl bg-slate-900 border border-slate-700 shadow-2xl shadow-black/50 z-50 overflow-hidden"
        >
          <!-- Saved Cubes -->
          <div class="p-1">
            <button
              v-for="cube in bt.savedCubes"
              :key="cube.id"
              @click="handleReconnect(cube)"
              class="w-full text-left px-3 py-2.5 rounded-lg hover:bg-indigo-500/10 transition-colors group/item"
            >
              <div class="flex items-center gap-3">
                <svg class="w-4 h-4 text-indigo-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-semibold text-white truncate">{{ cube.name }}</p>
                  <p class="text-xs text-slate-500 font-mono">{{ cube.mac_address }}</p>
                </div>
              </div>
            </button>
          </div>

          <!-- Divider -->
          <div class="border-t border-slate-800"></div>

          <!-- Scan for New -->
          <div class="p-1">
            <button
              @click="handleNewScan"
              class="w-full text-left px-3 py-2.5 rounded-lg hover:bg-slate-800 transition-colors flex items-center gap-3"
            >
              <svg class="w-4 h-4 text-slate-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
              </svg>
              <span class="text-sm text-slate-300">Scan for New Cube</span>
            </button>
          </div>
        </div>
      </Transition>
    </div>

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
        <span v-if="bt.deviceInfo?.battery_level != null" class="flex items-center gap-1">
          🔋 {{ bt.deviceInfo.battery_level }}%
        </span>
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>
      </div>
    </button>

    <!-- Dropdown Backdrop (click to close) -->
    <div
      v-if="showDropdown"
      @click="closeDropdown"
      class="fixed inset-0 z-40"
    ></div>

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
          <div class="flex-1 overflow-y-auto p-6 space-y-4">
            <!-- Connected Cube Summary -->
            <div class="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div class="w-2.5 h-2.5 rounded-full bg-emerald-500 shadow-lg shadow-emerald-500/50"></div>
                  <div>
                    <p class="text-lg font-bold text-white">{{ bt.connectedDeviceName || 'Unknown' }}</p>
                    <p v-if="bt.deviceInfo?.battery_level" class="text-xs text-slate-400">Battery: {{ bt.deviceInfo.battery_level }}%</p>
                  </div>
                </div>
                <button
                  @click="async () => { await bt.disconnect(); showCubeManager = false }"
                  class="px-3 py-1.5 rounded-lg bg-red-500/10 hover:bg-red-500/20 border border-red-500/20 text-red-400 hover:text-red-300 text-xs font-semibold transition-colors"
                >
                  Disconnect
                </button>
              </div>
            </div>

            <!-- Quick Actions -->
            <button
              @click="bt.startScan()"
              class="w-full text-left p-3 rounded-xl hover:bg-slate-800/50 border border-slate-800 text-sm text-slate-300 hover:text-white transition-colors flex items-center gap-3"
            >
              <svg class="w-4 h-4 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
              </svg>
              Connect Another Cube
            </button>

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
                      @click="handleDeleteCube(cube.id, cube.user_id)"
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
