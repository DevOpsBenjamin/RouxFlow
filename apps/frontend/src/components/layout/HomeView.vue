<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../../stores/auth'
import { useBluetoothStore } from '../../stores/bluetooth'
import { useSessionStore } from '../../stores/session'

const router = useRouter()
const auth = useAuthStore()
const bt = useBluetoothStore()
const session = useSessionStore()

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
      localStorage.setItem('rouxflow_data_notice_seen', 'true')
    }
  } else {
    localStorage.setItem('rouxflow_data_notice_seen', 'true')
  }
  showDataNotice.value = false
}

// Real stats from WASM via session store
const stats = computed(() => session.sessionStats)
const recentSolves = computed(() => session.solveList?.slice(0, 5) ?? [])
const hasData = computed(() => stats.value && stats.value.solve_count > 0)

function formatTime(ms: number | null | undefined): string {
  if (ms == null) return '--'
  const seconds = ms / 1000
  return seconds.toFixed(2)
}

// Feature cards
const features = [
  {
    name: 'Timer',
    description: 'Start a Roux solving session',
    route: 'Session',
    icon: 'timer',
    gradient: 'from-indigo-600 to-cyan-600',
    hoverGradient: 'hover:from-indigo-500 hover:to-cyan-500',
    requiresCube: true,
  },
  {
    name: 'Learn',
    description: 'Tutorials, sample solves & drills',
    route: 'LearnTutorial',
    icon: 'learn',
    gradient: 'from-purple-600 to-pink-600',
    hoverGradient: 'hover:from-purple-500 hover:to-pink-500',
    requiresCube: true,
  },
  {
    name: 'Stats',
    description: 'Detailed analytics & trends',
    route: 'Stats',
    icon: 'stats',
    gradient: 'from-emerald-600 to-teal-600',
    hoverGradient: 'hover:from-emerald-500 hover:to-teal-500',
    requiresCube: false,
  },
  {
    name: 'Leaderboard',
    description: 'See top solvers',
    route: 'Leaderboard',
    icon: 'leaderboard',
    gradient: 'from-amber-600 to-orange-600',
    hoverGradient: 'hover:from-amber-500 hover:to-orange-500',
    requiresCube: false,
  },
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
    <div v-else-if="bt.isConnected" class="bg-emerald-500/10 border border-emerald-500/20 rounded-2xl px-5 py-3 flex items-center justify-between">
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

    <!-- Quick Stats Bar (real data) -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
      <div class="bg-slate-900/50 border border-white/5 rounded-2xl p-4 text-center">
        <p class="text-[10px] text-slate-500 uppercase tracking-wider font-bold mb-1">Best</p>
        <p class="text-2xl font-black text-white font-mono">{{ formatTime(stats?.best_ms) }}</p>
      </div>
      <div class="bg-slate-900/50 border border-white/5 rounded-2xl p-4 text-center">
        <p class="text-[10px] text-slate-500 uppercase tracking-wider font-bold mb-1">Best Ao5</p>
        <p class="text-2xl font-black text-white font-mono">{{ formatTime(stats?.best_ao5_ms) }}</p>
      </div>
      <div class="bg-slate-900/50 border border-white/5 rounded-2xl p-4 text-center">
        <p class="text-[10px] text-slate-500 uppercase tracking-wider font-bold mb-1">Best Ao12</p>
        <p class="text-2xl font-black text-white font-mono">{{ formatTime(stats?.best_ao12_ms) }}</p>
      </div>
      <div class="bg-slate-900/50 border border-white/5 rounded-2xl p-4 text-center">
        <p class="text-[10px] text-slate-500 uppercase tracking-wider font-bold mb-1">Solves</p>
        <p class="text-2xl font-black text-white font-mono">{{ stats?.solve_count ?? 0 }}</p>
      </div>
    </div>

    <!-- Feature Cards Grid -->
    <div class="grid grid-cols-2 gap-3">
      <button
        v-for="feature in features"
        :key="feature.name"
        @click="router.push({ name: feature.route })"
        class="group relative overflow-hidden rounded-2xl p-5 text-left transition-all hover:scale-[1.02] active:scale-[0.98] bg-gradient-to-br shadow-lg"
        :class="[feature.gradient, feature.hoverGradient, `shadow-${feature.gradient.split('-')[1]}-500/10`]"
      >
        <div class="relative">
          <!-- Icon -->
          <div class="w-10 h-10 rounded-xl bg-white/10 flex items-center justify-center mb-3">
            <svg v-if="feature.icon === 'timer'" class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <svg v-else-if="feature.icon === 'learn'" class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
            </svg>
            <svg v-else-if="feature.icon === 'stats'" class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
            </svg>
            <svg v-else-if="feature.icon === 'leaderboard'" class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
            </svg>
          </div>
          <h3 class="text-lg font-bold text-white">{{ feature.name }}</h3>
          <p class="text-sm text-white/60 mt-0.5">{{ feature.description }}</p>
        </div>
        <svg class="absolute right-4 bottom-4 w-5 h-5 text-white/20 group-hover:text-white/40 group-hover:translate-x-0.5 transition-all" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>
      </button>
    </div>

    <!-- Recent Solves -->
    <div v-if="hasData" class="bg-slate-900/50 border border-white/5 rounded-2xl overflow-hidden">
      <div class="px-5 py-3 border-b border-white/5 flex items-center justify-between">
        <h3 class="text-sm font-bold text-white">Recent Solves</h3>
        <button
          @click="router.push({ name: 'Stats' })"
          class="text-xs text-indigo-400 hover:text-indigo-300 font-medium transition-colors"
        >
          View All
        </button>
      </div>
      <div class="divide-y divide-white/5">
        <div
          v-for="solve in recentSolves"
          :key="solve.id"
          class="px-5 py-3 flex items-center gap-4"
        >
          <span class="text-lg font-bold font-mono w-20" :class="solve.is_best ? 'text-emerald-400' : 'text-white'">
            {{ solve.penalty === 'DNF' ? 'DNF' : formatTime(solve.time_ms) }}
          </span>
          <span v-if="solve.penalty === '+2'" class="text-[10px] text-amber-400 font-bold">+2</span>
          <span class="text-xs text-slate-500 font-mono">{{ solve.turns }} moves</span>
          <span class="text-xs text-slate-600 ml-auto">{{ solve.tps.toFixed(1) }} TPS</span>
        </div>
      </div>
    </div>

    <!-- Empty state when no solves yet -->
    <div v-else class="bg-slate-900/30 border border-dashed border-white/5 rounded-2xl p-8 text-center">
      <p class="text-sm text-slate-500">No solves yet. Connect your cube and start a session!</p>
    </div>

    <!-- Community & Feedback Banner -->
    <div class="bg-gradient-to-r from-indigo-500/10 via-slate-900/40 to-cyan-500/10 border border-white/5 rounded-2xl p-4 flex flex-col sm:flex-row items-center justify-between gap-3 text-center sm:text-left">
      <div class="space-y-0.5">
        <p class="text-sm font-semibold text-white">Using RouxFlow? We'd love your feedback!</p>
        <p class="text-xs text-slate-400">Found a bug or want support for a new cube? Open an issue on GitHub.</p>
      </div>
      <div class="flex items-center gap-2">
        <a
          href="https://github.com/DevOpsBenjamin/RouxFlow/issues"
          target="_blank"
          rel="noopener noreferrer"
          class="px-3.5 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium rounded-lg transition-colors flex items-center gap-1.5 shrink-0"
        >
          💬 Feedback & Bug Report
        </a>
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
