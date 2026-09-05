<script setup lang="ts">
// Stats page — placeholder structure for sub-step analytics.
// Will be populated with real data from solve_analysis IndexedDB table
// once rouxflow-ai step detection is implemented.
</script>

<template>
  <div class="w-full h-full animate-in fade-in duration-300 overflow-y-auto p-4 md:p-8">
    <div class="max-w-5xl mx-auto">
      <!-- Header -->
      <div class="mb-8">
        <h1 class="text-2xl font-bold text-white">Stats</h1>
        <p class="text-sm text-slate-400 mt-1">
          Sub-step analytics and performance tracking for your Roux solves.
        </p>
      </div>

      <!-- Overview cards -->
      <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-8">
        <div class="rounded-xl border border-white/5 bg-slate-900/50 p-4">
          <p class="text-[10px] font-semibold text-slate-500 uppercase tracking-wider">Best</p>
          <p class="text-2xl font-mono font-bold text-white mt-1">--</p>
        </div>
        <div class="rounded-xl border border-white/5 bg-slate-900/50 p-4">
          <p class="text-[10px] font-semibold text-slate-500 uppercase tracking-wider">Ao5</p>
          <p class="text-2xl font-mono font-bold text-white mt-1">--</p>
        </div>
        <div class="rounded-xl border border-white/5 bg-slate-900/50 p-4">
          <p class="text-[10px] font-semibold text-slate-500 uppercase tracking-wider">Ao12</p>
          <p class="text-2xl font-mono font-bold text-white mt-1">--</p>
        </div>
        <div class="rounded-xl border border-white/5 bg-slate-900/50 p-4">
          <p class="text-[10px] font-semibold text-slate-500 uppercase tracking-wider">Total Solves</p>
          <p class="text-2xl font-mono font-bold text-white mt-1">--</p>
        </div>
      </div>

      <!-- Sub-step split -->
      <div class="rounded-xl border border-white/5 bg-slate-900/50 p-6 mb-6">
        <h2 class="text-sm font-bold text-white mb-1">Sub-Step Split</h2>
        <p class="text-xs text-slate-500 mb-5">Average time per Roux step across analyzed solves.</p>

        <div class="flex flex-col gap-4">
          <div v-for="step in ['FB', 'SB', 'CMLL', 'LSE']" :key="step" class="flex items-center gap-3">
            <span class="text-xs font-semibold text-slate-400 w-12">{{ step }}</span>
            <div class="flex-1 h-6 rounded-lg bg-slate-800 overflow-hidden">
              <div
                class="h-full rounded-lg opacity-40"
                :class="{
                  'bg-blue-500': step === 'FB',
                  'bg-green-500': step === 'SB',
                  'bg-yellow-500': step === 'CMLL',
                  'bg-purple-500': step === 'LSE',
                }"
                style="width: 0%"
              ></div>
            </div>
            <span class="text-xs font-mono text-slate-500 w-14 text-right">-- s</span>
          </div>
        </div>
      </div>

      <!-- Charts placeholder -->
      <div class="grid md:grid-cols-2 gap-6 mb-6">
        <!-- Daily average chart -->
        <div class="rounded-xl border border-white/5 bg-slate-900/50 p-6">
          <h2 class="text-sm font-bold text-white mb-1">Daily Average</h2>
          <p class="text-xs text-slate-500 mb-4">Average solve time per day over time.</p>
          <div class="h-40 flex items-center justify-center border border-dashed border-white/5 rounded-lg">
            <p class="text-xs text-slate-600">Chart will appear after enough solves are analyzed</p>
          </div>
        </div>

        <!-- TPS per step -->
        <div class="rounded-xl border border-white/5 bg-slate-900/50 p-6">
          <h2 class="text-sm font-bold text-white mb-1">TPS per Step</h2>
          <p class="text-xs text-slate-500 mb-4">Turns per second broken down by Roux step.</p>
          <div class="h-40 flex items-center justify-center border border-dashed border-white/5 rounded-lg">
            <p class="text-xs text-slate-600">Chart will appear after enough solves are analyzed</p>
          </div>
        </div>
      </div>

      <!-- Weak step recommendation -->
      <div class="rounded-xl border border-dashed border-indigo-500/20 bg-indigo-500/5 p-6 mb-6">
        <h2 class="text-sm font-bold text-indigo-300 mb-1">Weak Step Identification</h2>
        <p class="text-xs text-slate-400">
          Once enough solves are analyzed, this section will highlight your weakest Roux step
          and link to targeted drills in the Learn section.
        </p>
      </div>

      <!-- Move efficiency + rotations -->
      <div class="grid md:grid-cols-2 gap-6">
        <div class="rounded-xl border border-white/5 bg-slate-900/50 p-6">
          <h2 class="text-sm font-bold text-white mb-1">Move Efficiency</h2>
          <p class="text-xs text-slate-500 mb-4">Your move count vs optimal per step.</p>
          <div class="h-32 flex items-center justify-center border border-dashed border-white/5 rounded-lg">
            <p class="text-xs text-slate-600">Requires rouxflow-ai step analysis</p>
          </div>
        </div>

        <div class="rounded-xl border border-white/5 bg-slate-900/50 p-6">
          <h2 class="text-sm font-bold text-white mb-1">Rotation Trends</h2>
          <p class="text-xs text-slate-500 mb-4">Are you reducing cube rotations over time?</p>
          <div class="h-32 flex items-center justify-center border border-dashed border-white/5 rounded-lg">
            <p class="text-xs text-slate-600">Requires rouxflow-ai step analysis</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
