<script setup lang="ts">
// Sample solves will be fetched from Supabase and cached in IndexedDB.
// For now, show a static placeholder list demonstrating the UI structure.

const sampleSolves = [
  {
    id: '1',
    solver: 'Kian Mansour',
    time: '7.42',
    moves: 48,
    tps: '6.47',
    source: 'video',
    steps: { fb: 8, sb: 14, cmll: 1, lse: 12 },
  },
  {
    id: '2',
    solver: 'Kian Mansour',
    time: '8.11',
    moves: 52,
    tps: '6.41',
    source: 'video',
    steps: { fb: 9, sb: 16, cmll: 1, lse: 14 },
  },
  {
    id: '3',
    solver: 'Sean Patrick Villanueva',
    time: null,
    moves: 45,
    tps: null,
    source: 'speedsolving.com',
    steps: { fb: 7, sb: 12, cmll: 1, lse: 10 },
  },
]

const totalMoves = (s: typeof sampleSolves[number]) =>
  s.steps.fb + s.steps.sb + s.steps.cmll + s.steps.lse

function stepPercent(count: number, total: number) {
  return Math.round((count / total) * 100)
}
</script>

<template>
  <div class="max-w-3xl">
    <h2 class="text-lg font-bold text-white mb-1">Sample Solves</h2>
    <p class="text-sm text-slate-400 mb-6">
      Curated example solves from top Roux solvers. Study their step breakdowns and learn efficient solutions.
    </p>

    <!-- Solve list -->
    <div class="flex flex-col gap-3">
      <div
        v-for="solve in sampleSolves"
        :key="solve.id"
        class="rounded-xl border border-white/5 bg-slate-900/50 hover:bg-slate-900/80 transition-all cursor-pointer p-4"
      >
        <div class="flex items-center justify-between mb-3">
          <div>
            <h3 class="text-sm font-semibold text-white">{{ solve.solver }}</h3>
            <p class="text-xs text-slate-500">
              {{ solve.source === 'video' ? 'Video reconstruction' : 'speedsolving.com' }}
            </p>
          </div>
          <div class="flex items-center gap-4 text-right">
            <div v-if="solve.time">
              <p class="text-lg font-mono font-bold text-white">{{ solve.time }}s</p>
              <p class="text-[10px] text-slate-500 uppercase">Time</p>
            </div>
            <div>
              <p class="text-sm font-mono font-semibold text-slate-300">{{ solve.moves }} moves</p>
              <p v-if="solve.tps" class="text-[10px] text-slate-500">{{ solve.tps }} TPS</p>
            </div>
          </div>
        </div>

        <!-- Step breakdown bar -->
        <div class="flex h-2 rounded-full overflow-hidden bg-slate-800">
          <div
            class="bg-blue-500"
            :style="{ width: stepPercent(solve.steps.fb, totalMoves(solve)) + '%' }"
            :title="`FB: ${solve.steps.fb} moves`"
          ></div>
          <div
            class="bg-green-500"
            :style="{ width: stepPercent(solve.steps.sb, totalMoves(solve)) + '%' }"
            :title="`SB: ${solve.steps.sb} moves`"
          ></div>
          <div
            class="bg-yellow-500"
            :style="{ width: stepPercent(solve.steps.cmll, totalMoves(solve)) + '%' }"
            :title="`CMLL: ${solve.steps.cmll} moves`"
          ></div>
          <div
            class="bg-purple-500"
            :style="{ width: stepPercent(solve.steps.lse, totalMoves(solve)) + '%' }"
            :title="`LSE: ${solve.steps.lse} moves`"
          ></div>
        </div>

        <!-- Step legend -->
        <div class="flex items-center gap-4 mt-2 text-[10px] text-slate-500">
          <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-blue-500"></span> FB {{ solve.steps.fb }}</span>
          <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-green-500"></span> SB {{ solve.steps.sb }}</span>
          <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-yellow-500"></span> CMLL {{ solve.steps.cmll }}</span>
          <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-purple-500"></span> LSE {{ solve.steps.lse }}</span>
        </div>
      </div>
    </div>

    <!-- Playback placeholder -->
    <div class="mt-8 rounded-xl border border-dashed border-white/10 bg-slate-900/30 p-6 text-center">
      <p class="text-sm font-medium text-slate-400">Playback Coming Soon</p>
      <p class="text-xs text-slate-600 mt-1">
        Click a solve to watch it play back move-by-move with Cubeast-style controls.
      </p>
    </div>
  </div>
</template>
