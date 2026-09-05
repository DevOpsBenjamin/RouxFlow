<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { cm_get_solve_by_id_json } from '../../services/cube/bridge'
import SolveAnalysisCard from '../analysis/SolveAnalysisCard.vue'

const props = defineProps<{
    solveId: string
}>()

const router = useRouter()

const solve = computed(() => {
    try {
        const json = cm_get_solve_by_id_json(props.solveId)
        return json && json !== 'null' ? JSON.parse(json) : null
    } catch {
        return null
    }
})
</script>

<template>
  <div class="w-full max-w-3xl mx-auto p-[4vmin]">
    <!-- Back button -->
    <button
      @click="router.push({ name: 'Analysis' })"
      class="flex items-center gap-[0.8vmin] text-slate-500 hover:text-slate-300 transition-colors text-[1.5vmin] font-medium mb-[3vh]"
    >
      <svg class="w-[1.8vmin] h-[1.8vmin]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
      </svg>
      Back to Analysis
    </button>

    <!-- Solve card -->
    <SolveAnalysisCard v-if="solve" :solve="solve" />

    <!-- Not found -->
    <div v-else class="text-center py-[10vh]">
      <div class="text-[3vmin] text-slate-700 mb-[1vh]">Solve not found</div>
      <p class="text-[1.3vmin] text-slate-600">This solve may have been deleted or belongs to another session.</p>
      <button
        @click="router.push({ name: 'Analysis' })"
        class="mt-[2vh] text-[1.3vmin] text-indigo-400 hover:text-indigo-300 transition-colors"
      >
        Return to solve list
      </button>
    </div>
  </div>
</template>
