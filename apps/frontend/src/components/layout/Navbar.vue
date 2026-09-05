<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../../stores/auth'
import { useBluetoothStore } from '../../stores/bluetooth'
import ConnectCubeButton from '../cube/ConnectCubeButton.vue'

const router = useRouter()
const auth = useAuthStore()
const bt = useBluetoothStore()

async function handleSignOut() {
  await auth.signOut()
  router.push({ name: 'Landing' })
}

onMounted(() => {
  bt.loadSavedCubes(auth.user?.id ?? null)
})
</script>

<template>
  <header class="p-6 flex justify-between items-center border-b border-white/5 backdrop-blur-md sticky top-0 z-50 bg-slate-950/80">
    <div class="flex items-center gap-3 cursor-pointer group" @click="router.push({ name: 'Home' })">
      <div class="w-10 h-10 rounded-xl bg-indigo-500 flex items-center justify-center text-xl shadow-lg shadow-indigo-500/20 group-hover:scale-110 transition-transform">
        🧊
      </div>
      <h1 class="text-2xl font-black tracking-tight bg-gradient-to-r from-white to-slate-400 bg-clip-text text-transparent italic">
        ROUXFLOW
      </h1>
    </div>

    <nav class="hidden md:flex items-center gap-8 text-sm font-semibold text-slate-400">
      <button
        @click="router.push({ name: 'Home' })"
        class="hover:text-white transition-colors"
        :class="{ 'text-indigo-400': $route.name === 'Home' }"
      >
        Dashboard
      </button>
      <button
        @click="router.push({ name: 'Session' })"
        class="hover:text-white transition-colors"
        :class="{ 'text-indigo-400': $route.name === 'Session' }"
      >
        Timer
      </button>
      <button
        @click="router.push({ name: 'LearnTutorial' })"
        class="hover:text-white transition-colors"
        :class="{ 'text-indigo-400': $route.path.startsWith('/learn') }"
      >
        Learn
      </button>
      <button
        @click="router.push({ name: 'Analysis' })"
        class="hover:text-white transition-colors"
        :class="{ 'text-indigo-400': $route.name === 'Analysis' || $route.name === 'AnalysisDetail' }"
      >
        Analysis
      </button>
      <button
        @click="router.push({ name: 'Stats' })"
        class="hover:text-white transition-colors"
        :class="{ 'text-indigo-400': $route.name === 'Stats' }"
      >
        Stats
      </button>
      <button
        @click="router.push({ name: 'Leaderboard' })"
        class="hover:text-white transition-colors"
        :class="{ 'text-indigo-400': $route.name === 'Leaderboard' }"
      >
        Leaderboard
      </button>
      <button
        @click="router.push({ name: 'CubeManager' })"
        class="hover:text-white transition-colors"
        :class="{ 'text-indigo-400': $route.name === 'CubeManager' }"
      >
        Cube Manager
      </button>
      <button
        @click="router.push({ name: 'GyroDebug' })"
        class="hover:text-white transition-colors"
        :class="{ 'text-indigo-400': $route.name === 'GyroDebug' }"
      >
        Gyro Debug
      </button>
    </nav>

    <div class="flex items-center gap-4">
      <slot name="actions"></slot>
      
      <!-- Bluetooth Cube Connection -->
      <ConnectCubeButton />

      <!-- GitHub Link -->
      <a
        href="https://github.com/DevOpsBenjamin/RouxFlow"
        target="_blank"
        rel="noopener noreferrer"
        title="GitHub Repository & Feedback"
        class="p-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-slate-400 hover:text-white border border-white/5 transition-all flex items-center gap-2 text-xs font-medium"
      >
        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
          <path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/>
        </svg>
        <span class="hidden xl:inline">GitHub</span>
      </a>

      <!-- User Profile Dropdown / Button -->
      <div class="flex items-center gap-2 pl-4 border-l border-white/5">
        <button
          @click="router.push({ name: 'Profile' })"
          class="flex items-center gap-3 px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/5 transition-all text-sm font-medium"
        >
          <div v-if="auth.user?.user_metadata?.avatar_url" class="w-6 h-6 rounded-full overflow-hidden border border-white/10">
            <img :src="auth.user.user_metadata.avatar_url" alt="User" class="w-full h-full object-cover" />
          </div>
          <div v-else class="w-6 h-6 rounded-full bg-gradient-to-br from-indigo-500 to-purple-500 flex items-center justify-center text-[10px] text-white">
            {{ auth.displayName.charAt(0).toUpperCase() }}
          </div>
          <span class="text-slate-200 hidden sm:inline">{{ auth.displayName }}</span>
        </button>

        <button 
          @click="handleSignOut"
          :title="auth.isAuthenticated ? 'Sign Out' : 'Exit Guest Mode'"
          class="p-2.5 rounded-xl hover:bg-red-500/10 text-slate-500 hover:text-red-400 transition-colors"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
          </svg>
        </button>
      </div>
    </div>
  </header>
</template>
