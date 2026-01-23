<script setup lang="ts">
import Navbar from './components/layout/Navbar.vue'
import TimerDisplay from './components/timer/TimerDisplay.vue'
import SplitStats from './components/session/SplitStats.vue'
import MoveList from './components/session/MoveList.vue'
import SyncStatus from './components/cube/SyncStatus.vue'
import BluetoothManager from './components/cube/BluetoothManager.vue'
import SessionPicker from './components/session/SessionPicker.vue'
import ScrambleDisplay from './components/session/ScrambleDisplay.vue'
import { useTimerStore } from './stores/timer'

useTimerStore() // Initialize store
</script>

<template>
  <div class="min-h-screen bg-slate-950 text-slate-50 flex flex-col font-sans selection:bg-indigo-500/30">
    <Navbar>
      <template #actions>
        <SessionPicker />
        <BluetoothManager />
      </template>
    </Navbar>

    <main class="flex-1 flex flex-col items-center justify-center p-6 gap-8 text-center">
      <TimerDisplay />
      <SyncStatus />
      <MoveList />
      <SplitStats :phases="['FB', 'SB', 'CMLL', 'LSE']" />
    </main>

    <ScrambleDisplay 
      scramble="U' R2 U B2 D2 F2 L2 D' R2 F2 U L' F D2 B' D2 R' U B' L F'" 
    />
  </div>
</template>

<style>
body {
  background-color: #020617;
  margin: 0;
}
</style>
