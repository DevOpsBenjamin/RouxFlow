<script setup lang="ts">
import { computed } from 'vue'
import { useSessionStore } from '../../stores/session'
import SolveAnalysisCard from '../analysis/SolveAnalysisCard.vue'

const sessionStore = useSessionStore()
const solve = computed(() => sessionStore.selectedSolve)
</script>

<template>
  <SolveAnalysisCard v-if="solve" :solve="solve" showClose @close="sessionStore.selectSolve(null)" />
</template>
