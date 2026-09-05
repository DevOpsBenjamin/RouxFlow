<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

function goToApp() {
  localStorage.setItem('rouxflow_welcomed', 'true')
  router.push({ name: 'Landing' })
}
</script>

<template>
  <div class="w-full min-h-[100dvh] flex flex-col items-center justify-start bg-gradient-to-b from-slate-950 via-slate-900 to-slate-950 overflow-y-auto">
    <!-- Hero -->
    <header class="w-full text-center pt-16 pb-12 px-6">
      <h1 class="text-6xl md:text-8xl font-black tracking-tighter bg-gradient-to-br from-white via-indigo-200 to-indigo-500 bg-clip-text text-transparent italic leading-none mb-4">
        ROUXFLOW
      </h1>
      <p class="text-xl md:text-2xl text-slate-300 font-medium max-w-2xl mx-auto leading-relaxed">
        The Bluetooth smart cube training platform built exclusively for Roux method solvers.
      </p>
    </header>

    <div class="w-full max-w-3xl px-6 space-y-8 pb-16">
      <!-- What RouxFlow IS -->
      <section class="bg-slate-900/60 border border-white/5 rounded-2xl p-6 md:p-8">
        <h2 class="text-2xl font-bold text-white mb-5 flex items-center gap-3">
          <span class="w-10 h-10 rounded-xl bg-indigo-500/20 flex items-center justify-center text-lg shrink-0">+</span>
          What RouxFlow Is
        </h2>
        <ul class="space-y-4">
          <li class="flex items-start gap-3">
            <span class="w-1.5 h-1.5 rounded-full bg-indigo-400 mt-2.5 shrink-0"></span>
            <p class="text-slate-300 leading-relaxed">
              <span class="text-white font-semibold">Bluetooth smart cube timer</span> designed exclusively for the Roux method of speedcubing
            </p>
          </li>
          <li class="flex items-start gap-3">
            <span class="w-1.5 h-1.5 rounded-full bg-indigo-400 mt-2.5 shrink-0"></span>
            <p class="text-slate-300 leading-relaxed">
              <span class="text-white font-semibold">Real-time solve analysis</span> with phase-by-phase breakdown: First Block, Second Block, CMLL, and LSE
            </p>
          </li>
          <li class="flex items-start gap-3">
            <span class="w-1.5 h-1.5 rounded-full bg-indigo-400 mt-2.5 shrink-0"></span>
            <p class="text-slate-300 leading-relaxed">
              <span class="text-white font-semibold">WCA-style leaderboard</span> to compare your progress with other Roux solvers
              <span class="text-xs text-slate-500 ml-1">(coming soon)</span>
            </p>
          </li>
          <li class="flex items-start gap-3">
            <span class="w-1.5 h-1.5 rounded-full bg-indigo-400 mt-2.5 shrink-0"></span>
            <p class="text-slate-300 leading-relaxed">
              <span class="text-white font-semibold">AI-assisted training tools</span> with algorithm suggestions and solve optimization
              <span class="text-xs text-slate-500 ml-1">(coming soon)</span>
            </p>
          </li>
        </ul>
      </section>

      <!-- What RouxFlow is NOT -->
      <section class="bg-slate-900/40 border border-red-500/10 rounded-2xl p-6 md:p-8">
        <h2 class="text-2xl font-bold text-white mb-5 flex items-center gap-3">
          <span class="w-10 h-10 rounded-xl bg-red-500/15 flex items-center justify-center text-lg text-red-400 shrink-0 font-bold">&times;</span>
          What RouxFlow is Not
        </h2>
        <ul class="space-y-4">
          <li class="flex items-start gap-3">
            <span class="w-1.5 h-1.5 rounded-full bg-red-400/60 mt-2.5 shrink-0"></span>
            <p class="text-slate-400 leading-relaxed">
              <span class="text-slate-200 font-semibold">Not for CFOP solvers.</span>
              RouxFlow analyzes Roux phases only. If you use CFOP, check out <a href="https://cubeast.com" target="_blank" rel="noopener noreferrer" class="text-indigo-400 hover:underline">Cubeast</a> instead.
            </p>
          </li>
          <li class="flex items-start gap-3">
            <span class="w-1.5 h-1.5 rounded-full bg-red-400/60 mt-2.5 shrink-0"></span>
            <p class="text-slate-400 leading-relaxed">
              <span class="text-slate-200 font-semibold">Not for non-Bluetooth cubes.</span>
              RouxFlow requires a smart cube with Bluetooth connectivity. For manual timing, use <a href="https://cstimer.net" target="_blank" rel="noopener noreferrer" class="text-indigo-400 hover:underline">csTimer</a>.
            </p>
          </li>
          <li class="flex items-start gap-3">
            <span class="w-1.5 h-1.5 rounded-full bg-red-400/60 mt-2.5 shrink-0"></span>
            <p class="text-slate-400 leading-relaxed">
              <span class="text-slate-200 font-semibold">Not a general-purpose timer.</span>
              This is a focused, opinionated app for Roux practitioners. The scope is intentionally narrow.
            </p>
          </li>
        </ul>
      </section>

      <!-- CTA -->
      <div class="text-center pt-4 pb-8">
        <button
          @click="goToApp"
          class="group relative inline-flex items-center gap-3 px-10 py-4 bg-gradient-to-r from-indigo-600 to-indigo-500 text-white font-bold text-lg rounded-2xl hover:from-indigo-500 hover:to-indigo-400 transition-all hover:scale-[1.03] active:scale-95 shadow-xl shadow-indigo-500/25"
        >
          Go to App
          <svg class="w-5 h-5 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 8l4 4m0 0l-4 4m4-4H3" />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>
