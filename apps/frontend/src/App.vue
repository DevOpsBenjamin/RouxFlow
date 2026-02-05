<script setup lang="ts">
import LandingView from './components/layout/LandingView.vue'
import HomeView from './components/layout/HomeView.vue'
import SessionView from './components/layout/SessionView.vue'
import SolveAnalysis from './components/layout/SolveAnalysis.vue'
import ProfileView from './components/layout/ProfileView.vue'
import LeaderboardView from './components/layout/LeaderboardView.vue'
import SupportedCubesView from './components/layout/SupportedCubesView.vue'
import DeviceSelectionModal from './components/cube/DeviceSelectionModal.vue'
import CubeManagerView from './components/layout/CubeManagerView.vue'
import Navbar from './components/layout/Navbar.vue'
import MoyuDebugger from './components/cube/MoyuDebugger.vue'
import { useUIStore } from './stores/ui'
import { useAuthStore } from './stores/auth'

const ui = useUIStore()
const auth = useAuthStore()

// Auto-navigate to home after login or deep link success
import { watch } from 'vue'
watch(() => auth.isAuthenticated, (isLogged) => {
  if (isLogged && ui.currentState === 'Landing') {
    ui.setHome()
  }
})
</script>

<template>
  <div class="h-[100dvh] bg-slate-950 text-slate-50 flex flex-col font-sans selection:bg-indigo-500/30 overflow-hidden">
    <DeviceSelectionModal />
    
    <!-- Show Navbar if NOT on Landing -->
    <Navbar v-if="ui.currentState !== 'Landing'">
      <template #actions>
        <!-- Custom actions can go here if needed -->
      </template>
    </Navbar>

    <main class="flex-1 flex flex-col items-center justify-start overflow-y-auto overflow-x-hidden p-[2vmin]">
      <Transition 
        enter-active-class="transition duration-500 ease-out"
        enter-from-class="transform translate-y-4 opacity-0"
        enter-to-class="transform translate-y-0 opacity-100"
        leave-active-class="transition duration-300 ease-in"
        leave-from-class="transform translate-y-0 opacity-100"
        leave-to-class="transform translate-y-4 opacity-0"
        mode="out-in"
      >
        <MoyuDebugger v-if="ui.currentState === 'MoyuDebug'" />
        <LandingView v-else-if="ui.currentState === 'Landing'" />
        <HomeView v-else-if="ui.currentState === 'Home'" />
        <ProfileView v-else-if="ui.currentState === 'Profile'" />
        <LeaderboardView v-else-if="ui.currentState === 'Leaderboard'" />
        <SupportedCubesView v-else-if="ui.currentState === 'SupportedCubes'" />
        <CubeManagerView v-else-if="ui.currentState === 'CubeManager'" />
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
