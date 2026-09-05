<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()
</script>

<template>
  <div class="w-full max-w-4xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
    <!-- Header -->
    <div class="text-center space-y-4">
      <h2 class="text-4xl md:text-5xl font-black tracking-tight text-white">
        🏆 <span class="bg-gradient-to-r from-amber-400 to-orange-500 bg-clip-text text-transparent">Leaderboard</span>
      </h2>
      <p class="text-slate-400 text-lg">
        Global rankings coming soon
      </p>
    </div>

    <!-- WIP Card -->
    <div class="bg-slate-900/50 border border-white/5 rounded-3xl p-12 text-center space-y-6">
      <div class="text-8xl opacity-50">🚧</div>
      <div class="space-y-2">
        <h3 class="text-2xl font-bold text-white">Work in Progress</h3>
        <p class="text-slate-400 max-w-md mx-auto">
          We're building a global leaderboard where you can compete with cubers worldwide.
          WCA-compliant sessions will be ranked by Ao5.
        </p>
      </div>

      <div class="pt-4">
        <button 
          @click="router.push({ name: 'Home' })"
          class="px-8 py-3 bg-indigo-600 hover:bg-indigo-500 text-white font-bold rounded-xl transition-all"
        >
          Back to Dashboard
        </button>
      </div>
    </div>
  </div>
</template>
