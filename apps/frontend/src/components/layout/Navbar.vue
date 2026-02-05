<script setup lang="ts">
import { onMounted } from 'vue'
import { useAuthStore } from '../../stores/auth'
import { useUIStore } from '../../stores/ui'
import { useBluetoothStore } from '../../stores/bluetooth'
import { isTauri } from '../../services/cube/bridge'

const auth = useAuthStore()
const ui = useUIStore()
const bt = useBluetoothStore()

async function handleSignOut() {
  await auth.signOut()
  ui.setLanding()
}

onMounted(() => {
  bt.loadSavedCubes(auth.user?.id)
})
</script>

<template>
  <header class="p-6 flex justify-between items-center border-b border-white/5 backdrop-blur-md sticky top-0 z-50 bg-slate-950/80">
    <div class="flex items-center gap-3 cursor-pointer group" @click="ui.setHome()">
      <div class="w-10 h-10 rounded-xl bg-indigo-500 flex items-center justify-center text-xl shadow-lg shadow-indigo-500/20 group-hover:scale-110 transition-transform">
        🧊
      </div>
      <h1 class="text-2xl font-black tracking-tight bg-gradient-to-r from-white to-slate-400 bg-clip-text text-transparent italic">
        ROUXFLOW
      </h1>
    </div>

    <nav class="hidden md:flex items-center gap-8 text-sm font-semibold text-slate-400">
      <button 
        @click="ui.setHome()" 
        class="hover:text-white transition-colors"
        :class="{ 'text-indigo-400': ui.currentState === 'Home' }"
      >
        Dashboard
      </button>
      <button 
        @click="ui.setActiveSession()" 
        class="hover:text-white transition-colors"
        :class="{ 'text-indigo-400': ui.currentState === 'ActiveSession' }"
      >
        Timer
      </button>
      <button 
        @click="ui.setLeaderboard()" 
        class="hover:text-white transition-colors"
        :class="{ 'text-indigo-400': ui.currentState === 'Leaderboard' }"
      >
        Leaderboard
      </button>
      <button 
        @click="ui.setCubeManager()" 
        class="hover:text-white transition-colors"
        :class="{ 'text-indigo-400': ui.currentState === 'CubeManager' }"
      >
        Cube Manager
      </button>
      <button 
        @click="ui.setMoyuDebug()" 
        class="hover:text-red-400 transition-colors font-mono"
        :class="{ 'text-red-400': ui.currentState === 'MoyuDebug' }"
      >
        MoYu Debug
      </button>
    </nav>

    <div class="flex items-center gap-4">
      <slot name="actions"></slot>
      
      <!-- Bluetooth Manager Dropdown -->
      <div class="relative group/bt">
        <button 
          class="flex items-center gap-2 px-4 py-2 rounded-xl bg-indigo-500/10 hover:bg-indigo-500/20 border border-indigo-500/20 transition-all text-sm font-bold text-indigo-400"
        >
          <span class="text-lg">🔌</span>
          <span class="hidden lg:inline">Cubes</span>
          <span v-if="bt.savedCubes.length > 0" class="flex h-2 w-2 rounded-full bg-emerald-500"></span>
        </button>

        <!-- Dropdown Content -->
        <div class="absolute right-0 mt-2 w-72 origin-top-right rounded-2xl bg-slate-900 border border-white/10 shadow-2xl opacity-0 invisible group-hover/bt:opacity-100 group-hover/bt:visible transition-all p-4 space-y-4 z-[100]">
          <div class="flex items-center justify-between border-b border-white/5 pb-3">
             <div class="flex flex-col">
               <span class="text-xs font-black uppercase tracking-wider text-slate-500">Your Cubes</span>
               <button 
                 v-if="isTauri && auth.isAuthenticated" 
                 @click="bt.sync(auth.user!.id)"
                 class="text-[9px] text-indigo-400 hover:text-indigo-300 transition-colors flex items-center gap-1"
               >
                 <span>☁️</span> Sync to Cloud
               </button>
               
               <!-- Manage Connected Cube Link -->
               <button 
                  @click="ui.setCubeManager()"
                  class="mt-2 text-[10px] text-emerald-400 hover:text-emerald-300 font-bold flex items-center gap-1 transition-colors"
               >
                 <span>⚙️</span> Manage {{ bt.connectedDeviceName || 'Debug Cube' }}
               </button>
             </div>
             <button @click="bt.startScan()" class="text-[10px] bg-indigo-500/10 hover:bg-indigo-500/20 text-indigo-400 px-2 py-1 rounded-md font-bold transition-colors">+ New</button>
          </div>

          <div class="space-y-2 max-h-60 overflow-y-auto pr-1 custom-scrollbar">
            <div v-if="bt.savedCubes.length === 0" class="py-6 text-center text-slate-600 text-[10px] italic">
              No saved cubes yet.
            </div>
            <div 
              v-for="cube in bt.savedCubes" 
              :key="cube.id"
              class="flex items-center justify-between p-3 rounded-xl bg-white/5 border border-transparent hover:border-indigo-500/30 transition-all group/item"
            >
              <div class="flex items-center gap-3 overflow-hidden">
                <div class="w-8 h-8 rounded-lg bg-indigo-500/10 flex items-center justify-center text-sm">
                   {{ cube.device_type === 'moyu_ai' ? '🤖' : '🧊' }}
                </div>
                <div class="overflow-hidden">
                  <div class="text-[11px] font-bold text-slate-200 truncate">{{ cube.name }}</div>
                  <div class="text-[9px] text-slate-500 font-mono truncate">{{ cube.mac_address }}</div>
                </div>
              </div>
              <button 
                @click="bt.deleteCube(cube.id, auth.user?.id)"
                class="p-1.5 rounded-md text-slate-600 hover:text-red-400 hover:bg-red-400/10 transition-all opacity-0 group-hover/item:opacity-100"
              >
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- User Profile Dropdown / Button -->
      <div class="flex items-center gap-2 pl-4 border-l border-white/5">
        <button 
          @click="ui.openProfile()"
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
