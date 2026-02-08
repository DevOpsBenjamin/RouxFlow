<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../../stores/auth'
import { useBluetoothStore } from '../../stores/bluetooth'

const router = useRouter()
const auth = useAuthStore()
const bt = useBluetoothStore()

// Data ownership modal
const showDataNotice = ref(false)

const needsDataNotice = computed(() => {
  if (auth.isAuthenticated) {
    return !auth.user?.user_metadata?.data_consent_accepted
  }
  return !localStorage.getItem('rouxflow_data_notice_seen')
})

onMounted(() => {
  if (needsDataNotice.value) {
    showDataNotice.value = true
  }
})

async function acknowledgeDataNotice() {
  if (auth.isAuthenticated) {
    try {
      await auth.acceptDataConsent()
    } catch {
      // If offline, fall back to localStorage
      localStorage.setItem('rouxflow_data_notice_seen', 'true')
    }
  } else {
    localStorage.setItem('rouxflow_data_notice_seen', 'true')
  }
  showDataNotice.value = false
}

function startSolving() {
  router.push({ name: 'Session' })
}

// Mock data for placeholder stats
const mockStats = {
  best: '14.23',
  ao5: '16.87',
  ao12: '17.42',
  total: '127'
}

const mockSolves = [
  { time: '15.32', scramble: "R U R' U' R' F R2 U' R' U'", ao5: '16.87', date: 'Today' },
  { time: '14.23', scramble: "F R U' R' U R U R' F'", ao5: '17.01', date: 'Today' },
  { time: '16.91', scramble: "R' U' F R U R' U' F'", ao5: '17.55', date: 'Yesterday' },
  { time: '18.04', scramble: "U R U' R' U' F R F'", ao5: '18.12', date: 'Yesterday' },
  { time: '17.56', scramble: "R U2 R' U' R U' R'", ao5: '18.30', date: 'Yesterday' },
]
</script>

