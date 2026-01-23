<script setup lang="ts">
import LandingView from './components/layout/LandingView.vue'
import SessionView from './components/layout/SessionView.vue'
import SolveAnalysis from './components/layout/SolveAnalysis.vue'
import DeviceSelectionModal from './components/cube/DeviceSelectionModal.vue'
import Navbar from './components/layout/Navbar.vue'
import { useUIStore } from './stores/ui'

const ui = useUIStore()
</script>

<template>
  <div class="min-h-screen bg-slate-950 text-slate-50 flex flex-col font-sans selection:bg-indigo-500/30">
    <DeviceSelectionModal />
    <!-- Only show Navbar if on ActiveSession -->
    <Navbar v-if="ui.currentState === 'ActiveSession'">
      <template #actions>
        <button 
          @click="ui.setLanding()" 
          class="text-sm text-slate-400 hover:text-white transition-colors"
        >
          Exit Session
        </button>
      </template>
    </Navbar>

    <main class="flex-1 flex flex-col items-center justify-center overflow-x-hidden p-6">
      <Transition 
        enter-active-class="transition duration-500 ease-out"
        enter-from-class="transform translate-y-4 opacity-0"
        enter-to-class="transform translate-y-0 opacity-100"
        leave-active-class="transition duration-300 ease-in"
        leave-from-class="transform translate-y-0 opacity-100"
        leave-to-class="transform translate-y-4 opacity-0"
        mode="out-in"
      >
        <LandingView v-if="ui.currentState === 'Landing'" />
        <SolveAnalysis v-else-if="ui.currentState === 'Analysis'" :solve-id="ui.selectedSolveId || undefined" />
        <SessionView v-else />
      </Transition>
    </main>
  </div>
</template>

<style>
body {
  background-color: #020617;
  margin: 0;
}
</style>
