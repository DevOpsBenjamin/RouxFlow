<script setup lang="ts">
import DeviceSelectionModal from './components/cube/DeviceSelectionModal.vue'
import MacAddressInputModal from './components/cube/MacAddressInputModal.vue'
import Navbar from './components/layout/Navbar.vue'
import OfflineIndicator from './components/ui/OfflineIndicator.vue'
import { useAuthStore } from './stores/auth'
import { useBluetoothStore } from './stores/bluetooth'
import { watch } from 'vue'
import { useRouter } from 'vue-router'

const auth = useAuthStore()
const bt = useBluetoothStore()
const router = useRouter()

// Auto-navigate to home after login (immediate: catches OAuth redirect where user is already authed at mount)
watch(() => auth.isAuthenticated, (isLogged) => {
  if (isLogged && router.currentRoute.value.name === 'Landing') {
    router.push({ name: 'Home' })
  }
}, { immediate: true })

// Redirect to BluetoothRequired when cube disconnects on a requiresCube page
watch(() => bt.isConnected, (connected) => {
  if (!connected && router.currentRoute.value.meta.requiresCube) {
    router.push({ name: 'BluetoothRequired', query: { from: router.currentRoute.value.name as string } })
  }
})
</script>

<template>
  <div class="h-[100dvh] bg-slate-950 text-slate-50 flex flex-col font-sans selection:bg-indigo-500/30 overflow-hidden">
    <DeviceSelectionModal />
    <MacAddressInputModal
      :show="bt.showMacInput"
      :deviceName="bt.pendingConnection?.device?.name || 'Unknown Cube'"
      @submit="bt.submitMacAddress"
      @cancel="bt.cancelMacInput"
    />
    <OfflineIndicator />

    <!-- Show Navbar if NOT on Landing, Welcome, or Terms -->
    <Navbar v-if="!['Landing', 'Welcome', 'Terms'].includes($route.name as string)">
      <template #actions>
        <!-- Custom actions can go here if needed -->
      </template>
    </Navbar>

    <main class="flex-1 flex flex-col items-center justify-start overflow-y-auto overflow-x-hidden p-[2vmin]">
      <router-view v-slot="{ Component }">
        <Transition
          enter-active-class="transition duration-500 ease-out"
          enter-from-class="transform translate-y-4 opacity-0"
          enter-to-class="transform translate-y-0 opacity-100"
          leave-active-class="transition duration-300 ease-in"
          leave-from-class="transform translate-y-0 opacity-100"
          leave-to-class="transform translate-y-4 opacity-0"
          mode="out-in"
        >
          <component :is="Component" />
        </Transition>
      </router-view>
    </main>
  </div>
</template>

<style>
body {
  background-color: #020617;
  margin: 0;
}
</style>
