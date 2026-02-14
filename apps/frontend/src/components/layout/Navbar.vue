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
    </nav>

    <div class="flex items-center gap-4">
      <slot name="actions"></slot>
      
      <!-- Bluetooth Cube Connection -->
      <ConnectCubeButton />

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
