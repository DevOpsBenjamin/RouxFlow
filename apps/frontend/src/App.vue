<script setup lang="ts">
import { useTimerStore } from './stores/timer'

const timer = useTimerStore()
</script>

<template>
  <div class="min-h-screen bg-slate-950 text-slate-50 flex flex-col font-sans selection:bg-indigo-500/30">
    <!-- Header -->
    <header class="p-6 flex justify-between items-center border-b border-white/5 backdrop-blur-md sticky top-0 z-50">
      <div class="flex items-center gap-3">
        <span class="text-3xl">🧊</span>
        <h1 class="text-2xl font-bold tracking-tight bg-gradient-to-r from-indigo-400 to-cyan-400 bg-clip-text text-transparent">
          RouxFlow
        </h1>
      </div>
      <nav class="hidden md:flex gap-8 text-sm font-medium text-slate-400">
        <a href="#" class="hover:text-indigo-400 transition-colors">Session</a>
        <a href="#" class="hover:text-indigo-400 transition-colors">Training</a>
        <a href="#" class="hover:text-indigo-400 transition-colors">Stats</a>
      </nav>
      <div class="flex gap-4">
        <button class="px-4 py-2 rounded-full border border-white/10 hover:bg-white/5 transition-all text-sm font-medium">
          Connect Cube
        </button>
      </div>
    </header>

    <!-- Main Content -->
    <main class="flex-1 flex flex-col items-center justify-center p-6 gap-12">
      <!-- Timer Display -->
      <div class="relative group">
        <div class="absolute -inset-1 bg-gradient-to-r from-indigo-500 to-cyan-500 rounded-full blur opacity-25 group-hover:opacity-40 transition duration-1000"></div>
        <div class="relative px-20 py-12 bg-slate-900 rounded-3xl border border-white/5 flex flex-col items-center">
          <div class="text-8xl md:text-9xl font-mono font-light tracking-tighter tabular-nums">
            {{ timer.formattedTime }}
          </div>
          <div class="mt-4 text-slate-500 uppercase tracking-widest text-xs font-bold">
            Ready to flow
          </div>
        </div>
      </div>

      <!-- Quick Stats / Phase Splits Placeholder -->
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4 w-full max-w-4xl">
        <div v-for="phase in ['FB', 'SB', 'CMLL', 'LSE']" :key="phase" 
             class="p-4 rounded-2xl bg-white/5 border border-white/5 hover:border-white/10 transition-colors">
          <div class="text-xs text-slate-500 font-bold mb-1">{{ phase }}</div>
          <div class="text-xl font-mono text-slate-300">--.--</div>
        </div>
      </div>
    </main>

    <!-- Footer / Scramble -->
    <footer class="p-8 border-t border-white/5 bg-slate-900/50">
      <div class="max-w-4xl mx-auto text-center">
        <div class="text-slate-500 text-xs mb-4 font-bold uppercase tracking-widest">Next Scramble</div>
        <div class="text-lg md:text-xl font-mono text-slate-300">
          U' R2 U B2 D2 F2 L2 D' R2 F2 U L' F D2 B' D2 R' U B' L F'
        </div>
      </div>
    </footer>
  </div>
</template>

<style>
/* Custom animations or glassmorphism effects if needed */
body {
  background-color: #020617;
}
</style>
