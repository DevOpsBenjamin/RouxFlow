<script setup lang="ts">
import { ref } from 'vue'
import { CubeBridge } from '../../services/cube/bridge'

const isConnecting = ref(false)
const deviceName = ref('')

async function connect() {
  try {
    isConnecting.value = true
    const name = await CubeBridge.connect()
    deviceName.value = name
  } catch (error) {
    console.error('Bluetooth Error:', error)
  } finally {
    isConnecting.value = false
  }
}
</script>

<template>
  <button 
    @click="connect"
    :disabled="isConnecting"
    class="px-4 py-2 rounded-full border border-white/10 hover:bg-white/5 transition-all text-sm font-medium text-slate-200 disabled:opacity-50"
  >
    {{ isConnecting ? 'Connecting...' : deviceName || 'Connect Cube' }}
  </button>
</template>