<template>
  <div class="w-full max-w-4xl mx-auto space-y-6 animate-in fade-in slide-in-from-bottom-8 duration-700">
    <!-- Welcome Header -->
    <div class="text-center space-y-2">
      <h2 class="text-4xl md:text-5xl font-black tracking-tight text-white">
        <template v-if="auth.isGuest">
          Welcome to <span class="bg-gradient-to-r from-indigo-400 to-cyan-400 bg-clip-text text-transparent">RouxFlow</span>
        </template>
        <template v-else>
          Welcome back, <span class="bg-gradient-to-r from-indigo-400 to-cyan-400 bg-clip-text text-transparent">{{ auth.displayName }}</span>!
        </template>
      </h2>
      <p class="text-slate-400 text-lg max-w-2xl mx-auto">
        <template v-if="bt.isConnected">
          Your cube is ready. Let's solve.
        </template>
        <template v-else>
          Connect your Bluetooth cube to begin training.
        </template>
      </p>
    </div>

    <!-- Connection Error Banner -->
    <div v-if="bt.error" class="bg-red-500/10 border border-red-500/20 rounded-2xl px-5 py-3 flex items-center gap-3">
      <svg class="w-5 h-5 text-red-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
      </svg>
      <p class="text-red-300 text-sm">{{ bt.error }}</p>
    </div>

    <!-- Cube Connection Area -->
    <!-- State 1: No cube connected, no saved cubes -->
    <div v-if="!bt.isConnected && bt.savedCubes.length === 0" class="bg-gradient-to-br from-indigo-600/20 via-slate-900/60 to-slate-900/60 border border-indigo-500/20 rounded-3xl p-8 text-center space-y-5">
      <div class="w-16 h-16 rounded-2xl bg-indigo-500/20 flex items-center justify-center text-3xl mx-auto">
        <svg class="w-8 h-8 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
      </div>
      <div class="space-y-2">
        <h3 class="text-2xl font-bold text-white">Connect Your Cube</h3>
        <p class="text-slate-400 text-sm max-w-md mx-auto">
          Pair your Bluetooth smart cube to unlock real-time solve tracking, phase detection, and training tools.
        </p>
      </div>
      <div class="flex flex-col sm:flex-row items-center justify-center gap-3">
        <button
          @click="bt.startScan()"
          :disabled="bt.isConnecting"
          class="px-8 py-3.5 bg-gradient-to-r from-indigo-600 to-indigo-500 text-white font-bold rounded-xl hover:from-indigo-500 hover:to-indigo-400 transition-all hover:scale-[1.03] active:scale-95 disabled:opacity-50 shadow-lg shadow-indigo-500/25"
        >
          {{ bt.isConnecting ? 'Scanning...' : 'Scan for Cubes' }}
        </button>
        <button
          @click="router.push({ name: 'SupportedCubes' })"
          class="text-indigo-400 hover:text-indigo-300 text-sm font-medium transition-colors"
        >
          View Supported Cubes
        </button>
      </div>
    </div>

    <!-- State 2: No cube connected, has saved cubes -->
    <div v-else-if="!bt.isConnected && bt.savedCubes.length > 0" class="bg-slate-900/50 border border-white/5 rounded-3xl p-6 space-y-4">
      <div class="flex items-center gap-3">
        <span class="w-2.5 h-2.5 rounded-full bg-red-500"></span>
        <h3 class="text-lg font-bold text-white">No Cube Connected</h3>
      </div>
      <div class="space-y-2">
        <p class="text-slate-400 text-sm">Reconnect a saved cube or scan for a new one:</p>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="cube in bt.savedCubes"
            :key="cube.id"
            @click="bt.reconnectCube(cube)"
            :disabled="bt.isConnecting"
            class="flex items-center gap-2 px-4 py-2.5 bg-slate-800 border border-slate-700 rounded-xl text-sm font-medium text-slate-200 hover:border-indigo-500/50 hover:bg-slate-800/80 transition-all disabled:opacity-50"
          >
            <svg class="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            {{ cube.name }}
          </button>
          <button
            @click="bt.startScan()"
            :disabled="bt.isConnecting"
            class="flex items-center gap-2 px-4 py-2.5 border border-dashed border-slate-600 rounded-xl text-sm text-slate-400 hover:border-slate-500 hover:text-slate-300 transition-all disabled:opacity-50"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Scan New Cube
          </button>
        </div>
      </div>
    </div>

    <!-- State 3: Cube connected -->
    <div v-else-if="bt.isConnected" class="space-y-4">
      <!-- Connection status bar -->
      <div class="bg-emerald-500/10 border border-emerald-500/20 rounded-2xl px-5 py-3 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <span class="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse"></span>
          <div class="flex items-center gap-2 text-sm">
            <span class="text-emerald-300 font-semibold">{{ bt.connectedDeviceName || 'Cube' }}</span>
            <span v-if="bt.deviceInfo?.protocol" class="text-slate-500">{{ bt.deviceInfo.protocol }}</span>
            <span v-if="bt.deviceInfo?.battery != null" class="text-slate-400 flex items-center gap-1">
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7h14a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V9a2 2 0 012-2zm18 4h-2" />
              </svg>
              {{ bt.deviceInfo.battery }}%
            </span>
          </div>
        </div>
        <button
          @click="router.push({ name: 'CubeManager' })"
          class="text-slate-400 hover:text-white text-xs font-medium transition-colors"
        >
          Manage Cube
        </button>
      </div>

      <!-- Start Solving CTA -->
      <button
        @click="startSolving"
        class="group relative w-full overflow-hidden bg-gradient-to-r from-indigo-600 to-cyan-600 rounded-3xl p-8 text-center hover:from-indigo-500 hover:to-cyan-500 transition-all hover:shadow-2xl hover:shadow-indigo-500/20 active:scale-[0.99]"
      >
        <div class="relative space-y-2">
          <h3 class="text-3xl font-black text-white">Start Solving</h3>
          <p class="text-indigo-100/80 text-sm">Open the timer and begin your Roux training session</p>
        </div>
        <svg class="absolute right-6 top-1/2 -translate-y-1/2 w-8 h-8 text-white/30 group-hover:text-white/60 group-hover:translate-x-1 transition-all" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 8l4 4m0 0l-4 4m4-4H3" />
        </svg>
      </button>
    </div>

    <!-- Quick Stats Row (mock data) -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-3 opacity-70">
      <div class="bg-slate-900/50 border border-white/5 rounded-2xl p-4 text-center relative">
        <span class="absolute top-2 right-2 text-[9px] text-slate-600 font-bold uppercase tracking-wider">Demo</span>
        <p class="text-xs text-slate-500 uppercase tracking-wider font-bold mb-1">Best</p>
        <p class="text-2xl font-black text-white">{{ mockStats.best }}</p>
      </div>
      <div class="bg-slate-900/50 border border-white/5 rounded-2xl p-4 text-center relative">
        <span class="absolute top-2 right-2 text-[9px] text-slate-600 font-bold uppercase tracking-wider">Demo</span>
        <p class="text-xs text-slate-500 uppercase tracking-wider font-bold mb-1">Ao5</p>
        <p class="text-2xl font-black text-white">{{ mockStats.ao5 }}</p>
      </div>
      <div class="bg-slate-900/50 border border-white/5 rounded-2xl p-4 text-center relative">
        <span class="absolute top-2 right-2 text-[9px] text-slate-600 font-bold uppercase tracking-wider">Demo</span>
        <p class="text-xs text-slate-500 uppercase tracking-wider font-bold mb-1">Ao12</p>
        <p class="text-2xl font-black text-white">{{ mockStats.ao12 }}</p>
      </div>
      <div class="bg-slate-900/50 border border-white/5 rounded-2xl p-4 text-center relative">
        <span class="absolute top-2 right-2 text-[9px] text-slate-600 font-bold uppercase tracking-wider">Demo</span>
        <p class="text-xs text-slate-500 uppercase tracking-wider font-bold mb-1">Total</p>
        <p class="text-2xl font-black text-white">{{ mockStats.total }}</p>
      </div>
    </div>

    <!-- Recent Solves (mock data) -->
    <div class="bg-slate-900/50 border border-white/5 rounded-2xl overflow-hidden opacity-70 relative">
      <div class="px-5 py-3 border-b border-white/5 flex items-center justify-between">
        <h3 class="text-sm font-bold text-white">Recent Solves</h3>
        <span class="text-[9px] text-slate-600 font-bold uppercase tracking-wider">Placeholder</span>
      </div>
      <div class="divide-y divide-white/5">
        <div v-for="(solve, i) in mockSolves" :key="i" class="px-5 py-3 flex items-center gap-4">
          <span class="text-lg font-bold text-white w-16">{{ solve.time }}</span>
          <span class="text-xs text-slate-500 font-mono truncate flex-1">{{ solve.scramble }}</span>
          <span class="text-xs text-slate-400 hidden sm:inline">Ao5: {{ solve.ao5 }}</span>
          <span class="text-xs text-slate-600 w-16 text-right">{{ solve.date }}</span>
        </div>
      </div>
    </div>

    <!-- Data Ownership Modal -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition duration-300"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition duration-200"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div v-if="showDataNotice" class="fixed inset-0 z-50 flex items-center justify-center p-4">
          <!-- Backdrop -->
          <div class="absolute inset-0 bg-black/70 backdrop-blur-sm" />

          <!-- Modal -->
          <div class="relative bg-slate-900 border border-white/10 rounded-2xl max-w-lg w-full p-6 md:p-8 space-y-5 shadow-2xl">
            <div class="space-y-1">
              <h3 class="text-xl font-bold text-white">Before You Continue</h3>
              <p class="text-xs text-slate-500">A quick note about your data</p>
            </div>

            <div class="text-sm text-slate-300 leading-relaxed space-y-3">
              <p>
                We know Terms of Use are often skipped, so here's what matters:
              </p>
              <p class="bg-indigo-500/10 border border-indigo-500/20 rounded-xl p-4 text-indigo-200">
                <strong class="text-white">Your solves recorded on RouxFlow become the property of RouxFlow</strong>
                and are used to improve our algorithms and training tools.
              </p>
              <p>
                If you request account deletion under GDPR, we will remove all data linked to your account.
                However, anonymized solve data will be retained for training purposes, as it can no longer
                be linked to your identity.
              </p>
              <p class="text-slate-400">
                By continuing to use this app, you agree to these terms. If you do not agree, please stop using the app.
              </p>
            </div>

            <button
              @click="acknowledgeDataNotice"
              class="w-full py-3 bg-indigo-600 text-white font-bold rounded-xl hover:bg-indigo-500 transition-colors active:scale-[0.98]"
            >
              I understand
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>
