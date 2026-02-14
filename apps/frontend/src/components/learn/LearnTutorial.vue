<script setup lang="ts">
import { ref } from 'vue'

const steps = [
  {
    id: 'scramble',
    name: 'Scramble',
    description: 'Apply the scramble to your cube',
    detail: 'Follow the scramble sequence below, then confirm when ready.',
    color: 'slate',
  },
  {
    id: 'fb',
    name: 'First Block (FB)',
    description: 'Build a 1x2x3 block on the left side',
    detail: 'Solve the DL edge, FL edge, DBL corner, and DFL corner to form a complete block on the left.',
    color: 'blue',
  },
  {
    id: 'sb',
    name: 'Second Block (SB)',
    description: 'Build a 1x2x3 block on the right side',
    detail: 'Solve the DR edge, FR edge, DBR corner, and DFR corner without disturbing the first block.',
    color: 'green',
  },
  {
    id: 'cmll',
    name: 'CMLL',
    description: 'Orient and permute last-layer corners',
    detail: 'Use one algorithm to solve all 4 corners of the last layer. 42 cases grouped into 8 sets.',
    color: 'yellow',
  },
  {
    id: 'lse',
    name: 'Last Six Edges (LSE)',
    description: 'Solve the remaining 6 edges using M and U moves',
    detail: 'Three sub-steps: Edge Orientation (EO), UL/UR edges (ULUR), then the last 4 M-slice edges (L4E).',
    color: 'purple',
  },
]

const expandedStep = ref('scramble')
const moveCount = ref(0)

function toggleStep(id: string) {
  expandedStep.value = expandedStep.value === id ? '' : id
}

const stepColors: Record<string, string> = {
  slate: 'border-slate-500/20 bg-slate-500/5',
  blue: 'border-blue-500/20 bg-blue-500/5',
  green: 'border-green-500/20 bg-green-500/5',
  yellow: 'border-yellow-500/20 bg-yellow-500/5',
  purple: 'border-purple-500/20 bg-purple-500/5',
}

const stepHeaderColors: Record<string, string> = {
  slate: 'text-slate-300',
  blue: 'text-blue-300',
  green: 'text-green-300',
  yellow: 'text-yellow-300',
  purple: 'text-purple-300',
}
</script>

<template>
  <div class="flex flex-col lg:flex-row gap-6 h-full">
    <!-- Left: 3D Cube area -->
    <div class="lg:w-1/2 flex flex-col items-center justify-center">
      <div class="w-full max-w-md aspect-square rounded-2xl bg-slate-900/50 border border-white/5 flex items-center justify-center">
        <div class="text-center text-slate-500">
          <svg class="w-16 h-16 mx-auto mb-3 opacity-30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 7.5l-9-5.25L3 7.5m18 0l-9 5.25m9-5.25v9l-9 5.25M3 7.5l9 5.25M3 7.5v9l9 5.25m0-9v9" />
          </svg>
          <p class="text-sm font-medium">Live cube view</p>
          <p class="text-xs mt-1 text-slate-600">Connect a smart cube to see it here</p>
        </div>
      </div>
      <div class="mt-4 flex items-center gap-4 text-sm text-slate-400">
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full bg-slate-600"></div>
          <span>Moves: {{ moveCount }}</span>
        </div>
      </div>
    </div>

    <!-- Right: Step cards -->
    <div class="lg:w-1/2 flex flex-col gap-3">
      <h2 class="text-lg font-bold text-white mb-1">Interactive Roux Tutorial</h2>
      <p class="text-sm text-slate-400 mb-4">
        Follow along with your connected smart cube. Each step expands as you progress through the solve.
      </p>

      <div
        v-for="step in steps"
        :key="step.id"
        class="rounded-xl border transition-all cursor-pointer"
        :class="[
          stepColors[step.color],
          expandedStep === step.id ? 'ring-1 ring-white/10' : ''
        ]"
        @click="toggleStep(step.id)"
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-4 py-3">
          <div class="flex items-center gap-3">
            <div
              class="w-7 h-7 rounded-lg flex items-center justify-center text-xs font-bold"
              :class="stepHeaderColors[step.color]"
              style="background: rgba(255,255,255,0.05)"
            >
              {{ step.id === 'scramble' ? 'S' : step.name.charAt(0) }}
            </div>
            <div>
              <h3 class="text-sm font-semibold" :class="stepHeaderColors[step.color]">{{ step.name }}</h3>
              <p class="text-xs text-slate-500">{{ step.description }}</p>
            </div>
          </div>
          <svg
            class="w-4 h-4 text-slate-500 transition-transform"
            :class="{ 'rotate-180': expandedStep === step.id }"
            fill="none" stroke="currentColor" viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          </svg>
        </div>

        <!-- Expanded content -->
        <div
          v-if="expandedStep === step.id"
          class="px-4 pb-4 border-t border-white/5"
        >
          <p class="text-sm text-slate-300 mt-3">{{ step.detail }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
