<script setup lang="ts">
const drillModules = [
  {
    name: 'Block Building Trainer',
    description: 'Practice FB and SB with targeted scrambles. Track move count vs optimal, time, and efficiency.',
    icon: '\u{1F9F1}',
  },
  {
    name: 'CMLL Algorithm Trainer',
    description: '42 CMLL cases across 8 sets (O, H, Pi, U, T, S, AS, L). Learn, recognize, and execute.',
    icon: '\u{1F9E0}',
  },
  {
    name: 'LSE Trainer',
    description: 'Edge Orientation (EO), ULUR, L4E, and advanced EOLR practice.',
    icon: '\u{1F4A1}',
  },
  {
    name: 'Ergonomics Lab',
    description: 'Gyro-based M-slice speed benchmarks, regrip detection, and cube stability scoring.',
    icon: '\u{1F3CB}\u{FE0F}',
  },
]
</script>

<template>
  <div class="max-w-2xl">
    <h2 class="text-lg font-bold text-white mb-1">Drills</h2>
    <p class="text-sm text-slate-400 mb-6">
      Targeted practice modules for each phase of the Roux method.
    </p>

    <div class="flex flex-col gap-3">
      <div
        v-for="drill in drillModules"
        :key="drill.name"
        class="rounded-xl border border-white/5 bg-slate-900/30 p-4 flex items-start gap-4 opacity-60"
      >
        <div class="text-2xl mt-0.5">{{ drill.icon }}</div>
        <div class="flex-1">
          <div class="flex items-center gap-2">
            <h3 class="text-sm font-semibold text-white">{{ drill.name }}</h3>
            <span class="text-[9px] font-semibold px-1.5 py-0.5 rounded bg-white/5 text-slate-500">
              Coming Soon
            </span>
          </div>
          <p class="text-xs text-slate-500 mt-1">{{ drill.description }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
