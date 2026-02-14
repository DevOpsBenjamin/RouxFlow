<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()

const sections = [
  { id: 'tutorial', label: 'Roux Tutorial', icon: '\u{1F9E9}', route: 'LearnTutorial' },
  { id: 'sample-solves', label: 'Sample Solves', icon: '\u{1F3AC}', route: 'LearnSampleSolves' },
  { id: 'drills', label: 'Drills', icon: '\u{1F3AF}', route: 'LearnDrills', comingSoon: true },
  { id: 'guides', label: 'Guides', icon: '\u{1F4D6}', route: 'LearnGuides', comingSoon: true },
]

const activeSection = computed(() => {
  const name = route.name as string
  return sections.find(s => s.route === name)?.id ?? 'tutorial'
})

function navigate(section: typeof sections[number]) {
  router.push({ name: section.route })
}
</script>

<template>
  <div class="w-full h-full flex animate-in fade-in duration-300">
    <!-- Sidebar -->
    <nav class="hidden md:flex flex-col w-56 shrink-0 border-r border-white/5 bg-slate-950/50 p-4 gap-1">
      <h2 class="text-xs font-bold text-slate-500 uppercase tracking-wider px-3 mb-2">Learn</h2>
      <button
        v-for="section in sections"
        :key="section.id"
        @click="navigate(section)"
        class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-all text-left"
        :class="activeSection === section.id
          ? 'bg-indigo-500/10 text-indigo-300 border border-indigo-500/20'
          : 'text-slate-400 hover:text-white hover:bg-white/5 border border-transparent'"
      >
        <span class="text-base">{{ section.icon }}</span>
        <span>{{ section.label }}</span>
        <span
          v-if="section.comingSoon"
          class="ml-auto text-[9px] font-semibold px-1.5 py-0.5 rounded bg-white/5 text-slate-500"
        >
          Soon
        </span>
      </button>
    </nav>

    <!-- Mobile nav (horizontal pills) -->
    <div class="md:hidden fixed bottom-0 left-0 right-0 z-40 bg-slate-950/95 backdrop-blur-md border-t border-white/5 px-2 py-2 flex gap-1 overflow-x-auto">
      <button
        v-for="section in sections"
        :key="section.id"
        @click="navigate(section)"
        class="flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs font-medium whitespace-nowrap transition-all shrink-0"
        :class="activeSection === section.id
          ? 'bg-indigo-500/10 text-indigo-300 border border-indigo-500/20'
          : 'text-slate-400 hover:text-white border border-transparent'"
      >
        <span>{{ section.icon }}</span>
        <span>{{ section.label }}</span>
      </button>
    </div>

    <!-- Content area -->
    <main class="flex-1 overflow-y-auto p-4 md:p-6 pb-20 md:pb-6">
      <router-view />
    </main>
  </div>
</template>
