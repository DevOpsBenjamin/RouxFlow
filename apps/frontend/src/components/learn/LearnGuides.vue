<script setup lang="ts">
const guides = [
  {
    name: 'CFOP to Roux Transition',
    description: 'Map your CFOP knowledge to Roux: Cross to FB, F2L to SB, OLL to CMLL, PLL to LSE.',
    icon: '\u{1F504}',
  },
  {
    name: 'M-slice Finger Tricks',
    description: 'Push vs pull technique, home grip, and regrip minimization for fast M-slice execution.',
    icon: '\u{1F44B}',
  },
  {
    name: 'Inspection Strategy',
    description: 'What to look for in 15 seconds: FB planning, SB prediction, and advanced pre-planning.',
    icon: '\u{1F50D}',
  },
  {
    name: 'Common Mistakes',
    description: 'Top 10 habits that slow down Roux solvers and how to fix them.',
    icon: '\u{26A0}\u{FE0F}',
  },
]
</script>

<template>
  <div class="max-w-2xl">
    <h2 class="text-lg font-bold text-white mb-1">Guides</h2>
    <p class="text-sm text-slate-400 mb-6">
      Written guides with 3D cube demonstrations to deepen your Roux understanding.
    </p>

    <div class="flex flex-col gap-3">
      <div
        v-for="guide in guides"
        :key="guide.name"
        class="rounded-xl border border-white/5 bg-slate-900/30 p-4 flex items-start gap-4 opacity-60"
      >
        <div class="text-2xl mt-0.5">{{ guide.icon }}</div>
        <div class="flex-1">
          <div class="flex items-center gap-2">
            <h3 class="text-sm font-semibold text-white">{{ guide.name }}</h3>
            <span class="text-[9px] font-semibold px-1.5 py-0.5 rounded bg-white/5 text-slate-500">
              Coming Soon
            </span>
          </div>
          <p class="text-xs text-slate-500 mt-1">{{ guide.description }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
