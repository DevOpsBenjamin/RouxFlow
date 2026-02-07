<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useAuthStore } from '../../stores/auth'

const router = useRouter()
const auth = useAuthStore()

function startTraining() {
  router.push({ name: 'Session' })
}
</script>

<template>
  <div class="w-full max-w-4xl mx-auto space-y-6 animate-in fade-in slide-in-from-bottom-8 duration-700">
    <!-- Welcome Header -->
    <div class="text-center space-y-2">
      <h2 class="text-4xl md:text-5xl font-black tracking-tight text-white">
        Welcome back, <span class="bg-gradient-to-r from-indigo-400 to-cyan-400 bg-clip-text text-transparent">{{ auth.displayName }}</span>!
      </h2>
      <p class="text-slate-400 text-lg max-w-2xl mx-auto">
        Ready to master the Roux method? Connect your Bluetooth cube and let's get started.
      </p>
    </div>

    <!-- Main Actions Grid -->
    <div class="grid md:grid-cols-2 gap-6">
      <!-- Training Card -->
      <button 
        @click="startTraining"
        class="group relative overflow-hidden bg-slate-900/50 border border-white/5 rounded-3xl p-8 text-left hover:border-indigo-500/50 transition-all hover:shadow-2xl hover:shadow-indigo-500/10 active:scale-[0.98]"
      >
        <div class="absolute inset-0 bg-gradient-to-br from-indigo-600/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
        <div class="relative space-y-4">
          <div class="w-14 h-14 rounded-2xl bg-indigo-500/20 flex items-center justify-center text-3xl group-hover:scale-110 transition-transform">
            ⏱️
          </div>
          <div>
            <h3 class="text-2xl font-bold text-white mb-2">Start Training</h3>
            <p class="text-slate-400 leading-relaxed text-sm">
              Practice Roux phases (FB, SB, CMLL, LSE) with real-time analysis and automatic scramble validation.
            </p>
          </div>
          <div class="flex items-center gap-2 text-indigo-400 font-semibold text-sm pt-2">
            Go to Timer
            <svg class="w-4 h-4 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 8l4 4m0 0l-4 4m4-4H3" />
            </svg>
          </div>
        </div>
      </button>

      <!-- Stats Card (Coming Soon) -->
      <div class="group bg-slate-900/30 border border-white/5 rounded-3xl p-8 text-left opacity-60">
        <div class="space-y-4">
          <div class="w-14 h-14 rounded-2xl bg-slate-800 flex items-center justify-center text-3xl">
            📊
          </div>
          <div>
            <h3 class="text-2xl font-bold text-slate-300 mb-2">Detailed Stats</h3>
            <p class="text-slate-500 leading-relaxed text-sm">
              Track your progress over time, analyze your splits, and identify your weak points.
            </p>
          </div>
          <div class="text-slate-600 font-semibold text-xs uppercase tracking-widest pt-2">
            Coming Soon
          </div>
        </div>
      </div>
    </div>

    <!-- Requirements/Hardware Info -->
    <div class="bg-indigo-500/5 border border-indigo-500/10 rounded-2xl p-5">
      <div class="flex flex-col md:flex-row gap-4 items-start">
        <div class="w-10 h-10 rounded-xl bg-indigo-500/10 flex-shrink-0 flex items-center justify-center text-xl">
          🔌
        </div>
        <div class="space-y-2 flex-1">
          <h4 class="text-base font-bold text-white">Bluetooth Cube Required</h4>
          <p class="text-slate-400 text-sm leading-relaxed">
            Connect your smart cube via the <span class="text-indigo-400 font-semibold">Cubes</span> menu in the header to enable auto-timer and phase detection.
          </p>
          <button 
            @click="ui.setSupportedCubes()"
            class="text-indigo-400 hover:text-indigo-300 font-semibold text-xs underline underline-offset-2 transition-colors"
          >
            View Supported Cubes →
          </button>
        </div>
      </div>
    </div>
    <p class="text-center text-slate-600 text-xs italic pb-4">
      Tip: Use the Roux method for maximum efficiency. Focus on your First Block transition!
    </p>
  </div>
</template>
